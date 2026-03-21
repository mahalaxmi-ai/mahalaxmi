# Mahalaxmi Federation and Orchestration Protocol

**MFOP v1.0** · Draft for Peer Review

| | |
|---|---|
| Date | March 2026 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Florida, USA |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## Status of This Memo

This document is a pre-publication draft of the Mahalaxmi Federation and Orchestration Protocol (MFOP) specification, version 1.0. It is distributed for peer review and to solicit comments. This document describes a protocol for federated distributed AI orchestration across heterogeneous compute nodes with compliance-zone-aware routing, cryptographically signed billing receipts, and configurable economic settlement.

Comments and questions should be directed to the author at Ami.nunez@mahalaxmi.ai. The current draft and discussion threads are maintained at https://mahalaxmi.ai/mfop/draft. Discussion threads are at https://mahalaxmi.ai/mfop/discuss.

## Copyright Notice

Copyright © 2026 ThriveTech Services LLC. All rights reserved. Permission is granted to copy, distribute, and use this document in any medium without fee, provided that the author attribution, document title, and this copyright notice are preserved in all copies and derivative works.

## Abstract

This document defines the Mahalaxmi Federation and Orchestration Protocol (MFOP), a protocol for coordinating parallel AI agent execution across a distributed network of heterogeneous compute nodes. MFOP specifies node identity and registration, capability advertisement, compliance-zone-aware job routing, semantic input partitioning, cryptographically signed billing receipts, configurable economic settlement, and a layered security model using AI safety policy validation and execution sandbox isolation.

MFOP is designed to operate in three simultaneous deployment configurations: private enterprise meshes where nodes are owned and operated by a single organization, managed cloud pools operated by the platform provider, and open community marketplaces where any node operator may contribute compute in exchange for economic settlement. The protocol is agnostic to the underlying AI model provider and is designed to evolve with the AI safety and compliance landscape.

## 1. Introduction

The growth of large language model (LLM) deployments across enterprise environments has created a need for a coordination layer that can span heterogeneous compute infrastructure while satisfying compliance, billing, and safety requirements that vary by jurisdiction and industry.

MFOP addresses this need by defining a protocol for federated AI orchestration. A federation consists of one or more compute nodes, each of which may be operated by different entities under different compliance regimes. A submitter — a user, application, or automated system — presents a job to the federation. The federation routes the job to an appropriate node based on the job's compliance zone requirements, the node's capability advertisement, and the economic terms in effect.

This specification defines the wire protocol, data formats, cryptographic mechanisms, and behavioral requirements for all components of a conforming MFOP federation.

## 2. Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174].

**Federation** — A logical grouping of one or more MFOP-conformant compute nodes operating under a shared governance configuration.

**Node** — A compute resource registered with a federation that accepts, executes, and returns AI workloads. A node may be a single server, a cluster, or a cloud compute pool.

**Submitter** — An entity (user, application, or automated system) that presents AI workloads to the federation for execution.

**Compliance Zone** — A named policy context that constrains job routing, data handling, and output validation. Defined zones: public, enterprise (SOC2), hipaa, sox, fedramp.

**Job** — A discrete unit of AI workload submitted to the federation for execution. A job carries a payload, compliance zone assertion, and billing authorization.

**Receipt** — A cryptographically signed record of a completed job execution, including token counts, timestamps, node identity, and billing amounts.

**Economic Settlement** — The process by which accumulated billing receipts are converted into financial transfers between submitters, node operators, and the platform.

**PAK Key (Platform API Key)** — A bearer credential issued by the platform that authorizes access to federation API endpoints.

**NeMo Guardrails** — The NVIDIA NeMo safety framework used by MFOP nodes for AI safety policy validation and output filtering.

## 3. Node Identity and Registration

Each node in a MFOP federation is identified by a stable, globally unique node identifier (node_id). The node_id is a 128-bit UUID (version 4) assigned at registration time and persists across node restarts and software upgrades.

**3.1 Registration Flow**

A node initiates registration by sending a NodeRegistrationRequest to the federation's registration endpoint (POST /v1/federation/nodes/register). The request MUST include:

