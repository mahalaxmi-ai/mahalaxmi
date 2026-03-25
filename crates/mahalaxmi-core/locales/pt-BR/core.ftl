# Erros
error-config-file-not-found = Arquivo de configuração não encontrado em { $path }
error-config-parse-failed = Falha ao analisar a configuração: { $reason }
error-config-validation-failed = Validação da configuração falhou: { $details }
error-locale-not-supported = O idioma "{ $locale }" não é suportado
error-log-init-failed = Falha ao inicializar o registro: { $reason }
error-log-dir-create-failed = Falha ao criar o diretório de registro em { $path }: { $reason }
error-app-launch-failed = Falha ao iniciar o aplicativo: { $reason }

# Validação
validation-invalid-log-level = Nível de registro inválido "{ $level }". Níveis válidos: { $valid }
validation-workers-out-of-range = max_concurrent_workers deve estar entre { $min } e { $max }, recebido: { $value }
validation-manager-timeout-too-low = manager_timeout_seconds deve ser no mínimo { $min }, recebido: { $value }
validation-worker-timeout-too-low = worker_timeout_seconds deve ser no mínimo { $min }, recebido: { $value }
validation-offline-grace-too-low = offline_grace_days deve ser no mínimo { $min }, recebido: { $value }
validation-invalid-consensus-strategy = Estratégia de consenso inválida "{ $value }". Estratégias válidas: { $valid }
validation-invalid-data-directory = data_directory contém caracteres inválidos
validation-empty-default-provider = providers.default_provider não deve estar vazio
validation-invalid-theme = Tema inválido "{ $value }". Temas válidos: { $valid }
validation-font-size-out-of-range = terminal_font_size deve estar entre { $min } e { $max }, recebido: { $value }
validation-invalid-max-batch-retries = max_batch_retries deve ser >= 1, recebido: { $value }
validation-invalid-max-total-batches = max_total_batches deve ser >= 2, recebido: { $value }
validation-invalid-stall-detection-threshold = stall_detection_threshold deve ser >= 2, recebido: { $value }

# Configuração
config-loaded-successfully = Configuração carregada de { $path }
config-using-default = Nenhum arquivo de configuração encontrado, usando padrões
config-env-override = Valor de configuração substituído pela variável de ambiente { $var }
config-env-override-invalid = A variável de ambiente { $var } tem valor inválido "{ $value }", mantendo o valor configurado
config-generated-successfully = Configuração padrão gerada em { $path }
config-already-exists = O arquivo de configuração já existe em { $path }

# Registro
logging-initialized = Registro inicializado no nível { $level }
logging-rust-log-override = Variável de ambiente RUST_LOG detectada, substituindo o nível configurado
logging-file-path = Arquivo de registro: { $path }
logging-dir-create-failed-fallback = Falha ao criar o diretório de registro { $path }, utilizando apenas registro no console

# PTY
pty-open-failed = Falha ao abrir pseudoterminal: { $reason }
pty-spawn-failed = Falha ao executar "{ $program }" no PTY: { $reason }
pty-write-failed = Falha ao escrever no terminal { $terminal_id }: { $reason }
pty-read-failed = Falha ao ler do terminal { $terminal_id }: { $reason }
pty-resize-failed = Falha ao redimensionar o terminal { $terminal_id } para { $rows }x{ $cols }: { $reason }
pty-wait-failed = Falha ao verificar o status do processo do terminal { $terminal_id }: { $reason }
pty-kill-failed = Falha ao encerrar o processo no terminal { $terminal_id }: { $reason }

# Aplicação
app-starting = Mahalaxmi v{ $version } iniciando
app-ready = Mahalaxmi está pronto
app-shutting-down = Mahalaxmi encerrando

# Credenciais
credential-anthropic-api-key = Chave de API da Anthropic para Claude Code
credential-generic-api-key = Chave de API para { $provider }
credential-aws-iam-role = Função AWS IAM para { $provider }
credential-oauth-token = Token OAuth para { $provider }

