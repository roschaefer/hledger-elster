from __future__ import annotations

import json
import re
import subprocess
from datetime import date
from decimal import Decimal
from pathlib import Path

from domain.dataset import TaxDataset
from domain.posting import TaxPosting

# ── hledger ingestion ──────────────────────────────────────────────────────


def _posting_tags(posting: dict) -> dict[str, str]:
    # hledger lists most-specific account's tags first; first occurrence wins.
    result: dict[str, str] = {}
    for key, value in posting.get("ptags") or []:
        if key not in result:
            result[key] = value
    return result


def _source_posting(transaction: dict) -> dict | None:
    business_sources = [
        posting
        for posting in transaction.get("tpostings", [])
        if _posting_tags(posting).get("elster_account") == "business"
    ]
    if len(business_sources) == 1:
        return business_sources[0]

    tagged_sources = [
        posting
        for posting in transaction.get("tpostings", [])
        if _posting_tags(posting).get("elster_account") in {"business", "private"}
    ]
    if len(tagged_sources) == 1:
        return tagged_sources[0]

    return None


def _load_transactions(journal_path: Path) -> list[dict]:
    result = subprocess.run(
        [
            "hledger",
            "-f",
            str(journal_path),
            "print",
            "--output-format",
            "json",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


# ── enrichment ────────────────────────────────────────────────────────────


def _is_tax_relevant(tags: dict[str, str]) -> bool:
    return bool(tags.get("elster_form") or tags.get("elster_role") or tags.get("elster_deduction"))


def _to_decimal(value: str | None, default: str = "0") -> Decimal:
    return Decimal(value or default)


def _comment_has_ignore(*comments: str) -> bool:
    pattern = re.compile(r"(^|[,\s])elster_role\s*:\s*ignore($|[,\s])")
    return any(pattern.search(comment or "") for comment in comments)


def _fallback_tax_role(amount: Decimal, source_tags: dict[str, str]) -> str:
    if source_tags.get("elster_account") != "business":
        return ""
    if amount > Decimal("0"):
        return "drawing"
    if amount < Decimal("0"):
        return "contribution"
    return ""


def _enrich_posting(
    posting: dict,
    transaction_date: str,
    transaction_comment: str,
    description: str,
    source_account: str,
    source_tags: dict[str, str],
    source_file: str,
    source_line: int,
) -> TaxPosting | None:
    account = posting["paccount"]
    amounts = posting.get("pamount", [])
    if not amounts:
        return None

    quantity = amounts[0]["aquantity"]["floatingPoint"]
    amount = Decimal(f"{Decimal(str(quantity)):.2f}")
    posting_comment = (posting.get("pcomment") or "").strip()
    posting_date_raw = posting.get("pdate")
    posting_date = date.fromisoformat(posting_date_raw if posting_date_raw else transaction_date)

    tags = _posting_tags(posting)
    source_label = source_tags.get("elster_item", "")

    if _comment_has_ignore(transaction_comment, posting_comment):
        if account == source_account:
            return None
        return TaxPosting(
            posting_date=posting_date,
            source_account=source_account,
            counter_account=account,
            amount=amount,
            description=description,
            transaction_comment=transaction_comment,
            posting_comment=posting_comment,
            source_file=source_file,
            source_line=source_line,
            tax_form="",
            tax_deduction="",
            tax_role="ignore",
            calculation="",
            vat_rate=Decimal("0"),
            expense_share=Decimal("1"),
            vat_share=Decimal("0"),
            afa_years=0,
            derived_kind="",
            label="",
            source_label=source_label,
            section="",
            tax_period_year=0,
            source_is_business=source_tags.get("elster_account") == "business",
        )

    if not _is_tax_relevant(tags):
        if account == source_account:
            return None
        fallback_role = _fallback_tax_role(amount, source_tags)
        if not fallback_role:
            return None
        tags = {"elster_role": fallback_role}

    tax_deduction = tags.get("elster_deduction", "")
    tax_form = tags.get("elster_form", "")
    tax_role = tags.get("elster_role", "")
    calculation = tags.get("elster_calculation", "")
    vat_rate = _to_decimal(tags.get("elster_vat_rate"))
    expense_share = _to_decimal(tags.get("elster_expense_share"), "1")
    vat_share = _to_decimal(tags.get("elster_vat_share"), "0")
    afa_years_raw = tags.get("elster_afa_years")
    afa_years = int(afa_years_raw) if afa_years_raw else 0
    label = tags.get("elster_item", "")
    section = tags.get("elster_section", "")
    tax_period_raw = tags.get("elster_period", "")
    tax_period_year = int(tax_period_raw) if tax_period_raw else 0

    # GWG: elster_afa_years always overrides inherited elster_deduction
    if afa_years > 0:
        gross = abs(amount)
        net_cost = gross / (1 + vat_rate) if vat_rate > Decimal("0") else gross
        if net_cost > Decimal("800"):
            tax_deduction = "afa"
        else:
            tax_deduction = "full"
            afa_years = 0

    # für nicht_abzugsfaehig: full gross is what gets reported to ESt
    if tax_deduction == "nicht_abzugsfaehig":
        expense_share = Decimal("0")
        vat_share = Decimal("0")

    return TaxPosting(
        posting_date=posting_date,
        source_account=source_account,
        counter_account=account,
        amount=amount,
        description=description,
        transaction_comment=transaction_comment,
        posting_comment=posting_comment,
        source_file=source_file,
        source_line=source_line,
        tax_form=tax_form,
        tax_deduction=tax_deduction,
        tax_role=tax_role,
        calculation=calculation,
        vat_rate=vat_rate,
        expense_share=expense_share,
        vat_share=vat_share,
        afa_years=afa_years,
        derived_kind="",
        label=label,
        source_label=source_label,
        section=section,
        tax_period_year=tax_period_year,
        source_is_business=source_tags.get("elster_account") == "business",
    )


# ── top-level builder ──────────────────────────────────────────────────────


def build_dataset(journal_path: Path) -> TaxDataset:
    transactions = _load_transactions(journal_path)
    all_postings: list[TaxPosting] = []

    for transaction in transactions:
        source_posting = _source_posting(transaction)
        source_acct = source_posting["paccount"] if source_posting is not None else ""
        source_tags: dict[str, str] = {}
        if source_posting is not None:
            source_tags = _posting_tags(source_posting)
        source_positions = transaction.get("tsourcepos", []) or []
        source_file = source_positions[0]["sourceName"] if source_positions else ""
        source_line = source_positions[0]["sourceLine"] if source_positions else 0
        transaction_date = transaction["tdate"]
        transaction_comment = (transaction.get("tcomment") or "").strip()
        description = transaction.get("tdescription", "")

        for posting in transaction.get("tpostings", []):
            tp = _enrich_posting(
                posting=posting,
                transaction_date=transaction_date,
                transaction_comment=transaction_comment,
                description=description,
                source_account=source_acct,
                source_tags=source_tags,
                source_file=source_file,
                source_line=source_line,
            )
            if tp is not None:
                all_postings.append(tp)

    return TaxDataset(all_postings)
