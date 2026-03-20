//! Provider pricing model and token tracking types.
//!
//! Provides per-model cost data and utilities for estimating API call costs
//! based on token usage.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Pricing information for an AI provider model.
///
/// Costs are denominated in USD per one million tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPricing {
    /// Cost in USD per one million input (prompt) tokens.
    pub input_cost_per_million: f64,
    /// Cost in USD per one million output (completion) tokens.
    pub output_cost_per_million: f64,
}

impl ProviderPricing {
    /// Estimate the cost in USD for a given number of input and output tokens.
    pub fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.input_cost_per_million
            + (output_tokens as f64 / 1_000_000.0) * self.output_cost_per_million
    }
}

/// Returns the built-in pricing table keyed by lowercase model ID.
///
/// Contains pricing for all supported Claude, OpenAI, DeepSeek, and Qwen Coder models.
pub fn built_in_pricing() -> HashMap<String, ProviderPricing> {
    let mut map = HashMap::new();

    // Claude models
    map.insert(
        "claude-opus-4-6".into(),
        ProviderPricing {
            input_cost_per_million: 15.00,
            output_cost_per_million: 75.00,
        },
    );
    map.insert(
        "claude-sonnet-4-6".into(),
        ProviderPricing {
            input_cost_per_million: 3.00,
            output_cost_per_million: 15.00,
        },
    );
    map.insert(
        "claude-haiku-4-5-20251001".into(),
        ProviderPricing {
            input_cost_per_million: 0.80,
            output_cost_per_million: 4.00,
        },
    );

    // OpenAI models
    map.insert(
        "gpt-4o".into(),
        ProviderPricing {
            input_cost_per_million: 2.50,
            output_cost_per_million: 10.00,
        },
    );
    map.insert(
        "gpt-4o-mini".into(),
        ProviderPricing {
            input_cost_per_million: 0.15,
            output_cost_per_million: 0.60,
        },
    );

    // Tier 1 community models
    map.insert(
        "deepseek-chat".into(),
        ProviderPricing {
            input_cost_per_million: 0.27,
            output_cost_per_million: 1.10,
        },
    );
    map.insert(
        "qwen-coder-plus".into(),
        ProviderPricing {
            input_cost_per_million: 0.35,
            output_cost_per_million: 1.40,
        },
    );

    map
}

/// Token usage record for a single AI provider interaction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input (prompt) tokens consumed.
    #[serde(default)]
    pub input_tokens: u64,
    /// Number of output (completion) tokens generated.
    #[serde(default)]
    pub output_tokens: u64,
    /// Whether the token counts are exact (from provider response) or estimated.
    #[serde(default)]
    pub is_exact: bool,
    /// The model ID used for this interaction (lowercase).
    #[serde(default)]
    pub model_id: String,
}

impl TokenUsage {
    /// Estimate token counts from raw byte lengths.
    ///
    /// Uses rough heuristics: ~3 bytes per input token, ~4 bytes per output token.
    /// Always returns at least 1 token for non-zero byte counts.
    pub fn estimate_from_bytes(input_bytes: usize, output_bytes: usize, model_id: &str) -> Self {
        Self {
            input_tokens: (input_bytes / 3).max(1) as u64,
            output_tokens: (output_bytes / 4).max(1) as u64,
            is_exact: false,
            model_id: model_id.to_owned(),
        }
    }

    /// Calculate the estimated cost in USD for this token usage.
    ///
    /// Returns `0.0` if the model ID is not found in the built-in pricing table.
    pub fn estimated_cost_usd(&self) -> f64 {
        match built_in_pricing().get(&self.model_id) {
            Some(pricing) => pricing.estimate_cost(self.input_tokens, self.output_tokens),
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_input_cost() {
        let pricing = ProviderPricing {
            input_cost_per_million: 3.0,
            output_cost_per_million: 15.0,
        };
        assert_eq!(pricing.estimate_cost(1_000_000, 0), 3.0);
    }

    #[test]
    fn pricing_output_cost() {
        let pricing = ProviderPricing {
            input_cost_per_million: 3.0,
            output_cost_per_million: 15.0,
        };
        assert_eq!(pricing.estimate_cost(0, 1_000_000), 15.0);
    }

    #[test]
    fn estimate_from_bytes_token_counts() {
        let usage = TokenUsage::estimate_from_bytes(3000, 4000, "claude-sonnet-4-6");
        assert_eq!(usage.input_tokens, 1000);
        assert_eq!(usage.output_tokens, 1000);
    }

    #[test]
    fn unknown_model_cost_is_zero() {
        let usage = TokenUsage {
            model_id: "unknown-model-xyz".into(),
            ..Default::default()
        };
        assert_eq!(usage.estimated_cost_usd(), 0.0);
    }

    #[test]
    fn built_in_pricing_has_at_least_seven_entries() {
        assert!(built_in_pricing().len() >= 7);
    }

    #[test]
    fn default_token_usage_is_zero() {
        let usage = TokenUsage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert!(!usage.is_exact);
    }

    #[test]
    fn provider_pricing_round_trips_json() {
        let pricing = ProviderPricing {
            input_cost_per_million: 3.0,
            output_cost_per_million: 15.0,
        };
        let json = serde_json::to_string(&pricing).expect("serialize");
        let restored: ProviderPricing = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            restored.input_cost_per_million,
            pricing.input_cost_per_million
        );
        assert_eq!(
            restored.output_cost_per_million,
            pricing.output_cost_per_million
        );
    }

    #[test]
    fn token_usage_round_trips_json() {
        let usage = TokenUsage {
            input_tokens: 500,
            output_tokens: 250,
            is_exact: true,
            model_id: "claude-sonnet-4-6".into(),
        };
        let json = serde_json::to_string(&usage).expect("serialize");
        let restored: TokenUsage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.input_tokens, usage.input_tokens);
        assert_eq!(restored.output_tokens, usage.output_tokens);
        assert_eq!(restored.is_exact, usage.is_exact);
        assert_eq!(restored.model_id, usage.model_id);
    }
}
