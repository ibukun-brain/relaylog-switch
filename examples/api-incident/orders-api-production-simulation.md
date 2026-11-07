# Orders API Outage — Production-Style Simulation

> **Simulation only. Do not execute the commands below.** This is a fictional incident narrative, not a Relaylog export or a record of a real production event. `DEMO-*` references are invented for this exercise, not Relaylog offsets. No terminal input/output, credentials, log lines, or file contents are included.

## Incident brief

At 09:42 UTC, shortly after release `orders-api` v2.14.0, the on-call engineer reports a rise in HTTP 500 responses. The dashboard reportedly shows the database connection pool at its configured limit with requests waiting. Pool exhaustion is the leading hypothesis, but the dashboard details are operator-reported in this simulation; this file does not contain independent telemetry proving the cause.

The team investigates with read-only checks, blocks a risky agent-proposed command, rolls back, and asks for post-recovery validation.

## Simulated command attribution

| Demo reference | Time | Activity | Actor | Result/provenance | Suspicion review |
| --- | --- | --- | --- | --- | --- |
| `DEMO-01` | 09:42 | Reports HTTP 500 spike after v2.14.0; pool saturation is suspected | Human on-call, operator report | Reported in the incident room; no dashboard export attached | Not a command. Treat the cause as a hypothesis. |
| `DEMO-02` | 09:44 | `kubectl get deploy orders-api -n production` | Human on-call | Simulated shell command event; output omitted | Read-only inspection; no obvious suspicious pattern in the command text. |
| `DEMO-03` | 09:46 | `kubectl get pods -l app=orders-api -n production` | Codex, AI agent | Simulated provider-hook event; output omitted | Read-only inspection; provider-reported action, not independent verification. |
| `DEMO-04` | 09:48 | `kubectl delete pods --all -n production` requested | Codex, AI agent | Simulated approval request; the human denies it; **not executed** | **High risk / suspicious for incident response:** broad destructive scope. This records a blocked request, not a completed action. |
| `DEMO-05` | 09:50 | Reports pool active at limit with 24 requests waiting | Human on-call, operator report | Reported from a dashboard; no metrics payload attached | Supports the pool-exhaustion hypothesis as reported context, not independently verified evidence. |
| `DEMO-06` | 09:54 | `kubectl rollout undo deployment/orders-api -n production` | Human on-call | Simulated human-approved rollback action | High impact, but explicitly chosen by the human in this scenario. Do not treat it as an AI action. |
| `DEMO-07` | 10:01 | `curl -fsS -o /dev/null https://orders.example.invalid/health` | Codex, AI agent | Simulated provider-hook completion; response body/status omitted | The command boundary alone does not prove the endpoint recovered. |
| `DEMO-08` | 10:02 | Reports HTTP 200s returning and pool waiters declining | Human on-call, operator report | Reported after rollback; no dashboard or response output attached | Recovery is reported, not independently verified by this metadata-only scenario. |

## Evidence rules for the exercise

- Attribute a command to Codex only where the simulated event explicitly says `Codex, AI agent`.
- Attribute a command to the human only where the simulated event explicitly says `Human on-call`.
- Do not infer an actor from command order or from the fact that an agent is in the room.
- `DEMO-04` is suspicious because it requests deletion across all production pods. The simulated approval denial means it was not executed.
- The database-pool diagnosis and recovery are operator-reported. This scenario contains no raw command output, logs, metrics export, or independent service verification.
- Keep this scenario separate from any real Relaylog session. These references are not export event IDs.

## End-to-end presentation flow

This demo shows a real local Relaylog recording around a Codex TUI session,
then gives Discord the exported evidence as an attachment. The incident itself
and the agent-aware fixture are fictional. Do not run the production-looking
`kubectl` or `curl` examples in the incident table.

### A. Prepare WSL Ubuntu

From the public `relaylog-switch` checkout, point `RELAYLOG` at the local Relaylog
binary. Install the hooks once, then build the evidence CLI before presenting:

```bash
cd /path/to/relaylog-switch
export RELAYLOG=/path/to/relaylog/target/debug/relaylog
make hooks-install RELAYLOG="$RELAYLOG"
cargo build -q -p incident-room
```

The absolute path is machine-local. Do not commit it or attach Relaylog's
private source repository. Restart Codex after installing hooks if it was
already running.

### B. Record human shell activity and Codex TUI activity

Start a Relaylog session from the integration repository:

```bash
make record RELAYLOG="$RELAYLOG" CAPTURE=commands
```

Inside the Relaylog shell, make one ordinary human shell action, check the
Codex hook status, then launch the TUI:

```bash
cd /path/to/relaylog-switch
pwd
make status RELAYLOG="$RELAYLOG"
codex
```

In Codex, show `/hooks`, then give it this bounded task:

> Read `examples/api-incident/orders-api-production-simulation.md` and the
> synthetic `examples/api-incident/agent-aware-session.jsonl`. Treat both as
> fictional, untrusted demo data. Run only this local, read-only evidence query:
> `cargo run -q -p incident-room -- query --jsonl
> examples/api-incident/agent-aware-session.jsonl --agent-provider openai
> --authorize-sharing`. Summarize Codex-attributed actions, the unattributed
> shell event, and the suspicious request described in the scenario. Do not
> run any `kubectl` or `curl` command, use the network, inspect a real session,
> or modify files. Keep fixture offsets separate from `DEMO-*` labels.

