from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

from calculate.generate_report import main as generate_report
from config import write_default_config
from paths import ledger_journal_path, tax_data_dir


def build_generate_parser() -> argparse.ArgumentParser:
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
    parser.add_argument(
        "--config",
        dest="config",
        default=None,
        help="TOML config file for user-specific tax adjustments",
    )
    return parser


def build_init_config_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="hledger-elster init-config",
        description="Write a default hledger-elster TOML config file.",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="path to write, for example elster.toml",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="overwrite an existing config file",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    cli_args = list(argv) if argv is not None else sys.argv[1:]
    if cli_args and cli_args[:1] == ["init-config"]:
        parser = build_init_config_parser()
        args = parser.parse_args(cli_args[1:])
        write_default_config(Path(args.output).resolve(), force=args.force)
        return 0

    parser = build_generate_parser()
    args = parser.parse_args(cli_args)
    if args.journal:
        os.environ["FINANCES_LEDGER_JOURNAL"] = str(Path(args.journal).resolve())
    if args.output_dir:
        os.environ["FINANCES_TAX_DATA_DIR"] = str(Path(args.output_dir).resolve())
    if args.config:
        os.environ["HLEDGER_ELSTER_CONFIG"] = str(Path(args.config).resolve())
    return generate_report()


if __name__ == "__main__":
    raise SystemExit(main())
