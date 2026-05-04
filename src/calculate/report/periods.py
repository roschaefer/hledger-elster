from __future__ import annotations

from collections.abc import Callable
from decimal import Decimal

from domain.dataset import TaxDataset

TWOPLACES = Decimal("0.01")


def period_labels(year: int) -> list[str]:
    labels = [str(year)]
    for q in range(1, 5):
        labels.append(f"{year} Q{q}")
    for m in range(1, 13):
        labels.append(f"{year}-{m:02d}")
    return labels


def annual_labels(year: int) -> list[str]:
    return [str(year)]


def _quarter(month: int) -> int:
    return (month - 1) // 3 + 1


def filter_period(dataset: TaxDataset, year: int, label: str) -> TaxDataset:
    if label == str(year):
        return dataset.for_year(year)
    if "Q" in label:
        q = int(label.split("Q")[1])
        return dataset.for_quarter(year, q)
    m = int(label.split("-")[1])
    return dataset.for_month(year, m)


def aggregate_periods(
    dataset: TaxDataset,
    year: int,
    fn: Callable[[TaxDataset], Decimal],
    labels: list[str] | None = None,
) -> dict[str, Decimal]:
    if labels is None:
        labels = period_labels(year)
    return {label: fn(filter_period(dataset, year, label)) for label in labels}


def fmt(value: Decimal) -> str:
    return f"{value:.2f}"


def blank_row(labels: list[str]) -> dict[str, str]:
    return {"Kennzahl": "", **{label: "" for label in labels}}


def section_row(name: str, labels: list[str]) -> dict[str, str]:
    """Section header row — Kennzahl prefixed with '# ' so the writer can style it."""
    return {"Kennzahl": f"# {name}", **{label: "" for label in labels}}
