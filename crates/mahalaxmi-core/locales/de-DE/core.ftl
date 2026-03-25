# Fehler
error-config-file-not-found = Konfigurationsdatei nicht gefunden unter { $path }
error-config-parse-failed = Konfiguration konnte nicht analysiert werden: { $reason }
error-config-validation-failed = Konfigurationsvalidierung fehlgeschlagen: { $details }
error-locale-not-supported = Die Sprache „{ $locale }" wird nicht unterstützt
error-log-init-failed = Protokollierung konnte nicht initialisiert werden: { $reason }
error-log-dir-create-failed = Protokollverzeichnis konnte nicht erstellt werden unter { $path }: { $reason }
error-app-launch-failed = Anwendungsstart fehlgeschlagen: { $reason }

# Validierung
validation-invalid-log-level = Ungültiges Protokollierungslevel „{ $level }". Gültige Level: { $valid }
validation-workers-out-of-range = max_concurrent_workers muss zwischen { $min } und { $max } liegen, erhalten: { $value }
validation-manager-timeout-too-low = manager_timeout_seconds muss mindestens { $min } betragen, erhalten: { $value }
validation-worker-timeout-too-low = worker_timeout_seconds muss mindestens { $min } betragen, erhalten: { $value }
validation-offline-grace-too-low = offline_grace_days muss mindestens { $min } betragen, erhalten: { $value }
validation-invalid-consensus-strategy = Ungültige Konsensstrategie „{ $value }". Gültige Strategien: { $valid }
validation-invalid-data-directory = data_directory enthält ungültige Zeichen
validation-empty-default-provider = providers.default_provider darf nicht leer sein
validation-invalid-theme = Ungültiges Thema „{ $value }". Gültige Themen: { $valid }
validation-font-size-out-of-range = terminal_font_size muss zwischen { $min } und { $max } liegen, erhalten: { $value }
validation-invalid-max-batch-retries = max_batch_retries muss >= 1 sein, erhalten: { $value }
validation-invalid-max-total-batches = max_total_batches muss >= 2 sein, erhalten: { $value }
validation-invalid-stall-detection-threshold = stall_detection_threshold muss >= 2 sein, erhalten: { $value }

# Konfiguration
config-loaded-successfully = Konfiguration geladen von { $path }
config-using-default = Keine Konfigurationsdatei gefunden, Standardwerte werden verwendet
config-env-override = Konfigurationswert überschrieben durch Umgebungsvariable { $var }
config-env-override-invalid = Umgebungsvariable { $var } hat ungültigen Wert „{ $value }", konfigurierter Wert wird beibehalten
config-generated-successfully = Standardkonfiguration erzeugt unter { $path }
config-already-exists = Konfigurationsdatei existiert bereits unter { $path }

# Protokollierung
logging-initialized = Protokollierung initialisiert auf Level { $level }
logging-rust-log-override = Umgebungsvariable RUST_LOG erkannt, konfiguriertes Level wird überschrieben
logging-file-path = Protokolldatei: { $path }
logging-dir-create-failed-fallback = Protokollverzeichnis { $path } konnte nicht erstellt werden, nur Konsolenprotokollierung wird verwendet

# PTY
pty-open-failed = Pseudoterminal konnte nicht geöffnet werden: { $reason }
pty-spawn-failed = „{ $program }" konnte nicht im PTY gestartet werden: { $reason }
pty-write-failed = Schreiben in Terminal { $terminal_id } fehlgeschlagen: { $reason }
pty-read-failed = Lesen von Terminal { $terminal_id } fehlgeschlagen: { $reason }
pty-resize-failed = Größenänderung von Terminal { $terminal_id } auf { $rows }x{ $cols } fehlgeschlagen: { $reason }
pty-wait-failed = Prozessstatus für Terminal { $terminal_id } konnte nicht geprüft werden: { $reason }
pty-kill-failed = Prozess in Terminal { $terminal_id } konnte nicht beendet werden: { $reason }

