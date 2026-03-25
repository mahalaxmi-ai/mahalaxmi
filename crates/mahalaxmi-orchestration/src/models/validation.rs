//! Post-cycle requirement validation — types for holistic assessment of cycle output.
//!
//! After all workers complete, the validation agent assesses whether the
//! combination of completed tasks actually fulfills the user's original
//! requirements. This catches under-decomposition, scope drift, and
//! integration gaps that per-task verification misses.

use mahalaxmi_core::config::AcceptanceThreshold;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

use super::proposal::ProposedTask;

/// Overall fulfillment status of the validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FulfillmentStatus {
    /// All requirements met, all acceptance criteria passed.
    Fulfilled,
    /// Some requirements met but gaps remain.
    PartiallyFulfilled,
    /// Requirements not met — cycle output does not satisfy the request.
    NotFulfilled,
}

impl fmt::Display for FulfillmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fulfilled => write!(f, "Fulfilled"),
            Self::PartiallyFulfilled => write!(f, "Partially Fulfilled"),
            Self::NotFulfilled => write!(f, "Not Fulfilled"),
        }
    }
}

/// Severity of a requirement gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapSeverity {
    /// Blocks acceptance — a core requirement is unmet.
    Critical,
    /// Significant gap but not blocking.
    Major,
    /// Minor gap or polish issue.
    Minor,
}

impl fmt::Display for GapSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::Major => write!(f, "Major"),
            Self::Minor => write!(f, "Minor"),
        }
    }
}

/// Assessment of a single requirement or requirement group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementAssessment {
    /// The requirement text (extracted from original requirements).
    pub requirement: String,
    /// Whether this specific requirement was fulfilled.
    pub status: FulfillmentStatus,
    /// Evidence supporting the assessment.
    pub evidence: String,
    /// Which tasks contributed to this requirement.
    pub contributing_tasks: Vec<String>,
}

/// A specific gap between requirements and cycle output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementGap {
    /// The requirement or sub-requirement that is unmet.
    pub requirement: String,
    /// What was expected.
    pub expected: String,
    /// What was actually produced (or "nothing" if omitted entirely).
    pub actual: String,
    /// How severe this gap is.
    pub severity: GapSeverity,
    /// Suggested action to close the gap (used for gap task generation).
    pub suggested_action: String,
    /// Files that likely need modification to close this gap.
    pub affected_files: Vec<String>,
}

/// Result of evaluating a single acceptance criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterionResult {
    /// The criterion text.
    pub criterion: String,
    /// Which task this criterion belongs to.
    pub task_title: String,
    /// Whether the criterion was met.
    pub passed: bool,
    /// Evidence or explanation.
    pub evidence: String,
}

/// Result of running an acceptance command (build, test, lint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCommandResult {
    /// Command that was executed (e.g., "cargo test").
    pub command: String,
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Stdout output (truncated to reasonable size).
    pub stdout: String,
    /// Stderr output (truncated to reasonable size).
    pub stderr: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether this command passed (exit_code == 0).
    pub passed: bool,
}

/// Aggregated security scan result attached to a validation verdict.
///
/// Defined locally to avoid coupling between `mahalaxmi-orchestration` and
/// any app-layer crate. The shape mirrors `SecurityScanDto` in `mahalaxmi-app`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanDto {
    /// Number of Critical-severity findings.
    #[serde(default)]
    pub critical_count: usize,
    /// Number of High-severity findings.
    #[serde(default)]
    pub high_count: usize,
    /// Number of Medium-severity findings.
    #[serde(default)]
    pub medium_count: usize,
    /// Number of Low-severity findings.
    #[serde(default)]
    pub low_count: usize,
    /// All security findings from all scanners.
    #[serde(default)]
    pub findings: Vec<SecurityFindingDto>,
    /// `true` when no blocking findings were detected.
    #[serde(default = "crate::models::validation::default_passed")]
    pub passed: bool,
}

impl Default for SecurityScanDto {
    fn default() -> Self {
        Self {
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            findings: Vec::new(),
            passed: true,
        }
    }
}

/// Returns `true`; used as the default for [`SecurityScanDto::passed`].
pub fn default_passed() -> bool {
    true
}

/// A single security finding within a [`SecurityScanDto`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFindingDto {
    /// Severity string (e.g., "Critical", "High", "Medium", "Low").
    pub severity: String,
    /// Finding category string (e.g., "SecretExposed", "Vulnerability").
    pub kind: String,
    /// Source file path, if known.
    pub file: Option<String>,
    /// Line number within the file, if known.
    pub line: Option<usize>,
    /// Human-readable description of the finding.
    pub description: String,
    /// Actionable remediation guidance.
    pub remediation: String,
}

/// Complete validation verdict for a cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationVerdict {
    /// Cycle identifier that was validated.
    pub cycle_id: String,
    /// Provider that performed the validation.
    pub validator_provider_id: String,
    /// Overall fulfillment status.
    pub status: FulfillmentStatus,
    /// Per-requirement assessment (when the validator breaks down by requirement).
    pub requirement_assessments: Vec<RequirementAssessment>,
    /// Identified gaps between requirements and output.
    pub gaps: Vec<RequirementGap>,
    /// Per-criterion results.
    pub criteria_results: Vec<AcceptanceCriterionResult>,
    /// Results of acceptance commands (build, test, lint).
    pub command_results: Vec<AcceptanceCommandResult>,
    /// Human-readable summary.
    pub summary: String,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// Duration of the validation in milliseconds.
    pub duration_ms: u64,
    /// Security scan result for this cycle, if a security gate ran.
    ///
    /// `None` for cycles that predate Phase 8 (backward compatible via `#[serde(default)]`).
    #[serde(default)]
    pub security_scan: Option<SecurityScanDto>,
}

