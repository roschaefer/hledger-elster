use crate::posting::TaxPosting;
use indexmap::IndexMap;
use std::collections::BTreeMap;

/// An ordered collection of `TaxPosting`s with filter/group combinators.
///
/// Grouping methods use `IndexMap` (not `HashMap`) to preserve first-seen
/// insertion order, matching Python's dict semantics — this matters for
/// deterministic, byte-reproducible report output.
#[derive(Debug, Clone, Default)]
pub struct TaxDataset {
    postings: Vec<TaxPosting>,
}

impl TaxDataset {
    pub fn new(postings: Vec<TaxPosting>) -> Self {
        Self { postings }
    }

    pub fn iter(&self) -> impl Iterator<Item = &TaxPosting> {
        self.postings.iter()
    }

    pub fn len(&self) -> usize {
        self.postings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.postings.is_empty()
    }

    // ── filters ──────────────────────────────────────────────────────────

    pub fn for_form(&self, form: &str) -> TaxDataset {
        self.filter(|p| p.tax_form == form)
    }

    pub fn for_role(&self, role: &str) -> TaxDataset {
        self.filter(|p| p.tax_role == role)
    }

    pub fn for_deduction(&self, deduction: &str) -> TaxDataset {
        self.filter(|p| p.tax_deduction == deduction)
    }

    pub fn exclude_deduction(&self, deduction: &str) -> TaxDataset {
        self.filter(|p| p.tax_deduction != deduction)
    }

    pub fn for_account_prefix(&self, prefix: &str) -> TaxDataset {
        let nested = format!("{prefix}:");
        self.filter(|p| p.counter_account == prefix || p.counter_account.starts_with(&nested))
    }

    pub fn for_source_account(&self, account: &str) -> TaxDataset {
        self.filter(|p| p.source_account == account)
    }

    pub fn for_year(&self, year: i32) -> TaxDataset {
        self.filter(|p| p.year() == year)
    }

    pub fn for_quarter(&self, year: i32, quarter: u32) -> TaxDataset {
        self.filter(|p| p.year() == year && p.quarter() == quarter)
    }

    pub fn for_month(&self, year: i32, month: u32) -> TaxDataset {
        self.filter(|p| p.year() == year && p.month() == month)
    }

    // ── grouping ─────────────────────────────────────────────────────────

    pub fn group_by_counter_account(&self) -> IndexMap<String, TaxDataset> {
        self.group_by(|p| p.counter_account.clone())
    }

    pub fn group_by_source_account(&self) -> IndexMap<String, TaxDataset> {
        self.group_by(|p| p.source_account.clone())
    }

    pub fn group_by_year(&self) -> BTreeMap<i32, TaxDataset> {
        let mut buckets: BTreeMap<i32, Vec<TaxPosting>> = BTreeMap::new();
        for p in &self.postings {
            buckets.entry(p.year()).or_default().push(p.clone());
        }
        buckets
            .into_iter()
            .map(|(year, postings)| (year, TaxDataset::new(postings)))
            .collect()
    }

    fn filter(&self, predicate: impl Fn(&TaxPosting) -> bool) -> TaxDataset {
        TaxDataset::new(
            self.postings
                .iter()
                .filter(|p| predicate(p))
                .cloned()
                .collect(),
        )
    }

    fn group_by(&self, key: impl Fn(&TaxPosting) -> String) -> IndexMap<String, TaxDataset> {
        let mut buckets: IndexMap<String, Vec<TaxPosting>> = IndexMap::new();
        for p in &self.postings {
            buckets.entry(key(p)).or_default().push(p.clone());
        }
        buckets
            .into_iter()
            .map(|(k, postings)| (k, TaxDataset::new(postings)))
            .collect()
    }
}

impl<'a> IntoIterator for &'a TaxDataset {
    type Item = &'a TaxPosting;
    type IntoIter = std::slice::Iter<'a, TaxPosting>;

    fn into_iter(self) -> Self::IntoIter {
        self.postings.iter()
    }
}

impl FromIterator<TaxPosting> for TaxDataset {
    fn from_iter<T: IntoIterator<Item = TaxPosting>>(iter: T) -> Self {
        TaxDataset::new(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;

    fn sample() -> TaxDataset {
        TaxDataset::new(vec![
            {
                let mut p = posting(
                    "2024-01-05",
                    "assets:bank",
                    "expenses:business:hosting",
                    "10.00",
                );
                p.tax_form = "einnahmenueberschussrechnung".to_string();
                p.tax_deduction = "full".to_string();
                p
            },
            {
                let mut p = posting(
                    "2024-04-10",
                    "assets:bank",
                    "expenses:business:hosting",
                    "20.00",
                );
                p.tax_form = "einnahmenueberschussrechnung".to_string();
                p.tax_deduction = "afa".to_string();
                p
            },
            {
                let mut p = posting(
                    "2025-02-01",
                    "assets:bank",
                    "income:business:consulting",
                    "-50.00",
                );
                p.tax_form = "einnahmenueberschussrechnung".to_string();
                p.tax_deduction = "full".to_string();
                p
            },
        ])
    }

    #[test]
    fn for_form_filters_by_exact_match() {
        let ds = sample();
        assert_eq!(ds.for_form("einnahmenueberschussrechnung").len(), 3);
        assert_eq!(ds.for_form("einkommensteuer").len(), 0);
    }

    #[test]
    fn for_deduction_and_exclude_deduction_are_complementary() {
        let ds = sample();
        assert_eq!(ds.for_deduction("afa").len(), 1);
        assert_eq!(ds.exclude_deduction("afa").len(), 2);
    }

    #[test]
    fn for_account_prefix_matches_exact_and_nested_accounts() {
        let ds = sample();
        assert_eq!(ds.for_account_prefix("expenses:business:hosting").len(), 2);
        assert_eq!(ds.for_account_prefix("expenses:business").len(), 2);
        assert_eq!(
            ds.for_account_prefix("expenses:business:hosting:hetzner")
                .len(),
            0
        );
        // must not match "expenses:businessish" via naive prefix string match
        assert_eq!(ds.for_account_prefix("expenses:busi").len(), 0);
    }

    #[test]
    fn for_year_quarter_month_filter_by_posting_date() {
        let ds = sample();
        assert_eq!(ds.for_year(2024).len(), 2);
        assert_eq!(ds.for_year(2025).len(), 1);
        assert_eq!(ds.for_quarter(2024, 1).len(), 1);
        assert_eq!(ds.for_quarter(2024, 2).len(), 1);
        assert_eq!(ds.for_month(2024, 1).len(), 1);
    }

    #[test]
    fn group_by_year_is_sorted_ascending() {
        let ds = sample();
        let groups = ds.group_by_year();
        let years: Vec<i32> = groups.keys().copied().collect();
        assert_eq!(years, vec![2024, 2025]);
    }

    #[test]
    fn group_by_counter_account_preserves_first_seen_order() {
        let ds = sample();
        let groups = ds.group_by_counter_account();
        let accounts: Vec<&str> = groups.keys().map(String::as_str).collect();
        assert_eq!(
            accounts,
            vec!["expenses:business:hosting", "income:business:consulting"]
        );
    }
}
