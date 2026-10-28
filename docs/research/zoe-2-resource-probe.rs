use std::path::PathBuf;
use std::time::{Instant,SystemTime};
use zoetrope::console::{Console,ConsoleCommand,DiscoveryRoots,SessionKey};
use zoetrope::provider::Provider;
fn main() {
    let root=PathBuf::from(std::env::args_os().nth(1).unwrap());
    let mut console=Console::with_state(DiscoveryRoots{claude_projects:None,codex_home:Some(root.join("codex"))},root.join("state"));
    let cold=Instant::now();console.refresh(SystemTime::now());println!("cold_ms={}",cold.elapsed().as_millis());
    console.command(ConsoleCommand::ToggleHistory);
    let unchanged=Instant::now(); for _ in 0..20 {console.refresh(SystemTime::now());} println!("unchanged_mean_ms={:.3}",unchanged.elapsed().as_secs_f64()*1000.0/20.0);
    let search=Instant::now(); console.command(ConsoleCommand::Search("unique-early-document".into()));println!("uncached_artifact_search_ms={} visible_nodes={}",search.elapsed().as_millis(),console.graph().nodes.len());
    let key=SessionKey{provider:Provider::Codex,session_id:"goal-000".into()};
    let record=console.load_work(&key).unwrap(); assert_eq!(record.scope_history.len(),8); assert_eq!(record.documents.len(),1);
    println!("selected_history_restored={} progress={}",record.scope_history.len(),record.progress().label);
    let graph=Instant::now(); for _ in 0..100000 {std::hint::black_box(console.graph());} println!("cached_100k_graph_reads_ms={:.3}",graph.elapsed().as_secs_f64()*1000.0);
}
