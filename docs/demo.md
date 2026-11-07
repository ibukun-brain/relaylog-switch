# Demo Guide

## Deterministic local run

From the repository root:

```bash
cargo run -p incident-room -- demo \
  --fixture examples/api-incident/synthetic-session.jsonl
```

This prints the report structure and content-free record summary. To include the
synthetic shell metadata in the draft output, explicitly opt in:

```bash
cargo run -p incident-room -- demo \
  --fixture examples/api-incident/synthetic-session.jsonl \
  --authorize-sharing
```

The scenario premise and database-pool hypothesis are labeled as synthetic.
The JSONL contains legacy shell command metadata and exit statuses only. It does
not contain actual API responses, container logs, database output, timestamps,
actor IDs, or AI-provider events. It does not demonstrate that Codex, Claude
Code, or Gemini performed any recorded action. The report therefore does not
claim that the HTTP error or its cause was verified by the fixture.

## Agent-aware fixture replay

The second fixture mirrors the Relaylog core's nested agent payload v1 and
contains fictional Codex hook events alongside legacy shell activity. It
demonstrates mixed actor attribution, provider-session/tool-call correlation,
an agent-reported file operation, and partial-coverage limitations. It does not
claim that a real agent executed those events, and its shape still needs a
fresh export compatibility check against the updated private Relaylog binary.

```bash
cargo run -p incident-room -- inspect \
  --jsonl examples/api-incident/agent-aware-session.jsonl \
  --authorize-sharing
cargo run -p incident-room -- query \
  --jsonl examples/api-incident/agent-aware-session.jsonl \
  --agent-provider openai --authorize-sharing
cargo run -p incident-room -- query \
  --jsonl examples/api-incident/agent-aware-session.jsonl \
  --file-operations --authorize-sharing
```

## Switch walkthrough

1. Set up a Switch room with the existing coding-agent workflow in
   [`../switch/README.md`](../switch/README.md).
2. Ask the agent to inspect the agent-aware fixture and cite `offset-4`,
   `offset-128`, and `offset-192`.
3. Check that the Codex activity is identified as provider-reported, the
   legacy shell actor remains unknown, the file event contains paths but no
   file contents, and partial coverage is visible.
4. Ask the human participant to challenge the hypothesis and request missing
   evidence before any conclusion is called verified.
5. Show the `Human Review Status` remains a draft not reviewed by a human.

## WSL recording and Discord transcript

For the full live-terminal-to-Codex-TUI-to-Discord walkthrough, including the
`--relaylog-bin` path for a local Relaylog build, the metadata-only bundle
export, WSL attachment path, and teammate transcript, follow
[`orders-api-production-simulation.md`](../examples/api-incident/orders-api-production-simulation.md).
The live recording's coverage must be read from its attached bundle; the
synthetic fixture's stable event references must not be presented as events
from that live session.

## Short recording outline

1. State the incident and show the human and Switch agent in the shared room.
2. Run the local fixture demo and show the evidence order and report sections.
3. Have the agent produce a draft with evidence references and an explicit
   unresolved-questions section.
4. Review one hypothesis together and leave it unverified until the human
   agrees.
5. Explain that real session evidence needs session-specific operator approval
   before it enters the agent context.
