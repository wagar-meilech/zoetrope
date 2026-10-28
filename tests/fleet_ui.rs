#![cfg(feature = "native")]
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zoetrope::console::{DiscoveryRoots, SessionKey};
use zoetrope::fleet::{FleetConfig, FleetConsole};
use zoetrope::provider::Provider;
use zoetrope::workstate::{CriterionInput, Evidence, EvidenceKind, Store, WorkUpdate};

struct Home(PathBuf);
impl Home {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!("zoe-ui-{}-{now}-{n}", std::process::id())))
    }
    fn session(&self, id: &str, project: &str, archived: bool) {
        let sub = if archived {
            "archived_sessions"
        } else {
            "sessions/2026/10/23"
        };
        let path = self.0.join(format!("codex/{sub}/rollout-{id}.jsonl"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path,format!("{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"session_id\":\"{id}\",\"cwd\":\"{project}\",\"thread_source\":\"user\"}}}}\n")).unwrap();
    }
    fn key(&self, id: &str) -> SessionKey {
        SessionKey {
            provider: Provider::Codex,
            session_id: id.into(),
        }
    }
    fn goal(&self, id: &str, title: &str) {
        Store::new(self.0.join("state"))
            .apply(
                &self.key(id),
                WorkUpdate::SetGoal {
                    title: title.into(),
                    criteria: vec![
                        CriterionInput {
                            id: "build".into(),
                            label: "Build passes".into(),
                            weight: 1.0,
                        },
                        CriterionInput {
                            id: "check".into(),
                            label: "Behavior checked".into(),
                            weight: 1.0,
                        },
                    ],
                    reason: "Initial scope".into(),
                    provenance: "Operator".into(),
                },
            )
            .unwrap();
    }
    #[cfg(unix)]
    fn enable_unrelated_tracking(&self) {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
            .args(["tracking", "enable", "--project"])
            .arg(&self.0)
            .args([
                "--provider",
                "codex",
                "--session",
                "unrelated",
                "--source",
                "MISSING.md#Tasks",
                "--root",
            ])
            .arg(self.0.join("state"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    #[cfg(unix)]
    fn hold_goal(&self, session: &str) -> HeldLock {
        // Select the fixture's public record by identity. The state directory
        // may also contain tracking.lock or another goal's lock.
        let identity = serde_json::to_value(self.key(session)).unwrap();
        let records = std::fs::read_dir(self.0.join("state"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .filter(|path| {
                serde_json::from_slice::<serde_json::Value>(&std::fs::read(path).unwrap()).unwrap()
                    ["session"]
                    == identity
            })
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1, "expected one goal record for {session}");
        let lock = records[0].with_extension("lock");
        assert!(
            lock.exists(),
            "goal record must have its own contention lock"
        );
        HeldLock::acquire(&lock)
    }
    fn ui(&self) -> FleetConsole {
        FleetConsole::new(FleetConfig {
            roots: DiscoveryRoots {
                codex_home: Some(self.0.join("codex")),
                claude_projects: None,
            },
            state_root: self.0.join("state"),
            socket: self.0.join("no-runtime.sock"),
            ..FleetConfig::default()
        })
    }
}
impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn key(ui: &mut FleetConsole, code: KeyCode) {
    ui.handle_event(&Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}
fn text(ui: &mut FleetConsole, w: u16, h: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .chunks(w as usize)
        .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
fn wait_text(ui: &mut FleetConsole, needle: &str) -> String {
    let until = Instant::now() + Duration::from_secs(8);
    loop {
        ui.poll();
        let rendered = text(ui, 150, 42);
        if rendered.contains(needle) {
            return rendered;
        }
        assert!(Instant::now() < until, "did not show {needle}: {rendered}");
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn opening_fleet_shows_goal_progress_and_history_uses_the_same_graph() {
    let home = Home::new();
    home.session("current", "/projects/compiler", false);
    home.goal("current", "Improve compiler");
    home.session("past", "/projects/editor", true);
    home.goal("past", "Restore editor");
    Store::new(home.0.join("state"))
        .apply(
            &home.key("current"),
            WorkUpdate::Verify {
                id: "build".into(),
                evidence: vec![Evidence {
                    kind: EvidenceKind::UserAttestation,
                    detail: "Release build passed".into(),
                    provenance: "Operator".into(),
                }],
                revision: Some(1),
            },
        )
        .unwrap();
    let mut ui = home.ui();
    let current = wait_text(&mut ui, "Improve compiler");
    assert!(current.contains("1 of 2 verified"), "{current}");
    assert!(!current.contains("Restore editor"));
    key(&mut ui, KeyCode::Char('H'));
    let history = wait_text(&mut ui, "Restore editor");
    assert!(history.contains("All history"));
    assert!(history.contains("Improve compiler"));
    ui.shutdown();
}
fn type_text(ui: &mut FleetConsole, value: &str) {
    for ch in value.chars() {
        key(ui, KeyCode::Char(ch));
    }
}
#[test]
fn a_searched_goals_feedback_survives_typing_shortcuts_navigation_and_restart() {
    let home = Home::new();
    home.session("current", "/projects/compiler", false);
    home.goal("current", "Improve compiler");
    home.session("past", "/projects/editor", true);
    home.goal("past", "Restore editor");
    let mut ui = home.ui();
    wait_text(&mut ui, "Improve compiler");
    key(&mut ui, KeyCode::Char('H'));
    key(&mut ui, KeyCode::Char('/'));
    type_text(&mut ui, "Restore editor");
    key(&mut ui, KeyCode::Enter);
    wait_text(&mut ui, "Search: Restore editor");
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Selected: Restore editor");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "H q: please preserve draft");
    key(&mut ui, KeyCode::Esc);
    let saved = wait_text(&mut ui, "Draft saved");
    assert!(
        saved.contains("All history"),
        "typing H must not toggle history: {saved}"
    );
    ui.shutdown();
    let mut restarted = home.ui();
    wait_text(&mut restarted, "Improve compiler");
    key(&mut restarted, KeyCode::Char('H'));
    key(&mut restarted, KeyCode::Char('/'));
    type_text(&mut restarted, "Restore editor");
    key(&mut restarted, KeyCode::Enter);
    wait_text(&mut restarted, "Search: Restore editor");
    key(&mut restarted, KeyCode::Down);
    wait_text(&mut restarted, "Selected: Restore editor");
    key(&mut restarted, KeyCode::Char('f'));
    wait_text(&mut restarted, "H q: please preserve draft");
    restarted.shutdown();
}
fn ctrl(ui: &mut FleetConsole, ch: char) {
    ui.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char(ch),
        KeyModifiers::CONTROL,
    )));
}
#[test]
fn goal_criteria_can_be_edited_verified_with_attestation_and_reopened() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Compiler work");
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Compiler work");
    key(&mut ui, KeyCode::Char('c'));
    key(&mut ui, KeyCode::Char('e'));
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "Safer compiler");
    key(&mut ui, KeyCode::Enter);
    type_text(&mut ui, "Parses source");
    key(&mut ui, KeyCode::Enter);
    type_text(&mut ui, "Regression tests pass");
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Selected: Safer compiler");
    key(&mut ui, KeyCode::Char('v'));
    type_text(&mut ui, "I checked the parser with a real input");
    key(&mut ui, KeyCode::F(2));
    let verified = wait_text(&mut ui, "1 of 2 verified");
    assert!(verified.contains("I checked the parser"), "{verified}");
    key(&mut ui, KeyCode::Char('u'));
    let reopened = wait_text(&mut ui, "0 of 2 verified");
    assert!(
        reopened.contains("Prior verification retained"),
        "{reopened}"
    );
    ui.shutdown();
}
#[test]
fn an_incomplete_heavily_weighted_goal_never_claims_one_hundred_percent() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    let store = Store::new(home.0.join("state"));
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::SetGoal {
                title: "Release compiler".into(),
                criteria: vec![
                    CriterionInput {
                        id: "build".into(),
                        label: "Build".into(),
                        weight: 999.0,
                    },
                    CriterionInput {
                        id: "check".into(),
                        label: "Final check".into(),
                        weight: 1.0,
                    },
                ],
                reason: "scope".into(),
                provenance: "operator".into(),
            },
        )
        .unwrap();
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::Verify {
                id: "build".into(),
                evidence: vec![Evidence {
                    kind: EvidenceKind::UserAttestation,
                    detail: "Build passed".into(),
                    provenance: "operator".into(),
                }],
                revision: Some(1),
            },
        )
        .unwrap();
    let mut ui = home.ui();
    let view = wait_text(&mut ui, "1 of 2 verified");
    assert!(!view.contains("· 100%"), "{view}");
    assert!(view.contains("<100%"), "{view}");
    ui.shutdown();
}
#[cfg(unix)]
struct Peer {
    dir: PathBuf,
    child: std::process::Child,
}
#[cfg(unix)]
impl Peer {
    fn new(case: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = PathBuf::from("/tmp").join(format!("zui-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let child = std::process::Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/codex_fleet_peer.py"
            ))
            .arg(dir.join("peer.sock"))
            .arg(case)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !dir.join("peer.sock.ready").exists() {
            assert!(Instant::now() < deadline, "peer failed to start");
            std::thread::sleep(Duration::from_millis(10));
        }
        Self { dir, child }
    }
    fn finish(mut self) {
        let status = self.child.wait().unwrap();
        use std::io::Read;
        let mut err = String::new();
        self.child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut err)
            .unwrap();
        assert!(status.success(), "peer failed: {err}");
    }
}
#[cfg(unix)]
impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
#[cfg(unix)]
#[test]
fn guidance_queue_and_stop_reach_the_selected_child_without_parent_routing() {
    let peer = Peer::new("control");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser goal");
    let path = home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl");
    std::fs::write(path,"{\"type\":\"session_meta\",\"payload\":{\"id\":\"child-thread\",\"session_id\":\"root-goal\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root-goal\",\"agent_nickname\":\"Parser\"}}\n").unwrap();
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Parser goal");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Coordinator");
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    wait_text(&mut ui, "Working");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Keep the public interface");
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Accepted by Codex");
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "Do docs later");
    key(&mut ui, KeyCode::F(3));
    wait_text(&mut ui, "Queued by Codex");
    key(&mut ui, KeyCode::F(4));
    wait_text(&mut ui, "accepted the interruption");
    ui.shutdown();
    peer.finish();
}
#[test]
fn explicitly_selecting_coordinator_keeps_the_child_draft_with_its_owner() {
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser goal");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"root-goal","thread_source":"subagent","parent_thread_id":"root-goal","agent_nickname":"Parser"}}
"#).unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Parser goal");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Keep this draft for Parser");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    key(&mut ui, KeyCode::Char('C'));
    wait_text(&mut ui, "Recipient: Coordinator");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Coordinate the next work item");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    ui.shutdown();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["state", "--root"])
        .arg(home.0.join("state"))
        .args(["show", "codex", "root-goal"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let record: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let drafts = record["record"]["drafts"].as_array().unwrap();
    assert!(
        drafts
            .iter()
            .any(|d| d["target"]["agent_id"] == "child-thread"
                && d["text"] == "Keep this draft for Parser")
    );
    assert!(
        drafts
            .iter()
            .any(|d| d["target"]["agent_id"] == "main"
                && d["text"] == "Coordinate the next work item")
    );
}

#[cfg(unix)]
#[test]
fn criteria_requests_read_compact_context_without_saved_private_material() {
    let home = Home::new();
    let project = home.0.join("agent-project");
    std::fs::create_dir_all(&project).unwrap();
    home.session("goal", project.to_str().unwrap(), false);
    home.goal("goal", "Review completion criteria");
    let store = Store::new(home.0.join("state"));
    let target =
        serde_json::json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    for update in [
        serde_json::json!({"op":"save_draft","target":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"child"},"text":"PRIVATE SAVED DRAFT"}),
        serde_json::json!({"op":"capture_question","question":{"id":"private","target":target,
            "prompts":[{"id":"secret","question":"Private input","is_secret":true}],"state":"expired",
            "answers":{"secret":["PRIVATE SECRET ANSWER"]}}}),
        serde_json::json!({"op":"create_feedback","id":"saved-outbox","target":target,"text":"PRIVATE SAVED OUTBOX","mode":"guidance"}),
    ] {
        store
            .apply(&home.key("goal"), serde_json::from_value(update).unwrap())
            .unwrap();
    }
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        state_executable: Some(PathBuf::from(env!("CARGO_BIN_EXE_zoe"))),
        socket: home.0.join("no-runtime.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Review completion criteria");
    key(&mut ui, KeyCode::Char('A'));
    ui.try_shutdown().unwrap();
    let saved = store.read(&home.key("goal")).unwrap();
    let draft = &saved
        .drafts
        .iter()
        .find(|draft| draft.target.agent_id == "main")
        .unwrap()
        .text;
    assert!(draft.contains(" context codex 'goal'"), "{draft}");
    assert!(!draft.contains(" show "), "{draft}");
    let command = draft
        .lines()
        .find(|line| line.contains(" state --root ") && line.contains(" context "))
        .unwrap();
    let output = std::process::Command::new("/bin/sh")
        .args(["-c", command])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let context: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(context["goal"]["title"], "Review completion criteria");
    for field in ["record", "questions", "drafts", "outbox"] {
        assert!(context.get(field).is_none(), "{context}");
    }
    assert!(!String::from_utf8_lossy(&output.stdout).contains("PRIVATE"));
    assert!(output.stdout.len() <= 65_536);
}

#[test]
fn tracking_and_criteria_commands_share_the_ui_store_across_project_directories() {
    let home = Home::new();
    let agent_cwd = home.0.join("agent-project");
    std::fs::create_dir_all(&agent_cwd).unwrap();
    home.session("goal", agent_cwd.to_str().unwrap(), false);
    let relative_root = PathBuf::from(format!(
        ".{}-state",
        home.0.file_name().unwrap().to_str().unwrap()
    ));
    let state_home = Home(std::env::current_dir().unwrap().join(&relative_root));
    let config = FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: relative_root,
        socket: home.0.join("no-runtime.sock"),
        ..FleetConfig::default()
    };
    for action in ['T', 'A'] {
        let mut ui = FleetConsole::new(config.clone());
        wait_text(&mut ui, "No reported work");
        key(&mut ui, KeyCode::Char(action));
        ui.try_shutdown().unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
            .current_dir(&agent_cwd)
            .args(["state", "--root"])
            .arg(&state_home.0)
            .args(["show", "codex", "goal"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let saved: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let draft = saved["record"]["drafts"][0]["text"].as_str().unwrap();
        let read_command = draft
            .lines()
            .find(|line| line.contains(" state --root ") && line.contains(" context "))
            .unwrap();
        assert!(
            read_command.contains(&format!("--root '{}'", state_home.0.display())),
            "{read_command}"
        );
        assert!(
            draft
                .lines()
                .find(|line| line.contains(" apply "))
                .unwrap()
                .contains(&format!("--root '{}'", state_home.0.display())),
            "{draft}"
        );
    }
}

#[test]
fn tracking_is_an_editable_coordinator_draft_and_keeps_existing_feedback() {
    use zoetrope::console::AgentKey;
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser goal");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"root-goal","thread_source":"subagent","parent_thread_id":"root-goal","agent_nickname":"Parser"}}
"#).unwrap();
    let store = Store::new(home.0.join("state"));
    for (agent, draft) in [
        ("main", "Keep current acceptance criteria"),
        ("child-thread", "Keep the child draft"),
    ] {
        store
            .apply(
                &home.key("root-goal"),
                WorkUpdate::SaveDraft {
                    target: AgentKey {
                        session: home.key("root-goal"),
                        agent_id: agent.into(),
                    },
                    text: draft.into(),
                },
            )
            .unwrap();
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Parser goal");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    key(&mut ui, KeyCode::Char('T'));
    let draft = wait_text(&mut ui, "upsert_work_item");
    assert!(draft.contains("Recipient: Coordinator"), "{draft}");
    assert!(
        draft.contains("Guidance unavailable: Codex disconnected"),
        "{draft}"
    );
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    ui.shutdown();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["state", "--root"])
        .arg(home.0.join("state"))
        .args(["show", "codex", "root-goal"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let record: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let drafts = record["record"]["drafts"].as_array().unwrap();
    let coordinator = drafts
        .iter()
        .find(|d| d["target"]["agent_id"] == "main")
        .unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(
        coordinator.starts_with("Keep current acceptance criteria"),
        "{coordinator}"
    );
    assert!(
        coordinator.contains("context codex 'root-goal'"),
        "{coordinator}"
    );
    assert!(coordinator.contains("upsert_work_item"), "{coordinator}");
    assert!(
        coordinator.contains(&home.0.join("state").to_string_lossy().to_string()),
        "{coordinator}"
    );
    assert!(
        drafts
            .iter()
            .any(|d| d["target"]["agent_id"] == "child-thread"
                && d["text"] == "Keep the child draft")
    );
    assert!(record["record"]["outbox"].as_array().unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn visible_steering_actions_and_control_shortcuts_keep_text_literal() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let peer = Peer::new("control");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser goal");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"root-goal","thread_source":"subagent","parent_thread_id":"root-goal","agent_nickname":"Parser"}}
"#).unwrap();
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Parser goal");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    let rendered = wait_text(&mut ui, "[Steer f]");
    let (x, y) = rendered
        .lines()
        .enumerate()
        .find_map(|(y, line)| {
            line.find("[Steer f]")
                .map(|x| (unicode_width::UnicodeWidthStr::width(&line[..x]), y))
        })
        .unwrap();
    ui.handle_event(&Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x as u16,
        row: y as u16,
        modifiers: KeyModifiers::NONE,
    }));
    type_text(&mut ui, "f T q stays text");
    wait_text(&mut ui, "f T q stays text");
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "Keep the public interface");
    wait_text(&mut ui, "Guidance available: current turn");
    ctrl(&mut ui, 's');
    wait_text(&mut ui, "Accepted by Codex");
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "Do docs later");
    let rendered = text(&mut ui, 150, 42);
    let (x, y) = rendered
        .lines()
        .enumerate()
        .find_map(|(y, line)| {
            line.find("[Queue Ctrl-Q]")
                .map(|x| (unicode_width::UnicodeWidthStr::width(&line[..x]), y))
        })
        .expect(&rendered);
    ui.handle_event(&Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x as u16,
        row: y as u16,
        modifiers: KeyModifiers::NONE,
    }));
    wait_text(&mut ui, "Queued by Codex");
    ctrl(&mut ui, 'x');
    wait_text(&mut ui, "accepted the interruption");
    ui.shutdown();
    peer.finish();
}

