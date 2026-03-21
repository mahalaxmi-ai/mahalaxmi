# Protocole de Fédération et d'Orchestration Mahalaxmi

**MFOP v1.0** · Brouillon soumis à l'examen par les pairs

| | |
|---|---|
| Date | Mars 2026 |
| Author | Ami Hoepner Nuñez |
| Organization | ThriveTech Services LLC |
| Location | West Palm Beach, Floride, États-Unis |
| Contact | Ami.nunez@mahalaxmi.ai |
| Draft | https://mahalaxmi.ai/mfop/draft |
| Discussion | https://mahalaxmi.ai/mfop/discuss |

> **Peer Review Open** — This document is published for community feedback.
> Please [open an issue](https://github.com/mahalaxmi-ai/mahalaxmi/issues/new?template=spec-feedback.yml&labels=spec-feedback) to submit corrections, translation notes, or technical comments.

---

## Statut de ce mémo

Ce document est un brouillon prépublication de la spécification du Protocole de Fédération et d'Orchestration Mahalaxmi (MFOP), version 1.0. Il est distribué aux fins d'examen par les pairs et pour solliciter des commentaires. Ce document décrit un protocole d'orchestration IA distribuée fédérée sur des nœuds de calcul hétérogènes, avec un routage tenant compte des zones de conformité, des reçus de facturation signés cryptographiquement, et un règlement économique configurable.

Les commentaires et questions doivent être adressés à l'auteure à l'adresse Ami.nunez@mahalaxmi.ai. Le brouillon en cours et les fils de discussion sont maintenus à l'adresse https://mahalaxmi.ai/mfop/draft. Les fils de discussion sont disponibles à l'adresse https://mahalaxmi.ai/mfop/discuss.

## Avis de droit d'auteur

Copyright © 2026 ThriveTech Services LLC. Tous droits réservés. Permission est accordée de copier, distribuer et utiliser ce document sur tout support sans frais, à condition que l'attribution à l'auteure, le titre du document et cet avis de droit d'auteur soient préservés dans toutes les copies et œuvres dérivées.

## Résumé

Ce document définit le Protocole de Fédération et d'Orchestration Mahalaxmi (MFOP), un protocole de coordination de l'exécution parallèle d'agents IA sur un réseau distribué de nœuds de calcul hétérogènes. MFOP spécifie l'identité et l'enregistrement des nœuds, la publicité des capacités, le routage des tâches tenant compte des zones de conformité, le partitionnement sémantique des entrées, les reçus de facturation signés cryptographiquement, le règlement économique configurable, et un modèle de sécurité en couches utilisant la validation des politiques de sécurité IA et l'isolation des bacs à sable d'exécution.

MFOP est conçu pour fonctionner dans trois configurations de déploiement simultanées : les maillages d'entreprise privés où les nœuds sont détenus et exploités par une seule organisation, les pools cloud gérés exploités par le fournisseur de plateforme, et les places de marché communautaires ouvertes où tout opérateur de nœud peut contribuer du calcul en échange d'un règlement économique. Le protocole est agnostique vis-à-vis du fournisseur de modèle IA sous-jacent et est conçu pour évoluer avec le paysage de la sécurité IA et de la conformité.

## 1. Introduction

La croissance des déploiements de grands modèles de langage (LLM) dans les environnements d'entreprise a créé le besoin d'une couche de coordination capable de couvrir une infrastructure de calcul hétérogène tout en satisfaisant les exigences de conformité, de facturation et de sécurité qui varient selon la juridiction et le secteur d'activité.

MFOP répond à ce besoin en définissant un protocole d'orchestration IA fédérée. Une fédération est constituée d'un ou plusieurs nœuds de calcul, chacun pouvant être exploité par des entités différentes sous des régimes de conformité différents. Un soumissionnaire — un utilisateur, une application ou un système automatisé — présente une tâche à la fédération. La fédération achemine la tâche vers un nœud approprié en fonction des exigences de zone de conformité de la tâche, de la publicité des capacités du nœud, et des conditions économiques en vigueur.

Cette spécification définit le protocole filaire, les formats de données, les mécanismes cryptographiques, et les exigences comportementales pour tous les composants d'une fédération MFOP conforme.

## 2. Terminologie

Les mots-clés « DOIT », « NE DOIT PAS », « REQUIS », « DEVRA », « NE DEVRA PAS », « DEVRAIT », « NE DEVRAIT PAS », « RECOMMANDÉ », « NON RECOMMANDÉ », « PEUT » et « OPTIONNEL » dans ce document doivent être interprétés comme décrit dans BCP 14 [RFC2119] [RFC8174].

**Fédération** — Un regroupement logique d'un ou plusieurs nœuds de calcul conformes à MFOP fonctionnant sous une configuration de gouvernance partagée.

**Nœud** — Une ressource de calcul enregistrée auprès d'une fédération qui accepte, exécute et retourne des charges de travail IA. Un nœud peut être un serveur unique, un cluster ou un pool de calcul cloud.

**Soumissionnaire** — Une entité (utilisateur, application ou système automatisé) qui présente des charges de travail IA à la fédération pour exécution.

**Zone de conformité** — Un contexte de politique nommé qui contraint le routage des tâches, la gestion des données et la validation des sorties. Zones définies : public, enterprise (SOC2), hipaa, sox, fedramp.

**Tâche** — Une unité discrète de charge de travail IA soumise à la fédération pour exécution. Une tâche porte une charge utile, une assertion de zone de conformité et une autorisation de facturation.

**Reçu** — Un enregistrement signé cryptographiquement d'une exécution de tâche complétée, incluant le nombre de tokens, les horodatages, l'identité du nœud et les montants de facturation.

**Règlement économique** — Le processus par lequel les reçus de facturation accumulés sont convertis en transferts financiers entre les soumissionnaires, les opérateurs de nœuds et la plateforme.

**PAK Key (Clé API de plateforme)** — Un identifiant porteur émis par la plateforme qui autorise l'accès aux points de terminaison de l'API de fédération.

**NeMo Guardrails** — Le cadre de sécurité NVIDIA NeMo utilisé par les nœuds MFOP pour la validation des politiques de sécurité IA et le filtrage des sorties.

## 3. Identité et enregistrement des nœuds

Chaque nœud d'une fédération MFOP est identifié par un identifiant de nœud stable et globalement unique (node_id). Le node_id est un UUID 128 bits (version 4) attribué au moment de l'enregistrement et persiste au travers des redémarrages et mises à jour logicielles du nœud.

**3.1 Flux d'enregistrement**

Un nœud initie l'enregistrement en envoyant une NodeRegistrationRequest au point de terminaison d'enregistrement de la fédération (POST /v1/federation/nodes/register). La demande DOIT inclure :

- node_id : un UUID candidat (la fédération PEUT le remplacer)
- operator_id : l'UUID du compte opérateur enregistrant
- display_name : un nom lisible par l'humain pour le nœud (max 64 caractères)
- public_key : une clé publique Ed25519 encodée en base64url, utilisée pour la signature des reçus
- capability_advertisement : un objet CapabilityAdvertisement (voir Section 4)
- compliance_zones : l'ensemble des zones de conformité que le nœud est certifié à traiter
- endpoint_url : l'URL HTTPS à laquelle le nœud accepte les soumissions de tâches

La fédération retourne une NodeRegistrationResponse contenant le node_id attribué, un registration_token pour les appels authentifiés ultérieurs, et la configuration de facturation actuelle de la fédération.

**3.2 Ré-enregistrement et rotation des clés**

Les nœuds DOIVENT se ré-enregistrer lorsque leur paire de clés Ed25519 est renouvelée. Lors de la rotation des clés, le nœud soumet une demande de ré-enregistrement avec les anciennes et nouvelles clés publiques, signée avec l'ancienne clé privée. La fédération vérifie la signature de l'ancienne clé avant d'accepter la nouvelle. Il existe une fenêtre de chevauchement de 24 heures pendant laquelle les reçus signés avec l'une ou l'autre clé sont acceptés.

**3.3 Santé et désenregistrement des nœuds**

Les nœuds DOIVENT envoyer un battement de cœur à POST /v1/federation/nodes/{id}/heartbeat au moins une fois toutes les 60 secondes. Un nœud qui manque trois fenêtres de battement de cœur consécutives est marqué INACTIF et exclu du routage. Les nœuds peuvent se désenregistrer volontairement via DELETE /v1/federation/nodes/{id}.

## 4. Publicité des capacités

La publicité des capacités d'un nœud déclare les modèles IA disponibles sur le nœud, les caractéristiques matérielles pertinentes pour le routage des tâches, et les certifications de conformité détenues par l'opérateur du nœud.

**4.1 Objet CapabilityAdvertisement**

L'objet CapabilityAdvertisement comprend les champs suivants :

- models : un tableau d'objets ModelDescriptor (voir 4.2)
- hardware_class : l'un de { cpu, gpu_consumer, gpu_datacenter, tpu }
- vram_gb : total de la VRAM GPU disponible pour l'inférence, en gigaoctets (0 pour les nœuds CPU)
- max_context_tokens : la fenêtre de contexte maximale que le nœud peut traiter
- max_concurrent_jobs : le nombre maximum de tâches que le nœud exécutera simultanément
- compliance_certifications : un tableau d'identifiants de certification (ex. : « soc2-type2 », « hipaa-baa », « fedramp-moderate »)
- nemo_rails_version : la version du runtime NeMo Guardrails installée sur le nœud

**4.2 ModelDescriptor**

Chaque modèle disponible sur un nœud est décrit par un ModelDescriptor :

- model_id : une chaîne d'identifiant de modèle canonique (ex. : « meta-llama/Meta-Llama-3-70B-Instruct »)
- model_family : l'un de { llama, mistral, gemma, falcon, phi, custom }
- parameter_count_b : nombre approximatif de paramètres en milliards
- quantization : l'un de { fp16, bf16, int8, int4, none }
- context_window_tokens : la fenêtre de contexte maximale pour ce modèle
- supports_tool_use : booléen
- supports_vision : booléen

**4.3 Actualisation des capacités**

Les nœuds DOIVENT mettre à jour leur publicité des capacités via PUT /v1/federation/nodes/{id}/capabilities chaque fois que leurs modèles disponibles ou leur configuration matérielle changent. La fédération propage les publicités de capacités mises à jour vers la couche de routage en moins de 30 secondes.

## 5. Routage des tâches tenant compte des zones de conformité

MFOP achemine chaque tâche vers un nœud qui satisfait les exigences de zone de conformité de la tâche. La satisfaction de la zone de conformité est une contrainte stricte : une tâche NE DOIT PAS être acheminée vers un nœud qui n'est pas certifié pour la zone de conformité de la tâche.

**5.1 Zones de conformité**

MFOP définit cinq zones de conformité, ordonnées de la moins à la plus restrictive :

- public : Aucune exigence de conformité au-delà des rails de sécurité NeMo de base. Convient aux charges de travail IA à usage général.
- enterprise (SOC2) : Requiert la certification SOC 2 Type II. Ajoute la détection de résidence des données, la détection d'exfiltration des identifiants API, et l'application de la journalisation des accès.
- hipaa : Requiert un accord BAA HIPAA. Ajoute la détection de modèles PHI, la dé-identification PHI, et les vérifications de sortie au minimum nécessaire.
- sox : Requiert les contrôles de conformité SOX. Ajoute l'isolation des IIP financières, le blocage des prédictions de prix, et la détection de MNPI.
- fedramp : Requiert l'autorisation FedRAMP. Ajoute la gestion des CUI, la détection des contrôles à l'exportation, et l'application des marquages de classification.

**5.2 Algorithme de routage**

Lorsqu'une tâche est reçue, la couche de routage exécute l'algorithme suivant :

1. Filtrer : Identifier tous les nœuds avec le statut ACTIF qui sont certifiés pour la zone de conformité de la tâche.
2. Filtrer : Supprimer les nœuds dont le max_context_tokens est inférieur au nombre de tokens estimé de la tâche.
3. Filtrer : Supprimer les nœuds dont le max_concurrent_jobs est actuellement épuisé.
4. Scorer : Pour chaque nœud restant, calculer un score de routage : score = w_latency × latency_score + w_cost × cost_score + w_affinity × affinity_score. Pondérations par défaut : w_latency = 0,4, w_cost = 0,4, w_affinity = 0,2.
5. Sélectionner : Acheminer vers le nœud au score le plus élevé. En cas d'égalité, sélectionner uniformément au hasard.

Si aucun nœud ne satisfait tous les filtres, la tâche est mise en file d'attente avec un délai d'expiration configurable (par défaut : 120 secondes). Si aucun nœud ne devient disponible dans le délai imparti, la fédération retourne HTTP 503 avec un en-tête Retry-After.

**5.3 Règles d'affinité**

Les soumissionnaires PEUVENT spécifier des règles d'affinité dans leur soumission de tâche :

- node_affinity : une liste de node_ids préférés (préférence douce)
- anti_affinity : une liste de node_ids à exclure (contrainte stricte)
- geography : une région géographique préférée (code pays ISO 3166-1 alpha-2)

Les règles d'affinité n'affectent que la composante affinity_score ; la certification de zone de conformité et la capacité restent des contraintes strictes.

## 6. Partitionnement sémantique des entrées

Pour les tâches dont l'entrée dépasse le max_context_tokens d'un seul nœud, MFOP fournit un mécanisme de partitionnement sémantique qui divise l'entrée en sous-tâches cohérentes, achemine chaque sous-tâche indépendamment, et agrège les résultats.

**6.1 Stratégies de partitionnement**

MFOP définit trois stratégies de partitionnement :

- sliding_window : Divise l'entrée en fenêtres se chevauchant de taille et de chevauchement configurables. Convient aux tâches où la continuité du contexte aux frontières est importante (ex. : résumé de longs documents).
- semantic_boundary : Divise aux frontières sémantiques détectées (sauts de paragraphe, en-têtes de sections, transitions thématiques). Produit des sous-tâches plus cohérentes au prix de tailles variables.
- task_decomposition : Interprète l'entrée comme une liste de tâches structurée et achemine chaque tâche en tant que sous-tâche indépendante. Requiert que l'entrée soit conforme au schéma TaskList MFOP.

**6.2 Demande de partitionnement**

Un soumissionnaire demande une exécution partitionnée en définissant partition_strategy dans la soumission de tâche. Le moteur de partitionnement de la fédération divise l'entrée, attribue des identifiants de sous-tâches (parent_job_id + numéro de séquence), et achemine chaque sous-tâche indépendamment. Les sous-tâches héritent de la zone de conformité et de l'autorisation de facturation de la tâche parente.

**6.3 Agrégation**

Une fois toutes les sous-tâches terminées, la couche d'agrégation de la fédération assemble les résultats dans l'ordre des numéros de séquence. Pour les partitions sliding_window, l'agrégateur déduplique le contenu dans les régions de chevauchement en utilisant une fusion par plus longue sous-séquence commune. Le résultat assemblé est retourné au soumissionnaire sous la forme d'un JobResult unique avec un tableau de sub_job_receipts.

## 7. Reçus de facturation signés cryptographiquement

Chaque exécution de tâche complétée produit un BillingReceipt signé par le nœud exécutant. Les reçus signés constituent l'enregistrement faisant autorité pour le règlement économique et la résolution des litiges.

**7.1 Structure du reçu**

Un BillingReceipt contient :

- receipt_id : un UUID (version 4) unique à ce reçu
- job_id : l'UUID de la tâche complétée
- node_id : l'UUID du nœud exécutant
- submitter_id : l'UUID du soumissionnaire
- model_id : le modèle utilisé pour l'exécution
- compliance_zone : la zone de conformité sous laquelle la tâche a été exécutée
- input_tokens : le nombre de tokens d'entrée traités
- output_tokens : le nombre de tokens de sortie générés
- wall_time_ms : temps total d'exécution en millisecondes
- completed_at : horodatage RFC 3339 de l'achèvement de la tâche
- fee_schedule_id : l'UUID du BillingFeeConfig en vigueur au moment de l'exécution
- input_token_cost_usd : coût calculé des tokens d'entrée en USD (6 décimales)
- output_token_cost_usd : coût calculé des tokens de sortie en USD (6 décimales)
- platform_fee_usd : les frais de la plateforme pour cette tâche
- node_earnings_usd : les gains de l'opérateur du nœud pour cette tâche
- total_cost_usd : coût total pour le soumissionnaire

**7.2 Schéma de signature**

Les reçus sont signés avec Ed25519. Le nœud signe la sérialisation JSON canonique du reçu (clés triées, sans espaces) avec sa clé privée enregistrée. La signature est encodée en base64url et incluse dans le reçu dans le champ signature.

La fédération vérifie la signature du reçu à la réception en utilisant la clé publique enregistrée du nœud. Les reçus avec des signatures invalides sont rejetés et déclenchent une alerte d'intégrité du nœud.

**7.3 Stockage et récupération des reçus**

La fédération stocke tous les reçus pendant un minimum de 7 ans pour satisfaire les exigences d'audit de conformité. Les soumissionnaires peuvent récupérer leurs reçus via GET /v1/federation/receipts. Les opérateurs de nœuds peuvent récupérer les reçus des tâches qu'ils ont exécutées via GET /v1/federation/nodes/{id}/receipts.

## 8. Règlement économique configurable

MFOP sépare la facturation (l'accumulation des reçus signés) du règlement (le transfert financier des fonds). Le règlement est configurable et peut se produire selon des calendriers différents pour différents types de participants.

**8.1 BillingFeeConfig**

L'administrateur de la plateforme configure les taux de frais via un objet BillingFeeConfig. Chaque BillingFeeConfig possède un identifiant de version et une date d'entrée en vigueur ; la fédération applique la configuration en vigueur au moment de l'exécution de la tâche. Une nouvelle configuration peut être créée à tout moment ; elle entre en vigueur au début de la prochaine période de facturation.

Champs de BillingFeeConfig :

- input_token_rate_usd_per_1k : USD facturés par 1 000 tokens d'entrée
- output_token_rate_usd_per_1k : USD facturés par 1 000 tokens de sortie
- platform_fee_pct : le pourcentage de la plateforme sur le coût total des tokens (0–100)
- node_revenue_share_pct : le pourcentage de l'opérateur du nœud sur le coût total des tokens (0–100, doit totaliser ≤ 100 avec platform_fee_pct)
- settlement_period_days : fréquence à laquelle le règlement est effectué (ex. : 30)
- minimum_payout_usd : gains accumulés minimaux avant qu'un opérateur de nœud reçoive un versement

**8.2 Facturation des soumissionnaires**

Les soumissionnaires sont facturés sur une base de postpaiement. À la fin de chaque période de règlement, la fédération agrège tous les reçus du soumissionnaire et débite le moyen de paiement enregistré. La facture comprend une liste détaillée des reçus de tâches, groupés par zone de conformité et modèle.

**8.3 Règlement des opérateurs de nœuds**

Les opérateurs de nœuds sont payés via Stripe Connect à la fin de chaque période de règlement, à condition que leurs gains accumulés dépassent le seuil minimum_payout_usd. Les opérateurs qui n'atteignent pas le seuil reportent leurs gains à la période suivante.

## 9. Modèle de sécurité

MFOP met en œuvre un modèle de sécurité à trois couches : la sécurité du transport, la validation des politiques de sécurité IA, et l'isolation des bacs à sable d'exécution.

**9.1 Sécurité du transport**

Tous les points de terminaison de l'API MFOP DOIVENT être servis via HTTPS utilisant TLS 1.3 ou supérieur. Le TLS mutuel (mTLS) est RECOMMANDÉ pour la communication nœud-à-fédération dans les déploiements de maillage d'entreprise privé. L'authentification API utilise des PAK Keys transmises dans l'en-tête HTTP X-Channel-API-Key. Les PAK Keys sont des valeurs aléatoires de 256 bits encodées en base64url.

**9.2 Validation des politiques de sécurité IA**

Toutes les entrées et sorties de tâches sont validées par rapport aux politiques NeMo Guardrails avant l'exécution et avant la livraison au soumissionnaire. L'ensemble de politiques de base (requis pour toutes les zones de conformité) comprend :

- Détection et blocage des tentatives de contournement (jailbreak)
- Détection de contenu nuisible (violence, CSAM, facilitation d'automutilation)
- Détection de fuite de données personnelles dans les sorties
- Détection d'injection de prompt

Des politiques supplémentaires sont requises pour des zones de conformité spécifiques (voir Annexe B).

Les nœuds DOIVENT exécuter la version du runtime NeMo Guardrails spécifiée dans leur publicité de capacités. Les nœuds exécutant des versions obsolètes de Guardrails sont marqués comme DÉGRADÉS et exclus du routage pour les zones de conformité qui nécessitent des fonctionnalités de guardrails non présentes dans la version installée.

**9.3 Isolation des bacs à sable d'exécution**

Chaque tâche s'exécute dans un bac à sable isolé. Les nœuds DOIVENT mettre en œuvre l'isolation des bacs à sable en utilisant l'un des mécanismes suivants :

- gVisor (runsc) — RECOMMANDÉ pour les déploiements cloud
- MicroVM Firecracker — RECOMMANDÉ pour les déploiements sur matériel nu
- WASM (Wasmtime) — Autorisé pour les charges de travail d'inférence uniquement sur CPU

Les bacs à sable DOIVENT être détruits et recréés entre les tâches. L'état persistant des bacs à sable (ex. : poids des modèles) PEUT être partagé entre les tâches via un montage en lecture seule, mais l'état propre à une tâche (contexte, fichiers temporaires) NE DOIT PAS persister entre les tâches.

**9.4 Journalisation d'audit**

Toutes les décisions de routage des tâches, les signatures de reçus, et les événements de règlement sont écrits dans un journal d'audit en mode ajout uniquement. Le journal d'audit est enchaîné cryptographiquement à l'aide de hachages SHA-256 (chaque entrée inclut le hachage de l'entrée précédente). Le journal d'audit ne peut pas être modifié ; seules les opérations d'ajout sont autorisées.

## 10. Protocole filaire

MFOP utilise JSON sur HTTPS pour toutes les communications API. Les connexions WebSocket sont supportées pour la diffusion en continu des sorties de tâches (voir Section 10.2).

**10.1 Format des requêtes et des réponses**

Tous les corps de requêtes et de réponses sont en JSON encodé en UTF-8. Les requêtes DOIVENT inclure Content-Type: application/json. Les réponses réussies utilisent HTTP 200 ou 201. Les réponses d'erreur utilisent l'enveloppe d'erreur standard :

{ "error": { "code": "<code-lisible-par-la-machine>", "message": "<message-lisible-par-l'humain>", "details": { ... } } }

Codes d'erreur standard : UNAUTHORIZED, FORBIDDEN, NOT_FOUND, VALIDATION_ERROR, QUOTA_EXCEEDED, NO_ELIGIBLE_NODE, COMPLIANCE_VIOLATION, INTERNAL_ERROR.

**10.2 Sortie en continu**

Les nœuds qui supportent la sortie en continu exposent un point de terminaison WebSocket à wss://{node_endpoint}/v1/jobs/{id}/stream. Le client se connecte après la soumission de la tâche. Le nœud diffuse la sortie de tokens sous forme de messages delta encadrés en JSON :

{ "type": "delta", "text": "...", "token_count": N }

Le flux est terminé par un message de complétion :

{ "type": "done", "receipt": { ... } }

Le reçu dans le message de complétion est le BillingReceipt signé pour la tâche.

**10.3 Idempotence**

Les demandes de soumission de tâches DEVRAIENT inclure un en-tête Idempotency-Key (UUID). Si une demande avec le même Idempotency-Key est reçue dans la fenêtre d'idempotence (24 heures), la fédération retourne la réponse originale sans ré-exécuter la tâche. Cela protège contre les soumissions en double causées par les nouvelles tentatives réseau.

## Annexe A. Référence de l'API REST

Cette annexe liste les points de terminaison de l'API REST MFOP. Tous les points de terminaison requièrent un en-tête X-Channel-API-Key sauf indication contraire. Chemin de base : /v1/federation

| Méthode + Chemin | Nom | Description |
| --- | --- | --- |
| POST /v1/federation/nodes/register | Enregistrement du nœud | Enregistrer un nouveau nœud auprès de la fédération. |
| PUT /v1/federation/nodes/{id}/capabilities | Mise à jour des capacités | Mettre à jour la publicité des capacités d'un nœud. |
| POST /v1/federation/nodes/{id}/heartbeat | Battement de cœur du nœud | Signaler que le nœud est actif et accepte des tâches. |
| DELETE /v1/federation/nodes/{id} | Désenregistrement du nœud | Désenregistrer volontairement un nœud. |
| POST /v1/federation/jobs | Soumission de tâche | Soumettre une tâche à la fédération pour exécution. |
| GET /v1/federation/jobs/{id} | Statut de la tâche | Récupérer le statut actuel et le résultat d'une tâche. |
| GET /v1/federation/jobs/{id}/receipt | Reçu de tâche | Récupérer le reçu de facturation signé pour une tâche complétée. |
| GET /v1/federation/receipts | Reçus du soumissionnaire | Lister tous les reçus pour le soumissionnaire authentifié. |
| GET /v1/federation/nodes/{id}/receipts | Reçus du nœud | Lister tous les reçus pour les tâches exécutées par le nœud. |
| POST /v1/federation/nodes/{id}/stripe/onboard | Intégration Stripe Connect | Retourne l'URL d'intégration hébergée par Stripe pour la configuration du compte bancaire. |
| GET /v1/federation/nodes/{id}/earnings | Gains du fournisseur | Tokens de la période en cours, gains estimés, dernier versement. |
| GET /v1/federation/submitters/billing | Résumé de facturation du soumissionnaire | Coût de la période en cours, prochaine date de facturation. |
| PATCH /v1/admin/federation/billing-config | Mettre à jour le modèle de frais | Administrateur uniquement. Crée une nouvelle ligne BillingFeeConfig. Effective à la prochaine période. |

## Annexe B. Exigences de politique par zone de conformité

Chaque zone de conformité requiert des capacités de politique NeMo Guardrails spécifiques au-delà de la base. Le tableau suivant résume les rails minimaux requis par zone.

| Zone | Rails requis au-delà de la base |
| --- | --- |
| public | Base uniquement. Aucun rail supplémentaire requis. |
| enterprise (SOC2) | Détection des marqueurs de résidence des données. Détection d'exfiltration des identifiants API. Application de la journalisation des accès. |
| hipaa | Détection de modèles PHI : noms de patients, dates de naissance, MRN, codes ICD-10, descriptions de diagnostics, identifiants d'assurance maladie. Rail de dé-identification PHI : supprimer ou hacher les PHI avant l'invocation du modèle IA. Vérification du minimum nécessaire sur les sorties. |
| sox | Isolation des IIP financières : numéros de compte, numéros de routage, identifiants fiscaux. Blocage des prédictions de prix : déclarations de rendement ou de prix prospectives. Détection de MNPI : correspondance de modèles d'informations importantes non publiques. |
| fedramp | Gestion des CUI : règles de détection et de traitement des marqueurs d'informations non classifiées contrôlées. Contrôle des exportations : détection des sujets relevant de l'EAR/ITAR. Application des marquages de classification : bloquer les sorties contenant des marquages de classification. |

## Remerciements

L'auteure souhaite remercier l'équipe NVIDIA NeMo pour les plateformes NeMo Guardrails et NemoClaw OpenShell, qui fournissent l'infrastructure de sécurité fondamentale référencée dans cette spécification. Le modèle de sécurité MFOP est conçu pour évoluer avec ces plateformes à mesure qu'elles mûrissent.

Le modèle de sécurité à trois couches, la taxonomie des zones de conformité, le schéma de signature des reçus Ed25519, et l'architecture de facturation configurable décrits dans cette spécification ont été développés et affinés au cours d'un processus approfondi de conception et d'examen mené chez Thrive Tech Services LLC au début de l'année 2026.

Cette spécification est dédiée à la communauté mondiale des travailleurs du savoir — dans les disciplines juridiques, de la santé, de la recherche, financières et techniques — dont le travail est la raison pour laquelle l'orchestration IA fédérée est importante.

Fin de la spécification MFOP version 1.0 — Brouillon soumis à l'examen par les pairs
Thrive Tech Services LLC · Ami Hoepner Nuñez · Mars 2026

---

*ThriveTech Services LLC · Ami Hoepner Nuñez · Mars 2026*
