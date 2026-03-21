# Mahalaxmi 페더레이션 및 오케스트레이션 프로토콜

**MFOP v1.0** · 동료 검토를 위한 초안

| | |
|---|---|
| Date | 2026년 3월 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Florida, USA |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## 이 메모의 상태

이 문서는 Mahalaxmi 페더레이션 및 오케스트레이션 프로토콜(MFOP) 사양 버전 1.0의 출판 전 초안입니다. 동료 검토 및 의견 수렴을 위해 배포됩니다. 이 문서는 컴플라이언스 구역 인식 라우팅, 암호학적으로 서명된 청구 영수증, 구성 가능한 경제적 정산을 갖춘 이기종 컴퓨트 노드 전반에 걸친 페더레이션 분산 AI 오케스트레이션 프로토콜을 설명합니다.

의견과 질문은 Ami.nunez@mahalaxmi.ai의 저자에게 직접 보내주십시오. 현재 초안과 토론 스레드는 https://mahalaxmi.ai/mfop/draft 에서 유지 관리됩니다. 토론 스레드는 https://mahalaxmi.ai/mfop/discuss 에 있습니다.

## 저작권 고지

Copyright © 2026 ThriveTech Services LLC. All rights reserved. 저자 표시, 문서 제목 및 이 저작권 고지가 모든 사본 및 파생 저작물에 보존되는 조건 하에, 어떠한 매체에서도 이 문서를 무료로 복사, 배포 및 사용할 수 있는 권한이 부여됩니다.

## 초록

이 문서는 이기종 컴퓨트 노드의 분산 네트워크 전반에서 병렬 AI 에이전트 실행을 조율하기 위한 프로토콜인 Mahalaxmi 페더레이션 및 오케스트레이션 프로토콜(MFOP)을 정의합니다. MFOP는 노드 신원 및 등록, 능력 광고, 컴플라이언스 구역 인식 작업 라우팅, 의미론적 입력 분할, 암호학적으로 서명된 청구 영수증, 구성 가능한 경제적 정산, 그리고 AI 안전 정책 검증 및 실행 샌드박스 격리를 사용하는 계층화된 보안 모델을 명시합니다.

MFOP는 세 가지 동시 배포 구성으로 운영되도록 설계되었습니다: 단일 조직이 노드를 소유하고 운영하는 프라이빗 엔터프라이즈 메시, 플랫폼 제공업체가 운영하는 관리형 클라우드 풀, 그리고 모든 노드 운영자가 경제적 정산의 대가로 컴퓨트를 기여할 수 있는 오픈 커뮤니티 마켓플레이스. 이 프로토콜은 기반 AI 모델 제공업체에 구애받지 않으며, AI 안전 및 컴플라이언스 환경의 발전에 따라 진화하도록 설계되었습니다.

## 1. 소개

엔터프라이즈 환경 전반에서 대규모 언어 모델(LLM) 배포의 성장으로 인해, 관할권 및 산업에 따라 다양한 컴플라이언스, 청구, 안전 요구사항을 충족하면서 이기종 컴퓨트 인프라에 걸쳐 작동할 수 있는 조율 레이어의 필요성이 생겨났습니다.

MFOP는 페더레이션 AI 오케스트레이션을 위한 프로토콜을 정의함으로써 이 필요성을 해결합니다. 페더레이션은 하나 이상의 컴퓨트 노드로 구성되며, 각 노드는 서로 다른 컴플라이언스 체계 하에 서로 다른 주체에 의해 운영될 수 있습니다. 제출자(사용자, 애플리케이션 또는 자동화 시스템)는 페더레이션에 작업을 제출합니다. 페더레이션은 작업의 컴플라이언스 구역 요구사항, 노드의 능력 광고, 그리고 현재 적용되는 경제적 조건에 따라 적합한 노드로 작업을 라우팅합니다.

이 사양은 MFOP 페더레이션의 모든 구성 요소에 대한 와이어 프로토콜, 데이터 형식, 암호화 메커니즘, 그리고 동작 요구사항을 정의합니다.

## 2. 용어

