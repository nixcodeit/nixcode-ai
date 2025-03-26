use crate::app::AppEvent;
use crate::user_input::UserSingleLineInput;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::{Color, Modifier, Style, Widget};
use ratatui::widgets::{Block, BorderType, Borders, Clear};

// Define available commands with descriptions and aliases
struct CommandInfo {
    name: &'static str,           // Primary command name
    aliases: &'static [&'static str], // Alternate names/shortcuts
    description: &'static str,    // Description of what the command does
}

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
        name: "settings",
        aliases: &[],
        description: "Show settings view",
    },
    CommandInfo {
        name: "chat",
        aliases: &[],
        description: "Return to chat view",
    },
    CommandInfo {
        name: "help",
        aliases: &["?"],
        description: "Show help information",
    },
];

// Helper struct for displaying suggestions
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
        // If a suggestion is selected, use that instead of the command input
        let command_to_execute = if let Some(index) = self.selected_suggestion {
            if let Some(suggestion) = self.suggestions.get(index) {
                // Use the display name from the selected suggestion
                suggestion.display_name.clone()
            } else {
                // Fallback to input if suggestion index is invalid
                self.command.as_string()
            }
        } else {
            // No suggestion selected, use the command from input
            self.command.as_string()
        };

        // Check if the command is valid before executing
        if !self.is_valid_command(&command_to_execute) {
            // If invalid command, just show visual feedback but don't execute
            self.command_is_valid = false;
            return;
        }

        // Find if it's an alias and get the primary command
        let normalized_command = self.normalize_command(&command_to_execute);

        // Send the normalized command for execution
        if let Ok(_) = self.tx.send(AppEvent::Command(normalized_command)) {
            self.flush_command();
        }
    }

    // Check if a command exists (either as primary command or alias)
    fn is_valid_command(&self, input: &str) -> bool {
        let input = input.trim();
        if input.is_empty() {
            return false;
        }

        for cmd in AVAILABLE_COMMANDS {
            if cmd.name == input {
                return true;
            }

            if cmd.aliases.contains(&input) {
                return true;
            }
        }

        false
    }

    // Convert aliases to their primary command
    fn normalize_command(&self, input: &str) -> String {
        let input = input.trim();

        // Check if it's an alias
        for cmd in AVAILABLE_COMMANDS {
            if cmd.name == input {
                return input.to_string();
            }

            if cmd.aliases.contains(&input) {
                return cmd.name.to_string();
            }
        }

        // If not found, return as is
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

        // Build suggestions list with primary commands and aliases
        for cmd in AVAILABLE_COMMANDS {
            // Check if main command matches
            if cmd.name.to_lowercase().starts_with(&current_input) || current_input.is_empty() {
                self.suggestions.push(CommandSuggestion {
                    display_name: cmd.name.to_string(),
                    description: cmd.description,
                    is_alias: false,
                    original_command: cmd.name,
                });
            }

            // Check if any aliases match
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

        // Sort suggestions - put primary commands first, then sort alphabetically
        self.suggestions.sort_by(|a, b| {
            match (a.is_alias, b.is_alias) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.display_name.cmp(&b.display_name),
            }
        });

        // Reset selection if suggestions changed
        if self.selected_suggestion.is_some() &&
            (self.selected_suggestion.unwrap() >= self.suggestions.len()) {
            self.selected_suggestion = if self.suggestions.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    // Complete with selected suggestion
    fn complete_suggestion(&mut self) {
        if let Some(index) = self.selected_suggestion {
            if let Some(suggestion) = self.suggestions.get(index) {
                // Replace command with suggestion
                self.command = UserSingleLineInput::new(suggestion.display_name.to_string());
                self.command_is_valid = true; // Suggestion is always valid
            }
        }
    }

    // Navigate suggestions
    fn next_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            self.selected_suggestion = None;
            return;
        }

        match self.selected_suggestion {
            None => self.selected_suggestion = Some(0),
            Some(index) => {
                let next_index = (index + 1) % self.suggestions.len();
                self.selected_suggestion = Some(next_index);
            }
        }
    }

    fn prev_suggestion(&mut self) {
        if self.suggestions.is_empty() {
            self.selected_suggestion = None;
            return;
        }

        match self.selected_suggestion {
            None => self.selected_suggestion = Some(0),
            Some(index) => {
                let prev_index = if index == 0 {
                    self.suggestions.len() - 1
                } else {
                    index - 1
                };
                self.selected_suggestion = Some(prev_index);
            }
        }
    }

    pub(crate) fn handle_input_event(&mut self, event: &Event) {
        // First, check for navigation keys specifically so they don't get passed to input handling
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => {
                    self.execute_command();
                    return;
                },
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
                },
                KeyCode::Down => {
                    self.next_suggestion();
                    return;
                },
                KeyCode::Up => {
                    self.prev_suggestion();
                    return;
                },
                KeyCode::Esc => {
                    // Reset invalid command indication if Esc is pressed
                    self.command_is_valid = true;
                }
                _ => {
                    // For other keys, continue to normal input handling
                }
            },
            _ => {
                // Non-key events
            }
        }

        // Handle normal input for other keys
        self.command.handle_input_events(event);

        // Update suggestions when input changes
        self.update_suggestions();
    }

    // Render custom input text with color based on validity
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        // Choose input text color based on validity
        let style = if !self.command_is_valid && !self.command.as_string().is_empty() {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        };

        // Get the visible portion of text for the area
        let visible_text = self.command.get_visible_data(area.width as usize);

        // Render the text with appropriate style
        for (i, c) in visible_text.chars().enumerate() {
            if i < area.width as usize {
                buf.get_mut(area.x + i as u16, area.y).set_char(c).set_style(style);
            }
        }
    }
}

