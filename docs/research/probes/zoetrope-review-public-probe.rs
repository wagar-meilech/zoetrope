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

fn main() {
    use zoetrope::console::AgentKey;
    use zoetrope::workstate::{QuestionRecord,QuestionPrompt,QuestionState};
    let home=Home::new(); home.session("current","/projects/review",false); home.goal("current","Review goal");
    let store=Store::new(home.0.join("state"));
    store.apply(&home.key("current"),WorkUpdate::CaptureQuestion{question:QuestionRecord{id:"review-secret".into(),target:AgentKey{session:home.key("current"),agent_id:"main".into()},request_id:None,turn_id:None,prompts:vec![QuestionPrompt{id:"secret".into(),header:"Credential".into(),question:"Enter protected value".into(),options:vec![],is_other:true,is_secret:true}],state:QuestionState::Expired,answers:std::collections::BTreeMap::from([("secret".into(),vec!["REVIEW_SECRET_abc123".into()])])}}).unwrap();
    let mut ui=home.ui(); wait_text(&mut ui,"0 of 2 verified");
    store.apply(&home.key("current"),WorkUpdate::Verify{id:"build".into(),evidence:vec![Evidence{kind:EvidenceKind::UserAttestation,detail:"Verified outside UI".into(),provenance:"Review probe".into()}],revision:Some(1)}).unwrap();
    let until=Instant::now()+Duration::from_secs(7);
    while Instant::now()<until { ui.poll();std::thread::sleep(Duration::from_millis(10)); }
    let after=text(&mut ui,150,42);
    println!("AFTER_EXTERNAL_UPDATE old_zero_visible={} new_one_visible={}",after.contains("0 of 2 verified"),after.contains("1 of 2 verified"));
    key(&mut ui,KeyCode::Char('Q')); key(&mut ui,KeyCode::Char('e'));
    let secret=text(&mut ui,150,42); println!("SECRET_EDITOR_EXPOSED={}",secret.contains("REVIEW_SECRET_abc123"));
    key(&mut ui,KeyCode::Esc); wait_text(&mut ui,"Answer draft saved"); key(&mut ui,KeyCode::F(3));
    let copied=wait_text(&mut ui,"draft only until you send"); println!("SECRET_COPY_EXPOSED={}",copied.contains("REVIEW_SECRET_abc123"));
    ui.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('u'),KeyModifiers::CONTROL)));
    let large=format!("{} DO_NOT_TOUCH_DATABASE", "X".repeat(65536));
    ui.handle_event(&Event::Paste(large));
    let until=Instant::now()+Duration::from_secs(1);
    while Instant::now()<until { ui.poll();std::thread::sleep(Duration::from_millis(10)); }
    let record=store.read(&home.key("current")).unwrap();let draft=&record.drafts.iter().find(|d|d.target.agent_id=="main").unwrap().text;
    println!("PASTE_SAVED_BYTES={} TAIL_CONSTRAINT_RETAINED={} LIMIT_MESSAGE_VISIBLE={}",draft.len(),draft.contains("DO_NOT_TOUCH_DATABASE"),text(&mut ui,150,42).to_lowercase().contains("limit"));
    ui.shutdown();
}
