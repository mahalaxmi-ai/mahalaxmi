# Errors
error-config-file-not-found = Configuration file not found at { $path }
error-config-parse-failed = Failed to parse configuration: { $reason }
error-config-validation-failed = Configuration validation failed: { $details }
error-locale-not-supported = Locale "{ $locale }" is not supported
error-log-init-failed = Failed to initialize logging: { $reason }
error-log-dir-create-failed = Failed to create log directory at { $path }: { $reason }
error-app-launch-failed = Failed to launch application: { $reason }

# Validation
validation-invalid-log-level = Invalid log level "{ $level }". Valid levels: { $valid }
validation-workers-out-of-range = max_concurrent_workers must be between { $min } and { $max }, got { $value }
validation-manager-timeout-too-low = manager_timeout_seconds must be at least { $min }, got { $value }
validation-worker-timeout-too-low = worker_timeout_seconds must be at least { $min }, got { $value }
validation-offline-grace-too-low = offline_grace_days must be at least { $min }, got { $value }
validation-invalid-consensus-strategy = Invalid consensus_strategy "{ $value }". Valid strategies: { $valid }
validation-invalid-data-directory = data_directory contains invalid characters
validation-empty-default-provider = providers.default_provider must not be empty
validation-invalid-theme = Invalid theme "{ $value }". Valid themes: { $valid }
validation-font-size-out-of-range = terminal_font_size must be between { $min } and { $max }, got { $value }
validation-invalid-max-batch-retries = max_batch_retries must be >= 1, got { $value }
validation-invalid-max-total-batches = max_total_batches must be >= 2, got { $value }
validation-invalid-stall-detection-threshold = stall_detection_threshold must be >= 2, got { $value }

# Config
config-loaded-successfully = Configuration loaded from { $path }
config-using-default = No configuration file found, using defaults
config-env-override = Configuration value overridden by environment variable { $var }
config-env-override-invalid = Environment variable { $var } has invalid value "{ $value }", keeping configured value
config-generated-successfully = Default configuration generated at { $path }
config-already-exists = Configuration file already exists at { $path }

# Logging
logging-initialized = Logging initialized at { $level } level
logging-rust-log-override = RUST_LOG environment variable detected, overriding configured level
logging-file-path = Log file: { $path }
logging-dir-create-failed-fallback = Failed to create log directory { $path }, falling back to console-only logging

# PTY
pty-open-failed = Failed to open pseudo-terminal: { $reason }
pty-spawn-failed = Failed to spawn "{ $program }" in PTY: { $reason }
pty-write-failed = Failed to write to terminal { $terminal_id }: { $reason }
pty-read-failed = Failed to read from terminal { $terminal_id }: { $reason }
pty-resize-failed = Failed to resize terminal { $terminal_id } to { $rows }x{ $cols }: { $reason }
pty-wait-failed = Failed to check process status for terminal { $terminal_id }: { $reason }
pty-kill-failed = Failed to kill process in terminal { $terminal_id }: { $reason }

# App
app-starting = Mahalaxmi v{ $version } starting
app-ready = Mahalaxmi is ready
app-shutting-down = Mahalaxmi shutting down

# Credentials
credential-anthropic-api-key = Anthropic API key for Claude Code
credential-generic-api-key = API key for { $provider }
credential-aws-iam-role = AWS IAM role for { $provider }
credential-oauth-token = OAuth token for { $provider }

# Provider
error-provider-credentials-missing = { $provider } credentials missing: environment variable { $env_var } is not set
error-provider-credentials-invalid = { $provider } credentials are invalid: { $reason }
error-provider-not-found = Provider "{ $provider_id }" not found in registry
error-provider-no-default = No default AI provider configured
error-provider-command-build-failed = Failed to build { $provider } command: { $reason }
provider-registered = Provider "{ $provider }" registered with ID "{ $id }"
provider-set-default = Default provider set to "{ $provider }"
provider-credentials-valid = { $provider } credentials validated successfully
provider-validating = Validating { $provider } credentials
provider-list-header = Registered AI providers

