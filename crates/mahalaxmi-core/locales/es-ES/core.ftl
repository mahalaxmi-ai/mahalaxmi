# Errores
error-config-file-not-found = Archivo de configuración no encontrado en { $path }
error-config-parse-failed = Error al analizar la configuración: { $reason }
error-config-validation-failed = La validación de configuración falló: { $details }
error-locale-not-supported = El idioma "{ $locale }" no es compatible
error-log-init-failed = Error al inicializar el registro: { $reason }
error-log-dir-create-failed = Error al crear el directorio de registro en { $path }: { $reason }
error-app-launch-failed = Error al iniciar la aplicación: { $reason }

# Validación
validation-invalid-log-level = Nivel de registro inválido "{ $level }". Niveles válidos: { $valid }
validation-workers-out-of-range = max_concurrent_workers debe estar entre { $min } y { $max }, se obtuvo { $value }
validation-manager-timeout-too-low = manager_timeout_seconds debe ser al menos { $min }, se obtuvo { $value }
validation-worker-timeout-too-low = worker_timeout_seconds debe ser al menos { $min }, se obtuvo { $value }
validation-offline-grace-too-low = offline_grace_days debe ser al menos { $min }, se obtuvo { $value }
validation-invalid-consensus-strategy = Estrategia de consenso inválida "{ $value }". Estrategias válidas: { $valid }
validation-invalid-data-directory = data_directory contiene caracteres inválidos
validation-empty-default-provider = providers.default_provider no debe estar vacío
validation-invalid-theme = Tema inválido "{ $value }". Temas válidos: { $valid }
validation-font-size-out-of-range = terminal_font_size debe estar entre { $min } y { $max }, se obtuvo { $value }
validation-invalid-max-batch-retries = max_batch_retries debe ser >= 1, se obtuvo { $value }
validation-invalid-max-total-batches = max_total_batches debe ser >= 2, se obtuvo { $value }
validation-invalid-stall-detection-threshold = stall_detection_threshold debe ser >= 2, se obtuvo { $value }

# Configuración
config-loaded-successfully = Configuración cargada desde { $path }
config-using-default = No se encontró archivo de configuración, usando valores predeterminados
config-env-override = Valor de configuración reemplazado por la variable de entorno { $var }
config-env-override-invalid = La variable de entorno { $var } tiene un valor inválido "{ $value }", manteniendo el valor configurado
config-generated-successfully = Configuración predeterminada generada en { $path }
config-already-exists = El archivo de configuración ya existe en { $path }

# Registro
logging-initialized = Registro inicializado en nivel { $level }
logging-rust-log-override = Variable de entorno RUST_LOG detectada, anulando el nivel configurado
logging-file-path = Archivo de registro: { $path }
logging-dir-create-failed-fallback = Error al crear el directorio de registro { $path }, utilizando solo registro en consola

# PTY
pty-open-failed = Error al abrir el pseudoterminal: { $reason }
pty-spawn-failed = Error al ejecutar "{ $program }" en PTY: { $reason }
pty-write-failed = Error al escribir en la terminal { $terminal_id }: { $reason }
pty-read-failed = Error al leer de la terminal { $terminal_id }: { $reason }
pty-resize-failed = Error al redimensionar la terminal { $terminal_id } a { $rows }x{ $cols }: { $reason }
pty-wait-failed = Error al verificar el estado del proceso en la terminal { $terminal_id }: { $reason }
pty-kill-failed = Error al finalizar el proceso en la terminal { $terminal_id }: { $reason }

# Aplicación
app-starting = Mahalaxmi v{ $version } iniciando
app-ready = Mahalaxmi está listo
app-shutting-down = Mahalaxmi cerrándose

# Credenciales
credential-anthropic-api-key = Clave API de Anthropic para Claude Code
credential-generic-api-key = Clave API para { $provider }
credential-aws-iam-role = Rol de AWS IAM para { $provider }
credential-oauth-token = Token de OAuth para { $provider }

