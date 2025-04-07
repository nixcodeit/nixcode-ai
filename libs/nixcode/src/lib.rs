    // Create a message including reasoning if the model supports it
    pub fn create_message_with_capabilities(&self, text: String) -> LLMMessage {
        let mut message = LLMMessage::user()
            .with_text(text.clone())
            .to_owned();
            
        // If model supports thinking/reasoning, add a reasoning prompt
        if self.model.capabilities().supports_thinking() {
            // Add a specific reasoning/thinking section to engage the model's ability
            message = message.with_reasoning("".to_string()).to_owned();
        }
        
        message
    }