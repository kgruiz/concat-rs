mod cli;
mod config;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let raw_args: Vec<std::ffi::OsString> = std::env::args_os().collect();

    let cli = cli::Cli::parse_from(&raw_args);

    match cli.command {
        Some(cli::Command::Clean(_clean)) => {
            eprintln!("clean: not implemented yet");
            Ok(())
        }
        None => {
            let had_user_args = raw_args.len() > 1;
            let config = config::RunConfig::from_cli(cli, had_user_args)?;
            config.print_summary();
            eprintln!("run: not implemented yet");
            Ok(())
        }
    }
}
