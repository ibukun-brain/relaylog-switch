# Relaylog Incident Room

**Resolve incidents together. Keep the evidence.**

Relaylog Incident Room is an AI-assisted incident-response workflow connecting
structured terminal evidence with collaborative agents through Switch. This
repository is a standalone Rust workspace; the proprietary Relaylog recorder
and server are not included.

## Problem

Production investigations are split across terminals, logs, and team
conversations. Important command metadata and the reasoning around it can be
hard to review together after the incident.

## Solution

Relaylog exports a completed session as JSONL. This repository reads that
supported export, validates its order and completeness, applies an explicit
sharing boundary, and prepares structured evidence for an agent. Switch gives
the human and an existing coding agent a shared room to discuss the evidence
and draft a runbook.

The local demo is deterministic and uses fictional records. It does not call an
LLM or upload evidence.

## Architecture

```text
Private Relaylog recorder
        │ relaylog export <session-directory>
        ▼
      JSONL ──► relaylog-switch library ──► incident-room CLI
                                                │
                                      explicit operator approval
                                                ▼
                                  Switch-connected coding agent
                                                │
                                      shared room discussion
```

The public workspace consumes only the supported JSONL export. It has no Cargo
path or Git dependency on Relaylog, never reads the encrypted spool, and does
not expose a live subscription API. The current export supplies ordered spool
offsets and capture records; it does not supply a session ID or wall-clock
timestamps. Offsets are local evidence references, not global event IDs.

## Quick Start

Requires Rust 1.85 or newer.

```bash
cargo build --workspace
```

Run the synthetic incident draft:

```bash
cargo run -p incident-room -- demo \
  --fixture examples/api-incident/synthetic-session.jsonl
```

To include the synthetic command metadata in the local draft, explicitly opt
in:

```bash
cargo run -p incident-room -- demo \
  --fixture examples/api-incident/synthetic-session.jsonl \
  --authorize-sharing
```

Inspect an exported JSONL file. By default, this prints a content-free summary.
Add `--authorize-sharing` to prepare a structured bundle of approved metadata
for an agent. The command does not send the bundle anywhere.

```bash
cargo run -p incident-room -- inspect --jsonl /path/to/session.jsonl
cargo run -p incident-room -- inspect --jsonl /path/to/session.jsonl --authorize-sharing
```

For the optional Relaylog CLI path, the application invokes `relaylog export`
with separate process arguments and no shell. Sharing authorization is
required before the output bundle is prepared:

```bash
cargo run -p incident-room -- investigate \
  --session /path/to/session-directory \
  --authorize-sharing
```

This prints a local bundle to stdout. It does not upload evidence to Switch or
another service. See [Switch setup](switch/README.md) before handing real
evidence to an agent.

If the Relaylog session requires an interactive passphrase, export it from the
operator's terminal to a protected temporary JSONL file and use `inspect`
instead. The Incident Room subprocess closes stdin and cannot receive a
passphrase through standard input.

## Switch Workflow

The intended workflow uses a coding agent already supported by Switch Console.
The agent runs `incident-room` from this repository after the human approves a
specific session. Setup and the agent's operating instructions are in
[`switch/README.md`](switch/README.md) and
[`switch/agents/incident-investigator.md`](switch/agents/incident-investigator.md).

The repository does **not** claim a live Switch room was connected or tested in
this scaffold. Follow the setup and verify an agent reply in your own Switch
room before describing the workflow as a completed integration.

## Security and Privacy

- The main Relaylog recorder and server are proprietary and are not included.
- JSONL records are untrusted input. Parsing is bounded, validates the final
  capture-gap summary, and rejects missing, truncated, or incomplete evidence.
- Private-mode intervals and their markers are omitted. Inconsistent or
  unclosed intervals fail closed.
- Terminal input is never shared. Terminal output is omitted unless the
  separate `--include-terminal-output` option is used with
  `--authorize-sharing`; output and total bundle sizes are capped.
- Command text, working directories, origin fields, and output bytes can still
  contain sensitive information. Explicit approval is not a secret scanner and
  does not guarantee complete secret filtering.
- Evidence values are labeled untrusted data and are never executed by this
  application. Prompt-injection resistance also depends on the connected
  agent's instructions and tool permissions; it is not a guarantee about model
  behavior.
- Nothing is sent externally by this repository. If a Switch-connected agent
  invokes the CLI, its approved output becomes available to that agent.

Read [the security notes](docs/security.md) before using real sessions.

## Hackathon

This workflow is designed for the Flip the Switch Hackathon: a person and an AI
agent collaborate in a Switch room to investigate a real task. Switch supports
existing agents and a custom agent protocol. This scaffold chooses the existing
coding-agent path and leaves live room verification as a setup task. Official
resources: [hackathon overview](https://www.wemakedevs.org/hackathons/switch),
[Switch documentation](https://docs.switchagents.ai/), and [agent
protocol](https://docs.switchagents.ai/switch-rooms/internals/agent-protocol).

## Licensing

No license has been selected. The root `LICENSE` file is a status note, not a
license grant. See [the licensing decision note](docs/licensing-decision.md).

## Development

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
