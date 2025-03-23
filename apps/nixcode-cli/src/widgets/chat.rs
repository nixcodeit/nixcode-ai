use std::sync::Arc;
use crate::app::AppEvent;
use crate::input_mode::InputMode;
use crate::user_input::UserSingleLineInput;
use crate::widgets::message_widget::MessageWidget;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use nixcode::Nixcode;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use nixcode_llm_sdk::message::message::Message::{Assistant, User};
use nixcode_llm_sdk::message::response::MessageResponse;
use nixcode_llm_sdk::MessageResponseStreamEvent;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;
use nixcode::project::Project;
use nixcode_llm_sdk::config::LLMConfig;
use nixcode_llm_sdk::message::content::tools::{ToolResultContent, ToolUseContent, ToolUseState};
use nixcode_llm_sdk::tools::Tool;

pub struct Chat {
    vertical_scroll_state: ScrollbarState,
    messages: Vec<Message>,
    lines: Vec<Line<'static>>,
    paragraph: Paragraph<'static>,
    client: Arc<Nixcode>,
    input_mode: InputMode,
    app_event: UnboundedSender<AppEvent>,
    last_message_response: Option<MessageResponse>,
    tools_to_execute: Vec<ToolUseContent>,
    tools_results: Vec<ToolResultContent>,
    prompt: UserSingleLineInput,
    waiting: bool,
    area_size: (u16, u16),
    stick_to_bottom: bool,
    scroll: (u16, u16),
}

impl Chat {
    pub fn new(project: Project, input_mode: InputMode, app_event: UnboundedSender<AppEvent>) -> Result<Self> {
        let config = LLMConfig::new_anthropic()?;

        Ok(Chat {
            vertical_scroll_state: ScrollbarState::default(),
            client: Arc::new(Nixcode::new_anthropic(project, config).unwrap()),
            input_mode,
            app_event,
            prompt: Default::default(),
            messages: Vec::new(),
            scroll: (0, 0),
            waiting: false,
            lines: Vec::new(),
            paragraph: Paragraph::new(Vec::new()),
            stick_to_bottom: true,
            area_size: (0, 0),
            last_message_response: None,
            tools_results: vec![],
            tools_to_execute: vec![],
        })
    }

    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    pub async fn handle_input_events(&mut self, input_mode: InputMode, event: &Event) {
        self.set_input_mode(input_mode);

        match self.input_mode {
            InputMode::Normal => self.handle_normal_input_events(event).await,
            InputMode::Insert => self.handle_insert_input_events(event).await,
            _ => (),
        }
    }

