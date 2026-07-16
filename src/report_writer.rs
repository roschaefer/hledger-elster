use crate::config;
use crate::enrich;
use crate::est::est_rows;
use crate::euer::euer_rows;
use crate::herleitung::{self, TrailSheet, FORM_KEYS};
use crate::paths;
use crate::periods::ReportRow;
use crate::ust::ust_rows;
use anyhow::Result;
use rust_xlsxwriter::{Format, Workbook, Worksheet, XlsxAlign, XlsxColor};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

// Fixed content so repeated runs produce byte-identical files, replacing
// rust_xlsxwriter's own `Utc::now()`-stamped docProps/core.xml.
const CORE_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:creator>hledger-elster</dc:creator><dcterms:created xsi:type="dcterms:W3CDTF">2000-01-01T00:00:00Z</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">2000-01-01T00:00:00Z</dcterms:modified></cp:coreProperties>"#;

// ── ZIP stabilization ────────────────────────────────────────────────────

/// Re-packs an in-memory xlsx (a zip archive) with sorted entries, a fixed
/// per-entry timestamp, and fixed `docProps/core.xml` content, so that two
/// runs against the same input produce byte-identical files.
fn stabilize_xlsx(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))?;

    let mut entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        entries.push((name, contents));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let fixed_time = zip::DateTime::from_date_and_time(2000, 1, 1, 0, 0, 0)
        .map_err(|e| anyhow::anyhow!("invalid fixed zip timestamp: {e:?}"))?;
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(fixed_time);

    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, contents) in entries {
        writer.start_file(&name, options)?;
        if name == "docProps/core.xml" {
            writer.write_all(CORE_XML)?;
        } else {
            writer.write_all(&contents)?;
        }
    }
    Ok(writer.finish()?.into_inner())
}

// ── cell styles ──────────────────────────────────────────────────────────

fn default_format() -> Format {
    Format::new()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xFFFFFF))
}

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xD9E1F2))
        .set_align(XlsxAlign::Center)
}

/// Same as `header_format` but without center alignment -- matches
/// `_write_trail_sheet`'s header styling, which (unlike the summary/USt
/// writers) never applied `Alignment(horizontal="center")` in the Python
/// implementation.
fn header_format_plain() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xD9E1F2))
}

fn bold_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xFFFFFF))
}

fn section_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xE9EFF7))
}

fn blank_format() -> Format {
    Format::new()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xF2F2F2))
}

fn subtotal_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xDDEBF7))
}

fn total_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(XlsxColor::RGB(0x000000))
        .set_background_color(XlsxColor::RGB(0xFCE4D6))
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

// ── filename / sheet-title sanitization ─────────────────────────────────

fn tab_csv_name(sheet_name: &str) -> String {
    let normalized: String = sheet_name
        .nfkd()
        .filter(char::is_ascii)
        .collect::<String>()
        .to_lowercase();
    let mut stem = String::new();
    let mut last_was_dash = false;
    for c in normalized.chars() {
        if c.is_ascii_alphanumeric() {
            stem.push(c);
            last_was_dash = false;
        } else if !last_was_dash {
            stem.push('-');
            last_was_dash = true;
        }
    }
    let stem = stem.trim_matches('-');
    let stem = if stem.is_empty() { "sheet" } else { stem };
    format!("{stem}.csv")
}

fn xlsx_sheet_title(sheet_name: &str) -> String {
    let title: String = sheet_name
        .chars()
        .map(|c| if "[]:*?/\\".contains(c) { '-' } else { c })
        .collect();
    let title = title.trim();
    let title = if title.is_empty() { "Sheet" } else { title };
    title.chars().take(31).collect()
}

// ── xlsx sheet writers ───────────────────────────────────────────────────

fn is_blank_row(row: &ReportRow) -> bool {
    row.values().all(|v| v.is_empty())
}

fn is_section_header(row: &ReportRow) -> bool {
    row.get("Kennzahl")
        .map(|k| k.starts_with("# "))
        .unwrap_or(false)
}

fn write_row(
    ws: &mut Worksheet,
    row_idx: u32,
    headers: &[String],
    row: &ReportRow,
    format: &Format,
) -> Result<()> {
    for (col_idx, header) in headers.iter().enumerate() {
        let value = row.get(header).map(String::as_str).unwrap_or("");
        ws.write_string(row_idx, col_idx as u16, value, format)?;
    }
    Ok(())
}

