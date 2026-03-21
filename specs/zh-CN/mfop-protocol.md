# Mahalaxmi 联邦与编排协议

**MFOP v1.0** · 同行评审草稿

| | |
|---|---|
| Date | 2026年3月 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | 美国佛罗里达州西棕榈滩 |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## 本备忘录状态

本文档是 Mahalaxmi 联邦与编排协议（MFOP）规范 1.0 版本的预发布草稿，面向同行评审发布，征集意见。本文档描述了一种协议，用于在异构计算节点上实现联邦式分布式 AI 编排，支持合规区域感知路由、密码学签名计费收据以及可配置的经济结算。

意见和问题请发送至作者邮箱 Ami.nunez@mahalaxmi.ai。当前草稿及讨论线程维护于 https://mahalaxmi.ai/mfop/draft，讨论区地址为 https://mahalaxmi.ai/mfop/discuss。

## 版权声明

版权所有 © 2026 ThriveTech Services LLC。保留所有权利。在任何介质中，允许免费复制、分发和使用本文档，但须在所有副本及衍生作品中保留作者署名、文档标题及本版权声明。

## 摘要

本文档定义了 Mahalaxmi 联邦与编排协议（MFOP），这是一种用于在异构计算节点分布式网络中协调并行 AI 代理执行的协议。MFOP 规定了节点身份与注册、能力通告、合规区域感知的作业路由、语义输入分区、密码学签名计费收据、可配置的经济结算，以及使用 AI 安全策略验证和执行沙箱隔离的分层安全模型。

MFOP 被设计为同时支持三种部署配置：由单一组织拥有并运营节点的私有企业网格、由平台提供商运营的托管云资源池，以及任何节点运营商均可贡献算力并获得经济结算的开放社区市场。该协议对底层 AI 模型提供商保持中立，并设计为随 AI 安全与合规领域的发展持续演进。

## 1. 引言

大型语言模型（LLM）在企业环境中的大规模部署，产生了对协调层的需求——该协调层需要能够跨越异构计算基础设施，同时满足因司法管辖区和行业不同而各异的合规、计费和安全要求。

MFOP 通过定义联邦式 AI 编排协议来满足这一需求。一个联邦由一个或多个计算节点组成，每个节点可由不同实体在不同合规制度下运营。提交方——用户、应用程序或自动化系统——向联邦提交作业。联邦根据作业的合规区域要求、节点的能力通告以及当前的经济条款，将作业路由至合适的节点。

本规范定义了符合 MFOP 标准的联邦中所有组件的线路协议、数据格式、密码机制和行为要求。

## 2. 术语

本文档中的关键词"必须"（MUST）、"不得"（MUST NOT）、"必需"（REQUIRED）、"应当"（SHALL）、"不应当"（SHALL NOT）、"应该"（SHOULD）、"不应该"（SHOULD NOT）、"推荐"（RECOMMENDED）、"不推荐"（NOT RECOMMENDED）、"可以"（MAY）和"可选"（OPTIONAL），须按照 BCP 14 [RFC2119] [RFC8174] 中的描述加以解释。

**联邦（Federation）** — 在共享治理配置下运营的一个或多个符合 MFOP 标准的计算节点的逻辑分组。

**节点（Node）** — 在联邦中注册、接受、执行并返回 AI 工作负载的计算资源。节点可以是单台服务器、集群或云计算资源池。

**提交方（Submitter）** — 向联邦提交 AI 工作负载以供执行的实体（用户、应用程序或自动化系统）。

**合规区域（Compliance Zone）** — 对作业路由、数据处理和输出验证加以约束的具名策略上下文。已定义的区域：public（公共）、enterprise（企业，SOC2）、hipaa、sox、fedramp。

**作业（Job）** — 提交给联邦执行的一个离散 AI 工作负载单元。作业携带有效负载、合规区域声明和计费授权。

**收据（Receipt）** — 已完成作业执行的密码学签名记录，包含令牌计数、时间戳、节点身份和计费金额。

**经济结算（Economic Settlement）** — 将累积的计费收据转换为提交方、节点运营商与平台之间财务转账的过程。

**PAK Key（平台 API 密钥）** — 由平台颁发的持有者凭证，用于授权访问联邦 API 端点。

**NeMo Guardrails** — NVIDIA NeMo 安全框架，由 MFOP 节点用于 AI 安全策略验证和输出过滤。

