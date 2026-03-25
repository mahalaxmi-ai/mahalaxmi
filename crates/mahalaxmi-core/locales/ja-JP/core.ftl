# エラー
error-config-file-not-found = 設定ファイルが見つかりません: { $path }
error-config-parse-failed = 設定の解析に失敗しました: { $reason }
error-config-validation-failed = 設定の検証に失敗しました: { $details }
error-locale-not-supported = ロケール「{ $locale }」はサポートされていません
error-log-init-failed = ログの初期化に失敗しました: { $reason }
error-log-dir-create-failed = ログディレクトリの作成に失敗しました ({ $path }): { $reason }
error-app-launch-failed = アプリケーションの起動に失敗しました: { $reason }

# 検証
validation-invalid-log-level = 無効なログレベル「{ $level }」です。有効なレベル: { $valid }
validation-workers-out-of-range = max_concurrent_workersは{ $min }から{ $max }の間である必要があります。指定値: { $value }
validation-manager-timeout-too-low = manager_timeout_secondsは最低{ $min }である必要があります。指定値: { $value }
validation-worker-timeout-too-low = worker_timeout_secondsは最低{ $min }である必要があります。指定値: { $value }
validation-offline-grace-too-low = offline_grace_daysは最低{ $min }である必要があります。指定値: { $value }
validation-invalid-consensus-strategy = 無効なコンセンサス戦略「{ $value }」です。有効な戦略: { $valid }
validation-invalid-data-directory = data_directoryに無効な文字が含まれています
validation-empty-default-provider = providers.default_providerは空にできません
validation-invalid-theme = 無効なテーマ「{ $value }」です。有効なテーマ: { $valid }
validation-font-size-out-of-range = terminal_font_sizeは{ $min }から{ $max }の間である必要があります。指定値: { $value }
validation-invalid-max-batch-retries = max_batch_retriesu306f1u4ee5u4e0au3067u3042u308bu5fc5u8981u304cu3042u308au307eu3059u3002u6307u5b9au5024: { $value }
validation-invalid-max-total-batches = max_total_batchesu306f2u4ee5u4e0au3067u3042u308bu5fc5u8981u304cu3042u308au307eu3059u3002u6307u5b9au5024: { $value }
validation-invalid-stall-detection-threshold = stall_detection_thresholdu306f2u4ee5u4e0au3067u3042u308bu5fc5u8981u304cu3042u308au307eu3059u3002u6307u5b9au5024: { $value }

# 設定
config-loaded-successfully = 設定を読み込みました: { $path }
config-using-default = 設定ファイルが見つかりません。デフォルト値を使用します
config-env-override = 環境変数{ $var }により設定値が上書きされました
config-env-override-invalid = 環境変数{ $var }の値「{ $value }」は無効です。設定値を維持します
config-generated-successfully = デフォルト設定を生成しました: { $path }
config-already-exists = 設定ファイルは既に存在します: { $path }

# ログ
logging-initialized = ログが{ $level }レベルで初期化されました
logging-rust-log-override = RUST_LOG環境変数が検出されました。設定されたレベルを上書きします
logging-file-path = ログファイル: { $path }
logging-dir-create-failed-fallback = ログディレクトリ{ $path }の作成に失敗しました。コンソールのみのログに切り替えます

# PTY
pty-open-failed = 疑似端末を開けませんでした: { $reason }
pty-spawn-failed = PTYで「{ $program }」を起動できませんでした: { $reason }
pty-write-failed = ターミナル{ $terminal_id }への書き込みに失敗しました: { $reason }
pty-read-failed = ターミナル{ $terminal_id }からの読み取りに失敗しました: { $reason }
pty-resize-failed = ターミナル{ $terminal_id }を{ $rows }x{ $cols }にリサイズできませんでした: { $reason }
pty-wait-failed = ターミナル{ $terminal_id }のプロセス状態を確認できませんでした: { $reason }
pty-kill-failed = ターミナル{ $terminal_id }のプロセスを終了できませんでした: { $reason }

# アプリケーション
app-starting = Mahalaxmi v{ $version } を起動しています
app-ready = Mahalaxmiの準備が完了しました
app-shutting-down = Mahalaxmiをシャットダウンしています

