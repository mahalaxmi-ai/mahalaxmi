# Erreurs
error-config-file-not-found = Fichier de configuration introuvable : { $path }
error-config-parse-failed = Impossible d'analyser la configuration : { $reason }
error-config-validation-failed = La validation de la configuration a échoué : { $details }
error-locale-not-supported = La langue « { $locale } » n'est pas prise en charge
error-log-init-failed = Impossible d'initialiser la journalisation : { $reason }
error-log-dir-create-failed = Impossible de créer le répertoire de journalisation { $path } : { $reason }
error-app-launch-failed = Échec du lancement de l'application : { $reason }

# Validation
validation-invalid-log-level = Niveau de journalisation « { $level } » non valide. Niveaux acceptés : { $valid }
validation-workers-out-of-range = max_concurrent_workers doit être compris entre { $min } et { $max }, valeur reçue : { $value }
validation-manager-timeout-too-low = manager_timeout_seconds doit être au minimum { $min }, valeur reçue : { $value }
validation-worker-timeout-too-low = worker_timeout_seconds doit être au minimum { $min }, valeur reçue : { $value }
validation-offline-grace-too-low = offline_grace_days doit être au minimum { $min }, valeur reçue : { $value }
validation-invalid-consensus-strategy = Stratégie de consensus invalide « { $value } ». Stratégies valides : { $valid }
validation-invalid-data-directory = data_directory contient des caractères invalides
validation-empty-default-provider = providers.default_provider ne doit pas être vide
validation-invalid-theme = Thème invalide « { $value } ». Thèmes valides : { $valid }
validation-font-size-out-of-range = terminal_font_size doit être compris entre { $min } et { $max }, valeur reçue : { $value }
validation-invalid-max-batch-retries = max_batch_retries doit être >= 1, valeur reçue : { $value }
validation-invalid-max-total-batches = max_total_batches doit être >= 2, valeur reçue : { $value }
validation-invalid-stall-detection-threshold = stall_detection_threshold doit être >= 2, valeur reçue : { $value }

# Configuration
config-loaded-successfully = Configuration chargée depuis { $path }
config-using-default = Aucun fichier de configuration trouvé, utilisation des valeurs par défaut
config-env-override = Valeur de configuration remplacée par la variable d'environnement { $var }
config-env-override-invalid = La variable d'environnement { $var } a une valeur invalide « { $value } », conservation de la valeur configurée
config-generated-successfully = Configuration par défaut générée dans { $path }
config-already-exists = Le fichier de configuration existe déjà dans { $path }

# Journalisation
logging-initialized = Journalisation initialisée au niveau { $level }
logging-rust-log-override = Variable d'environnement RUST_LOG détectée, le niveau configuré est remplacé
logging-file-path = Fichier de journalisation : { $path }
logging-dir-create-failed-fallback = Impossible de créer le répertoire de journalisation { $path }, utilisation de la journalisation console uniquement

# PTY
pty-open-failed = Impossible d'ouvrir le pseudo-terminal : { $reason }
pty-spawn-failed = Impossible de lancer « { $program } » dans le PTY : { $reason }
pty-write-failed = Impossible d'écrire dans le terminal { $terminal_id } : { $reason }
pty-read-failed = Impossible de lire depuis le terminal { $terminal_id } : { $reason }
pty-resize-failed = Impossible de redimensionner le terminal { $terminal_id } à { $rows }x{ $cols } : { $reason }
pty-wait-failed = Impossible de vérifier l'état du processus du terminal { $terminal_id } : { $reason }
pty-kill-failed = Impossible de terminer le processus du terminal { $terminal_id } : { $reason }

# Application
app-starting = Mahalaxmi v{ $version } en cours de démarrage
app-ready = Mahalaxmi est prêt
app-shutting-down = Arrêt de Mahalaxmi en cours

# Identifiants
credential-anthropic-api-key = Clé API Anthropic pour Claude Code
credential-generic-api-key = Clé API pour { $provider }
credential-aws-iam-role = Rôle AWS IAM pour { $provider }
credential-oauth-token = Jeton OAuth pour { $provider }

