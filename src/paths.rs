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

fn default_ledger_journal() -> PathBuf {
    cwd()
        .join("examples")
        .join("ledger")
        .join("hledger.journal")
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

pub fn ledger_journal_path() -> PathBuf {
    env_path("FINANCES_LEDGER_JOURNAL", default_ledger_journal())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Environment variables are process-global, so serialize tests that touch them.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
    fn ledger_journal_path_defaults_under_cwd_examples() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("FINANCES_LEDGER_JOURNAL");
        let path = ledger_journal_path();
        assert!(path.ends_with("examples/ledger/hledger.journal"));
    }

    #[test]
    fn ledger_journal_path_honors_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("FINANCES_LEDGER_JOURNAL", "/tmp/custom.journal");
        let path = ledger_journal_path();
        std::env::remove_var("FINANCES_LEDGER_JOURNAL");
        assert_eq!(path, PathBuf::from("/tmp/custom.journal"));
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
