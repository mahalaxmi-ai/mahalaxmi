# Protocolo de Federación y Orquestación Mahalaxmi

**MFOP v1.0** · Borrador para Revisión por Pares

| | |
|---|---|
| Date | Marzo 2026 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Florida, EE. UU. |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## Estado de Este Memorando

Este documento es un borrador previo a la publicación de la especificación del Protocolo de Federación y Orquestación Mahalaxmi (MFOP), versión 1.0. Se distribuye para revisión por pares y para solicitar comentarios. Este documento describe un protocolo para la orquestación de IA distribuida y federada a través de nodos de cómputo heterogéneos, con enrutamiento consciente de zonas de cumplimiento, recibos de facturación firmados criptográficamente y liquidación económica configurable.

Los comentarios y preguntas deben dirigirse a la autora a Ami.nunez@mahalaxmi.ai. El borrador actual y los hilos de discusión se mantienen en https://mahalaxmi.ai/mfop/draft. Los hilos de discusión se encuentran en https://mahalaxmi.ai/mfop/discuss.

## Aviso de Derechos de Autor

Copyright © 2026 ThriveTech Services LLC. Todos los derechos reservados. Se otorga permiso para copiar, distribuir y utilizar este documento en cualquier medio sin cargo, siempre que la atribución a la autora, el título del documento y este aviso de derechos de autor se conserven en todas las copias y trabajos derivados.

## Resumen

Este documento define el Protocolo de Federación y Orquestación Mahalaxmi (MFOP), un protocolo para coordinar la ejecución paralela de agentes de IA a través de una red distribuida de nodos de cómputo heterogéneos. MFOP especifica la identidad y el registro de nodos, la publicación de capacidades, el enrutamiento de trabajos consciente de zonas de cumplimiento, la partición semántica de entradas, los recibos de facturación firmados criptográficamente, la liquidación económica configurable y un modelo de seguridad por capas que utiliza validación de políticas de seguridad de IA y aislamiento en entornos de ejecución controlados.

MFOP está diseñado para operar en tres configuraciones de despliegue simultáneas: mallas empresariales privadas donde los nodos son propiedad y están operados por una sola organización, grupos de nube administrados por el proveedor de la plataforma, y mercados comunitarios abiertos donde cualquier operador de nodos puede contribuir cómputo a cambio de liquidación económica. El protocolo es agnóstico al proveedor de modelos de IA subyacente y está diseñado para evolucionar con el panorama de seguridad y cumplimiento de la IA.

## 1. Introducción

El crecimiento de los despliegues de modelos de lenguaje de gran escala (LLM) en entornos empresariales ha creado la necesidad de una capa de coordinación capaz de abarcar infraestructura de cómputo heterogénea mientras satisface requisitos de cumplimiento, facturación y seguridad que varían según la jurisdicción y el sector.

MFOP responde a esta necesidad definiendo un protocolo para la orquestación federada de IA. Una federación consiste en uno o más nodos de cómputo, cada uno de los cuales puede ser operado por diferentes entidades bajo diferentes regímenes de cumplimiento. Un emisor — un usuario, una aplicación o un sistema automatizado — presenta un trabajo a la federación. La federación enruta el trabajo a un nodo apropiado según los requisitos de zona de cumplimiento del trabajo, la publicación de capacidades del nodo y los términos económicos vigentes.

Esta especificación define el protocolo de comunicación, los formatos de datos, los mecanismos criptográficos y los requisitos de comportamiento para todos los componentes de una federación MFOP conforme.

## 2. Terminología

Las palabras clave "DEBE", "NO DEBE", "REQUERIDO", "DEBERÁ", "NO DEBERÁ", "DEBERÍA", "NO DEBERÍA", "RECOMENDADO", "NO RECOMENDADO", "PUEDE" y "OPCIONAL" en este documento DEBEN interpretarse según lo descrito en BCP 14 [RFC2119] [RFC8174].

**Federación** — Agrupación lógica de uno o más nodos de cómputo conformes con MFOP que operan bajo una configuración de gobernanza compartida.