- node_id: a candidate UUID (the federation MAY override this)
- operator_id: the UUID of the registering operator account
- display_name: a human-readable name for the node (max 64 characters)
- public_key: an Ed25519 public key in base64url encoding, used for receipt signing
- capability_advertisement: a CapabilityAdvertisement object (see Section 4)
- compliance_zones: the set of compliance zones the node is certified to handle
- endpoint_url: the HTTPS URL at which the node accepts job submissions

The federation returns a NodeRegistrationResponse containing the assigned node_id, a registration_token for subsequent authenticated calls, and the federation's current billing configuration.

**3.2 Re-registration and Key Rotation**

Nodes MUST re-register when their Ed25519 key pair is rotated. During key rotation, the node submits a re-registration request with both the old and new public keys, signed with the old private key. The federation verifies the old-key signature before accepting the new key. There is a 24-hour overlap window during which receipts signed with either key are accepted.

**3.3 Node Health and Deregistration**

Nodes MUST send a heartbeat to POST /v1/federation/nodes/{id}/heartbeat at least once every 60 seconds. A node that misses three consecutive heartbeat windows is marked INACTIVE and excluded from routing. Nodes may deregister voluntarily via DELETE /v1/federation/nodes/{id}.

## 4. Capability Advertisement

A node's capability advertisement declares the AI models available on the node, the hardware characteristics relevant to job routing, and the compliance certifications held by the node operator.

**4.1 CapabilityAdvertisement Object**

The CapabilityAdvertisement object includes the following fields:

- models: an array of ModelDescriptor objects (see 4.2)
- hardware_class: one of { cpu, gpu_consumer, gpu_datacenter, tpu }
- vram_gb: total GPU VRAM available for inference, in gigabytes (0 for CPU nodes)
- max_context_tokens: the maximum context window the node can service
- max_concurrent_jobs: the maximum number of jobs the node will execute simultaneously
- compliance_certifications: an array of certification identifiers (e.g., "soc2-type2", "hipaa-baa", "fedramp-moderate")
- nemo_rails_version: the version of the NeMo Guardrails runtime installed on the node

**4.2 ModelDescriptor**

Each model available on a node is described by a ModelDescriptor:

- model_id: a canonical model identifier string (e.g., "meta-llama/Meta-Llama-3-70B-Instruct")
- model_family: one of { llama, mistral, gemma, falcon, phi, custom }
- parameter_count_b: approximate parameter count in billions
- quantization: one of { fp16, bf16, int8, int4, none }
- context_window_tokens: the maximum context window for this model
- supports_tool_use: boolean
- supports_vision: boolean

**4.3 Capability Refresh**

Nodes MUST update their capability advertisement via PUT /v1/federation/nodes/{id}/capabilities whenever their available models or hardware configuration changes. The federation propagates updated capability advertisements to the routing layer within 30 seconds.

## 5. Compliance-Zone-Aware Job Routing

MFOP routes each job to a node that satisfies the job's compliance zone requirements. Compliance zone satisfaction is a hard constraint: a job MUST NOT be routed to a node that is not certified for the job's compliance zone.

**5.1 Compliance Zones**

MFOP defines five compliance zones, ordered from least to most restrictive:

- public: No compliance requirements beyond the baseline NeMo safety rails. Suitable for general-purpose AI workloads.
- enterprise (SOC2): Requires SOC 2 Type II certification. Adds data residency detection, API credential exfiltration detection, and access logging enforcement.
- hipaa: Requires HIPAA BAA. Adds PHI pattern detection, PHI de-identification, and minimum-necessary output checks.
- sox: Requires SOX compliance controls. Adds financial PII isolation, price prediction blocking, and MNPI detection.
- fedramp: Requires FedRAMP authorization. Adds CUI handling, export control detection, and classification marking enforcement.

**5.2 Routing Algorithm**

When a job is received, the routing layer executes the following algorithm:

