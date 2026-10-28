//! Bounded display formatting for cached text. No IO, source edits or inference.
use super::{Color, Modifier, Style, visual};
use crate::workstate::WorkStatus;
use ratatui::text::{Line, Span, Text};
use std::borrow::Cow;

const PATH: Color = Color::Rgb(148, 226, 213);
const CODE: Color = Color::Rgb(203, 166, 247);
const MAX_BYTES: usize = 32 * 1024;
const MAX_LINES: usize = 512;

/// Source formatting state ends at each typed fragment, before later UI text.
pub(super) enum Fragment<'a> {
    Markdown(Cow<'a, str>),
    Shell(&'a str),
    Compact { text: Cow<'a, str>, width: u16 },
}
impl<'a> From<String> for Fragment<'a> {
    fn from(text: String) -> Self {
        Self::Markdown(Cow::Owned(text))
    }
}
impl<'a> From<&'a str> for Fragment<'a> {
    fn from(text: &'a str) -> Self {
        Self::Markdown(Cow::Borrowed(text))
    }
}

pub(super) fn render<'a>(
    fragments: &'a [Fragment<'_>],
    status: Option<WorkStatus>,
    compact: bool,
) -> Text<'a> {
    let mut output = Vec::new();
    let mut remaining_bytes = MAX_BYTES;
    let mut remaining_lines = MAX_LINES;
    for (at, fragment) in fragments.iter().enumerate() {
        let (text, shell, clipped_width) = match fragment {
            Fragment::Markdown(text) => (text.as_ref(), false, None),
            Fragment::Shell(text) => (*text, true, None),
            Fragment::Compact { text, width } => (text.as_ref(), false, Some(*width)),
        };
        let text = if clipped_width.is_some() {
            text.lines().next().unwrap_or("")
        } else {
            text
        };
        if text.is_empty() {
            continue;
        }
        if clipped_width.is_none() {
            gap(&mut output);
        }
        let mut end = text.len().min(remaining_bytes);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        remaining_bytes -= end;
        let mut lines = text[..end].lines();
        let mut code_fence = None;
        if shell {
            output.push(Line::styled("╭─ sh", Style::default().fg(CODE)));
        }
        let allowed_lines = remaining_lines;
        for line in lines.by_ref().take(allowed_lines) {
            remaining_lines -= 1;
            if let Some(width) = clipped_width {
                output.push(clip_line(prose_line(line, status), width));
                continue;
            }
            if shell {
                output.push(code_line(line));
                continue;
            }
            if let Some((marker, count)) = code_fence {
                if fence(line).is_some_and(|(next, width, tail)| {
                    next == marker && width >= count && tail.is_empty()
                }) {
                    output.push(Line::styled("╰─", Style::default().fg(CODE)));
                    code_fence = None;
                } else {
                    output.push(code_line(line));
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
            output.push(prose_line(line, status));
        }
        let limited = end < text.len() || lines.next().is_some();
        if shell {
            output.push(Line::styled("╰─", Style::default().fg(CODE)));
        } else if code_fence.is_some() {
            output.push(Line::styled(
                if limited {
                    "╰─ code continues in omitted text"
                } else {
                    "╰─ unclosed fence · recorded text retained"
                },
                Style::default().fg(Color::Rgb(250, 179, 135)),
            ));
        }
        if limited || ((remaining_bytes == 0 || remaining_lines == 0) && at + 1 < fragments.len()) {
            gap(&mut output);
            output.push(Line::styled(
                "… Display limited; more text remains in the recorded source.",
                Style::default().fg(Color::Gray),
            ));
            break;
        }
    }
    Text::from(output)
}

fn prose_line(line: &str, status: Option<WorkStatus>) -> Line<'_> {
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
    Line::from(spans)
}