# Proveedor
error-provider-credentials-missing = Faltan las credenciales de { $provider }: la variable de entorno { $env_var } no está configurada
error-provider-credentials-invalid = Las credenciales de { $provider } no son válidas: { $reason }
error-provider-not-found = Proveedor "{ $provider_id }" no encontrado en el registro
error-provider-no-default = No hay proveedor de IA predeterminado configurado
error-provider-command-build-failed = Error al construir el comando de { $provider }: { $reason }
provider-registered = Proveedor "{ $provider }" registrado con ID "{ $id }"
provider-set-default = Proveedor predeterminado establecido como "{ $provider }"
provider-credentials-valid = Credenciales de { $provider } validadas correctamente
provider-validating = Validando credenciales de { $provider }
provider-list-header = Proveedores de IA registrados

# PTY (extendido)
error-pty-open-failed = Error al abrir PTY: { $reason }
error-pty-spawn-failed = Error al ejecutar "{ $program }" en PTY: { $reason }
error-pty-write-failed = Error al escribir en el terminal { $terminal_id }: { $reason }
error-pty-read-failed = Error al leer del terminal { $terminal_id }: { $reason }
error-pty-resize-failed = Error al redimensionar el terminal { $terminal_id } a { $rows }x{ $cols }: { $reason }
error-pty-kill-failed = Error al terminar el proceso del terminal { $terminal_id }: { $reason }
error-pty-wait-failed = Error al verificar el estado del terminal { $terminal_id }: { $reason }
error-pty-terminal-not-found = Terminal { $terminal_id } no encontrado
error-pty-max-concurrent-reached = Límite máximo de terminales concurrentes ({ $max }) alcanzado
pty-process-spawned = Proceso "{ $program }" iniciado en terminal { $terminal_id }
pty-process-exited = El proceso del terminal { $terminal_id } finalizó con código { $exit_code }
pty-session-closed = Sesión del terminal { $terminal_id } cerrada
pty-resized = Terminal { $terminal_id } redimensionado a { $rows }x{ $cols }
pty-reader-eof = Lector del terminal { $terminal_id } alcanzó el fin del flujo
pty-reader-error = Error del lector del terminal { $terminal_id }: { $reason }

# Errores de orquestación
error-orchestration-invalid-transition = Transición de estado inválida de { $from } a { $to }
error-orchestration-circular-dependency = Dependencia circular detectada: { $cycle }
error-orchestration-worker-not-found = Trabajador { $worker_id } no encontrado en la cola
error-orchestration-max-retries-exceeded = El trabajador { $worker_id } superó el máximo de reintentos ({ $max_retries })
error-orchestration-no-proposals = No se recibieron propuestas de gerentes
error-orchestration-plan-validation-failed = La validación del plan de ejecución falló: { $errors }
error-orchestration-consensus-failed = El motor de consenso falló: { $reason }
error-orchestration-queue-full = La cola de trabajadores está llena (máximo { $max })
error-orchestration-manager-timeout = El gerente { $manager_id } agotó el tiempo después de { $timeout }s
error-orchestration-worker-timeout = El trabajador { $worker_id } agotó el tiempo después de { $timeout }s

# Información de orquestación
orchestration-cycle-started = Ciclo de orquestación { $cycle_id } iniciado
orchestration-state-changed = Estado cambiado: { $from } -> { $to }
orchestration-manager-completed = Gerente { $manager_id } completado con { $task_count } tareas
orchestration-consensus-reached = Consenso alcanzado: { $agreed } acordadas, { $dissenting } disidentes
orchestration-plan-created = Plan de ejecución creado: { $phases } fases, { $workers } trabajadores
orchestration-worker-started = Trabajador { $worker_id } iniciado: { $task }
orchestration-worker-completed = Trabajador { $worker_id } completado en { $duration }ms
orchestration-worker-failed = Trabajador { $worker_id } falló: { $error }
orchestration-cycle-completed = Ciclo completado en { $duration }ms (tasa de éxito: { $success_rate })
orchestration-worker-retrying = Trabajador { $worker_id } reintentando (intento { $attempt }/{ $max })

