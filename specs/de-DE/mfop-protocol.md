# Mahalaxmi Federation and Orchestration Protocol

**MFOP v1.0** · Entwurf zur Begutachtung

| | |
|---|---|
| Date | März 2026 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Florida, USA |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## Status dieses Memos

Dieses Dokument ist ein Vorabveröffentlichungsentwurf der Spezifikation des Mahalaxmi Federation and Orchestration Protocol (MFOP), Version 1.0. Es wird zur Begutachtung durch Fachleute verteilt und zur Einholung von Kommentaren. Dieses Dokument beschreibt ein Protokoll für die föderierte verteilte KI-Orchestrierung über heterogene Rechenknoten mit Compliance-Zone-gerechtem Routing, kryptographisch signierten Abrechnungsbelegen und konfigurierbarer wirtschaftlicher Abrechnung.

Kommentare und Fragen sind an die Autorin unter Ami.nunez@mahalaxmi.ai zu richten. Der aktuelle Entwurf und Diskussionsstränge werden unter https://mahalaxmi.ai/mfop/draft gepflegt. Diskussionsstränge befinden sich unter https://mahalaxmi.ai/mfop/discuss.

## Urheberrechtshinweis

Copyright © 2026 ThriveTech Services LLC. Alle Rechte vorbehalten. Die Genehmigung wird erteilt, dieses Dokument in beliebigen Medien ohne Gebühr zu kopieren, zu verbreiten und zu verwenden, vorausgesetzt, dass die Autorennennung, der Dokumententitel und dieser Urheberrechtshinweis in allen Kopien und abgeleiteten Werken erhalten bleiben.

## Zusammenfassung

Dieses Dokument definiert das Mahalaxmi Federation and Orchestration Protocol (MFOP), ein Protokoll zur Koordinierung der parallelen Ausführung von KI-Agenten über ein verteiltes Netzwerk heterogener Rechenknoten. MFOP legt Knotenidentität und -registrierung, Fähigkeitswerbung, Compliance-Zone-gerechtes Job-Routing, semantische Eingabeaufteilung, kryptographisch signierte Abrechnungsbelege, konfigurierbare wirtschaftliche Abrechnung und ein mehrschichtiges Sicherheitsmodell unter Verwendung von KI-Sicherheitsrichtlinienvalidierung und Ausführungs-Sandbox-Isolation fest.

MFOP ist so konzipiert, dass es in drei gleichzeitigen Einsatzkonfigurationen betrieben werden kann: private Unternehmens-Meshes, bei denen Knoten von einer einzigen Organisation besessen und betrieben werden, verwaltete Cloud-Pools, die vom Plattformanbieter betrieben werden, und offene Community-Marktplätze, bei denen jeder Knotenbetreiber Rechenleistung gegen wirtschaftliche Vergütung beitragen kann. Das Protokoll ist gegenüber dem zugrunde liegenden KI-Modellanbieter agnostisch und ist darauf ausgelegt, sich mit der KI-Sicherheits- und Compliance-Landschaft weiterzuentwickeln.

## 1. Einleitung

Das Wachstum von Large Language Model (LLM)-Einsätzen in Unternehmensumgebungen hat einen Bedarf an einer Koordinierungsschicht geschaffen, die heterogene Recheninfrastruktur überbrücken kann und dabei Compliance-, Abrechnungs- und Sicherheitsanforderungen erfüllt, die je nach Rechtsgebiet und Branche variieren.

MFOP begegnet diesem Bedarf, indem es ein Protokoll für die föderierte KI-Orchestrierung definiert. Eine Föderation besteht aus einem oder mehreren Rechenknoten, die jeweils von verschiedenen Einheiten unter verschiedenen Compliance-Regimen betrieben werden können. Ein Einreicher — ein Benutzer, eine Anwendung oder ein automatisiertes System — übergibt einen Job an die Föderation. Die Föderation leitet den Job an einen geeigneten Knoten weiter, basierend auf den Compliance-Zone-Anforderungen des Jobs, der Fähigkeitswerbung des Knotens und den geltenden wirtschaftlichen Bedingungen.