impl ValidationVerdict {
    /// Count gaps by severity.
    pub fn gap_count_by_severity(&self) -> (usize, usize, usize) {
        let critical = self
            .gaps
            .iter()
            .filter(|g| g.severity == GapSeverity::Critical)
            .count();
        let major = self
            .gaps
            .iter()
            .filter(|g| g.severity == GapSeverity::Major)
            .count();
        let minor = self
            .gaps
            .iter()
            .filter(|g| g.severity == GapSeverity::Minor)
            .count();
        (critical, major, minor)
    }

    /// Number of acceptance criteria that passed.
    pub fn criteria_passed_count(&self) -> usize {
        self.criteria_results.iter().filter(|c| c.passed).count()
    }

    /// Total number of acceptance criteria evaluated.
    pub fn criteria_total(&self) -> usize {
        self.criteria_results.len()
    }

    /// Returns true if this verdict satisfies the given acceptance threshold.
    ///
    /// Confidence check: returns false when `self.confidence < threshold.min_confidence`.
    ///
    /// Status check driven by `threshold.min_status`:
    /// - `"fulfilled"` — only `FulfillmentStatus::Fulfilled` passes.
    /// - `"partially_fulfilled"` — `Fulfilled` and `PartiallyFulfilled` pass; `NotFulfilled` fails.
    /// - `"any"` — all statuses pass regardless of fulfillment.
    /// - Any other string is treated as `"partially_fulfilled"` (conservative fallback).
    pub fn meets_threshold(&self, threshold: &AcceptanceThreshold) -> bool {
        if self.confidence < threshold.min_confidence {
            return false;
        }
        match threshold.min_status.as_str() {
            "fulfilled" => self.status == FulfillmentStatus::Fulfilled,
            "any" => true,
            _ => self.status != FulfillmentStatus::NotFulfilled,
        }
    }

    /// Parse a ValidationVerdict from the validator provider's JSON output.
    ///
    /// Attempts to extract a JSON object from the output text, handling
    /// cases where the provider wraps the JSON in markdown code fences
    /// or adds explanatory text around it.
    pub fn parse_from_output(
        output: &str,
        cycle_id: &str,
        validator_provider_id: &str,
        duration_ms: u64,
    ) -> Option<Self> {
        let json_str = extract_json_block(output)?;

        let raw: RawVerdict = serde_json::from_str(json_str).ok()?;

        let status = match raw.status.to_lowercase().as_str() {
            "fulfilled" => FulfillmentStatus::Fulfilled,
            "partially_fulfilled" | "partiallyfulfilled" | "partial" => {
                FulfillmentStatus::PartiallyFulfilled
            }
            "not_fulfilled" | "notfulfilled" | "failed" => FulfillmentStatus::NotFulfilled,
            _ => return None, // status is required and must be recognized
        };

        let requirement_assessments = raw
            .requirement_assessments
            .unwrap_or_default()
            .into_iter()
            .map(|a| RequirementAssessment {
                requirement: a.requirement.unwrap_or_default(),
                status: match a.status.unwrap_or_default().to_lowercase().as_str() {
                    "fulfilled" => FulfillmentStatus::Fulfilled,
                    "not_fulfilled" | "notfulfilled" | "failed" => FulfillmentStatus::NotFulfilled,
                    _ => FulfillmentStatus::PartiallyFulfilled,
                },
                evidence: a.evidence.unwrap_or_default(),
                contributing_tasks: a.contributing_tasks.unwrap_or_default(),
            })
            .collect();

        let gaps = raw
            .gaps
            .unwrap_or_default()
            .into_iter()
            .map(|g| {
                let severity = match g.severity.unwrap_or_default().to_lowercase().as_str() {
                    "critical" => GapSeverity::Critical,
                    "major" => GapSeverity::Major,
                    _ => GapSeverity::Minor,
                };
                RequirementGap {
                    requirement: g.requirement.unwrap_or_default(),
                    expected: g.expected.unwrap_or_default(),
                    actual: g.actual.unwrap_or_default(),
                    severity,
                    suggested_action: g.suggested_action.unwrap_or_default(),
                    affected_files: g.affected_files.unwrap_or_default(),
                }
            })
            .collect();

        let criteria_results = raw
            .criteria_results
            .unwrap_or_default()
            .into_iter()
            .map(|c| AcceptanceCriterionResult {
                criterion: c.criterion.unwrap_or_default(),
                task_title: c.task_title.unwrap_or_default(),
                passed: c.passed.unwrap_or(false),
                evidence: c.evidence.unwrap_or_default(),
            })
            .collect();

        Some(Self {
            cycle_id: cycle_id.to_owned(),
            validator_provider_id: validator_provider_id.to_owned(),
            status,
            requirement_assessments,
            gaps,
            criteria_results,
            command_results: Vec::new(), // Populated separately by acceptance runner
            summary: raw.summary.unwrap_or_default(),
            confidence: raw.confidence.unwrap_or(0.5).clamp(0.0, 1.0),
            duration_ms,
            security_scan: None,
        })
    }

