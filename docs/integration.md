# Integration Guide

## Relaylog export

This reader consumes post-session JSONL exports only. The outer Relaylog
capture wrapper contains an ordered `spool_offset` and nested `record`, then a
final `capture_gaps` summary. Agent records use the verified shape
`{"type":"agent","session_id":"…","evidence":{…}}`; their nested payload
has schema version 1. Legacy shell records remain supported and have no actor
or agent attribution. Newer shell-hook events may carry nested actor and source
metadata; the adapter preserves those fields, including an explicit `unknown`
actor and the source's coverage classification. It does not infer a person from
the machine account or promote an agent hook's `partial` coverage to complete.

Use a JSONL file without installing Relaylog:

```bash
incident-room inspect --jsonl /path/to/session.jsonl
```

Export through the optional local CLI:

```bash
incident-room investigate --session /path/to/session-directory --authorize-sharing
```

For a locally built Relaylog binary that is not on `PATH`, specify it
explicitly (argument paths with spaces remain a single safe process argument):

```bash
incident-room investigate \
  --session /path/to/session-directory \
  --relaylog-bin /path/to/relaylog/target/debug/relaylog \
  --authorize-sharing
```

The repository Makefile also provides a guarded shortcut:

```bash
make investigate RELAYLOG=/path/to/relaylog \
  SESSION_DIR=/path/to/session-directory AUTHORIZE_SHARING=yes
```

The explicit `AUTHORIZE_SHARING=yes` is required before that target prepares
metadata for the agent.

The executable is started without an interactive shell. Stdout parsing is
bounded by line and total size. Stderr is drained but not returned or logged.
Missing executables, nonzero exit status, malformed JSONL, and missing capture
summaries are actionable errors. Well-formed capture-loss counters are kept in
the timeline and reported as limitations. The timeline also contains a
`capture_coverage` assessment with counts for unrecognized events, unattributed
shell events, partial or unknown source coverage, and unmatched agent
operations. `known_incomplete` means the import contains a positive gap signal;
`unverified` means no explicit gap was found, not that capture is complete.
Relaylog currently supplies no session-wide completeness attestation, so the
adapter never sets `verified_complete` to true. `inspect` works without Relaylog.

With explicit approval, `inspect` returns a bounded evidence bundle and a
unified human/agent timeline. Agent payloads preserve source observation time,
actor/provider, provider session, tool invocation, command/result, and
file-operation metadata. Export order is preserved and source timestamps do
not reorder concurrent activities. Agent-hook records are source reports, not
independent verification; completion events do not mean success. Legacy shell
start and finish events remain uncorrelated. Unsupported agent schema versions
and event types are retained only as safe evidence references.

## Read-only agent queries

No MCP server or Switch protocol client exists in this repository. A
Switch-connected local coding agent can invoke the CLI as its read-only evidence
tool after explicit approval for the specific session:

```bash
incident-room query --jsonl /path/to/session.jsonl \
  --agent-provider openai --authorize-sharing
incident-room query --jsonl /path/to/session.jsonl \
  --event offset-192 --authorize-sharing
incident-room query --jsonl /path/to/session.jsonl \
  --failed --authorize-sharing
incident-room query --jsonl /path/to/session.jsonl \
  --file-operations --authorize-sharing
incident-room query --jsonl /path/to/session.jsonl \
  --tool-invocation call-123 --offset 0 --limit 100 --authorize-sharing
incident-room query --jsonl /path/to/session-a.jsonl \
  --jsonl /path/to/session-b.jsonl --event session-2/offset-128 \
  --authorize-sharing
```

Queries can combine selectors, return at most 500 events and 512 KiB, and page
in export order with `--offset`. For one file, `--event` accepts `offset-N`;
with multiple files it accepts `session-N/offset-N`. Repeat `--jsonl` to query up to eight
explicitly selected sessions together. The CLI assigns temporary `session-1`,
`session-2`, … source references in argument order so equal spool offsets remain
distinguishable. These aliases exist only for that command invocation; no
Switch room, Discord thread, or Relaylog session mapping is persisted here.
The local authorization flag cannot enforce Discord-room isolation; keep the
connected agent's workspace and approved export paths scoped to the team.
Library users can call `build_timeline`, `InvestigationTimeline::event`,
`failed_operations`, `events_by_agent_provider`,
`events_by_tool_invocation`, and `file_operations` directly.

The Incident Room subprocess receives null stdin. For a passphrase-protected
session requiring interactive input, export JSONL in the operator's terminal
to a protected file, review it, then explicitly authorize the CLI query.

## Schema compatibility

The public parser implements the Relaylog core's provider-neutral agent payload
v1 nested within the existing capture record wrapper. Unsupported payload
versions and unknown event types are safe reference-only items. The checked
core currently has a Codex adapter. Claude Code and Gemini CLI are supported by
the normalized design but their provider adapters are not present in that
checked tree, so their capture is not claimed.

The checked Codex hook marks its provider coverage `partial`. This adapter now
parses and reports that status instead of treating the affected records as
unknown or silently presenting them as complete. It cannot recover events the
producer did not capture; improving hook coverage requires a change in Relaylog
core followed by a new export-contract fixture.

`examples/api-incident/agent-aware-session.jsonl` is fictional and authored to
the verified serialization contract. It was not emitted by the proprietary
Relaylog binary. Validate the current private binary's export against this
shape before claiming end-to-end compatibility. The separate terminal control
protocol is not treated as JSONL evidence.

The parser never opens Relaylog's encrypted spool or private FIFO. It does not
subscribe to live events. Session IDs from Relaylog, agent providers, Switch,
and Discord remain separate. This repository does not persist room/session
mappings or attach Switch-started agents to Relaylog sessions.

## Switch setup with an existing agent

1. Install Switch Console and connect an existing coding-agent provider using
   the [official quickstart](https://docs.switchagents.ai/switch-rooms/getting-started).
2. Configure the agent to run on the machine containing this checkout and use
   the repository as its working directory.
3. Add the agent and human investigator to a Switch room. Use the brief in
   [`../switch/agents/incident-investigator.md`](../switch/agents/incident-investigator.md).
4. Ask the agent to query the synthetic agent-aware fixture and cite its
   `offset-N` event references. Keep agent reports distinct from shell
   observations and inferences.
5. For real sessions, obtain session-specific operator approval before the
   agent runs `incident-room investigate` or `incident-room query`.
6. Review findings together. The runbook stays a draft until human review.

Switch Console handles the connected agent's room authentication. This
repository stores no Switch credentials and does not send data to Discord or
Switch itself. An actual room exchange must be verified in the configured
environment before claiming end-to-end Switch integration.

## External interfaces not implemented here

The current Switch docs describe a provider protocol, but a custom client is
outside this integration's current path: an existing Switch-connected coding
agent invokes the CLI locally. No Discord bot, HTTP/WS endpoint, authentication
schema, MCP server, live Relaylog subscription, or new session-mapping database
is invented here. If Relaylog publishes an authorized server-side stream or a
secure session-attachment interface, it can be added against that contract.