# Provedor
error-provider-credentials-missing = Credenciais do { $provider } ausentes: variável de ambiente { $env_var } não está definida
error-provider-credentials-invalid = As credenciais do { $provider } são inválidas: { $reason }
error-provider-not-found = Provedor "{ $provider_id }" não encontrado no registro
error-provider-no-default = Nenhum provedor de IA padrão configurado
error-provider-command-build-failed = Falha ao construir o comando do { $provider }: { $reason }
provider-registered = Provedor "{ $provider }" registrado com ID "{ $id }"
provider-set-default = Provedor padrão definido como "{ $provider }"
provider-credentials-valid = Credenciais do { $provider } validadas com sucesso
provider-validating = Validando credenciais do { $provider }
provider-list-header = Provedores de IA registrados

# PTY (estendido)
error-pty-open-failed = Falha ao abrir PTY: { $reason }
error-pty-spawn-failed = Falha ao executar "{ $program }" no PTY: { $reason }
error-pty-write-failed = Falha ao escrever no terminal { $terminal_id }: { $reason }
error-pty-read-failed = Falha ao ler do terminal { $terminal_id }: { $reason }
error-pty-resize-failed = Falha ao redimensionar o terminal { $terminal_id } para { $rows }x{ $cols }: { $reason }
error-pty-kill-failed = Falha ao encerrar o processo do terminal { $terminal_id }: { $reason }
error-pty-wait-failed = Falha ao verificar o status do terminal { $terminal_id }: { $reason }
error-pty-terminal-not-found = Terminal { $terminal_id } não encontrado
error-pty-max-concurrent-reached = Limite máximo de terminais simultâneos ({ $max }) atingido
pty-process-spawned = Processo "{ $program }" iniciado no terminal { $terminal_id }
pty-process-exited = Processo do terminal { $terminal_id } encerrado com código { $exit_code }
pty-session-closed = Sessão do terminal { $terminal_id } encerrada
pty-resized = Terminal { $terminal_id } redimensionado para { $rows }x{ $cols }
pty-reader-eof = Leitor do terminal { $terminal_id } atingiu o fim do fluxo
pty-reader-error = Erro do leitor do terminal { $terminal_id }: { $reason }

# Erros de orquestração
error-orchestration-invalid-transition = Transição de estado inválida de { $from } para { $to }
error-orchestration-circular-dependency = Dependência circular detectada: { $cycle }
error-orchestration-worker-not-found = Trabalhador { $worker_id } não encontrado na fila
error-orchestration-max-retries-exceeded = O trabalhador { $worker_id } excedeu o máximo de tentativas ({ $max_retries })
error-orchestration-no-proposals = Nenhuma proposta de gerente recebida
error-orchestration-plan-validation-failed = Validação do plano de execução falhou: { $errors }
error-orchestration-consensus-failed = Motor de consenso falhou: { $reason }
error-orchestration-queue-full = Fila de trabalhadores está cheia (máximo { $max })
error-orchestration-manager-timeout = Gerente { $manager_id } expirou após { $timeout }s
error-orchestration-worker-timeout = Trabalhador { $worker_id } expirou após { $timeout }s

# Informações de orquestração
orchestration-cycle-started = Ciclo de orquestração { $cycle_id } iniciado
orchestration-state-changed = Estado alterado: { $from } -> { $to }
orchestration-manager-completed = Gerente { $manager_id } concluído com { $task_count } tarefas
orchestration-consensus-reached = Consenso alcançado: { $agreed } aprovadas, { $dissenting } rejeitadas
orchestration-plan-created = Plano de execução criado: { $phases } fases, { $workers } trabalhadores
orchestration-worker-started = Trabalhador { $worker_id } iniciado: { $task }
orchestration-worker-completed = Trabalhador { $worker_id } concluído em { $duration }ms
orchestration-worker-failed = Trabalhador { $worker_id } falhou: { $error }
orchestration-cycle-completed = Ciclo concluído em { $duration }ms (taxa de sucesso: { $success_rate })
orchestration-worker-retrying = Trabalhador { $worker_id } tentando novamente (tentativa { $attempt }/{ $max })

