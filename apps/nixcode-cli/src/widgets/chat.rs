use crate::app::AppEvent;
use crate::input_mode::InputMode;
use crate::user_input::UserSingleLineInput;
use crate::widgets::message_widget::MessageWidget;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use nixcode::Nixcode;
use nixcode_llm_sdk::errors::llm::LLMError;
use nixcode_llm_sdk::message::content::tools::{ToolResultContent, ToolUseContent, ToolUseState};
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use nixcode_llm_sdk::message::message::Message::{Assistant, User};
use nixcode_llm_sdk::message::response::MessageResponse;
use nixcode_llm_sdk::message::usage::Usage;
use nixcode_llm_sdk::{ErrorContent, MessageResponseStreamEvent};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::prelude::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

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
    area_size: (u16, u16), // (width, height)
    stick_to_bottom: bool,
    scroll: usize,      // Simplified to a single value for vertical scrolling
    total_lines: usize, // Keep track of total line count
    llm_error: Option<ErrorContent>,
    usage: Usage,
}

impl Chat {
    pub fn new(
        client: Arc<Nixcode>,
        input_mode: InputMode,
        app_event: UnboundedSender<AppEvent>,
    ) -> Self {
        Chat {
            vertical_scroll_state: ScrollbarState::default(),
            client,
            input_mode,
            app_event,
            prompt: Default::default(),
            messages: Vec::new(),
            scroll: 0,
            waiting: false,
            lines: Vec::new(),
            paragraph: Paragraph::new(Vec::new()),
            stick_to_bottom: true,
            area_size: (0, 0),
            last_message_response: None,
            tools_results: vec![],
            tools_to_execute: vec![],
            total_lines: 0,
            usage: Usage::default(),
            llm_error: None,
        }
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
                KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
                KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
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
                self.usage += last_response.usage.clone();
            }
            MessageResponseStreamEvent::MessageDelta(delta) => {
                self.usage.output_tokens += delta.get_usage().output_tokens;
                *last_response += delta;
            }
            MessageResponseStreamEvent::ContentBlockStart(content) => {
                *last_response += content;
            }
            MessageResponseStreamEvent::ContentBlockDelta(delta) => {
                *last_response += delta;
            }
            MessageResponseStreamEvent::ContentBlockStop(content) => {
                if let Content::ToolUse(tool_use) = last_response.get_content(content.index) {
                    self.app_event
                        .send(AppEvent::ToolAddToExecute(tool_use))
                        .ok();
                }
            }
            MessageResponseStreamEvent::MessageStop => {
                self.app_event.send(AppEvent::ExecuteTools).ok();
            }
            MessageResponseStreamEvent::Error { error } => {
                self.llm_error = Some(error.clone());
            }
            _ => (),
        }

        last_message.set_content(last_response.content.clone());

        self.update_chat_widgets();
    }

    fn update_chat_widgets(&mut self) {
        let mut lines: Vec<Line> = self
            .messages
            .clone()
            .into_iter()
            .flat_map(MessageWidget::get_lines)
            .collect();

        if let Some(error) = &self.llm_error {
            lines.push(Line::raw(format!("Error: {:?}", error)).red().bold());
        }

        self.paragraph = Paragraph::new(lines.clone()).wrap(Wrap { trim: true });

        // Calculate the total line count based on the content and area width
        let total_lines = if self.area_size.0 > 0 {
            let line_width = self.area_size.0 as usize;
            self.paragraph.line_count(line_width as u16)
        } else {
            lines.len()
        };

        self.total_lines = total_lines;
        self.lines = lines;

        // Update scrollbar state with new content length
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(self.total_lines.saturating_sub(self.area_size.1 as usize))
            .viewport_content_length(self.area_size.1 as usize);

        // If sticking to bottom, update scroll position
        if self.stick_to_bottom {
            self.scroll_to_bottom();
        }
    }

    pub fn handle_llm_error(&mut self, error: LLMError) {
        self.waiting = false;
        self.llm_error = Some(error.into());
        self.update_chat_widgets();
    }

    // Simplified to use a single scroll value
    pub fn set_vertical_scroll(&mut self, scroll: usize) {
        let max_scroll = self.get_max_scroll();
        self.scroll = scroll.min(max_scroll);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.scroll);
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.set_vertical_scroll(self.scroll - 1);
            // Only update stick_to_bottom if we're not at the bottom anymore
            self.stick_to_bottom = self.scroll >= self.get_max_scroll();
        }
    }

    pub fn scroll_down(&mut self) {
        let max_scroll = self.get_max_scroll();
        if self.scroll < max_scroll {
            self.set_vertical_scroll(self.scroll + 1);
            // Check if we reached the bottom
            self.stick_to_bottom = self.scroll >= max_scroll;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        let max_scroll = self.get_max_scroll();
        self.set_vertical_scroll(max_scroll);
        self.stick_to_bottom = true;
    }

    // Calculate maximum valid scroll position
    fn get_max_scroll(&self) -> usize {
        let viewport_height = self.area_size.1 as usize;
        if self.total_lines <= viewport_height {
            0
        } else {
            self.total_lines - viewport_height
        }
    }

    pub fn reset_scroll(&mut self) {
        if self.stick_to_bottom {
            self.scroll_to_bottom();
        } else {
            self.set_vertical_scroll(0);
        }
    }

    pub fn set_area_size(&mut self, size: (u16, u16)) {
        let old_size = self.area_size;
        self.area_size = size;

        // If the area width changed, we need to recalculate line wrapping
        if old_size.0 != size.0 {
            self.update_chat_widgets();
        } else {
            // Just update the scrollbar viewport height
            self.vertical_scroll_state = self
                .vertical_scroll_state
                .viewport_content_length(size.1 as usize);

            // Check if we need to adjust scroll position
            if self.stick_to_bottom {
                self.scroll_to_bottom();
            } else {
                // Make sure scroll position is still valid
                self.set_vertical_scroll(self.scroll);
            }
        }
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.update_chat_widgets();
    }

    async fn send_message(&mut self, message: Option<Message>) {
        self.llm_error = None;
        if let Some(message) = message {
            self.add_message(message);
        }

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
        if self.waiting {
            return;
        }

        let message = self.prompt.as_string().trim().to_string();
        if message.is_empty() {
            return;
        }

        let message = User(Content::new_text(message).into());
        self.prompt.flush();

        self.send_message(Some(message)).await;
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
        self.update_chat_widgets();
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

        let create_input_cache_cost =
            self.usage.cache_creation_input_tokens.unwrap_or(0) as f64 / 1_000_000.0 * 3.75;
        let read_input_cache_cost =
            self.usage.cache_read_input_tokens.unwrap_or(0) as f64 / 1_000_000.0 * 0.30;
        let input_cost = self.usage.input_tokens as f64 / 1_000_000.0 * 3.0;
        let output_cost = self.usage.output_tokens as f64 / 1_000_000.0 * 15.0;
        let total_cost = create_input_cache_cost + read_input_cache_cost + input_cost + output_cost;

        let cache_write_tokens = self.usage.cache_creation_input_tokens.unwrap_or(0);
        let cache_read_tokens = self.usage.cache_read_input_tokens.unwrap_or(0);
        let input_tokens = self.usage.input_tokens;
        let output_tokens = self.usage.output_tokens;

        // Add provider info to the title
        let provider = &self.client.get_config().llm.default_provider;
        let model = &self.client.get_model();

        let mut title_line_spans = vec![Span::from(format!(" Chat [{}/{}]", provider, model))];

        if self.client.get_project().has_repo_path() {
            title_line_spans.push(Span::styled(" [git] ", Style::new().green().bold()))
        } else {
            title_line_spans.push(Span::styled(" [git] ", Style::new().red()))
        }

        let mut main_area = Block::bordered()
            .title(Line::from(title_line_spans))
            .border_type(BorderType::Rounded)
            .title_bottom(Line::raw(format!(" ${:.4} ", total_cost)).right_aligned())
            .title_bottom(
                Line::raw(format!(
                    " Cache (R/W): ({}, {}), Input: {}, Output: {} ",
                    cache_read_tokens, cache_write_tokens, input_tokens, output_tokens
                ))
                .centered(),
            );

        if !self.client.has_init_analysis() {
            main_area = main_area.title(
                Line::from(" Project analysis not initialized ")
                    .red()
                    .bold()
                    .right_aligned(),
            );
        } else {
            main_area = main_area.title(
                Line::from(" Project analysis initialized ")
                    .green()
                    .bold()
                    .right_aligned(),
            );
        }

        if self.waiting {
            main_area = main_area.title_bottom(
                Span::styled(" Waiting for response ", Style::new().bold().italic())
                    .add_modifier(Modifier::SLOW_BLINK)
                    .add_modifier(Modifier::DIM),
            );
        }

        let scroll = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        frame.render_widget(main_area, area);

        // Apply the scroll to the paragraph
        let scrolled_paragraph = self.paragraph.clone().scroll((self.scroll as u16, 0));
        frame.render_widget(scrolled_paragraph, inner);

        frame.render_stateful_widget(scroll, inner, &mut self.vertical_scroll_state);
    }

    pub fn render_frame(&mut self, frame: &mut Frame, area: Rect) {
        let [chat_area, input_area, input_inner_area] = self.get_layout(area);

        self.render_chat(frame, chat_area);

        frame.render_widget(
            Block::bordered()
                .title(" Input ")
                .border_type(BorderType::Rounded),
            input_area,
        );
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
                    let result = client.execute_tool(name.as_str(), props).await;

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

        self.send_message(Some(message)).await;
    }

    pub fn remove_last_message(&mut self) {
        if self.waiting {
            return;
        }

        if let Some(Assistant(content)) = self.messages.last() {
            if content.is_empty() {
                // pop twice because empty content is not visible for the user
                self.messages.pop();
            }
        }

        self.messages.pop();
        self.llm_error = None;
        self.update_chat_widgets();
    }

    pub fn clear_chat(&mut self) {
        if self.waiting {
            return;
        }

        self.last_message_response = None;
        self.tools_results.clear();
        self.tools_to_execute.clear();
        self.messages.clear();
        self.lines.clear();
        self.paragraph = Paragraph::new(Vec::new());
        self.vertical_scroll_state = ScrollbarState::default();
        self.scroll = 0;
        self.total_lines = 0;
        self.usage = Usage::default();
    }

    /// Retry last message that was sent by the user
    pub async fn retry_last_message(&mut self) {
        if self.waiting {
            return;
        }

        loop {
            let last_message = self.messages.last();
            if last_message.is_none() {
                break;
            }

            if let Assistant(_) = last_message.unwrap() {
                self.messages.pop();
                continue;
            }

            break;
        }

        if self.messages.len() == 0 {
            return;
        }

        self.send_message(None).await;
    }
}
