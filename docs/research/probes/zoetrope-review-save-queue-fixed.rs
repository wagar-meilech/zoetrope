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
    let home=Home::new(); home.session("current","/projects/review",false); home.goal("current","Review goal");
    let mut ui=home.ui(); wait_text(&mut ui,"Selected: Review goal");
    let lock_path=std::fs::read_dir(home.0.join("state")).unwrap().filter_map(Result::ok).map(|e|e.path()).find(|p|p.extension().is_some_and(|e|e=="lock")).unwrap();
    let lock=std::fs::OpenOptions::new().read(true).write(true).open(lock_path).unwrap();fs2::FileExt::lock_exclusive(&lock).unwrap();
    for at in 0..40 {
        key(&mut ui,KeyCode::Char('f'));
        ui.handle_event(&Event::Paste(if at==39 {"LAST_UNSAVED_SENTINEL".into()}else{format!("draft-{at}")}));
        key(&mut ui,KeyCode::Esc);
    }
    let screen=text(&mut ui,150,42);
    println!("SAVE_BUSY_REPORTED={} LAST_DRAFT_STILL_VISIBLE={}",screen.contains("Save queue busy"),screen.contains("LAST_UNSAVED_SENTINEL"));
    fs2::FileExt::unlock(&lock).unwrap();drop(lock);
    let until=Instant::now()+Duration::from_secs(2);
    while Instant::now()<until {ui.poll();std::thread::sleep(Duration::from_millis(10));}
    ui.shutdown();
    let record=Store::new(home.0.join("state")).read(&home.key("current")).unwrap();
    println!("LAST_DRAFT_PERSISTED={}",record.drafts.iter().any(|d|d.text.contains("LAST_UNSAVED_SENTINEL")));
}
