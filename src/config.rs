use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub const DEFAULT_CONFIG_TEXT: &str = r#"[euer.home_office_pauschale]
enabled = true
default_days = "max"
# Set per-year days when the default does not match your situation.
# 2020-2022: 5 EUR/day, capped at 600 EUR.
# 2023+: 6 EUR/day, capped at 1260 EUR.

[euer.home_office_pauschale.days]
# 2024 = 210
"#;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config not found: {0}")]
    NotFound(String),
    #[error("Config already exists: {0}")]
    AlreadyExists(String),
    #[error("Config key euer.home_office_pauschale must be a table")]
    HomeOfficeNotATable,
    #[error("Config key euer.home_office_pauschale.enabled must be a boolean")]
    EnabledNotABool,
    #[error("Config key euer.home_office_pauschale.default_days must be \"max\" or an integer")]
    DefaultDaysInvalid,
    #[error("Config key euer.home_office_pauschale.default_days must not be negative")]
    DefaultDaysNegative,
    #[error("Config key euer.home_office_pauschale.days must be a table")]
    DaysNotATable,
    #[error("Home-office days for {0} must be an integer")]
    DaysValueNotInteger(String),
    #[error("Home-office days for {0} must not be negative")]
    DaysValueNegative(String),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum DefaultDays {
    #[default]
    Max,
    Fixed(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HomeOfficePauschaleConfig {
    pub enabled: bool,
    pub default_days: DefaultDays,
    pub days_by_year: HashMap<i32, i64>,
}

impl Default for HomeOfficePauschaleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_days: DefaultDays::Max,
            days_by_year: HashMap::new(),
        }
    }
}

