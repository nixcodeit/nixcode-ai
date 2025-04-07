use crate::message::common::llm_message::Usage;

pub fn sonnet37_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 3.75;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 0.30;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 3.0;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 15.0;
    create_input_cache_cost + read_input_cache_cost + input_cost + output_cost
}
