use crate::afa;
use crate::classification::{euer_expenses, euer_income};
use crate::dataset::TaxDataset;
use crate::drawing::{is_contribution, is_drawing};
use crate::posting::TaxPosting;
use crate::ust::VatAdvanceError;
use indexmap::IndexMap;
use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::HashSet;

fn q(v: Decimal) -> Decimal {
    v.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

fn fmt(v: Decimal) -> String {
    format!("{:.2}", q(v))
}

#[derive(Debug, Clone)]
pub struct TrailRow {
    pub cells: Vec<String>,
    /// 0 = total, 1 = bank subtotal, 2 = transaction. Not consumed by the xlsx
    /// writer (styling there is driven by `fill`); kept as row-hierarchy
    /// metadata asserted by tests, matching the Python dataclass's own comment.
    #[allow(dead_code)]
    pub outline_level: i32,
    pub bold: bool,
    /// "" | "subtotal" | "total"
    pub fill: String,
}

impl TrailRow {
    fn new(cells: Vec<String>, outline_level: i32) -> Self {
        Self {
            cells,
            outline_level,
            bold: false,
            fill: String::new(),
        }
    }

    fn styled(cells: Vec<String>, outline_level: i32, fill: &str) -> Self {
        Self {
            cells,
            outline_level,
            bold: true,
            fill: fill.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrailSheet {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<TrailRow>,
}

// ── helpers ──────────────────────────────────────────────────────────────

fn gross(p: &TaxPosting) -> Decimal {
    q(p.amount.abs())
}

fn signed_gross(p: &TaxPosting) -> Decimal {
    q(p.amount)
}

fn net(p: &TaxPosting) -> Decimal {
    let g = p.amount.abs();
    if p.vat_mode == "contains_vat" && p.vat_rate > Decimal::ZERO {
        q(g / (Decimal::ONE + p.vat_rate))
    } else {
        q(g)
    }
}

fn vat_amount(p: &TaxPosting) -> Decimal {
    q(gross(p) - net(p))
}

fn signed_net(p: &TaxPosting) -> Decimal {
    if p.vat_mode == "contains_vat" && p.vat_rate > Decimal::ZERO {
        q(p.amount / (Decimal::ONE + p.vat_rate))
    } else {
        q(p.amount)
    }
}

fn signed_vat_amount(p: &TaxPosting) -> Decimal {
    q(signed_gross(p) - signed_net(p))
}

fn signed_deductible_net(p: &TaxPosting) -> Decimal {
    q(signed_net(p) * p.expense_share)
}

fn signed_deductible_vat(p: &TaxPosting) -> Decimal {
    q(signed_vat_amount(p) * p.input_vat_share)
}

fn reverse_charge_vat(p: &TaxPosting) -> Decimal {
    q(p.amount * p.vat_rate)
}

fn deductible_reverse_charge_vat(p: &TaxPosting) -> Decimal {
    q(reverse_charge_vat(p) * p.input_vat_share)
}

fn short(account: &str) -> String {
    account.rsplit(':').next().unwrap_or(account).to_string()
}

/// Human-readable label for the source (bank) account.
fn bank_label(p: &TaxPosting) -> String {
    if p.source_label.is_empty() {
        short(&p.source_account)
    } else {
        p.source_label.clone()
    }
}

/// Uses `elster_item` from the first matching posting if available, else the last account component.
fn sheet_label(ds: &TaxDataset, account: &str) -> String {
    ds.iter()
        .find(|p| !p.label.is_empty())
        .map(|p| p.label.clone())
        .unwrap_or_else(|| short(account))
}

fn posting_label(p: &TaxPosting) -> String {
    if p.label.is_empty() {
        short(&p.counter_account)
    } else {
        p.label.clone()
    }
}

fn est_section_dataset(dataset: &TaxDataset, year: i32) -> TaxDataset {
    dataset
        .iter()
        .filter(|p| p.tax_form == "einkommensteuer" && !p.section.is_empty() && p.year() == year)
        .cloned()
        .collect()
}

/// Groups postings by source account, sorted by account then date.
fn by_bank(postings: &[TaxPosting]) -> IndexMap<String, Vec<TaxPosting>> {
    let mut sorted = postings.to_vec();
    sorted.sort_by(|a, b| {
        (a.source_account.as_str(), a.posting_date)
            .cmp(&(b.source_account.as_str(), b.posting_date))
    });
    let mut groups: IndexMap<String, Vec<TaxPosting>> = IndexMap::new();
    for p in sorted {
        groups.entry(p.source_account.clone()).or_default().push(p);
    }
    groups
}

/// Short label for a bank subtotal row: "Σ <bank_label>".
fn bank_subtotal_label(ps: &[TaxPosting]) -> String {
    match ps.first() {
        Some(p) => format!("Σ {}", bank_label(p)),
        None => "Σ".to_string(),
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

fn unique_name(name: &str, seen: &mut HashSet<String>) -> String {
    let candidate = truncate_chars(name, 31);
    if !seen.contains(&candidate) {
        seen.insert(candidate.clone());
        return candidate;
    }
    let mut i = 2;
    loop {
        let suffix = format!(" ({i})");
        let max_base = 31usize.saturating_sub(suffix.chars().count());
        let candidate = format!("{}{}", truncate_chars(name, max_base), suffix);
        if !seen.contains(&candidate) {
            seen.insert(candidate.clone());
            return candidate;
        }
        i += 1;
    }
}

// ── sheet builders ───────────────────────────────────────────────────────

fn expense_sheet(ds: &TaxDataset, account: &str) -> TrailSheet {
    let headers = [
        "Konto",
        "Datum",
        "Beschreibung",
        "Brutto",
        "Netto",
        "Anteil",
        "Abziehbar",
    ]
    .map(String::from)
    .to_vec();
    let mut rows = Vec::new();
    let (mut t_gross, mut t_net, mut t_ded) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);

    for ps in by_bank(&ds.iter().cloned().collect::<Vec<_>>()).values() {
        let (mut b_gross, mut b_net, mut b_ded) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
        for p in ps {
            let (g, n, d) = (signed_gross(p), signed_net(p), signed_deductible_net(p));
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.description.clone(),
                    fmt(g),
                    fmt(n),
                    fmt(p.expense_share),
                    fmt(d),
                ],
                2,
            ));
            b_gross += g;
            b_net += n;
            b_ded += d;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                String::new(),
                fmt(q(b_gross)),
                fmt(q(b_net)),
                String::new(),
                fmt(q(b_ded)),
            ],
            1,
            "subtotal",
        ));
        t_gross += b_gross;
        t_net += b_net;
        t_ded += b_ded;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            fmt(q(t_gross)),
            fmt(q(t_net)),
            String::new(),
            fmt(q(t_ded)),
        ],
        0,
        "total",
    ));
    TrailSheet {
        name: sheet_label(ds, account),
        headers,
        rows,
    }
}

