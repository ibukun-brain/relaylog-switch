# Synthetic API 500 Incident

This fictional example uses the current Relaylog JSONL export shape. Its
session-like data, operators, commands, hostnames, and service names are all
invented. It contains command-start and command-finish metadata, plus a balanced
private-mode marker pair. It has no captured terminal input or output.

The scenario premise is that an API returned HTTP 500 errors. The investigation
checks API health, container status, logs, database readiness, restarts a
fictional API container, and checks health again. The shell metadata records
commands and shell exit statuses only; it does not record the HTTP status body,
container log lines, or database diagnostic output in this fixture. The
database connection pool exhaustion hypothesis is scenario context, not a
finding supported by these JSONL records.

Run the deterministic local draft with:

```bash
cargo run -p incident-room -- demo --fixture examples/api-incident/synthetic-session.jsonl
```

Add `--authorize-sharing` to include the fictional command metadata in the
local report output.
