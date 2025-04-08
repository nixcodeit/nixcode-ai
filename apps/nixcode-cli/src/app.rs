use crate::command_popup::CommandPopup;
use crate::input_mode::InputMode;
use crate::model_popup::ModelPopup;
use crate::utils::highlights::THEME;
use crate::widgets::chat::Chat;
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use nixcode::events::NixcodeEvent;
use nixcode::project::Project;
use nixcode::{NewNixcodeResult, Nixcode};
use nixcode_llm_sdk::message::anthropic::events::ErrorEventContent;
use nixcode_llm_sdk::models::llm_model::LLMModel;
use nixcode_llm_sdk::providers::LLMProvider;
use ratatui::prelude::{Color, Modifier, Stylize};
use ratatui::widgets::Block;
use ratatui::{DefaultTerminal, Frame};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[allow(dead_code)]
pub enum AppEvent {
    SetInputMode(InputMode),
    Command(String),
    UpdateChatWidgets,
    RetryLastMessage,
    RemoveLastMessage,
    ClearChat,
    ShowModelPopup,
    ChangeModel(&'static LLMModel),
    Quit,
    Render,
    ChatError(ErrorEventContent),
}

enum AppView {
    Chat,
}

pub struct App {
    should_quit: bool,
    chat_view: Chat,

    current_view: AppView,
    input_mode: InputMode,

    rx: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,

    nixcode_rx: tokio::sync::mpsc::UnboundedReceiver<NixcodeEvent>,
    nixcode: Arc<Nixcode>,

    command_popup: CommandPopup,
    model_popup: Option<ModelPopup>,
    is_changing_model: bool,
}

impl App {
    pub(crate) fn new(nixcode: NewNixcodeResult) -> Result<Self> {
        let input_mode = InputMode::Normal;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();

        let (nixcode_rx, client) = nixcode;
        let nixcode = Arc::new(client);
        let chat = Chat::new(nixcode.clone(), input_mode, tx.clone());

        Ok(App {
            input_mode,
            should_quit: false,
            current_view: AppView::Chat,
            command_popup: CommandPopup::new(tx.clone()),
            model_popup: None,
            is_changing_model: false,
            chat_view: chat,
            nixcode,
            rx,
            tx,
            nixcode_rx,
        })
    }

    async fn handle_input_events(&mut self, event: Event) {
        // If model popup is open, handle its events first
        if self.model_popup.is_some() {
            let keep_popup_open = self
                .model_popup
                .as_mut()
                .unwrap()
                .handle_input_event(&event);
            if !keep_popup_open {
                self.model_popup = None;
            }
            return;
        }

        match self.current_view {
            AppView::Chat => {
                self.chat_view
                    .handle_input_events(self.input_mode, &event)
                    .await
            }
            _ => todo!(),
        }

        match self.input_mode {
            InputMode::Insert => self.handle_insert_input_events(&event),
            InputMode::Normal => self.handle_normal_input_events(&event),
            InputMode::Command => self.handle_command_input_events(event),
        }
    }

