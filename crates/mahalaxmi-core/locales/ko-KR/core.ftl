# 오류
error-config-file-not-found = { $path }에서 설정 파일을 찾을 수 없습니다
error-config-parse-failed = 설정 파싱에 실패했습니다: { $reason }
error-config-validation-failed = 설정 유효성 검사에 실패했습니다: { $details }
error-locale-not-supported = 로케일 "{ $locale }"은(는) 지원되지 않습니다
error-log-init-failed = 로깅 초기화에 실패했습니다: { $reason }
error-log-dir-create-failed = { $path }에 로그 디렉터리를 생성할 수 없습니다: { $reason }
error-app-launch-failed = 애플리케이션 실행 실패: { $reason }

# 유효성 검사
validation-invalid-log-level = 잘못된 로그 수준 "{ $level }". 유효한 수준: { $valid }
validation-workers-out-of-range = max_concurrent_workers는 { $min }에서 { $max } 사이여야 합니다. 현재 값: { $value }
validation-manager-timeout-too-low = manager_timeout_seconds는 최소 { $min }이어야 합니다. 현재 값: { $value }
validation-worker-timeout-too-low = worker_timeout_seconds는 최소 { $min }이어야 합니다. 현재 값: { $value }
validation-offline-grace-too-low = offline_grace_days는 최소 { $min }이어야 합니다. 현재 값: { $value }
validation-invalid-consensus-strategy = 잘못된 합의 전략 "{ $value }". 유효한 전략: { $valid }
validation-invalid-data-directory = data_directory에 잘못된 문자가 포함되어 있습니다
validation-empty-default-provider = providers.default_provider는 비어 있을 수 없습니다
validation-invalid-theme = 잘못된 테마 "{ $value }". 유효한 테마: { $valid }
validation-font-size-out-of-range = terminal_font_size는 { $min }에서 { $max } 사이여야 합니다. 현재 값: { $value }
validation-invalid-max-batch-retries = max_batch_retriesub294 1 uc774uc0c1uc774uc5b4uc57c ud569ub2c8ub2e4. ud604uc7ac uac12: { $value }
validation-invalid-max-total-batches = max_total_batchesub294 2 uc774uc0c1uc774uc5b4uc57c ud569ub2c8ub2e4. ud604uc7ac uac12: { $value }
validation-invalid-stall-detection-threshold = stall_detection_thresholdub294 2 uc774uc0c1uc774uc5b4uc57c ud569ub2c8ub2e4. ud604uc7ac uac12: { $value }

# 설정
config-loaded-successfully = { $path }에서 설정을 로드했습니다
config-using-default = 설정 파일을 찾을 수 없어 기본값을 사용합니다
config-env-override = 환경 변수 { $var }에 의해 설정 값이 재정의되었습니다
config-env-override-invalid = 환경 변수 { $var }의 값 "{ $value }"이(가) 잘못되었습니다. 설정된 값을 유지합니다
config-generated-successfully = { $path }에 기본 설정이 생성되었습니다
config-already-exists = 설정 파일이 이미 { $path }에 존재합니다

# 로깅
logging-initialized = { $level } 수준으로 로깅이 초기화되었습니다
logging-rust-log-override = RUST_LOG 환경 변수가 감지되어 구성된 수준을 재정의합니다
logging-file-path = 로그 파일: { $path }
logging-dir-create-failed-fallback = 로그 디렉터리 { $path }을(를) 생성할 수 없어 콘솔 전용 로깅으로 전환합니다

# PTY
pty-open-failed = 의사 터미널을 열 수 없습니다: { $reason }
pty-spawn-failed = PTY에서 "{ $program }"을(를) 실행할 수 없습니다: { $reason }
pty-write-failed = 터미널 { $terminal_id }에 쓰기 실패: { $reason }
pty-read-failed = 터미널 { $terminal_id }에서 읽기 실패: { $reason }
pty-resize-failed = 터미널 { $terminal_id }을(를) { $rows }x{ $cols }로 크기 조정 실패: { $reason }
pty-wait-failed = 터미널 { $terminal_id }의 프로세스 상태 확인 실패: { $reason }
pty-kill-failed = 터미널 { $terminal_id }의 프로세스 종료 실패: { $reason }