impl HomeOfficePauschaleConfig {
    pub fn amount_for_year(&self, year: i32) -> Decimal {
        let zero = Decimal::new(0, 2);
        if !self.enabled {
            return zero;
        }
        let Some((rate, cap, _max_days)) = home_office_policy(year) else {
            return zero;
        };

        let days = match self.days_by_year.get(&year).copied() {
            Some(days) => days,
            None => match self.default_days {
                DefaultDays::Max => return cap,
                DefaultDays::Fixed(days) => days,
            },
        };

        let amount = Decimal::from(days) * rate;
        amount
            .min(cap)
            .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaxConfig {
    pub home_office_pauschale: HomeOfficePauschaleConfig,
}

pub fn load_config(path: Option<&Path>) -> Result<TaxConfig, ConfigError> {
    let Some(path) = path else {
        return Ok(TaxConfig::default());
    };
    if !path.exists() {
        return Err(ConfigError::NotFound(path.display().to_string()));
    }
    let raw_text = std::fs::read_to_string(path)?;
    let raw: toml::Value = raw_text.parse()?;
    parse_config(&raw)
}

pub fn write_default_config(path: &Path, force: bool) -> Result<(), ConfigError> {
    if path.exists() && !force {
        return Err(ConfigError::AlreadyExists(path.display().to_string()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, DEFAULT_CONFIG_TEXT)?;
    Ok(())
}

fn table_get(value: &toml::Value, key: &str) -> Option<toml::Value> {
    value.as_table()?.get(key).cloned()
}

fn parse_config(raw: &toml::Value) -> Result<TaxConfig, ConfigError> {
    let home_office_raw =
        table_get(raw, "euer").and_then(|euer| table_get(&euer, "home_office_pauschale"));

    let home_office_table = match home_office_raw {
        None => toml::map::Map::new(),
        Some(v) => v
            .as_table()
            .ok_or(ConfigError::HomeOfficeNotATable)?
            .clone(),
    };

    let enabled = match home_office_table.get("enabled") {
        None => true,
        Some(toml::Value::Boolean(b)) => *b,
        Some(_) => return Err(ConfigError::EnabledNotABool),
    };

    let default_days = match home_office_table.get("default_days") {
        None => DefaultDays::Max,
        Some(toml::Value::String(s)) if s == "max" => DefaultDays::Max,
        Some(toml::Value::String(_)) => return Err(ConfigError::DefaultDaysInvalid),
        Some(toml::Value::Integer(n)) if *n < 0 => return Err(ConfigError::DefaultDaysNegative),
        Some(toml::Value::Integer(n)) => DefaultDays::Fixed(*n),
        Some(_) => return Err(ConfigError::DefaultDaysInvalid),
    };

    let days_table = match home_office_table.get("days").cloned() {
        None => toml::map::Map::new(),
        Some(v) => v.as_table().ok_or(ConfigError::DaysNotATable)?.clone(),
    };

    let mut days_by_year = HashMap::new();
    for (year_raw, days_value) in &days_table {
        let days = match days_value {
            toml::Value::Integer(n) => *n,
            _ => return Err(ConfigError::DaysValueNotInteger(year_raw.clone())),
        };
        if days < 0 {
            return Err(ConfigError::DaysValueNegative(year_raw.clone()));
        }
        let year: i32 = year_raw
            .parse()
            .map_err(|_| ConfigError::DaysValueNotInteger(year_raw.clone()))?;
        days_by_year.insert(year, days);
    }

    Ok(TaxConfig {
        home_office_pauschale: HomeOfficePauschaleConfig {
            enabled,
            default_days,
            days_by_year,
        },
    })
}

fn home_office_policy(year: i32) -> Option<(Decimal, Decimal, i32)> {
    if (2020..=2022).contains(&year) {
        Some((Decimal::new(500, 2), Decimal::new(60000, 2), 120))
    } else if year >= 2023 {
        Some((Decimal::new(600, 2), Decimal::new(126000, 2), 210))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn load_config_with_no_path_returns_defaults() {
        let config = load_config(None).unwrap();
        assert!(config.home_office_pauschale.enabled);
        assert_eq!(config.home_office_pauschale.default_days, DefaultDays::Max);
        assert!(config.home_office_pauschale.days_by_year.is_empty());
    }

    #[test]
    fn load_config_missing_file_errors() {
        let err = load_config(Some(Path::new("/nonexistent/elster.toml"))).unwrap_err();
        assert!(matches!(err, ConfigError::NotFound(_)));
    }

    #[test]
    fn amount_for_year_defaults_to_cap_when_days_unspecified() {
        let config = HomeOfficePauschaleConfig::default();
        assert_eq!(
            config.amount_for_year(2021),
            Decimal::from_str("600.00").unwrap()
        );
        assert_eq!(
            config.amount_for_year(2024),
            Decimal::from_str("1260.00").unwrap()
        );
    }

    #[test]
    fn amount_for_year_disabled_returns_zero() {
        let config = HomeOfficePauschaleConfig {
            enabled: false,
            ..Default::default()
        };
        assert_eq!(
            config.amount_for_year(2024),
            Decimal::from_str("0.00").unwrap()
        );
    }

    #[test]
    fn amount_for_year_outside_policy_window_returns_zero() {
        let config = HomeOfficePauschaleConfig::default();
        assert_eq!(
            config.amount_for_year(2019),
            Decimal::from_str("0.00").unwrap()
        );
    }

    #[test]
    fn amount_for_year_uses_per_year_override_and_caps_it() {
        let mut days_by_year = HashMap::new();
        days_by_year.insert(2024, 300);
        let config = HomeOfficePauschaleConfig {
            enabled: true,
            default_days: DefaultDays::Max,
            days_by_year,
        };
        // 300 days * 6.00 EUR = 1800.00, capped at 1260.00
        assert_eq!(
            config.amount_for_year(2024),
            Decimal::from_str("1260.00").unwrap()
        );
    }

    #[test]
    fn amount_for_year_uses_fixed_default_days() {
        let config = HomeOfficePauschaleConfig {
            enabled: true,
            default_days: DefaultDays::Fixed(100),
            days_by_year: HashMap::new(),
        };
        assert_eq!(
            config.amount_for_year(2024),
            Decimal::from_str("600.00").unwrap()
        );
    }

    #[test]
    fn parse_config_rejects_non_table_home_office_pauschale() {
        let raw: toml::Value = "euer.home_office_pauschale = 1".parse().unwrap();
        let err = parse_config(&raw).unwrap_err();
        assert!(matches!(err, ConfigError::HomeOfficeNotATable));
    }

    #[test]
    fn parse_config_rejects_non_bool_enabled() {
        let raw: toml::Value = "[euer.home_office_pauschale]\nenabled = 1".parse().unwrap();
        let err = parse_config(&raw).unwrap_err();
        assert!(matches!(err, ConfigError::EnabledNotABool));
    }

    #[test]
    fn parse_config_rejects_invalid_default_days_string() {
        let raw: toml::Value = "[euer.home_office_pauschale]\ndefault_days = \"lots\""
            .parse()
            .unwrap();
        let err = parse_config(&raw).unwrap_err();
        assert!(matches!(err, ConfigError::DefaultDaysInvalid));
    }

    #[test]
    fn parse_config_rejects_negative_default_days() {
        let raw: toml::Value = "[euer.home_office_pauschale]\ndefault_days = -1"
            .parse()
            .unwrap();
        let err = parse_config(&raw).unwrap_err();
        assert!(matches!(err, ConfigError::DefaultDaysNegative));
    }

    #[test]
    fn parse_config_rejects_negative_days_value() {
        let raw: toml::Value = "[euer.home_office_pauschale.days]\n\"2024\" = -5"
            .parse()
            .unwrap();
        let err = parse_config(&raw).unwrap_err();
        assert!(matches!(err, ConfigError::DaysValueNegative(_)));
    }

    #[test]
    fn parse_config_accepts_full_example() {
        let raw: toml::Value = DEFAULT_CONFIG_TEXT.parse().unwrap();
        let config = parse_config(&raw).unwrap();
        assert!(config.home_office_pauschale.enabled);
        assert_eq!(config.home_office_pauschale.default_days, DefaultDays::Max);
    }

    #[test]
    fn parse_config_reads_per_year_days() {
        let raw: toml::Value = "[euer.home_office_pauschale.days]\n\"2024\" = 210"
            .parse()
            .unwrap();
        let config = parse_config(&raw).unwrap();
        assert_eq!(
            config.home_office_pauschale.days_by_year.get(&2024),
            Some(&210)
        );
    }
}