1. Filter: Identify all nodes with status ACTIVE that are certified for the job's compliance zone.
2. Filter: Remove nodes whose max_context_tokens is less than the job's estimated token count.
3. Filter: Remove nodes whose max_concurrent_jobs is currently exhausted.
4. Score: For each remaining node, compute a routing score: score = w_latency × latency_score + w_cost × cost_score + w_affinity × affinity_score. Default weights: w_latency = 0.4, w_cost = 0.4, w_affinity = 0.2.
5. Select: Route to the highest-scoring node. On tie, select uniformly at random.

If no node satisfies all filters, the job is queued with a configurable timeout (default: 120 seconds). If no node becomes available within the timeout, the federation returns HTTP 503 with a Retry-After header.

**5.3 Affinity Rules**

Submitters MAY specify affinity rules in their job submission:

- node_affinity: a list of preferred node_ids (soft preference)
- anti_affinity: a list of node_ids to exclude (hard constraint)
- geography: a preferred geographic region (ISO 3166-1 alpha-2 country code)

Affinity rules affect only the affinity_score component; compliance zone certification and capacity remain hard constraints.

## 6. Semantic Input Partitioning

For jobs whose input exceeds a single node's max_context_tokens, MFOP provides a semantic partitioning mechanism that splits the input into coherent sub-jobs, routes each sub-job independently, and aggregates the results.

**6.1 Partition Strategies**

MFOP defines three partition strategies:

- sliding_window: Splits the input into overlapping windows of configurable size and overlap. Suitable for tasks where context continuity at boundaries is important (e.g., long-document summarization).
- semantic_boundary: Splits at detected semantic boundaries (paragraph breaks, section headers, topic transitions). Produces more coherent sub-jobs at the cost of variable sub-job sizes.
- task_decomposition: Interprets the input as a structured task list and routes each task as an independent sub-job. Requires the input to conform to the MFOP TaskList schema.

**6.2 Partition Request**

A submitter requests partitioned execution by setting partition_strategy in the job submission. The federation's partition engine splits the input, assigns sub-job IDs (parent_job_id + sequence number), and routes each sub-job independently. Sub-jobs inherit the compliance zone and billing authorization of the parent job.

**6.3 Aggregation**

Once all sub-jobs complete, the federation's aggregation layer assembles the results in sequence-number order. For sliding_window partitions, the aggregator de-duplicates content in the overlap regions using a longest-common-subsequence merge. The assembled result is returned to the submitter as a single JobResult with an array of sub_job_receipts.

## 7. Cryptographically Signed Billing Receipts

Every completed job execution produces a BillingReceipt signed by the executing node. Signed receipts are the authoritative record for economic settlement and dispute resolution.

**7.1 Receipt Structure**

A BillingReceipt contains:

- receipt_id: a UUID (version 4) unique to this receipt
- job_id: the UUID of the completed job
- node_id: the UUID of the executing node
- submitter_id: the UUID of the submitter
- model_id: the model used for execution
- compliance_zone: the compliance zone under which the job executed
- input_tokens: the number of input tokens processed
- output_tokens: the number of output tokens generated
- wall_time_ms: total execution time in milliseconds
- completed_at: RFC 3339 timestamp of job completion
- fee_schedule_id: the UUID of the BillingFeeConfig in effect at execution time
- input_token_cost_usd: computed input token cost in USD (6 decimal places)
- output_token_cost_usd: computed output token cost in USD (6 decimal places)
- platform_fee_usd: the platform's fee for this job
- node_earnings_usd: the node operator's earnings for this job
- total_cost_usd: total cost to the submitter

**7.2 Signature Scheme**

Receipts are signed using Ed25519. The node signs the canonical JSON serialization of the receipt (keys sorted, no whitespace) with its registered private key. The signature is base64url-encoded and included in the receipt as the signature field.

The federation verifies the receipt signature upon receipt using the node's registered public key. Receipts with invalid signatures are rejected and trigger a node integrity alert.

**7.3 Receipt Storage and Retrieval**

The federation stores all receipts for a minimum of 7 years to support compliance audit requirements. Submitters may retrieve their receipts via GET /v1/federation/receipts. Node operators may retrieve receipts for jobs they executed via GET /v1/federation/nodes/{id}/receipts.

## 8. Configurable Economic Settlement

