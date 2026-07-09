//! Reads back the CSV files this crate writes (see `report_writer`) into the
//! same in-memory `ReportRow` / `TrailSheet` values the export pipeline
//! builds before writing. Exists to guarantee CSV-on-disk and in-memory
//! representation never drift apart — see AGENTS.md's
//! "CSV/xlsx equivalence invariant".

use crate::herleitung::{TrailRow, TrailSheet};
use crate::periods::ReportRow;
use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsvImportError {
    #[error("failed to read CSV at {path}: {source}")]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },
}

fn open(path: &Path) -> Result<csv::Reader<std::fs::File>, CsvImportError> {
    csv::Reader::from_path(path).map_err(|source| CsvImportError::Csv {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads a `ReportRow` CSV (as written by `report_writer::write_rows_csv`)
/// back into `Vec<ReportRow>`, preserving column order via `IndexMap`.
///
/// Note: if the original `Vec<ReportRow>` was empty, `write_rows_csv` never
/// creates the file at all — callers must not call this on a path that was
/// never written for an empty row set.
pub fn read_report_rows(path: &Path) -> Result<Vec<ReportRow>, CsvImportError> {
    let to_err = |source: csv::Error| CsvImportError::Csv {
        path: path.to_path_buf(),
        source,
    };

    let mut reader = open(path)?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(to_err)?
        .iter()
        .map(str::to_string)
        .collect();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(to_err)?;
        let row: ReportRow = headers
            .iter()
            .cloned()
            .zip(record.iter().map(str::to_string))
            .collect::<IndexMap<_, _>>();
        rows.push(row);
    }
    Ok(rows)
}

/// "" | "subtotal" | "total", re-derived from `cells[0]` per the
/// GESAMT / "Σ "-prefix convention used by every herleitung.rs sheet builder.
fn classify_trail_row(cells: &[String]) -> (bool, String) {
    match cells.first().map(String::as_str) {
        Some("GESAMT") => (true, "total".to_string()),
        Some(s) if s.starts_with("Σ ") => (true, "subtotal".to_string()),
        _ => (false, String::new()),
    }
}

/// Reads a `TrailSheet` CSV (as written by `report_writer::write_trail_csv`)
/// back into a `TrailSheet`. `outline_level` is not recoverable from CSV
/// (dead-code metadata, never written) and defaults to `0`; `name` must be
/// supplied by the caller since it isn't stored in the CSV itself (the
/// on-disk filename is a lossy, truncated slug of it — see `tab_csv_name`
/// in `report_writer`).
pub fn read_trail_sheet(
    path: &Path,
    name: impl Into<String>,
) -> Result<TrailSheet, CsvImportError> {
    let to_err = |source: csv::Error| CsvImportError::Csv {
        path: path.to_path_buf(),
        source,
    };

    let mut reader = open(path)?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(to_err)?
        .iter()
        .map(str::to_string)
        .collect();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(to_err)?;
        let cells: Vec<String> = record.iter().map(str::to_string).collect();
        let (bold, fill) = classify_trail_row(&cells);
        rows.push(TrailRow {
            cells,
            outline_level: 0,
            bold,
            fill,
        });
    }
    Ok(TrailSheet {
        name: name.into(),
        headers,
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_csv(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn read_report_rows_preserves_column_order() {
        let f = write_temp_csv("Kennzahl,2024\nUmsatz,1000.00\n");
        let rows = read_report_rows(f.path()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].keys().collect::<Vec<_>>(), vec!["Kennzahl", "2024"]);
        assert_eq!(rows[0]["2024"], "1000.00");
    }

    #[test]
    fn read_trail_sheet_reclassifies_gesamt_and_subtotal_rows() {
        let f = write_temp_csv("Datum,Betrag\n2024-01-01,10.00\nΣ Bank,10.00\nGESAMT,10.00\n");
        let sheet = read_trail_sheet(f.path(), "Test").unwrap();
        assert_eq!(sheet.name, "Test");
        assert_eq!(
            (sheet.rows[0].bold, sheet.rows[0].fill.as_str()),
            (false, "")
        );
        assert_eq!(
            (sheet.rows[1].bold, sheet.rows[1].fill.as_str()),
            (true, "subtotal")
        );
        assert_eq!(
            (sheet.rows[2].bold, sheet.rows[2].fill.as_str()),
            (true, "total")
        );
    }
}