fn set_column_widths_for_rows(
    ws: &mut Worksheet,
    headers: &[String],
    rows: &[ReportRow],
) -> Result<()> {
    for (col_idx, header) in headers.iter().enumerate() {
        let max_len = rows
            .iter()
            .map(|r| char_len(r.get(header).map(String::as_str).unwrap_or("")))
            .max()
            .unwrap_or(0);
        let width = (max_len.max(char_len(header)) + 2).min(50);
        ws.set_column_width(col_idx as u16, width as f64)?;
    }
    Ok(())
}

fn write_summary_sheet(ws: &mut Worksheet, rows: &[ReportRow]) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let headers: Vec<String> = rows[0].keys().cloned().collect();
    let header_fmt = header_format();
    for (col_idx, header) in headers.iter().enumerate() {
        ws.write_string(0, col_idx as u16, header, &header_fmt)?;
    }

    let default_fmt = default_format();
    let section_fmt = section_format();
    let blank_fmt = blank_format();

    for (i, row) in rows.iter().enumerate() {
        let row_idx = (i + 1) as u32;
        if is_section_header(row) {
            let mut display = row.clone();
            let stripped = display.get("Kennzahl").unwrap()[2..].to_string();
            display.insert("Kennzahl".to_string(), stripped);
            write_row(ws, row_idx, &headers, &display, &section_fmt)?;
        } else if is_blank_row(row) {
            write_row(ws, row_idx, &headers, row, &blank_fmt)?;
        } else {
            write_row(ws, row_idx, &headers, row, &default_fmt)?;
        }
    }

    set_column_widths_for_rows(ws, &headers, rows)
}

/// Writer for the vertical USt layout.
///
/// Row type is inferred from the `Zeitraum` column:
///   - "YYYY-MM" -> monthly   (no fill)
///   - "YYYY QN" -> quarterly -> subtotal fill + bold
///   - "YYYY"    -> annual    -> total fill + bold
///   - all empty -> blank separator
fn write_ust_sheet(ws: &mut Worksheet, rows: &[ReportRow]) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let headers: Vec<String> = rows[0].keys().cloned().collect();
    let header_fmt = header_format();
    for (col_idx, header) in headers.iter().enumerate() {
        ws.write_string(0, col_idx as u16, header, &header_fmt)?;
    }

    let default_fmt = default_format();
    let blank_fmt = blank_format();
    let subtotal_fmt = subtotal_format();
    let total_fmt = total_format();

    for (i, row) in rows.iter().enumerate() {
        let row_idx = (i + 1) as u32;
        let zeitraum = row.get("Zeitraum").map(String::as_str).unwrap_or("");
        let format = if is_blank_row(row) {
            &blank_fmt
        } else if zeitraum.contains('Q') {
            &subtotal_fmt
        } else if !zeitraum.contains('-') && !zeitraum.is_empty() {
            &total_fmt
        } else {
            &default_fmt
        };
        write_row(ws, row_idx, &headers, row, format)?;
    }

    set_column_widths_for_rows(ws, &headers, rows)
}

fn write_trail_sheet(ws: &mut Worksheet, sheet: &TrailSheet) -> Result<()> {
    let header_fmt = header_format_plain();
    for (col_idx, header) in sheet.headers.iter().enumerate() {
        ws.write_string(0, col_idx as u16, header, &header_fmt)?;
    }

    let default_fmt = default_format();
    let bold_fmt = bold_format();
    let subtotal_fmt = subtotal_format();
    let total_fmt = total_format();

    for (i, trail_row) in sheet.rows.iter().enumerate() {
        let row_idx = (i + 1) as u32;
        // `fill` (subtotal/total) takes precedence since it already implies bold;
        // `bold` is otherwise applied independently, matching the Python writer's
        // two separate `if trail_row.bold` / `if trail_row.fill == ...` checks.
        let format = match (trail_row.fill.as_str(), trail_row.bold) {
            ("subtotal", _) => &subtotal_fmt,
            ("total", _) => &total_fmt,
            (_, true) => &bold_fmt,
            _ => &default_fmt,
        };
        for (col_idx, cell) in trail_row.cells.iter().enumerate() {
            ws.write_string(row_idx, col_idx as u16, cell, format)?;
        }
    }

    for (col_idx, header) in sheet.headers.iter().enumerate() {
        let max_len = sheet
            .rows
            .iter()
            .filter_map(|r| r.cells.get(col_idx))
            .map(|c| char_len(c))
            .max()
            .unwrap_or(0);
        let width = (max_len.max(char_len(header)) + 2).min(50);
        ws.set_column_width(col_idx as u16, width as f64)?;
    }
    Ok(())
}

