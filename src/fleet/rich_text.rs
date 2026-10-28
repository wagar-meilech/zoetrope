//! Bounded display formatting for cached text. No IO, source edits or inference.
use super::{Color, Modifier, Style, visual};
use crate::workstate::WorkStatus;
use ratatui::text::{Line, Span, Text};

const PATH: Color = Color::Rgb(148, 226, 213);
const CODE: Color = Color::Rgb(203, 166, 247);
const MAX_BYTES: usize = 32 * 1024;
const MAX_LINES: usize = 512;

pub(super) fn render(text: &str, status: Option<WorkStatus>, compact: bool) -> Text<'_> {
    let mut end = text.len().min(MAX_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let mut output = Vec::new();
    let mut lines = text[..end].lines();
    let mut code_fence = None;
    for line in lines.by_ref().take(MAX_LINES) {
        if let Some((marker, count)) = code_fence {
            if fence(line).is_some_and(|(next, width, tail)| {
                next == marker && width >= count && tail.is_empty()
            }) {
                output.push(Line::styled("╰─", Style::default().fg(CODE)));
                code_fence = None;
            } else {
                let mut spans = vec![Span::styled("│ ", Style::default().fg(CODE))];
                tokens(line, Style::default().fg(CODE), &mut spans);
                output.push(Line::from(spans).style(Style::default().bg(Color::Rgb(30, 32, 46))));
            }
            continue;
        }
        if let Some((marker, count, language)) = fence(line) {
            if !compact {
                gap(&mut output);
            }
            output.push(Line::from(vec![
                Span::styled("╭─ ", Style::default().fg(CODE)),
                literal(
                    if language.is_empty() {
                        "code"
                    } else {
                        language
                    },
                    Style::default().fg(CODE),
                ),
            ]));
            code_fence = Some((marker, count));
            continue;
        }
        if line.trim().is_empty() {
            gap(&mut output);
            continue;
        }
        let heading = line.trim_start_matches('#');
        if (1..=6).contains(&(line.len() - heading.len())) && heading.starts_with(' ') {
            if !compact {
                gap(&mut output);
            }
            let style = Style::default()
                .fg(visual::ACCENT)
                .add_modifier(Modifier::BOLD);
            let mut spans = vec![Span::styled("── ", style)];
            inline(heading.trim(), style, &mut spans);
            output.push(Line::from(spans));
            continue;
        }
        if matches!(line.trim(), "---" | "***" | "___") {
            output.push(Line::styled(
                "──────────────────",
                Style::default().fg(Color::DarkGray),
            ));
            continue;
        }
        let style = if line.starts_with("Status: ") || line.starts_with("› [") {
            Style::default().fg(visual::Cue::work(status).color)
        } else if line.starts_with("Blocker:")
            || line.starts_with("error:")
            || line.starts_with("error[")
        {
            Style::default().fg(Color::LightRed)
        } else if line.starts_with("warning:") || line.starts_with("warning[") {
            Style::default().fg(Color::Rgb(250, 179, 135))
        } else if line.starts_with("Source:") || line.starts_with("Recorded:") {
            Style::default().fg(Color::Gray)
        } else {
            Style::default()
        };
        let mut spans = Vec::new();
        let value = if let Some(value) = line.strip_prefix("> ") {
            spans.push(Span::styled("│ ", Style::default().fg(visual::ACCENT)));
            value
        } else if let Some(value) = list_item(line, &mut spans) {
            value
        } else if let Some((label, value)) = line.split_once(':') {
            if label.len() <= 32
                && !label.is_empty()
                && (value.is_empty() || value.starts_with(char::is_whitespace))
                && label
                    .chars()
                    .all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
            {
                spans.push(Span::styled(
                    format!("{label}:"),
                    style.add_modifier(Modifier::BOLD),
                ));
                value
            } else {
                line
            }
        } else {
            line
        };
        inline(value, style, &mut spans);
        output.push(Line::from(spans));
    }
    let limited = end < text.len() || lines.next().is_some();
    if code_fence.is_some() {
        output.push(Line::styled(
            if limited {
                "╰─ code continues in omitted text"
            } else {
                "╰─ unclosed fence · recorded text retained"
            },
            Style::default().fg(Color::Rgb(250, 179, 135)),
        ));
    }
    if limited {
        gap(&mut output);
        output.push(Line::styled(
            "… Display limited; more text remains in the recorded source.",
            Style::default().fg(Color::Gray),
        ));
    }
    Text::from(output)
}

fn fence(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let count = trimmed.chars().take_while(|ch| *ch == marker).count();
    (count >= 3).then(|| (marker, count, trimmed[count..].trim()))
}

