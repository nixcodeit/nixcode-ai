pub struct ModelCapabilitiesBuilder {
    supports_streaming: bool,
    supports_cache: bool,
    supports_thinking: bool,
}

impl ModelCapabilitiesBuilder {
    pub fn new() -> Self {
        Self {
            supports_streaming: false,
            supports_cache: false,
            supports_thinking: false,
        }
    }

    pub fn with_streaming(mut self) -> Self {
        self.supports_streaming = true;
        self
    }

    pub fn with_cache(mut self) -> Self {
        self.supports_cache = true;
        self
    }

    pub fn with_thinking(mut self) -> Self {
        self.supports_thinking = true;
        self
    }

    pub fn build(self) -> ModelCapabilities {
        ModelCapabilities {
            supports_streaming: self.supports_streaming,
            supports_cache: self.supports_cache,
            supports_thinking: self.supports_thinking,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct ModelCapabilities {
    pub supports_streaming: bool,
    pub supports_cache: bool,
    pub supports_thinking: bool,
}

impl ModelCapabilities {
    pub fn supports_streaming(&self) -> bool {
        self.supports_streaming
    }

    pub fn supports_cache(&self) -> bool {
        self.supports_cache
    }

    pub fn supports_thinking(&self) -> bool {
        self.supports_thinking
    }
}
