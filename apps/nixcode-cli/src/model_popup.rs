use crate::app::AppEvent;
use crate::popup_utils;
use crate::utils::highlights::THEME;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use nixcode_llm_sdk::models::llm_model::{AllModels, LLMModel};
use nixcode_llm_sdk::providers::LLMProvider;
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::{Color, Modifier, Style, Stylize, Widget};
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use tokio::sync::mpsc::UnboundedSender;

pub struct ModelPopup {
    tx: UnboundedSender<AppEvent>,
    selected_index: usize,
    current_model: &'static LLMModel,
}

impl ModelPopup {
    pub fn new(tx: UnboundedSender<AppEvent>, current_model: &'static LLMModel) -> Self {
        // Find the index of the current model
        let selected_index = AllModels
            .iter()
            .position(|&model| model.model_name() == current_model.model_name())
            .unwrap_or(0);

        Self {
            tx,
            selected_index,
            current_model,
        }
    }

    pub fn handle_input_event(&mut self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        return false; // Close popup without selection
                    }
                    KeyCode::Enter => {
                        // Select the current model
                        if let Some(&model) = AllModels.get(self.selected_index) {
                            self.tx.send(AppEvent::ChangeModel(model)).ok();
                        }
                        return false; // Close popup after selection
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // Move selection down
                        if !AllModels.is_empty() {
                            self.selected_index = (self.selected_index + 1) % AllModels.len();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // Move selection up
                        if !AllModels.is_empty() {
                            self.selected_index = if self.selected_index == 0 {
                                AllModels.len() - 1
                            } else {
                                self.selected_index - 1
                            };
                        }
                    }
                    _ => {}
                }
            }
        }
        true // Keep popup open
    }
}

impl Widget for &ModelPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_height = (AllModels.len() + 2) as u16; // Number of models + header + hint
        let popup_width = 60;

        let popup_area = popup_utils::centered_rect(popup_width, popup_height, area);

        // Clear the area
        Clear.render(popup_area, buf);

        // Create the block
        let mut block = Block::bordered()
            .title(" Select LLM Model ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        if let Some(bg) = THEME.settings.background {
            let c = Color::Rgb(bg.r, bg.g, bg.b);
            block = block.bg(c);
        }

        block.render(popup_area, buf);

        // Inner area for content
        let inner_area = popup_area.inner(Margin {
            vertical: 1,
            horizontal: 2,
        });

        // Render model list
        for (i, &model) in AllModels.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let is_current = model.model_name() == self.current_model.model_name();

            let y_position = inner_area.y + i as u16;

            // Model provider style
            let mut provider_style = Style::default()
                .fg(get_provider_color(model.provider()))
                .add_modifier(Modifier::BOLD);
            if is_selected {
                provider_style = provider_style.bg(Color::DarkGray);
            }

            // Model name style with proper modifier handling
            let mut model_style = Style::default()
                .fg(if is_current {
                    Color::Green
                } else {
                    Color::White
                })
                .bg(if is_selected {
                    Color::DarkGray
                } else {
                    Color::Reset
                });

            // Only add bold if current model
            if is_current {
                model_style = model_style.add_modifier(Modifier::BOLD);
            }

            // Selection marker
            if is_selected {
                buf.set_string(inner_area.x, y_position, "> ", provider_style);
            } else {
                buf.set_string(inner_area.x, y_position, "  ", provider_style);
            }

            // Provider name
            let provider_text = format!("[{}]", model.provider().name());
            buf.set_string(inner_area.x + 1, y_position, &provider_text, provider_style);

            // Model name
            let display_text = if is_current {
                format!("{} (current)", model)
            } else {
                format!("{}", model)
            };

            buf.set_string(
                inner_area.x + 1 + provider_text.len() as u16 + 1,
                y_position,
                &display_text,
                model_style,
            );
        }

        // Render hint at bottom
        let hint_y = popup_area.y + popup_area.height - 1;
        let hint_text = "↑/↓: Navigate  Enter: Select  Esc: Cancel";
        buf.set_string(
            popup_area.x + (popup_area.width - hint_text.len() as u16) / 2,
            hint_y,
            hint_text,
            Style::default().fg(Color::DarkGray),
        );
    }
}

fn get_provider_color(provider: &LLMProvider) -> Color {
    match provider {
        LLMProvider::Anthropic => Color::Rgb(163, 77, 253), // Purple
        LLMProvider::OpenAI => Color::Rgb(16, 163, 127),    // Green
        LLMProvider::Groq => Color::Rgb(255, 165, 0),       // Orange
        LLMProvider::OpenRouter => Color::Rgb(59, 130, 246), // Blue
        LLMProvider::Gemini => Color::Rgb(234, 67, 53),     // Red
    }
}