MFOP separates billing (the accumulation of signed receipts) from settlement (the financial transfer of funds). Settlement is configurable and may occur on different schedules for different participant types.

**8.1 BillingFeeConfig**

The platform administrator configures fee rates via a BillingFeeConfig object. Each BillingFeeConfig has a version identifier and an effective date; the federation applies the config in effect at the time of job execution. A new config may be created at any time; it takes effect at the start of the next billing period.

BillingFeeConfig fields:

- input_token_rate_usd_per_1k: USD charged per 1,000 input tokens
- output_token_rate_usd_per_1k: USD charged per 1,000 output tokens
- platform_fee_pct: the platform's percentage of total token cost (0–100)
- node_revenue_share_pct: the node operator's percentage of total token cost (0–100, must sum to ≤ 100 with platform_fee_pct)
- settlement_period_days: how often settlement runs (e.g., 30)
- minimum_payout_usd: minimum accumulated earnings before a node operator receives a payout

**8.2 Submitter Billing**

Submitters are billed on a postpay basis. At the end of each settlement period, the federation aggregates all receipts for the submitter and charges the payment method on file. The invoice includes an itemized list of job receipts, grouped by compliance zone and model.

**8.3 Node Operator Settlement**

Node operators are paid out via Stripe Connect at the end of each settlement period, provided their accumulated earnings exceed the minimum_payout_usd threshold. Operators who do not meet the threshold roll their earnings forward to the next period.

## 9. Security Model

MFOP implements a three-layer security model: transport security, AI safety policy validation, and execution sandbox isolation.

**9.1 Transport Security**

All MFOP API endpoints MUST be served over HTTPS using TLS 1.3 or higher. Mutual TLS (mTLS) is RECOMMENDED for node-to-federation communication in private enterprise mesh deployments. API authentication uses PAK Keys transmitted as the X-Channel-API-Key HTTP header. PAK Keys are 256-bit random values encoded in base64url.

**9.2 AI Safety Policy Validation**

All job inputs and outputs are validated against NeMo Guardrails policies before execution and before delivery to the submitter. The baseline policy set (required for all compliance zones) includes:

- Jailbreak detection and blocking
- Harmful content detection (violence, CSAM, self-harm facilitation)
- PII leakage detection in outputs
- Prompt injection detection

Additional policies are required for specific compliance zones (see Appendix B).

Nodes MUST run the NeMo Guardrails runtime version specified in their capability advertisement. Nodes running outdated Guardrails versions are flagged as DEGRADED and excluded from routing for compliance zones that require guardrails features not present in the installed version.

**9.3 Execution Sandbox Isolation**

Each job executes in an isolated sandbox. Nodes MUST implement sandbox isolation using one of the following mechanisms:

- gVisor (runsc) — RECOMMENDED for cloud deployments
- Firecracker microVMs — RECOMMENDED for bare-metal deployments
- WASM (Wasmtime) — Permitted for CPU-only inference workloads

Sandboxes MUST be destroyed and recreated between jobs. Persistent sandbox state (e.g., model weights) may be shared across jobs via a read-only mount, but job-specific state (context, temporary files) MUST NOT persist between jobs.

**9.4 Audit Logging**

All job routing decisions, receipt signatures, and settlement events are written to an append-only audit log. The audit log is cryptographically chained using SHA-256 hashes (each entry includes the hash of the previous entry). The audit log may not be modified; only append operations are permitted.

## 10. Wire Protocol

MFOP uses JSON over HTTPS for all API communication. WebSocket connections are supported for streaming job output (see Section 10.2).

**10.1 Request and Response Format**

All request and response bodies are UTF-8 encoded JSON. Requests MUST include Content-Type: application/json. Successful responses use HTTP 200 or 201. Error responses use the standard error envelope:

{ "error": { "code": "<machine-readable-code>", "message": "<human-readable-message>", "details": { ... } } }

Standard error codes: UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, QUOTA_EXCEEDED, NO_ELIGIBLE_NODE, COMPLIANCE_VIOLATION, INTERNAL_ERROR.

**10.2 Streaming Output**