# Errores de detección
error-detection-rule-compile-failed = Error al compilar patrón de regla de detección: { $reason }
error-detection-no-rules-loaded = No se cargaron reglas de detección
error-detection-invalid-pattern = Patrón de detección inválido "{ $pattern }": { $reason }

# Información de detección
detection-rule-matched = Regla de detección "{ $rule }" coincidió, acción: { $action }
detection-rule-cooldown = Regla de detección "{ $rule }" suprimida por enfriamiento ({ $remaining_ms }ms restantes)
detection-rules-loaded = { $count } reglas de detección cargadas
detection-provider-rules-applied = { $count } reglas aplicadas para proveedor { $provider }
detection-error-pattern-detected = Patrón de error detectado: "{ $pattern }" (visto { $count } veces)
detection-root-cause-hypothesis = Hipótesis de causa raíz: { $category } (confianza: { $confidence })
detection-recurring-error = Error recurrente: "{ $message }" (ocurrió { $count } veces)
detection-action-executed = Acción { $action } ejecutada para regla "{ $rule }"
detection-cooldowns-reset = Enfriamientos reiniciados para { $rule_count } reglas

# Errores de plantillas
error-template-not-found = Plantilla { $template_id } no encontrada
error-template-category-not-found = Categoría de plantilla { $category_id } no encontrada
error-template-composition-failed = La composición de plantilla falló: { $reason }
error-template-include-not-found = Archivo incluido no encontrado: { $path }
error-template-circular-include = Inclusión circular detectada (profundidad máxima { $depth } excedida)
error-template-placeholder-unresolved = Marcador de posición sin resolver: ${ $placeholder }
error-template-validation-failed = La validación de plantilla falló con { $count } errores
error-template-activation-failed = La activación de plantilla falló: { $reason }
error-template-catalog-load-failed = Error al cargar el catálogo de plantillas: { $path }
error-template-invalid-version = Formato de versión de plantilla inválido: { $version }

# Información de plantillas
template-catalog-loaded = Catálogo de plantillas cargado con { $count } plantillas
template-activated = Plantilla { $template_id } activada exitosamente
template-composition-complete = Composición completada: { $included } incluidas, { $overridden } anuladas
template-placeholders-resolved = { $count } marcadores de posición resueltos
template-validation-passed = Validación de plantilla aprobada para dominio { $domain }
template-validation-warnings = Validación de plantilla completada con { $count } advertencias
template-include-resolved = Inclusión resuelta: { $path }
template-provider-instructions-injected = Instrucciones del proveedor inyectadas para { $provider }
template-project-config-loaded = Configuración del proyecto cargada desde { $path }
template-domain-validator-registered = Validador de dominio registrado: { $domain }

# Errores de licencia
error-license-file-not-found = Archivo de licencia no encontrado en { $path }
error-license-file-invalid = Archivo de licencia inválido en { $path }: { $reason }
error-license-file-write-failed = Error al escribir archivo de licencia en { $path }: { $reason }
error-license-signature-invalid = Verificación de firma de licencia fallida
error-license-signature-decode-failed = Error al decodificar firma de licencia: { $reason }
error-license-serialization-failed = Error al serializar datos de licencia: { $reason }
error-license-signing-failed = Error al firmar licencia: { $reason }
error-license-feature-denied = La función '{ $feature }' no está disponible en el nivel { $tier }
error-license-worker-limit = Los { $requested } trabajadores solicitados exceden el límite de { $limit } del nivel { $tier }
error-license-manager-limit = Los { $requested } gestores solicitados exceden el límite de { $limit } del nivel { $tier }. Active su suscripción en Configuración → Licencia para desbloquear más.
error-license-category-denied = La categoría '{ $category }' requiere nivel { $required_tier } (actual: { $tier })
error-license-fingerprint-hostname = Error al determinar el nombre del host: { $reason }
error-license-fingerprint-username = Error al determinar el nombre de usuario: { $reason }

