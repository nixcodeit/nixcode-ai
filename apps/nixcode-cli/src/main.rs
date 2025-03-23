use app::App;
use colored::Colorize;
use dotenv::dotenv;
use ratatui::prelude::*;
use std::env;
use std::env::current_dir;
use std::path::PathBuf;
use tokio_stream::StreamExt;
use nixcode::project::Project;

mod app;
mod command_popup;
mod input_mode;
mod popup_utils;
mod status_bar;
mod user_input;
mod widgets;

fn validate_env_vars() {
    env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY is required");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    validate_env_vars();

    let project = Project::new(current_dir().unwrap_or(PathBuf::from(".")));

    let mut terminal = ratatui::init();
    let mut app = App::new(project).expect("Failed to create app");
    let app_result = app.run(&mut terminal).await;

    ratatui::restore();

    app_result
}

// #[tokio::main]
// async fn main() {
//     dotenv().ok();
//     validate_env_vars();
//
//     let options = LLMConfig {
//         api_key: env::var("ANTHROPIC_API_KEY").unwrap(),
//         model: "claude-3-7-sonnet-20250219".into(),
//         max_tokens: 5,
//         max_input_tokens: 8192 * 2,
//         tools: vec![Tool::new(
//             "get_weather".into(),
//             "Get the weather for a location".into(),
//             json_schema!({
//                 "location": String,
//             }),
//         )],
//     };
//
//     let client = AnthropicClient::new(options);
//
//     if client.is_err() {
//         println!("Failed to create client");
//         return;
//     }
//
//     let (_, mut client) = client.unwrap();
//
//     let mut rl = DefaultEditor::new().unwrap();
//     let mut next_line_content: Vec<Content> = vec![];
//
//     loop {
//         let readline = match rl.readline("You > ") {
//             Ok(line) => line,
//             Err(ReadlineError::Interrupted) => {
//                 println!("Interrupted, exiting...");
//                 break;
//             }
//             Err(ReadlineError::Eof) => {
//                 println!("EOF, exiting...");
//                 break;
//             }
//             Err(err) => {
//                 println!("Error: {}", err);
//                 break;
//             }
//         };
//
//         rl.add_history_entry(&readline).unwrap();
//
//         if readline.trim().to_lowercase() == "exit" {
//             println!("{}", "Goodbye!".green());
//             break;
//         }
//
//         next_line_content.push(Content::new_text(readline.trim()));
//
//         let message = User(next_line_content.clone());
//
//         next_line_content.clear();
//
//         let response = client.send(Some(message)).await;
//
//         if let Err(err) = response {
//             println!("Failed to send message, error: {:?}", err);
//             return;
//         }
//
//         let response = response.unwrap();
//
//         println!("Assistant > {}", response.get_text());
//
//         for tool in response.tools_usage() {
//             println!("Asked for tool use: {:#?}", &tool);
//             if tool.name_is("get_weather") {
//                 let tool_result = tool.create_response("23 stopnie celcjusza");
//                 let tool_result_content = Content::new_tool_result(tool_result);
//                 next_line_content.push(tool_result_content);
//             }
//         }
//
//         if !next_line_content.is_empty() {
//             let message = User(next_line_content.clone());
//
//             next_line_content.clear();
//             client
//                 .send(Some(message))
//                 .await
//                 .expect("Failed to send tool_result message");
//         }
//     }
//
//     println!("Total usage: {:?}", client.get_usage());
// }
