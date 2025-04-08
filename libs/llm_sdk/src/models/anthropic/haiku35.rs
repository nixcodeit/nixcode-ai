use crate::message::common::llm_message::Usage;

pub fn haiku35_cost_calculation(usage: Usage) -> f64 {
    let create_input_cache_cost = usage.cache_writes.unwrap_or(0) as f64 / 1_000_000.0 * 1.0;
    let read_input_cache_cost = usage.cache_reads.unwrap_or(0) as f64 / 1_000_000.0 * 0.08;
    let input_cost = usage.input_tokens as f64 / 1_000_000.0 * 0.80;
    let output_cost = usage.output_tokens as f64 / 1_000_000.0 * 4.0;
    (create_input_cache_cost + read_input_cache_cost + input_cost + output_cost).max(0.0)
}