# Fournisseur
error-provider-credentials-missing = Identifiants { $provider } manquants : la variable d'environnement { $env_var } n'est pas définie
error-provider-credentials-invalid = Les identifiants { $provider } sont invalides : { $reason }
error-provider-not-found = Fournisseur « { $provider_id } » non trouvé dans le registre
error-provider-no-default = Aucun fournisseur d'IA par défaut configuré
error-provider-command-build-failed = Échec de la construction de la commande { $provider } : { $reason }
provider-registered = Fournisseur « { $provider } » enregistré avec l'ID « { $id } »
provider-set-default = Fournisseur par défaut défini sur « { $provider } »
provider-credentials-valid = Identifiants { $provider } validés avec succès
provider-validating = Validation des identifiants { $provider }
provider-list-header = Fournisseurs d'IA enregistrés

# PTY (étendu)
error-pty-open-failed = Échec de l'ouverture du PTY : { $reason }
error-pty-spawn-failed = Échec du lancement de « { $program } » dans le PTY : { $reason }
error-pty-write-failed = Échec de l'écriture dans le terminal { $terminal_id } : { $reason }
error-pty-read-failed = Échec de la lecture du terminal { $terminal_id } : { $reason }
error-pty-resize-failed = Échec du redimensionnement du terminal { $terminal_id } à { $rows }x{ $cols } : { $reason }
error-pty-kill-failed = Échec de l'arrêt du processus du terminal { $terminal_id } : { $reason }
error-pty-wait-failed = Échec de la vérification de l'état du terminal { $terminal_id } : { $reason }
error-pty-terminal-not-found = Terminal { $terminal_id } non trouvé
error-pty-max-concurrent-reached = Nombre maximum de terminaux simultanés ({ $max }) atteint
pty-process-spawned = Processus « { $program } » lancé dans le terminal { $terminal_id }
pty-process-exited = Le processus du terminal { $terminal_id } s'est terminé avec le code { $exit_code }
pty-session-closed = Session du terminal { $terminal_id } fermée
pty-resized = Terminal { $terminal_id } redimensionné à { $rows }x{ $cols }
pty-reader-eof = Le lecteur du terminal { $terminal_id } a atteint la fin du flux
pty-reader-error = Erreur du lecteur du terminal { $terminal_id } : { $reason }

# Erreurs d'orchestration
error-orchestration-invalid-transition = Transition d'état invalide de { $from } vers { $to }
error-orchestration-circular-dependency = Dépendance circulaire détectée : { $cycle }
error-orchestration-worker-not-found = Travailleur { $worker_id } introuvable dans la file d'attente
error-orchestration-max-retries-exceeded = Le travailleur { $worker_id } a dépassé le nombre maximum de tentatives ({ $max_retries })
error-orchestration-no-proposals = Aucune proposition de gestionnaire reçue
error-orchestration-plan-validation-failed = La validation du plan d'exécution a échoué : { $errors }
error-orchestration-consensus-failed = Le moteur de consensus a échoué : { $reason }
error-orchestration-queue-full = La file de travailleurs est pleine (maximum { $max })
error-orchestration-manager-timeout = Le gestionnaire { $manager_id } a expiré après { $timeout }s
error-orchestration-worker-timeout = Le travailleur { $worker_id } a expiré après { $timeout }s

# Informations d'orchestration
orchestration-cycle-started = Cycle d'orchestration { $cycle_id } démarré
orchestration-state-changed = État modifié : { $from } -> { $to }
orchestration-manager-completed = Gestionnaire { $manager_id } terminé avec { $task_count } tâches
orchestration-consensus-reached = Consensus atteint : { $agreed } acceptées, { $dissenting } rejetées
orchestration-plan-created = Plan d'exécution créé : { $phases } phases, { $workers } travailleurs
orchestration-worker-started = Travailleur { $worker_id } démarré : { $task }
orchestration-worker-completed = Travailleur { $worker_id } terminé en { $duration }ms
orchestration-worker-failed = Travailleur { $worker_id } échoué : { $error }
orchestration-cycle-completed = Cycle terminé en { $duration }ms (taux de réussite : { $success_rate })
orchestration-worker-retrying = Travailleur { $worker_id } nouvelle tentative (tentative { $attempt }/{ $max })