# 애플리케이션
app-starting = Mahalaxmi v{ $version } 시작 중
app-ready = Mahalaxmi가 준비되었습니다
app-shutting-down = Mahalaxmi를 종료하는 중

# 인증 정보
credential-anthropic-api-key = Claude Code용 Anthropic API 키
credential-generic-api-key = { $provider }용 API 키
credential-aws-iam-role = { $provider }용 AWS IAM 역할
credential-oauth-token = { $provider }용 OAuth 토큰

# 공급자
error-provider-credentials-missing = { $provider } 인증 정보 누락: 환경 변수 { $env_var }이(가) 설정되지 않았습니다
error-provider-credentials-invalid = { $provider } 인증 정보가 유효하지 않습니다: { $reason }
error-provider-not-found = 레지스트리에서 공급자 "{ $provider_id }"을(를) 찾을 수 없습니다
error-provider-no-default = 기본 AI 공급자가 구성되지 않았습니다
error-provider-command-build-failed = { $provider } 명령 구성 실패: { $reason }
provider-registered = 공급자 "{ $provider }"이(가) ID "{ $id }"(으)로 등록되었습니다
provider-set-default = 기본 공급자가 "{ $provider }"(으)로 설정되었습니다
provider-credentials-valid = { $provider } 인증 정보가 성공적으로 확인되었습니다
provider-validating = { $provider } 인증 정보 확인 중
provider-list-header = 등록된 AI 공급자

# PTY (확장)
error-pty-open-failed = PTY 열기 실패: { $reason }
error-pty-spawn-failed = PTY에서 "{ $program }" 실행 실패: { $reason }
error-pty-write-failed = 터미널 { $terminal_id }에 쓰기 실패: { $reason }
error-pty-read-failed = 터미널 { $terminal_id }에서 읽기 실패: { $reason }
error-pty-resize-failed = 터미널 { $terminal_id }을(를) { $rows }x{ $cols }(으)로 크기 조정 실패: { $reason }
error-pty-kill-failed = 터미널 { $terminal_id } 프로세스 종료 실패: { $reason }
error-pty-wait-failed = 터미널 { $terminal_id } 프로세스 상태 확인 실패: { $reason }
error-pty-terminal-not-found = 터미널 { $terminal_id }을(를) 찾을 수 없습니다
error-pty-max-concurrent-reached = 최대 동시 터미널 수({ $max })에 도달했습니다
pty-process-spawned = 터미널 { $terminal_id }에서 프로세스 "{ $program }"이(가) 시작되었습니다
pty-process-exited = 터미널 { $terminal_id } 프로세스가 코드 { $exit_code }(으)로 종료되었습니다
pty-session-closed = 터미널 세션 { $terminal_id }이(가) 닫혔습니다
pty-resized = 터미널 { $terminal_id }이(가) { $rows }x{ $cols }(으)로 크기 조정되었습니다
pty-reader-eof = 터미널 { $terminal_id } 리더가 스트림 끝에 도달했습니다
pty-reader-error = 터미널 { $terminal_id } 리더 오류: { $reason }

# 오케스트레이션 오류
error-orchestration-invalid-transition = { $from }에서 { $to }로의 잘못된 상태 전환
error-orchestration-circular-dependency = 순환 의존성이 감지되었습니다: { $cycle }
error-orchestration-worker-not-found = 워커 { $worker_id }을(를) 대기열에서 찾을 수 없습니다
error-orchestration-max-retries-exceeded = 워커 { $worker_id }이(가) 최대 재시도 횟수를 초과했습니다 ({ $max_retries })
error-orchestration-no-proposals = 관리자 제안이 수신되지 않았습니다
error-orchestration-plan-validation-failed = 실행 계획 검증 실패: { $errors }
error-orchestration-consensus-failed = 합의 엔진 실패: { $reason }
error-orchestration-queue-full = 워커 대기열이 가득 찼습니다 (최대 { $max })
error-orchestration-manager-timeout = 관리자 { $manager_id }이(가) { $timeout }초 후에 시간 초과되었습니다
error-orchestration-worker-timeout = 워커 { $worker_id }이(가) { $timeout }초 후에 시간 초과되었습니다