// ── CSV writers ──────────────────────────────────────────────────────────

fn write_rows_csv(
    path: &Path,
    rows: &[ReportRow],
    touched_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if rows.is_empty() {
        return Ok(());
    }
    let headers: Vec<String> = rows[0].keys().cloned().collect();
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_path(path)?;
    writer.write_record(&headers)?;
    for row in rows {
        let record: Vec<&str> = headers
            .iter()
            .map(|h| row.get(h).map(String::as_str).unwrap_or(""))
            .collect();
        writer.write_record(&record)?;
    }
    writer.flush()?;
    touched_files.insert(path.canonicalize()?);
    Ok(())
}

fn write_trail_csv(
    path: &Path,
    sheet: &TrailSheet,
    touched_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_path(path)?;
    writer.write_record(&sheet.headers)?;
    for row in &sheet.rows {
        writer.write_record(&row.cells)?;
    }
    writer.flush()?;
    touched_files.insert(path.canonicalize()?);
    Ok(())
}

fn save_xlsx(
    path: &Path,
    mut workbook: Workbook,
    touched_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = workbook.close_to_buffer()?;
    let stabilized = stabilize_xlsx(raw)?;
    std::fs::write(path, stabilized)?;
    touched_files.insert(path.canonicalize()?);
    Ok(())
}

fn unlink_if_exists(path: &Path, touched_files: &mut HashSet<PathBuf>) -> Result<()> {
    if path.exists() {
        let canonical = path.canonicalize()?;
        std::fs::remove_file(path)?;
        touched_files.insert(canonical);
    }
    Ok(())
}

fn warn_about_untouched_files(data_dir: &Path, touched_files: &HashSet<PathBuf>) {
    if !data_dir.exists() {
        return;
    }
    let mut untouched: Vec<PathBuf> = walkdir::WalkDir::new(data_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.path().canonicalize().ok())
        .filter(|p| !touched_files.contains(p))
        .collect();
    if untouched.is_empty() {
        return;
    }
    untouched.sort();

    eprintln!();
    eprintln!(
        "Warning: untouched files remain in ELSTER export directory. \
         Consider emptying the export directory and running the tool again."
    );
    for path in &untouched {
        eprintln!("  {}", path.display());
    }
    eprintln!();
}

// ── main ─────────────────────────────────────────────────────────────────

