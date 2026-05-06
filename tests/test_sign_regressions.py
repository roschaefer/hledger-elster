from __future__ import annotations

from pathlib import Path

import pytest

from calculate.report.est import est_rows
from calculate.report.euer import euer_rows
from calculate.report.herleitung import herleitung_sheets
from calculate.report.ust import ust_rows
from config import TaxConfig
from ingest.enrich import build_dataset


def _journal_entry(p: dict[str, str]) -> str:
    lines = [f"{p['date']} {p['description']}"]
    lines.append(f"    {p['account']}    {p['amount']} {p['currency']}")
    lines.append(f"    {p['source_account']}")
    return "\n".join(lines)


def _build_dataset(tmp_path: Path, account_lines: list[str], postings: list[dict[str, str]]):
    journal = tmp_path / "test.journal"
    journal.write_text(
        "\n".join(account_lines) + "\n\n" + "\n\n".join(_journal_entry(p) for p in postings) + "\n",
        encoding="utf-8",
    )
    return build_dataset(journal)


def _posting(
    transaction_id: str,
    date: str,
    description: str,
    account: str,
    amount: str,
    *,
    source_account: str = "assets:kontist:geschaeftskonto",
) -> dict[str, str]:
    return {
        "transaction_id": transaction_id,
        "date": date,
        "description": description,
        "account": account,
        "amount": amount,
        "currency": "EUR",
        "source_account": source_account,
    }


def test_reimbursements_reduce_euer_and_vorsteuer_totals(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:dkb:kreditkarte  ;elster_account:private, elster_item:Kreditkartenkonto",
            "account expenses:business  ;elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat_rate:0.19, elster_vat_share:1.00",
            "account expenses:business:hosting:aws  ;elster_item:AWS",
        ],
        [
            _posting(
                "1",
                "2024-02-02",
                "AWS EMEA",
                "expenses:business:hosting:aws",
                "45.21",
                source_account="assets:dkb:kreditkarte",
            ),
            _posting(
                "2",
                "2024-02-06",
                "AWS EMEA",
                "expenses:business:hosting:aws",
                "-13.23",
                source_account="assets:dkb:kreditkarte",
            ),
            _posting(
                "3",
                "2024-02-06",
                "AWS EMEA",
                "expenses:business:hosting:aws",
                "-45.16",
                source_account="assets:dkb:kreditkarte",
            ),
        ],
    )

    euer = euer_rows(dataset, 2024, TaxConfig())
    assert next(row for row in euer if row["Kennzahl"] == "AWS")["2024"] == "-11.08"

    vorsteuer = next(
        sheet
        for sheet in herleitung_sheets(dataset, 2024)["einnahmen-ueberschuss-rechnung"]
        if sheet.name == "Vorsteuer"
    )
    assert vorsteuer.rows[-2].cells == ["Σ Kreditkartenkonto", "", "", "-13.18", "-2.10", "", "-2.10"]


def test_euer_income_with_vat_is_not_vorsteuer(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:kontist:geschaeftskonto  ;elster_account:business, elster_item:Geschäftskonto",
            "account income:business:consulting  ;elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_item:Betriebseinnahmen",
            "account expenses:business:hosting:aws  ;elster_form:einnahmenueberschussrechnung, elster_deduction:full, elster_vat_rate:0.19, elster_vat_share:1.00, elster_item:AWS",
        ],
        [
            _posting("1", "2024-01-10", "Customer invoice", "income:business:consulting", "-119.00"),
            _posting("2", "2024-01-11", "AWS EMEA", "expenses:business:hosting:aws", "11.90"),
        ],
    )

    euer = euer_rows(dataset, 2024, TaxConfig())
    assert (
        next(row for row in euer if row["Kennzahl"] == "Umsatzsteuerpflichtige Betriebseinnahmen")["2024"] == "100.00"
    )
    assert next(row for row in euer if row["Kennzahl"] == "Vereinnahmte Umsatzsteuer")["2024"] == "19.00"
    assert next(row for row in euer if row["Kennzahl"] == "AWS")["2024"] == "10.00"

    ust = ust_rows(dataset, 2024)
    assert next(row for row in ust if row["Zeitraum"] == "2024")["Abziehbare Vorsteuerbeträge"] == "1.90"

    vorsteuer = next(sheet for sheet in herleitung_sheets(dataset, 2024)["umsatzsteuer"] if sheet.name == "Vorsteuer")
    assert "Customer invoice" not in {row.cells[2] for row in vorsteuer.rows if len(row.cells) > 2}
    assert vorsteuer.rows[-2].cells == ["Σ Geschäftskonto", "", "", "11.90", "1.90", "", "1.90"]


