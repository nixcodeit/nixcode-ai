use crate::app::AppEvent;
use crate::user_input::UserSingleLineInput;
use crate::utils::highlights::THEME;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::{Color, Modifier, Style, Stylize, Widget};
use ratatui::widgets::{Block, BorderType, Borders, Clear};

/// Information about a command and its aliases
struct CommandInfo {
    name: &'static str,               // Primary command name
    aliases: &'static [&'static str], // Alternate names/shortcuts
    description: &'static str,        // Description of what the command does
}

const MAX_DISPLAYED_SUGGESTIONS: usize = 5;

const AVAILABLE_COMMANDS: &[CommandInfo] = &[
    CommandInfo {
        name: "quit",
        aliases: &["exit", "q"],
        description: "Exit the application",
    },
    CommandInfo {
        name: "clear",
        aliases: &[],
        description: "Clear the chat history",
    },
    CommandInfo {
        name: "retry",
        aliases: &[],
        description: "Retry the last message",
    },
    CommandInfo {
        name: "remove-last-message",
        aliases: &["remove-last", "remove-last-msg", "remove-msg", "rlm"],
        description: "Remove the last message from the chat",
    },
    CommandInfo {
        name: "model",
        aliases: &["models", "m"],
        description: "List and select LLM models",
    },
];

/// Represents a command suggestion shown in the popup
struct CommandSuggestion {
    display_name: String,
    description: &'static str,
    is_alias: bool,
    original_command: &'static str,
}

pub struct CommandPopup {
    command: UserSingleLineInput,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
    suggestions: Vec<CommandSuggestion>,
    selected_suggestion: Option<usize>,
    command_is_valid: bool,
}

impl CommandPopup {
    pub(crate) fn new(tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) -> Self {
        CommandPopup {
            command: UserSingleLineInput::default(),
            tx,
            suggestions: Vec::new(),
            selected_suggestion: None,
            command_is_valid: true,
        }
    }

    fn flush_command(&mut self) {
        self.command.flush();
        self.suggestions.clear();
        self.selected_suggestion = None;
        self.command_is_valid = true;
    }

    fn execute_command(&mut self) {
        // Get the command to execute (either from selection or input)
        let command_to_execute = match self.selected_suggestion {
            Some(index) => self.suggestions.get(index).map_or_else(
                || self.command.as_string(),
                |suggestion| suggestion.display_name.clone(),
            ),
            None => self.command.as_string(),
        };

        // Check if the command is valid before executing
        if !self.is_valid_command(&command_to_execute) {
            self.command_is_valid = false;
            return;
        }

        // Normalize the command (resolve aliases to primary commands)
        let normalized_command = self.normalize_command(&command_to_execute);

        // Send the command for execution
        if self.tx.send(AppEvent::Command(normalized_command)).is_ok() {
            self.flush_command();
        }
    }

    // Check if a command exists (either as primary command or alias)
    fn is_valid_command(&self, input: &str) -> bool {
        let input = input.trim();
        if input.is_empty() {
            return false;
        }

        // Handle special case of model command with arguments: model <model-name>
        if input.starts_with("model ") {
            return true;
        }

        AVAILABLE_COMMANDS
            .iter()
            .any(|cmd| cmd.name == input || cmd.aliases.contains(&input))
    }

    // Convert aliases to their primary command
    fn normalize_command(&self, input: &str) -> String {
        let input = input.trim();

        // Find the primary command for the input (or return input as-is)
        for cmd in AVAILABLE_COMMANDS {
            if cmd.name == input {
                return input.to_string();
            }

            if cmd.aliases.contains(&input) {
                return cmd.name.to_string();
            }
        }

        input.to_string()
    }

    pub fn get_input_area(area: Rect) -> Rect {
        area.inner(Margin::new(1, 1))
    }

    pub fn get_input_position(&self, area: Rect) -> (u16, u16) {
        let input_area = Self::get_input_area(area);
        self.command.get_cursor_position(input_area)
    }

