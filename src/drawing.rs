use crate::posting::TaxPosting;
use rust_decimal::Decimal;

const EUER_FORM: &str = "einnahmenueberschussrechnung";

/// Roles that are never Entnahmen/Einlagen regardless of source account.
const NON_DRAWING_ROLES: &[&str] = &[
    "vat_payment",
    "vat_advance",
    "income_tax",
    "income_tax_advance",
    "income_tax_final",
    "tax_payment",
    "ignore",
];

/// True when a posting represents a private withdrawal from the business account.
///
/// A drawing is any outflow from the business sphere that is not a business
/// expense (EÜR) and not a tax payment — regardless of how the counter-account
/// is categorised for user-defined tax form sections.
pub fn is_drawing(p: &TaxPosting) -> bool {
    p.source_is_business
        && p.amount > Decimal::ZERO
        && p.tax_form != EUER_FORM
        && !NON_DRAWING_ROLES.contains(&p.tax_role.as_str())
}

/// True when a posting represents a private deposit into the business account.
pub fn is_contribution(p: &TaxPosting) -> bool {
    p.source_is_business
        && p.amount < Decimal::ZERO
        && p.tax_form != EUER_FORM
        && !NON_DRAWING_ROLES.contains(&p.tax_role.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;

    fn business_posting(amount: &str, tax_form: &str, tax_role: &str) -> TaxPosting {
        let mut p = posting("2024-01-01", "assets:bank", "liabilities:owner", amount);
        p.source_is_business = true;
        p.tax_form = tax_form.to_string();
        p.tax_role = tax_role.to_string();
        p
    }

    #[test]
    fn positive_outflow_from_business_is_a_drawing() {
        let p = business_posting("25.00", "", "");
        assert!(is_drawing(&p));
        assert!(!is_contribution(&p));
    }

    #[test]
    fn negative_inflow_to_business_is_a_contribution() {
        let p = business_posting("-40.00", "", "");
        assert!(is_contribution(&p));
        assert!(!is_drawing(&p));
    }

    #[test]
    fn euer_expenses_are_never_drawings() {
        let p = business_posting("25.00", "einnahmenueberschussrechnung", "");
        assert!(!is_drawing(&p));
    }

    #[test]
    fn tax_payments_are_never_drawings_or_contributions() {
        let p = business_posting("25.00", "", "tax_payment");
        assert!(!is_drawing(&p));
        let n = business_posting("-25.00", "", "vat_advance");
        assert!(!is_contribution(&n));
    }

    #[test]
    fn non_business_source_is_never_a_drawing() {
        let mut p = business_posting("25.00", "", "");
        p.source_is_business = false;
        assert!(!is_drawing(&p));
    }
}
