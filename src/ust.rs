use crate::aggregates;
use crate::classification::{euer_expenses, euer_income};
use crate::dataset::TaxDataset;
use crate::periods::{blank_row, filter_period, fmt, section_row, vat_period_labels, ReportRow};
use indexmap::IndexMap;
use rust_decimal::{Decimal, RoundingStrategy};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct VatAdvanceError(String);

impl VatAdvanceError {
    pub fn new(message: String) -> Self {
        Self(message)
    }
}

fn quantize(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

fn vat_advance_payments(dataset: &TaxDataset) -> Result<TaxDataset, VatAdvanceError> {
    let advance_ds = dataset.for_role("vat_advance");
    let invalid: Vec<_> = advance_ds
        .iter()
        .filter(|p| p.amount != Decimal::ZERO && p.tax_period.is_empty())
        .collect();
    if !invalid.is_empty() {
        let examples = invalid
            .iter()
            .take(3)
            .map(|p| {
                format!(
                    "{} {} ({} -> {})",
                    p.posting_date, p.description, p.source_account, p.counter_account
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(VatAdvanceError(format!(
            "vat_advance postings require elster_period. Missing elster_period for {} posting(s): {examples}",
            invalid.len()
        )));
    }
    Ok(advance_ds
        .iter()
        .filter(|p| p.amount != Decimal::ZERO)
        .cloned()
        .collect())
}

fn vat_advance_period(dataset: &TaxDataset, year: i32, label: &str) -> Option<TaxDataset> {
    if label == year.to_string() {
        return Some(
            dataset
                .iter()
                .filter(|p| p.tax_period_year == year)
                .cloned()
                .collect(),
        );
    }
    let period_postings: Vec<_> = dataset
        .iter()
        .filter(|p| p.tax_period == label)
        .cloned()
        .collect();
    if period_postings.is_empty() {
        None
    } else {
        Some(TaxDataset::new(period_postings))
    }
}

const EINNAHME: &str = "Einnahme (Netto)";
const VEREINNAHMTE_UST: &str = "Vereinnahmte Umsatzsteuer";
const RC_EU_BASE: &str = "§13b EU Leistung (Netto)";
const RC_EU_VAT: &str = "§13b EU Umsatzsteuer";
const RC_NON_EU_BASE: &str = "§13b Non-EU Leistung (Netto)";
const RC_NON_EU_VAT: &str = "§13b Non-EU Umsatzsteuer";
const VORSTEUER: &str = "Abziehbare Vorsteuerbeträge";
const VORAUSZAHLUNGSSOLL: &str = "Vorauszahlungssoll";
const BEREITS_ENTRICHTET: &str = "Bereits Entrichtet";

/// One row per Kennzahl and one column per period, like the EÜR and ESt.
pub fn ust_rows(dataset: &TaxDataset, year: i32) -> Result<Vec<ReportRow>, VatAdvanceError> {
    let income_ds = euer_income(dataset);
    let euer_ds = euer_expenses(dataset);
    let vat_advance = vat_advance_payments(dataset)?;
    let has_reverse_charge = euer_ds
        .for_year(year)
        .iter()
        .any(|p| p.vat_mode.starts_with("reverse_charge_"));

    let labels = vat_period_labels(year);
    let columns: Vec<IndexMap<&str, String>> = labels
        .iter()
        .map(|lbl| {
            let income_p = filter_period(&income_ds, year, lbl);
            let euer_p = filter_period(&euer_ds, year, lbl);
            let collected = aggregates::collected_vat(&income_p);
            let reverse_charge_eu_vat = aggregates::reverse_charge_vat(&euer_p, "eu");
            let reverse_charge_non_eu_vat = aggregates::reverse_charge_vat(&euer_p, "non_eu");
            let vorsteuer =
                aggregates::deductible_vat(&euer_p) + aggregates::reverse_charge_input_vat(&euer_p);
            let soll =
                quantize(collected + reverse_charge_eu_vat + reverse_charge_non_eu_vat - vorsteuer);
            let bereits_entrichtet = vat_advance_period(&vat_advance, year, lbl)
                .as_ref()
                .map(aggregates::signed_total)
                .map(fmt)
                .unwrap_or_default();
            IndexMap::from([
                (EINNAHME, fmt(aggregates::net_amount(&income_p))),
                (VEREINNAHMTE_UST, fmt(collected)),
                (
                    RC_EU_BASE,
                    fmt(aggregates::reverse_charge_base(&euer_p, "eu")),
                ),
                (RC_EU_VAT, fmt(reverse_charge_eu_vat)),
                (
                    RC_NON_EU_BASE,
                    fmt(aggregates::reverse_charge_base(&euer_p, "non_eu")),
                ),
                (RC_NON_EU_VAT, fmt(reverse_charge_non_eu_vat)),
                (VORSTEUER, fmt(vorsteuer)),
                (VORAUSZAHLUNGSSOLL, fmt(soll)),
                (BEREITS_ENTRICHTET, bereits_entrichtet),
            ])
        })
        .collect();

    let item_row = |kennzahl: &str| -> ReportRow {
        let mut row = ReportRow::new();
        row.insert("Kennzahl".to_string(), kennzahl.to_string());
        for (lbl, column) in labels.iter().zip(&columns) {
            row.insert(lbl.clone(), column[kennzahl].clone());
        }
        row
    };

    let mut rows = vec![
        section_row("Lieferungen und sonstige Leistungen", &labels),
        item_row(EINNAHME),
        item_row(VEREINNAHMTE_UST),
    ];
    if has_reverse_charge {
        rows.push(blank_row(&labels));
        rows.push(section_row(
            "Leistungsempfänger als Steuerschuldner (§13b UStG)",
            &labels,
        ));
        for kennzahl in [RC_EU_BASE, RC_EU_VAT, RC_NON_EU_BASE, RC_NON_EU_VAT] {
            rows.push(item_row(kennzahl));
        }
    }
    rows.push(blank_row(&labels));
    rows.push(item_row(VORSTEUER));
    rows.push(blank_row(&labels));
    rows.push(section_row("Berechnung der Umsatzsteuer", &labels));
    rows.push(item_row(VORAUSZAHLUNGSSOLL));
    rows.push(item_row(BEREITS_ENTRICHTET));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enrich::build_dataset;
    use std::path::Path;

    fn example_dataset() -> TaxDataset {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        build_dataset(&manifest_dir.join("examples/ledger/hledger.journal")).unwrap()
    }

    fn kennzahl<'a>(rows: &'a [ReportRow], label: &str) -> Option<&'a ReportRow> {
        rows.iter()
            .find(|r| r.get("Kennzahl").map(String::as_str) == Some(label))
    }

    #[test]
    fn ust_2024_annual_column_matches_golden_values() {
        let dataset = example_dataset();
        let rows = ust_rows(&dataset, 2024).unwrap();
        assert_eq!(
            kennzahl(&rows, "Bereits Entrichtet").unwrap()["2024"],
            "190,00"
        );
        assert_eq!(
            kennzahl(&rows, "Vereinnahmte Umsatzsteuer").unwrap()["2024"],
            "190,00"
        );
    }

    #[test]
    fn ust_rows_have_a_column_per_month_quarter_and_year_like_the_other_forms() {
        let dataset = example_dataset();
        let rows = ust_rows(&dataset, 2024).unwrap();
        let headers: Vec<&String> = rows[0].keys().collect();
        assert_eq!(headers[0], "Kennzahl");
        assert_eq!(headers.len(), 1 + 17);
        assert_eq!(*headers.last().unwrap(), "2024");
    }

    #[test]
    fn ust_rows_omit_reverse_charge_rows_when_unused() {
        let dataset = example_dataset();
        let rows = ust_rows(&dataset, 2024).unwrap();
        assert!(kennzahl(&rows, "§13b EU Umsatzsteuer").is_none());
    }

    #[test]
    fn vat_advance_missing_period_is_rejected() {
        use crate::posting::test_support::posting;
        let mut p = posting("2024-01-01", "assets:bank", "expenses:tax", "100.00");
        p.tax_role = "vat_advance".to_string();
        let ds = TaxDataset::new(vec![p]);
        let err = ust_rows(&ds, 2024).unwrap_err();
        assert!(err.to_string().contains("require elster_period"));
    }
}