# Erreurs de détection
error-detection-rule-compile-failed = Échec de la compilation du modèle de règle de détection : { $reason }
error-detection-no-rules-loaded = Aucune règle de détection chargée
error-detection-invalid-pattern = Modèle de détection invalide « { $pattern } » : { $reason }

# Informations de détection
detection-rule-matched = Règle de détection « { $rule } » correspondante, action : { $action }
detection-rule-cooldown = Règle de détection « { $rule } » supprimée par temps de recharge ({ $remaining_ms }ms restantes)
detection-rules-loaded = { $count } règles de détection chargées
detection-provider-rules-applied = { $count } règles appliquées pour le fournisseur { $provider }
detection-error-pattern-detected = Modèle d'erreur détecté : « { $pattern } » (vu { $count } fois)
detection-root-cause-hypothesis = Hypothèse de cause racine : { $category } (confiance : { $confidence })
detection-recurring-error = Erreur récurrente : « { $message } » (survenue { $count } fois)
detection-action-executed = Action { $action } exécutée pour la règle « { $rule } »
detection-cooldowns-reset = Temps de recharge réinitialisés pour { $rule_count } règles

# Erreurs de modèles
error-template-not-found = Modèle { $template_id } introuvable
error-template-category-not-found = Catégorie de modèle { $category_id } introuvable
error-template-composition-failed = La composition du modèle a échoué : { $reason }
error-template-include-not-found = Fichier inclus introuvable : { $path }
error-template-circular-include = Inclusion circulaire détectée (profondeur maximale { $depth } dépassée)
error-template-placeholder-unresolved = Espace réservé non résolu : ${ $placeholder }
error-template-validation-failed = La validation du modèle a échoué avec { $count } erreurs
error-template-activation-failed = L'activation du modèle a échoué : { $reason }
error-template-catalog-load-failed = Échec du chargement du catalogue de modèles : { $path }
error-template-invalid-version = Format de version de modèle invalide : { $version }

# Informations sur les modèles
template-catalog-loaded = Catalogue de modèles chargé avec { $count } modèles
template-activated = Modèle { $template_id } activé avec succès
template-composition-complete = Composition terminée : { $included } inclus, { $overridden } remplacés
template-placeholders-resolved = { $count } espaces réservés résolus
template-validation-passed = Validation du modèle réussie pour le domaine { $domain }
template-validation-warnings = Validation du modèle terminée avec { $count } avertissements
template-include-resolved = Inclusion résolue : { $path }
template-provider-instructions-injected = Instructions du fournisseur injectées pour { $provider }
template-project-config-loaded = Configuration du projet chargée depuis { $path }
template-domain-validator-registered = Validateur de domaine enregistré : { $domain }

# Erreurs de licence
error-license-file-not-found = Fichier de licence introuvable à { $path }
error-license-file-invalid = Fichier de licence invalide à { $path } : { $reason }
error-license-file-write-failed = Échec de l'écriture du fichier de licence à { $path } : { $reason }
error-license-signature-invalid = Échec de la vérification de la signature de licence
error-license-signature-decode-failed = Échec du décodage de la signature de licence : { $reason }
error-license-serialization-failed = Échec de la sérialisation des données de licence : { $reason }
error-license-signing-failed = Échec de la signature de licence : { $reason }
error-license-feature-denied = La fonctionnalité '{ $feature }' n'est pas disponible au niveau { $tier }
error-license-worker-limit = Les { $requested } agents demandés dépassent la limite de { $limit } du niveau { $tier }
error-license-manager-limit = Les { $requested } gestionnaires demandés dépassent la limite de { $limit } du niveau { $tier }. Activez votre abonnement dans Paramètres → Licence pour débloquer davantage.
error-license-category-denied = La catégorie '{ $category }' nécessite le niveau { $required_tier } (actuel : { $tier })
error-license-fingerprint-hostname = Impossible de déterminer le nom d'hôte : { $reason }
error-license-fingerprint-username = Impossible de déterminer le nom d'utilisateur : { $reason }

