# Switch Setup

## Import into Switch Console

Use the importable agent template at
[`templates/relaylog-ai-investigator.template.yaml`](templates/relaylog-ai-investigator.template.yaml).
In Switch Console, open **Templates**, choose **Import template**, and select
that YAML file. Then use the imported template and set:

1. **Messaging app** to Discord.
2. **Agent machine** to a Switch-connected machine with Codex installed and
   signed in.
3. **Relaylog checkout** to this repository on that machine.
4. Choose a room name and keep the suggested agent identifier, or choose
   another unused identifier.

The template explicitly sets the provider to Codex and creates a new
Switch room bridged to the selected messaging app, with the new agent and
template user in it. It does not rewrite the existing `relaylogAIagent` or add
the new agent to an existing room. Add the other Discord participants to the
new room after creation. If you need to keep using an existing room, add or
edit the agent manually using the instructions below.

The working directory and Relaylog session must be reachable from the selected
agent runtime. A Windows-hosted provider does not automatically get access to
Linux paths such as `/home/...` in WSL. If the provider cannot run the adapter
in that WSL distribution, run the approved adapter command in WSL and attach
its JSON bundle to the room. The template does not grant access to Relaylog's
encrypted spool or upload evidence itself.

The template follows Switch's YAML template format and creates a bridged room;
it does not create a separate Discord bot. See
the [Switch template language reference](https://github.com/sandbox-quantum/switch/blob/main/docs/TEMPLATES.md)
for the supported template fields.

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

## Manual setup without importing the template

1. Install Switch Console and follow the
   [Switch quickstart](https://docs.switchagents.ai/switch-rooms/getting-started)
   to connect an existing coding-agent provider.
2. Add or edit an agent in Switch Console. Set its run location to the computer
   with this checkout, and set its working directory to the repository root.
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
   After approval, run `cargo run -p incident-room -- investigate --session
   <directory> --authorize-sharing`. If the Relaylog binary is a local build
   outside `PATH`, add `--relaylog-bin <executable>`. The equivalent guarded
   Make target, run from this checkout inside WSL, is
   `make investigate RELAYLOG=<relaylog-binary> SESSION_DIR=<session-directory>
   AUTHORIZE_SHARING=yes`. This prints the prepared metadata JSON into the
   agent's local execution context; it does not send it to an external service
   through this application. If Relaylog needs an interactive passphrase, have
   the operator export JSONL locally and use
   `incident-room inspect --jsonl <file> --authorize-sharing` instead.
7. Ask the agent to inspect `timeline`, `agent_evidence_schema_versions`, and
   `limitations`, `capture_gaps`, and `capture_coverage`. State the coverage
   status and whether completeness is verified; a clear drop-counter footer
   alone is not proof of completeness. Every factual claim should cite an
   `offset-N` reference. Legacy shell events remain actor-unknown; only
   explicit actor metadata can support AI or human attribution. Do not infer
   identity from the Switch agent that ran the CLI.
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

Do not invoke `relaylog-switch inspect-session`; that command does not exist.
The supported session path is `incident-room investigate`, with optional
`--relaylog-bin`, or the guarded `make investigate` target. A Switch agent only
has access to local files when its configured runner can reach that machine
and filesystem. For a WSL session, attach the operator-generated JSON bundle
to the room if the agent runs outside that WSL environment.

## Verification status

The CLI and fixture path work locally. The live Switch setup above is
documented, but no room connection or agent reply has been verified as part of
this scaffold. Do not claim the full collaborative workflow is finished until
the manual room exchange succeeds.