# PTY (extended)
error-pty-open-failed = Failed to open PTY: { $reason }
error-pty-spawn-failed = Failed to spawn "{ $program }" in PTY: { $reason }
error-pty-write-failed = Failed to write to terminal { $terminal_id }: { $reason }
error-pty-read-failed = Failed to read from terminal { $terminal_id }: { $reason }
error-pty-resize-failed = Failed to resize terminal { $terminal_id } to { $rows }x{ $cols }: { $reason }
error-pty-kill-failed = Failed to kill terminal { $terminal_id } process: { $reason }
error-pty-wait-failed = Failed to check terminal { $terminal_id } process status: { $reason }
error-pty-terminal-not-found = Terminal { $terminal_id } not found
error-pty-max-concurrent-reached = Maximum concurrent terminals ({ $max }) reached
pty-process-spawned = PTY process "{ $program }" spawned in terminal { $terminal_id }
pty-process-exited = Terminal { $terminal_id } process exited with code { $exit_code }
pty-session-closed = Terminal session { $terminal_id } closed
pty-resized = Terminal { $terminal_id } resized to { $rows }x{ $cols }
pty-reader-eof = Terminal { $terminal_id } reader reached end of stream
pty-reader-error = Terminal { $terminal_id } reader error: { $reason }

# Orchestration errors
error-orchestration-invalid-transition = Invalid state transition from { $from } to { $to }
error-orchestration-circular-dependency = Circular dependency detected: { $cycle }
error-orchestration-worker-not-found = Worker { $worker_id } not found in queue
error-orchestration-max-retries-exceeded = Worker { $worker_id } exceeded maximum retries ({ $max_retries })
error-orchestration-no-proposals = No manager proposals received
error-orchestration-plan-validation-failed = Execution plan validation failed: { $errors }
error-orchestration-consensus-failed = Consensus engine failed: { $reason }
error-orchestration-queue-full = Worker queue is full (max { $max })
error-orchestration-manager-timeout = Manager { $manager_id } timed out after { $timeout }s
error-orchestration-worker-timeout = Worker { $worker_id } timed out after { $timeout }s

# Orchestration info
orchestration-cycle-started = Orchestration cycle { $cycle_id } started
orchestration-state-changed = State changed: { $from } -> { $to }
orchestration-manager-completed = Manager { $manager_id } completed with { $task_count } tasks
orchestration-consensus-reached = Consensus reached: { $agreed } agreed, { $dissenting } dissenting
orchestration-plan-created = Execution plan created: { $phases } phases, { $workers } workers
orchestration-worker-started = Worker { $worker_id } started: { $task }
orchestration-worker-completed = Worker { $worker_id } completed in { $duration }ms
orchestration-worker-failed = Worker { $worker_id } failed: { $error }
orchestration-cycle-completed = Cycle completed in { $duration }ms (success rate: { $success_rate })
orchestration-worker-retrying = Worker { $worker_id } retrying (attempt { $attempt }/{ $max })

# Detection errors
error-detection-rule-compile-failed = Failed to compile detection rule pattern: { $reason }
error-detection-no-rules-loaded = No detection rules loaded
error-detection-invalid-pattern = Invalid detection pattern "{ $pattern }": { $reason }

# Detection info
detection-rule-matched = Detection rule "{ $rule }" matched, action: { $action }
detection-rule-cooldown = Detection rule "{ $rule }" suppressed by cooldown ({ $remaining_ms }ms remaining)
detection-rules-loaded = { $count } detection rules loaded
detection-provider-rules-applied = { $count } rules applied for provider { $provider }
detection-error-pattern-detected = Error pattern detected: "{ $pattern }" (seen { $count } times)
detection-root-cause-hypothesis = Root cause hypothesis: { $category } (confidence: { $confidence })
detection-recurring-error = Recurring error: "{ $message }" (occurred { $count } times)
detection-action-executed = Action { $action } executed for rule "{ $rule }"
detection-cooldowns-reset = Cooldowns reset for { $rule_count } rules

