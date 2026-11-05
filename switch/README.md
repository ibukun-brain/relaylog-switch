# Switch Setup

## Integration approach

Use an existing coding agent that Switch Console already supports. The agent
works from this repository and invokes `incident-room` after a person approves
sharing a specific session. This repository does not implement a custom Switch
protocol client, connect to a room itself, or upload JSONL.

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
   `cargo run -p incident-room -- demo --fixture examples/api-incident/synthetic-session.jsonl`
   and discuss the draft with the human.
6. For a real session, the agent must ask for session-specific approval first.
   After approval, it can run
   `cargo run -p incident-room -- investigate --session <directory> --authorize-sharing`.
   This prints the prepared JSON into the agent's local execution context; it
   does not send it to an external service through this application. If Relaylog
   needs an interactive passphrase, have the operator export JSONL locally and
   use `incident-room inspect --jsonl <file> --authorize-sharing` instead.
7. Ask for an evidence-backed draft in the room. Every factual claim should
   cite an `offset-N` reference and remain marked as a hypothesis until a human
   reviews it.

The CLI flag is an explicit consent step, not an authentication mechanism. A
Switch-connected agent must still wait for the operator's approval before
running it on a real session. Never enable terminal-output sharing without a
separate approval.

## Verification status

The CLI and fixture path work locally. The live Switch setup above is
documented, but no room connection or agent reply has been verified as part of
this scaffold. Do not claim the full collaborative workflow is finished until
the manual room exchange succeeds.
