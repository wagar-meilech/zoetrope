use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use zoetrope::console::DiscoveryRoots;
use zoetrope::fleet::{FleetConfig, FleetConsole};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args_os().nth(1).ok_or("fixture root")?);
    let mut ui = FleetConsole::new(FleetConfig {
        roots: DiscoveryRoots {
            claude_projects: None,
            codex_home: Some(root.join("codex")),
        },
        state_root: root.join("state"),
        socket: root.join("unavailable.sock"),
        ..FleetConfig::default()
    });
    let until = Instant::now() + Duration::from_secs(1);
    while Instant::now() < until {
        ui.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
    for (name, width, height, focus) in [
        ("default", 150u16, 42u16, false),
        ("work", 150, 42, true),
        ("narrow work", 60, 28, true),
    ] {
        if focus {
            ui.handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Char('w'),
                KeyModifiers::NONE,
            )));
        }
        ui.poll();
        let mut terminal = Terminal::new(TestBackend::new(width, height))?;
        terminal.draw(|frame| ui.draw(frame))?;
        terminal.draw(|frame| ui.draw(frame))?;
        println!("VIEW {name} {width}x{height}");
        let buffer = terminal.backend().buffer();
        for row in 0..height {
            let line = (0..width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>();
            println!("{}", line.trim_end());
        }
    }
    ui.shutdown();
    Ok(())
}
