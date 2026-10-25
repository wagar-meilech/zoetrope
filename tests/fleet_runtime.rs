#![cfg(all(feature = "native", unix))]

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("/tmp").join(format!(
            "zrt-{}-{:08x}-{}",
            std::process::id(),
            nonce & 0xffff_ffff,
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let project = root.join("project");
        let sessions = root.join("codex/sessions/2026/10/25");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&sessions).unwrap();
        let header = serde_json::json!({"type":"session_meta","payload":{
            "id":"root-goal",
            "session_id":"root-goal",
            "cwd":std::fs::canonicalize(&project).unwrap(),"thread_source":"user","source":"cli"
        }});
        let prompt = serde_json::json!({"timestamp":"2026-10-25T10:00:00Z","type":"event_msg","payload":{
            "type":"item_completed","thread_id":"root-goal",
            "item":{"type":"UserMessage","content":[{"type":"input_text","text":"Inspect runtime fixture goal"}]}
        }});
        std::fs::write(
            sessions.join("rollout-runtime.jsonl"),
            format!("{header}\n{prompt}\n"),
        )
        .unwrap();
        zoetrope::workstate::Store::new(root.join("state"))
            .apply(
                &zoetrope::console::SessionKey {
                    provider: zoetrope::provider::Provider::Codex,
                    session_id: "root-goal".into(),
                },
                zoetrope::workstate::WorkUpdate::SetGoal {
                    title: "Inspect runtime fixture goal".into(),
                    criteria: vec![zoetrope::workstate::CriterionInput {
                        id: "recording".into(),
                        label: "Recording remains readable".into(),
                        weight: 1.0,
                    }],
                    reason: "Executable fixture".into(),
                    provenance: "operator".into(),
                },
            )
            .unwrap();
        Self(root)
    }

    fn run(&self, scenario: &str) -> serde_json::Value {
        let output = Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/fleet_terminal.py"
            ))
            .arg(env!("CARGO_BIN_EXE_zoe"))
            .arg(&self.0)
            .arg(scenario)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "PTY driver failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).expect("PTY driver returns observations")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn bare_launch_opens_the_fleet_and_quits_cleanly() {
    let observed = Fixture::new().run("bare");
    assert_eq!(observed["fleet"], true, "{observed}");
    assert_eq!(observed["exit"], 0, "{observed}");
}

#[test]
fn project_flag_preserves_the_current_project_session_view() {
    let observed = Fixture::new().run("project");
    assert_eq!(observed["project"], true, "{observed}");
    assert_eq!(observed["fleet"], false, "{observed}");
    assert_eq!(observed["exit"], 0, "{observed}");
}

#[test]
fn feedback_autosaves_without_another_keypress() {
    let observed = Fixture::new().run("autosave");
    assert_eq!(observed["autosaved"], true, "{observed}");
    assert_eq!(observed["exit"], 0, "{observed}");
}

#[test]
fn terminal_paste_keeps_multiline_feedback_literal_and_restores_paste_mode() {
    let observed = Fixture::new().run("paste");
    assert_eq!(observed["paste_enabled"], true, "{observed}");
    assert_eq!(observed["autosaved"], true, "{observed}");
    assert_eq!(observed["paste_disabled"], true, "{observed}");
    assert_eq!(observed["exit"], 0, "{observed}");
}

#[test]
fn replay_returns_to_the_same_goal_and_draft_without_live_actions() {
    let fixture = Fixture::new();
    let observed = fixture.run("replay");
    assert_eq!(observed["replay"], true, "{observed}");
    assert_eq!(observed["returned"], true, "{observed}");
    assert_eq!(observed["draft"], true, "{observed}");
    assert_eq!(observed["peer_exit"], 0, "{observed}");
    assert_eq!(observed["exit"], 0, "{observed}");

    let output = Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["state", "--root"])
        .arg(fixture.0.join("state"))
        .args(["show", "codex", "root-goal"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let saved: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(saved["record"]["drafts"][0]["text"], "Keep H/q R draft");
    assert_eq!(saved["record"]["outbox"], serde_json::json!([]));
}
