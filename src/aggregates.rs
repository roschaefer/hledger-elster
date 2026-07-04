use crate::dataset::TaxDataset;
use crate::posting::TaxPosting;
use rust_decimal::{Decimal, RoundingStrategy};

fn quantize(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Net amount for a single posting.
fn net(posting: &TaxPosting) -> Decimal {
    let gross = posting.amount.abs();
    if posting.vat_mode == "contains_vat" && posting.vat_rate > Decimal::ZERO {
        gross / (Decimal::ONE + posting.vat_rate)
    } else {
        gross
    }
}

/// VAT amount for a single posting.
fn vat(posting: &TaxPosting) -> Decimal {
    posting.amount.abs() - net(posting)
}

/// Sum of absolute gross amounts across all postings in the dataset.
pub fn gross_amount(dataset: &TaxDataset) -> Decimal {
    quantize(dataset.iter().map(|p| p.amount.abs()).sum())
}

/// Sum of raw amounts preserving sign — for expense accounts where refunds should reduce the total.
pub fn signed_total(dataset: &TaxDataset) -> Decimal {
    quantize(dataset.iter().map(|p| p.amount).sum())
}

/// Sum of net amounts (gross / (1 + vat_rate)) across all postings.
pub fn net_amount(dataset: &TaxDataset) -> Decimal {
    quantize(dataset.iter().map(net).sum())
}

/// Net amount preserving sign — for expense aggregations where refunds should reduce the total.
fn signed_net(posting: &TaxPosting) -> Decimal {
    if posting.vat_mode == "contains_vat" && posting.vat_rate > Decimal::ZERO {
        posting.amount / (Decimal::ONE + posting.vat_rate)
    } else {
        posting.amount
    }
}

fn signed_vat(posting: &TaxPosting) -> Decimal {
    posting.amount - signed_net(posting)
}

/// Sum of deductible net amounts.
///
/// For each posting: net * expense_share, rounded per posting, then summed.
/// Sign is preserved so refunds correctly reduce the total.
pub fn deductible_net(dataset: &TaxDataset) -> Decimal {
    let total: Decimal = dataset
        .iter()
        .map(|p| quantize(signed_net(p) * p.expense_share))
        .sum();
    quantize(total)
}

/// Sum of deductible input VAT amounts.
///
/// For each posting: vat * input_vat_share, rounded per posting, then summed.
/// Sign is preserved so VAT refunds correctly reduce the total.
pub fn deductible_vat(dataset: &TaxDataset) -> Decimal {
    let total: Decimal = dataset
        .iter()
        .map(|p| quantize(signed_vat(p) * p.input_vat_share))
        .sum();
    quantize(total)
}

/// VAT collected on income postings (gross - net per posting, summed).
///
/// Use with a dataset filtered to income postings.
pub fn collected_vat(dataset: &TaxDataset) -> Decimal {
    let total: Decimal = dataset.iter().map(|p| quantize(vat(p))).sum();
    quantize(total)
}

/// Sum net reverse-charge bases preserving signs for refunds/corrections.
pub fn reverse_charge_base(dataset: &TaxDataset, kind: &str) -> Decimal {
    let mode = format!("reverse_charge_{kind}");
    let total: Decimal = dataset
        .iter()
        .filter(|p| p.vat_mode == mode)
        .map(|p| p.amount)
        .sum();
    quantize(total)
}

/// VAT owed by the recipient for reverse-charge postings.
pub fn reverse_charge_vat(dataset: &TaxDataset, kind: &str) -> Decimal {
    let mode = format!("reverse_charge_{kind}");
    let total: Decimal = dataset
        .iter()
        .filter(|p| p.vat_mode == mode)
        .map(|p| quantize(p.amount * p.vat_rate))
        .sum();
    quantize(total)
}

/// Deductible input VAT for reverse-charge postings.
///
/// Reverse-charge invoices are booked net. The recipient owes German VAT and,
/// when the expense is business-deductible, deducts that same VAT as input VAT.
pub fn reverse_charge_input_vat(dataset: &TaxDataset) -> Decimal {
    let total: Decimal = dataset
        .iter()
        .filter(|p| p.vat_mode.starts_with("reverse_charge_"))
        .map(|p| quantize(p.amount * p.vat_rate * p.input_vat_share))
        .sum();
    quantize(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;
    use std::str::FromStr;

    fn with_vat(mut p: TaxPosting, mode: &str, rate: &str) -> TaxPosting {
        p.vat_mode = mode.to_string();
        p.vat_rate = Decimal::from_str(rate).unwrap();
        p
    }

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn gross_amount_sums_absolute_values() {
        let ds = TaxDataset::new(vec![
            posting("2024-01-01", "a", "b", "-10.00"),
            posting("2024-01-02", "a", "b", "5.00"),
        ]);
        assert_eq!(gross_amount(&ds), d("15.00"));
    }

    #[test]
    fn signed_total_preserves_sign_so_refunds_reduce_the_total() {
        let ds = TaxDataset::new(vec![
            posting("2024-01-01", "a", "b", "100.00"),
            posting("2024-01-02", "a", "b", "-20.00"), // reimbursement
        ]);
        assert_eq!(signed_total(&ds), d("80.00"));
    }

    #[test]
    fn net_amount_strips_vat_for_contains_vat_postings() {
        let ds = TaxDataset::new(vec![with_vat(
            posting("2024-01-01", "a", "b", "119.00"),
            "contains_vat",
            "0.19",
        )]);
        assert_eq!(net_amount(&ds), d("100.00"));
    }

    #[test]
    fn collected_vat_sums_gross_minus_net_for_income_postings() {
        let ds = TaxDataset::new(vec![with_vat(
            posting("2024-01-01", "a", "income:business", "-119.00"),
            "contains_vat",
            "0.19",
        )]);
        assert_eq!(collected_vat(&ds), d("19.00"));
    }

    #[test]
    fn deductible_net_applies_expense_share_and_rounds_per_posting() {
        let mut p = with_vat(
            posting("2024-01-01", "a", "expenses:phone", "100.00"),
            "contains_vat",
            "0.19",
        );
        p.expense_share = d("0.20");
        let ds = TaxDataset::new(vec![p]);
        // net = 100/1.19 = 84.03361..., * 0.20 = 16.8067..., rounds to 16.81
        assert_eq!(deductible_net(&ds), d("16.81"));
    }

    #[test]
    fn deductible_vat_applies_input_vat_share_and_rounds_per_posting() {
        let mut p = with_vat(
            posting("2024-01-01", "a", "expenses:phone", "100.00"),
            "contains_vat",
            "0.19",
        );
        p.input_vat_share = d("0.20");
        let ds = TaxDataset::new(vec![p]);
        // vat = 100 - 100/1.19 = 15.9664..., * 0.20 = 3.1933, rounds to 3.19
        assert_eq!(deductible_vat(&ds), d("3.19"));
    }

    #[test]
    fn reverse_charge_base_only_sums_matching_kind() {
        let ds = TaxDataset::new(vec![
            with_vat(
                posting("2024-01-01", "a", "b", "100.00"),
                "reverse_charge_eu",
                "0.19",
            ),
            with_vat(
                posting("2024-01-02", "a", "b", "50.00"),
                "reverse_charge_non_eu",
                "0.19",
            ),
        ]);
        assert_eq!(reverse_charge_base(&ds, "eu"), d("100.00"));
        assert_eq!(reverse_charge_base(&ds, "non_eu"), d("50.00"));
    }

    #[test]
    fn reverse_charge_vat_is_amount_times_rate() {
        let ds = TaxDataset::new(vec![with_vat(
            posting("2024-01-01", "a", "b", "100.00"),
            "reverse_charge_eu",
            "0.19",
        )]);
        assert_eq!(reverse_charge_vat(&ds, "eu"), d("19.00"));
    }

    #[test]
    fn reverse_charge_input_vat_applies_input_vat_share() {
        let mut p = with_vat(
            posting("2024-01-01", "a", "b", "100.00"),
            "reverse_charge_eu",
            "0.19",
        );
        p.input_vat_share = d("1");
        let ds = TaxDataset::new(vec![p]);
        assert_eq!(reverse_charge_input_vat(&ds), d("19.00"));
    }
}
