# Protocolo de Federação e Orquestração Mahalaxmi

**MFOP v1.0** · Rascunho para Revisão por Pares

| | |
|---|---|
| Date | Março de 2026 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Flórida, EUA |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## Status deste Memorando

Este documento é um rascunho pré-publicação da especificação do Protocolo de Federação e Orquestração Mahalaxmi (MFOP), versão 1.0. Ele é distribuído para revisão por pares e para solicitar comentários. Este documento descreve um protocolo para orquestração de IA distribuída federada em nós de computação heterogêneos, com roteamento ciente de zonas de conformidade, recibos de faturamento assinados criptograficamente e liquidação econômica configurável.

Comentários e perguntas devem ser direcionados ao autor em Ami.nunez@mahalaxmi.ai. O rascunho atual e os tópicos de discussão são mantidos em https://mahalaxmi.ai/mfop/draft. Os tópicos de discussão estão em https://mahalaxmi.ai/mfop/discuss.

## Aviso de Direitos Autorais

Copyright © 2026 ThriveTech Services LLC. Todos os direitos reservados. É concedida permissão para copiar, distribuir e usar este documento em qualquer meio sem custo, desde que a atribuição ao autor, o título do documento e este aviso de direitos autorais sejam preservados em todas as cópias e obras derivadas.

## Resumo

Este documento define o Protocolo de Federação e Orquestração Mahalaxmi (MFOP), um protocolo para coordenar a execução paralela de agentes de IA em uma rede distribuída de nós de computação heterogêneos. O MFOP especifica identidade e registro de nós, anúncio de capacidades, roteamento de tarefas ciente de zonas de conformidade, particionamento semântico de entradas, recibos de faturamento assinados criptograficamente, liquidação econômica configurável e um modelo de segurança em camadas usando validação de políticas de segurança de IA e isolamento de sandbox de execução.

O MFOP é projetado para operar em três configurações de implantação simultâneas: malhas empresariais privadas, nas quais os nós são de propriedade e operados por uma única organização; pools de nuvem gerenciados pelo provedor da plataforma; e marketplaces comunitários abertos, nos quais qualquer operador de nó pode contribuir com computação em troca de liquidação econômica. O protocolo é agnóstico em relação ao provedor de modelo de IA subjacente e foi projetado para evoluir junto com o cenário de segurança e conformidade de IA.

## 1. Introdução

O crescimento das implantações de modelos de linguagem de grande escala (LLM) em ambientes empresariais criou a necessidade de uma camada de coordenação capaz de abranger infraestrutura de computação heterogênea, ao mesmo tempo em que satisfaz requisitos de conformidade, faturamento e segurança que variam por jurisdição e setor.

O MFOP atende a essa necessidade definindo um protocolo para orquestração federada de IA. Uma federação consiste em um ou mais nós de computação, cada um dos quais pode ser operado por diferentes entidades sob diferentes regimes de conformidade. Um submetente — um usuário, uma aplicação ou um sistema automatizado — apresenta uma tarefa à federação. A federação roteia a tarefa para um nó apropriado com base nos requisitos de zona de conformidade da tarefa, no anúncio de capacidades do nó e nos termos econômicos vigentes.

Esta especificação define o protocolo de comunicação, os formatos de dados, os mecanismos criptográficos e os requisitos comportamentais para todos os componentes de uma federação MFOP em conformidade.

## 2. Terminologia

As palavras-chave "DEVE", "NÃO DEVE", "OBRIGATÓRIO", "DEVERÁ", "NÃO DEVERÁ", "DEVERIA", "NÃO DEVERIA", "RECOMENDADO", "NÃO RECOMENDADO", "PODE" e "OPCIONAL" neste documento devem ser interpretadas conforme descrito no BCP 14 [RFC2119] [RFC8174].

**Federação** — Um agrupamento lógico de um ou mais nós de computação em conformidade com o MFOP, operando sob uma configuração de governança compartilhada.

