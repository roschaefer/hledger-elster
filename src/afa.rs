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

/// One year's write-off of an asset: the straight-line AfA, and the remaining
/// book value when the asset leaves the business that year.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct YearAmounts {
    pub afa: Decimal,
    pub residual: Decimal,
}

impl YearAmounts {
    pub fn total(&self) -> Decimal {
        self.afa + self.residual
    }
}

fn month_index(year: i32, month: u32) -> i32 {
    year * 12 + month as i32 - 1
}

/// Straight-line depreciation of a single AfA posting in a given year.
///
/// The depreciation period is `posting.afa_years` years starting from the
/// month of purchase; months in the purchase year count pro-rata. Each year is
/// rounded on its own and the last year takes the remainder, so the years add
/// up to the rounded net cost. A disposal stops the AfA after its month and
/// writes off the remaining book value in that year.
pub fn for_year(posting: &TaxPosting, year: i32) -> YearAmounts {
    if posting.afa_years <= 0 {
        return YearAmounts::default();
    }
    let purchase = month_index(posting.posting_date.year(), posting.posting_date.month());
    let total_months = posting.afa_years * 12;
    let window_end = purchase + total_months - 1;
    let disposal = posting
        .afa_disposal
        .map(|d| month_index(d.year(), d.month()))
        .filter(|&d| d < window_end);
    let last = disposal.unwrap_or(window_end);
    let cost = quantize(net_cost(posting));

    let regular = |y: i32| -> Decimal {
        let first = purchase.max(y * 12);
        let end = last.min(y * 12 + 11);
        if end < first {
            return Decimal::ZERO;
        }
        quantize(net_cost(posting) * Decimal::from(end - first + 1) / Decimal::from(total_months))
    };
    let first_year = posting.posting_date.year();
    let last_year = last.div_euclid(12);
    if year < first_year || year > last_year {
        return YearAmounts::default();
    }
    let earlier: Decimal = (first_year..year).map(regular).sum();
    if disposal.is_some() {
        let afa = regular(year);
        let residual = if year == last_year {
            cost - earlier - afa
        } else {
            Decimal::ZERO
        };
        YearAmounts { afa, residual }
    } else if year == last_year {
        YearAmounts {
            afa: cost - earlier,
            residual: Decimal::ZERO,
        }
    } else {
        YearAmounts {
            afa: regular(year),
            residual: Decimal::ZERO,
        }
    }
}

/// The years in which a posting writes off anything.
pub fn years(posting: &TaxPosting) -> impl Iterator<Item = i32> + '_ {
    let first = posting.posting_date.year();
    (first..=first + posting.afa_years.max(0))
        .filter(move |&y| for_year(posting, y) != YearAmounts::default())
}

/// The deductible part of `for_year`, by the posting's expense share.
///
/// Shares are taken of the running totals, so the deductible years still add
/// up to the deductible share of the net cost.
pub fn deductible_for_year(posting: &TaxPosting, year: i32) -> YearAmounts {
    let share = |v: Decimal| quantize(v * posting.expense_share);
    let first_year = posting.posting_date.year();
    let before: Decimal = (first_year..year)
        .map(|y| for_year(posting, y).total())
        .sum();
    let this = for_year(posting, year);
    YearAmounts {
        afa: share(before + this.afa) - share(before),
        residual: share(before + this.total()) - share(before + this.afa),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::posting::test_support::posting;
    use chrono::NaiveDate;
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
        p.expense_share = Decimal::ONE;
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

    fn afa(afa: &str, residual: &str) -> YearAmounts {
        YearAmounts {
            afa: Decimal::from_str(afa).unwrap(),
            residual: Decimal::from_str(residual).unwrap(),
        }
    }

    #[test]
    fn pro_rates_the_purchase_year_and_puts_the_rounding_remainder_in_the_last_year() {
        // 1000.00 / 36 months = 27.777...: Jun-Dec 2024 is 7 months, 2027 is 5.
        let p = afa_posting("2024-06-15", "1000.00", 3, "", "0");
        assert_eq!(for_year(&p, 2023), YearAmounts::default());
        assert_eq!(for_year(&p, 2024), afa("194.44", "0"));
        assert_eq!(for_year(&p, 2025), afa("333.33", "0"));
        assert_eq!(for_year(&p, 2026), afa("333.33", "0"));
        assert_eq!(for_year(&p, 2027), afa("138.90", "0"));
        assert_eq!(for_year(&p, 2028), YearAmounts::default());
    }

    #[test]
    fn years_add_up_to_the_rounded_net_cost() {
        // net 966.6386... like a 1150.30 EUR laptop with 19% VAT, bought in December.
        let p = afa_posting("2020-12-30", "1150.30", 3, "contains_vat", "0.19");
        let total: Decimal = (2020..=2023).map(|y| for_year(&p, y).total()).sum();
        assert_eq!(total, Decimal::from_str("966.64").unwrap());
        assert_eq!(for_year(&p, 2023), afa("295.37", "0"));
    }

    #[test]
    fn matches_the_example_journal_computer() {
        let p = afa_posting("2024-05-01", "1190.00", 3, "contains_vat", "0.19");
        assert_eq!(for_year(&p, 2024), afa("222.22", "0"));
        assert_eq!(for_year(&p, 2025), afa("333.33", "0"));
    }

    #[test]
    fn a_disposal_stops_afa_after_its_month_and_writes_off_the_rest() {
        let mut p = afa_posting("2024-01-10", "3600.00", 3, "", "0");
        p.afa_disposal = Some(NaiveDate::from_ymd_opt(2025, 3, 20).unwrap());
        assert_eq!(for_year(&p, 2024), afa("1200.00", "0"));
        assert_eq!(for_year(&p, 2025), afa("300.00", "2100.00"));
        assert_eq!(for_year(&p, 2026), YearAmounts::default());
    }

    #[test]
    fn a_disposal_after_the_last_month_changes_nothing() {
        let mut p = afa_posting("2024-01-10", "3600.00", 1, "", "0");
        p.afa_disposal = Some(NaiveDate::from_ymd_opt(2024, 12, 5).unwrap());
        assert_eq!(for_year(&p, 2024), afa("3600.00", "0"));
    }

    #[test]
    fn deductible_years_add_up_to_the_share_of_the_net_cost() {
        let mut p = afa_posting("2024-06-15", "1000.00", 3, "", "0");
        p.expense_share = Decimal::from_str("0.333").unwrap();
        let total: Decimal = (2024..=2027)
            .map(|y| deductible_for_year(&p, y).total())
            .sum();
        assert_eq!(total, Decimal::from_str("333.00").unwrap());
    }

    #[test]
    fn deductible_splits_the_share_between_afa_and_residual() {
        let mut p = afa_posting("2024-01-10", "3600.00", 3, "", "0");
        p.expense_share = Decimal::from_str("0.5").unwrap();
        p.afa_disposal = Some(NaiveDate::from_ymd_opt(2025, 3, 20).unwrap());
        assert_eq!(deductible_for_year(&p, 2025), afa("150.00", "1050.00"));
    }

    #[test]
    fn is_zero_when_afa_years_is_zero() {
        let p = afa_posting("2024-06-15", "1000.00", 0, "", "0");
        assert_eq!(for_year(&p, 2024), YearAmounts::default());
    }
}