Diese Spezifikation definiert das Leitungsprotokoll, Datenformate, kryptographische Mechanismen und Verhaltensanforderungen für alle Komponenten einer konformen MFOP-Föderation.

## 2. Terminologie

Die Schlüsselwörter "MUSS", "DARF NICHT", "ERFORDERLICH", "SOLL", "SOLL NICHT", "SOLLTE", "SOLLTE NICHT", "EMPFOHLEN", "NICHT EMPFOHLEN", "KANN" und "OPTIONAL" in diesem Dokument sind gemäß BCP 14 [RFC2119] [RFC8174] zu interpretieren.

**Föderation** — Eine logische Gruppierung von einem oder mehreren MFOP-konformen Rechenknoten, die unter einer gemeinsamen Governance-Konfiguration betrieben werden.

**Knoten** — Eine Rechenressource, die bei einer Föderation registriert ist und KI-Workloads annimmt, ausführt und zurückgibt. Ein Knoten kann ein einzelner Server, ein Cluster oder ein Cloud-Compute-Pool sein.

**Einreicher** — Eine Einheit (Benutzer, Anwendung oder automatisiertes System), die KI-Workloads zur Ausführung an die Föderation übermittelt.

**Compliance-Zone** — Ein benannter Richtlinienkontext, der Job-Routing, Datenverarbeitung und Ausgabevalidierung einschränkt. Definierte Zonen: public, enterprise (SOC2), hipaa, sox, fedramp.

**Job** — Eine diskrete Einheit eines KI-Workloads, die zur Ausführung an die Föderation übermittelt wird. Ein Job enthält eine Nutzlast, eine Compliance-Zone-Zusicherung und eine Abrechnungsautorisierung.

**Beleg** — Ein kryptographisch signierter Datensatz einer abgeschlossenen Job-Ausführung, einschließlich Token-Anzahl, Zeitstempel, Knotenidentität und Abrechnungsbeträgen.

**Wirtschaftliche Abrechnung** — Der Prozess, durch den angesammelte Abrechnungsbelege in finanzielle Überweisungen zwischen Einreichern, Knotenbetreibern und der Plattform umgewandelt werden.

**PAK Key (Platform API Key)** — Eine Bearer-Berechtigung, die von der Plattform ausgestellt wird und den Zugriff auf Föderations-API-Endpunkte autorisiert.

**NeMo Guardrails** — Das NVIDIA NeMo Sicherheits-Framework, das von MFOP-Knoten für die Validierung von KI-Sicherheitsrichtlinien und Ausgabefilterung verwendet wird.

## 3. Knotenidentität und Registrierung

Jeder Knoten in einer MFOP-Föderation wird durch einen stabilen, global eindeutigen Knotenbezeichner (node_id) identifiziert. Die node_id ist ein 128-Bit-UUID (Version 4), der bei der Registrierung vergeben wird und über Knotenneustart und Software-Upgrades hinaus bestehen bleibt.

**3.1 Registrierungsablauf**

Ein Knoten initiiert die Registrierung, indem er eine NodeRegistrationRequest an den Registrierungsendpunkt der Föderation (POST /v1/federation/nodes/register) sendet. Die Anfrage MUSS enthalten:

- node_id: ein Kandidaten-UUID (die Föderation KANN diesen überschreiben)
- operator_id: der UUID des registrierenden Betreiberkontos
- display_name: ein menschenlesbarer Name für den Knoten (max. 64 Zeichen)
- public_key: ein Ed25519-öffentlicher Schlüssel in base64url-Kodierung, verwendet für die Beleguнterzeichnung
- capability_advertisement: ein CapabilityAdvertisement-Objekt (siehe Abschnitt 4)
- compliance_zones: die Menge der Compliance-Zonen, für die der Knoten zertifiziert ist
- endpoint_url: die HTTPS-URL, unter der der Knoten Job-Übermittlungen annimmt

Die Föderation gibt eine NodeRegistrationResponse zurück, die die zugewiesene node_id, ein registration_token für nachfolgende authentifizierte Aufrufe und die aktuelle Abrechnungskonfiguration der Föderation enthält.

**3.2 Neu-Registrierung und Schlüsselrotation**

