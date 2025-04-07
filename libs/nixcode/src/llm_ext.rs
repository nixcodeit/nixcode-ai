use nixcode_llm_sdk::message::common::llm_message::LLMMessage;
use nixcode_llm_sdk::models::llm_model::LLMModel;
use std::fmt::Debug;

/// Extension trait for LLMModel to provide additional functionality
pub trait LLMModelExt {
    /// Get the model's original LLMModel reference
    fn as_model(&self) -> &'static LLMModel;
    
    /// Check if this model supports streaming
    fn supports_streaming(&self) -> bool;
    
    /// Check if this model supports thinking/reasoning
    fn supports_thinking(&self) -> bool;
    
    /// Create a message optimized for this model's capabilities
    fn create_message(&self, text: String) -> LLMMessage {
        let mut message = LLMMessage::user()
            .with_text(text.clone())
            .to_owned();
            
        // If model supports thinking/reasoning, add a reasoning prompt
        if self.supports_thinking() {
            // Add a specific reasoning/thinking section
            message = message.with_reasoning("".to_string()).to_owned();
        }
        
        message
    }
}

/// Implement the extension trait for LLMModel
impl LLMModelExt for &'static LLMModel {
    fn as_model(&self) -> &'static LLMModel {
        self
    }
    
    fn supports_streaming(&self) -> bool {
        // Default to true for all models
        // In a real implementation, you would use model.capabilities().supports_streaming()
        true
    }
    
    fn supports_thinking(&self) -> bool {
        // For now, enable thinking just for Anthropic models
        // In a real implementation, you would use model.capabilities().supports_thinking()
        match self.provider().name() {
            "Anthropic" => true,
            _ => false
        }
    }
}