#[cfg(unix)]
#[test]
fn restricted_child_explains_input_before_sending_and_keeps_stop_available() {
    let peer = Peer::new("restricted");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser goal");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"root-goal","thread_source":"subagent","parent_thread_id":"root-goal","agent_nickname":"Parser"}}
"#).unwrap();
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Parser goal");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    let restricted = wait_text(
        &mut ui,
        "Guidance unavailable: agent does not accept direct input",
    );
    assert!(
        restricted.contains("Queue unavailable: agent does not accept direct input"),
        "{restricted}"
    );
    assert!(
        restricted.contains("Stop available: current turn"),
        "{restricted}"
    );
    key(&mut ui, KeyCode::F(4));
    wait_text(&mut ui, "accepted the interruption");
    ui.shutdown();
    peer.finish();
}

#[test]
fn an_expired_question_keeps_its_answer_draft_and_can_be_copied_to_feedback() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionPrompt, QuestionRecord, QuestionState};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Compiler questions");
    let target = AgentKey {
        session: home.key("goal"),
        agent_id: "main".into(),
    };
    Store::new(home.0.join("state"))
        .apply(
            &target.session,
            WorkUpdate::CaptureQuestion {
                question: QuestionRecord {
                    id: "earlier".into(),
                    target: target.clone(),
                    request_id: None,
                    turn_id: None,
                    prompts: vec![QuestionPrompt {
                        id: "choice".into(),
                        header: "Color".into(),
                        question: "Which color should the output use?".into(),
                        options: vec![],
                        is_other: true,
                        is_secret: false,
                    }],
                    state: QuestionState::Expired,
                    answers: Default::default(),
                },
            },
        )
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Compiler questions");
    key(&mut ui, KeyCode::Char('Q'));
    wait_text(&mut ui, "Original request expired");
    key(&mut ui, KeyCode::Char('e'));
    type_text(&mut ui, "Use blue with readable contrast");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Answer draft saved");
    ui.shutdown();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Compiler questions");
    key(&mut ui, KeyCode::Char('Q'));
    wait_text(&mut ui, "Use blue with readable contrast");
    key(&mut ui, KeyCode::F(5));
    let copied = wait_text(&mut ui, "Answer to an earlier question");
    assert!(copied.contains("F2 Send guidance"));
    assert!(!copied.contains("Accepted by Codex"));
    ui.shutdown();
}
#[cfg(unix)]
#[test]
fn native_questions_require_a_fresh_join_and_explicit_choice_before_submission() {
    let peer = Peer::new("question");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser questions");
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Parser questions");
    key(&mut ui, KeyCode::Char('Q'));
    key(&mut ui, KeyCode::Char('L'));
    let question = wait_text(&mut ui, "Choose a label");
    assert!(question.contains("No answer chosen"), "{question}");
    key(&mut ui, KeyCode::Char('2'));
    wait_text(&mut ui, "Answer draft: Beta");
    key(&mut ui, KeyCode::F(2));
    let resolved = wait_text(&mut ui, "Resolved in Codex");
    assert!(resolved.contains("Answer draft: Beta"));
    assert!(resolved.contains("0 of 2 verified") || !resolved.contains("100%"));
    ui.shutdown();
    peer.finish();
}
#[cfg(unix)]
#[test]
fn quiet_waiting_runtime_work_is_current_without_resuming_history() {
    let peer = Peer::new("observe");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Quiet compiler");
    let path = home
        .0
        .join("codex/sessions/2026/10/23/rollout-root-goal.jsonl");
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(100)))
        .unwrap();
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    let shown = wait_text(&mut ui, "Selected: Quiet compiler");
    assert!(shown.contains("Current work"));
    wait_text(&mut ui, "Waiting for your answer");
    ui.shutdown();
    peer.finish();
}
#[cfg(unix)]
#[test]
fn documents_open_only_the_explicitly_selected_exact_path() {
    use std::os::unix::fs::PermissionsExt;
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{DocumentChange, DocumentRecord};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Compiler documents");
    let producer = AgentKey {
        session: home.key("goal"),
        agent_id: "main".into(),
    };
    let first = home.0.join("first.md");
    let selected = home.0.join("Review ü #+%2F &.md");
    std::fs::write(&first, "first").unwrap();
    std::fs::write(&selected, "selected").unwrap();
    let store = Store::new(home.0.join("state"));
    for (at, path) in [&first, &selected].iter().enumerate() {
        store
            .apply(
                &producer.session,
                WorkUpdate::AddDocument {
                    document: DocumentRecord {
                        id: format!("doc-{at}"),
                        path: (*path).clone(),
                        producer: producer.clone(),
                        source: "recorded file change".into(),
                        change: DocumentChange::Created,
                        changed_at: None,
                    },
                },
            )
            .unwrap();
    }
    let code = home.0.join("code.py");
    let opener = home.0.join("open.py");
    let trace = home.0.join("opened.txt");
    std::fs::write(
        &code,
        "#!/usr/bin/env python3\nprint('wagar.zoetrope-preview')\n",
    )
    .unwrap();
    std::fs::write(&opener,format!("#!/usr/bin/env python3\nimport sys,pathlib,urllib.parse\nu=urllib.parse.urlsplit(sys.argv[1])\nf=urllib.parse.parse_qs(u.query)['uri'][0]\npathlib.Path({:?}).write_text(urllib.parse.unquote(urllib.parse.urlsplit(f).path))\n",trace.to_string_lossy())).unwrap();
    for path in [&code, &opener] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: home.0.join("none.sock"),
        documents: zoetrope::documents::DocumentLauncher::new(code, opener),
        state_executable: None,
    });
    wait_text(&mut ui, "Selected: Compiler documents");
    key(&mut ui, KeyCode::Char('d'));
    let listed = wait_text(&mut ui, "Review ü #+%2F &.md");
    assert!(listed.contains("Coordinator"));
    assert!(
        !trace.exists(),
        "merely listing artifacts must not open an editor"
    );
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Enter);
    wait_text(&mut ui, "Requested VS Code preview");
    assert_eq!(
        std::fs::read_to_string(trace).unwrap(),
        selected.to_string_lossy()
    );
    ui.shutdown();
}
#[test]
fn recorded_async_questions_are_retained_without_inventing_native_pending_ids() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Async questions");
    let path = home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl");
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"request_user_input_async","call_id":"async-call","arguments":serde_json::json!({"questions":[{"title":"Which implementation should we use?","options":["Use Rust","Use TypeScript"]}]}).to_string()}})).unwrap();
    writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"functions.exec","call_id":"code-call","arguments":serde_json::json!({"code":"request_user_input_async({title:'Fake question'})"}).to_string()}})).unwrap();
    drop(file);
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Async questions");
    key(&mut ui, KeyCode::Char('Q'));
    let question = wait_text(&mut ui, "Which implementation should we use?");
    assert!(question.contains("Original request expired"));
    assert!(!question.contains("Fake question"));
    key(&mut ui, KeyCode::Char('2'));
    wait_text(&mut ui, "Answer draft: Use TypeScript");
    ui.shutdown();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Async questions");
    key(&mut ui, KeyCode::Char('Q'));
    let restored = wait_text(&mut ui, "Answer draft: Use TypeScript");
    assert!(restored.contains("Questions · 1 of 1"));
    ui.shutdown();
}
#[test]
fn public_commentary_and_shell_purpose_are_readable_without_private_reasoning() {
    let home = Home::new();
    home.session("raw-thread-id", "/projects/compiler", false);
    let path = home
        .0
        .join("codex/sessions/2026/10/23/rollout-raw-thread-id.jsonl");
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    for record in [
        serde_json::json!({"type":"event_msg","payload":{"type":"item_completed","item":{"type":"UserMessage","content":[{"type":"text","text":"Fix compiler diagnostics"}]}}}),
        serde_json::json!({"type":"response_item","payload":{"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":"Checking compiler diagnostics"}]}}),
        serde_json::json!({"type":"response_item","payload":{"type":"message","role":"assistant","channel":"analysis","content":[{"type":"output_text","text":"PRIVATE ANALYSIS PLACEHOLDER"}]}}),
        serde_json::json!({"type":"event_msg","payload":{"type":"item_completed","item":{"type":"Reasoning","summary_text":["PRIVATE SUMMARY PLACEHOLDER"]}}}),
        serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"functions.exec_command","call_id":"bash","arguments":serde_json::json!({"cmd":"cargo test --locked","justification":"Check parser behavior"}).to_string()}}),
        serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"read_file","call_id":"read","arguments":"{\"path\":\"src/main.rs\"}"}}),
    ] {
        writeln!(file, "{record}").unwrap();
    }
    drop(file);
    let mut ui = home.ui();
    let shown = wait_text(&mut ui, "Selected: Fix compiler diagnostics");
    assert!(shown.contains("Checking compiler diagnostics"), "{shown}");
    assert!(shown.contains("cargo test --locked"), "{shown}");
    assert!(shown.contains("Check parser behavior"), "{shown}");
    assert!(!shown.contains("PRIVATE"));
    assert!(!shown.contains("functions.exec_command"));
    assert!(!shown.contains("read_file"));
    assert!(!shown.contains("raw-thread-id"));
    ui.shutdown();
}
#[cfg(unix)]
#[test]
fn transcript_and_live_question_share_one_draft_and_resolved_answers_can_be_feedback() {
    let peer = Peer::new("question");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "One question source");
    let path = home
        .0
        .join("codex/sessions/2026/10/23/rollout-root-goal.jsonl");
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"request_user_input","call_id":"native-question-call","arguments":serde_json::json!({"questions":[{"id":"choice","header":"Label","question":"Choose a label","options":[{"label":"Alpha","description":"First choice"},{"label":"Beta","description":"Second choice"}]}]}).to_string()}})).unwrap();
    drop(file);
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: One question source");
    key(&mut ui, KeyCode::Char('Q'));
    wait_text(&mut ui, "Original request expired");
    key(&mut ui, KeyCode::Char('2'));
    wait_text(&mut ui, "Answer draft: Beta");
    key(&mut ui, KeyCode::Char('L'));
    let joined = wait_text(&mut ui, "Pending in the current Codex connection");
    assert!(joined.contains("Questions · 1 of 1"), "{joined}");
    assert!(joined.contains("Answer draft: Beta"), "{joined}");
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Resolved in Codex");
    key(&mut ui, KeyCode::F(5));
    wait_text(&mut ui, "Answer to an earlier question");
    ui.shutdown();
    peer.finish();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call_output","call_id":"native-question-call","output":serde_json::json!({"answers":{"choice":{"answers":["Beta"]}}}).to_string()}})).unwrap();
    drop(file);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(100)))
        .unwrap();
    let mut history = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("fresh-state"),
        socket: home.0.join("none.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut history, "No current work");
    key(&mut history, KeyCode::Char('H'));
    wait_text(&mut history, "Selected: Untitled goal");
    key(&mut history, KeyCode::Char('Q'));
    wait_text(&mut history, "Resolved in Codex");
    history.shutdown();
}
#[test]
fn plan_adoption_and_criteria_requests_stay_explicit_and_use_the_same_goal_record() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{PlanProposal, PlanStep};
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    let child = home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl");
    std::fs::write(child,"{\"type\":\"session_meta\",\"payload\":{\"id\":\"child-thread\",\"session_id\":\"root-goal\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root-goal\",\"agent_nickname\":\"Parser\"}}\n").unwrap();
    let state = home.0.join("state user's");
    let store = Store::new(state.clone());
    let producer = AgentKey {
        session: home.key("root-goal"),
        agent_id: "main".into(),
    };
    store
        .apply(
            &producer.session,
            WorkUpdate::SetGoal {
                title: "Compiler plan".into(),
                criteria: vec![CriterionInput {
                    id: "initial".into(),
                    label: "Initial outcome".into(),
                    weight: 1.0,
                }],
                reason: "scope".into(),
                provenance: "operator".into(),
            },
        )
        .unwrap();
    for (id, step) in [
        ("first", "First proposed outcome"),
        ("second", "Second proposed outcome"),
    ] {
        store
            .apply(
                &producer.session,
                WorkUpdate::ImportPlan {
                    proposal: PlanProposal {
                        id: id.into(),
                        producer: AgentKey {
                            session: producer.session.clone(),
                            agent_id: if id == "first" {
                                "main".into()
                            } else {
                                "child-thread".into()
                            },
                        },
                        steps: vec![PlanStep {
                            step: step.into(),
                            status: "completed".into(),
                        }],
                        source: "recorded plan".into(),
                        recorded_at: None,
                    },
                },
            )
            .unwrap();
    }
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: state.clone(),
        socket: home.0.join("none.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Compiler plan");
    key(&mut ui, KeyCode::Char('c'));
    wait_text(&mut ui, "Draft plan (unverified) 1 of 2");
    key(&mut ui, KeyCode::Char(']'));
    wait_text(&mut ui, "Draft plan (unverified) 2 of 2");
    wait_text(&mut ui, "Proposed by Parser");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Plan adopted as unverified outcomes");
    let adopted = text(&mut ui, 150, 42);
    assert!(adopted.contains("Second proposed outcome"), "{adopted}");
    assert!(adopted.contains("0 of 1 verified"), "{adopted}");
    key(&mut ui, KeyCode::Char('p'));
    wait_text(&mut ui, "Checkpoint preference saved");
    key(&mut ui, KeyCode::Esc);
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Coordinator");
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    key(&mut ui, KeyCode::Char('A'));
    wait_text(&mut ui, "Ask me before meaningful milestones");
    let request = text(&mut ui, 800, 60);
    assert!(request.contains("context codex 'root-goal'"), "{request}");
    assert!(!request.contains("context codex 'child-thread'"));
    assert!(request.contains("state user'\\''s'"), "{request}");
    assert!(request.contains("Recipient: Parser"));
    assert!(!request.contains("Accepted by Codex"));
    ui.shutdown();
}