# Template errors
error-template-not-found = Template { $template_id } not found
error-template-category-not-found = Template category { $category_id } not found
error-template-composition-failed = Template composition failed: { $reason }
error-template-include-not-found = Include file not found: { $path }
error-template-circular-include = Circular include detected (max depth { $depth } exceeded)
error-template-placeholder-unresolved = Unresolved placeholder: ${ $placeholder }
error-template-validation-failed = Template validation failed with { $count } errors
error-template-activation-failed = Template activation failed: { $reason }
error-template-catalog-load-failed = Failed to load template catalog: { $path }
error-template-invalid-version = Invalid template version format: { $version }

# Template info
template-catalog-loaded = Template catalog loaded with { $count } templates
template-activated = Template { $template_id } activated successfully
template-composition-complete = Composition complete: { $included } included, { $overridden } overridden
template-placeholders-resolved = { $count } placeholders resolved
template-validation-passed = Template validation passed for domain { $domain }
template-validation-warnings = Template validation completed with { $count } warnings
template-include-resolved = Include resolved: { $path }
template-provider-instructions-injected = Provider instructions injected for { $provider }
template-project-config-loaded = Project configuration loaded from { $path }
template-domain-validator-registered = Domain validator registered: { $domain }

# Licensing errors
error-license-file-not-found = License file not found at { $path }
error-license-file-invalid = Invalid license file at { $path }: { $reason }
error-license-file-write-failed = Failed to write license file to { $path }: { $reason }
error-license-signature-invalid = License signature verification failed
error-license-signature-decode-failed = Failed to decode license signature: { $reason }
error-license-serialization-failed = Failed to serialize license data: { $reason }
error-license-signing-failed = Failed to sign license: { $reason }
error-license-feature-denied = Feature '{ $feature }' is not available on the { $tier } tier
error-license-worker-limit = Requested { $requested } workers exceeds the { $tier } tier limit of { $limit }
error-license-manager-limit = Requested { $requested } managers exceeds the { $tier } tier limit of { $limit }. Activate your subscription in Settings → License to unlock more.
error-license-category-denied = Category '{ $category }' requires { $required_tier } tier (current: { $tier })
error-license-fingerprint-hostname = Failed to determine hostname: { $reason }
error-license-fingerprint-username = Failed to determine username: { $reason }

# Licensing status
license-trial-active = Trial license active ({ $days } days remaining)
license-trial-expiring-soon = Trial expiring soon ({ $days } days remaining)
license-trial-expiring-very-soon = Trial expiring very soon ({ $days } days remaining)
license-trial-expired = Trial license has expired
license-expires-later = License expires in { $days } days
license-expires-soon = License expiring soon ({ $days } days remaining)
license-expires-very-soon = License expiring very soon ({ $days } days remaining)
license-expires-today = License expires today
license-grace-period = License expired, grace period active ({ $days } days remaining)
license-expired = License has expired

# Platform support — error keys
error-platform-unsupported = Platform not supported: { $platform }
error-platform-wsl-not-detected = WSL environment not detected
error-platform-wsl-path-invalid = Invalid path for WSL translation: { $path }
error-platform-layout-no-space = Container too small for panel layout ({ $width }x{ $height })
error-platform-layout-invalid-count = Invalid panel count: { $count }
error-platform-hotkey-registration-failed = Hotkey registration failed (conflict): { $shortcut }
error-platform-hotkey-parse-failed = Failed to parse hotkey shortcut: { $shortcut }
error-platform-shutdown-timeout = Shutdown timed out for process { $pid } ({ $label })
error-platform-shutdown-failed = Failed to shut down process { $pid } ({ $label })
error-platform-shell-not-found = Default shell not found

# Platform support — info keys
platform-detected = Platform detected: { $os } ({ $arch })
platform-wsl-detected = WSL detected: { $distro } (WSL{ $version })
platform-wsl-path-translated = Path translated: { $from } → { $to }
platform-layout-calculated = Layout calculated: { $panels } panels in { $rows }x{ $cols } grid
platform-layout-optimized = Layout optimized: { $utilization }% utilization
platform-hotkey-registered = Hotkey registered: { $command } → { $shortcut }
platform-hotkey-unregistered = Hotkey unregistered: { $command }
platform-shutdown-initiated = Shutdown initiated for { $count } processes
platform-shutdown-completed = Shutdown completed: { $count } processes in { $duration }ms
platform-shell-detected = Shell detected: { $shell } ({ $path })