# 오케스트레이션 정보
orchestration-cycle-started = 오케스트레이션 주기 { $cycle_id } 시작됨
orchestration-state-changed = 상태 변경됨: { $from } -> { $to }
orchestration-manager-completed = 관리자 { $manager_id } 완료, { $task_count }개 작업
orchestration-consensus-reached = 합의 도달: { $agreed }개 동의, { $dissenting }개 반대
orchestration-plan-created = 실행 계획 생성됨: { $phases }개 단계, { $workers }개 워커
orchestration-worker-started = 워커 { $worker_id } 시작됨: { $task }
orchestration-worker-completed = 워커 { $worker_id }이(가) { $duration }ms에 완료됨
orchestration-worker-failed = 워커 { $worker_id } 실패: { $error }
orchestration-cycle-completed = 주기가 { $duration }ms에 완료됨 (성공률: { $success_rate })
orchestration-worker-retrying = 워커 { $worker_id } 재시도 중 (시도 { $attempt }/{ $max })

# 감지 오류
error-detection-rule-compile-failed = 감지 규칙 패턴 컴파일 실패: { $reason }
error-detection-no-rules-loaded = 감지 규칙이 로드되지 않았습니다
error-detection-invalid-pattern = 잘못된 감지 패턴 "{ $pattern }": { $reason }

# 감지 정보
detection-rule-matched = 감지 규칙 "{ $rule }" 일치, 동작: { $action }
detection-rule-cooldown = 감지 규칙 "{ $rule }"이(가) 쿨다운에 의해 억제됨 ({ $remaining_ms }ms 남음)
detection-rules-loaded = { $count }개의 감지 규칙이 로드됨
detection-provider-rules-applied = 제공자 { $provider }에 { $count }개의 규칙 적용됨
detection-error-pattern-detected = 오류 패턴 감지됨: "{ $pattern }" ({ $count }회 확인)
detection-root-cause-hypothesis = 근본 원인 가설: { $category } (신뢰도: { $confidence })
detection-recurring-error = 반복 오류: "{ $message }" ({ $count }회 발생)
detection-action-executed = 동작 { $action }이(가) 규칙 "{ $rule }"에 대해 실행됨
detection-cooldowns-reset = { $rule_count }개 규칙의 쿨다운이 재설정됨

# 템플릿 오류
error-template-not-found = 템플릿 { $template_id }을(를) 찾을 수 없습니다
error-template-category-not-found = 템플릿 카테고리 { $category_id }을(를) 찾을 수 없습니다
error-template-composition-failed = 템플릿 구성 실패: { $reason }
error-template-include-not-found = 포함 파일을 찾을 수 없습니다: { $path }
error-template-circular-include = 순환 포함이 감지되었습니다 (최대 깊이 { $depth } 초과)
error-template-placeholder-unresolved = 미해결 자리 표시자: ${ $placeholder }
error-template-validation-failed = 템플릿 유효성 검사가 { $count }개의 오류로 실패했습니다
error-template-activation-failed = 템플릿 활성화 실패: { $reason }
error-template-catalog-load-failed = 템플릿 카탈로그 로드 실패: { $path }
error-template-invalid-version = 잘못된 템플릿 버전 형식: { $version }

