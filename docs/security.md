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
- Nonzero capture-loss counters, truncation, unknown shell event types, and
  unknown capture types fail closed for agent projection.
- There is no general secret detector. Command arguments, working directories,
  origin values, and even allowed metadata can contain secrets or personal
  information. Review the selected session before sharing it.

The parser bounds an individual JSONL line to 5 MiB, the whole export to 32 MiB,
and the record count to 100,000. Parser errors do not include raw JSON lines.
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

## Repository hygiene

`.env`, local evidence directories, binaries, logs, and build output are ignored.
The tracked incident fixture is fictional and contains no terminal output or
credentials. The `.env.example` intentionally contains no keys. No private
Relaylog source, dependency, history, recording, or internal URL belongs in this
repository.