**Nó** — Um recurso de computação registrado em uma federação que aceita, executa e retorna cargas de trabalho de IA. Um nó pode ser um único servidor, um cluster ou um pool de computação em nuvem.

**Submetente** — Uma entidade (usuário, aplicação ou sistema automatizado) que apresenta cargas de trabalho de IA à federação para execução.

**Zona de Conformidade** — Um contexto de política nomeado que restringe o roteamento de tarefas, o tratamento de dados e a validação de saídas. Zonas definidas: public, enterprise (SOC2), hipaa, sox, fedramp.

**Tarefa** — Uma unidade discreta de carga de trabalho de IA submetida à federação para execução. Uma tarefa carrega um payload, uma declaração de zona de conformidade e uma autorização de faturamento.

**Recibo** — Um registro assinado criptograficamente de uma execução de tarefa concluída, incluindo contagens de tokens, carimbos de tempo, identidade do nó e valores de faturamento.

**Liquidação Econômica** — O processo pelo qual os recibos de faturamento acumulados são convertidos em transferências financeiras entre submetentes, operadores de nós e a plataforma.

**PAK Key (Chave de API da Plataforma)** — Uma credencial bearer emitida pela plataforma que autoriza o acesso aos endpoints da API da federação.

**NeMo Guardrails** — O framework de segurança NVIDIA NeMo utilizado pelos nós MFOP para validação de políticas de segurança de IA e filtragem de saídas.

## 3. Identidade e Registro de Nós

Cada nó em uma federação MFOP é identificado por um identificador de nó estável e globalmente único (node_id). O node_id é um UUID de 128 bits (versão 4) atribuído no momento do registro e persiste entre reinicializações do nó e atualizações de software.

**3.1 Fluxo de Registro**

Um nó inicia o registro enviando um NodeRegistrationRequest ao endpoint de registro da federação (POST /v1/federation/nodes/register). A requisição DEVE incluir:

- node_id: um UUID candidato (a federação PODE substituí-lo)
- operator_id: o UUID da conta do operador que está se registrando
- display_name: um nome legível por humanos para o nó (máximo de 64 caracteres)
- public_key: uma chave pública Ed25519 em codificação base64url, utilizada para assinar recibos
- capability_advertisement: um objeto CapabilityAdvertisement (veja a Seção 4)
- compliance_zones: o conjunto de zonas de conformidade para as quais o nó está certificado
- endpoint_url: a URL HTTPS na qual o nó aceita submissões de tarefas

A federação retorna um NodeRegistrationResponse contendo o node_id atribuído, um registration_token para chamadas autenticadas subsequentes e a configuração de faturamento atual da federação.

**3.2 Re-registro e Rotação de Chaves**

Os nós DEVEM se registrar novamente quando seu par de chaves Ed25519 for rotacionado. Durante a rotação de chaves, o nó submete uma requisição de re-registro com as chaves pública antiga e nova, assinada com a chave privada antiga. A federação verifica a assinatura com a chave antiga antes de aceitar a nova chave. Há uma janela de sobreposição de 24 horas durante a qual recibos assinados com qualquer uma das chaves são aceitos.

**3.3 Saúde do Nó e Cancelamento de Registro**

Os nós DEVEM enviar um heartbeat para POST /v1/federation/nodes/{id}/heartbeat pelo menos uma vez a cada 60 segundos. Um nó que perde três janelas consecutivas de heartbeat é marcado como INATIVO e excluído do roteamento. Os nós podem cancelar o registro voluntariamente via DELETE /v1/federation/nodes/{id}.

## 4. Anúncio de Capacidades

O anúncio de capacidades de um nó declara os modelos de IA disponíveis no nó, as características de hardware relevantes para o roteamento de tarefas e as certificações de conformidade detidas pelo operador do nó.

**4.1 Objeto CapabilityAdvertisement**

