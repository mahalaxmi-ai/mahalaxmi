//! Internationalization message key constants.
//!
//! All user-facing strings in Mahalaxmi are referenced by these keys.
//! The actual translations live in `.ftl` locale files.

/// Message key constants organized by domain.
pub mod keys {
    /// Error message keys.
    pub mod error {
        /// Configuration file not found. Args: path
        pub const CONFIG_FILE_NOT_FOUND: &str = "error-config-file-not-found";
        /// Failed to parse configuration. Args: reason
        pub const CONFIG_PARSE_FAILED: &str = "error-config-parse-failed";
        /// Configuration validation failed. Args: details
        pub const CONFIG_VALIDATION_FAILED: &str = "error-config-validation-failed";
        /// Unsupported locale. Args: locale
        pub const LOCALE_NOT_SUPPORTED: &str = "error-locale-not-supported";
        /// Failed to initialize logging. Args: reason
        pub const LOG_INIT_FAILED: &str = "error-log-init-failed";
        /// Failed to create log directory. Args: path, reason
        pub const LOG_DIR_CREATE_FAILED: &str = "error-log-dir-create-failed";
        /// Failed to launch application. Args: reason
        pub const APP_LAUNCH_FAILED: &str = "error-app-launch-failed";
    }

    /// Configuration message keys.
    pub mod config {
        /// Configuration loaded successfully. Args: path
        pub const LOADED: &str = "config-loaded-successfully";
        /// Using default configuration.
        pub const USING_DEFAULTS: &str = "config-using-default";
        /// Environment variable override applied. Args: var
        pub const ENV_OVERRIDE: &str = "config-env-override";
        /// Environment variable has invalid value. Args: var, value
        pub const ENV_OVERRIDE_INVALID: &str = "config-env-override-invalid";
        /// Default configuration generated. Args: path
        pub const GENERATED_SUCCESSFULLY: &str = "config-generated-successfully";
        /// Configuration file already exists. Args: path
        pub const ALREADY_EXISTS: &str = "config-already-exists";
    }

    /// Logging message keys.
    pub mod logging {
        /// Logging initialized. Args: level
        pub const INITIALIZED: &str = "logging-initialized";
        /// RUST_LOG environment variable detected, overriding configured level.
        pub const RUST_LOG_OVERRIDE: &str = "logging-rust-log-override";
        /// Log file path. Args: path
        pub const FILE_PATH: &str = "logging-file-path";
        /// Failed to create log directory, falling back to console-only. Args: path
        pub const DIR_CREATE_FAILED_FALLBACK: &str = "logging-dir-create-failed-fallback";
    }

    /// Configuration validation message keys.
    pub mod validation {
        /// Invalid log level. Args: level, valid
        pub const INVALID_LOG_LEVEL: &str = "validation-invalid-log-level";
        /// Workers out of range. Args: value, min, max
        pub const WORKERS_OUT_OF_RANGE: &str = "validation-workers-out-of-range";
        /// Manager timeout too low. Args: value, min
        pub const MANAGER_TIMEOUT_TOO_LOW: &str = "validation-manager-timeout-too-low";
        /// Worker timeout too low. Args: value, min
        pub const WORKER_TIMEOUT_TOO_LOW: &str = "validation-worker-timeout-too-low";
        /// Offline grace days too low. Args: value, min
        pub const OFFLINE_GRACE_TOO_LOW: &str = "validation-offline-grace-too-low";
        /// Invalid consensus strategy. Args: value, valid
        pub const INVALID_CONSENSUS_STRATEGY: &str = "validation-invalid-consensus-strategy";
        /// Invalid data directory path.
        pub const INVALID_DATA_DIRECTORY: &str = "validation-invalid-data-directory";
        /// Empty default provider.
        pub const EMPTY_DEFAULT_PROVIDER: &str = "validation-empty-default-provider";
        /// Invalid theme. Args: value, valid
        pub const INVALID_THEME: &str = "validation-invalid-theme";
        /// Font size out of range. Args: value, min, max
        pub const FONT_SIZE_OUT_OF_RANGE: &str = "validation-font-size-out-of-range";
        /// max_batch_retries must be >= 1. Args: value
        pub const INVALID_MAX_BATCH_RETRIES: &str = "validation-invalid-max-batch-retries";
        /// max_total_batches must be >= 2. Args: value
        pub const INVALID_MAX_TOTAL_BATCHES: &str = "validation-invalid-max-total-batches";
        /// stall_detection_threshold must be >= 2. Args: value
        pub const INVALID_STALL_DETECTION_THRESHOLD: &str =
            "validation-invalid-stall-detection-threshold";
    }

    /// PTY terminal message keys.
    pub mod pty {
        /// Failed to open PTY. Args: reason
        pub const OPEN_FAILED: &str = "error-pty-open-failed";
        /// Failed to spawn process in PTY. Args: program, reason
        pub const SPAWN_FAILED: &str = "error-pty-spawn-failed";
        /// Failed to write to PTY. Args: terminal_id, reason
        pub const WRITE_FAILED: &str = "error-pty-write-failed";
        /// Failed to read from PTY. Args: terminal_id, reason
        pub const READ_FAILED: &str = "error-pty-read-failed";
        /// Failed to resize PTY. Args: terminal_id, rows, cols, reason
        pub const RESIZE_FAILED: &str = "error-pty-resize-failed";
        /// Failed to kill PTY process. Args: terminal_id, reason
        pub const KILL_FAILED: &str = "error-pty-kill-failed";
        /// Failed to wait on PTY process. Args: terminal_id, reason
        pub const WAIT_FAILED: &str = "error-pty-wait-failed";
        /// Terminal not found. Args: terminal_id
        pub const TERMINAL_NOT_FOUND: &str = "error-pty-terminal-not-found";
        /// Maximum concurrent terminals reached. Args: max
        pub const MAX_CONCURRENT_REACHED: &str = "error-pty-max-concurrent-reached";
        /// PTY process spawned. Args: terminal_id, program
        pub const PROCESS_SPAWNED: &str = "pty-process-spawned";
        /// PTY process exited. Args: terminal_id, exit_code
        pub const PROCESS_EXITED: &str = "pty-process-exited";
        /// Terminal session closed. Args: terminal_id
        pub const SESSION_CLOSED: &str = "pty-session-closed";
        /// Terminal resized. Args: terminal_id, rows, cols
        pub const RESIZED: &str = "pty-resized";
        /// Reader EOF. Args: terminal_id
        pub const READER_EOF: &str = "pty-reader-eof";
        /// Reader error. Args: terminal_id, reason
        pub const READER_ERROR: &str = "pty-reader-error";
    }

    /// Application lifecycle message keys.
    pub mod app {
        /// Application starting. Args: version
        pub const STARTING: &str = "app-starting";
        /// Application is ready.
        pub const READY: &str = "app-ready";
        /// Application shutting down.
        pub const SHUTTING_DOWN: &str = "app-shutting-down";
    }

    /// AI provider message keys.
    pub mod provider {
        /// Provider credentials are missing. Args: provider, env_var
        pub const CREDENTIALS_MISSING: &str = "error-provider-credentials-missing";
        /// Provider credentials are invalid. Args: provider, reason
        pub const CREDENTIALS_INVALID: &str = "error-provider-credentials-invalid";
        /// Provider not found in registry. Args: provider_id
        pub const NOT_FOUND: &str = "error-provider-not-found";
        /// No default provider configured.
        pub const NO_DEFAULT: &str = "error-provider-no-default";
        /// Provider command build failed. Args: provider, reason
        pub const COMMAND_BUILD_FAILED: &str = "error-provider-command-build-failed";
        /// Provider registered. Args: provider, id
        pub const REGISTERED: &str = "provider-registered";
        /// Provider set as default. Args: provider
        pub const SET_DEFAULT: &str = "provider-set-default";
        /// Provider credentials validated. Args: provider
        pub const CREDENTIALS_VALID: &str = "provider-credentials-valid";
        /// Provider validation started. Args: provider
        pub const VALIDATING: &str = "provider-validating";
        /// Provider listing header.
        pub const LIST_HEADER: &str = "provider-list-header";
    }

