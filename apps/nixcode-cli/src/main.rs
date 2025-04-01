use app::App;
use dotenv::dotenv;
use nixcode::project::Project;
use nixcode::Nixcode;
use std::env::current_dir;
use std::path::PathBuf;

mod app;
mod command_popup;
mod input_mode;
mod popup_utils;
mod status_bar;
mod user_input;
mod widgets;
mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Create project from current directory
    let project = Project::new(current_dir().unwrap_or(PathBuf::from(".")));

    // Create Nixcode client with config from environment or files
    let nixcode_result = Nixcode::new_from_env(project);

    // Check if Nixcode creation was successful
    let nixcode = match nixcode_result {
        Ok(client) => client,
        Err(err) => {
            let msg = format!("{:?}", err);
            eprintln!("Failed to initialize nixcode client: {}", msg);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, msg));
        }
    };

    // Initialize terminal UI
    let mut terminal = ratatui::init();

    // Create app with the nixcode client
    let mut app = App::new(nixcode).expect("Failed to create app");

    // Run the application
    let app_result = app.run(&mut terminal).await;

    // Restore terminal state
    ratatui::restore();

    app_result
}