O objeto CapabilityAdvertisement inclui os seguintes campos:

- models: um array de objetos ModelDescriptor (veja 4.2)
- hardware_class: um de { cpu, gpu_consumer, gpu_datacenter, tpu }
- vram_gb: total de VRAM de GPU disponível para inferência, em gigabytes (0 para nós CPU)
- max_context_tokens: a janela de contexto máxima que o nó pode atender
- max_concurrent_jobs: o número máximo de tarefas que o nó executará simultaneamente
- compliance_certifications: um array de identificadores de certificação (p. ex., "soc2-type2", "hipaa-baa", "fedramp-moderate")
- nemo_rails_version: a versão do runtime NeMo Guardrails instalada no nó

**4.2 ModelDescriptor**

Cada modelo disponível em um nó é descrito por um ModelDescriptor:

- model_id: uma string de identificador canônico do modelo (p. ex., "meta-llama/Meta-Llama-3-70B-Instruct")
- model_family: um de { llama, mistral, gemma, falcon, phi, custom }
- parameter_count_b: contagem aproximada de parâmetros em bilhões
- quantization: um de { fp16, bf16, int8, int4, none }
- context_window_tokens: a janela de contexto máxima para este modelo
- supports_tool_use: booleano
- supports_vision: booleano

**4.3 Atualização de Capacidades**

Os nós DEVEM atualizar seu anúncio de capacidades via PUT /v1/federation/nodes/{id}/capabilities sempre que seus modelos disponíveis ou configuração de hardware mudar. A federação propaga os anúncios de capacidades atualizados para a camada de roteamento em até 30 segundos.

## 5. Roteamento de Tarefas Ciente de Zonas de Conformidade

O MFOP roteia cada tarefa para um nó que satisfaz os requisitos de zona de conformidade da tarefa. A satisfação da zona de conformidade é uma restrição rígida: uma tarefa NÃO DEVE ser roteada para um nó que não esteja certificado para a zona de conformidade da tarefa.

**5.1 Zonas de Conformidade**

O MFOP define cinco zonas de conformidade, ordenadas da menos restritiva para a mais restritiva:

- public: Nenhum requisito de conformidade além dos rails de segurança NeMo de linha de base. Adequado para cargas de trabalho de IA de uso geral.
- enterprise (SOC2): Requer certificação SOC 2 Tipo II. Adiciona detecção de residência de dados, detecção de exfiltração de credenciais de API e aplicação de registro de acesso.
- hipaa: Requer BAA HIPAA. Adiciona detecção de padrões PHI, desidentificação de PHI e verificações de saída de mínimo necessário.
- sox: Requer controles de conformidade SOX. Adiciona isolamento de PII financeiro, bloqueio de previsão de preços e detecção de MNPI.
- fedramp: Requer autorização FedRAMP. Adiciona tratamento de CUI, detecção de controle de exportação e aplicação de marcação de classificação.

**5.2 Algoritmo de Roteamento**

Quando uma tarefa é recebida, a camada de roteamento executa o seguinte algoritmo:

1. Filtrar: Identificar todos os nós com status ATIVO que estão certificados para a zona de conformidade da tarefa.
2. Filtrar: Remover nós cujo max_context_tokens seja inferior à contagem de tokens estimada da tarefa.
3. Filtrar: Remover nós cujo max_concurrent_jobs esteja esgotado no momento.
4. Pontuar: Para cada nó restante, calcular uma pontuação de roteamento: pontuação = w_latency × pontuação_latência + w_cost × pontuação_custo + w_affinity × pontuação_afinidade. Pesos padrão: w_latency = 0,4, w_cost = 0,4, w_affinity = 0,2.
5. Selecionar: Rotear para o nó de maior pontuação. Em caso de empate, selecionar aleatoriamente de forma uniforme.

Se nenhum nó satisfizer todos os filtros, a tarefa é enfileirada com um tempo limite configurável (padrão: 120 segundos). Se nenhum nó ficar disponível dentro do tempo limite, a federação retorna HTTP 503 com um cabeçalho Retry-After.