#[test]
fn progress_is_clickable_and_narrow_terminals_keep_help_and_pasted_drafts_usable() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Narrow goal");
    Store::new(home.0.join("state"))
        .apply(
            &home.key("goal"),
            WorkUpdate::Verify {
                id: "build".into(),
                evidence: vec![Evidence {
                    kind: EvidenceKind::UserAttestation,
                    detail: "Build passed".into(),
                    provenance: "operator".into(),
                }],
                revision: Some(1),
            },
        )
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Narrow goal");
    let rendered = text(&mut ui, 150, 42);
    let (x, y) = rendered
        .lines()
        .enumerate()
        .filter_map(|(y, line)| {
            line.find("1 of 2 verified")
                .filter(|x| *x > 80)
                .map(|x| (x, y))
        })
        .next()
        .unwrap();
    ui.handle_event(&Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x as u16,
        row: y as u16,
        modifiers: KeyModifiers::NONE,
    }));
    wait_text(&mut ui, "Build passes");
    key(&mut ui, KeyCode::Esc);
    key(&mut ui, KeyCode::Char('?'));
    let help = text(&mut ui, 60, 28);
    assert!(help.contains("Keyboard help"), "{help}");
    assert!(help.contains("F2 Send guidance"), "{help}");
    assert!(help.contains("Q saved questions"), "{help}");
    key(&mut ui, KeyCode::Esc);
    key(&mut ui, KeyCode::Char('f'));
    ui.handle_event(&Event::Paste("H q stays text\n🙂 tail".into()));
    let draft = text(&mut ui, 60, 28);
    assert!(draft.contains("H q stays text"), "{draft}");
    assert!(draft.contains("tail"), "{draft}");
    assert!(draft.contains("F2 Send guidance"), "{draft}");
    assert!(draft.contains("Current work"), "{draft}");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    ui.shutdown();
}
#[cfg(unix)]
#[test]
fn pending_deliveries_are_readable_and_acknowledgment_preserves_a_newer_draft() {
    let peer = Peer::new("pending");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Delivery records");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"),"{\"type\":\"session_meta\",\"payload\":{\"id\":\"child-thread\",\"session_id\":\"root-goal\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root-goal\",\"agent_nickname\":\"Parser\"}}\n").unwrap();
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Delivery records");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Parser");
    wait_text(&mut ui, "Working");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Keep the public interface");
    key(&mut ui, KeyCode::F(2));
    let until = Instant::now() + Duration::from_secs(3);
    while !peer.dir.join("peer.sock.sending").exists() {
        assert!(Instant::now() < until);
        ui.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "New instruction draft");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    key(&mut ui, KeyCode::Char('b'));
    let pending = wait_text(&mut ui, "Guidance to Parser · Sending");
    assert!(!pending.contains("Delivery unknown"), "{pending}");
    std::fs::write(peer.dir.join("peer.sock.release"), "").unwrap();
    wait_text(&mut ui, "Accepted by Codex");
    key(&mut ui, KeyCode::Char('f'));
    wait_text(&mut ui, "New instruction draft");
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "Keep the public interface");
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Accepted by Codex");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    key(&mut ui, KeyCode::Char('b'));
    let delivered = wait_text(&mut ui, "Saved deliveries");
    assert!(!delivered.contains("turn/steer"));
    ui.shutdown();
    peer.finish();
    let mut restored = home.ui();
    wait_text(&mut restored, "Selected: Delivery records");
    key(&mut restored, KeyCode::Char('b'));
    wait_text(&mut restored, "Guidance to Parser · Accepted by Codex");
    restored.shutdown();
}

#[cfg(unix)]
#[test]
fn background_observation_reaches_later_agents_without_resuming_them() {
    let peer = Peer::new("many");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", true);
    home.goal("root-goal", "Quiet later work");
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Quiet later work");
    wait_text(&mut ui, "Waiting for your answer");
    let current = text(&mut ui, 150, 42);
    assert!(current.contains("Current work"), "{current}");
    ui.shutdown();
    peer.finish();
}

#[test]
fn selected_goal_refreshes_external_progress_and_details_without_navigation() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{DocumentChange, DocumentRecord};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "External updates");
    let mut ui = home.ui();
    wait_text(&mut ui, "0 of 2 verified");
    let store = Store::new(home.0.join("state"));
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::Verify {
                id: "build".into(),
                revision: Some(1),
                evidence: vec![Evidence {
                    kind: EvidenceKind::UserAttestation,
                    detail: "External build passed".into(),
                    provenance: "CLI".into(),
                }],
            },
        )
        .unwrap();
    wait_text(&mut ui, "1 of 2 verified");
    key(&mut ui, KeyCode::Char('d'));
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::AddDocument {
                document: DocumentRecord {
                    id: "external".into(),
                    path: home.0.join("external-review.md"),
                    producer: AgentKey {
                        session: home.key("goal"),
                        agent_id: "main".into(),
                    },
                    source: "CLI".into(),
                    change: DocumentChange::Created,
                    changed_at: None,
                },
            },
        )
        .unwrap();
    wait_text(&mut ui, "external-review.md");
    ui.shutdown();
}

#[test]
fn secret_answers_stay_hidden_during_editing_and_cannot_be_exposed_as_feedback() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionPrompt, QuestionRecord, QuestionState};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Private answer");
    let store = Store::new(home.0.join("state"));
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::CaptureQuestion {
                question: QuestionRecord {
                    id: "secret".into(),
                    target: AgentKey {
                        session: home.key("goal"),
                        agent_id: "main".into(),
                    },
                    request_id: None,
                    turn_id: None,
                    state: QuestionState::Expired,
                    prompts: vec![QuestionPrompt {
                        id: "token".into(),
                        header: "Token".into(),
                        question: "Provide token".into(),
                        options: vec![],
                        is_other: true,
                        is_secret: true,
                    }],
                    answers: [("token".into(), vec!["PRIVATE_SENTINEL".into()])].into(),
                },
            },
        )
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Private answer");
    key(&mut ui, KeyCode::Char('Q'));
    wait_text(&mut ui, "Saved (hidden)");
    key(&mut ui, KeyCode::Char('e'));
    assert!(!text(&mut ui, 150, 42).contains("PRIVATE_SENTINEL"));
    ui.handle_event(&Event::Paste("MORE_PRIVATE".into()));
    assert!(!text(&mut ui, 150, 42).contains("MORE_PRIVATE"));
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Answer draft saved");
    key(&mut ui, KeyCode::F(5));
    let shown = wait_text(&mut ui, "Secret answers cannot be copied");
    assert!(!shown.contains("PRIVATE_SENTINEL"));
    assert!(!shown.contains("MORE_PRIVATE"));
    ui.shutdown();
    let record = store.read(&home.key("goal")).unwrap();
    assert!(record.outbox.is_empty());
    assert!(record.drafts.is_empty());
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Private answer");
    key(&mut ui, KeyCode::Char('Q'));
    key(&mut ui, KeyCode::Char('e'));
    assert!(!text(&mut ui, 150, 42).contains("PRIVATE_SENTINEL"));
    ui.shutdown();
}

#[test]
fn oversized_paste_is_rejected_whole_and_preserves_the_saved_utf8_draft() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Bounded feedback");
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Bounded feedback");
    key(&mut ui, KeyCode::Char('f'));
    ui.handle_event(&Event::Paste("Keep this constraint".into()));
    ui.handle_event(&Event::Paste(format!(
        "{}TAIL_CONSTRAINT",
        "é".repeat(32768)
    )));
    let shown = text(&mut ui, 150, 42);
    assert!(shown.contains("64 KiB"), "{shown}");
    assert!(shown.contains("Keep this constraint"));
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    ui.shutdown();
    let record = Store::new(home.0.join("state"))
        .read(&home.key("goal"))
        .unwrap();
    assert_eq!(record.drafts[0].text, "Keep this constraint");
}

#[test]
fn explicit_codex_public_phases_are_readable_but_unknown_and_analysis_are_hidden() {
    use std::io::Write;
    for phase in ["commentary", "final_answer"] {
        let home = Home::new();
        home.session("goal", "/projects/compiler", false);
        home.goal("goal", "Public phases");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl"))
            .unwrap();
        for (classification, message) in [
            (Some(phase), "Readable public result"),
            (Some("analysis"), "PRIVATE_PHASE"),
            (None, "PRIVATE_UNKNOWN"),
        ] {
            writeln!(
                file,
                "{}",
                serde_json::json!({"type":"response_item","payload":{
                "type":"message","role":"assistant","phase":classification,
                "content":[{"type":"output_text","text":message}]}})
            )
            .unwrap();
        }
        drop(file);
        let mut ui = home.ui();
        let shown = wait_text(&mut ui, "Readable public result");
        assert!(!shown.contains("PRIVATE"));
        ui.shutdown();
    }
}

#[cfg(unix)]
struct HeldLock(std::process::Child);
#[cfg(unix)]
impl HeldLock {
    fn acquire(path: &std::path::Path) -> Self {
        use std::io::BufRead;
        let mut child = std::process::Command::new("python3")
            .args(["-c", "import fcntl,sys; f=open(sys.argv[1], 'a'); fcntl.flock(f, fcntl.LOCK_EX); print('locked', flush=True); sys.stdin.read()"])
            .arg(path).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
        let mut ready = String::new();
        std::io::BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut ready)
            .unwrap();
        assert_eq!(ready.trim(), "locked");
        Self(child)
    }
    fn release(mut self) {
        drop(self.0.stdin.take());
        assert!(self.0.wait().unwrap().success());
    }
}
#[cfg(unix)]
impl Drop for HeldLock {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(unix)]
#[test]
fn feedback_stays_in_the_composer_when_the_save_queue_is_full_then_saves_on_exit() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.enable_unrelated_tracking();
    home.goal("goal", "Busy save");
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Busy save");
    key(&mut ui, KeyCode::Char('c'));
    wait_text(&mut ui, "Build passes");
    let lock = home.hold_goal("goal");
    key(&mut ui, KeyCode::Char('c'));
    for _ in 0..48 {
        key(&mut ui, KeyCode::Char('p'));
    }
    key(&mut ui, KeyCode::Char('f'));
    ui.handle_event(&Event::Paste("KEEP_UNTIL_DURABLE".into()));
    key(&mut ui, KeyCode::Esc);
    let shown = text(&mut ui, 150, 42);
    assert!(shown.contains("KEEP_UNTIL_DURABLE"), "{shown}");
    assert!(shown.contains("queue"), "{shown}");
    lock.release();
    // An orderly exit must flush even the save that could not be enqueued.
    ui.shutdown();
    let record = Store::new(home.0.join("state"))
        .read(&home.key("goal"))
        .unwrap();
    assert_eq!(record.drafts[0].text, "KEEP_UNTIL_DURABLE");
}

#[test]
fn failed_storage_keeps_feedback_and_answer_editors_available_until_a_durable_retry() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionPrompt, QuestionRecord, QuestionState};
    for answer in [false, true] {
        let home = Home::new();
        home.session("goal", "/projects/compiler", false);
        home.goal("goal", "Storage retry");
        let store = Store::new(home.0.join("state"));
        store
            .apply(
                &home.key("goal"),
                WorkUpdate::CaptureQuestion {
                    question: QuestionRecord {
                        id: "earlier".into(),
                        target: AgentKey {
                            session: home.key("goal"),
                            agent_id: "main".into(),
                        },
                        request_id: None,
                        turn_id: None,
                        state: QuestionState::Expired,
                        prompts: vec![QuestionPrompt {
                            id: "choice".into(),
                            header: "Choice".into(),
                            question: "Which approach?".into(),
                            options: vec![],
                            is_other: true,
                            is_secret: false,
                        }],
                        answers: Default::default(),
                    },
                },
            )
            .unwrap();
        let mut ui = home.ui();
        wait_text(&mut ui, "Selected: Storage retry");
        if answer {
            key(&mut ui, KeyCode::Char('Q'));
            key(&mut ui, KeyCode::Char('e'));
        } else {
            key(&mut ui, KeyCode::Char('f'));
        }
        ui.handle_event(&Event::Paste("RETAIN_ON_WRITE_ERROR".into()));
        // Real filesystem failure, outside the UI: the configured directory is
        // temporarily replaced by a regular file, then repaired for retry.
        std::fs::rename(home.0.join("state"), home.0.join("state-backup")).unwrap();
        std::fs::write(home.0.join("state"), "temporarily unavailable").unwrap();
        key(&mut ui, if answer { KeyCode::F(2) } else { KeyCode::Esc });
        let shown = wait_text(&mut ui, "Could not save draft");
        assert!(shown.contains("RETAIN_ON_WRITE_ERROR"), "{shown}");
        assert!(!shown.contains("Draft saved"));
        assert!(
            ui.try_shutdown().is_err(),
            "failed shutdown must retain editable data"
        );
        assert!(text(&mut ui, 150, 42).contains("RETAIN_ON_WRITE_ERROR"));
        std::fs::remove_file(home.0.join("state")).unwrap();
        std::fs::rename(home.0.join("state-backup"), home.0.join("state")).unwrap();
        key(&mut ui, KeyCode::Esc);
        wait_text(
            &mut ui,
            if answer {
                "Answer draft saved"
            } else {
                "Draft saved"
            },
        );
        ui.try_shutdown().unwrap();
        let record = store.read(&home.key("goal")).unwrap();
        if answer {
            assert_eq!(
                record.questions[0].answers["choice"],
                ["RETAIN_ON_WRITE_ERROR"]
            );
        } else {
            assert_eq!(record.drafts[0].text, "RETAIN_ON_WRITE_ERROR");
        }
    }
}

#[cfg(unix)]
#[test]
fn an_answer_stays_editable_under_save_backpressure_and_flushes_on_orderly_shutdown() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionOption, QuestionPrompt, QuestionRecord, QuestionState};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.enable_unrelated_tracking();
    home.goal("goal", "Busy answer save");
    let store = Store::new(home.0.join("state"));
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::CaptureQuestion {
                question: QuestionRecord {
                    id: "earlier".into(),
                    target: AgentKey {
                        session: home.key("goal"),
                        agent_id: "main".into(),
                    },
                    request_id: None,
                    turn_id: None,
                    state: QuestionState::Expired,
                    prompts: vec![QuestionPrompt {
                        id: "choice".into(),
                        header: "Choice".into(),
                        question: "Which approach?".into(),
                        options: vec![QuestionOption {
                            label: "Rust".into(),
                            description: "Fast".into(),
                        }],
                        is_other: true,
                        is_secret: false,
                    }],
                    answers: Default::default(),
                },
            },
        )
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Busy answer save");
    key(&mut ui, KeyCode::Char('Q'));
    let lock = home.hold_goal("goal");
    for _ in 0..48 {
        key(&mut ui, KeyCode::Char('1'));
    }
    key(&mut ui, KeyCode::Char('e'));
    ui.handle_event(&Event::Paste("FINAL_ANSWER_CONSTRAINT".into()));
    key(&mut ui, KeyCode::F(2));
    let shown = text(&mut ui, 150, 42);
    assert!(shown.contains("FINAL_ANSWER_CONSTRAINT"), "{shown}");
    assert!(shown.contains("queue"), "{shown}");
    lock.release();
    ui.try_shutdown().unwrap();
    assert_eq!(
        store.read(&home.key("goal")).unwrap().questions[0].answers["choice"],
        ["FINAL_ANSWER_CONSTRAINT"]
    );
}

#[cfg(unix)]
#[test]
fn repeating_the_local_copy_key_after_completion_never_submits_to_codex() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionPrompt, QuestionRecord, QuestionState};
    let peer = Peer::new("observe");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Safe local copy");
    Store::new(home.0.join("state"))
        .apply(
            &home.key("root-goal"),
            WorkUpdate::CaptureQuestion {
                question: QuestionRecord {
                    id: "earlier".into(),
                    target: AgentKey {
                        session: home.key("root-goal"),
                        agent_id: "main".into(),
                    },
                    request_id: None,
                    turn_id: None,
                    state: QuestionState::Expired,
                    prompts: vec![QuestionPrompt {
                        id: "choice".into(),
                        header: "Choice".into(),
                        question: "Which approach?".into(),
                        options: vec![],
                        is_other: true,
                        is_secret: false,
                    }],
                    answers: [("choice".into(), vec!["LOCAL_COPY_ONLY".into()])].into(),
                },
            },
        )
        .unwrap();
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Safe local copy");
    key(&mut ui, KeyCode::Char('Q'));
    key(&mut ui, KeyCode::F(5));
    wait_text(&mut ui, "Answer to an earlier question");
    // Conventional terminals report held keys as ordinary Press events.
    for _ in 0..12 {
        key(&mut ui, KeyCode::F(5));
        ui.poll();
    }
    let until = Instant::now() + Duration::from_millis(400);
    while Instant::now() < until {
        ui.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(text(&mut ui, 150, 42).contains("LOCAL_COPY_ONLY"));
    ui.shutdown();
    peer.finish(); // observe peer rejects every native mutation
}