    fn handle_esc_normal_mode(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => {
                    self.set_input_mode(InputMode::Normal);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn handle_insert_input_events(&mut self, event: &Event) {
        self.handle_esc_normal_mode(event);
    }

    fn handle_command_input_events(&mut self, event: Event) {
        self.handle_esc_normal_mode(&event);
        self.command_popup.handle_input_event(&event);
    }

    fn handle_normal_input_events(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char(':') => {
                    self.set_input_mode(InputMode::Command);
                }
                KeyCode::Char('i') => {
                    self.set_input_mode(InputMode::Insert);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn set_input_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
        self.chat_view.set_input_mode(input_mode);
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> tokio::io::Result<()> {
        let mut events = EventStream::new();

        while !self.should_quit {
            self.draw(terminal).await?;

            tokio::select! {
                Some(event) = self.rx.recv() => {
                    self.handle_app_event(event).await;
                },
                Some(Ok(event)) = events.next() => {
                    self.handle_input_events(event).await;
                },
                Some(nixcode_event) = self.nixcode_rx.recv() => {
                    self.handle_nixcode_event(nixcode_event).await;
                }
            }
        }

        Ok(())
    }

    async fn handle_nixcode_event(&mut self, event: NixcodeEvent) {
        log::debug!("Received nixcode event: {:?}", event);
        match event {
            NixcodeEvent::GeneratedResponse => {
                let nixcode = self.nixcode.clone();
                tokio::spawn(async move {
                    nixcode.execute_tools().await;
                });
                self.chat_view.update_chat_widgets().await;
            }
            NixcodeEvent::ToolsFinished => {
                let nixcode = self.nixcode.clone();
                tokio::spawn(async move {
                    nixcode.send_tools_results().await;
                });
                self.chat_view.update_chat_widgets().await;
            }
            NixcodeEvent::Error(error) => {
                self.tx.send(AppEvent::ChatError(error.into())).ok();
            }
            _ => self.chat_view.update_chat_widgets().await,
        }
    }

    async fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::SetInputMode(mode) => self.set_input_mode(mode),
            AppEvent::Command(command) => self.execute_command(command).await,
            AppEvent::Quit => self.quit(),
            AppEvent::UpdateChatWidgets => self.chat_view.update_chat_widgets().await,
            AppEvent::Render => (),
            AppEvent::RetryLastMessage => self.chat_view.retry_last_message().await,
            AppEvent::ClearChat => self.chat_view.clear_chat().await,
            AppEvent::RemoveLastMessage => self.chat_view.remove_last_message().await,
            AppEvent::ShowModelPopup => self.show_model_popup(),
            AppEvent::ChangeModel(model) => self.change_model(model).await,
            AppEvent::ChatError(error) => self.chat_view.on_error(error).await,
        }
    }

    async fn draw(&mut self, terminal: &mut DefaultTerminal) -> tokio::io::Result<()> {
        terminal.draw(|frame| self.draw_frame(frame))?;

        Ok(())
    }

    fn draw_frame(&mut self, frame: &mut Frame) {
        let area = frame.area();
        use crate::status_bar::StatusBar;
        use ratatui::layout::Constraint::*;
        use ratatui::layout::{Layout, Position};
        let vertical = Layout::vertical([Min(1), Length(1)]);
        let [main_area, status_area] = vertical.areas(area);

        if let Some(bg) = THEME.settings.background {
            let c = Color::Rgb(bg.r, bg.g, bg.b);
            frame.render_widget(Block::new().bg(c), frame.area());
        }

        match self.current_view {
            AppView::Chat => self.chat_view.render_frame(frame, main_area),
        }

        // Render model status in status bar including current model
        frame.render_widget(
            StatusBar::new(self.input_mode).with_model(self.nixcode.get_model()),
            status_area,
        );

        let mut cursor_position: Option<Position> = None;

        if let InputMode::Command = self.input_mode {
            let popup_area = crate::popup_utils::popup_area(area, 60);
            let (x, y) = self.command_popup.get_input_position(popup_area);
            cursor_position = Some(Position::new(x, y));

            frame.render_widget(Block::new().add_modifier(Modifier::DIM), main_area);
            frame.render_widget(&self.command_popup, popup_area);
        } else if let InputMode::Insert = self.input_mode {
            let (x, y) = self.chat_view.get_cursor_position(main_area);
            cursor_position = Some(Position::new(x, y));
        }

        // Render model popup if it's active
        if let Some(model_popup) = &self.model_popup {
            frame.render_widget(Block::new().add_modifier(Modifier::DIM), frame.area());
            frame.render_widget(model_popup, frame.area());
            cursor_position = None; // Don't show cursor in model popup
        }

        if let Some(cursor_position) = cursor_position {
            frame.set_cursor_position(cursor_position);
        }
    }

    async fn execute_command(&mut self, command: String) {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        let main_command = parts[0];

        match main_command {
            "quit" => self.quit(),
            "clear" => {
                self.tx.send(AppEvent::ClearChat).ok();
            }
            "retry" => {
                self.tx.send(AppEvent::RetryLastMessage).ok();
            }
            "remove-last-message" => {
                self.tx.send(AppEvent::RemoveLastMessage).ok();
            }
            "model" => {
                self.tx.send(AppEvent::ShowModelPopup).ok();
            }
            _ => {
                log::warn!("Command not implemented: {}", command);
            }
        }

        self.set_input_mode(InputMode::Normal);
    }

    fn show_model_popup(&mut self) {
        let current_model = self.nixcode.get_model();
        self.model_popup = Some(ModelPopup::new(self.tx.clone(), current_model));
    }

    async fn change_model(&mut self, model: &'static LLMModel) {
        if self.is_changing_model {
            return; // Prevent concurrent model changes
        }

        self.is_changing_model = true;

        // Create a new Nixcode instance with the selected model
        if let Ok(()) = self.nixcode.reset().await {
            // Create a new Nixcode client with the new model
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let project = Project::new(cwd);
            let mut config = self.nixcode.get_config().clone();

            // Update the config to use the provider of the selected model
            let provider = match model.provider() {
                LLMProvider::Anthropic => "anthropic",
                LLMProvider::OpenAI => "openai",
                LLMProvider::Groq => "groq",
                LLMProvider::OpenRouter => "open_router",
                LLMProvider::Gemini => "gemini",
            };

            // Set the default provider in the config to match the model's provider
            config.llm.default_provider = provider.to_string();

            // Create a new client with the updated config and model
            match Nixcode::new_with_config(project, config) {
                Ok((new_rx, client)) => {
                    // Update the client with the new model
                    let nixcode = Arc::new(client.with_model(model));

                    // Update the current Nixcode instance
                    self.nixcode = nixcode.clone();
                    self.nixcode_rx = new_rx;

                    // Update the chat view with the new Nixcode instance
                    self.chat_view.update_nixcode(nixcode);

                    // Update chat widgets
                    self.chat_view.update_chat_widgets().await;

                    log::info!("Model changed to {} with provider {}", model, provider);
                }
                Err(e) => {
                    log::error!("Failed to change model: {:?}", e);
                }
            }
        }

        self.is_changing_model = false;
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn show_chat_view(&mut self) {
        self.current_view = AppView::Chat;
    }

    fn show_help(&mut self) {
        // A placeholder for future help implementation
        // For now, we'll just return to normal mode
    }
}