Knoten MÜSSEN sich neu registrieren, wenn ihr Ed25519-Schlüsselpaar rotiert wird. Während der Schlüsselrotation übermittelt der Knoten eine Neu-Registrierungsanfrage mit dem alten und dem neuen öffentlichen Schlüssel, signiert mit dem alten privaten Schlüssel. Die Föderation verifiziert die Altsignatur, bevor sie den neuen Schlüssel akzeptiert. Es gibt ein 24-Stunden-Überlappungsfenster, in dem Belege, die mit einem der beiden Schlüssel signiert sind, akzeptiert werden.

**3.3 Knotengesundheit und Abmeldung**

Knoten MÜSSEN mindestens einmal alle 60 Sekunden einen Heartbeat an POST /v1/federation/nodes/{id}/heartbeat senden. Ein Knoten, der drei aufeinanderfolgende Heartbeat-Fenster verpasst, wird als INAKTIV markiert und vom Routing ausgeschlossen. Knoten können sich freiwillig über DELETE /v1/federation/nodes/{id} abmelden.

## 4. Fähigkeitswerbung

Die Fähigkeitswerbung eines Knotens deklariert die auf dem Knoten verfügbaren KI-Modelle, die für das Job-Routing relevanten Hardware-Eigenschaften und die vom Knotenbetreiber gehaltenen Compliance-Zertifizierungen.

**4.1 CapabilityAdvertisement-Objekt**

Das CapabilityAdvertisement-Objekt enthält folgende Felder:

- models: ein Array von ModelDescriptor-Objekten (siehe 4.2)
- hardware_class: eines von { cpu, gpu_consumer, gpu_datacenter, tpu }
- vram_gb: gesamter verfügbarer GPU-VRAM für Inferenz, in Gigabyte (0 für CPU-Knoten)
- max_context_tokens: das maximale Kontextfenster, das der Knoten bedienen kann
- max_concurrent_jobs: die maximale Anzahl an Jobs, die der Knoten gleichzeitig ausführen wird
- compliance_certifications: ein Array von Zertifizierungsbezeichnern (z. B. "soc2-type2", "hipaa-baa", "fedramp-moderate")
- nemo_rails_version: die Version der auf dem Knoten installierten NeMo Guardrails-Laufzeitumgebung

**4.2 ModelDescriptor**

Jedes auf einem Knoten verfügbare Modell wird durch einen ModelDescriptor beschrieben:

- model_id: eine kanonische Modellbezeichnerzeichenkette (z. B. "meta-llama/Meta-Llama-3-70B-Instruct")
- model_family: eines von { llama, mistral, gemma, falcon, phi, custom }
- parameter_count_b: ungefähre Parameteranzahl in Milliarden
- quantization: eines von { fp16, bf16, int8, int4, none }
- context_window_tokens: das maximale Kontextfenster für dieses Modell
- supports_tool_use: boolescher Wert
- supports_vision: boolescher Wert

**4.3 Fähigkeitsaktualisierung**

Knoten MÜSSEN ihre Fähigkeitswerbung über PUT /v1/federation/nodes/{id}/capabilities aktualisieren, wann immer sich ihre verfügbaren Modelle oder Hardware-Konfiguration ändert. Die Föderation überträgt aktualisierte Fähigkeitswerbungen innerhalb von 30 Sekunden an die Routing-Schicht.

## 5. Compliance-Zone-gerechtes Job-Routing

MFOP leitet jeden Job an einen Knoten weiter, der die Compliance-Zone-Anforderungen des Jobs erfüllt. Die Erfüllung der Compliance-Zone ist eine harte Bedingung: Ein Job DARF NICHT an einen Knoten weitergeleitet werden, der nicht für die Compliance-Zone des Jobs zertifiziert ist.

**5.1 Compliance-Zonen**

MFOP definiert fünf Compliance-Zonen, geordnet von der geringsten zur restriktivsten:

- public: Keine Compliance-Anforderungen über die grundlegenden NeMo-Sicherheitsschienen hinaus. Geeignet für allgemeine KI-Workloads.
- enterprise (SOC2): Erfordert SOC 2 Typ II-Zertifizierung. Fügt Datenresidenz-Erkennung, API-Anmeldedaten-Exfiltrations-Erkennung und Zugriffsprotokollierungsdurchsetzung hinzu.
- hipaa: Erfordert HIPAA BAA. Fügt PHI-Mustererkennung, PHI-De-Identifizierung und Prüfungen für minimalnotwendige Ausgaben hinzu.
- sox: Erfordert SOX-Compliance-Kontrollen. Fügt finanzielle PII-Isolation, Preisvorhersagesperrung und MNPI-Erkennung hinzu.
- fedramp: Erfordert FedRAMP-Autorisierung. Fügt CUI-Handhabung, Exportkontroll-Erkennung und Durchsetzung von Klassifizierungskennzeichnungen hinzu.

**5.2 Routing-Algorithmus**

Wenn ein Job empfangen wird, führt die Routing-Schicht den folgenden Algorithmus aus:

1. Filtern: Alle Knoten mit Status AKTIV identifizieren, die für die Compliance-Zone des Jobs zertifiziert sind.
2. Filtern: Knoten entfernen, deren max_context_tokens kleiner ist als die geschätzte Token-Anzahl des Jobs.
3. Filtern: Knoten entfernen, deren max_concurrent_jobs aktuell erschöpft ist.
4. Bewerten: Für jeden verbleibenden Knoten einen Routing-Score berechnen: score = w_latency × latency_score + w_cost × cost_score + w_affinity × affinity_score. Standardgewichtungen: w_latency = 0,4, w_cost = 0,4, w_affinity = 0,2.
5. Auswählen: An den Knoten mit dem höchsten Score weiterleiten. Bei Gleichstand gleichmäßig zufällig auswählen.

Wenn kein Knoten alle Filter erfüllt, wird der Job mit einem konfigurierbaren Timeout in die Warteschlange gestellt (Standard: 120 Sekunden). Wenn innerhalb des Timeouts kein Knoten verfügbar wird, gibt die Föderation HTTP 503 mit einem Retry-After-Header zurück.

**5.3 Affinitätsregeln**

Einreicher KÖNNEN Affinitätsregeln in ihrer Job-Übermittlung angeben:

- node_affinity: eine Liste bevorzugter node_ids (weiche Präferenz)
- anti_affinity: eine Liste von node_ids, die ausgeschlossen werden sollen (harte Bedingung)
- geography: eine bevorzugte geografische Region (ISO 3166-1 Alpha-2-Ländercode)

Affinitätsregeln beeinflussen nur die affinity_score-Komponente; Compliance-Zone-Zertifizierung und Kapazität bleiben harte Bedingungen.

## 6. Semantische Eingabeaufteilung

Für Jobs, deren Eingabe die max_context_tokens eines einzelnen Knotens überschreitet, bietet MFOP einen semantischen Aufteilungsmechanismus, der die Eingabe in kohärente Teilaufgaben aufteilt, jeden Teilauftrag unabhängig weiterleitet und die Ergebnisse zusammenführt.

**6.1 Aufteilungsstrategien**

MFOP definiert drei Aufteilungsstrategien:

- sliding_window: Teilt die Eingabe in überlappende Fenster konfigurierbarer Größe und Überlappung auf. Geeignet für Aufgaben, bei denen Kontextkontinuität an Grenzen wichtig ist (z. B. Zusammenfassung langer Dokumente).
- semantic_boundary: Teilt an erkannten semantischen Grenzen (Absatzumbrüche, Abschnittsüberschriften, Themenübergänge). Erzeugt kohärentere Teilaufgaben auf Kosten variabler Teilaufgabengrößen.
- task_decomposition: Interpretiert die Eingabe als strukturierte Aufgabenliste und leitet jede Aufgabe als unabhängigen Teilauftrag weiter. Erfordert, dass die Eingabe dem MFOP-TaskList-Schema entspricht.

**6.2 Aufteilungsanfrage**

Ein Einreicher fordert eine partitionierte Ausführung an, indem er partition_strategy in der Job-Übermittlung setzt. Die Partitionierungsmaschine der Föderation teilt die Eingabe auf, weist Teilauftrags-IDs zu (parent_job_id + Sequenznummer) und leitet jeden Teilauftrag unabhängig weiter. Teilaufträge erben die Compliance-Zone und Abrechnungsautorisierung des übergeordneten Jobs.

