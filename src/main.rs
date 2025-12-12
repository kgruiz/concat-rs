mod clean;
mod cleanup;
mod cli;
mod config;
mod discover;
mod filter;
mod inputs;
mod output;
mod output_name;
mod run;
mod sort;
mod text_detect;
mod tree;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let raw_args: Vec<std::ffi::OsString> = std::env::args_os().collect();

    let cli = cli::Cli::parse_from(&raw_args);

    match cli.command {
        Some(cli::Command::Clean(clean_args)) => clean::run(clean_args),
        None => {
            let had_user_args = raw_args.len() > 1;
            let config = config::RunConfig::from_cli(cli, had_user_args)?;
            config.print_summary();

            run::run(config)
        }
    }
}
