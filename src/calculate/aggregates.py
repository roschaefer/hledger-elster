from __future__ import annotations

from decimal import ROUND_HALF_UP, Decimal

from domain.dataset import TaxDataset
from domain.posting import TaxPosting

TWOPLACES = Decimal("0.01")


def _quantize(value: Decimal) -> Decimal:
    return value.quantize(TWOPLACES, rounding=ROUND_HALF_UP)


def _net(posting: TaxPosting) -> Decimal:
    """Net amount for a single posting (gross / (1 + vat_rate))."""
    gross = abs(posting.amount)
    if posting.vat_rate > Decimal("0"):
        return gross / (Decimal("1") + posting.vat_rate)
    return gross


def _vat(posting: TaxPosting) -> Decimal:
    """VAT amount for a single posting."""
    gross = abs(posting.amount)
    net = _net(posting)
    return gross - net


def gross_amount(dataset: TaxDataset) -> Decimal:
    """Sum of absolute gross amounts across all postings in the dataset."""
    return _quantize(sum((abs(p.amount) for p in dataset), start=Decimal("0")))


def signed_total(dataset: TaxDataset) -> Decimal:
    """Sum of raw amounts preserving sign — for expense accounts where refunds should reduce the total."""
    return _quantize(sum((p.amount for p in dataset), start=Decimal("0")))


def net_amount(dataset: TaxDataset) -> Decimal:
    """Sum of net amounts (gross / (1 + vat_rate)) across all postings."""
    return _quantize(sum((_net(p) for p in dataset), start=Decimal("0")))


def _signed_net(posting: TaxPosting) -> Decimal:
    """Net amount preserving sign — for expense aggregations where refunds should reduce the total."""
    if posting.vat_rate > Decimal("0"):
        return posting.amount / (Decimal("1") + posting.vat_rate)
    return posting.amount


def _signed_vat(posting: TaxPosting) -> Decimal:
    return posting.amount - _signed_net(posting)


def deductible_net(dataset: TaxDataset) -> Decimal:
    """
    Sum of deductible net amounts.

    For each posting: net * expense_share, rounded per posting, then summed.
    Sign is preserved so refunds correctly reduce the total.
    """
    total = Decimal("0")
    for p in dataset:
        total += _quantize(_signed_net(p) * p.expense_share)
    return _quantize(total)


def deductible_vat(dataset: TaxDataset) -> Decimal:
    """
    Sum of deductible input VAT amounts.

    For each posting: vat * vat_share, rounded per posting, then summed.
    Sign is preserved so VAT refunds correctly reduce the total.
    """
    total = Decimal("0")
    for p in dataset:
        total += _quantize(_signed_vat(p) * p.vat_share)
    return _quantize(total)


def collected_vat(dataset: TaxDataset) -> Decimal:
    """
    VAT collected on income postings (gross - net per posting, summed).

    Use with a dataset filtered to income postings.
    """
    total = Decimal("0")
    for p in dataset:
        total += _quantize(_vat(p))
    return _quantize(total)