**6.3 Zusammenführung**

Sobald alle Teilaufträge abgeschlossen sind, stellt die Zusammenführungsschicht der Föderation die Ergebnisse in der Reihenfolge der Sequenznummern zusammen. Für sliding_window-Partitionen dedupliziert der Aggregator Inhalte in den Überlappungsbereichen mittels einer Longest-Common-Subsequence-Zusammenführung. Das zusammengestellte Ergebnis wird dem Einreicher als einzelnes JobResult mit einem Array von sub_job_receipts zurückgegeben.

## 7. Kryptographisch signierte Abrechnungsbelege

Jede abgeschlossene Job-Ausführung erzeugt einen BillingReceipt, der vom ausführenden Knoten signiert wird. Signierte Belege sind der maßgebliche Datensatz für wirtschaftliche Abrechnung und Streitbeilegung.

**7.1 Belegstruktur**

Ein BillingReceipt enthält:

- receipt_id: ein UUID (Version 4), eindeutig für diesen Beleg
- job_id: der UUID des abgeschlossenen Jobs
- node_id: der UUID des ausführenden Knotens
- submitter_id: der UUID des Einreichers
- model_id: das für die Ausführung verwendete Modell
- compliance_zone: die Compliance-Zone, unter der der Job ausgeführt wurde
- input_tokens: die Anzahl der verarbeiteten Eingabe-Token
- output_tokens: die Anzahl der generierten Ausgabe-Token
- wall_time_ms: gesamte Ausführungszeit in Millisekunden
- completed_at: RFC 3339-Zeitstempel des Job-Abschlusses
- fee_schedule_id: der UUID der zum Ausführungszeitpunkt geltenden BillingFeeConfig
- input_token_cost_usd: berechnete Eingabe-Token-Kosten in USD (6 Dezimalstellen)
- output_token_cost_usd: berechnete Ausgabe-Token-Kosten in USD (6 Dezimalstellen)
- platform_fee_usd: die Plattformgebühr für diesen Job
- node_earnings_usd: die Einnahmen des Knotenbetreibers für diesen Job
- total_cost_usd: Gesamtkosten für den Einreicher

**7.2 Signaturschema**

Belege werden mit Ed25519 signiert. Der Knoten signiert die kanonische JSON-Serialisierung des Belegs (Schlüssel sortiert, kein Leerraum) mit seinem registrierten privaten Schlüssel. Die Signatur ist base64url-kodiert und im Beleg als Signaturfeld enthalten.

Die Föderation verifiziert die Belegsignatur beim Empfang anhand des registrierten öffentlichen Schlüssels des Knotens. Belege mit ungültigen Signaturen werden abgelehnt und lösen einen Knotenintegritätsalarm aus.

**7.3 Belegspeicherung und -abruf**

Die Föderation speichert alle Belege mindestens 7 Jahre lang, um Compliance-Audit-Anforderungen zu unterstützen. Einreicher können ihre Belege über GET /v1/federation/receipts abrufen. Knotenbetreiber können Belege für von ihnen ausgeführte Jobs über GET /v1/federation/nodes/{id}/receipts abrufen.

## 8. Konfigurierbare wirtschaftliche Abrechnung

MFOP trennt Abrechnung (die Anhäufung signierter Belege) von Abwicklung (die finanzielle Überweisung von Geldern). Die Abwicklung ist konfigurierbar und kann für verschiedene Teilnehmertypen nach unterschiedlichen Zeitplänen erfolgen.

**8.1 BillingFeeConfig**

Der Plattformadministrator konfiguriert Gebührensätze über ein BillingFeeConfig-Objekt. Jede BillingFeeConfig hat eine Versionskennung und ein Gültigkeitsdatum; die Föderation wendet die zum Zeitpunkt der Job-Ausführung geltende Konfiguration an. Eine neue Konfiguration kann jederzeit erstellt werden; sie tritt zu Beginn des nächsten Abrechnungszeitraums in Kraft.

