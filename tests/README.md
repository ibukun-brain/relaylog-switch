# Test Layout

Library contract and privacy tests live in
`crates/relaylog-switch/tests/`; CLI report tests live beside the application
workflow in `apps/incident-room/src/`. All fixtures and subprocess stubs are
synthetic. The workspace test suite has no network, credential, private-crate,
or production-recording dependency.