# Erros de detecção
error-detection-rule-compile-failed = Falha ao compilar padrão de regra de detecção: { $reason }
error-detection-no-rules-loaded = Nenhuma regra de detecção carregada
error-detection-invalid-pattern = Padrão de detecção inválido "{ $pattern }": { $reason }

# Informações de detecção
detection-rule-matched = Regra de detecção "{ $rule }" correspondeu, ação: { $action }
detection-rule-cooldown = Regra de detecção "{ $rule }" suprimida por tempo de espera ({ $remaining_ms }ms restantes)
detection-rules-loaded = { $count } regras de detecção carregadas
detection-provider-rules-applied = { $count } reglas aplicadas para provedor { $provider }
detection-error-pattern-detected = Padrão de erro detectado: "{ $pattern }" (visto { $count } vezes)
detection-root-cause-hypothesis = Hipótese de causa raiz: { $category } (confiança: { $confidence })
detection-recurring-error = Erro recorrente: "{ $message }" (ocorreu { $count } vezes)
detection-action-executed = Ação { $action } executada para regra "{ $rule }"
detection-cooldowns-reset = Tempos de espera reiniciados para { $rule_count } regras

# Erros de modelos
error-template-not-found = Modelo { $template_id } não encontrado
error-template-category-not-found = Categoria de modelo { $category_id } não encontrada
error-template-composition-failed = A composição do modelo falhou: { $reason }
error-template-include-not-found = Arquivo incluído não encontrado: { $path }
error-template-circular-include = Inclusão circular detectada (profundidade máxima { $depth } excedida)
error-template-placeholder-unresolved = Espaço reservado não resolvido: ${ $placeholder }
error-template-validation-failed = A validação do modelo falhou com { $count } erros
error-template-activation-failed = A ativação do modelo falhou: { $reason }
error-template-catalog-load-failed = Falha ao carregar o catálogo de modelos: { $path }
error-template-invalid-version = Formato de versão de modelo inválido: { $version }

# Informações de modelos
template-catalog-loaded = Catálogo de modelos carregado com { $count } modelos
template-activated = Modelo { $template_id } ativado com sucesso
template-composition-complete = Composição concluída: { $included } incluídos, { $overridden } substituídos
template-placeholders-resolved = { $count } espaços reservados resolvidos
template-validation-passed = Validação do modelo aprovada para domínio { $domain }
template-validation-warnings = Validação do modelo concluída com { $count } avisos
template-include-resolved = Inclusão resolvida: { $path }
template-provider-instructions-injected = Instruções do provedor injetadas para { $provider }
template-project-config-loaded = Configuração do projeto carregada de { $path }
template-domain-validator-registered = Validador de domínio registrado: { $domain }

# Erros de licença
error-license-file-not-found = Arquivo de licença não encontrado em { $path }
error-license-file-invalid = Arquivo de licença inválido em { $path }: { $reason }
error-license-file-write-failed = Falha ao gravar arquivo de licença em { $path }: { $reason }
error-license-signature-invalid = Falha na verificação da assinatura da licença
error-license-signature-decode-failed = Falha ao decodificar assinatura da licença: { $reason }
error-license-serialization-failed = Falha ao serializar dados da licença: { $reason }
error-license-signing-failed = Falha ao assinar licença: { $reason }
error-license-feature-denied = O recurso '{ $feature }' não está disponível no plano { $tier }
error-license-worker-limit = Os { $requested } trabalhadores solicitados excedem o limite de { $limit } do plano { $tier }
error-license-manager-limit = Os { $requested } gerenciadores solicitados excedem o limite de { $limit } do plano { $tier }. Ative sua assinatura em Configurações → Licença para desbloquear mais.
error-license-category-denied = A categoria '{ $category }' requer o plano { $required_tier } (atual: { $tier })
error-license-fingerprint-hostname = Falha ao determinar o nome do host: { $reason }
error-license-fingerprint-username = Falha ao determinar o nome de usuário: { $reason }