이 문서에서 "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", "OPTIONAL"이라는 핵심 용어는 BCP 14 [RFC2119] [RFC8174]에 설명된 바에 따라 해석되어야 합니다. 한국어로는 각각 "반드시 ~해야 한다", "~해서는 안 된다", "필수", "~해야 한다", "~해서는 안 된다", "~하는 것이 좋다", "~하지 않는 것이 좋다", "권장됨", "권장되지 않음", "~해도 된다", "선택 사항"으로 해석됩니다.

**페더레이션(Federation)** — 공유 거버넌스 구성 하에 운영되는 하나 이상의 MFOP 준수 컴퓨트 노드의 논리적 그룹.

**노드(Node)** — 페더레이션에 등록되어 AI 워크로드를 수락, 실행 및 반환하는 컴퓨트 리소스. 노드는 단일 서버, 클러스터, 또는 클라우드 컴퓨트 풀일 수 있습니다.

**제출자(Submitter)** — 실행을 위해 페더레이션에 AI 워크로드를 제출하는 주체(사용자, 애플리케이션 또는 자동화 시스템).

**컴플라이언스 구역(Compliance Zone)** — 작업 라우팅, 데이터 처리 및 출력 검증을 제한하는 명명된 정책 컨텍스트. 정의된 구역: public, enterprise (SOC2), hipaa, sox, fedramp.

**작업(Job)** — 실행을 위해 페더레이션에 제출되는 AI 워크로드의 개별 단위. 작업에는 페이로드, 컴플라이언스 구역 주장, 청구 인증이 포함됩니다.

**영수증(Receipt)** — 토큰 수, 타임스탬프, 노드 신원, 청구 금액을 포함하여 완료된 작업 실행의 암호학적으로 서명된 기록.

**경제적 정산(Economic Settlement)** — 누적된 청구 영수증을 제출자, 노드 운영자, 플랫폼 간의 재정적 이체로 전환하는 프로세스.

**PAK Key (플랫폼 API 키)** — 페더레이션 API 엔드포인트에 대한 접근을 인증하는 플랫폼이 발행한 베어러 자격 증명.

**NeMo Guardrails** — MFOP 노드가 AI 안전 정책 검증 및 출력 필터링에 사용하는 NVIDIA NeMo 안전 프레임워크.

## 3. 노드 신원 및 등록

MFOP 페더레이션의 각 노드는 안정적이고 전역적으로 고유한 노드 식별자(node_id)로 식별됩니다. node_id는 등록 시 할당되는 128비트 UUID(버전 4)이며, 노드 재시작 및 소프트웨어 업그레이드 전반에 걸쳐 유지됩니다.

**3.1 등록 흐름**

노드는 페더레이션의 등록 엔드포인트(POST /v1/federation/nodes/register)에 NodeRegistrationRequest를 전송하여 등록을 시작합니다. 요청에는 반드시 다음이 포함되어야 합니다:

- node_id: 후보 UUID (페더레이션이 이를 재정의해도 된다)
- operator_id: 등록하는 운영자 계정의 UUID
- display_name: 노드의 사람이 읽을 수 있는 이름 (최대 64자)
- public_key: 영수증 서명에 사용되는 base64url 인코딩의 Ed25519 공개 키
- capability_advertisement: CapabilityAdvertisement 객체 (섹션 4 참조)
- compliance_zones: 노드가 처리 인증을 받은 컴플라이언스 구역 집합
- endpoint_url: 노드가 작업 제출을 수락하는 HTTPS URL

페더레이션은 할당된 node_id, 이후 인증된 호출을 위한 registration_token, 그리고 페더레이션의 현재 청구 구성을 포함하는 NodeRegistrationResponse를 반환합니다.

**3.2 재등록 및 키 순환**

노드는 Ed25519 키 쌍이 순환될 때 반드시 재등록해야 합니다. 키 순환 중에 노드는 이전 개인 키로 서명된 이전 및 새 공개 키를 모두 포함한 재등록 요청을 제출합니다. 페더레이션은 새 키를 수락하기 전에 이전 키 서명을 검증합니다. 어느 키로 서명된 영수증이든 수락되는 24시간 중복 창이 있습니다.

**3.3 노드 상태 확인 및 등록 해제**

노드는 반드시 최소 60초마다 POST /v1/federation/nodes/{id}/heartbeat에 하트비트를 전송해야 합니다. 세 번의 연속 하트비트 창을 놓친 노드는 INACTIVE로 표시되어 라우팅에서 제외됩니다. 노드는 DELETE /v1/federation/nodes/{id}를 통해 자발적으로 등록을 해제할 수 있습니다.