fn income_sheet(ds: &TaxDataset) -> TrailSheet {
    let headers = [
        "Konto",
        "Datum",
        "Beschreibung",
        "Brutto",
        "Netto",
        "USt-Betrag",
    ]
    .map(String::from)
    .to_vec();
    let mut rows = Vec::new();
    let (mut t_gross, mut t_net, mut t_vat) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);

    for ps in by_bank(&ds.iter().cloned().collect::<Vec<_>>()).values() {
        let (mut b_gross, mut b_net, mut b_vat) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
        for p in ps {
            let (g, n, v) = (gross(p), net(p), vat_amount(p));
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.description.clone(),
                    fmt(g),
                    fmt(n),
                    fmt(v),
                ],
                2,
            ));
            b_gross += g;
            b_net += n;
            b_vat += v;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                String::new(),
                fmt(q(b_gross)),
                fmt(q(b_net)),
                fmt(q(b_vat)),
            ],
            1,
            "subtotal",
        ));
        t_gross += b_gross;
        t_net += b_net;
        t_vat += b_vat;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            fmt(q(t_gross)),
            fmt(q(t_net)),
            fmt(q(t_vat)),
        ],
        0,
        "total",
    ));
    TrailSheet {
        name: "Einnahmen".to_string(),
        headers,
        rows,
    }
}