# Status da licença
license-trial-active = Licença de avaliação ativa ({ $days } dias restantes)
license-trial-expiring-soon = Avaliação expirando em breve ({ $days } dias restantes)
license-trial-expiring-very-soon = Avaliação expirando muito em breve ({ $days } dias restantes)
license-trial-expired = A licença de avaliação expirou
license-expires-later = A licença expira em { $days } dias
license-expires-soon = Licença expirando em breve ({ $days } dias restantes)
license-expires-very-soon = Licença expirando muito em breve ({ $days } dias restantes)
license-expires-today = A licença expira hoje
license-grace-period = Licença expirada, período de carência ativo ({ $days } dias restantes)
license-expired = A licença expirou

# Suporte à plataforma — chaves de erro
error-platform-unsupported = Plataforma não suportada: { $platform }
error-platform-wsl-not-detected = Ambiente WSL não detectado
error-platform-wsl-path-invalid = Caminho inválido para tradução WSL: { $path }
error-platform-layout-no-space = Contêiner muito pequeno para layout de painéis ({ $width }x{ $height })
error-platform-layout-invalid-count = Número de painéis inválido: { $count }
error-platform-hotkey-registration-failed = Falha ao registrar atalho (conflito): { $shortcut }
error-platform-hotkey-parse-failed = Falha ao analisar atalho de teclado: { $shortcut }
error-platform-shutdown-timeout = Tempo esgotado para encerrar processo { $pid } ({ $label })
error-platform-shutdown-failed = Falha ao encerrar processo { $pid } ({ $label })
error-platform-shell-not-found = Shell padrão não encontrado

# Suporte à plataforma — chaves informativas
platform-detected = Plataforma detectada: { $os } ({ $arch })
platform-wsl-detected = WSL detectado: { $distro } (WSL{ $version })
platform-wsl-path-translated = Caminho traduzido: { $from } → { $to }
platform-layout-calculated = Layout calculado: { $panels } painéis em grade { $rows }x{ $cols }
platform-layout-optimized = Layout otimizado: { $utilization }% utilização
platform-hotkey-registered = Atalho registrado: { $command } → { $shortcut }
platform-hotkey-unregistered = Atalho removido: { $command }
platform-shutdown-initiated = Encerramento iniciado para { $count } processos
platform-shutdown-completed = Encerramento concluído: { $count } processos em { $duration }ms
platform-shell-detected = Shell detectado: { $shell } ({ $path })

# Erros de memória
error-memory-not-found = Entrada de memória não encontrada: { $id }
error-memory-duplicate = Entrada de memória duplicada: { $id }
error-memory-persistence-failed = Falha ao persistir armazenamento de memória: { $reason }
error-memory-load-failed = Falha ao carregar armazenamento de memória: { $reason }
error-memory-invalid-confidence = Pontuação de confiança inválida: { $value } (deve ser entre 0.0 e 1.0)
error-memory-store-full = Armazenamento de memória cheio (máximo de { $max } entradas)
error-memory-invalid-query = Consulta de memória inválida: { $reason }
error-memory-serialization = Falha na serialização de memória: { $reason }
error-memory-invalid-entry = Entrada de memória inválida: { $reason }
error-memory-session-mismatch = Incompatibilidade de sessão: esperado { $expected }, obtido { $actual }

# Informações de memória
memory-store-created = Armazenamento de memória criado para sessão { $session_id }
memory-entry-added = Entrada de memória adicionada: { $title } (tipo: { $memory_type })
memory-entry-updated = Entrada de memória atualizada: { $id }
memory-entry-removed = Entrada de memória removida: { $id }
memory-store-cleared = Armazenamento de memória limpo ({ $count } entradas removidas)
memory-persisted = Armazenamento de memória persistido em { $path }
memory-loaded = Armazenamento de memória carregado de { $path } ({ $count } entradas)
memory-query-executed = Consulta de memória retornou { $count } resultados
memory-injected = { $count } memórias injetadas ({ $tokens } tokens)
memory-stats = Estatísticas de memória: { $total } entradas, confiança média { $avg_confidence }