def test_euer_non_deductible_expense_is_visible_but_not_counted(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:kontist:geschaeftskonto  ;elster_account:business, elster_item:Geschäftskonto",
            "account expenses:business:penalty  ;elster_form:einnahmenueberschussrechnung, elster_deduction:non_deductible, elster_item:Nicht abzugsfähig",
        ],
        [
            _posting("1", "2024-01-10", "Business penalty", "expenses:business:penalty", "100.00"),
        ],
    )

    euer = euer_rows(dataset, 2024, TaxConfig())
    assert next(row for row in euer if row["Kennzahl"] == "Nicht abzugsfähig")["2024"] == "0.00"
    assert next(row for row in euer if row["Kennzahl"] == "Summe Betriebskosten")["2024"] == "0.00"


def test_income_tax_reversal_nets_out_in_est_summary(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:kontist:geschaeftskonto  ;elster_account:business, elster_item:Geschäftskonto",
            "account expenses:taxes:einkommensteuer:vorauszahlung  ;elster_role:income_tax_advance, elster_item:ESt-Vorauszahlung",
        ],
        [
            _posting(
                "1", "2024-03-14", "STEUERVERWALTUNG NRW", "expenses:taxes:einkommensteuer:vorauszahlung", "4000.00"
            ),
            _posting(
                "2", "2024-06-13", "STEUERVERWALTUNG NRW", "expenses:taxes:einkommensteuer:vorauszahlung", "-4000.00"
            ),
            _posting(
                "3", "2024-06-13", "STEUERVERWALTUNG NRW", "expenses:taxes:einkommensteuer:vorauszahlung", "4000.00"
            ),
        ],
    )

    est = est_rows(dataset, 2024)
    assert next(row for row in est if row["Kennzahl"] == "ESt-Vorauszahlung")["2024"] == "4000.00"


def test_est_insurance_rows_use_tax_metadata_not_account_case(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account Assets:DKB:Girokonto  ;elster_account:private, elster_item:Girokonto",
            "account Expenses:Insurance:Health:AOK:KV  ;elster_form:einkommensteuer, elster_section:Vorsorgeaufwand, elster_item:Krankenversicherung",
            "account Expenses:Insurance:Health:AOK:PV  ;elster_form:einkommensteuer, elster_section:Vorsorgeaufwand, elster_item:Pflegeversicherung",
            "account Expenses:Insurance:Liability:Haftpflicht  ;elster_form:einkommensteuer, elster_section:Vorsorgeaufwand, elster_item:Haftpflichtversicherung",
        ],
        [
            _posting(
                "1",
                "2024-01-15",
                "AOK",
                "Expenses:Insurance:Health:AOK:KV",
                "1200.00",
                source_account="Assets:DKB:Girokonto",
            ),
            _posting(
                "2",
                "2024-01-15",
                "AOK",
                "Expenses:Insurance:Health:AOK:PV",
                "577.31",
                source_account="Assets:DKB:Girokonto",
            ),
            _posting(
                "3",
                "2024-02-01",
                "Haftpflicht",
                "Expenses:Insurance:Liability:Haftpflicht",
                "57.88",
                source_account="Assets:DKB:Girokonto",
            ),
        ],
    )

    est = est_rows(dataset, 2024)
    assert next(row for row in est if row["Kennzahl"] == "Krankenversicherung")["2024"] == "1200.00"
    assert next(row for row in est if row["Kennzahl"] == "Pflegeversicherung")["2024"] == "577.31"
    assert next(row for row in est if row["Kennzahl"] == "Haftpflichtversicherung")["2024"] == "57.88"

    est_sheets = herleitung_sheets(dataset, 2024)["einkommensteuer"]
    assert {sheet.name for sheet in est_sheets} >= {
        "Krankenversicherung",
        "Pflegeversicherung",
        "Haftpflichtversicherung",
    }


def test_est_sonderausgaben_donations_are_exported_from_section_metadata(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:dkb:girokonto  ;elster_account:private, elster_item:Girokonto",
            "account expenses:charity:drk  ;elster_form:einkommensteuer, elster_section:Sonderausgaben, elster_item:Spenden",
        ],
        [
            _posting(
                "1",
                "2024-12-01",
                "DRK donation",
                "expenses:charity:drk",
                "50.00",
                source_account="assets:dkb:girokonto",
            ),
        ],
    )

    est = est_rows(dataset, 2024)
    assert next(row for row in est if row["Kennzahl"] == "Spenden")["2024"] == "50.00"
    assert "Summe privat gezahlt" not in {row["Kennzahl"] for row in est}

    est_sheets = herleitung_sheets(dataset, 2024)["einkommensteuer"]
    assert {sheet.name for sheet in est_sheets} >= {"Spenden"}


