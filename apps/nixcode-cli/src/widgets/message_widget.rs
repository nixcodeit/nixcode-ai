use crate::utils::highlights::highlight_code;
use nixcode_llm_sdk::message::common::llm_message::LLMMessage;
use ratatui::prelude::*;
use ratatui::text::Line;
use serde_json::Value;
use syntect::util::LinesWithEndings;

pub struct MessageWidget {}

fn format_text(text: String, author: Option<Span>) -> Vec<Line> {
    if text.is_empty() {
        return vec![];
    }

    let parsed_text = highlight_code(text.clone(), "md");

    if let Ok(mut t) = parsed_text {
        if t.len() > 0 {
            let first_lane = t.get_mut(0).unwrap();
            let mut spans = if let Some(author) = author.clone() {
                vec![author.clone()]
            } else {
                vec![]
            };
            spans.extend(first_lane.clone().spans);
            *first_lane = Line::from(spans);
        }

        t.push(Line::from(vec![]));

        return t;
    }

    let mut lines: Vec<Line> = vec![];
    let x: Vec<String> = text.split("\n").map(|x| x.trim_end().to_string()).collect();

    for i in 0..x.len() {
        if i == 0 {
            if let Some(author) = author.clone() {
                lines.push(Line::from(vec![author, Span::raw(x[i].clone())]));
                continue;
            }
        }

        lines.push(Line::from(x[i].clone()));
    }

    lines.push(Line::from(vec![]));

    lines
}

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
                Value::String(s) if s.len() > 100 || s.contains("\n") => {
                    "[long content]".to_string()
                }
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

    pub fn get_lines<'a>(message: LLMMessage) -> Vec<Line<'a>> {
        let author = match message.role.as_str() {
            "user" => Span::styled("You > ", Style::new().green()),
            "assistant" => Span::styled("Assistant > ", Style::new().yellow()),
            "system" | "developer" => Span::styled("System > ", Style::new().dark_gray()),
            _ => Span::styled("Unknown > ", Style::new().red()),
        }
        .bold();

        let mut lines = vec![];

        if let Some(text) = message.reasoning {
            let mut reasoning_lines = vec![];

            for line_str in LinesWithEndings::from(text.as_str()) {
                reasoning_lines.push(
                    Line::from(vec![Span::raw(String::from(line_str))])
                        .italic()
                        .gray(),
                );
            }

            lines.extend(reasoning_lines);
        }

        if let Some(text) = message.text {
            lines.extend(format_text(text, Some(author.clone())));
        }

        if let Some(tool_calls) = message.tool_calls {
            for tool_call in tool_calls {
                let (name, params) = tool_call.get_execute_params();
                let formatted_params = Self::format_tool_params(&params);
                let tool_info = format!("{}({})", name, formatted_params);

                lines.push(Line::from(tool_info).bold());
            }
        }

        if let Some(tools_results) = message.tool_results {
            for tool_result in tools_results {
                let x = tool_result
                    .call_id
                    .unwrap_or_else(|| "unknown id".to_string());
                let content = tool_result.result;
                let split_iterator = content.split("\n");
                let total_lines = split_iterator.clone().count();
                let mut lines2 = vec![Line::from(Span::raw(format!("[{}] ", x)).bold())];

                split_iterator
                    .take(5)
                    .for_each(|line| lines2.push(Line::from(String::from(line))));

                let missing_lines = total_lines.saturating_sub(5);
                if missing_lines > 0 {
                    lines2.push(Line::from(format!("... {} more lines", missing_lines)).italic());
                }

                lines2.push(Line::from(vec![]));
                lines.extend(lines2);
            }
        }

        lines
    }
}