# 템플릿 정보
template-catalog-loaded = { $count }개의 템플릿이 포함된 템플릿 카탈로그가 로드되었습니다
template-activated = 템플릿 { $template_id }이(가) 성공적으로 활성화되었습니다
template-composition-complete = 구성 완료: { $included }개 포함, { $overridden }개 재정의
template-placeholders-resolved = { $count }개의 자리 표시자가 해결되었습니다
template-validation-passed = 도메인 { $domain }에 대한 템플릿 유효성 검사 통과
template-validation-warnings = 템플릿 유효성 검사가 { $count }개의 경고와 함께 완료되었습니다
template-include-resolved = 포함이 해결되었습니다: { $path }
template-provider-instructions-injected = 제공자 { $provider }에 대한 지침이 삽입되었습니다
template-project-config-loaded = { $path }에서 프로젝트 설정이 로드되었습니다
template-domain-validator-registered = 도메인 유효성 검사기가 등록되었습니다: { $domain }

# 라이선스 오류
error-license-file-not-found = 라이선스 파일을 찾을 수 없습니다: { $path }
error-license-file-invalid = 잘못된 라이선스 파일 ({ $path }): { $reason }
error-license-file-write-failed = 라이선스 파일 쓰기 실패 ({ $path }): { $reason }
error-license-signature-invalid = 라이선스 서명 검증에 실패했습니다
error-license-signature-decode-failed = 라이선스 서명 디코딩에 실패했습니다: { $reason }
error-license-serialization-failed = 라이선스 데이터 직렬화에 실패했습니다: { $reason }
error-license-signing-failed = 라이선스 서명에 실패했습니다: { $reason }
error-license-feature-denied = 기능 '{ $feature }'은(는) { $tier } 플랜에서 사용할 수 없습니다
error-license-worker-limit = 요청한 워커 수 { $requested }이(가) { $tier } 플랜의 제한 { $limit }을(를) 초과합니다
error-license-manager-limit = 요청한 매니저 수 { $requested }이(가) { $tier } 플랜의 제한 { $limit }을(를) 초과합니다. 설정 → 라이선스에서 구독을 활성화하여 더 많은 기능을 잠금 해제하세요.
error-license-category-denied = 카테고리 '{ $category }'에는 { $required_tier } 플랜이 필요합니다 (현재: { $tier })
error-license-fingerprint-hostname = 호스트명을 확인할 수 없습니다: { $reason }
error-license-fingerprint-username = 사용자명을 확인할 수 없습니다: { $reason }

# 라이선스 상태
license-trial-active = 평가판 라이선스 활성 (잔여 { $days }일)
license-trial-expiring-soon = 평가 기간이 곧 만료됩니다 (잔여 { $days }일)
license-trial-expiring-very-soon = 평가 기간 만료가 임박합니다 (잔여 { $days }일)
license-trial-expired = 평가판 라이선스가 만료되었습니다
license-expires-later = 라이선스가 { $days }일 후에 만료됩니다
license-expires-soon = 라이선스가 곧 만료됩니다 (잔여 { $days }일)
license-expires-very-soon = 라이선스 만료가 임박합니다 (잔여 { $days }일)
license-expires-today = 라이선스가 오늘 만료됩니다
license-grace-period = 라이선스가 만료되었습니다. 유예 기간 중입니다 (잔여 { $days }일)
license-expired = 라이선스가 만료되었습니다

# 플랫폼 지원 — 오류 키
error-platform-unsupported = 지원되지 않는 플랫폼: { $platform }
error-platform-wsl-not-detected = WSL 환경이 감지되지 않았습니다
error-platform-wsl-path-invalid = WSL 변환에 유효하지 않은 경로: { $path }
error-platform-layout-no-space = 패널 레이아웃을 위한 컨테이너가 너무 작습니다 ({ $width }x{ $height })
error-platform-layout-invalid-count = 유효하지 않은 패널 수: { $count }
error-platform-hotkey-registration-failed = 단축키 등록 실패 (충돌): { $shortcut }
error-platform-hotkey-parse-failed = 단축키 분석 실패: { $shortcut }
error-platform-shutdown-timeout = 프로세스 { $pid } ({ $label }) 종료 시간 초과
error-platform-shutdown-failed = 프로세스 { $pid } ({ $label }) 종료 실패
error-platform-shell-not-found = 기본 셸을 찾을 수 없습니다