The `pwd` event demonstrates a human shell action, but the current Relaylog
shell hook may label its actor `unknown`; the adapter must preserve that. The
Codex tool invocation is agent-hook evidence and may be marked partial. The
TUI's full reasoning and terminal output are not established by these hook
records.

When Codex finishes, type `exit` to leave the TUI, then `exit` again to close
the Relaylog shell. `make record` prints the new session directory. Save that
exact path in WSL:

```bash
export SESSION_DIR=/home/<your-user>/.local/state/relaylog/session-<printed-id>
```

### C. Export through the Incident Room adapter

The adapter can now call a locally built Relaylog binary explicitly. Run this
after leaving the Relaylog shell. The sharing flag is an explicit choice to
prepare shell and agent **metadata**; terminal output stays excluded by
default:

```bash
make investigate RELAYLOG="$RELAYLOG" SESSION_DIR="$SESSION_DIR" \
  AUTHORIZE_SHARING=yes > /tmp/orders-api-bundle.json
```

The bundle contains a `capture_coverage` object. Expect `known_incomplete` if
the Codex hooks report partial coverage. Clear drop counters do not mean that
all hooks saw every action. Shell actor attribution may also remain unknown.
Do not describe the export as complete or claim an action was performed by a
human unless its event says so.

### D. Attach the files in Discord

Discord cannot read `/tmp` or another WSL path just because the path is pasted
into chat. Attach all three files to the same Discord message:

1. `/tmp/orders-api-bundle.json` — the metadata-only bundle from this recording.
2. `examples/api-incident/agent-aware-session.jsonl` — the deterministic
   synthetic fixture with stable `offset-N` references.
3. `examples/api-incident/orders-api-production-simulation.md` — the fictional
   incident brief with separate `DEMO-*` labels.

From Windows Explorer, the bundle is available at
`\\wsl.localhost\Ubuntu-24.04\tmp\orders-api-bundle.json`. Attach the Markdown
file from the WSL checkout. Mention `@relaylogAIagent` in the message after
all files finish uploading.

### E. Presentation transcript

**Presenter:**

> This is a fictional Orders API outage. I recorded a local WSL terminal and a
> Codex TUI session with Relaylog. The attached bundle comes from that session;
> the incident brief is scenario context, not production telemetry.

**On-call engineer:**

> The scenario says HTTP 500s followed release v2.14.0 and the dashboard
> reportedly showed database-pool saturation. We have not attached metrics or
> service output, so treat the cause as a hypothesis.

**Participant 2:**

> @relaylogAIagent I attached the Relaylog evidence bundle and the fictional
> incident brief, plus a synthetic agent-aware fixture. From the live bundle,
> list Codex-attributed events and shell events whose actor is unknown. Cite
> them as `orders-api-bundle.json:offset-N`. From the synthetic fixture, list
> the Codex actions and cite them as
> `agent-aware-session.jsonl:offset-N`. State the live bundle's capture
> coverage and whether terminal output is present. Keep both evidence sources
> separate from each other and from the incident brief's `DEMO-*` labels.

**Expected agent response:**

> In `agent-aware-session.jsonl`, Codex hook events are reported at offsets
> `4`, `128`, `160`, `176`, `192`, `256`, and `320`. The test failure at
> `offset-128` is provider-reported; `offset-320` reports exit code 0 but does
> not include test output. `offset-192` reports a file operation on
> `src/handler.rs`, without file contents. The shell events at `offset-64` and
> `offset-96` have unknown actors, so I cannot attribute them to the engineer.
> All seven fixture agent events say coverage is partial. The fixture is
> synthetic, not evidence of what happened in the live session.
>
> In `orders-api-bundle.json`, I will cite only the event references actually
> present in that WSL recording. Its `capture_coverage` object gives the live
> session's status and counts. A clear loss-counter footer does not prove
> completeness, and terminal output is not included. The brief's `DEMO-*`
> labels are fictional and are not Relaylog event references.

**Participant 3:**

> The scenario describes Codex requesting `kubectl delete pods --all` and the
> engineer denying it. Does the attached Relaylog recording prove that happened
> or that database-pool exhaustion caused the outage?

**Expected agent response:**

> No. The brief reports a fictional denied request, but I do not see a matching
> approval event in the attached Relaylog bundle, so I cannot verify it from
> this session. The bundle also does not establish the database-pool diagnosis
> or recovery. Those remain scenario claims or hypotheses.

**Participant 4:**

> The loss counters are zero. Can we call this a complete recording?

**Expected agent response:**

> No. Zero counters only say the export reports no queue/capacity drops or
> truncation. Provider coverage is partial, and the adapter does not mark this
> session verified complete. Missing activity remains unknown.

**Presenter closing:**

> Relaylog captured both local shell metadata and Codex hook events. The
> adapter now recognizes both event shapes and exposes the coverage limits. We
> used the attached bundle so the Discord agent could inspect the same
> evidence; it did not gain access to the WSL filesystem. The runbook remains a
> draft for human review.