# Estado de licencia
license-trial-active = Licencia de prueba activa ({ $days } días restantes)
license-trial-expiring-soon = Prueba expirando pronto ({ $days } días restantes)
license-trial-expiring-very-soon = Prueba expirando muy pronto ({ $days } días restantes)
license-trial-expired = La licencia de prueba ha expirado
license-expires-later = La licencia expira en { $days } días
license-expires-soon = Licencia expirando pronto ({ $days } días restantes)
license-expires-very-soon = Licencia expirando muy pronto ({ $days } días restantes)
license-expires-today = La licencia expira hoy
license-grace-period = Licencia expirada, período de gracia activo ({ $days } días restantes)
license-expired = La licencia ha expirado

# Soporte de plataforma — claves de error
error-platform-unsupported = Plataforma no soportada: { $platform }
error-platform-wsl-not-detected = Entorno WSL no detectado
error-platform-wsl-path-invalid = Ruta inválida para traducción WSL: { $path }
error-platform-layout-no-space = Contenedor demasiado pequeño para el diseño de paneles ({ $width }x{ $height })
error-platform-layout-invalid-count = Número de paneles inválido: { $count }
error-platform-hotkey-registration-failed = Registro de atajo fallido (conflicto): { $shortcut }
error-platform-hotkey-parse-failed = Error al analizar atajo de teclado: { $shortcut }
error-platform-shutdown-timeout = Tiempo de espera agotado para el proceso { $pid } ({ $label })
error-platform-shutdown-failed = Error al detener el proceso { $pid } ({ $label })
error-platform-shell-not-found = Shell predeterminado no encontrado

# Soporte de plataforma — claves informativas
platform-detected = Plataforma detectada: { $os } ({ $arch })
platform-wsl-detected = WSL detectado: { $distro } (WSL{ $version })
platform-wsl-path-translated = Ruta traducida: { $from } → { $to }
platform-layout-calculated = Diseño calculado: { $panels } paneles en cuadrícula { $rows }x{ $cols }
platform-layout-optimized = Diseño optimizado: { $utilization }% utilización
platform-hotkey-registered = Atajo registrado: { $command } → { $shortcut }
platform-hotkey-unregistered = Atajo eliminado: { $command }
platform-shutdown-initiated = Apagado iniciado para { $count } procesos
platform-shutdown-completed = Apagado completado: { $count } procesos en { $duration }ms
platform-shell-detected = Shell detectado: { $shell } ({ $path })

# Errores de memoria
error-memory-not-found = Entrada de memoria no encontrada: { $id }
error-memory-duplicate = Entrada de memoria duplicada: { $id }
error-memory-persistence-failed = Error al persistir el almacén de memoria: { $reason }
error-memory-load-failed = Error al cargar el almacén de memoria: { $reason }
error-memory-invalid-confidence = Puntuación de confianza inválida: { $value } (debe ser 0.0-1.0)
error-memory-store-full = El almacén de memoria está lleno (máximo { $max } entradas)
error-memory-invalid-query = Consulta de memoria inválida: { $reason }
error-memory-serialization = Error en la serialización de memoria: { $reason }
error-memory-invalid-entry = Entrada de memoria inválida: { $reason }
error-memory-session-mismatch = Discrepancia de sesión: esperada { $expected }, obtenida { $actual }

# Información de memoria
memory-store-created = Almacén de memoria creado para la sesión { $session_id }
memory-entry-added = Entrada de memoria añadida: { $title } (tipo: { $memory_type })
memory-entry-updated = Entrada de memoria actualizada: { $id }
memory-entry-removed = Entrada de memoria eliminada: { $id }
memory-store-cleared = Almacén de memoria limpiado ({ $count } entradas eliminadas)
memory-persisted = Almacén de memoria persistido en { $path }
memory-loaded = Almacén de memoria cargado desde { $path } ({ $count } entradas)
memory-query-executed = Consulta de memoria devolvió { $count } resultados
memory-injected = { $count } memorias inyectadas ({ $tokens } tokens)
memory-stats = Estadísticas de memoria: { $total } entradas, confianza promedio { $avg_confidence }