# 플랫폼 지원 — 정보 키
platform-detected = 플랫폼 감지: { $os } ({ $arch })
platform-wsl-detected = WSL 감지: { $distro } (WSL{ $version })
platform-wsl-path-translated = 경로 변환: { $from } → { $to }
platform-layout-calculated = 레이아웃 계산: { $panels } 패널, { $rows }x{ $cols } 그리드
platform-layout-optimized = 레이아웃 최적화: { $utilization }% 활용도
platform-hotkey-registered = 단축키 등록: { $command } → { $shortcut }
platform-hotkey-unregistered = 단축키 해제: { $command }
platform-shutdown-initiated = { $count } 프로세스 종료 시작
platform-shutdown-completed = 종료 완료: { $count } 프로세스, { $duration }ms
platform-shell-detected = 셸 감지: { $shell } ({ $path })

# 메모리 오류
error-memory-not-found = 메모리 항목을 찾을 수 없습니다: { $id }
error-memory-duplicate = 중복된 메모리 항목: { $id }
error-memory-persistence-failed = 메모리 저장소 영속화에 실패했습니다: { $reason }
error-memory-load-failed = 메모리 저장소 로드에 실패했습니다: { $reason }
error-memory-invalid-confidence = 잘못된 신뢰도 점수: { $value } (0.0에서 1.0 사이여야 합니다)
error-memory-store-full = 메모리 저장소가 가득 찼습니다 (최대 { $max } 항목)
error-memory-invalid-query = 잘못된 메모리 쿼리: { $reason }
error-memory-serialization = 메모리 직렬화에 실패했습니다: { $reason }
error-memory-invalid-entry = 잘못된 메모리 항목: { $reason }
error-memory-session-mismatch = 세션 불일치: 예상 { $expected }, 실제 { $actual }

# 메모리 정보
memory-store-created = 세션 { $session_id }의 메모리 저장소가 생성되었습니다
memory-entry-added = 메모리 항목이 추가되었습니다: { $title } (유형: { $memory_type })
memory-entry-updated = 메모리 항목이 업데이트되었습니다: { $id }
memory-entry-removed = 메모리 항목이 제거되었습니다: { $id }
memory-store-cleared = 메모리 저장소가 비워졌습니다 ({ $count } 항목 제거)
memory-persisted = 메모리 저장소가 { $path }에 영속화되었습니다
memory-loaded = { $path }에서 메모리 저장소를 로드했습니다 ({ $count } 항목)
memory-query-executed = 메모리 쿼리가 { $count }개의 결과를 반환했습니다
memory-injected = { $count }개의 메모리가 주입되었습니다 ({ $tokens } 토큰)
memory-stats = 메모리 통계: { $total } 항목, 평균 신뢰도 { $avg_confidence }

# 인덱싱 오류
error-indexing-parse-failed = { $file } 파싱에 실패했습니다: { $reason }
error-indexing-file-read-failed = 파일 { $file } 읽기에 실패했습니다: { $reason }
error-indexing-unsupported-language = 파일 확장자에 대해 지원되지 않는 언어: { $extension }
error-indexing-extraction-failed = { $file }의 심볼 추출에 실패했습니다: { $reason }
error-indexing-graph-cycle-detected = 의존성 순환이 감지되었습니다: { $files }
error-indexing-fingerprint-failed = { $file }의 핑거프린트 계산에 실패했습니다: { $reason }
error-indexing-build-failed = 인덱스 빌드에 실패했습니다: { $reason }
error-indexing-update-failed = 증분 업데이트에 실패했습니다: { $reason }