# Erros de indexação
error-indexing-parse-failed = Falha ao analisar { $file }: { $reason }
error-indexing-file-read-failed = Falha ao ler o arquivo { $file }: { $reason }
error-indexing-unsupported-language = Linguagem não suportada para a extensão de arquivo: { $extension }
error-indexing-extraction-failed = A extração de símbolos falhou para { $file }: { $reason }
error-indexing-graph-cycle-detected = Ciclo de dependência detectado: { $files }
error-indexing-fingerprint-failed = Falha ao calcular a impressão digital para { $file }: { $reason }
error-indexing-build-failed = A construção do índice falhou: { $reason }
error-indexing-update-failed = A atualização incremental falhou: { $reason }

# Informações de indexação
indexing-file-indexed = Arquivo indexado: { $file } ({ $language })
indexing-symbols-extracted = { $count } símbolos extraídos de { $file }
indexing-graph-built = Grafo de dependências construído: { $files } arquivos, { $edges } arestas
indexing-ranking-computed = Classificação calculada para { $symbols } símbolos
indexing-repomap-generated = Mapa do repositório gerado: { $symbols } símbolos, { $tokens } tokens
indexing-index-built = Índice do código fonte construído: { $files } arquivos, { $symbols } símbolos
indexing-incremental-update = Atualização incremental: { $added } adicionados, { $modified } modificados, { $removed } removidos
indexing-language-registered = Linguagem registrada: { $language }

# Erros de contexto
error-context-budget-exceeded = Orçamento de tokens de contexto excedido: usados { $used }, orçamento { $budget }
error-context-invalid-allocations = As alocações do orçamento devem somar <= 1.0, obtido { $sum }
error-context-build-failed = A construção do contexto falhou para a tarefa { $task_id }: { $reason }
error-context-invalid-format = Formato de contexto inválido: { $format }

# Informações de contexto
context-budget-allocated = Orçamento de tokens alocado: { $total } tokens ({ $repo_map } mapa do repositório, { $files } arquivos, { $memory } memória, { $task } tarefa)
context-files-scored = { $count } arquivos avaliados por relevância (principal: { $top_file })
context-chunks-created = { $count } fragmentos de código criados ({ $tokens } tokens)
context-assembled = Contexto montado: { $sections } seções, { $tokens } tokens usados de { $budget } orçamento
context-injected = Contexto injetado para o worker { $worker_id } ({ $tokens } tokens, { $files } arquivos)
context-skipped = Preparação de contexto ignorada: { $reason }

# Erros MCP
error-mcp-parse-failed = Falha ao analisar mensagem JSON-RPC: { $reason }
error-mcp-invalid-request = Requisição JSON-RPC inválida: { $reason }
error-mcp-method-not-found = Método não encontrado: { $method }
error-mcp-invalid-params = Parâmetros inválidos: { $reason }
error-mcp-internal-error = Erro interno do servidor MCP: { $reason }
error-mcp-not-initialized = O servidor MCP não foi inicializado
error-mcp-tool-not-found = Ferramenta não encontrada: { $tool }
error-mcp-tool-execution-failed = Execução da ferramenta "{ $tool }" falhou: { $reason }
error-mcp-transport-error = Erro de transporte MCP: { $reason }
error-mcp-shutdown-failed = Falha ao encerrar o servidor MCP: { $reason }

# Informações MCP
mcp-server-started = Servidor MCP iniciado (transporte { $transport })
mcp-server-stopped = Servidor MCP encerrado
mcp-client-initialized = Cliente MCP inicializado: { $client_name }
mcp-tool-called = Ferramenta chamada: { $tool }
mcp-tool-completed = Ferramenta "{ $tool }" concluída em { $duration }ms
mcp-request-received = Requisição recebida: { $method }
mcp-response-sent = Resposta enviada: { $method }
mcp-transport-ready = Transporte MCP pronto: { $transport }

