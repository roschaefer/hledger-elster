from __future__ import annotations

from pathlib import Path

from calculate.report.euer import euer_rows
from calculate.report.herleitung import herleitung_sheets
from config import TaxConfig
from ingest.enrich import build_dataset


def test_business_account_fallback_classifies_unmapped_postings_as_drawings_and_contributions(tmp_path: Path) -> None:
    journal = tmp_path / "test.journal"
    journal.write_text(
        "\n".join(
            [
                "account assets:kontist:geschaeftskonto  ; elster_account:business, elster_item:Geschäftskonto",
                "account income:business  ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen",
                "",
            ]
        )
        + "\n"
        + "\n\n".join(
            [
                "2024-01-10 Private purchase\n"
                "    expenses:personal:misc           25.00 EUR\n"
                "    assets:kontist:geschaeftskonto  -25.00 EUR",
                "2024-01-11 Owner deposit\n"
                "    liabilities:owner               -40.00 EUR\n"
                "    assets:kontist:geschaeftskonto   40.00 EUR",
                "2024-01-12 Move to private PayPal  ; elster_role:ignore\n"
                "    transfers:kontist-paypal         50.00 EUR\n"
                "    assets:kontist:geschaeftskonto  -50.00 EUR",
                "2024-01-13 Business income\n"
                "    income:business                -119.00 EUR\n"
                "    assets:kontist:geschaeftskonto  119.00 EUR",
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    dataset = build_dataset(journal)

    summary_rows = euer_rows(dataset, 2024, TaxConfig())
    assert next(row for row in summary_rows if row["Kennzahl"] == "Entnahmen")["2024"] == "25.00"
    assert next(row for row in summary_rows if row["Kennzahl"] == "Einlagen")["2024"] == "40.00"

    sheets = herleitung_sheets(dataset, 2024)["einnahmen-ueberschuss-rechnung"]
    entnahmen = next(sheet for sheet in sheets if sheet.name == "Entnahmen")
    einlagen = next(sheet for sheet in sheets if sheet.name == "Einlagen")
    ignored = herleitung_sheets(dataset, 2024)["ignoriert"][0]

    assert entnahmen.headers == ["Konto", "Datum", "Beschreibung", "Betrag"]
    assert entnahmen.rows[-1].cells[-1] == "25.00"
    assert entnahmen.rows[-2].cells[0] == "Σ Geschäftskonto"

    assert einlagen.headers == ["Konto", "Datum", "Beschreibung", "Betrag"]
    assert einlagen.rows[-1].cells[-1] == "40.00"
    assert einlagen.rows[-2].cells[0] == "Σ Geschäftskonto"

    assert ignored.headers == ["Konto", "Datum", "Beschreibung", "Gegenkonto", "Betrag"]
    assert len([row for row in ignored.rows if row.outline_level == 2]) == 1
    assert ignored.rows[0].cells == [
        "Geschäftskonto",
        "2024-01-12",
        "Move to private PayPal",
        "transfers:kontist-paypal",
        "50.00",
    ]
