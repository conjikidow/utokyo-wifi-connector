mod cli;
mod commands;
mod credentials;
mod error;
mod network;
mod nm;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::error::Error;
use crate::nm::{NetworkManager, Nmcli};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let nm = NetworkManager::new(Nmcli);

    let result = match &cli.command {
        Commands::Setup(args) => commands::setup::run(&nm, args),
        Commands::Delete(args) => commands::delete::run(&nm, args),
        Commands::Status(args) => commands::status::run(&nm, args),
        Commands::Completions(args) => {
            commands::completions::run(args);
            Ok(())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::Reported) => ExitCode::FAILURE,
        Err(Error::Interrupted) => {
            eprintln!("error: {}", Error::Interrupted);
            ExitCode::from(130)
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
