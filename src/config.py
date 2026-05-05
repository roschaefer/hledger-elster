from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from decimal import Decimal
from pathlib import Path
from typing import Any

DEFAULT_CONFIG_TEXT = """[euer.home_office_pauschale]
enabled = true
default_days = "max"
# Set per-year days when the default does not match your situation.
# 2020-2022: 5 EUR/day, capped at 600 EUR.
# 2023+: 6 EUR/day, capped at 1260 EUR.

[euer.home_office_pauschale.days]
# 2024 = 210
"""

ZERO = Decimal("0.00")
TWOPLACES = Decimal("0.01")


@dataclass(frozen=True)
class HomeOfficePauschaleConfig:
    enabled: bool = True
    default_days: int | str = "max"
    days_by_year: dict[int, int] = field(default_factory=dict)

    def amount_for_year(self, year: int) -> Decimal:
        if not self.enabled:
            return ZERO

        policy = _home_office_policy(year)
        if policy is None:
            return ZERO

        rate, cap, _max_days = policy
        configured_days = self.days_by_year.get(year)
        if configured_days is None:
            if self.default_days == "max":
                return cap
            configured_days = int(self.default_days)

        amount = Decimal(configured_days) * rate
        return min(amount, cap).quantize(TWOPLACES)


@dataclass(frozen=True)
class TaxConfig:
    home_office_pauschale: HomeOfficePauschaleConfig = field(default_factory=HomeOfficePauschaleConfig)


def load_config(path: Path | None) -> TaxConfig:
    if path is None:
        return TaxConfig()
    if not path.exists():
        raise FileNotFoundError(f"Config not found: {path}")
    with path.open("rb") as fh:
        raw = tomllib.load(fh)
    return _parse_config(raw)


def write_default_config(path: Path, force: bool = False) -> None:
    if path.exists() and not force:
        raise FileExistsError(f"Config already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(DEFAULT_CONFIG_TEXT, encoding="utf-8")


def _parse_config(raw: dict[str, Any]) -> TaxConfig:
    home_office_raw = raw.get("euer", {}).get("home_office_pauschale", {})
    if not isinstance(home_office_raw, dict):
        raise ValueError("Config key euer.home_office_pauschale must be a table")

    enabled = home_office_raw.get("enabled", True)
    if not isinstance(enabled, bool):
        raise ValueError("Config key euer.home_office_pauschale.enabled must be a boolean")

    default_days = home_office_raw.get("default_days", "max")
    if default_days != "max" and not isinstance(default_days, int):
        raise ValueError('Config key euer.home_office_pauschale.default_days must be "max" or an integer')
    if isinstance(default_days, int) and default_days < 0:
        raise ValueError("Config key euer.home_office_pauschale.default_days must not be negative")

    days_raw = home_office_raw.get("days", {})
    if not isinstance(days_raw, dict):
        raise ValueError("Config key euer.home_office_pauschale.days must be a table")

    days_by_year: dict[int, int] = {}
    for year_raw, days in days_raw.items():
        if not isinstance(days, int):
            raise ValueError(f"Home-office days for {year_raw} must be an integer")
        if days < 0:
            raise ValueError(f"Home-office days for {year_raw} must not be negative")
        days_by_year[int(year_raw)] = days

    return TaxConfig(
        home_office_pauschale=HomeOfficePauschaleConfig(
            enabled=enabled,
            default_days=default_days,
            days_by_year=days_by_year,
        )
    )


def _home_office_policy(year: int) -> tuple[Decimal, Decimal, int] | None:
    if 2020 <= year <= 2022:
        return Decimal("5.00"), Decimal("600.00"), 120
    if year >= 2023:
        return Decimal("6.00"), Decimal("1260.00"), 210
    return None