impl Widget for &CommandPopup {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let suggestion_height = self.suggestions.len().min(5) as u16;
        let has_suggestions = suggestion_height > 0;

        let popup_height = 3 + if has_suggestions { suggestion_height + 1 } else { 0 };
        let popup_width = area.width;

        let popup_area = Rect {
            x: area.x,
            y: area.y,
            width: popup_width,
            height: popup_height,
        };

        Clear.render(popup_area, buf);

        let title = if !self.command_is_valid {
            "Invalid Command"
        } else {
            "Command"
        };

        let title_style = if !self.command_is_valid {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        };

        let input_block = Block::bordered()
            .title(title)
            .title_style(title_style)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        input_block.clone().render(popup_area, buf);

        let input_area = CommandPopup::get_input_area(popup_area);

        self.command.render(input_area, buf);

        if has_suggestions {
            let suggestions_area = Rect {
                x: area.x + 2,
                y: area.y + 3,
                width: area.width - 4,
                height: suggestion_height,
            };

            for (i, suggestion) in self.suggestions.iter().enumerate().take(5) {
                let suggestion_line_area = Rect {
                    x: suggestions_area.x,
                    y: suggestions_area.y + i as u16,
                    width: suggestions_area.width,
                    height: 1,
                };

                let is_selected = Some(i) == self.selected_suggestion;

                if is_selected {
                    for x in suggestion_line_area.x..suggestion_line_area.x + suggestion_line_area.width {
                        for y in suggestion_line_area.y..suggestion_line_area.y + suggestion_line_area.height {
                            buf.get_mut(x, y).set_style(Style::default().bg(Color::DarkGray));
                        }
                    }
                }

                let name_style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                };

                let alias_style = if is_selected {
                    Style::default().fg(Color::Green).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Green)
                };

                let desc_style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let mut x_offset = 0;

                let name = &suggestion.display_name;
                buf.set_string(
                    suggestion_line_area.x + x_offset,
                    suggestion_line_area.y,
                    name,
                    name_style,
                );
                x_offset += name.len() as u16;

                if suggestion.is_alias {
                    let alias_text = format!(" (alias of {})", suggestion.original_command);
                    buf.set_string(
                        suggestion_line_area.x + x_offset,
                        suggestion_line_area.y,
                        &alias_text,
                        alias_style,
                    );
                    x_offset += alias_text.len() as u16;
                }

                buf.set_string(
                    suggestion_line_area.x + x_offset,
                    suggestion_line_area.y,
                    ": ",
                    desc_style,
                );
                x_offset += 2;

                buf.set_string(
                    suggestion_line_area.x + x_offset,
                    suggestion_line_area.y,
                    suggestion.description,
                    desc_style,
                );
            }

            if !self.suggestions.is_empty() {
                let hint_area = Rect {
                    x: area.x + 2,
                    y: area.y + popup_height - 1,
                    width: area.width - 4,
                    height: 1,
                };

                buf.set_string(
                    hint_area.x,
                    hint_area.y,
                    "↑/↓: Navigate  Tab: Complete  Enter: Execute",
                    Style::default().fg(Color::DarkGray),
                );
            }
        } else if !self.command_is_valid {
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