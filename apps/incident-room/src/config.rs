use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "incident-room",
    version,
    about = "Inspect Relaylog evidence for a Switch collaboration workflow"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Load JSONL and show a content-free summary by default.
    Inspect {
        /// JSONL exported by `relaylog export`.
        #[arg(long, value_name = "FILE")]
        jsonl: PathBuf,
        /// Authorize preparing session metadata for an agent or room.
        #[arg(long)]
        authorize_sharing: bool,
        /// Also authorize terminal output, bounded to 256 KiB.
        #[arg(long, requires = "authorize_sharing")]
        include_terminal_output: bool,
    },
    /// Run the deterministic, fictional incident-room example.
    Demo {
        /// Synthetic Relaylog JSONL fixture.
        #[arg(long, value_name = "FILE")]
        fixture: PathBuf,
        /// Include evidence values in the draft for a local demonstration.
        #[arg(long)]
        authorize_sharing: bool,
    },
    /// Export and prepare one local Relaylog session for agent review.
    Investigate {
        /// Relaylog session directory passed to `relaylog export`.
        #[arg(long, value_name = "DIRECTORY")]
        session: PathBuf,
        /// Required consent before session metadata is prepared for an agent.
        #[arg(long)]
        authorize_sharing: bool,
    },
}
