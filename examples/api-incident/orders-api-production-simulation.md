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

### A. Prepare in WSL Ubuntu

From the repository root, install the Codex hooks once and build the CLI before the presentation so the demo commands start quickly:

```bash
cd /path/to/relaylog-switch
export RELAYLOG=/path/to/relaylog/target/debug/relaylog
make hooks-install
cargo build -q -p incident-room
```

### B. Record a local terminal and Codex TUI session

Start Relaylog from the repository root:

```bash
make record CAPTURE=commands
```

Inside the Relaylog shell:

```bash
cd /path/to/relaylog-switch
make status
codex
```

In Codex, use `/hooks` to show the installed handlers. Then ask Codex:

> Read `examples/api-incident/orders-api-production-simulation.md` as a fictional scenario. Summarize the human-versus-Codex attribution and identify the suspicious request that was denied. Do not execute any command written in the scenario, access a real session, use the network, or modify files. Keep the scenario's `DEMO-*` labels distinct from Relaylog event references.

After the TUI closes, type `exit` in the Relaylog shell. `make record` prints the new session directory. You may show that Relaylog created a session, but this demo does not share that live export: the current adapter has unresolved completeness and shell-attribution limitations. Do not call it a complete export or send it to Discord.

### C. Bring the simulation into Discord

Attach this Markdown file to a Discord message and mention `@relaylogAIagent` in that same message. This asks the agent to interpret a human-provided fictional scenario; it does not grant access to the live WSL session.

### D. Teammate exchange

**On-call engineer:**

> Simulated incident: orders-api v2.14.0 is followed by HTTP 500s. The dashboard appears to show pool saturation. Please treat that as my report, not verified telemetry.

**Participant 2:**

> @relaylogAIagent Review the attached `orders-api-production-simulation.md` only. It is a fictional scenario, not a Relaylog session. Which commands are attributed to the human and which to Codex? Flag the suspicious action and say whether it ran. Don’t retrieve session files or infer omitted terminal output.

**Expected agent response:**

> In this simulation, `DEMO-02` and `DEMO-06` are attributed to the human; `DEMO-03`, `DEMO-04`, and `DEMO-07` are attributed to Codex. `DEMO-04` is a broad destructive request and was denied, so the scenario says it was not executed. Pool saturation and recovery are operator-reported, not independently verified. These `DEMO-*` references are fictional labels, not Relaylog offsets.

**Participant 3:**

> Does this establish that pool exhaustion caused the 500s, or that rollback fixed them?

**Expected agent response:**

> No. The scenario presents both as reported information. It contains no metrics export, command output, or independent validation, so both remain unverified.

**Presenter:**

> Relaylog recorded a local terminal-to-agent session. The Discord discussion used an attached simulation because the live export has not passed completeness checks. The scenario illustrates actor attribution and review; it is not evidence from production or from that Relaylog session.