# État de la licence
license-trial-active = Licence d'essai active ({ $days } jours restants)
license-trial-expiring-soon = Essai expirant bientôt ({ $days } jours restants)
license-trial-expiring-very-soon = Essai expirant très bientôt ({ $days } jours restants)
license-trial-expired = La licence d'essai a expiré
license-expires-later = La licence expire dans { $days } jours
license-expires-soon = Licence expirant bientôt ({ $days } jours restants)
license-expires-very-soon = Licence expirant très bientôt ({ $days } jours restants)
license-expires-today = La licence expire aujourd'hui
license-grace-period = Licence expirée, période de grâce active ({ $days } jours restants)
license-expired = La licence a expiré

# Support plateforme — clés d'erreur
error-platform-unsupported = Plateforme non supportée : { $platform }
error-platform-wsl-not-detected = Environnement WSL non détecté
error-platform-wsl-path-invalid = Chemin invalide pour la traduction WSL : { $path }
error-platform-layout-no-space = Conteneur trop petit pour la disposition des panneaux ({ $width }x{ $height })
error-platform-layout-invalid-count = Nombre de panneaux invalide : { $count }
error-platform-hotkey-registration-failed = Échec de l'enregistrement du raccourci (conflit) : { $shortcut }
error-platform-hotkey-parse-failed = Échec de l'analyse du raccourci clavier : { $shortcut }
error-platform-shutdown-timeout = Délai d'attente dépassé pour le processus { $pid } ({ $label })
error-platform-shutdown-failed = Échec de l'arrêt du processus { $pid } ({ $label })
error-platform-shell-not-found = Shell par défaut introuvable

# Support plateforme — clés informatives
platform-detected = Plateforme détectée : { $os } ({ $arch })
platform-wsl-detected = WSL détecté : { $distro } (WSL{ $version })
platform-wsl-path-translated = Chemin traduit : { $from } → { $to }
platform-layout-calculated = Disposition calculée : { $panels } panneaux en grille { $rows }x{ $cols }
platform-layout-optimized = Disposition optimisée : { $utilization }% utilisation
platform-hotkey-registered = Raccourci enregistré : { $command } → { $shortcut }
platform-hotkey-unregistered = Raccourci supprimé : { $command }
platform-shutdown-initiated = Arrêt initié pour { $count } processus
platform-shutdown-completed = Arrêt terminé : { $count } processus en { $duration }ms
platform-shell-detected = Shell détecté : { $shell } ({ $path })

# Erreurs de mémoire
error-memory-not-found = Entrée de mémoire introuvable : { $id }
error-memory-duplicate = Entrée de mémoire en double : { $id }
error-memory-persistence-failed = Échec de la persistance du magasin de mémoire : { $reason }
error-memory-load-failed = Échec du chargement du magasin de mémoire : { $reason }
error-memory-invalid-confidence = Score de confiance invalide : { $value } (doit être entre 0.0 et 1.0)
error-memory-store-full = Le magasin de mémoire est plein (maximum { $max } entrées)
error-memory-invalid-query = Requête de mémoire invalide : { $reason }
error-memory-serialization = Échec de la sérialisation de la mémoire : { $reason }
error-memory-invalid-entry = Entrée de mémoire invalide : { $reason }
error-memory-session-mismatch = Discordance de session : attendue { $expected }, obtenue { $actual }

# Informations de mémoire
memory-store-created = Magasin de mémoire créé pour la session { $session_id }
memory-entry-added = Entrée de mémoire ajoutée : { $title } (type : { $memory_type })
memory-entry-updated = Entrée de mémoire mise à jour : { $id }
memory-entry-removed = Entrée de mémoire supprimée : { $id }
memory-store-cleared = Magasin de mémoire vidé ({ $count } entrées supprimées)
memory-persisted = Magasin de mémoire persisté dans { $path }
memory-loaded = Magasin de mémoire chargé depuis { $path } ({ $count } entrées)
memory-query-executed = Requête de mémoire a retourné { $count } résultats
memory-injected = { $count } mémoires injectées ({ $tokens } jetons)
memory-stats = Statistiques de mémoire : { $total } entrées, confiance moyenne { $avg_confidence }