    /// Orchestration engine message keys.
    pub mod orchestration {
        /// Invalid state transition. Args: from, to
        pub const INVALID_TRANSITION: &str = "error-orchestration-invalid-transition";
        /// Circular dependency detected. Args: cycle
        pub const CIRCULAR_DEPENDENCY: &str = "error-orchestration-circular-dependency";
        /// Worker not found. Args: worker_id
        pub const WORKER_NOT_FOUND: &str = "error-orchestration-worker-not-found";
        /// Maximum retries exceeded. Args: worker_id, max_retries
        pub const MAX_RETRIES_EXCEEDED: &str = "error-orchestration-max-retries-exceeded";
        /// No proposals received.
        pub const NO_PROPOSALS: &str = "error-orchestration-no-proposals";
        /// Plan validation failed. Args: errors
        pub const PLAN_VALIDATION_FAILED: &str = "error-orchestration-plan-validation-failed";
        /// Consensus failed. Args: reason
        pub const CONSENSUS_FAILED: &str = "error-orchestration-consensus-failed";
        /// Queue is full. Args: max
        pub const QUEUE_FULL: &str = "error-orchestration-queue-full";
        /// Manager timed out. Args: manager_id, timeout
        pub const MANAGER_TIMEOUT: &str = "error-orchestration-manager-timeout";
        /// Worker timed out. Args: worker_id, timeout
        pub const WORKER_TIMEOUT: &str = "error-orchestration-worker-timeout";
        /// Cycle started. Args: cycle_id
        pub const CYCLE_STARTED: &str = "orchestration-cycle-started";
        /// State changed. Args: from, to
        pub const STATE_CHANGED: &str = "orchestration-state-changed";
        /// Manager completed. Args: manager_id, task_count
        pub const MANAGER_COMPLETED: &str = "orchestration-manager-completed";
        /// Consensus reached. Args: agreed, dissenting
        pub const CONSENSUS_REACHED: &str = "orchestration-consensus-reached";
        /// Plan created. Args: phases, workers
        pub const PLAN_CREATED: &str = "orchestration-plan-created";
        /// Worker started. Args: worker_id, task
        pub const WORKER_STARTED: &str = "orchestration-worker-started";
        /// Worker completed. Args: worker_id, duration
        pub const WORKER_COMPLETED: &str = "orchestration-worker-completed";
        /// Worker failed. Args: worker_id, error
        pub const WORKER_FAILED: &str = "orchestration-worker-failed";
        /// Cycle completed. Args: duration, success_rate
        pub const CYCLE_COMPLETED: &str = "orchestration-cycle-completed";
        /// Worker retrying. Args: worker_id, attempt, max
        pub const WORKER_RETRYING: &str = "orchestration-worker-retrying";
        /// Manager quorum not met. Args: received, expected, min_quorum
        pub const QUORUM_NOT_MET: &str = "error-orchestration-quorum-not-met";
        /// File conflict serialization edge added. Args: later_task, earlier_task, file
        pub const FILE_CONFLICT_SERIALIZED: &str = "orchestration-file-conflict-serialized";
        /// Per-worker build check failed. Args: worker_id, project_type
        pub const BUILD_CHECK_FAILED: &str = "orchestration-build-check-failed";
        /// Per-worker build check passed. Args: worker_id, project_type
        pub const BUILD_CHECK_PASSED: &str = "orchestration-build-check-passed";
    }

    /// Detection engine message keys.
    pub mod detection {
        /// Rule pattern compilation failed. Args: reason
        pub const RULE_COMPILE_FAILED: &str = "error-detection-rule-compile-failed";
        /// No rules loaded.
        pub const NO_RULES_LOADED: &str = "error-detection-no-rules-loaded";
        /// Invalid pattern. Args: pattern, reason
        pub const INVALID_PATTERN: &str = "error-detection-invalid-pattern";
        /// Rule matched. Args: rule, action
        pub const RULE_MATCHED: &str = "detection-rule-matched";
        /// Rule suppressed by cooldown. Args: rule, remaining_ms
        pub const RULE_COOLDOWN: &str = "detection-rule-cooldown";
        /// Rules loaded. Args: count
        pub const RULES_LOADED: &str = "detection-rules-loaded";
        /// Provider rules applied. Args: provider, count
        pub const PROVIDER_RULES_APPLIED: &str = "detection-provider-rules-applied";
        /// Error pattern detected. Args: pattern, count
        pub const ERROR_PATTERN_DETECTED: &str = "detection-error-pattern-detected";
        /// Root cause hypothesis. Args: category, confidence
        pub const ROOT_CAUSE_HYPOTHESIS: &str = "detection-root-cause-hypothesis";
        /// Recurring error. Args: message, count
        pub const RECURRING_ERROR: &str = "detection-recurring-error";
        /// Action executed. Args: action, rule
        pub const ACTION_EXECUTED: &str = "detection-action-executed";
        /// Cooldown reset. Args: rule_count
        pub const COOLDOWNS_RESET: &str = "detection-cooldowns-reset";
    }

    /// Credential description keys (for CredentialSpec.description_key).
    pub mod credential {
        /// Anthropic API key for Claude Code.
        pub const ANTHROPIC_API_KEY: &str = "credential-anthropic-api-key";
        /// Generic API key credential.
        pub const GENERIC_API_KEY: &str = "credential-generic-api-key";
        /// AWS IAM role credential.
        pub const AWS_IAM_ROLE: &str = "credential-aws-iam-role";
        /// OAuth token credential.
        pub const OAUTH_TOKEN: &str = "credential-oauth-token";
    }

    /// Template system message keys.
    pub mod templates {
        // Error keys (10)
        /// Template not found. Args: template_id
        pub const NOT_FOUND: &str = "error-template-not-found";
        /// Category not found. Args: category_id
        pub const CATEGORY_NOT_FOUND: &str = "error-template-category-not-found";
        /// Template composition failed. Args: reason
        pub const COMPOSITION_FAILED: &str = "error-template-composition-failed";
        /// Include file not found. Args: path
        pub const INCLUDE_NOT_FOUND: &str = "error-template-include-not-found";
        /// Circular include detected. Args: depth
        pub const CIRCULAR_INCLUDE: &str = "error-template-circular-include";
        /// Unresolved placeholder. Args: placeholder
        pub const PLACEHOLDER_UNRESOLVED: &str = "error-template-placeholder-unresolved";
        /// Template validation failed. Args: count
        pub const VALIDATION_FAILED: &str = "error-template-validation-failed";
        /// Template activation failed. Args: reason
        pub const ACTIVATION_FAILED: &str = "error-template-activation-failed";
        /// Catalog load failed. Args: path
        pub const CATALOG_LOAD_FAILED: &str = "error-template-catalog-load-failed";
        /// Invalid version format. Args: version
        pub const INVALID_VERSION: &str = "error-template-invalid-version";

        // Info keys (10)
        /// Catalog loaded. Args: count
        pub const CATALOG_LOADED: &str = "template-catalog-loaded";
        /// Template activated. Args: template_id
        pub const ACTIVATED: &str = "template-activated";
        /// Composition complete. Args: included, overridden
        pub const COMPOSITION_COMPLETE: &str = "template-composition-complete";
        /// Placeholders resolved. Args: count
        pub const PLACEHOLDERS_RESOLVED: &str = "template-placeholders-resolved";
        /// Validation passed. Args: domain
        pub const VALIDATION_PASSED: &str = "template-validation-passed";
        /// Validation warnings. Args: count
        pub const VALIDATION_WARNINGS: &str = "template-validation-warnings";
        /// Include resolved. Args: path
        pub const INCLUDE_RESOLVED: &str = "template-include-resolved";
        /// Provider instructions injected. Args: provider
        pub const PROVIDER_INSTRUCTIONS_INJECTED: &str = "template-provider-instructions-injected";
        /// Project config loaded. Args: path
        pub const PROJECT_CONFIG_LOADED: &str = "template-project-config-loaded";
        /// Domain validator registered. Args: domain
        pub const DOMAIN_VALIDATOR_REGISTERED: &str = "template-domain-validator-registered";
    }

