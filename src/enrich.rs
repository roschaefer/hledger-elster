use crate::dataset::TaxDataset;
use crate::journal::{load_transactions, JournalError, Posting, Transaction};
use crate::posting::TaxPosting;
use chrono::NaiveDate;
use regex::Regex;
use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;
use thiserror::Error;

pub const EUER_FORM: &str = "einnahmenueberschussrechnung";

const VAT_MODES: &[&str] = &[
    "contains_vat",
    "reverse_charge_eu",
    "reverse_charge_non_eu",
    "not_applicable",
];
const VAT_MODES_REQUIRING_RATE: &[&str] =
    &["contains_vat", "reverse_charge_eu", "reverse_charge_non_eu"];

/// Every `elster_*` tag this module reads. README.md's tag tables are the
/// authoritative contract for journal authors (see specs/README.md); this
/// list exists so a test can catch drift between the two instead of relying
/// on remembering to update README in the same change.
#[cfg(test)]
const KNOWN_TAGS: &[&str] = &[
    "elster_account",
    "elster_afa_years",
    "elster_calculation",
    "elster_deduction",
    "elster_expense_share",
    "elster_form",
    "elster_input_vat_share",
    "elster_item",
    "elster_period",
    "elster_role",
    "elster_section",
    "elster_vat",
    "elster_vat_rate",
];

