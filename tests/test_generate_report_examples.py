from __future__ import annotations

import csv
from pathlib import Path

import openpyxl

from calculate.generate_report import main


def _read_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


def _rgb(color) -> str:
    return color.rgb[-6:]


def test_generate_report_writes_expected_example_outputs(monkeypatch, tmp_path: Path) -> None:
    tools_root = Path(__file__).resolve().parents[1]
    monkeypatch.setenv("FINANCES_LEDGER_JOURNAL", str(tools_root / "examples" / "ledger" / "hledger.journal"))
    monkeypatch.setenv("FINANCES_TAX_DATA_DIR", str(tmp_path))

    assert main() == 0

    euer_2024 = _read_rows(tmp_path / "2024" / "steuererklaerung" / "einnahmen-ueberschuss-rechnung.csv")
    ust_2024 = _read_rows(tmp_path / "2024" / "steuererklaerung" / "umsatzsteuer.csv")
    est_2024 = _read_rows(tmp_path / "2024" / "steuererklaerung" / "einkommensteuer.csv")
    est_2025 = _read_rows(tmp_path / "2025" / "steuererklaerung" / "einkommensteuer.csv")

    assert (
        next(row for row in euer_2024 if row["Kennzahl"] == "Umsatzsteuerpflichtige Betriebseinnahmen")["2024"]
        == "1000.00"
    )
    assert next(row for row in euer_2024 if row["Kennzahl"] == "Vereinnahmte Umsatzsteuer")["2024"] == "190.00"
    assert next(row for row in euer_2024 if row["Kennzahl"] == "Steuerpflichtiger Gewinn/Verlust")["2024"] == "-824.22"
    assert next(row for row in est_2024 if row["Kennzahl"] == "Krankenversicherung")["2024"] == "840.00"
    assert next(row for row in est_2024 if row["Kennzahl"] == "Pflegeversicherung")["2024"] == "240.00"
    assert next(row for row in est_2024 if row["Kennzahl"] == "Zusatzbeitrag")["2024"] == "120.00"
    assert (
        next(row for row in est_2024 if row["Kennzahl"] == "Langzeit-Auslandskrankenversicherung")["2024"] == "343.50"
    )
    assert next(row for row in est_2024 if row["Kennzahl"] == "Kurzzeit-Auslandskrankenversicherung")["2024"] == "9.50"
    assert next(row for row in est_2024 if row["Kennzahl"] == "Haftpflichtversicherung")["2024"] == "57.88"
    assert next(row for row in est_2024 if row["Kennzahl"] == "Summe privat gezahlt")["2024"] == "1610.88"
    assert next(row for row in est_2025 if row["Kennzahl"] == "Krankenversicherung")["2025"] == "910.00"
    assert next(row for row in est_2025 if row["Kennzahl"] == "Pflegeversicherung")["2025"] == "260.00"
    assert next(row for row in est_2025 if row["Kennzahl"] == "Zusatzbeitrag")["2025"] == "130.00"
    assert next(row for row in est_2025 if row["Kennzahl"] == "ESt-Abschlusszahlung")["2025"] == "50.00"
    assert next(row for row in ust_2024 if row["Zeitraum"] == "2024")["Bereits Entrichtet"] == "190.00"

    workbook = openpyxl.load_workbook(tmp_path / "2024" / "steuererklaerung.xlsx")
    euer_sheet = workbook["EÜR"]
    assert _rgb(euer_sheet["A1"].font.color) == "000000"
    assert _rgb(euer_sheet["A1"].fill.fgColor) == "D9E1F2"
    assert _rgb(euer_sheet["A2"].font.color) == "000000"
    assert _rgb(euer_sheet["A2"].fill.fgColor) == "E9EFF7"
    assert _rgb(euer_sheet["A3"].font.color) == "000000"
    assert _rgb(euer_sheet["A3"].fill.fgColor) == "FFFFFF"
