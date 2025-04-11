use crate::input_mode::InputMode;
use nixcode_llm_sdk::models::llm_model::LLMModel;
use nixcode_llm_sdk::providers::LLMProvider;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Fill, Length};
use ratatui::layout::{Layout, Margin, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Stylize, Widget};
use ratatui::widgets::Block;
use std::sync::Arc;

pub struct StatusBar {
    current_mode: InputMode,
    current_model: Option<Arc<LLMModel>>,
}

impl StatusBar {
    pub(crate) fn new(status: InputMode) -> Self {
        StatusBar {
            current_mode: status,
            current_model: None,
        }
    }

    pub(crate) fn with_model(mut self, model: Arc<LLMModel>) -> Self {
        self.current_model = Some(model);
        self
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
        // Render the mode info on the left
        let mode_line = Line::from(vec![
            Span::raw("Mode: "),
            Span::styled(
                format!(" {} ", self.current_mode.to_string()),
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let mode_line_width = mode_line.width() + 1;

        // Create layout with three sections or two sections based on model info availability
        let horizontal = if self.current_model.is_some() {
            Layout::horizontal([
                Length(mode_line_width as u16),      // Mode info fixed width
                Fill(1),                             // Model info takes remaining space
                Length(right_content_length as u16), // Date+version fixed width
            ])
        } else {
            Layout::horizontal([
                Fill(1),                             // Mode info takes available space
                Length(right_content_length as u16), // Date+version fixed width
            ])
        };

        let inner_margin = area.inner(Margin::new(1, 0));
        let areas = if self.current_model.is_some() {
            let areas: [Rect; 3] = horizontal.areas(inner_margin);
            areas
        } else {
            let areas: [Rect; 2] = horizontal.areas(inner_margin);
            [areas[0], areas[1], Rect::default()] // Add a dummy third area
        };

        Block::new().bg(Color::DarkGray).render(area, buf);

        mode_line.render(areas[0], buf);

        // Render model info in the middle if available
        if let Some(model) = self.current_model {
            let provider_color = match model.provider() {
                LLMProvider::Anthropic => Color::Rgb(163, 77, 253), // Purple
                LLMProvider::OpenAI => Color::Rgb(16, 163, 127),    // Green
                LLMProvider::Groq => Color::Rgb(255, 165, 0),       // Orange
                LLMProvider::OpenRouter => Color::Rgb(59, 130, 246), // Blue
                LLMProvider::Gemini | LLMProvider::GenAI => Color::Rgb(234, 67, 53), // Red
                LLMProvider::Llama => Color::Rgb(255, 255, 255),    // White
            };

            let model_line = Line::from(vec![
                Span::raw("Model: "),
                Span::styled(
                    format!(" {} ", model.provider().name()),
                    Style::new()
                        .fg(Color::Black)
                        .bg(provider_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{}", model),
                    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
            ]);

            model_line.render(areas[1], buf);

            // Render date and version on the right (third area)
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
            .render(areas[2], buf);
        } else {
            // If no model info, render date and version in the second area
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
            .render(areas[1], buf);
        }
    }
}