**Nodo** — Recurso de cómputo registrado en una federación que acepta, ejecuta y devuelve cargas de trabajo de IA. Un nodo puede ser un único servidor, un clúster o un grupo de cómputo en la nube.

**Emisor** — Entidad (usuario, aplicación o sistema automatizado) que presenta cargas de trabajo de IA a la federación para su ejecución.

**Zona de Cumplimiento** — Contexto de política con nombre que restringe el enrutamiento de trabajos, el manejo de datos y la validación de salidas. Zonas definidas: public, enterprise (SOC2), hipaa, sox, fedramp.

**Trabajo** — Unidad discreta de carga de trabajo de IA enviada a la federación para su ejecución. Un trabajo lleva una carga útil, una declaración de zona de cumplimiento y una autorización de facturación.

**Recibo** — Registro firmado criptográficamente de una ejecución de trabajo completada, que incluye recuentos de tokens, marcas de tiempo, identidad del nodo y montos de facturación.

**Liquidación Económica** — Proceso por el cual los recibos de facturación acumulados se convierten en transferencias financieras entre emisores, operadores de nodos y la plataforma.

**PAK Key (Clave API de Plataforma)** — Credencial de portador emitida por la plataforma que autoriza el acceso a los endpoints de la API de la federación.

**NeMo Guardrails** — El framework de seguridad NVIDIA NeMo utilizado por los nodos MFOP para la validación de políticas de seguridad de IA y el filtrado de salidas.

## 3. Identidad y Registro de Nodos

Cada nodo en una federación MFOP se identifica mediante un identificador de nodo estable y globalmente único (node_id). El node_id es un UUID de 128 bits (versión 4) asignado en el momento del registro y persiste a través de reinicios del nodo y actualizaciones de software.

**3.1 Flujo de Registro**

Un nodo inicia el registro enviando una NodeRegistrationRequest al endpoint de registro de la federación (POST /v1/federation/nodes/register). La solicitud DEBE incluir:

- node_id: un UUID candidato (la federación PUEDE reemplazarlo)
- operator_id: el UUID de la cuenta del operador que registra
- display_name: un nombre legible por humanos para el nodo (máx. 64 caracteres)
- public_key: una clave pública Ed25519 en codificación base64url, utilizada para firmar recibos
- capability_advertisement: un objeto CapabilityAdvertisement (ver Sección 4)
- compliance_zones: el conjunto de zonas de cumplimiento para las que el nodo está certificado
- endpoint_url: la URL HTTPS en la que el nodo acepta envíos de trabajos

La federación devuelve una NodeRegistrationResponse que contiene el node_id asignado, un registration_token para llamadas autenticadas posteriores y la configuración de facturación actual de la federación.

**3.2 Re-registro y Rotación de Claves**

Los nodos DEBEN re-registrarse cuando su par de claves Ed25519 es rotado. Durante la rotación de claves, el nodo envía una solicitud de re-registro con las claves pública antigua y nueva, firmada con la clave privada antigua. La federación verifica la firma de la clave antigua antes de aceptar la nueva clave. Existe una ventana de superposición de 24 horas durante la cual se aceptan recibos firmados con cualquiera de las dos claves.

**3.3 Estado de Salud y Cancelación de Registro de Nodos**

Los nodos DEBEN enviar un latido a POST /v1/federation/nodes/{id}/heartbeat al menos una vez cada 60 segundos. Un nodo que omita tres ventanas consecutivas de latido se marca como INACTIVO y se excluye del enrutamiento. Los nodos pueden cancelar su registro voluntariamente mediante DELETE /v1/federation/nodes/{id}.

## 4. Publicación de Capacidades

La publicación de capacidades de un nodo declara los modelos de IA disponibles en el nodo, las características de hardware relevantes para el enrutamiento de trabajos y las certificaciones de cumplimiento que posee el operador del nodo.

**4.1 Objeto CapabilityAdvertisement**

El objeto CapabilityAdvertisement incluye los siguientes campos:

- models: una lista de objetos ModelDescriptor (ver 4.2)
- hardware_class: uno de { cpu, gpu_consumer, gpu_datacenter, tpu }
- vram_gb: total de VRAM de GPU disponible para inferencia, en gigabytes (0 para nodos CPU)
- max_context_tokens: la ventana de contexto máxima que el nodo puede atender
- max_concurrent_jobs: el número máximo de trabajos que el nodo ejecutará simultáneamente
- compliance_certifications: una lista de identificadores de certificación (p. ej., "soc2-type2", "hipaa-baa", "fedramp-moderate")
- nemo_rails_version: la versión del entorno de ejecución de NeMo Guardrails instalado en el nodo

**4.2 ModelDescriptor**

Cada modelo disponible en un nodo se describe mediante un ModelDescriptor:

- model_id: una cadena de identificador de modelo canónico (p. ej., "meta-llama/Meta-Llama-3-70B-Instruct")
- model_family: uno de { llama, mistral, gemma, falcon, phi, custom }
- parameter_count_b: recuento aproximado de parámetros en miles de millones
- quantization: uno de { fp16, bf16, int8, int4, none }
- context_window_tokens: la ventana de contexto máxima para este modelo
- supports_tool_use: booleano
- supports_vision: booleano

**4.3 Actualización de Capacidades**

Los nodos DEBEN actualizar su publicación de capacidades mediante PUT /v1/federation/nodes/{id}/capabilities siempre que cambien sus modelos disponibles o la configuración de hardware. La federación propaga las publicaciones de capacidades actualizadas a la capa de enrutamiento en un plazo de 30 segundos.

## 5. Enrutamiento de Trabajos Consciente de Zonas de Cumplimiento

MFOP enruta cada trabajo a un nodo que satisface los requisitos de zona de cumplimiento del trabajo. La satisfacción de la zona de cumplimiento es una restricción estricta: un trabajo NO DEBE enrutarse a un nodo que no esté certificado para la zona de cumplimiento del trabajo.

**5.1 Zonas de Cumplimiento**

MFOP define cinco zonas de cumplimiento, ordenadas de menor a mayor restricción:

- public: Sin requisitos de cumplimiento más allá de los rieles de seguridad NeMo de referencia. Adecuado para cargas de trabajo de IA de propósito general.
- enterprise (SOC2): Requiere certificación SOC 2 Tipo II. Agrega detección de residencia de datos, detección de exfiltración de credenciales de API y aplicación de registro de accesos.
- hipaa: Requiere HIPAA BAA. Agrega detección de patrones PHI, des-identificación de PHI y verificaciones de salida de mínimo necesario.
- sox: Requiere controles de cumplimiento SOX. Agrega aislamiento de PII financiera, bloqueo de predicción de precios y detección de MNPI.
- fedramp: Requiere autorización FedRAMP. Agrega manejo de CUI, detección de control de exportaciones y aplicación del marcado de clasificación.

**5.2 Algoritmo de Enrutamiento**

Cuando se recibe un trabajo, la capa de enrutamiento ejecuta el siguiente algoritmo:

1. Filtrar: Identificar todos los nodos con estado ACTIVO que estén certificados para la zona de cumplimiento del trabajo.
2. Filtrar: Eliminar los nodos cuyo max_context_tokens sea inferior al recuento de tokens estimado del trabajo.
3. Filtrar: Eliminar los nodos cuyo max_concurrent_jobs esté actualmente agotado.
4. Puntuar: Para cada nodo restante, calcular una puntuación de enrutamiento: puntuación = w_latencia × puntuación_latencia + w_costo × puntuación_costo + w_afinidad × puntuación_afinidad. Pesos predeterminados: w_latencia = 0.4, w_costo = 0.4, w_afinidad = 0.2.
5. Seleccionar: Enrutar al nodo con mayor puntuación. En caso de empate, seleccionar de forma aleatoria uniforme.

