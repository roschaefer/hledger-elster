from __future__ import annotations

from decimal import Decimal, ROUND_HALF_UP
from datetime import date

from domain.posting import TaxPosting


TWOPLACES = Decimal("0.01")


def _quantize(value: Decimal) -> Decimal:
    return value.quantize(TWOPLACES, rounding=ROUND_HALF_UP)


def net_cost(posting: TaxPosting) -> Decimal:
    """Net acquisition cost (gross / (1 + vat_rate))."""
    gross = abs(posting.amount)
    if posting.vat_rate > Decimal("0"):
        return gross / (Decimal("1") + posting.vat_rate)
    return gross


def depreciation_for_year(posting: TaxPosting, year: int) -> Decimal:
    """
    Straight-line annual depreciation for a single AfA posting in a given year.

    The depreciation period is posting.afa_years full calendar years starting
    from the month of purchase. Months in the purchase year count pro-rata.

    Returns 0 if the year is outside the depreciation window.
    """
    if posting.afa_years <= 0:
        return Decimal("0")

    purchase = posting.posting_date
    total_months = posting.afa_years * 12
    cost = net_cost(posting)
    monthly = cost / Decimal(total_months)

    # months in `year` that fall within the depreciation window
    window_start = date(purchase.year, purchase.month, 1)
    window_end_month = purchase.month - 1 + total_months  # total months from window start
    window_end_year = purchase.year + (window_end_month - 1) // 12
    window_end_cal_month = ((window_end_month - 1) % 12) + 1

    year_start_month = 1 if year > purchase.year else purchase.month
    year_end_month = 12

    # clip to depreciation window end
    if year == window_end_year:
        year_end_month = min(year_end_month, window_end_cal_month)
    elif year > window_end_year:
        return Decimal("0")

    if year < purchase.year:
        return Decimal("0")

    active_months = year_end_month - year_start_month + 1
    if active_months <= 0:
        return Decimal("0")

    return _quantize(monthly * active_months)