    async fn handle_normal_input_events(&mut self, event: &Event) {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('j') | KeyCode::Up => self.scroll_up(),
                KeyCode::Char('k') | KeyCode::Down => self.scroll_down(),
                _ => (),
            },
            _ => (),
        }
    }

    async fn handle_insert_input_events(&mut self, event: &Event) {
        self.prompt.handle_input_events(event);

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => {
                    self.send_user_message().await;
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn add_chunk(&mut self, message: MessageResponseStreamEvent) {
        if self.last_message_response.is_none() || self.messages.last().is_none() {
            return;
        }

        let last_message = self.messages.last_mut().unwrap();
        let last_response = self.last_message_response.as_mut().unwrap();
        match message {
            MessageResponseStreamEvent::MessageStart(msg) => {
                *last_response += msg;
            },
            MessageResponseStreamEvent::MessageDelta(delta) => {
                *last_response += delta;
            },
            MessageResponseStreamEvent::ContentBlockStart(content) => {
                *last_response += content;
            },
            MessageResponseStreamEvent::ContentBlockDelta(delta) => {
                *last_response += delta;
            },
            MessageResponseStreamEvent::ContentBlockStop(content) => {
                if let Content::ToolUse(tool_use) = last_response.get_content(content.index) {
                    self.app_event.send(AppEvent::ToolAddToExecute(tool_use)).ok();
                }
            },
            MessageResponseStreamEvent::MessageStop => {
                self.app_event.send(AppEvent::ExecuteTools).ok();
            }
            _ => (),
        }

        last_message.set_content(last_response.content.clone());

        self.update_chat_widgets();
    }

    fn update_chat_widgets(&mut self) {
        let lines: Vec<Line> = self.messages
            .clone()
            .into_iter()
            .flat_map(MessageWidget::get_lines)
            .collect();

        let paragraph = Paragraph::new(lines.clone()).scroll(self.scroll)
            .wrap(Wrap { trim: true });

        let line_count = paragraph
            .line_count(self.area_size.0).saturating_sub(self.area_size.0 as usize);

        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(line_count);

        self.lines = lines;
        self.paragraph = paragraph;

        if self.stick_to_bottom {
            self.scroll_to_bottom();
        }
    }

    pub fn handle_llm_error(&mut self, error: LLMError) {
        self.waiting = false;
        self.last_message_response = None;
        eprintln!("{:?}", error);
    }

    pub fn set_vertical_scroll(&mut self, scroll: u16) {
        self.scroll.0 = scroll;
        self.vertical_scroll_state = self.vertical_scroll_state.position(scroll as usize);
    }

    pub fn scroll_up(&mut self) {
        self.set_vertical_scroll(self.scroll.0.saturating_sub(1));

        self.stick_to_bottom = self.scroll.0 >= self.get_bottom_position();
    }

    pub fn scroll_down(&mut self) {
        self.set_vertical_scroll(self.scroll.0.saturating_add(1));

        self.stick_to_bottom = self.scroll.0 >= self.get_bottom_position();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.set_vertical_scroll(self.get_bottom_position());
    }

    pub fn get_bottom_position(&self) -> u16 {
        (self.lines.len() as u16)
            .saturating_sub(self.area_size.1)
    }

    pub fn reset_scroll(&mut self) {
        if self.stick_to_bottom {
            self.scroll_to_bottom();
            return;
        }

        self.set_vertical_scroll(0);
    }

    pub fn set_area_size(&mut self, size: (u16, u16)) {
        self.area_size = size;
        let (_, height) = size;

        self.vertical_scroll_state = self
            .vertical_scroll_state
            .viewport_content_length(height as usize);
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.update_chat_widgets();
    }

    async fn send_message(&mut self, message: Message) {
        self.add_message(message);

        let tx = self.app_event.clone();
        let messages = self.messages.clone();

        let client = self.client.clone();

        tokio::spawn({
            async move {
                tx.send(AppEvent::ChatGeneratingResponse).ok();
                let response = client.send(messages).await;
                if let Err(err) = response {
                    tx.send(AppEvent::ChatError(err)).ok();
                    return;
                }

                let mut response = response.unwrap();
                while let Some(data) = response.recv().await {
                    tx.send(AppEvent::ChatChunk(data)).ok();
                }

                tx.send(AppEvent::ChatGeneratedResponse).ok();
            }
        });
    }

    async fn send_user_message(&mut self) {
        let message = self.prompt.as_string();
        let message = User(Content::new_text(message).into());
        self.prompt.flush();

        self.send_message(message).await;
    }

    pub fn handle_message_chunk(&mut self, chunk: MessageResponseStreamEvent) {
        self.add_chunk(chunk);
    }

    pub fn waiting_for_response(&mut self) {
        self.waiting = true;
        self.last_message_response = Some(MessageResponse::default());
        self.messages.push(Assistant(vec![]));
    }

    pub fn generated_response(&mut self) {
        self.waiting = false;
    }

    fn get_layout(&self, area: Rect) -> [Rect; 3] {
        let horizontal = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]);
        let [chat, input] = horizontal.areas(area);
        let input_inner = input.inner(Margin::new(1, 1));

        [chat, input, input_inner]
    }

    pub fn get_cursor_position(&self, area: Rect) -> (u16, u16) {
        let [_, _, input] = self.get_layout(area);
        self.prompt.get_cursor_position(input)
    }

    fn render_chat(&mut self, frame: &mut Frame, area: Rect) {
        let inner = area.inner(Margin::new(1, 1));
        self.set_area_size((inner.width, inner.height));

        let mut main_area = Block::bordered().title("Chat");

        if self.waiting {
            main_area = main_area.title_bottom(
                Span::styled(" Waiting for response ", Style::new().bold().italic())
                    .add_modifier(Modifier::SLOW_BLINK)
                    .add_modifier(Modifier::DIM)
            );
        }

        let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_widget(main_area, area);
        frame.render_widget(&self.paragraph, inner);
        frame.render_stateful_widget(scroll, inner, &mut self.vertical_scroll_state);
    }

    pub fn render_frame(&mut self, frame: &mut Frame, area: Rect) {
        let [chat_area, input_area, input_inner_area] = self.get_layout(area);

        self.render_chat(frame, chat_area);

        frame.render_widget(Block::bordered().title("Input"), input_area);
        frame.render_widget(&self.prompt, input_inner_area);
    }

    pub fn add_tool_to_execute(&mut self, tool: ToolUseContent) {
        self.tools_to_execute.push(tool);
    }

    pub fn start_tool(&mut self, tool: ToolUseContent) {
        let last_message = self.messages.last_mut().unwrap();
        last_message.set_tool_state(tool.get_id(), ToolUseState::Executing);
    }

    pub async fn tool_finished(&mut self, result: ToolResultContent) {
        let tool_id = result.get_tool_use_id();
        self.tools_results.push(result);

        let last_message = self.messages.last_mut().unwrap();
        last_message.set_tool_state(tool_id, ToolUseState::Executed);

        if self.tools_results.len() == self.tools_to_execute.len() {
            self.send_tools_results().await;
        }
    }

    pub fn execute_tools(&mut self) {
        let tools = self.tools_to_execute.clone();

        for tool in tools {
            tokio::spawn({
                let client = self.client.clone();
                let tx = self.app_event.clone();

                async move {
                    let (name, props) = tool.get_execute_params();
                    tx.send(AppEvent::ToolStart(tool.clone())).ok();
                    let result = client.execute_tool(name.as_str(), props);
                    let result = if let Ok(value) = result {
                        let value = serde_json::from_value(value).unwrap_or_else(|e| e.to_string());
                        tool.create_response(value)
                    } else {
                        tool.create_response("Error executing tool".to_string())
                    };
                    tx.send(AppEvent::ToolEnd(result)).ok();
                }
            });
        }
    }

    async fn send_tools_results(&mut self) {
        let contents = self.tools_results.clone();
        self.tools_results.clear();
        self.tools_to_execute.clear();

        let message = User(Content::new_tool_results(contents));

        self.send_message(message).await;
    }
}
