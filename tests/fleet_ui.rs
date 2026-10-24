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
fn type_text(ui:&mut FleetConsole,value:&str){for ch in value.chars(){key(ui,KeyCode::Char(ch));}}
#[test]
fn a_searched_goals_feedback_survives_typing_shortcuts_navigation_and_restart() {
 let home=Home::new();home.session("current","/projects/compiler",false);home.goal("current","Improve compiler");home.session("past","/projects/editor",true);home.goal("past","Restore editor");
 let mut ui=home.ui();wait_text(&mut ui,"Improve compiler");key(&mut ui,KeyCode::Char('H'));key(&mut ui,KeyCode::Char('/'));type_text(&mut ui,"Restore editor");key(&mut ui,KeyCode::Enter);wait_text(&mut ui,"Search: Restore editor");
 key(&mut ui,KeyCode::Down);wait_text(&mut ui,"Selected: Restore editor");key(&mut ui,KeyCode::Char('f'));type_text(&mut ui,"H q: please preserve draft");key(&mut ui,KeyCode::Esc);let saved=wait_text(&mut ui,"Draft saved");assert!(saved.contains("All history"),"typing H must not toggle history: {saved}");ui.shutdown();
 let mut restarted=home.ui();wait_text(&mut restarted,"Improve compiler");key(&mut restarted,KeyCode::Char('H'));key(&mut restarted,KeyCode::Char('/'));type_text(&mut restarted,"Restore editor");key(&mut restarted,KeyCode::Enter);wait_text(&mut restarted,"Search: Restore editor");key(&mut restarted,KeyCode::Down);wait_text(&mut restarted,"Selected: Restore editor");key(&mut restarted,KeyCode::Char('f'));wait_text(&mut restarted,"H q: please preserve draft");restarted.shutdown();
}
fn ctrl(ui:&mut FleetConsole,ch:char){ui.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char(ch),KeyModifiers::CONTROL)));}
#[test]
fn goal_criteria_can_be_edited_verified_with_attestation_and_reopened() {
 let home=Home::new();home.session("goal","/projects/compiler",false);home.goal("goal","Compiler work");let mut ui=home.ui();wait_text(&mut ui,"Selected: Compiler work");
 key(&mut ui,KeyCode::Char('c'));key(&mut ui,KeyCode::Char('e'));ctrl(&mut ui,'u');type_text(&mut ui,"Safer compiler");key(&mut ui,KeyCode::Enter);type_text(&mut ui,"Parses source");key(&mut ui,KeyCode::Enter);type_text(&mut ui,"Regression tests pass");key(&mut ui,KeyCode::F(2));wait_text(&mut ui,"Selected: Safer compiler");
 key(&mut ui,KeyCode::Char('v'));type_text(&mut ui,"I checked the parser with a real input");key(&mut ui,KeyCode::F(2));let verified=wait_text(&mut ui,"1 of 2 verified");assert!(verified.contains("I checked the parser"),"{verified}");
 key(&mut ui,KeyCode::Char('u'));let reopened=wait_text(&mut ui,"0 of 2 verified");assert!(reopened.contains("Prior verification retained"),"{reopened}");ui.shutdown();
}
#[test]
fn an_incomplete_heavily_weighted_goal_never_claims_one_hundred_percent() {
 let home=Home::new();home.session("goal","/projects/compiler",false);let store=Store::new(home.0.join("state"));store.apply(&home.key("goal"),WorkUpdate::SetGoal{title:"Release compiler".into(),criteria:vec![CriterionInput{id:"build".into(),label:"Build".into(),weight:999.0},CriterionInput{id:"check".into(),label:"Final check".into(),weight:1.0}],reason:"scope".into(),provenance:"operator".into()}).unwrap();store.apply(&home.key("goal"),WorkUpdate::Verify{id:"build".into(),evidence:vec![Evidence{kind:EvidenceKind::UserAttestation,detail:"Build passed".into(),provenance:"operator".into()}],revision:Some(1)}).unwrap();let mut ui=home.ui();let view=wait_text(&mut ui,"1 of 2 verified");assert!(!view.contains("· 100%"),"{view}");assert!(view.contains("<100%"),"{view}");ui.shutdown();
}
#[cfg(unix)]
struct Peer {dir:PathBuf,child:std::process::Child}
#[cfg(unix)]
impl Peer {
 fn new(case:&str)->Self{let nonce=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();let dir=PathBuf::from("/tmp").join(format!("zui-{}-{nonce}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();let child=std::process::Command::new("python3").arg(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/codex_fleet_peer.py")).arg(dir.join("peer.sock")).arg(case).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).spawn().unwrap();let deadline=Instant::now()+Duration::from_secs(3);while !dir.join("peer.sock.ready").exists(){assert!(Instant::now()<deadline,"peer failed to start");std::thread::sleep(Duration::from_millis(10));}Self{dir,child}}
 fn finish(mut self){let status=self.child.wait().unwrap();use std::io::Read;let mut err=String::new();self.child.stderr.take().unwrap().read_to_string(&mut err).unwrap();assert!(status.success(),"peer failed: {err}");}
}
#[cfg(unix)]
impl Drop for Peer {fn drop(&mut self){let _=self.child.kill();let _=self.child.wait();let _=std::fs::remove_dir_all(&self.dir);}}
#[cfg(unix)]
#[test]
fn guidance_queue_and_stop_reach_the_selected_child_without_parent_routing(){
 let peer=Peer::new("control");let home=Home::new();home.session("root-goal","/projects/parser",false);home.goal("root-goal","Parser goal");let path=home.0.join("codex/sessions/2026/10/23/rollout-child.jsonl");std::fs::write(path,"{\"type\":\"session_meta\",\"payload\":{\"id\":\"child-thread\",\"session_id\":\"root-goal\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root-goal\",\"agent_nickname\":\"Parser\"}}\n").unwrap();
 let mut ui=FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(home.0.join("codex")),claude_projects:None},state_root:home.0.join("state"),socket:peer.dir.join("peer.sock"),..FleetConfig::default()});wait_text(&mut ui,"Selected: Parser goal");key(&mut ui,KeyCode::Enter);wait_text(&mut ui,"Coordinator");key(&mut ui,KeyCode::Down);wait_text(&mut ui,"Recipient: Coordinator");key(&mut ui,KeyCode::Down);wait_text(&mut ui,"Recipient: Parser");wait_text(&mut ui,"Working");key(&mut ui,KeyCode::Char('f'));type_text(&mut ui,"Keep the public interface");key(&mut ui,KeyCode::F(2));wait_text(&mut ui,"Accepted by Codex");ctrl(&mut ui,'u');type_text(&mut ui,"Do docs later");key(&mut ui,KeyCode::F(3));wait_text(&mut ui,"Queued by Codex");key(&mut ui,KeyCode::F(4));wait_text(&mut ui,"accepted the interruption");ui.shutdown();peer.finish();
}
#[test]
fn an_expired_question_keeps_its_answer_draft_and_can_be_copied_to_feedback(){
 use zoetrope::console::AgentKey;use zoetrope::workstate::{QuestionRecord,QuestionPrompt,QuestionState};
 let home=Home::new();home.session("goal","/projects/compiler",false);home.goal("goal","Compiler questions");let target=AgentKey{session:home.key("goal"),agent_id:"main".into()};Store::new(home.0.join("state")).apply(&target.session,WorkUpdate::CaptureQuestion{question:QuestionRecord{id:"earlier".into(),target:target.clone(),request_id:None,turn_id:None,prompts:vec![QuestionPrompt{id:"choice".into(),header:"Color".into(),question:"Which color should the output use?".into(),options:vec![],is_other:true,is_secret:false}],state:QuestionState::Expired,answers:Default::default()}}).unwrap();
 let mut ui=home.ui();wait_text(&mut ui,"Selected: Compiler questions");key(&mut ui,KeyCode::Char('Q'));wait_text(&mut ui,"Original request expired");key(&mut ui,KeyCode::Char('e'));type_text(&mut ui,"Use blue with readable contrast");key(&mut ui,KeyCode::Esc);wait_text(&mut ui,"Answer draft saved");ui.shutdown();
 let mut ui=home.ui();wait_text(&mut ui,"Selected: Compiler questions");key(&mut ui,KeyCode::Char('Q'));wait_text(&mut ui,"Use blue with readable contrast");key(&mut ui,KeyCode::F(3));let copied=wait_text(&mut ui,"Answer to an earlier question");assert!(copied.contains("F2 Send guidance"));assert!(!copied.contains("Accepted by Codex"));ui.shutdown();
}
#[cfg(unix)]
#[test]
fn native_questions_require_a_fresh_join_and_explicit_choice_before_submission(){
 let peer=Peer::new("question");let home=Home::new();home.session("root-goal","/projects/parser",false);home.goal("root-goal","Parser questions");let mut ui=FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(home.0.join("codex")),claude_projects:None},state_root:home.0.join("state"),socket:peer.dir.join("peer.sock"),..FleetConfig::default()});wait_text(&mut ui,"Selected: Parser questions");key(&mut ui,KeyCode::Char('Q'));key(&mut ui,KeyCode::Char('L'));let question=wait_text(&mut ui,"Choose a label");assert!(question.contains("No answer chosen"),"{question}");key(&mut ui,KeyCode::Char('2'));wait_text(&mut ui,"Answer draft: Beta");key(&mut ui,KeyCode::F(2));let resolved=wait_text(&mut ui,"Resolved in Codex");assert!(resolved.contains("Answer draft: Beta"));assert!(resolved.contains("0 of 2 verified")||!resolved.contains("100%"));ui.shutdown();peer.finish();
}
#[cfg(unix)]
#[test]
fn quiet_waiting_runtime_work_is_current_without_resuming_history(){
 let peer=Peer::new("observe");let home=Home::new();home.session("root-goal","/projects/parser",false);home.goal("root-goal","Quiet compiler");let path=home.0.join("codex/sessions/2026/10/23/rollout-root-goal.jsonl");std::fs::File::options().write(true).open(path).unwrap().set_times(std::fs::FileTimes::new().set_modified(UNIX_EPOCH+Duration::from_secs(100))).unwrap();let mut ui=FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(home.0.join("codex")),claude_projects:None},state_root:home.0.join("state"),socket:peer.dir.join("peer.sock"),..FleetConfig::default()});let shown=wait_text(&mut ui,"Selected: Quiet compiler");assert!(shown.contains("Current work"));wait_text(&mut ui,"Waiting for your answer");ui.shutdown();peer.finish();
}
#[cfg(unix)]
#[test]
fn documents_open_only_the_explicitly_selected_exact_path(){
 use zoetrope::console::AgentKey;use zoetrope::workstate::{DocumentRecord,DocumentChange};use std::os::unix::fs::PermissionsExt;
 let home=Home::new();home.session("goal","/projects/compiler",false);home.goal("goal","Compiler documents");let producer=AgentKey{session:home.key("goal"),agent_id:"main".into()};let first=home.0.join("first.md");let selected=home.0.join("Review ü #+%2F &.md");std::fs::write(&first,"first").unwrap();std::fs::write(&selected,"selected").unwrap();let store=Store::new(home.0.join("state"));for(at,path)in [&first,&selected].iter().enumerate(){store.apply(&producer.session,WorkUpdate::AddDocument{document:DocumentRecord{id:format!("doc-{at}"),path:(*path).clone(),producer:producer.clone(),source:"recorded file change".into(),change:DocumentChange::Created,changed_at:None}}).unwrap();}
 let code=home.0.join("code.py");let opener=home.0.join("open.py");let trace=home.0.join("opened.txt");std::fs::write(&code,"#!/usr/bin/env python3\nprint('wagar.zoetrope-preview')\n").unwrap();std::fs::write(&opener,format!("#!/usr/bin/env python3\nimport sys,pathlib,urllib.parse\nu=urllib.parse.urlsplit(sys.argv[1])\nf=urllib.parse.parse_qs(u.query)['uri'][0]\npathlib.Path({:?}).write_text(urllib.parse.unquote(urllib.parse.urlsplit(f).path))\n",trace.to_string_lossy())).unwrap();for path in [&code,&opener]{std::fs::set_permissions(path,std::fs::Permissions::from_mode(0o755)).unwrap();}
 let mut ui=FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(home.0.join("codex")),claude_projects:None},state_root:home.0.join("state"),socket:home.0.join("none.sock"),documents:zoetrope::documents::DocumentLauncher::new(code,opener)});wait_text(&mut ui,"Selected: Compiler documents");key(&mut ui,KeyCode::Char('d'));let listed=wait_text(&mut ui,"Review ü #+%2F &.md");assert!(listed.contains("Coordinator"));assert!(!trace.exists(),"merely listing artifacts must not open an editor");key(&mut ui,KeyCode::Down);key(&mut ui,KeyCode::Enter);wait_text(&mut ui,"Requested VS Code preview");assert_eq!(std::fs::read_to_string(trace).unwrap(),selected.to_string_lossy());ui.shutdown();
}
#[test]
fn recorded_async_questions_are_retained_without_inventing_native_pending_ids(){
 let home=Home::new();home.session("goal","/projects/compiler",false);home.goal("goal","Async questions");let path=home.0.join("codex/sessions/2026/10/23/rollout-goal.jsonl");use std::io::Write;let mut file=std::fs::OpenOptions::new().append(true).open(path).unwrap();writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"request_user_input_async","call_id":"async-call","arguments":serde_json::json!({"questions":[{"title":"Which implementation should we use?","options":["Use Rust","Use TypeScript"]}]}).to_string()}})).unwrap();writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"functions.exec","call_id":"code-call","arguments":serde_json::json!({"code":"request_user_input_async({title:'Fake question'})"}).to_string()}})).unwrap();drop(file);
 let mut ui=home.ui();wait_text(&mut ui,"Selected: Async questions");key(&mut ui,KeyCode::Char('Q'));let question=wait_text(&mut ui,"Which implementation should we use?");assert!(question.contains("Original request expired"));assert!(!question.contains("Fake question"));key(&mut ui,KeyCode::Char('2'));wait_text(&mut ui,"Answer draft: Use TypeScript");ui.shutdown();let mut ui=home.ui();wait_text(&mut ui,"Selected: Async questions");key(&mut ui,KeyCode::Char('Q'));let restored=wait_text(&mut ui,"Answer draft: Use TypeScript");assert!(restored.contains("Questions · 1 of 1"));ui.shutdown();
}
#[test]
fn public_commentary_and_shell_purpose_are_readable_without_private_reasoning(){
 let home=Home::new();home.session("raw-thread-id","/projects/compiler",false);let path=home.0.join("codex/sessions/2026/10/23/rollout-raw-thread-id.jsonl");use std::io::Write;let mut file=std::fs::OpenOptions::new().append(true).open(path).unwrap();for record in [serde_json::json!({"type":"event_msg","payload":{"type":"item_completed","item":{"type":"UserMessage","content":[{"type":"text","text":"Fix compiler diagnostics"}]}}}),serde_json::json!({"type":"response_item","payload":{"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":"Checking compiler diagnostics"}]}}),serde_json::json!({"type":"response_item","payload":{"type":"message","role":"assistant","channel":"analysis","content":[{"type":"output_text","text":"PRIVATE ANALYSIS PLACEHOLDER"}]}}),serde_json::json!({"type":"event_msg","payload":{"type":"item_completed","item":{"type":"Reasoning","summary_text":["PRIVATE SUMMARY PLACEHOLDER"]}}}),serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"functions.exec_command","call_id":"bash","arguments":serde_json::json!({"cmd":"cargo test --locked","justification":"Check parser behavior"}).to_string()}}),serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"read_file","call_id":"read","arguments":"{\"path\":\"src/main.rs\"}"}})]{writeln!(file,"{record}").unwrap();}drop(file);
 let mut ui=home.ui();let shown=wait_text(&mut ui,"Selected: Fix compiler diagnostics");assert!(shown.contains("Checking compiler diagnostics"),"{shown}");assert!(shown.contains("cargo test --locked"),"{shown}");assert!(shown.contains("Check parser behavior"),"{shown}");assert!(!shown.contains("PRIVATE"));assert!(!shown.contains("functions.exec_command"));assert!(!shown.contains("read_file"));assert!(!shown.contains("raw-thread-id"));ui.shutdown();
}
#[cfg(unix)]
#[test]
fn transcript_and_live_question_share_one_draft_and_resolved_answers_can_be_feedback(){
 let peer=Peer::new("question");let home=Home::new();home.session("root-goal","/projects/parser",false);home.goal("root-goal","One question source");let path=home.0.join("codex/sessions/2026/10/23/rollout-root-goal.jsonl");use std::io::Write;let mut file=std::fs::OpenOptions::new().append(true).open(&path).unwrap();writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call","name":"request_user_input","call_id":"native-question-call","arguments":serde_json::json!({"questions":[{"id":"choice","header":"Label","question":"Choose a label","options":[{"label":"Alpha","description":"First choice"},{"label":"Beta","description":"Second choice"}]}]}).to_string()}})).unwrap();drop(file);
 let mut ui=FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(home.0.join("codex")),claude_projects:None},state_root:home.0.join("state"),socket:peer.dir.join("peer.sock"),..FleetConfig::default()});wait_text(&mut ui,"Selected: One question source");key(&mut ui,KeyCode::Char('Q'));wait_text(&mut ui,"Original request expired");key(&mut ui,KeyCode::Char('2'));wait_text(&mut ui,"Answer draft: Beta");key(&mut ui,KeyCode::Char('L'));let joined=wait_text(&mut ui,"Pending in the current Codex connection");assert!(joined.contains("Questions · 1 of 1"),"{joined}");assert!(joined.contains("Answer draft: Beta"),"{joined}");key(&mut ui,KeyCode::F(2));wait_text(&mut ui,"Resolved in Codex");key(&mut ui,KeyCode::F(3));wait_text(&mut ui,"Answer to an earlier question");ui.shutdown();peer.finish();
 let mut file=std::fs::OpenOptions::new().append(true).open(&path).unwrap();writeln!(file,"{}",serde_json::json!({"type":"response_item","payload":{"type":"function_call_output","call_id":"native-question-call","output":serde_json::json!({"answers":{"choice":{"answers":["Beta"]}}}).to_string()}})).unwrap();drop(file);std::fs::File::options().write(true).open(&path).unwrap().set_times(std::fs::FileTimes::new().set_modified(UNIX_EPOCH+Duration::from_secs(100))).unwrap();
 let mut history=FleetConsole::new(FleetConfig{roots:DiscoveryRoots{codex_home:Some(home.0.join("codex")),claude_projects:None},state_root:home.0.join("fresh-state"),socket:home.0.join("none.sock"),..FleetConfig::default()});wait_text(&mut history,"No current work");key(&mut history,KeyCode::Char('H'));wait_text(&mut history,"Selected: Untitled goal");key(&mut history,KeyCode::Char('Q'));wait_text(&mut history,"Resolved in Codex");history.shutdown();
}