    /// Platform support message keys.
    pub mod platform {
        /// Platform not supported. Args: platform
        pub const UNSUPPORTED: &str = "error-platform-unsupported";
        /// WSL not detected.
        pub const WSL_NOT_DETECTED: &str = "error-platform-wsl-not-detected";
        /// Invalid WSL path. Args: path
        pub const WSL_PATH_INVALID: &str = "error-platform-wsl-path-invalid";
        /// Container too small for layout. Args: width, height
        pub const LAYOUT_NO_SPACE: &str = "error-platform-layout-no-space";
        /// Invalid panel count. Args: count
        pub const LAYOUT_INVALID_COUNT: &str = "error-platform-layout-invalid-count";
        /// Hotkey registration failed (conflict). Args: shortcut
        pub const HOTKEY_REGISTRATION_FAILED: &str = "error-platform-hotkey-registration-failed";
        /// Hotkey parse failed. Args: shortcut
        pub const HOTKEY_PARSE_FAILED: &str = "error-platform-hotkey-parse-failed";
        /// Shutdown timed out. Args: pid, label
        pub const SHUTDOWN_TIMEOUT: &str = "error-platform-shutdown-timeout";
        /// Shutdown failed. Args: pid, label
        pub const SHUTDOWN_FAILED: &str = "error-platform-shutdown-failed";
        /// Shell not found.
        pub const SHELL_NOT_FOUND: &str = "error-platform-shell-not-found";

        /// Platform detected. Args: os, arch
        pub const DETECTED: &str = "platform-detected";
        /// WSL detected. Args: distro, version
        pub const WSL_DETECTED: &str = "platform-wsl-detected";
        /// WSL path translated. Args: from, to
        pub const WSL_PATH_TRANSLATED: &str = "platform-wsl-path-translated";
        /// Layout calculated. Args: panels, rows, cols
        pub const LAYOUT_CALCULATED: &str = "platform-layout-calculated";
        /// Layout optimized. Args: utilization
        pub const LAYOUT_OPTIMIZED: &str = "platform-layout-optimized";
        /// Hotkey registered. Args: command, shortcut
        pub const HOTKEY_REGISTERED: &str = "platform-hotkey-registered";
        /// Hotkey unregistered. Args: command
        pub const HOTKEY_UNREGISTERED: &str = "platform-hotkey-unregistered";
        /// Shutdown initiated. Args: count
        pub const SHUTDOWN_INITIATED: &str = "platform-shutdown-initiated";
        /// Shutdown completed. Args: count, duration
        pub const SHUTDOWN_COMPLETED: &str = "platform-shutdown-completed";
        /// Shell detected. Args: shell, path
        pub const SHELL_DETECTED: &str = "platform-shell-detected";
    }

    /// Memory system message keys.
    pub mod memory {
        // Error keys (10)
        /// Memory entry not found. Args: id
        pub const NOT_FOUND: &str = "error-memory-not-found";
        /// Duplicate memory entry. Args: id
        pub const DUPLICATE: &str = "error-memory-duplicate";
        /// Persistence failed. Args: reason
        pub const PERSISTENCE_FAILED: &str = "error-memory-persistence-failed";
        /// Load failed. Args: reason
        pub const LOAD_FAILED: &str = "error-memory-load-failed";
        /// Invalid confidence score. Args: value
        pub const INVALID_CONFIDENCE: &str = "error-memory-invalid-confidence";
        /// Store is full. Args: max
        pub const STORE_FULL: &str = "error-memory-store-full";
        /// Invalid query. Args: reason
        pub const INVALID_QUERY: &str = "error-memory-invalid-query";
        /// Serialization failed. Args: reason
        pub const SERIALIZATION: &str = "error-memory-serialization";
        /// Invalid entry. Args: reason
        pub const INVALID_ENTRY: &str = "error-memory-invalid-entry";
        /// Session mismatch. Args: expected, actual
        pub const SESSION_MISMATCH: &str = "error-memory-session-mismatch";

        // Info keys (10)
        /// Store created. Args: session_id
        pub const STORE_CREATED: &str = "memory-store-created";
        /// Entry added. Args: title, memory_type
        pub const ENTRY_ADDED: &str = "memory-entry-added";
        /// Entry updated. Args: id
        pub const ENTRY_UPDATED: &str = "memory-entry-updated";
        /// Entry removed. Args: id
        pub const ENTRY_REMOVED: &str = "memory-entry-removed";
        /// Store cleared. Args: count
        pub const STORE_CLEARED: &str = "memory-store-cleared";
        /// Store persisted. Args: path
        pub const PERSISTED: &str = "memory-persisted";
        /// Store loaded. Args: path, count
        pub const LOADED: &str = "memory-loaded";
        /// Query executed. Args: count
        pub const QUERY_EXECUTED: &str = "memory-query-executed";
        /// Memories injected. Args: count, tokens
        pub const INJECTED: &str = "memory-injected";
        /// Memory stats. Args: total, avg_confidence
        pub const STATS: &str = "memory-stats";
    }

    /// Indexing system message keys.
    pub mod indexing {
        // Error keys (8)
        /// Parse failed. Args: file, reason
        pub const PARSE_FAILED: &str = "error-indexing-parse-failed";
        /// File read failed. Args: file, reason
        pub const FILE_READ_FAILED: &str = "error-indexing-file-read-failed";
        /// Unsupported language. Args: extension
        pub const UNSUPPORTED_LANGUAGE: &str = "error-indexing-unsupported-language";
        /// Extraction failed. Args: file, reason
        pub const EXTRACTION_FAILED: &str = "error-indexing-extraction-failed";
        /// Graph cycle detected. Args: files
        pub const GRAPH_CYCLE_DETECTED: &str = "error-indexing-graph-cycle-detected";
        /// Fingerprint computation failed. Args: file, reason
        pub const FINGERPRINT_FAILED: &str = "error-indexing-fingerprint-failed";
        /// Index build failed. Args: reason
        pub const BUILD_FAILED: &str = "error-indexing-build-failed";
        /// Incremental update failed. Args: reason
        pub const UPDATE_FAILED: &str = "error-indexing-update-failed";

        // Info keys (8)
        /// File indexed. Args: file, language
        pub const FILE_INDEXED: &str = "indexing-file-indexed";
        /// Symbols extracted. Args: count, file
        pub const SYMBOLS_EXTRACTED: &str = "indexing-symbols-extracted";
        /// Graph built. Args: files, edges
        pub const GRAPH_BUILT: &str = "indexing-graph-built";
        /// Ranking computed. Args: symbols
        pub const RANKING_COMPUTED: &str = "indexing-ranking-computed";
        /// Repo map generated. Args: symbols, tokens
        pub const REPOMAP_GENERATED: &str = "indexing-repomap-generated";
        /// Index built. Args: files, symbols
        pub const INDEX_BUILT: &str = "indexing-index-built";
        /// Incremental update completed. Args: added, modified, removed
        pub const INCREMENTAL_UPDATE: &str = "indexing-incremental-update";
        /// Language registered. Args: language
        pub const LANGUAGE_REGISTERED: &str = "indexing-language-registered";
    }