# Memory errors
error-memory-not-found = Memory entry not found: { $id }
error-memory-duplicate = Duplicate memory entry: { $id }
error-memory-persistence-failed = Failed to persist memory store: { $reason }
error-memory-load-failed = Failed to load memory store: { $reason }
error-memory-invalid-confidence = Invalid confidence score: { $value } (must be 0.0-1.0)
error-memory-store-full = Memory store is full (maximum { $max } entries)
error-memory-invalid-query = Invalid memory query: { $reason }
error-memory-serialization = Memory serialization failed: { $reason }
error-memory-invalid-entry = Invalid memory entry: { $reason }
error-memory-session-mismatch = Session mismatch: expected { $expected }, got { $actual }

# Memory info
memory-store-created = Memory store created for session { $session_id }
memory-entry-added = Memory entry added: { $title } (type: { $memory_type })
memory-entry-updated = Memory entry updated: { $id }
memory-entry-removed = Memory entry removed: { $id }
memory-store-cleared = Memory store cleared ({ $count } entries removed)
memory-persisted = Memory store persisted to { $path }
memory-loaded = Memory store loaded from { $path } ({ $count } entries)
memory-query-executed = Memory query returned { $count } results
memory-injected = Injected { $count } memories ({ $tokens } tokens)
memory-stats = Memory stats: { $total } entries, avg confidence { $avg_confidence }

# Indexing errors
error-indexing-parse-failed = Failed to parse { $file }: { $reason }
error-indexing-file-read-failed = Failed to read file { $file }: { $reason }
error-indexing-unsupported-language = Unsupported language for file extension: { $extension }
error-indexing-extraction-failed = Symbol extraction failed for { $file }: { $reason }
error-indexing-graph-cycle-detected = Dependency cycle detected: { $files }
error-indexing-fingerprint-failed = Failed to compute fingerprint for { $file }: { $reason }
error-indexing-build-failed = Index build failed: { $reason }
error-indexing-update-failed = Incremental update failed: { $reason }

# Indexing info
indexing-file-indexed = File indexed: { $file } ({ $language })
indexing-symbols-extracted = { $count } symbols extracted from { $file }
indexing-graph-built = Dependency graph built: { $files } files, { $edges } edges
indexing-ranking-computed = Ranking computed for { $symbols } symbols
indexing-repomap-generated = Repo map generated: { $symbols } symbols, { $tokens } tokens
indexing-index-built = Codebase index built: { $files } files, { $symbols } symbols
indexing-incremental-update = Incremental update: { $added } added, { $modified } modified, { $removed } removed
indexing-language-registered = Language registered: { $language }

# Context errors
error-context-budget-exceeded = Context token budget exceeded: used { $used }, budget { $budget }
error-context-invalid-allocations = Budget allocations must sum to <= 1.0, got { $sum }
error-context-build-failed = Context build failed for task { $task_id }: { $reason }
error-context-invalid-format = Invalid context format: { $format }

# Context info
context-budget-allocated = Token budget allocated: { $total } tokens ({ $repo_map } repo map, { $files } files, { $memory } memory, { $task } task)
context-files-scored = Scored { $count } files for relevance (top: { $top_file })
context-chunks-created = Created { $count } code chunks ({ $tokens } tokens)
context-assembled = Context assembled: { $sections } sections, { $tokens } tokens used of { $budget } budget
context-injected = Context injected for worker { $worker_id } ({ $tokens } tokens, { $files } files)
context-skipped = Context preparation skipped: { $reason }

# MCP errors
error-mcp-parse-failed = Failed to parse JSON-RPC message: { $reason }
error-mcp-invalid-request = Invalid JSON-RPC request: { $reason }
error-mcp-method-not-found = Method not found: { $method }
error-mcp-invalid-params = Invalid parameters: { $reason }
error-mcp-internal-error = Internal MCP server error: { $reason }
error-mcp-not-initialized = MCP server has not been initialized
error-mcp-tool-not-found = Tool not found: { $tool }
error-mcp-tool-execution-failed = Tool "{ $tool }" execution failed: { $reason }
error-mcp-transport-error = MCP transport error: { $reason }
error-mcp-shutdown-failed = MCP server shutdown failed: { $reason }

