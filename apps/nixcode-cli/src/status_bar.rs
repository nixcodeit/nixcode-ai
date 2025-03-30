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
        // Get application version from Cargo.toml
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        let now = chrono::Local::now();
        let formatted_date = now.format("%d/%m/%Y %H:%M:%S").to_string();
        let version_text = format!("v{}", VERSION);

        // Calculate total length of the right side content (date + version)
        let right_content_length = formatted_date.len() + 1 + version_text.len();

        // Create layout with three sections: mode info, fill space, date+version
        let horizontal = Layout::horizontal([Fill(1), Length(right_content_length as u16)]);
        let [inner_area, right_area] = horizontal.areas(area.inner(Margin::new(1, 0)));

        Block::new().bg(Color::DarkGray).render(area, buf);

        // Render the mode info on the left
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

        // Render date and version on the right
        Line::from(vec![
            Span::styled(
                version_text,
                Style::new()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::from(formatted_date),
        ])
        .render(right_area, buf);
    }
}
