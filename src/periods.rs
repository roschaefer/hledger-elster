use crate::dataset::TaxDataset;
use indexmap::IndexMap;
use rust_decimal::Decimal;

/// A single output row, keyed by column name in display order — mirrors
/// Python's `dict[str, str]` rows, whose column set varies by report (e.g.
/// USt gains reverse-charge columns only when needed).
pub type ReportRow = IndexMap<String, String>;

pub fn annual_labels(year: i32) -> Vec<String> {
    vec![year.to_string()]
}

pub fn filter_period(dataset: &TaxDataset, year: i32, label: &str) -> TaxDataset {
    if label == year.to_string() {
        return dataset.for_year(year);
    }
    if let Some(q) = label.split('Q').nth(1) {
        return dataset.for_quarter(year, q.parse().unwrap());
    }
    let m: u32 = label.split('-').nth(1).unwrap().parse().unwrap();
    dataset.for_month(year, m)
}

pub fn aggregate_periods(
    dataset: &TaxDataset,
    year: i32,
    f: impl Fn(&TaxDataset) -> Decimal,
    labels: &[String],
) -> IndexMap<String, Decimal> {
    labels
        .iter()
        .map(|label| (label.clone(), f(&filter_period(dataset, year, label))))
        .collect()
}

pub fn fmt(value: Decimal) -> String {
    format!("{value:.2}")
}

pub fn blank_row(labels: &[String]) -> ReportRow {
    let mut row = ReportRow::new();
    row.insert("Kennzahl".to_string(), String::new());
    for label in labels {
        row.insert(label.clone(), String::new());
    }
    row
}

/// Section header row — `Kennzahl` prefixed with "# " so the writer can style it.
pub fn section_row(name: &str, labels: &[String]) -> ReportRow {
    let mut row = ReportRow::new();
    row.insert("Kennzahl".to_string(), format!("# {name}"));
    for label in labels {
        row.insert(label.clone(), String::new());
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;
    use std::str::FromStr;

    #[test]
    fn filter_period_dispatches_on_label_shape() {
        let ds = TaxDataset::new(vec![
            posting("2024-02-15", "a", "b", "1"),
            posting("2024-05-15", "a", "b", "1"),
        ]);
        assert_eq!(filter_period(&ds, 2024, "2024").len(), 2);
        assert_eq!(filter_period(&ds, 2024, "2024 Q1").len(), 1);
        assert_eq!(filter_period(&ds, 2024, "2024-05").len(), 1);
    }

    #[test]
    fn fmt_always_shows_two_decimal_places() {
        assert_eq!(fmt(Decimal::from_str("5").unwrap()), "5.00");
        assert_eq!(fmt(Decimal::from_str("-5.5").unwrap()), "-5.50");
    }

    #[test]
    fn blank_and_section_rows_have_empty_label_columns() {
        let labels = vec!["2024".to_string()];
        let blank = blank_row(&labels);
        assert_eq!(blank.get("Kennzahl").unwrap(), "");
        assert_eq!(blank.get("2024").unwrap(), "");

        let section = section_row("Betriebseinnahmen", &labels);
        assert_eq!(section.get("Kennzahl").unwrap(), "# Betriebseinnahmen");
    }
}