# MCP info
mcp-server-started = MCP server started ({ $transport } transport)
mcp-server-stopped = MCP server stopped
mcp-client-initialized = MCP client initialized: { $client_name }
mcp-tool-called = Tool called: { $tool }
mcp-tool-completed = Tool "{ $tool }" completed in { $duration }ms
mcp-request-received = Request received: { $method }
mcp-response-sent = Response sent: { $method }
mcp-transport-ready = MCP transport ready: { $transport }

# Graph errors
error-graph-entity-not-found = Graph entity not found: { $id }
error-graph-relationship-failed = Failed to add relationship: { $reason }
error-graph-build-failed = Failed to build knowledge graph: { $reason }
error-graph-update-failed = Failed to update knowledge graph: { $reason }
error-graph-load-failed = Failed to load knowledge graph from { $path }: { $reason }
error-graph-save-failed = Failed to save knowledge graph to { $path }: { $reason }
error-graph-max-entities-exceeded = Knowledge graph exceeded maximum entity limit: { $count } / { $max }

# Graph info
graph-built = Knowledge graph built with { $entities } entities and { $relationships } relationships
graph-updated = Knowledge graph updated: { $added } added, { $removed } removed
graph-entity-added = Entity added to knowledge graph: { $name } ({ $kind })
graph-entity-removed = Entity removed from knowledge graph: { $name }
graph-persisted = Knowledge graph persisted to { $path }
graph-loaded = Knowledge graph loaded from { $path } ({ $entities } entities)
graph-query-executed = Graph query executed in { $ms }ms, { $results } results

# Platform API errors
error-platform-api-request-failed = Platform API request failed: { $reason }
error-platform-api-unauthorized = Platform API authentication failed — check channel_api_key
error-platform-api-not-found = Platform resource not found: { $resource }
error-platform-api-rate-limited = Platform API rate limited — retry after { $seconds }s
error-platform-api-server-error = Platform server error ({ $status }): { $message }
error-platform-trial-not-eligible = This device is not eligible for a trial: { $reason }
error-platform-activation-failed = License activation failed: { $reason }
error-platform-validation-failed = License validation failed: { $reason }
error-platform-deactivation-failed = Device deactivation failed: { $reason }
error-platform-cache-read-failed = Failed to read license cache from { $path }: { $reason }
error-platform-cache-write-failed = Failed to write license cache to { $path }: { $reason }
error-platform-cache-decrypt-failed = Failed to decrypt license cache (key mismatch or corruption)
error-platform-not-configured = Platform integration not configured — set platform_base_url in config

# Platform API info
platform-api-trial-activated = Trial activated: { $tier } tier, { $days } days
platform-api-license-activated = License activated: { $tier } tier (activation { $activation_id })
platform-api-license-validated = License validated: { $tier } tier, { $days } days remaining
platform-api-heartbeat-sent = Heartbeat sent (activation { $activation_id })
platform-api-device-deactivated = Device deactivated from license
platform-api-cache-updated = License cache updated at { $path }
platform-api-offline-fallback = Platform unreachable, using cached license (cached { $days_ago } days ago)

# Messaging errors
error-messaging-not-registered = Messaging client is not registered
error-messaging-registration-failed = Messaging registration failed: { $reason }
error-messaging-send-failed = Failed to send message: { $reason }
error-messaging-poll-failed = Failed to poll messages: { $reason }
error-messaging-ack-failed = Failed to acknowledge message { $message_id }: { $reason }
error-messaging-disabled = Messaging is disabled for this license

# Messaging info
messaging-registered = Messaging registered for device { $device_id }
messaging-unregistered = Messaging unregistered
messaging-message-received = Message received: { $subject } (type: { $message_type })
messaging-message-sent = Message sent (id: { $message_id })
messaging-poll-completed = Message poll completed: { $count } new messages

