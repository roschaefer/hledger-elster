use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

/// A single enriched hledger posting, ready for tax-form aggregation.
///
/// `amount` follows hledger's sign convention: positive = expense, negative = income.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxPosting {
    // Bookkeeping dimensions
    pub posting_date: NaiveDate,
    pub source_account: String,
    pub counter_account: String,
    pub amount: Decimal,
    pub description: String,
    pub transaction_comment: String,
    pub posting_comment: String,
    pub source_file: String,
    pub source_line: u32,

    // Tax enrichment — resolved from account directives at ingest time.
    /// "einnahmenueberschussrechnung" | "einkommensteuer" | ""
    pub tax_form: String,
    /// "full" | "proportional" | "non_deductible" | "afa" | ""
    pub tax_deduction: String,
    /// "tax_payment" | "income_tax" | "vat_payment" | "vat_advance" |
    /// "income_tax_advance" | "income_tax_final" | "drawing" | "contribution" | "ignore" | ""
    pub tax_role: String,
    /// "" | "manual"
    pub calculation: String,
    /// "contains_vat" | "reverse_charge_eu" | "reverse_charge_non_eu" | "not_applicable" | ""
    pub vat_mode: String,
    pub vat_rate: Decimal,
    pub expense_share: Decimal,
    pub input_vat_share: Decimal,

    /// AfA metadata — only set when `tax_deduction == "afa"`; 0 otherwise.
    pub afa_years: i32,

    /// "" or "abschreibung" for synthetic rows.
    pub derived_kind: String,

    // Human-readable metadata resolved from account directives.
    pub label: String,
    pub source_label: String,
    pub section: String,
    pub tax_period: String,
    pub tax_period_year: i32,
    pub source_is_business: bool,
}

impl TaxPosting {
    pub fn year(&self) -> i32 {
        self.posting_date.year()
    }

    pub fn quarter(&self) -> u32 {
        (self.posting_date.month() - 1) / 3 + 1
    }

    pub fn month(&self) -> u32 {
        self.posting_date.month()
    }
}

#[cfg(test)]
pub mod test_support {
    use super::TaxPosting;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    /// Builds a minimal `TaxPosting` with sensible empty/zero defaults, mirroring
    /// the small `_posting`/`_journal_entry` test builders in the Python test suite.
    pub fn posting(
        date: &str,
        source_account: &str,
        counter_account: &str,
        amount: &str,
    ) -> TaxPosting {
        TaxPosting {
            posting_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            source_account: source_account.to_string(),
            counter_account: counter_account.to_string(),
            amount: amount.parse::<Decimal>().unwrap(),
            description: String::new(),
            transaction_comment: String::new(),
            posting_comment: String::new(),
            source_file: String::new(),
            source_line: 0,
            tax_form: String::new(),
            tax_deduction: String::new(),
            tax_role: String::new(),
            calculation: String::new(),
            vat_mode: String::new(),
            vat_rate: Decimal::ZERO,
            expense_share: Decimal::ZERO,
            input_vat_share: Decimal::ZERO,
            afa_years: 0,
            derived_kind: String::new(),
            label: String::new(),
            source_label: String::new(),
            section: String::new(),
            tax_period: String::new(),
            tax_period_year: 0,
            source_is_business: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::posting;

    #[test]
    fn year_quarter_month_are_derived_from_posting_date() {
        let p = posting("2024-05-17", "assets:bank", "expenses:foo", "10.00");
        assert_eq!(p.year(), 2024);
        assert_eq!(p.quarter(), 2);
        assert_eq!(p.month(), 5);
    }

    #[test]
    fn quarter_boundaries() {
        assert_eq!(posting("2024-01-01", "a", "b", "1").quarter(), 1);
        assert_eq!(posting("2024-03-31", "a", "b", "1").quarter(), 1);
        assert_eq!(posting("2024-04-01", "a", "b", "1").quarter(), 2);
        assert_eq!(posting("2024-06-30", "a", "b", "1").quarter(), 2);
        assert_eq!(posting("2024-07-01", "a", "b", "1").quarter(), 3);
        assert_eq!(posting("2024-09-30", "a", "b", "1").quarter(), 3);
        assert_eq!(posting("2024-10-01", "a", "b", "1").quarter(), 4);
        assert_eq!(posting("2024-12-31", "a", "b", "1").quarter(), 4);
    }
}