## 3. 节点身份与注册

MFOP 联邦中的每个节点由一个稳定、全球唯一的节点标识符（node_id）标识。node_id 是在注册时分配的 128 位 UUID（第 4 版），在节点重启和软件升级后保持不变。

**3.1 注册流程**

节点通过向联邦的注册端点（POST /v1/federation/nodes/register）发送 NodeRegistrationRequest 来发起注册。请求**必须**包含：

- node_id：候选 UUID（联邦**可以**覆盖此值）
- operator_id：注册运营商账户的 UUID
- display_name：节点的人类可读名称（最多 64 个字符）
- public_key：以 base64url 编码的 Ed25519 公钥，用于收据签名
- capability_advertisement：CapabilityAdvertisement 对象（见第 4 节）
- compliance_zones：节点经认证可处理的合规区域集合
- endpoint_url：节点接受作业提交的 HTTPS URL

联邦返回 NodeRegistrationResponse，包含分配的 node_id、用于后续认证调用的 registration_token，以及联邦当前的计费配置。

**3.2 重新注册与密钥轮换**

节点在轮换其 Ed25519 密钥对时**必须**重新注册。在密钥轮换期间，节点提交包含新旧两个公钥的重新注册请求，并使用旧私钥签名。联邦在接受新密钥之前验证旧密钥签名。存在 24 小时的重叠窗口期，在此期间使用任一密钥签名的收据均被接受。

**3.3 节点健康状态与注销**

节点**必须**至少每 60 秒向 POST /v1/federation/nodes/{id}/heartbeat 发送一次心跳。连续错过三个心跳窗口的节点将被标记为 INACTIVE 并从路由中排除。节点可通过 DELETE /v1/federation/nodes/{id} 主动注销。

## 4. 能力通告

节点的能力通告声明了节点上可用的 AI 模型、与作业路由相关的硬件特性，以及节点运营商持有的合规认证。

**4.1 CapabilityAdvertisement 对象**

CapabilityAdvertisement 对象包含以下字段：

- models：ModelDescriptor 对象数组（见 4.2）
- hardware_class：{ cpu, gpu_consumer, gpu_datacenter, tpu } 之一
- vram_gb：可用于推理的 GPU 显存总量，单位为吉字节（CPU 节点为 0）
- max_context_tokens：节点可服务的最大上下文窗口大小
- max_concurrent_jobs：节点可同时执行的最大作业数
- compliance_certifications：认证标识符数组（例如 "soc2-type2"、"hipaa-baa"、"fedramp-moderate"）
- nemo_rails_version：节点上安装的 NeMo Guardrails 运行时版本

**4.2 ModelDescriptor**

节点上每个可用模型由一个 ModelDescriptor 描述：

- model_id：规范模型标识符字符串（例如 "meta-llama/Meta-Llama-3-70B-Instruct"）
- model_family：{ llama, mistral, gemma, falcon, phi, custom } 之一
- parameter_count_b：以十亿为单位的近似参数数量
- quantization：{ fp16, bf16, int8, int4, none } 之一
- context_window_tokens：该模型的最大上下文窗口大小
- supports_tool_use：布尔值
- supports_vision：布尔值

**4.3 能力刷新**

当节点的可用模型或硬件配置发生变化时，节点**必须**通过 PUT /v1/federation/nodes/{id}/capabilities 更新其能力通告。联邦在 30 秒内将更新后的能力通告传播至路由层。

## 5. 合规区域感知的作业路由

MFOP 将每个作业路由至满足其合规区域要求的节点。合规区域满足条件是硬性约束：作业**不得**被路由至未经认证支持该合规区域的节点。

**5.1 合规区域**

MFOP 定义了五个合规区域，按限制程度从低到高排列：

- public（公共）：除基线 NeMo 安全护栏外无额外合规要求，适用于通用 AI 工作负载。
- enterprise（企业，SOC2）：需要 SOC 2 Type II 认证，增加数据驻留检测、API 凭证泄露检测和访问日志强制执行。
- hipaa：需要 HIPAA BAA，增加 PHI 模式检测、PHI 去标识化和最小必要输出检查。
- sox：需要 SOX 合规控制，增加金融 PII 隔离、价格预测拦截和 MNPI 检测。
- fedramp：需要 FedRAMP 授权，增加 CUI 处理、出口管制检测和分类标记强制执行。

**5.2 路由算法**