/// Abziehbare Vorsteuer for EÜR expense postings.
fn input_vat_sheet(ds: &TaxDataset) -> TrailSheet {
    let headers = [
        "Konto",
        "Datum",
        "Beschreibung",
        "Brutto",
        "USt-Betrag",
        "USt-Anteil",
        "Abziehbar",
    ]
    .map(String::from)
    .to_vec();
    let mut rows = Vec::new();
    let postings: Vec<TaxPosting> = ds
        .iter()
        .filter(|p| p.vat_mode == "contains_vat" && p.vat_rate > Decimal::ZERO)
        .cloned()
        .collect();
    let (mut t_gross, mut t_vat, mut t_ded) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);

    for ps in by_bank(&postings).values() {
        let (mut b_gross, mut b_vat, mut b_ded) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
        for p in ps {
            let (g, v, d) = (
                signed_gross(p),
                signed_vat_amount(p),
                signed_deductible_vat(p),
            );
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.description.clone(),
                    fmt(g),
                    fmt(v),
                    fmt(p.input_vat_share),
                    fmt(d),
                ],
                2,
            ));
            b_gross += g;
            b_vat += v;
            b_ded += d;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                String::new(),
                fmt(q(b_gross)),
                fmt(q(b_vat)),
                String::new(),
                fmt(q(b_ded)),
            ],
            1,
            "subtotal",
        ));
        t_gross += b_gross;
        t_vat += b_vat;
        t_ded += b_ded;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            fmt(q(t_gross)),
            fmt(q(t_vat)),
            String::new(),
            fmt(q(t_ded)),
        ],
        0,
        "total",
    ));
    TrailSheet {
        name: "Vorsteuer".to_string(),
        headers,
        rows,
    }
}

fn reverse_charge_sheet(ds: &TaxDataset) -> TrailSheet {
    let headers = [
        "Konto",
        "Datum",
        "Beschreibung",
        "Art",
        "Netto",
        "USt-Satz",
        "USt-Schuld",
        "Abziehbare Vorsteuer",
    ]
    .map(String::from)
    .to_vec();
    let mut rows = Vec::new();
    let postings: Vec<TaxPosting> = ds
        .iter()
        .filter(|p| p.vat_mode.starts_with("reverse_charge_"))
        .cloned()
        .collect();
    let (mut t_net, mut t_vat, mut t_ded) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);

    for ps in by_bank(&postings).values() {
        let (mut b_net, mut b_vat, mut b_ded) = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
        for p in ps {
            let net_v = q(p.amount);
            let vat_v = reverse_charge_vat(p);
            let ded_v = deductible_reverse_charge_vat(p);
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.description.clone(),
                    p.vat_mode
                        .strip_prefix("reverse_charge_")
                        .unwrap_or(&p.vat_mode)
                        .to_string(),
                    fmt(net_v),
                    fmt(p.vat_rate),
                    fmt(vat_v),
                    fmt(ded_v),
                ],
                2,
            ));
            b_net += net_v;
            b_vat += vat_v;
            b_ded += ded_v;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                String::new(),
                String::new(),
                fmt(q(b_net)),
                String::new(),
                fmt(q(b_vat)),
                fmt(q(b_ded)),
            ],
            1,
            "subtotal",
        ));
        t_net += b_net;
        t_vat += b_vat;
        t_ded += b_ded;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            String::new(),
            fmt(q(t_net)),
            String::new(),
            fmt(q(t_vat)),
            fmt(q(t_ded)),
        ],
        0,
        "total",
    ));
    TrailSheet {
        name: "§13b Reverse Charge".to_string(),
        headers,
        rows,
    }
}

