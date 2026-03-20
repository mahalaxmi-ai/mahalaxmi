# 错误
error-config-file-not-found = 未找到配置文件: { $path }
error-config-parse-failed = 配置解析失败: { $reason }
error-config-validation-failed = 配置验证失败: { $details }
error-locale-not-supported = 不支持的语言环境"{ $locale }"
error-log-init-failed = 日志初始化失败: { $reason }
error-log-dir-create-failed = 无法在{ $path }创建日志目录: { $reason }
error-app-launch-failed = 应用程序启动失败: { $reason }

# 验证
validation-invalid-log-level = 无效的日志级别"{ $level }"。有效级别: { $valid }
validation-workers-out-of-range = max_concurrent_workers必须在{ $min }到{ $max }之间，当前值: { $value }
validation-manager-timeout-too-low = manager_timeout_seconds必须至少为{ $min }，当前值: { $value }
validation-worker-timeout-too-low = worker_timeout_seconds必须至少为{ $min }，当前值: { $value }
validation-offline-grace-too-low = offline_grace_days必须至少为{ $min }，当前值: { $value }
validation-invalid-consensus-strategy = 无效的共识策略"{ $value }"。有效策略: { $valid }
validation-invalid-data-directory = data_directory包含无效字符
validation-empty-default-provider = providers.default_provider不能为空
validation-invalid-theme = 无效的主题"{ $value }"。有效主题: { $valid }
validation-font-size-out-of-range = terminal_font_size必须在{ $min }到{ $max }之间，当前值: { $value }
validation-invalid-max-batch-retries = max_batch_retriesu5fc5u987b>=1uff0cu5f53u524du5024: { $value }
validation-invalid-max-total-batches = max_total_batchesu5fc5u987b>=2uff0cu5f53u524du5024: { $value }
validation-invalid-stall-detection-threshold = stall_detection_thresholdu5fc5u987b>=2uff0cu5f53u524du5024: { $value }

# 配置
config-loaded-successfully = 已从{ $path }加载配置
config-using-default = 未找到配置文件，使用默认值
config-env-override = 配置值已被环境变量{ $var }覆盖
config-env-override-invalid = 环境变量{ $var }的值"{ $value }"无效，保持已配置的值
config-generated-successfully = 已在{ $path }生成默认配置
config-already-exists = 配置文件已存在于{ $path }

# 日志
logging-initialized = 日志已在{ $level }级别初始化
logging-rust-log-override = 检测到RUST_LOG环境变量，覆盖已配置的日志级别
logging-file-path = 日志文件: { $path }
logging-dir-create-failed-fallback = 无法创建日志目录{ $path }，回退到仅控制台日志

# PTY
pty-open-failed = 无法打开伪终端: { $reason }
pty-spawn-failed = 无法在PTY中启动"{ $program }": { $reason }
pty-write-failed = 写入终端{ $terminal_id }失败: { $reason }
pty-read-failed = 从终端{ $terminal_id }读取失败: { $reason }
pty-resize-failed = 调整终端{ $terminal_id }大小为{ $rows }x{ $cols }失败: { $reason }
pty-wait-failed = 检查终端{ $terminal_id }进程状态失败: { $reason }
pty-kill-failed = 终止终端{ $terminal_id }中的进程失败: { $reason }

# 应用
app-starting = Mahalaxmi v{ $version } 正在启动
app-ready = Mahalaxmi已就绪
app-shutting-down = Mahalaxmi正在关闭

# 凭证
credential-anthropic-api-key = Claude Code的Anthropic API密钥
credential-generic-api-key = { $provider }的API密钥
credential-aws-iam-role = { $provider }的AWS IAM角色
credential-oauth-token = { $provider }的OAuth令牌

# 提供商
error-provider-credentials-missing = { $provider }凭证缺失：环境变量{ $env_var }未设置
error-provider-credentials-invalid = { $provider }凭证无效：{ $reason }
error-provider-not-found = 注册表中未找到提供商"{ $provider_id }"
error-provider-no-default = 未配置默认AI提供商
error-provider-command-build-failed = 构建{ $provider }命令失败：{ $reason }
provider-registered = 提供商"{ $provider }"已注册，ID为"{ $id }"
provider-set-default = 默认提供商已设为"{ $provider }"
provider-credentials-valid = { $provider }凭证验证成功
provider-validating = 正在验证{ $provider }凭证
provider-list-header = 已注册AI提供商

