use nixcode_llm_sdk::message::message::Message;
use ratatui::prelude::Span;
use ratatui::text::Line;

pub struct MessageWidget {
    message: Message,
}

impl MessageWidget {
    pub fn new(message: Message) -> Self {
        Self { message }
    }

    pub fn get_lines(self) -> Vec<String> {
        let author = match self.message {
            Message::User { .. } => "You >",
            Message::Assistant { .. } => "Assistant >",
            Message::System { .. } => "System >",
        };

        let content = self.message.get_content();

        let lines: Vec<String> = content
            .into_iter()
            .filter(|c| c.is_text())
            .map(|c| c.get_text())
            .map(|c| c.unwrap().text.to_string())
            .collect();

        if lines.len() == 0 {
            return lines;
        }

        // let x = lines.get_mut(0).unwrap();

        lines
    }
}