fn afa_sheet(postings: &[TaxPosting], account: &str, year: i32) -> TrailSheet {
    let headers = [
        "Beschreibung",
        "Kaufdatum",
        "Kaufpreis (Netto)",
        "AfA-Jahre",
        "Monat. AfA",
        &format!("AfA {year}"),
    ]
    .map(String::from)
    .to_vec();
    let mut rows = Vec::new();
    let mut total = Decimal::ZERO;

    let mut sorted = postings.to_vec();
    sorted.sort_by_key(|p| p.posting_date);
    for p in &sorted {
        if p.afa_years <= 0 {
            continue;
        }
        let cost = afa::net_cost(p);
        let monthly = q(cost / Decimal::from(p.afa_years * 12));
        let annual = afa::depreciation_for_year(p, year);
        rows.push(TrailRow::new(
            vec![
                p.description.clone(),
                p.posting_date.to_string(),
                fmt(q(cost)),
                p.afa_years.to_string(),
                fmt(monthly),
                fmt(annual),
            ],
            1,
        ));
        total += annual;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            fmt(q(total)),
        ],
        0,
        "total",
    ));
    let label = postings
        .iter()
        .find(|p| !p.label.is_empty())
        .map(|p| p.label.clone())
        .unwrap_or_else(|| short(account));
    TrailSheet {
        name: format!("AfA {label}"),
        headers,
        rows,
    }
}

fn gross_sheet(ds: &TaxDataset, name: &str, signed: bool) -> TrailSheet {
    let headers = ["Konto", "Datum", "Beschreibung", "Betrag"]
        .map(String::from)
        .to_vec();
    let mut rows = Vec::new();
    let mut total = Decimal::ZERO;

    for ps in by_bank(&ds.iter().cloned().collect::<Vec<_>>()).values() {
        let mut b_total = Decimal::ZERO;
        for p in ps {
            let amount = if signed { p.amount } else { gross(p) };
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.description.clone(),
                    fmt(q(amount)),
                ],
                2,
            ));
            b_total += amount;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                String::new(),
                fmt(q(b_total)),
            ],
            1,
            "subtotal",
        ));
        total += b_total;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            fmt(q(total)),
        ],
        0,
        "total",
    ));
    TrailSheet {
        name: name.to_string(),
        headers,
        rows,
    }
}

fn vat_outflows_sheet(ds: &TaxDataset) -> TrailSheet {
    gross_sheet(
        ds,
        "An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer",
        true,
    )
}

fn vat_refunds_sheet(ds: &TaxDataset) -> TrailSheet {
    gross_sheet(
        ds,
        "Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer",
        true,
    )
}

fn ignored_sheet(ds: &TaxDataset, year: i32) -> TrailSheet {
    let headers = ["Konto", "Datum", "Beschreibung", "Gegenkonto", "Betrag"]
        .map(String::from)
        .to_vec();
    let mut rows = Vec::new();
    let mut total = Decimal::ZERO;

    let ignored: Vec<TaxPosting> = ds
        .for_role("ignore")
        .for_year(year)
        .iter()
        .cloned()
        .collect();
    for ps in by_bank(&ignored).values() {
        let mut b_total = Decimal::ZERO;
        for p in ps {
            let amount = q(p.amount);
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.description.clone(),
                    p.counter_account.clone(),
                    fmt(amount),
                ],
                2,
            ));
            b_total += amount;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                String::new(),
                String::new(),
                fmt(q(b_total)),
            ],
            1,
            "subtotal",
        ));
        total += b_total;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            String::new(),
            String::new(),
            fmt(q(total)),
        ],
        0,
        "total",
    ));
    TrailSheet {
        name: "Ignoriert".to_string(),
        headers,
        rows,
    }
}

