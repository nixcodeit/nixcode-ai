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
use ratatui::prelude::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;
use nixcode_llm_sdk::config::LLMConfig;

pub struct Chat {
    vertical_scroll_state: ScrollbarState,

    messages: Vec<Message>,
    lines: Vec<String>,
    client: Arc<Nixcode>,
    input_mode: InputMode,
    app_event: UnboundedSender<AppEvent>,
    last_message_response: Option<MessageResponse>,

    prompt: UserSingleLineInput,
    waiting: bool,

    area_height: u16,

    stick_to_bottom: bool,

    scroll: (u16, u16),
}

impl Chat {
    pub fn new(input_mode: InputMode, app_event: UnboundedSender<AppEvent>) -> Result<Self> {
        let config = LLMConfig::new_anthropic()?;

        Ok(Chat {
            vertical_scroll_state: ScrollbarState::default(),
            client: Arc::new(Nixcode::new_anthropic(config).unwrap()),
            input_mode,
            app_event,
            prompt: Default::default(),
            messages: Vec::new(),
            scroll: (0, 0),
            waiting: false,
            lines: Vec::new(),
            stick_to_bottom: true,
            area_height: 0,
            last_message_response: None,
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
        let mut last_response = self.last_message_response.as_mut().unwrap();
        match message {
            MessageResponseStreamEvent::ContentBlockStart(content) => {
                *last_response += content;
            }
            MessageResponseStreamEvent::ContentBlockDelta(delta) => {
                *last_response += delta;
            }
            MessageResponseStreamEvent::ContentBlockStop(..) => {}
            _ => (),
        }

        last_message.set_content(last_response.content.clone());

        self.recalculate_chat();
    }

    fn recalculate_chat(&mut self) {
        let lines: Vec<String> = self
            .messages
            .clone()
            .into_iter()
            .map(MessageWidget::new)
            .flat_map(|m| m.get_lines())
            .flat_map(|string_line| {
                string_line
                    .split("\n")
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>()
            })
            .collect();

        let len = lines.len().saturating_sub(self.area_height as usize);
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(len);
        self.lines = lines;
        self.scroll_to_bottom();
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
        let waiting_line = if self.waiting { 1 } else { 0 };
        (self.lines.len() as u16)
            .saturating_add(waiting_line)
            .saturating_sub(self.area_height)
    }

    pub fn reset_scroll(&mut self) {
        if self.stick_to_bottom {
            self.scroll_to_bottom();
            return;
        }

        self.set_vertical_scroll(0);
    }

    pub fn set_area_height(&mut self, height: u16) {
        self.area_height = height;
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .viewport_content_length(height as usize);
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.recalculate_chat();
    }

    async fn send_user_message(&mut self) {
        let message = self.prompt.as_string();
        let message = User(Content::new_text(message).into());
        self.add_message(message);

        let tx = self.app_event.clone();
        let messages = self.messages.clone();
        self.prompt.flush();

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
        frame.render_widget(Block::bordered().title("Chat"), area);

        let inner = area.inner(Margin::new(1, 1));
        self.set_area_height(inner.height);

        let mut lines: Vec<Line> = self.lines.iter().map(|l| Line::from(l.clone())).collect();

        if self.waiting {
            lines.push(Line::from("Waiting for response...").bold().italic().gray());
        }

        if self.stick_to_bottom {
            self.scroll_to_bottom();
        }

        let paragraph = Paragraph::new(lines).scroll(self.scroll);

        let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_widget(paragraph, inner);
        frame.render_stateful_widget(scroll, inner, &mut self.vertical_scroll_state);
    }

    pub fn render_frame(&mut self, frame: &mut Frame, area: Rect) {
        let [chat_area, input_area, input_inner_area] = self.get_layout(area);

        self.render_chat(frame, chat_area);

        frame.render_widget(Block::bordered().title("Input"), input_area);
        frame.render_widget(&self.prompt, input_inner_area);
    }
}
