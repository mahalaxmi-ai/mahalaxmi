//! AI provider abstraction layer for Mahalaxmi.
//!
//! Defines the `AiProvider` trait and implementations for multiple AI CLI tools
//! including Claude Code, OpenAI Foundry, AWS Bedrock, and generic CLI providers.
//! Users bring their own subscriptions and API keys.

pub mod aider;
pub mod bedrock;
pub mod chatgpt;
pub mod claude;
pub mod copilot;
pub mod cost;
pub mod credential_store;
pub mod credentials;
pub mod custom_cli;
pub mod gemini;
pub mod generic;
pub mod grok;
pub mod handoff;
pub mod markers;
pub mod metadata;
pub mod metrics;
pub mod mock;
pub mod ollama;
pub mod registry;
pub mod resilience;
pub mod router;
pub mod security;
pub mod tier1;
pub mod traits;
pub mod types;

pub use aider::AiderProvider;
pub use bedrock::BedrockProvider;
pub use chatgpt::ChatGptProvider;
pub use claude::ClaudeCodeProvider;
pub use copilot::CopilotProvider;
pub use cost::{built_in_pricing, ProviderPricing, TokenUsage};
pub use credential_store::{
    credential_key, probe_keyring, resolve_provider_credentials, ChainedCredentialStore,
    CredentialStore, EncryptedFileCredentialStore, EnvCredentialStore, KeyringCredentialStore,
    MemoryCredentialStore,
};
pub use credentials::{AuthMethod, AuthMode, CredentialSpec, ProviderStatus};
pub use custom_cli::CustomCliProvider;
pub use gemini::GeminiProvider;
pub use generic::GenericCliProvider;
pub use grok::GrokProvider;
pub use handoff::{HandoffContext, HandoffResult, HandoffTracker};
pub use markers::OutputMarkers;
pub use metadata::{
    find_binary, find_binary_async, parse_version_from_output, version_meets_minimum,
    DeploymentConstraint, PlatformInstallCommands, ProviderMetadata,
};
pub use metrics::{MetricEvent, PerformanceComparison, PerformanceTracker, ProviderMetrics};
pub use mock::MockProvider;
pub use ollama::OllamaProvider;
pub use registry::{pricing_for, ProviderRegistry};
pub use resilience::{CircuitBreaker, CircuitBreakerConfig, CircuitState, ProviderHealthTracker};
pub use router::{
    strategy_score, RoutingConstraints, RoutingDecision, RoutingStrategy, TaskRouter,
};
pub use security::{
    apply_security_routing, classify_task, is_local_by_capabilities, is_local_provider,
    SecurityClassification, SecurityRoutingMode, SecurityRoutingResult,
};
pub use traits::AiProvider;
pub use types::{CostTier, Proficiency, ProviderCapabilities, TaskType};

pub use mahalaxmi_core::config::MahalaxmiConfig;
pub use mahalaxmi_core::error::MahalaxmiError;
pub use mahalaxmi_core::i18n::locale::SupportedLocale;
pub use mahalaxmi_core::i18n::I18nService;
pub use mahalaxmi_core::types::{ProcessCommand, ProviderId};
pub use mahalaxmi_core::MahalaxmiResult;
