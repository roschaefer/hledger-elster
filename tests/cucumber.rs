use cucumber::{gherkin::Step, given, then, when, World};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::process::Command;

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

#[derive(Debug, World)]
#[world(init = ElsterWorld::new)]
pub struct ElsterWorld {
    _tmp: TempDir,
    work_dir: PathBuf,
    last_stdout: String,
    last_stderr: String,
    last_exit_code: i32,
}

impl ElsterWorld {
    async fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let work_dir = tmp
            .path()
            .canonicalize()
            .unwrap_or_else(|_| tmp.path().to_path_buf());
        Self {
            _tmp: tmp,
            work_dir,
            last_stdout: String::new(),
            last_stderr: String::new(),
            last_exit_code: 0,
        }
    }

    /// Resolves a scenario-relative path against the work dir, rejecting
    /// absolute paths and `..` components -- mirrors elster_steps.py's
    /// `_resolve_work_path` safety check.
    fn resolve(&self, path: &str) -> PathBuf {
        let relative = Path::new(path);
        assert!(
            !relative.is_absolute()
                && !relative
                    .components()
                    .any(|c| c == std::path::Component::ParentDir),
            "Unsafe scenario path: {path}"
        );
        self.work_dir.join(relative)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// Gherkin docstrings reserve their first line for an optional media-type
// annotation (`"""toml`), which the gherkin crate's raw `docstring` value
// always includes as a literal first line -- empty when no media type is
// given (confirmed empirically: none of this suite's docstrings use a media
// type, and the raw value always starts with a leading "\n"). Drop that line
// unconditionally, then strip common leading whitespace from the rest,
// matching the behavior of Python behave's `context.text`.
fn docstring(step: &Step) -> String {
    let raw = step.docstring.as_deref().unwrap_or("");
    let lines: Vec<&str> = raw.split('\n').collect();
    let content = if lines.is_empty() {
        &lines[..]
    } else {
        &lines[1..]
    };

    let min_indent = content
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let result = content
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    result.trim_end_matches('\n').to_string()
}

fn gherkin_table(step: &Step) -> Vec<Vec<String>> {
    let table = step.table.as_ref().expect("step requires a table");
    table.rows.clone()
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

#[given(regex = r#"^a file named "([^"]+)" with content:$"#)]
async fn write_file(world: &mut ElsterWorld, step: &Step, path: String) {
    let content = docstring(step);
    let target = world.resolve(&path);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, format!("{content}\n")).unwrap();
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when(regex = r#"^I run "([^"]+)"$"#)]
async fn run_command(world: &mut ElsterWorld, command: String) {
    let args: Vec<&str> = command.split_whitespace().collect();
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_hledger-elster"));
    let (program, rest): (PathBuf, &[&str]) =
        if args.len() >= 2 && args[0] == "hledger" && args[1] == "elster" {
            (bin, &args[2..])
        } else if !args.is_empty() && args[0] == "hledger-elster" {
            (bin, &args[1..])
        } else {
            panic!("Unsupported command: {command}");
        };

    let output = Command::new(program)
        .args(rest)
        .current_dir(&world.work_dir)
        .output()
        .await
        .expect("failed to run command");

    world.last_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    world.last_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    world.last_exit_code = output.status.code().unwrap_or(-1);

    assert!(
        output.status.success(),
        "command failed (exit {}): {command}\nstdout:\n{}\nstderr:\n{}",
        world.last_exit_code,
        world.last_stdout,
        world.last_stderr,
    );
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then(regex = r#"^the file "([^"]+)" should contain exactly:$"#)]
async fn file_should_contain_exactly(world: &mut ElsterWorld, step: &Step, path: String) {
    let expected = format!("{}\n", docstring(step));
    let actual_path = world.resolve(&path);
    assert!(
        actual_path.exists(),
        "Expected output file was not created: {path}"
    );
    let actual = std::fs::read_to_string(&actual_path).unwrap();
    assert_eq!(actual, expected);
}

#[then(regex = r#"^the CSV file "([^"]+)" should contain exactly:$"#)]
async fn csv_file_should_contain_exactly(world: &mut ElsterWorld, step: &Step, path: String) {
    let actual_path = world.resolve(&path);
    assert!(
        actual_path.exists(),
        "Expected output file was not created: {path}"
    );

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&actual_path)
        .unwrap();
    let actual: Vec<Vec<String>> = reader
        .records()
        .map(|r| r.unwrap().iter().map(str::to_string).collect())
        .collect();

    assert_eq!(actual, gherkin_table(step));
}

#[then(regex = r"^stderr should contain:$")]
async fn stderr_should_contain(world: &mut ElsterWorld, step: &Step) {
    let expected = docstring(step);
    assert!(
        world.last_stderr.contains(&expected),
        "stderr did not contain:\n{expected}\n\nActual stderr:\n{}",
        world.last_stderr,
    );
}

#[then(regex = r"^stdout should contain:$")]
async fn stdout_should_contain(world: &mut ElsterWorld, step: &Step) {
    let expected = docstring(step);
    assert!(
        world.last_stdout.contains(&expected),
        "stdout did not contain:\n{expected}\n\nActual stdout:\n{}",
        world.last_stdout,
    );
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let features = format!("{}/features", env!("OUT_DIR"));
    ElsterWorld::run(features).await;
}