#[test]
#[ignore = "manual public large-history resource probe; no timing threshold"]
fn large_fleet_placement_resource_probe() {
    for count in [1000, 4000] {
        let home = Home::new();
        for at in 0..count {
            home.session(&format!("history-{at:05}"), "/projects/history", true);
        }
        let mut ui = home.ui();
        key(&mut ui, KeyCode::Char('H'));
        let started = Instant::now();
        let mut peak_poll = Duration::ZERO;
        loop {
            let poll_at = Instant::now();
            ui.poll();
            peak_poll = peak_poll.max(poll_at.elapsed());
            if text(&mut ui, 150, 42).contains("Selected: Untitled goal") {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(60),
                "history did not load"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let first = text(&mut ui, 150, 42);
        // A no-op refresh retains identity/positions and the rendered view.
        let until = Instant::now() + Duration::from_millis(100);
        while Instant::now() < until {
            ui.poll();
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(first, text(&mut ui, 150, 42));
        eprintln!(
            "history={count} startup_ms={} peak_ui_poll_us={}",
            started.elapsed().as_millis(),
            peak_poll.as_micros()
        );
        ui.shutdown();
    }
}

#[test]
fn criteria_edits_also_remain_available_when_storage_rejects_a_save() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Original scope");
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Original scope");
    key(&mut ui, KeyCode::Char('c'));
    key(&mut ui, KeyCode::Char('e'));
    ctrl(&mut ui, 'u');
    ui.handle_event(&Event::Paste(
        "RETAINED_SCOPE\nMeaningful behavior checked".into(),
    ));
    std::fs::rename(home.0.join("state"), home.0.join("state-backup")).unwrap();
    std::fs::write(home.0.join("state"), "temporarily unavailable").unwrap();
    key(&mut ui, KeyCode::F(2));
    let shown = wait_text(&mut ui, "Could not save");
    assert!(shown.contains("RETAINED_SCOPE"), "{shown}");
    std::fs::remove_file(home.0.join("state")).unwrap();
    std::fs::rename(home.0.join("state-backup"), home.0.join("state")).unwrap();
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Selected: RETAINED_SCOPE");
    ui.shutdown();
}

#[test]
fn an_orphan_agent_does_not_remove_the_connections_for_other_visible_work() {
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.goal("goal", "Connected work");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-orphan.jsonl"),
        serde_json::json!({"type":"session_meta","payload":{"id":"orphan","session_id":"goal",
            "thread_source":"subagent","parent_thread_id":"missing-parent","agent_nickname":"Unlinked helper"}}).to_string() + "\n").unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Connected work");
    key(&mut ui, KeyCode::Char('a'));
    wait_text(&mut ui, "Agents expanded");
    let shown = text(&mut ui, 150, 50);
    assert_eq!(
        shown.chars().filter(|ch| *ch == '▶').count(),
        2,
        "Both project/goal and goal/coordinator edges must remain visible: {shown}"
    );
    // The unlinked agent stays searchable/visible with its real ancestry; it
    // must not be silently adopted by the coordinator.
    key(&mut ui, KeyCode::Char('/'));
    type_text(&mut ui, "Unlinked helper");
    key(&mut ui, KeyCode::Enter);
    wait_text(&mut ui, "Search: Unlinked helper");
    let shown = text(&mut ui, 150, 50);
    assert!(shown.contains("Unlinked helper"), "{shown}");
    ui.shutdown();
}

#[cfg(unix)]
#[test]
fn a_delayed_answer_copy_preserves_a_newer_feedback_composer_and_its_target() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionPrompt, QuestionRecord, QuestionState};
    let home = Home::new();
    home.session("goal", "/projects/compiler", false);
    home.enable_unrelated_tracking();
    home.goal("goal", "Copy acknowledgment");
    let target = AgentKey {
        session: home.key("goal"),
        agent_id: "main".into(),
    };
    let store = Store::new(home.0.join("state"));
    store
        .apply(
            &home.key("goal"),
            WorkUpdate::CaptureQuestion {
                question: QuestionRecord {
                    id: "earlier".into(),
                    target: target.clone(),
                    request_id: None,
                    turn_id: None,
                    state: QuestionState::Expired,
                    prompts: vec![QuestionPrompt {
                        id: "choice".into(),
                        header: "Choice".into(),
                        question: "Which approach?".into(),
                        options: vec![],
                        is_other: true,
                        is_secret: false,
                    }],
                    answers: [("choice".into(), vec!["OLDER_COPIED_ANSWER".into()])].into(),
                },
            },
        )
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Copy acknowledgment");
    key(&mut ui, KeyCode::Char('Q'));
    wait_text(&mut ui, "OLDER_COPIED_ANSWER");
    let lock = home.hold_goal("goal");
    key(&mut ui, KeyCode::F(5));
    key(&mut ui, KeyCode::Char('f'));
    ui.handle_event(&Event::Paste("KEEP_NEWER_DRAFT".into()));
    assert!(text(&mut ui, 150, 42).contains("KEEP_NEWER_DRAFT"));
    lock.release();
    wait_text(&mut ui, "Answer copied");
    assert!(text(&mut ui, 150, 42).contains("KEEP_NEWER_DRAFT"));
    ui.try_shutdown().unwrap();
    let record = store.read(&home.key("goal")).unwrap();
    assert!(
        record
            .drafts
            .iter()
            .any(|d| d.target == target && d.text == "KEEP_NEWER_DRAFT")
    );
    assert!(
        record
            .outbox
            .iter()
            .any(|d| d.target == target && d.text.contains("OLDER_COPIED_ANSWER"))
    );
}

#[cfg(unix)]
#[test]
fn saving_an_answer_keeps_the_editor_open_so_held_save_cannot_become_submission() {
    let peer = Peer::new("question");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Deliberate answer");
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Selected: Deliberate answer");
    key(&mut ui, KeyCode::Char('Q'));
    key(&mut ui, KeyCode::Char('L'));
    wait_text(&mut ui, "Pending in the current Codex connection");
    key(&mut ui, KeyCode::Char('e'));
    type_text(&mut ui, "Beta");
    key(&mut ui, KeyCode::F(2));
    let shown = wait_text(&mut ui, "Answer draft saved");
    assert!(shown.contains("F2 save draft"), "{shown}");
    for _ in 0..8 {
        key(&mut ui, KeyCode::F(2));
        ui.poll();
    }
    assert!(text(&mut ui, 150, 42).contains("F2 save draft"));
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Answer draft saved");
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Resolved in Codex");
    ui.shutdown();
    peer.finish();
}

#[test]
fn opening_a_goal_shows_recorded_plan_work_status_owner_and_source_time() {
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    let path = home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl");
    let plan = serde_json::json!({"timestamp":"2026-10-27T12:00:00Z","type":"response_item","payload":{
        "type":"function_call","name":"update_plan","call_id":"plan-one",
        "arguments":serde_json::json!({"plan":[
            {"step":"Inspect navigation","status":"completed"},
            {"step":"Expose steering","status":"inProgress"},
            {"step":"Check delivery","status":"pending"},
            {"step":"External review","status":"provider-new-status"}
        ]}).to_string()
    }});
    writeln!(
        std::fs::OpenOptions::new().append(true).open(path).unwrap(),
        "{plan}"
    )
    .unwrap();
    let mut ui = home.ui();
    let shown = wait_text(&mut ui, "Expose steering");
    assert!(shown.contains("Work toward goal"), "{shown}");
    assert!(shown.contains("reported done"), "{shown}");
    assert!(shown.contains("in progress"), "{shown}");
    assert!(shown.contains("planned"), "{shown}");
    assert!(shown.contains("provider-new-status"), "{shown}");
    assert!(shown.contains("Inspect navigation"), "{shown}");
    assert!(shown.contains("Expose steering"), "{shown}");
    assert!(shown.contains("Check delivery"), "{shown}");
    assert!(shown.contains("Coordinator"), "{shown}");
    assert!(shown.contains("2026-10-27T12:00:00Z"), "{shown}");
    assert!(shown.contains("0 of 2 verified"), "{shown}");
    ui.shutdown();
}

#[test]
fn withdrawing_a_native_plan_clears_work_and_expires_its_selected_entry() {
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Withdraw native work");
    let path = home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl");
    let append_plan = |call: &str, plan: serde_json::Value| {
        let record = serde_json::json!({"timestamp":"2026-10-27T13:00:00Z","type":"response_item","payload":{
            "type":"function_call","name":"update_plan","call_id":call,
            "arguments":serde_json::json!({"plan":plan}).to_string()
        }});
        writeln!(
            std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap(),
            "{record}"
        )
        .unwrap();
    };
    append_plan(
        "active",
        serde_json::json!([{"step":"Withdrawn work","status":"in_progress"}]),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Withdrawn work");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Selected work: Withdrawn work");

    append_plan("withdrawn", serde_json::json!([]));
    let cleared = wait_text(&mut ui, "No reported work");
    assert!(!cleared.contains("Withdrawn work"), "{cleared}");
    assert!(cleared.contains("Selected work unavailable"), "{cleared}");
    append_plan(
        "replacement",
        serde_json::json!([{"step":"New work","status":"in_progress"}]),
    );
    let expired = wait_text(&mut ui, "Selected work is no longer recorded");
    assert!(!expired.contains("Selected work: New work"), "{expired}");
    assert!(expired.contains("Selected work unavailable"), "{expired}");
    ui.shutdown();
}

fn save_work_report(home: &Home, session: &str, revision: u64, item: serde_json::Value) {
    let update: WorkUpdate = serde_json::from_value(serde_json::json!({
        "op":"upsert_work_item","expected_revision":revision,"item":item
    }))
    .unwrap();
    Store::new(home.0.join("state"))
        .apply(&home.key(session), update)
        .unwrap();
}

#[test]
fn explicit_work_is_default_and_explains_its_context_dependencies_and_outcomes() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    let owner =
        serde_json::json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    let native: WorkUpdate = serde_json::from_value(serde_json::json!({"op":"import_plan","proposal":{
        "id":"native","producer":owner,"steps":[{"step":"Opaque fallback","status":"pending"}],"source":"native plan"
    }})).unwrap();
    Store::new(home.0.join("state"))
        .apply(&home.key("goal"), native)
        .unwrap();
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"steering","label":"Expose input capability","owner":owner,"status":"blocked","source":"worker report",
            "parent_id":"design","depends_on":["runtime-api"],
            "outcome_links":[{"criterion_id":"check","goal_revision":1}],
            "purpose":"Explain input before sending","change":"Child input is restricted",
            "next":"Offer coordinator selection","blocker":"Await runtime docs"
        }),
    );
    let mut ui = home.ui();
    let shown = wait_text(&mut ui, "Expose input capability");
    assert!(!shown.contains("Opaque fallback"), "{shown}");
    assert!(shown.contains("blocked"), "{shown}");
    assert!(shown.contains("Explain input before sending"), "{shown}");
    assert!(shown.contains("Child input is restricted"), "{shown}");
    assert!(shown.contains("Offer coordinator selection"), "{shown}");
    assert!(shown.contains("Await runtime docs"), "{shown}");
    assert!(shown.contains("runtime-api (unresolved)"), "{shown}");
    assert!(shown.contains("design (unresolved)"), "{shown}");
    assert!(shown.contains("Behavior checked"), "{shown}");
    assert!(shown.contains("worker report"), "{shown}");
    assert!(shown.contains("0 of 2 verified"), "{shown}");
    ui.shutdown();
}

#[cfg(unix)]
#[test]
fn logical_work_owner_targets_its_native_thread_and_pins_an_open_draft() {
    let peer = Peer::new("control");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Parser work");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"root-goal","thread_source":"subagent","source":{"subagent":{"thread_spawn":{"parent_thread_id":"root-goal","agent_path":"/root/parser","agent_nickname":"Parser"}}}}}
"#).unwrap();
    let owner = serde_json::json!({"session":{"provider":"codex","session_id":"root-goal"},"agent_id":"/root/parser"});
    save_work_report(
        &home,
        "root-goal",
        0,
        serde_json::json!({"id":"steering","label":"Expose steering","owner":owner,"status":"active","source":"worker"}),
    );
    save_work_report(
        &home,
        "root-goal",
        0,
        serde_json::json!({"id":"docs","label":"Write usage","owner":{"session":{"provider":"codex","session_id":"root-goal"},"agent_id":"main"},"status":"active","source":"coordinator"}),
    );
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Expose steering");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Recipient: Parser");
    save_work_report(
        &home,
        "root-goal",
        1,
        serde_json::json!({"id":"steering","label":"Review steering","status":"done"}),
    );
    let refreshed = wait_text(&mut ui, "Selected work: Review steering");
    assert!(refreshed.contains("Recipient: Parser"), "{refreshed}");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Keep the public interface");
    // A refreshed duplicate makes the reported alias ambiguous while the
    // already-open draft keeps the exact recipient chosen before that change.
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-conflict.jsonl"), r#"{"type":"session_meta","payload":{"id":"conflicting-child","session_id":"root-goal","thread_source":"subagent","parent_thread_id":"root-goal","agent_path":"/root/parser"}}
"#).unwrap();
    save_work_report(
        &home,
        "root-goal",
        2,
        serde_json::json!({"id":"steering","owner":{"session":{"provider":"codex","session_id":"root-goal"},"agent_id":"main"}}),
    );
    Store::new(home.0.join("state"))
        .apply(
            &home.key("root-goal"),
            WorkUpdate::Verify {
                id: "build".into(),
                revision: Some(1),
                evidence: vec![Evidence {
                    kind: EvidenceKind::UserAttestation,
                    detail: "Build checked".into(),
                    provenance: "operator".into(),
                }],
            },
        )
        .unwrap();
    let pinned = wait_text(&mut ui, "1 of 2 verified");
    assert!(pinned.contains("Recipient: Parser"), "{pinned}");
    assert!(pinned.contains("Keep the public interface"), "{pinned}");
    key(&mut ui, KeyCode::F(2));
    wait_text(&mut ui, "Accepted by Codex");
    ctrl(&mut ui, 'u');
    type_text(&mut ui, "Do docs later");
    key(&mut ui, KeyCode::F(3));
    wait_text(&mut ui, "Queued by Codex");
    key(&mut ui, KeyCode::F(4));
    wait_text(&mut ui, "accepted the interruption");
    ui.shutdown();
    peer.finish();
}

fn click_console_action(ui: &mut FleetConsole, label: &str) {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let rendered = text(ui, 150, 42);
    let (column, row) = rendered
        .lines()
        .enumerate()
        .find_map(|(row, line)| {
            line.find(label)
                .map(|column| (unicode_width::UnicodeWidthStr::width(&line[..column]), row))
        })
        .unwrap_or_else(|| panic!("missing action {label}: {rendered}"));
    ui.handle_event(&Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: column as u16,
        row: row as u16,
        modifiers: KeyModifiers::NONE,
    }));
}

#[test]
fn visible_work_and_agents_actions_switch_inspection_without_losing_the_goal() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({"id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
        "status":"active","source":"coordinator"}),
    );
    let mut ui = home.ui();
    let shown = wait_text(&mut ui, "Expose steering");
    assert!(shown.contains("[Work w]"), "{shown}");
    assert!(shown.contains("[Agents a]"), "{shown}");
    click_console_action(&mut ui, "[Agents a]");
    let agents = wait_text(&mut ui, "Agents expanded");
    assert!(agents.contains("Agent inspection"), "{agents}");
    assert!(agents.contains("Selected: Understand steering"), "{agents}");
    click_console_action(&mut ui, "[Work w]");
    let work = wait_text(&mut ui, "Selected work: Expose steering");
    assert!(work.contains("Work toward goal"), "{work}");
    assert!(work.contains("0 of 2 verified"), "{work}");
    ui.shutdown();
}

#[test]
fn narrow_work_inspection_keeps_current_context_and_control_availability_visible() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    for at in 0..6 {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":format!("step-{at}"),"label":format!("Meaningful work phase {at}"),
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
                "status":if at == 4 {"active"} else {"planned"},"source":"coordinator",
                "purpose":"Keep recipient clear","next":"Check direct input"
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Meaningful work phase 4");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Selected work: Meaningful work phase 4");
    let shown = text(&mut ui, 60, 28);
    assert!(shown.contains("Meaningful work phase 4"), "{shown}");
    assert!(shown.contains("›"), "{shown}");
    assert!(shown.contains("Owner: Coordinator"), "{shown}");
    assert!(shown.contains("Purpose: Keep recipient clear"), "{shown}");
    assert!(shown.contains("Next: Check direct input"), "{shown}");
    assert!(shown.contains("Guidance unavailable"), "{shown}");
    assert!(shown.contains("Queue unavailable"), "{shown}");
    assert!(shown.contains("Stop unavailable"), "{shown}");
    ui.shutdown();
}

#[test]
fn commands_inspection_uses_the_selected_work_owner_and_dates_public_narration() {
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    let child = home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl");
    std::fs::write(&child, r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"goal","thread_source":"subagent","parent_thread_id":"goal","agent_nickname":"Parser","agent_path":"/root/parser"}}