# PTY（扩展）
error-pty-open-failed = 打开PTY失败：{ $reason }
error-pty-spawn-failed = 在PTY中启动"{ $program }"失败：{ $reason }
error-pty-write-failed = 写入终端{ $terminal_id }失败：{ $reason }
error-pty-read-failed = 读取终端{ $terminal_id }失败：{ $reason }
error-pty-resize-failed = 调整终端{ $terminal_id }大小为{ $rows }x{ $cols }失败：{ $reason }
error-pty-kill-failed = 终止终端{ $terminal_id }进程失败：{ $reason }
error-pty-wait-failed = 检查终端{ $terminal_id }进程状态失败：{ $reason }
error-pty-terminal-not-found = 未找到终端{ $terminal_id }
error-pty-max-concurrent-reached = 已达到最大并发终端数（{ $max }）
pty-process-spawned = 进程"{ $program }"已在终端{ $terminal_id }中启动
pty-process-exited = 终端{ $terminal_id }进程以代码{ $exit_code }退出
pty-session-closed = 终端会话{ $terminal_id }已关闭
pty-resized = 终端{ $terminal_id }已调整为{ $rows }x{ $cols }
pty-reader-eof = 终端{ $terminal_id }读取器已到达流末尾
pty-reader-error = 终端{ $terminal_id }读取器错误：{ $reason }

# 编排错误
error-orchestration-invalid-transition = 从{ $from }到{ $to }的无效状态转换
error-orchestration-circular-dependency = 检测到循环依赖: { $cycle }
error-orchestration-worker-not-found = 工作器{ $worker_id }未在队列中找到
error-orchestration-max-retries-exceeded = 工作器{ $worker_id }超过最大重试次数（{ $max_retries }）
error-orchestration-no-proposals = 未收到管理器提案
error-orchestration-plan-validation-failed = 执行计划验证失败: { $errors }
error-orchestration-consensus-failed = 共识引擎失败: { $reason }
error-orchestration-queue-full = 工作器队列已满（最大{ $max }）
error-orchestration-manager-timeout = 管理器{ $manager_id }在{ $timeout }秒后超时
error-orchestration-worker-timeout = 工作器{ $worker_id }在{ $timeout }秒后超时

# 编排信息
orchestration-cycle-started = 编排周期{ $cycle_id }已启动
orchestration-state-changed = 状态已更改: { $from } -> { $to }
orchestration-manager-completed = 管理器{ $manager_id }完成，共{ $task_count }个任务
orchestration-consensus-reached = 达成共识: { $agreed }项同意，{ $dissenting }项反对
orchestration-plan-created = 执行计划已创建: { $phases }个阶段，{ $workers }个工作器
orchestration-worker-started = 工作器{ $worker_id }已启动: { $task }
orchestration-worker-completed = 工作器{ $worker_id }在{ $duration }ms内完成
orchestration-worker-failed = 工作器{ $worker_id }失败: { $error }
orchestration-cycle-completed = 周期在{ $duration }ms内完成（成功率: { $success_rate }）
orchestration-worker-retrying = 工作器{ $worker_id }正在重试（第{ $attempt }/{ $max }次尝试）

# 检测错误
error-detection-rule-compile-failed = 编译检测规则模式失败: { $reason }
error-detection-no-rules-loaded = 未加载检测规则
error-detection-invalid-pattern = 无效的检测模式"{ $pattern }": { $reason }

# 检测信息
detection-rule-matched = 检测规则"{ $rule }"匹配，动作: { $action }
detection-rule-cooldown = 检测规则"{ $rule }"被冷却时间抑制（剩余{ $remaining_ms }ms）
detection-rules-loaded = 已加载{ $count }条检测规则
detection-provider-rules-applied = 已为提供者{ $provider }应用{ $count }条规则
detection-error-pattern-detected = 检测到错误模式: "{ $pattern }"（出现{ $count }次）
detection-root-cause-hypothesis = 根本原因假设: { $category }（置信度: { $confidence }）
detection-recurring-error = 重复错误: "{ $message }"（发生{ $count }次）
detection-action-executed = 动作{ $action }已对规则"{ $rule }"执行
detection-cooldowns-reset = 已重置{ $rule_count }条规则的冷却时间