    /// MCP server message keys.
    pub mod mcp {
        // Error keys (10)
        /// JSON-RPC parse error. Args: reason
        pub const PARSE_FAILED: &str = "error-mcp-parse-failed";
        /// Invalid JSON-RPC request. Args: reason
        pub const INVALID_REQUEST: &str = "error-mcp-invalid-request";
        /// Method not found. Args: method
        pub const METHOD_NOT_FOUND: &str = "error-mcp-method-not-found";
        /// Invalid method parameters. Args: reason
        pub const INVALID_PARAMS: &str = "error-mcp-invalid-params";
        /// Internal server error. Args: reason
        pub const INTERNAL_ERROR: &str = "error-mcp-internal-error";
        /// Server not initialized.
        pub const NOT_INITIALIZED: &str = "error-mcp-not-initialized";
        /// Tool not found. Args: tool
        pub const TOOL_NOT_FOUND: &str = "error-mcp-tool-not-found";
        /// Tool execution failed. Args: tool, reason
        pub const TOOL_EXECUTION_FAILED: &str = "error-mcp-tool-execution-failed";
        /// Transport error. Args: reason
        pub const TRANSPORT_ERROR: &str = "error-mcp-transport-error";
        /// Shutdown failed. Args: reason
        pub const SHUTDOWN_FAILED: &str = "error-mcp-shutdown-failed";

        // Info keys (8)
        /// MCP server started. Args: transport
        pub const SERVER_STARTED: &str = "mcp-server-started";
        /// MCP server stopped.
        pub const SERVER_STOPPED: &str = "mcp-server-stopped";
        /// Client initialized. Args: client_name
        pub const CLIENT_INITIALIZED: &str = "mcp-client-initialized";
        /// Tool called. Args: tool
        pub const TOOL_CALLED: &str = "mcp-tool-called";
        /// Tool completed. Args: tool, duration
        pub const TOOL_COMPLETED: &str = "mcp-tool-completed";
        /// Request received. Args: method
        pub const REQUEST_RECEIVED: &str = "mcp-request-received";
        /// Response sent. Args: method
        pub const RESPONSE_SENT: &str = "mcp-response-sent";
        /// Transport ready. Args: transport
        pub const TRANSPORT_READY: &str = "mcp-transport-ready";
    }

    /// Knowledge graph message keys.
    pub mod graph {
        // Error keys (7)
        /// Entity not found. Args: id
        pub const ENTITY_NOT_FOUND: &str = "error-graph-entity-not-found";
        /// Failed to add relationship. Args: reason
        pub const RELATIONSHIP_FAILED: &str = "error-graph-relationship-failed";
        /// Graph build failed. Args: reason
        pub const BUILD_FAILED: &str = "error-graph-build-failed";
        /// Graph update failed. Args: reason
        pub const UPDATE_FAILED: &str = "error-graph-update-failed";
        /// Graph load failed. Args: path, reason
        pub const LOAD_FAILED: &str = "error-graph-load-failed";
        /// Graph save failed. Args: path, reason
        pub const SAVE_FAILED: &str = "error-graph-save-failed";
        /// Max entities exceeded. Args: count, max
        pub const MAX_ENTITIES_EXCEEDED: &str = "error-graph-max-entities-exceeded";

        // Info keys (7)
        /// Graph built. Args: entities, relationships
        pub const GRAPH_BUILT: &str = "graph-built";
        /// Graph updated. Args: added, removed
        pub const GRAPH_UPDATED: &str = "graph-updated";
        /// Entity added. Args: name, kind
        pub const ENTITY_ADDED: &str = "graph-entity-added";
        /// Entity removed. Args: name
        pub const ENTITY_REMOVED: &str = "graph-entity-removed";
        /// Graph persisted. Args: path
        pub const GRAPH_PERSISTED: &str = "graph-persisted";
        /// Graph loaded. Args: path, entities
        pub const GRAPH_LOADED: &str = "graph-loaded";
        /// Query executed. Args: ms, results
        pub const QUERY_EXECUTED: &str = "graph-query-executed";
    }

    /// Context preparation message keys.
    pub mod context {
        // Error keys (4)
        /// Context token budget exceeded. Args: used, budget
        pub const BUDGET_EXCEEDED: &str = "error-context-budget-exceeded";
        /// Budget allocations must sum to <= 1.0. Args: sum
        pub const INVALID_ALLOCATIONS: &str = "error-context-invalid-allocations";
        /// Context build failed. Args: task_id, reason
        pub const BUILD_FAILED: &str = "error-context-build-failed";
        /// Invalid context format. Args: format
        pub const INVALID_FORMAT: &str = "error-context-invalid-format";

        // Info keys (6)
        /// Token budget allocated. Args: total, repo_map, files, memory, task
        pub const BUDGET_ALLOCATED: &str = "context-budget-allocated";
        /// Files scored for relevance. Args: count, top_file
        pub const FILES_SCORED: &str = "context-files-scored";
        /// Code chunks created. Args: count, tokens
        pub const CHUNKS_CREATED: &str = "context-chunks-created";
        /// Context assembled. Args: sections, tokens, budget
        pub const ASSEMBLED: &str = "context-assembled";
        /// Context injected for worker. Args: worker_id, tokens, files
        pub const INJECTED: &str = "context-injected";
        /// Context preparation skipped. Args: reason
        pub const SKIPPED: &str = "context-skipped";
    }

    /// Licensing message keys.
    pub mod licensing {
        // Error keys (12)
        /// License file not found. Args: path
        pub const FILE_NOT_FOUND: &str = "error-license-file-not-found";
        /// Invalid license file. Args: path, reason
        pub const FILE_INVALID: &str = "error-license-file-invalid";
        /// Failed to write license file. Args: path, reason
        pub const FILE_WRITE_FAILED: &str = "error-license-file-write-failed";
        /// License signature verification failed.
        pub const SIGNATURE_INVALID: &str = "error-license-signature-invalid";
        /// Failed to decode license signature. Args: reason
        pub const SIGNATURE_DECODE_FAILED: &str = "error-license-signature-decode-failed";
        /// Failed to serialize license data. Args: reason
        pub const SERIALIZATION_FAILED: &str = "error-license-serialization-failed";
        /// Failed to sign license. Args: reason
        pub const SIGNING_FAILED: &str = "error-license-signing-failed";
        /// Feature denied for tier. Args: feature, tier
        pub const FEATURE_DENIED: &str = "error-license-feature-denied";
        /// Worker limit exceeded. Args: requested, limit, tier
        pub const WORKER_LIMIT: &str = "error-license-worker-limit";
        /// Manager limit exceeded. Args: requested, limit, tier
        pub const MANAGER_LIMIT: &str = "error-license-manager-limit";
        /// Category denied for tier. Args: category, tier, required_tier
        pub const CATEGORY_DENIED: &str = "error-license-category-denied";
        /// Fingerprint hostname error. Args: reason
        pub const FINGERPRINT_HOSTNAME: &str = "error-license-fingerprint-hostname";
        /// Fingerprint username error. Args: reason
        pub const FINGERPRINT_USERNAME: &str = "error-license-fingerprint-username";

        // Info/status keys (10)
        /// Trial license active. Args: days
        pub const TRIAL_ACTIVE: &str = "license-trial-active";
        /// Trial expiring soon. Args: days
        pub const TRIAL_EXPIRING_SOON: &str = "license-trial-expiring-soon";
        /// Trial expiring very soon. Args: days
        pub const TRIAL_EXPIRING_VERY_SOON: &str = "license-trial-expiring-very-soon";
        /// Trial license expired.
        pub const TRIAL_EXPIRED: &str = "license-trial-expired";
        /// License expires later. Args: days
        pub const EXPIRES_LATER: &str = "license-expires-later";
        /// License expiring soon. Args: days
        pub const EXPIRES_SOON: &str = "license-expires-soon";
        /// License expiring very soon. Args: days
        pub const EXPIRES_VERY_SOON: &str = "license-expires-very-soon";
        /// License expires today.
        pub const EXPIRES_TODAY: &str = "license-expires-today";
        /// License in grace period. Args: days
        pub const GRACE_PERIOD: &str = "license-grace-period";
        /// License expired.
        pub const EXPIRED: &str = "license-expired";
    }
}