"#).unwrap();
    let message = serde_json::json!({"timestamp":"2026-10-27T10:00:00Z","type":"response_item","payload":{"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":"Checking child input"}]}});
    let command = serde_json::json!({"timestamp":"2026-10-27T12:00:00Z","type":"response_item","payload":{"type":"function_call","name":"exec_command","call_id":"child-command","arguments":serde_json::json!({"cmd":"cargo test --parser","description":"Check child delivery"}).to_string()}});
    let mut stream = std::fs::OpenOptions::new()
        .append(true)
        .open(child)
        .unwrap();
    writeln!(stream, "{message}\n{command}").unwrap();
    let parent_command = serde_json::json!({"timestamp":"2026-10-27T12:30:00Z","type":"response_item","payload":{"type":"function_call","name":"exec_command","call_id":"parent-command","arguments":serde_json::json!({"cmd":"printf coordinator-only"}).to_string()}});
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl"))
            .unwrap(),
        "{parent_command}"
    )
    .unwrap();
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({"id":"steering","label":"Inspect child command",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"/root/parser"},
        "status":"active","source":"worker"}),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Inspect child command");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Recipient: Parser");
    click_console_action(&mut ui, "[Commands x]");
    let shown = wait_text(&mut ui, "cargo test --parser");
    assert!(shown.contains("Checking child input"), "{shown}");
    assert!(shown.contains("2026-10-27T10:00:00Z"), "{shown}");
    assert!(shown.contains("2026-10-27T12:00:00Z"), "{shown}");
    assert!(shown.contains("Codex commentary"), "{shown}");
    assert!(shown.contains("Check child delivery"), "{shown}");
    assert!(!shown.contains("coordinator-only"), "{shown}");
    let undated = serde_json::json!({"type":"response_item","payload":{"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":"Latest undated child update"}]}});
    writeln!(stream, "{undated}").unwrap();
    let refreshed = wait_text(&mut ui, "Latest undated child update");
    assert!(refreshed.contains("Recorded: unknown"), "{refreshed}");
    assert!(!refreshed.contains("2026-10-27T10:00:00Z"), "{refreshed}");
    assert!(refreshed.contains("2026-10-27T12:00:00Z"), "{refreshed}");
    ui.shutdown();
}

#[test]
fn work_focused_coordinator_and_tracking_actions_stay_explicit_and_request_missing_criteria() {
    let home = Home::new();
    let project = home.0.join("project's tracking");
    std::fs::create_dir_all(&project).unwrap();
    home.session("goal", project.to_str().unwrap(), false);
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"goal","thread_source":"subagent","parent_thread_id":"goal","agent_nickname":"Parser"}}
"#).unwrap();
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({"id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"child-thread"},
        "status":"active","source":"worker"}),
    );
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        state_executable: Some(PathBuf::from(env!("CARGO_BIN_EXE_zoe"))),
        socket: home.0.join("no-runtime.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Expose steering");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Recipient: Parser");
    key(&mut ui, KeyCode::Char('C'));
    wait_text(&mut ui, "Recipient: Coordinator");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Recipient: Parser");
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Keep the child draft");
    key(&mut ui, KeyCode::Esc);
    wait_text(&mut ui, "Draft saved");
    key(&mut ui, KeyCode::Esc);
    key(&mut ui, KeyCode::Char('T'));
    let tracking = wait_text(&mut ui, "Tracking draft for Coordinator");
    assert!(tracking.contains("Recipient: Coordinator"), "{tracking}");
    ui.shutdown();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["state", "--root"])
        .arg(home.0.join("state"))
        .args(["show", "codex", "goal"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let record: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let drafts = record["record"]["drafts"].as_array().unwrap();
    let coordinator = drafts
        .iter()
        .find(|draft| draft["target"]["agent_id"] == "main")
        .unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(
        coordinator.contains(
            home.0
                .join("codex/skills/zoetrope-tracking/SKILL.md")
                .to_str()
                .unwrap()
        ),
        "{coordinator}"
    );
    assert!(
        coordinator.contains(env!("CARGO_BIN_EXE_zoe")),
        "{coordinator}"
    );
    let status_command = coordinator
        .lines()
        .find(|line| line.contains(" tracking status "))
        .expect("tracking setup must supply a usable exact-target status command");
    let checked = std::process::Command::new("/bin/sh")
        .args(["-c", status_command])
        .output()
        .unwrap();
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let scope: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
    assert_eq!(scope["project"], project.to_str().unwrap());
    assert_eq!(
        scope["session"],
        serde_json::json!({"provider":"codex","session_id":"goal"})
    );
    assert_eq!(scope["project_opt_in"], false);
    assert!(coordinator.contains("authorized"), "{coordinator}");
    assert!(coordinator.contains("unavailable"), "{coordinator}");
    assert!(coordinator.contains("set_goal"), "{coordinator}");
    assert!(coordinator.contains("final checks"), "{coordinator}");
    assert!(coordinator.contains("unverified"), "{coordinator}");
    assert!(
        drafts
            .iter()
            .any(|draft| draft["target"]["agent_id"] == "child-thread"
                && draft["text"] == "Keep the child draft")
    );
    assert_eq!(record["record"]["goal"], serde_json::Value::Null);
    assert_eq!(record["record"]["outbox"], serde_json::json!([]));
}

#[test]
fn unindexed_work_owners_keep_a_useful_recipient_id_and_full_identity_in_details() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({"id":"steering","label":"Review unavailable child",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"unindexed-child-0123456789abcdef"},
        "status":"active","source":"coordinator"}),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Review unavailable child");
    key(&mut ui, KeyCode::Char('w'));
    let shown = wait_text(&mut ui, "Recipient: Agent unindexed");
    assert!(
        shown.contains("Owner ID: unindexed-child-0123456789abcdef"),
        "{shown}"
    );
    key(&mut ui, KeyCode::Char('C'));
    wait_text(&mut ui, "Recipient: Coordinator");
    ui.shutdown();
}

#[test]
fn native_plan_revisions_do_not_retarget_selected_work_and_coordinator_is_explicit_recovery() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand native plans");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"goal","thread_source":"subagent","parent_thread_id":"goal","agent_nickname":"Parser"}}
"#).unwrap();
    let store = Store::new(home.0.join("state"));
    let plan = |id: &str, label: &str| -> WorkUpdate {
        serde_json::from_value(serde_json::json!({"op":"import_plan","proposal":{
            "id":id,"producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"child-thread"},
            "steps":[{"step":label,"status":"in_progress"}],"source":"native plan"
        }})).unwrap()
    };
    store
        .apply(&home.key("goal"), plan("first-native", "Original work"))
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Original work");
    key(&mut ui, KeyCode::Char('w'));
    wait_text(&mut ui, "Recipient: Parser");
    store
        .apply(&home.key("goal"), plan("next-native", "Replacement work"))
        .unwrap();
    let expired = wait_text(&mut ui, "Selected work is no longer recorded");
    assert!(expired.contains("Selected work unavailable"), "{expired}");
    assert!(
        !expired.contains("Selected work: Replacement work"),
        "{expired}"
    );
    key(&mut ui, KeyCode::Char('C'));
    wait_text(&mut ui, "Recipient: Coordinator");
    key(&mut ui, KeyCode::Char('w'));
    let selected = wait_text(&mut ui, "Selected work: Replacement work");
    assert!(selected.contains("Recipient: Parser"), "{selected}");
    ui.shutdown();
}

#[test]
fn work_scope_links_go_stale_and_unknown_dependencies_stay_distinct_from_none() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Understand steering");
    let owner =
        serde_json::json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({"id":"steering","label":"Expose steering","owner":owner,
        "status":"active","source":"coordinator","outcome_links":[{"criterion_id":"check","goal_revision":1}]}),
    );
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({"id":"final","label":"Final checks","owner":owner,
        "status":"planned","source":"coordinator","depends_on":[]}),
    );
    Store::new(home.0.join("state"))
        .apply(
            &home.key("goal"),
            WorkUpdate::SetGoal {
                title: "Understand steering".into(),
                criteria: vec![
                    CriterionInput {
                        id: "build".into(),
                        label: "Build passes".into(),
                        weight: 1.0,
                    },
                    CriterionInput {
                        id: "check".into(),
                        label: "Behavior checked".into(),
                        weight: 1.0,
                    },
                    CriterionInput {
                        id: "docs".into(),
                        label: "Usage documented".into(),
                        weight: 1.0,
                    },
                ],
                reason: "Add usage documentation".into(),
                provenance: "operator".into(),
            },
        )
        .unwrap();
    let mut ui = home.ui();
    let scope = wait_text(&mut ui, "stale scope");
    assert!(scope.contains("revision 1; current 2"), "{scope}");
    assert!(scope.contains("Dependencies: not declared"), "{scope}");
    assert!(scope.contains("0 of 3 verified"), "{scope}");
    key(&mut ui, KeyCode::Char('w'));
    key(&mut ui, KeyCode::Down);
    let independent = wait_text(&mut ui, "Selected work: Final checks");
    assert!(
        independent.contains("Dependencies: none declared"),
        "{independent}"
    );
    save_work_report(
        &home,
        "goal",
        1,
        serde_json::json!({"id":"final","status":"done"}),
    );
    let done = wait_text(&mut ui, "reported done");
    assert!(done.contains("0 of 3 verified"), "{done}");
    ui.shutdown();
}

#[test]
fn work_opens_task_canvas_and_back_restores_the_fleet_view() {
    let home = Home::new();
    home.session("goal", "/projects/steering", false);
    home.goal("goal", "Ship navigation");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"inspect","label":"Inspect work graph", "status":"active", "source":"task report",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "depends_on":[]
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Inspect work graph");
    let fleet = text(&mut ui, 150, 42);
    let canvas = |screen: &str| {
        screen
            .lines()
            .skip(2)
            .take(38)
            .map(|row| row.chars().take(90).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert!(!canvas(&fleet).contains("Inspect work graph"), "{fleet}");
    key(&mut ui, KeyCode::Char('w'));
    let work = text(&mut ui, 150, 42);
    assert!(work.contains("Work graph"), "{work}");
    assert!(
        work.contains("Fleet / steering / Ship navigation"),
        "{work}"
    );
    assert!(canvas(&work).contains("Inspect work graph"), "{work}");
    key(&mut ui, KeyCode::Esc);
    let back = text(&mut ui, 150, 42);
    assert_eq!(
        canvas(&back),
        canvas(&fleet),
        "back must retain Fleet placement/camera"
    );
    key(&mut ui, KeyCode::Enter);
    assert!(text(&mut ui, 150, 42).contains("Work graph"));
    ui.shutdown();
}

#[test]
fn work_canvas_draws_declared_prerequisites_and_keeps_parent_grouping_separate() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Declared work");
    for (id, label, depends, parent) in [
        ("ready", "Ready prerequisite", serde_json::json!([]), None),
        (
            "build",
            "Build dependent",
            serde_json::json!(["ready"]),
            Some("group"),
        ),
        ("group", "Parent grouping", serde_json::Value::Null, None),
    ] {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":id,"label":label,"status":"active","source":"task report",
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
                "depends_on":depends,"parent_id":parent
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Ready prerequisite");
    key(&mut ui, KeyCode::Char('w'));
    key(&mut ui, KeyCode::Char('o'));
    let shown = text(&mut ui, 200, 60);
    let canvas = shown
        .lines()
        .skip(2)
        .take(56)
        .map(|row| row.chars().take(120).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(canvas.contains("Ready prerequisite"), "{shown}");
    assert!(canvas.contains("Build dependent"), "{shown}");
    assert!(canvas.contains("Parent grouping"), "{shown}");
    assert!(
        canvas.contains("requires"),
        "dependency must be drawn as an edge: {shown}"
    );
    assert!(shown.contains("1 prerequisite edge"), "{shown}");
    key(&mut ui, KeyCode::Down);
    let selected = text(&mut ui, 200, 60);
    assert!(
        selected.contains("Parent: Parent grouping (group)"),
        "{selected}"
    );
    assert!(selected.contains("grouping only"), "{selected}");
    ui.shutdown();
}

#[test]
fn cyclic_missing_and_disconnected_work_remain_visible_and_selectable() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Imperfect declarations");
    for (id, label, deps) in [
        ("a", "Cycle first", serde_json::json!(["b"])),
        (
            "b",
            "Cycle second",
            serde_json::json!(["a", "missing-input"]),
        ),
        ("c", "Independent work", serde_json::Value::Null),
    ] {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":id,"label":label,"status":"active","source":"task report",
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},"depends_on":deps
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Cycle first");
    key(&mut ui, KeyCode::Char('w'));
    key(&mut ui, KeyCode::Char('o'));
    let shown = text(&mut ui, 200, 60);
    assert!(shown.contains("2 cyclic tasks"), "{shown}");
    assert!(shown.contains("Unresolved: missing-input"), "{shown}");
    assert!(shown.contains("Independent work"), "{shown}");
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    let independent = text(&mut ui, 200, 60);
    assert!(
        independent.contains("Selected work: Independent work"),
        "{independent}"
    );
    assert!(
        independent.contains("Dependencies: not declared"),
        "{independent}"
    );
    ui.shutdown();
}

#[test]
fn task_mouse_selection_and_dragged_camera_survive_status_refresh() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Stable graph");
    for (id, label, owner) in [
        ("a", "First task", "main"),
        ("b", "Second task", "worker-two"),
    ] {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":id,"label":label,"status":"active","source":"task report",
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":owner},"depends_on":[]
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "First task");
    key(&mut ui, KeyCode::Char('w'));
    let locate = |screen: &str, needle: &str| {
        screen
            .lines()
            .enumerate()
            .skip(2)
            .find_map(|(row, line)| {
                let line: String = line.chars().take(90).collect();
                line.find(needle).map(|column| {
                    (
                        unicode_width::UnicodeWidthStr::width(&line[..column]) as u16,
                        row as u16,
                    )
                })
            })
            .unwrap_or_else(|| panic!("missing canvas label {needle}: {screen}"))
    };
    let (x, y) = locate(&text(&mut ui, 150, 42), "Second task");
    let pointer = |ui: &mut FleetConsole, kind, x, y| {
        ui.handle_event(&Event::Mouse(MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        }));
    };
    pointer(&mut ui, MouseEventKind::Down(MouseButton::Left), x, y);
    pointer(&mut ui, MouseEventKind::Up(MouseButton::Left), x, y);
    let selected = text(&mut ui, 150, 42);
    assert!(
        selected.contains("Selected work: Second task"),
        "{selected}"
    );
    assert!(
        selected.contains("Recipient: Agent worker-two"),
        "{selected}"
    );
    pointer(&mut ui, MouseEventKind::Down(MouseButton::Left), x, y);
    pointer(
        &mut ui,
        MouseEventKind::Drag(MouseButton::Left),
        x + 6,
        y + 2,
    );
    pointer(&mut ui, MouseEventKind::Up(MouseButton::Left), x + 6, y + 2);
    let dragged = text(&mut ui, 150, 42);
    let moved = locate(&dragged, "Second task");
    assert_ne!(moved, (x, y), "drag should move the task");
    key(&mut ui, KeyCode::Right);
    let panned = locate(&text(&mut ui, 150, 42), "Second task");
    save_work_report(
        &home,
        "goal",
        1,
        serde_json::json!({"id":"b","status":"blocked","next":"Keep inspecting"}),
    );
    let refreshed = wait_text(&mut ui, "Keep inspecting");
    assert!(
        refreshed.contains("Selected work: Second task"),
        "{refreshed}"
    );
    assert_eq!(
        locate(&refreshed, "Second task"),
        panned,
        "refresh must preserve dragged position and camera"
    );
    key(&mut ui, KeyCode::Char('x'));
    assert!(text(&mut ui, 150, 42).contains("Commands and public activity"));
    key(&mut ui, KeyCode::Esc);
    key(&mut ui, KeyCode::Char('R'));
    assert!(text(&mut ui, 150, 42).contains("Read-only replay requested"));
    ui.shutdown();
}

#[test]
fn agents_back_returns_to_the_selected_work_and_its_camera() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Route navigation");
    for id in ["first", "second"] {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":id,"label":format!("Task {id}"),"status":"active","source":"report",
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
            }),
        );
    }
    let mut ui = home.ui();
    let fleet = wait_text(&mut ui, "Task first");
    assert!(fleet.contains("Zoetrope · Fleet"), "{fleet}");
    key(&mut ui, KeyCode::Char('w'));
    key(&mut ui, KeyCode::Down);
    text(&mut ui, 150, 42);
    key(&mut ui, KeyCode::Right);
    let work = text(&mut ui, 150, 42);
    let canvas = |screen: &str| {
        screen
            .lines()
            .skip(3)
            .take(37)
            .map(|row| row.chars().take(90).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    };
    key(&mut ui, KeyCode::Char('a'));
    let agents = wait_text(&mut ui, "Agents expanded");
    assert!(agents.contains("Zoetrope · Agents"), "{agents}");
    key(&mut ui, KeyCode::Esc);
    let restored = text(&mut ui, 150, 42);
    assert!(
        restored.contains("Selected work: Task second"),
        "{restored}"
    );
    assert_eq!(
        canvas(&restored),
        canvas(&work),
        "Back restores the preceding Work camera and selection"
    );
    click_console_action(&mut ui, "[Back Esc]");
    assert!(text(&mut ui, 150, 42).contains("Zoetrope · Fleet"));
    ui.shutdown();
}

