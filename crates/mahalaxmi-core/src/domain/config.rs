//! Domain configuration types and the `DomainConfig` trait.
//!
//! A *domain* (e.g. "coding") bundles the prompt text used by manager and worker
//! agents. Prompts can be inlined directly in `config.yaml` or loaded from
//! separate `.prompt` files that live alongside the config.

use serde::{Deserialize, Serialize};

/// Where a prompt section's text comes from.
///
/// In `config.yaml` use either a bare string (`inline`) or a map with a
/// `file` key pointing to a path relative to the domain directory.
///
/// ```yaml
/// # Inline
/// worker_system_role: "You are an AI coding agent…"
///
/// # File reference
/// worker_system_role:
///   file: worker.prompt
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptSource {
    /// Literal inline text.
    Inline(String),
    /// Load from a file path relative to the domain directory.
    File { file: String },
}

/// A named constraint section, optionally with a worker-count-dependent variant.
///
/// When `capped` is `Some`, it is used for `worker_count > 0`; `source` is used
/// for `worker_count == 0` (uncapped / auto-scale mode) or when `capped` is `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintSection {
    /// Human-readable section name (used as the section header in the prompt).
    pub name: String,
    /// Content used when `worker_count == 0` (auto-scale) or as the sole source.
    pub source: PromptSource,
    /// Optional alternative used when `worker_count > 0`.
    /// Supports `${worker_count}` placeholder substitution.
    #[serde(default)]
    pub capped: Option<PromptSource>,
}

/// Strategy the manager uses to decompose a goal into worker tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecompositionStrategy {
    /// Default: manager decomposes as a software engineering task.
    #[default]
    SoftwareDevelopment,
    /// Split the input into overlapping text sections and assign each to a worker.
    SectionBased {
        /// Maximum tokens per section.
        max_section_tokens: usize,
        /// Number of tokens to overlap between adjacent sections.
        overlap_tokens: usize,
    },
    /// Assign one worker per named role.
    RoleBased {
        /// Ordered list of role names; one worker is spawned per role.
        roles: Vec<String>,
    },
    /// Manager uses its own judgment; no decomposition hint is injected.
    ManagerDefined,
}

impl DecompositionStrategy {
    /// Returns a human-readable hint to inject into the manager prompt.
    ///
    /// `SoftwareDevelopment` and `ManagerDefined` return an empty string so
    /// they produce no additional text in the prompt. The other variants
    /// return descriptive guidance.
    pub fn hint(&self) -> String {
        match self {
            Self::SoftwareDevelopment | Self::ManagerDefined => String::new(),
            Self::SectionBased {
                max_section_tokens,
                overlap_tokens,
            } => format!(
                "Split the input into sections of at most {max_section_tokens} tokens with {overlap_tokens} tokens of overlap between adjacent sections."
            ),
            Self::RoleBased { roles } => {
                let role_list = roles
                    .iter()
                    .enumerate()
                    .map(|(i, r)| format!("  {}. {}", i + 1, r))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "RoleBased decomposition — HARD CONSTRAINT: produce exactly {count} tasks, one per specialist role below. \
                     Each task's \"title\" field MUST equal its role identifier verbatim. \
                     Each task MUST be scoped exclusively to its role's domain — do not blend responsibilities across tasks.\n\n\
                     Roles (ordered):\n{roles}",
                    count = roles.len(),
                    roles = role_list,
                )
            }
        }
    }
}

/// Default threshold for `ConsensusAlgorithm::Majority`.
fn default_majority_threshold() -> f32 {
    0.67
}

/// Algorithm used to reconcile multiple worker outputs into a final result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusAlgorithm {
    /// Default: all workers must agree.
    #[default]
    Unanimous,
    /// A configurable fraction of workers must agree.
    Majority {
        /// Fraction of workers that must agree (default 0.67 — supermajority).
        #[serde(default = "default_majority_threshold")]
        threshold: f32,
    },
    /// Select the best output according to the given criteria.
    BestOfN {
        /// Criteria injected into the manager reconciliation prompt.
        selection_criteria: String,
    },
    /// A dedicated manager session synthesises all worker outputs into one result.
    Synthesis {
        /// Additional instruction prepended to the synthesis manager prompt.
        synthesis_prompt: String,
    },
    /// A manager session adjudicates conflicts using a custom prompt.
    ManagerAdjudicated {
        /// Replaces the hardcoded arbitration prompt when conflicts are detected.
        conflict_resolution_prompt: String,
    },
}

/// Format in which the cycle result is emitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Default: create a git branch, commit changes, and open a pull request.
    #[default]
    PullRequest,
    /// Organise worker outputs into a structured markdown report.
    StructuredReport {
        /// Ordered section headings in the report.
        sections: Vec<String>,
        /// Output file path relative to the project directory.
        output_file: String,
    },
    /// Serialise cycle output to JSON.
    JsonExport {
        /// Output file path relative to the project directory.
        output_file: String,
        /// Optional JSON Schema for validation before writing.
        schema: Option<String>,
    },
    /// Write cycle output as a markdown file.
    MarkdownFile {
        /// Output file path relative to the project directory.
        output_file: String,
        /// Optional template file path.
        template: Option<String>,
    },
}

