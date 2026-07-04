use crate::aggregates;
use crate::dataset::TaxDataset;
use crate::periods::{aggregate_periods, annual_labels, blank_row, fmt, section_row, ReportRow};
use crate::posting::TaxPosting;
use indexmap::IndexMap;
use rust_decimal::Decimal;

const MANUAL_PLACEHOLDER: &str = "MANUAL";

fn last_component(account: &str) -> String {
    account.rsplit(':').next().unwrap_or(account).to_string()
}

fn est_section_dataset(dataset: &TaxDataset, year: i32) -> TaxDataset {
    dataset
        .iter()
        .filter(|p| p.tax_form == "einkommensteuer" && !p.section.is_empty() && p.year() == year)
        .cloned()
        .collect()
}

fn account_label(dataset: &TaxDataset, account: &str, year: i32) -> String {
    let filtered = dataset.for_account_prefix(account).for_year(year);
    let label = filtered
        .iter()
        .next()
        .filter(|p| !p.label.is_empty())
        .map(|p| p.label.clone());
    label.unwrap_or_else(|| last_component(account))
}

fn posting_label(p: &TaxPosting) -> String {
    if p.label.is_empty() {
        last_component(&p.counter_account)
    } else {
        p.label.clone()
    }
}

fn requires_manual_calculation(dataset: &TaxDataset) -> bool {
    dataset.iter().any(|p| p.calculation == "manual")
}

pub fn est_rows(dataset: &TaxDataset, year: i32) -> Vec<ReportRow> {
    let labels = annual_labels(year);

    // ── income tax payments ─────────────────────────────────────────────
    let advance_ds = dataset.for_role("income_tax_advance");
    let final_ds = dataset.for_role("income_tax_final");

    let mut tax_rows: Vec<ReportRow> = Vec::new();
    for role_ds in [&advance_ds, &final_ds] {
        let mut accounts: Vec<String> = role_ds.iter().map(|p| p.counter_account.clone()).collect();
        accounts.sort();
        accounts.dedup();
        for account in accounts {
            let acc_ds = dataset.for_account_prefix(&account);
            let totals = aggregate_periods(&acc_ds, year, aggregates::signed_total, &labels);
            if totals.values().all(|v| *v == Decimal::ZERO) {
                continue;
            }
            let label = account_label(dataset, &account, year);
            let mut row = ReportRow::new();
            row.insert("Kennzahl".to_string(), label);
            for lbl in &labels {
                row.insert(lbl.clone(), fmt(*totals.get(lbl).unwrap()));
            }
            tax_rows.push(row);
        }
    }

    // ── ESt account sections ────────────────────────────────────────────
    let section_ds = est_section_dataset(dataset, year);
    let mut by_section: IndexMap<String, IndexMap<String, Vec<TaxPosting>>> = IndexMap::new();
    for p in section_ds.iter() {
        by_section
            .entry(p.section.clone())
            .or_default()
            .entry(posting_label(p))
            .or_default()
            .push(p.clone());
    }

    let mut section_names: Vec<String> = by_section.keys().cloned().collect();
    section_names.sort();

    let mut section_rows: Vec<ReportRow> = Vec::new();
    for sec_name in &section_names {
        if !sec_name.is_empty() {
            section_rows.push(section_row(sec_name, &labels));
        }
        let by_label = by_section.get(sec_name).unwrap();
        let mut posting_labels: Vec<String> = by_label.keys().cloned().collect();
        posting_labels.sort();
        for label in posting_labels {
            let acc_ds = TaxDataset::new(by_label.get(&label).unwrap().clone());
            let totals = aggregate_periods(&acc_ds, year, aggregates::signed_total, &labels);
            let manual = requires_manual_calculation(&acc_ds);
            let mut row = ReportRow::new();
            row.insert("Kennzahl".to_string(), label);
            for lbl in &labels {
                let value = if manual {
                    MANUAL_PLACEHOLDER.to_string()
                } else {
                    fmt(*totals.get(lbl).unwrap())
                };
                row.insert(lbl.clone(), value);
            }
            section_rows.push(row);
        }
    }

    let mut rows: Vec<ReportRow> = Vec::new();
    if !tax_rows.is_empty() {
        rows.extend(tax_rows);
        rows.push(blank_row(&labels));
    }
    rows.extend(section_rows);
    rows
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

    fn row<'a>(rows: &'a [ReportRow], label: &str) -> &'a ReportRow {
        rows.iter()
            .find(|r| r.get("Kennzahl").map(String::as_str) == Some(label))
            .unwrap()
    }

    #[test]
    fn est_2024_matches_golden_values_from_python_test_suite() {
        let dataset = example_dataset();
        let rows = est_rows(&dataset, 2024);

        assert_eq!(row(&rows, "Krankenversicherung")["2024"], "840.00");
        assert_eq!(row(&rows, "Pflegeversicherung")["2024"], "240.00");
        assert_eq!(row(&rows, "Zusatzbeitrag")["2024"], "120.00");
        assert_eq!(
            row(&rows, "Langzeit-Auslandskrankenversicherung")["2024"],
            "343.50"
        );
        assert_eq!(
            row(&rows, "Kurzzeit-Auslandskrankenversicherung")["2024"],
            "9.50"
        );
        assert_eq!(row(&rows, "Haftpflichtversicherung")["2024"], "57.88");
        assert_eq!(row(&rows, "ESt-Vorauszahlung")["2024"], "400.00");
    }

    #[test]
    fn est_2025_matches_golden_values_from_python_test_suite() {
        let dataset = example_dataset();
        let rows = est_rows(&dataset, 2025);

        assert_eq!(row(&rows, "Krankenversicherung")["2025"], "910.00");
        assert_eq!(row(&rows, "Pflegeversicherung")["2025"], "260.00");
        assert_eq!(row(&rows, "Zusatzbeitrag")["2025"], "130.00");
        assert_eq!(row(&rows, "ESt-Abschlusszahlung")["2025"], "50.00");
    }

    #[test]
    fn section_headers_are_emitted_for_non_empty_sections() {
        let dataset = example_dataset();
        let rows = est_rows(&dataset, 2024);
        assert!(rows
            .iter()
            .any(|r| r.get("Kennzahl").map(String::as_str) == Some("# Vorsorgeaufwand")));
    }
}
