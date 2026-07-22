use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitMetadata {
    pub commit_hash: String,
    pub commit_date: String,
}

#[derive(Debug, Error)]
pub enum CommitEvidenceError {
    #[error("failed to run git: {0}")]
    GitIo(#[from] std::io::Error),
    #[error("not inside a Git repository")]
    NotGitRepository,
    #[error("Git command failed: {0}")]
    GitCommand(String),
    #[error("working tree has uncommitted changes; commit or stash them before exporting commit evidence")]
    DirtyWorkingTree,
    #[error("commit evidence output must be outside the Git repository: {0}")]
    OutputInsideRepository(PathBuf),
    #[error("failed to write commit evidence PDF at {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

fn run_git(args: &[&str]) -> Result<String, CommitEvidenceError> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if args == ["rev-parse", "--show-toplevel"] {
            return Err(CommitEvidenceError::NotGitRepository);
        }
        return Err(CommitEvidenceError::GitCommand(stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn current_repository_root() -> Result<PathBuf, CommitEvidenceError> {
    Ok(PathBuf::from(run_git(&["rev-parse", "--show-toplevel"])?))
}

pub fn current_git_metadata() -> Result<GitMetadata, CommitEvidenceError> {
    current_repository_root()?;
    let status = run_git(&["status", "--porcelain", "--untracked-files=all"])?;
    if !status.is_empty() {
        return Err(CommitEvidenceError::DirtyWorkingTree);
    }

    let metadata = run_git(&["log", "-1", "--format=%H%n%cI", "HEAD"])?;
    let mut lines = metadata.lines();
    let commit_hash = lines
        .next()
        .ok_or_else(|| CommitEvidenceError::GitCommand("missing commit hash".to_string()))?
        .to_string();
    let commit_date = lines
        .next()
        .ok_or_else(|| CommitEvidenceError::GitCommand("missing commit date".to_string()))?
        .to_string();
    Ok(GitMetadata {
        commit_hash,
        commit_date,
    })
}

fn win_ansi_encoding(ch: char) -> Option<u8> {
    match ch {
        '\u{2022}' => Some(0x80),
        '\u{2020}' => Some(0x81),
        '\u{2021}' => Some(0x82),
        '\u{2026}' => Some(0x83),
        '\u{2014}' => Some(0x84),
        '\u{2013}' => Some(0x85),
        '\u{0192}' => Some(0x86),
        '\u{2044}' => Some(0x87),
        '\u{2039}' => Some(0x88),
        '\u{203A}' => Some(0x89),
        '\u{2212}' => Some(0x8A),
        '\u{2030}' => Some(0x8B),
        '\u{201E}' => Some(0x8C),
        '\u{201C}' => Some(0x8D),
        '\u{201D}' => Some(0x8E),
        '\u{2018}' => Some(0x8F),
        '\u{2019}' => Some(0x90),
        '\u{201A}' => Some(0x91),
        '\u{2122}' => Some(0x92),
        '\u{FB01}' => Some(0x93),
        '\u{FB02}' => Some(0x94),
        '\u{0141}' => Some(0x95),
        '\u{0152}' => Some(0x96),
        '\u{0160}' => Some(0x97),
        '\u{0178}' => Some(0x98),
        '\u{017D}' => Some(0x99),
        '\u{0131}' => Some(0x9A),
        '\u{0142}' => Some(0x9B),
        '\u{0153}' => Some(0x9C),
        '\u{0161}' => Some(0x9D),
        '\u{017E}' => Some(0x9E),
        '\u{20AC}' => Some(0xA0),
        _ if ('\u{00A1}'..='\u{00FF}').contains(&ch) => Some(ch as u8),
        _ if ch.is_ascii() => Some(ch as u8),
        _ => None,
    }
}

fn escape_pdf_text(text: &str) -> String {
    text.chars().fold(String::new(), |mut out, ch| {
        match win_ansi_encoding(ch) {
            Some(b'\\') => out.push_str("\\\\"),
            Some(b'(') => out.push_str("\\("),
            Some(b')') => out.push_str("\\)"),
            Some(b'\n') => out.push_str("\\n"),
            Some(b'\r') => {}
            Some(byte) if byte.is_ascii_graphic() || byte == b' ' => out.push(byte as char),
            Some(byte) => {
                let _ = write!(out, "\\{byte:03o}");
            }
            None => out.push('?'),
        }
        out
    })
}

fn text_line(x: i32, y: i32, font: &str, size: i32, text: &str) -> String {
    format!(
        "BT /{font} {size} Tf {x} {y} Td ({}) Tj ET\n",
        escape_pdf_text(text)
    )
}

pub fn pdf_bytes(metadata: &GitMetadata) -> Vec<u8> {
    const GOBD_URL: &str = "https://ao.bundesfinanzministerium.de/ao/2023/Anhaenge/BMF-Schreiben-und-gleichlautende-Laendererlasse/Anhang-64/inhalt.html";

    let lines = [
        (72, 760, "F1", 18, "Git-Commit-Nachweis"),
        (
            72,
            720,
            "F1",
            11,
            "Dieses Dokument belegt den Git-Commit des Repository,",
        ),
        (
            72,
            704,
            "F1",
            11,
            "das für die Erstellung der elektronischen Steuererklärung verwendet wurde.",
        ),
        (
            72,
            676,
            "F1",
            11,
            "Bezug: GoBD - Grundsätze zur ordnungsmäßigen Führung und",
        ),
        (
            72,
            660,
            "F1",
            11,
            "Aufbewahrung von Büchern, Aufzeichnungen und Unterlagen in",
        ),
        (
            72,
            644,
            "F1",
            11,
            "elektronischer Form sowie zum Datenzugriff.",
        ),
        (
            72,
            576,
            "F1",
            11,
            "Ziele: Unveränderbarkeit und Nachprüfbarkeit der zugrunde liegenden Daten.",
        ),
        (72, 536, "F1", 11, "Commit-Hash:"),
        (72, 514, "F2", 10, metadata.commit_hash.as_str()),
        (72, 472, "F1", 11, "Commit-Datum:"),
        (72, 450, "F2", 10, metadata.commit_date.as_str()),
    ];

    let mut content = String::new();
    for (x, y, font, size, text) in lines {
        content.push_str(&text_line(x, y, font, size, text));
    }

    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.28 841.89] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 6 0 R /Annots [7 0 R 8 0 R 9 0 R] >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>"
            .to_string(),
        format!("<< /Length {} >>\nstream\n{}endstream", content.len(), content),
        format!(
            "<< /Type /Annot /Subtype /Link /Rect [72 673 502 687] /Border [0 0 0] /A << /S /URI /URI ({}) >> >>",
            escape_pdf_text(GOBD_URL)
        ),
        format!(
            "<< /Type /Annot /Subtype /Link /Rect [72 657 520 671] /Border [0 0 0] /A << /S /URI /URI ({}) >> >>",
            escape_pdf_text(GOBD_URL)
        ),
        format!(
            "<< /Type /Annot /Subtype /Link /Rect [72 641 333 655] /Border [0 0 0] /A << /S /URI /URI ({}) >> >>",
            escape_pdf_text(GOBD_URL)
        ),
    ];

    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (idx, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        let _ = write!(pdf, "{} 0 obj\n{}\nendobj\n", idx + 1, object);
    }

    let xref_start = pdf.len();
    let _ = write!(pdf, "xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        let _ = writeln!(pdf, "{offset:010} 00000 n ");
    }
    let _ = write!(
        pdf,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        objects.len() + 1,
        xref_start
    );
    pdf.into_bytes()
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                resolved.pop();
            }
            _ => resolved.push(component.as_os_str()),
        }
    }
    resolved
}

