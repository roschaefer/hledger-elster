use crate::afa;
use crate::dataset::TaxDataset;
use crate::periods::{fmt, ReportRow};
use crate::posting::TaxPosting;
use rust_decimal::{Decimal, RoundingStrategy};

const COLUMNS: [&str; 7] = [
    "Anschaffungsdatum",
    "Anschaffungskosten",
    "Buchwert 01.01.",
    "Zugänge",
    "AfA",
    "Abgänge",
    "Buchwert 31.12.",
];

fn quantize(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

#[derive(Default)]
struct Line {
    cost: Decimal,
    opening: Decimal,
    additions: Decimal,
    afa: Decimal,
    disposals: Decimal,
}

impl Line {
    fn closing(&self) -> Decimal {
        self.opening + self.additions - self.afa - self.disposals
    }

    fn add(&mut self, other: &Line) {
        self.cost += other.cost;
        self.opening += other.opening;
        self.additions += other.additions;
        self.afa += other.afa;
        self.disposals += other.disposals;
    }
}

/// The asset's line in a year, in its deductible share, so the AfA column
/// adds up to the EÜR's AfA rows. `None` once it is written off.
fn line(p: &TaxPosting, year: i32) -> Option<Line> {
    let first_year = p.year();
    if year < first_year {
        return None;
    }
    let cost = quantize(quantize(afa::net_cost(p)) * p.expense_share);
    let written_off: Decimal = (first_year..year)
        .map(|y| afa::deductible_for_year(p, y).total())
        .sum();
    let this = afa::deductible_for_year(p, year);
    let (opening, additions) = if year == first_year {
        (Decimal::ZERO, cost)
    } else {
        (cost - written_off, Decimal::ZERO)
    };
    if opening == Decimal::ZERO && additions == Decimal::ZERO {
        return None;
    }
    Some(Line {
        cost,
        opening,
        additions,
        afa: this.afa,
        disposals: this.residual,
    })
}

fn row(name: &str, date: &str, l: &Line) -> ReportRow {
    let values = [
        date.to_string(),
        fmt(l.cost),
        fmt(l.opening),
        fmt(l.additions),
        fmt(l.afa),
        fmt(l.disposals),
        fmt(l.closing()),
    ];
    let mut row = ReportRow::new();
    row.insert("Kennzahl".to_string(), name.to_string());
    for (column, value) in COLUMNS.iter().zip(values) {
        row.insert(column.to_string(), value);
    }
    row
}

/// Anlageverzeichnis (Anlage AVEÜR): every depreciated asset that has a book
/// value during the year. Empty when there is none.
pub fn aveur_rows(dataset: &TaxDataset, year: i32) -> Vec<ReportRow> {
    let mut assets: Vec<TaxPosting> = dataset
        .for_deduction("afa")
        .iter()
        .filter(|p| p.expense_share > Decimal::ZERO)
        .cloned()
        .collect();
    assets.sort_by(|a, b| (a.posting_date, &a.description).cmp(&(b.posting_date, &b.description)));

    let mut rows = Vec::new();
    let mut total = Line::default();
    for p in &assets {
        if let Some(l) = line(p, year) {
            rows.push(row(&p.description, &p.posting_date.to_string(), &l));
            total.add(&l);
        }
    }
    if !rows.is_empty() {
        rows.push(row("Summe Anlageverzeichnis", "", &total));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;
    use chrono::NaiveDate;
    use std::str::FromStr;

    fn asset(date: &str, net: &str, years: i32) -> TaxPosting {
        let mut p = posting(date, "assets:bank", "expenses:hardware:computer", net);
        p.afa_years = years;
        p.tax_deduction = "afa".to_string();
        p.expense_share = Decimal::ONE;
        p.description = "Laptop".to_string();
        p
    }

    fn cells(rows: &[ReportRow], name: &str) -> Vec<String> {
        let row = rows.iter().find(|r| r["Kennzahl"] == name).unwrap();
        COLUMNS.iter().map(|c| row[*c].clone()).collect()
    }

    #[test]
    fn an_asset_is_an_addition_in_its_purchase_year() {
        let ds = TaxDataset::new(vec![asset("2024-01-10", "3600.00", 3)]);
        let rows = aveur_rows(&ds, 2024);
        assert_eq!(
            cells(&rows, "Laptop"),
            [
                "2024-01-10",
                "3600,00",
                "0,00",
                "3600,00",
                "1200,00",
                "0,00",
                "2400,00"
            ]
        );
    }

    #[test]
    fn a_later_year_starts_from_the_book_value() {
        let ds = TaxDataset::new(vec![asset("2024-01-10", "3600.00", 3)]);
        let rows = aveur_rows(&ds, 2025);
        assert_eq!(
            cells(&rows, "Laptop"),
            [
                "2024-01-10",
                "3600,00",
                "2400,00",
                "0,00",
                "1200,00",
                "0,00",
                "1200,00"
            ]
        );
    }

    #[test]
    fn a_disposal_removes_the_remaining_book_value() {
        let mut p = asset("2024-01-10", "3600.00", 3);
        p.afa_disposal = Some(NaiveDate::from_ymd_opt(2025, 3, 20).unwrap());
        let ds = TaxDataset::new(vec![p]);
        assert_eq!(
            cells(&aveur_rows(&ds, 2025), "Laptop"),
            [
                "2024-01-10",
                "3600,00",
                "2400,00",
                "0,00",
                "300,00",
                "2100,00",
                "0,00"
            ]
        );
        assert!(aveur_rows(&ds, 2026).is_empty());
    }

    #[test]
    fn a_written_off_asset_is_no_longer_listed() {
        let ds = TaxDataset::new(vec![asset("2024-01-10", "3600.00", 1)]);
        assert!(aveur_rows(&ds, 2025).is_empty());
    }

    #[test]
    fn a_partly_private_asset_is_listed_with_its_business_share() {
        let mut p = asset("2024-01-10", "1200.00", 5);
        p.expense_share = Decimal::from_str("0.20").unwrap();
        let ds = TaxDataset::new(vec![p]);
        assert_eq!(
            cells(&aveur_rows(&ds, 2024), "Laptop"),
            [
                "2024-01-10",
                "240,00",
                "0,00",
                "240,00",
                "48,00",
                "0,00",
                "192,00"
            ]
        );
    }

    #[test]
    fn a_non_deductible_asset_is_not_listed() {
        let mut p = asset("2024-01-10", "1200.00", 3);
        p.expense_share = Decimal::ZERO;
        assert!(aveur_rows(&TaxDataset::new(vec![p]), 2024).is_empty());
    }

    #[test]
    fn the_sum_row_adds_up_all_assets() {
        let ds = TaxDataset::new(vec![
            asset("2024-01-10", "3600.00", 3),
            asset("2024-07-01", "1200.00", 1),
        ]);
        assert_eq!(
            cells(&aveur_rows(&ds, 2024), "Summe Anlageverzeichnis"),
            ["", "4800,00", "0,00", "4800,00", "1800,00", "0,00", "3000,00"]
        );
    }
}