# 模板错误
error-template-not-found = 未找到模板 { $template_id }
error-template-category-not-found = 未找到模板类别 { $category_id }
error-template-composition-failed = 模板组合失败：{ $reason }
error-template-include-not-found = 未找到包含文件：{ $path }
error-template-circular-include = 检测到循环包含（超过最大深度 { $depth }）
error-template-placeholder-unresolved = 未解析的占位符：${ $placeholder }
error-template-validation-failed = 模板验证失败，存在 { $count } 个错误
error-template-activation-failed = 模板激活失败：{ $reason }
error-template-catalog-load-failed = 加载模板目录失败：{ $path }
error-template-invalid-version = 无效的模板版本格式：{ $version }

# 模板信息
template-catalog-loaded = 模板目录已加载，包含 { $count } 个模板
template-activated = 模板 { $template_id } 激活成功
template-composition-complete = 组合完成：{ $included } 个已包含，{ $overridden } 个已覆盖
template-placeholders-resolved = 已解析 { $count } 个占位符
template-validation-passed = 域 { $domain } 的模板验证通过
template-validation-warnings = 模板验证完成，存在 { $count } 个警告
template-include-resolved = 包含已解析：{ $path }
template-provider-instructions-injected = 已为提供商 { $provider } 注入指令
template-project-config-loaded = 已从 { $path } 加载项目配置
template-domain-validator-registered = 域验证器已注册：{ $domain }

# 许可证错误
error-license-file-not-found = 未找到许可证文件：{ $path }
error-license-file-invalid = 无效的许可证文件 ({ $path })：{ $reason }
error-license-file-write-failed = 写入许可证文件失败 ({ $path })：{ $reason }
error-license-signature-invalid = 许可证签名验证失败
error-license-signature-decode-failed = 解码许可证签名失败：{ $reason }
error-license-serialization-failed = 序列化许可证数据失败：{ $reason }
error-license-signing-failed = 签署许可证失败：{ $reason }
error-license-feature-denied = 功能"{ $feature }"在{ $tier }方案中不可用
error-license-worker-limit = 请求的{ $requested }个工作器超出了{ $tier }方案的上限{ $limit }
error-license-manager-limit = 请求的{ $requested }个管理器超出了{ $tier }方案的上限{ $limit }。请在设置 → 许可证中激活订阅以解锁更多权限。
error-license-category-denied = 类别"{ $category }"需要{ $required_tier }方案（当前：{ $tier }）
error-license-fingerprint-hostname = 无法确定主机名：{ $reason }
error-license-fingerprint-username = 无法确定用户名：{ $reason }

# 许可证状态
license-trial-active = 试用许可证有效（剩余{ $days }天）
license-trial-expiring-soon = 试用期即将到期（剩余{ $days }天）
license-trial-expiring-very-soon = 试用期即将到期（剩余{ $days }天）
license-trial-expired = 试用许可证已过期
license-expires-later = 许可证将在{ $days }天后过期
license-expires-soon = 许可证即将过期（剩余{ $days }天）
license-expires-very-soon = 许可证即将过期（剩余{ $days }天）
license-expires-today = 许可证今日到期
license-grace-period = 许可证已过期，宽限期生效中（剩余{ $days }天）
license-expired = 许可证已过期

# 平台支持 — 错误键
error-platform-unsupported = 不支持的平台: { $platform }
error-platform-wsl-not-detected = 未检测到WSL环境
error-platform-wsl-path-invalid = WSL转换的路径无效: { $path }
error-platform-layout-no-space = 容器太小，无法进行面板布局 ({ $width }x{ $height })
error-platform-layout-invalid-count = 无效的面板数量: { $count }
error-platform-hotkey-registration-failed = 快捷键注册失败（冲突）: { $shortcut }
error-platform-hotkey-parse-failed = 快捷键解析失败: { $shortcut }
error-platform-shutdown-timeout = 进程 { $pid } ({ $label }) 关闭超时
error-platform-shutdown-failed = 进程 { $pid } ({ $label }) 关闭失败
error-platform-shell-not-found = 未找到默认Shell

# 平台支持 — 信息键
platform-detected = 检测到平台: { $os } ({ $arch })
platform-wsl-detected = 检测到WSL: { $distro } (WSL{ $version })
platform-wsl-path-translated = 路径已转换: { $from } → { $to }
platform-layout-calculated = 布局已计算: { $panels } 个面板，{ $rows }x{ $cols } 网格
platform-layout-optimized = 布局已优化: { $utilization }% 利用率
platform-hotkey-registered = 快捷键已注册: { $command } → { $shortcut }
platform-hotkey-unregistered = 快捷键已注销: { $command }
platform-shutdown-initiated = 已启动 { $count } 个进程的关闭
platform-shutdown-completed = 关闭完成: { $count } 个进程，耗时 { $duration }ms
platform-shell-detected = 检测到Shell: { $shell } ({ $path })