# Graph errors
error-graph-entity-not-found = Entidade do grafo não encontrada: { $id }
error-graph-relationship-failed = Falha ao adicionar relacionamento: { $reason }
error-graph-build-failed = Falha ao construir o grafo de conhecimento: { $reason }
error-graph-update-failed = Falha ao atualizar o grafo de conhecimento: { $reason }
error-graph-load-failed = Falha ao carregar o grafo de conhecimento de { $path }: { $reason }
error-graph-save-failed = Falha ao salvar o grafo de conhecimento em { $path }: { $reason }
error-graph-max-entities-exceeded = O grafo de conhecimento excedeu o limite máximo de entidades: { $count } / { $max }

# Graph info
graph-built = Grafo de conhecimento construído com { $entities } entidades e { $relationships } relacionamentos
graph-updated = Grafo de conhecimento atualizado: { $added } adicionadas, { $removed } removidas
graph-entity-added = Entidade adicionada ao grafo de conhecimento: { $name } ({ $kind })
graph-entity-removed = Entidade removida do grafo de conhecimento: { $name }
graph-persisted = Grafo de conhecimento persistido em { $path }
graph-loaded = Grafo de conhecimento carregado de { $path } ({ $entities } entidades)
graph-query-executed = Consulta do grafo executada em { $ms }ms, { $results } resultados

# Erros de API da plataforma
error-platform-api-request-failed = A requisição à API da plataforma falhou: { $reason }
error-platform-api-unauthorized = A autenticação da API da plataforma falhou — verifique channel_api_key
error-platform-api-not-found = Recurso da plataforma não encontrado: { $resource }
error-platform-api-rate-limited = Limite de taxa da API da plataforma atingido — tente novamente após { $seconds }s
error-platform-api-server-error = Erro do servidor da plataforma ({ $status }): { $message }
error-platform-trial-not-eligible = Este dispositivo não é elegível para avaliação: { $reason }
error-platform-activation-failed = A ativação da licença falhou: { $reason }
error-platform-validation-failed = A validação da licença falhou: { $reason }
error-platform-deactivation-failed = A desativação do dispositivo falhou: { $reason }
error-platform-cache-read-failed = Falha ao ler o cache de licença de { $path }: { $reason }
error-platform-cache-write-failed = Falha ao gravar o cache de licença em { $path }: { $reason }
error-platform-cache-decrypt-failed = Falha ao descriptografar o cache de licença (chave incorreta ou corrupção)
error-platform-not-configured = A integração da plataforma não está configurada — defina platform_base_url na configuração

# Informações de API da plataforma
platform-api-trial-activated = Avaliação ativada: plano { $tier }, { $days } dias
platform-api-license-activated = Licença ativada: plano { $tier } (ativação { $activation_id })
platform-api-license-validated = Licença validada: plano { $tier }, { $days } dias restantes
platform-api-heartbeat-sent = Heartbeat enviado (ativação { $activation_id })
platform-api-device-deactivated = Dispositivo desativado da licença
platform-api-cache-updated = Cache de licença atualizado em { $path }
platform-api-offline-fallback = Plataforma inacessível, usando licença em cache (armazenada há { $days_ago } dias)

# Erros de mensageria
error-messaging-not-registered = O cliente de mensageria não está registrado
error-messaging-registration-failed = O registro de mensageria falhou: { $reason }
error-messaging-send-failed = Falha ao enviar mensagem: { $reason }
error-messaging-poll-failed = Falha ao consultar mensagens: { $reason }
error-messaging-ack-failed = Falha ao confirmar recebimento da mensagem { $message_id }: { $reason }
error-messaging-disabled = A mensageria está desabilitada para esta licença

# Informações de mensageria
messaging-registered = Mensageria registrada para o dispositivo { $device_id }
messaging-unregistered = Mensageria desregistrada
messaging-message-received = Mensagem recebida: { $subject } (tipo: { $message_type })
messaging-message-sent = Mensagem enviada (id: { $message_id })
messaging-poll-completed = Consulta de mensagens concluída: { $count } novas mensagens

