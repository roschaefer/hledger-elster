use crate::aggregates;
use crate::classification::{euer_expenses, euer_income};
use crate::dataset::TaxDataset;
use crate::periods::{filter_period, fmt, ReportRow};
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

pub fn ust_rows(dataset: &TaxDataset, year: i32) -> Result<Vec<ReportRow>, VatAdvanceError> {
    let income_ds = euer_income(dataset);
    let euer_ds = euer_expenses(dataset);
    let vat_advance = vat_advance_payments(dataset)?;
    let has_reverse_charge = euer_ds
        .for_year(year)
        .iter()
        .any(|p| p.vat_mode.starts_with("reverse_charge_"));

    const COL_VORAUSZAHLUNGSSOLL: &str = "Bereits Entrichtet";

    let make_row = |lbl: &str, for_vorauszahlung: Option<&TaxDataset>| -> ReportRow {
        let income_p = filter_period(&income_ds, year, lbl);
        let euer_p = filter_period(&euer_ds, year, lbl);
        let net = aggregates::net_amount(&income_p);
        let collected = aggregates::collected_vat(&income_p);
        let reverse_charge_eu_base = aggregates::reverse_charge_base(&euer_p, "eu");
        let reverse_charge_eu_vat = aggregates::reverse_charge_vat(&euer_p, "eu");
        let reverse_charge_non_eu_base = aggregates::reverse_charge_base(&euer_p, "non_eu");
        let reverse_charge_non_eu_vat = aggregates::reverse_charge_vat(&euer_p, "non_eu");
        let reverse_charge_vat_total = reverse_charge_eu_vat + reverse_charge_non_eu_vat;
        let vorsteuer =
            aggregates::deductible_vat(&euer_p) + aggregates::reverse_charge_input_vat(&euer_p);
        let uberschuss = quantize(collected + reverse_charge_vat_total - vorsteuer);
        let vorauszahlung_value = for_vorauszahlung
            .map(aggregates::signed_total)
            .map(fmt)
            .unwrap_or_default();

        let mut row = ReportRow::new();
        row.insert("Zeitraum".to_string(), lbl.to_string());
        row.insert("Einnahme (Netto)".to_string(), fmt(net));
        row.insert("Vereinnahmte Umsatzsteuer".to_string(), fmt(collected));
        if has_reverse_charge {
            row.insert(
                "§13b EU Leistung (Netto)".to_string(),
                fmt(reverse_charge_eu_base),
            );
            row.insert(
                "§13b EU Umsatzsteuer".to_string(),
                fmt(reverse_charge_eu_vat),
            );
            row.insert(
                "§13b Non-EU Leistung (Netto)".to_string(),
                fmt(reverse_charge_non_eu_base),
            );
            row.insert(
                "§13b Non-EU Umsatzsteuer".to_string(),
                fmt(reverse_charge_non_eu_vat),
            );
        }
        row.insert("Abziehbare Vorsteuerbeträge".to_string(), fmt(vorsteuer));
        row.insert("Vorauszahlungssoll".to_string(), fmt(uberschuss));
        row.insert(COL_VORAUSZAHLUNGSSOLL.to_string(), vorauszahlung_value);
        row
    };

    let mut rows: Vec<ReportRow> = Vec::new();

    for m in 1..=12 {
        let lbl = format!("{year}-{m:02}");
        let period = vat_advance_period(&vat_advance, year, &lbl);
        rows.push(make_row(&lbl, period.as_ref()));
    }

    for q in 1..=4 {
        let lbl = format!("{year} Q{q}");
        let period = vat_advance_period(&vat_advance, year, &lbl);
        rows.push(make_row(&lbl, period.as_ref()));
    }

    let lbl = year.to_string();
    let period = vat_advance_period(&vat_advance, year, &lbl);
    rows.push(make_row(&lbl, period.as_ref()));

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

    #[test]
    fn ust_2024_annual_row_matches_golden_values() {
        let dataset = example_dataset();
        let rows = ust_rows(&dataset, 2024).unwrap();
        let annual = rows
            .iter()
            .find(|r| r.get("Zeitraum").map(String::as_str) == Some("2024"))
            .unwrap();
        assert_eq!(annual["Bereits Entrichtet"], "190.00");
        assert_eq!(annual["Vereinnahmte Umsatzsteuer"], "190.00");
    }

    #[test]
    fn ust_rows_has_twelve_months_four_quarters_and_one_annual_row() {
        let dataset = example_dataset();
        let rows = ust_rows(&dataset, 2024).unwrap();
        assert_eq!(rows.len(), 17);
    }

    #[test]
    fn ust_rows_omits_reverse_charge_columns_when_unused() {
        let dataset = example_dataset();
        let rows = ust_rows(&dataset, 2024).unwrap();
        assert!(!rows[0].contains_key("§13b EU Umsatzsteuer"));
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