Si ningún nodo satisface todos los filtros, el trabajo se pone en cola con un tiempo de espera configurable (predeterminado: 120 segundos). Si ningún nodo queda disponible dentro del tiempo de espera, la federación devuelve HTTP 503 con un encabezado Retry-After.

**5.3 Reglas de Afinidad**

Los emisores PUEDEN especificar reglas de afinidad en su envío de trabajos:

- node_affinity: una lista de node_ids preferidos (preferencia suave)
- anti_affinity: una lista de node_ids a excluir (restricción estricta)
- geography: una región geográfica preferida (código de país ISO 3166-1 alfa-2)

Las reglas de afinidad afectan únicamente al componente puntuación_afinidad; la certificación de zona de cumplimiento y la capacidad permanecen como restricciones estrictas.

## 6. Partición Semántica de Entradas

Para trabajos cuya entrada supera el max_context_tokens de un único nodo, MFOP proporciona un mecanismo de partición semántica que divide la entrada en sub-trabajos coherentes, enruta cada sub-trabajo de forma independiente y agrega los resultados.

**6.1 Estrategias de Partición**

MFOP define tres estrategias de partición:

- sliding_window: Divide la entrada en ventanas superpuestas de tamaño y superposición configurables. Adecuado para tareas donde la continuidad del contexto en los límites es importante (p. ej., resumen de documentos largos).
- semantic_boundary: Divide en límites semánticos detectados (saltos de párrafo, encabezados de sección, transiciones de tema). Produce sub-trabajos más coherentes a costa de tamaños de sub-trabajo variables.
- task_decomposition: Interpreta la entrada como una lista de tareas estructurada y enruta cada tarea como un sub-trabajo independiente. Requiere que la entrada se ajuste al esquema TaskList de MFOP.

**6.2 Solicitud de Partición**

Un emisor solicita la ejecución particionada configurando partition_strategy en el envío del trabajo. El motor de partición de la federación divide la entrada, asigna IDs de sub-trabajo (parent_job_id + número de secuencia) y enruta cada sub-trabajo de forma independiente. Los sub-trabajos heredan la zona de cumplimiento y la autorización de facturación del trabajo padre.

**6.3 Agregación**

Una vez que todos los sub-trabajos se completan, la capa de agregación de la federación ensambla los resultados en orden de número de secuencia. Para las particiones sliding_window, el agregador elimina duplicados del contenido en las regiones de superposición utilizando una fusión de subsecuencia común más larga. El resultado ensamblado se devuelve al emisor como un único JobResult con una lista de sub_job_receipts.

## 7. Recibos de Facturación Firmados Criptográficamente

Cada ejecución de trabajo completada produce un BillingReceipt firmado por el nodo ejecutor. Los recibos firmados son el registro autoritativo para la liquidación económica y la resolución de disputas.

**7.1 Estructura del Recibo**

Un BillingReceipt contiene:

- receipt_id: un UUID (versión 4) único para este recibo
- job_id: el UUID del trabajo completado
- node_id: el UUID del nodo ejecutor
- submitter_id: el UUID del emisor
- model_id: el modelo utilizado para la ejecución
- compliance_zone: la zona de cumplimiento bajo la que se ejecutó el trabajo
- input_tokens: el número de tokens de entrada procesados
- output_tokens: el número de tokens de salida generados
- wall_time_ms: tiempo total de ejecución en milisegundos
- completed_at: marca de tiempo RFC 3339 de finalización del trabajo
- fee_schedule_id: el UUID del BillingFeeConfig vigente en el momento de la ejecución
- input_token_cost_usd: costo calculado de tokens de entrada en USD (6 decimales)
- output_token_cost_usd: costo calculado de tokens de salida en USD (6 decimales)
- platform_fee_usd: la tarifa de la plataforma por este trabajo
- node_earnings_usd: las ganancias del operador del nodo por este trabajo
- total_cost_usd: costo total para el emisor

**7.2 Esquema de Firma**

