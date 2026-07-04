use crate::afa;
use crate::aggregates;
use crate::classification::{euer_expenses, euer_income};
use crate::config::TaxConfig;
use crate::dataset::TaxDataset;
use crate::drawing::{is_contribution, is_drawing};
use crate::periods::{aggregate_periods, annual_labels, blank_row, fmt, section_row, ReportRow};
use crate::posting::TaxPosting;
use indexmap::IndexMap;
use rust_decimal::{Decimal, RoundingStrategy};

fn quantize(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Section ordering for EÜR Betriebsausgaben.
const SECTION_ORDER: &[&str] = &[
    "Bezogene Fremdleistungen",
    "Fortbildungskosten",
    "Rechts- und Steuerberatung",
    "Arbeitsmittel",
];

fn section_sort_key(name: &str) -> usize {
    SECTION_ORDER
        .iter()
        .position(|s| *s == name)
        .unwrap_or(SECTION_ORDER.len())
}

fn last_component(account: &str) -> String {
    account.rsplit(':').next().unwrap_or(account).to_string()
}

fn posting_label(p: &TaxPosting) -> String {
    if p.label.is_empty() {
        last_component(&p.counter_account)
    } else {
        p.label.clone()
    }
}

fn row_from_values(name: &str, values: &IndexMap<String, Decimal>) -> ReportRow {
    let mut row = ReportRow::new();
    row.insert("Kennzahl".to_string(), name.to_string());
    for (label, value) in values {
        row.insert(label.clone(), fmt(*value));
    }
    row
}

/// Distribute annual AfA across periods by counting active months in each period.
fn afa_for_label(postings: &[TaxPosting], year: i32, label: &str) -> Decimal {
    let total: Decimal = postings
        .iter()
        .map(|p| afa_posting_for_label(p, year, label))
        .sum();
    quantize(total)
}

fn afa_posting_for_label(p: &TaxPosting, year: i32, label: &str) -> Decimal {
    use chrono::Datelike;

    let purchase = p.posting_date;
    let total_months = p.afa_years * 12;
    if total_months <= 0 {
        return Decimal::ZERO;
    }

    let cost = afa::net_cost(p);
    let monthly = cost / Decimal::from(total_months);

    let window_end_month_abs =
        (purchase.year() - 1) * 12 + purchase.month() as i32 - 1 + total_months;
    let window_end_year = (window_end_month_abs - 1).div_euclid(12) + 1;
    let window_end_cal_month = (window_end_month_abs - 1).rem_euclid(12) + 1;

    let month_in_window = |y: i32, m: i32| -> bool {
        if y < purchase.year() || (y == purchase.year() && m < purchase.month() as i32) {
            return false;
        }
        if y > window_end_year || (y == window_end_year && m > window_end_cal_month) {
            return false;
        }
        true
    };

    let months: Vec<i32> = if label == year.to_string() {
        (1..=12).filter(|&m| month_in_window(year, m)).collect()
    } else if label.contains('Q') {
        let q: i32 = label.split('Q').nth(1).unwrap().parse().unwrap();
        let start = (q - 1) * 3 + 1;
        (start..start + 3)
            .filter(|&m| month_in_window(year, m))
            .collect()
    } else {
        let m: i32 = label.split('-').nth(1).unwrap().parse().unwrap();
        if month_in_window(year, m) {
            vec![m]
        } else {
            vec![]
        }
    };

    monthly * Decimal::from(months.len())
}

pub fn euer_rows(dataset: &TaxDataset, year: i32, config: &TaxConfig) -> Vec<ReportRow> {
    let labels = annual_labels(year);
    let euer_ds = euer_expenses(dataset);

    // ── Betriebseinnahmen ────────────────────────────────────────────────
    let income_ds = euer_income(dataset);
    let net_totals = aggregate_periods(&income_ds, year, aggregates::net_amount, &labels);
    let collected_totals = aggregate_periods(&income_ds, year, aggregates::collected_vat, &labels);

    let mut einnahmen_net_row = ReportRow::new();
    einnahmen_net_row.insert(
        "Kennzahl".to_string(),
        "Umsatzsteuerpflichtige Betriebseinnahmen".to_string(),
    );
    let mut einnahmen_ust_row = ReportRow::new();
    einnahmen_ust_row.insert(
        "Kennzahl".to_string(),
        "Vereinnahmte Umsatzsteuer".to_string(),
    );
    let mut einnahmen_total: IndexMap<String, Decimal> = IndexMap::new();

    for lbl in &labels {
        let net = *net_totals.get(lbl).unwrap();
        let collected = *collected_totals.get(lbl).unwrap();
        einnahmen_net_row.insert(lbl.clone(), fmt(net));
        einnahmen_ust_row.insert(lbl.clone(), fmt(collected));
        einnahmen_total.insert(lbl.clone(), quantize(net + collected));
    }

    // ── regular expense accounts, grouped by section ────────────────────
    let regular_ds = euer_ds.exclude_deduction("afa").for_year(year);

    let mut by_section: IndexMap<String, IndexMap<String, Vec<TaxPosting>>> = IndexMap::new();
    for p in regular_ds.iter() {
        by_section
            .entry(p.section.clone())
            .or_default()
            .entry(posting_label(p))
            .or_default()
            .push(p.clone());
    }

    let mut section_names: Vec<String> = by_section.keys().cloned().collect();
    section_names.sort_by_key(|s| section_sort_key(s));

    let mut expense_sections: Vec<(String, Vec<ReportRow>)> = Vec::new();
    let mut summe_betriebskosten: IndexMap<String, Decimal> =
        labels.iter().map(|l| (l.clone(), Decimal::ZERO)).collect();

    for section_name in &section_names {
        let by_label = by_section.get(section_name).unwrap();
        let mut posting_labels: Vec<String> = by_label.keys().cloned().collect();
        posting_labels.sort();

        let mut section_rows: Vec<ReportRow> = Vec::new();
        for label in posting_labels {
            let acc_ds = TaxDataset::new(by_label.get(&label).unwrap().clone());
            let mut row = ReportRow::new();
            row.insert("Kennzahl".to_string(), label);
            let totals = aggregate_periods(&acc_ds, year, aggregates::deductible_net, &labels);
            for lbl in &labels {
                let value = *totals.get(lbl).unwrap();
                row.insert(lbl.clone(), fmt(value));
                *summe_betriebskosten.get_mut(lbl).unwrap() += value;
            }
            section_rows.push(row);
        }
        if !section_rows.is_empty() {
            expense_sections.push((section_name.clone(), section_rows));
        }
    }
    for lbl in &labels {
        let v = *summe_betriebskosten.get(lbl).unwrap();
        summe_betriebskosten.insert(lbl.clone(), quantize(v));
    }

    // ── AfA accounts ─────────────────────────────────────────────────────
    let afa_postings: Vec<TaxPosting> = dataset.for_deduction("afa").iter().cloned().collect();
    let mut afa_accounts: Vec<String> = afa_postings
        .iter()
        .map(|p| p.counter_account.clone())
        .collect();
    afa_accounts.sort();
    afa_accounts.dedup();

    let mut afa_sections: IndexMap<String, Vec<ReportRow>> = IndexMap::new();
    let mut afa_totals: IndexMap<String, Decimal> =
        labels.iter().map(|l| (l.clone(), Decimal::ZERO)).collect();

    for account in &afa_accounts {
        let acc_postings: Vec<TaxPosting> = afa_postings
            .iter()
            .filter(|p| &p.counter_account == account)
            .cloned()
            .collect();
        let label = acc_postings
            .iter()
            .find(|p| !p.label.is_empty())
            .map(|p| p.label.clone())
            .unwrap_or_else(|| last_component(account));
        let section_name = acc_postings
            .iter()
            .find(|p| !p.section.is_empty())
            .map(|p| p.section.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "AfA".to_string());

        let mut row = ReportRow::new();
        row.insert("Kennzahl".to_string(), format!("AfA {label}"));
        for lbl in &labels {
            let value = afa_for_label(&acc_postings, year, lbl);
            row.insert(lbl.clone(), fmt(value));
            *afa_totals.get_mut(lbl).unwrap() += value;
        }
        afa_sections.entry(section_name).or_default().push(row);
    }
    for lbl in &labels {
        let v = *afa_totals.get(lbl).unwrap();
        afa_totals.insert(lbl.clone(), quantize(v));
    }

    // ── Home-Office-Pauschale ────────────────────────────────────────────
    let annual_hop = config.home_office_pauschale.amount_for_year(year);

    // ── UStVA payments (ELSTER EÜR line 57) — advance + final settlements ──
    let vat_paid_ds: TaxDataset = dataset
        .iter()
        .filter(|p| {
            matches!(p.tax_role.as_str(), "vat_payment" | "vat_advance") && p.amount > Decimal::ZERO
        })
        .cloned()
        .collect();
    let vat_refund_ds: TaxDataset = dataset
        .iter()
        .filter(|p| p.tax_role == "vat_payment" && p.amount < Decimal::ZERO)
        .cloned()
        .collect();
    let ust_paid_totals = aggregate_periods(&vat_paid_ds, year, aggregates::gross_amount, &labels);
    let ust_refund_totals =
        aggregate_periods(&vat_refund_ds, year, aggregates::gross_amount, &labels);

    // ── Entnahmen / Einlagen from business accounts ─────────────────────
    let drawing_ds: TaxDataset = dataset.iter().filter(|p| is_drawing(p)).cloned().collect();
    let drawing_totals = aggregate_periods(&drawing_ds, year, aggregates::gross_amount, &labels);
    let contribution_ds: TaxDataset = dataset
        .iter()
        .filter(|p| is_contribution(p))
        .cloned()
        .collect();
    let contribution_totals =
        aggregate_periods(&contribution_ds, year, aggregates::gross_amount, &labels);

    // ── summary rows ─────────────────────────────────────────────────────
    let mut summe_betriebseinnahmen: IndexMap<String, Decimal> = IndexMap::new();
    let mut summe_betriebsausgaben: IndexMap<String, Decimal> = IndexMap::new();
    let mut gewinn_totals: IndexMap<String, Decimal> = IndexMap::new();

    for lbl in &labels {
        let einnahmen = *einnahmen_total.get(lbl).unwrap();
        let refund = *ust_refund_totals.get(lbl).unwrap();
        summe_betriebseinnahmen.insert(lbl.clone(), quantize(einnahmen + refund));

        let betriebsausgaben = quantize(
            *summe_betriebskosten.get(lbl).unwrap()
                + *afa_totals.get(lbl).unwrap()
                + annual_hop
                + *ust_paid_totals.get(lbl).unwrap(),
        );
        summe_betriebsausgaben.insert(lbl.clone(), betriebsausgaben);
        gewinn_totals.insert(lbl.clone(), quantize(einnahmen - betriebsausgaben));
    }

    let mut rows: Vec<ReportRow> = vec![
        section_row("Betriebseinnahmen", &labels),
        einnahmen_net_row,
        einnahmen_ust_row,
        row_from_values(
            "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer",
            &ust_refund_totals,
        ),
        row_from_values("Summe Betriebseinnahmen", &summe_betriebseinnahmen),
        blank_row(&labels),
        section_row("Betriebsausgaben", &labels),
    ];

    for (section_name, section_rows) in expense_sections {
        if !section_name.is_empty() {
            rows.push(section_row(&section_name, &labels));
        }
        rows.extend(section_rows);
    }

    let mut afa_section_names: Vec<String> = afa_sections.keys().cloned().collect();
    afa_section_names.sort_by_key(|s| section_sort_key(s));
    for section_name in afa_section_names {
        rows.push(section_row(&section_name, &labels));
        rows.extend(afa_sections.get(&section_name).unwrap().clone());
    }

    rows.push(blank_row(&labels));
    if annual_hop != Decimal::ZERO {
        let hop_values: IndexMap<String, Decimal> =
            labels.iter().map(|l| (l.clone(), annual_hop)).collect();
        rows.push(row_from_values("Home-Office-Pauschale", &hop_values));
    }
    rows.extend([
        row_from_values(
            "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer",
            &ust_paid_totals,
        ),
        row_from_values("Summe Betriebskosten", &summe_betriebskosten),
        row_from_values("Summe Betriebsausgaben", &summe_betriebsausgaben),
        blank_row(&labels),
        section_row("Ermittlung des Gewinns", &labels),
        row_from_values("Steuerpflichtiger Gewinn/Verlust", &gewinn_totals),
        blank_row(&labels),
        section_row("Zusätzliche Angaben bei Einzelunternehmen", &labels),
        row_from_values("Entnahmen", &drawing_totals),
        row_from_values("Einlagen", &contribution_totals),
    ]);

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
    fn euer_2024_matches_golden_values_from_python_test_suite() {
        let dataset = example_dataset();
        let rows = euer_rows(&dataset, 2024, &TaxConfig::default());

        assert_eq!(
            row(&rows, "Umsatzsteuerpflichtige Betriebseinnahmen")["2024"],
            "1000.00"
        );
        assert_eq!(row(&rows, "Vereinnahmte Umsatzsteuer")["2024"], "190.00");
        assert_eq!(
            row(&rows, "Steuerpflichtiger Gewinn/Verlust")["2024"],
            "-824.22"
        );
    }

    #[test]
    fn deductible_net_matches_golden_values_for_labeled_expenses() {
        let dataset = example_dataset();
        let rows = euer_rows(&dataset, 2024, &TaxConfig::default());

        assert_eq!(row(&rows, "Serverkosten Wasabi")["2024"], "20.00");
        assert_eq!(row(&rows, "Mobiltelefon")["2024"], "2.00");
        assert_eq!(row(&rows, "Steuerberatung")["2024"], "100.00");
    }

    #[test]
    fn afa_row_spans_multiple_years() {
        let dataset = example_dataset();
        let rows_2024 = euer_rows(&dataset, 2024, &TaxConfig::default());
        let rows_2025 = euer_rows(&dataset, 2025, &TaxConfig::default());

        assert_eq!(row(&rows_2024, "AfA Computer-Kauf")["2024"], "222.22");
        assert_eq!(row(&rows_2025, "AfA Computer-Kauf")["2025"], "333.33");
    }

    #[test]
    fn drawing_row_reflects_owner_draw_transaction() {
        let dataset = example_dataset();
        let rows = euer_rows(&dataset, 2024, &TaxConfig::default());
        assert_eq!(row(&rows, "Entnahmen")["2024"], "500.00");
    }
}
