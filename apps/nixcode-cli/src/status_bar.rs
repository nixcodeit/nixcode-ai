use crate::input_mode::InputMode;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Margin, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Stylize, Widget};
use ratatui::widgets::Block;

pub struct StatusBar {
    current_mode: InputMode,
}

impl StatusBar {
    pub(crate) fn new(status: InputMode) -> Self {
        StatusBar {
            current_mode: status,
        }
    }
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let now = chrono::Local::now();
        let formatted_date = now.format("%d/%m/%Y %H:%M:%S").to_string();
        let horizontal = Layout::horizontal([Fill(1), Length(formatted_date.len() as u16)]);
        let [inner_area, date_area] = horizontal.areas(area.inner(Margin::new(1, 0)));

        Block::new().bg(Color::DarkGray).render(area, buf);
        Line::from(vec![
            Span::raw("Mode: "),
            Span::styled(
                format!(" {} ", self.current_mode.to_string()),
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .render(inner_area, buf);
        Span::from(formatted_date).render(date_area, buf);
    }
}
