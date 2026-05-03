from __future__ import annotations

from collections import defaultdict
from typing import Iterator

from .posting import TaxPosting


class TaxDataset:
    def __init__(self, postings: list[TaxPosting]) -> None:
        self._postings = postings

    def __iter__(self) -> Iterator[TaxPosting]:
        return iter(self._postings)

    def __len__(self) -> int:
        return len(self._postings)

    def __repr__(self) -> str:
        return f"TaxDataset({len(self._postings)} postings)"

    # ── filters ────────────────────────────────────────────────────────────

    def for_form(self, form: str) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.tax_form == form])

    def for_role(self, role: str) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.tax_role == role])

    def for_deduction(self, deduction: str) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.tax_deduction == deduction])

    def exclude_deduction(self, deduction: str) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.tax_deduction != deduction])

    def for_account_prefix(self, prefix: str) -> TaxDataset:
        return TaxDataset(
            [p for p in self._postings if p.counter_account == prefix or p.counter_account.startswith(f"{prefix}:")]
        )

    def for_source_account(self, account: str) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.source_account == account])

    def for_year(self, year: int) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.year == year])

    def for_quarter(self, year: int, quarter: int) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.year == year and p.quarter == quarter])

    def for_month(self, year: int, month: int) -> TaxDataset:
        return TaxDataset([p for p in self._postings if p.year == year and p.month == month])

    # ── grouping ───────────────────────────────────────────────────────────

    def group_by_counter_account(self) -> dict[str, TaxDataset]:
        buckets: dict[str, list[TaxPosting]] = defaultdict(list)
        for p in self._postings:
            buckets[p.counter_account].append(p)
        return {account: TaxDataset(postings) for account, postings in buckets.items()}

    def group_by_source_account(self) -> dict[str, TaxDataset]:
        buckets: dict[str, list[TaxPosting]] = defaultdict(list)
        for p in self._postings:
            buckets[p.source_account].append(p)
        return {account: TaxDataset(postings) for account, postings in buckets.items()}

    def group_by_year(self) -> dict[int, TaxDataset]:
        buckets: dict[int, list[TaxPosting]] = defaultdict(list)
        for p in self._postings:
            buckets[p.year].append(p)
        return {year: TaxDataset(postings) for year, postings in sorted(buckets.items())}