# 認証情報
credential-anthropic-api-key = Claude Code用Anthropic APIキー
credential-generic-api-key = { $provider }用APIキー
credential-aws-iam-role = { $provider }用AWS IAMロール
credential-oauth-token = { $provider }用OAuthトークン

# プロバイダー
error-provider-credentials-missing = { $provider }の認証情報がありません：環境変数{ $env_var }が設定されていません
error-provider-credentials-invalid = { $provider }の認証情報が無効です：{ $reason }
error-provider-not-found = プロバイダー「{ $provider_id }」がレジストリに見つかりません
error-provider-no-default = デフォルトのAIプロバイダーが設定されていません
error-provider-command-build-failed = { $provider }コマンドの構築に失敗しました：{ $reason }
provider-registered = プロバイダー「{ $provider }」がID「{ $id }」で登録されました
provider-set-default = デフォルトプロバイダーが「{ $provider }」に設定されました
provider-credentials-valid = { $provider }の認証情報が正常に検証されました
provider-validating = { $provider }の認証情報を検証中
provider-list-header = 登録済みAIプロバイダー

# PTY（拡張）
error-pty-open-failed = PTYを開けませんでした：{ $reason }
error-pty-spawn-failed = PTYで「{ $program }」を起動できませんでした：{ $reason }
error-pty-write-failed = ターミナル{ $terminal_id }への書き込みに失敗しました：{ $reason }
error-pty-read-failed = ターミナル{ $terminal_id }からの読み取りに失敗しました：{ $reason }
error-pty-resize-failed = ターミナル{ $terminal_id }を{ $rows }x{ $cols }にリサイズできませんでした：{ $reason }
error-pty-kill-failed = ターミナル{ $terminal_id }のプロセスを終了できませんでした：{ $reason }
error-pty-wait-failed = ターミナル{ $terminal_id }のプロセス状態を確認できませんでした：{ $reason }
error-pty-terminal-not-found = ターミナル{ $terminal_id }が見つかりません
error-pty-max-concurrent-reached = 同時ターミナル数の上限（{ $max }）に達しました
pty-process-spawned = ターミナル{ $terminal_id }でプロセス「{ $program }」が起動しました
pty-process-exited = ターミナル{ $terminal_id }のプロセスがコード{ $exit_code }で終了しました
pty-session-closed = ターミナルセッション{ $terminal_id }が閉じられました
pty-resized = ターミナル{ $terminal_id }が{ $rows }x{ $cols }にリサイズされました
pty-reader-eof = ターミナル{ $terminal_id }のリーダーがストリームの終端に達しました
pty-reader-error = ターミナル{ $terminal_id }のリーダーエラー：{ $reason }

# オーケストレーションエラー
error-orchestration-invalid-transition = { $from }から{ $to }への無効な状態遷移
error-orchestration-circular-dependency = 循環依存関係が検出されました: { $cycle }
error-orchestration-worker-not-found = ワーカー{ $worker_id }がキューに見つかりません
error-orchestration-max-retries-exceeded = ワーカー{ $worker_id }が最大リトライ回数を超えました（{ $max_retries }）
error-orchestration-no-proposals = マネージャー提案が受信されませんでした
error-orchestration-plan-validation-failed = 実行計画の検証に失敗しました: { $errors }
error-orchestration-consensus-failed = コンセンサスエンジンが失敗しました: { $reason }
error-orchestration-queue-full = ワーカーキューが満杯です（最大{ $max }）
error-orchestration-manager-timeout = マネージャー{ $manager_id }が{ $timeout }秒後にタイムアウトしました
error-orchestration-worker-timeout = ワーカー{ $worker_id }が{ $timeout }秒後にタイムアウトしました

# オーケストレーション情報
orchestration-cycle-started = オーケストレーションサイクル{ $cycle_id }が開始されました
orchestration-state-changed = 状態が変更されました: { $from } -> { $to }
orchestration-manager-completed = マネージャー{ $manager_id }が{ $task_count }個のタスクで完了しました
orchestration-consensus-reached = コンセンサス達成: { $agreed }件合意、{ $dissenting }件反対
orchestration-plan-created = 実行計画が作成されました: { $phases }フェーズ、{ $workers }ワーカー
orchestration-worker-started = ワーカー{ $worker_id }が開始されました: { $task }
orchestration-worker-completed = ワーカー{ $worker_id }が{ $duration }msで完了しました
orchestration-worker-failed = ワーカー{ $worker_id }が失敗しました: { $error }
orchestration-cycle-completed = サイクルが{ $duration }msで完了しました（成功率: { $success_rate }）
orchestration-worker-retrying = ワーカー{ $worker_id }がリトライ中（試行{ $attempt }/{ $max }）

