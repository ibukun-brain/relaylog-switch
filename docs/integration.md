# Integration Guide

## Relaylog export

This reader consumes only the post-session JSONL export. The export contract
was checked read-only against the local Relaylog implementation while creating
this scaffold. The public fixture exercises its externally visible shape and
uses fictional values. The export has no session ID or timestamp; spool offsets
are local ordering references only.

Use a JSONL file without installing Relaylog:

```bash
incident-room inspect --jsonl /path/to/session.jsonl
```

Use the optional local Relaylog CLI:

```bash
incident-room investigate --session /path/to/session-directory --authorize-sharing
```

The executable is invoked without an interactive shell. Its stdout is parsed
with line and total size limits. Its stderr is drained but not included in the
agent bundle. Missing executables, nonzero exit status, incomplete exports, and
malformed JSONL are actionable errors. `inspect` works independently from the
Relaylog installation.

The Incident Room child process receives null stdin and cannot accept a
passphrase through standard input. For a passphrase-protected session that needs
interactive input, export JSONL from the operator's terminal to an owner-only
temporary file. After the operator approves that export, prepare its bundle with:

```bash
incident-room inspect --jsonl /path/to/session.jsonl --authorize-sharing
```

## Switch setup with an existing agent

The Hackathon resources identify existing agents such as Codex and Claude Code
as supported, and Switch's quickstart uses Switch Console to connect an agent
to a room. The agent instructions for this repository are in
[`../switch/agents/incident-investigator.md`](../switch/agents/incident-investigator.md).

1. Install Switch Console and connect an existing coding-agent provider by
   following the [official quickstart](https://docs.switchagents.ai/switch-rooms/getting-started).
2. Add an agent through Switch Console. Set its run location to the computer
   with this checkout and set its working directory to this repository.
3. Use the incident investigator instructions as the agent brief, then add
   the agent and a human operator to a Switch room.
4. Start with the synthetic fixture. Ask the agent to summarize evidence IDs
   and to keep the synthetic hypothesis separate from observed records.
5. For a real incident, the agent must ask the operator to approve the specific
   session before running `incident-room investigate`. The resulting JSON is
   local command output available to the connected agent; it is not uploaded by
   this application.
6. Review the agent's claims in the room. Require each finding to cite an
   export-local evidence reference and keep conclusions marked as drafts until
   a human approves them.

The connected coding agent's normal Switch Console provider and room access
handle its authentication. This repository stores no Switch credentials.

## Custom agent protocol

No custom client is needed for the initial workflow. Current Switch protocol
documentation describes a WebSocket connection and HTTP operations authenticated
with an agent API key; registration uses a separate registration key. The older
SSE transport is deprecated. This project does not implement or invent those
endpoints, and it does not store either key. See the official [agent
protocol](https://docs.switchagents.ai/switch-rooms/internals/agent-protocol).

## Verification status

The local parser and deterministic demo are implemented. The manual Switch setup
is documented, but an actual agent reply in a Switch room has not been verified
in this repository's environment. Do not claim the end-to-end Switch workflow
is complete until a human performs that setup and observes a real room exchange.
