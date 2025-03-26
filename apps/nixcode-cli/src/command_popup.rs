use crate::app::AppEvent;
use crate::user_input::UserSingleLineInput;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::Widget;
use ratatui::widgets::{Block, BorderType, Clear};

pub struct CommandPopup {
    command: UserSingleLineInput,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
}

impl CommandPopup {
    pub(crate) fn new(tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) -> Self {
        CommandPopup {
            command: UserSingleLineInput::default(),
            tx,
        }
    }

    fn flush_command(&mut self) {
        self.command.flush();
    }

    fn execute_command(&mut self) {
        if let Ok(_) = self.tx.send(AppEvent::Command(self.command.as_string())) {
            self.flush_command();
        }
    }

    pub fn get_input_area(area: Rect) -> Rect {
        area.inner(Margin::new(1, 1))
    }

    pub fn get_input_position(&self, area: Rect) -> (u16, u16) {
        let input_area = Self::get_input_area(area);
        self.command.get_cursor_position(input_area)
    }

    pub(crate) fn handle_input_event(&mut self, event: &Event) {
        self.command.handle_input_events(event);

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => {
                    self.execute_command();
                }
                _ => (),
            },
            _ => (),
        }
    }
}

impl Widget for &CommandPopup {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().title("Command").border_type(BorderType::Rounded);
        let input_area = CommandPopup::get_input_area(area);

        Clear.render(area, buf);
        block.render(area, buf);
        self.command.render(input_area, buf);
    }
}
