//! Native terminal ownership and event-driven fleet execution.

use std::io::stdout;

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use futures::StreamExt;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{Duration, Instant},
};

use super::{FleetConfig, FleetConsole};
use crate::{
    console::SessionKey,
    provider::Target,
    state::{App, Mode, Transport},
    tailer::{TailRequest, UiEvent},
};

struct Replay {
    app: App,
    watch: mpsc::Sender<TailRequest>,
    events: mpsc::Receiver<UiEvent>,
    feeder: JoinHandle<()>,
    frame_at: Instant,
}

impl Replay {
    fn open((key, path): (SessionKey, std::path::PathBuf)) -> Self {
        let (watch, requests) = mpsc::channel(32);
        let (updates, events) = mpsc::channel(32);
        // A new bounded channel has capacity; watching performs no IO here.
        let _ = watch.try_send(TailRequest::Watch(Target::Path(path)));
        let feeder = tokio::spawn(async move {
            if let Err(error) =
                crate::tailer::run(requests, updates.clone(), true, 8.0, Some(key.provider)).await
            {
                let _ = updates.send(UiEvent::Error(error.to_string())).await;
            }
        });
        Self {
            app: App::new(key.session_id, Mode::Replay),
            watch,
            events,
            feeder,
            frame_at: Instant::now(),
        }
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.frame_at);
        self.frame_at = now;
        let _ = self.app.flow.tick_auto_pan(elapsed);
        self.app.flow.tick_animation(elapsed);
        self.app.tick_camera(elapsed);
        self.app.tick_timeline(elapsed);
    }

    fn next_frame(&self) -> Option<Instant> {
        (matches!(self.app.transport(), Transport::Playing | Transport::Live)
            || self.app.camera_glide.is_some()
            || self.app.pending_seek.is_some())
        .then_some(self.frame_at + Duration::from_millis(16))
    }

    fn close(self) -> JoinHandle<()> {
        // Closing both channels cancels observation without interrupting native
        // work. Retain this handle until a possible full-file load has finished.
        drop(self.watch);
        drop(self.events);
        self.feeder
    }
}

enum Wake {
    Input(Option<std::io::Result<Event>>),
    Replay(Option<UiEvent>),
    Fleet,
    Draft,
    Frame,
    Retired,
}

async fn replay_update(replay: &mut Option<Replay>) -> Option<UiEvent> {
    if let Some(replay) = replay {
        replay.events.recv().await
    } else {
        std::future::pending().await
    }
}

async fn retirement(retiring: &mut Option<JoinHandle<()>>) {
    if let Some(feeder) = retiring {
        let _ = feeder.await;
    } else {
        std::future::pending().await
    }
}

async fn frame_due(deadline: Option<Instant>) {
    if let Some(deadline) = deadline {
        tokio::time::sleep_until(deadline).await;
    } else {
        std::future::pending().await
    }
}

struct RestoreTerminal;

fn repaint(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    // Fullscreen resize invalidates both buffers without a cursor query. A
    // Terminal::clear query would compete with our single input stream.
    let area = terminal.get_frame().area();
    terminal.resize(area)
}

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
    let mut replay: Option<Replay> = None;
    let mut retiring = None;
    let mut dirty = true;
    let result = loop {
        let fleet_changed = app.poll();
        dirty |= fleet_changed && replay.is_none();
        if dirty {
            if let Some(replay) = replay.as_mut() {
                replay.tick();
            }
            if let Err(error) = terminal.draw(|frame| {
                if let Some(replay) = replay.as_mut() {
                    crate::ui::draw(frame, &mut replay.app);
                    let area = frame.area();
                    frame.render_widget(
                        Paragraph::new(" Read-only replay · q back to fleet ")
                            .style(Style::default().fg(Color::Yellow).bg(Color::Black)),
                        Rect::new(area.x, area.y, area.width, area.height.min(1)),
                    );
                } else {
                    app.draw(frame);
                }
            }) {
                break Err(error.into());
            }
            dirty = false;
        }
        let deadline = replay.as_ref().and_then(Replay::next_frame);
        let save_at = app.dirty_since.map(|at| {
            let due = Instant::from_std(at) + Duration::from_millis(500);
            // A full worker queue keeps the draft dirty. Retry at a bounded
            // rate instead of repeatedly selecting an expired deadline.
            due.max(Instant::now() + Duration::from_millis(25))
        });
        let wake = tokio::select! {
            _ = notify.notified() => Wake::Fleet,
            event = events.next() => Wake::Input(event),
            update = replay_update(&mut replay) => Wake::Replay(update),
            _ = frame_due(deadline) => Wake::Frame,
            _ = frame_due(save_at) => Wake::Draft,
            _ = retirement(&mut retiring) => Wake::Retired,
        };
        match wake {
            Wake::Fleet | Wake::Draft => {}
            Wake::Frame => dirty = true,
            Wake::Retired => retiring = None,
            Wake::Replay(Some(event)) => {
                if let Some(replay) = replay.as_mut() {
                    replay.app.handle_ui_event(event);
                    dirty = true;
                }
            }
            Wake::Replay(None) => {
                if let Some(replay) = replay.take() {
                    retiring = Some(replay.close());
                    dirty = true;
                }
            }
            Wake::Input(Some(Ok(Event::Key(key)))) if key.kind == KeyEventKind::Release => {}
            Wake::Input(Some(Ok(event))) => {
                let key = if let Event::Key(key) = &event {
                    Some(key)
                } else {
                    None
                };
                if key.is_some_and(|key| {
                    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
                }) {
                    break Ok(());
                }
                if let Some(detail) = replay.as_mut() {
                    if key.is_some_and(|key| matches!(key.code, KeyCode::Char('q' | 'Q'))) {
                        retiring = replay.take().map(Replay::close);
                        if let Err(error) = repaint(&mut terminal) {
                            break Err(error.into());
                        }
                    } else {
                        let _ = crate::handler::handle_event(&event, &mut detail.app);
                    }
                    dirty = true;
                } else if key.is_some_and(|key| key.code == KeyCode::Char('R'))
                    && app.edit.is_none()
                    && !app.searching
                    && !app.show_help
                {
                    if retiring
                        .as_ref()
                        .is_some_and(|task: &JoinHandle<()>| !task.is_finished())
                    {
                        app.status =
                            "Previous recording is closing; try replay again shortly".into();
                    } else if let Some(source) = app.replay_source() {
                        retiring = None;
                        replay = Some(Replay::open(source));
                        if let Err(error) = repaint(&mut terminal) {
                            break Err(error.into());
                        }
                    } else {
                        app.status = "Select a goal with a local recording to replay".into();
                    }
                    dirty = true;
                } else {
                    if app.handle_event(&event) {
                        break Ok(());
                    }
                    dirty = true;
                }
            }
            Wake::Input(Some(Err(error))) => break Err(error.into()),
            Wake::Input(None) => break Ok(()),
        }
    };
    drop(events);
    drop(restore);
    if let Some(replay) = replay {
        retiring = Some(replay.close());
    }
    if let Some(feeder) = retiring {
        let _ = feeder.await;
    }
    tokio::task::spawn_blocking(move || app.shutdown()).await?;
    result
}