#[derive(Debug, Error)]
pub enum EnrichError {
    #[error("{0}")]
    TagValidation(String),
    #[error("{0}")]
    VatPeriod(String),
    #[error("{0}")]
    Deduction(String),
    #[error("invalid numeric tag value: {0}")]
    InvalidNumber(String),
    #[error(transparent)]
    Journal(#[from] JournalError),
}

fn quantize(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

fn validate_posting_tags(posting: &Posting) -> Result<(), EnrichError> {
    let tags = posting.tags();
    let account = &posting.paccount;

    if !tags
        .get("elster_vat_share")
        .map(String::as_str)
        .unwrap_or("")
        .is_empty()
    {
        return Err(EnrichError::TagValidation(format!(
            "Unsupported elster_vat_share for account \"{account}\". Use elster_input_vat_share instead."
        )));
    }
    if !tags
        .get("elster_reverse_charge")
        .map(String::as_str)
        .unwrap_or("")
        .is_empty()
    {
        return Err(EnrichError::TagValidation(format!(
            "Unsupported elster_reverse_charge for account \"{account}\". \
             Use elster_vat:reverse_charge_eu or elster_vat:reverse_charge_non_eu instead."
        )));
    }
    if !tags
        .get("elster_reverse_charge_rate")
        .map(String::as_str)
        .unwrap_or("")
        .is_empty()
    {
        return Err(EnrichError::TagValidation(format!(
            "Unsupported elster_reverse_charge_rate for account \"{account}\". Use elster_vat_rate instead."
        )));
    }

    let forms = posting.tag_values("elster_form");
    if forms.len() > 1 {
        return Err(EnrichError::TagValidation(format!(
            "Conflicting elster_form tags for account \"{account}\": {}. \
             A posting can be routed to either EÜR or ESt, not both.",
            forms.join(", ")
        )));
    }

    let vat_mode = tags.get("elster_vat").map(String::as_str).unwrap_or("");
    if !vat_mode.is_empty() && !VAT_MODES.contains(&vat_mode) {
        return Err(EnrichError::TagValidation(format!(
            "Unsupported elster_vat:{vat_mode} for account \"{account}\". \
             Use \"contains_vat\", \"reverse_charge_eu\", \"reverse_charge_non_eu\", or \"not_applicable\"."
        )));
    }
    if tags.get("elster_form").map(String::as_str) == Some(EUER_FORM) && vat_mode.is_empty() {
        return Err(EnrichError::TagValidation(format!(
            "elster_form:{EUER_FORM} for account \"{account}\" requires elster_vat."
        )));
    }
    let has_vat_rate = !tags
        .get("elster_vat_rate")
        .map(String::as_str)
        .unwrap_or("")
        .is_empty();
    if VAT_MODES_REQUIRING_RATE.contains(&vat_mode) && !has_vat_rate {
        return Err(EnrichError::TagValidation(format!(
            "elster_vat:{vat_mode} for account \"{account}\" requires elster_vat_rate."
        )));
    }
    if vat_mode == "not_applicable" && has_vat_rate {
        return Err(EnrichError::TagValidation(format!(
            "elster_vat:not_applicable for account \"{account}\" cannot be combined with elster_vat_rate."
        )));
    }
    let has_input_vat_share = !tags
        .get("elster_input_vat_share")
        .map(String::as_str)
        .unwrap_or("")
        .is_empty();
    if vat_mode == "not_applicable" && has_input_vat_share {
        return Err(EnrichError::TagValidation(format!(
            "elster_vat:not_applicable for account \"{account}\" cannot be combined with elster_input_vat_share."
        )));
    }
    if vat_mode.starts_with("reverse_charge_")
        && tags.get("elster_form").map(String::as_str) != Some(EUER_FORM)
    {
        return Err(EnrichError::TagValidation(format!(
            "elster_vat:{vat_mode} for account \"{account}\" requires elster_form:{EUER_FORM}."
        )));
    }

    Ok(())
}

fn source_posting(transaction: &Transaction) -> Option<&Posting> {
    let business_sources: Vec<&Posting> = transaction
        .tpostings
        .iter()
        .filter(|p| p.tags().get("elster_account").map(String::as_str) == Some("business"))
        .collect();
    if business_sources.len() == 1 {
        return Some(business_sources[0]);
    }

    let tagged_sources: Vec<&Posting> = transaction
        .tpostings
        .iter()
        .filter(|p| {
            matches!(
                p.tags().get("elster_account").map(String::as_str),
                Some("business") | Some("private")
            )
        })
        .collect();
    if tagged_sources.len() == 1 {
        return Some(tagged_sources[0]);
    }

    None
}

fn is_tax_relevant(tags: &HashMap<String, String>) -> bool {
    !tags
        .get("elster_form")
        .map(String::as_str)
        .unwrap_or("")
        .is_empty()
        || !tags
            .get("elster_role")
            .map(String::as_str)
            .unwrap_or("")
            .is_empty()
        || !tags
            .get("elster_deduction")
            .map(String::as_str)
            .unwrap_or("")
            .is_empty()
}

fn to_decimal(value: Option<&str>, default: &str) -> Result<Decimal, EnrichError> {
    let raw = match value {
        Some(v) if !v.is_empty() => v,
        _ => default,
    };
    raw.parse::<Decimal>()
        .map_err(|_| EnrichError::InvalidNumber(raw.to_string()))
}

fn parse_tax_period(value: &str) -> Result<(String, i32), EnrichError> {
    static YEAR_ONLY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}$").unwrap());
    static YEAR_QUARTER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(\d{4})-Q([1-4])$").unwrap());
    static YEAR_MONTH: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(\d{4})-(0[1-9]|1[0-2])$").unwrap());

    if value.is_empty() {
        return Ok((String::new(), 0));
    }
    if YEAR_ONLY.is_match(value) {
        let year: i32 = value.parse().unwrap();
        return Ok((value.to_string(), year));
    }
    if let Some(caps) = YEAR_QUARTER.captures(value) {
        let year_str = &caps[1];
        let quarter = &caps[2];
        let year: i32 = year_str.parse().unwrap();
        return Ok((format!("{year_str} Q{quarter}"), year));
    }
    if let Some(caps) = YEAR_MONTH.captures(value) {
        let year: i32 = caps[1].parse().unwrap();
        return Ok((value.to_string(), year));
    }
    Err(EnrichError::VatPeriod(format!(
        "Unsupported elster_period: {value}. Use YYYY, YYYY-Qn, or YYYY-MM."
    )))
}

fn comment_has_ignore(comments: &[&str]) -> bool {
    static IGNORE_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(^|[,\s])elster_role\s*:\s*ignore($|[,\s])").unwrap());
    comments
        .iter()
        .any(|comment| IGNORE_PATTERN.is_match(comment))
}

fn fallback_tax_role(amount: Decimal, source_tags: &HashMap<String, String>) -> &'static str {
    if source_tags.get("elster_account").map(String::as_str) != Some("business") {
        return "";
    }
    if amount > Decimal::ZERO {
        "drawing"
    } else if amount < Decimal::ZERO {
        "contribution"
    } else {
        ""
    }
}

