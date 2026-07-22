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
        let work_dir = tmp.path().join("work");
        std::fs::create_dir_all(&work_dir).unwrap();
        let work_dir = work_dir.canonicalize().unwrap_or(work_dir);
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

    fn resolve_outside_work_dir(&self, path: &str) -> PathBuf {
        let relative = Path::new(path);
        assert!(
            !relative.is_absolute()
                && !relative
                    .components()
                    .any(|c| c == std::path::Component::ParentDir),
            "Unsafe scenario path: {path}"
        );
        self.work_dir.parent().unwrap().join(relative)
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

async fn run_git(world: &ElsterWorld, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(&world.work_dir)
        .output()
        .await
        .unwrap_or_else(|e| panic!("failed to run git {args:?}: {e}"))
}

async fn run_elster_command(world: &mut ElsterWorld, command: &str) -> bool {
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

    output.status.success()
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

#[given(regex = r#"^a git repository$"#)]
async fn git_repository(world: &mut ElsterWorld) {
    let output = run_git(world, &["init"]).await;
    assert!(
        output.status.success(),
        "git init failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    for args in [
        ["config", "user.email", "cucumber@example.invalid"],
        ["config", "user.name", "Cucumber Tests"],
        ["config", "commit.gpgsign", "false"],
    ] {
        let output = run_git(world, &args).await;
        assert!(
            output.status.success(),
            "git {args:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[given(regex = r#"^a file named "([^"]+)" with content:$"#)]
async fn write_file(world: &mut ElsterWorld, step: &Step, path: String) {
    let content = docstring(step);
    let target = world.resolve(&path);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(&target, format!("{content}\n")).unwrap();
}

#[given(regex = r#"^I commit all files$"#)]
async fn commit_all_files(world: &mut ElsterWorld) {
    let output = run_git(world, &["add", "."]).await;
    assert!(
        output.status.success(),
        "git add failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = run_git(world, &["commit", "-m", "scenario state"]).await;
    assert!(
        output.status.success(),
        "git commit failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when(regex = r#"^I run "([^"]+)"$"#)]
async fn run_command(world: &mut ElsterWorld, command: String) {
    let success = run_elster_command(world, &command).await;
    assert!(
        success,
        "command failed (exit {}): {command}\nstdout:\n{}\nstderr:\n{}",
        world.last_exit_code, world.last_stdout, world.last_stderr,
    );
}

#[when(regex = r#"^I run "([^"]+)" and it fails$"#)]
async fn run_command_fails(world: &mut ElsterWorld, command: String) {
    let success = run_elster_command(world, &command).await;
    assert!(
        !success,
        "command unexpectedly succeeded: {command}\nstdout:\n{}\nstderr:\n{}",
        world.last_stdout, world.last_stderr,
    );
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then(regex = r#"^the file "([^"]+)" should exist$"#)]
async fn file_should_exist(world: &mut ElsterWorld, path: String) {
    assert!(
        world.resolve(&path).exists(),
        "Expected output file was not created: {path}"
    );
}

#[then(regex = r#"^the file "([^"]+)" should not exist$"#)]
async fn file_should_not_exist(world: &mut ElsterWorld, path: String) {
    assert!(
        !world.resolve(&path).exists(),
        "Expected output file not to exist: {path}"
    );
}

#[then(regex = r#"^the file outside the repository "([^"]+)" should exist$"#)]
async fn file_outside_repository_should_exist(world: &mut ElsterWorld, path: String) {
    assert!(
        world.resolve_outside_work_dir(&path).exists(),
        "Expected output file outside repository was not created: {path}"
    );
}

#[then(regex = r#"^the file outside the repository "([^"]+)" should not exist$"#)]
async fn file_outside_repository_should_not_exist(world: &mut ElsterWorld, path: String) {
    assert!(
        !world.resolve_outside_work_dir(&path).exists(),
        "Expected output file outside repository not to exist: {path}"
    );
}

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

#[then(regex = r#"^the PDF file "([^"]+)" should contain the current git commit hash$"#)]
async fn pdf_file_should_contain_current_git_commit_hash(world: &mut ElsterWorld, path: String) {
    let output = run_git(world, &["rev-parse", "HEAD"]).await;
    assert!(
        output.status.success(),
        "git rev-parse failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let bytes = std::fs::read(world.resolve(&path)).unwrap();
    let content = String::from_utf8_lossy(&bytes);
    assert!(
        content.contains(&hash),
        "PDF file {path} did not contain current git commit hash {hash}"
    );
}

#[then(
    regex = r#"^the PDF file outside the repository "([^"]+)" should contain the current git commit hash$"#
)]
async fn pdf_file_outside_repository_should_contain_current_git_commit_hash(
    world: &mut ElsterWorld,
    path: String,
) {
    let output = run_git(world, &["rev-parse", "HEAD"]).await;
    assert!(
        output.status.success(),
        "git rev-parse failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let bytes = std::fs::read(world.resolve_outside_work_dir(&path)).unwrap();
    let content = String::from_utf8_lossy(&bytes);
    assert!(
        content.contains(&hash),
        "PDF file outside repository {path} did not contain current git commit hash {hash}"
    );
}

#[then(regex = r#"^the git working tree should be clean$"#)]
async fn git_working_tree_should_be_clean(world: &mut ElsterWorld) {
    let output = run_git(world, &["status", "--porcelain"]).await;
    assert!(
        output.status.success(),
        "git status failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let status = String::from_utf8_lossy(&output.stdout);
    assert!(
        status.trim().is_empty(),
        "git working tree is dirty:\n{status}"
    );
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

#[then(regex = r#"^the xlsx file "([^"]+)" tab "([^"]+)" should equal the CSV file "([^"]+)"$"#)]
async fn xlsx_tab_should_equal_csv(
    world: &mut ElsterWorld,
    xlsx_path: String,
    tab: String,
    csv_path: String,
) {
    use calamine::{open_workbook, Reader, Xlsx};

    let xlsx_full = world.resolve(&xlsx_path);
    let mut workbook: Xlsx<_> = open_workbook(&xlsx_full)
        .unwrap_or_else(|e| panic!("failed to open xlsx file {xlsx_path}: {e}"));
    let range = workbook
        .worksheet_range(&tab)
        .unwrap_or_else(|e| panic!("tab \"{tab}\" not found in {xlsx_path}: {e}"));
    let xlsx_rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(|cell| cell.to_string()).collect())
        .collect();

    let csv_full = world.resolve(&csv_path);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(&csv_full)
        .unwrap();
    let csv_rows: Vec<Vec<String>> = reader
        .records()
        .map(|r| r.unwrap().iter().map(str::to_string).collect())
        .collect();

    // Section headers are the one documented exception (specs/01-csv-xlsx-equivalence.md):
    // report_writer::write_summary_sheet strips a "# " Kennzahl prefix and renders the row
    // bold in xlsx instead, since CSV has no bold to fall back on and keeps the marker
    // literal. Normalize that one marker so this step can still assert real equivalence
    // everywhere else.
    let normalized_csv_rows: Vec<Vec<String>> = csv_rows
        .into_iter()
        .map(|mut row| {
            if let Some(first) = row.first_mut() {
                if let Some(stripped) = first.strip_prefix("# ") {
                    *first = stripped.to_string();
                }
            }
            row
        })
        .collect();

    assert_eq!(
        xlsx_rows, normalized_csv_rows,
        "xlsx tab \"{tab}\" in {xlsx_path} does not match {csv_path}"
    );
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