#[cfg(test)]
mod tests {
    use super::keys;

    #[test]
    fn error_keys_follow_fluent_convention() {
        assert!(keys::error::CONFIG_FILE_NOT_FOUND.starts_with("error-"));
        assert!(keys::error::CONFIG_PARSE_FAILED.starts_with("error-"));
        assert!(keys::error::CONFIG_VALIDATION_FAILED.starts_with("error-"));
        assert!(keys::error::LOCALE_NOT_SUPPORTED.starts_with("error-"));
        assert!(keys::error::LOG_INIT_FAILED.starts_with("error-"));
        assert!(keys::error::LOG_DIR_CREATE_FAILED.starts_with("error-"));
        assert!(keys::error::APP_LAUNCH_FAILED.starts_with("error-"));
    }

    #[test]
    fn config_keys_follow_fluent_convention() {
        assert!(keys::config::LOADED.starts_with("config-"));
        assert!(keys::config::USING_DEFAULTS.starts_with("config-"));
        assert!(keys::config::ENV_OVERRIDE.starts_with("config-"));
        assert!(keys::config::ENV_OVERRIDE_INVALID.starts_with("config-"));
        assert!(keys::config::GENERATED_SUCCESSFULLY.starts_with("config-"));
        assert!(keys::config::ALREADY_EXISTS.starts_with("config-"));
    }

    #[test]
    fn logging_keys_follow_fluent_convention() {
        assert!(keys::logging::INITIALIZED.starts_with("logging-"));
        assert!(keys::logging::RUST_LOG_OVERRIDE.starts_with("logging-"));
        assert!(keys::logging::FILE_PATH.starts_with("logging-"));
        assert!(keys::logging::DIR_CREATE_FAILED_FALLBACK.starts_with("logging-"));
    }

    #[test]
    fn pty_keys_follow_fluent_convention() {
        let pty_keys = [
            keys::pty::OPEN_FAILED,
            keys::pty::SPAWN_FAILED,
            keys::pty::WRITE_FAILED,
            keys::pty::READ_FAILED,
            keys::pty::RESIZE_FAILED,
            keys::pty::KILL_FAILED,
            keys::pty::WAIT_FAILED,
            keys::pty::TERMINAL_NOT_FOUND,
            keys::pty::MAX_CONCURRENT_REACHED,
            keys::pty::PROCESS_SPAWNED,
            keys::pty::PROCESS_EXITED,
            keys::pty::SESSION_CLOSED,
            keys::pty::RESIZED,
            keys::pty::READER_EOF,
            keys::pty::READER_ERROR,
        ];
        for key in &pty_keys {
            assert!(
                key.starts_with("error-pty-") || key.starts_with("pty-"),
                "Key '{}' does not follow pty convention",
                key
            );
        }
    }

    #[test]
    fn provider_keys_follow_fluent_convention() {
        let provider_keys = [
            keys::provider::CREDENTIALS_MISSING,
            keys::provider::CREDENTIALS_INVALID,
            keys::provider::NOT_FOUND,
            keys::provider::NO_DEFAULT,
            keys::provider::COMMAND_BUILD_FAILED,
            keys::provider::REGISTERED,
            keys::provider::SET_DEFAULT,
            keys::provider::CREDENTIALS_VALID,
            keys::provider::VALIDATING,
            keys::provider::LIST_HEADER,
        ];
        for key in &provider_keys {
            assert!(
                key.starts_with("error-provider-") || key.starts_with("provider-"),
                "Key '{}' does not follow provider convention",
                key
            );
        }
    }

    #[test]
    fn credential_keys_follow_fluent_convention() {
        let credential_keys = [
            keys::credential::ANTHROPIC_API_KEY,
            keys::credential::GENERIC_API_KEY,
            keys::credential::AWS_IAM_ROLE,
            keys::credential::OAUTH_TOKEN,
        ];
        for key in &credential_keys {
            assert!(
                key.starts_with("credential-"),
                "Key '{}' does not follow credential convention",
                key
            );
        }
    }

    #[test]
    fn orchestration_keys_follow_fluent_convention() {
        let orchestration_keys = [
            keys::orchestration::INVALID_TRANSITION,
            keys::orchestration::CIRCULAR_DEPENDENCY,
            keys::orchestration::WORKER_NOT_FOUND,
            keys::orchestration::MAX_RETRIES_EXCEEDED,
            keys::orchestration::NO_PROPOSALS,
            keys::orchestration::PLAN_VALIDATION_FAILED,
            keys::orchestration::CONSENSUS_FAILED,
            keys::orchestration::QUEUE_FULL,
            keys::orchestration::MANAGER_TIMEOUT,
            keys::orchestration::WORKER_TIMEOUT,
            keys::orchestration::CYCLE_STARTED,
            keys::orchestration::STATE_CHANGED,
            keys::orchestration::MANAGER_COMPLETED,
            keys::orchestration::CONSENSUS_REACHED,
            keys::orchestration::PLAN_CREATED,
            keys::orchestration::WORKER_STARTED,
            keys::orchestration::WORKER_COMPLETED,
            keys::orchestration::WORKER_FAILED,
            keys::orchestration::CYCLE_COMPLETED,
            keys::orchestration::WORKER_RETRYING,
            keys::orchestration::QUORUM_NOT_MET,
            keys::orchestration::FILE_CONFLICT_SERIALIZED,
            keys::orchestration::BUILD_CHECK_FAILED,
            keys::orchestration::BUILD_CHECK_PASSED,
        ];
        for key in &orchestration_keys {
            assert!(
                key.starts_with("error-orchestration-") || key.starts_with("orchestration-"),
                "Key '{}' does not follow orchestration convention",
                key
            );
        }
    }

    #[test]
    fn detection_keys_follow_fluent_convention() {
        let detection_keys = [
            keys::detection::RULE_COMPILE_FAILED,
            keys::detection::NO_RULES_LOADED,
            keys::detection::INVALID_PATTERN,
            keys::detection::RULE_MATCHED,
            keys::detection::RULE_COOLDOWN,
            keys::detection::RULES_LOADED,
            keys::detection::PROVIDER_RULES_APPLIED,
            keys::detection::ERROR_PATTERN_DETECTED,
            keys::detection::ROOT_CAUSE_HYPOTHESIS,
            keys::detection::RECURRING_ERROR,
            keys::detection::ACTION_EXECUTED,
            keys::detection::COOLDOWNS_RESET,
        ];
        for key in &detection_keys {
            assert!(
                key.starts_with("error-detection-") || key.starts_with("detection-"),
                "Key '{}' does not follow detection convention",
                key
            );
        }
    }

    #[test]
    fn app_keys_follow_fluent_convention() {
        assert!(keys::app::STARTING.starts_with("app-"));
        assert!(keys::app::READY.starts_with("app-"));
        assert!(keys::app::SHUTTING_DOWN.starts_with("app-"));
    }