# Errores de indexación
error-indexing-parse-failed = Error al analizar { $file }: { $reason }
error-indexing-file-read-failed = Error al leer el archivo { $file }: { $reason }
error-indexing-unsupported-language = Lenguaje no soportado para la extensión de archivo: { $extension }
error-indexing-extraction-failed = La extracción de símbolos falló para { $file }: { $reason }
error-indexing-graph-cycle-detected = Ciclo de dependencia detectado: { $files }
error-indexing-fingerprint-failed = Error al calcular la huella digital para { $file }: { $reason }
error-indexing-build-failed = La construcción del índice falló: { $reason }
error-indexing-update-failed = La actualización incremental falló: { $reason }

# Información de indexación
indexing-file-indexed = Archivo indexado: { $file } ({ $language })
indexing-symbols-extracted = { $count } símbolos extraídos de { $file }
indexing-graph-built = Grafo de dependencias construido: { $files } archivos, { $edges } aristas
indexing-ranking-computed = Clasificación calculada para { $symbols } símbolos
indexing-repomap-generated = Mapa del repositorio generado: { $symbols } símbolos, { $tokens } tokens
indexing-index-built = Índice del código fuente construido: { $files } archivos, { $symbols } símbolos
indexing-incremental-update = Actualización incremental: { $added } añadidos, { $modified } modificados, { $removed } eliminados
indexing-language-registered = Lenguaje registrado: { $language }

# Errores de contexto
error-context-budget-exceeded = Presupuesto de tokens de contexto excedido: usados { $used }, presupuesto { $budget }
error-context-invalid-allocations = Las asignaciones del presupuesto deben sumar <= 1.0, obtenido { $sum }
error-context-build-failed = La construcción del contexto falló para la tarea { $task_id }: { $reason }
error-context-invalid-format = Formato de contexto inválido: { $format }

# Información de contexto
context-budget-allocated = Presupuesto de tokens asignado: { $total } tokens ({ $repo_map } mapa de repositorio, { $files } archivos, { $memory } memoria, { $task } tarea)
context-files-scored = { $count } archivos puntuados por relevancia (principal: { $top_file })
context-chunks-created = { $count } fragmentos de código creados ({ $tokens } tokens)
context-assembled = Contexto ensamblado: { $sections } secciones, { $tokens } tokens usados de { $budget } presupuesto
context-injected = Contexto inyectado para el trabajador { $worker_id } ({ $tokens } tokens, { $files } archivos)
context-skipped = Preparación de contexto omitida: { $reason }

# MCP errors
error-mcp-parse-failed = Error al analizar mensaje JSON-RPC: { $reason }
error-mcp-invalid-request = Solicitud JSON-RPC inválida: { $reason }
error-mcp-method-not-found = Método no encontrado: { $method }
error-mcp-invalid-params = Parámetros inválidos: { $reason }
error-mcp-internal-error = Error interno del servidor MCP: { $reason }
error-mcp-not-initialized = El servidor MCP no ha sido inicializado
error-mcp-tool-not-found = Herramienta no encontrada: { $tool }
error-mcp-tool-execution-failed = La ejecución de la herramienta "{ $tool }" falló: { $reason }
error-mcp-transport-error = Error de transporte MCP: { $reason }
error-mcp-shutdown-failed = Error al detener el servidor MCP: { $reason }

# MCP info
mcp-server-started = Servidor MCP iniciado (transporte { $transport })
mcp-server-stopped = Servidor MCP detenido
mcp-client-initialized = Cliente MCP inicializado: { $client_name }
mcp-tool-called = Herramienta invocada: { $tool }
mcp-tool-completed = Herramienta "{ $tool }" completada en { $duration }ms
mcp-request-received = Solicitud recibida: { $method }
mcp-response-sent = Respuesta enviada: { $method }
mcp-transport-ready = Transporte MCP listo: { $transport }