Los recibos se firman usando Ed25519. El nodo firma la serialización JSON canónica del recibo (claves ordenadas, sin espacios en blanco) con su clave privada registrada. La firma está codificada en base64url e incluida en el recibo como el campo signature.

La federación verifica la firma del recibo al recibirlo usando la clave pública registrada del nodo. Los recibos con firmas inválidas son rechazados y activan una alerta de integridad del nodo.

**7.3 Almacenamiento y Recuperación de Recibos**

La federación almacena todos los recibos por un mínimo de 7 años para satisfacer los requisitos de auditoría de cumplimiento. Los emisores pueden recuperar sus recibos mediante GET /v1/federation/receipts. Los operadores de nodos pueden recuperar los recibos de los trabajos que ejecutaron mediante GET /v1/federation/nodes/{id}/receipts.

## 8. Liquidación Económica Configurable

MFOP separa la facturación (la acumulación de recibos firmados) de la liquidación (la transferencia financiera de fondos). La liquidación es configurable y puede ocurrir en distintos calendarios para diferentes tipos de participantes.

**8.1 BillingFeeConfig**

El administrador de la plataforma configura las tarifas mediante un objeto BillingFeeConfig. Cada BillingFeeConfig tiene un identificador de versión y una fecha de entrada en vigor; la federación aplica la configuración vigente en el momento de la ejecución del trabajo. Se puede crear una nueva configuración en cualquier momento; entra en vigor al inicio del siguiente período de facturación.

Campos de BillingFeeConfig:

- input_token_rate_usd_per_1k: USD cobrados por cada 1,000 tokens de entrada
- output_token_rate_usd_per_1k: USD cobrados por cada 1,000 tokens de salida
- platform_fee_pct: el porcentaje de la plataforma sobre el costo total de tokens (0–100)
- node_revenue_share_pct: el porcentaje del operador del nodo sobre el costo total de tokens (0–100, debe sumar ≤ 100 junto con platform_fee_pct)
- settlement_period_days: frecuencia con la que se ejecuta la liquidación (p. ej., 30)
- minimum_payout_usd: ganancias acumuladas mínimas antes de que un operador de nodo reciba un pago

**8.2 Facturación de Emisores**

Los emisores son facturados en modalidad pospago. Al final de cada período de liquidación, la federación agrega todos los recibos del emisor y cobra al método de pago registrado. La factura incluye una lista detallada de recibos de trabajos, agrupados por zona de cumplimiento y modelo.

**8.3 Liquidación de Operadores de Nodos**

Los operadores de nodos reciben sus pagos mediante Stripe Connect al final de cada período de liquidación, siempre que sus ganancias acumuladas superen el umbral minimum_payout_usd. Los operadores que no alcanzan el umbral trasladan sus ganancias al período siguiente.

## 9. Modelo de Seguridad

MFOP implementa un modelo de seguridad de tres capas: seguridad de transporte, validación de políticas de seguridad de IA y aislamiento en entornos de ejecución controlados.

**9.1 Seguridad de Transporte**

Todos los endpoints de la API de MFOP DEBEN servirse sobre HTTPS usando TLS 1.3 o superior. TLS mutuo (mTLS) es RECOMENDADO para la comunicación nodo-federación en despliegues de malla empresarial privada. La autenticación de API utiliza PAK Keys transmitidas como encabezado HTTP X-Channel-API-Key. Las PAK Keys son valores aleatorios de 256 bits codificados en base64url.

**9.2 Validación de Políticas de Seguridad de IA**

Todas las entradas y salidas de trabajos se validan contra las políticas de NeMo Guardrails antes de la ejecución y antes de la entrega al emisor. El conjunto de políticas de referencia (requerido para todas las zonas de cumplimiento) incluye:

- Detección y bloqueo de intentos de evasión (jailbreak)
- Detección de contenido dañino (violencia, CSAM, facilitación de autolesiones)
- Detección de filtración de PII en salidas
- Detección de inyección de instrucciones (prompt injection)

Se requieren políticas adicionales para zonas de cumplimiento específicas (ver Apéndice B).