#[test]
fn dense_work_graph_reports_omissions_and_navigation_stays_bounded() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Bounded declarations");
    for at in 0..80 {
        let dependencies: Vec<_> = (0..64)
            .filter(|dependency| *dependency != at)
            .map(|dependency| format!("task-{dependency}"))
            .collect();
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":format!("task-{at}"),"label":format!("Bounded task {at}"),"status":"active","source":"report",
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}, "depends_on":dependencies
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Bounded task 0");
    let started = Instant::now();
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 150, 42);
    assert!(shown.contains("references omitted"), "{shown}");
    assert!(shown.contains("2048 prerequisite edges"), "{shown}");
    for _ in 0..79 {
        key(&mut ui, KeyCode::Down);
    }
    let last = text(&mut ui, 150, 42);
    assert!(last.contains("Selected work: Bounded task 79"), "{last}");
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "bounded interaction took {:?}",
        started.elapsed()
    );
    ui.shutdown();
}

#[test]
fn bound_source_tasks_draw_dependencies_and_human_ownership_never_selects_a_recipient() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Source projection");
    let project = home.0.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("WORK.md"), "## Tasks\n| ID | Task | Status | Owner | Blocked by | Next |\n| --- | --- | --- | --- | --- | --- |\n| T1 | Native implementation | active | main | None | Run checks |\n| T2 | Human approval | planned | Alice | T1 | Review evidence |\n").unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["tracking", "enable", "--project"])
        .arg(&project)
        .args([
            "--provider",
            "codex",
            "--session",
            "goal",
            "--source",
            "WORK.md#Tasks",
            "--root",
        ])
        .arg(home.0.join("state"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Native implementation");
    key(&mut ui, KeyCode::Char('w'));
    let native = text(&mut ui, 150, 42);
    assert!(native.contains("1 prerequisite edge"), "{native}");
    key(&mut ui, KeyCode::Down);
    let human = text(&mut ui, 150, 42);
    assert!(human.contains("Selected work: Human approval"), "{human}");
    assert!(human.contains("Owner: Alice"), "{human}");
    assert!(human.contains("Selected work unavailable"), "{human}");
    assert!(!human.contains("Recipient: Coordinator"), "{human}");
    assert!(human.contains("Source ID:"), "{human}");
    assert!(human.contains("WORK.md#Tasks"), "{human}");
    ui.shutdown();
}

#[test]
fn source_snapshot_replaces_the_old_plan_and_removed_task_selection_expires() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Source lifetime");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"legacy", "label":"Obsolete standalone report", "status":"active", "source":"old report",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
        }),
    );
    let project = home.0.join("project");
    std::fs::create_dir_all(&project).unwrap();
    let write_source = |body: &str| {
        std::fs::write(
            project.join("WORK.md"),
            format!("## Tasks\n| ID | Task | Status | Owner |\n| --- | --- | --- | --- |\n{body}"),
        )
        .unwrap()
    };
    let sync = |operation: &str| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"));
        command.args(["tracking", operation]);
        if operation == "enable" {
            command.args(["--source", "WORK.md#Tasks"]);
        }
        let output = command
            .args(["--project"])
            .arg(&project)
            .args(["--provider", "codex", "--session", "goal", "--root"])
            .arg(home.0.join("state"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    write_source("| T1 | Current document task | active | main |\n");
    sync("enable");
    let mut ui = home.ui();
    let current = wait_text(&mut ui, "Current document task");
    assert!(!current.contains("Obsolete standalone report"), "{current}");
    key(&mut ui, KeyCode::Char('w'));
    write_source("");
    sync("sync");
    let retired = wait_text(&mut ui, "No reported work");
    assert!(retired.contains("Selected work unavailable"), "{retired}");
    assert!(!retired.contains("Current document task"), "{retired}");
    assert!(!retired.contains("Obsolete standalone report"), "{retired}");
    ui.shutdown();
}

#[test]
fn work_details_prioritize_task_context_and_keep_full_commands_behind_commands() {
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Focused details");
    let command = serde_json::json!({"timestamp":"2026-10-27T12:00:00Z","type":"response_item","payload":{
        "type":"function_call","name":"exec_command","call_id":"command",
        "arguments":serde_json::json!({"cmd":"cargo test --known-public-command"}).to_string()
    }});
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl"))
            .unwrap(),
        "{command}"
    )
    .unwrap();
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"focus", "label":"Inspect current task", "status":"blocked", "source":"attributed worker report",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Understand the intended result", "next":"Check the public seam", "blocker":"Await review"
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Inspect current task");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 200, 70);
    assert!(
        !shown.contains("cargo test --known-public-command"),
        "{shown}"
    );
    assert!(
        shown.find("Purpose: Understand").unwrap() < shown.find("Source: attributed").unwrap(),
        "{shown}"
    );
    key(&mut ui, KeyCode::Char('x'));
    wait_text(&mut ui, "cargo test --known-public-command");
    ui.shutdown();
}

#[cfg(unix)]
#[test]
fn ambiguous_work_owner_disables_controls_without_rerouting_to_coordinator() {
    let peer = Peer::new("observe");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Ambiguous ownership");
    for native in ["child-thread", "other-child"] {
        let meta = serde_json::json!({"type":"session_meta","payload":{
            "id":native,"session_id":"root-goal","thread_source":"subagent",
            "source":{"subagent":{"thread_spawn":{"parent_thread_id":"root-goal","agent_path":"/root/parser"}}}
        }});
        std::fs::write(
            home.0
                .join(format!("codex/sessions/2026/10/23/rollout-{native}.jsonl")),
            format!("{meta}\n"),
        )
        .unwrap();
    }
    save_work_report(
        &home,
        "root-goal",
        0,
        serde_json::json!({
            "id":"parser","label":"Resolve parser ownership","status":"active","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"root-goal"},"agent_id":"/root/parser"}
        }),
    );
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Resolve parser ownership");
    key(&mut ui, KeyCode::Char('w'));
    let shown = wait_text(&mut ui, "Guidance unavailable: owner path is ambiguous");
    assert!(shown.contains("Owner ID: /root/parser"), "{shown}");
    assert!(
        shown.contains("Queue unavailable: owner path is ambiguous"),
        "{shown}"
    );
    assert!(
        shown.contains("Stop unavailable: owner path is ambiguous"),
        "{shown}"
    );
    key(&mut ui, KeyCode::Char('f'));
    wait_text(&mut ui, "Feedback unavailable: owner path is ambiguous");
    for action in [KeyCode::F(2), KeyCode::F(3), KeyCode::F(4)] {
        key(&mut ui, action);
        wait_text(&mut ui, "owner path is ambiguous");
    }
    ui.shutdown();
    peer.finish();
}

#[cfg(unix)]
#[test]
fn missing_work_owner_is_explained_without_matching_a_similar_agent_name() {
    let peer = Peer::new("observe");
    let home = Home::new();
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Missing ownership");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"root-goal","thread_source":"subagent","parent_thread_id":"root-goal","agent_path":"/root/different","agent_nickname":"/root/missing"}}
"#).unwrap();
    save_work_report(
        &home,
        "root-goal",
        0,
        serde_json::json!({
            "id":"parser","label":"Missing parser ownership","status":"active","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"root-goal"},"agent_id":"/root/missing"}
        }),
    );
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Missing parser ownership");
    key(&mut ui, KeyCode::Char('w'));
    let shown = wait_text(
        &mut ui,
        "Guidance unavailable: owner path has no native match",
    );
    assert!(shown.contains("Owner ID: /root/missing"), "{shown}");
    key(&mut ui, KeyCode::Char('f'));
    wait_text(
        &mut ui,
        "Feedback unavailable: owner path has no native match",
    );
    for action in [KeyCode::F(2), KeyCode::F(3), KeyCode::F(4)] {
        key(&mut ui, action);
    }
    ui.shutdown();
    peer.finish();
}

#[test]
fn readable_work_view_restores_the_selected_task_after_panning_offscreen() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Readable navigation");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"readable","label":"Find this task", "status":"active", "source":"report",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Find this task");
    key(&mut ui, KeyCode::Char('w'));
    text(&mut ui, 150, 42);
    let canvas = |screen: String| {
        screen
            .lines()
            .skip(3)
            .take(37)
            .map(|row| row.chars().take(90).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    };
    for _ in 0..30 {
        key(&mut ui, KeyCode::Right);
    }
    assert!(!canvas(text(&mut ui, 150, 42)).contains("Find this task"));
    key(&mut ui, KeyCode::Char('r'));
    assert!(canvas(text(&mut ui, 150, 42)).contains("Find this task"));
    ui.shutdown();
}

#[test]
fn reordering_source_rows_preserves_task_identity_placement_and_camera() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Stable source order");
    let project = home.0.join("project");
    std::fs::create_dir_all(&project).unwrap();
    let first = "| T1 | First stable task | active | main | Original order |\n";
    let second = "| T2 | Second stable task | planned | main | |\n";
    let write_source = |rows: String| {
        std::fs::write(project.join("WORK.md"), format!("## Tasks\n| ID | Task | Status | Owner | Next |\n| --- | --- | --- | --- | --- |\n{rows}")).unwrap()
    };
    let sync = |operation: &str| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"));
        command
            .args(["tracking", operation, "--project"])
            .arg(&project)
            .args(["--provider", "codex", "--session", "goal", "--root"])
            .arg(home.0.join("state"));
        if operation == "enable" {
            command.args(["--source", "WORK.md#Tasks"]);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    write_source(format!("{first}{second}"));
    sync("enable");
    let mut ui = home.ui();
    wait_text(&mut ui, "First stable task");
    key(&mut ui, KeyCode::Char('w'));
    text(&mut ui, 150, 42);
    key(&mut ui, KeyCode::Right);
    let canvas = |screen: String| {
        screen
            .lines()
            .skip(3)
            .take(37)
            .map(|row| row.chars().take(90).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let before = canvas(text(&mut ui, 150, 42));
    write_source(format!(
        "{second}{}",
        first.replace("Original order", "Rows reordered")
    ));
    sync("sync");
    let after = wait_text(&mut ui, "Rows reordered");
    assert!(
        after.contains("Selected work: First stable task"),
        "{after}"
    );
    assert_eq!(canvas(after), before, "row order is not a topology change");
    ui.shutdown();
}

#[test]
fn a_resolved_logical_report_suppresses_its_owners_native_plan_fallback() {
    let home = Home::new();
    home.session("goal", "/projects/graph", false);
    home.goal("goal", "Owner fallback");
    std::fs::write(home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl"), r#"{"type":"session_meta","payload":{"id":"child-thread","session_id":"goal","thread_source":"subagent","parent_thread_id":"goal","agent_path":"/root/parser","agent_nickname":"Parser"}}
"#).unwrap();
    let plan: WorkUpdate = serde_json::from_value(serde_json::json!({"op":"import_plan", "proposal":{
        "id":"native-plan","producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"child-thread"},
        "steps":[{"step":"Old native fallback","status":"in_progress"}],"source":"native"
    }})).unwrap();
    Store::new(home.0.join("state"))
        .apply(&home.key("goal"), plan)
        .unwrap();
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"report","label":"Reported parser task","status":"active","source":"report",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"/root/parser"}
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Reported parser task");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 150, 42);
    assert!(!shown.contains("Old native fallback"), "{shown}");
    assert!(shown.contains("Work graph · 1 tasks"), "{shown}");
    ui.shutdown();
}

#[test]
fn work_statuses_have_consistent_color_and_non_color_cues() {
    use ratatui::style::Color;
    let home = Home::new();
    home.session("goal", "/projects/visual", false);
    home.goal("goal", "Status visual language");
    for (at, status) in ["active", "planned", "blocked", "done", "cancelled"]
        .iter()
        .enumerate()
    {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":format!("task-{at}"), "label":format!("Visual phase {at}"), "status":status,
                "source":"coordinator", "depends_on":[],
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Visual phase 0");
    key(&mut ui, KeyCode::Char('w'));
    for (at, glyph, color) in [
        (0, "●", Color::Cyan),
        (1, "○", Color::LightBlue),
        (2, "!", Color::LightRed),
        (3, "✓", Color::Green),
        (4, "×", Color::DarkGray),
    ] {
        let mut terminal = Terminal::new(TestBackend::new(150, 42)).unwrap();
        terminal.draw(|f| ui.draw(f)).unwrap();
        let buffer = terminal.backend().buffer();
        let expected = format!("{glyph} Visual phase {at}");
        let row = buffer
            .content()
            .chunks(150)
            .find(|row| {
                row.iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>()
                    .contains(&expected)
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing semantic title {expected}: {}",
                    text(&mut ui, 150, 42)
                )
            });
        let cue = row.iter().find(|cell| cell.symbol() == glyph).unwrap();
        assert_eq!(cue.fg, color, "status cue for phase {at}");
        key(&mut ui, KeyCode::Down);
    }
    let shown = text(&mut ui, 150, 42);
    assert!(shown.contains("Legend:"), "{shown}");
    assert!(shown.contains("✓ done"), "{shown}");
    assert!(shown.contains("? unknown"), "{shown}");
    ui.shutdown();
}

#[test]
fn overview_retains_status_and_selection_before_readable_zoom_restores_titles() {
    use ratatui::style::{Color, Modifier};
    let home = Home::new();
    home.session("goal", "/projects/overview", false);
    home.goal("goal", "Inspect an overview");
    for at in 0..20 {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":format!("task-{at}"), "label":format!("Overview task {at}"),
                "status":if at == 0 {"active"} else {"planned"}, "source":"coordinator",
                "depends_on":if at == 0 {vec![]} else {vec![format!("task-{}",at-1)]},
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Overview task 0");
    key(&mut ui, KeyCode::Char('w'));
    text(&mut ui, 150, 42);
    key(&mut ui, KeyCode::Char('o'));
    let mut terminal = Terminal::new(TestBackend::new(150, 42)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    let selected = buffer.content().iter().find(|cell| {
        cell.symbol() == "●" && cell.bg == Color::Cyan && cell.modifier.contains(Modifier::REVERSED)
    });
    assert!(
        selected.is_some(),
        "overview must carry the active task and a monochrome selection cue: {}",
        text(&mut ui, 150, 42)
    );
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Char('r'));
    let readable = text(&mut ui, 150, 42);
    assert!(readable.contains("○ Overview task 1"), "{readable}");
    ui.shutdown();
}

#[test]
fn reported_work_counts_keep_cancellation_unknown_and_verified_outcomes_separate() {
    let home = Home::new();
    home.session("goal", "/projects/progress", false);
    home.goal("goal", "Honest progress");
    for (at, status) in ["active", "planned", "blocked", "done", "cancelled"]
        .iter()
        .enumerate()
    {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":format!("task-{at}"),"label":format!("Progress phase {at}"),"status":status,"source":"attributed coordinator",
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
            }),
        );
    }
    Store::new(home.0.join("state")).apply(&home.key("goal"),serde_json::from_value(serde_json::json!({
        "op":"import_plan","proposal":{"id":"unknown-plan",
        "producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"child-thread"},
        "steps":[{"step":"Await external status","status":"new-provider-state"}],"source":"native plan"}
    })).unwrap()).unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Progress phase 0");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 200, 50);
    assert!(shown.contains("Reported work: 1 of 6 done"), "{shown}");
    for status in [
        "1 active",
        "1 blocked",
        "1 planned",
        "1 cancelled",
        "1 unknown",
    ] {
        assert!(shown.contains(status), "missing {status}: {shown}");
    }
    assert!(shown.contains("Outcomes: 0 of 2 verified"), "{shown}");
    assert!(shown.contains("Source: attributed coordinator"), "{shown}");
    assert!(shown.contains("Recorded:"), "{shown}");
    ui.shutdown();
}

#[test]
fn narrow_work_view_keeps_a_readable_task_card_beside_focused_context() {
    let home = Home::new();
    home.session("goal", "/projects/narrow", false);
    home.goal("goal", "A usable narrow graph");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"current","label":"Inspect narrow graph","status":"active","source":"coordinator", "depends_on":[],
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Keep the graph useful","next":"Check small terminal","blocker":"Await review"
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Inspect narrow graph");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 60, 28);
    assert!(
        shown.contains("● Inspect narrow graph"),
        "a readable task card must fit: {shown}"
    );
    assert!(
        shown.contains("no prerequisites"),
        "the graph card must retain dependency context: {shown}"
    );
    for context in [
        "Owner: Coordinator",
        "Purpose: Keep the graph useful",
        "Next: Check small terminal",
        "Blocker: Await review",
        "Guidance unavailable",
        "Queue unavailable",
        "Stop unavailable",
    ] {
        assert!(shown.contains(context), "missing {context}: {shown}");
    }
    assert!(shown.contains("Reported:"), "{shown}");
    assert!(shown.contains("Outcomes:"), "{shown}");
    assert!(shown.contains("Legend:"), "{shown}");
    assert!(shown.contains("[Commands x]"), "{shown}");
    ui.shutdown();
}

