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