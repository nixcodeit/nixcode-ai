use crate::app::AppEvent;
use crate::input_mode::InputMode;
use crate::user_input::UserSingleLineInput;
use crate::widgets::message_widget::MessageWidget;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use nixcode::Nixcode;
use nixcode_llm_sdk::message::anthropic::events::ErrorEventContent;
use nixcode_llm_sdk::message::common::llm_message::LLMMessage;
use nixcode_llm_sdk::message::usage::AnthropicUsage;
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
    lines: Vec<Line<'static>>,
    paragraph: Paragraph<'static>,
    client: Arc<Nixcode>,
    input_mode: InputMode,
    app_event: UnboundedSender<AppEvent>,
    prompt: UserSingleLineInput,
    area_size: (u16, u16), // (width, height)
    stick_to_bottom: bool,
    scroll: usize,      // Simplified to a single value for vertical scrolling
    total_lines: usize, // Keep track of total line count
    usage: AnthropicUsage,
    waiting: bool,
    error: Option<ErrorEventContent>,
    total_cost: f64,
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
            scroll: 0,
            lines: Vec::new(),
            paragraph: Paragraph::new(Vec::new()),
            stick_to_bottom: true,
            area_size: (0, 0),
            total_lines: 0,
            usage: AnthropicUsage::default(),
            waiting: false,
            error: None,
            total_cost: 0.0,
        }
    }

    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    // Add method to update the Nixcode instance
    pub fn update_nixcode(&mut self, client: Arc<Nixcode>) {
        self.client = client;
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

    pub async fn update_chat_widgets(&mut self) {
        let messages = self.client.get_messages().await;
        let llm_error = self.client.get_error().await;
        self.usage = self.client.get_usage().await;
        let mut lines: Vec<Line> = messages
            .clone()
            .into_iter()
            .flat_map(MessageWidget::get_lines)
            .collect();

        self.waiting = self.client.is_waiting().await;
        self.total_cost = messages
            .iter()
            .filter_map(|x| x.usage.clone())
            .map(|x| x.cost)
            .sum::<f64>()
            .max(0.0);

        if let Some(error) = llm_error {
            lines.push(Line::raw(format!("Error: {:?}", error)).red().bold());
        }

        self.paragraph = Paragraph::new(lines.clone()).wrap(Wrap { trim: false });

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

    pub fn set_area_size(&mut self, size: (u16, u16)) {
        let old_size = self.area_size;
        self.area_size = size;

        // If the area width changed, we need to recalculate line wrapping
        if old_size.0 != size.0 {
            return;
        }

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

    async fn send_message(&mut self, message: Option<LLMMessage>) {
        let client = self.client.clone();

        tokio::spawn(async move {
            client.send_message(message).await;
        });
    }

    async fn send_user_message(&mut self) {
        if self.client.is_waiting().await {
            return;
        }

        let message_text = self.prompt.as_string().trim().to_string();
        if message_text.is_empty() {
            return;
        }

        // Create a regular LLM message
        let message = LLMMessage::user()
            .with_text(message_text.clone())
            .to_owned();
        self.prompt.flush();

        self.send_message(Some(message)).await;
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

        // Add provider info to the title
        let model = self.client.get_model();
        let provider = model.provider().name();

        let mut title_line_spans = vec![Span::from(format!(" Chat [{} / {}]", provider, model))];

        if self.client.get_project().has_repo_path() {
            title_line_spans.push(Span::styled(" [git] ", Style::new().green().bold()))
        } else {
            title_line_spans.push(Span::styled(" [git] ", Style::new().red()))
        }

        let mut main_area = Block::bordered()
            .title(Line::from(title_line_spans))
            .border_type(BorderType::Rounded)
            .title_bottom(Line::raw(format!(" ${:.4} ", self.total_cost)).right_aligned());

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

    pub async fn clear_chat(&mut self) {
        if let Err(_) = self.client.clone().reset().await {
            return;
        }

        self.lines.clear();
        self.paragraph = Paragraph::new(Vec::new());
        self.vertical_scroll_state = ScrollbarState::default();
        self.scroll = 0;
        self.total_lines = 0;
        self.usage = AnthropicUsage::default();
    }

    /// Retry last message that was sent by the user
    pub async fn retry_last_message(&mut self) {
        let client = self.client.clone();
        tokio::spawn(async move {
            client.retry_last_message().await;
        });
        self.update_chat_widgets().await;
    }

    pub async fn remove_last_message(&mut self) {
        self.client.remove_last_message().await;
        self.update_chat_widgets().await;
    }

    pub async fn on_error(&mut self, error: ErrorEventContent) {
        self.error = Some(error);
        self.update_chat_widgets().await;
    }
}