# Erreurs d'indexation
error-indexing-parse-failed = Échec de l'analyse de { $file } : { $reason }
error-indexing-file-read-failed = Échec de la lecture du fichier { $file } : { $reason }
error-indexing-unsupported-language = Langage non pris en charge pour l'extension de fichier : { $extension }
error-indexing-extraction-failed = L'extraction des symboles a échoué pour { $file } : { $reason }
error-indexing-graph-cycle-detected = Cycle de dépendance détecté : { $files }
error-indexing-fingerprint-failed = Échec du calcul de l'empreinte pour { $file } : { $reason }
error-indexing-build-failed = Échec de la construction de l'index : { $reason }
error-indexing-update-failed = Échec de la mise à jour incrémentale : { $reason }

# Informations d'indexation
indexing-file-indexed = Fichier indexé : { $file } ({ $language })
indexing-symbols-extracted = { $count } symboles extraits de { $file }
indexing-graph-built = Graphe de dépendances construit : { $files } fichiers, { $edges } arêtes
indexing-ranking-computed = Classement calculé pour { $symbols } symboles
indexing-repomap-generated = Carte du dépôt générée : { $symbols } symboles, { $tokens } jetons
indexing-index-built = Index du code source construit : { $files } fichiers, { $symbols } symboles
indexing-incremental-update = Mise à jour incrémentale : { $added } ajoutés, { $modified } modifiés, { $removed } supprimés
indexing-language-registered = Langage enregistré : { $language }

# Erreurs de contexte
error-context-budget-exceeded = Budget de tokens de contexte dépassé : utilisés { $used }, budget { $budget }
error-context-invalid-allocations = Les allocations du budget doivent totaliser <= 1.0, obtenu { $sum }
error-context-build-failed = La construction du contexte a échoué pour la tâche { $task_id } : { $reason }
error-context-invalid-format = Format de contexte invalide : { $format }

# Informations de contexte
context-budget-allocated = Budget de tokens alloué : { $total } tokens ({ $repo_map } carte du dépôt, { $files } fichiers, { $memory } mémoire, { $task } tâche)
context-files-scored = { $count } fichiers évalués par pertinence (principal : { $top_file })
context-chunks-created = { $count } fragments de code créés ({ $tokens } tokens)
context-assembled = Contexte assemblé : { $sections } sections, { $tokens } tokens utilisés sur { $budget } budget
context-injected = Contexte injecté pour le worker { $worker_id } ({ $tokens } tokens, { $files } fichiers)
context-skipped = Préparation du contexte ignorée : { $reason }

# MCP errors
error-mcp-parse-failed = Échec de l'analyse du message JSON-RPC : { $reason }
error-mcp-invalid-request = Requête JSON-RPC invalide : { $reason }
error-mcp-method-not-found = Méthode introuvable : { $method }
error-mcp-invalid-params = Paramètres invalides : { $reason }
error-mcp-internal-error = Erreur interne du serveur MCP : { $reason }
error-mcp-not-initialized = Le serveur MCP n'a pas été initialisé
error-mcp-tool-not-found = Outil introuvable : { $tool }
error-mcp-tool-execution-failed = L'exécution de l'outil « { $tool } » a échoué : { $reason }
error-mcp-transport-error = Erreur de transport MCP : { $reason }
error-mcp-shutdown-failed = Échec de l'arrêt du serveur MCP : { $reason }

# MCP info
mcp-server-started = Serveur MCP démarré (transport { $transport })
mcp-server-stopped = Serveur MCP arrêté
mcp-client-initialized = Client MCP initialisé : { $client_name }
mcp-tool-called = Outil appelé : { $tool }
mcp-tool-completed = Outil « { $tool } » terminé en { $duration }ms
mcp-request-received = Requête reçue : { $method }
mcp-response-sent = Réponse envoyée : { $method }
mcp-transport-ready = Transport MCP prêt : { $transport }

