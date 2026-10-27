//! Shared visual vocabulary for cached Fleet and Work presentation.
use super::*;

pub(super) const ACCENT: Color = Color::Indexed(178);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Cue {
    pub glyph: &'static str,
    pub color: Color,
}
impl Cue {
    pub fn work(status: &str) -> Self {
        match status {
            "planned" => Self {
                glyph: "○",
                color: Color::LightBlue,
            },
            "in progress" | "Working" => Self {
                glyph: "●",
                color: Color::Cyan,
            },
            "blocked"
            | "Waiting for your answer"
            | "Waiting for approval in Codex"
            | "Codex reported a problem" => Self {
                glyph: "!",
                color: Color::LightRed,
            },
            "reported done" => Self {
                glyph: "✓",
                color: Color::Green,
            },
            "cancelled" => Self {
                glyph: "×",
                color: Color::DarkGray,
            },
            "Archived" => Self {
                glyph: "▤",
                color: Color::DarkGray,
            },
            "Idle in Codex" | "Not loaded in Codex" => Self {
                glyph: "○",
                color: Color::DarkGray,
            },
            _ => Self {
                glyph: "?",
                color: Color::Magenta,
            },
        }
    }
    pub fn fleet(id: &FleetNodeId, status: &str) -> Self {
        if !status.is_empty() {
            return Self::work(status);
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
        ("○ planned", "planned"),
        ("● active", "in progress"),
        ("! blocked", "blocked"),
        ("✓ done", "reported done"),
        ("× cancelled", "cancelled"),
        ("? unknown", "unknown"),
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
        let label = if b.0 - a.0 >= 7 {
            ctx.label.map(ratatui::text::Text::raw)
        } else {
            None
        };
        ctx.render_path(&rataflow::EdgeStyle::default(), label.as_ref(), buf);
    }
}

/// Apply the same semantic colors to focused details, retaining field labels.
pub(super) fn details(text: &str) -> ratatui::text::Text<'_> {
    use ratatui::text::{Line, Span, Text};
    Text::from(
        text.lines()
            .map(|line| {
                let status = line.strip_prefix("Status: ").or_else(|| {
                    line.strip_prefix("› [")
                        .and_then(|tail| tail.split_once(']').map(|(status, _)| status))
                });
                if let Some(status) = status {
                    Line::styled(line, Style::default().fg(Cue::work(status).color))
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
