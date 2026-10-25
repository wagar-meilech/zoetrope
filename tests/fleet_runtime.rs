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
        let root = std::env::temp_dir().join(format!(
            "zoe-runtime-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let project = root.join("project");
        let sessions = root.join("codex/sessions/2026/10/25");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&sessions).unwrap();
        let header = serde_json::json!({"type":"session_meta","payload":{
            "id":"11111111-2222-3333-4444-555555555555",
            "session_id":"11111111-2222-3333-4444-555555555555",
            "cwd":std::fs::canonicalize(&project).unwrap(),"thread_source":"user","source":"cli"
        }});
        let prompt = serde_json::json!({"timestamp":"2026-10-25T10:00:00Z","type":"event_msg","payload":{
            "type":"item_completed","thread_id":"11111111-2222-3333-4444-555555555555",
            "item":{"type":"UserMessage","content":[{"type":"input_text","text":"Inspect runtime fixture goal"}]}
        }});
        std::fs::write(
            sessions.join("rollout-runtime.jsonl"),
            format!("{header}\n{prompt}\n"),
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
