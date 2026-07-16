use crate::posting::TaxPosting;
use chrono::Datelike;
use rust_decimal::{Decimal, RoundingStrategy};

fn quantize(value: Decimal) -> Decimal {
    value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Net acquisition cost.
pub fn net_cost(posting: &TaxPosting) -> Decimal {
    let gross = posting.amount.abs();
    if posting.vat_mode == "contains_vat" && posting.vat_rate > Decimal::ZERO {
        gross / (Decimal::ONE + posting.vat_rate)
    } else {
        gross
    }
}

/// Straight-line annual depreciation for a single AfA posting in a given year.
///
/// The depreciation period is `posting.afa_years` full calendar years starting
/// from the month of purchase. Months in the purchase year count pro-rata.
///
/// Returns 0 if the year is outside the depreciation window.
pub fn depreciation_for_year(posting: &TaxPosting, year: i32) -> Decimal {
    if posting.afa_years <= 0 {
        return Decimal::ZERO;
    }

    let purchase = posting.posting_date;
    let total_months = posting.afa_years * 12;
    let cost = net_cost(posting);
    let monthly = cost / Decimal::from(total_months);

    // months from window start (purchase month) to the end of the depreciation window
    let window_end_month = purchase.month() as i32 - 1 + total_months;
    let window_end_year = purchase.year() + (window_end_month - 1) / 12;
    let window_end_cal_month = (window_end_month - 1) % 12 + 1;

    let year_start_month = if year > purchase.year() {
        1
    } else {
        purchase.month() as i32
    };
    let mut year_end_month = 12;

    // clip to depreciation window end
    if year == window_end_year {
        year_end_month = year_end_month.min(window_end_cal_month);
    } else if year > window_end_year {
        return Decimal::ZERO;
    }

    if year < purchase.year() {
        return Decimal::ZERO;
    }

    let active_months = year_end_month - year_start_month + 1;
    if active_months <= 0 {
        return Decimal::ZERO;
    }

    quantize(monthly * Decimal::from(active_months))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;
    use std::str::FromStr;

    fn afa_posting(
        date: &str,
        gross: &str,
        years: i32,
        vat_mode: &str,
        vat_rate: &str,
    ) -> TaxPosting {
        let mut p = posting(date, "assets:bank", "expenses:hardware:computer", gross);
        p.afa_years = years;
        p.vat_mode = vat_mode.to_string();
        p.vat_rate = Decimal::from_str(vat_rate).unwrap();
        p.tax_deduction = "afa".to_string();
        p
    }

    #[test]
    fn net_cost_strips_vat_when_contains_vat() {
        let p = afa_posting("2024-06-15", "1190.00", 3, "contains_vat", "0.19");
        assert_eq!(net_cost(&p), Decimal::from_str("1000").unwrap());
    }

    #[test]
    fn net_cost_is_gross_without_vat_mode() {
        let p = afa_posting("2024-06-15", "1000.00", 3, "", "0");
        assert_eq!(net_cost(&p), Decimal::from_str("1000.00").unwrap());
    }

    #[test]
    fn depreciation_pro_rates_purchase_year_and_final_year() {
        // 3-year AfA on 1000.00 net, purchased 2024-06 (month 6): 36 months total,
        // 7 months active in 2024 (Jun-Dec), 12 in 2025, 12 in 2026, 5 in 2027.
        let p = afa_posting("2024-06-15", "1000.00", 3, "", "0");
        let monthly = Decimal::from_str("1000").unwrap() / Decimal::from(36);

        assert_eq!(depreciation_for_year(&p, 2023), Decimal::ZERO);
        assert_eq!(
            depreciation_for_year(&p, 2024),
            (monthly * Decimal::from(7))
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        );
        assert_eq!(
            depreciation_for_year(&p, 2025),
            (monthly * Decimal::from(12))
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        );
        assert_eq!(
            depreciation_for_year(&p, 2026),
            (monthly * Decimal::from(12))
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        );
        assert_eq!(
            depreciation_for_year(&p, 2027),
            (monthly * Decimal::from(5))
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        );
        assert_eq!(depreciation_for_year(&p, 2028), Decimal::ZERO);
    }

    #[test]
    fn depreciation_for_full_calendar_year_purchase_matches_example_journal() {
        // examples/ledger/hledger.journal's computer purchase: net 1000.00 (1190.00
        // gross, 19% VAT), 3-year AfA, purchased in a year where test_euer_examples.py
        // asserts a 2024 AfA amount of 222.22 and 2025 of 333.33.
        let p = afa_posting("2024-05-01", "1190.00", 3, "contains_vat", "0.19");
        assert_eq!(
            depreciation_for_year(&p, 2024),
            Decimal::from_str("222.22").unwrap()
        );
        assert_eq!(
            depreciation_for_year(&p, 2025),
            Decimal::from_str("333.33").unwrap()
        );
    }

    #[test]
    fn depreciation_is_zero_when_afa_years_is_zero() {
        let p = afa_posting("2024-06-15", "1000.00", 0, "", "0");
        assert_eq!(depreciation_for_year(&p, 2024), Decimal::ZERO);
    }
}