# Provider credential descriptions
credential-xai-api-key = Chave API xAI para Grok (XAI_API_KEY)
credential-openai-api-key = Chave API OpenAI (OPENAI_API_KEY)
credential-google-api-key = Chave API Google para Gemini (GOOGLE_API_KEY)
credential-gh-auth = Autenticação GitHub via gh CLI (gh auth login)

# Built-in category names
category-SoftwareDevelopment = Desenvolvimento de Software
category-LinuxDevelopment = Desenvolvimento Linux
category-macOSDevelopment = Desenvolvimento macOS
category-PythonDevelopment = Desenvolvimento Python
category-AIFrameworks = Frameworks de IA e ML
category-GraphQL = Frameworks GraphQL
category-DataScience = Ciência de Dados e Análise
category-Legal = Jurídico / Paralegal
category-Music = Produção Musical
category-PhysicalSystems = Sistemas Físicos e Fenômenos
category-BacteriaScience = Ciência de Bactérias e Microbiologia
category-NursingScience = Ciência de Enfermagem e Prática Clínica
category-ElectronDevelopment = Desenvolvimento Desktop com Electron
category-GameDevelopment = Desenvolvimento de Jogos
category-3DModeling = Modelagem 3D e Criação de Conteúdo Digital
category-Custom = Modelos Personalizados

# Built-in category descriptions
category-SoftwareDevelopment-desc = Modelos para criação de aplicações, APIs, bancos de dados e scripts
category-LinuxDevelopment-desc = Modelos para administração de sistemas Linux, scripts shell e desenvolvimento de servidores
category-macOSDevelopment-desc = Modelos para aplicações macOS, desenvolvimento Swift/Objective-C e frameworks Apple
category-PythonDevelopment-desc = Modelos para aplicações Python, scripts, frameworks web e automação
category-AIFrameworks-desc = Modelos para agentes de IA, orquestração de LLM, chatbots e aplicações de ML
category-GraphQL-desc = Modelos para servidores GraphQL, clientes e desenvolvimento de APIs
category-DataScience-desc = Modelos para o ciclo de vida de ciência de dados: matemática, engenharia de dados, ML, aprendizado profundo, MLOps
category-Legal-desc = Modelos para processamento de documentos jurídicos, pesquisa e gerenciamento de casos
category-Music-desc = Modelos para DAWs, desenvolvimento de plugins, síntese modular e integração de hardware
category-PhysicalSystems-desc = Modelos para física industrial, monitoramento de processos, sistemas de controle e análise preditiva
category-BacteriaScience-desc = Modelos para microbiologia, genômica, metagenômica, resistência antimicrobiana e diagnósticos
category-NursingScience-desc = Modelos para educação em enfermagem, prática clínica, cuidado ao paciente e análise de saúde
category-ElectronDevelopment-desc = Modelos para aplicações desktop multiplataforma com Electron e ferramentas modernas
category-GameDevelopment-desc = Modelos para motores de jogo, frameworks e desenvolvimento de entretenimento interativo
category-3DModeling-desc = Modelos para modelagem 3D, VFX, animação e ferramentas de criação de conteúdo digital
category-Custom-desc = Modelos personalizados criados pelo usuário

# Provider status
provider-not-installed = O provedor { $provider } requer { $binary } que não está instalado
provider-binary-found = { $binary } encontrado em { $path }
provider-test-timeout = O teste de conexão expirou após { $seconds } segundos
provider-test-failed = Teste do provedor falhou: { $error }
provider-env-saved = { $env_var } salva para { $provider }

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
error-orchestration-quorum-not-met = Quórum de gerentes não atingido: { $received } de { $expected } propostas recebidas (mínimo: { $min_quorum })
orchestration-file-conflict-serialized = Tarefa { $later_task } serializada após { $earlier_task } — ambas reivindicam o arquivo '{ $file }'
orchestration-build-check-failed = Verificação de compilação { $project_type } por trabalhador { $worker_id } falhou
orchestration-build-check-passed = Verificação de compilação { $project_type } por trabalhador { $worker_id } aprovada