fn ensure_output_outside_repository(
    path: &Path,
    repository_root: &Path,
) -> Result<(), CommitEvidenceError> {
    let output = normalize_path(path);
    let repository_root = normalize_path(repository_root);
    if output.starts_with(&repository_root) {
        return Err(CommitEvidenceError::OutputInsideRepository(output));
    }
    Ok(())
}

pub fn write_commit_evidence(path: &Path) -> Result<(), CommitEvidenceError> {
    let repository_root = current_repository_root()?;
    ensure_output_outside_repository(path, &repository_root)?;
    let metadata = current_git_metadata()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CommitEvidenceError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(path, pdf_bytes(&metadata)).map_err(|source| CommitEvidenceError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> GitMetadata {
        GitMetadata {
            commit_hash: "b8f5c39ef50bb49d6924a56c1827a83ce1033854".to_string(),
            commit_date: "2026-07-22T10:11:12+02:00".to_string(),
        }
    }

    #[test]
    fn pdf_is_deterministic_for_same_metadata() {
        assert_eq!(pdf_bytes(&metadata()), pdf_bytes(&metadata()));
    }

    #[test]
    fn pdf_contains_copyable_commit_hash_and_german_context() {
        let pdf = String::from_utf8(pdf_bytes(&metadata())).unwrap();
        assert!(pdf.starts_with("%PDF-"));
        assert!(pdf.contains("/MediaBox [0 0 595.28 841.89]"));
        assert!(pdf.contains("/Encoding /WinAnsiEncoding"));
        assert!(pdf.contains("b8f5c39ef50bb49d6924a56c1827a83ce1033854"));
        assert!(pdf.contains("Git-Commit-Nachweis"));
        assert!(pdf.contains("GoBD"));
        assert!(!pdf.contains("Quelle:"));
        assert!(!pdf.contains("ao.bundesfinanzministerium.de/ao/2023/Anhaenge/) Tj"));
        assert!(pdf.contains("ao.bundesfinanzministerium.de"));
        assert!(pdf.contains("/Subtype /Link"));
        assert!(pdf.contains("/S /URI"));
        assert!(pdf.contains("\\344nderbarkeit und Nachpr\\374fbarkeit"));
        assert!(!pdf.contains("/tmp/tax-repo"));
    }

    #[test]
    fn output_inside_repository_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let repository = tmp.path().join("tax-repo");
        std::fs::create_dir(&repository).unwrap();

        let err =
            ensure_output_outside_repository(&repository.join("commit-evidence.pdf"), &repository)
                .unwrap_err();
        assert!(matches!(
            err,
            CommitEvidenceError::OutputInsideRepository(_)
        ));
    }

    #[test]
    fn output_outside_repository_is_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let repository = tmp.path().join("tax-repo");
        std::fs::create_dir(&repository).unwrap();

        ensure_output_outside_repository(&tmp.path().join("commit-evidence.pdf"), &repository)
            .unwrap();
    }
}