# 인덱싱 정보
indexing-file-indexed = 파일 인덱싱 완료: { $file } ({ $language })
indexing-symbols-extracted = { $file }에서 { $count }개의 심볼을 추출했습니다
indexing-graph-built = 의존성 그래프 구축 완료: { $files }개 파일, { $edges }개 엣지
indexing-ranking-computed = { $symbols }개 심볼에 대한 순위를 계산했습니다
indexing-repomap-generated = 저장소 맵 생성 완료: { $symbols }개 심볼, { $tokens }개 토큰
indexing-index-built = 코드베이스 인덱스 구축 완료: { $files }개 파일, { $symbols }개 심볼
indexing-incremental-update = 증분 업데이트: { $added }개 추가, { $modified }개 수정, { $removed }개 삭제
indexing-language-registered = 언어가 등록되었습니다: { $language }

# 컨텍스트 오류
error-context-budget-exceeded = 컨텍스트 토큰 예산 초과: 사용 { $used }, 예산 { $budget }
error-context-invalid-allocations = 예산 할당 합계는 <= 1.0이어야 합니다. 합계 { $sum }
error-context-build-failed = 작업 { $task_id }의 컨텍스트 빌드에 실패했습니다: { $reason }
error-context-invalid-format = 잘못된 컨텍스트 형식: { $format }

# 컨텍스트 정보
context-budget-allocated = 토큰 예산 할당됨: { $total } 토큰 ({ $repo_map } 저장소 맵, { $files } 파일, { $memory } 메모리, { $task } 작업)
context-files-scored = { $count }개 파일의 관련성을 평가했습니다 (최상위: { $top_file })
context-chunks-created = { $count }개의 코드 청크를 생성했습니다 ({ $tokens } 토큰)
context-assembled = 컨텍스트 조립 완료: { $sections }개 섹션, { $budget } 예산 중 { $tokens } 토큰 사용
context-injected = 워커 { $worker_id }에 컨텍스트 주입됨 ({ $tokens } 토큰, { $files }개 파일)
context-skipped = 컨텍스트 준비 건너뛰기: { $reason }

# MCP 오류
error-mcp-parse-failed = JSON-RPC 메시지 구문 분석 실패: { $reason }
error-mcp-invalid-request = 잘못된 JSON-RPC 요청: { $reason }
error-mcp-method-not-found = 메서드를 찾을 수 없음: { $method }
error-mcp-invalid-params = 잘못된 매개변수: { $reason }
error-mcp-internal-error = MCP 서버 내부 오류: { $reason }
error-mcp-not-initialized = MCP 서버가 초기화되지 않았습니다
error-mcp-tool-not-found = 도구를 찾을 수 없음: { $tool }
error-mcp-tool-execution-failed = 도구 "{ $tool }" 실행 실패: { $reason }
error-mcp-transport-error = MCP 전송 오류: { $reason }
error-mcp-shutdown-failed = MCP 서버 종료 실패: { $reason }

# MCP 정보
mcp-server-started = MCP 서버 시작됨 ({ $transport } 전송)
mcp-server-stopped = MCP 서버 중지됨
mcp-client-initialized = MCP 클라이언트 초기화됨: { $client_name }
mcp-tool-called = 도구 호출됨: { $tool }
mcp-tool-completed = 도구 "{ $tool }" { $duration }ms 만에 완료
mcp-request-received = 요청 수신됨: { $method }
mcp-response-sent = 응답 전송됨: { $method }
mcp-transport-ready = MCP 전송 준비 완료: { $transport }

# Graph errors
error-graph-entity-not-found = 그래프 엔티티를 찾을 수 없습니다: { $id }
error-graph-relationship-failed = 관계 추가 실패: { $reason }
error-graph-build-failed = 지식 그래프 구축 실패: { $reason }
error-graph-update-failed = 지식 그래프 업데이트 실패: { $reason }
error-graph-load-failed = { $path }에서 지식 그래프 로드 실패: { $reason }
error-graph-save-failed = { $path }에 지식 그래프 저장 실패: { $reason }
error-graph-max-entities-exceeded = 지식 그래프가 최대 엔티티 제한을 초과했습니다: { $count } / { $max }

