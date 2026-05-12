from __future__ import annotations

from dataclasses import dataclass
from datetime import date
from decimal import Decimal


@dataclass(frozen=True)
class TaxPosting:
    # Bookkeeping dimensions
    posting_date: date
    source_account: str  # e.g. "assets:dkb:girokonto"
    counter_account: str  # e.g. "expenses:business:hosting:hetzner"
    amount: Decimal  # positive = expense, negative = income (hledger sign convention)
    description: str
    transaction_comment: str
    posting_comment: str
    source_file: str
    source_line: int

    # Tax enrichment — resolved from account directives at ingest time
    tax_form: str  # "einnahmenueberschussrechnung" | "einkommensteuer" | ""
    tax_deduction: str  # "full" | "proportional" | "non_deductible" | "afa" | ""
    tax_role: str  # "tax_payment" | "income_tax" | "vat_payment" | "vat_advance" | "income_tax_advance" | "income_tax_final" | "drawing" | "contribution" | "ignore" | ""
    calculation: str  # "" | "manual"
    vat_mode: str  # "contains_vat" | "reverse_charge_eu" | "reverse_charge_non_eu" | "not_applicable" | ""
    vat_rate: Decimal  # 0.19 / 0.07 / 0.00
    expense_share: Decimal
    input_vat_share: Decimal

    # AfA metadata — only set when tax_deduction == "afa"
    afa_years: int  # 0 if not AfA

    # Synthetic rows
    derived_kind: str  # "" or "abschreibung"

    # Human-readable metadata resolved from account directives
    label: str = ""  # Report item from elster_item tag on counter account
    source_label: str = ""  # Report item from elster_item tag on source account
    section: str = ""  # Section from elster_section tag (EÜR, ESt, ...)
    tax_period: str = ""  # Fiscal period this posting belongs to (from elster_period tag)
    tax_period_year: int = 0  # Fiscal year from elster_period; 0 = use transaction year
    source_is_business: bool = False  # True when source account carries elster_account:business

    @property
    def year(self) -> int:
        return self.posting_date.year

    @property
    def quarter(self) -> int:
        return ((self.posting_date.month - 1) // 3) + 1

    @property
    def month(self) -> int:
        return self.posting_date.month