**5.3 Regras de Afinidade**

Os submetentes PODEM especificar regras de afinidade em sua submissão de tarefa:

- node_affinity: uma lista de node_ids preferenciais (preferência suave)
- anti_affinity: uma lista de node_ids a excluir (restrição rígida)
- geography: uma região geográfica preferida (código de país ISO 3166-1 alfa-2)

As regras de afinidade afetam apenas o componente pontuação_afinidade; a certificação de zona de conformidade e a capacidade permanecem como restrições rígidas.

## 6. Particionamento Semântico de Entradas

Para tarefas cuja entrada excede o max_context_tokens de um único nó, o MFOP fornece um mecanismo de particionamento semântico que divide a entrada em sub-tarefas coerentes, roteia cada sub-tarefa de forma independente e agrega os resultados.

**6.1 Estratégias de Particionamento**

O MFOP define três estratégias de particionamento:

- sliding_window: Divide a entrada em janelas sobrepostas de tamanho e sobreposição configuráveis. Adequado para tarefas em que a continuidade de contexto nas fronteiras é importante (p. ex., sumarização de documentos longos).
- semantic_boundary: Divide nas fronteiras semânticas detectadas (quebras de parágrafo, cabeçalhos de seção, transições de tópico). Produz sub-tarefas mais coerentes ao custo de tamanhos de sub-tarefa variáveis.
- task_decomposition: Interpreta a entrada como uma lista de tarefas estruturadas e roteia cada tarefa como uma sub-tarefa independente. Requer que a entrada esteja em conformidade com o esquema TaskList do MFOP.

**6.2 Requisição de Particionamento**

Um submetente solicita execução particionada definindo partition_strategy na submissão da tarefa. O motor de particionamento da federação divide a entrada, atribui IDs de sub-tarefa (parent_job_id + número de sequência) e roteia cada sub-tarefa de forma independente. As sub-tarefas herdam a zona de conformidade e a autorização de faturamento da tarefa pai.

**6.3 Agregação**

Assim que todas as sub-tarefas forem concluídas, a camada de agregação da federação monta os resultados na ordem do número de sequência. Para partições sliding_window, o agregador remove duplicatas de conteúdo nas regiões de sobreposição usando uma mesclagem de subsequência comum mais longa. O resultado montado é retornado ao submetente como um único JobResult com um array de sub_job_receipts.

## 7. Recibos de Faturamento Assinados Criptograficamente

Cada execução de tarefa concluída produz um BillingReceipt assinado pelo nó executor. Os recibos assinados são o registro autoritativo para liquidação econômica e resolução de disputas.

**7.1 Estrutura do Recibo**

Um BillingReceipt contém:

- receipt_id: um UUID (versão 4) único para este recibo
- job_id: o UUID da tarefa concluída
- node_id: o UUID do nó executor
- submitter_id: o UUID do submetente
- model_id: o modelo usado para execução
- compliance_zone: a zona de conformidade sob a qual a tarefa foi executada
- input_tokens: o número de tokens de entrada processados
- output_tokens: o número de tokens de saída gerados
- wall_time_ms: tempo total de execução em milissegundos
- completed_at: carimbo de tempo RFC 3339 de conclusão da tarefa
- fee_schedule_id: o UUID do BillingFeeConfig vigente no momento da execução
- input_token_cost_usd: custo calculado de tokens de entrada em USD (6 casas decimais)
- output_token_cost_usd: custo calculado de tokens de saída em USD (6 casas decimais)
- platform_fee_usd: a taxa da plataforma para esta tarefa
- node_earnings_usd: os ganhos do operador do nó para esta tarefa
- total_cost_usd: custo total para o submetente

**7.2 Esquema de Assinatura**

