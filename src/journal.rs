use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("failed to run hledger: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("hledger exited with an error:\n{0}")]
    HledgerFailed(String),
    #[error("failed to parse hledger JSON output: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    #[serde(default)]
    pub tcomment: String,
    pub tdate: String,
    #[serde(default)]
    pub tdescription: String,
    #[serde(default)]
    pub tpostings: Vec<Posting>,
    #[serde(default)]
    pub tsourcepos: Vec<SourcePos>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Posting {
    pub paccount: String,
    #[serde(default)]
    pub pamount: Vec<Amount>,
    #[serde(default)]
    pub pcomment: Option<String>,
    #[serde(default)]
    pub pdate: Option<String>,
    #[serde(default)]
    pub ptags: Vec<(String, String)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Amount {
    pub aquantity: Quantity,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Quantity {
    #[serde(rename = "decimalMantissa")]
    pub decimal_mantissa: i64,
    #[serde(rename = "decimalPlaces")]
    pub decimal_places: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourcePos {
    #[serde(rename = "sourceName")]
    pub source_name: String,
    #[serde(rename = "sourceLine")]
    pub source_line: u32,
}

impl Posting {
    /// hledger lists the most-specific account's tags first; first occurrence wins.
    pub fn tags(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (key, value) in &self.ptags {
            result.entry(key.clone()).or_insert_with(|| value.clone());
        }
        result
    }

    pub fn tag_values(&self, key: &str) -> Vec<String> {
        let mut values = Vec::new();
        for (tag_key, value) in &self.ptags {
            if tag_key == key && !values.contains(value) {
                values.push(value.clone());
            }
        }
        values
    }
}

pub fn load_transactions(journal_path: &Path) -> Result<Vec<Transaction>, JournalError> {
    let output = Command::new("hledger")
        .arg("-f")
        .arg(journal_path)
        .args(["print", "--output-format", "json"])
        .output()
        .map_err(JournalError::Spawn)?;

    if !output.status.success() {
        return Err(JournalError::HledgerFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let transactions: Vec<Transaction> = serde_json::from_slice(&output.stdout)?;
    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn posting_with_tags(tags: &[(&str, &str)]) -> Posting {
        Posting {
            paccount: "assets:bank".to_string(),
            pamount: vec![],
            pcomment: None,
            pdate: None,
            ptags: tags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn tags_keeps_first_occurrence_when_duplicated() {
        let posting = posting_with_tags(&[("elster_item", "specific"), ("elster_item", "general")]);
        assert_eq!(
            posting.tags().get("elster_item").map(String::as_str),
            Some("specific")
        );
    }

    #[test]
    fn tag_values_deduplicates_but_preserves_multiple_distinct_values() {
        let posting = posting_with_tags(&[
            ("elster_form", "einnahmenueberschussrechnung"),
            ("elster_form", "einkommensteuer"),
            ("elster_form", "einnahmenueberschussrechnung"),
        ]);
        assert_eq!(
            posting.tag_values("elster_form"),
            vec![
                "einnahmenueberschussrechnung".to_string(),
                "einkommensteuer".to_string()
            ]
        );
    }
}