# Anwendung
app-starting = Mahalaxmi v{ $version } wird gestartet
app-ready = Mahalaxmi ist bereit
app-shutting-down = Mahalaxmi wird heruntergefahren

# Anmeldedaten
credential-anthropic-api-key = Anthropic-API-Schlüssel für Claude Code
credential-generic-api-key = API-Schlüssel für { $provider }
credential-aws-iam-role = AWS-IAM-Rolle für { $provider }
credential-oauth-token = OAuth-Token für { $provider }

# Anbieter
error-provider-credentials-missing = { $provider }-Anmeldedaten fehlen: Umgebungsvariable { $env_var } ist nicht gesetzt
error-provider-credentials-invalid = { $provider }-Anmeldedaten sind ungültig: { $reason }
error-provider-not-found = Anbieter „{ $provider_id }" nicht in der Registrierung gefunden
error-provider-no-default = Kein Standard-KI-Anbieter konfiguriert
error-provider-command-build-failed = Fehler beim Erstellen des { $provider }-Befehls: { $reason }
provider-registered = Anbieter „{ $provider }" mit ID „{ $id }" registriert
provider-set-default = Standardanbieter auf „{ $provider }" gesetzt
provider-credentials-valid = { $provider }-Anmeldedaten erfolgreich validiert
provider-validating = Validierung der { $provider }-Anmeldedaten
provider-list-header = Registrierte KI-Anbieter

# PTY (erweitert)
error-pty-open-failed = Fehler beim Öffnen des PTY: { $reason }
error-pty-spawn-failed = Fehler beim Starten von „{ $program }" im PTY: { $reason }
error-pty-write-failed = Fehler beim Schreiben in Terminal { $terminal_id }: { $reason }
error-pty-read-failed = Fehler beim Lesen von Terminal { $terminal_id }: { $reason }
error-pty-resize-failed = Fehler beim Ändern der Größe von Terminal { $terminal_id } auf { $rows }x{ $cols }: { $reason }
error-pty-kill-failed = Fehler beim Beenden des Prozesses von Terminal { $terminal_id }: { $reason }
error-pty-wait-failed = Fehler beim Prüfen des Status von Terminal { $terminal_id }: { $reason }
error-pty-terminal-not-found = Terminal { $terminal_id } nicht gefunden
error-pty-max-concurrent-reached = Maximale Anzahl gleichzeitiger Terminals ({ $max }) erreicht
pty-process-spawned = Prozess „{ $program }" in Terminal { $terminal_id } gestartet
pty-process-exited = Prozess von Terminal { $terminal_id } mit Code { $exit_code } beendet
pty-session-closed = Terminalsitzung { $terminal_id } geschlossen
pty-resized = Terminal { $terminal_id } auf { $rows }x{ $cols } geändert
pty-reader-eof = Leser von Terminal { $terminal_id } hat das Ende des Datenstroms erreicht
pty-reader-error = Fehler des Lesers von Terminal { $terminal_id }: { $reason }

# Orchestrierungsfehler
error-orchestration-invalid-transition = Ungültiger Zustandsübergang von { $from } nach { $to }
error-orchestration-circular-dependency = Zirkuläre Abhängigkeit erkannt: { $cycle }
error-orchestration-worker-not-found = Worker { $worker_id } nicht in der Warteschlange gefunden
error-orchestration-max-retries-exceeded = Worker { $worker_id } hat die maximalen Wiederholungsversuche überschritten ({ $max_retries })
error-orchestration-no-proposals = Keine Manager-Vorschläge erhalten
error-orchestration-plan-validation-failed = Validierung des Ausführungsplans fehlgeschlagen: { $errors }
error-orchestration-consensus-failed = Konsens-Engine fehlgeschlagen: { $reason }
error-orchestration-queue-full = Worker-Warteschlange ist voll (Maximum { $max })
error-orchestration-manager-timeout = Manager { $manager_id } hat nach { $timeout }s das Zeitlimit überschritten
error-orchestration-worker-timeout = Worker { $worker_id } hat nach { $timeout }s das Zeitlimit überschritten

