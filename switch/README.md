# Switch Setup

## Integration approach

Use an existing coding agent that Switch Console already supports. The agent
works from this repository and invokes `incident-room` after a person approves
sharing a specific session. This repository does not implement a custom Switch
protocol client, MCP server, Discord handler, connect to a room itself, persist
room/session mappings, or upload JSONL. Switch room and Discord delivery remain
the responsibility of the existing Switch setup.

The official Flip the Switch Hackathon describes using existing agents such as
Codex and Claude Code with Switch. Current Switch agent-protocol documentation
describes WebSocket events, HTTP operations, an agent API key, and a separate
registration key. The previous SSE transport is deprecated. This scaffold
avoids that custom-client path; see the [official protocol
docs](https://docs.switchagents.ai/switch-rooms/internals/agent-protocol) if a
future implementation needs one.

## Manual setup

1. Install Switch Console and follow the
   [Switch quickstart](https://docs.switchagents.ai/switch-rooms/getting-started)
   to connect an existing coding-agent provider.
2. Add an agent in Switch Console. Set its run location to the computer with
   this checkout, and set its working directory to the repository root.
3. Use [`agents/incident-investigator.md`](agents/incident-investigator.md) as
   its instructions. The agent needs permission to run Cargo in this directory.
4. Create or choose a Switch room and include the human operator and agent.
5. Start with the tracked synthetic fixture. Ask the agent to run
   `cargo run -p incident-room -- inspect --jsonl examples/api-incident/agent-aware-session.jsonl --authorize-sharing`
   and query its Codex records with
   `cargo run -p incident-room -- query --jsonl examples/api-incident/agent-aware-session.jsonl --agent-provider openai --authorize-sharing`.
   Discuss attribution and limitations with the human. The events are
   synthetic replay, not captured Codex activity.
6. For a real session, the agent must ask for session-specific approval first.
   After approval, it can run
   `cargo run -p incident-room -- investigate --session <directory> --authorize-sharing`.
   This prints the prepared JSON into the agent's local execution context; it
   does not send it to an external service through this application. If Relaylog
   needs an interactive passphrase, have the operator export JSONL locally and
   use `incident-room inspect --jsonl <file> --authorize-sharing` instead.
7. Ask the agent to inspect `timeline`, `agent_evidence_schema_versions`, and
   `limitations`. Every factual claim should cite an `offset-N` reference.
   Legacy shell events remain actor-unknown; only explicit v1 actor metadata
   can support AI or human attribution. Do not infer identity from the Switch
   agent that ran the CLI.
8. Keep reports as drafts. Separate observed events from operator-reported
   context and model-generated inference; do not claim a human reviewed or
   approved a change unless a source event says so.

The parser supports Relaylog's nested agent evidence payload schema v1. The
checked private core currently has a Codex adapter only; this repository does
not claim Claude Code or Gemini CLI capture until those providers emit the
normalized contract. The checked-in agent-aware JSONL is synthetic and has not
been exported by the private binary. A fresh export compatibility check and a
real Switch room exchange remain verification tasks.

The CLI flag is an explicit consent step, not an authentication mechanism. A
Switch-connected agent must still wait for the operator's approval before
running it on a real session. Never enable terminal-output sharing without a
separate approval.

## Verification status

The CLI and fixture path work locally. The live Switch setup above is
documented, but no room connection or agent reply has been verified as part of
this scaffold. Do not claim the full collaborative workflow is finished until
the manual room exchange succeeds.