fn vat_advance_sheet(ds: &TaxDataset, year: i32) -> Result<TrailSheet, VatAdvanceError> {
    let invalid: Vec<&TaxPosting> = ds
        .iter()
        .filter(|p| {
            p.tax_role == "vat_advance" && p.amount != Decimal::ZERO && p.tax_period.is_empty()
        })
        .collect();
    if !invalid.is_empty() {
        let examples = invalid
            .iter()
            .take(3)
            .map(|p| {
                format!(
                    "{} {} ({} -> {})",
                    p.posting_date, p.description, p.source_account, p.counter_account
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Err(VatAdvanceError::new(format!(
            "vat_advance postings require elster_period. Missing elster_period for {} posting(s): {examples}",
            invalid.len()
        )));
    }

    let headers = ["Konto", "Datum", "Steuerperiode", "Beschreibung", "Betrag"]
        .map(String::from)
        .to_vec();
    let mut rows = Vec::new();
    let mut total = Decimal::ZERO;
    let relevant: Vec<TaxPosting> = ds
        .iter()
        .filter(|p| p.amount != Decimal::ZERO && p.tax_period_year == year)
        .cloned()
        .collect();

    for ps in by_bank(&relevant).values() {
        let mut b_total = Decimal::ZERO;
        for p in ps {
            let amount = q(p.amount);
            rows.push(TrailRow::new(
                vec![
                    bank_label(p),
                    p.posting_date.to_string(),
                    p.tax_period.clone(),
                    p.description.clone(),
                    fmt(amount),
                ],
                2,
            ));
            b_total += amount;
        }
        rows.push(TrailRow::styled(
            vec![
                bank_subtotal_label(ps),
                String::new(),
                year.to_string(),
                String::new(),
                fmt(q(b_total)),
            ],
            1,
            "subtotal",
        ));
        total += b_total;
    }

    rows.push(TrailRow::styled(
        vec![
            "GESAMT".to_string(),
            String::new(),
            year.to_string(),
            String::new(),
            fmt(q(total)),
        ],
        0,
        "total",
    ));
    Ok(TrailSheet {
        name: "Bereits Entrichtet".to_string(),
        headers,
        rows,
    })
}

// ── per-form sheet lists ─────────────────────────────────────────────────

fn collect(sheets: &mut Vec<TrailSheet>, seen: &mut HashSet<String>, mut sheet: TrailSheet) {
    if sheet.rows.len() <= 1 {
        // only a GESAMT row = no data
        return;
    }
    sheet.name = unique_name(&sheet.name, seen);
    sheets.push(sheet);
}

fn euer_sheets(dataset: &TaxDataset, year: i32) -> Result<Vec<TrailSheet>, VatAdvanceError> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    let income_ds = euer_income(dataset).for_year(year);
    collect(&mut result, &mut seen, income_sheet(&income_ds));

    let euer_ds = euer_expenses(dataset);
    let mut by_label: IndexMap<String, Vec<TaxPosting>> = IndexMap::new();
    for p in euer_ds.exclude_deduction("afa").for_year(year).iter() {
        by_label
            .entry(posting_label(p))
            .or_default()
            .push(p.clone());
    }
    let mut labels: Vec<String> = by_label.keys().cloned().collect();
    labels.sort();
    for label in labels {
        collect(
            &mut result,
            &mut seen,
            expense_sheet(
                &TaxDataset::new(by_label.get(&label).unwrap().clone()),
                &label,
            ),
        );
    }

    let afa_postings: Vec<TaxPosting> = dataset.for_deduction("afa").iter().cloned().collect();
    let mut afa_accounts: Vec<String> = afa_postings
        .iter()
        .map(|p| p.counter_account.clone())
        .collect();
    afa_accounts.sort();
    afa_accounts.dedup();
    for account in &afa_accounts {
        let acc_ps: Vec<TaxPosting> = afa_postings
            .iter()
            .filter(|p| &p.counter_account == account)
            .cloned()
            .collect();
        collect(&mut result, &mut seen, afa_sheet(&acc_ps, account, year));
    }

    collect(
        &mut result,
        &mut seen,
        input_vat_sheet(&euer_ds.for_year(year)),
    );

    let vat_payment_ds: TaxDataset = dataset
        .for_year(year)
        .iter()
        .filter(|p| p.tax_role == "vat_payment" && p.amount > Decimal::ZERO)
        .cloned()
        .collect();
    let vat_outflows_ds: TaxDataset = dataset
        .for_year(year)
        .iter()
        .filter(|p| {
            matches!(p.tax_role.as_str(), "vat_payment" | "vat_advance") && p.amount > Decimal::ZERO
        })
        .cloned()
        .collect();
    let vat_refund_ds: TaxDataset = dataset
        .for_year(year)
        .iter()
        .filter(|p| p.tax_role == "vat_payment" && p.amount < Decimal::ZERO)
        .cloned()
        .collect();
    collect(&mut result, &mut seen, vat_outflows_sheet(&vat_outflows_ds));
    collect(&mut result, &mut seen, vat_refunds_sheet(&vat_refund_ds));
    collect(
        &mut result,
        &mut seen,
        gross_sheet(&vat_payment_ds, "USt-Abschlusszahlungen", false),
    );
    collect(
        &mut result,
        &mut seen,
        vat_advance_sheet(&dataset.for_role("vat_advance"), year)?,
    );

    let drawing_ds: TaxDataset = dataset
        .for_year(year)
        .iter()
        .filter(|p| is_drawing(p))
        .cloned()
        .collect();
    collect(
        &mut result,
        &mut seen,
        gross_sheet(&drawing_ds, "Entnahmen", false),
    );
    let contribution_ds: TaxDataset = dataset
        .for_year(year)
        .iter()
        .filter(|p| is_contribution(p))
        .cloned()
        .collect();
    collect(
        &mut result,
        &mut seen,
        gross_sheet(&contribution_ds, "Einlagen", false),
    );

    Ok(result)
}

fn ust_sheets(dataset: &TaxDataset, year: i32) -> Result<Vec<TrailSheet>, VatAdvanceError> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    let income_ds = euer_income(dataset).for_year(year);
    collect(&mut result, &mut seen, income_sheet(&income_ds));

    let vat_payment_ds: TaxDataset = dataset
        .for_year(year)
        .iter()
        .filter(|p| p.tax_role == "vat_payment" && p.amount > Decimal::ZERO)
        .cloned()
        .collect();
    collect(
        &mut result,
        &mut seen,
        gross_sheet(&vat_payment_ds, "USt-Abschlusszahlungen", false),
    );
    collect(
        &mut result,
        &mut seen,
        vat_advance_sheet(&dataset.for_role("vat_advance"), year)?,
    );

    let euer_ds = euer_expenses(dataset);
    collect(
        &mut result,
        &mut seen,
        input_vat_sheet(&euer_ds.for_year(year)),
    );
    collect(
        &mut result,
        &mut seen,
        reverse_charge_sheet(&euer_ds.for_year(year)),
    );

    Ok(result)
}

fn est_sheets(dataset: &TaxDataset, year: i32) -> Vec<TrailSheet> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    let section_ds = est_section_dataset(dataset, year);
    let mut by_label: IndexMap<String, Vec<TaxPosting>> = IndexMap::new();
    for p in section_ds.iter() {
        by_label
            .entry(posting_label(p))
            .or_default()
            .push(p.clone());
    }
    let mut labels: Vec<String> = by_label.keys().cloned().collect();
    labels.sort();
    for label in labels {
        collect(
            &mut result,
            &mut seen,
            gross_sheet(
                &TaxDataset::new(by_label.get(&label).unwrap().clone()),
                &label,
                true,
            ),
        );
    }

    collect(
        &mut result,
        &mut seen,
        gross_sheet(
            &dataset.for_role("income_tax_advance").for_year(year),
            "ESt Vorauszahlung",
            false,
        ),
    );
    collect(
        &mut result,
        &mut seen,
        gross_sheet(
            &dataset.for_role("income_tax_final").for_year(year),
            "ESt Abschluss",
            false,
        ),
    );

    result
}