    /// Convert gaps into `ProposedTask` objects for a continuation cycle.
    ///
    /// Only generates tasks for Critical and Major gaps. Minor gaps are
    /// included in the prompt context but not as explicit tasks.
    pub fn gaps_as_proposed_tasks(&self) -> Vec<ProposedTask> {
        self.gaps
            .iter()
            .filter(|gap| matches!(gap.severity, GapSeverity::Critical | GapSeverity::Major))
            .map(|gap| {
                let complexity = match gap.severity {
                    GapSeverity::Critical => 8,
                    GapSeverity::Major => 5,
                    GapSeverity::Minor => 2,
                };
                let priority = match gap.severity {
                    GapSeverity::Critical => 1,
                    GapSeverity::Major => 50,
                    GapSeverity::Minor => 100,
                };
                let description = format!(
                    "Expected: {}\nActual: {}\n\nAction: {}",
                    gap.expected, gap.actual, gap.suggested_action
                );
                let mut task = ProposedTask::new(
                    format!("[Gap Fix] {}", truncate(&gap.requirement, 80)),
                    description,
                )
                .with_complexity(complexity)
                .with_priority(priority);

                if !gap.suggested_action.is_empty() {
                    task = task.with_acceptance_criterion(format!(
                        "Requirement fulfilled: {}",
                        gap.requirement
                    ));
                    task = task
                        .with_acceptance_criterion(format!("Expected behavior: {}", gap.expected));
                }
                for file in &gap.affected_files {
                    task = task.with_affected_file(file);
                }
                task
            })
            .collect()
    }

    /// Render the verdict as a Markdown document suitable for export or display.
    ///
    /// Sections with no backing data (`## Requirement Assessments`, `## Gaps`,
    /// `## Acceptance Commands`, `## Acceptance Criteria`) are **omitted entirely**
    /// when their respective `Vec`s are empty, keeping the output concise.
    pub fn to_markdown(&self) -> String {
        let mut md = String::with_capacity(4096);

        md.push_str(&format!(
            "# Validation Verdict — Cycle {}\n\n\
             **Status**: {}\n\
             **Confidence**: {:.0}%\n\
             **Validator**: {}\n\
             **Duration**: {}ms\n",
            self.cycle_id,
            self.status,
            self.confidence * 100.0,
            self.validator_provider_id,
            self.duration_ms,
        ));

        md.push_str(&format!("\n## Summary\n\n{}\n", self.summary));

        if !self.requirement_assessments.is_empty() {
            md.push_str("\n## Requirement Assessments\n\n");
            for ra in &self.requirement_assessments {
                let icon = match ra.status {
                    FulfillmentStatus::Fulfilled => "✅",
                    FulfillmentStatus::PartiallyFulfilled => "⚠️",
                    FulfillmentStatus::NotFulfilled => "❌",
                };
                md.push_str(&format!(
                    "- {} **{}**: {}\n",
                    icon, ra.requirement, ra.evidence
                ));
            }
        }

        if !self.gaps.is_empty() {
            md.push_str("\n## Gaps\n\n");
            for gap in &self.gaps {
                md.push_str(&format!("### [{}] {}\n\n", gap.severity, gap.requirement));
                md.push_str(&format!("- **Expected**: {}\n", gap.expected));
                md.push_str(&format!("- **Actual**: {}\n", gap.actual));
                md.push_str(&format!("- **Action**: {}\n", gap.suggested_action));
                if !gap.affected_files.is_empty() {
                    md.push_str(&format!("- **Files**: {}\n", gap.affected_files.join(", ")));
                }
                md.push('\n');
            }
        }

        if !self.command_results.is_empty() {
            md.push_str("\n## Acceptance Commands\n\n");
            md.push_str("| Command | Status | Duration |\n");
            md.push_str("|---------|--------|----------|\n");
            for cmd in &self.command_results {
                let status = if cmd.passed { "✅ PASS" } else { "❌ FAIL" };
                md.push_str(&format!(
                    "| `{}` | {} | {}ms |\n",
                    cmd.command, status, cmd.duration_ms
                ));
            }
        }

        if !self.criteria_results.is_empty() {
            md.push_str("\n## Acceptance Criteria\n\n");
            for c in &self.criteria_results {
                let icon = if c.passed { "✅" } else { "❌" };
                md.push_str(&format!("- {} {}\n", icon, c.criterion));
            }
        }

        md
    }

    /// Render the verdict as a human-readable summary for prompt injection.
    pub fn to_prompt_summary(&self) -> String {
        let mut parts = Vec::new();

        // Status header
        let status_line = match self.status {
            FulfillmentStatus::Fulfilled => "Previous cycle FULFILLED all requirements.".to_owned(),
            FulfillmentStatus::PartiallyFulfilled => {
                format!(
                    "Previous cycle PARTIALLY FULFILLED requirements ({} gaps remaining).",
                    self.gaps.len()
                )
            }
            FulfillmentStatus::NotFulfilled => {
                "Previous cycle did NOT FULFILL requirements. Major rework needed.".to_owned()
            }
        };
        parts.push(status_line);
        parts.push(String::new());

        // Validator summary
        if !self.summary.is_empty() {
            parts.push(format!("Validator assessment: {}", self.summary));
            parts.push(String::new());
        }

        // Requirement assessments
        if !self.requirement_assessments.is_empty() {
            parts.push("Per-requirement assessment:".to_owned());
            for ra in &self.requirement_assessments {
                let icon = match ra.status {
                    FulfillmentStatus::Fulfilled => "FULFILLED",
                    FulfillmentStatus::PartiallyFulfilled => "PARTIAL",
                    FulfillmentStatus::NotFulfilled => "NOT MET",
                };
                parts.push(format!("  - [{}] {}", icon, ra.requirement));
                if !ra.evidence.is_empty() {
                    parts.push(format!("    Evidence: {}", ra.evidence));
                }
                if !ra.contributing_tasks.is_empty() {
                    parts.push(format!("    Tasks: {}", ra.contributing_tasks.join(", ")));
                }
            }
            parts.push(String::new());
        }

        // Gaps (most important for continuation cycles)
        if !self.gaps.is_empty() {
            parts.push("Requirement gaps to address:".to_owned());
            for gap in &self.gaps {
                let severity = match gap.severity {
                    GapSeverity::Critical => "CRITICAL",
                    GapSeverity::Major => "MAJOR",
                    GapSeverity::Minor => "MINOR",
                };
                parts.push(format!("  - [{}] {}", severity, gap.requirement));
                parts.push(format!("    Expected: {}", gap.expected));
                parts.push(format!("    Actual: {}", gap.actual));
                parts.push(format!("    Action: {}", gap.suggested_action));
                if !gap.affected_files.is_empty() {
                    parts.push(format!("    Files: {}", gap.affected_files.join(", ")));
                }
            }
            parts.push(String::new());
        }

        // Failed acceptance criteria
        let failed_criteria: Vec<&AcceptanceCriterionResult> =
            self.criteria_results.iter().filter(|c| !c.passed).collect();
        if !failed_criteria.is_empty() {
            parts.push("Failed acceptance criteria:".to_owned());
            for c in failed_criteria {
                let task_ref = if c.task_title.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", c.task_title)
                };
                parts.push(format!("  - {}{}: {}", c.criterion, task_ref, c.evidence));
            }
            parts.push(String::new());
        }

