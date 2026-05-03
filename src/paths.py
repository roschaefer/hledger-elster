from __future__ import annotations

import os
from pathlib import Path


TOOLS_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_LEDGER_JOURNAL = TOOLS_ROOT / "examples" / "ledger" / "hledger.journal"
DEFAULT_TAX_DATA_DIR = TOOLS_ROOT / "data" / "exports"


def env_path(name: str, default: Path) -> Path:
    value = os.environ.get(name)
    return Path(value).resolve() if value else default


def ledger_journal_path() -> Path:
    return env_path("FINANCES_LEDGER_JOURNAL", DEFAULT_LEDGER_JOURNAL)


def tax_data_dir() -> Path:
    return env_path("FINANCES_TAX_DATA_DIR", DEFAULT_TAX_DATA_DIR)