收到作业时，路由层执行以下算法：

1. 筛选：识别所有状态为 ACTIVE 且已通过该作业合规区域认证的节点。
2. 筛选：移除 max_context_tokens 小于作业估算令牌数的节点。
3. 筛选：移除 max_concurrent_jobs 当前已满的节点。
4. 评分：对每个剩余节点计算路由分数：score = w_latency × latency_score + w_cost × cost_score + w_affinity × affinity_score。默认权重：w_latency = 0.4，w_cost = 0.4，w_affinity = 0.2。
5. 选择：路由至得分最高的节点。如出现并列，随机均匀选择。

若没有节点满足所有筛选条件，作业将进入队列并等待可配置的超时时间（默认：120 秒）。若在超时内仍无节点可用，联邦返回 HTTP 503 并附带 Retry-After 头部。

**5.3 亲和性规则**

提交方**可以**在作业提交中指定亲和性规则：

- node_affinity：首选 node_id 列表（软偏好）
- anti_affinity：需排除的 node_id 列表（硬约束）
- geography：首选地理区域（ISO 3166-1 alpha-2 国家代码）

亲和性规则仅影响 affinity_score 分量；合规区域认证和容量仍为硬性约束。

## 6. 语义输入分区

对于输入超过单个节点 max_context_tokens 的作业，MFOP 提供语义分区机制，将输入拆分为语义连贯的子作业，独立路由每个子作业，并聚合结果。

**6.1 分区策略**

MFOP 定义了三种分区策略：

- sliding_window（滑动窗口）：将输入拆分为可配置大小和重叠度的重叠窗口。适用于边界处上下文连续性重要的任务（例如长文档摘要）。
- semantic_boundary（语义边界）：在检测到的语义边界处（段落分隔、章节标题、主题转换）拆分。以子作业大小不均匀为代价，产生更连贯的子作业。
- task_decomposition（任务分解）：将输入解释为结构化任务列表，将每个任务作为独立子作业路由。要求输入符合 MFOP TaskList 模式。

**6.2 分区请求**

提交方通过在作业提交中设置 partition_strategy 来请求分区执行。联邦的分区引擎拆分输入，分配子作业 ID（parent_job_id + 序号），并独立路由每个子作业。子作业继承父作业的合规区域和计费授权。

**6.3 聚合**

所有子作业完成后，联邦的聚合层按序号顺序组装结果。对于 sliding_window 分区，聚合器使用最长公共子序列合并对重叠区域的内容进行去重。组装后的结果作为包含 sub_job_receipts 数组的单个 JobResult 返回给提交方。

## 7. 密码学签名计费收据

每次完成的作业执行均会产生一份由执行节点签名的 BillingReceipt。签名收据是经济结算和争议解决的权威记录。

**7.1 收据结构**

BillingReceipt 包含：

- receipt_id：此收据唯一的 UUID（第 4 版）
- job_id：已完成作业的 UUID
- node_id：执行节点的 UUID
- submitter_id：提交方的 UUID
- model_id：用于执行的模型
- compliance_zone：作业执行所在的合规区域
- input_tokens：处理的输入令牌数
- output_tokens：生成的输出令牌数
- wall_time_ms：总执行时间（毫秒）
- completed_at：作业完成的 RFC 3339 时间戳
- fee_schedule_id：执行时有效的 BillingFeeConfig 的 UUID
- input_token_cost_usd：计算得出的输入令牌费用（美元，6 位小数）
- output_token_cost_usd：计算得出的输出令牌费用（美元，6 位小数）
- platform_fee_usd：平台对本次作业收取的费用
- node_earnings_usd：节点运营商从本次作业获得的收益
- total_cost_usd：提交方的总费用

**7.2 签名方案**

收据使用 Ed25519 签名。节点使用其注册的私钥对收据的规范 JSON 序列化（键名排序，无空白字符）进行签名。签名以 base64url 编码，并作为 signature 字段包含在收据中。

联邦在收到收据时使用节点注册的公钥验证收据签名。签名无效的收据将被拒绝，并触发节点完整性告警。

**7.3 收据存储与检索**

联邦将所有收据至少保存 7 年，以支持合规审计要求。提交方可通过 GET /v1/federation/receipts 检索其收据。节点运营商可通过 GET /v1/federation/nodes/{id}/receipts 检索其执行作业的收据。

## 8. 可配置的经济结算

