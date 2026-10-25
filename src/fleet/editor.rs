use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// Small UTF-8 editor; cursor offsets always stay at character boundaries.
pub(super) struct Editor {
    pub text: String,
    pub cursor: usize,
}
impl Editor {
    pub fn new(text: String) -> Self {
        let cursor = text.len();
        Self { text, cursor }
    }
    pub fn insert(&mut self, text: &str) -> bool {
        let mut inserted = String::new();
        let available = 65536usize.saturating_sub(self.text.len());
        for ch in text
            .chars()
            .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        {
            if inserted.len() + ch.len_utf8() > available {
                break;
            }
            inserted.push(ch);
        }
        if inserted.is_empty() {
            return false;
        }
        self.text.insert_str(self.cursor, &inserted);
        self.cursor += inserted.len();
        true
    }
    pub fn wrapped(&self, width: usize) -> (Vec<String>, usize, usize) {
        use unicode_width::UnicodeWidthChar;
        let width = width.max(1);
        let mut lines = vec![String::new()];
        let mut row = 0;
        let mut column = 0;
        let mut cursor = (0, 0);
        for (offset, ch) in self.text.char_indices() {
            if offset == self.cursor {
                cursor = (row, column);
            }
            if ch == '\n' {
                lines.push(String::new());
                row += 1;
                column = 0;
                continue;
            }
            let text = if ch == '\t' {
                "    ".into()
            } else {
                ch.to_string()
            };
            for symbol in text.chars() {
                let cells = symbol.width().unwrap_or(0);
                if column + cells > width {
                    lines.push(String::new());
                    row += 1;
                    column = 0;
                }
                lines[row].push(symbol);
                column += cells;
                if column >= width {
                    lines.push(String::new());
                    row += 1;
                    column = 0;
                }
            }
        }
        if self.cursor == self.text.len() {
            cursor = (row, column);
        }
        (lines, cursor.0, cursor.1)
    }
    pub fn key(&mut self, key: &KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.text.clear();
                self.cursor = 0;
                return true;
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => self.cursor = 0,
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.cursor = self.text.len()
            }
            KeyCode::Left => {
                self.cursor = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(i, _)| i)
            }
            KeyCode::Right => {
                if let Some(ch) = self.text[self.cursor..].chars().next() {
                    self.cursor += ch.len_utf8();
                }
            }
            KeyCode::Home => {
                self.cursor = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1)
            }
            KeyCode::End => {
                self.cursor += self.text[self.cursor..]
                    .find('\n')
                    .unwrap_or(self.text.len() - self.cursor)
            }
            KeyCode::Backspace if self.cursor > 0 => {
                let previous = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .unwrap()
                    .0;
                self.text.drain(previous..self.cursor);
                self.cursor = previous;
                return true;
            }
            KeyCode::Delete if self.cursor < self.text.len() => {
                let n = self.text[self.cursor..].chars().next().unwrap().len_utf8();
                self.text.drain(self.cursor..self.cursor + n);
                return true;
            }
            KeyCode::Enter if self.text.len() < 65536 => {
                self.text.insert(self.cursor, '\n');
                self.cursor += 1;
                return true;
            }
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    && self.text.len() + ch.len_utf8() <= 65536 =>
            {
                self.text.insert(self.cursor, ch);
                self.cursor += ch.len_utf8();
                return true;
            }
            _ => {}
        }
        false
    }
}