## 4. 능력 광고

노드의 능력 광고는 해당 노드에서 사용 가능한 AI 모델, 작업 라우팅과 관련된 하드웨어 특성, 그리고 노드 운영자가 보유한 컴플라이언스 인증을 선언합니다.

**4.1 CapabilityAdvertisement 객체**

CapabilityAdvertisement 객체에는 다음 필드가 포함됩니다:

- models: ModelDescriptor 객체의 배열 (4.2 참조)
- hardware_class: { cpu, gpu_consumer, gpu_datacenter, tpu } 중 하나
- vram_gb: 추론에 사용 가능한 총 GPU VRAM (기가바이트 단위, CPU 노드는 0)
- max_context_tokens: 노드가 처리할 수 있는 최대 컨텍스트 창
- max_concurrent_jobs: 노드가 동시에 실행할 최대 작업 수
- compliance_certifications: 인증 식별자 배열 (예: "soc2-type2", "hipaa-baa", "fedramp-moderate")
- nemo_rails_version: 노드에 설치된 NeMo Guardrails 런타임 버전

**4.2 ModelDescriptor**

노드에서 사용 가능한 각 모델은 ModelDescriptor로 설명됩니다:

- model_id: 정규 모델 식별자 문자열 (예: "meta-llama/Meta-Llama-3-70B-Instruct")
- model_family: { llama, mistral, gemma, falcon, phi, custom } 중 하나
- parameter_count_b: 십억 단위의 대략적인 파라미터 수
- quantization: { fp16, bf16, int8, int4, none } 중 하나
- context_window_tokens: 이 모델의 최대 컨텍스트 창
- supports_tool_use: 불리언
- supports_vision: 불리언

**4.3 능력 갱신**

노드는 사용 가능한 모델이나 하드웨어 구성이 변경될 때마다 반드시 PUT /v1/federation/nodes/{id}/capabilities를 통해 능력 광고를 업데이트해야 합니다. 페더레이션은 업데이트된 능력 광고를 30초 이내에 라우팅 레이어에 전파합니다.

## 5. 컴플라이언스 구역 인식 작업 라우팅

MFOP는 각 작업을 해당 작업의 컴플라이언스 구역 요구사항을 충족하는 노드로 라우팅합니다. 컴플라이언스 구역 충족은 하드 제약 조건입니다: 작업은 해당 작업의 컴플라이언스 구역에 대해 인증받지 않은 노드로 라우팅되어서는 안 됩니다.

**5.1 컴플라이언스 구역**

MFOP는 가장 낮은 제한에서 가장 높은 제한 순으로 다섯 가지 컴플라이언스 구역을 정의합니다:

- public: 기준 NeMo 안전 레일 이외의 컴플라이언스 요구사항 없음. 범용 AI 워크로드에 적합.
- enterprise (SOC2): SOC 2 Type II 인증 필요. 데이터 레지던시 감지, API 자격 증명 유출 감지, 접근 로깅 강제 적용이 추가됨.
- hipaa: HIPAA BAA 필요. PHI 패턴 감지, PHI 비식별화, 최소 필요 출력 확인이 추가됨.
- sox: SOX 컴플라이언스 통제 필요. 금융 PII 격리, 가격 예측 차단, MNPI 감지가 추가됨.
- fedramp: FedRAMP 인증 필요. CUI 처리, 수출 통제 감지, 분류 표시 강제 적용이 추가됨.

**5.2 라우팅 알고리즘**

작업이 수신되면, 라우팅 레이어는 다음 알고리즘을 실행합니다:

1. 필터링: 상태가 ACTIVE이고 작업의 컴플라이언스 구역에 대해 인증된 모든 노드를 식별합니다.
2. 필터링: max_context_tokens이 작업의 예상 토큰 수보다 적은 노드를 제거합니다.
3. 필터링: max_concurrent_jobs가 현재 소진된 노드를 제거합니다.
4. 점수 계산: 남은 각 노드에 대해 라우팅 점수를 계산합니다: score = w_latency × latency_score + w_cost × cost_score + w_affinity × affinity_score. 기본 가중치: w_latency = 0.4, w_cost = 0.4, w_affinity = 0.2.
5. 선택: 가장 높은 점수의 노드로 라우팅합니다. 동점인 경우 균일하게 무작위로 선택합니다.