# Orchestrierungsinformationen
orchestration-cycle-started = Orchestrierungszyklus { $cycle_id } gestartet
orchestration-state-changed = Zustand geändert: { $from } -> { $to }
orchestration-manager-completed = Manager { $manager_id } abgeschlossen mit { $task_count } Aufgaben
orchestration-consensus-reached = Konsens erreicht: { $agreed } zugestimmt, { $dissenting } abgelehnt
orchestration-plan-created = Ausführungsplan erstellt: { $phases } Phasen, { $workers } Worker
orchestration-worker-started = Worker { $worker_id } gestartet: { $task }
orchestration-worker-completed = Worker { $worker_id } abgeschlossen in { $duration }ms
orchestration-worker-failed = Worker { $worker_id } fehlgeschlagen: { $error }
orchestration-cycle-completed = Zyklus abgeschlossen in { $duration }ms (Erfolgsrate: { $success_rate })
orchestration-worker-retrying = Worker { $worker_id } erneuter Versuch (Versuch { $attempt }/{ $max })

# Erkennungsfehler
error-detection-rule-compile-failed = Kompilierung des Erkennungsregelmusters fehlgeschlagen: { $reason }
error-detection-no-rules-loaded = Keine Erkennungsregeln geladen
error-detection-invalid-pattern = Ungültiges Erkennungsmuster „{ $pattern }": { $reason }

# Erkennungsinformationen
detection-rule-matched = Erkennungsregel „{ $rule }" übereinstimmend, Aktion: { $action }
detection-rule-cooldown = Erkennungsregel „{ $rule }" durch Abklingzeit unterdrückt ({ $remaining_ms }ms verbleibend)
detection-rules-loaded = { $count } Erkennungsregeln geladen
detection-provider-rules-applied = { $count } Regeln für Anbieter { $provider } angewendet
detection-error-pattern-detected = Fehlermuster erkannt: „{ $pattern }" ({ $count } Mal gesehen)
detection-root-cause-hypothesis = Ursachenhypothese: { $category } (Konfidenz: { $confidence })
detection-recurring-error = Wiederkehrender Fehler: „{ $message }" ({ $count } Mal aufgetreten)
detection-action-executed = Aktion { $action } für Regel „{ $rule }" ausgeführt
detection-cooldowns-reset = Abklingzeiten für { $rule_count } Regeln zurückgesetzt

# Vorlagenfehler
error-template-not-found = Vorlage { $template_id } nicht gefunden
error-template-category-not-found = Vorlagenkategorie { $category_id } nicht gefunden
error-template-composition-failed = Vorlagenkomposition fehlgeschlagen: { $reason }
error-template-include-not-found = Include-Datei nicht gefunden: { $path }
error-template-circular-include = Zirkulärer Include erkannt (maximale Tiefe { $depth } überschritten)
error-template-placeholder-unresolved = Unaufgelöster Platzhalter: ${ $placeholder }
error-template-validation-failed = Vorlagenvalidierung mit { $count } Fehlern fehlgeschlagen
error-template-activation-failed = Vorlagenaktivierung fehlgeschlagen: { $reason }
error-template-catalog-load-failed = Laden des Vorlagenkatalogs fehlgeschlagen: { $path }
error-template-invalid-version = Ungültiges Vorlagenversionformat: { $version }

# Vorlageninformationen
template-catalog-loaded = Vorlagenkatalog mit { $count } Vorlagen geladen
template-activated = Vorlage { $template_id } erfolgreich aktiviert
template-composition-complete = Komposition abgeschlossen: { $included } eingeschlossen, { $overridden } überschrieben
template-placeholders-resolved = { $count } Platzhalter aufgelöst
template-validation-passed = Vorlagenvalidierung bestanden für Domäne { $domain }
template-validation-warnings = Vorlagenvalidierung mit { $count } Warnungen abgeschlossen
template-include-resolved = Include aufgelöst: { $path }
template-provider-instructions-injected = Anbieteranweisungen eingefügt für { $provider }
template-project-config-loaded = Projektkonfiguration geladen von { $path }
template-domain-validator-registered = Domänenvalidator registriert: { $domain }