MFOP 将计费（签名收据的累积）与结算（资金的财务转账）分离。结算是可配置的，可按照不同参与者类型的不同时间表进行。

**8.1 BillingFeeConfig**

平台管理员通过 BillingFeeConfig 对象配置费率。每个 BillingFeeConfig 具有版本标识符和生效日期；联邦应用作业执行时有效的配置。新配置可随时创建，并在下一个计费周期开始时生效。

BillingFeeConfig 字段：

- input_token_rate_usd_per_1k：每 1,000 个输入令牌收取的美元费用
- output_token_rate_usd_per_1k：每 1,000 个输出令牌收取的美元费用
- platform_fee_pct：平台占总令牌费用的百分比（0–100）
- node_revenue_share_pct：节点运营商占总令牌费用的百分比（0–100，与 platform_fee_pct 之和必须 ≤ 100）
- settlement_period_days：结算运行频率（例如 30 天）
- minimum_payout_usd：节点运营商收到付款所需的最低累计收益

**8.2 提交方计费**

提交方按后付费方式计费。在每个结算周期结束时，联邦汇总提交方的所有收据，并向其存档的支付方式收费。发票包含按合规区域和模型分组的作业收据明细列表。

**8.3 节点运营商结算**

节点运营商在每个结算周期结束时通过 Stripe Connect 获得付款，前提是其累计收益超过 minimum_payout_usd 阈值。未达到阈值的运营商将其收益结转至下一周期。

## 9. 安全模型

MFOP 实现三层安全模型：传输安全、AI 安全策略验证和执行沙箱隔离。

**9.1 传输安全**

所有 MFOP API 端点**必须**通过使用 TLS 1.3 或更高版本的 HTTPS 提供服务。在私有企业网格部署中，节点到联邦的通信**推荐**使用双向 TLS（mTLS）。API 认证使用以 X-Channel-API-Key HTTP 头部传输的 PAK Key。PAK Key 是以 base64url 编码的 256 位随机值。

**9.2 AI 安全策略验证**

所有作业的输入和输出在执行前及交付给提交方前均须通过 NeMo Guardrails 策略验证。所有合规区域均须遵守的基线策略集包括：

- 越狱攻击检测与拦截
- 有害内容检测（暴力、儿童性剥削材料、自伤促进）
- 输出中的 PII 泄露检测
- 提示注入检测

特定合规区域需要额外的策略（见附录 B）。

节点**必须**运行其能力通告中指定的 NeMo Guardrails 运行时版本。运行过时 Guardrails 版本的节点将被标记为 DEGRADED，并从需要已安装版本中不具备的护栏功能的合规区域路由中排除。

**9.3 执行沙箱隔离**

每个作业在隔离沙箱中执行。节点**必须**使用以下机制之一实现沙箱隔离：

- gVisor（runsc）— 云部署**推荐**
- Firecracker 微型虚拟机 — 裸机部署**推荐**
- WASM（Wasmtime）— 仅限纯 CPU 推理工作负载

沙箱**必须**在作业之间销毁并重建。持久沙箱状态（例如模型权重）**可以**通过只读挂载在作业间共享，但作业特定状态（上下文、临时文件）**不得**在作业间持久化。

**9.4 审计日志**

所有作业路由决策、收据签名和结算事件均写入仅追加审计日志。审计日志使用 SHA-256 哈希进行密码学链式记录（每个条目包含前一条目的哈希值）。审计日志不得修改，仅允许追加操作。

## 10. 线路协议

MFOP 对所有 API 通信使用基于 HTTPS 的 JSON。WebSocket 连接支持流式作业输出（见第 10.2 节）。

**10.1 请求与响应格式**

所有请求和响应体均为 UTF-8 编码的 JSON。请求**必须**包含 Content-Type: application/json。成功响应使用 HTTP 200 或 201。错误响应使用标准错误信封：

{ "error": { "code": "<机器可读代码>", "message": "<人类可读消息>", "details": { ... } } }

标准错误代码：UNAUTHORIZED、FORBIDDEN、NOT_FOUND、VALIDATION_ERROR、QUOTA_EXCEEDED、NO_ELIGIBLE_NODE、COMPLIANCE_VIOLATION、INTERNAL_ERROR。

**10.2 流式输出**

支持流式输出的节点在 wss://{node_endpoint}/v1/jobs/{id}/stream 暴露 WebSocket 端点。客户端在作业提交后连接。节点以 JSON 帧格式的增量消息流式传输令牌输出：