BillingFeeConfig-Felder:

- input_token_rate_usd_per_1k: in USD berechneter Betrag pro 1.000 Eingabe-Token
- output_token_rate_usd_per_1k: in USD berechneter Betrag pro 1.000 Ausgabe-Token
- platform_fee_pct: der Prozentsatz der Gesamttoken-Kosten für die Plattform (0–100)
- node_revenue_share_pct: der Prozentsatz der Gesamttoken-Kosten für den Knotenbetreiber (0–100, muss zusammen mit platform_fee_pct ≤ 100 ergeben)
- settlement_period_days: wie oft die Abwicklung durchgeführt wird (z. B. 30)
- minimum_payout_usd: minimale angesammelte Einnahmen, bevor ein Knotenbetreiber eine Auszahlung erhält

**8.2 Einreicher-Abrechnung**

Einreicher werden auf Nachzahlungsbasis abgerechnet. Am Ende jedes Abrechnungszeitraums aggregiert die Föderation alle Belege für den Einreicher und belastet die hinterlegte Zahlungsmethode. Die Rechnung enthält eine aufgeschlüsselte Liste der Job-Belege, gruppiert nach Compliance-Zone und Modell.

**8.3 Knotenbetreiber-Abwicklung**

Knotenbetreiber werden am Ende jedes Abrechnungszeitraums über Stripe Connect ausgezahlt, sofern ihre angesammelten Einnahmen den Schwellenwert minimum_payout_usd überschreiten. Betreiber, die den Schwellenwert nicht erreichen, übertragen ihre Einnahmen in den nächsten Zeitraum.

## 9. Sicherheitsmodell

MFOP implementiert ein dreischichtiges Sicherheitsmodell: Transportsicherheit, KI-Sicherheitsrichtlinienvalidierung und Ausführungs-Sandbox-Isolation.

**9.1 Transportsicherheit**

Alle MFOP-API-Endpunkte MÜSSEN über HTTPS mit TLS 1.3 oder höher bereitgestellt werden. Mutual TLS (mTLS) wird EMPFOHLEN für die Knoten-zu-Föderations-Kommunikation in privaten Unternehmens-Mesh-Einsätzen. Die API-Authentifizierung verwendet PAK Keys, die als X-Channel-API-Key HTTP-Header übermittelt werden. PAK Keys sind 256-Bit-Zufallswerte, die in base64url kodiert sind.

**9.2 KI-Sicherheitsrichtlinienvalidierung**

Alle Job-Eingaben und -Ausgaben werden vor der Ausführung und vor der Übermittlung an den Einreicher gegen NeMo Guardrails-Richtlinien validiert. Der grundlegende Richtliniensatz (für alle Compliance-Zonen erforderlich) umfasst:

- Jailbreak-Erkennung und -Sperrung
- Erkennung schädlicher Inhalte (Gewalt, CSAM, Beihilfe zur Selbstverletzung)
- PII-Leckerkennung in Ausgaben
- Prompt-Injection-Erkennung

Zusätzliche Richtlinien sind für bestimmte Compliance-Zonen erforderlich (siehe Anhang B).

Knoten MÜSSEN die in ihrer Fähigkeitswerbung angegebene NeMo Guardrails-Laufzeitversion ausführen. Knoten, die veraltete Guardrails-Versionen ausführen, werden als DEGRADIERT gekennzeichnet und vom Routing für Compliance-Zonen ausgeschlossen, die Guardrails-Funktionen erfordern, die in der installierten Version nicht vorhanden sind.

**9.3 Ausführungs-Sandbox-Isolation**

Jeder Job wird in einer isolierten Sandbox ausgeführt. Knoten MÜSSEN die Sandbox-Isolation mit einem der folgenden Mechanismen implementieren:

- gVisor (runsc) — EMPFOHLEN für Cloud-Einsätze
- Firecracker Micro-VMs — EMPFOHLEN für Bare-Metal-Einsätze
- WASM (Wasmtime) — Zulässig für reine CPU-Inferenz-Workloads

