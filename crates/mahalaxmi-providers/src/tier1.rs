//! Tier 1 provider definitions — pre-configured GenericCliProvider instances.
//!
//! Each function returns a `GenericCliProvider` configured with the correct
//! binary, arguments, credential requirements, capabilities, and metadata.

use crate::credentials::{AuthMethod, AuthMode, CredentialSpec};
use crate::generic::GenericCliProvider;
use crate::markers::OutputMarkers;
use crate::metadata::ProviderMetadata;
use crate::types::{CostTier, Proficiency, ProviderCapabilities, TaskType};

/// Default output markers for most CLI AI tools.
fn default_markers() -> OutputMarkers {
    OutputMarkers::new(r"\$\s*$", r"(?i)(error|fatal|failed)", r"(>\s*$|waiting)")
        .expect("default markers are valid regex")
}

/// Create a Kiro provider (Amazon's AI coding assistant).
pub fn kiro_provider() -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: 128_000,
        cost_tier: CostTier::Medium,
        avg_latency_ms: 4000,
        supports_concurrent_sessions: true,
        ..Default::default()
    };
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Excellent);
    caps.task_proficiency
        .insert(TaskType::Planning, Proficiency::Excellent);
    caps.task_proficiency
        .insert(TaskType::Debugging, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::Testing, Proficiency::Good);

    GenericCliProvider::new("kiro", "Kiro", "kiro", default_markers())
        .with_capabilities(caps)
        .with_credential(CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("AWS_ACCESS_KEY_ID".to_owned()),
            description_key: "credential-aws-access-key".to_owned(),
            required: false,
        })
        .with_metadata(
            ProviderMetadata::new("Install Kiro from https://kiro.dev")
                .with_install_url("https://kiro.dev")
                .with_test_args(vec!["--version".to_string()])
                // Kiro supports its own login flow and AWS credentials
                .with_auth_mode(AuthMode::CliLogin {
                    login_command: "kiro auth login".to_string(),
                    check_command: "kiro auth status".to_string(),
                })
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "AWS_ACCESS_KEY_ID".to_string(),
                })
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "AWS_SECRET_ACCESS_KEY".to_string(),
                }),
        )
}

/// Create a Goose provider (Block's open-source AI agent).
pub fn goose_provider() -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: 128_000,
        cost_tier: CostTier::Free,
        avg_latency_ms: 3000,
        supports_concurrent_sessions: true,
        ..Default::default()
    };
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::Debugging, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::Refactoring, Proficiency::Good);

    GenericCliProvider::new("goose", "Goose", "goose", default_markers())
        .with_capabilities(caps)
        .with_metadata(
            ProviderMetadata::new("pip install goose-ai")
                .with_install_url("https://github.com/block/goose")
                .with_test_args(vec!["--version".to_string()])
                // Goose connects to LLM backends — user configures their preferred provider
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "OPENAI_API_KEY".to_string(),
                })
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "ANTHROPIC_API_KEY".to_string(),
                })
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "GOOGLE_API_KEY".to_string(),
                }),
        )
}

/// Create a DeepSeek provider.
pub fn deepseek_provider() -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: 64_000,
        cost_tier: CostTier::Low,
        avg_latency_ms: 2000,
        supports_concurrent_sessions: true,
        ..Default::default()
    };
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Excellent);
    caps.task_proficiency
        .insert(TaskType::Debugging, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::Testing, Proficiency::Good);

    GenericCliProvider::new("deepseek", "DeepSeek", "deepseek", default_markers())
        .with_capabilities(caps)
        .with_credential(CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("DEEPSEEK_API_KEY".to_owned()),
            description_key: "credential-deepseek-api-key".to_owned(),
            required: true,
        })
        .with_metadata(
            ProviderMetadata::new("pip install deepseek-cli")
                .with_install_url("https://www.deepseek.com/")
                .with_test_args(vec!["--version".to_string()])
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "DEEPSEEK_API_KEY".to_string(),
                }),
        )
}

/// Create a Qwen Coder provider (Alibaba's AI coding model).
pub fn qwen_provider() -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: 32_000,
        cost_tier: CostTier::Low,
        avg_latency_ms: 2500,
        supports_concurrent_sessions: true,
        ..Default::default()
    };
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::Refactoring, Proficiency::Good);

    GenericCliProvider::new("qwen", "Qwen Coder", "qwen", default_markers())
        .with_capabilities(caps)
        .with_credential(CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("DASHSCOPE_API_KEY".to_owned()),
            description_key: "credential-dashscope-api-key".to_owned(),
            required: true,
        })
        .with_metadata(
            ProviderMetadata::new("pip install dashscope")
                .with_install_url("https://www.alibabacloud.com/product/modelstudio")
                .with_test_args(vec!["--version".to_string()])
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "DASHSCOPE_API_KEY".to_string(),
                }),
        )
}

/// Create an OpenCode provider (open-source terminal AI tool).
pub fn opencode_provider() -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: 128_000,
        cost_tier: CostTier::Free,
        avg_latency_ms: 3000,
        supports_concurrent_sessions: true,
        ..Default::default()
    };
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::Debugging, Proficiency::Good);

    GenericCliProvider::new("opencode", "OpenCode", "opencode", default_markers())
        .with_capabilities(caps)
        .with_metadata(
            ProviderMetadata::new("go install github.com/opencode-ai/opencode@latest")
                .with_install_url("https://github.com/opencode-ai/opencode")
                .with_test_args(vec!["--version".to_string()])
                // OpenCode supports multiple LLM backends via configuration
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "OPENAI_API_KEY".to_string(),
                })
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "ANTHROPIC_API_KEY".to_string(),
                }),
        )
}

/// Create a Cody provider (Sourcegraph's AI coding assistant).
pub fn cody_provider() -> GenericCliProvider {
    let mut caps = ProviderCapabilities {
        supports_streaming: true,
        supports_tool_use: true,
        max_context_tokens: 100_000,
        cost_tier: CostTier::Medium,
        avg_latency_ms: 3500,
        supports_concurrent_sessions: true,
        ..Default::default()
    };
    caps.task_proficiency
        .insert(TaskType::CodeGeneration, Proficiency::Good);
    caps.task_proficiency
        .insert(TaskType::CodeReview, Proficiency::Excellent);
    caps.task_proficiency
        .insert(TaskType::Documentation, Proficiency::Good);

    GenericCliProvider::new("cody", "Cody", "cody", default_markers())
        .with_capabilities(caps)
        .with_credential(CredentialSpec {
            method: AuthMethod::ApiKey,
            env_var_name: Some("SRC_ACCESS_TOKEN".to_owned()),
            description_key: "credential-sourcegraph-token".to_owned(),
            required: true,
        })
        .with_metadata(
            ProviderMetadata::new("npm install -g @sourcegraph/cody")
                .with_install_url("https://sourcegraph.com/cody")
                .with_test_args(vec!["--version".to_string()])
                .with_auth_mode(AuthMode::ApiKey {
                    env_var: "SRC_ACCESS_TOKEN".to_string(),
                }),
        )
}

/// Register all Tier 1 providers into a registry.
pub fn register_tier1_providers(registry: &mut crate::ProviderRegistry) {
    registry.register(Box::new(kiro_provider()));
    registry.register(Box::new(goose_provider()));
    registry.register(Box::new(deepseek_provider()));
    registry.register(Box::new(qwen_provider()));
    registry.register(Box::new(opencode_provider()));
    registry.register(Box::new(cody_provider()));
}
