mod afa;
mod aggregates;
mod classification;
mod config;
mod dataset;
mod drawing;
mod enrich;
mod est;
mod euer;
mod herleitung;
mod journal;
mod paths;
mod periods;
mod posting;
mod ust;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "hledger-elster",
    about = "Generate ELSTER-oriented tax exports from an hledger journal."
)]
struct GenerateArgs {
    #[arg(short = 'f', long = "file")]
    file: Option<PathBuf>,

    #[arg(short = 'o', long = "output-dir")]
    output_dir: Option<PathBuf>,

    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Parser)]
#[command(
    name = "hledger-elster init-config",
    about = "Write a default hledger-elster TOML config file."
)]
struct InitConfigArgs {
    #[arg(long)]
    output: PathBuf,

    #[arg(long)]
    force: bool,
}

fn main() -> anyhow::Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();

    if raw_args.get(1).map(String::as_str) == Some("init-config") {
        let args = InitConfigArgs::parse_from(
            std::iter::once(raw_args[0].clone()).chain(raw_args[2..].iter().cloned()),
        );
        if let Err(err) = run_init_config(&args) {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
        return Ok(());
    }

    let args = GenerateArgs::parse();
    if let Err(err) = run_generate(&args) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

fn run_init_config(args: &InitConfigArgs) -> anyhow::Result<()> {
    let path = paths::resolve(&args.output);
    config::write_default_config(&path, args.force)?;
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
    // TODO(Phase 5): wire up ingest::build_dataset + report_writer::generate_report.
    anyhow::bail!("not yet implemented");
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
