use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::time::{Duration, Instant};
use zoetrope::console::{DiscoveryRoots, SessionKey};
use zoetrope::fleet::{FleetConfig, FleetConsole};
use zoetrope::provider::Provider;
use zoetrope::workstate::{CriterionInput, Store, WorkUpdate};
fn main() {
    for count in [1024usize, 2048, 4096, 8192] {
        for (name, unit) in [("control", "ax]("), ("unclosed-targets", "[x](")] {
            let root = std::env::temp_dir().join(format!(
                "zoe-link-cost-{}-{count}-{name}",
                std::process::id()
            ));
            let sessions = root.join("codex/sessions/2026/10/28");
            std::fs::create_dir_all(&sessions).unwrap();
            std::fs::write(sessions.join("rollout-goal.jsonl"),"{\"type\":\"session_meta\",\"payload\":{\"id\":\"goal\",\"session_id\":\"goal\",\"cwd\":\"/disposable/link-cost\",\"thread_source\":\"user\"}}\n").unwrap();
            let key = SessionKey {
                provider: Provider::Codex,
                session_id: "goal".into(),
            };
            Store::new(root.join("state"))
                .apply(
                    &key,
                    WorkUpdate::SetGoal {
                        title: "Link cost".into(),
                        criteria: vec![CriterionInput {
                            id: "c".into(),
                            label: unit.repeat(count),
                            weight: 1.0,
                        }],
                        reason: "Disposable public render probe".into(),
                        provenance: "review probe".into(),
                    },
                )
                .unwrap();
            let mut ui = FleetConsole::new(FleetConfig {
                roots: DiscoveryRoots {
                    codex_home: Some(root.join("codex")),
                    claude_projects: None,
                },
                state_root: root.join("state"),
                socket: root.join("no-runtime.sock"),
                ..FleetConfig::default()
            });
            let mut terminal = Terminal::new(TestBackend::new(150, 42)).unwrap();
            let until = Instant::now() + Duration::from_secs(5);
            loop {
                ui.poll();
                terminal.draw(|f| ui.draw(f)).unwrap();
                let shown: String = terminal
                    .backend()
                    .buffer()
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect();
                if shown.contains("Selected: Link cost") {
                    break;
                }
                assert!(Instant::now() < until);
                std::thread::sleep(Duration::from_millis(5));
            }
            ui.handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::NONE,
            )));
            let start = Instant::now();
            terminal.draw(|f| ui.draw(f)).unwrap();
            println!(
                "kind={name} source_bytes={} draw_ms={:.3}",
                count * unit.len(),
                start.elapsed().as_secs_f64() * 1000.0
            );
            ui.shutdown();
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
