from __future__ import annotations

import os
from pathlib import Path

import pytest

from domain.dataset import TaxDataset
from ingest.enrich import build_dataset

TOOLS_ROOT = Path(__file__).resolve().parents[1]
EXAMPLE_LEDGER_JOURNAL = TOOLS_ROOT / "examples" / "ledger" / "hledger.journal"

_item_markers: dict[str, list] = {}


def ledger_journal_path() -> Path:
    configured = os.environ.get("FINANCES_TEST_LEDGER_JOURNAL")
    if configured:
        return Path(configured).resolve()
    return EXAMPLE_LEDGER_JOURNAL


@pytest.fixture(scope="session")
def dataset() -> TaxDataset:
    return build_dataset(ledger_journal_path())


def pytest_collection_finish(session: pytest.Session) -> None:
    for item in session.items:
        _item_markers[item.nodeid] = item.own_markers


def pytest_configure(config: pytest.Config) -> None:
    config.addinivalue_line(
        "markers",
        "confirmed(previously, reason): deviation resolved — prior filing was incorrect",
    )
    config.addinivalue_line(
        "markers",
        "needs_review(previously, reason): deviation under investigation — does not fail the build",
    )


def pytest_terminal_summary(
    terminalreporter: pytest.TerminalReporter,
    exitstatus: int,
    config: pytest.Config,
) -> None:
    passed = terminalreporter.stats.get("passed", [])

    confirmed: list[tuple] = []
    under_review: list[tuple] = []
    for report in passed:
        for marker in _item_markers.get(report.nodeid, []):
            if marker.name == "confirmed":
                confirmed.append((report, marker))
            elif marker.name == "needs_review":
                under_review.append((report, marker))

    if confirmed:
        terminalreporter.write_sep("=", "CONFIRMED DIFFERENCES", green=True)
        for report, marker in confirmed:
            terminalreporter.write_line(f"  {report.nodeid}")
            terminalreporter.write_line(
                f"    previously: {marker.kwargs['previously']}  reason: {marker.kwargs['reason']}"
            )

    if under_review:
        terminalreporter.write_sep("=", "NEEDS REVIEW", yellow=True)
        for report, marker in under_review:
            terminalreporter.write_line(f"  {report.nodeid}")
            terminalreporter.write_line(
                f"    previously: {marker.kwargs['previously']}  reason: {marker.kwargs['reason']}"
            )