Nodes that support streaming output expose a WebSocket endpoint at wss://{node_endpoint}/v1/jobs/{id}/stream. The client connects after job submission. The node streams token output as JSON-framed delta messages:

{ "type": "delta", "text": "...", "token_count": N }

The stream is terminated with a completion message:

{ "type": "done", "receipt": { ... } }

The receipt in the completion message is the signed BillingReceipt for the job.

**10.3 Idempotency**

Job submission requests SHOULD include an Idempotency-Key header (UUID). If a request with the same Idempotency-Key is received within the idempotency window (24 hours), the federation returns the original response without re-executing the job. This protects against duplicate submissions caused by network retries.

## Appendix A. REST API Reference

This appendix lists the MFOP REST API endpoints. All endpoints require an X-Channel-API-Key header unless noted otherwise. Base path: /v1/federation

| Method + Path | Name | Description |
| --- | --- | --- |
| POST /v1/federation/nodes/register | Node registration | Register a new node with the federation. |
| PUT /v1/federation/nodes/{id}/capabilities | Capability update | Update a node's capability advertisement. |
| POST /v1/federation/nodes/{id}/heartbeat | Node heartbeat | Signal that the node is alive and accepting jobs. |
| DELETE /v1/federation/nodes/{id} | Node deregistration | Voluntarily deregister a node. |
| POST /v1/federation/jobs | Job submission | Submit a job to the federation for execution. |
| GET /v1/federation/jobs/{id} | Job status | Retrieve the current status and result of a job. |
| GET /v1/federation/jobs/{id}/receipt | Job receipt | Retrieve the signed billing receipt for a completed job. |
| GET /v1/federation/receipts | Submitter receipts | List all receipts for the authenticated submitter. |
| GET /v1/federation/nodes/{id}/receipts | Node receipts | List all receipts for jobs executed by the node. |
| POST /v1/federation/nodes/{id}/stripe/onboard | Stripe Connect onboarding | Returns Stripe-hosted onboarding URL for bank account setup. |
| GET /v1/federation/nodes/{id}/earnings | Provider earnings | Current period tokens, estimated earnings, last payout. |
| GET /v1/federation/submitters/billing | Submitter billing summary | Current period cost, next billing date. |
| PATCH /v1/admin/federation/billing-config | Update fee model | Admin only. Creates new BillingFeeConfig row. Effective next period. |

## Appendix B. Compliance Zone Policy Requirements

Each compliance zone requires specific NeMo Guardrails policy capabilities beyond the baseline. The following table summarizes the minimum required rails per zone.

| Zone | Required Rails Beyond Baseline |
| --- | --- |
| public | Baseline only. No additional rails required. |
| enterprise (SOC2) | Data residency marker detection. API credential exfiltration detection. Access logging enforcement. |
| hipaa | PHI pattern detection: patient names, dates of birth, MRN, ICD-10 codes, diagnosis descriptions, health insurance IDs. PHI de-identification rail: strip or hash PHI before AI model invocation. Minimum necessary check on outputs. |
| sox | Financial PII isolation: account numbers, routing numbers, tax IDs. Price prediction block: forward-looking return or price statements. MNPI detection: material non-public information pattern matching. |
| fedramp | CUI handling: Controlled Unclassified Information marker detection and handling rules. Export control: EAR/ITAR subject matter detection. Classification marking enforcement: block outputs containing classification markings. |

## Acknowledgements

The author wishes to acknowledge the NVIDIA NeMo team for the NeMo Guardrails and NemoClaw OpenShell platforms, which provide the foundational security infrastructure referenced in this specification. The MFOP security model is designed to evolve with these platforms as they mature.

The three-layer security model, compliance zone taxonomy, Ed25519 receipt signing scheme, and configurable billing architecture described in this specification were developed and refined through an extensive design and review process conducted at Thrive Tech Services LLC in early 2026.

This specification is dedicated to the global community of knowledge workers — in legal, healthcare, research, financial, and technical disciplines — whose work is the reason federated AI orchestration matters.

End of MFOP Specification Version 1.0 — Draft for Peer Review
Thrive Tech Services LLC · Ami Hoepner Nuñez · March 2026

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · March 2026*