# 记忆错误
error-memory-not-found = 未找到记忆条目：{ $id }
error-memory-duplicate = 重复的记忆条目：{ $id }
error-memory-persistence-failed = 记忆存储持久化失败：{ $reason }
error-memory-load-failed = 记忆存储加载失败：{ $reason }
error-memory-invalid-confidence = 无效的置信度分数：{ $value }（必须在0.0到1.0之间）
error-memory-store-full = 记忆存储已满（最多{ $max }个条目）
error-memory-invalid-query = 无效的记忆查询：{ $reason }
error-memory-serialization = 记忆序列化失败：{ $reason }
error-memory-invalid-entry = 无效的记忆条目：{ $reason }
error-memory-session-mismatch = 会话不匹配：预期 { $expected }，实际 { $actual }

# 记忆信息
memory-store-created = 已为会话 { $session_id } 创建记忆存储
memory-entry-added = 已添加记忆条目：{ $title }（类型：{ $memory_type }）
memory-entry-updated = 已更新记忆条目：{ $id }
memory-entry-removed = 已删除记忆条目：{ $id }
memory-store-cleared = 已清空记忆存储（删除了{ $count }个条目）
memory-persisted = 记忆存储已持久化到 { $path }
memory-loaded = 已从 { $path } 加载记忆存储（{ $count }个条目）
memory-query-executed = 记忆查询返回了{ $count }个结果
memory-injected = 已注入{ $count }条记忆（{ $tokens }个令牌）
memory-stats = 记忆统计：{ $total }个条目，平均置信度 { $avg_confidence }

# 索引错误
error-indexing-parse-failed = 解析 { $file } 失败：{ $reason }
error-indexing-file-read-failed = 读取文件 { $file } 失败：{ $reason }
error-indexing-unsupported-language = 不支持的文件扩展名对应语言：{ $extension }
error-indexing-extraction-failed = { $file } 的符号提取失败：{ $reason }
error-indexing-graph-cycle-detected = 检测到依赖循环：{ $files }
error-indexing-fingerprint-failed = 计算 { $file } 的指纹失败：{ $reason }
error-indexing-build-failed = 索引构建失败：{ $reason }
error-indexing-update-failed = 增量更新失败：{ $reason }

# 索引信息
indexing-file-indexed = 文件已索引：{ $file }（{ $language }）
indexing-symbols-extracted = 从 { $file } 中提取了 { $count } 个符号
indexing-graph-built = 依赖图已构建：{ $files } 个文件，{ $edges } 条边
indexing-ranking-computed = 已为 { $symbols } 个符号计算排名
indexing-repomap-generated = 仓库地图已生成：{ $symbols } 个符号，{ $tokens } 个令牌
indexing-index-built = 代码库索引已构建：{ $files } 个文件，{ $symbols } 个符号
indexing-incremental-update = 增量更新：{ $added } 个新增，{ $modified } 个修改，{ $removed } 个删除
indexing-language-registered = 语言已注册：{ $language }

# 上下文错误
error-context-budget-exceeded = 上下文令牌预算超出：已使用 { $used }，预算 { $budget }
error-context-invalid-allocations = 预算分配总和必须 <= 1.0，实际为 { $sum }
error-context-build-failed = 任务 { $task_id } 的上下文构建失败：{ $reason }
error-context-invalid-format = 无效的上下文格式：{ $format }

# 上下文信息
context-budget-allocated = 令牌预算已分配：{ $total } 个令牌（{ $repo_map } 仓库地图，{ $files } 文件，{ $memory } 内存，{ $task } 任务）
context-files-scored = 已评估 { $count } 个文件的相关性（最高：{ $top_file }）
context-chunks-created = 已创建 { $count } 个代码片段（{ $tokens } 个令牌）
context-assembled = 上下文已组装：{ $sections } 个部分，{ $budget } 预算中已使用 { $tokens } 个令牌
context-injected = 已为工作器 { $worker_id } 注入上下文（{ $tokens } 个令牌，{ $files } 个文件）
context-skipped = 已跳过上下文准备：{ $reason }

