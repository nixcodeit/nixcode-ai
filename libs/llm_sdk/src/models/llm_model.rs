use crate::message::common::llm_message::Usage;
use crate::models::anthropic::haiku35::haiku35_cost_calculation;
use crate::models::anthropic::sonnet37::sonnet37_cost_calculation;
use crate::models::capabilities::{ModelCapabilities, ModelCapabilitiesBuilder};
use crate::providers::LLMProvider;
use lazy_static::lazy_static;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

lazy_static! {
    pub static ref Sonnet37: LLMModel = LLMModelBuilder::new()
        .model_name("claude-3-7-sonnet-latest")
        .display_name("Claude 3.7 Sonnet")
        .provider(LLMProvider::Anthropic)
        .capabilities(
            ModelCapabilitiesBuilder::new()
                .with_cache()
                .with_streaming()
                .with_streaming()
                .build()
        )
        .cost_calculation(Arc::new(sonnet37_cost_calculation))
        .build();
    pub static ref Haiku35: LLMModel = LLMModelBuilder::new()
        .model_name("claude-3-5-haiku-latest")
        .display_name("Claude 3.5 Haiku")
        .provider(LLMProvider::Anthropic)
        .capabilities(
            ModelCapabilitiesBuilder::new()
                .with_cache()
                .with_streaming()
                .with_thinking()
                .build()
        )
        .cost_calculation(Arc::new(haiku35_cost_calculation))
        .build();
    pub static ref Gpt4o: LLMModel = LLMModelBuilder::new()
        .model_name("gpt-4o")
        .display_name("4o")
        .provider(LLMProvider::OpenAI)
        .build();
    pub static ref Gpt3oMini: LLMModel = LLMModelBuilder::new()
        .model_name("3o-mini")
        .display_name("3o Mini")
        .provider(LLMProvider::OpenAI)
        .build();
    pub static ref Llama4: LLMModel = LLMModelBuilder::new()
        .model_name("meta-llama/llama-4-scout-17b-16e-instruct")
        .display_name("Llama 4 Scout")
        .provider(LLMProvider::Groq)
        .build();
    pub static ref QwenQwq32b: LLMModel = LLMModelBuilder::new()
        .model_name("qwen-qwq-32b")
        .display_name("Qwen Qwq 32b")
        .provider(LLMProvider::Groq)
        .build();
    pub static ref QuasarAlpha: LLMModel = LLMModelBuilder::new()
        .model_name("openrouter/quasar-alpha")
        .display_name("Quasar Alpha")
        .provider(LLMProvider::OpenRouter)
        .build();
    pub static ref Llama4OpenRouter: LLMModel = LLMModelBuilder::new()
        .model_name("meta-llama/llama-4-scout")
        .display_name("Llama 4 Scout")
        .provider(LLMProvider::OpenRouter)
        .build();
    pub static ref Gemini25Pro: LLMModel = LLMModelBuilder::new()
        .model_name("google/gemini-2.5-pro-preview-03-25")
        .display_name("Google: Gemini 2.5 Pro Preview")
        .provider(LLMProvider::OpenRouter)
        .build();
    pub static ref DeepSeekV3: LLMModel = LLMModelBuilder::new()
        .model_name("deepseek/deepseek-chat")
        .display_name("DeepSeek V3")
        .cost_calculation(Arc::new(deepseek_v3_cost_calulcation))
        .provider(LLMProvider::OpenRouter)
        .build();
    pub static ref DeepSeekR1: LLMModel = LLMModelBuilder::new()
        .model_name("deepseek-r1-distill-llama-70b")
        .display_name("DeepSeek R1")
        .provider(LLMProvider::Groq)
        .cost_calculation(Arc::new(deepseek_r1_cost_calculation))
        .build();
    pub static ref AllModels: Vec<&'static LLMModel> = vec![
        &Sonnet37,
        &Haiku35,
        &Gpt4o,
        &Gpt3oMini,
        &Llama4,
        &QwenQwq32b,
        &QuasarAlpha,
        &Llama4OpenRouter
    ];
}

fn deepseek_r1_cost_calculation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.75;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 0.99;
    input_cost + output_cost
}

fn deepseek_v3_cost_calulcation(usage: Usage) -> f64 {
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.4;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 0.89;
    input_cost + output_cost
}

pub type CostCalculation = Arc<dyn Fn(Usage) -> f64 + Send + Sync>;

#[derive(Clone)]
pub struct LLMModel {
    display_name: String,
    model_name: String,
    provider: LLMProvider,
    cost_calculation: CostCalculation,
    capabilities: ModelCapabilities,
}

#[derive(Default)]
pub struct LLMModelBuilder {
    display_name: Option<String>,
    model_name: Option<String>,
    provider: LLMProvider,
    cost_calculation: Option<CostCalculation>,
    capabilities: Option<ModelCapabilities>,
}

impl LLMModelBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn model_name(mut self, model_name: impl Into<String>) -> Self {
        self.model_name = Some(model_name.into());
        self
    }

    pub fn provider(mut self, provider: LLMProvider) -> Self {
        self.provider = provider;
        self
    }

    pub fn cost_calculation(mut self, cost_calculation: CostCalculation) -> Self {
        self.cost_calculation = Some(cost_calculation);
        self
    }

    pub fn capabilities(mut self, capabilities: ModelCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    pub fn build(self) -> LLMModel {
        LLMModel {
            display_name: self.display_name.unwrap_or_default(),
            model_name: self.model_name.unwrap_or_default(),
            provider: self.provider,
            cost_calculation: self.cost_calculation.unwrap_or_else(|| Arc::new(|_| 0.0)),
            capabilities: self.capabilities.unwrap_or_else(|| Default::default()),
        }
    }
}

impl Display for LLMModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name.as_ref())
    }
}

impl Into<String> for LLMModel {
    fn into(self) -> String {
        self.model_name.clone()
    }
}

impl LLMModel {
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn provider(&self) -> &LLMProvider {
        &self.provider
    }

    pub fn calculate_cost(&self, usage: Usage) -> f64 {
        (self.cost_calculation)(usage)
    }
    
    pub fn capabilities(&self) -> &ModelCapabilities {
        &self.capabilities
    }
}