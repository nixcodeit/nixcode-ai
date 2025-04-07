    pub static ref DeepSeekR1: LLMModel = LLMModelBuilder::new()
        .model_name("deepseek-r1-distill-llama-70b")
        .display_name("DeepSeek R1")
        .provider(LLMProvider::Groq)
        .cost_calculation(Arc::new(deepseek_r1_cost_calculation))
        .capabilities(
            ModelCapabilitiesBuilder::new()
                .with_streaming()
                .build()
        )
        .build();