Los nodos DEBEN ejecutar la versión del entorno de ejecución de NeMo Guardrails especificada en su publicación de capacidades. Los nodos que ejecutan versiones desactualizadas de Guardrails se marcan como DEGRADADOS y se excluyen del enrutamiento para las zonas de cumplimiento que requieren funciones de guardrails no presentes en la versión instalada.

**9.3 Aislamiento en Entornos de Ejecución Controlados**

Cada trabajo se ejecuta en un entorno controlado aislado. Los nodos DEBEN implementar aislamiento de entorno de ejecución usando uno de los siguientes mecanismos:

- gVisor (runsc) — RECOMENDADO para despliegues en la nube
- Firecracker microVMs — RECOMENDADO para despliegues en hardware dedicado
- WASM (Wasmtime) — Permitido para cargas de trabajo de inferencia únicamente en CPU

Los entornos de ejecución DEBEN destruirse y recrearse entre trabajos. El estado persistente del entorno de ejecución (p. ej., pesos del modelo) PUEDE compartirse entre trabajos mediante un montaje de solo lectura, pero el estado específico del trabajo (contexto, archivos temporales) NO DEBE persistir entre trabajos.

**9.4 Registro de Auditoría**

Todas las decisiones de enrutamiento de trabajos, firmas de recibos y eventos de liquidación se escriben en un registro de auditoría de solo anexo. El registro de auditoría está encadenado criptográficamente usando hashes SHA-256 (cada entrada incluye el hash de la entrada anterior). El registro de auditoría no puede modificarse; solo se permiten operaciones de anexado.

## 10. Protocolo de Comunicación

MFOP utiliza JSON sobre HTTPS para toda la comunicación de la API. Las conexiones WebSocket son compatibles para la transmisión en flujo de salida de trabajos (ver Sección 10.2).

**10.1 Formato de Solicitud y Respuesta**

Todos los cuerpos de solicitud y respuesta son JSON codificado en UTF-8. Las solicitudes DEBEN incluir Content-Type: application/json. Las respuestas exitosas usan HTTP 200 o 201. Las respuestas de error usan el sobre de error estándar:

{ "error": { "code": "<código-legible-por-máquina>", "message": "<mensaje-legible-por-humanos>", "details": { ... } } }

Códigos de error estándar: UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, QUOTA_EXCEEDED, NO_ELIGIBLE_NODE, COMPLIANCE_VIOLATION, INTERNAL_ERROR.

**10.2 Salida en Flujo**

Los nodos que admiten salida en flujo exponen un endpoint WebSocket en wss://{node_endpoint}/v1/jobs/{id}/stream. El cliente se conecta tras el envío del trabajo. El nodo transmite la salida de tokens como mensajes delta en formato JSON:

{ "type": "delta", "text": "...", "token_count": N }

El flujo se termina con un mensaje de finalización:

{ "type": "done", "receipt": { ... } }

El recibo en el mensaje de finalización es el BillingReceipt firmado del trabajo.

**10.3 Idempotencia**

Las solicitudes de envío de trabajos DEBERÍAN incluir un encabezado Idempotency-Key (UUID). Si se recibe una solicitud con la misma Idempotency-Key dentro de la ventana de idempotencia (24 horas), la federación devuelve la respuesta original sin volver a ejecutar el trabajo. Esto protege contra envíos duplicados causados por reintentos de red.

## Apéndice A. Referencia de la API REST

Este apéndice lista los endpoints de la API REST de MFOP. Todos los endpoints requieren un encabezado X-Channel-API-Key salvo que se indique lo contrario. Ruta base: /v1/federation

