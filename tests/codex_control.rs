#![cfg(all(feature = "native", unix))]
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Peer {
    dir: PathBuf,
    child: std::process::Child,
}
impl Peer {
    fn new(case: &str) -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            PathBuf::from("/tmp").join(format!("zoe-ctrl-{}-{nonce}-{unique}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let child = Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/codex_peer.py"
            ))
            .arg(dir.join("peer.sock"))
            .arg(case)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let peer = Self { dir, child };
        let deadline = Instant::now() + Duration::from_secs(3);
        while !peer.dir.join("peer.sock.ready").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        peer
    }
    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_zoe"))
            .args(["control", "--socket"])
            .arg(self.dir.join("peer.sock"))
            .args(args)
            .output()
            .unwrap()
    }
    fn finish(mut self) {
        let status = self.child.wait().unwrap();
        let mut error = String::new();
        use std::io::Read;
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut error)
            .unwrap();
        assert!(status.success(), "external peer failed: {error}");
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn status_reads_the_exact_live_thread_without_starting_or_joining_work() {
    let peer = Peer::new("status");
    let result = peer.run(&["status", "child-thread"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("Working"), "{text}");
    assert!(text.contains("turn-current"), "{text}");
    peer.finish();
}

#[test]
fn guidance_is_accepted_by_the_selected_child_and_exact_active_turn() {
    let peer = Peer::new("guide");
    let result = peer.run(&[
        "guide",
        "child-thread",
        "--turn",
        "turn-current",
        "Keep the public interface",
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("Accepted"), "{text}");
    assert!(text.contains("completed") && text.contains("not"), "{text}");
    peer.finish();
}

#[test]
fn queueing_is_distinct_from_guidance_and_keeps_a_stable_submission_id() {
    for (case, args, expected) in [(
        "queue",
        vec![
            "queue",
            "child-thread",
            "--message-id",
            "zoe-item-17",
            "Do the docs later",
        ],
        "Queued",
    )] {
        let peer = Peer::new(case);
        let result = peer.run(&args);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(String::from_utf8_lossy(&result.stdout).contains(expected));
        peer.finish();
    }
}

#[test]
fn stopping_interrupts_only_the_pinned_turn_of_the_selected_thread() {
    let peer = Peer::new("stop");
    let result = peer.run(&["stop", "child-thread", "--turn", "turn-current"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(
        text.contains("Accepted") && text.contains("interruption"),
        "{text}"
    );
    peer.finish();
}

#[test]
fn idle_guidance_explicitly_starts_input_on_the_loaded_idle_thread() {
    let peer = Peer::new("idle");
    let result = peer.run(&["guide", "child-thread", "--idle", "Continue the work"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("new-turn"));
    peer.finish();
}

#[test]
fn join_replays_questions_without_policy_overrides_and_observes_native_resolution() {
    let peer = Peer::new("join");
    let result = peer.run(&["join", "child-thread"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(
        text.contains("Choose a label") && text.contains("QuestionResolved"),
        "{text}"
    );
    peer.finish();
}

#[test]
fn an_answer_requires_a_current_question_replay_and_preserves_explicit_choices() {
    let peer = Peer::new("answer");
    let result = peer.run(&[
        "answer",
        "child-thread",
        "--turn",
        "turn-current",
        "--request",
        "7",
        "--answers",
        r#"{"choice":["Beta"]}"#,
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(
        text.contains("Submitted") && text.contains("awaiting"),
        "{text}"
    );
    peer.finish();
}

#[test]
fn a_question_resolved_elsewhere_cannot_receive_a_stale_saved_answer() {
    let peer = Peer::new("resolved-answer");
    let result = peer.run(&[
        "answer",
        "child-thread",
        "--turn",
        "turn-current",
        "--request",
        "7",
        "--answers",
        r#"{"choice":["Beta"]}"#,
    ]);
    assert!(!result.status.success());
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(text.contains("no longer confirmed pending"), "{text}");
    peer.finish();
}

#[test]
fn loaded_inventory_pages_without_resuming_historical_threads() {
    let peer = Peer::new("loaded");
    let result = peer.run(&["loaded"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(
        text.contains("quiet-working") && text.contains("waiting-question"),
        "{text}"
    );
    peer.finish();
}

#[test]
fn runtime_rejection_is_known_but_lost_acknowledgment_is_unknown_and_not_retried() {
    for (case, expected) in [("reject", "Rejected"), ("disconnect", "Unknown")] {
        let peer = Peer::new(case);
        let result = peer.run(&[
            "guide",
            "child-thread",
            "--turn",
            "turn-current",
            "Keep the public interface",
        ]);
        assert!(!result.status.success());
        let text = String::from_utf8_lossy(&result.stdout);
        assert!(text.contains(expected), "{text}");
        peer.finish();
    }
}

#[test]
fn stale_turn_and_unavailable_child_input_never_redirect_to_the_parent() {
    for (case, turn) in [("stale", "old-turn"), ("no-direct", "turn-current")] {
        let peer = Peer::new(case);
        let result = peer.run(&[
            "guide",
            "child-thread",
            "--turn",
            turn,
            "Keep the public interface",
        ]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stdout).contains("Unavailable"));
        peer.finish();
    }
}

#[test]
fn input_restrictions_do_not_prevent_an_exact_child_interruption_request() {
    let peer = Peer::new("restricted-stop");
    let result = peer.run(&["stop", "child-thread", "--turn", "turn-current"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("Accepted"));
    peer.finish();
}

#[test]
fn an_incompatible_steering_acknowledgment_is_not_claimed_as_accepted() {
    let peer = Peer::new("wrong-ack");
    let result = peer.run(&[
        "guide",
        "child-thread",
        "--turn",
        "turn-current",
        "Keep the public interface",
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("Unknown"));
    peer.finish();
}
