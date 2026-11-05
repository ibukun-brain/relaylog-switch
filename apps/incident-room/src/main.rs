mod config;
mod workflow;

use clap::Parser;
use config::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
    match workflow::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("incident-room: {error}");
            ExitCode::FAILURE
        }
    }
}
