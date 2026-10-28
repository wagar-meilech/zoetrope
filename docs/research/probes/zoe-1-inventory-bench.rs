use std::{fs::{self,File}, io::{Seek,SeekFrom,Write}, time::{Instant,SystemTime}};
use zoetrope::console::{Console, ConsoleCommand, DiscoveryRoots};
fn main() {
    let home = std::env::temp_dir().join(format!("zoe-1-inventory-bench-{}", std::process::id()));
    let archived = home.join("archived_sessions");
    fs::create_dir_all(&archived).unwrap();
    let count = 1000;
    for n in 0..count {
        let mut file = File::create(archived.join(format!("rollout-history-{n}.jsonl"))).unwrap();
        writeln!(file, "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"goal-{n}\",\"session_id\":\"goal-{n}\",\"cwd\":\"/projects/benchmark\",\"thread_source\":\"user\",\"source\":\"cli\"}}}}").unwrap();
        file.set_len(16 * 1024 * 1024).unwrap();
        file.seek(SeekFrom::End(-1)).unwrap();
        file.write_all(b"\n").unwrap();
    }
    let mut console = Console::new(DiscoveryRoots { claude_projects: None, codex_home: Some(home.clone()) });
    let start = Instant::now();
    console.refresh(SystemTime::now());
    let initial = start.elapsed();
    console.command(ConsoleCommand::ToggleHistory);
    let start = Instant::now();
    for _ in 0..20 { console.refresh(SystemTime::now()); }
    let refresh = start.elapsed() / 20;
    let start = Instant::now();
    for _ in 0..100_000 { std::hint::black_box(console.graph()); }
    let graph = start.elapsed();
    println!("sessions={count}; logical_history_bytes={}; initial_refresh_ms={:.3}; unchanged_refresh_mean_ms={:.3}; borrowed_graph_100000_reads_ms={:.3}; visible_nodes={}", count as u64 *16*1024*1024, initial.as_secs_f64()*1000., refresh.as_secs_f64()*1000., graph.as_secs_f64()*1000., console.graph().nodes.len());
    fs::remove_dir_all(home).unwrap();
}
