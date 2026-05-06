from __future__ import annotations

from decimal import Decimal

from domain.dataset import TaxDataset
from domain.posting import TaxPosting

ZERO = Decimal("0.00")
EUER_FORM = "einnahmenueberschussrechnung"
EXPENSE_DEDUCTIONS = {"full", "proportional", "non_deductible", "afa"}


def _account_has_prefix(account: str, prefix: str) -> bool:
    normalized = account.casefold()
    return normalized == prefix or normalized.startswith(f"{prefix}:")


def is_euer_expense(p: TaxPosting) -> bool:
    if p.tax_form != EUER_FORM:
        return False
    if _account_has_prefix(p.counter_account, "income"):
        return False
    return _account_has_prefix(p.counter_account, "expenses") or p.tax_deduction in EXPENSE_DEDUCTIONS


def is_euer_income(p: TaxPosting) -> bool:
    if p.tax_form != EUER_FORM:
        return False
    if _account_has_prefix(p.counter_account, "income"):
        return True
    if is_euer_expense(p):
        return False
    return p.amount < ZERO


def euer_expenses(dataset: TaxDataset) -> TaxDataset:
    return TaxDataset([p for p in dataset if is_euer_expense(p)])


def euer_income(dataset: TaxDataset) -> TaxDataset:
    return TaxDataset([p for p in dataset if is_euer_income(p)])