# Provider credential descriptions
credential-xai-api-key = xAI API key for Grok (XAI_API_KEY)
credential-openai-api-key = OpenAI API key (OPENAI_API_KEY)
credential-google-api-key = Google API key for Gemini (GOOGLE_API_KEY)
credential-gh-auth = GitHub authentication via gh CLI (gh auth login)

# Built-in category names
category-SoftwareDevelopment = Software Development
category-LinuxDevelopment = Linux Development
category-macOSDevelopment = macOS Development
category-PythonDevelopment = Python Development
category-AIFrameworks = AI & ML Frameworks
category-GraphQL = GraphQL Frameworks
category-DataScience = Data Science & Analytics
category-Legal = Legal / Paralegal
category-Music = Music Production
category-PhysicalSystems = Physical Systems & Phenomena
category-BacteriaScience = Bacteria Science & Microbiology
category-NursingScience = Nursing Science & Clinical Practice
category-ElectronDevelopment = Electron Desktop Development
category-GameDevelopment = Game Development
category-3DModeling = 3D Modeling & Digital Content Creation
category-Custom = Custom Templates

# Built-in category descriptions
category-SoftwareDevelopment-desc = Templates for creating applications, APIs, databases, and scripts
category-LinuxDevelopment-desc = Templates for Linux system administration, shell scripting, and server development
category-macOSDevelopment-desc = Templates for macOS applications, Swift/Objective-C development, and Apple frameworks
category-PythonDevelopment-desc = Templates for Python applications, scripts, web frameworks, and automation
category-AIFrameworks-desc = Templates for AI agents, LLM orchestration, chatbots, and ML applications
category-GraphQL-desc = Templates for GraphQL servers, clients, and API development
category-DataScience-desc = Templates for data science lifecycle: mathematics, data engineering, ML, deep learning, MLOps
category-Legal-desc = Templates for legal document processing, research, and case management
category-Music-desc = Templates for DAWs, plugin development, modular synthesis, and hardware integration
category-PhysicalSystems-desc = Templates for industrial physics, process monitoring, control systems, and predictive analytics
category-BacteriaScience-desc = Templates for microbiology, genomics, metagenomics, antimicrobial resistance, and diagnostics
category-NursingScience-desc = Templates for nursing education, clinical practice, patient care, and healthcare analytics
category-ElectronDevelopment-desc = Templates for cross-platform desktop applications with Electron and modern tooling
category-GameDevelopment-desc = Templates for game engines, frameworks, and interactive entertainment development
category-3DModeling-desc = Templates for 3D modeling, VFX, animation, and digital content creation tools
category-Custom-desc = User-created custom templates

# Provider status
provider-not-installed = Provider { $provider } requires { $binary } which is not installed
provider-binary-found = Found { $binary } at { $path }
provider-test-timeout = Connection test timed out after { $seconds } seconds
provider-test-failed = Provider test failed: { $error }
provider-env-saved = Saved { $env_var } for { $provider }

# Worktree git integration messages
worktree-git-not-found = git executable not found: { $detail }
worktree-git-check-failed = git is not functional in this environment
worktree-not-git-repo = Path is not a git repository: { $path }
worktree-not-found = No active worktree found for this worker
worktree-dir-create-failed = Failed to create worktree directory at { $path }: { $detail }
worktree-merge-exec-failed = Failed to spawn git merge process: { $detail }
worktree-gitignore-read-failed = Failed to read .gitignore: { $detail }
worktree-gitignore-write-failed = Failed to write .gitignore: { $detail }
worktree-gitignore-create-failed = Failed to create .gitignore: { $detail }
worktree-git-exec-failed = Failed to spawn git command ({ $cmd }): { $detail }
worktree-git-cmd-failed = git command failed ({ $cmd }): { $detail }

# Autonomous Sprint Completion (WAVE 3)
error-orchestration-quorum-not-met = Manager quorum not met: { $received } of { $expected } proposals received (minimum: { $min_quorum })
orchestration-file-conflict-serialized = Task { $later_task } serialized after { $earlier_task } — both claim file '{ $file }'
orchestration-build-check-failed = Per-worker { $project_type } build check failed for worker { $worker_id }
orchestration-build-check-passed = Per-worker { $project_type } build check passed for worker { $worker_id }