Sandboxes MÜSSEN zwischen Jobs zerstört und neu erstellt werden. Dauerhafter Sandbox-Zustand (z. B. Modellgewichte) KANN über ein schreibgeschütztes Mount zwischen Jobs geteilt werden, aber jobspezifischer Zustand (Kontext, temporäre Dateien) DARF NICHT zwischen Jobs fortbestehen.

**9.4 Prüfungsprotokollierung**

Alle Job-Routing-Entscheidungen, Beleguнterzeichnungen und Abwicklungsereignisse werden in ein append-only-Prüfungsprotokoll geschrieben. Das Prüfungsprotokoll ist kryptographisch mit SHA-256-Hashes verkettet (jeder Eintrag enthält den Hash des vorherigen Eintrags). Das Prüfungsprotokoll darf nicht geändert werden; nur Anhängeoperationen sind zulässig.

## 10. Leitungsprotokoll

MFOP verwendet JSON über HTTPS für die gesamte API-Kommunikation. WebSocket-Verbindungen werden für das Streaming von Job-Ausgaben unterstützt (siehe Abschnitt 10.2).

**10.1 Anfrage- und Antwortformat**

Alle Anfrage- und Antwortkörper sind UTF-8-kodiertes JSON. Anfragen MÜSSEN Content-Type: application/json enthalten. Erfolgreiche Antworten verwenden HTTP 200 oder 201. Fehlerantworten verwenden den Standard-Fehlerumschlag:

{ "error": { "code": "<maschinenlesbarer-code>", "message": "<menschenlesbarer-message>", "details": { ... } } }

Standard-Fehlercodes: UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, QUOTA_EXCEEDED, NO_ELIGIBLE_NODE, COMPLIANCE_VIOLATION, INTERNAL_ERROR.

**10.2 Streaming-Ausgabe**

Knoten, die Streaming-Ausgabe unterstützen, stellen einen WebSocket-Endpunkt unter wss://{node_endpoint}/v1/jobs/{id}/stream bereit. Der Client verbindet sich nach der Job-Übermittlung. Der Knoten streamt Token-Ausgaben als JSON-gerahmte Delta-Nachrichten:

{ "type": "delta", "text": "...", "token_count": N }

Der Stream wird mit einer Abschlussnachricht beendet:

{ "type": "done", "receipt": { ... } }

Der Beleg in der Abschlussnachricht ist der signierte BillingReceipt für den Job.

**10.3 Idempotenz**

Job-Übermittlungsanfragen SOLLTEN einen Idempotency-Key-Header (UUID) enthalten. Wenn eine Anfrage mit demselben Idempotency-Key innerhalb des Idempotenzfensters (24 Stunden) empfangen wird, gibt die Föderation die ursprüngliche Antwort zurück, ohne den Job erneut auszuführen. Dies schützt vor doppelten Übermittlungen, die durch Netzwerk-Wiederholungsversuche verursacht werden.

## Anhang A. REST-API-Referenz

Dieser Anhang listet die MFOP-REST-API-Endpunkte auf. Alle Endpunkte erfordern einen X-Channel-API-Key-Header, sofern nicht anders angegeben. Basispfad: /v1/federation