| Método + Ruta | Nombre | Descripción |
| --- | --- | --- |
| POST /v1/federation/nodes/register | Registro de nodo | Registra un nuevo nodo en la federación. |
| PUT /v1/federation/nodes/{id}/capabilities | Actualización de capacidades | Actualiza la publicación de capacidades de un nodo. |
| POST /v1/federation/nodes/{id}/heartbeat | Latido de nodo | Indica que el nodo está activo y aceptando trabajos. |
| DELETE /v1/federation/nodes/{id} | Cancelación de registro de nodo | Cancela el registro de un nodo voluntariamente. |
| POST /v1/federation/jobs | Envío de trabajo | Envía un trabajo a la federación para su ejecución. |
| GET /v1/federation/jobs/{id} | Estado del trabajo | Recupera el estado actual y el resultado de un trabajo. |
| GET /v1/federation/jobs/{id}/receipt | Recibo del trabajo | Recupera el recibo de facturación firmado de un trabajo completado. |
| GET /v1/federation/receipts | Recibos del emisor | Lista todos los recibos del emisor autenticado. |
| GET /v1/federation/nodes/{id}/receipts | Recibos del nodo | Lista todos los recibos de los trabajos ejecutados por el nodo. |
| POST /v1/federation/nodes/{id}/stripe/onboard | Alta en Stripe Connect | Devuelve la URL de alta alojada en Stripe para la configuración de cuenta bancaria. |
| GET /v1/federation/nodes/{id}/earnings | Ganancias del proveedor | Tokens del período actual, ganancias estimadas, último pago. |
| GET /v1/federation/submitters/billing | Resumen de facturación del emisor | Costo del período actual, próxima fecha de facturación. |
| PATCH /v1/admin/federation/billing-config | Actualizar modelo de tarifas | Solo administrador. Crea una nueva fila BillingFeeConfig. Entra en vigor en el próximo período. |

## Apéndice B. Requisitos de Políticas por Zona de Cumplimiento

Cada zona de cumplimiento requiere capacidades específicas de políticas de NeMo Guardrails más allá de la referencia. La siguiente tabla resume los rieles mínimos requeridos por zona.

| Zona | Rieles Requeridos Además de la Referencia |
| --- | --- |
| public | Solo referencia. No se requieren rieles adicionales. |
| enterprise (SOC2) | Detección de marcadores de residencia de datos. Detección de exfiltración de credenciales de API. Aplicación de registro de accesos. |
| hipaa | Detección de patrones PHI: nombres de pacientes, fechas de nacimiento, MRN, códigos ICD-10, descripciones de diagnósticos, identificadores de seguro médico. Riel de des-identificación de PHI: eliminar o aplicar hash a PHI antes de la invocación del modelo de IA. Verificación de mínimo necesario en salidas. |
| sox | Aislamiento de PII financiera: números de cuenta, números de ruta, identificaciones fiscales. Bloqueo de predicción de precios: declaraciones de rendimiento o precio con proyección futura. Detección de MNPI: coincidencia de patrones de información material no pública. |
| fedramp | Manejo de CUI: detección de marcadores de Información No Clasificada Controlada y reglas de manejo. Control de exportaciones: detección de materia sujeta a EAR/ITAR. Aplicación del marcado de clasificación: bloquear salidas que contengan marcados de clasificación. |

## Agradecimientos

La autora desea reconocer al equipo de NVIDIA NeMo por las plataformas NeMo Guardrails y NemoClaw OpenShell, que proporcionan la infraestructura de seguridad fundamental referenciada en esta especificación. El modelo de seguridad de MFOP está diseñado para evolucionar con estas plataformas a medida que maduran.

El modelo de seguridad de tres capas, la taxonomía de zonas de cumplimiento, el esquema de firma de recibos Ed25519 y la arquitectura de facturación configurable descritos en esta especificación fueron desarrollados y refinados a través de un extenso proceso de diseño y revisión llevado a cabo en Thrive Tech Services LLC a principios de 2026.

Esta especificación está dedicada a la comunidad global de trabajadores del conocimiento — en disciplinas legales, de salud, investigación, finanzas y técnicas — cuyo trabajo es la razón por la que la orquestación federada de IA importa.

Fin de la Especificación MFOP Versión 1.0 — Borrador para Revisión por Pares
Thrive Tech Services LLC · Ami Hoepner Nuñez · Marzo 2026

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · Marzo 2026*