    #[test]
    fn validation_keys_follow_fluent_convention() {
        assert!(keys::validation::INVALID_LOG_LEVEL.starts_with("validation-"));
        assert!(keys::validation::WORKERS_OUT_OF_RANGE.starts_with("validation-"));
        assert!(keys::validation::MANAGER_TIMEOUT_TOO_LOW.starts_with("validation-"));
        assert!(keys::validation::WORKER_TIMEOUT_TOO_LOW.starts_with("validation-"));
        assert!(keys::validation::OFFLINE_GRACE_TOO_LOW.starts_with("validation-"));
        assert!(keys::validation::INVALID_CONSENSUS_STRATEGY.starts_with("validation-"));
        assert!(keys::validation::INVALID_DATA_DIRECTORY.starts_with("validation-"));
        assert!(keys::validation::EMPTY_DEFAULT_PROVIDER.starts_with("validation-"));
        assert!(keys::validation::INVALID_THEME.starts_with("validation-"));
        assert!(keys::validation::FONT_SIZE_OUT_OF_RANGE.starts_with("validation-"));
        assert!(keys::validation::INVALID_MAX_BATCH_RETRIES.starts_with("validation-"));
        assert!(keys::validation::INVALID_MAX_TOTAL_BATCHES.starts_with("validation-"));
        assert!(
            keys::validation::INVALID_STALL_DETECTION_THRESHOLD.starts_with("validation-")
        );
    }

    #[test]
    fn templates_keys_follow_fluent_convention() {
        let templates_keys = [
            keys::templates::NOT_FOUND,
            keys::templates::CATEGORY_NOT_FOUND,
            keys::templates::COMPOSITION_FAILED,
            keys::templates::INCLUDE_NOT_FOUND,
            keys::templates::CIRCULAR_INCLUDE,
            keys::templates::PLACEHOLDER_UNRESOLVED,
            keys::templates::VALIDATION_FAILED,
            keys::templates::ACTIVATION_FAILED,
            keys::templates::CATALOG_LOAD_FAILED,
            keys::templates::INVALID_VERSION,
            keys::templates::CATALOG_LOADED,
            keys::templates::ACTIVATED,
            keys::templates::COMPOSITION_COMPLETE,
            keys::templates::PLACEHOLDERS_RESOLVED,
            keys::templates::VALIDATION_PASSED,
            keys::templates::VALIDATION_WARNINGS,
            keys::templates::INCLUDE_RESOLVED,
            keys::templates::PROVIDER_INSTRUCTIONS_INJECTED,
            keys::templates::PROJECT_CONFIG_LOADED,
            keys::templates::DOMAIN_VALIDATOR_REGISTERED,
        ];
        for key in &templates_keys {
            assert!(
                key.starts_with("error-template-") || key.starts_with("template-"),
                "Key '{}' does not follow template convention",
                key
            );
        }
    }