Os recibos são assinados usando Ed25519. O nó assina a serialização JSON canônica do recibo (chaves ordenadas, sem espaços em branco) com sua chave privada registrada. A assinatura é codificada em base64url e incluída no recibo como o campo signature.

A federação verifica a assinatura do recibo ao recebê-lo, usando a chave pública registrada do nó. Recibos com assinaturas inválidas são rejeitados e disparam um alerta de integridade do nó.

**7.3 Armazenamento e Recuperação de Recibos**

A federação armazena todos os recibos por um mínimo de 7 anos para atender aos requisitos de auditoria de conformidade. Os submetentes podem recuperar seus recibos via GET /v1/federation/receipts. Os operadores de nós podem recuperar os recibos das tarefas que executaram via GET /v1/federation/nodes/{id}/receipts.

## 8. Liquidação Econômica Configurável

O MFOP separa o faturamento (o acúmulo de recibos assinados) da liquidação (a transferência financeira de fundos). A liquidação é configurável e pode ocorrer em diferentes cronogramas para diferentes tipos de participantes.

**8.1 BillingFeeConfig**

O administrador da plataforma configura as taxas via um objeto BillingFeeConfig. Cada BillingFeeConfig possui um identificador de versão e uma data de vigência; a federação aplica a configuração vigente no momento da execução da tarefa. Uma nova configuração pode ser criada a qualquer momento; ela entra em vigor no início do próximo período de faturamento.

Campos do BillingFeeConfig:

- input_token_rate_usd_per_1k: USD cobrado por 1.000 tokens de entrada
- output_token_rate_usd_per_1k: USD cobrado por 1.000 tokens de saída
- platform_fee_pct: percentual da plataforma sobre o custo total de tokens (0–100)
- node_revenue_share_pct: percentual do operador do nó sobre o custo total de tokens (0–100, deve somar ≤ 100 com platform_fee_pct)
- settlement_period_days: frequência com que a liquidação é executada (p. ex., 30)
- minimum_payout_usd: ganhos mínimos acumulados antes de o operador do nó receber um pagamento

**8.2 Faturamento do Submetente**

Os submetentes são faturados em regime pós-pago. Ao final de cada período de liquidação, a federação agrega todos os recibos do submetente e cobra o método de pagamento cadastrado. A fatura inclui uma lista detalhada de recibos de tarefas, agrupados por zona de conformidade e modelo.

**8.3 Liquidação para Operadores de Nós**

Os operadores de nós recebem pagamentos via Stripe Connect ao final de cada período de liquidação, desde que seus ganhos acumulados excedam o limite minimum_payout_usd. Operadores que não atingem o limite acumulam seus ganhos para o próximo período.

## 9. Modelo de Segurança

O MFOP implementa um modelo de segurança em três camadas: segurança de transporte, validação de políticas de segurança de IA e isolamento de sandbox de execução.

**9.1 Segurança de Transporte**

Todos os endpoints da API MFOP DEVEM ser servidos via HTTPS usando TLS 1.3 ou superior. TLS mútuo (mTLS) é RECOMENDADO para comunicação nó-para-federação em implantações de malha empresarial privada. A autenticação da API usa PAK Keys transmitidas como o cabeçalho HTTP X-Channel-API-Key. As PAK Keys são valores aleatórios de 256 bits codificados em base64url.

**9.2 Validação de Políticas de Segurança de IA**

Todas as entradas e saídas de tarefas são validadas contra as políticas do NeMo Guardrails antes da execução e antes da entrega ao submetente. O conjunto de políticas de linha de base (obrigatório para todas as zonas de conformidade) inclui:

- Detecção e bloqueio de jailbreak
- Detecção de conteúdo prejudicial (violência, CSAM, facilitação de automutilação)
- Detecção de vazamento de PII nas saídas
- Detecção de injeção de prompt

Políticas adicionais são obrigatórias para zonas de conformidade específicas (veja o Apêndice B).