    // Update suggestions based on current input
    fn update_suggestions(&mut self) {
        self.suggestions.clear();
        let current_input = self.command.as_string().to_lowercase().trim().to_string();

        // Update command validity
        self.command_is_valid = current_input.is_empty() || self.is_valid_command(&current_input);

        // Build suggestions list with matching commands and aliases
        for cmd in AVAILABLE_COMMANDS {
            // Add main command if it matches
            if cmd.name.to_lowercase().starts_with(&current_input) || current_input.is_empty() {
                self.suggestions.push(CommandSuggestion {
                    display_name: cmd.name.to_string(),
                    description: cmd.description,
                    is_alias: false,
                    original_command: cmd.name,
                });
            }

            // Add any matching aliases
            for &alias in cmd.aliases {
                if alias.to_lowercase().starts_with(&current_input) || current_input.is_empty() {
                    self.suggestions.push(CommandSuggestion {
                        display_name: alias.to_string(),
                        description: cmd.description,
                        is_alias: true,
                        original_command: cmd.name,
                    });
                }
            }
        }

        // Sort suggestions - primary commands first, then alphabetically
        self.suggestions
            .sort_by(|a, b| match (a.is_alias, b.is_alias) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.display_name.cmp(&b.display_name),
            });

        // Reset selection if needed
        if self.selected_suggestion.is_some()
            && (self.selected_suggestion.unwrap() >= self.suggestions.len())
        {
            self.selected_suggestion = if self.suggestions.is_empty() {
                None
            } else {
                Some(0)
            }
        }
    }

    // Replace command with the selected suggestion
    fn complete_suggestion(&mut self) {
        if let Some(index) = self.selected_suggestion {
            if let Some(suggestion) = self.suggestions.get(index) {
                self.command = UserSingleLineInput::new(suggestion.display_name.clone());
                self.command_is_valid = true;
            }
        }
    }

    // Navigate to next suggestion
    fn next_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            self.selected_suggestion = None;
            return;
        }

        self.selected_suggestion = Some(match self.selected_suggestion {
            None => 0,
            Some(index) => (index + 1) % self.suggestions.len().min(MAX_DISPLAYED_SUGGESTIONS),
        });
    }

    // Navigate to previous suggestion
    fn prev_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            self.selected_suggestion = None;
            return;
        }

        self.selected_suggestion = Some(match self.selected_suggestion {
            None => 0,
            Some(index) => {
                if index == 0 {
                    self.suggestions.len() - 1
                } else {
                    index - 1
                }
            }
        });
    }

    pub(crate) fn handle_input_event(&mut self, event: &Event) {
        // Handle special keys for navigation and command execution
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        self.execute_command();
                        return;
                    }
                    KeyCode::Tab => {
                        if self.suggestions.is_empty() {
                            self.update_suggestions();
                            if !self.suggestions.is_empty() {
                                self.selected_suggestion = Some(0);
                            }
                        } else if self.selected_suggestion.is_some() {
                            self.complete_suggestion();
                        }
                        return;
                    }
                    KeyCode::Down => {
                        self.next_suggestion();
                        return;
                    }
                    KeyCode::Up => {
                        self.prev_suggestion();
                        return;
                    }
                    KeyCode::Esc => {
                        // Reset invalid command indication
                        self.command_is_valid = true;
                    }
                    _ => {}
                }
            }
        }

        // Handle normal input for other keys
        self.command.handle_input_events(event);

        // Update suggestions when input changes
        self.update_suggestions();
    }
}