        // Failed commands
        let failed_commands: Vec<&AcceptanceCommandResult> =
            self.command_results.iter().filter(|c| !c.passed).collect();
        if !failed_commands.is_empty() {
            parts.push("Failed acceptance commands:".to_owned());
            for c in failed_commands {
                parts.push(format!("  - `{}` (exit code {})", c.command, c.exit_code));
                if !c.stderr.is_empty() {
                    let truncated = if c.stderr.len() > 500 {
                        format!("{}... [truncated]", &c.stderr[..500])
                    } else {
                        c.stderr.clone()
                    };
                    parts.push(format!("    stderr: {}", truncated));
                }
            }
            parts.push(String::new());
        }

        parts.join("\n")
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

// ── Raw deserialization structs (lenient parsing) ──────────────────────────

#[derive(Deserialize)]
struct RawVerdict {
    status: String,
    requirement_assessments: Option<Vec<RawAssessment>>,
    gaps: Option<Vec<RawGap>>,
    criteria_results: Option<Vec<RawCriterionResult>>,
    summary: Option<String>,
    confidence: Option<f64>,
}

#[derive(Deserialize)]
struct RawAssessment {
    requirement: Option<String>,
    status: Option<String>,
    evidence: Option<String>,
    contributing_tasks: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawGap {
    requirement: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    severity: Option<String>,
    suggested_action: Option<String>,
    affected_files: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RawCriterionResult {
    criterion: Option<String>,
    task_title: Option<String>,
    passed: Option<bool>,
    evidence: Option<String>,
}

// ── JSON extraction helper ─────────────────────────────────────────────────

#[cfg(test)]
mod threshold_tests {
    use super::*;
    use mahalaxmi_core::config::AcceptanceThreshold;

    fn verdict(status: FulfillmentStatus, confidence: f64) -> ValidationVerdict {
        ValidationVerdict {
            cycle_id: "c1".to_owned(),
            validator_provider_id: "test".to_owned(),
            status,
            requirement_assessments: vec![],
            gaps: vec![],
            criteria_results: vec![],
            command_results: vec![],
            summary: String::new(),
            confidence,
            duration_ms: 0,
            security_scan: None,
        }
    }

    fn threshold(min_confidence: f64, min_status: &str) -> AcceptanceThreshold {
        AcceptanceThreshold {
            min_confidence,
            min_status: min_status.to_owned(),
            hard_gate: false,
        }
    }

    #[test]
    fn passes_when_exceeds_all_thresholds() {
        let v = verdict(FulfillmentStatus::Fulfilled, 0.95);
        assert!(v.meets_threshold(&threshold(0.7, "partially_fulfilled")));
    }

    #[test]
    fn fails_when_confidence_below_minimum() {
        let v = verdict(FulfillmentStatus::Fulfilled, 0.5);
        assert!(!v.meets_threshold(&threshold(0.7, "partially_fulfilled")));
    }

    #[test]
    fn fails_when_status_below_required_level() {
        let v = verdict(FulfillmentStatus::NotFulfilled, 0.9);
        assert!(!v.meets_threshold(&threshold(0.7, "partially_fulfilled")));
    }

    #[test]
    fn min_status_any_passes_regardless_of_fulfillment() {
        for status in [
            FulfillmentStatus::Fulfilled,
            FulfillmentStatus::PartiallyFulfilled,
            FulfillmentStatus::NotFulfilled,
        ] {
            let v = verdict(status, 0.9);
            assert!(
                v.meets_threshold(&threshold(0.7, "any")),
                "status {:?} should pass with min_status=any",
                status
            );
        }
    }

    #[test]
    fn min_status_fulfilled_rejects_partially_fulfilled() {
        let v = verdict(FulfillmentStatus::PartiallyFulfilled, 0.9);
        assert!(!v.meets_threshold(&threshold(0.7, "fulfilled")));
    }

    #[test]
    fn min_status_fulfilled_rejects_not_fulfilled() {
        let v = verdict(FulfillmentStatus::NotFulfilled, 0.9);
        assert!(!v.meets_threshold(&threshold(0.7, "fulfilled")));
    }

    #[test]
    fn min_status_partially_fulfilled_accepts_fulfilled() {
        let v = verdict(FulfillmentStatus::Fulfilled, 0.9);
        assert!(v.meets_threshold(&threshold(0.7, "partially_fulfilled")));
    }

    #[test]
    fn min_status_partially_fulfilled_accepts_partially_fulfilled() {
        let v = verdict(FulfillmentStatus::PartiallyFulfilled, 0.9);
        assert!(v.meets_threshold(&threshold(0.7, "partially_fulfilled")));
    }

    #[test]
    fn min_status_partially_fulfilled_rejects_not_fulfilled() {
        let v = verdict(FulfillmentStatus::NotFulfilled, 0.9);
        assert!(!v.meets_threshold(&threshold(0.7, "partially_fulfilled")));
    }

    #[test]
    fn default_threshold_passes_typical_verdict() {
        let v = verdict(FulfillmentStatus::PartiallyFulfilled, 0.8);
        assert!(v.meets_threshold(&AcceptanceThreshold::default()));
    }
}

// ── P13: Multi-validator consensus ────────────────────────────────────────

/// Vote record from a single validator for one requirement.
///
/// Used by [`RequirementConsensus`] to track per-provider assessments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementConsensus {
    /// The requirement text that this consensus record covers.
    pub requirement: String,
    /// Individual votes — each tuple is `(provider_id, status)`.
    pub votes: Vec<(String, FulfillmentStatus)>,
    /// The status resolved by applying the consensus strategy across votes.
    pub resolved_status: FulfillmentStatus,
    /// Whether all voters agreed on the same status.
    pub unanimous: bool,
}

/// A merged `ValidationVerdict` produced from multiple independent validators.
///
/// Use [`ConsensusVerdict::merge`] to build one from a collection of verdicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVerdict {
    /// The resolved merged verdict (suitable for downstream consumption).
    pub verdict: ValidationVerdict,
    /// The individual verdicts from each validator, preserved for audit.
    pub individual_verdicts: Vec<ValidationVerdict>,
    /// Per-requirement breakdown of how each validator voted.
    pub requirement_consensus: Vec<RequirementConsensus>,
    /// The consensus strategy that was applied.
    pub strategy: String,
    /// Number of individual validators that agreed with the resolved status.
    pub agreement_count: usize,
    /// Total number of validators that participated.
    pub total_validators: usize,
}

impl ConsensusVerdict {
    /// Merge multiple [`ValidationVerdict`]s into a single consensus verdict.
    ///
    /// # Strategies
    ///
    /// | `strategy`   | Resolved `status` |
    /// |--------------|-------------------|
    /// | `"any_pass"` | `Fulfilled` if *any* verdict is `Fulfilled`; else `PartiallyFulfilled` if *any* is partial; else `NotFulfilled`. |
    /// | `"all_pass"` | `Fulfilled` only when *all* are `Fulfilled`; `NotFulfilled` if *any* is `NotFulfilled`; else `PartiallyFulfilled`. |
    /// | `"majority"` *(default)* | Most common status wins; ties are broken conservatively (`NotFulfilled > PartiallyFulfilled > Fulfilled`). |
    ///
    /// An empty `verdicts` slice returns a safe fallback with `status = NotFulfilled`
    /// and zero counters — it never panics.
    pub fn merge(verdicts: Vec<ValidationVerdict>, strategy: &str) -> Self {
        if verdicts.is_empty() {
            return Self {
                verdict: ValidationVerdict {
                    cycle_id: String::new(),
                    validator_provider_id: String::new(),
                    status: FulfillmentStatus::NotFulfilled,
                    requirement_assessments: Vec::new(),
                    gaps: Vec::new(),
                    criteria_results: Vec::new(),
                    command_results: Vec::new(),
                    summary: String::new(),
                    confidence: 0.0,
                    duration_ms: 0,
                    security_scan: None,
                },
                individual_verdicts: Vec::new(),
                requirement_consensus: Vec::new(),
                strategy: strategy.to_owned(),
                agreement_count: 0,
                total_validators: 0,
            };
        }

        let total_validators = verdicts.len();

        if total_validators == 1 {
            let v = verdicts.into_iter().next().unwrap();
            return Self {
                verdict: v.clone(),
                individual_verdicts: vec![v],
                requirement_consensus: Vec::new(),
                strategy: strategy.to_owned(),
                agreement_count: 1,
                total_validators: 1,
            };
        }

        let resolved_status = resolve_status(&verdicts, strategy);

        let avg_confidence =
            verdicts.iter().map(|v| v.confidence).sum::<f64>() / total_validators as f64;
        let max_duration = verdicts.iter().map(|v| v.duration_ms).max().unwrap_or(0);

        let agreement_count = verdicts
            .iter()
            .filter(|v| v.status == resolved_status)
            .count();

        let mut provider_ids: Vec<&str> = verdicts
            .iter()
            .map(|v| v.validator_provider_id.as_str())
            .collect();
        provider_ids.sort_unstable();
        let consensus_provider_id = format!("consensus({})", provider_ids.join(","));

        let merged_gaps = merge_gaps_by_majority(&verdicts);
        let requirement_consensus = build_requirement_consensus(&verdicts);

        let first = &verdicts[0];
        let summary = format!(
            "{}/{} validators agreed ({} strategy): {}",
            agreement_count, total_validators, strategy, first.summary
        );

        let verdict = ValidationVerdict {
            cycle_id: first.cycle_id.clone(),
            validator_provider_id: consensus_provider_id,
            status: resolved_status,
            requirement_assessments: first.requirement_assessments.clone(),
            gaps: merged_gaps,
            criteria_results: first.criteria_results.clone(),
            command_results: first.command_results.clone(),
            summary,
            confidence: avg_confidence,
            duration_ms: max_duration,
            security_scan: None,
        };

        Self {
            verdict,
            individual_verdicts: verdicts,
            requirement_consensus,
            strategy: strategy.to_owned(),
            agreement_count,
            total_validators,
        }
    }
}

/// Resolve the overall status from multiple verdicts using the given strategy.
fn resolve_status(verdicts: &[ValidationVerdict], strategy: &str) -> FulfillmentStatus {
    match strategy {
        "any_pass" => {
            if verdicts
                .iter()
                .any(|v| v.status == FulfillmentStatus::Fulfilled)
            {
                FulfillmentStatus::Fulfilled
            } else if verdicts
                .iter()
                .any(|v| v.status == FulfillmentStatus::PartiallyFulfilled)
            {
                FulfillmentStatus::PartiallyFulfilled
            } else {
                FulfillmentStatus::NotFulfilled
            }
        }
        "all_pass" => {
            if verdicts
                .iter()
                .all(|v| v.status == FulfillmentStatus::Fulfilled)
            {
                FulfillmentStatus::Fulfilled
            } else if verdicts
                .iter()
                .any(|v| v.status == FulfillmentStatus::NotFulfilled)
            {
                FulfillmentStatus::NotFulfilled
            } else {
                FulfillmentStatus::PartiallyFulfilled
            }
        }
        _ => {
            // majority: most common status wins; ties break conservatively
            let fulfilled_count = verdicts
                .iter()
                .filter(|v| v.status == FulfillmentStatus::Fulfilled)
                .count();
            let partial_count = verdicts
                .iter()
                .filter(|v| v.status == FulfillmentStatus::PartiallyFulfilled)
                .count();
            let not_count = verdicts
                .iter()
                .filter(|v| v.status == FulfillmentStatus::NotFulfilled)
                .count();
            let max_count = fulfilled_count.max(partial_count).max(not_count);
            if not_count == max_count {
                FulfillmentStatus::NotFulfilled
            } else if partial_count == max_count {
                FulfillmentStatus::PartiallyFulfilled
            } else {
                FulfillmentStatus::Fulfilled
            }
        }
    }
}

/// Merge gaps from multiple verdicts: include a gap if it appears in a majority of verdicts.
///
/// Uses case-insensitive word-overlap (Jaccard similarity ≥ 0.3) to identify the same gap
/// across different providers.
fn merge_gaps_by_majority(verdicts: &[ValidationVerdict]) -> Vec<RequirementGap> {
    let threshold = (verdicts.len() / 2) + 1;
    let mut representative_gaps: Vec<(RequirementGap, usize)> = Vec::new();

    for verdict in verdicts {
        for gap in &verdict.gaps {
            let matched = representative_gaps
                .iter_mut()
                .find(|(rep, _)| gaps_are_similar(&rep.requirement, &gap.requirement));
            if let Some((_, count)) = matched {
                *count += 1;
            } else {
                representative_gaps.push((gap.clone(), 1));
            }
        }
    }

    representative_gaps
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(gap, _)| gap)
        .collect()
}