모든 필터를 충족하는 노드가 없는 경우, 작업은 구성 가능한 타임아웃(기본값: 120초)으로 대기열에 추가됩니다. 타임아웃 내에 노드가 사용 가능해지지 않으면, 페더레이션은 Retry-After 헤더와 함께 HTTP 503을 반환합니다.

**5.3 어피니티 규칙**

제출자는 작업 제출 시 어피니티 규칙을 지정해도 됩니다:

- node_affinity: 선호하는 node_id 목록 (소프트 선호)
- anti_affinity: 제외할 node_id 목록 (하드 제약)
- geography: 선호하는 지리적 지역 (ISO 3166-1 alpha-2 국가 코드)

어피니티 규칙은 affinity_score 구성 요소에만 영향을 미치며, 컴플라이언스 구역 인증 및 용량은 하드 제약 조건으로 유지됩니다.

## 6. 의미론적 입력 분할

입력이 단일 노드의 max_context_tokens을 초과하는 작업의 경우, MFOP는 입력을 일관된 서브 작업으로 분할하고, 각 서브 작업을 독립적으로 라우팅하며, 결과를 집계하는 의미론적 분할 메커니즘을 제공합니다.

**6.1 분할 전략**

MFOP는 세 가지 분할 전략을 정의합니다:

- sliding_window: 입력을 구성 가능한 크기와 겹침으로 중첩되는 창으로 분할합니다. 경계에서 컨텍스트 연속성이 중요한 작업(예: 장문 문서 요약)에 적합합니다.
- semantic_boundary: 감지된 의미론적 경계(단락 구분, 섹션 헤더, 주제 전환)에서 분할합니다. 가변 서브 작업 크기를 감수하고 더 일관된 서브 작업을 생성합니다.
- task_decomposition: 입력을 구조화된 작업 목록으로 해석하고 각 작업을 독립적인 서브 작업으로 라우팅합니다. 입력이 MFOP TaskList 스키마를 준수해야 합니다.

**6.2 분할 요청**

제출자는 작업 제출 시 partition_strategy를 설정하여 분할 실행을 요청합니다. 페더레이션의 분할 엔진은 입력을 분할하고, 서브 작업 ID(parent_job_id + 일련 번호)를 할당하며, 각 서브 작업을 독립적으로 라우팅합니다. 서브 작업은 상위 작업의 컴플라이언스 구역 및 청구 인증을 상속받습니다.

**6.3 집계**

모든 서브 작업이 완료되면, 페더레이션의 집계 레이어는 일련 번호 순으로 결과를 조합합니다. sliding_window 분할의 경우, 집계자는 최장 공통 부분 수열 병합을 사용하여 겹침 영역의 콘텐츠를 중복 제거합니다. 조합된 결과는 sub_job_receipts 배열과 함께 단일 JobResult로 제출자에게 반환됩니다.

## 7. 암호학적으로 서명된 청구 영수증

모든 완료된 작업 실행은 실행 노드가 서명한 BillingReceipt를 생성합니다. 서명된 영수증은 경제적 정산 및 분쟁 해결을 위한 권위 있는 기록입니다.

**7.1 영수증 구조**

BillingReceipt에는 다음이 포함됩니다:

- receipt_id: 이 영수증에 고유한 UUID(버전 4)
- job_id: 완료된 작업의 UUID
- node_id: 실행 노드의 UUID
- submitter_id: 제출자의 UUID
- model_id: 실행에 사용된 모델
- compliance_zone: 작업이 실행된 컴플라이언스 구역
- input_tokens: 처리된 입력 토큰 수
- output_tokens: 생성된 출력 토큰 수
- wall_time_ms: 밀리초 단위의 총 실행 시간
- completed_at: 작업 완료의 RFC 3339 타임스탬프
- fee_schedule_id: 실행 시 적용된 BillingFeeConfig의 UUID
- input_token_cost_usd: 계산된 입력 토큰 비용 (USD, 소수점 6자리)
- output_token_cost_usd: 계산된 출력 토큰 비용 (USD, 소수점 6자리)
- platform_fee_usd: 이 작업에 대한 플랫폼 수수료
- node_earnings_usd: 이 작업에 대한 노드 운영자 수익
- total_cost_usd: 제출자의 총 비용

