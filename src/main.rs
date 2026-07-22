use clap::{Parser, Subcommand};
use hledger_elster::{commit_evidence, config, paths, report_writer};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "hledger-elster",
    version,
    about = "Generate ELSTER-oriented tax exports from an hledger journal."
)]
struct Cli {
    #[command(flatten)]
    generate: GenerateArgs,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Write a default hledger-elster TOML config file.
    InitConfig {
        #[arg(long)]
        output: PathBuf,

        #[arg(long)]
        force: bool,
    },
    /// Write a PDF identifying the current clean Git commit.
    ExportCommitEvidence {
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(clap::Args)]
struct GenerateArgs {
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    #[arg(short = 'o', long = "output-dir")]
    output_dir: Option<PathBuf>,

    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::InitConfig { output, force }) => run_init_config(&output, force),
        Some(Commands::ExportCommitEvidence { output }) => run_export_commit_evidence(&output),
        None => run_generate(&cli.generate),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

fn run_init_config(output: &Path, force: bool) -> anyhow::Result<()> {
    let path = paths::resolve(output);
    config::write_default_config(&path, force)?;
    Ok(())
}

fn run_export_commit_evidence(output: &Path) -> anyhow::Result<()> {
    let path = paths::resolve(output);
    commit_evidence::write_commit_evidence(&path)?;
    Ok(())
}

fn run_generate(args: &GenerateArgs) -> anyhow::Result<()> {
    if let Some(file) = &args.file {
        std::env::set_var("FINANCES_LEDGER_JOURNAL", paths::resolve(file));
    }
    if let Some(output_dir) = &args.output_dir {
        std::env::set_var("FINANCES_TAX_DATA_DIR", paths::resolve(output_dir));
    }
    if let Some(config) = &args.config {
        std::env::set_var("HLEDGER_ELSTER_CONFIG", paths::resolve(config));
    }
    let exit_code = report_writer::generate_report()?;
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rust_decimal::{Decimal, RoundingStrategy};
    use std::str::FromStr;

    fn quantize(value: Decimal) -> Decimal {
        value.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
    }

    fn q(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn rounds_half_up_away_from_zero_like_python_round_half_up() {
        let cases: &[(&str, &str)] = &[
            ("0.005", "0.01"),
            ("0.015", "0.02"),
            ("0.025", "0.03"),
            ("0.125", "0.13"),
            ("2.675", "2.68"),
            ("-0.005", "-0.01"),
            ("-0.015", "-0.02"),
            ("-0.125", "-0.13"),
            ("-2.675", "-2.68"),
            ("1.004", "1.00"),
            ("1.005", "1.01"),
            ("1.006", "1.01"),
            ("0.00", "0.00"),
            ("-0.00", "0.00"),
            ("100.005", "100.01"),
            ("-100.005", "-100.01"),
            ("0.5", "0.50"),
            ("1234.565", "1234.57"),
            ("-1234.565", "-1234.57"),
            ("0.115", "0.12"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                quantize(q(input)),
                q(expected),
                "quantize({input}) should equal {expected}"
            );
        }
    }
}
