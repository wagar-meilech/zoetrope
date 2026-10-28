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
    use zoetrope::console::AgentKey;use zoetrope::workstate::{QuestionRecord,QuestionPrompt,QuestionState};
    let home=Home::new();home.session("current","/projects/review",false);home.goal("current","Review goal");let store=Store::new(home.0.join("state"));
    store.apply(&home.key("current"),WorkUpdate::CaptureQuestion{question:QuestionRecord{id:"copy-race".into(),target:AgentKey{session:home.key("current"),agent_id:"main".into()},request_id:None,turn_id:None,prompts:vec![QuestionPrompt{id:"pick".into(),header:"Question".into(),question:"Earlier question".into(),options:vec![],is_other:true,is_secret:false}],state:QuestionState::Expired,answers:std::collections::BTreeMap::from([("pick".into(),vec!["EARLIER_COPIED_ANSWER".into()])])}}).unwrap();
    let mut ui=home.ui();wait_text(&mut ui,"Selected: Review goal");key(&mut ui,KeyCode::Char('Q'));
    let lock_path=std::fs::read_dir(home.0.join("state")).unwrap().filter_map(Result::ok).map(|e|e.path()).find(|p|p.extension().is_some_and(|e|e=="lock")).unwrap();let lock=std::fs::OpenOptions::new().read(true).write(true).open(lock_path).unwrap();fs2::FileExt::lock_exclusive(&lock).unwrap();
    key(&mut ui,KeyCode::F(5));key(&mut ui,KeyCode::Char('f'));ui.handle_event(&Event::Paste("KEEP_NEWER_DRAFT".into()));
    println!("BEFORE_COPY_ACK_NEW_VISIBLE={}",text(&mut ui,150,42).contains("KEEP_NEWER_DRAFT"));fs2::FileExt::unlock(&lock).unwrap();drop(lock);
    let until=Instant::now()+Duration::from_secs(2);while Instant::now()<until{ui.poll();std::thread::sleep(Duration::from_millis(10));}
    let rendered=text(&mut ui,150,42);println!("AFTER_COPY_ACK_NEW_VISIBLE={} OLD_COPY_VISIBLE={}",rendered.contains("KEEP_NEWER_DRAFT"),rendered.contains("EARLIER_COPIED_ANSWER"));ui.shutdown();let record=store.read(&home.key("current")).unwrap();println!("AFTER_COPY_ACK_NEW_PERSISTED={}",record.drafts.iter().any(|d|d.text.contains("KEEP_NEWER_DRAFT")));
}
