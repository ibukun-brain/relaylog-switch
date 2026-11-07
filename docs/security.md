# Security and Privacy

## Data flow

The public library accepts a JSONL file or consumes stdout from
`relaylog export`. It never accesses Relaylog's encrypted spool or private FIFO.
No network client, AI model client, automatic upload, or evidence persistence
is implemented.

## Authorization and filtering

- Preparing an agent bundle requires an explicit `SharingPolicy`.
- The CLI requires `--authorize-sharing` before it prepares evidence for an
  agent. This flag records operator intent; it is not identity authentication
  and cannot prove that the caller owns the session.
- Input bytes are always excluded. Output bytes are excluded by default and
  require separate `--include-terminal-output` consent; total output is capped
  at 256 KiB and the serialized bundle at 1 MiB.
- Private intervals are omitted, including their start and end markers.
  Inconsistent or unclosed private boundaries reject the whole bundle.
- Unknown capture or shell variants are reduced to an evidence reference and
  an `unrecognized` marker; their raw payload is not forwarded to an agent.
- Agent v1 payloads are strictly deserialized and bounded. Unknown agent event
  types and unsupported schema versions are reduced to references; raw payloads
  are not forwarded. Known agent command/detail fields remain untrusted and may
  contain sensitive values.
- Relaylog's known `unstructured` shell marker is projected as a capture
  limitation; its raw reason string is not forwarded.
- Valid exports with nonzero capture-loss counters or truncation remain
  inspectable. The bundle exposes the counters and marks the timeline
  incomplete; missing events must be treated as unknown.
- A clear `capture_gaps` footer only means the exporter reported no queue or
  capacity drops and no truncation marker. It does not attest to complete hook
  coverage. The adapter reports `known_incomplete` when it sees a positive gap
  signal and otherwise reports `unverified`; it never declares an export
  complete.
- There is no general secret detector. Shell and agent command arguments,
  working directories, file paths, origin values, and even allowed metadata
  can contain secrets or personal information. Review the selected session
  before sharing it.

The parser bounds an individual JSONL line to 5 MiB, the whole export to 32 MiB,
and the record count to 100,000. Multi-session CLI queries accept at most eight
regular files and 64 MiB total input, then cap results at 500 events and 512
KiB. Parser errors do not include raw JSON lines.
The optional subprocess uses `Command` arguments, not a shell string; stderr is
drained and discarded, and nonzero exit diagnostics do not echo stderr.

## Untrusted evidence and agents

Every command, output value, operator field, and fixture value is untrusted
data. Bundles use structured JSON, keep evidence separate from fixed handling
guidance, and label values `untrusted_data`. The Switch agent brief instructs
the model not to follow instructions embedded in evidence. These controls reduce
confusion but do not provide a formal guarantee against prompt injection or
override the agent provider's system instructions. Give the connected agent
only the tools needed to inspect evidence; do not let it execute commands copied
from a transcript or perform recovery actions without human approval.

Legacy shell events do not establish whether a command was initiated by a
human or AI agent. Newer shell-hook events may carry actor/source metadata, but
the checked Relaylog hook reports the actor as unknown; the adapter preserves
that uncertainty and does not infer human attribution. Agent v1 actor/provider
fields support attribution as provider-reported hook evidence, not independent OS verification. The
Switch-connected agent's identity is not evidence of who performed actions in
the imported session. Claude Code and Gemini CLI capture are not claimed until
Relaylog core adapters emit the normalized v1 contract.

This repository has no Switch room/session mapping or room-aware authorization
boundary, so the local CLI cannot enforce that one Discord room is isolated
from another team's evidence. Keep each agent workspace and approved session
set scoped to the intended team until the existing Switch integration provides
an authorized mapping mechanism.

## Repository hygiene

`.env`, local evidence directories, binaries, logs, and build output are ignored.
The tracked incident fixture is fictional and contains no terminal output or
credentials. The `.env.example` intentionally contains no keys. No private
Relaylog source, dependency, history, recording, or internal URL belongs in this
repository.
