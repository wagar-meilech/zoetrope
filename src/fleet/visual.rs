//! Shared visual vocabulary for cached Fleet and Work presentation.
use super::*;
use crate::control::RuntimeStatus;
use crate::workstate::WorkStatus;

pub(super) const ACCENT: Color = Color::Indexed(178);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Cue {
    pub glyph: &'static str,
    pub color: Color,
}
impl Cue {
    pub fn work(status: Option<WorkStatus>) -> Self {
        match status {
            Some(WorkStatus::Planned) => Self {
                glyph: "○",
                color: Color::LightBlue,
            },
            Some(WorkStatus::Active) => Self {
                glyph: "●",
                color: Color::Cyan,
            },
            Some(WorkStatus::Blocked) => Self {
                glyph: "!",
                color: Color::LightRed,
            },
            Some(WorkStatus::Done) => Self {
                glyph: "✓",
                color: Color::Green,
            },
            Some(WorkStatus::Cancelled) => Self {
                glyph: "×",
                color: Color::DarkGray,
            },
            None => Self {
                glyph: "?",
                color: Color::Magenta,
            },
        }
    }
    pub fn runtime(status: &RuntimeStatus) -> Self {
        match status {
            RuntimeStatus::Working => Self::work(Some(WorkStatus::Active)),
            RuntimeStatus::WaitingForAnswer
            | RuntimeStatus::WaitingForApproval
            | RuntimeStatus::Error => Self::work(Some(WorkStatus::Blocked)),
            RuntimeStatus::Idle | RuntimeStatus::NotLoaded => Self {
                glyph: "○",
                color: Color::DarkGray,
            },
            RuntimeStatus::Unknown => Self::work(None),
        }
    }
    pub fn fleet(id: &FleetNodeId, status: Option<&RuntimeStatus>, archived: bool) -> Self {
        if let Some(status) = status {
            return Self::runtime(status);
        }
        if archived {
            return Self {
                glyph: "▤",
                color: Color::DarkGray,
            };
        }
        match id {
            FleetNodeId::Project(_) => Self {
                glyph: "▦",
                color: Color::Magenta,
            },
            FleetNodeId::Session(_) => Self {
                glyph: "◇",
                color: Color::LightBlue,
            },
            FleetNodeId::Agent(_) => Self {
                glyph: "◆",
                color: Color::Cyan,
            },
        }
    }
}
pub(super) fn theme() -> rataflow::Theme {
    let mut palette = rataflow::Theme::Dark.palette();
    palette.accent = ACCENT;
    rataflow::Theme::Custom(palette)
}
pub(super) fn legend() -> ratatui::text::Line<'static> {
    use ratatui::text::Span;
    let mut spans = vec![Span::raw("Legend: ")];
    for (label, status) in [
        ("○ planned", Some(WorkStatus::Planned)),
        ("● active", Some(WorkStatus::Active)),
        ("! blocked", Some(WorkStatus::Blocked)),
        ("✓ done", Some(WorkStatus::Done)),
        ("× cancelled", Some(WorkStatus::Cancelled)),
        ("? unknown", None),
    ] {
        spans.push(Span::styled(
            format!("{label}  "),
            Style::default().fg(Cue::work(status).color),
        ));
    }
    ratatui::text::Line::from(spans)
}

/// Keep prerequisite labels at reading zoom; avoid text collisions in overview.
#[derive(Debug, Clone, Default)]
pub(super) struct SemanticEdge;
impl rataflow::EdgeContent for SemanticEdge {
    fn compute_path(&self, ctx: &rataflow::EdgePathContext) -> rataflow::Path {
        rataflow::EdgeContent::compute_path(&StepEdge::default(), ctx)
    }
    fn render(&self, ctx: &rataflow::EdgeRenderContext, buf: &mut Buffer) {
        let a = ctx.world_to_terminal(rataflow::Position::new(0.0, 0.0));
        let b = ctx.world_to_terminal(rataflow::Position::new(10.0, 0.0));
        let label = if b.0 - a.0 >= 9 {
            ctx.label.map(ratatui::text::Text::raw)
        } else {
            None
        };
        ctx.render_path(&rataflow::EdgeStyle::default(), label.as_ref(), buf);
    }
}

/// Apply the same semantic colors to focused details, retaining field labels.
pub(super) fn details(text: &str, work_status: Option<WorkStatus>) -> ratatui::text::Text<'_> {
    use ratatui::text::{Line, Span, Text};
    Text::from(
        text.lines()
            .map(|line| {
                if line.starts_with("Status: ") || line.starts_with("› [") {
                    Line::styled(line, Style::default().fg(Cue::work(work_status).color))
                } else if line.starts_with("Blocker:") {
                    Line::styled(line, Style::default().fg(Color::LightRed))
                } else if ["Purpose:", "Next:", "Owner:", "Selected work:"]
                    .iter()
                    .any(|label| line.starts_with(label))
                {
                    let (label, value) = line.split_once(':').expect("known field label");
                    Line::from(vec![
                        Span::styled(
                            format!("{label}:"),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(value),
                    ])
                } else {
                    Line::raw(line)
                }
            })
            .collect::<Vec<_>>(),
    )
}