# 検出エラー
error-detection-rule-compile-failed = 検出ルールパターンのコンパイルに失敗しました: { $reason }
error-detection-no-rules-loaded = 検出ルールが読み込まれていません
error-detection-invalid-pattern = 無効な検出パターン「{ $pattern }」: { $reason }

# 検出情報
detection-rule-matched = 検出ルール「{ $rule }」が一致しました、アクション: { $action }
detection-rule-cooldown = 検出ルール「{ $rule }」がクールダウンにより抑制されました（残り{ $remaining_ms }ms）
detection-rules-loaded = { $count }個の検出ルールが読み込まれました
detection-provider-rules-applied = プロバイダー{ $provider }に{ $count }個のルールが適用されました
detection-error-pattern-detected = エラーパターンが検出されました:「{ $pattern }」（{ $count }回確認）
detection-root-cause-hypothesis = 根本原因の仮説: { $category }（確信度: { $confidence }）
detection-recurring-error = 繰り返しエラー:「{ $message }」（{ $count }回発生）
detection-action-executed = アクション{ $action }がルール「{ $rule }」に対して実行されました
detection-cooldowns-reset = { $rule_count }個のルールのクールダウンがリセットされました

# テンプレートエラー
error-template-not-found = テンプレート { $template_id } が見つかりません
error-template-category-not-found = テンプレートカテゴリ { $category_id } が見つかりません
error-template-composition-failed = テンプレートの合成に失敗しました: { $reason }
error-template-include-not-found = インクルードファイルが見つかりません: { $path }
error-template-circular-include = 循環インクルードが検出されました（最大深度 { $depth } を超過）
error-template-placeholder-unresolved = 未解決のプレースホルダー: ${ $placeholder }
error-template-validation-failed = テンプレート検証が { $count } 件のエラーで失敗しました
error-template-activation-failed = テンプレートのアクティベーションに失敗しました: { $reason }
error-template-catalog-load-failed = テンプレートカタログの読み込みに失敗しました: { $path }
error-template-invalid-version = テンプレートバージョン形式が無効です: { $version }

# テンプレート情報
template-catalog-loaded = テンプレートカタログが { $count } 件のテンプレートで読み込まれました
template-activated = テンプレート { $template_id } が正常にアクティベートされました
template-composition-complete = 合成完了: { $included } 件インクルード、{ $overridden } 件オーバーライド
template-placeholders-resolved = { $count } 件のプレースホルダーが解決されました
template-validation-passed = ドメイン { $domain } のテンプレート検証に合格しました
template-validation-warnings = テンプレート検証が { $count } 件の警告で完了しました
template-include-resolved = インクルードが解決されました: { $path }
template-provider-instructions-injected = プロバイダー { $provider } の指示が挿入されました
template-project-config-loaded = プロジェクト設定が { $path } から読み込まれました
template-domain-validator-registered = ドメインバリデーターが登録されました: { $domain }

# ライセンスエラー
error-license-file-not-found = ライセンスファイルが見つかりません: { $path }
error-license-file-invalid = 無効なライセンスファイル ({ $path }): { $reason }
error-license-file-write-failed = ライセンスファイルの書き込みに失敗しました ({ $path }): { $reason }
error-license-signature-invalid = ライセンス署名の検証に失敗しました
error-license-signature-decode-failed = ライセンス署名のデコードに失敗しました: { $reason }
error-license-serialization-failed = ライセンスデータのシリアライズに失敗しました: { $reason }
error-license-signing-failed = ライセンスの署名に失敗しました: { $reason }
error-license-feature-denied = 機能「{ $feature }」は{ $tier }プランでは利用できません
error-license-worker-limit = 要求された{ $requested }ワーカーは{ $tier }プランの上限{ $limit }を超えています
error-license-manager-limit = 要求された{ $requested }マネージャーは{ $tier }プランの上限{ $limit }を超えています。詳細を解除するには設定 → ライセンスでサブスクリプションを有効化してください。
error-license-category-denied = カテゴリ「{ $category }」には{ $required_tier }プランが必要です（現在: { $tier }）
error-license-fingerprint-hostname = ホスト名の取得に失敗しました: { $reason }
error-license-fingerprint-username = ユーザー名の取得に失敗しました: { $reason }

