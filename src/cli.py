from __future__ import annotations

import argparse
import os
from pathlib import Path

from calculate.generate_report import main as generate_report
from paths import ledger_journal_path, tax_data_dir


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="hledger-elster",
        description="Generate ELSTER-oriented tax exports from an hledger journal.",
    )
    parser.add_argument(
        "-f",
        "--file",
        dest="journal",
        default=None,
        help=f"hledger journal to read (default: {ledger_journal_path()})",
    )
    parser.add_argument(
        "-o",
        "--output-dir",
        dest="output_dir",
        default=None,
        help=f"directory for tax exports (default: {tax_data_dir()})",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.journal:
        os.environ["FINANCES_LEDGER_JOURNAL"] = str(Path(args.journal).resolve())
    if args.output_dir:
        os.environ["FINANCES_TAX_DATA_DIR"] = str(Path(args.output_dir).resolve())
    return generate_report()


if __name__ == "__main__":
    raise SystemExit(main())
