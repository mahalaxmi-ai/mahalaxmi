//! Domain loader — reads a domain directory and produces a [`LoadedDomain`].

use std::path::Path;

use anyhow::{Context, Result};

use super::config::{
    ConsensusAlgorithm, DecompositionStrategy, DomainBehavior, DomainConfig, InputFormat,
    OutputFormat, PromptSource,
};

/// A domain configuration loaded from disk.
///
/// All prompt files are read at construction time and stored in memory.
/// At query time, `${worker_count}` placeholders are substituted and the
/// appropriate worker-count variant (uncapped vs capped) is selected.
pub struct LoadedDomain {
    id: String,
    manager_uncapped: String,
    manager_capped: String,
    worker_system_role: String,
    /// `(name, uncapped_content, capped_content)`.
    /// `capped_content` is `None` for sections with no worker-count variant.
    constraint_sections: Vec<(String, String, Option<String>)>,
    decomposition_strategy: DecompositionStrategy,
    consensus_algorithm: ConsensusAlgorithm,
    output_format: OutputFormat,
    input_format: InputFormat,
}

impl LoadedDomain {
    /// Load a domain from `domain_dir`.
    ///
    /// Expects `domain_dir/config.yaml` and any `file:` references relative
    /// to `domain_dir`.
    pub fn load(domain_dir: &Path) -> Result<Self> {
        let config_path = domain_dir.join("config.yaml");
        let config_str = std::fs::read_to_string(&config_path)
            .with_context(|| format!("reading domain config at {}", config_path.display()))?;
        // Use singleton_map_recursive so that enum variants in config.yaml can
        // be written as `{variant_name: {fields}}` maps (the human-friendly YAML
        // style) rather than YAML native tags (`!variant_name`).
        let de = serde_yaml::Deserializer::from_str(&config_str);
        let behavior: DomainBehavior =
            serde_yaml::with::singleton_map_recursive::deserialize(de)
                .with_context(|| {
                    format!("parsing domain config at {}", config_path.display())
                })?;

        let resolve = |source: &PromptSource| -> Result<String> {
            match source {
                PromptSource::Inline(s) => Ok(s.clone()),
                PromptSource::File { file } => {
                    let path = domain_dir.join(file);
                    let raw = std::fs::read_to_string(&path)
                        .with_context(|| format!("reading prompt file {}", path.display()))?;
                    // Strip trailing newline: Unix text files conventionally end
                    // with \n but the hardcoded Rust strings do not.
                    Ok(raw.trim_end().to_owned())
                }
            }
        };

        let manager_uncapped = resolve(&behavior.manager_uncapped)?;
        let manager_capped = resolve(&behavior.manager_capped)?;
        let worker_system_role = resolve(&behavior.worker_system_role)?;

        let mut constraint_sections = Vec::new();
        for section in &behavior.constraint_sections {
            let uncapped = resolve(&section.source)?;
            let capped = section.capped.as_ref().map(&resolve).transpose()?;
            constraint_sections.push((section.name.clone(), uncapped, capped));
        }

        let decomposition_strategy = behavior.decomposition_strategy.unwrap_or_default();
        let consensus_algorithm = behavior.consensus_algorithm.unwrap_or_default();
        let output_format = behavior.output_format.unwrap_or_default();
        let input_format = behavior.input_format.unwrap_or_default();

        Ok(Self {
            id: behavior.id,
            manager_uncapped,
            manager_capped,
            worker_system_role,
            constraint_sections,
            decomposition_strategy,
            consensus_algorithm,
            output_format,
            input_format,
        })
    }
}

/// Substitute `${worker_count}` placeholder in `template`.
fn substitute(template: &str, worker_count: u32) -> String {
    template.replace("${worker_count}", &worker_count.to_string())
}

impl DomainConfig for LoadedDomain {
    fn id(&self) -> &str {
        &self.id
    }

    fn manager_system_role(&self, worker_count: u32) -> String {
        if worker_count == 0 {
            self.manager_uncapped.clone()
        } else {
            substitute(&self.manager_capped, worker_count)
        }
    }

    fn worker_system_role(&self) -> String {
        self.worker_system_role.clone()
    }

    fn constraint_sections(&self, worker_count: u32) -> Vec<(String, String)> {
        self.constraint_sections
            .iter()
            .map(|(name, uncapped, capped)| {
                let content = if worker_count == 0 {
                    uncapped.clone()
                } else {
                    match capped {
                        Some(c) => substitute(c, worker_count),
                        None => uncapped.clone(),
                    }
                };
                (name.clone(), content)
            })
            .collect()
    }

    fn decomposition_strategy(&self) -> DecompositionStrategy {
        self.decomposition_strategy.clone()
    }

    fn consensus_algorithm(&self) -> ConsensusAlgorithm {
        self.consensus_algorithm.clone()
    }

    fn output_format(&self) -> OutputFormat {
        self.output_format.clone()
    }

    fn input_format(&self) -> InputFormat {
        self.input_format.clone()
    }
}