**7.2 서명 방식**

영수증은 Ed25519를 사용하여 서명됩니다. 노드는 영수증의 정규 JSON 직렬화(키 정렬, 공백 없음)를 등록된 개인 키로 서명합니다. 서명은 base64url로 인코딩되어 영수증의 signature 필드에 포함됩니다.

페더레이션은 노드의 등록된 공개 키를 사용하여 수신 시 영수증 서명을 검증합니다. 유효하지 않은 서명이 있는 영수증은 거부되고 노드 무결성 경고를 트리거합니다.

**7.3 영수증 저장 및 조회**

페더레이션은 컴플라이언스 감사 요구사항을 지원하기 위해 최소 7년간 모든 영수증을 저장합니다. 제출자는 GET /v1/federation/receipts를 통해 영수증을 조회할 수 있습니다. 노드 운영자는 GET /v1/federation/nodes/{id}/receipts를 통해 자신이 실행한 작업의 영수증을 조회할 수 있습니다.

## 8. 구성 가능한 경제적 정산

MFOP는 청구(서명된 영수증 누적)를 정산(자금의 재정적 이체)과 분리합니다. 정산은 구성 가능하며 참여자 유형에 따라 다른 일정으로 발생할 수 있습니다.

**8.1 BillingFeeConfig**

플랫폼 관리자는 BillingFeeConfig 객체를 통해 수수료율을 구성합니다. 각 BillingFeeConfig에는 버전 식별자와 발효일이 있으며, 페더레이션은 작업 실행 시점에 적용되는 구성을 사용합니다. 새 구성은 언제든지 생성할 수 있으며, 다음 청구 기간 시작 시 적용됩니다.

BillingFeeConfig 필드:

- input_token_rate_usd_per_1k: 입력 토큰 1,000개당 청구되는 USD
- output_token_rate_usd_per_1k: 출력 토큰 1,000개당 청구되는 USD
- platform_fee_pct: 총 토큰 비용에서 플랫폼의 비율 (0–100)
- node_revenue_share_pct: 총 토큰 비용에서 노드 운영자의 비율 (0–100, platform_fee_pct와 합산하여 ≤ 100이어야 함)
- settlement_period_days: 정산이 실행되는 빈도 (예: 30)
- minimum_payout_usd: 노드 운영자가 지급받기 전 최소 누적 수익

**8.2 제출자 청구**

제출자는 후불 방식으로 청구됩니다. 각 정산 기간 말에 페더레이션은 제출자의 모든 영수증을 집계하고 등록된 결제 수단에 청구합니다. 청구서에는 컴플라이언스 구역 및 모델별로 그룹화된 작업 영수증의 항목별 목록이 포함됩니다.

**8.3 노드 운영자 정산**

노드 운영자는 누적 수익이 minimum_payout_usd 임계값을 초과하는 경우, 각 정산 기간 말에 Stripe Connect를 통해 지급받습니다. 임계값을 충족하지 못하는 운영자는 수익을 다음 기간으로 이월합니다.

## 9. 보안 모델

MFOP는 세 가지 레이어의 보안 모델을 구현합니다: 전송 보안, AI 안전 정책 검증, 실행 샌드박스 격리.

**9.1 전송 보안**

모든 MFOP API 엔드포인트는 반드시 TLS 1.3 이상을 사용하는 HTTPS를 통해 제공되어야 합니다. 프라이빗 엔터프라이즈 메시 배포의 노드-페더레이션 통신에는 상호 TLS(mTLS)가 권장됩니다. API 인증은 X-Channel-API-Key HTTP 헤더로 전송되는 PAK Key를 사용합니다. PAK Key는 base64url로 인코딩된 256비트 랜덤 값입니다.

**9.2 AI 안전 정책 검증**

모든 작업 입력 및 출력은 실행 전과 제출자에게 전달하기 전에 NeMo Guardrails 정책에 따라 검증됩니다. 기준 정책 집합(모든 컴플라이언스 구역에 필수)에는 다음이 포함됩니다:

- 탈옥 감지 및 차단
- 유해 콘텐츠 감지 (폭력, 아동 성착취물, 자해 조장)
- 출력의 PII 유출 감지
- 프롬프트 인젝션 감지

특정 컴플라이언스 구역에는 추가 정책이 필요합니다 (부록 B 참조).

노드는 반드시 능력 광고에 명시된 NeMo Guardrails 런타임 버전을 실행해야 합니다. 오래된 Guardrails 버전을 실행하는 노드는 DEGRADED로 표시되고, 설치된 버전에 없는 guardrails 기능을 요구하는 컴플라이언스 구역의 라우팅에서 제외됩니다.

**9.3 실행 샌드박스 격리**

각 작업은 격리된 샌드박스에서 실행됩니다. 노드는 반드시 다음 메커니즘 중 하나를 사용하여 샌드박스 격리를 구현해야 합니다:

- gVisor (runsc) — 클라우드 배포에 권장됨
- Firecracker 마이크로VM — 베어메탈 배포에 권장됨
- WASM (Wasmtime) — CPU 전용 추론 워크로드에 허용됨

샌드박스는 반드시 작업 사이에 소멸되고 재생성되어야 합니다. 영구적인 샌드박스 상태(예: 모델 가중치)는 읽기 전용 마운트를 통해 작업 간에 공유될 수 있지만, 작업별 상태(컨텍스트, 임시 파일)는 반드시 작업 사이에 유지되어서는 안 됩니다.

**9.4 감사 로깅**

모든 작업 라우팅 결정, 영수증 서명 및 정산 이벤트는 추가 전용 감사 로그에 기록됩니다. 감사 로그는 SHA-256 해시를 사용하여 암호학적으로 연결됩니다(각 항목에는 이전 항목의 해시가 포함됩니다). 감사 로그는 수정할 수 없으며, 추가 작업만 허용됩니다.

## 10. 와이어 프로토콜

MFOP는 모든 API 통신에 HTTPS를 통한 JSON을 사용합니다. WebSocket 연결은 스트리밍 작업 출력을 지원합니다 (섹션 10.2 참조).

**10.1 요청 및 응답 형식**

모든 요청 및 응답 본문은 UTF-8 인코딩된 JSON입니다. 요청에는 반드시 Content-Type: application/json이 포함되어야 합니다. 성공적인 응답은 HTTP 200 또는 201을 사용합니다. 오류 응답은 표준 오류 봉투를 사용합니다:

{ "error": { "code": "<machine-readable-code>", "message": "<human-readable-message>", "details": { ... } } }

표준 오류 코드: UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, QUOTA_EXCEEDED, NO_ELIGIBLE_NODE, COMPLIANCE_VIOLATION, INTERNAL_ERROR.

**10.2 스트리밍 출력**

스트리밍 출력을 지원하는 노드는 wss://{node_endpoint}/v1/jobs/{id}/stream에 WebSocket 엔드포인트를 노출합니다. 클라이언트는 작업 제출 후 연결합니다. 노드는 JSON 프레임 델타 메시지로 토큰 출력을 스트리밍합니다:

{ "type": "delta", "text": "...", "token_count": N }

스트림은 완료 메시지로 종료됩니다:

{ "type": "done", "receipt": { ... } }

완료 메시지의 영수증은 해당 작업의 서명된 BillingReceipt입니다.

**10.3 멱등성**

작업 제출 요청에는 Idempotency-Key 헤더(UUID)를 포함하는 것이 좋습니다. 멱등성 창(24시간) 내에 동일한 Idempotency-Key를 가진 요청이 수신되면, 페더레이션은 작업을 재실행하지 않고 원래 응답을 반환합니다. 이는 네트워크 재시도로 인한 중복 제출을 방지합니다.

## 부록 A. REST API 참조

이 부록은 MFOP REST API 엔드포인트를 나열합니다. 별도로 명시되지 않는 한 모든 엔드포인트에는 X-Channel-API-Key 헤더가 필요합니다. 기본 경로: /v1/federation