# ライセンス状態
license-trial-active = 試用ライセンス有効（残り{ $days }日）
license-trial-expiring-soon = 試用期間がまもなく終了します（残り{ $days }日）
license-trial-expiring-very-soon = 試用期間の終了が間近です（残り{ $days }日）
license-trial-expired = 試用ライセンスの有効期限が切れました
license-expires-later = ライセンスは{ $days }日後に期限切れになります
license-expires-soon = ライセンスがまもなく期限切れになります（残り{ $days }日）
license-expires-very-soon = ライセンスの期限切れが間近です（残り{ $days }日）
license-expires-today = ライセンスは本日期限切れになります
license-grace-period = ライセンスの有効期限が切れました。猶予期間中です（残り{ $days }日）
license-expired = ライセンスの有効期限が切れました

# プラットフォームサポート — エラーキー
error-platform-unsupported = サポートされていないプラットフォーム: { $platform }
error-platform-wsl-not-detected = WSL環境が検出されませんでした
error-platform-wsl-path-invalid = WSL変換に無効なパス: { $path }
error-platform-layout-no-space = パネルレイアウトにはコンテナが小さすぎます ({ $width }x{ $height })
error-platform-layout-invalid-count = 無効なパネル数: { $count }
error-platform-hotkey-registration-failed = ホットキー登録に失敗しました（競合）: { $shortcut }
error-platform-hotkey-parse-failed = ホットキーショートカットの解析に失敗: { $shortcut }
error-platform-shutdown-timeout = プロセス { $pid } ({ $label }) のシャットダウンがタイムアウトしました
error-platform-shutdown-failed = プロセス { $pid } ({ $label }) のシャットダウンに失敗しました
error-platform-shell-not-found = デフォルトシェルが見つかりません

# プラットフォームサポート — 情報キー
platform-detected = プラットフォーム検出: { $os } ({ $arch })
platform-wsl-detected = WSL検出: { $distro } (WSL{ $version })
platform-wsl-path-translated = パス変換: { $from } → { $to }
platform-layout-calculated = レイアウト計算: { $panels } パネル、{ $rows }x{ $cols } グリッド
platform-layout-optimized = レイアウト最適化: { $utilization }% 使用率
platform-hotkey-registered = ホットキー登録: { $command } → { $shortcut }
platform-hotkey-unregistered = ホットキー解除: { $command }
platform-shutdown-initiated = { $count } プロセスのシャットダウンを開始
platform-shutdown-completed = シャットダウン完了: { $count } プロセス、{ $duration }ms
platform-shell-detected = シェル検出: { $shell } ({ $path })

# メモリエラー
error-memory-not-found = メモリエントリが見つかりません: { $id }
error-memory-duplicate = メモリエントリが重複しています: { $id }
error-memory-persistence-failed = メモリストアの永続化に失敗しました: { $reason }
error-memory-load-failed = メモリストアの読み込みに失敗しました: { $reason }
error-memory-invalid-confidence = 無効な信頼度スコア: { $value }（0.0～1.0の範囲が必要です）
error-memory-store-full = メモリストアが満杯です（最大{ $max }エントリ）
error-memory-invalid-query = 無効なメモリクエリ: { $reason }
error-memory-serialization = メモリのシリアライズに失敗しました: { $reason }
error-memory-invalid-entry = 無効なメモリエントリ: { $reason }
error-memory-session-mismatch = セッション不一致: 期待値 { $expected }、実際値 { $actual }

# メモリ情報
memory-store-created = セッション { $session_id } のメモリストアを作成しました
memory-entry-added = メモリエントリを追加しました: { $title }（タイプ: { $memory_type }）
memory-entry-updated = メモリエントリを更新しました: { $id }
memory-entry-removed = メモリエントリを削除しました: { $id }
memory-store-cleared = メモリストアをクリアしました（{ $count }エントリを削除）
memory-persisted = メモリストアを { $path } に永続化しました
memory-loaded = メモリストアを { $path } から読み込みました（{ $count }エントリ）
memory-query-executed = メモリクエリが { $count } 件の結果を返しました
memory-injected = { $count } 件のメモリを注入しました（{ $tokens } トークン）
memory-stats = メモリ統計: { $total } エントリ、平均信頼度 { $avg_confidence }