/// Format of the input consumed by the cycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InputFormat {
    /// Default: scan the git repository file tree.
    #[default]
    Codebase,
    /// Accept a document file (PDF, DOCX, TXT, MD) as input.
    DocumentFile {
        /// MIME type extensions accepted (e.g. `["pdf", "docx", "txt"]`).
        accepted_types: Vec<String>,
        /// Maximum file size in megabytes.
        max_file_size_mb: usize,
    },
    /// Accept raw text provided directly in the requirements field.
    TextInput,
}

/// Raw deserialized domain behavior loaded from `config.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainBehavior {
    /// Unique domain identifier (e.g. "coding").
    pub id: String,
    /// Human-readable description of the domain.
    pub description: String,
    /// Manager system role when `worker_count == 0` (uncapped / auto-scale).
    pub manager_uncapped: PromptSource,
    /// Manager system role when `worker_count > 0`. Supports `${worker_count}`.
    pub manager_capped: PromptSource,
    /// Worker system role (static — no parameterisation).
    pub worker_system_role: PromptSource,
    /// Ordered constraint sections for the manager prompt.
    #[serde(default)]
    pub constraint_sections: Vec<ConstraintSection>,
    /// How the manager decomposes the goal into worker tasks.
    #[serde(default)]
    pub decomposition_strategy: Option<DecompositionStrategy>,
    /// How worker outputs are reconciled into the final result.
    #[serde(default)]
    pub consensus_algorithm: Option<ConsensusAlgorithm>,
    /// Format in which the cycle result is emitted.
    #[serde(default)]
    pub output_format: Option<OutputFormat>,
    /// Format of the input consumed by the cycle.
    #[serde(default)]
    pub input_format: Option<InputFormat>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_based_hint_contains_exactly_n_tasks() {
        let strategy = DecompositionStrategy::RoleBased {
            roles: vec!["a".to_owned(), "b".to_owned()],
        };
        let hint = strategy.hint();
        assert!(
            hint.contains("exactly 2 tasks"),
            "RoleBased hint must state 'exactly 2 tasks', got: {hint}"
        );
        assert!(hint.contains("1. a"), "hint must number roles; got: {hint}");
        assert!(hint.contains("2. b"), "hint must number roles; got: {hint}");
    }

    #[test]
    fn role_based_hint_four_roles_states_exactly_four() {
        let strategy = DecompositionStrategy::RoleBased {
            roles: vec![
                "vulnerability_analyst".to_owned(),
                "dependency_auditor".to_owned(),
                "secrets_scanner".to_owned(),
                "authentication_reviewer".to_owned(),
            ],
        };
        let hint = strategy.hint();
        assert!(
            hint.contains("exactly 4 tasks"),
            "RoleBased hint for 4 roles must say 'exactly 4 tasks', got: {hint}"
        );
        assert!(
            hint.contains("vulnerability_analyst"),
            "hint must list all roles"
        );
        assert!(
            hint.contains("authentication_reviewer"),
            "hint must list all roles"
        );
    }

    #[test]
    fn software_development_and_manager_defined_hints_are_empty() {
        assert_eq!(DecompositionStrategy::SoftwareDevelopment.hint(), "");
        assert_eq!(DecompositionStrategy::ManagerDefined.hint(), "");
    }
}

/// Object-safe interface for a loaded domain configuration.
///
/// Implementors provide the resolved prompt strings given the runtime
/// `worker_count`. The strings are ready to be embedded in a prompt — no
/// further formatting is required by the caller.
pub trait DomainConfig: Send + Sync {
    /// The unique domain identifier.
    fn id(&self) -> &str;

    /// Manager system-role preamble, parameterised by worker count.
    ///
    /// `worker_count == 0` selects the uncapped (auto-scale) variant.
    fn manager_system_role(&self, worker_count: u32) -> String;

    /// Worker system-role preamble (static).
    fn worker_system_role(&self) -> String;

    /// Ordered list of `(section_name, resolved_content)` pairs for all
    /// constraint sections, with `${worker_count}` already substituted.
    fn constraint_sections(&self, worker_count: u32) -> Vec<(String, String)>;

    /// Strategy used to decompose the goal into worker tasks.
    fn decomposition_strategy(&self) -> DecompositionStrategy {
        DecompositionStrategy::default()
    }

    /// Algorithm used to reconcile worker outputs.
    fn consensus_algorithm(&self) -> ConsensusAlgorithm {
        ConsensusAlgorithm::default()
    }

    /// Format in which the cycle result is emitted.
    fn output_format(&self) -> OutputFormat {
        OutputFormat::default()
    }

    /// Format of the input consumed by the cycle.
    fn input_format(&self) -> InputFormat {
        InputFormat::default()
    }

    /// Human-readable decomposition hint to inject into the manager prompt.
    ///
    /// Returns an empty string for `SoftwareDevelopment` and `ManagerDefined`
    /// so existing prompts are unchanged.
    fn decomposition_hint(&self) -> String {
        self.decomposition_strategy().hint()
    }
}