# Lizenzfehler
error-license-file-not-found = Lizenzdatei nicht gefunden unter { $path }
error-license-file-invalid = Ungültige Lizenzdatei unter { $path }: { $reason }
error-license-file-write-failed = Lizenzdatei konnte nicht nach { $path } geschrieben werden: { $reason }
error-license-signature-invalid = Überprüfung der Lizenzsignatur fehlgeschlagen
error-license-signature-decode-failed = Dekodierung der Lizenzsignatur fehlgeschlagen: { $reason }
error-license-serialization-failed = Serialisierung der Lizenzdaten fehlgeschlagen: { $reason }
error-license-signing-failed = Signierung der Lizenz fehlgeschlagen: { $reason }
error-license-feature-denied = Die Funktion '{ $feature }' ist im Tarif { $tier } nicht verfügbar
error-license-worker-limit = Die angeforderten { $requested } Worker überschreiten das Limit von { $limit } im Tarif { $tier }
error-license-manager-limit = Die angeforderten { $requested } Manager überschreiten das Limit von { $limit } im Tarif { $tier }. Aktivieren Sie Ihr Abonnement unter Einstellungen → Lizenz, um mehr freizuschalten.
error-license-category-denied = Die Kategorie '{ $category }' erfordert den Tarif { $required_tier } (aktuell: { $tier })
error-license-fingerprint-hostname = Hostname konnte nicht ermittelt werden: { $reason }
error-license-fingerprint-username = Benutzername konnte nicht ermittelt werden: { $reason }

# Lizenzstatus
license-trial-active = Testlizenz aktiv ({ $days } Tage verbleibend)
license-trial-expiring-soon = Testphase läuft bald ab ({ $days } Tage verbleibend)
license-trial-expiring-very-soon = Testphase läuft sehr bald ab ({ $days } Tage verbleibend)
license-trial-expired = Die Testlizenz ist abgelaufen
license-expires-later = Lizenz läuft in { $days } Tagen ab
license-expires-soon = Lizenz läuft bald ab ({ $days } Tage verbleibend)
license-expires-very-soon = Lizenz läuft sehr bald ab ({ $days } Tage verbleibend)
license-expires-today = Die Lizenz läuft heute ab
license-grace-period = Lizenz abgelaufen, Kulanzzeit aktiv ({ $days } Tage verbleibend)
license-expired = Die Lizenz ist abgelaufen

# Plattformunterstützung — Fehlerschlüssel
error-platform-unsupported = Plattform nicht unterstützt: { $platform }
error-platform-wsl-not-detected = WSL-Umgebung nicht erkannt
error-platform-wsl-path-invalid = Ungültiger Pfad für WSL-Übersetzung: { $path }
error-platform-layout-no-space = Container zu klein für Panel-Layout ({ $width }x{ $height })
error-platform-layout-invalid-count = Ungültige Panel-Anzahl: { $count }
error-platform-hotkey-registration-failed = Tastenkürzel-Registrierung fehlgeschlagen (Konflikt): { $shortcut }
error-platform-hotkey-parse-failed = Tastenkürzel konnte nicht analysiert werden: { $shortcut }
error-platform-shutdown-timeout = Zeitüberschreitung beim Herunterfahren von Prozess { $pid } ({ $label })
error-platform-shutdown-failed = Herunterfahren von Prozess { $pid } fehlgeschlagen ({ $label })
error-platform-shell-not-found = Standard-Shell nicht gefunden

