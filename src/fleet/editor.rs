use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// Small UTF-8 editor; cursor offsets always stay at character boundaries.
pub(super) struct Editor {
    pub text: String,
    pub cursor: usize,
    pub secret: bool,
}
impl Editor {
    pub fn new(text: String) -> Self {
        let cursor = text.len();
        Self {
            text,
            cursor,
            secret: false,
        }
    }
    pub fn insert(&mut self, text: &str) -> Result<bool, ()> {
        let mut inserted = String::new();
        let available = 65536usize.saturating_sub(self.text.len());
        for ch in text
            .chars()
            .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        {
            if inserted.len() + ch.len_utf8() > available {
                return Err(());
            }
            inserted.push(ch);
        }
        if inserted.is_empty() {
            return Ok(false);
        }
        self.text.insert_str(self.cursor, &inserted);
        self.cursor += inserted.len();
        Ok(true)
    }
    pub fn wrapped(&self, width: usize) -> (Vec<String>, usize, usize) {
        use unicode_width::UnicodeWidthChar;
        let width = width.max(1);
        let mut lines = vec![String::new()];
        let mut row = 0;
        let mut column = 0;
        let mut cursor = (0, 0);
        for (offset, raw) in self.text.char_indices() {
            let ch = if self.secret { '*' } else { raw };
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
    pub fn key(&mut self, key: &KeyEvent) -> Result<bool, ()> {
        match key.code {
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.text.clear();
                self.cursor = 0;
                return Ok(true);
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
                return Ok(true);
            }
            KeyCode::Delete if self.cursor < self.text.len() => {
                let n = self.text[self.cursor..].chars().next().unwrap().len_utf8();
                self.text.drain(self.cursor..self.cursor + n);
                return Ok(true);
            }
            KeyCode::Enter => return self.insert("\n"),
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                return self.insert(ch.encode_utf8(&mut [0; 4]));
            }
            _ => {}
        }
        Ok(false)
    }
}