#[test]
fn an_unknown_native_status_never_inherits_a_known_status_color_from_its_label() {
    use ratatui::style::Color;
    let home = Home::new();
    home.session("goal", "/projects/unknown", false);
    home.goal("goal", "Honest unknown status");
    Store::new(home.0.join("state")).apply(&home.key("goal"),serde_json::from_value(serde_json::json!({
        "op":"import_plan","proposal":{"id":"unknown-plan",
        "producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
        "steps":[{"step":"Unrecognized native state","status":"reported done"}],"source":"native plan"}
    })).unwrap()).unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Unrecognized native state");
    key(&mut ui, KeyCode::Char('w'));
    let mut terminal = Terminal::new(TestBackend::new(150, 42)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    let title = buffer.content().chunks(150).find(|row| {
        row.iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
            .contains("? Unrecognized native state")
    });
    assert!(
        title.is_some(),
        "unknown must retain its own glyph: {}",
        text(&mut ui, 150, 42)
    );
    assert_eq!(
        title
            .unwrap()
            .iter()
            .find(|cell| cell.symbol() == "?")
            .unwrap()
            .fg,
        Color::Magenta
    );
    let shown = text(&mut ui, 150, 42);
    assert!(shown.contains("unknown: reported done"), "{shown}");
    assert!(shown.contains("Reported work: 0 of 1 done"), "{shown}");
    ui.shutdown();
}

#[test]
fn focused_work_marks_selection_without_color_and_styles_its_blocker_consistently() {
    use ratatui::style::Color;
    let home = Home::new();
    home.session("goal", "/projects/attention", false);
    home.goal("goal", "Focused attention");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"review","label":"Review delivery","status":"blocked","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Make the gate clear","next":"Read review","blocker":"Review needed"
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Review delivery");
    key(&mut ui, KeyCode::Char('w'));
    let mut terminal = Terminal::new(TestBackend::new(150, 42)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    for value in ["Status: blocked", "Blocker: Review needed"] {
        let row = terminal
            .backend()
            .buffer()
            .content()
            .chunks(150)
            .find(|row| {
                row.iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>()
                    .contains(value)
            })
            .unwrap();
        let start = row
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
            .chars()
            .collect::<Vec<_>>()
            .windows(value.chars().count())
            .position(|part| part.iter().copied().eq(value.chars()))
            .unwrap();
        assert_eq!(row[start].fg, Color::LightRed, "{value}");
    }
    let shown = text(&mut ui, 150, 42);
    assert!(
        shown.contains("› selected"),
        "graph selection needs a non-color cue: {shown}"
    );
    ui.shutdown();
}

#[test]
fn opening_work_from_search_keeps_its_breadcrumb_and_visible_back_action() {
    let home = Home::new();
    home.session("goal", "/projects/search", false);
    home.goal("goal", "Searchable work");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"review","label":"Inspect search navigation","status":"active","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Searchable work");
    key(&mut ui, KeyCode::Char('/'));
    type_text(&mut ui, "Searchable work");
    key(&mut ui, KeyCode::Enter);
    wait_text(&mut ui, "Search: Searchable work");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 150, 42);
    assert!(
        shown.contains("[Back Esc] Fleet / search / Searchable work"),
        "{shown}"
    );
    assert!(shown.contains("Search: Searchable work"), "{shown}");
    click_console_action(&mut ui, "[Back Esc]");
    let restored = text(&mut ui, 150, 42);
    assert!(restored.starts_with("Zoetrope · Fleet"), "{restored}");
    ui.shutdown();
}

#[test]
fn narrow_graph_limit_notices_keep_the_status_legend_visible() {
    let home = Home::new();
    home.session("goal", "/projects/legend", false);
    home.goal("goal", "Bounded graph legend");
    for at in 0..40 {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":format!("task-{at}"),"label":format!("Dense work {at}"),"status":"active","source":"coordinator",
                "depends_on":(0..40).filter(|dependency|*dependency!=at).map(|dependency|format!("task-{dependency}")).chain((0..24).map(|missing|format!("missing-{missing}"))).collect::<Vec<_>>(),
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
            }),
        );
    }
    let mut ui = home.ui();
    wait_text(&mut ui, "Dense work 0");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 60, 28);
    assert!(shown.contains("LIMIT:"), "{shown}");
    for status in [
        "○ planned",
        "● active",
        "! blocked",
        "✓ done",
        "× cancelled",
        "? unknown",
    ] {
        assert!(
            shown.contains(status),
            "limit diagnostics must not hide {status}: {shown}"
        );
    }
    ui.shutdown();
}

#[cfg(unix)]
#[test]
fn source_native_owner_requires_exact_root_metadata_before_selecting_or_sending() {
    let peer = Peer::new("observe");
    let home = Home::new();
    home.session("foreign-root", "/projects/unrelated", true);
    home.session("root-goal", "/projects/parser", false);
    home.goal("root-goal", "Source native boundary");
    let child = home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl");
    std::fs::write(&child, "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child-thread\",\"session_id\":\"foreign-root\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"foreign-root\",\"agent_nickname\":\"Parser\"}}\n").unwrap();
    let project = home.0.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("WORK.md"), "## Tasks\n| ID | Task | Status | Owner |\n| --- | --- | --- | --- |\n| T1 | Foreign source owner | active | native:child-thread |\n| T2 | Missing source owner | active | native:missing-thread |\n| T3 | Exact root source owner | active | native:root-goal |\n").unwrap();
    let enabled = std::process::Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["tracking", "enable", "--project"])
        .arg(&project)
        .args([
            "--provider",
            "codex",
            "--session",
            "root-goal",
            "--source",
            "WORK.md#Tasks",
            "--root",
        ])
        .arg(home.0.join("state"))
        .output()
        .unwrap();
    assert!(
        enabled.status.success(),
        "{}",
        String::from_utf8_lossy(&enabled.stderr)
    );
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            codex_home: Some(home.0.join("codex")),
            claude_projects: None,
        },
        state_root: home.0.join("state"),
        socket: peer.dir.join("peer.sock"),
        ..FleetConfig::default()
    });
    wait_text(&mut ui, "Foreign source owner");
    key(&mut ui, KeyCode::Char('w'));
    let foreign = wait_text(
        &mut ui,
        "Guidance unavailable: source owner lacks native scope",
    );
    assert!(foreign.contains("Owner: native:child-thread"), "{foreign}");
    key(&mut ui, KeyCode::Char('f'));
    wait_text(
        &mut ui,
        "Feedback unavailable: source owner lacks native scope",
    );
    for code in ['g', 'q', 'x'] {
        ui.handle_event(&Event::Key(KeyEvent::new(
            KeyCode::Char(code),
            KeyModifiers::CONTROL,
        )));
    }
    key(&mut ui, KeyCode::Down);
    let missing = wait_text(&mut ui, "Selected work: Missing source owner");
    assert!(
        missing.contains("Guidance unavailable: source owner lacks native scope"),
        "{missing}"
    );
    key(&mut ui, KeyCode::Char('f'));
    wait_text(
        &mut ui,
        "Feedback unavailable: source owner lacks native scope",
    );
    std::fs::write(&child, "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child-thread\",\"session_id\":\"root-goal\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root-goal\",\"agent_nickname\":\"Parser\"}}\n").unwrap();
    key(&mut ui, KeyCode::Up);
    wait_text(&mut ui, "Guidance available: current turn");
    let resolved = text(&mut ui, 150, 42);
    assert!(resolved.contains("Recipient: Parser"), "{resolved}");
    assert!(
        resolved.contains("Owner: native:child-thread"),
        "{resolved}"
    );
    key(&mut ui, KeyCode::Down);
    key(&mut ui, KeyCode::Down);
    wait_text(&mut ui, "Recipient: Coordinator");
    ui.shutdown();
    peer.finish(); // Its public protocol fixture rejects every native control message.
}

#[test]
fn tall_work_overview_retains_meaningful_labels_across_a_twelve_task_branching_plan() {
    let home = Home::new();
    home.session("goal", "/projects/readable-overview", false);
    home.goal("goal", "Read the whole approach");
    for (id, label, deps) in [
        ("plan", "Plan approved", vec![]),
        ("checkout", "Checkout", vec!["plan"]),
        ("menus", "Choose menus", vec!["checkout"]),
        ("custom", "Customize", vec!["menus"]),
        ("locations", "Locations", vec!["menus"]),
        ("closing", "Close order", vec!["custom", "locations"]),
        ("payment", "Payment", vec!["closing"]),
        ("reminders", "Reminders", vec!["payment"]),
        ("corrections", "Corrections", vec!["payment"]),
        ("deploy", "Deploy", vec!["reminders", "corrections"]),
        ("cancelled", "Cancelled step", vec![]),
    ] {
        save_work_report(
            &home,
            "goal",
            0,
            serde_json::json!({
                "id":id,"label":label,"status":if id=="closing" {"active"} else if id=="cancelled" {"cancelled"} else {"planned"},
                "source":"coordinator","depends_on":deps,
                "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"}
            }),
        );
    }
    Store::new(home.0.join("state")).apply(&home.key("goal"),serde_json::from_value(serde_json::json!({
        "op":"import_plan","proposal":{"id":"unknown-plan",
        "producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"external-worker"},
        "steps":[{"step":"External review","status":"awaiting-review"}],"source":"native plan"}
    })).unwrap()).unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Close order");
    key(&mut ui, KeyCode::Char('w'));
    text(&mut ui, 106, 48);
    key(&mut ui, KeyCode::Char('o'));
    let shown = text(&mut ui, 106, 48);
    let graph = shown
        .lines()
        .skip(2)
        .take(44)
        .map(|line| line.chars().take(63).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    for label in [
        "Plan approved",
        "Choose menus",
        "Customize",
        "Locations",
        "Close order",
        "Payment",
        "Reminders",
        "Corrections",
        "Deploy",
    ] {
        assert!(
            graph.contains(label),
            "the overview must retain meaningful task labels, including both branches; missing {label}: {shown}"
        );
    }
    ui.shutdown();
}

#[test]
fn fleet_and_work_headers_use_a_readable_cool_accent() {
    use ratatui::style::Color;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Readable presentation");
    let mut ui = home.ui();
    wait_text(&mut ui, "Readable presentation");
    for work in [false, true] {
        if work {
            key(&mut ui, KeyCode::Char('w'));
        }
        let mut terminal = Terminal::new(TestBackend::new(150, 42)).unwrap();
        terminal.draw(|f| ui.draw(f)).unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), "Z");
        assert_eq!(buffer[(0, 0)].fg, Color::Rgb(137, 180, 250));
        let panel = buffer
            .content()
            .chunks(150)
            .find(|row| {
                row.iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>()
                    .contains("Selected work")
            })
            .unwrap();
        assert_eq!(panel[90].fg, Color::Rgb(137, 180, 250));
    }
    ui.shutdown();
}

fn rendered_cells<'a>(
    buffer: &'a ratatui::buffer::Buffer,
    value: &str,
) -> &'a [ratatui::buffer::Cell] {
    let symbols: Vec<_> = value.chars().map(|ch| ch.to_string()).collect();
    buffer
        .content()
        .windows(symbols.len())
        .find(|cells| {
            cells
                .iter()
                .map(|cell| cell.symbol())
                .eq(symbols.iter().map(String::as_str))
        })
        .unwrap_or_else(|| panic!("missing visible text {value:?}"))
}

#[test]
fn work_context_separates_sections_and_highlights_paths_code_and_emphasis() {
    use ratatui::style::{Color, Modifier};
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Readable context");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"parser","label":"Inspect parser","status":"active","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Inspect src/parser.rs:42 and `parse_input()` before **shipping**.",
            "next":"Read [the spec](docs/plan.md)."
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Inspect parser");
    key(&mut ui, KeyCode::Char('w'));
    let mut terminal = Terminal::new(TestBackend::new(220, 70)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    for path in ["src/parser.rs:42", "docs/plan.md"] {
        assert!(
            rendered_cells(buffer, path)
                .iter()
                .all(|cell| cell.fg == Color::Rgb(148, 226, 213)),
            "path {path} needs its own readable color"
        );
    }
    assert!(
        rendered_cells(buffer, "parse_input()")
            .iter()
            .all(|cell| cell.fg == Color::Rgb(203, 166, 247))
    );
    assert!(
        rendered_cells(buffer, "shipping")
            .iter()
            .all(|cell| cell.modifier.contains(Modifier::BOLD))
    );
    for heading in ["Context", "Provenance", "Controls"] {
        assert!(
            rendered_cells(buffer, heading)
                .iter()
                .all(|cell| cell.modifier.contains(Modifier::BOLD)),
            "section {heading} needs a visible heading"
        );
    }
    let shown = text(&mut ui, 220, 70);
    assert!(!shown.contains("**shipping**"), "{shown}");
    assert!(
        shown.contains("the spec (docs/plan.md)"),
        "links must retain their target: {shown}"
    );
    key(&mut ui, KeyCode::Char('f'));
    type_text(&mut ui, "Keep **literal** `code` in src/parser.rs");
    let editor = text(&mut ui, 220, 70);
    assert!(
        editor.contains("Keep **literal** `code` in src/parser.rs"),
        "formatting must leave editable drafts literal: {editor}"
    );
    ui.shutdown();
}

#[test]
fn recorded_messages_render_paragraphs_lists_and_code_without_rewriting_commands() {
    use ratatui::style::{Color, Modifier};
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Readable activity");
    let recording = home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl");
    let message = "## Delivery plan\nRead src/parser.rs:42, then `cargo test`.\n\n- [x] Updated README.md\n- [ ] Check ./tests/parser.rs\n\n> Keep changes small.\n\n```rust\nlet pattern = \"**literal**\";\n    // src/parser.rs\n```\n\nwarning: inspect C:\\repo\\src\\main.rs:8\nUnmatched `literal stays visible.";
    let command = "printf '%s\\n' '**literal**' && cargo test --offline";
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&recording)
        .unwrap();
    for payload in [
        serde_json::json!({"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":message}]}),
        serde_json::json!({"type":"function_call","name":"exec_command","call_id":"recorded-command","arguments":serde_json::json!({"cmd":command}).to_string()}),
    ] {
        writeln!(file,"{}",serde_json::json!({"timestamp":"2026-10-28T00:15:00Z","type":"response_item","payload":payload})).unwrap();
    }
    drop(file);
    let original = std::fs::read(&recording).unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Readable activity");
    key(&mut ui, KeyCode::Char('x'));
    wait_text(&mut ui, "Latest shell command");
    let mut terminal = Terminal::new(TestBackend::new(240, 90)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    assert!(
        rendered_cells(buffer, "Delivery plan")
            .iter()
            .all(|cell| cell.fg == Color::Rgb(137, 180, 250)
                && cell.modifier.contains(Modifier::BOLD))
    );
    for path in [
        "src/parser.rs:42",
        "README.md",
        "./tests/parser.rs",
        "C:\\repo\\src\\main.rs:8",
    ] {
        assert!(
            rendered_cells(buffer, path)
                .iter()
                .all(|cell| cell.fg == Color::Rgb(148, 226, 213)),
            "path {path}"
        );
    }
    let shown = text(&mut ui, 240, 90);
    for content in [
        "[x] Updated README.md",
        "[ ] Check ./tests/parser.rs",
        "│ Keep changes small.",
        "╭─ rust",
        "│ let pattern = \"**literal**\";",
        "│     // src/parser.rs",
        command,
        "Unmatched `literal stays visible.",
    ] {
        assert!(
            shown.contains(content),
            "missing readable/literal content {content:?}: {shown}"
        );
    }
    assert!(
        rendered_cells(buffer, "warning:")
            .iter()
            .all(|cell| cell.fg == Color::Rgb(250, 179, 135))
    );
    let rows: Vec<_> = shown.lines().collect();
    let paragraph = rows
        .iter()
        .position(|row| row.contains("Read src/parser.rs:42"))
        .unwrap();
    assert!(
        !rows[paragraph + 1].contains("Updated README.md"),
        "paragraph/list blocks need breathing room: {shown}"
    );
    ui.shutdown();
    assert_eq!(
        std::fs::read(&recording).unwrap(),
        original,
        "display formatting must leave recorded text unchanged"
    );
}