def test_conflicting_elster_form_tags_are_rejected(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="Conflicting elster_form tags"):
        _build_dataset(
            tmp_path,
            [
                "account assets:dkb:girokonto  ;elster_account:private, elster_item:Girokonto",
                "account expenses:mixed  ;elster_form:einkommensteuer",
                "account expenses:mixed:position  ;elster_form:einnahmenueberschussrechnung, elster_item:Mixed",
            ],
            [
                _posting(
                    "1",
                    "2024-12-01",
                    "Mixed posting",
                    "expenses:mixed:position",
                    "50.00",
                    source_account="assets:dkb:girokonto",
                ),
            ],
        )


def test_est_sections_are_user_defined_groupings(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:dkb:girokonto  ;elster_account:private, elster_item:Girokonto",
            "account expenses:private:one  ;elster_form:einkommensteuer, elster_section:Freie Gruppe A, elster_item:Erste Position",
            "account expenses:private:two  ;elster_form:einkommensteuer, elster_section:Freie Gruppe B, elster_item:Zweite Position",
        ],
        [
            _posting(
                "1",
                "2024-01-01",
                "First custom group",
                "expenses:private:one",
                "10.00",
                source_account="assets:dkb:girokonto",
            ),
            _posting(
                "2",
                "2024-01-02",
                "Second custom group",
                "expenses:private:two",
                "20.00",
                source_account="assets:dkb:girokonto",
            ),
        ],
    )

    est = est_rows(dataset, 2024)
    assert "# Freie Gruppe A" in {row["Kennzahl"] for row in est}
    assert "# Freie Gruppe B" in {row["Kennzahl"] for row in est}
    assert next(row for row in est if row["Kennzahl"] == "Erste Position")["2024"] == "10.00"
    assert next(row for row in est if row["Kennzahl"] == "Zweite Position")["2024"] == "20.00"


def test_vat_advance_reversal_should_net_out_in_ust_exports(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:kontist:geschaeftskonto  ;elster_account:business, elster_item:Geschäftskonto",
            "account expenses:taxes:umsatzsteuer:vorauszahlung  ;elster_role:vat_advance, elster_item:USt-Vorauszahlung",
            "account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ;elster_period:2024",
        ],
        [
            _posting(
                "1", "2024-02-15", "STEUERVERWALTUNG NRW", "expenses:taxes:umsatzsteuer:vorauszahlung:2024", "100.00"
            ),
            _posting(
                "2", "2024-03-01", "STEUERVERWALTUNG NRW", "expenses:taxes:umsatzsteuer:vorauszahlung:2024", "-40.00"
            ),
        ],
    )

    ust = ust_rows(dataset, 2024)
    assert next(row for row in ust if row["Zeitraum"] == "2024")["Bereits Entrichtet"] == "60.00"

    bezahlt = next(
        sheet for sheet in herleitung_sheets(dataset, 2024)["umsatzsteuer"] if sheet.name == "Bereits Entrichtet"
    )
    assert bezahlt.rows[-2].cells == ["Σ Geschäftskonto", "", "2024", "", "60.00"]


def test_euer_paid_vat_includes_refunds_and_herleitung_sheet_shows_signed_totals(tmp_path: Path) -> None:
    dataset = _build_dataset(
        tmp_path,
        [
            "account assets:kontist:geschaeftskonto  ;elster_account:business, elster_item:Geschäftskonto",
            "account expenses:taxes:umsatzsteuer  ;elster_role:vat_payment",
            "account expenses:taxes:umsatzsteuer:vorauszahlung  ;elster_role:vat_advance",
            "account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ;elster_period:2024",
        ],
        [
            _posting(
                "1", "2024-02-15", "STEUERVERWALTUNG NRW", "expenses:taxes:umsatzsteuer:vorauszahlung:2024", "100.00"
            ),
            _posting("2", "2024-03-01", "STEUERVERWALTUNG NRW", "expenses:taxes:umsatzsteuer", "-40.00"),
        ],
    )

    euer = euer_rows(dataset, 2024, TaxConfig())
    assert (
        next(row for row in euer if row["Kennzahl"] == "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer")[
            "2024"
        ]
        == "100.00"
    )
    assert (
        next(row for row in euer if row["Kennzahl"] == "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer")[
            "2024"
        ]
        == "40.00"
    )

    vat_sheet = next(
        sheet
        for sheet in herleitung_sheets(dataset, 2024)["einnahmen-ueberschuss-rechnung"]
        if sheet.name.startswith("An das Finanzamt gezahlte")
    )
    assert vat_sheet.rows[-2].cells == ["Σ Geschäftskonto", "", "", "100.00"]
