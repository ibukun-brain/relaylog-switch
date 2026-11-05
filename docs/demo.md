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
The JSONL contains shell command metadata and exit statuses only. It does not
contain actual API responses, container logs, database output, or timestamps.
The report therefore does not claim that the HTTP error or its cause was
verified by the fixture.

## Switch walkthrough

1. Set up a Switch room with the existing coding-agent workflow in
   [`../switch/README.md`](../switch/README.md).
2. Ask the agent to inspect the synthetic fixture and cite IDs such as
   `offset-4` in the draft.
3. Point out that the command and exit status are observed metadata, while the
   HTTP 500 and database pool exhaustion are synthetic scenario context.
4. Ask the human participant to challenge the hypothesis and request missing
   evidence before any conclusion is called verified.
5. Show the `Human Review Status` remains `Not reviewed by a human`.

## Short recording outline

1. State the incident and show the human and Switch agent in the shared room.
2. Run the local fixture demo and show the evidence order and report sections.
3. Have the agent produce a draft with evidence references and an explicit
   unresolved-questions section.
4. Review one hypothesis together and leave it unverified until the human
   agrees.
5. Explain that real session evidence needs session-specific operator approval
   before it enters the agent context.