#[allow(clippy::too_many_arguments)]
fn enrich_posting(
    posting: &Posting,
    transaction_date: &str,
    transaction_comment: &str,
    description: &str,
    source_account: &str,
    source_tags: &HashMap<String, String>,
    source_file: &str,
    source_line: u32,
) -> Result<Option<TaxPosting>, EnrichError> {
    let account = posting.paccount.clone();
    if posting.pamount.is_empty() {
        return Ok(None);
    }

    // Rebuild the exact decimal from hledger's own mantissa/scale rather than its
    // lossy `floatingPoint` convenience field, then apply the same ROUND_HALF_UP
    // quantization used throughout the rest of the codebase. This is bit-identical
    // to the Python port for every real EUR amount (<=2 fractional digits), which
    // is the only case that occurs in practice; it also avoids depending on
    // Python's incidental ROUND_HALF_EVEN-via-string-formatting behavior for the
    // (unreachable, for currency journals) >2-decimal-place case.
    let quantity = &posting.pamount[0].aquantity;
    let amount = quantize(Decimal::new(
        quantity.decimal_mantissa,
        quantity.decimal_places,
    ));

    let posting_comment = posting
        .pcomment
        .clone()
        .unwrap_or_default()
        .trim()
        .to_string();
    let posting_date_raw = posting.pdate.as_deref();
    let posting_date =
        NaiveDate::parse_from_str(posting_date_raw.unwrap_or(transaction_date), "%Y-%m-%d")
            .map_err(|e| EnrichError::VatPeriod(format!("invalid date: {e}")))?;

    let mut tags = posting.tags();
    validate_posting_tags(posting)?;
    let source_label = source_tags.get("elster_item").cloned().unwrap_or_default();
    let source_is_business =
        source_tags.get("elster_account").map(String::as_str) == Some("business");

    if comment_has_ignore(&[transaction_comment, &posting_comment]) {
        if account == source_account {
            return Ok(None);
        }
        return Ok(Some(TaxPosting {
            posting_date,
            source_account: source_account.to_string(),
            counter_account: account,
            amount,
            description: description.to_string(),
            transaction_comment: transaction_comment.to_string(),
            posting_comment,
            source_file: source_file.to_string(),
            source_line,
            tax_form: String::new(),
            tax_deduction: String::new(),
            tax_role: "ignore".to_string(),
            calculation: String::new(),
            vat_mode: String::new(),
            vat_rate: Decimal::ZERO,
            expense_share: Decimal::ONE,
            input_vat_share: Decimal::ZERO,
            afa_years: 0,
            derived_kind: String::new(),
            label: String::new(),
            source_label,
            section: String::new(),
            tax_period: String::new(),
            tax_period_year: 0,
            source_is_business,
        }));
    }

    if !is_tax_relevant(&tags) {
        if account == source_account {
            return Ok(None);
        }
        let fallback_role = fallback_tax_role(amount, source_tags);
        if fallback_role.is_empty() {
            return Ok(None);
        }
        tags = HashMap::from([("elster_role".to_string(), fallback_role.to_string())]);
    }

    let mut tax_deduction = tags.get("elster_deduction").cloned().unwrap_or_default();
    let tax_form = tags.get("elster_form").cloned().unwrap_or_default();
    let tax_role = tags.get("elster_role").cloned().unwrap_or_default();
    let calculation = tags.get("elster_calculation").cloned().unwrap_or_default();
    let vat_mode = tags.get("elster_vat").cloned().unwrap_or_default();
    let vat_rate = to_decimal(tags.get("elster_vat_rate").map(String::as_str), "0")?;
    let mut expense_share = to_decimal(tags.get("elster_expense_share").map(String::as_str), "1")?;
    let mut input_vat_share = to_decimal(
        tags.get("elster_input_vat_share").map(String::as_str),
        &expense_share.to_string(),
    )?;
    let afa_years_raw = tags
        .get("elster_afa_years")
        .map(String::as_str)
        .unwrap_or("");
    let mut afa_years: i32 = if afa_years_raw.is_empty() {
        0
    } else {
        afa_years_raw
            .parse()
            .map_err(|_| EnrichError::InvalidNumber(afa_years_raw.to_string()))?
    };
    let label = tags.get("elster_item").cloned().unwrap_or_default();
    let section = tags.get("elster_section").cloned().unwrap_or_default();
    let tax_period_raw = tags.get("elster_period").cloned().unwrap_or_default();
    let (tax_period, tax_period_year) = parse_tax_period(&tax_period_raw)?;

    // GWG: elster_afa_years always overrides inherited elster_deduction.
    if afa_years > 0 {
        let gross = amount.abs();
        let net_cost = if vat_mode == "contains_vat" && vat_rate > Decimal::ZERO {
            gross / (Decimal::ONE + vat_rate)
        } else {
            gross
        };
        if net_cost > Decimal::from(800) {
            tax_deduction = "afa".to_string();
        } else {
            tax_deduction = "full".to_string();
            afa_years = 0;
        }
    }

    if tax_deduction == "nicht_abzugsfaehig" {
        return Err(EnrichError::Deduction(
            "Unsupported elster_deduction:nicht_abzugsfaehig. Use \"non_deductible\" instead."
                .to_string(),
        ));
    }

    if tax_deduction == "non_deductible" {
        expense_share = Decimal::ZERO;
        input_vat_share = Decimal::ZERO;
    }

    Ok(Some(TaxPosting {
        posting_date,
        source_account: source_account.to_string(),
        counter_account: account,
        amount,
        description: description.to_string(),
        transaction_comment: transaction_comment.to_string(),
        posting_comment,
        source_file: source_file.to_string(),
        source_line,
        tax_form,
        tax_deduction,
        tax_role,
        calculation,
        vat_mode,
        vat_rate,
        expense_share,
        input_vat_share,
        afa_years,
        derived_kind: String::new(),
        label,
        source_label,
        section,
        tax_period,
        tax_period_year,
        source_is_business,
    }))
}

