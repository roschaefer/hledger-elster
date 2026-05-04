from __future__ import annotations

from decimal import Decimal

from domain.posting import TaxPosting


ZERO = Decimal("0")

# Roles that are never Entnahmen/Einlagen regardless of source account.
_NON_DRAWING_ROLES = frozenset({
    "vat_payment", "vat_advance",
    "income_tax", "income_tax_advance", "income_tax_final", "tax_payment",
    "ignore",
})


def is_drawing(p: TaxPosting) -> bool:
    """True when a posting represents a private withdrawal from the business account.

    A drawing is any outflow from the business sphere that is not a business
    expense (EÜR) and not a tax payment — regardless of how the counter-account
    is categorised for other tax forms (e.g. ESt Vorsorgeaufwand).
    """
    return (
        p.source_is_business
        and p.amount > ZERO
        and p.tax_form != "einnahmenueberschussrechnung"
        and p.tax_role not in _NON_DRAWING_ROLES
    )


def is_contribution(p: TaxPosting) -> bool:
    """True when a posting represents a private deposit into the business account."""
    return (
        p.source_is_business
        and p.amount < ZERO
        and p.tax_form != "einnahmenueberschussrechnung"
        and p.tax_role not in _NON_DRAWING_ROLES
    )