# Plattformunterstützung — Informationsschlüssel
platform-detected = Plattform erkannt: { $os } ({ $arch })
platform-wsl-detected = WSL erkannt: { $distro } (WSL{ $version })
platform-wsl-path-translated = Pfad übersetzt: { $from } → { $to }
platform-layout-calculated = Layout berechnet: { $panels } Panels im { $rows }x{ $cols } Raster
platform-layout-optimized = Layout optimiert: { $utilization }% Auslastung
platform-hotkey-registered = Tastenkürzel registriert: { $command } → { $shortcut }
platform-hotkey-unregistered = Tastenkürzel entfernt: { $command }
platform-shutdown-initiated = Herunterfahren für { $count } Prozesse eingeleitet
platform-shutdown-completed = Herunterfahren abgeschlossen: { $count } Prozesse in { $duration }ms
platform-shell-detected = Shell erkannt: { $shell } ({ $path })

# Speicherfehler
error-memory-not-found = Speichereintrag nicht gefunden: { $id }
error-memory-duplicate = Doppelter Speichereintrag: { $id }
error-memory-persistence-failed = Speicherung des Speicherspeichers fehlgeschlagen: { $reason }
error-memory-load-failed = Laden des Speicherspeichers fehlgeschlagen: { $reason }
error-memory-invalid-confidence = Ungültiger Vertrauenswert: { $value } (muss zwischen 0.0 und 1.0 liegen)
error-memory-store-full = Speicherspeicher ist voll (maximal { $max } Einträge)
error-memory-invalid-query = Ungültige Speicherabfrage: { $reason }
error-memory-serialization = Speicherserialisierung fehlgeschlagen: { $reason }
error-memory-invalid-entry = Ungültiger Speichereintrag: { $reason }
error-memory-session-mismatch = Sitzungsdiskrepanz: erwartet { $expected }, erhalten { $actual }

# Speicherinformationen
memory-store-created = Speicherspeicher für Sitzung { $session_id } erstellt
memory-entry-added = Speichereintrag hinzugefügt: { $title } (Typ: { $memory_type })
memory-entry-updated = Speichereintrag aktualisiert: { $id }
memory-entry-removed = Speichereintrag entfernt: { $id }
memory-store-cleared = Speicherspeicher geleert ({ $count } Einträge entfernt)
memory-persisted = Speicherspeicher gesichert in { $path }
memory-loaded = Speicherspeicher geladen von { $path } ({ $count } Einträge)
memory-query-executed = Speicherabfrage ergab { $count } Ergebnisse
memory-injected = { $count } Erinnerungen injiziert ({ $tokens } Token)
memory-stats = Speicherstatistik: { $total } Einträge, durchschnittliches Vertrauen { $avg_confidence }

# Indexierungsfehler
error-indexing-parse-failed = Analyse von { $file } fehlgeschlagen: { $reason }
error-indexing-file-read-failed = Lesen der Datei { $file } fehlgeschlagen: { $reason }
error-indexing-unsupported-language = Nicht unterstützte Sprache für Dateierweiterung: { $extension }
error-indexing-extraction-failed = Symbolextraktion für { $file } fehlgeschlagen: { $reason }
error-indexing-graph-cycle-detected = Abhängigkeitszyklus erkannt: { $files }
error-indexing-fingerprint-failed = Berechnung des Fingerabdrucks für { $file } fehlgeschlagen: { $reason }
error-indexing-build-failed = Indexerstellung fehlgeschlagen: { $reason }
error-indexing-update-failed = Inkrementelle Aktualisierung fehlgeschlagen: { $reason }

# Indexierungsinformationen
indexing-file-indexed = Datei indexiert: { $file } ({ $language })
indexing-symbols-extracted = { $count } Symbole aus { $file } extrahiert
indexing-graph-built = Abhängigkeitsgraph erstellt: { $files } Dateien, { $edges } Kanten
indexing-ranking-computed = Rangfolge für { $symbols } Symbole berechnet
indexing-repomap-generated = Repository-Karte generiert: { $symbols } Symbole, { $tokens } Token
indexing-index-built = Quellcode-Index erstellt: { $files } Dateien, { $symbols } Symbole
indexing-incremental-update = Inkrementelle Aktualisierung: { $added } hinzugefügt, { $modified } geändert, { $removed } entfernt
indexing-language-registered = Sprache registriert: { $language }

