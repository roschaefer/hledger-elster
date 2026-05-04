from __future__ import annotations

import csv
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
EXAMPLES_DIR = PROJECT_ROOT / "tests" / "examples" / "e2e"


def _read_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


class EndToEndExampleTest(unittest.TestCase):
    def run_fixture(self, fixture_name: str, *, extra_file: str | None = None) -> tuple[Path, subprocess.CompletedProcess[str]]:
        fixture = EXAMPLES_DIR / fixture_name
        out_dir = Path(tempfile.mkdtemp(prefix=f"hledger-elster-{fixture_name}-"))
        if extra_file is not None:
            stale_path = out_dir / extra_file
            stale_path.parent.mkdir(parents=True, exist_ok=True)
            stale_path.write_text("stale\n", encoding="utf-8")

        env = os.environ.copy()
        env["PYTHONPATH"] = str(PROJECT_ROOT / "src")
        env["FINANCES_LEDGER_JOURNAL"] = str(fixture / "journal.journal")
        env["FINANCES_TAX_DATA_DIR"] = str(out_dir)

        result = subprocess.run(
            [
                sys.executable,
                str(PROJECT_ROOT / "src" / "calculate" / "generate_report.py"),
            ],
            cwd=PROJECT_ROOT,
            env=env,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return out_dir, result

    def test_health_care_split_fixture_exports_all_est_rows(self) -> None:
        out_dir, _ = self.run_fixture("health-care-split")
        rows = _read_rows(out_dir / "2024" / "steuererklaerung" / "einkommensteuer.csv")

        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Krankenversicherung")["2024"], "840.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Pflegeversicherung")["2024"], "240.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Zusatzbeitrag")["2024"], "120.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Langzeit-Auslandskrankenversicherung")["2024"], "343.50")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Kurzzeit-Auslandskrankenversicherung")["2024"], "9.50")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Haftpflichtversicherung")["2024"], "57.88")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Summe privat gezahlt")["2024"], "1610.88")

    def test_donations_fixture_exports_user_defined_est_section(self) -> None:
        out_dir, _ = self.run_fixture("donations")
        rows = _read_rows(out_dir / "2024" / "steuererklaerung" / "einkommensteuer.csv")

        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "# Sonderausgaben")["2024"], "")
        self.assertEqual([row["Kennzahl"] for row in rows].count("Spenden"), 1)
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Spenden")["2024"], "75.00")
        self.assertEqual(
            next(row for row in rows if row["Kennzahl"] == "Parteispende (§34g/§10b manuell berechnen)")["2024"],
            "MANUAL",
        )
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Summe privat gezahlt")["2024"], "75.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Abziehbar (Netto)")["2024"], "75.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Summe abziehbar")["2024"], "75.00")

        trail_rows = _read_rows(out_dir / "2024" / "herleitung" / "einkommensteuer" / "spenden.csv")
        self.assertEqual(
            next(row for row in trail_rows if row["Konto"] == "Girokonto" and row["Beschreibung"] == "Example charity donation")["Betrag"],
            "50.00",
        )
        self.assertEqual(
            next(row for row in trail_rows if row["Konto"] == "Girokonto" and row["Beschreibung"] == "Another charity donation")["Betrag"],
            "25.00",
        )

        manual_trail_rows = _read_rows(
            out_dir / "2024" / "herleitung" / "einkommensteuer" / "parteispende-(§34g-§10b-manuell.csv"
        )
        self.assertEqual(
            next(
                row for row in manual_trail_rows
                if row["Konto"] == "Girokonto" and row["Beschreibung"] == "Example political party donation"
            )["Betrag"],
            "100.00",
        )

    def test_business_account_fallback_fixture_exports_drawings_and_contributions(self) -> None:
        out_dir, _ = self.run_fixture("business-account-fallback")
        rows = _read_rows(out_dir / "2024" / "steuererklaerung" / "einnahmen-ueberschuss-rechnung.csv")

        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Umsatzsteuerpflichtige Betriebseinnahmen")["2024"], "100.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Vereinnahmte Umsatzsteuer")["2024"], "19.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Entnahmen")["2024"], "125.00")
        self.assertEqual(next(row for row in rows if row["Kennzahl"] == "Einlagen")["2024"], "40.00")

    def test_vat_reversal_fixture_nets_out_paid_vat(self) -> None:
        out_dir, _ = self.run_fixture("vat-reversal")
        rows = _read_rows(out_dir / "2024" / "steuererklaerung" / "umsatzsteuer.csv")

        self.assertEqual(next(row for row in rows if row["Zeitraum"] == "2024")["Bereits Entrichtet"], "60.00")

    def test_vat_year_vs_period_fixture_distinguishes_euer_from_ust(self) -> None:
        out_dir, _ = self.run_fixture("vat-year-vs-period")

        euer_2024 = _read_rows(out_dir / "2024" / "steuererklaerung" / "einnahmen-ueberschuss-rechnung.csv")
        euer_2025 = _read_rows(out_dir / "2025" / "steuererklaerung" / "einnahmen-ueberschuss-rechnung.csv")
        ust_2024 = _read_rows(out_dir / "2024" / "steuererklaerung" / "umsatzsteuer.csv")
        ust_2025 = _read_rows(out_dir / "2025" / "steuererklaerung" / "umsatzsteuer.csv")

        self.assertEqual(
            next(
                row for row in euer_2024
                if row["Kennzahl"] == "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer"
            )["2024"],
            "25.00",
        )
        self.assertEqual(
            next(
                row for row in euer_2024
                if row["Kennzahl"] == "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer"
            )["2024"],
            "100.00",
        )
        self.assertEqual(
            next(
                row for row in euer_2025
                if row["Kennzahl"] == "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer"
            )["2025"],
            "0.00",
        )
        self.assertEqual(
            next(
                row for row in euer_2025
                if row["Kennzahl"] == "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer"
            )["2025"],
            "310.00",
        )

        self.assertEqual(next(row for row in ust_2024 if row["Zeitraum"] == "2024")["Bereits Entrichtet"], "90.00")
        self.assertEqual(next(row for row in ust_2025 if row["Zeitraum"] == "2025")["Bereits Entrichtet"], "150.00")

    def test_warns_about_untouched_files_in_export_directory(self) -> None:
        _, result = self.run_fixture("health-care-split", extra_file="2024/steuererklaerung/stale.csv")

        self.assertIn(
            "Warning: untouched files remain in ELSTER export directory. "
            "Consider emptying the export directory and running the tool again.",
            result.stderr,
        )
        self.assertIn("stale.csv", result.stderr)
        self.assertIn("\nWarning:", result.stderr)
        self.assertTrue(result.stderr.endswith("\n\n"), result.stderr)


if __name__ == "__main__":
    unittest.main()