/// Returns true if two requirement strings are considered similar (same gap).
///
/// Uses case-insensitive substring containment as the primary heuristic,
/// falling back to a Jaccard word-overlap score ≥ 0.3.
fn gaps_are_similar(a: &str, b: &str) -> bool {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    if a_lower.contains(b_lower.as_str()) || b_lower.contains(a_lower.as_str()) {
        return true;
    }
    let a_words: HashSet<&str> = a_lower.split_whitespace().collect();
    let b_words: HashSet<&str> = b_lower.split_whitespace().collect();
    let intersection = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();
    if union == 0 {
        return false;
    }
    (intersection as f64 / union as f64) >= 0.3
}

/// Build per-requirement consensus records across all validators.
fn build_requirement_consensus(verdicts: &[ValidationVerdict]) -> Vec<RequirementConsensus> {
    let mut all_requirements: Vec<String> = verdicts
        .iter()
        .flat_map(|v| {
            v.requirement_assessments
                .iter()
                .map(|ra| ra.requirement.clone())
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    all_requirements.sort();

    all_requirements
        .into_iter()
        .map(|req| {
            let votes: Vec<(String, FulfillmentStatus)> = verdicts
                .iter()
                .filter_map(|v| {
                    v.requirement_assessments
                        .iter()
                        .find(|ra| ra.requirement == req)
                        .map(|ra| (v.validator_provider_id.clone(), ra.status))
                })
                .collect();
            let statuses: Vec<FulfillmentStatus> = votes.iter().map(|(_, s)| *s).collect();
            let unanimous = statuses.windows(2).all(|w| w[0] == w[1]);
            let resolved_status = if statuses.is_empty() {
                FulfillmentStatus::PartiallyFulfilled
            } else {
                let fulfilled = statuses
                    .iter()
                    .filter(|&&s| s == FulfillmentStatus::Fulfilled)
                    .count();
                let partial = statuses
                    .iter()
                    .filter(|&&s| s == FulfillmentStatus::PartiallyFulfilled)
                    .count();
                let not_count = statuses
                    .iter()
                    .filter(|&&s| s == FulfillmentStatus::NotFulfilled)
                    .count();
                let max_c = fulfilled.max(partial).max(not_count);
                if not_count == max_c {
                    FulfillmentStatus::NotFulfilled
                } else if partial == max_c {
                    FulfillmentStatus::PartiallyFulfilled
                } else {
                    FulfillmentStatus::Fulfilled
                }
            };
            RequirementConsensus {
                requirement: req,
                votes,
                resolved_status,
                unanimous,
            }
        })
        .collect()
}

/// Extract a JSON block from text that may contain markdown fences or prose.
fn extract_json_block(text: &str) -> Option<&str> {
    // Try markdown code fence first: ```json ... ```
    if let Some(start) = text.find("```json") {
        let json_start = start + 7;
        if let Some(end) = text[json_start..].find("```") {
            let block = text[json_start..json_start + end].trim();
            if !block.is_empty() {
                return Some(block);
            }
        }
    }

    // Try plain code fence: ``` ... ```
    if let Some(start) = text.find("```") {
        let fence_start = start + 3;
        if let Some(end) = text[fence_start..].find("```") {
            let block = text[fence_start..fence_start + end].trim();
            if block.starts_with('{') {
                return Some(block);
            }
        }
    }

    // Try bare JSON object
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return Some(&text[start..=end]);
            }
        }
    }

    None
}