# MCP错误
error-mcp-parse-failed = 解析JSON-RPC消息失败：{ $reason }
error-mcp-invalid-request = 无效的JSON-RPC请求：{ $reason }
error-mcp-method-not-found = 方法未找到：{ $method }
error-mcp-invalid-params = 无效参数：{ $reason }
error-mcp-internal-error = MCP服务器内部错误：{ $reason }
error-mcp-not-initialized = MCP服务器尚未初始化
error-mcp-tool-not-found = 工具未找到：{ $tool }
error-mcp-tool-execution-failed = 工具"{ $tool }"执行失败：{ $reason }
error-mcp-transport-error = MCP传输错误：{ $reason }
error-mcp-shutdown-failed = MCP服务器关闭失败：{ $reason }

# MCP信息
mcp-server-started = MCP服务器已启动（{ $transport } 传输）
mcp-server-stopped = MCP服务器已停止
mcp-client-initialized = MCP客户端已初始化：{ $client_name }
mcp-tool-called = 工具已调用：{ $tool }
mcp-tool-completed = 工具"{ $tool }"在 { $duration }ms 内完成
mcp-request-received = 收到请求：{ $method }
mcp-response-sent = 已发送响应：{ $method }
mcp-transport-ready = MCP传输就绪：{ $transport }

# Graph errors
error-graph-entity-not-found = 未找到图实体：{ $id }
error-graph-relationship-failed = 添加关系失败：{ $reason }
error-graph-build-failed = 构建知识图谱失败：{ $reason }
error-graph-update-failed = 更新知识图谱失败：{ $reason }
error-graph-load-failed = 从 { $path } 加载知识图谱失败：{ $reason }
error-graph-save-failed = 保存知识图谱到 { $path } 失败：{ $reason }
error-graph-max-entities-exceeded = 知识图谱超过最大实体限制：{ $count } / { $max }

# Graph info
graph-built = 知识图谱已构建：{ $entities } 个实体，{ $relationships } 个关系
graph-updated = 知识图谱已更新：添加 { $added } 个，删除 { $removed } 个
graph-entity-added = 实体已添加到知识图谱：{ $name }（{ $kind }）
graph-entity-removed = 实体已从知识图谱移除：{ $name }
graph-persisted = 知识图谱已持久化到 { $path }
graph-loaded = 知识图谱已从 { $path } 加载（{ $entities } 个实体）
graph-query-executed = 图查询在 { $ms }ms 内完成，{ $results } 个结果

# 平台API错误
error-platform-api-request-failed = 平台API请求失败：{ $reason }
error-platform-api-unauthorized = 平台API认证失败 — 请检查channel_api_key
error-platform-api-not-found = 平台资源未找到：{ $resource }
error-platform-api-rate-limited = 平台API速率受限 — 请在 { $seconds }秒后重试
error-platform-api-server-error = 平台服务器错误（{ $status }）：{ $message }
error-platform-trial-not-eligible = 此设备不符合试用条件：{ $reason }
error-platform-activation-failed = 许可证激活失败：{ $reason }
error-platform-validation-failed = 许可证验证失败：{ $reason }
error-platform-deactivation-failed = 设备停用失败：{ $reason }
error-platform-cache-read-failed = 从 { $path } 读取许可证缓存失败：{ $reason }
error-platform-cache-write-failed = 向 { $path } 写入许可证缓存失败：{ $reason }
error-platform-cache-decrypt-failed = 许可证缓存解密失败（密钥不匹配或数据损坏）
error-platform-not-configured = 平台集成未配置 — 请在配置中设置platform_base_url

# 平台API信息
platform-api-trial-activated = 试用已激活：{ $tier } 方案，{ $days } 天
platform-api-license-activated = 许可证已激活：{ $tier } 方案（激活 { $activation_id }）
platform-api-license-validated = 许可证已验证：{ $tier } 方案，剩余 { $days } 天
platform-api-heartbeat-sent = 心跳已发送（激活 { $activation_id }）
platform-api-device-deactivated = 设备已从许可证中停用
platform-api-cache-updated = 许可证缓存已更新：{ $path }
platform-api-offline-fallback = 平台不可达，使用缓存的许可证（缓存于 { $days_ago } 天前）