# Kontextfehler
error-context-budget-exceeded = Kontext-Token-Budget überschritten: verwendet { $used }, Budget { $budget }
error-context-invalid-allocations = Budgetzuweisungen müssen in der Summe <= 1.0 sein, erhalten { $sum }
error-context-build-failed = Kontexterstellung fehlgeschlagen für Aufgabe { $task_id }: { $reason }
error-context-invalid-format = Ungültiges Kontextformat: { $format }

# Kontextinformationen
context-budget-allocated = Token-Budget zugewiesen: { $total } Token ({ $repo_map } Repository-Karte, { $files } Dateien, { $memory } Speicher, { $task } Aufgabe)
context-files-scored = { $count } Dateien nach Relevanz bewertet (Top: { $top_file })
context-chunks-created = { $count } Code-Fragmente erstellt ({ $tokens } Token)
context-assembled = Kontext zusammengestellt: { $sections } Abschnitte, { $tokens } Token verwendet von { $budget } Budget
context-injected = Kontext injiziert für Worker { $worker_id } ({ $tokens } Token, { $files } Dateien)
context-skipped = Kontextvorbereitung übersprungen: { $reason }

# MCP-Fehler
error-mcp-parse-failed = JSON-RPC-Nachricht konnte nicht analysiert werden: { $reason }
error-mcp-invalid-request = Ungültige JSON-RPC-Anfrage: { $reason }
error-mcp-method-not-found = Methode nicht gefunden: { $method }
error-mcp-invalid-params = Ungültige Parameter: { $reason }
error-mcp-internal-error = Interner MCP-Serverfehler: { $reason }
error-mcp-not-initialized = MCP-Server wurde nicht initialisiert
error-mcp-tool-not-found = Werkzeug nicht gefunden: { $tool }
error-mcp-tool-execution-failed = Ausführung des Werkzeugs „{ $tool }" fehlgeschlagen: { $reason }
error-mcp-transport-error = MCP-Transportfehler: { $reason }
error-mcp-shutdown-failed = MCP-Server-Herunterfahren fehlgeschlagen: { $reason }

# MCP-Info
mcp-server-started = MCP-Server gestartet ({ $transport }-Transport)
mcp-server-stopped = MCP-Server gestoppt
mcp-client-initialized = MCP-Client initialisiert: { $client_name }
mcp-tool-called = Werkzeug aufgerufen: { $tool }
mcp-tool-completed = Werkzeug „{ $tool }" abgeschlossen in { $duration }ms
mcp-request-received = Anfrage empfangen: { $method }
mcp-response-sent = Antwort gesendet: { $method }
mcp-transport-ready = MCP-Transport bereit: { $transport }

# Graph errors
error-graph-entity-not-found = Graph-Entität nicht gefunden: { $id }
error-graph-relationship-failed = Beziehung konnte nicht hinzugefügt werden: { $reason }
error-graph-build-failed = Wissensgraph konnte nicht erstellt werden: { $reason }
error-graph-update-failed = Wissensgraph konnte nicht aktualisiert werden: { $reason }
error-graph-load-failed = Wissensgraph konnte nicht von { $path } geladen werden: { $reason }
error-graph-save-failed = Wissensgraph konnte nicht in { $path } gespeichert werden: { $reason }
error-graph-max-entities-exceeded = Wissensgraph hat die maximale Entitätsgrenze überschritten: { $count } / { $max }

# Graph info
graph-built = Wissensgraph erstellt mit { $entities } Entitäten und { $relationships } Beziehungen
graph-updated = Wissensgraph aktualisiert: { $added } hinzugefügt, { $removed } entfernt
graph-entity-added = Entität zum Wissensgraph hinzugefügt: { $name } ({ $kind })
graph-entity-removed = Entität aus dem Wissensgraph entfernt: { $name }
graph-persisted = Wissensgraph gespeichert in { $path }
graph-loaded = Wissensgraph geladen von { $path } ({ $entities } Entitäten)
graph-query-executed = Graph-Abfrage ausgeführt in { $ms }ms, { $results } Ergebnisse