# インデックスエラー
error-indexing-parse-failed = { $file } の解析に失敗しました: { $reason }
error-indexing-file-read-failed = ファイル { $file } の読み取りに失敗しました: { $reason }
error-indexing-unsupported-language = ファイル拡張子に対応する言語がサポートされていません: { $extension }
error-indexing-extraction-failed = { $file } のシンボル抽出に失敗しました: { $reason }
error-indexing-graph-cycle-detected = 依存関係の循環が検出されました: { $files }
error-indexing-fingerprint-failed = { $file } のフィンガープリント計算に失敗しました: { $reason }
error-indexing-build-failed = インデックスの構築に失敗しました: { $reason }
error-indexing-update-failed = 増分更新に失敗しました: { $reason }

# インデックス情報
indexing-file-indexed = ファイルをインデックスしました: { $file } ({ $language })
indexing-symbols-extracted = { $file } から { $count } 個のシンボルを抽出しました
indexing-graph-built = 依存関係グラフを構築しました: { $files } ファイル、{ $edges } エッジ
indexing-ranking-computed = { $symbols } 個のシンボルのランキングを計算しました
indexing-repomap-generated = リポジトリマップを生成しました: { $symbols } シンボル、{ $tokens } トークン
indexing-index-built = コードベースインデックスを構築しました: { $files } ファイル、{ $symbols } シンボル
indexing-incremental-update = 増分更新: { $added } 追加、{ $modified } 変更、{ $removed } 削除
indexing-language-registered = 言語が登録されました: { $language }

# コンテキストエラー
error-context-budget-exceeded = コンテキストトークン予算超過: 使用 { $used }、予算 { $budget }
error-context-invalid-allocations = 予算配分の合計は <= 1.0 でなければなりません。合計 { $sum }
error-context-build-failed = タスク { $task_id } のコンテキスト構築に失敗しました: { $reason }
error-context-invalid-format = 無効なコンテキスト形式: { $format }

# コンテキスト情報
context-budget-allocated = トークン予算配分: { $total } トークン ({ $repo_map } リポジトリマップ、{ $files } ファイル、{ $memory } メモリ、{ $task } タスク)
context-files-scored = { $count } ファイルの関連性を評価しました (トップ: { $top_file })
context-chunks-created = { $count } 個のコードチャンクを作成しました ({ $tokens } トークン)
context-assembled = コンテキスト組み立て完了: { $sections } セクション、{ $budget } 予算中 { $tokens } トークン使用
context-injected = ワーカー { $worker_id } にコンテキストを注入しました ({ $tokens } トークン、{ $files } ファイル)
context-skipped = コンテキスト準備をスキップしました: { $reason }

# MCPエラー
error-mcp-parse-failed = JSON-RPCメッセージの解析に失敗しました: { $reason }
error-mcp-invalid-request = 無効なJSON-RPCリクエスト: { $reason }
error-mcp-method-not-found = メソッドが見つかりません: { $method }
error-mcp-invalid-params = 無効なパラメータ: { $reason }
error-mcp-internal-error = MCP内部サーバーエラー: { $reason }
error-mcp-not-initialized = MCPサーバーが初期化されていません
error-mcp-tool-not-found = ツールが見つかりません: { $tool }
error-mcp-tool-execution-failed = ツール「{ $tool }」の実行に失敗しました: { $reason }
error-mcp-transport-error = MCPトランスポートエラー: { $reason }
error-mcp-shutdown-failed = MCPサーバーのシャットダウンに失敗しました: { $reason }

# MCP情報
mcp-server-started = MCPサーバーが起動しました ({ $transport } トランスポート)
mcp-server-stopped = MCPサーバーが停止しました
mcp-client-initialized = MCPクライアントが初期化されました: { $client_name }
mcp-tool-called = ツールが呼び出されました: { $tool }
mcp-tool-completed = ツール「{ $tool }」が { $duration }ms で完了しました
mcp-request-received = リクエストを受信しました: { $method }
mcp-response-sent = レスポンスを送信しました: { $method }
mcp-transport-ready = MCPトランスポート準備完了: { $transport }