# Graph errors
error-graph-entity-not-found = Entité du graphe introuvable : { $id }
error-graph-relationship-failed = Échec de l'ajout de la relation : { $reason }
error-graph-build-failed = Échec de la construction du graphe de connaissances : { $reason }
error-graph-update-failed = Échec de la mise à jour du graphe de connaissances : { $reason }
error-graph-load-failed = Échec du chargement du graphe de connaissances depuis { $path } : { $reason }
error-graph-save-failed = Échec de la sauvegarde du graphe de connaissances dans { $path } : { $reason }
error-graph-max-entities-exceeded = Le graphe de connaissances a dépassé la limite maximale d'entités : { $count } / { $max }

# Graph info
graph-built = Graphe de connaissances construit avec { $entities } entités et { $relationships } relations
graph-updated = Graphe de connaissances mis à jour : { $added } ajoutées, { $removed } supprimées
graph-entity-added = Entité ajoutée au graphe de connaissances : { $name } ({ $kind })
graph-entity-removed = Entité supprimée du graphe de connaissances : { $name }
graph-persisted = Graphe de connaissances persisté dans { $path }
graph-loaded = Graphe de connaissances chargé depuis { $path } ({ $entities } entités)
graph-query-executed = Requête du graphe exécutée en { $ms }ms, { $results } résultats

# Erreurs de l'API plateforme
error-platform-api-request-failed = La requête à l'API de la plateforme a échoué : { $reason }
error-platform-api-unauthorized = L'authentification de l'API de la plateforme a échoué — vérifiez channel_api_key
error-platform-api-not-found = Ressource de la plateforme introuvable : { $resource }
error-platform-api-rate-limited = Limite de débit de l'API de la plateforme atteinte — réessayez après { $seconds }s
error-platform-api-server-error = Erreur du serveur de la plateforme ({ $status }) : { $message }
error-platform-trial-not-eligible = Cet appareil n'est pas éligible à un essai : { $reason }
error-platform-activation-failed = L'activation de la licence a échoué : { $reason }
error-platform-validation-failed = La validation de la licence a échoué : { $reason }
error-platform-deactivation-failed = La désactivation de l'appareil a échoué : { $reason }
error-platform-cache-read-failed = Échec de la lecture du cache de licence depuis { $path } : { $reason }
error-platform-cache-write-failed = Échec de l'écriture du cache de licence dans { $path } : { $reason }
error-platform-cache-decrypt-failed = Échec du déchiffrement du cache de licence (clé incorrecte ou corruption)
error-platform-not-configured = L'intégration de la plateforme n'est pas configurée — définissez platform_base_url dans la configuration

# Informations de l'API plateforme
platform-api-trial-activated = Essai activé : niveau { $tier }, { $days } jours
platform-api-license-activated = Licence activée : niveau { $tier } (activation { $activation_id })
platform-api-license-validated = Licence validée : niveau { $tier }, { $days } jours restants
platform-api-heartbeat-sent = Battement de cœur envoyé (activation { $activation_id })
platform-api-device-deactivated = Appareil désactivé de la licence
platform-api-cache-updated = Cache de licence mis à jour dans { $path }
platform-api-offline-fallback = Plateforme injoignable, utilisation de la licence en cache (mise en cache il y a { $days_ago } jours)

# Erreurs de messagerie
error-messaging-not-registered = Le client de messagerie n'est pas enregistré
error-messaging-registration-failed = L'enregistrement de la messagerie a échoué : { $reason }
error-messaging-send-failed = Échec de l'envoi du message : { $reason }
error-messaging-poll-failed = Échec de la récupération des messages : { $reason }
error-messaging-ack-failed = Échec de l'accusé de réception du message { $message_id } : { $reason }
error-messaging-disabled = La messagerie est désactivée pour cette licence

# Informations de messagerie
messaging-registered = Messagerie enregistrée pour l'appareil { $device_id }
messaging-unregistered = Messagerie désenregistrée
messaging-message-received = Message reçu : { $subject } (type : { $message_type })
messaging-message-sent = Message envoyé (id : { $message_id })
messaging-poll-completed = Récupération des messages terminée : { $count } nouveaux messages