# Graph info
graph-built = 지식 그래프 구축 완료: { $entities }개 엔티티, { $relationships }개 관계
graph-updated = 지식 그래프 업데이트됨: { $added }개 추가, { $removed }개 제거
graph-entity-added = 지식 그래프에 엔티티 추가됨: { $name } ({ $kind })
graph-entity-removed = 지식 그래프에서 엔티티 제거됨: { $name }
graph-persisted = 지식 그래프가 { $path }에 저장됨
graph-loaded = 지식 그래프가 { $path }에서 로드됨 ({ $entities }개 엔티티)
graph-query-executed = 그래프 쿼리가 { $ms }ms에 실행됨, { $results }개 결과

# 플랫폼 API 오류
error-platform-api-request-failed = 플랫폼 API 요청 실패: { $reason }
error-platform-api-unauthorized = 플랫폼 API 인증 실패 — channel_api_key를 확인하세요
error-platform-api-not-found = 플랫폼 리소스를 찾을 수 없음: { $resource }
error-platform-api-rate-limited = 플랫폼 API 속도 제한 — { $seconds }초 후에 재시도하세요
error-platform-api-server-error = 플랫폼 서버 오류 ({ $status }): { $message }
error-platform-trial-not-eligible = 이 기기는 평가판 대상이 아닙니다: { $reason }
error-platform-activation-failed = 라이선스 활성화 실패: { $reason }
error-platform-validation-failed = 라이선스 검증 실패: { $reason }
error-platform-deactivation-failed = 기기 비활성화 실패: { $reason }
error-platform-cache-read-failed = { $path }에서 라이선스 캐시 읽기 실패: { $reason }
error-platform-cache-write-failed = { $path }에 라이선스 캐시 쓰기 실패: { $reason }
error-platform-cache-decrypt-failed = 라이선스 캐시 복호화 실패 (키 불일치 또는 손상)
error-platform-not-configured = 플랫폼 통합이 구성되지 않았습니다 — 설정에서 platform_base_url을 설정하세요

# 플랫폼 API 정보
platform-api-trial-activated = 평가판 활성화됨: { $tier } 플랜, { $days }일
platform-api-license-activated = 라이선스 활성화됨: { $tier } 플랜 (활성화 { $activation_id })
platform-api-license-validated = 라이선스 검증됨: { $tier } 플랜, 잔여 { $days }일
platform-api-heartbeat-sent = 하트비트 전송됨 (활성화 { $activation_id })
platform-api-device-deactivated = 기기가 라이선스에서 비활성화됨
platform-api-cache-updated = 라이선스 캐시가 { $path }에 업데이트됨
platform-api-offline-fallback = 플랫폼에 연결할 수 없어 캐시된 라이선스를 사용합니다 ({ $days_ago }일 전에 캐시됨)

# 메시징 오류
error-messaging-not-registered = 메시징 클라이언트가 등록되지 않았습니다
error-messaging-registration-failed = 메시징 등록 실패: { $reason }
error-messaging-send-failed = 메시지 전송 실패: { $reason }
error-messaging-poll-failed = 메시지 폴링 실패: { $reason }
error-messaging-ack-failed = 메시지 { $message_id } 수신 확인 실패: { $reason }
error-messaging-disabled = 이 라이선스에서는 메시징이 비활성화되어 있습니다

# 메시징 정보
messaging-registered = 기기 { $device_id }에 대한 메시징이 등록됨
messaging-unregistered = 메시징이 등록 해제됨
messaging-message-received = 메시지 수신됨: { $subject } (유형: { $message_type })
messaging-message-sent = 메시지 전송됨 (ID: { $message_id })
messaging-poll-completed = 메시지 폴링 완료: { $count }개의 새 메시지

# Provider credential descriptions
credential-xai-api-key = Grok용 xAI API 키 (XAI_API_KEY)
credential-openai-api-key = OpenAI API 키 (OPENAI_API_KEY)
credential-google-api-key = Gemini용 Google API 키 (GOOGLE_API_KEY)
credential-gh-auth = gh CLI를 통한 GitHub 인증 (gh auth login)