# Graph errors
error-graph-entity-not-found = グラフエンティティが見つかりません: { $id }
error-graph-relationship-failed = 関係の追加に失敗しました: { $reason }
error-graph-build-failed = ナレッジグラフの構築に失敗しました: { $reason }
error-graph-update-failed = ナレッジグラフの更新に失敗しました: { $reason }
error-graph-load-failed = { $path } からナレッジグラフの読み込みに失敗しました: { $reason }
error-graph-save-failed = { $path } へのナレッジグラフの保存に失敗しました: { $reason }
error-graph-max-entities-exceeded = ナレッジグラフがエンティティの最大数を超えました: { $count } / { $max }

# Graph info
graph-built = ナレッジグラフを構築しました: { $entities } エンティティ、{ $relationships } 関係
graph-updated = ナレッジグラフを更新しました: { $added } 追加、{ $removed } 削除
graph-entity-added = エンティティをナレッジグラフに追加しました: { $name } ({ $kind })
graph-entity-removed = エンティティをナレッジグラフから削除しました: { $name }
graph-persisted = ナレッジグラフを { $path } に保存しました
graph-loaded = ナレッジグラフを { $path } から読み込みました ({ $entities } エンティティ)
graph-query-executed = グラフクエリを { $ms }ms で実行しました。{ $results } 件の結果

# プラットフォームAPIエラー
error-platform-api-request-failed = プラットフォームAPIリクエストが失敗しました: { $reason }
error-platform-api-unauthorized = プラットフォームAPI認証に失敗しました — channel_api_keyを確認してください
error-platform-api-not-found = プラットフォームリソースが見つかりません: { $resource }
error-platform-api-rate-limited = プラットフォームAPIのレート制限に達しました — { $seconds }秒後に再試行してください
error-platform-api-server-error = プラットフォームサーバーエラー ({ $status }): { $message }
error-platform-trial-not-eligible = このデバイスは試用の対象外です: { $reason }
error-platform-activation-failed = ライセンスのアクティベーションに失敗しました: { $reason }
error-platform-validation-failed = ライセンスの検証に失敗しました: { $reason }
error-platform-deactivation-failed = デバイスの無効化に失敗しました: { $reason }
error-platform-cache-read-failed = { $path } からライセンスキャッシュの読み取りに失敗しました: { $reason }
error-platform-cache-write-failed = { $path } へのライセンスキャッシュの書き込みに失敗しました: { $reason }
error-platform-cache-decrypt-failed = ライセンスキャッシュの復号に失敗しました（鍵の不一致またはデータ破損）
error-platform-not-configured = プラットフォーム連携が設定されていません — 設定でplatform_base_urlを指定してください

# プラットフォームAPI情報
platform-api-trial-activated = 試用が有効化されました: { $tier } プラン、{ $days } 日間
platform-api-license-activated = ライセンスが有効化されました: { $tier } プラン（アクティベーション { $activation_id }）
platform-api-license-validated = ライセンスが検証されました: { $tier } プラン、残り { $days } 日
platform-api-heartbeat-sent = ハートビートを送信しました（アクティベーション { $activation_id }）
platform-api-device-deactivated = デバイスがライセンスから無効化されました
platform-api-cache-updated = ライセンスキャッシュが { $path } で更新されました
platform-api-offline-fallback = プラットフォームに到達できません。キャッシュされたライセンスを使用します（{ $days_ago } 日前にキャッシュ）

# メッセージングエラー
error-messaging-not-registered = メッセージングクライアントが登録されていません
error-messaging-registration-failed = メッセージング登録に失敗しました: { $reason }
error-messaging-send-failed = メッセージの送信に失敗しました: { $reason }
error-messaging-poll-failed = メッセージのポーリングに失敗しました: { $reason }
error-messaging-ack-failed = メッセージ { $message_id } の確認応答に失敗しました: { $reason }
error-messaging-disabled = この許可証ではメッセージングが無効です