#[cfg(test)]
mod to_markdown_tests {
    use super::*;

    fn base_verdict() -> ValidationVerdict {
        ValidationVerdict {
            cycle_id: "cycle-42".to_owned(),
            validator_provider_id: "claude-code".to_owned(),
            status: FulfillmentStatus::PartiallyFulfilled,
            requirement_assessments: vec![],
            gaps: vec![],
            criteria_results: vec![],
            command_results: vec![],
            summary: "Core auth works but OAuth is missing.".to_owned(),
            confidence: 0.75,
            duration_ms: 1234,
            security_scan: None,
        }
    }

    #[test]
    fn markdown_contains_header_and_summary() {
        let md = base_verdict().to_markdown();
        assert!(
            md.contains("# Validation Verdict"),
            "missing header: {}",
            md
        );
        assert!(md.contains("## Summary"), "missing Summary section");
        assert!(md.contains("Core auth works but OAuth is missing."));
    }

    #[test]
    fn markdown_uses_icons_in_assessments() {
        let mut v = base_verdict();
        v.requirement_assessments = vec![
            RequirementAssessment {
                requirement: "User login".to_owned(),
                status: FulfillmentStatus::Fulfilled,
                evidence: "works".to_owned(),
                contributing_tasks: vec![],
            },
            RequirementAssessment {
                requirement: "OAuth".to_owned(),
                status: FulfillmentStatus::PartiallyFulfilled,
                evidence: "partial".to_owned(),
                contributing_tasks: vec![],
            },
            RequirementAssessment {
                requirement: "2FA".to_owned(),
                status: FulfillmentStatus::NotFulfilled,
                evidence: "missing".to_owned(),
                contributing_tasks: vec![],
            },
        ];
        let md = v.to_markdown();
        assert!(md.contains("✅"), "missing fulfilled icon");
        assert!(md.contains("⚠️"), "missing partial icon");
        assert!(md.contains("❌"), "missing not-fulfilled icon");
    }

