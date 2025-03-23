use std::ops::Add;
use nixcode_llm_sdk::message::message::Message;
use ratatui::prelude::*;
use ratatui::text::{Line, Text};

pub struct MessageWidget {
    message: Message,
}

impl MessageWidget {
    pub fn new(message: Message) -> Self {
        Self { message }
    }

    pub fn get_lines<'a>(self, viewport_width: u16) -> Vec<Line<'a>> {
        let author = match self.message {
            Message::User { .. } => Span::styled("You > ", Style::new().green()),
            Message::Assistant { .. } => Span::styled("Assistant > ", Style::new().yellow()),
            Message::System { .. } => Span::styled("System > ", Style::new().dark_gray()),
        }.bold();

        let content = self.message.get_content();

        let lines: Vec<Line> = content
            .into_iter()
            .filter(|c| c.is_text())
            .map(|c| c.get_text())
            .map(|c| Line::from(vec![author.clone(), Span::raw(c.unwrap().get_text())]))
            .collect();

        lines
    }
}