pub fn build_dataset(journal_path: &Path) -> Result<TaxDataset, EnrichError> {
    let transactions = load_transactions(journal_path)?;
    let mut all_postings: Vec<TaxPosting> = Vec::new();

    for transaction in &transactions {
        let source = source_posting(transaction);
        let source_account = source.map(|p| p.paccount.as_str()).unwrap_or("");
        let source_tags = source.map(|p| p.tags()).unwrap_or_default();
        let (source_file, source_line) = transaction
            .tsourcepos
            .first()
            .map(|pos| (pos.source_name.as_str(), pos.source_line))
            .unwrap_or(("", 0));
        let transaction_comment = transaction.tcomment.trim();

        for posting in &transaction.tpostings {
            if let Some(tp) = enrich_posting(
                posting,
                &transaction.tdate,
                transaction_comment,
                &transaction.tdescription,
                source_account,
                &source_tags,
                source_file,
                source_line,
            )? {
                all_postings.push(tp);
            }
        }
    }

    Ok(TaxDataset::new(all_postings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn journal_with(body: &str) -> NamedTempFile {
        let mut file = NamedTempFile::with_suffix(".journal").unwrap();
        write!(file, "{body}").unwrap();
        file
    }

    fn build(body: &str) -> TaxDataset {
        let file = journal_with(body);
        build_dataset(file.path()).unwrap()
    }

    fn try_build(body: &str) -> Result<TaxDataset, EnrichError> {
        let file = journal_with(body);
        build_dataset(file.path())
    }

    #[test]
    fn readme_tag_tables_match_known_tags() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let readme = std::fs::read_to_string(format!("{manifest_dir}/README.md")).unwrap();
        let tag_re = Regex::new(r"elster_[a-z_]+").unwrap();
        let documented: std::collections::BTreeSet<&str> =
            tag_re.find_iter(&readme).map(|m| m.as_str()).collect();
        let known: std::collections::BTreeSet<&str> = KNOWN_TAGS.iter().copied().collect();
        assert_eq!(
            documented, known,
            "README.md's elster_* tag tables have drifted from enrich.rs::KNOWN_TAGS"
        );
    }

    #[test]
    fn rejects_deprecated_elster_vat_share() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_vat_share:1.0\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("Use elster_input_vat_share instead"));
    }

    #[test]
    fn rejects_deprecated_elster_reverse_charge() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_reverse_charge:1\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("reverse_charge_eu or elster_vat:reverse_charge_non_eu"));
    }

    #[test]
    fn rejects_conflicting_elster_form_tags() {
        // hledger keeps duplicate tags with the same key, most-specific first.
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_form:einnahmenueberschussrechnung, elster_form:einkommensteuer\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("Conflicting elster_form tags"));
    }

    #[test]
    fn euer_form_requires_elster_vat() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_form:einnahmenueberschussrechnung\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("requires elster_vat"));
    }

    #[test]
    fn vat_mode_requiring_rate_without_rate_errors() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("requires elster_vat_rate"));
    }

    #[test]
    fn not_applicable_cannot_combine_with_vat_rate() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_form:einnahmenueberschussrechnung, elster_vat:not_applicable, elster_vat_rate:0.19\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("cannot be combined with elster_vat_rate"));
    }

    #[test]
    fn reverse_charge_requires_euer_form() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_form:einkommensteuer, elster_vat:reverse_charge_eu, elster_vat_rate:0.19\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("requires elster_form:einnahmenueberschussrechnung"));
    }

    #[test]
    fn rejects_deprecated_nicht_abzugsfaehig_deduction() {
        let err = try_build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_deduction:nicht_abzugsfaehig\n    assets:bank  -10.00 EUR\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("Use \"non_deductible\" instead"));
    }

    #[test]
    fn non_deductible_zeroes_expense_and_input_vat_share() {
        let ds = build(
            "2024-01-01 Test\n    expenses:foo  10.00 EUR  ; elster_deduction:non_deductible, elster_expense_share:0.5\n    assets:bank  -10.00 EUR\n",
        );
        let p = ds
            .iter()
            .find(|p| p.counter_account == "expenses:foo")
            .unwrap();
        assert_eq!(p.expense_share, Decimal::ZERO);
        assert_eq!(p.input_vat_share, Decimal::ZERO);
    }

    #[test]
    fn gwg_below_threshold_becomes_full_deduction_with_no_afa_years() {
        // net cost 700.00 / 1.19 = 588.24, below the 800 EUR GWG threshold -> full.
        let ds = build(
            "2024-01-01 Test\n    expenses:foo  700.00 EUR  ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_afa_years:3\n    assets:bank  -700.00 EUR\n",
        );
        let p = ds
            .iter()
            .find(|p| p.counter_account == "expenses:foo")
            .unwrap();
        assert_eq!(p.tax_deduction, "full");
        assert_eq!(p.afa_years, 0);
    }

    #[test]
    fn gwg_above_threshold_keeps_afa_deduction() {
        // net cost 1000.00 / 1.19 = 840.34, above the 800 EUR GWG threshold -> afa.
        let ds = build(
            "2024-01-01 Test\n    expenses:foo  1000.00 EUR  ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_afa_years:3\n    assets:bank  -1000.00 EUR\n",
        );
        let p = ds
            .iter()
            .find(|p| p.counter_account == "expenses:foo")
            .unwrap();
        assert_eq!(p.tax_deduction, "afa");
        assert_eq!(p.afa_years, 3);
    }

    #[test]
    fn ignore_comment_produces_ignore_role_and_is_excluded_from_source_posting() {
        let ds = build(
            "account assets:bank  ; elster_account:business\n\n2024-01-01 Test  ; elster_role:ignore\n    transfers:internal  50.00 EUR\n    assets:bank  -50.00 EUR\n",
        );
        assert_eq!(ds.len(), 1);
        let p = ds.iter().next().unwrap();
        assert_eq!(p.tax_role, "ignore");
        assert_eq!(p.counter_account, "transfers:internal");
    }

    #[test]
    fn business_source_expense_amount_falls_back_to_drawing() {
        let ds = build(
            "account assets:bank  ; elster_account:business\n\n2024-01-01 Test\n    expenses:personal:misc  25.00 EUR\n    assets:bank  -25.00 EUR\n",
        );
        let p = ds
            .iter()
            .find(|p| p.counter_account == "expenses:personal:misc")
            .unwrap();
        assert_eq!(p.tax_role, "drawing");
        assert!(p.source_is_business);
    }

    #[test]
    fn business_source_income_amount_falls_back_to_contribution() {
        let ds = build(
            "account assets:bank  ; elster_account:business\n\n2024-01-01 Test\n    liabilities:owner  -40.00 EUR\n    assets:bank  40.00 EUR\n",
        );
        let p = ds
            .iter()
            .find(|p| p.counter_account == "liabilities:owner")
            .unwrap();
        assert_eq!(p.tax_role, "contribution");
    }

    #[test]
    fn parse_tax_period_accepts_year_quarter_and_month() {
        assert_eq!(
            parse_tax_period("2024").unwrap(),
            ("2024".to_string(), 2024)
        );
        assert_eq!(
            parse_tax_period("2024-Q2").unwrap(),
            ("2024 Q2".to_string(), 2024)
        );
        assert_eq!(
            parse_tax_period("2024-05").unwrap(),
            ("2024-05".to_string(), 2024)
        );
        assert_eq!(parse_tax_period("").unwrap(), (String::new(), 0));
    }

    #[test]
    fn parse_tax_period_rejects_unsupported_format() {
        let err = parse_tax_period("2024/05").unwrap_err();
        assert!(err.to_string().contains("Unsupported elster_period"));
    }

    #[test]
    fn builds_dataset_from_example_ledger_without_error() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let journal = manifest_dir.join("examples/ledger/hledger.journal");
        let ds = build_dataset(&journal).unwrap();
        assert!(!ds.is_empty());
        assert_eq!(ds.for_year(2024).len() + ds.for_year(2025).len(), ds.len());
    }

    #[test]
    fn comment_has_ignore_matches_various_delimiters() {
        assert!(comment_has_ignore(&["elster_role:ignore"]));
        assert!(comment_has_ignore(&["foo, elster_role: ignore, bar"]));
        assert!(!comment_has_ignore(&["elster_role:ignored"]));
        assert!(!comment_has_ignore(&["nothing here"]));
    }
}