    #[test]
    fn all_keys_use_kebab_case() {
        let all_keys = [
            keys::error::CONFIG_FILE_NOT_FOUND,
            keys::error::CONFIG_PARSE_FAILED,
            keys::error::CONFIG_VALIDATION_FAILED,
            keys::error::LOCALE_NOT_SUPPORTED,
            keys::error::LOG_INIT_FAILED,
            keys::error::LOG_DIR_CREATE_FAILED,
            keys::error::APP_LAUNCH_FAILED,
            keys::config::LOADED,
            keys::config::USING_DEFAULTS,
            keys::config::ENV_OVERRIDE,
            keys::config::ENV_OVERRIDE_INVALID,
            keys::config::GENERATED_SUCCESSFULLY,
            keys::config::ALREADY_EXISTS,
            keys::validation::INVALID_LOG_LEVEL,
            keys::validation::WORKERS_OUT_OF_RANGE,
            keys::validation::MANAGER_TIMEOUT_TOO_LOW,
            keys::validation::WORKER_TIMEOUT_TOO_LOW,
            keys::validation::OFFLINE_GRACE_TOO_LOW,
            keys::validation::INVALID_CONSENSUS_STRATEGY,
            keys::validation::INVALID_DATA_DIRECTORY,
            keys::validation::EMPTY_DEFAULT_PROVIDER,
            keys::validation::INVALID_THEME,
            keys::validation::FONT_SIZE_OUT_OF_RANGE,
            keys::validation::INVALID_MAX_BATCH_RETRIES,
            keys::validation::INVALID_MAX_TOTAL_BATCHES,
            keys::validation::INVALID_STALL_DETECTION_THRESHOLD,
            keys::logging::INITIALIZED,
            keys::logging::RUST_LOG_OVERRIDE,
            keys::logging::FILE_PATH,
            keys::logging::DIR_CREATE_FAILED_FALLBACK,
            keys::pty::OPEN_FAILED,
            keys::pty::SPAWN_FAILED,
            keys::pty::WRITE_FAILED,
            keys::pty::READ_FAILED,
            keys::pty::RESIZE_FAILED,
            keys::pty::KILL_FAILED,
            keys::pty::WAIT_FAILED,
            keys::pty::TERMINAL_NOT_FOUND,
            keys::pty::MAX_CONCURRENT_REACHED,
            keys::pty::PROCESS_SPAWNED,
            keys::pty::PROCESS_EXITED,
            keys::pty::SESSION_CLOSED,
            keys::pty::RESIZED,
            keys::pty::READER_EOF,
            keys::pty::READER_ERROR,
            keys::app::STARTING,
            keys::app::READY,
            keys::app::SHUTTING_DOWN,
            // Provider keys
            keys::provider::CREDENTIALS_MISSING,
            keys::provider::CREDENTIALS_INVALID,
            keys::provider::NOT_FOUND,
            keys::provider::NO_DEFAULT,
            keys::provider::COMMAND_BUILD_FAILED,
            keys::provider::REGISTERED,
            keys::provider::SET_DEFAULT,
            keys::provider::CREDENTIALS_VALID,
            keys::provider::VALIDATING,
            keys::provider::LIST_HEADER,
            // Credential keys
            keys::credential::ANTHROPIC_API_KEY,
            keys::credential::GENERIC_API_KEY,
            keys::credential::AWS_IAM_ROLE,
            keys::credential::OAUTH_TOKEN,
            // Orchestration keys
            keys::orchestration::INVALID_TRANSITION,
            keys::orchestration::CIRCULAR_DEPENDENCY,
            keys::orchestration::WORKER_NOT_FOUND,
            keys::orchestration::MAX_RETRIES_EXCEEDED,
            keys::orchestration::NO_PROPOSALS,
            keys::orchestration::PLAN_VALIDATION_FAILED,
            keys::orchestration::CONSENSUS_FAILED,
            keys::orchestration::QUEUE_FULL,
            keys::orchestration::MANAGER_TIMEOUT,
            keys::orchestration::WORKER_TIMEOUT,
            keys::orchestration::CYCLE_STARTED,
            keys::orchestration::STATE_CHANGED,
            keys::orchestration::MANAGER_COMPLETED,
            keys::orchestration::CONSENSUS_REACHED,
            keys::orchestration::PLAN_CREATED,
            keys::orchestration::WORKER_STARTED,
            keys::orchestration::WORKER_COMPLETED,
            keys::orchestration::WORKER_FAILED,
            keys::orchestration::CYCLE_COMPLETED,
            keys::orchestration::WORKER_RETRYING,
            keys::orchestration::QUORUM_NOT_MET,
            keys::orchestration::FILE_CONFLICT_SERIALIZED,
            keys::orchestration::BUILD_CHECK_FAILED,
            keys::orchestration::BUILD_CHECK_PASSED,
            // Detection keys
            keys::detection::RULE_COMPILE_FAILED,
            keys::detection::NO_RULES_LOADED,
            keys::detection::INVALID_PATTERN,
            keys::detection::RULE_MATCHED,
            keys::detection::RULE_COOLDOWN,
            keys::detection::RULES_LOADED,
            keys::detection::PROVIDER_RULES_APPLIED,
            keys::detection::ERROR_PATTERN_DETECTED,
            keys::detection::ROOT_CAUSE_HYPOTHESIS,
            keys::detection::RECURRING_ERROR,
            keys::detection::ACTION_EXECUTED,
            keys::detection::COOLDOWNS_RESET,
            // Template keys
            keys::templates::NOT_FOUND,
            keys::templates::CATEGORY_NOT_FOUND,
            keys::templates::COMPOSITION_FAILED,
            keys::templates::INCLUDE_NOT_FOUND,
            keys::templates::CIRCULAR_INCLUDE,
            keys::templates::PLACEHOLDER_UNRESOLVED,
            keys::templates::VALIDATION_FAILED,
            keys::templates::ACTIVATION_FAILED,
            keys::templates::CATALOG_LOAD_FAILED,
            keys::templates::INVALID_VERSION,
            keys::templates::CATALOG_LOADED,
            keys::templates::ACTIVATED,
            keys::templates::COMPOSITION_COMPLETE,
            keys::templates::PLACEHOLDERS_RESOLVED,
            keys::templates::VALIDATION_PASSED,
            keys::templates::VALIDATION_WARNINGS,
            keys::templates::INCLUDE_RESOLVED,
            keys::templates::PROVIDER_INSTRUCTIONS_INJECTED,
            keys::templates::PROJECT_CONFIG_LOADED,
            keys::templates::DOMAIN_VALIDATOR_REGISTERED,
            // Platform keys
            keys::platform::UNSUPPORTED,
            keys::platform::WSL_NOT_DETECTED,
            keys::platform::WSL_PATH_INVALID,
            keys::platform::LAYOUT_NO_SPACE,
            keys::platform::LAYOUT_INVALID_COUNT,
            keys::platform::HOTKEY_REGISTRATION_FAILED,
            keys::platform::HOTKEY_PARSE_FAILED,
            keys::platform::SHUTDOWN_TIMEOUT,
            keys::platform::SHUTDOWN_FAILED,
            keys::platform::SHELL_NOT_FOUND,
            keys::platform::DETECTED,
            keys::platform::WSL_DETECTED,
            keys::platform::WSL_PATH_TRANSLATED,
            keys::platform::LAYOUT_CALCULATED,
            keys::platform::LAYOUT_OPTIMIZED,
            keys::platform::HOTKEY_REGISTERED,
            keys::platform::HOTKEY_UNREGISTERED,
            keys::platform::SHUTDOWN_INITIATED,
            keys::platform::SHUTDOWN_COMPLETED,
            keys::platform::SHELL_DETECTED,
            // Licensing keys
            keys::licensing::FILE_NOT_FOUND,
            keys::licensing::FILE_INVALID,
            keys::licensing::FILE_WRITE_FAILED,
            keys::licensing::SIGNATURE_INVALID,
            keys::licensing::SIGNATURE_DECODE_FAILED,
            keys::licensing::SERIALIZATION_FAILED,
            keys::licensing::SIGNING_FAILED,
            keys::licensing::FEATURE_DENIED,
            keys::licensing::WORKER_LIMIT,
            keys::licensing::CATEGORY_DENIED,
            keys::licensing::FINGERPRINT_HOSTNAME,
            keys::licensing::FINGERPRINT_USERNAME,
            keys::licensing::TRIAL_ACTIVE,
            keys::licensing::TRIAL_EXPIRING_SOON,
            keys::licensing::TRIAL_EXPIRING_VERY_SOON,
            keys::licensing::TRIAL_EXPIRED,
            keys::licensing::EXPIRES_LATER,
            keys::licensing::EXPIRES_SOON,
            keys::licensing::EXPIRES_VERY_SOON,
            keys::licensing::EXPIRES_TODAY,
            keys::licensing::GRACE_PERIOD,
            keys::licensing::EXPIRED,
            // Indexing keys
            keys::indexing::PARSE_FAILED,
            keys::indexing::FILE_READ_FAILED,
            keys::indexing::UNSUPPORTED_LANGUAGE,
            keys::indexing::EXTRACTION_FAILED,
            keys::indexing::GRAPH_CYCLE_DETECTED,
            keys::indexing::FINGERPRINT_FAILED,
            keys::indexing::BUILD_FAILED,
            keys::indexing::UPDATE_FAILED,
            keys::indexing::FILE_INDEXED,
            keys::indexing::SYMBOLS_EXTRACTED,
            keys::indexing::GRAPH_BUILT,
            keys::indexing::RANKING_COMPUTED,
            keys::indexing::REPOMAP_GENERATED,
            keys::indexing::INDEX_BUILT,
            keys::indexing::INCREMENTAL_UPDATE,
            keys::indexing::LANGUAGE_REGISTERED,
            // Memory keys
            keys::memory::NOT_FOUND,
            keys::memory::DUPLICATE,
            keys::memory::PERSISTENCE_FAILED,
            keys::memory::LOAD_FAILED,
            keys::memory::INVALID_CONFIDENCE,
            keys::memory::STORE_FULL,
            keys::memory::INVALID_QUERY,
            keys::memory::SERIALIZATION,
            keys::memory::INVALID_ENTRY,
            keys::memory::SESSION_MISMATCH,
            keys::memory::STORE_CREATED,
            keys::memory::ENTRY_ADDED,
            keys::memory::ENTRY_UPDATED,
            keys::memory::ENTRY_REMOVED,
            keys::memory::STORE_CLEARED,
            keys::memory::PERSISTED,
            keys::memory::LOADED,
            keys::memory::QUERY_EXECUTED,
            keys::memory::INJECTED,
            keys::memory::STATS,
            // Context keys
            keys::context::BUDGET_EXCEEDED,
            keys::context::INVALID_ALLOCATIONS,
            keys::context::BUILD_FAILED,
            keys::context::INVALID_FORMAT,
            keys::context::BUDGET_ALLOCATED,
            keys::context::FILES_SCORED,
            keys::context::CHUNKS_CREATED,
            keys::context::ASSEMBLED,
            keys::context::INJECTED,
            keys::context::SKIPPED,
            // MCP keys
            keys::mcp::PARSE_FAILED,
            keys::mcp::INVALID_REQUEST,
            keys::mcp::METHOD_NOT_FOUND,
            keys::mcp::INVALID_PARAMS,
            keys::mcp::INTERNAL_ERROR,
            keys::mcp::NOT_INITIALIZED,
            keys::mcp::TOOL_NOT_FOUND,
            keys::mcp::TOOL_EXECUTION_FAILED,
            keys::mcp::TRANSPORT_ERROR,
            keys::mcp::SHUTDOWN_FAILED,
            keys::mcp::SERVER_STARTED,
            keys::mcp::SERVER_STOPPED,
            keys::mcp::CLIENT_INITIALIZED,
            keys::mcp::TOOL_CALLED,
            keys::mcp::TOOL_COMPLETED,
            keys::mcp::REQUEST_RECEIVED,
            keys::mcp::RESPONSE_SENT,
            keys::mcp::TRANSPORT_READY,
            // Graph keys
            keys::graph::ENTITY_NOT_FOUND,
            keys::graph::RELATIONSHIP_FAILED,
            keys::graph::BUILD_FAILED,
            keys::graph::UPDATE_FAILED,
            keys::graph::LOAD_FAILED,
            keys::graph::SAVE_FAILED,
            keys::graph::MAX_ENTITIES_EXCEEDED,
            keys::graph::GRAPH_BUILT,
            keys::graph::GRAPH_UPDATED,
            keys::graph::ENTITY_ADDED,
            keys::graph::ENTITY_REMOVED,
            keys::graph::GRAPH_PERSISTED,
            keys::graph::GRAPH_LOADED,
            keys::graph::QUERY_EXECUTED,
        ];
        for key in &all_keys {
            assert!(
                key.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "Key '{}' is not kebab-case",
                key
            );
        }
    }