# Provider credential descriptions
credential-xai-api-key = Clé API xAI pour Grok (XAI_API_KEY)
credential-openai-api-key = Clé API OpenAI (OPENAI_API_KEY)
credential-google-api-key = Clé API Google pour Gemini (GOOGLE_API_KEY)
credential-gh-auth = Authentification GitHub via gh CLI (gh auth login)

# Built-in category names
category-SoftwareDevelopment = Développement Logiciel
category-LinuxDevelopment = Développement Linux
category-macOSDevelopment = Développement macOS
category-PythonDevelopment = Développement Python
category-AIFrameworks = Frameworks IA et ML
category-GraphQL = Frameworks GraphQL
category-DataScience = Science des Données et Analytique
category-Legal = Juridique / Parajuridique
category-Music = Production Musicale
category-PhysicalSystems = Systèmes Physiques et Phénomènes
category-BacteriaScience = Science des Bactéries et Microbiologie
category-NursingScience = Sciences Infirmières et Pratique Clinique
category-ElectronDevelopment = Développement Bureau avec Electron
category-GameDevelopment = Développement de Jeux Vidéo
category-3DModeling = Modélisation 3D et Création de Contenu Numérique
category-Custom = Modèles Personnalisés

# Built-in category descriptions
category-SoftwareDevelopment-desc = Modèles pour créer des applications, APIs, bases de données et scripts
category-LinuxDevelopment-desc = Modèles pour l'administration système Linux, les scripts shell et le développement serveur
category-macOSDevelopment-desc = Modèles pour les applications macOS, le développement Swift/Objective-C et les frameworks Apple
category-PythonDevelopment-desc = Modèles pour les applications Python, scripts, frameworks web et automatisation
category-AIFrameworks-desc = Modèles pour les agents IA, l'orchestration de LLM, les chatbots et les applications ML
category-GraphQL-desc = Modèles pour les serveurs GraphQL, clients et développement d'APIs
category-DataScience-desc = Modèles pour le cycle de vie de la science des données : mathématiques, ingénierie des données, ML, apprentissage profond, MLOps
category-Legal-desc = Modèles pour le traitement de documents juridiques, la recherche et la gestion de dossiers
category-Music-desc = Modèles pour les DAWs, le développement de plugins, la synthèse modulaire et l'intégration matérielle
category-PhysicalSystems-desc = Modèles pour la physique industrielle, la surveillance des processus, les systèmes de contrôle et l'analytique prédictive
category-BacteriaScience-desc = Modèles pour la microbiologie, la génomique, la métagénomique, la résistance antimicrobienne et les diagnostics
category-NursingScience-desc = Modèles pour l'éducation infirmière, la pratique clinique, les soins aux patients et l'analytique de santé
category-ElectronDevelopment-desc = Modèles pour les applications bureau multiplateformes avec Electron et les outils modernes
category-GameDevelopment-desc = Modèles pour les moteurs de jeu, frameworks et développement de divertissement interactif
category-3DModeling-desc = Modèles pour la modélisation 3D, les VFX, l'animation et les outils de création de contenu numérique
category-Custom-desc = Modèles personnalisés créés par l'utilisateur

# Provider status
provider-not-installed = Le fournisseur { $provider } nécessite { $binary } qui n'est pas installé
provider-binary-found = { $binary } trouvé à { $path }
provider-test-timeout = Le test de connexion a expiré après { $seconds } secondes
provider-test-failed = Échec du test du fournisseur : { $error }
provider-env-saved = { $env_var } enregistrée pour { $provider }

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
error-orchestration-quorum-not-met = Quorum des gestionnaires non atteint : { $received } sur { $expected } propositions reçues (minimum : { $min_quorum })
orchestration-file-conflict-serialized = Tâche { $later_task } sérialisée après { $earlier_task } — les deux réclament le fichier '{ $file }'
orchestration-build-check-failed = Vérification de compilation { $project_type } par travailleur { $worker_id } échouée
orchestration-build-check-passed = Vérification de compilation { $project_type } par travailleur { $worker_id } réussie
