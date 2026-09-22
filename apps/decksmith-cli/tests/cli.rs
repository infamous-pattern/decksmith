use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_decksmithctl"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn valid_output_is_json_without_logs() {
    let o = run(&["virtual", "info"], "");
    assert!(o.status.success());
    assert!(o.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap()["dials"],
        4
    );
}
#[test]
fn invalid_input_is_not_echoed_to_logs() {
    let o = run(&["virtual", "emulate"], "private-canary-content\n");
    assert!(!o.status.success());
    assert!(o.stdout.is_empty());
    let log: serde_json::Value = serde_json::from_slice(&o.stderr).unwrap();
    assert_eq!(log["fields"]["error_code"], "invalid_json");
    assert!(
        !String::from_utf8(o.stderr)
            .unwrap()
            .contains("private-canary-content")
    );
}
#[test]
fn rejects_oversized_input_and_unknown_commands() {
    assert!(
        !run(&["virtual", "emulate"], &"x".repeat(17000))
            .status
            .success()
    );
    let o = run(&["bogus"], "");
    assert!(!o.status.success());
    assert!(o.stdout.is_empty());
}
#[test]
fn round_trip_preserves_events() {
    let s = "{\"timestamp_ms\":0,\"event\":{\"type\":\"key\",\"index\":0,\"pressed\":true}}\n";
    let o = run(&["virtual", "emulate"], s);
    assert!(o.status.success());
    assert!(o.stderr.is_empty());
    assert_eq!(String::from_utf8(o.stdout).unwrap(), s);
}

#[test]
fn semantic_replay_emits_double_long_repeat_and_cancels_on_disconnect() {
    let source = include_str!("../../../examples/triggers.jsonl");
    let o = run(&["virtual", "resolve"], source);
    assert!(o.status.success());
    assert!(o.stderr.is_empty());
    let rows: Vec<serde_json::Value> = String::from_utf8(o.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let types: Vec<&str> = rows
        .iter()
        .filter_map(|r| r["trigger"]["kind"]["type"].as_str())
        .collect();
    assert_eq!(
        types,
        vec![
            "raw_press",
            "raw_release",
            "raw_press",
            "raw_release",
            "double_press",
            "raw_press",
            "long_press",
            "hold_repeat"
        ]
    );
    assert_eq!(
        rows.iter().filter(|r| r["record"] == "lifecycle").count(),
        2
    );
}
#[test]
fn replay_rejects_input_while_disconnected() {
    let o = run(
        &["virtual", "resolve"],
        "{\"op\":\"disconnect\",\"timestamp_ms\":0}\n{\"op\":\"input\",\"input\":{\"timestamp_ms\":1,\"event\":{\"type\":\"key\",\"index\":0,\"pressed\":true}}}\n",
    );
    assert!(!o.status.success());
}

#[test]
fn hardware_commands_require_explicit_exclusive_flag() {
    // These calls must be rejected by argument parsing before touching hardware.
    for args in [["hardware", "info"], ["hardware", "monitor"]] {
        let o = run(&args, "");
        assert!(!o.status.success());
        let log: serde_json::Value = serde_json::from_slice(&o.stderr).unwrap();
        assert_eq!(log["fields"]["error_code"], "invalid_command");
    }
}

#[test]
fn monitor_rejects_invalid_duration_before_opening_device() {
    for seconds in ["0", "121", "-1", "invalid"] {
        let o = run(
            &["hardware", "monitor", "--exclusive", "--seconds", seconds],
            "",
        );
        assert!(!o.status.success());
        let log: serde_json::Value = serde_json::from_slice(&o.stderr).unwrap();
        assert_eq!(log["fields"]["error_code"], "invalid_command");
    }
}