| Methode + Pfad | Name | Beschreibung |
| --- | --- | --- |
| POST /v1/federation/nodes/register | Knotenregistrierung | Einen neuen Knoten bei der Föderation registrieren. |
| PUT /v1/federation/nodes/{id}/capabilities | Fähigkeitsaktualisierung | Die Fähigkeitswerbung eines Knotens aktualisieren. |
| POST /v1/federation/nodes/{id}/heartbeat | Knoten-Heartbeat | Signalisieren, dass der Knoten aktiv ist und Jobs annimmt. |
| DELETE /v1/federation/nodes/{id} | Knotenabmeldung | Einen Knoten freiwillig abmelden. |
| POST /v1/federation/jobs | Job-Übermittlung | Einen Job zur Ausführung an die Föderation übermitteln. |
| GET /v1/federation/jobs/{id} | Job-Status | Den aktuellen Status und das Ergebnis eines Jobs abrufen. |
| GET /v1/federation/jobs/{id}/receipt | Job-Beleg | Den signierten Abrechnungsbeleg für einen abgeschlossenen Job abrufen. |
| GET /v1/federation/receipts | Einreicher-Belege | Alle Belege für den authentifizierten Einreicher auflisten. |
| GET /v1/federation/nodes/{id}/receipts | Knotenbelege | Alle Belege für vom Knoten ausgeführte Jobs auflisten. |
| POST /v1/federation/nodes/{id}/stripe/onboard | Stripe Connect Onboarding | Gibt die von Stripe gehostete Onboarding-URL für die Bankkonto-Einrichtung zurück. |
| GET /v1/federation/nodes/{id}/earnings | Anbieter-Einnahmen | Token des aktuellen Zeitraums, geschätzte Einnahmen, letzte Auszahlung. |
| GET /v1/federation/submitters/billing | Einreicher-Abrechnungsübersicht | Kosten des aktuellen Zeitraums, nächstes Abrechnungsdatum. |
| PATCH /v1/admin/federation/billing-config | Gebührenmodell aktualisieren | Nur Administrator. Erstellt eine neue BillingFeeConfig-Zeile. Wirksam ab dem nächsten Zeitraum. |

## Anhang B. Compliance-Zone-Richtlinienanforderungen

Jede Compliance-Zone erfordert spezifische NeMo Guardrails-Richtlinienfähigkeiten über den Grundsatz hinaus. Die folgende Tabelle fasst die minimal erforderlichen Schienen pro Zone zusammen.

| Zone | Erforderliche Schienen über den Grundsatz hinaus |
| --- | --- |
| public | Nur Grundsatz. Keine zusätzlichen Schienen erforderlich. |
| enterprise (SOC2) | Datenresidenz-Markierungserkennung. API-Anmeldedaten-Exfiltrations-Erkennung. Zugriffsprotokollierungsdurchsetzung. |
| hipaa | PHI-Mustererkennung: Patientennamen, Geburtsdaten, MRN, ICD-10-Codes, Diagnosebeschreibungen, Krankenversicherungs-IDs. PHI-De-Identifizierungsschiene: PHI vor dem KI-Modellaufruf entfernen oder hashen. Prüfung der minimalnotwendigen Ausgaben. |
| sox | Finanzielle PII-Isolation: Kontonummern, Bankleitzahlen, Steuer-IDs. Preisvorhersagesperrung: zukunftsgerichtete Rendite- oder Preisaussagen. MNPI-Erkennung: Musterabgleich für wesentliche, nicht-öffentliche Informationen. |
| fedramp | CUI-Handhabung: Erkennung und Handhabungsregeln für Controlled Unclassified Information-Markierungen. Exportkontrolle: EAR/ITAR-Sachverhaltserkennung. Durchsetzung von Klassifizierungskennzeichnungen: Ausgaben mit Klassifizierungskennzeichnungen sperren. |

## Danksagungen

Die Autorin möchte dem NVIDIA NeMo-Team für die NeMo Guardrails und NemoClaw OpenShell-Plattformen danken, die die grundlegende Sicherheitsinfrastruktur bereitstellen, auf die in dieser Spezifikation Bezug genommen wird. Das MFOP-Sicherheitsmodell ist so konzipiert, dass es sich mit diesen Plattformen weiterentwickelt, wenn sie reifen.

Das dreischichtige Sicherheitsmodell, die Compliance-Zone-Taxonomie, das Ed25519-Beleguнterzeichnungsschema und die konfigurierbare Abrechnungsarchitektur, die in dieser Spezifikation beschrieben werden, wurden durch einen umfangreichen Design- und Überprüfungsprozess entwickelt und verfeinert, der Anfang 2026 bei Thrive Tech Services LLC durchgeführt wurde.

Diese Spezifikation ist der weltweiten Gemeinschaft der Wissensarbeiter gewidmet — in den Bereichen Recht, Gesundheitswesen, Forschung, Finanzen und Technik — deren Arbeit der Grund dafür ist, dass föderierte KI-Orchestrierung von Bedeutung ist.

Ende der MFOP-Spezifikation Version 1.0 — Entwurf zur Begutachtung
Thrive Tech Services LLC · Ami Hoepner Nuñez · März 2026

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · März 2026*