#[test]
fn recorded_control_characters_are_visible_without_changing_literal_code() {
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Readable control characters");
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl"))
        .unwrap();
    let message =
        "## Trace\n```text\n\t\u{1b}[31m src/naïve.rs\n```\nLiteral **unfinished stays visible.";
    let command = "printf '%s' '```' src/naïve.rs";
    for payload in [
        serde_json::json!({"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":message}]}),
        serde_json::json!({"type":"function_call","name":"exec_command","call_id":"literal-fence","arguments":serde_json::json!({"cmd":command}).to_string()}),
    ] {
        writeln!(
            file,
            "{}",
            serde_json::json!({"type":"response_item","payload":payload})
        )
        .unwrap();
    }
    drop(file);
    let mut ui = home.ui();
    wait_text(&mut ui, "Readable control characters");
    key(&mut ui, KeyCode::Char('x'));
    wait_text(&mut ui, "Latest shell command");
    let shown = text(&mut ui, 220, 70);
    assert!(
        shown.contains("│     ␛[31m src/naïve.rs"),
        "tabs/controls need readable glyphs: {shown}"
    );
    assert!(
        !shown.contains('\u{1b}'),
        "recorded escape characters must not enter terminal cells"
    );
    assert!(
        shown.contains(command),
        "literal command fences must remain visible: {shown}"
    );
    assert!(
        shown.contains("Literal **unfinished stays visible."),
        "unknown markup must retain its text: {shown}"
    );
    ui.shutdown();
}

#[test]
fn literal_brackets_do_not_consume_later_links_or_balanced_path_parentheses() {
    use ratatui::style::{Color, Modifier};
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Literal brackets and links");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"links", "label":"Read source links", "status":"active", "source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Keep [raw] and read [the spec](docs/SPEC.md).",
            "next":"Keep [unfinished before [the draft](docs/(draft)/SPEC.md)."
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Read source links");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 240, 80);
    assert!(
        shown.contains("Keep [raw] and read the spec (docs/SPEC.md)."),
        "literal brackets must survive a later link: {shown}"
    );
    assert!(
        shown.contains("Keep [unfinished before the draft (docs/(draft)/SPEC.md)."),
        "an unmatched bracket must not hide a later valid link: {shown}"
    );
    let mut terminal = Terminal::new(TestBackend::new(240, 80)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    assert!(
        rendered_cells(buffer, "[raw]")
            .iter()
            .all(|cell| !cell.modifier.contains(Modifier::UNDERLINED))
    );
    assert!(
        rendered_cells(buffer, "the spec")
            .iter()
            .all(|cell| cell.modifier.contains(Modifier::UNDERLINED))
    );
    assert!(
        rendered_cells(buffer, "docs/(draft)/SPEC.md")
            .iter()
            .all(|cell| cell.fg == Color::Rgb(148, 226, 213))
    );
    ui.shutdown();
}

#[test]
fn recorded_message_fences_cannot_absorb_commands_or_pane_footers() {
    use ratatui::style::{Color, Modifier};
    use std::io::Write;
    for closed in [false, true] {
        let home = Home::new();
        home.session("goal", "/projects/readability", false);
        home.goal("goal", "Independent activity sections");
        let path = home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl");
        let message = format!(
            "## Inspection\n```text\n## Latest shell command\nInside recorded code{}",
            if closed { "\n```" } else { "" }
        );
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        for payload in [
            serde_json::json!({"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":message}]}),
            serde_json::json!({"type":"function_call","name":"exec_command","call_id":"independent-command","arguments":serde_json::json!({"cmd":"cargo test --offline","justification":"Recorded command purpose"}).to_string()}),
        ] {
            writeln!(
                file,
                "{}",
                serde_json::json!({"type":"response_item","payload":payload})
            )
            .unwrap();
        }
        drop(file);
        let original = std::fs::read(&path).unwrap();
        let mut ui = home.ui();
        wait_text(&mut ui, "Inside recorded code");
        for view in [None, Some('a'), Some('x')] {
            if let Some(view) = view {
                key(&mut ui, KeyCode::Char(view));
            }
            let shown = text(&mut ui, 240, 90);
            assert!(
                shown.contains("│ ## Latest shell command"),
                "a trusted-looking heading inside recorded code must remain code: {shown}"
            );
            assert!(
                shown.contains("── Latest shell command"),
                "generated command section must remain independent: {shown}"
            );
            assert_eq!(
                shown.contains("unclosed fence"),
                !closed,
                "unfinished source formatting must be reported at its own end: {shown}"
            );
            assert!(
                shown.contains("│ cargo test --offline"),
                "the separate command must remain literal code: {shown}"
            );
            let mut terminal = Terminal::new(TestBackend::new(240, 90)).unwrap();
            terminal.draw(|f| ui.draw(f)).unwrap();
            let heading = rendered_cells(terminal.backend().buffer(), "── Latest shell command");
            assert!(
                heading
                    .iter()
                    .all(|cell| cell.fg == Color::Rgb(137, 180, 250)
                        && cell.modifier.contains(Modifier::BOLD))
            );
            if view == Some('x') {
                let footer = rendered_cells(terminal.backend().buffer(), "Esc back");
                assert!(
                    footer
                        .iter()
                        .all(|cell| cell.fg != Color::Rgb(203, 166, 247)),
                    "pane footer must not inherit source code formatting"
                );
            }
        }
        ui.shutdown();
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
}

#[test]
fn narrow_context_formats_before_clipping_and_keeps_multiline_detail_below() {
    use ratatui::style::Modifier;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Compact formatted context");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"compact","label":"Read compact context","status":"active","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Review src/parser.rs:42 and **keep scope focused throughout this important follow-up**.",
            "next":"Run `cargo test --locked`, then read [the spec](docs/WORK.md) to review scope.",
            "change":"Source fields are imported deterministically.\n\nSecond paragraph retained in recorded detail."
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Read compact context");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 60, 28);
    for literal_markup in ["**", "`cargo", "[the spec]"] {
        assert!(
            !shown.contains(literal_markup),
            "compact text must clip styled output rather than Markdown syntax: {shown}"
        );
    }
    assert!(
        shown.contains("the spec ("),
        "compact link should remain understandable: {shown}"
    );
    assert!(
        shown.contains('…'),
        "long compact fields need visible omission: {shown}"
    );
    assert!(
        !shown.contains("Second paragraph"),
        "a multi-line field must not displace compact controls: {shown}"
    );
    for cue in [
        "Guidance unavailable",
        "Queue unavailable",
        "Stop unavailable",
    ] {
        assert!(
            shown.contains(cue),
            "compact control availability must remain visible: {shown}"
        );
    }
    let mut terminal = Terminal::new(TestBackend::new(60, 28)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    assert!(
        rendered_cells(terminal.backend().buffer(), "keep scope")
            .iter()
            .all(|cell| cell.modifier.contains(Modifier::BOLD))
    );
    let full = text(&mut ui, 240, 80);
    assert!(
        full.contains("Second paragraph retained in recorded detail."),
        "full context must remain available: {full}"
    );
    ui.shutdown();
}

#[test]
fn path_highlighting_keeps_navigation_words_and_surrounding_code_distinct() {
    use ratatui::style::Color;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Conservative path highlights");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"paths","label":"Inspect path cues","status":"active","source":"coordinator",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Use PageUp/PageDown and guidance/queue/stop to inspect docs/plan.md.",
            "next":"Call `read_source(\"src/parser.rs\");` then inspect ./generated and /project/src."
        }),
    );
    let mut ui = home.ui();
    wait_text(&mut ui, "Inspect path cues");
    key(&mut ui, KeyCode::Char('w'));
    let mut terminal = Terminal::new(TestBackend::new(240, 80)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    for words in ["PageUp/PageDown", "guidance/queue/stop"] {
        assert!(
            rendered_cells(buffer, words)
                .iter()
                .all(|cell| cell.fg != Color::Rgb(148, 226, 213)),
            "ordinary slash-separated words must not look like paths: {words}"
        );
    }
    for path in [
        "docs/plan.md",
        "src/parser.rs",
        "./generated",
        "/project/src",
    ] {
        assert!(
            rendered_cells(buffer, path)
                .iter()
                .all(|cell| cell.fg == Color::Rgb(148, 226, 213)),
            "recognizable path needs a focused cue: {path}"
        );
    }
    for syntax in ["read_source(\"", "\");"] {
        assert!(
            rendered_cells(buffer, syntax)
                .iter()
                .all(|cell| cell.fg == Color::Rgb(203, 166, 247)),
            "surrounding code must keep its code style: {syntax}"
        );
    }
    ui.shutdown();
}

#[test]
fn independently_formatted_activity_sections_share_the_pane_display_limit() {
    use std::io::Write;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Bounded activity sections");
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl"))
        .unwrap();
    for payload in [
        serde_json::json!({"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":"Recorded row\n".repeat(512)}]}),
        serde_json::json!({"type":"function_call","name":"exec_command","call_id":"bounded-command","arguments":serde_json::json!({"cmd":"cargo test --must-be-omitted"}).to_string()}),
    ] {
        writeln!(
            file,
            "{}",
            serde_json::json!({"type":"response_item","payload":payload})
        )
        .unwrap();
    }
    drop(file);
    let mut ui = home.ui();
    wait_text(&mut ui, "Bounded activity sections");
    key(&mut ui, KeyCode::Char('x'));
    let shown = text(&mut ui, 240, 600);
    assert!(
        shown.contains("Recorded row"),
        "selected source should be present"
    );
    assert!(
        shown.contains("Display limited; more text remains in the recorded source."),
        "the pane must disclose its shared source-line limit"
    );
    assert!(
        !shown.contains("cargo test --must-be-omitted"),
        "later fragments must not reset the pane budget"
    );
    ui.shutdown();
}

#[test]
fn malformed_work_fields_cannot_absorb_later_fields_provenance_or_controls() {
    use ratatui::style::{Color, Modifier};
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Independent work fields");
    save_work_report(
        &home,
        "goal",
        0,
        serde_json::json!({
            "id":"fields","label":"Scoped task\n```text\nUnclosed task label","status":"active",
            "source":"coordinator\n```text\nUnclosed source field",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "purpose":"Read the code\n```rust\n## Controls\nUnclosed field code",
            "next":"Check the source", "change":"Keep each field independent."
        }),
    );
    let original = serde_json::to_value(
        Store::new(home.0.join("state"))
            .read(&home.key("goal"))
            .unwrap()
            .work_items,
    )
    .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Scoped task");
    key(&mut ui, KeyCode::Char('w'));
    let shown = text(&mut ui, 240, 120);
    assert!(
        shown.contains("unclosed fence"),
        "each incomplete field must remain inspectable: {shown}"
    );
    assert!(
        shown.contains("│ ## Controls"),
        "a source heading inside code must remain literal: {shown}"
    );
    let mut terminal = Terminal::new(TestBackend::new(240, 120)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    let buffer = terminal.backend().buffer();
    for field in [
        "Status: in progress",
        "Next: Check the source",
        "Source: coordinator",
        "Recorded:",
        "Guidance unavailable",
    ] {
        assert!(
            rendered_cells(buffer, field)
                .iter()
                .all(|cell| cell.fg != Color::Rgb(203, 166, 247)),
            "later field must not inherit another value's code style: {field}"
        );
    }
    for heading in ["── Provenance", "── Controls"] {
        assert!(
            rendered_cells(buffer, heading)
                .iter()
                .all(|cell| cell.fg == Color::Rgb(137, 180, 250)
                    && cell.modifier.contains(Modifier::BOLD)),
            "trusted section must keep its own style: {heading}"
        );
    }
    ui.shutdown();
    let after = serde_json::to_value(
        Store::new(home.0.join("state"))
            .read(&home.key("goal"))
            .unwrap()
            .work_items,
    )
    .unwrap();
    assert_eq!(
        after, original,
        "formatting must not rewrite reported fields"
    );
}

#[test]
fn malformed_read_only_panel_values_cannot_absorb_later_rows_or_hints() {
    use ratatui::style::Color;
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Independent read-only panels");
    let store = Store::new(home.0.join("state"));
    let owner =
        serde_json::json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    let bad_path = home.0.join("First document\n```text\nUnclosed document.md");
    let safe_path = home.0.join("second-safe.md");
    for update in [
        serde_json::json!({"op":"set_goal","title":"Independent read-only panels","criteria":[{"id":"build","label":"First criterion\n```text\nUnclosed criterion","weight":1},{"id":"check","label":"Second safe criterion","weight":1}],"reason":"Owned formatting fixture","provenance":"test"}),
        serde_json::json!({"op":"verify","id":"build","revision":2,"evidence":[{"kind":"user_attestation","detail":"Fixture evidence\n```text\nUnclosed evidence","provenance":"owned fixture"}]}),
        serde_json::json!({"op":"capture_question","question":{"id":"question","target":owner,"prompts":[{"id":"choice","header":"Prompt header\n```text\nUnclosed header","question":"Safe question body","options":[{"label":"First option\n```text\nUnclosed option","description":"Option description\n```text\nUnclosed description"},{"label":"Second safe option","description":"Independent option detail"}]}],"state":"expired","answers":{}}}),
        serde_json::json!({"op":"add_document","document":{"id":"first","path":bad_path,"producer":owner,"source":"owned fixture","change":"created"}}),
        serde_json::json!({"op":"add_document","document":{"id":"second","path":safe_path,"producer":owner,"source":"owned fixture","change":"created"}}),
        serde_json::json!({"op":"create_feedback","id":"safe","target":owner,"text":"Later receipt remains readable","mode":"guidance"}),
        serde_json::json!({"op":"create_feedback","id":"bad","target":owner,"text":"```text\nRecorded literal feedback","mode":"guidance"}),
    ] {
        store
            .apply(&home.key("goal"), serde_json::from_value(update).unwrap())
            .unwrap();
    }
    let original = serde_json::to_value(store.read(&home.key("goal")).unwrap()).unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Independent read-only panels");
    for (keycode, later) in [
        (
            'c',
            vec!["Second safe criterion", "Scope revision", "e edit"],
        ),
        (
            'Q',
            vec![
                "Safe question body",
                "Second safe option",
                "Defaults are never submitted",
                "F2 submit answer",
            ],
        ),
        (
            'd',
            vec!["Coordinator · Created", "Opening is always explicit"],
        ),
        ('b', vec!["Later receipt remains readable"]),
    ] {
        key(&mut ui, KeyCode::Esc);
        key(&mut ui, KeyCode::Char(keycode));
        wait_text(
            &mut ui,
            match keycode {
                'c' => "First criterion",
                'Q' => "Prompt header",
                'd' => "First document",
                _ => "Later receipt remains readable",
            },
        );
        let mut terminal = Terminal::new(TestBackend::new(240, 120)).unwrap();
        terminal.draw(|f| ui.draw(f)).unwrap();
        for expected in later {
            assert!(
                rendered_cells(terminal.backend().buffer(), expected)
                    .iter()
                    .all(|cell| cell.fg != Color::Rgb(203, 166, 247)),
                "later row or hint in pane {keycode} must not inherit another field's fence: {expected}"
            );
        }
    }
    ui.shutdown();
    assert_eq!(
        serde_json::to_value(store.read(&home.key("goal")).unwrap()).unwrap(),
        original,
        "read-only formatting must leave all recorded values unchanged"
    );
}

#[test]
fn many_incomplete_link_targets_keep_literal_text_and_a_later_valid_link() {
    use ratatui::style::{Color, Modifier};
    let home = Home::new();
    home.session("goal", "/projects/readability", false);
    home.goal("goal", "Many incomplete targets");
    Store::new(home.0.join("state"))
        .apply(
            &home.key("goal"),
            WorkUpdate::SetGoal {
                title: "Many incomplete targets".into(),
                criteria: vec![CriterionInput {
                    id: "long".into(),
                    label: format!(
                        "{} Read [the spec](docs/(draft)/SPEC.md).",
                        "[x]( ".repeat(1024)
                    ),
                    weight: 1.0,
                }],
                reason: "Owned public rendering regression".into(),
                provenance: "test".into(),
            },
        )
        .unwrap();
    let mut ui = home.ui();
    wait_text(&mut ui, "Selected: Many incomplete targets");
    key(&mut ui, KeyCode::Char('c'));
    let shown = text(&mut ui, 240, 160);
    assert!(
        shown.contains("[x]("),
        "incomplete targets must remain literal"
    );
    assert!(
        shown.contains("the spec (docs/(draft)/SPEC.md)"),
        "a later balanced link must still be recognized: {shown}"
    );
    let mut terminal = Terminal::new(TestBackend::new(240, 160)).unwrap();
    terminal.draw(|f| ui.draw(f)).unwrap();
    assert!(
        rendered_cells(terminal.backend().buffer(), "the spec")
            .iter()
            .all(|cell| cell.modifier.contains(Modifier::UNDERLINED))
    );
    assert!(
        rendered_cells(terminal.backend().buffer(), "docs/(draft)/SPEC.md")
            .iter()
            .all(|cell| cell.fg == Color::Rgb(148, 226, 213))
    );
    ui.shutdown();
}
