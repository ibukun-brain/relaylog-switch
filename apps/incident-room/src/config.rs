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
    /// Query a bounded, read-only slice of an authorized investigation timeline.
    Query {
        /// JSONL export to include; repeat to combine authorized sessions.
        #[arg(long, value_name = "FILE", required = true, action = clap::ArgAction::Append)]
        jsonl: Vec<PathBuf>,
        /// Select offset-N or session-N/offset-N for combined exports.
        #[arg(long, value_name = "REFERENCE")]
        event: Option<String>,
        /// Select events attributed to this provider (for example openai or codex).
        #[arg(long, value_name = "PROVIDER")]
        agent_provider: Option<String>,
        /// Select events sharing a provider tool-call ID.
        #[arg(long, value_name = "ID")]
        tool_invocation: Option<String>,
        /// Select shell failures and agent-reported failures.
        #[arg(long)]
        failed: bool,
        /// Select agent file-operation events.
        #[arg(long)]
        file_operations: bool,
        /// Maximum number of returned events, capped at 500.
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u16).range(1..=500))]
        limit: u16,
        /// Number of matching events to skip (export-order pagination).
        #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u32).range(0..=100000))]
        offset: u32,
        /// Required consent before evidence details are returned.
        #[arg(long)]
        authorize_sharing: bool,
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
        /// Path to a Relaylog executable that is not available on PATH.
        #[arg(long, value_name = "EXECUTABLE")]
        relaylog_bin: Option<PathBuf>,
        /// Required consent before session metadata is prepared for an agent.
        #[arg(long)]
        authorize_sharing: bool,
    },
}