Os nós DEVEM executar a versão do runtime NeMo Guardrails especificada em seu anúncio de capacidades. Nós executando versões desatualizadas do Guardrails são sinalizados como DEGRADADOS e excluídos do roteamento para zonas de conformidade que exigem funcionalidades do guardrails não presentes na versão instalada.

**9.3 Isolamento de Sandbox de Execução**

Cada tarefa é executada em um sandbox isolado. Os nós DEVEM implementar isolamento de sandbox usando um dos seguintes mecanismos:

- gVisor (runsc) — RECOMENDADO para implantações em nuvem
- Firecracker microVMs — RECOMENDADO para implantações em bare-metal
- WASM (Wasmtime) — Permitido para cargas de trabalho de inferência somente em CPU

Os sandboxes DEVEM ser destruídos e recriados entre as tarefas. O estado persistente do sandbox (p. ex., pesos do modelo) pode ser compartilhado entre tarefas via uma montagem somente leitura, mas o estado específico da tarefa (contexto, arquivos temporários) NÃO DEVE persistir entre tarefas.

**9.4 Registro de Auditoria**

Todas as decisões de roteamento de tarefas, assinaturas de recibos e eventos de liquidação são gravados em um log de auditoria somente para acréscimo. O log de auditoria é encadeado criptograficamente usando hashes SHA-256 (cada entrada inclui o hash da entrada anterior). O log de auditoria não pode ser modificado; apenas operações de acréscimo são permitidas.

## 10. Protocolo de Comunicação

O MFOP usa JSON sobre HTTPS para toda comunicação via API. Conexões WebSocket são suportadas para transmissão em streaming da saída de tarefas (veja a Seção 10.2).

**10.1 Formato de Requisição e Resposta**

Todos os corpos de requisição e resposta são JSON codificado em UTF-8. As requisições DEVEM incluir Content-Type: application/json. Respostas bem-sucedidas usam HTTP 200 ou 201. Respostas de erro usam o envelope de erro padrão:

{ "error": { "code": "<código-legível-por-máquina>", "message": "<mensagem-legível-por-humanos>", "details": { ... } } }

Códigos de erro padrão: UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, QUOTA_EXCEEDED, NO_ELIGIBLE_NODE, COMPLIANCE_VIOLATION, INTERNAL_ERROR.

**10.2 Saída em Streaming**

Nós que suportam saída em streaming expõem um endpoint WebSocket em wss://{node_endpoint}/v1/jobs/{id}/stream. O cliente se conecta após a submissão da tarefa. O nó transmite a saída de tokens como mensagens delta em formato JSON:

{ "type": "delta", "text": "...", "token_count": N }

O stream é encerrado com uma mensagem de conclusão:

{ "type": "done", "receipt": { ... } }

O recibo na mensagem de conclusão é o BillingReceipt assinado para a tarefa.

**10.3 Idempotência**

As requisições de submissão de tarefas DEVERIAM incluir um cabeçalho Idempotency-Key (UUID). Se uma requisição com o mesmo Idempotency-Key for recebida dentro da janela de idempotência (24 horas), a federação retorna a resposta original sem re-executar a tarefa. Isso protege contra submissões duplicadas causadas por novas tentativas de rede.

## Apêndice A. Referência da API REST

Este apêndice lista os endpoints da API REST do MFOP. Todos os endpoints requerem um cabeçalho X-Channel-API-Key, salvo indicação em contrário. Caminho base: /v1/federation