# Graph errors
error-graph-entity-not-found = Entidad del grafo no encontrada: { $id }
error-graph-relationship-failed = Error al añadir relación: { $reason }
error-graph-build-failed = Error al construir el grafo de conocimiento: { $reason }
error-graph-update-failed = Error al actualizar el grafo de conocimiento: { $reason }
error-graph-load-failed = Error al cargar el grafo de conocimiento desde { $path }: { $reason }
error-graph-save-failed = Error al guardar el grafo de conocimiento en { $path }: { $reason }
error-graph-max-entities-exceeded = El grafo de conocimiento superó el límite máximo de entidades: { $count } / { $max }

# Graph info
graph-built = Grafo de conocimiento construido con { $entities } entidades y { $relationships } relaciones
graph-updated = Grafo de conocimiento actualizado: { $added } añadidas, { $removed } eliminadas
graph-entity-added = Entidad añadida al grafo de conocimiento: { $name } ({ $kind })
graph-entity-removed = Entidad eliminada del grafo de conocimiento: { $name }
graph-persisted = Grafo de conocimiento persistido en { $path }
graph-loaded = Grafo de conocimiento cargado desde { $path } ({ $entities } entidades)
graph-query-executed = Consulta del grafo ejecutada en { $ms }ms, { $results } resultados

# Errores de API de plataforma
error-platform-api-request-failed = La solicitud a la API de plataforma falló: { $reason }
error-platform-api-unauthorized = La autenticación con la API de plataforma falló — verifique channel_api_key
error-platform-api-not-found = Recurso de plataforma no encontrado: { $resource }
error-platform-api-rate-limited = Límite de tasa de la API de plataforma alcanzado — reintente después de { $seconds }s
error-platform-api-server-error = Error del servidor de plataforma ({ $status }): { $message }
error-platform-trial-not-eligible = Este dispositivo no es elegible para una prueba: { $reason }
error-platform-activation-failed = La activación de licencia falló: { $reason }
error-platform-validation-failed = La validación de licencia falló: { $reason }
error-platform-deactivation-failed = La desactivación del dispositivo falló: { $reason }
error-platform-cache-read-failed = Error al leer la caché de licencia de { $path }: { $reason }
error-platform-cache-write-failed = Error al escribir la caché de licencia en { $path }: { $reason }
error-platform-cache-decrypt-failed = Error al descifrar la caché de licencia (clave incorrecta o corrupción)
error-platform-not-configured = La integración de plataforma no está configurada — establezca platform_base_url en la configuración

# Información de API de plataforma
platform-api-trial-activated = Prueba activada: nivel { $tier }, { $days } días
platform-api-license-activated = Licencia activada: nivel { $tier } (activación { $activation_id })
platform-api-license-validated = Licencia validada: nivel { $tier }, { $days } días restantes
platform-api-heartbeat-sent = Heartbeat enviado (activación { $activation_id })
platform-api-device-deactivated = Dispositivo desactivado de la licencia
platform-api-cache-updated = Caché de licencia actualizada en { $path }
platform-api-offline-fallback = Plataforma inalcanzable, usando licencia en caché (almacenada hace { $days_ago } días)

# Errores de mensajería
error-messaging-not-registered = El cliente de mensajería no está registrado
error-messaging-registration-failed = El registro de mensajería falló: { $reason }
error-messaging-send-failed = Error al enviar mensaje: { $reason }
error-messaging-poll-failed = Error al consultar mensajes: { $reason }
error-messaging-ack-failed = Error al confirmar recepción del mensaje { $message_id }: { $reason }
error-messaging-disabled = La mensajería está deshabilitada para esta licencia

# Información de mensajería
messaging-registered = Mensajería registrada para el dispositivo { $device_id }
messaging-unregistered = Mensajería desregistrada
messaging-message-received = Mensaje recibido: { $subject } (tipo: { $message_type })
messaging-message-sent = Mensaje enviado (id: { $message_id })
messaging-poll-completed = Consulta de mensajes completada: { $count } mensajes nuevos