{ "type": "delta", "text": "...", "token_count": N }

流以完成消息终止：

{ "type": "done", "receipt": { ... } }

完成消息中的 receipt 是该作业的已签名 BillingReceipt。

**10.3 幂等性**

作业提交请求**应该**包含 Idempotency-Key 头部（UUID）。若在幂等性窗口期（24 小时）内收到具有相同 Idempotency-Key 的请求，联邦将返回原始响应而不重新执行作业。这可防止由网络重试导致的重复提交。

## 附录 A. REST API 参考

本附录列出 MFOP REST API 端点。除另有说明外，所有端点均需要 X-Channel-API-Key 头部。基础路径：/v1/federation

| 方法 + 路径 | 名称 | 描述 |
| --- | --- | --- |
| POST /v1/federation/nodes/register | 节点注册 | 向联邦注册新节点。 |
| PUT /v1/federation/nodes/{id}/capabilities | 能力更新 | 更新节点的能力通告。 |
| POST /v1/federation/nodes/{id}/heartbeat | 节点心跳 | 通知节点存活并正在接受作业。 |
| DELETE /v1/federation/nodes/{id} | 节点注销 | 主动注销节点。 |
| POST /v1/federation/jobs | 作业提交 | 向联邦提交作业以供执行。 |
| GET /v1/federation/jobs/{id} | 作业状态 | 获取作业的当前状态和结果。 |
| GET /v1/federation/jobs/{id}/receipt | 作业收据 | 获取已完成作业的已签名计费收据。 |
| GET /v1/federation/receipts | 提交方收据 | 列出已认证提交方的所有收据。 |
| GET /v1/federation/nodes/{id}/receipts | 节点收据 | 列出该节点执行作业的所有收据。 |
| POST /v1/federation/nodes/{id}/stripe/onboard | Stripe Connect 入驻 | 返回用于银行账户设置的 Stripe 托管入驻 URL。 |
| GET /v1/federation/nodes/{id}/earnings | 提供商收益 | 当前周期令牌数、预计收益、上次付款。 |
| GET /v1/federation/submitters/billing | 提交方账单摘要 | 当前周期费用、下次账单日期。 |
| PATCH /v1/admin/federation/billing-config | 更新费率模型 | 仅限管理员。创建新的 BillingFeeConfig 行，下一周期生效。 |

## 附录 B. 合规区域策略要求

每个合规区域在基线之外要求特定的 NeMo Guardrails 策略能力。下表汇总了每个区域所需的最低护栏配置。

| 区域 | 基线之外所需护栏 |
| --- | --- |
| public | 仅基线，无需额外护栏。 |
| enterprise (SOC2) | 数据驻留标记检测。API 凭证泄露检测。访问日志强制执行。 |
| hipaa | PHI 模式检测：患者姓名、出生日期、MRN、ICD-10 代码、诊断描述、医疗保险 ID。PHI 去标识化护栏：在调用 AI 模型前对 PHI 进行脱敏或哈希处理。对输出进行最小必要性检查。 |
| sox | 金融 PII 隔离：账户号码、路由号码、税务 ID。价格预测拦截：前瞻性收益或价格陈述。MNPI 检测：重大非公开信息模式匹配。 |
| fedramp | CUI 处理：受控非密信息标记检测与处理规则。出口管制：EAR/ITAR 涉及主题检测。分类标记强制执行：拦截含有分类标记的输出。 |

## 致谢

作者谨向 NVIDIA NeMo 团队致谢，感谢其开发的 NeMo Guardrails 和 NemoClaw OpenShell 平台，这些平台为本规范所引用的基础安全基础设施提供了支撑。MFOP 安全模型的设计旨在随这些平台的成熟持续演进。

本规范中描述的三层安全模型、合规区域分类体系、Ed25519 收据签名方案和可配置计费架构，是 Thrive Tech Services LLC 在 2026 年初经过广泛设计和评审过程开发并完善的成果。

本规范谨献给全球知识工作者群体——包括法律、医疗、研究、金融和技术领域的从业者——正是他们的工作，使联邦式 AI 编排成为一项有意义的事业。

MFOP 规范 1.0 版本完 — 同行评审草稿
Thrive Tech Services LLC · Ami Hoepner Nuñez · 2026年3月

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · 2026年3月*