# Built-in category names
category-SoftwareDevelopment = 소프트웨어 개발
category-LinuxDevelopment = Linux 개발
category-macOSDevelopment = macOS 개발
category-PythonDevelopment = Python 개발
category-AIFrameworks = AI 및 ML 프레임워크
category-GraphQL = GraphQL 프레임워크
category-DataScience = 데이터 과학 및 분석
category-Legal = 법률 / 법률보조
category-Music = 음악 제작
category-PhysicalSystems = 물리 시스템 및 현상
category-BacteriaScience = 세균 과학 및 미생물학
category-NursingScience = 간호 과학 및 임상 실습
category-ElectronDevelopment = Electron 데스크톱 개발
category-GameDevelopment = 게임 개발
category-3DModeling = 3D 모델링 및 디지털 콘텐츠 제작
category-Custom = 사용자 정의 템플릿

# Built-in category descriptions
category-SoftwareDevelopment-desc = 애플리케이션, API, 데이터베이스 및 스크립트 생성용 템플릿
category-LinuxDevelopment-desc = Linux 시스템 관리, 셸 스크립팅 및 서버 개발용 템플릿
category-macOSDevelopment-desc = macOS 애플리케이션, Swift/Objective-C 개발 및 Apple 프레임워크용 템플릿
category-PythonDevelopment-desc = Python 애플리케이션, 스크립트, 웹 프레임워크 및 자동화용 템플릿
category-AIFrameworks-desc = AI 에이전트, LLM 오케스트레이션, 챗봇 및 ML 애플리케이션용 템플릿
category-GraphQL-desc = GraphQL 서버, 클라이언트 및 API 개발용 템플릿
category-DataScience-desc = 데이터 과학 라이프사이클용 템플릿: 수학, 데이터 엔지니어링, ML, 딥러닝, MLOps
category-Legal-desc = 법률 문서 처리, 리서치 및 사건 관리용 템플릿
category-Music-desc = DAW, 플러그인 개발, 모듈러 신디사이저 및 하드웨어 통합용 템플릿
category-PhysicalSystems-desc = 산업 물리학, 프로세스 모니터링, 제어 시스템 및 예측 분석용 템플릿
category-BacteriaScience-desc = 미생물학, 유전체학, 메타유전체학, 항균 내성 및 진단용 템플릿
category-NursingScience-desc = 간호 교육, 임상 실습, 환자 간호 및 의료 분석용 템플릿
category-ElectronDevelopment-desc = Electron 및 최신 도구를 사용한 크로스 플랫폼 데스크톱 애플리케이션용 템플릿
category-GameDevelopment-desc = 게임 엔진, 프레임워크 및 인터랙티브 엔터테인먼트 개발용 템플릿
category-3DModeling-desc = 3D 모델링, VFX, 애니메이션 및 디지털 콘텐츠 제작 도구용 템플릿
category-Custom-desc = 사용자가 만든 사용자 정의 템플릿

# Provider status
provider-not-installed = 제공자 { $provider }는 { $binary }가 필요하지만 설치되지 않았습니다
provider-binary-found = { $path }에서 { $binary }를 찾았습니다
provider-test-timeout = { $seconds }초 후 연결 테스트 시간 초과
provider-test-failed = 제공자 테스트 실패: { $error }
provider-env-saved = { $provider }의 { $env_var }를 저장했습니다

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
error-orchestration-quorum-not-met = 관리자 쿼럼 미달: { $received }/{ $expected } 제안 수신 (최소: { $min_quorum })
orchestration-file-conflict-serialized = 작업 { $later_task }이 { $earlier_task } 이후로 직렬화됨 — 둘 다 파일 '{ $file }'을 요청
orchestration-build-check-failed = 워커 { $worker_id }의 워커별 { $project_type } 빌드 검사 실패
orchestration-build-check-passed = 워커 { $worker_id }의 워커별 { $project_type } 빌드 검사 통과