# Provider credential descriptions
credential-xai-api-key = Clave API de xAI para Grok (XAI_API_KEY)
credential-openai-api-key = Clave API de OpenAI (OPENAI_API_KEY)
credential-google-api-key = Clave API de Google para Gemini (GOOGLE_API_KEY)
credential-gh-auth = Autenticación de GitHub vía gh CLI (gh auth login)

# Built-in category names
category-SoftwareDevelopment = Desarrollo de Software
category-LinuxDevelopment = Desarrollo Linux
category-macOSDevelopment = Desarrollo macOS
category-PythonDevelopment = Desarrollo Python
category-AIFrameworks = Frameworks de IA y ML
category-GraphQL = Frameworks GraphQL
category-DataScience = Ciencia de Datos y Analítica
category-Legal = Legal / Paralegal
category-Music = Producción Musical
category-PhysicalSystems = Sistemas Físicos y Fenómenos
category-BacteriaScience = Ciencia de Bacterias y Microbiología
category-NursingScience = Ciencia de Enfermería y Práctica Clínica
category-ElectronDevelopment = Desarrollo de Escritorio con Electron
category-GameDevelopment = Desarrollo de Videojuegos
category-3DModeling = Modelado 3D y Creación de Contenido Digital
category-Custom = Plantillas Personalizadas

# Built-in category descriptions
category-SoftwareDevelopment-desc = Plantillas para crear aplicaciones, APIs, bases de datos y scripts
category-LinuxDevelopment-desc = Plantillas para administración de sistemas Linux, scripting de shell y desarrollo de servidores
category-macOSDevelopment-desc = Plantillas para aplicaciones macOS, desarrollo en Swift/Objective-C y frameworks de Apple
category-PythonDevelopment-desc = Plantillas para aplicaciones Python, scripts, frameworks web y automatización
category-AIFrameworks-desc = Plantillas para agentes de IA, orquestación de LLM, chatbots y aplicaciones de ML
category-GraphQL-desc = Plantillas para servidores GraphQL, clientes y desarrollo de APIs
category-DataScience-desc = Plantillas para el ciclo de vida de ciencia de datos: matemáticas, ingeniería de datos, ML, aprendizaje profundo, MLOps
category-Legal-desc = Plantillas para procesamiento de documentos legales, investigación y gestión de casos
category-Music-desc = Plantillas para DAWs, desarrollo de plugins, síntesis modular e integración de hardware
category-PhysicalSystems-desc = Plantillas para física industrial, monitoreo de procesos, sistemas de control y analítica predictiva
category-BacteriaScience-desc = Plantillas para microbiología, genómica, metagenómica, resistencia antimicrobiana y diagnósticos
category-NursingScience-desc = Plantillas para educación en enfermería, práctica clínica, atención al paciente y analítica de salud
category-ElectronDevelopment-desc = Plantillas para aplicaciones de escritorio multiplataforma con Electron y herramientas modernas
category-GameDevelopment-desc = Plantillas para motores de juego, frameworks y desarrollo de entretenimiento interactivo
category-3DModeling-desc = Plantillas para modelado 3D, VFX, animación y herramientas de creación de contenido digital
category-Custom-desc = Plantillas personalizadas creadas por el usuario

# Provider status
provider-not-installed = El proveedor { $provider } requiere { $binary } que no está instalado
provider-binary-found = { $binary } encontrado en { $path }
provider-test-timeout = La prueba de conexión expiró después de { $seconds } segundos
provider-test-failed = Prueba del proveedor fallida: { $error }
provider-env-saved = { $env_var } guardada para { $provider }

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
error-orchestration-quorum-not-met = Quórum de gerentes no alcanzado: { $received } de { $expected } propuestas recibidas (mínimo: { $min_quorum })
orchestration-file-conflict-serialized = Tarea { $later_task } serializada después de { $earlier_task } — ambas reclaman el archivo '{ $file }'
orchestration-build-check-failed = Verificación de compilación { $project_type } por trabajador { $worker_id } fallida
orchestration-build-check-passed = Verificación de compilación { $project_type } por trabajador { $worker_id } superada
