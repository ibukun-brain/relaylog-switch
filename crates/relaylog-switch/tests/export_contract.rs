use std::io::Cursor;

use relaylog_switch::{CaptureRecord, EvidenceProvenance, ExportError, load_jsonl};

const VALID_EXPORT: &str = concat!(
    "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\",\"command\":\"curl -fsS http://api.example.test/health\",\"cwd\":\"/srv/example-api\",\"shell_depth\":1}}}\n",
    "{\"spool_offset\":128,\"record\":{\"type\":\"origin\",\"value\":\"human\",\"source\":\"operator\"}}\n",
    "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
);

#[test]
fn valid_jsonl_preserves_order_and_provenance() {
    let parsed = load_jsonl(Cursor::new(VALID_EXPORT)).expect("verified export should parse");

    assert_eq!(parsed.records().len(), 2);
    assert_eq!(parsed.records()[0].id().spool_offset(), 4);
    assert_eq!(parsed.records()[1].id().spool_offset(), 128);
    assert_eq!(
        parsed.records()[0].provenance(),
        EvidenceProvenance::Observed
    );
    assert_eq!(
        parsed.records()[1].provenance(),
        EvidenceProvenance::OperatorReported
    );
    assert!(matches!(
        parsed.records()[0].record(),
        CaptureRecord::Shell { event } if event["type"] == "started"
    ));
}

#[test]
fn empty_input_is_rejected_as_an_incomplete_export() {
    let error = load_jsonl(Cursor::new("")).expect_err("empty input has no required summary");

    assert!(matches!(
        error,
        ExportError::MissingCaptureGaps { after_line: 0 }
    ));
}

#[test]
fn malformed_json_is_rejected_without_echoing_the_line() {
    let error = load_jsonl(Cursor::new("{not-json secret-value}\n"))
        .expect_err("malformed evidence must be rejected");

    assert!(matches!(error, ExportError::MalformedJson { line: 1, .. }));
    assert!(!error.to_string().contains("secret-value"));
}

#[test]
fn unknown_shell_event_is_preserved_for_local_review() {
    let jsonl = concat!(
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"future_event\",\"note\":\"untrusted text\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let parsed = load_jsonl(Cursor::new(jsonl)).expect("opaque shell event stays reviewable");

    assert_eq!(parsed.records().len(), 1);
    assert_eq!(
        parsed.records()[0].provenance(),
        EvidenceProvenance::Observed
    );
}

#[test]
fn optional_shell_metadata_can_be_absent() {
    let jsonl = concat!(
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let parsed = load_jsonl(Cursor::new(jsonl)).expect("nested metadata remains opaque");

    assert_eq!(parsed.records().len(), 1);
}

#[test]
fn duplicate_spool_offsets_are_rejected() {
    let jsonl = concat!(
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n",
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"finished\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("duplicate offsets lose provenance");

    assert!(matches!(error, ExportError::NonIncreasingOffset { .. }));
}

#[test]
fn out_of_order_spool_offsets_are_rejected_without_sorting() {
    let jsonl = concat!(
        "{\"spool_offset\":128,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n",
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"finished\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("source order is part of provenance");

    assert!(matches!(error, ExportError::NonIncreasingOffset { .. }));
}

#[test]
fn data_after_the_final_capture_summary_is_rejected() {
    let jsonl = concat!(
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n",
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("summary must be the last line");

    assert!(matches!(
        error,
        ExportError::DataAfterCaptureGaps { line: 2 }
    ));
}

#[test]
fn reported_capture_loss_is_rejected() {
    let jsonl = concat!(
        "{\"spool_offset\":4,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":1,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("loss makes the timeline incomplete");

    assert!(matches!(error, ExportError::EvidenceLoss));
}

#[test]
fn oversized_jsonl_line_is_rejected_before_parsing() {
    let oversized_value = "x".repeat(5 * 1024 * 1024);
    let jsonl = format!(
        "{{\"spool_offset\":4,\"record\":{{\"type\":\"shell\",\"event\":{{\"type\":\"started\",\"note\":\"{oversized_value}\"}}}}}}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("large lines must stay bounded");

    assert!(matches!(error, ExportError::LineTooLarge { line: 1 }));
}

#[test]
fn unsupported_wrapper_fields_fail_closed() {
    let jsonl = concat!(
        "{\"spool_offset\":4,\"privacy\":\"unknown\",\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("unknown wrapper metadata is ambiguous");

    assert!(matches!(error, ExportError::InvalidRecord { line: 1 }));
}

#[test]
fn duplicate_json_keys_are_rejected() {
    let jsonl = concat!(
        "{\"spool_offset\":4,\"spool_offset\":8,\"record\":{\"type\":\"shell\",\"event\":{\"type\":\"started\"}}}\n",
        "{\"type\":\"capture_gaps\",\"counts\":{\"queue_dropped_records\":0,\"capacity_dropped_records\":0},\"truncated_at\":null}\n"
    );

    let error = load_jsonl(Cursor::new(jsonl)).expect_err("duplicate keys are ambiguous");

    assert!(matches!(error, ExportError::MalformedJson { line: 1, .. }));
}
