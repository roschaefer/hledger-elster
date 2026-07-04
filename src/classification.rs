use crate::dataset::TaxDataset;
use crate::posting::TaxPosting;
use rust_decimal::Decimal;

const EUER_FORM: &str = "einnahmenueberschussrechnung";
const EXPENSE_DEDUCTIONS: &[&str] = &["full", "proportional", "non_deductible", "afa"];

fn account_has_prefix(account: &str, prefix: &str) -> bool {
    let normalized = account.to_lowercase();
    normalized == prefix || normalized.starts_with(&format!("{prefix}:"))
}

pub fn is_euer_expense(p: &TaxPosting) -> bool {
    if p.tax_form != EUER_FORM {
        return false;
    }
    if account_has_prefix(&p.counter_account, "income") {
        return false;
    }
    account_has_prefix(&p.counter_account, "expenses")
        || EXPENSE_DEDUCTIONS.contains(&p.tax_deduction.as_str())
}

pub fn is_euer_income(p: &TaxPosting) -> bool {
    if p.tax_form != EUER_FORM {
        return false;
    }
    if account_has_prefix(&p.counter_account, "income") {
        return true;
    }
    if is_euer_expense(p) {
        return false;
    }
    p.amount < Decimal::ZERO
}

pub fn euer_expenses(dataset: &TaxDataset) -> TaxDataset {
    dataset
        .iter()
        .filter(|p| is_euer_expense(p))
        .cloned()
        .collect()
}

pub fn euer_income(dataset: &TaxDataset) -> TaxDataset {
    dataset
        .iter()
        .filter(|p| is_euer_income(p))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;

    fn euer_posting(counter_account: &str, amount: &str, deduction: &str) -> TaxPosting {
        let mut p = posting("2024-01-01", "assets:bank", counter_account, amount);
        p.tax_form = EUER_FORM.to_string();
        p.tax_deduction = deduction.to_string();
        p
    }

    #[test]
    fn income_prefixed_accounts_are_always_income_not_expense() {
        let p = euer_posting("income:business:consulting", "-100.00", "");
        assert!(is_euer_income(&p));
        assert!(!is_euer_expense(&p));
    }

    #[test]
    fn income_prefix_match_is_case_insensitive() {
        let p = euer_posting("Income:Business:Consulting", "-100.00", "");
        assert!(is_euer_income(&p));
    }

    #[test]
    fn expenses_prefixed_accounts_are_expenses() {
        let p = euer_posting("expenses:business:hosting", "10.00", "full");
        assert!(is_euer_expense(&p));
        assert!(!is_euer_income(&p));
    }

    #[test]
    fn non_expenses_account_with_expense_deduction_tag_is_still_an_expense() {
        let p = euer_posting("liabilities:owner", "10.00", "full");
        assert!(is_euer_expense(&p));
    }

    #[test]
    fn negative_amount_without_income_prefix_or_expense_deduction_is_income() {
        let p = euer_posting("liabilities:owner", "-10.00", "");
        assert!(is_euer_income(&p));
        assert!(!is_euer_expense(&p));
    }

    #[test]
    fn non_euer_form_is_neither() {
        let mut p = euer_posting("expenses:business:hosting", "10.00", "full");
        p.tax_form = "einkommensteuer".to_string();
        assert!(!is_euer_expense(&p));
        assert!(!is_euer_income(&p));
    }

    #[test]
    fn euer_expenses_and_income_filter_a_dataset() {
        let ds = TaxDataset::new(vec![
            euer_posting("income:business:consulting", "-100.00", ""),
            euer_posting("expenses:business:hosting", "10.00", "full"),
        ]);
        assert_eq!(euer_income(&ds).len(), 1);
        assert_eq!(euer_expenses(&ds).len(), 1);
    }
}