# 消息传递错误
error-messaging-not-registered = 消息客户端未注册
error-messaging-registration-failed = 消息注册失败：{ $reason }
error-messaging-send-failed = 发送消息失败：{ $reason }
error-messaging-poll-failed = 轮询消息失败：{ $reason }
error-messaging-ack-failed = 确认消息 { $message_id } 失败：{ $reason }
error-messaging-disabled = 此许可证已禁用消息传递

# 消息传递信息
messaging-registered = 消息传递已为设备 { $device_id } 注册
messaging-unregistered = 消息传递已注销
messaging-message-received = 收到消息：{ $subject }（类型：{ $message_type }）
messaging-message-sent = 消息已发送（ID：{ $message_id }）
messaging-poll-completed = 消息轮询完成：{ $count } 条新消息

# Provider credential descriptions
credential-xai-api-key = Grok 的 xAI API 密钥 (XAI_API_KEY)
credential-openai-api-key = OpenAI API 密钥 (OPENAI_API_KEY)
credential-google-api-key = Gemini 的 Google API 密钥 (GOOGLE_API_KEY)
credential-gh-auth = 通过 gh CLI 进行 GitHub 认证 (gh auth login)

# Built-in category names
category-SoftwareDevelopment = 软件开发
category-LinuxDevelopment = Linux 开发
category-macOSDevelopment = macOS 开发
category-PythonDevelopment = Python 开发
category-AIFrameworks = AI 和 ML 框架
category-GraphQL = GraphQL 框架
category-DataScience = 数据科学与分析
category-Legal = 法律 / 律师助理
category-Music = 音乐制作
category-PhysicalSystems = 物理系统与现象
category-BacteriaScience = 细菌科学与微生物学
category-NursingScience = 护理科学与临床实践
category-ElectronDevelopment = Electron 桌面开发
category-GameDevelopment = 游戏开发
category-3DModeling = 3D 建模与数字内容创作
category-Custom = 自定义模板

# Built-in category descriptions
category-SoftwareDevelopment-desc = 用于创建应用程序、API、数据库和脚本的模板
category-LinuxDevelopment-desc = 用于 Linux 系统管理、Shell 脚本和服务器开发的模板
category-macOSDevelopment-desc = 用于 macOS 应用程序、Swift/Objective-C 开发和 Apple 框架的模板
category-PythonDevelopment-desc = 用于 Python 应用程序、脚本、Web 框架和自动化的模板
category-AIFrameworks-desc = 用于 AI 代理、LLM 编排、聊天机器人和 ML 应用程序的模板
category-GraphQL-desc = 用于 GraphQL 服务器、客户端和 API 开发的模板
category-DataScience-desc = 用于数据科学生命周期的模板：数学、数据工程、ML、深度学习、MLOps
category-Legal-desc = 用于法律文档处理、研究和案件管理的模板
category-Music-desc = 用于 DAW、插件开发、模块化合成和硬件集成的模板
category-PhysicalSystems-desc = 用于工业物理、过程监控、控制系统和预测分析的模板
category-BacteriaScience-desc = 用于微生物学、基因组学、宏基因组学、抗菌耐药性和诊断的模板
category-NursingScience-desc = 用于护理教育、临床实践、患者护理和医疗分析的模板
category-ElectronDevelopment-desc = 用于使用 Electron 和现代工具构建跨平台桌面应用程序的模板
category-GameDevelopment-desc = 用于游戏引擎、框架和交互式娱乐开发的模板
category-3DModeling-desc = 用于 3D 建模、VFX、动画和数字内容创作工具的模板
category-Custom-desc = 用户创建的自定义模板

# Provider status
provider-not-installed = 提供商 { $provider } 需要 { $binary }，但未安装
provider-binary-found = 在 { $path } 找到 { $binary }
provider-test-timeout = 连接测试在 { $seconds } 秒后超时
provider-test-failed = 提供商测试失败：{ $error }
provider-env-saved = 已为 { $provider } 保存 { $env_var }

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
error-orchestration-quorum-not-met = 管理器法定人数不足：收到 { $received }/{ $expected } 个提案（最少: { $min_quorum }）
orchestration-file-conflict-serialized = 任务 { $later_task } 在 { $earlier_task } 之后序列化 — 两者都声明文件 '{ $file }'
orchestration-build-check-failed = 工作者 { $worker_id } 的每工作者 { $project_type } 构建检查失败
orchestration-build-check-passed = 工作者 { $worker_id } 的每工作者 { $project_type } 构建检查通过