| Método + Caminho | Nome | Descrição |
| --- | --- | --- |
| POST /v1/federation/nodes/register | Registro de nó | Registrar um novo nó na federação. |
| PUT /v1/federation/nodes/{id}/capabilities | Atualização de capacidades | Atualizar o anúncio de capacidades de um nó. |
| POST /v1/federation/nodes/{id}/heartbeat | Heartbeat do nó | Sinalizar que o nó está ativo e aceitando tarefas. |
| DELETE /v1/federation/nodes/{id} | Cancelamento de registro do nó | Cancelar voluntariamente o registro de um nó. |
| POST /v1/federation/jobs | Submissão de tarefa | Submeter uma tarefa à federação para execução. |
| GET /v1/federation/jobs/{id} | Status da tarefa | Recuperar o status atual e o resultado de uma tarefa. |
| GET /v1/federation/jobs/{id}/receipt | Recibo da tarefa | Recuperar o recibo de faturamento assinado de uma tarefa concluída. |
| GET /v1/federation/receipts | Recibos do submetente | Listar todos os recibos do submetente autenticado. |
| GET /v1/federation/nodes/{id}/receipts | Recibos do nó | Listar todos os recibos de tarefas executadas pelo nó. |
| POST /v1/federation/nodes/{id}/stripe/onboard | Integração Stripe Connect | Retorna a URL de integração hospedada pelo Stripe para configuração de conta bancária. |
| GET /v1/federation/nodes/{id}/earnings | Ganhos do provedor | Tokens do período atual, ganhos estimados e último pagamento. |
| GET /v1/federation/submitters/billing | Resumo de faturamento do submetente | Custo do período atual e próxima data de faturamento. |
| PATCH /v1/admin/federation/billing-config | Atualizar modelo de tarifas | Somente administrador. Cria nova linha BillingFeeConfig. Vigente no próximo período. |

## Apêndice B. Requisitos de Políticas por Zona de Conformidade

Cada zona de conformidade requer capacidades específicas de política NeMo Guardrails além da linha de base. A tabela a seguir resume os rails mínimos obrigatórios por zona.

| Zona | Rails Obrigatórios Além da Linha de Base |
| --- | --- |
| public | Somente linha de base. Nenhum rail adicional obrigatório. |
| enterprise (SOC2) | Detecção de marcador de residência de dados. Detecção de exfiltração de credenciais de API. Aplicação de registro de acesso. |
| hipaa | Detecção de padrões PHI: nomes de pacientes, datas de nascimento, MRN, códigos ICD-10, descrições de diagnósticos, IDs de planos de saúde. Rail de desidentificação de PHI: remover ou fazer hash de PHI antes da invocação do modelo de IA. Verificação de mínimo necessário nas saídas. |
| sox | Isolamento de PII financeiro: números de conta, números de roteamento, CPF/CNPJ e identificadores fiscais. Bloqueio de previsão de preços: declarações prospectivas de retorno ou preço. Detecção de MNPI: correspondência de padrões de informações materiais não públicas. |
| fedramp | Tratamento de CUI: detecção de marcadores de Informações Não Classificadas Controladas e regras de tratamento. Controle de exportação: detecção de assuntos sujeitos a EAR/ITAR. Aplicação de marcação de classificação: bloquear saídas contendo marcações de classificação. |

## Agradecimentos

A autora deseja reconhecer a equipe NVIDIA NeMo pelas plataformas NeMo Guardrails e NemoClaw OpenShell, que fornecem a infraestrutura de segurança fundamental referenciada nesta especificação. O modelo de segurança do MFOP é projetado para evoluir junto com essas plataformas à medida que amadurecem.

O modelo de segurança em três camadas, a taxonomia de zonas de conformidade, o esquema de assinatura de recibos Ed25519 e a arquitetura de faturamento configurável descritos nesta especificação foram desenvolvidos e refinados por meio de um extenso processo de design e revisão conduzido na Thrive Tech Services LLC no início de 2026.

Esta especificação é dedicada à comunidade global de trabalhadores do conhecimento — nas áreas jurídica, de saúde, pesquisa, financeira e técnica — cujo trabalho é a razão pela qual a orquestração federada de IA é importante.

Fim da Especificação MFOP Versão 1.0 — Rascunho para Revisão por Pares
Thrive Tech Services LLC · Ami Hoepner Nuñez · Março de 2026

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · Março de 2026*