pub fn generate_report() -> Result<i32> {
    let journal_path = match paths::ledger_journal_path() {
        Some(path) => path,
        None => {
            eprintln!("No journal specified. Use -f/--file or set FINANCES_LEDGER_JOURNAL.");
            return Ok(1);
        }
    };
    let data_dir = paths::tax_data_dir();
    let config = config::load_config(paths::elster_config_path().as_deref())?;
    let mut touched_files: HashSet<PathBuf> = HashSet::new();

    if !journal_path.exists() {
        eprintln!("Journal not found: {}", journal_path.display());
        return Ok(1);
    }

    let dataset = enrich::build_dataset(&journal_path)?;
    let mut years: Vec<i32> = dataset.iter().map(|p| p.year()).collect();
    years.sort_unstable();
    years.dedup();

    for year in years {
        let base = data_dir.join(year.to_string());
        println!("  {year}");

        // ── steuererklaerung.xlsx + steuererklaerung/ CSVs ──────────────
        let euer = euer_rows(&dataset, year, &config);
        let ust = ust_rows(&dataset, year)?;
        let est = est_rows(&dataset, year);

        let mut workbook = Workbook::new_from_buffer();
        {
            let ws = workbook.add_worksheet();
            ws.set_name("EÜR")?;
            write_summary_sheet(ws, &euer)?;
        }
        {
            let ws = workbook.add_worksheet();
            ws.set_name("USt")?;
            write_ust_sheet(ws, &ust)?;
        }
        {
            let ws = workbook.add_worksheet();
            ws.set_name("ESt")?;
            write_summary_sheet(ws, &est)?;
        }
        save_xlsx(
            &base.join("steuererklaerung.xlsx"),
            workbook,
            &mut touched_files,
        )?;

        write_rows_csv(
            &base
                .join("steuererklaerung")
                .join("einnahmen-ueberschuss-rechnung.csv"),
            &euer,
            &mut touched_files,
        )?;
        write_rows_csv(
            &base.join("steuererklaerung").join("umsatzsteuer.csv"),
            &ust,
            &mut touched_files,
        )?;
        write_rows_csv(
            &base.join("steuererklaerung").join("einkommensteuer.csv"),
            &est,
            &mut touched_files,
        )?;

        // ── herleitung/ ──────────────────────────────────────────────────
        let all_herleitung = herleitung::herleitung_sheets(&dataset, year)?;

        for &form_key in FORM_KEYS {
            let sheets = match all_herleitung.get(form_key) {
                Some(sheets) if !sheets.is_empty() => sheets,
                _ => continue,
            };

            let mut workbook = Workbook::new_from_buffer();
            for sheet in sheets {
                let ws = workbook.add_worksheet();
                ws.set_name(&xlsx_sheet_title(&sheet.name))?;
                write_trail_sheet(ws, sheet)?;
            }
            save_xlsx(
                &base.join("herleitung").join(format!("{form_key}.xlsx")),
                workbook,
                &mut touched_files,
            )?;

            for sheet in sheets {
                write_trail_csv(
                    &base
                        .join("herleitung")
                        .join(form_key)
                        .join(tab_csv_name(&sheet.name)),
                    sheet,
                    &mut touched_files,
                )?;
            }
        }

        let ignored_sheets = all_herleitung.get("ignoriert").cloned().unwrap_or_default();
        if !ignored_sheets.is_empty() {
            let mut workbook = Workbook::new_from_buffer();
            for sheet in &ignored_sheets {
                let ws = workbook.add_worksheet();
                ws.set_name(&xlsx_sheet_title(&sheet.name))?;
                write_trail_sheet(ws, sheet)?;
            }
            save_xlsx(
                &base.join("herleitung").join("ignoriert.xlsx"),
                workbook,
                &mut touched_files,
            )?;
            for sheet in &ignored_sheets {
                write_trail_csv(
                    &base.join("herleitung").join(tab_csv_name(&sheet.name)),
                    sheet,
                    &mut touched_files,
                )?;
            }
        } else {
            unlink_if_exists(
                &base.join("herleitung").join("ignoriert.xlsx"),
                &mut touched_files,
            )?;
            unlink_if_exists(
                &base.join("herleitung").join("ignoriert.csv"),
                &mut touched_files,
            )?;
        }
    }

    warn_about_untouched_files(&data_dir, &touched_files);
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_csv_name_slugifies_unicode_and_collapses_separators() {
        assert_eq!(
            tab_csv_name("Serverkosten Wasabi"),
            "serverkosten-wasabi.csv"
        );
        assert_eq!(
            tab_csv_name("§13b Reverse Charge"),
            "13b-reverse-charge.csv"
        );
        // herleitung::unique_name truncates sheet names to 31 chars *before* this
        // function ever sees them, so a 36-char label like
        // "Langzeit-Auslandskrankenversicherung" arrives here already truncated to
        // "Langzeit-Auslandskrankenversich" (31 chars) -- verified end-to-end below.
        assert_eq!(
            tab_csv_name("Langzeit-Auslandskrankenversich"),
            "langzeit-auslandskrankenversich.csv"
        );
    }

    #[test]
    fn tab_csv_name_falls_back_to_sheet_when_nothing_alnum_remains() {
        assert_eq!(tab_csv_name("§€$"), "sheet.csv");
    }

    #[test]
    fn xlsx_sheet_title_replaces_invalid_chars_and_truncates() {
        assert_eq!(xlsx_sheet_title("A/B:C"), "A-B-C");
        let long_name = "x".repeat(40);
        assert_eq!(xlsx_sheet_title(&long_name).chars().count(), 31);
    }

    #[test]
    fn generate_report_writes_expected_example_outputs() {
        let _guard = crate::paths::ENV_LOCK.lock().unwrap();

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let journal = manifest_dir.join("examples/ledger/hledger.journal");
        let out_dir = tempfile::tempdir().unwrap();

        std::env::set_var("FINANCES_LEDGER_JOURNAL", &journal);
        std::env::set_var("FINANCES_TAX_DATA_DIR", out_dir.path());
        std::env::remove_var("HLEDGER_ELSTER_CONFIG");

        let exit_code = generate_report().unwrap();

        std::env::remove_var("FINANCES_LEDGER_JOURNAL");
        std::env::remove_var("FINANCES_TAX_DATA_DIR");

        assert_eq!(exit_code, 0);

        let euer_2024 = crate::csv_import::read_report_rows(
            &out_dir
                .path()
                .join("2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv"),
        )
        .unwrap();
        let row = |rows: &[ReportRow], label: &str| {
            rows.iter()
                .find(|r| r.get("Kennzahl").map(String::as_str) == Some(label))
                .unwrap()
                .clone()
        };
        assert_eq!(
            row(&euer_2024, "Umsatzsteuerpflichtige Betriebseinnahmen")["2024"],
            "1000.00"
        );
        assert_eq!(
            row(&euer_2024, "Steuerpflichtiger Gewinn/Verlust")["2024"],
            "-824.22"
        );

        let ust_2024 = crate::csv_import::read_report_rows(
            &out_dir
                .path()
                .join("2024/steuererklaerung/umsatzsteuer.csv"),
        )
        .unwrap();
        let annual = ust_2024
            .iter()
            .find(|r| r.get("Zeitraum").map(String::as_str) == Some("2024"))
            .unwrap();
        assert_eq!(annual["Bereits Entrichtet"], "190.00");

        assert!(out_dir
            .path()
            .join("2024/herleitung/einkommensteuer/langzeit-auslandskrankenversich.csv")
            .exists());

        // No ignored postings in the example journal -- no ignoriert files should exist.
        assert!(!out_dir
            .path()
            .join("2024/herleitung/ignoriert.xlsx")
            .exists());

        // Every generated path must be pure printable ASCII (matches the Python
        // suite's export-hygiene requirement for portable filenames).
        for entry in walkdir::WalkDir::new(out_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let name = entry.file_name().to_string_lossy();
            assert!(
                name.chars().all(|c| (' '..='~').contains(&c)),
                "non-ASCII path: {name}"
            );
        }
    }

    #[test]
    fn csv_round_trip_preserves_report_rows_and_trail_sheets() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let journal = manifest_dir.join("examples/ledger/hledger.journal");
        let out_dir = tempfile::tempdir().unwrap();
        let year = 2024;

        let config = config::load_config(None).unwrap();
        let dataset = enrich::build_dataset(&journal).unwrap();

        let euer = euer_rows(&dataset, year, &config);
        let ust = ust_rows(&dataset, year).unwrap();
        let est = est_rows(&dataset, year);
        let all_herleitung = herleitung::herleitung_sheets(&dataset, year).unwrap();

        let mut touched = HashSet::new();

        let euer_path = out_dir.path().join("euer.csv");
        write_rows_csv(&euer_path, &euer, &mut touched).unwrap();
        assert_eq!(
            crate::csv_import::read_report_rows(&euer_path).unwrap(),
            euer
        );

        let ust_path = out_dir.path().join("ust.csv");
        write_rows_csv(&ust_path, &ust, &mut touched).unwrap();
        assert_eq!(crate::csv_import::read_report_rows(&ust_path).unwrap(), ust);

        if !est.is_empty() {
            let est_path = out_dir.path().join("est.csv");
            write_rows_csv(&est_path, &est, &mut touched).unwrap();
            assert_eq!(crate::csv_import::read_report_rows(&est_path).unwrap(), est);
        }

        let mut checked_any_trail_sheet = false;
        for (form_key, sheets) in all_herleitung.iter() {
            for sheet in sheets {
                let path = out_dir
                    .path()
                    .join(format!("{form_key}-{}.csv", sheet.name));
                write_trail_csv(&path, sheet, &mut touched).unwrap();
                let read_back =
                    crate::csv_import::read_trail_sheet(&path, sheet.name.clone()).unwrap();
                // `outline_level` is dead-code row-hierarchy metadata that is
                // never written to CSV (see csv_import::read_trail_sheet) and
                // therefore can't round-trip -- everything else must match
                // exactly.
                let mut expected = sheet.clone();
                for row in &mut expected.rows {
                    row.outline_level = 0;
                }
                assert_eq!(read_back, expected);
                checked_any_trail_sheet = true;
            }
        }
        assert!(
            checked_any_trail_sheet,
            "expected at least one Herleitung sheet in the example journal"
        );
    }

    #[test]
    fn stabilize_xlsx_is_deterministic_across_runs() {
        let mut workbook = Workbook::new_from_buffer();
        let ws = workbook.add_worksheet();
        ws.set_name("Test").unwrap();
        let fmt = default_format();
        ws.write_string(0, 0, "hello", &fmt).unwrap();
        let raw = workbook.close_to_buffer().unwrap();
        let first = stabilize_xlsx(raw.clone()).unwrap();
        // A second stabilization pass over the same raw bytes must be identical.
        let second = stabilize_xlsx(raw).unwrap();
        assert_eq!(first, second);
    }
}
