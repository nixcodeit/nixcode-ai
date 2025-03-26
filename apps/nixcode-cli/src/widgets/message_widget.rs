use nixcode_llm_sdk::message::content::tools::ToolUseState;
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use ratatui::prelude::*;
use ratatui::text::Line;

pub struct MessageWidget {}

impl MessageWidget {
    pub fn get_lines<'a>(message: Message) -> Vec<Line<'a>> {
        let author = match message {
            Message::User { .. } => Span::styled("You > ", Style::new().green()),
            Message::Assistant { .. } => Span::styled("Assistant > ", Style::new().yellow()),
            Message::System { .. } => Span::styled("System > ", Style::new().dark_gray()),
        }.bold();

        let lines: Vec<Line> = message.get_content()
            .into_iter()
            .flat_map(|content| {
                match content {
                    Content::Thinking(content) => {
                        vec![
                            Line::from(vec![Span::raw("Thinking: "), Span::raw(content.get_text())]).italic(),
                            Line::from(vec![]),
                        ]
                    }
                    Content::Text(text) => {
                        let mut lines: Vec<Line> = vec![];
                        let x: Vec<String> = text.get_text()
                            .split("\n")
                            .map(|x| x.trim_end().to_string())
                            .collect();

                        for i in 0..x.len() {
                            if i == 0 {
                                lines.push(Line::from(vec![author.clone(), Span::raw(x[i].clone())]));
                                continue;
                            }

                            lines.push(Line::from(x[i].clone()));
                        }

                        lines.push(Line::from(vec![]));

                        lines
                    },
                    Content::ToolUse(tool_use) => {
                        vec![
                            match tool_use.get_state() {
                                ToolUseState::Created => Line::from(format!("[{}] waiting", tool_use.get_tool_name())).bold(),
                                ToolUseState::Executing => Line::from(format!("[{}] executing", tool_use.get_tool_name())).bold(),
                                ToolUseState::Executed => Line::from(format!("[{}] finished", tool_use.get_tool_name())).bold(),
                                ToolUseState::Error => Line::from(format!("[{}] failed", tool_use.get_tool_name())).bold(),
                            },
                            Line::from(vec![]),
                        ]
                    }
                    Content::ToolResult(tool_result) => {
                        let content = tool_result.get_content();
                        let split_iterator = content.split("\n");
                        let total_lines = split_iterator.clone().count();
                        let str_lines = split_iterator.take(5).map(|x| x.to_string()).collect::<Vec<String>>();
                        let mut lines = vec![
                            Line::from(Span::raw(format!("[{}] ", tool_result.get_tool_use_id()))),
                        ];

                        for i in 0..str_lines.len() {
                            lines.push(Line::from(str_lines[i].clone()).italic());
                        }

                        let missing_lines = total_lines.saturating_sub(5);
                        if missing_lines > 0 {
                            lines.push(Line::from(format!("... {} more lines", missing_lines)).italic());
                        }

                        lines.push(Line::from(vec![]));

                        lines
                    }
                    _ => vec![Line::from("Unknown content type".to_string())],
                }
            })
            .collect();

        lines
    }
}