fn list_item<'a>(line: &'a str, spans: &mut Vec<Span<'a>>) -> Option<&'a str> {
    let trimmed = line.trim_start_matches(' ');
    let indent = &line[..line.len() - trimmed.len()];
    let (marker, mut value) =
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            ("• ", &trimmed[2..])
        } else {
            let (number, value) = trimmed.split_once(". ")?;
            if number.is_empty()
                || number.len() > 9
                || !number.bytes().all(|byte| byte.is_ascii_digit())
            {
                return None;
            }
            (&trimmed[..number.len() + 2], value)
        };
    spans.push(Span::raw(indent));
    spans.push(Span::styled(marker, Style::default().fg(visual::ACCENT)));
    if value.starts_with("[x] ") || value.starts_with("[X] ") || value.starts_with("[ ] ") {
        spans.push(Span::styled(
            &value[..4],
            Style::default().fg(if value.starts_with("[ ]") {
                Color::Gray
            } else {
                Color::Green
            }),
        ));
        value = &value[4..];
    }
    Some(value)
}

fn gap(lines: &mut Vec<Line<'_>>) {
    if lines.last().is_some_and(|line| !line.spans.is_empty()) {
        lines.push(Line::default());
    }
}

fn inline<'a>(mut text: &'a str, style: Style, spans: &mut Vec<Span<'a>>) {
    while !text.is_empty() {
        let Some(at) = text.find(['`', '*', '[']) else {
            tokens(text, style, spans);
            break;
        };
        tokens(&text[..at], style, spans);
        text = &text[at..];
        let marker = if text.starts_with("**") {
            "**"
        } else if text.starts_with('`') {
            "`"
        } else if text.starts_with('*') {
            "*"
        } else {
            "["
        };
        if marker == "[" {
            if let Some(mid) = text.find("](")
                && let Some(end) = text[mid + 2..].find(')')
            {
                tokens(
                    &text[1..mid],
                    style.add_modifier(Modifier::UNDERLINED),
                    spans,
                );
                spans.push(Span::styled(" (", style));
                spans.push(literal(&text[mid + 2..mid + 2 + end], style.fg(PATH)));
                spans.push(Span::styled(")", style));
                text = &text[mid + 3 + end..];
                continue;
            }
        } else if let Some(end) = text[marker.len()..].find(marker) {
            let value = &text[marker.len()..marker.len() + end];
            let content_style = match marker {
                "`" => style.fg(if is_path(value) { PATH } else { CODE }),
                "**" => style.add_modifier(Modifier::BOLD),
                _ => style.add_modifier(Modifier::ITALIC),
            };
            tokens(value, content_style, spans);
            text = &text[marker.len() * 2 + end..];
            continue;
        }
        // An unmatched delimiter is literal; avoid repeatedly scanning its tail.
        tokens(text, style, spans);
        break;
    }
}

fn tokens<'a>(text: &'a str, style: Style, spans: &mut Vec<Span<'a>>) {
    for token in text.split_inclusive(char::is_whitespace) {
        let content = token.trim_end_matches(char::is_whitespace);
        let (value, whitespace) = token.split_at(content.len());
        if !value.is_empty() {
            spans.push(literal(
                value,
                if is_path(value) {
                    style.fg(PATH)
                } else {
                    style
                },
            ));
        }
        if !whitespace.is_empty() {
            spans.push(literal(whitespace, style));
        }
    }
}

fn literal(value: &str, style: Style) -> Span<'_> {
    if value.chars().any(char::is_control) {
        let mut visible = String::with_capacity(value.len());
        for ch in value.chars() {
            match ch {
                '\t' => visible.push_str("    "),
                '\u{00}'..='\u{1f}' => {
                    visible.push(char::from_u32(0x2400 + ch as u32).unwrap_or('�'))
                }
                '\u{7f}' => visible.push('␡'),
                ch if ch.is_control() => visible.push('�'),
                ch => visible.push(ch),
            }
        }
        Span::styled(visible, style)
    } else {
        Span::styled(value, style)
    }
}

fn is_path(value: &str) -> bool {
    let value = value.trim_matches([
        '"', '\'', '(', ')', '[', ']', '{', '}', '<', '>', ',', '.', ';', '!', '?',
    ]);
    if value.starts_with(['/', '\\'])
        || value.starts_with("~/")
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains("://")
    {
        return value.len() > 1;
    }
    if value.contains('/') || value.contains('\\') {
        return value.chars().any(char::is_alphabetic);
    }
    let file = value.split(':').next().unwrap_or(value);
    file.rsplit_once('.').is_some_and(|(_, extension)| {
        matches!(
            extension,
            "rs" | "md"
                | "toml"
                | "lock"
                | "json"
                | "jsonl"
                | "yaml"
                | "yml"
                | "py"
                | "sh"
                | "js"
                | "ts"
                | "tsx"
                | "jsx"
                | "css"
                | "html"
                | "txt"
                | "sql"
                | "go"
                | "c"
                | "h"
                | "svg"
        )
    })
}