    #[test]
    fn markdown_omits_gaps_section_when_empty() {
        let md = base_verdict().to_markdown();
        assert!(
            !md.contains("## Gaps"),
            "Gaps heading should be omitted when no gaps"
        );
    }

    #[test]
    fn markdown_includes_gaps_section_when_non_empty() {
        let mut v = base_verdict();
        v.gaps = vec![RequirementGap {
            requirement: "OAuth".to_owned(),
            expected: "Google login".to_owned(),
            actual: "not implemented".to_owned(),
            severity: GapSeverity::Critical,
            suggested_action: "Add OAuth routes".to_owned(),
            affected_files: vec!["src/oauth.rs".to_owned()],
        }];
        let md = v.to_markdown();
        assert!(md.contains("## Gaps"), "Gaps section should be present");
        assert!(
            md.contains("### [Critical]"),
            "gap severity heading missing"
        );
        assert!(md.contains("**Expected**"), "expected field missing");
        assert!(md.contains("**Actual**"), "actual field missing");
        assert!(md.contains("**Action**"), "action field missing");
        assert!(md.contains("**Files**"), "files field missing");
    }

    #[test]
    fn markdown_omits_acceptance_sections_when_empty() {
        let md = base_verdict().to_markdown();
        assert!(
            !md.contains("## Acceptance Commands"),
            "commands section should be absent"
        );
        assert!(
            !md.contains("## Acceptance Criteria"),
            "criteria section should be absent"
        );
    }

    #[test]
    fn markdown_includes_acceptance_commands_table() {
        let mut v = base_verdict();
        v.command_results = vec![AcceptanceCommandResult {
            command: "cargo test".to_owned(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 500,
            passed: true,
        }];
        let md = v.to_markdown();
        assert!(
            md.contains("## Acceptance Commands"),
            "commands section missing"
        );
        assert!(
            md.contains("| Command | Status | Duration |"),
            "table header missing"
        );
        assert!(md.contains("cargo test"), "command text missing");
    }

    #[test]
    fn markdown_includes_acceptance_criteria() {
        let mut v = base_verdict();
        v.criteria_results = vec![
            AcceptanceCriterionResult {
                criterion: "build passes".to_owned(),
                task_title: String::new(),
                passed: true,
                evidence: String::new(),
            },
            AcceptanceCriterionResult {
                criterion: "tests pass".to_owned(),
                task_title: String::new(),
                passed: false,
                evidence: String::new(),
            },
        ];
        let md = v.to_markdown();
        assert!(
            md.contains("## Acceptance Criteria"),
            "criteria section missing"
        );
        assert!(md.contains("✅"), "pass icon missing");
        assert!(md.contains("❌"), "fail icon missing");
    }
}

#[cfg(test)]
mod consensus_tests {
    use super::*;

    fn make_verdict(
        status: FulfillmentStatus,
        confidence: f64,
        provider: &str,
        cycle: &str,
    ) -> ValidationVerdict {
        ValidationVerdict {
            cycle_id: cycle.to_owned(),
            validator_provider_id: provider.to_owned(),
            status,
            requirement_assessments: vec![],
            gaps: vec![],
            criteria_results: vec![],
            command_results: vec![],
            summary: format!("summary from {}", provider),
            confidence,
            duration_ms: 100,
            security_scan: None,
        }
    }

    #[test]
    fn single_verdict_pass_through_with_agreement_count_one() {
        let v = make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p1", "c1");
        let cv = ConsensusVerdict::merge(vec![v.clone()], "majority");
        assert_eq!(cv.total_validators, 1);
        assert_eq!(cv.agreement_count, 1);
        assert_eq!(cv.verdict.status, FulfillmentStatus::Fulfilled);
        assert_eq!(cv.verdict.confidence, 0.9);
    }