// ── public entry point ───────────────────────────────────────────────────

pub const FORM_KEYS: &[&str] = &[
    "einnahmen-ueberschuss-rechnung",
    "umsatzsteuer",
    "einkommensteuer",
];

pub fn herleitung_sheets(
    dataset: &TaxDataset,
    year: i32,
) -> Result<IndexMap<String, Vec<TrailSheet>>, VatAdvanceError> {
    let ignored = ignored_sheet(dataset, year);
    let mut result = IndexMap::new();
    result.insert(
        "einnahmen-ueberschuss-rechnung".to_string(),
        euer_sheets(dataset, year)?,
    );
    result.insert("umsatzsteuer".to_string(), ust_sheets(dataset, year)?);
    result.insert("einkommensteuer".to_string(), est_sheets(dataset, year));
    result.insert(
        "ignoriert".to_string(),
        if ignored.rows.len() > 1 {
            vec![ignored]
        } else {
            vec![]
        },
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enrich::build_dataset;
    use std::path::Path;

    fn example_dataset() -> TaxDataset {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        build_dataset(&manifest_dir.join("examples/ledger/hledger.journal")).unwrap()
    }

    fn find<'a>(sheets: &'a [TrailSheet], name: &str) -> &'a TrailSheet {
        sheets.iter().find(|s| s.name == name).unwrap()
    }

    #[test]
    fn euer_sheets_include_income_and_expense_trails() {
        let dataset = example_dataset();
        let sheets = herleitung_sheets(&dataset, 2024).unwrap();
        let euer = &sheets["einnahmen-ueberschuss-rechnung"];
        let income = find(euer, "Einnahmen");
        // GESAMT row's gross column (index 3) should match the golden 1190.00 gross income.
        assert_eq!(income.rows.last().unwrap().cells[3], "1190.00");

        let wasabi = find(euer, "Serverkosten Wasabi");
        assert_eq!(wasabi.rows.last().unwrap().cells[6], "20.00");
    }

    #[test]
    fn afa_sheet_uses_outline_level_one_for_line_items_and_zero_for_total() {
        let dataset = example_dataset();
        let sheets = herleitung_sheets(&dataset, 2024).unwrap();
        let euer = &sheets["einnahmen-ueberschuss-rechnung"];
        let afa = find(euer, "AfA Computer-Kauf");
        assert_eq!(afa.rows[0].outline_level, 1);
        assert_eq!(afa.rows.last().unwrap().outline_level, 0);
        assert_eq!(afa.rows.last().unwrap().cells[5], "222.22");
    }

    #[test]
    fn afa_sheet_rounds_net_cost_half_up_instead_of_truncating() {
        use crate::posting::test_support::posting;
        use std::str::FromStr;

        // 1150.30 / 1.19 = 966.6386...5, which must round up to 966.64 for display.
        // A naive `format!("{v:.2}")` on the unrounded rust_decimal::Decimal truncates
        // to 966.63 instead, silently corrupting the Herleitung audit trail even though
        // the actual filed AfA figures (which round only their final yearly total) stay
        // correct.
        let mut p = posting(
            "2020-12-30",
            "assets:bank",
            "expenses:hardware:computer",
            "1150.30",
        );
        p.vat_mode = "contains_vat".to_string();
        p.vat_rate = Decimal::from_str("0.19").unwrap();
        p.afa_years = 3;
        p.tax_deduction = "afa".to_string();

        let sheet = afa_sheet(&[p], "expenses:hardware:computer", 2020);
        assert_eq!(sheet.rows[0].cells[2], "966.64");
    }

    #[test]
    fn ignoriert_sheet_is_absent_when_there_are_no_ignored_postings() {
        let dataset = example_dataset();
        let sheets = herleitung_sheets(&dataset, 2024).unwrap();
        assert!(sheets["ignoriert"].is_empty());
    }

    #[test]
    fn ust_sheets_include_input_vat_trail() {
        let dataset = example_dataset();
        let sheets = herleitung_sheets(&dataset, 2024).unwrap();
        let ust = &sheets["umsatzsteuer"];
        assert!(ust.iter().any(|s| s.name == "Vorsteuer"));
    }

    #[test]
    fn est_sheets_group_by_posting_label() {
        let dataset = example_dataset();
        let sheets = herleitung_sheets(&dataset, 2024).unwrap();
        let est = &sheets["einkommensteuer"];
        let kv = find(est, "Krankenversicherung");
        assert_eq!(kv.rows.last().unwrap().cells[3], "840.00");
    }

    #[test]
    fn sheet_names_are_deduplicated_and_truncated_to_31_chars() {
        let mut seen = HashSet::new();
        let a = unique_name("Langzeit-Auslandskrankenversicherung", &mut seen);
        assert_eq!(a.chars().count(), 31);
        let b = unique_name("Langzeit-Auslandskrankenversicherung", &mut seen);
        assert_ne!(a, b);
        assert!(b.chars().count() <= 31);
    }

    #[test]
    fn vat_advance_missing_period_is_rejected_with_context() {
        use crate::posting::test_support::posting;
        let mut p = posting("2024-01-01", "assets:bank", "expenses:tax", "50.00");
        p.tax_role = "vat_advance".to_string();
        let ds = TaxDataset::new(vec![p]);
        let err = herleitung_sheets(&ds, 2024).unwrap_err();
        assert!(err.to_string().contains("require elster_period"));
    }
}