| 메서드 + 경로 | 이름 | 설명 |
| --- | --- | --- |
| POST /v1/federation/nodes/register | 노드 등록 | 페더레이션에 새 노드를 등록합니다. |
| PUT /v1/federation/nodes/{id}/capabilities | 능력 업데이트 | 노드의 능력 광고를 업데이트합니다. |
| POST /v1/federation/nodes/{id}/heartbeat | 노드 하트비트 | 노드가 활성 상태이며 작업을 수락 중임을 신호합니다. |
| DELETE /v1/federation/nodes/{id} | 노드 등록 해제 | 자발적으로 노드를 등록 해제합니다. |
| POST /v1/federation/jobs | 작업 제출 | 실행을 위해 페더레이션에 작업을 제출합니다. |
| GET /v1/federation/jobs/{id} | 작업 상태 | 작업의 현재 상태 및 결과를 조회합니다. |
| GET /v1/federation/jobs/{id}/receipt | 작업 영수증 | 완료된 작업의 서명된 청구 영수증을 조회합니다. |
| GET /v1/federation/receipts | 제출자 영수증 | 인증된 제출자의 모든 영수증을 나열합니다. |
| GET /v1/federation/nodes/{id}/receipts | 노드 영수증 | 노드가 실행한 모든 작업의 영수증을 나열합니다. |
| POST /v1/federation/nodes/{id}/stripe/onboard | Stripe Connect 온보딩 | 은행 계좌 설정을 위한 Stripe 호스팅 온보딩 URL을 반환합니다. |
| GET /v1/federation/nodes/{id}/earnings | 제공자 수익 | 현재 기간 토큰, 예상 수익, 마지막 지급. |
| GET /v1/federation/submitters/billing | 제출자 청구 요약 | 현재 기간 비용, 다음 청구일. |
| PATCH /v1/admin/federation/billing-config | 수수료 모델 업데이트 | 관리자 전용. 새 BillingFeeConfig 행을 생성합니다. 다음 기간부터 적용됩니다. |

## 부록 B. 컴플라이언스 구역 정책 요구사항

각 컴플라이언스 구역은 기준을 넘어 특정 NeMo Guardrails 정책 기능을 필요로 합니다. 다음 표는 구역별 최소 필수 레일을 요약합니다.

| 구역 | 기준 이외의 필수 레일 |
| --- | --- |
| public | 기준만 해당. 추가 레일 불필요. |
| enterprise (SOC2) | 데이터 레지던시 마커 감지. API 자격 증명 유출 감지. 접근 로깅 강제 적용. |
| hipaa | PHI 패턴 감지: 환자 이름, 생년월일, MRN, ICD-10 코드, 진단 설명, 건강 보험 ID. PHI 비식별화 레일: AI 모델 호출 전 PHI 제거 또는 해시 처리. 출력에 대한 최소 필요 확인. |
| sox | 금융 PII 격리: 계좌 번호, 라우팅 번호, 납세자 ID. 가격 예측 차단: 미래 지향적 수익 또는 가격 진술. MNPI 감지: 중요 비공개 정보 패턴 매칭. |
| fedramp | CUI 처리: 통제 비분류 정보 마커 감지 및 처리 규칙. 수출 통제: EAR/ITAR 주제 사항 감지. 분류 표시 강제 적용: 분류 표시가 포함된 출력 차단. |

## 감사의 말

저자는 이 사양에서 참조된 기반 보안 인프라를 제공하는 NeMo Guardrails 및 NemoClaw OpenShell 플랫폼에 대해 NVIDIA NeMo 팀에 감사드립니다. MFOP 보안 모델은 이러한 플랫폼이 성숙해짐에 따라 함께 발전하도록 설계되었습니다.

이 사양에 설명된 세 레이어 보안 모델, 컴플라이언스 구역 분류 체계, Ed25519 영수증 서명 방식, 구성 가능한 청구 아키텍처는 2026년 초 Thrive Tech Services LLC에서 수행된 광범위한 설계 및 검토 프로세스를 통해 개발되고 다듬어졌습니다.

이 사양은 법률, 의료, 연구, 금융, 기술 분야의 전 세계 지식 근로자 커뮤니티에 헌정됩니다. 페더레이션 AI 오케스트레이션이 중요한 이유는 바로 그들의 작업 때문입니다.

MFOP 사양 버전 1.0 종료 — 동료 검토를 위한 초안
Thrive Tech Services LLC · Ami Hoepner Nuñez · 2026년 3월

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · 2026년 3월*