    #[test]
    fn three_fulfilled_majority_returns_fulfilled() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p1", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.8, "p2", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.85, "p3", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "majority");
        assert_eq!(cv.verdict.status, FulfillmentStatus::Fulfilled);
        assert_eq!(cv.agreement_count, 3);
        assert_eq!(cv.total_validators, 3);
    }

    #[test]
    fn two_fulfilled_one_not_majority_returns_fulfilled() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p1", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.8, "p2", "c1"),
            make_verdict(FulfillmentStatus::NotFulfilled, 0.3, "p3", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "majority");
        assert_eq!(cv.verdict.status, FulfillmentStatus::Fulfilled);
        assert_eq!(cv.agreement_count, 2);
    }

    #[test]
    fn two_not_fulfilled_one_fulfilled_majority_returns_not_fulfilled() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::NotFulfilled, 0.3, "p1", "c1"),
            make_verdict(FulfillmentStatus::NotFulfilled, 0.4, "p2", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p3", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "majority");
        assert_eq!(cv.verdict.status, FulfillmentStatus::NotFulfilled);
        assert_eq!(cv.agreement_count, 2);
    }

    #[test]
    fn any_pass_one_fulfilled_returns_fulfilled() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::NotFulfilled, 0.3, "p1", "c1"),
            make_verdict(FulfillmentStatus::NotFulfilled, 0.4, "p2", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p3", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "any_pass");
        assert_eq!(cv.verdict.status, FulfillmentStatus::Fulfilled);
    }

    #[test]
    fn all_pass_two_fulfilled_one_partial_returns_partial() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p1", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.85, "p2", "c1"),
            make_verdict(FulfillmentStatus::PartiallyFulfilled, 0.6, "p3", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "all_pass");
        assert_eq!(cv.verdict.status, FulfillmentStatus::PartiallyFulfilled);
    }

    #[test]
    fn confidence_averaged_across_all_verdicts() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::Fulfilled, 0.6, "p1", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.8, "p2", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 1.0, "p3", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "majority");
        let expected = (0.6 + 0.8 + 1.0) / 3.0;
        assert!(
            (cv.verdict.confidence - expected).abs() < 1e-9,
            "expected avg confidence ~{:.4}",
            expected
        );
    }

    #[test]
    fn validator_provider_id_consensus_format() {
        let verdicts = vec![
            make_verdict(FulfillmentStatus::Fulfilled, 0.9, "claude-code", "c1"),
            make_verdict(FulfillmentStatus::Fulfilled, 0.8, "gpt-4o", "c1"),
        ];
        let cv = ConsensusVerdict::merge(verdicts, "majority");
        assert_eq!(
            cv.verdict.validator_provider_id,
            "consensus(claude-code,gpt-4o)"
        );
    }

    #[test]
    fn to_markdown_full_verdict_contains_all_sections() {
        let mut v = make_verdict(FulfillmentStatus::Fulfilled, 0.95, "claude-code", "cycle-1");
        v.summary = "All requirements met.".to_owned();
        v.requirement_assessments = vec![RequirementAssessment {
            requirement: "Feature X".to_owned(),
            status: FulfillmentStatus::Fulfilled,
            evidence: "implemented".to_owned(),
            contributing_tasks: vec![],
        }];
        v.gaps = vec![RequirementGap {
            requirement: "Feature Y".to_owned(),
            expected: "expected".to_owned(),
            actual: "missing".to_owned(),
            severity: GapSeverity::Major,
            suggested_action: "implement it".to_owned(),
            affected_files: vec![],
        }];
        v.command_results = vec![AcceptanceCommandResult {
            command: "cargo test".to_owned(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 100,
            passed: true,
        }];
        v.criteria_results = vec![AcceptanceCriterionResult {
            criterion: "build passes".to_owned(),
            task_title: String::new(),
            passed: true,
            evidence: String::new(),
        }];
        let md = v.to_markdown();
        assert!(md.contains("# Validation Verdict"), "header missing");
        assert!(md.contains("## Summary"), "summary missing");
        assert!(
            md.contains("✅") || md.contains("⚠️") || md.contains("❌"),
            "icons missing"
        );
    }

    #[test]
    fn to_markdown_omits_gaps_heading_when_empty() {
        let v = make_verdict(FulfillmentStatus::Fulfilled, 0.9, "p1", "c1");
        let md = v.to_markdown();
        assert!(
            !md.contains("## Gaps"),
            "Gaps heading should be absent when gaps vec is empty"
        );
    }

    #[test]
    fn validation_verdict_deserializes_without_security_scan_field() {
        // JSON from a pre-Phase-8 cycle that has no security_scan field.
        let json = r#"{
            "cycle_id": "c1",
            "validator_provider_id": "claude-code",
            "status": "Fulfilled",
            "requirement_assessments": [],
            "gaps": [],
            "criteria_results": [],
            "command_results": [],
            "summary": "all good",
            "confidence": 0.95,
            "duration_ms": 500
        }"#;
        let verdict: ValidationVerdict =
            serde_json::from_str(json).expect("must deserialize without security_scan");
        assert!(
            verdict.security_scan.is_none(),
            "security_scan must default to None when absent"
        );
    }

    #[test]
    fn security_scan_dto_default_has_zero_counts_and_passed_true() {
        let dto = SecurityScanDto::default();
        assert_eq!(dto.critical_count, 0);
        assert_eq!(dto.high_count, 0);
        assert_eq!(dto.medium_count, 0);
        assert_eq!(dto.low_count, 0);
        assert!(dto.findings.is_empty());
        assert!(
            dto.passed,
            "default SecurityScanDto must have passed = true"
        );
    }
}
