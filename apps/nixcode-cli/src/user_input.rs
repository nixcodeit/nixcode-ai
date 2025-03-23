use crossterm::event::{Event, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Span, Widget};

pub struct UserSingleLineInput {
    data: String,
    cursor: usize,
}

impl Default for UserSingleLineInput {
    fn default() -> Self {
        Self {
            data: String::new(),
            cursor: 0,
        }
    }
}

impl UserSingleLineInput {
    pub fn new(data: String) -> Self {
        let data_len = data.len();

        Self {
            data,
            cursor: data_len,
        }
    }

    pub fn as_string(&self) -> String {
        self.data.clone()
    }

    pub fn insert(&mut self, c: char) {
        self.data.insert(self.cursor, c);
        self.move_cursor(1);
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor > 0 {
            self.data.remove(self.cursor - 1);
        }
        self.move_cursor(-1);
    }

    pub fn handle_delete(&mut self) {
        let pos = self.cursor;
        if self.cursor <= self.data.len() {
            self.data.remove(pos);
        }
    }

    pub fn flush(&mut self) {
        self.data.clear();
        self.cursor = 0;
    }

    pub fn move_cursor(&mut self, offset: i16) {
        let new_cursor = (self.cursor as i16 + offset).clamp(0, self.data.len() as i16);
        self.cursor = new_cursor as usize;
    }

    pub fn handle_input_events(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                crossterm::event::KeyCode::Left => self.move_cursor(-1),
                crossterm::event::KeyCode::Right => self.move_cursor(1),
                crossterm::event::KeyCode::Backspace => self.handle_backspace(),
                crossterm::event::KeyCode::Delete => self.handle_delete(),
                crossterm::event::KeyCode::Char(c) => self.insert(c),
                _ => (),
            },
            _ => (),
        }
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn get_cursor_position(&self, area: Rect) -> (u16, u16) {
        (area.x + self.cursor as u16, area.y)
    }
}

impl Widget for &UserSingleLineInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.get_data();

        Span::raw(text).render(area, buf);
    }
}

impl Into<String> for &UserSingleLineInput {
    fn into(self) -> String {
        self.data.clone()
    }
}
