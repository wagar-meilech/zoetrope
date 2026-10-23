#![cfg(feature = "native")]
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use zoetrope::console::{DiscoveryRoots, SessionKey};
use zoetrope::fleet::{FleetConfig, FleetConsole};
use zoetrope::provider::Provider;
use zoetrope::workstate::{CriterionInput, Evidence, EvidenceKind, Store, WorkUpdate};

struct Home(PathBuf);
impl Home {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n=NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
        let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        Self(std::env::temp_dir().join(format!("zoe-ui-{}-{now}-{n}",std::process::id())))
    }
    fn session(&self,id:&str,project:&str,archived:bool) {
        let sub=if archived {"archived_sessions"} else {"sessions/2026/10/23"};
        let path=self.0.join(format!("codex/{sub}/rollout-{id}.jsonl"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path,format!("{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"session_id\":\"{id}\",\"cwd\":\"{project}\",\"thread_source\":\"user\"}}}}\n")).unwrap();
    }
    fn key(&self,id:&str)->SessionKey { SessionKey{provider:Provider::Codex,session_id:id.into()} }
    fn goal(&self,id:&str,title:&str) {
        Store::new(self.0.join("state")).apply(&self.key(id),WorkUpdate::SetGoal{title:title.into(),criteria:vec![CriterionInput{id:"build".into(),label:"Build passes".into(),weight:1.0},CriterionInput{id:"check".into(),label:"Behavior checked".into(),weight:1.0}],reason:"Initial scope".into(),provenance:"Operator".into()}).unwrap();
    }
    fn ui(&self)->FleetConsole {
        FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(self.0.join("codex")),claude_projects:None},state_root:self.0.join("state"),socket:self.0.join("no-runtime.sock"),..FleetConfig::default()})
    }
}
impl Drop for Home {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
fn key(ui:&mut FleetConsole,code:KeyCode){ui.handle_event(&Event::Key(KeyEvent::new(code,KeyModifiers::NONE)));}
fn text(ui:&mut FleetConsole,w:u16,h:u16)->String {
 let mut terminal=Terminal::new(TestBackend::new(w,h)).unwrap(); terminal.draw(|f|ui.draw(f)).unwrap();
 terminal.backend().buffer().content().chunks(w as usize).map(|row|row.iter().map(|c|c.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}
fn wait_text(ui:&mut FleetConsole,needle:&str)->String {
 let until=Instant::now()+Duration::from_secs(8); loop {ui.poll(); let rendered=text(ui,150,42);if rendered.contains(needle){return rendered;} assert!(Instant::now()<until,"did not show {needle}: {rendered}");std::thread::sleep(Duration::from_millis(10));}
}
#[test]
fn opening_fleet_shows_goal_progress_and_history_uses_the_same_graph() {
 let home=Home::new();home.session("current","/projects/compiler",false);home.goal("current","Improve compiler");home.session("past","/projects/editor",true);home.goal("past","Restore editor");
 Store::new(home.0.join("state")).apply(&home.key("current"),WorkUpdate::Verify{id:"build".into(),evidence:vec![Evidence{kind:EvidenceKind::UserAttestation,detail:"Release build passed".into(),provenance:"Operator".into()}],revision:Some(1)}).unwrap();
 let mut ui=home.ui();let current=wait_text(&mut ui,"Improve compiler");assert!(current.contains("1 of 2 verified"),"{current}");assert!(!current.contains("Restore editor"));
 key(&mut ui,KeyCode::Char('H'));let history=wait_text(&mut ui,"Restore editor");assert!(history.contains("All history"));assert!(history.contains("Improve compiler"));ui.shutdown();
}
