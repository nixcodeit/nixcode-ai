use crossterm::event::{Event, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Span, Widget};
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

pub struct UserSingleLineInput {
    data: String,
    cursor: usize,
    scroll_offset: usize, // Track horizontal scroll position
    last_area_width: u16, // Remember the last area width for proper cursor handling
}

impl Default for UserSingleLineInput {
    fn default() -> Self {
        Self {
            data: String::new(),
            cursor: 0,
            scroll_offset: 0,
            last_area_width: 0,
        }
    }
}

impl UserSingleLineInput {
    pub fn new(data: String) -> Self {
        let data_len = data.len();

        Self {
            data,
            cursor: data_len,
            scroll_offset: 0,
            last_area_width: 0,
        }
    }

    pub fn as_string(&self) -> String {
        self.data.clone()
    }

    pub fn insert(&mut self, c: char) {
        self.data.insert(self.cursor, c);
        self.cursor += 1;
        self.adjust_scroll_offset();
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor > 0 {
            self.data.remove(self.cursor - 1);
            self.cursor -= 1;
            self.adjust_scroll_offset();
        }
    }

    pub fn handle_delete(&mut self) {
        if self.cursor < self.data.len() {
            self.data.remove(self.cursor);
            self.adjust_scroll_offset();
        }
    }

    pub fn flush(&mut self) {
        self.data.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    pub fn move_cursor(&mut self, offset: i16) {
        let new_cursor = (self.cursor as i16 + offset).clamp(0, self.data.len() as i16);
        self.cursor = new_cursor as usize;
        self.adjust_scroll_offset();
    }

    // Adjust scroll offset to ensure cursor is visible
    fn adjust_scroll_offset(&mut self) {
        if self.last_area_width == 0 {
            return; // Skip if we don't know the area width yet
        }

        let visible_width = self.last_area_width as usize;
        let cursor_width = self.data[..self.cursor].width();
        
        // Reserve 1 character of margin on the right edge
        let right_margin = 1;
        let effective_width = visible_width.saturating_sub(right_margin);

        // If cursor is beyond visible area to the right
        if cursor_width >= self.scroll_offset + effective_width {
            // Scroll to make cursor visible with margin
            self.scroll_offset = cursor_width.saturating_sub(effective_width);
        }
        // If cursor is beyond visible area to the left
        else if cursor_width < self.scroll_offset {
            // Scroll to place cursor at left edge
            self.scroll_offset = cursor_width;
        }
    }

    pub fn handle_input_events(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match key.code {
                    crossterm::event::KeyCode::Left => self.move_cursor(-1),
                    crossterm::event::KeyCode::Right => self.move_cursor(1),
                    crossterm::event::KeyCode::Home => {
                        self.cursor = 0;
                        self.scroll_offset = 0;
                    },
                    crossterm::event::KeyCode::End => {
                        self.cursor = self.data.len();
                        self.adjust_scroll_offset();
                    },
                    crossterm::event::KeyCode::Backspace => self.handle_backspace(),
                    crossterm::event::KeyCode::Delete => self.handle_delete(),
                    crossterm::event::KeyCode::Char(c) => self.insert(c),
                    _ => (),
                }
            },
            _ => (),
        }
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn get_visible_data(&self, width: usize) -> &str {
        let data_width = self.data.width();

        // If the entire text fits within width or there's no text
        if data_width <= width || self.data.is_empty() {
            return &self.data;
        }

        // Find the substring to display based on scroll offset
        let mut width_so_far = 0;
        let mut start_idx = 0;
        
        // Find start index based on scroll offset
        for (idx, c) in self.data.char_indices() {
            if width_so_far >= self.scroll_offset {
                start_idx = idx;
                break;
            }
            width_so_far += c.width().unwrap_or(1);
        }

        // For end index, we'll try to fill the entire width
        width_so_far = 0;
        let mut end_idx = self.data.len();
        
        for (idx, c) in self.data[start_idx..].char_indices() {
            let char_width = c.width().unwrap_or(1);
            // Check if adding this character would exceed the available width
            if width_so_far + char_width > width {
                end_idx = start_idx + idx;
                break;
            }
            width_so_far += char_width;
        }

        &self.data[start_idx..end_idx]
    }

    pub fn get_cursor_position(&self, area: Rect) -> (u16, u16) {
        // Calculate visible cursor position accounting for scroll offset
        let text_before_cursor = &self.data[..self.cursor];
        let cursor_width = text_before_cursor.width();
        
        // Calculate cursor position in visible area
        let visible_cursor_x = cursor_width.saturating_sub(self.scroll_offset);
        
        // Ensure cursor doesn't go beyond the visible area
        let max_visible_x = area.width.saturating_sub(1);
        let clamped_x = visible_cursor_x.min(max_visible_x as usize) as u16;
        
        (area.x + clamped_x, area.y)
    }
}

impl Widget for &UserSingleLineInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Create a mutable copy to adjust scroll offset
        let mut mutable_self = UserSingleLineInput {
            data: self.data.clone(),
            cursor: self.cursor,
            scroll_offset: self.scroll_offset,
            last_area_width: self.last_area_width,
        };
        
        // Store the area width for future calculations
        mutable_self.last_area_width = area.width;
        
        // Adjust scroll offset to ensure cursor is visible
        mutable_self.adjust_scroll_offset();

        // Get the visible portion of text
        let visible_text = mutable_self.get_visible_data(area.width as usize);

        // Render the visible text
        Span::raw(visible_text).render(area, buf);
    }
}

impl Into<String> for &UserSingleLineInput {
    fn into(self) -> String {
        self.data.clone()
    }
}