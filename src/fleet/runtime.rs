//! Native terminal ownership and event-driven fleet execution.

use std::io::stdout;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyEventKind};
use crossterm::execute;
use futures::StreamExt;

use super::{FleetConfig, FleetConsole};

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        let _ = execute!(stdout(), DisableMouseCapture);
        ratatui::restore();
    }
}

/// Run the native console. A single owned input stream serves this terminal.
pub async fn run(config: FleetConfig) -> anyhow::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let restore = RestoreTerminal;
    execute!(stdout(), EnableMouseCapture)?;
    crate::tui::install_panic_hook();

    let mut app = FleetConsole::new(config);
    let notify = app.notify.clone();
    let mut events = EventStream::new();
    let mut dirty = true;
    let result = loop {
        dirty |= app.poll();
        if dirty {
            if let Err(error) = terminal.draw(|frame| app.draw(frame)) {
                break Err(error.into());
            }
            dirty = false;
        }
        tokio::select! {
            _ = notify.notified() => {},
            event = events.next() => match event {
                Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Release => {},
                Some(Ok(event)) => {
                    if app.handle_event(&event) { break Ok(()); }
                    dirty = true;
                },
                Some(Err(error)) => break Err(error.into()),
                None => break Ok(()),
            },
        }
    };
    drop(events);
    drop(restore);
    tokio::task::spawn_blocking(move || app.shutdown()).await?;
    result
}