# Plattform-API-Fehler
error-platform-api-request-failed = Plattform-API-Anfrage fehlgeschlagen: { $reason }
error-platform-api-unauthorized = Plattform-API-Authentifizierung fehlgeschlagen — überprüfen Sie channel_api_key
error-platform-api-not-found = Plattform-Ressource nicht gefunden: { $resource }
error-platform-api-rate-limited = Plattform-API-Ratenlimit erreicht — erneut versuchen nach { $seconds }s
error-platform-api-server-error = Plattform-Serverfehler ({ $status }): { $message }
error-platform-trial-not-eligible = Dieses Gerät ist nicht für eine Testversion berechtigt: { $reason }
error-platform-activation-failed = Lizenzaktivierung fehlgeschlagen: { $reason }
error-platform-validation-failed = Lizenzvalidierung fehlgeschlagen: { $reason }
error-platform-deactivation-failed = Gerätedeaktivierung fehlgeschlagen: { $reason }
error-platform-cache-read-failed = Lizenzcache konnte nicht von { $path } gelesen werden: { $reason }
error-platform-cache-write-failed = Lizenzcache konnte nicht nach { $path } geschrieben werden: { $reason }
error-platform-cache-decrypt-failed = Entschlüsselung des Lizenzcache fehlgeschlagen (falscher Schlüssel oder Beschädigung)
error-platform-not-configured = Plattform-Integration nicht konfiguriert — setzen Sie platform_base_url in der Konfiguration

# Plattform-API-Informationen
platform-api-trial-activated = Testversion aktiviert: Tarif { $tier }, { $days } Tage
platform-api-license-activated = Lizenz aktiviert: Tarif { $tier } (Aktivierung { $activation_id })
platform-api-license-validated = Lizenz validiert: Tarif { $tier }, { $days } Tage verbleibend
platform-api-heartbeat-sent = Heartbeat gesendet (Aktivierung { $activation_id })
platform-api-device-deactivated = Gerät von der Lizenz deaktiviert
platform-api-cache-updated = Lizenzcache aktualisiert unter { $path }
platform-api-offline-fallback = Plattform nicht erreichbar, verwende zwischengespeicherte Lizenz (vor { $days_ago } Tagen zwischengespeichert)

# Nachrichtenfehler
error-messaging-not-registered = Nachrichtenclient ist nicht registriert
error-messaging-registration-failed = Nachrichtenregistrierung fehlgeschlagen: { $reason }
error-messaging-send-failed = Nachricht konnte nicht gesendet werden: { $reason }
error-messaging-poll-failed = Nachrichtenabfrage fehlgeschlagen: { $reason }
error-messaging-ack-failed = Bestätigung der Nachricht { $message_id } fehlgeschlagen: { $reason }
error-messaging-disabled = Nachrichtenversand ist für diese Lizenz deaktiviert

# Nachrichteninformationen
messaging-registered = Nachrichtenversand registriert für Gerät { $device_id }
messaging-unregistered = Nachrichtenversand abgemeldet
messaging-message-received = Nachricht empfangen: { $subject } (Typ: { $message_type })
messaging-message-sent = Nachricht gesendet (ID: { $message_id })
messaging-poll-completed = Nachrichtenabfrage abgeschlossen: { $count } neue Nachrichten

# Provider credential descriptions
credential-xai-api-key = xAI API-Schlüssel für Grok (XAI_API_KEY)
credential-openai-api-key = OpenAI API-Schlüssel (OPENAI_API_KEY)
credential-google-api-key = Google API-Schlüssel für Gemini (GOOGLE_API_KEY)
credential-gh-auth = GitHub-Authentifizierung über gh CLI (gh auth login)

