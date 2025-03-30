use nixcode_llm_sdk::message::content::tools::ToolUseState;
use nixcode_llm_sdk::message::content::Content;
use nixcode_llm_sdk::message::message::Message;
use ratatui::prelude::*;
use ratatui::text::Line;
use serde_json::Value;

pub struct MessageWidget {}

impl MessageWidget {
    // Helper function to format tool parameters
    fn format_tool_params(params: &Value) -> String {
        if !params.is_object() {
            return String::new();
        }

        let obj = params.as_object().unwrap();
        let mut formatted_params = Vec::new();

        for (key, value) in obj {
            let formatted_value = match value {
                Value::String(s) if s.len() > 50 => "[long content]".to_string(),
                Value::String(s) => serde_json::to_string(s).unwrap_or(format!("\"{}\"", s)),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                Value::Array(_) => "[array]".to_string(),
                Value::Object(_) => "{object}".to_string(),
            };
            formatted_params.push(format!("{}: {}", key, formatted_value));
        }

        formatted_params.join(", ")
    }

    pub fn get_lines<'a>(message: Message) -> Vec<Line<'a>> {
        let author = match message {
            Message::User { .. } => Span::styled("You > ", Style::new().green()),
            Message::Assistant { .. } => Span::styled("Assistant > ", Style::new().yellow()),
            Message::System { .. } => Span::styled("System > ", Style::new().dark_gray()),
        }
        .bold();

        let lines: Vec<Line> = message
            .get_content()
            .into_iter()
            .flat_map(|content| match content {
                Content::Thinking(content) => {
                    vec![
                        Line::from(vec![Span::raw("Thinking: "), Span::raw(content.get_text())])
                            .italic(),
                        Line::from(vec![]),
                    ]
                }
                Content::Text(text) => {
                    let mut lines: Vec<Line> = vec![];
                    let x: Vec<String> = text
                        .get_text()
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
                }
                Content::ToolUse(tool_use) => {
                    let (_, params) = tool_use.get_execute_params();
                    let formatted_params = Self::format_tool_params(&params);
                    let tool_info = if formatted_params.is_empty() {
                        format!("[{}]", tool_use.get_tool_name())
                    } else {
                        format!("[{}]({})", tool_use.get_tool_name(), formatted_params)
                    };

                    vec![
                        match tool_use.get_state() {
                            ToolUseState::Created => {
                                Line::from(format!("{} waiting", tool_info)).bold()
                            }
                            ToolUseState::Executing => {
                                Line::from(format!("{} executing", tool_info)).bold()
                            }
                            ToolUseState::Executed => {
                                Line::from(format!("{} finished", tool_info)).bold()
                            }
                            ToolUseState::Error => {
                                Line::from(format!("{} failed", tool_info)).bold()
                            }
                        },
                        Line::from(vec![]),
                    ]
                }
                Content::ToolResult(tool_result) => {
                    let content = tool_result.get_content();
                    let split_iterator = content.split("\n");
                    let total_lines = split_iterator.clone().count();
                    let mut lines = vec![Line::from(Span::raw(format!(
                        "[{}] ",
                        tool_result.get_tool_use_id()
                    )))];

                    split_iterator
                        .take(5)
                        .for_each(|line| lines.push(Line::from(String::from(line))));

                    let missing_lines = total_lines.saturating_sub(5);
                    if missing_lines > 0 {
                        lines
                            .push(Line::from(format!("... {} more lines", missing_lines)).italic());
                    }

                    lines.push(Line::from(vec![]));

                    lines
                }
                _ => vec![Line::from("Unknown content type".to_string())],
            })
            .collect();

        lines
    }
}