fn clip_line(line: Line<'_>, width: u16) -> Line<'_> {
    use unicode_width::UnicodeWidthStr;
    if line.width() <= width as usize {
        return line;
    }
    if width == 0 {
        return Line::default();
    }
    let mut remaining = width.saturating_sub(1) as usize;
    let mut clipped = Vec::new();
    for span in line.spans {
        if span.width() <= remaining {
            remaining -= span.width();
            clipped.push(span);
        } else {
            let mut value = String::new();
            for grapheme in span.styled_graphemes(Style::default()) {
                let width = UnicodeWidthStr::width(grapheme.symbol);
                if width > remaining {
                    break;
                }
                remaining -= width;
                value.push_str(grapheme.symbol);
            }
            clipped.push(Span::styled(value, span.style));
            break;
        }
    }
    clipped.push(Span::styled("…", Style::default().fg(Color::Gray)));
    Line::from(clipped)
}

fn code_line(line: &str) -> Line<'_> {
    let mut spans = vec![Span::styled("│ ", Style::default().fg(CODE))];
    tokens(line, Style::default().fg(CODE), &mut spans);
    Line::from(spans).style(Style::default().bg(Color::Rgb(30, 32, 46)))
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
            if let Some((label, target, tail)) = link(text) {
                tokens(label, style.add_modifier(Modifier::UNDERLINED), spans);
                spans.push(Span::styled(" (", style));
                spans.push(literal(target, style.fg(PATH)));
                spans.push(Span::styled(")", style));
                text = tail;
            } else {
                spans.push(literal("[", style));
                text = &text[1..];
            }
            continue;
        } else if let Some(end) = text[marker.len()..].find(marker) {
            let value = &text[marker.len()..marker.len() + end];
            let content_style = match marker {
                "`" => style.fg(CODE),
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

fn link(text: &str) -> Option<(&str, &str, &str)> {
    // Nested labels are outside this subset. Match the immediate closing label,
    // never a later unrelated bracket pair.
    let close = text[1..].find(['[', ']'])? + 1;
    if text.as_bytes()[close] != b']' {
        return None;
    }
    let target = text[close + 1..].strip_prefix('(')?;
    let mut depth = 1usize;
    for (at, ch) in target.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&text[1..close], &target[..at], &target[at + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn tokens<'a>(text: &'a str, style: Style, spans: &mut Vec<Span<'a>>) {
    let mut start = 0;
    let mut candidate = None;
    for (at, ch) in text.char_indices() {
        let next = path_character(ch);
        if candidate.is_some_and(|current| current != next) {
            token(&text[start..at], candidate == Some(true), style, spans);
            start = at;
        }
        candidate = Some(next);
    }
    if start < text.len() {
        token(&text[start..], candidate == Some(true), style, spans);
    }
}

fn token<'a>(value: &'a str, candidate: bool, style: Style, spans: &mut Vec<Span<'a>>) {
    let path = value.trim_end_matches(['.', ':']);
    if candidate && is_path(path) {
        spans.push(literal(path, style.fg(PATH)));
        if path.len() < value.len() {
            spans.push(literal(&value[path.len()..], style));
        }
    } else {
        spans.push(literal(value, style));
    }
}

fn path_character(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | '\\' | ':' | '~' | '@')
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
    if value.starts_with('/')
        || value.starts_with("\\\\")
        || value.starts_with("~/")
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains("://")
        || (value.as_bytes().get(1) == Some(&b':')
            && matches!(value.as_bytes().get(2), Some(b'/' | b'\\')))
    {
        return value.len() > 1;
    }
    // An unanchored slash alone is ambiguous (e.g. PageUp/PageDown). Require a
    // recognizable file suffix; explicit relative/absolute paths remain above.
    let file = value
        .split(':')
        .next()
        .unwrap_or(value)
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(value);
    matches!(
        file,
        "Dockerfile" | "Makefile" | ".gitignore" | ".dockerignore" | ".env"
    ) || file.rsplit_once('.').is_some_and(|(_, extension)| {
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