    #[test]
    fn platform_keys_follow_fluent_convention() {
        let platform_keys = [
            keys::platform::UNSUPPORTED,
            keys::platform::WSL_NOT_DETECTED,
            keys::platform::WSL_PATH_INVALID,
            keys::platform::LAYOUT_NO_SPACE,
            keys::platform::LAYOUT_INVALID_COUNT,
            keys::platform::HOTKEY_REGISTRATION_FAILED,
            keys::platform::HOTKEY_PARSE_FAILED,
            keys::platform::SHUTDOWN_TIMEOUT,
            keys::platform::SHUTDOWN_FAILED,
            keys::platform::SHELL_NOT_FOUND,
            keys::platform::DETECTED,
            keys::platform::WSL_DETECTED,
            keys::platform::WSL_PATH_TRANSLATED,
            keys::platform::LAYOUT_CALCULATED,
            keys::platform::LAYOUT_OPTIMIZED,
            keys::platform::HOTKEY_REGISTERED,
            keys::platform::HOTKEY_UNREGISTERED,
            keys::platform::SHUTDOWN_INITIATED,
            keys::platform::SHUTDOWN_COMPLETED,
            keys::platform::SHELL_DETECTED,
        ];
        for key in &platform_keys {
            assert!(
                key.starts_with("error-platform-") || key.starts_with("platform-"),
                "Key '{}' does not follow platform convention",
                key
            );
        }
    }

    #[test]
    fn memory_keys_follow_fluent_convention() {
        let memory_keys = [
            keys::memory::NOT_FOUND,
            keys::memory::DUPLICATE,
            keys::memory::PERSISTENCE_FAILED,
            keys::memory::LOAD_FAILED,
            keys::memory::INVALID_CONFIDENCE,
            keys::memory::STORE_FULL,
            keys::memory::INVALID_QUERY,
            keys::memory::SERIALIZATION,
            keys::memory::INVALID_ENTRY,
            keys::memory::SESSION_MISMATCH,
            keys::memory::STORE_CREATED,
            keys::memory::ENTRY_ADDED,
            keys::memory::ENTRY_UPDATED,
            keys::memory::ENTRY_REMOVED,
            keys::memory::STORE_CLEARED,
            keys::memory::PERSISTED,
            keys::memory::LOADED,
            keys::memory::QUERY_EXECUTED,
            keys::memory::INJECTED,
            keys::memory::STATS,
        ];
        for key in &memory_keys {
            assert!(
                key.starts_with("error-memory-") || key.starts_with("memory-"),
                "Key '{}' does not follow memory convention",
                key
            );
        }
    }

    #[test]
    fn indexing_keys_follow_fluent_convention() {
        let indexing_keys = [
            keys::indexing::PARSE_FAILED,
            keys::indexing::FILE_READ_FAILED,
            keys::indexing::UNSUPPORTED_LANGUAGE,
            keys::indexing::EXTRACTION_FAILED,
            keys::indexing::GRAPH_CYCLE_DETECTED,
            keys::indexing::FINGERPRINT_FAILED,
            keys::indexing::BUILD_FAILED,
            keys::indexing::UPDATE_FAILED,
            keys::indexing::FILE_INDEXED,
            keys::indexing::SYMBOLS_EXTRACTED,
            keys::indexing::GRAPH_BUILT,
            keys::indexing::RANKING_COMPUTED,
            keys::indexing::REPOMAP_GENERATED,
            keys::indexing::INDEX_BUILT,
            keys::indexing::INCREMENTAL_UPDATE,
            keys::indexing::LANGUAGE_REGISTERED,
        ];
        for key in &indexing_keys {
            assert!(
                key.starts_with("error-indexing-") || key.starts_with("indexing-"),
                "Key '{}' does not follow indexing convention",
                key
            );
        }
    }

    #[test]
    fn mcp_keys_follow_fluent_convention() {
        let mcp_keys = [
            keys::mcp::PARSE_FAILED,
            keys::mcp::INVALID_REQUEST,
            keys::mcp::METHOD_NOT_FOUND,
            keys::mcp::INVALID_PARAMS,
            keys::mcp::INTERNAL_ERROR,
            keys::mcp::NOT_INITIALIZED,
            keys::mcp::TOOL_NOT_FOUND,
            keys::mcp::TOOL_EXECUTION_FAILED,
            keys::mcp::TRANSPORT_ERROR,
            keys::mcp::SHUTDOWN_FAILED,
            keys::mcp::SERVER_STARTED,
            keys::mcp::SERVER_STOPPED,
            keys::mcp::CLIENT_INITIALIZED,
            keys::mcp::TOOL_CALLED,
            keys::mcp::TOOL_COMPLETED,
            keys::mcp::REQUEST_RECEIVED,
            keys::mcp::RESPONSE_SENT,
            keys::mcp::TRANSPORT_READY,
        ];
        for key in &mcp_keys {
            assert!(
                key.starts_with("error-mcp-") || key.starts_with("mcp-"),
                "Key '{}' does not follow mcp convention",
                key
            );
        }
    }

    #[test]
    fn graph_keys_follow_fluent_convention() {
        let graph_keys = [
            keys::graph::ENTITY_NOT_FOUND,
            keys::graph::RELATIONSHIP_FAILED,
            keys::graph::BUILD_FAILED,
            keys::graph::UPDATE_FAILED,
            keys::graph::LOAD_FAILED,
            keys::graph::SAVE_FAILED,
            keys::graph::MAX_ENTITIES_EXCEEDED,
            keys::graph::GRAPH_BUILT,
            keys::graph::GRAPH_UPDATED,
            keys::graph::ENTITY_ADDED,
            keys::graph::ENTITY_REMOVED,
            keys::graph::GRAPH_PERSISTED,
            keys::graph::GRAPH_LOADED,
            keys::graph::QUERY_EXECUTED,
        ];
        for key in &graph_keys {
            assert!(
                key.starts_with("error-graph-") || key.starts_with("graph-"),
                "Key '{}' does not follow graph convention",
                key
            );
        }
    }

    #[test]
    fn context_keys_follow_fluent_convention() {
        let context_keys = [
            keys::context::BUDGET_EXCEEDED,
            keys::context::INVALID_ALLOCATIONS,
            keys::context::BUILD_FAILED,
            keys::context::INVALID_FORMAT,
            keys::context::BUDGET_ALLOCATED,
            keys::context::FILES_SCORED,
            keys::context::CHUNKS_CREATED,
            keys::context::ASSEMBLED,
            keys::context::INJECTED,
            keys::context::SKIPPED,
        ];
        for key in &context_keys {
            assert!(
                key.starts_with("error-context-") || key.starts_with("context-"),
                "Key '{}' does not follow context convention",
                key
            );
        }
    }

    #[test]
    fn licensing_keys_follow_fluent_convention() {
        let licensing_keys = [
            keys::licensing::FILE_NOT_FOUND,
            keys::licensing::FILE_INVALID,
            keys::licensing::FILE_WRITE_FAILED,
            keys::licensing::SIGNATURE_INVALID,
            keys::licensing::SIGNATURE_DECODE_FAILED,
            keys::licensing::SERIALIZATION_FAILED,
            keys::licensing::SIGNING_FAILED,
            keys::licensing::FEATURE_DENIED,
            keys::licensing::WORKER_LIMIT,
            keys::licensing::CATEGORY_DENIED,
            keys::licensing::FINGERPRINT_HOSTNAME,
            keys::licensing::FINGERPRINT_USERNAME,
            keys::licensing::TRIAL_ACTIVE,
            keys::licensing::TRIAL_EXPIRING_SOON,
            keys::licensing::TRIAL_EXPIRING_VERY_SOON,
            keys::licensing::TRIAL_EXPIRED,
            keys::licensing::EXPIRES_LATER,
            keys::licensing::EXPIRES_SOON,
            keys::licensing::EXPIRES_VERY_SOON,
            keys::licensing::EXPIRES_TODAY,
            keys::licensing::GRACE_PERIOD,
            keys::licensing::EXPIRED,
        ];
        for key in &licensing_keys {
            assert!(
                key.starts_with("error-license-") || key.starts_with("license-"),
                "Key '{}' does not follow licensing convention",
                key
            );
        }
    }
}