# Built-in category names
category-SoftwareDevelopment = Softwareentwicklung
category-LinuxDevelopment = Linux-Entwicklung
category-macOSDevelopment = macOS-Entwicklung
category-PythonDevelopment = Python-Entwicklung
category-AIFrameworks = KI- und ML-Frameworks
category-GraphQL = GraphQL-Frameworks
category-DataScience = Data Science und Analytik
category-Legal = Recht / Rechtsassistenz
category-Music = Musikproduktion
category-PhysicalSystems = Physikalische Systeme und Phänomene
category-BacteriaScience = Bakterienwissenschaft und Mikrobiologie
category-NursingScience = Pflegewissenschaft und Klinische Praxis
category-ElectronDevelopment = Electron-Desktop-Entwicklung
category-GameDevelopment = Spieleentwicklung
category-3DModeling = 3D-Modellierung und Digitale Inhaltserstellung
category-Custom = Benutzerdefinierte Vorlagen

# Built-in category descriptions
category-SoftwareDevelopment-desc = Vorlagen für die Erstellung von Anwendungen, APIs, Datenbanken und Skripten
category-LinuxDevelopment-desc = Vorlagen für Linux-Systemadministration, Shell-Scripting und Serverentwicklung
category-macOSDevelopment-desc = Vorlagen für macOS-Anwendungen, Swift/Objective-C-Entwicklung und Apple-Frameworks
category-PythonDevelopment-desc = Vorlagen für Python-Anwendungen, Skripte, Web-Frameworks und Automatisierung
category-AIFrameworks-desc = Vorlagen für KI-Agenten, LLM-Orchestrierung, Chatbots und ML-Anwendungen
category-GraphQL-desc = Vorlagen für GraphQL-Server, Clients und API-Entwicklung
category-DataScience-desc = Vorlagen für den Data-Science-Lebenszyklus: Mathematik, Datenengineering, ML, Deep Learning, MLOps
category-Legal-desc = Vorlagen für die Bearbeitung juristischer Dokumente, Recherche und Fallverwaltung
category-Music-desc = Vorlagen für DAWs, Plugin-Entwicklung, modulare Synthese und Hardware-Integration
category-PhysicalSystems-desc = Vorlagen für industrielle Physik, Prozessüberwachung, Steuerungssysteme und prädiktive Analytik
category-BacteriaScience-desc = Vorlagen für Mikrobiologie, Genomik, Metagenomik, antimikrobielle Resistenz und Diagnostik
category-NursingScience-desc = Vorlagen für Pflegeausbildung, klinische Praxis, Patientenversorgung und Gesundheitsanalytik
category-ElectronDevelopment-desc = Vorlagen für plattformübergreifende Desktop-Anwendungen mit Electron und modernen Werkzeugen
category-GameDevelopment-desc = Vorlagen für Spiele-Engines, Frameworks und interaktive Unterhaltungsentwicklung
category-3DModeling-desc = Vorlagen für 3D-Modellierung, VFX, Animation und digitale Inhaltserstellungswerkzeuge
category-Custom-desc = Vom Benutzer erstellte benutzerdefinierte Vorlagen

# Provider status
provider-not-installed = Anbieter { $provider } benötigt { $binary }, das nicht installiert ist
provider-binary-found = { $binary } gefunden unter { $path }
provider-test-timeout = Verbindungstest abgelaufen nach { $seconds } Sekunden
provider-test-failed = Anbieter-Test fehlgeschlagen: { $error }
provider-env-saved = { $env_var } für { $provider } gespeichert

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
error-orchestration-quorum-not-met = Manager-Quorum nicht erreicht: { $received } von { $expected } Vorschlägen empfangen (Minimum: { $min_quorum })
orchestration-file-conflict-serialized = Aufgabe { $later_task } nach { $earlier_task } serialisiert — beide beanspruchen Datei '{ $file }'
orchestration-build-check-failed = Per-Worker { $project_type }-Build-Prüfung für Worker { $worker_id } fehlgeschlagen
orchestration-build-check-passed = Per-Worker { $project_type }-Build-Prüfung für Worker { $worker_id } bestanden