# メッセージング情報
messaging-registered = デバイス { $device_id } のメッセージングが登録されました
messaging-unregistered = メッセージングが登録解除されました
messaging-message-received = メッセージを受信しました: { $subject }（タイプ: { $message_type }）
messaging-message-sent = メッセージを送信しました（ID: { $message_id }）
messaging-poll-completed = メッセージポーリング完了: { $count } 件の新着メッセージ

# Provider credential descriptions
credential-xai-api-key = Grok 用 xAI API キー (XAI_API_KEY)
credential-openai-api-key = OpenAI API キー (OPENAI_API_KEY)
credential-google-api-key = Gemini 用 Google API キー (GOOGLE_API_KEY)
credential-gh-auth = gh CLI による GitHub 認証 (gh auth login)

# Built-in category names
category-SoftwareDevelopment = ソフトウェア開発
category-LinuxDevelopment = Linux開発
category-macOSDevelopment = macOS開発
category-PythonDevelopment = Python開発
category-AIFrameworks = AI・MLフレームワーク
category-GraphQL = GraphQLフレームワーク
category-DataScience = データサイエンスとアナリティクス
category-Legal = 法務 / パラリーガル
category-Music = 音楽制作
category-PhysicalSystems = 物理システムと現象
category-BacteriaScience = 細菌科学と微生物学
category-NursingScience = 看護科学と臨床実践
category-ElectronDevelopment = Electronデスクトップ開発
category-GameDevelopment = ゲーム開発
category-3DModeling = 3Dモデリングとデジタルコンテンツ制作
category-Custom = カスタムテンプレート

# Built-in category descriptions
category-SoftwareDevelopment-desc = アプリケーション、API、データベース、スクリプト作成用テンプレート
category-LinuxDevelopment-desc = Linuxシステム管理、シェルスクリプト、サーバー開発用テンプレート
category-macOSDevelopment-desc = macOSアプリケーション、Swift/Objective-C開発、Appleフレームワーク用テンプレート
category-PythonDevelopment-desc = Pythonアプリケーション、スクリプト、Webフレームワーク、自動化用テンプレート
category-AIFrameworks-desc = AIエージェント、LLMオーケストレーション、チャットボット、MLアプリケーション用テンプレート
category-GraphQL-desc = GraphQLサーバー、クライアント、API開発用テンプレート
category-DataScience-desc = データサイエンスライフサイクル用テンプレート：数学、データエンジニアリング、ML、ディープラーニング、MLOps
category-Legal-desc = 法律文書処理、リサーチ、ケース管理用テンプレート
category-Music-desc = DAW、プラグイン開発、モジュラーシンセシス、ハードウェア連携用テンプレート
category-PhysicalSystems-desc = 産業物理学、プロセス監視、制御システム、予測分析用テンプレート
category-BacteriaScience-desc = 微生物学、ゲノミクス、メタゲノミクス、抗菌薬耐性、診断用テンプレート
category-NursingScience-desc = 看護教育、臨床実践、患者ケア、ヘルスケアアナリティクス用テンプレート
category-ElectronDevelopment-desc = Electronとモダンツールによるクロスプラットフォームデスクトップアプリケーション用テンプレート
category-GameDevelopment-desc = ゲームエンジン、フレームワーク、インタラクティブエンターテインメント開発用テンプレート
category-3DModeling-desc = 3Dモデリング、VFX、アニメーション、デジタルコンテンツ制作ツール用テンプレート
category-Custom-desc = ユーザー作成のカスタムテンプレート

# Provider status
provider-not-installed = プロバイダー { $provider } には { $binary } が必要ですが、インストールされていません
provider-binary-found = { $binary } が { $path } に見つかりました
provider-test-timeout = 接続テストが { $seconds } 秒後にタイムアウトしました
provider-test-failed = プロバイダーテスト失敗: { $error }
provider-env-saved = { $provider } の { $env_var } を保存しました

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
error-orchestration-quorum-not-met = マネージャークォーラム未達成: { $received }/{ $expected } の提案を受信 (最小: { $min_quorum })
orchestration-file-conflict-serialized = タスク { $later_task } を { $earlier_task } の後にシリアル化 — 両方がファイル '{ $file }' を要求
orchestration-build-check-failed = ワーカー { $worker_id } のワーカーごとの { $project_type } ビルドチェック失敗
orchestration-build-check-passed = ワーカー { $worker_id } のワーカーごとの { $project_type } ビルドチェック合格