impl Widget for &CommandPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Calculate dimensions
        let suggestion_height = self.suggestions.len().min(MAX_DISPLAYED_SUGGESTIONS) as u16;
        let has_suggestions = suggestion_height > 0;
        let popup_height = 3 + if has_suggestions {
            suggestion_height + 1
        } else {
            0
        };
        let popup_width = area.width;

        let popup_area = Rect {
            x: area.x,
            y: area.y,
            width: popup_width,
            height: popup_height,
        };

        // Clear the popup area
        Clear.render(popup_area, buf);

        // Set title based on command validity
        let (title, title_style) = if !self.command_is_valid {
            (
                "Invalid Command",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else {
            (
                "Command",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        };

        // Render the input block
        let mut input_block = Block::bordered()
            .title(title)
            .title_style(title_style)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        if let Some(bg) = THEME.settings.background {
            let c = Color::Rgb(bg.r, bg.g, bg.b);
            input_block = input_block.bg(c);
        }

        input_block.render(popup_area, buf);

        // Render the input field
        let input_area = CommandPopup::get_input_area(popup_area);
        self.command.render(input_area, buf);

        if has_suggestions {
            // Render suggestions
            self.render_suggestions(popup_area, buf, suggestion_height);
        } else if !self.command_is_valid {
            // Render error message for invalid commands
            let hint_area = Rect {
                x: area.x + 2,
                y: area.y + 3,
                width: area.width - 4,
                height: 1,
            };

            buf.set_string(
                hint_area.x,
                hint_area.y,
                "Command not found. Type 'help' to see available commands.",
                Style::default().fg(Color::Red),
            );
        }
    }
}

// Extension impl with helper methods for rendering
impl CommandPopup {
    fn render_suggestions(&self, popup_area: Rect, buf: &mut Buffer, suggestion_height: u16) {
        let suggestions_area = Rect {
            x: popup_area.x + 2,
            y: popup_area.y + 3,
            width: popup_area.width - 4,
            height: suggestion_height,
        };

        // Render each suggestion (up to 5)
        for (i, suggestion) in self.suggestions.iter().enumerate().take(5) {
            let suggestion_line_area = Rect {
                x: suggestions_area.x,
                y: suggestions_area.y + i as u16,
                width: suggestions_area.width,
                height: 1,
            };

            let is_selected = Some(i) == self.selected_suggestion;
            self.render_suggestion(buf, suggestion_line_area, suggestion, is_selected);
        }

        // Add keyboard hint at the bottom if we have suggestions
        if !self.suggestions.is_empty() {
            let hint_area = Rect {
                x: popup_area.x + 2,
                y: popup_area.y + popup_area.height - 1,
                width: popup_area.width - 4,
                height: 1,
            };

            buf.set_string(
                hint_area.x,
                hint_area.y,
                "↑/↓: Navigate  Tab: Complete  Enter: Execute",
                Style::default().fg(Color::DarkGray),
            );
        }
    }

    fn render_suggestion(
        &self,
        buf: &mut Buffer,
        area: Rect,
        suggestion: &CommandSuggestion,
        is_selected: bool,
    ) {
        // Set background for selected suggestion
        if is_selected {
            for x in area.x..area.x + area.width {
                for y in area.y..area.y + area.height {
                    // Use the new recommended approach instead of get_mut
                    buf[(x, y)].set_style(Style::default().bg(Color::DarkGray));
                }
            }
        }

        // Define styles based on selection state
        let name_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
            .bg(if is_selected {
                Color::DarkGray
            } else {
                Color::Reset
            });

        let alias_style = Style::default().fg(Color::Green).bg(if is_selected {
            Color::DarkGray
        } else {
            Color::Reset
        });

        let desc_style = Style::default()
            .fg(if is_selected {
                Color::White
            } else {
                Color::Gray
            })
            .bg(if is_selected {
                Color::DarkGray
            } else {
                Color::Reset
            });

        // Render command name and description
        let mut x_offset = 0;

        buf.set_string(
            area.x + x_offset,
            area.y,
            &suggestion.display_name,
            name_style,
        );
        x_offset += suggestion.display_name.len() as u16;

        // Show alias information if applicable
        if suggestion.is_alias {
            let alias_text = format!(" (alias of {})", suggestion.original_command);
            buf.set_string(area.x + x_offset, area.y, &alias_text, alias_style);
            x_offset += alias_text.len() as u16;
        }

        buf.set_string(area.x + x_offset, area.y, ": ", desc_style);
        x_offset += 2;

        buf.set_string(
            area.x + x_offset,
            area.y,
            suggestion.description,
            desc_style,
        );
    }
}