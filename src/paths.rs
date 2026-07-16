use std::path::{Path, PathBuf};

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Makes `path` absolute, resolving relative paths against the current working
/// directory. Mirrors Python's `Path(value).resolve()` for the parts of that
/// call this tool relies on, without requiring the path to exist (unlike
/// `std::fs::canonicalize`) — output paths and not-yet-written config files
/// routinely don't exist yet.
pub fn resolve(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd().join(path)
    }
}

fn default_tax_data_dir() -> PathBuf {
    cwd().join("data").join("exports")
}

fn env_path(name: &str, default: PathBuf) -> PathBuf {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => resolve(Path::new(&value)),
        _ => default,
    }
}

/// No default: unlike `tax_data_dir`, there is no sensible fallback journal
/// location a real installation would have (this crate's own `examples/`
/// fixture is not something a user's checkout ever has). Callers must
/// require `-f`/`--file` or `FINANCES_LEDGER_JOURNAL` and error clearly when
/// neither is set, rather than silently resolving to a path that happens to
/// exist only in this repository.
pub fn ledger_journal_path() -> Option<PathBuf> {
    std::env::var("FINANCES_LEDGER_JOURNAL")
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| resolve(Path::new(&v)))
}

pub fn tax_data_dir() -> PathBuf {
    env_path("FINANCES_TAX_DATA_DIR", default_tax_data_dir())
}

pub fn elster_config_path() -> Option<PathBuf> {
    std::env::var("HLEDGER_ELSTER_CONFIG")
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| resolve(Path::new(&v)))
}

// Environment variables are process-global, so any test across the crate that
// touches FINANCES_LEDGER_JOURNAL / FINANCES_TAX_DATA_DIR / HLEDGER_ELSTER_CONFIG
// (or spawns the pipeline that reads them) must serialize on this lock.
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_leaves_absolute_paths_untouched() {
        let _guard = ENV_LOCK.lock().unwrap();
        let absolute = if cfg!(windows) {
            PathBuf::from(r"C:\tmp\journal.hledger")
        } else {
            PathBuf::from("/tmp/journal.hledger")
        };
        assert_eq!(resolve(&absolute), absolute);
    }

    #[test]
    fn resolve_joins_relative_paths_with_cwd() {
        let _guard = ENV_LOCK.lock().unwrap();
        let resolved = resolve(Path::new("foo/bar.journal"));
        assert!(resolved.is_absolute());
        assert!(resolved.ends_with("foo/bar.journal"));
    }

    #[test]
    fn ledger_journal_path_is_none_when_unset() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FINANCES_LEDGER_JOURNAL");
        assert_eq!(ledger_journal_path(), None);
    }

    #[test]
    fn ledger_journal_path_honors_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FINANCES_LEDGER_JOURNAL", "/tmp/custom.journal");
        let path = ledger_journal_path();
        std::env::remove_var("FINANCES_LEDGER_JOURNAL");
        assert_eq!(path, Some(PathBuf::from("/tmp/custom.journal")));
    }

    #[test]
    fn tax_data_dir_defaults_under_cwd_data_exports() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FINANCES_TAX_DATA_DIR");
        let path = tax_data_dir();
        assert!(path.ends_with("data/exports"));
    }

    #[test]
    fn elster_config_path_is_none_when_unset() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("HLEDGER_ELSTER_CONFIG");
        assert_eq!(elster_config_path(), None);
    }

    #[test]
    fn elster_config_path_honors_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("HLEDGER_ELSTER_CONFIG", "/tmp/elster.toml");
        let path = elster_config_path();
        std::env::remove_var("HLEDGER_ELSTER_CONFIG");
        assert_eq!(path, Some(PathBuf::from("/tmp/elster.toml")));
    }
}
