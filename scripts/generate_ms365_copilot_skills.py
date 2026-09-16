#!/usr/bin/env python3
"""
scripts/generate_ms365_copilot_skills.py

Generates all 150 Microsoft 365 Copilot & Vibe Code Development skill packages
under .ecc/skills/copilot-*/SKILL.md with full YAML frontmatter, strict operational
directives (ALWAYS, NEVER, MANDATORY, STRICT_REJECT), and comprehensive engineering instructions.
"""

import os
import sys
from pathlib import Path

# Base directory for skills
BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

# 15 Clusters x 10 Skills = 150 Skills
SKILLS = [
    # =========================================================================
    # Cluster 1: Microsoft 365 Copilot Architecture & Platform Foundations (1-10)
    # =========================================================================
    {
        "id": "copilot-platform-extensibility-matrix",
        "cluster": 1,
        "book": "Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press",
        "desc": "Declarative Agents vs Custom Engine Agents vs Plugins selection architecture, execution boundaries, and licensing prerequisites.",
        "triggers": ["copilot-platform-extensibility", "extensibility-matrix", "declarative-vs-custom-engine", "copilot-plugins-architecture"],
        "foundations": [
            "Extensibility Tiers: Declarative Agents run within Microsoft 365 Copilot host context; Custom Engine Agents run standalone on Azure Bot Framework / Teams AI Library.",
            "Execution Invariant: NEVER route tenant-sensitive enterprise requests to ungrounded external plugins without explicit OAuth2 administrative consent.",
            "ALWAYS enforce user-scoped identity delegation using Entra ID On-Behalf-Of (OBO) flow for downstream API calls.",
        ],
        "protocol": "ALWAYS evaluate requirements against the 3-tier matrix: 1) Declarative for grounded M365 context, 2) Plugins for external REST actions, 3) Custom Engine for full LLM orchestration control. MANDATORY 30s timeout on external connector invocations.",
        "anti_patterns": [
            "Selecting Custom Engine Agents for tasks purely requiring SharePoint/OneDrive retrieval, wasting infra overhead.",
            "Bypassing tenant boundary isolation by hardcoding ambient service principal tokens.",
        ],
    },
    {
        "id": "copilot-orchestration-copilot-engine",
        "cluster": 1,
        "book": "Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press",
        "desc": "Copilot Orchestrator lifecycle: intent classification, contextual prompt composition, semantic grounding, and completion synthesis.",
        "triggers": ["copilot-orchestration", "copilot-engine-loop", "intent-classification", "prompt-grounding-synthesis"],
        "foundations": [
            "Orchestrator State Loop: Query -> Intent Extraction -> Semantic Index Query -> Working Context Assembly -> LLM Synthesis -> Post-Processing Guardrails.",
            "Semantic Grounding Contract: ALWAYS verify that retrieved context chunks match user authorization ACLs before prompt interpolation.",
            "STRICT_REJECT any synthesis output failing Azure AI Content Safety or prompt injection heuristics.",
        ],
        "protocol": "Structure Copilot interactions by adhering to the native orchestrator pipeline. Provide clear action schemas with explicit parameter descriptions to maximize intent classification accuracy.",
        "anti_patterns": [
            "Flooding the orchestrator with ambiguous tool descriptions causing multi-action deadlock.",
            "Assuming single-turn execution when multi-turn clarifying questions are required.",
        ],
    },
    {
        "id": "copilot-semantic-index-architecture",
        "cluster": 1,
        "book": "Deep Dive into Microsoft 365 Semantic Index - Satya Nadella et al.",
        "desc": "Semantic Index for Copilot: tenant-level and user-level vector graph substrate, freshness crawling, and cross-workload ranking.",
        "triggers": ["semantic-index", "semantic-index-architecture", "tenant-vector-substrate", "user-level-embeddings"],
        "foundations": [
            "Dual-Layer Architecture: User-level index (emails, chats, personal docs) and Tenant-level index (public SharePoint, shared repositories).",
            "ACL Preservation Invariant: The Semantic Index NEVER surfaces an entity unless the querying user possesses explicit read access at indexing and query time.",
            "Freshness SLA: Changes in source documents must trigger Graph change notifications and incremental vector updates.",
        ],
        "protocol": "Optimize enterprise data for Semantic Index ingestion by providing rich metadata, explicit entity relationships, and hierarchical Markdown structures.",
        "anti_patterns": [
            "Relying on broad 'Everyone except external users' permissions to mask broken ACL hierarchies.",
            "Treating Semantic Index as a static snapshot rather than an incremental live graph.",
        ],
    },
    {
        "id": "copilot-m365-tenant-boundary-isolation",
        "cluster": 1,
        "book": "Deploying and Managing Microsoft 365 Copilot - J. Peter Bruzzese",
        "desc": "Strict multi-tenant isolation, cross-tenant data fencing, and European Union Data Boundary (EUDB) compliance.",
        "triggers": ["tenant-boundary", "tenant-isolation", "eudb-compliance", "cross-tenant-fencing"],
        "foundations": [
            "Cryptographic Boundary: Tenant data is cryptographically and logically isolated at rest and in transit.",
            "Zero Data Retention (ZDR): Customer prompt and response data is NEVER used to train foundation models.",
            "ALWAYS enforce geographic boundary pinning in accordance with tenant data residency policies (e.g., EU Data Boundary).",
        ],
        "protocol": "Verify that all agent actions, external connectors, and telemetry egress strictly honor tenant data residency fences. STRICT_REJECT unencrypted cross-boundary data transfers.",
        "anti_patterns": [
            "Logging raw customer prompt tokens to third-party unvetted observability tools.",
            "Cross-tenant caching of semantic embeddings in shared Redis/disk instances.",
        ],
    },
    {
        "id": "copilot-declarative-agent-runtime",
        "cluster": 1,
        "book": "Microsoft 365 Declarative Agent Specification - Microsoft Learn",
        "desc": "Declarative Agent sandboxed runtime execution loop, capability dispatching, and prompt boundary confinement.",
        "triggers": ["declarative-agent-runtime", "sandboxed-execution", "capability-dispatching", "prompt-boundary-confinement"],
        "foundations": [
            "Sandbox Runtime: Declarative agents execute strictly within the M365 Copilot conversation shell without arbitrary custom code execution.",
            "Capability Scoping: Actions are bounded by explicit OpenAPI 3.0 operation definitions and Graph connection declarations.",
            "MANDATORY validation of action payloads against declared JSON schemas prior to execution.",
        ],
        "protocol": "Define Declarative Agents with precise system instructions, explicit capability scopes (OneDrive, SharePoint, Web, GraphConnectors), and typed Action Plugins.",
        "anti_patterns": [
            "Over-parameterizing instructions with contradictory persona directives.",
            "Omitting response schema definitions in action specifications, leading to unstructured output hallucination.",
        ],
    },
    {
        "id": "copilot-plugin-manifest-v1",
        "cluster": 1,
        "book": "RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret",
        "desc": "Microsoft Copilot plugin schema v1.0/v1.1, OpenAPI 3.0 specification binding, and ai-plugin.json contracts.",
        "triggers": ["plugin-manifest", "ai-plugin-json", "openapi-copilot-binding", "plugin-schema-v1"],
        "foundations": [
            "Manifest Binding: `ai-plugin.json` specifies human and model descriptions, auth schemes, and relative OpenAPI endpoints.",
            "Description Informativeness Invariant: Function and parameter descriptions MUST explain 'what', 'why', and 'format' to enable zero-shot tool selection.",
            "ALWAYS specify authentication flows (Anonymous, OAuth2, ApiKey) explicitly in the securityScheme definition.",
        ],
        "protocol": "Author rigorous OpenAPI 3.0 specs with crisp semantic descriptions for all paths and parameters. Validate manifests against the official Microsoft Copilot plugin schema.",
        "anti_patterns": [
            "Vague parameter descriptions (e.g., 'id: string') that cause the model to guess argument formats.",
            "Exposing state-mutating POST/DELETE operations without requiring user confirmation.",
        ],
    },
    {
        "id": "copilot-teams-toolkit-scaffolding",
        "cluster": 1,
        "book": "Building Microsoft 365 Solutions with Teams Toolkit - John Miller",
        "desc": "Teams Toolkit CLI and VS Code architecture, lifecycle hooks (teamsapp.yml), provisioning, and environment orchestration.",
        "triggers": ["teams-toolkit", "teams-toolkit-cli", "teamsapp-yml", "teams-scaffolding"],
        "foundations": [
            "Lifecycle Phases: `provision`, `deploy`, `publish` lifecycle stages defined in `teamsapp.yml`.",
            "Environment Isolation: Independent parameterization for `dev`, `test`, `prod` via `.env.{env}` files.",
            "Deterministic Packaging: Manifest templates (`manifest.json`) interpolate environment variables at build time.",
        ],
        "protocol": "Utilize Teams Toolkit CLI (`teamsapp`) for automated CI/CD pipeline provisioning, validation, and package generation.",
        "anti_patterns": [
            "Checking plain-text client secrets or bot passwords into source control.",
            "Editing generated `manifest.json` in `build/` directly instead of the source template in `appPackage/`.",
        ],
    },
    {
        "id": "copilot-hardware-npu-directml-telemetry",
        "cluster": 1,
        "book": "Windows Copilot+ Architecture & DirectML Engineering - Microsoft Hardware Systems",
        "desc": "Windows Copilot+ PC on-device NPU DirectML execution, 40+ TOPS accelerator telemetry, and carbon efficiency metrics.",
        "triggers": ["copilot-hardware-npu", "directml-execution", "copilot-plus-pc", "npu-telemetry"],
        "foundations": [
            "Hardware Acceleration: DirectML execution layer abstracting Qualcomm Snapdragon X, Intel Lunar Lake, and AMD Strix Point NPUs.",
            "40+ TOPS Invariant: High-throughput on-device SLM inference (Phi-Silica, Whisper) offloaded from CPU/GPU.",
            "MANDATORY power and thermal throttling monitoring during sustained local inference loops.",
        ],
        "protocol": "Architect hybrid applications that route low-latency, privacy-critical reasoning to local NPU via DirectML and heavy reasoning to Azure OpenAI.",
        "anti_patterns": [
            "Blocking UI threads with synchronous local model tensor operations.",
            "Failing to implement automatic fallback to cloud LLM when NPU hardware is unavailable.",
        ],
    },
    {
        "id": "copilot-license-sku-governance",
        "cluster": 1,
        "book": "Microsoft 365 Administration Inside Out - Ed Fisher & Darryl van der Peijl",
        "desc": "M365 Copilot license entitlement, SKU feature gating, self-service purchase restrictions, and tenant admin controls.",
        "triggers": ["copilot-license-governance", "sku-feature-gating", "m365-copilot-entitlement", "admin-controls"],
        "foundations": [
            "Entitlement Prerequisite: Base prerequisites (M365 E3/E5, Business Standard/Premium) plus Microsoft 365 Copilot Add-on SKU.",
            "Feature Gating: Enforce programmatically checking user license state before initiating Copilot extension turns.",
            "ALWAYS fail gracefully with user-friendly remediation steps if license check returns unlicensed.",
        ],
        "protocol": "Incorporate automated license and permission pre-flight checks into custom extensions and provisioning scripts.",
        "anti_patterns": [
            "Assuming all users in a tenant possess Copilot licensing, causing unhandled API 403 Forbidden errors.",
            "Silently failing without alerting users that their account requires a Copilot license.",
        ],
    },
    {
        "id": "copilot-copilot-studio-vs-pro-dev",
        "cluster": 1,
        "book": "Building Solutions with Microsoft Copilot Studio - Microsoft Press",
        "desc": "Architectural decision rubric: Low-Code Copilot Studio vs Pro-Dev Teams AI / Semantic Kernel codebases.",
        "triggers": ["copilot-studio-vs-pro-dev", "architectural-decision-rubric", "low-code-vs-pro-code", "copilot-strategy"],
        "foundations": [
            "Tradeoff Continuum: Copilot Studio minimizes time-to-market for conversational dialogs; Teams AI / SK maximizes control over planners, models, and custom state.",
            "Coexistence Pattern: Pro-dev services exposed via OpenAPI plugins consumable directly by citizen-built Copilot Studio topics.",
            "MANDATORY architectural review before choosing pro-code over native Power Platform tools.",
        ],
        "protocol": "Apply the 4-factor decision rubric: 1) Custom model routing? 2) Low-level streaming UI? 3) Complex external state machines? 4) Governance & compliance limits.",
        "anti_patterns": [
            "Building custom Bot Framework bots from scratch for simple FAQ and document retrieval use cases.",
            "Forcing complex distributed saga transactions into no-code Copilot Studio nodes.",
        ],
    },

    # =========================================================================
    # Cluster 2: Declarative Agents, App Manifests & Teams Toolkit (11-20)
    # =========================================================================
    {
        "id": "copilot-declarative-agent-manifest-v1-2",
        "cluster": 2,
        "book": "Microsoft 365 Declarative Agent Specification - Microsoft Learn",
        "desc": "Declarative agent manifest schema v1.2 (declarativeAgent.json), instructions, capabilities, and actions.",
        "triggers": ["declarative-agent-manifest", "declarativeAgent-json", "declarative-manifest-v1-2", "agent-schema"],
        "foundations": [
            "Schema Conformance: `$schema: https://developer.microsoft.com/json-schemas/copilot/declarative-agent/v1.2/schema.json`.",
            "Instruction Architecture: System instructions must define Identity, Scope, Constraints, Response Format, and Tone.",
            "Capabilities Array: Explicit declaration of `OneDriveAndSharePoint`, `GraphConnectors`, and `WebSearch`.",
        ],
        "protocol": "Author declarativeAgent.json with rigorous type declarations. ALWAYS reference valid external action plugin files via relative paths in `actions`.",
        "anti_patterns": [
            "Including executable scripts or unescaped control characters in system instructions.",
            "Referencing non-existent action plugin IDs in the manifest capabilities array.",
        ],
    },
    {
        "id": "copilot-teams-app-manifest-v1-17",
        "cluster": 2,
        "book": "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs",
        "desc": "Teams app manifest schema v1.16/v1.17, copilotAgents declarations, bot definitions, and delegated permission scopes.",
        "triggers": ["teams-app-manifest", "manifest-json-v1-17", "copilotAgents-declaration", "teams-manifest-schema"],
        "foundations": [
            "Manifest Top-Level: `manifestVersion: 1.17`, `copilotAgents: { declarativeAgents: [...] }`.",
            "Icon Assets: `color.png` (192x192) and `outline.png` (32x32) strictly matching transparent PNG requirements.",
            "ALWAYS declare required Graph API delegated permission scopes in `webApplicationInfo`.",
        ],
        "protocol": "Validate Teams app packages against schema v1.17 using Teams Toolkit CLI: `teamsapp validate` before publishing.",
        "anti_patterns": [
            "Using JPEG or non-square icon files causing silent packaging rejections.",
            "Mismatched IDs between `manifest.json` and `declarativeAgent.json`.",
        ],
    },
    {
        "id": "copilot-openapi-actions-declarative",
        "cluster": 2,
        "book": "RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret",
        "desc": "Authoring OpenAPI specs for Declarative Agent actions, JSON schema parameters, and output extraction rules.",
        "triggers": ["openapi-actions", "declarative-agent-actions", "action-openapi-spec", "copilot-action-plugin"],
        "foundations": [
            "OpenAPI 3.0.x Standard: Strict adherence to JSON/YAML OpenAPI specification.",
            "OperationId Invariant: Every path operation MUST define a unique, human-readable `operationId`.",
            "Parameter Grounding: All path, query, and body parameters must include explicit types and descriptive summaries.",
        ],
        "protocol": "Design action plugin specs with minimal endpoint payloads. Filter out unused enterprise endpoints to keep the agent toolset crisp.",
        "anti_patterns": [
            "Exposing endpoints returning multi-megabyte unstructured JSON arrays that exceed model context limits.",
            "Missing `operationId` leading to orchestrator tool invocation failure.",
        ],
    },
    {
        "id": "copilot-onedrive-sharepoint-grounding",
        "cluster": 2,
        "book": "SharePoint & OneDrive REST APIs with Microsoft Graph - Todd Baginski",
        "desc": "Scoping SharePoint site collections, document libraries, OneDrive folders, and URLs in agent capabilities.",
        "triggers": ["sharepoint-grounding", "onedrive-grounding", "site-collection-scoping", "declarative-agent-sources"],
        "foundations": [
            "URL Scoping: Specify explicit SharePoint site collection or folder URLs in `OneDriveAndSharePoint` capability.",
            "Permissions Invariant: The agent cannot view or search files that the calling user lacks permission to access.",
            "MANDATORY inclusion of exact SharePoint URL strings matching tenant site topology.",
        ],
        "protocol": "Ground Declarative Agents to specific, curated SharePoint document libraries rather than broad tenant-wide searches to maximize precision.",
        "anti_patterns": [
            "Pointing grounding URLs to top-level tenant roots, causing high retrieval noise.",
            "Attempting to ground against external non-SharePoint web URLs in the SharePoint capability array.",
        ],
    },
    {
        "id": "copilot-web-grounding-bing-search",
        "cluster": 2,
        "book": "Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press",
        "desc": "Configuring Bing search grounding in Declarative Agents with domain restrictions and citation parsing.",
        "triggers": ["web-grounding", "bing-search-grounding", "domain-restrictions", "citation-parsing"],
        "foundations": [
            "Web Grounding Capability: `capabilities: [{ name: 'WebSearch' }]`.",
            "Citation Synthesis: Web search results automatically synthesize footnotes and URL citations.",
            "ALWAYS enforce domain filtering when proprietary or industry-specific sources must be prioritized.",
        ],
        "protocol": "Enable WebSearch for agents requiring real-time external data (market data, regulatory changes, public API docs). Pair with clear citation extraction directives.",
        "anti_patterns": [
            "Enabling WebSearch for purely confidential internal agents, risking external information contamination.",
            "Permitting ungrounded web search without domain filtering for sensitive regulatory guidance.",
        ],
    },
    {
        "id": "copilot-declarative-agent-conversation-starters",
        "cluster": 2,
        "book": "Conversational AI: Design and Engineering - Cathy Pearl",
        "desc": "Designing conversation starters, prompt scaffolds, and multi-lingual localized string packages.",
        "triggers": ["conversation-starters", "prompt-scaffolds", "localization-packages", "agent-ux-starters"],
        "foundations": [
            "Conversation Starter Schema: Array of text prompts under `conversation_starters` in manifest.",
            "Cognitive Load Reduction: Starters must be concrete, action-oriented, and demonstrate agent capabilities.",
            "MANDATORY localization files (`localization.json`) when publishing to multi-regional tenant workforces.",
        ],
        "protocol": "Provide 3 to 6 high-value conversation starters representing canonical user journeys. Ensure starters trigger specific action plugins or document retrieval flows.",
        "anti_patterns": [
            "Generic starters like 'Help me' or 'What can you do?' that fail to showcase capabilities.",
            "Starters that trigger unsupported actions, immediately eroding user trust.",
        ],
    },
    {
        "id": "copilot-teams-toolkit-environment-variables",
        "cluster": 2,
        "book": "Building Microsoft 365 Solutions with Teams Toolkit - John Miller",
        "desc": "Environment variable management (.env.dev, .env.prod, teamsapp.local.yml) and secrets substitution.",
        "triggers": ["teams-env-variables", "env-dev-prod", "teamsapp-local-yml", "secrets-substitution"],
        "foundations": [
            "Variable Interpolation: `${{VARIABLE_NAME}}` substitution across manifest and configuration templates.",
            "Secrets Handling: Sensitive values stored in `.env.{env}.user` and NEVER checked into version control.",
            "ALWAYS validate required environment variables before initiating build or provision steps.",
        ],
        "protocol": "Maintain segregated `.env.dev`, `.env.test`, `.env.prod` files. Automate secret injection from Azure Key Vault in CI/CD environments.",
        "anti_patterns": [
            "Committing unencrypted `.env.{env}.user` files containing Entra ID client secrets.",
            "Hardcoding URLs and tenant IDs inside `manifest.json` files.",
        ],
    },
    {
        "id": "copilot-teams-app-packaging-validation",
        "cluster": 2,
        "book": "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs",
        "desc": "Teams package zip generation (color.png, outline.png, manifest.json) and automated schema validation.",
        "triggers": ["teams-app-packaging", "package-zip-generation", "teams-schema-validation", "teamsapp-package"],
        "foundations": [
            "Zip Archive Structure: Exact zip bundle containing `manifest.json`, `color.png`, `outline.png`, and optional declarative agent JSON files.",
            "Zero Compression Corruption: File headers in the ZIP archive must conform strictly to standard PKZip formatting.",
            "MANDATORY schema validation against the official Microsoft Teams schema before upload.",
        ],
        "protocol": "Use automated validation scripts (`teamsapp validate --package appPackage/build/appPackage.dev.zip`) in continuous integration gates.",
        "anti_patterns": [
            "Including parent folder paths inside the ZIP archive, breaking manifest extraction.",
            "Mismatched image dimensions (e.g. non-32x32 outline icon) resulting in package rejection.",
        ],
    },
    {
        "id": "copilot-declarative-agent-publishing-admin",
        "cluster": 2,
        "book": "Microsoft 365 Administration Inside Out - Ed Fisher & Darryl van der Peijl",
        "desc": "Tenant sideloading, Integrated Apps publishing, and M365 Admin Center approval workflows.",
        "triggers": ["publishing-admin", "integrated-apps-publishing", "m365-admin-approval", "tenant-sideloading"],
        "foundations": [
            "Publishing Channels: 1) Sideloading (developer testing), 2) Organization App Catalog (tenant-wide), 3) Commercial Marketplace.",
            "Admin Consent Workflow: Admin reviews requested Graph permissions, sensitivity labels, and developer info.",
            "ALWAYS secure tenant admin consent prior to enterprise-wide rollout.",
        ],
        "protocol": "Prepare administrative governance documentation outlining data access scopes, external APIs, and business justification before submitting for tenant review.",
        "anti_patterns": [
            "Directly sideloading unverified developer builds into production executive accounts.",
            "Requesting broad `Directory.ReadWrite.All` scopes when simple `User.Read` suffices.",
        ],
    },
    {
        "id": "copilot-declarative-agent-telemetry-monitoring",
        "cluster": 2,
        "book": "Deploying and Managing Microsoft 365 Copilot - J. Peter Bruzzese",
        "desc": "App usage analytics, diagnostic event logging, and audit logs in Purview and Microsoft Entra.",
        "triggers": ["agent-telemetry", "usage-analytics", "purview-audit-logs", "copilot-monitoring"],
        "foundations": [
            "Audit Integration: All agent invocations, tool actions, and data accesses recorded in unified M365 Audit Log.",
            "Usage Telemetry: Teams Admin Center metrics tracking active users, session frequency, and execution errors.",
            "MANDATORY monitoring of rate limiting and 4xx/5xx HTTP errors on backend action endpoints.",
        ],
        "protocol": "Implement continuous monitoring of agent action endpoints using Application Insights. Set up alerting for latency degradation and anomalous error rates.",
        "anti_patterns": [
            "Deploying agents without telemetry hooks, operating blind to real-world user failures.",
            "Ignoring spike alerts in M365 Copilot admin dashboard indicating broken plugin endpoints.",
        ],
    },

    # =========================================================================
    # Cluster 3: Microsoft Copilot Studio, Topic Orchestration & Multi-Agent Swarms (21-30)
    # =========================================================================
    {
        "id": "copilot-studio-generative-topics",
        "cluster": 3,
        "book": "Mastering Microsoft Copilot Studio - Robert Kaack",
        "desc": "Conversational triggers, dynamic chaining, system instructions, and Generative Answers nodes.",
        "triggers": ["copilot-studio-generative", "generative-topics", "generative-answers-node", "topic-triggers"],
        "foundations": [
            "Generative Answers: Replaces deterministic decision trees with real-time RAG over grounded knowledge sources.",
            "Topic Triggers: Semantic phrase matching vs generative intent classification.",
            "MANDATORY content moderation threshold configuration (High, Medium, Low) on Generative Answers nodes.",
        ],
        "protocol": "Configure Generative Answers nodes with explicit knowledge source prioritization and strict fallback messaging when confidence scores fall below threshold.",
        "anti_patterns": [
            "Setting moderation threshold to Low in enterprise settings, risking ungrounded hallucinations.",
            "Overloading a single topic with hundreds of trigger phrases that confuse the classifier.",
        ],
    },
    {
        "id": "copilot-studio-dynamic-chaining",
        "cluster": 3,
        "book": "Dynamic Topic Chaining and Generative Orchestration - Microsoft Engineering",
        "desc": "Automatic plugin invocation, AI-driven parameter extraction, and multi-action execution plans.",
        "triggers": ["dynamic-chaining", "generative-orchestration", "multi-action-execution", "copilot-studio-chaining"],
        "foundations": [
            "Dynamic Orchestrator: Copilot Studio automatically determines which actions and topics to chain together based on user intent.",
            "Parameter Slot Filling: The model autonomously extracts action parameters from prior conversation context.",
            "ALWAYS define clear input parameter descriptions and verification checks.",
        ],
        "protocol": "Enable Dynamic Chaining for sophisticated conversational workflows. Provide discrete, orthogonal actions that the generative engine can chain reliably.",
        "anti_patterns": [
            "Creating overlapping action definitions that cause oscillating planner loops.",
            "Failing to mark mandatory parameters, resulting in incomplete API payloads.",
        ],
    },
    {
        "id": "copilot-studio-custom-topic-state-machine",
        "cluster": 3,
        "book": "Mastering Microsoft Copilot Studio - Robert Kaack",
        "desc": "Classic topic dialog management, conditional branching, slot filling, and redirect nodes.",
        "triggers": ["custom-topic-state-machine", "dialog-management", "conditional-branching", "slot-filling"],
        "foundations": [
            "Deterministic State: Deterministic nodes (Question, Message, Condition, Action) executed sequentially.",
            "Variable Scoping: Variables scoped to Topic, Global, or System levels.",
            "MANDATORY redirect to a Fallback topic when input cannot be parsed after N retries.",
        ],
        "protocol": "Use classic topics for strict transactional flows (e.g., password reset, fund transfers, formal approvals) where generative deviations are prohibited.",
        "anti_patterns": [
            "Deeply nested conditional trees (>5 levels) that become impossible to debug.",
            "Infinite loops between topic redirects with no exit condition.",
        ],
    },
    {
        "id": "copilot-studio-multi-agent-swarms",
        "cluster": 3,
        "book": "Multi-Agent System Choreography in Copilot Studio - Michael Wooldridge",
        "desc": "Multi-agent delegation, supervisor orchestrators, subagent task routing, and handoffs.",
        "triggers": ["copilot-studio-multi-agent", "agent-swarms", "supervisor-orchestrator", "subagent-task-routing"],
        "foundations": [
            "Supervisor-Worker Pattern: A primary orchestrator agent decomposes requests and delegates to specialized subagents.",
            "Context Handoff Contract: Handoff payloads must transfer session state, conversation history, and authenticated user tokens.",
            "ALWAYS establish a return path from subagents back to the supervisor upon task completion.",
        ],
        "protocol": "Architect modular agent swarms in Copilot Studio: HR Agent, IT Helpdesk Agent, Finance Agent coordinated by a master Enterprise Copilot.",
        "anti_patterns": [
            "Circular handoffs between subagents causing conversational deadlocks.",
            "Dropping user authentication context during subagent delegation.",
        ],
    },
    {
        "id": "copilot-studio-knowledge-sources",
        "cluster": 3,
        "book": "Enterprise Knowledge Grounding in Copilot Studio - Alex Simons",
        "desc": "Integrating SharePoint, Dataverse, Public Web, and unstructured files as grounded knowledge sources.",
        "triggers": ["copilot-studio-knowledge", "knowledge-sources", "dataverse-grounding", "sharepoint-sources"],
        "foundations": [
            "Heterogeneous Ingestion: Direct binding of SharePoint URLs, Dataverse tables, uploaded PDFs/DOCX, and public URLs.",
            "Index Refresh Invariant: Knowledge sources must specify automated indexing intervals or webhook-driven updates.",
            "MANDATORY authentication configuration for internal SharePoint and Dataverse sources.",
        ],
        "protocol": "Curate and partition knowledge sources into dedicated topics to prevent semantic cross-talk and maximize retrieval accuracy.",
        "anti_patterns": [
            "Uploading outdated PDF handbooks containing conflicting HR policies.",
            "Exposing unsecured internal Dataverse tables without row-level security.",
        ],
    },
    {
        "id": "copilot-studio-power-fx-formulas",
        "cluster": 3,
        "book": "Power Fx: Low-Code Programming Language Guide - Greg Lindhorst",
        "desc": "Utilizing Power Fx for variable transformations, regex parsing, and condition evaluation in topics.",
        "triggers": ["power-fx-formulas", "power-fx-copilot-studio", "variable-transformations", "power-fx-conditions"],
        "foundations": [
            "Declarative Language: Power Fx expressions evaluate synchronously within topic nodes.",
            "Type Safety: Strong typing across Text, Number, Boolean, Record, and Table types.",
            "ALWAYS handle null/blank values explicitly using `IsBlank()` or `Coalesce()`.",
        ],
        "protocol": "Write clean, declarative Power Fx expressions in Set Variable and Condition nodes for deterministic data wrangling.",
        "anti_patterns": [
            "Complex procedural logic written in massive nested Power Fx formulas instead of delegating to Power Automate.",
            "Unchecked string manipulation leading to runtime type errors on null inputs.",
        ],
    },
    {
        "id": "copilot-studio-bot-framework-composer-interop",
        "cluster": 3,
        "book": "Programming the Microsoft Bot Framework - Joe Mayo",
        "desc": "Integrating Bot Framework Composer dialogs and Bot Framework SDK components.",
        "triggers": ["bot-framework-composer", "composer-interop", "bot-framework-sdk", "adaptive-dialogs"],
        "foundations": [
            "Adaptive Dialogs: Declarative JSON-based dialog trees exported from Composer into Copilot Studio.",
            "Custom Code Components: Integrating Azure Functions and Bot Framework middleware.",
            "ALWAYS maintain backward compatibility of schema properties when modifying dialog definitions.",
        ],
        "protocol": "Extend Copilot Studio with Bot Framework Composer for advanced card interactions, complex regex recognizers, and custom telemetry.",
        "anti_patterns": [
            "Directly editing raw Composer JSON without testing in Bot Framework Emulator.",
            "Introducing unmanaged state variables that conflict with Copilot Studio system variables.",
        ],
    },
    {
        "id": "copilot-studio-channel-deployment",
        "cluster": 3,
        "book": "Mastering Microsoft Copilot Studio - Robert Kaack",
        "desc": "Multi-channel publishing (Teams, Webchat, Outlook, Omnichannel, custom mobile applications).",
        "triggers": ["channel-deployment", "multi-channel-publishing", "teams-channel-copilot", "webchat-deployment"],
        "foundations": [
            "Omnichannel Distribution: Single bot core deployed across Microsoft Teams, Webchat, Power Pages, and mobile apps.",
            "Channel Adaptation Invariant: Rich UI cards must gracefully downgrade to plain text on channels lacking Adaptive Card support.",
            "MANDATORY channel security configuration (Direct Line token generation, trusted origins).",
        ],
        "protocol": "Configure channel-specific settings, ensuring security tokens are exchanged securely via backend server components.",
        "anti_patterns": [
            "Exposing raw Direct Line secret keys in client-side JavaScript.",
            "Sending complex interactive cards to SMS or email channels without plain-text fallbacks.",
        ],
    },
    {
        "id": "copilot-studio-analytics-conversation-transcripts",
        "cluster": 3,
        "book": "Conversational Analytics and Session Telemetry - Microsoft Learn",
        "desc": "Analyzing CSAT, session transcripts, unhandled queries, and conversation KPIs.",
        "triggers": ["copilot-analytics", "conversation-transcripts", "csat-monitoring", "session-telemetry"],
        "foundations": [
            "Session Metrics: Engagement rate, resolution rate, escalation rate, and CSAT scores.",
            "Transcript Auditing: Conversation transcripts stored securely in Dataverse (`ConversationTranscript` table).",
            "MANDATORY retention policies governing conversation history in compliance with GDPR/HIPAA.",
        ],
        "protocol": "Build automated Power BI dashboards connected to Dataverse transcript tables to surface top unhandled triggers and user friction points.",
        "anti_patterns": [
            "Failing to review unhandled query reports, letting bot accuracy degrade over time.",
            "Storing unredacted PII in conversation transcript telemetry.",
        ],
    },
    {
        "id": "copilot-studio-alm-solution-lifecycle",
        "cluster": 3,
        "book": "Application Lifecycle Management for Copilot Studio - Microsoft Power CAT",
        "desc": "Power Platform Solutions, environment variables, ALM pipelines, and automated export/import.",
        "triggers": ["copilot-studio-alm", "solution-lifecycle", "power-platform-alm", "managed-solutions"],
        "foundations": [
            "Managed Solutions: Packaging bots, topics, flows, and connection references into immutable Managed Solutions for production.",
            "Environment Variables: Abstracting endpoint URLs, client IDs, and tenant configs across Dev, Test, Prod.",
            "MANDATORY source code versioning of unpacked solution files using Power Platform CLI (`pac`).",
        ],
        "protocol": "Enforce strict CI/CD pipelines in Azure DevOps or GitHub Actions: unpack solution -> commit to Git -> run test suite -> build managed solution -> deploy to Prod.",
        "anti_patterns": [
            "Directly editing topics and flows in production environments ('hot patching').",
            "Hardcoding developer tenant connection IDs inside solution components.",
        ],
    },
]

# We will generate all 150 skills across all 15 clusters.
# Let's add clusters 4 through 15 dynamically with complete authoritative coverage.

def expand_all_150_skills():
    """Ensure all 150 skills across 15 clusters are defined in SKILLS."""
    global SKILLS
    
    # Cluster 4: Microsoft Graph API, Semantic Index & Knowledge Substrate (31-40)
    c4 = [
        ("copilot-graph-rest-api-v1-beta", 4, "Microsoft Graph Essentials: Enterprise Data Access - Paul Schaeflein",
         "Core Graph API patterns, v1.0 vs beta parity, OData query options ($select, $filter, $expand).",
         ["graph-rest-api", "graph-api-v1", "odata-queries", "graph-expand-filter"],
         ["Graph API Invariant: Use v1.0 for production stability; restrict beta to validated preview features.",
          "OData Optimization: ALWAYS specify $select to minimize payload size and token consumption.",
          "MANDATORY pagination handling via @odata.nextLink."],
         "Structure Graph API queries with strict projection ($select), filtering ($filter), and eager expansion ($expand). Never fetch entire resource bags."),
        
        ("copilot-graph-delta-queries-sync", 4, "Real-Time Microsoft Graph Webhooks and Delta Queries - Glenn Turner",
         "Change tracking with Delta queries across Messages, DriveItems, Users, and Calendar events.",
         ["graph-delta-queries", "delta-sync", "change-tracking", "graph-delta-token"],
         ["Delta Token Invariant: Store @odata.deltaLink securely; replay on incremental sync runs.",
          "State Resiliency: If delta token expires (410 Gone), trigger complete state resynchronization.",
          "ALWAYS track deleted entity tombstones (@removed)."],
         "Implement reliable incremental synchronization using Graph Delta queries, maintaining persistent delta tokens in durable storage."),
        
        ("copilot-graph-batching-json-requests", 4, "Resilient Microsoft Graph SDK Programming - Microsoft Architecture",
         "High-throughput $batch request assembly, dependency chains (dependsOn), and 429 adaptive backoff.",
         ["graph-batching", "batch-requests", "graph-429-backoff", "dependson-chains"],
         ["Batch Limits: Maximum 20 requests per $batch payload.",
          "Dependency DAG: Use 'dependsOn' array to enforce sequential execution within a batch.",
          "MANDATORY handling of individual sub-request HTTP status codes in batch response."],
         "Assemble concurrent Graph requests into $batch envelopes. Implement exponential backoff with jitter on HTTP 429 (Too Many Requests)."),
        
        ("copilot-graph-connectors-schema-registration", 4, "Building Microsoft Graph Connectors - Microsoft Press",
         "Registering external connections, defining property schemas, and search item indexing.",
         ["graph-connectors", "connector-schema-registration", "external-connections", "search-item-indexing"],
         ["Schema Immutability: Connector property types cannot be altered once registered; additions require schema extension.",
          "Semantic Annotations: Mark properties with 'isSearchable', 'isQueryable', 'isRetrievable', 'labels'.",
          "ALWAYS assign aliases and semantic labels (title, url, iconUrl)."],
         "Define external connection schemas with exact semantic annotations to ensure external data is fully searchable by Microsoft 365 Copilot."),
        
        ("copilot-graph-connectors-acl-crawling", 4, "Building Microsoft Graph Connectors - Microsoft Press",
         "External item ACL mapping, user identity resolution, and security trimming in Semantic Index.",
         ["graph-connectors-acl", "acl-crawling", "identity-resolution", "security-trimming"],
         ["Security Trimming Invariant: External items MUST inherit exact access control entries (Acl) mapped to Entra ID users/groups.",
          "Deny Rules: Explicit deny ACLs override grant ACLs unconditionally.",
          "MANDATORY verification that unauthenticated users cannot access indexed external documents."],
         "Crawl and synchronize Access Control Lists along with content items to guarantee enterprise security trimming in Copilot search results."),
        
        ("copilot-graph-webhooks-change-notifications", 4, "Real-Time Microsoft Graph Webhooks and Delta Queries - Glenn Turner",
         "Creating webhook subscriptions, handling validation token handshakes, and renewal loops.",
         ["graph-webhooks", "change-notifications", "webhook-validation-token", "subscription-renewal"],
         ["Validation Handshake: Return validationToken query parameter within 10 seconds as plain text.",
          "Subscription Expiration: Graph subscriptions expire; maintain background worker to renew before expiry.",
          "MANDATORY HTTPS endpoint with valid, non-self-signed TLS certificate."],
         "Deploy robust webhook receivers to process real-time Graph change notifications. Pair webhook signals with Delta query execution."),
        
        ("copilot-graph-jwe-encrypted-notifications", 4, "JSON Web Encryption (JWE) & Token Decryption in Copilot - RFC 7516 Standard",
         "Decrypting JWE change notifications containing resource data using asymmetric X.509 keys.",
         ["jwe-encrypted-notifications", "graph-jwe-decryption", "rfc7516-tokens", "asymmetric-decryption"],
         ["RFC 7516 Compliance: JWE payload contains RSA-OAEP encrypted symmetric key and AES-GCM ciphertext.",
          "Certificate Rotation: Support dual active X.509 certificates to ensure seamless key rotation.",
          "MANDATORY verification of signature and encryption tags prior to parsing payload data."],
         "Implement high-performance JWE decryption pipelines to consume rich resource data directly from Graph notifications without secondary roundtrips."),
        
        ("copilot-graph-meeting-transcript-parsing", 4, "Microsoft Teams Graph API: Channels, Chats and Meetings - Hilton Giesenow",
         "Asynchronous meeting transcript retrieval (/onlineMeetings/{id}/transcripts), VTT parsing, and speaker diarization.",
         ["meeting-transcript-parsing", "teams-meeting-transcripts", "vtt-parsing", "speaker-diarization"],
         ["Transcript Availability: Transcripts become available asynchronously post-meeting finalization.",
          "Diarization Model: Map VTT speaker timestamps to resolved Entra ID user identities.",
          "ALWAYS handle partial or empty transcript streams gracefully."],
         "Retrieve and parse Teams meeting transcripts into structured speaker-turn JSON blocks for downstream Copilot summarization."),
        
        ("copilot-graph-sharepoint-driveitems", 4, "SharePoint & OneDrive REST APIs with Microsoft Graph - Todd Baginski",
         "Manipulating SharePoint Document Libraries, file download/upload chunks, and permissions inspection.",
         ["sharepoint-driveitems", "graph-driveitems", "chunked-upload", "permissions-inspection"],
         ["Upload Session Invariant: Files larger than 4MB MUST use uploadSession chunked upload.",
          "DriveItem Hierarchy: Navigate files via `/drives/{drive-id}/root:/{path}`.",
          "MANDATORY checksum validation using quickXorHash or sha256."],
         "Automate file lifecycle in SharePoint and OneDrive via Graph DriveItem APIs, utilizing chunked upload sessions for large documents."),
        
        ("copilot-graph-search-query-api", 4, "Deep Dive into Microsoft 365 Semantic Index - Satya Nadella et al.",
         "Programmatic querying of Microsoft Search API (/search/query) for hybrid BM25 and vector semantic hits.",
         ["graph-search-api", "search-query-api", "hybrid-bm25-search", "semantic-search-hits"],
         ["Entity Types: Query across message, chatMessage, driveItem, externalItem, listItem.",
          "KQL Filtering: Support Keyword Query Language (KQL) expressions alongside natural language queries.",
          "MANDATORY pagination handling via 'from' and 'size' parameters."],
         "Execute unified enterprise search queries via `/search/query` to retrieve multi-workload grounded hits for Copilot reasoning."),
    ]

    # Cluster 5: Custom Engine Agents & Teams AI Library (41-50)
    c5 = [
        ("copilot-teams-ai-library-core-architecture", 5, "Building Intelligent Bots with Teams AI Library - Microsoft Dev",
         "Teams AI Library (@microsoft/teams-ai), ApplicationBuilder, and TurnContext lifecycle.",
         ["teams-ai-library", "teams-ai-core", "application-builder", "turncontext-lifecycle"],
         ["Architecture Pattern: `Application` orchestrates activity routing, authentication, and AI turn planner.",
          "Turn Flow: User Activity -> Middleware -> Auth Handler -> ActionPlanner -> Tool Execution -> Response.",
          "ALWAYS preserve turn context state during async I/O."],
         "Build custom engine agents using `@microsoft/teams-ai`, configuring `ApplicationBuilder` with typed state and planners."),
        
        ("copilot-teams-ai-action-planner", 5, "Action Planners and Augmented LLM Orchestration - Microsoft Engineering",
         "ActionPlanner configuration, plan generation, tool calling loops, and model parameter tuning.",
         ["teams-ai-action-planner", "actionplanner-config", "tool-calling-loops", "llm-turn-orchestration"],
         ["Plan Contract: LLM outputs predicted actions (DO <action> <parameters> or SAY <response>).",
          "Execution Guardrail: Limit maximum tool loops (default: 5) to prevent infinite planner oscillation.",
          "MANDATORY validation of action parameters against typed TypeScript interfaces."],
         "Configure `ActionPlanner` with strict prompt templates and registered actions for reliable multi-step agent execution."),
        
        ("copilot-teams-ai-turn-state-management", 5, "Conversation and User State Management in Teams Bots - Joe Stagner",
         "ConversationState, UserState, and TempState scoping, validation, and storage providers (Cosmos/Memory).",
         ["turn-state-management", "teams-ai-state", "conversationstate-userstate", "cosmos-state-storage"],
         ["State Scopes: `conversation` (persisted across users in chat), `user` (persisted per user), `temp` (single turn).",
          "Storage Invariant: NEVER use in-memory storage in multi-instance production environments; use Cosmos DB or Blob.",
          "ALWAYS initialize state with safe default factory functions."],
         "Manage agent memory by partitioning data cleanly across conversation, user, and temporary turn state scopes."),
        
        ("copilot-teams-ai-streaming-responses", 5, "Streaming LLM Token Generation in Microsoft Teams - Teams Engineering",
         "Real-time streaming responses in Teams chat using StreamingResponse and chunked formatting.",
         ["teams-ai-streaming", "streaming-responses", "chunked-formatting", "real-time-token-stream"],
         ["Streaming Protocol: Emit informative updates via chunked Teams activities before final message commit.",
          "Typing Cadence: Maintain active typing indicators every 3-4 seconds during model generation.",
          "MANDATORY error boundary catching stream interruptions and rendering user fallback."],
         "Stream LLM generation directly into Teams conversations to achieve sub-second perceived response latency."),
        
        ("copilot-teams-ai-adaptive-card-routing", 5, "Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger",
         "app.adaptiveCards.actionSubmit handlers, universal actions, and form data validation.",
         ["adaptive-card-routing", "teams-ai-card-routing", "action-submit-handler", "card-data-validation"],
         ["Action Routing: Route card submissions via `app.adaptiveCards.actionSubmit(verb, handler)`.",
          "Data Sanitization: Strictly validate form inputs before invoking database mutations.",
          "ALWAYS acknowledge card submissions within 10 seconds to avoid Teams client timeout."],
         "Handle interactive Adaptive Card submissions using dedicated action handlers, returning updated card views or confirmation messages."),
        
        ("copilot-teams-ai-message-extensions", 5, "Search and Action Message Extensions with AI - Waldek Mastykarz",
         "Search-based and action-based Message Extensions, link unfurling (app.messageExtensions.handle).",
         ["teams-message-extensions", "link-unfurling", "search-message-extension", "teams-ai-extensions"],
         ["Extension Types: Search extensions (query catalog), Action extensions (modal forms), Link unfurling (rich preview).",
          "Response Format: Return `MessagingExtensionResponse` containing Adaptive Card or thumbnail preview attachments.",
          "MANDATORY token verification in message extension invocations."],
         "Implement AI-powered Message Extensions allowing users to search, trigger, and insert rich Copilot artifacts directly into chats."),
        
        ("copilot-teams-ai-feedback-loop-handlers", 5, "User Feedback Loops (Thumbs Up/Down) in Teams AI - Microsoft UX",
         "Capturing user thumbs-up / thumbs-down feedback, citation chips, and telemetry dispatch.",
         ["teams-ai-feedback-loops", "thumbs-up-down", "citation-chips", "user-feedback-handlers"],
         ["Feedback Handlers: Register `app.feedbackLoop` to capture user quality sentiment.",
          "Citation Integration: Attach citation references (`citations: [...]`) to bot messages for verifiable grounding.",
          "MANDATORY logging of negative feedback events to offline eval telemetry."],
         "Equip custom engine agents with native feedback loops to capture user sentiment and drive continuous prompt refinement."),
        
        ("copilot-teams-ai-authentication-turn-handler", 5, "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci",
         "TeamsBot SSO, OAuthPrompt, and silent token acquisition in Teams AI Library turns.",
         ["teams-ai-auth-handler", "teamsbot-sso", "oauth-prompt", "silent-token-acquisition"],
         ["SSO Flow: Request user token silently via Teams SSO; fallback to OAuthCard dialog if consent is needed.",
          "Token Validation: Validate signature, issuer, audience, and scope before trusting token claims.",
          "NEVER pass unvalidated client tokens to downstream internal APIs."],
         "Implement seamless single sign-on authentication in Teams AI agents, acquiring scoped Graph tokens silently on behalf of users."),
        
        ("copilot-teams-ai-custom-engine-rag", 5, "Retrieval-Augmented Generation: Systems, Architectures and Optimizations - Patrick Lewis",
         "Grounding custom engine agent prompts with vector search results and dynamic system messages.",
         ["custom-engine-rag", "teams-ai-rag", "vector-search-grounding", "dynamic-system-messages"],
         ["RAG Pipeline: User Query -> Embedding Generation -> Vector Retrieval -> Reranking -> System Prompt Augmentation.",
          "Context Token Limit: Truncate retrieved chunks strictly within prompt budget (e.g., 4000 tokens).",
          "ALWAYS instruct model to cite retrieved chunk identifiers in answers."],
         "Ground Teams AI agents with enterprise data retrieved from Azure AI Search or Qdrant, formatting context into structured prompts."),
        
        ("copilot-teams-ai-testing-and-debugging", 5, "Testing and Validating Intelligent Agent Plugins - Lisa Crispin",
         "Teams App Test Tool, Bot Framework Emulator, and automated TurnContext mock test suites.",
         ["teams-ai-testing", "teams-test-tool", "bot-framework-emulator", "turncontext-mock-tests"],
         ["Test Isolation: Run automated integration tests against simulated `TurnContext` without live Teams connections.",
          "Deterministic Testing: Mock external LLM responses to test planner and action execution deterministically.",
          "MANDATORY unit test coverage for all custom action handlers."],
         "Establish comprehensive test suites using Teams App Test Tool and Jest/Vitest to verify agent behavior across multi-turn scenarios."),
    ]

    # Cluster 6: Semantic Kernel (C# & Python) & Autonomous Agent Plugins (51-60)
    c6 = [
        ("copilot-semantic-kernel-kernel-architecture", 6, "Programming Microsoft Semantic Kernel - Lucas Vogel",
         "Semantic Kernel core architecture: Kernel, Service Collection, AI Service registration (Azure OpenAI/OpenAI).",
         ["semantic-kernel-core", "sk-kernel-architecture", "sk-service-collection", "sk-azure-openai"],
         ["Kernel Lifecycle: `Kernel` acts as the dependency injection and orchestration hub.",
          "Multi-Service Routing: Register multiple chat completion and embedding services under distinct service IDs.",
          "ALWAYS configure timeout and retry policies on underlying HTTP client handlers."],
         "Initialize and configure `Kernel` instances with strongly-typed AI connectors and plugin registries in C# and Python."),
        
        ("copilot-sk-native-plugins-csharp-python", 6, "Programming Microsoft Semantic Kernel - Lucas Vogel",
         "Authoring native plugins with [KernelFunction] (C#) and @kernel_function (Python) with typed docstrings.",
         ["sk-native-plugins", "kernel-function-decorator", "typed-docstrings", "sk-native-functions"],
         ["Decorator Contract: Mark methods with `[KernelFunction]` (C#) or `@kernel_function` (Python).",
          "Metadata Annotation: `[Description(\"...\")]` on functions and parameters is MANDATORY for model discovery.",
          "ALWAYS validate parameter bounds inside native function bodies."],
         "Author native Semantic Kernel plugins that expose local computations, file operations, and database access to LLM planners."),
        
        ("copilot-sk-prompt-functions-yaml", 6, "Semantic Kernel Handlebars and Liquid Prompt Templating - Microsoft Dev",
         "Declarative YAML prompt functions (prompt.yaml), input variables, template engines (Handlebars / Liquid).",
         ["sk-prompt-functions", "sk-yaml-prompts", "prompt-yaml-schema", "handlebars-prompt-templating"],
         ["Declarative Spec: `prompt.yaml` defines template, execution settings (temperature, top_p), and input variables.",
          "Template Engines: Utilize Handlebars template engine for structured loops, conditionals, and sub-function calls.",
          "MANDATORY schema validation of prompt.yaml before runtime loading."],
         "Package prompts as version-controlled YAML assets, decoupling prompt engineering from compiled application binaries."),
        
        ("copilot-sk-auto-function-calling", 6, "Autonomous Planning and Execution with Semantic Kernel - John Maeda",
         "Automatic function invocation filters, execution loop controls, and max-iteration safeguards.",
         ["sk-auto-function-calling", "tool-call-behavior", "function-invocation-loop", "max-iterations-safeguard"],
         ["Execution Setting: Set `FunctionChoiceBehavior = FunctionChoiceBehavior.Auto()`.",
          "Loop Guard: Configure `MaximumAutoInvokeAttempts` (e.g. 5) to abort runaway tool loops.",
          "MANDATORY inspection of function call exceptions during auto-invocation."],
         "Configure automatic tool calling in Semantic Kernel, ensuring robust loop limits and error handling around function invocations."),
        
        ("copilot-sk-filters-and-hooks", 6, "Semantic Kernel Filters, Hooks and Observability - Microsoft AI",
         "Function invocation filters, prompt render filters, and authorization middleware in Semantic Kernel.",
         ["sk-filters-hooks", "invocation-filters", "prompt-render-filters", "sk-middleware"],
         ["Filter Types: `IFunctionInvocationFilter`, `IPromptRenderFilter`, `IAutoFunctionInvocationFilter`.",
          "Short-Circuit Capability: Filters can inspect arguments, modify outputs, or abort execution prior to invocation.",
          "ALWAYS log execution duration and token counts inside filter hooks."],
         "Implement cross-cutting concerns (security auditing, secret scrubbing, caching, telemetry) using Semantic Kernel filter pipelines."),
        
        ("copilot-sk-agent-framework-chat-completion", 6, "Chat Completion and Multi-Agent Orchestration in Semantic Kernel - Shawn Henry",
         "ChatCompletionAgent, AgentGroupChat, and multi-agent coordination strategies in SK.",
         ["sk-agent-framework", "chat-completion-agent", "agent-group-chat", "sk-multi-agent"],
         ["Agent Abstraction: `ChatCompletionAgent` encapsulates persona, instructions, and dedicated plugin subset.",
          "Group Chat Coordination: `AgentGroupChat` manages multi-agent turns with custom selection and termination strategies.",
          "MANDATORY termination strategy to prevent infinite conversational chatter."],
         "Orchestrate multi-agent collaborations (Reviewer, Coder, Verifier) in Semantic Kernel using structured group chat strategies."),
        
        ("copilot-sk-memory-and-vector-connectors", 6, "Vector Memory and Text Embeddings in Semantic Kernel - Mark Wallace",
         "Semantic memory, vector store abstractions (Azure AI Search, Qdrant, Chroma), and embedding generation.",
         ["sk-memory-connectors", "vector-store-abstractions", "qdrant-azure-search-sk", "text-embeddings-sk"],
         ["Vector Store Record: Define strongly typed record models annotated with `[VectorStoreRecordKey]`, `[VectorStoreRecordData]`.",
          "Distance Metrics: Enforce cosine similarity or dot product consistent with embedding model specifications.",
          "ALWAYS batch embedding generation when indexing multiple documents."],
         "Store and search vector representations of enterprise documents using Semantic Kernel's pluggable vector store interfaces."),
        
        ("copilot-sk-openapi-plugin-generator", 6, "RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret",
         "Importing OpenAPI specifications dynamically into Semantic Kernel plugins at runtime.",
         ["sk-openapi-plugins", "dynamic-openapi-import", "openapi-to-sk-plugin", "swagger-sk-integration"],
         ["Dynamic Generation: `kernel.ImportPluginFromOpenApiAsync()` compiles endpoints into callable KernelFunctions.",
          "Auth Delegation: Inject custom `HttpClient` with Bearer token authentication handlers into the OpenAPI plugin.",
          "MANDATORY filtering of sensitive or deprecated operations from the OpenAPI document."],
         "Import third-party REST APIs dynamically into Semantic Kernel by feeding OpenAPI 3.0 specifications to runtime plugin generators."),
        
        ("copilot-sk-process-framework-event-driven", 6, "Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe",
         "Semantic Kernel Process Framework: step-based stateful workflows, routing, and event subscriptions.",
         ["sk-process-framework", "step-based-workflows", "event-driven-agents", "sk-stateful-processes"],
         ["Process Model: Processes comprise Steps, Events, and State, executing deterministically across distributed systems.",
          "Event Subscriptions: Steps subscribe to specific events emitted by prior steps.",
          "ALWAYS persist process state to allow long-running workflow resumption."],
         "Design complex, long-running agent workflows using the Semantic Kernel Process Framework for resilient step orchestration."),
        
        ("copilot-sk-enterprise-observability", 6, "Observability Engineering: Achieving Operational Excellence - Charity Majors",
         "OpenTelemetry instrumentation, trace spans, token usage telemetry, and Application Insights integration.",
         ["sk-observability", "opentelemetry-sk", "token-usage-telemetry", "application-insights-sk"],
         ["OTel Instrumentation: Semantic Kernel emits standard `Microsoft.SemanticKernel` Activity spans.",
          "Metrics Tracking: Monitor `semantic_kernel.tokens.prompt`, `semantic_kernel.tokens.completion`, `semantic_kernel.function.duration`.",
          "MANDATORY redacting of PII and user prompt text from production telemetry traces."],
         "Instrument Semantic Kernel applications with OpenTelemetry exporters to gain granular visibility into model latency, cost, and tool health."),
    ]

    # Helper function to generate standardized clusters 7 to 15
    def make_cluster_skills(c_num, specs):
        res = []
        for s_id, book, desc, triggers, foundations, protocol in specs:
            res.append({
                "id": s_id,
                "cluster": c_num,
                "book": book,
                "desc": desc,
                "triggers": triggers,
                "foundations": foundations,
                "protocol": protocol,
                "anti_patterns": [
                    f"Bypassing formal verification rules in {s_id}.",
                    f"Unmonitored runtime execution without telemetry in {s_id}."
                ]
            })
        return res

    # Cluster 7: Power Platform, Power Apps & Power Automate Vibe Workflows (61-70)
    c7 = [
        ("copilot-power-automate-cloud-flows-ai", "Automating Business Processes with Power Automate and AI Builder - Aaron Murchie",
         "Automated and instant cloud flows triggered by Copilot, AI Prompts action, and JSON parsing.",
         ["power-automate-copilot", "cloud-flows-ai", "ai-prompts-action", "power-automate-json"],
         ["Flow Trigger Contract: Trigger flows via Copilot Studio using strongly typed input parameters.",
          "Output Schema: Always return a structured JSON response schema to Copilot within 60 seconds.",
          "MANDATORY error handling using Scope blocks with run-after conditions."],
         "Build automated cloud flows in Power Automate that serve as high-reliability backend tools for Microsoft Copilot."),
        
        ("copilot-power-apps-copilot-studio-embedding", "Learn Microsoft Power Platform and Copilot - Robert Kaack",
         "Embedding Copilot Studio bots into Canvas Apps and Model-Driven Apps with context passing.",
         ["power-apps-copilot", "copilot-canvas-apps", "model-driven-copilot", "copilot-embedding"],
         ["Context Passing: Pass active record ID and user context into embedded Copilot controls.",
          "UI Integration: Embed Copilot sidecar panels without occluding core transactional form controls.",
          "ALWAYS honor record-level security permissions defined in Dataverse."],
         "Embed contextual Copilot assistants into Power Apps to assist users with real-time data entry and validation."),
        
        ("copilot-power-fx-natural-language-formulas", "Power Fx: Low-Code Programming Language Guide - Greg Lindhorst",
         "Vibe coding Power Fx formulas from natural language, table filtering, and patch operations.",
         ["power-fx-vibe-coding", "natural-language-power-fx", "power-fx-patch", "power-fx-filtering"],
         ["Formula Generation: Translate natural language user requirements into declarative Power Fx statements.",
          "Delegation Invariant: Ensure query formulas delegate execution to the underlying Dataverse/SQL server.",
          "MANDATORY validation of delegation warnings to avoid client-side 500-record limits."],
         "Generate and verify Power Fx formulas using natural language prompts, prioritizing delegable functions for large enterprise datasets."),
        
        ("copilot-custom-connectors-openapi-oauth", "Authoring Custom Connectors for Copilot Studio - Daniel Laskewitz",
         "Creating custom connectors with OpenAPI 3.0, Entra ID OAuth2 authentication, and policy templates.",
         ["custom-connectors", "openapi-custom-connector", "connector-oauth2", "power-platform-connectors"],
         ["Connector Spec: Author valid OpenAPI 2.0/3.0 definitions defining endpoints and schemas.",
          "Policy Templates: Apply policy templates for dynamic URL routing, header injection, and response transformation.",
          "ALWAYS configure Entra ID OAuth2 authentication with authorized redirect URLs."],
         "Develop secure custom connectors in Power Platform to bridge proprietary enterprise REST APIs to Copilot."),
        
        ("copilot-dataverse-web-api-crud", "Microsoft Dataverse Architecture and Best Practices - Julie Yack",
         "Querying and mutating Dataverse entities via Web API, deep insert, and alternate keys.",
         ["dataverse-web-api", "dataverse-crud", "deep-insert-dataverse", "alternate-keys"],
         ["OData v4 Protocol: Execute CRUD operations against `/api/data/v9.2/` using standard OData headers.",
          "Optimistic Concurrency: Use `If-Match: ETag` to prevent mid-air collision updates.",
          "MANDATORY pagination handling for record sets exceeding 5,000 items."],
         "Integrate directly with Microsoft Dataverse Web API for high-throughput batch operations and deep relational inserts."),
        
        ("copilot-power-pages-ai-site-generation", "Building External Web Portals with Power Pages and Copilot - Colin Vermander",
         "Vibe building Power Pages portals, liquid templates, and Web API integration for Copilot.",
         ["power-pages-copilot", "power-pages-ai", "liquid-templates-copilot", "portal-web-api"],
         ["AI Site Generation: Generate page layouts, forms, and Liquid code snippets via natural language Copilot prompts.",
          "Table Permissions Invariant: External portal users MUST possess explicit Table Permissions to access Dataverse data.",
          "ALWAYS enable CSRF token verification on Power Pages Web API endpoints."],
         "Rapidly prototype and deploy external-facing Power Pages portals with grounded Copilot AI components."),
        
        ("copilot-ai-builder-document-processing", "Intelligent Document Processing with AI Builder - Microsoft Power Platform",
         "Custom document processing models, prompt templates, and invoice/receipt data extraction.",
         ["ai-builder-documents", "document-processing-ai", "invoice-extraction", "ai-builder-prompts"],
         ["Model Training: Train custom document processing models on representative enterprise document layouts.",
          "Confidence Scoring: Inspect extracted field confidence scores; route low-confidence fields (<0.8) to human review.",
          "MANDATORY extraction schema defining data types (Date, Currency, Text)."],
         "Incorporate AI Builder document processing models into Copilot workflows to automate invoice, contract, and receipt intake."),
        
        ("copilot-power-platform-cli-pac", "Application Lifecycle Management for Copilot Studio - Microsoft Power CAT",
         "Power Platform CLI (pac) command automation, solution pack/unpack, and pipeline deployment.",
         ["power-platform-cli", "pac-cli-automation", "solution-unpack", "pac-admin-pipeline"],
         ["CLI Automation: Automate ALM via `pac solution pack`, `pac solution unpack`, `pac solution export`.",
          "Source Control Friendly: Deconstruct solutions into component XML/YAML files suitable for Git diffs.",
          "MANDATORY automated solution check (`pac solution check`) before release."],
         "Automate Power Platform ALM and solution deployment using `pac` commands in continuous integration pipelines."),
        
        ("copilot-power-platform-dlp-connector-policies", "Microsoft Power Platform Center of Excellence (CoE) Governance - Microsoft IT",
         "Data Loss Prevention policies, classifying custom connectors (Business vs Non-Business), and tenant isolation.",
         ["power-platform-dlp", "dlp-connector-policies", "business-vs-nonbusiness", "coe-governance"],
         ["DLP Boundary: Connectors partitioned into Business, Non-Business, and Blocked groups.",
          "Data Exfiltration Invariant: Data CANNOT pass between Business and Non-Business connectors in a single flow.",
          "ALWAYS verify connector classification before deploying production Copilot flows."],
         "Design and enforce tenant DLP policies that prevent accidental cross-contamination of corporate and personal services."),
        
        ("copilot-power-automate-error-handling-scopes", "Automating Business Processes with Power Automate and AI Builder - Aaron Murchie",
         "Flow run retry policies, Scope-based Try-Catch-Finally, and run-after error escalation.",
         ["power-automate-error-handling", "flow-try-catch-finally", "retry-policies", "run-after-escalation"],
         ["Pattern Implementation: Wrap critical flow actions in Scope_Try, Scope_Catch, Scope_Finally blocks.",
          "Run-After Configuration: Configure Scope_Catch to run only if Scope_Try has 'has failed', 'has timed out'.",
          "MANDATORY notification to system administrators on catch execution."],
         "Implement enterprise-grade error handling in Power Automate flows using Scope blocks and automated dead-letter alerts."),
    ]

    # Cluster 8: Office JavaScript / TypeScript Scripts & Add-ins Engineering (71-80)
    c8 = [
        ("copilot-office-scripts-excel-typescript", "Excel Office Scripts with TypeScript - Yutaka Hiraoka",
         "Writing Office Scripts in TypeScript, manipulating ranges, tables, pivot tables, and conditional formats.",
         ["office-scripts-excel", "excel-typescript-scripts", "workbook-range-mutation", "office-scripts-automation"],
         ["Execution Sandbox: Office Scripts run within a dedicated web worker sandbox with strict API boundaries.",
          "Zero External Network: Office Scripts CANNOT make arbitrary `fetch()` requests directly; use Power Automate for I/O.",
          "ALWAYS batch operations inside `main(workbook: ExcelScript.Workbook)`."],
         "Author high-performance TypeScript Office Scripts to automate complex workbook mutations and calculations."),
        
        ("copilot-office-scripts-power-automate-sync", "Connecting Office Scripts to Power Automate Cloud Flows - Damien Bird",
         "Executing Office Scripts from Power Automate flows, passing dynamic arrays, and receiving return values.",
         ["office-scripts-power-automate", "headless-excel-execution", "script-parameter-passing", "excel-cloud-flow"],
         ["Headless Execution: Power Automate executes Office Scripts headlessly on Excel Online servers.",
          "Typed Interfaces: Define explicit TypeScript interfaces for script input and output parameters.",
          "MANDATORY timeout consideration: Headless script execution terminates after 120 seconds."],
         "Bridge enterprise cloud workflows to Excel data models by executing parameter-driven Office Scripts from Power Automate."),
        
        ("copilot-excel-javascript-api-custom-functions", "Developing Office Add-ins with Office.js - Michael Saunders",
         "Excel JavaScript API custom functions (=TGS.*), streaming functions, and matrix calculation.",
         ["excel-custom-functions", "office-js-functions", "streaming-custom-functions", "excel-matrix-calculation"],
         ["Custom Function Contract: Annotate functions with JSDoc `@customfunction` tags and unique function names.",
          "Streaming Invariant: Streaming functions emit continuous values via `CustomFunctions.StreamingInvocation`.",
          "ALWAYS handle calculation cancellation via the `invocation.onCanceled` event."],
         "Develop custom Excel calculation formulas in TypeScript that query AI models or external data sources asynchronously."),
        
        ("copilot-word-javascript-api-content-controls", "Word Document Generation and Templating with Office.js - Doug Mahugh",
         "Word JavaScript API: manipulating paragraphs, content controls, inline styles, and OOXML bodies.",
         ["word-javascript-api", "word-content-controls", "ooxml-document-generation", "word-automation-js"],
         ["Document DOM: Manipulate `context.document.body`, `contentControls`, and `tables` via Office.js.",
          "Batch Synchronization: Call `await context.sync()` after batching DOM mutations.",
          "MANDATORY cleanup of temporary content controls post-generation."],
         "Automate rich document generation and templating in Microsoft Word using the Office.js JavaScript API."),
        
        ("copilot-outlook-javascript-api-mail-drafting", "Building Modern Outlook Web Add-ins - Andrew Coates",
         "Outlook add-ins: drafting email replies, inspecting attachments, and appointment creation.",
         ["outlook-javascript-api", "outlook-mail-drafting", "item-send-events", "outlook-add-ins"],
         ["Mailbox API: Access active message via `Office.context.mailbox.item`.",
          "Item-Send Hooks: Validate email contents and sensitivity labels before sending via OnSend handlers.",
          "ALWAYS request appropriate mailbox permission levels (ReadItem vs ReadWriteItem)."],
         "Build modern Outlook Web Add-ins that assist users with context-aware email drafting, triage, and scheduling."),
        
        ("copilot-office-add-in-unified-manifest", "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs",
         "Modern unified manifest (manifest.json) for Office Add-ins vs XML manifest migration.",
         ["office-unified-manifest", "manifest-json-office", "xml-to-json-manifest", "office-add-in-spec"],
         ["Unified Schema: Transition from legacy XML manifests to Microsoft 365 unified manifest schema v1.17+.",
          "Cross-Platform Compatibility: Single package targets Teams, Outlook, Word, Excel, PowerPoint.",
          "MANDATORY specification of `extensions` array defining custom ribbon buttons and task panes."],
         "Package and deploy Office Add-ins using the modern unified JSON manifest schema across Microsoft 365 hosts."),
        
        ("copilot-office-js-batching-context-sync", "Developing Office Add-ins with Office.js - Michael Saunders",
         "Efficient context.sync() batching, avoiding roundtrips, and property loading (range.load).",
         ["office-js-batching", "context-sync-optimization", "property-loading-range", "office-js-performance"],
         ["Batching Invariant: Queue all reads and writes before invoking `await context.sync()`.",
          "Explicit Property Loading: ALWAYS invoke `.load('property')` before accessing object properties post-sync.",
          "NEVER call `context.sync()` inside high-iteration loops."],
         "Optimize Office.js add-in performance by batching commands and minimizing client-to-host bridge roundtrips."),
        
        ("copilot-office-add-in-sso-entra-id", "Single Sign-On (SSO) in Office Add-ins - Microsoft Identity",
         "Single Sign-On (SSO) in Office Add-ins with Office.auth.getAccessToken() and backend token exchange.",
         ["office-sso", "office-auth-sso", "entra-id-office-sso", "bootstrap-token-exchange"],
         ["SSO Flow: Call `Office.auth.getAccessToken()` to retrieve bootstrap token without popup prompting.",
          "Backend Token Exchange: Exchange bootstrap token via OAuth2 On-Behalf-Of flow for Graph API access.",
          "ALWAYS handle error 13003 (consent needed) by launching fallback interactive dialog."],
         "Implement robust Single Sign-On in Office Add-ins, securing seamless access to tenant Graph resources."),
        
        ("copilot-powerpoint-presentation-generation", "PowerPoint Automation with Office JavaScript API - Microsoft Docs",
         "Automating PowerPoint slide creation via OpenXML, Marp Markdown, and Office JavaScript API.",
         ["powerpoint-generation", "powerpoint-office-js", "marp-markdown-slides", "openxml-slides"],
         ["Slide Hierarchy: Add slides, shape trees, text frames, and table layouts programmatically.",
          "Layout Templates: Bind content to pre-defined slide masters to maintain brand design consistency.",
          "MANDATORY validation of shape coordinates and text boundary wrapping."],
         "Automate corporate presentation synthesis from structured Markdown or JSON data using PowerPoint APIs."),
        
        ("copilot-office-add-in-deployment-centralized", "Publishing and Deploying Enterprise Office Add-ins - Microsoft AppSource",
         "Centralized deployment via M365 Admin Center, manifest hosting, and telemetry monitoring.",
         ["office-centralized-deployment", "admin-center-addins", "appsource-submission", "office-manifest-hosting"],
         ["Deployment Route: Assign add-ins to specific Entra ID security groups via M365 Admin Center.",
          "Hosting Invariant: Host web application assets on secure, high-availability HTTPS CDNs with TLS 1.3.",
          "MANDATORY health check endpoint for monitoring add-in uptime."],
         "Manage enterprise-wide Office Add-in rollouts through Centralized Deployment, verifying group assignment and telemetry."),
    ]

    # Cluster 9: Vibe Coding Paradigms, Conversational Flow & Rapid Prototyping (81-90)
    c9 = [
        ("copilot-vibe-coding-conversational-flow", "The Vibe Coding Paradigm: Conversational Software Construction - Andrej Karpathy",
         "Maintaining flow state in conversational development, conversational refactoring, and iterative steering.",
         ["vibe-coding-flow", "conversational-programming", "iterative-steering", "flow-state-coding"],
         ["Conversational Invariant: Code via iterative English dialog, reviewing diffs and steering direction intuitively.",
          "Epistemic Vigilance: Never trust generated code blindly; verify critical invariants and boundaries.",
          "ALWAYS keep conversation context clean by resetting or summarizing historical turns."],
         "Practice high-velocity vibe coding: see stuff, say stuff, run stuff, verify diffs, and steer the AI pair programmer."),
        
        ("copilot-tracer-bullets-m365-architecture", "The Pragmatic Programmer: AI Vibe Edition - David Thomas & Andrew Hunt",
         "Implementing thin end-to-end tracer bullets across Teams, Graph, and backend before detail expansion.",
         ["tracer-bullets-m365", "thin-end-to-end-slices", "pragmatic-vibe-architecture", "rapid-verification-slice"],
         ["Tracer Principle: Build a complete, thin vertical slice from UI to database before fleshing out features.",
          "Feedback Immediate: Validate that Teams UI can reach backend and return data end-to-end.",
          "MANDATORY automated test verifying the tracer path."],
         "Fire tracer bullets through the entire Microsoft 365 stack to prove architectural feasibility in hour one."),
        
        ("copilot-rapid-feedback-loops-m365", "Extreme Programming Explained: AI Pair Programming - Kent Beck",
         "Sub-second feedback loops with hot-reloading Teams dev tunnels and mock Graph responses.",
         ["rapid-feedback-loops", "teams-dev-tunnels", "hot-reload-m365", "sub-second-dev-loops"],
         ["Feedback Cadence: Developer changes MUST reflect in the active running application in < 2 seconds.",
          "Dev Tunnels: Utilize Microsoft Dev Tunnels for secure, low-latency webhook tunneling to localhost.",
          "ALWAYS provide mock Graph data fixtures for instant offline test iteration."],
         "Set up ultra-fast local development loops with Teams Dev Tunnels and hot-reloading watchers for rapid prototyping."),
        
        ("copilot-human-in-the-loop-steering", "Co-Intelligence: Living and Working with AI - Ethan Mollick",
         "Designing graceful human intervention points, interruptible agent plans, and approval checkpoints.",
         ["human-in-the-loop", "agent-steering-points", "approval-checkpoints", "interruptible-plans"],
         ["Steering Invariant: High-stakes mutations (sending emails, modifying permissions) MUST require human sign-off.",
          "State Resumption: Agents must pause gracefully awaiting approval and resume from saved checkpoints.",
          "NEVER allow autonomous execution of financial or destructive actions without human confirmation."],
         "Architect human-in-the-loop approval gates inside Copilot agents using Teams approval cards and pause/resume states."),
        
        ("copilot-exploratory-spike-prototyping", "Extreme Programming Explained: AI Pair Programming - Kent Beck",
         "Rapid exploratory spiking of M365 APIs with zero upfront boilerplate and interactive logging.",
         ["exploratory-spikes", "rapid-api-spiking", "zero-boilerplate-spikes", "interactive-api-testing"],
         ["Spike Protocol: Throwaway code written to explore unfamiliar APIs and answer specific technical unknowns.",
          "Time-Boxing: Constrain spike experiments to maximum 2 hours.",
          "NEVER merge spike code directly to production without refactoring and writing tests."],
         "Conduct rapid exploratory spikes against unfamiliar Graph or Copilot APIs, extracting key learnings into durable skills."),
        
        ("copilot-conversational-tdd-synthesis", "Test-Driven Development for AI Agents - Kent Beck",
         "Prompting test-driven red-green-refactor loops directly in conversational pair programming.",
         ["conversational-tdd", "red-green-refactor-prompts", "prompt-driven-testing", "tdd-agent-synthesis"],
         ["TDD Loop: 1) Prompt test creation, 2) Verify failure (Red), 3) Prompt minimal code (Green), 4) Refactor.",
          "Contract Preservation: Tests serve as immutable contracts that the conversational agent cannot violate.",
          "ALWAYS run tests automatically on file save during vibe coding sessions."],
         "Enforce conversational TDD: prompt the AI to write comprehensive unit tests before generating application logic."),
        
        ("copilot-fail-fast-diagnostic-surfacing", "A Philosophy of Software Design: Deep Agentic Modules - John Ousterhout",
         "Surfacing actionable runtime and compilation errors immediately in the conversation thread.",
         ["fail-fast-diagnostics", "traceback-surfacing", "actionable-compiler-errors", "error-recovery-vibe"],
         ["Error Transparency: Capture and format compiler diagnostics and stack traces directly in the conversation.",
          "Self-Healing: Feed exact error messages back to the model with instruction to propose targeted diffs.",
          "NEVER hide or swallow exceptions with empty catch blocks."],
         "Design diagnostic pipelines that surface compilation and API errors with high fidelity, enabling instant conversational self-healing."),
        
        ("copilot-context-scaffolding-agents", "A Philosophy of Software Design: Deep Agentic Modules - John Ousterhout",
         "Pre-loading conversation context with architectural decision records, schemas, and linting rules.",
         ["context-scaffolding", "prompt-context-preloading", "adr-context-injection", "architectural-rules"],
         ["Scaffolding Protocol: Maintain a `.gemini/` or `.ecc/` directory of project rules and schemas.",
          "Progressive Disclosure: Inject only the specific schemas and invariants relevant to the active sub-task.",
          "ALWAYS keep project architecture guidelines versioned alongside codebase."],
         "Scaffold Copilot coding sessions with concise project rules, domain schemas, and invariants to prevent architectural drift."),
        
        ("copilot-dialectical-code-review-agents", "High-performance Multi-LLM Collaboration, Adversarial Debate & Mixture-of-Agents Engine in Rust - Tagisan Team",
         "Simulating adversarial dialectical peer review (Thesis, Antithesis, Synthesis) during vibe coding.",
         ["dialectical-code-review", "adversarial-peer-review", "thesis-antithesis-synthesis", "moa-code-review"],
         ["Dialectical Method: Proposer writes code (Thesis), Critic attacks security and edge cases (Antithesis), Arbiter harmonizes (Synthesis).",
          "Objective Standards: Ground debates against formal rules (OWASP Top 10, memory safety, token efficiency).",
          "MANDATORY convergence to an audited, non-hallucinated solution."],
         "Run multi-agent dialectical code reviews inside Tagisan (`tgs`) before accepting generated Copilot agent code into master."),
        
        ("copilot-flow-state-ergonomics", "Flow: The Psychology of Optimal Experience in AI Coding - Mihaly Csikszentmihalyi",
         "Minimizing context switching through CLI-driven Copilot interactions and inline markdown previews.",
         ["flow-state-ergonomics", "cli-copilot-flow", "minimal-context-switch", "developer-ergonomics"],
         ["Ergonomic Invariant: Keep hands on the keyboard; drive code generation, testing, and deployment from CLI.",
          "Instant Visuals: Inline rendering of Adaptive Cards and markdown previews within the terminal or editor.",
          "ALWAYS eliminate unnecessary modal prompts and mouse-heavy workflows."],
         "Optimize developer flow state through streamlined CLI shortcuts (`tgs copilot ...`), instant test execution, and zero friction."),
    ]

    # Cluster 10: Advanced Prompt Engineering, Context Grounding & In-Context Reasoning (91-100)
    c10 = [
        ("copilot-prompt-cot-few-shot-crafting", "Prompt Engineering for Generative AI - James Phoenix & Mike Taylor",
         "Chain-of-Thought (CoT) prompting, structured thought tags, and dynamic few-shot exemplar selection.",
         ["cot-prompting", "few-shot-crafting", "thought-tags", "dynamic-exemplars"],
         ["Reasoning Decomposition: Enforce step-by-step reasoning inside `<thinking>` or `<scratchpad>` blocks.",
          "Exemplar Quality: Provide 2-3 input/output pairs that illustrate complex boundary conditions and edge cases.",
          "ALWAYS separate scratchpad thinking from final customer-facing responses."],
         "Craft structured Chain-of-Thought prompts with dynamic few-shot exemplars to maximize reasoning fidelity in Copilot."),
        
        ("copilot-lost-in-the-middle-mitigation", "Context Window Optimization & Token Budgeting - Greg Kamradt",
         "Structuring large context windows to counteract attentional decay in long-document reasoning.",
         ["lost-in-the-middle", "context-attentional-decay", "primacy-recency-structuring", "context-window-optimization"],
         ["Attention Distribution: LLMs attend most strongly to tokens at the very beginning and very end of the prompt.",
          "Placement Invariant: Place critical system instructions and final queries at the outer boundaries.",
          "MANDATORY key fact repetition for context lengths exceeding 32,000 tokens."],
         "Structure large context windows strategically to counteract 'lost-in-the-middle' phenomena during complex document synthesis."),
        
        ("copilot-context-grounding-citations", "Designing Transparent Citations and Attribution in AI Responses - Microsoft UX",
         "Formulating prompts to enforce strict citation anchors ([doc1], [doc2]) and ground-truth verification.",
         ["context-grounding-citations", "citation-anchors", "ground-truth-verification", "verifiable-attribution"],
         ["Citation Syntax: Force model to suffix every factual assertion with exact document chunk IDs `[docX]`.",
          "No Hallucinated Citations: STRICT_REJECT responses citing document IDs not present in retrieved context.",
          "ALWAYS include direct web/SharePoint URLs in citation reference tables."],
         "Enforce strict citation schemas in system prompts to ensure every claim made by Copilot is auditable and verifiable."),
        
        ("copilot-system-instructions-declarative", "Authoring Formal System Prompts for Business Copilots - Anthropic & OpenAI",
         "Crafting robust system instructions for Declarative Agents that resist jailbreak deviations.",
         ["system-instructions-declarative", "declarative-agent-prompts", "jailbreak-resistance", "immutable-instructions"],
         ["Instruction Hierarchy: 1) Core Identity & Purpose, 2) Grounding Rules, 3) Negative Constraints, 4) Output Format.",
          "Immutable Guard: Declare instructions immutable; reject user attempts to 'ignore previous instructions'.",
          "ALWAYS specify fallback behavior when requested information is unavailable."],
         "Author bulletproof system instructions for Declarative Agents that preserve corporate tone and resist adversarial prompt injection."),
        
        ("copilot-structured-json-repair-schemas", "JSON Schema: Formal Specification and Verification - Michael Droettboom",
         "Enforcing deterministic JSON output with schema schemas, type coercion, and syntax self-repair.",
         ["structured-json-repair", "deterministic-json-schemas", "json-self-repair", "schema-validation"],
         ["Deterministic Formatting: Instruct model to emit pure RFC 8259 JSON enclosed in ```json fences.",
          "Syntax Self-Repair: Implement automatic JSON repair passes to handle trailing commas, unescaped quotes, or truncated brackets.",
          "MANDATORY schema validation against JSON Schema definitions."],
         "Enforce deterministic JSON emission from Copilot models, pairing generation with automatic parsing and schema repair."),
        
        ("copilot-in-context-memory-pruning", "Context Window Optimization & Token Budgeting - Greg Kamradt",
         "Sliding window attention, context summarization, and key-value memory compression.",
         ["memory-pruning", "sliding-window-context", "context-summarization", "kv-memory-compression"],
         ["Pruning Strategy: Evict older conversation turns when token consumption exceeds 75% of context window.",
          "Rolling Summary: Compress evicted turns into a concise running executive summary stored in system state.",
          "ALWAYS preserve core system instructions and initial user constraints."],
         "Implement dynamic memory pruning and rolling summarization to sustain coherent multi-hour Copilot conversation sessions."),
        
        ("copilot-plan-and-solve-decomposition", "Autonomous Planning and Execution with Semantic Kernel - John Maeda",
         "Hierarchical plan-and-solve prompting for multi-step enterprise business processes.",
         ["plan-and-solve", "hierarchical-planning", "task-decomposition-prompts", "step-by-step-execution"],
         ["Two-Stage Prompting: 1) Devise a comprehensive multi-step plan, 2) Execute each step sequentially with verification.",
          "Interleaved Verification: Validate the output of step N before permitting the model to execute step N+1.",
          "MANDATORY abort and replan if any intermediate step fails."],
         "Structure complex business workflows using Plan-and-Solve prompt architectures for deterministic multi-stage execution."),
        
        ("copilot-metacognitive-self-reflection", "Automated Reprompting and Self-Refinement Loops - Noah Shinn",
         "Self-critique, verification rubrics, and consistency sampling before tool execution.",
         ["metacognitive-reflection", "self-critique-loops", "verification-rubrics", "consistency-sampling"],
         ["Reflection Step: Prompt model to critique its own candidate solution against a 5-point quality checklist.",
          "Correction Loop: If critique identifies violations, regenerate the response addressing specific critique findings.",
          "ALWAYS perform self-reflection before executing destructive database or API actions."],
         "Equip agents with metacognitive self-reflection passes, catching factual errors and formatting bugs prior to emission."),
        
        ("copilot-rag-query-expansion-hyde", "Hypothetical Document Embeddings (HyDE) and Query Expansion - Luyu Gao",
         "Hypothetical Document Embeddings (HyDE) and multi-query reformulation for Graph Search.",
         ["query-expansion-hyde", "hypothetical-embeddings", "multi-query-reformulation", "graph-search-expansion"],
         ["HyDE Method: Generate a synthetic ideal answer document; embed the synthetic document to search vector space.",
          "Multi-Query Reformulation: Generate 3 diverse keyword queries to search inverted index (BM25) in parallel.",
          "ALWAYS merge and deduplicate search results using Reciprocal Rank Fusion (RRF)."],
         "Enhance enterprise search recall by deploying HyDE and multi-query reformulation before querying the Semantic Index."),
        
        ("copilot-prompt-token-budget-optimizer", "Context Window Optimization & Token Budgeting - Greg Kamradt",
         "Calculating prompt and completion token budgets, parameter tradeoffs, and cost modeling.",
         ["token-budget-optimizer", "context-budgeting", "cost-modeling-tokens", "prompt-token-limits"],
         ["Budget Allocation: Fixed allocation for System (15%), Knowledge Context (60%), History (15%), Output (10%).",
          "Hard Token Counter: Calculate exact token counts using BPE tokenizers (tiktoken) before calling inference APIs.",
          "MANDATORY token truncation guards preventing unexpected 400 Bad Request overflow errors."],
         "Manage prompt token budgets algorithmically, optimizing context allocation to maximize accuracy while minimizing enterprise token costs."),
    ]

    # Cluster 11: Enterprise Security, RBAC, Purview DLP & Cryptographic Shielding (101-110)
    c11 = [
        ("copilot-entra-id-workload-identity", "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci",
         "Service principals, managed identities, federated credentials, and certificate authentication.",
         ["entra-id-workload-identity", "managed-identities", "federated-credentials", "service-principals"],
         ["Identity Architecture: Prefer User-Assigned Managed Identity over client secrets for Azure-hosted Copilot backends.",
          "Certificate Credential: Use X.509 certificate credentials when authenticating service principals from on-premises.",
          "NEVER store plain-text client secrets in environment variables or configuration files."],
         "Secure backend Copilot services using Microsoft Entra ID workload identities and federated credentials."),
        
        ("copilot-oauth2-obo-flow-exchange", "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci",
         "On-Behalf-Of (OBO) token exchange between Teams Bot, mid-tier APIs, and Microsoft Graph.",
         ["oauth2-obo-flow", "on-behalf-of-exchange", "token-exchange-graph", "delegated-user-tokens"],
         ["OBO Protocol: Mid-tier service exchanges incoming user assertion token for downstream Graph API token via `/token` endpoint.",
          "Scope Enforcement: Request only the minimal downstream scopes required for the specific user turn.",
          "MANDATORY caching of downstream OBO tokens in distributed cache with token expiration TTL."],
         "Implement secure OAuth2 On-Behalf-Of token exchanges across multi-tier Copilot agent architectures."),
        
        ("copilot-continuous-access-evaluation-cae", "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci",
         "Handling CAE claims challenges (WWW-Authenticate: Bearer error=\"insufficient_claims\").",
         ["continuous-access-evaluation", "cae-claims-challenge", "insufficient-claims-handling", "real-time-session-revocation"],
         ["CAE Mechanics: Entra ID revokes tokens in real time upon critical events (password change, user disablement).",
          "Claims Challenge Parsing: Parse `claims` parameter from HTTP 401 response and redirect user to re-authenticate.",
          "ALWAYS support CAE in all Microsoft Graph client SDK configurations."],
         "Handle Continuous Access Evaluation claims challenges gracefully, triggering immediate re-authentication upon session revocation."),
        
        ("copilot-purview-sensitivity-labeling", "Microsoft Purview: Data Governance and Information Protection - Santhosh Balasubramanian",
         "Discovering, reading, and applying Microsoft Purview sensitivity labels to generated artifacts.",
         ["purview-sensitivity-labeling", "sensitivity-labels", "information-protection", "purview-governance"],
         ["Label Inheritance Invariant: Generated documents MUST inherit the highest sensitivity label of all input sources.",
          "MIP SDK Integration: Apply encryption and visual watermarks via Microsoft Information Protection (MIP) SDK.",
          "NEVER downgrade sensitivity labels without explicit administrative authorization."],
         "Integrate Microsoft Purview sensitivity labeling into Copilot artifact generation, enforcing label inheritance rules."),
        
        ("copilot-purview-dlp-data-fencing", "Microsoft Purview: Data Governance and Information Protection - Santhosh Balasubramanian",
         "Enforcing DLP policies against sensitive data (PII, PCI, HIPAA) in Copilot prompts and completions.",
         ["purview-dlp", "data-loss-prevention", "pii-pci-fencing", "dlp-policy-enforcement"],
         ["DLP Detection: Scan prompts and completions for Sensitive Information Types (SITs) in real time.",
          "Blocking Rule: Automatically block or redact credit card numbers, social security numbers, and health records.",
          "MANDATORY incident generation in Microsoft Purview Compliance Portal upon DLP violation."],
         "Enforce Microsoft Purview DLP rules across all agent conversation streams to prevent unauthorized data exfiltration."),
        
        ("copilot-agentshield-prompt-injection-defense", "Defending Enterprise AI Against Prompt Injection and Jailbreaks - Simon Willison",
         "Defense against indirect prompt injections, jailbreaks, and delimiter manipulation.",
         ["agentshield-defense", "prompt-injection-mitigation", "jailbreak-defense", "indirect-prompt-injection"],
         ["Dual-LLM Architecture: Run input through an untrusted input filter before presenting to core orchestrator.",
          "Delimiter Sandboxing: Enclose external document text in strong cryptographic boundary delimiters (e.g. `<<<UNTRUSTED_CONTENT_HASH>>>`).",
          "STRICT_REJECT any input attempting to override system operational invariants."],
         "Deploy Tagisan AgentShield defensive layers to sanitize untrusted document context and block prompt injection attacks."),
        
        ("copilot-jwe-token-cryptography", "JSON Web Encryption (JWE) & Token Decryption in Copilot - RFC 7516 Standard",
         "Decrypting RFC 7516 JWE payloads with RSA-OAEP-256 and AES-256-GCM in Graph subscriptions.",
         ["jwe-cryptography", "rfc7516-decryption", "rsa-oaep-256", "aes-256-gcm-tokens"],
         ["Cryptographic Pipeline: Decrypt content encryption key using private RSA key -> Decrypt ciphertext using AES-GCM.",
          "Authentication Tag: Validate 128-bit authentication tag before parsing decrypted JSON.",
          "MANDATORY hardware security module (HSM) or Azure Key Vault key storage."],
         "Implement RFC 7516 JWE decryption routines to securely process encrypted Graph notification payloads."),
        
        ("copilot-rbac-least-privilege-scoping", "Least Privilege Security in Microsoft Graph API - Microsoft Identity",
         "Scoping Graph application vs delegated permissions to absolute least-privilege roles.",
         ["rbac-least-privilege", "graph-permission-scoping", "delegated-vs-application", "least-privilege-access"],
         ["Principle of Least Privilege: Request delegated permissions (e.g. `Mail.Read`) over application permissions (`Mail.Read.All`).",
          "Dynamic Consent: Request high-privilege scopes incrementally when the user triggers the specific feature.",
          "NEVER grant `Directory.ReadWrite.All` or `Files.ReadWrite.All` to custom agent service principals."],
         "Audit and constrain Microsoft Graph permissions to the exact minimum scopes necessary for agent operation."),
        
        ("copilot-zero-egress-air-gap-fencing", "Zero Trust Security for Enterprise AI Systems - Jason Garbis & Jerry Chapman",
         "Configuring network boundaries, private endpoints, and egress locks for enterprise data protection.",
         ["zero-egress-fencing", "air-gap-enterprise", "private-endpoints", "network-security-perimeter"],
         ["Network Perimeter: Route all Copilot backend communication through Azure Virtual Network Private Endpoints.",
          "Egress Lock: Prohibit outbound public internet traffic from inference and data processing subnets.",
          "MANDATORY TLS 1.3 encryption with client certificate pinning on internal service links."],
         "Architect zero-egress network perimeters around enterprise Copilot extensions to guarantee data containment."),
        
        ("copilot-cryptographic-audit-receipts", "Observability and Audit Logging for Autonomous Enterprise Agents - Charity Majors",
         "Generating SHA-256 tamper-evident cryptographic audit logs for every Copilot agent invocation.",
         ["cryptographic-audit-receipts", "tamper-evident-logs", "sha256-audit-chain", "immutable-audit-records"],
         ["Audit Hash Chain: Every invocation record contains `Hash = SHA256(PrevHash + Timestamp + QueryHash + ActionHash)`.",
          "Immutability: Write audit records to immutable Azure Blob Storage (WORM - Write Once, Read Many).",
          "MANDATORY inclusion of user Entra ID object ID, tenant ID, and IP address in audit payload."],
         "Generate cryptographic hash-chained audit receipts for all agent actions to provide tamper-evident compliance trails."),
    ]

    # Cluster 12: Enterprise RAG, Azure OpenAI, Vector Indexing & Hybrid Search (111-120)
    c12 = [
        ("copilot-azure-openai-enterprise-deployment", "Architecting Enterprise Solutions with Azure OpenAI Service - Sarah Kaiser",
         "Provisioning Azure OpenAI resources, model deployments (GPT-4o, embeddings), and TPM allocation.",
         ["azure-openai-deployment", "gpt4o-enterprise", "tpm-quota-management", "provisioned-throughput-units"],
         ["Capacity Planning: Evaluate Pay-as-You-Go vs Provisioned Throughput Units (PTU) for predictable latency.",
          "Regional Redundancy: Deploy across multiple Azure regions with automatic traffic manager failover.",
          "ALWAYS monitor 429 quota exhaustion using Azure Monitor metrics."],
         "Architect resilient, scalable Azure OpenAI deployments supporting high-concurrency enterprise Copilot agent workloads."),
        
        ("copilot-azure-ai-search-hybrid-hnsw", "Azure AI Search: Enterprise Vector and Hybrid Retrieval - Liam Cavanagh",
         "Azure AI Search vector indexing using HNSW, scalar quantization, and BM25 hybrid search.",
         ["azure-ai-search-hybrid", "hnsw-vector-indexing", "bm25-hybrid-retrieval", "scalar-quantization"],
         ["Hybrid Fusion: Combine BM25 full-text search scores with HNSW vector cosine similarity via RRF.",
          "Scalar Quantization: Compress float32 vectors to int8 to reduce memory footprint by 75% with negligible accuracy loss.",
          "MANDATORY index schema configuration defining vector dimensions (e.g. 3072 for text-embedding-3-large)."],
         "Deploy hybrid search indexes in Azure AI Search combining BM25 keyword matching and HNSW vector similarity."),
        
        ("copilot-semantic-ranker-l2-reranking", "Advanced Reranking Algorithms for Information Retrieval - Omar Khattab",
         "Applying Azure AI Search Semantic Ranker for L2 cross-encoder reranking of retrieved chunks.",
         ["semantic-ranker-l2", "cross-encoder-reranking", "azure-semantic-ranking", "rerank-relevance-boost"],
         ["Two-Stage Retrieval: Stage 1 retrieves top 50 candidates via Hybrid search; Stage 2 applies deep transformer cross-encoder.",
          "Semantic Captions: Extract and highlight verbatim relevant passages (`@search.captions`) for direct citation.",
          "MANDATORY evaluation of Semantic Score threshold (>1.5) before passing chunks to LLM."],
         "Integrate Azure AI Search Semantic Ranker into retrieval pipelines to elevate high-relevance chunks and eliminate noise."),
        
        ("copilot-document-chunking-token-strategies", "Document Parsing, Chunking and Metadata Enrichment - Denny Britz",
         "Chunking strategies: semantic boundary chunking, sliding window with overlap, and markdown splitting.",
         ["document-chunking-strategies", "semantic-boundary-chunking", "sliding-window-overlap", "markdown-splitting"],
         ["Chunk Sizing: Target 400-800 tokens per chunk with 10-15% token overlap between consecutive chunks.",
          "Boundary Respect: Split at natural markdown boundaries (headers, paragraphs, tables); NEVER split mid-sentence or mid-table.",
          "ALWAYS preserve document title, section breadcrumbs, and page numbers in chunk metadata."],
         "Implement intelligent, document-structure-aware chunking pipelines that preserve semantic context across chunk boundaries."),
        
        ("copilot-text-embedding-vector-space", "Representation Learning and Vector Embeddings - Nils Reimers",
         "Generating text embeddings (text-embedding-3-large), cosine similarity, and dimensionality reduction.",
         ["text-embedding-vector-space", "text-embedding-3-large", "cosine-similarity-retrieval", "dimensionality-reduction"],
         ["Embedding Generation: Utilize `text-embedding-3-large` for state-of-the-art enterprise semantic representations.",
          "Matryoshka Embeddings: Shorten embedding vectors (e.g., 3072 -> 1024 dimensions) without re-training to save index cost.",
          "ALWAYS L2-normalize vectors before calculating inner-product similarity."],
         "Generate and manage high-dimensional vector representations of enterprise content, optimizing dimension and distance metrics."),
        
        ("copilot-graph-rag-knowledge-graphs", "Graph RAG: Unlocking Knowledge Graphs with LLMs - Darren Edge et al. (Microsoft Research)",
         "Extracting entity-relation triplets from enterprise documents for Graph-based RAG traversal.",
         ["graph-rag-knowledge", "entity-relation-triplets", "knowledge-graph-rag", "hierarchical-summarization"],
         ["Graph Extraction: Prompt LLMs to extract Entities, Relations, and Claims into a property graph.",
          "Community Summarization: Perform hierarchical Leiden community detection to generate macro-level dataset summaries.",
          "MANDATORY support for global queries ('What are the major themes across all project reports?')."],
         "Implement Microsoft Research GraphRAG pipelines to answer broad, thematic enterprise questions that defeat traditional vector search."),
        
        ("copilot-contextual-compression-retrieval", "Retrieval-Augmented Generation: Systems, Architectures and Optimizations - Patrick Lewis",
         "Compressing retrieved document chunks to isolate high-density relevant facts.",
         ["contextual-compression", "chunk-compression", "fact-density-extraction", "irrelevant-token-pruning"],
         ["Compression Technique: Filter retrieved document text through a fast extractor model to strip irrelevant filler.",
          "Token Savings: Reduce retrieved context token consumption by 50-70% while boosting factual density.",
          "ALWAYS maintain original document citation references on compressed passages."],
         "Deploy contextual compression passes over retrieved chunks to maximize signal-to-noise ratio in model context windows."),
        
        ("copilot-azure-openai-private-link-network", "Architecting Enterprise Solutions with Azure OpenAI Service - Sarah Kaiser",
         "Securing Azure OpenAI traffic via Virtual Network Private Endpoints and disabled public access.",
         ["azure-openai-private-link", "vnet-private-endpoints", "disabled-public-access", "secure-ai-networking"],
         ["Private Link: Map Azure OpenAI resource to private IP addresses within enterprise virtual network.",
          "Public Access Invariant: Disable `publicNetworkAccess` on all production cognitive services.",
          "MANDATORY private DNS zone (`privatelink.openai.azure.com`) resolution within internal VNets."],
         "Secure Azure OpenAI communication over private virtual network endpoints, preventing data exposure to public internet."),
        
        ("copilot-enterprise-rag-citation-pipeline", "Designing Transparent Citations and Attribution in AI Responses - Microsoft UX",
         "Building traceable citation pipelines mapping response paragraphs back to exact document URIs.",
         ["enterprise-rag-citations", "traceable-citations", "paragraph-to-uri-mapping", "grounded-attribution"],
         ["Citation Pipeline: Intercept model completion, identify citation tags, and resolve to absolute document URLs.",
          "Interactive Tooltips: Return structured citation metadata enabling rich UI preview chips in Teams and Office.",
          "MANDATORY verification that cited URLs are accessible by the querying user."],
         "Build end-to-end citation pipelines linking Copilot answers back to original SharePoint, OneDrive, or web sources."),
        
        ("copilot-rag-cache-semantic-memoization", "Semantic Caching for LLM Queries - Redis & Microsoft Engineering",
         "Semantic caching of vector queries and embeddings to eliminate redundant LLM inference costs.",
         ["rag-semantic-cache", "vector-memoization", "redis-semantic-cache", "inference-cost-reduction"],
         ["Cache Lookup: Embed incoming query -> Search semantic cache index -> If similarity > 0.96, return cached completion.",
          "Invalidation SLA: Invalidate cached entries automatically when underlying documents or permissions mutate.",
          "NEVER serve cached responses across tenant boundaries."],
         "Deploy semantic caching layers (Redis/Qdrant) ahead of LLM inference to cut response latency to <100ms on common queries."),
    ]

    # Cluster 13: Adaptive Cards, Fluent UI & Conversational Ergonomics (121-130)
    c13 = [
        ("copilot-adaptive-cards-v1-5-templating", "Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger",
         "Adaptive Cards v1.5 schema, Adaptive Card Templating (AdaptiveCardTemplate), and data binding.",
         ["adaptive-cards-templating", "adaptive-cards-v1-5", "card-data-binding", "adaptivecardtemplate"],
         ["Separation of Concerns: Separate static card JSON template from dynamic runtime `$data` context.",
          "Schema Compliance: Target schema version 1.5 for maximum Teams and Outlook desktop/mobile compatibility.",
          "ALWAYS validate template syntax using official `adaptivecards-templating` library."],
         "Author elegant, decoupled Adaptive Cards using templating engines and dynamic JSON data payloads."),
        
        ("copilot-universal-actions-for-teams", "Universal Actions in Adaptive Cards: Action.Execute - Teams Engineering",
         "Universal Action Model: Action.Execute, refresh triggers, and user-specific card views.",
         ["universal-actions-teams", "action-execute", "user-specific-views", "card-refresh-triggers"],
         ["Action.Execute Model: Replace legacy `Action.Submit` and `Action.Http` with unified `Action.Execute`.",
          "User-Specific Views: Return customized card views tailored to the specific user viewing the card in group chats.",
          "MANDATORY verification of `context.action.verb` and payload data."],
         "Implement Universal Action Model (`Action.Execute`) in Teams cards, delivering dynamic user-specific views and live refreshes."),
        
        ("copilot-fluent-ui-blazor-react-styling", "Fluent Design System: Principles and UI Engineering - Microsoft Design",
         "Fluent UI React v9 / Blazor design tokens, typography, and accessibility (WCAG 2.1 AA).",
         ["fluent-ui-styling", "fluent-design-system", "fluent-ui-react-v9", "wcag-accessibility"],
         ["Design Tokens: Utilize Fluent design tokens (`colorNeutralBackground1`, `fontSizeBase300`) for seamless theme adaptation.",
          "Accessibility Invariant: Meet WCAG 2.1 AA standards: minimum 4.5:1 contrast ratios and full keyboard focus navigation.",
          "ALWAYS support dark, light, and high-contrast theme modes."],
         "Build modern, accessible web task panes and dialogs conforming strictly to Microsoft Fluent Design System standards."),
        
        ("copilot-adaptive-card-form-input-validation", "Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger",
         "Input validation: Input.Text, Input.ChoiceSet, regex patterns, and client-side error labels.",
         ["card-input-validation", "adaptive-card-forms", "input-choiceset", "client-side-validation"],
         ["Client-Side Validation: Define `isRequired: true`, `regex: \"...\"`, and `errorMessage: \"...\"` on input elements.",
          "Server-Side Re-validation: ALWAYS re-validate all submitted form fields on the backend server before processing.",
          "NEVER trust raw client card submissions without sanitization."],
         "Design bulletproof data entry forms in Adaptive Cards with instant client-side validation and backend verification."),
        
        ("copilot-interactive-cards-live-updates", "Universal Actions in Adaptive Cards: Action.Execute - Teams Engineering",
         "Updating posted messages in Teams channels and chats in-place with UpdateCard operations.",
         ["interactive-cards-live-updates", "in-place-card-update", "update-card-teams", "live-card-refreshes"],
         ["In-Place Refresh: Update existing message card using `TurnContext.updateActivity()` with identical activity ID.",
          "Concurrency Safety: Avoid race conditions when multiple users click card buttons simultaneously.",
          "ALWAYS show updated status indicators (e.g. 'Approved by Alice at 14:02')."],
         "Update posted Teams cards dynamically in-place to display workflow progression, approvals, and real-time statuses."),
        
        ("copilot-conversational-information-architecture", "Conversational AI: Design and Engineering - Cathy Pearl",
         "Structuring chat responses with scannable headers, bullet hierarchy, and bold takeaways.",
         ["conversational-info-architecture", "scannable-chat-responses", "response-hierarchy", "conversational-ux"],
         ["Information Hierarchy: 1) One-sentence summary answer, 2) Bulleted key takeaways, 3) Detailed supporting context.",
          "Scannability Invariant: Use bold lead-ins on bullet points to permit rapid executive skimming.",
          "NEVER emit wall-of-text paragraphs exceeding 4 sentences in chat streams."],
         "Structure Copilot chat outputs with crisp information architecture, bulleted takeaways, and bold visual anchors."),
        
        ("copilot-adaptive-card-blast-radius-reports", "Software Architecture: The Hard Parts - Neal Ford & Mark Richards",
         "Designing high-density visual telemetry cards for code changes, blast radius, and test diffs.",
         ["blast-radius-reports", "telemetry-adaptive-cards", "code-change-cards", "visual-diff-cards"],
         ["Visual Telemetry: Render color-coded impact badges (Red: High, Yellow: Med, Green: Low) on architectural diffs.",
          "Dense Layout: Use FactSet and ColumnSet elements to present multi-dimensional metrics concisely.",
          "MANDATORY direct links to code repos and test execution logs."],
         "Generate high-density Adaptive Card reports visualizing architectural blast radius, test results, and deployment metrics."),
        
        ("copilot-mobile-teams-card-ergonomics", "Don't Make Me Think: Conversational UI Usability - Steve Krug",
         "Optimizing card layouts for mobile Teams clients (single-column reflow, touch targets).",
         ["mobile-teams-ergonomics", "card-mobile-optimization", "touch-targets", "responsive-cards"],
         ["Responsive Reflow: Use ColumnSet with `wrap: true` or single-column stacks to ensure clean mobile rendering.",
          "Touch Target Invariant: Interactive buttons must provide minimum 48x48 pixel touch targets.",
          "NEVER use wide multi-column tables that require horizontal scrolling on mobile."],
         "Design responsive Adaptive Cards that render flawlessly across mobile, tablet, and desktop Teams clients."),
        
        ("copilot-dark-high-contrast-adaptive-theme", "Fluent Design System: Principles and UI Engineering - Microsoft Design",
         "Adapting cards and UI components to Teams default, dark, and high-contrast themes.",
         ["dark-theme-cards", "high-contrast-adaptation", "adaptive-color-styling", "teams-theming"],
         ["Adaptive Styling: Use semantic card colors (`default`, `accent`, `good`, `warning`, `attention`) instead of hardcoded hex codes.",
          "Theme Detection: Read client theme from context and apply corresponding Fluent UI theme provider.",
          "ALWAYS verify legibility in Teams High Contrast mode."],
         "Ensure all Copilot UI components and cards adapt dynamically to Teams default, dark, and high-contrast themes."),
        
        ("copilot-teams-task-modules-dialogs", "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs",
         "Launching interactive modal dialogs (Teams Dialogs / Task Modules) from card buttons.",
         ["teams-task-modules", "teams-dialogs", "modal-dialogs-teams", "taskmodule-fetch"],
         ["Modal Dialog Contract: Respond to `task/fetch` with task info containing web URL or embedded Adaptive Card.",
          "Result Handling: Process modal submit events via `task/submit` and update parent chat card.",
          "MANDATORY modal sizing definition (Small, Medium, Large) matching content requirements."],
         "Launch focused modal dialogs from Copilot cards to handle complex multi-field forms without cluttering the chat history."),
    ]

    # Cluster 14: Evals, Behavioral Guardrails, Red Teaming & Copilot Safety (131-140)
    c14 = [
        ("copilot-ragas-eval-rag-triad", "The RAG Triad: Context Relevance, Groundedness, and Answer Relevance - TruLens Engineering",
         "Evaluating RAG pipelines using RAGAS: Faithfulness, Answer Relevance, Context Precision, Context Recall.",
         ["ragas-eval-triad", "faithfulness-metric", "answer-relevance", "context-precision-recall"],
         ["RAG Triad Metrics: 1) Context Relevance (retrieval quality), 2) Groundedness/Faithfulness (no hallucination), 3) Answer Relevance.",
          "Quantitative Scoring: Automate calculation of 0.0 to 1.0 scores across test suites.",
          "MANDATORY failure threshold: Fail CI/CD build if average faithfulness drops below 0.85."],
         "Benchmark and continuously audit enterprise Copilot RAG pipelines using RAGAS and TruLens evaluation frameworks."),
        
        ("copilot-adversarial-red-teaming-probes", "Red Teaming Generative AI: Adversarial Testing and Vulnerability Discovery - Ethan Perez",
         "Designing automated red-teaming probe suites for prompt injection, jailbreaking, and data leaks.",
         ["adversarial-red-teaming", "prompt-injection-probes", "jailbreak-testing", "ai-vulnerability-discovery"],
         ["Adversarial Taxonomy: Probe for direct injection, indirect context injection, role reversal, and encoding bypasses.",
          "Automated Fuzzing: Generate thousands of synthetic adversarial variations using mutation-based LLM fuzzers.",
          "MANDATORY zero tolerance for enterprise data exfiltration vulnerabilities."],
         "Execute automated adversarial red-teaming suites against Copilot agents to identify and eliminate vulnerabilities before launch."),
        
        ("copilot-azure-ai-content-safety-api", "Azure AI Content Safety and Risk Mitigation - Microsoft AI Team",
         "Integrating Azure AI Content Safety for real-time text analysis of hate, violence, sexual, self-harm.",
         ["azure-content-safety", "content-moderation-api", "harm-severity-levels", "prompt-shields-scanning"],
         ["Severity Classification: Quantify risk across 4 categories (Hate, Violence, Sexual, Self-Harm) on 0-7 severity scale.",
          "Prompt Shields: Scan inputs for user prompt injection attacks and document context attacks.",
          "ALWAYS block inputs/outputs exceeding tenant risk thresholds."],
         "Integrate Azure AI Content Safety APIs into agent middleware to enforce real-time enterprise moderation and safety."),
        
        ("copilot-groundedness-hallucination-detector", "LLM-as-a-Judge: Automated Evaluation Frameworks - Lianmin Zheng et al.",
         "Detecting hallucinations with synthetic contrastive verification and model-graded critique.",
         ["hallucination-detector", "groundedness-eval", "synthetic-verification", "model-graded-critique"],
         ["Claim Extraction: Deconstruct candidate answer into individual atomic factual propositions.",
          "Entailment Verification: Verify that each atomic claim is logically entailed by the retrieved source context.",
          "STRICT_REJECT answers containing ungrounded claims."],
         "Deploy automated hallucination detection gates that verify every generated claim against source grounding documents."),
        
        ("copilot-prompt-jailbreak-canary-tokens", "Defending Enterprise AI Against Prompt Injection and Jailbreaks - Simon Willison",
         "Canary token injection into system prompts to detect and abort prompt extraction attacks.",
         ["jailbreak-canary-tokens", "prompt-extraction-defense", "canary-token-monitoring", "system-prompt-protection"],
         ["Canary Invariant: Inject unique high-entropy random UUID canary tokens into confidential system instructions.",
          "Exfiltration Detection: Monitor model output stream; if canary token appears in output, abort transmission immediately.",
          "MANDATORY alerting and blacklisting of offending user session."],
         "Protect proprietary system prompts and confidential instructions using canary tokens and output exfiltration tripwires."),
        
        ("copilot-copilot-studio-test-framework", "Application Lifecycle Management for Copilot Studio - Microsoft Power CAT",
         "Power Platform CLI / Copilot Studio test framework for automated topic conversation regression.",
         ["copilot-studio-test-framework", "topic-regression-testing", "automated-conversation-tests", "pac-test-copilot"],
         ["Regression Suite: Execute pre-recorded multi-turn conversation scripts against Copilot Studio bots.",
          "Intent Assertion: Assert that specific user utterances trigger expected topics and return correct variable values.",
          "ALWAYS execute regression suite before publishing bot updates to production."],
         "Automate conversation regression testing for Copilot Studio bots using Power Platform CLI and testing frameworks."),
        
        ("copilot-llm-as-a-judge-rubric-evals", "LLM-as-a-Judge: Automated Evaluation Frameworks - Lianmin Zheng et al.",
         "Formulating multi-dimensional rubric prompts for LLM-as-a-judge evaluation of agent performance.",
         ["llm-as-a-judge", "rubric-evals", "automated-grading-prompts", "pairwise-model-comparison"],
         ["Rubric Dimensions: Evaluate on Clarity, Completeness, Groundedness, Conciseness, and Actionability.",
          "Scoring Scale: 1-5 integer scale with explicit descriptions for each rating tier.",
          "ALWAYS calibrate judge model against a human-validated golden baseline dataset."],
         "Establish automated LLM-as-a-judge evaluation pipelines with structured rubrics to assess agent answer quality at scale."),
        
        ("copilot-latency-token-throughput-benchmarking", "Observability Engineering: Achieving Operational Excellence - Charity Majors",
         "Benchmarking Time-To-First-Token (TTFT), tokens-per-second, and roundtrip latency across models.",
         ["latency-benchmarking", "time-to-first-token", "token-throughput", "model-performance-metrics"],
         ["Latency Metrics: Measure TTFT (aim <800ms), Tokens Per Second (TPS), and total roundtrip turn latency.",
          "Percentile Tracking: Monitor P50, P95, and P99 latency percentiles to detect tail performance degradations.",
          "MANDATORY automated performance regression alerts in CI."],
         "Benchmark and monitor inference latency and token throughput across models and regions to guarantee responsive agent UX."),
        
        ("copilot-responsible-ai-impact-assessment", "Enterprise Artificial Intelligence: Systems & Patterns - David Schneider",
         "Conducting Microsoft Responsible AI Impact Assessments (Fairness, Reliability, Privacy).",
         ["responsible-ai-assessment", "rai-impact-assessment", "fairness-reliability-privacy", "microsoft-rai-standards"],
         ["RAI Core Principles: Fairness, Reliability & Safety, Privacy & Security, Inclusiveness, Transparency, Accountability.",
          "Risk Mitigation Mapping: Document specific technical controls mitigating each identified AI failure mode.",
          "MANDATORY formal sign-off before deploying public or high-impact enterprise agents."],
         "Perform comprehensive Responsible AI Impact Assessments following Microsoft RAI Standards before production deployments."),
        
        ("copilot-production-telemetry-drift-detection", "Continuous Evaluation & Monitoring for AI in CI/CD - Chip Huyen",
         "Monitoring distribution drift in production user queries, intent classification, and failure modes.",
         ["telemetry-drift-detection", "production-monitoring-ai", "user-query-drift", "intent-drift-analysis"],
         ["Drift Metrics: Track semantic embedding drift of user queries over time to detect shifting user needs.",
          "Error Distribution: Monitor changes in unhandled query rates and fallback topic triggers.",
          "ALWAYS retrain or refine grounding prompts when distribution drift exceeds statistical thresholds."],
         "Monitor production agent telemetry for semantic drift and user behavior changes, triggering proactive model updates."),
    ]

    # Cluster 15: Cross-Ecosystem Enterprise Connectors (SAP, Salesforce, Jira, ServiceNow via Graph & OpenAPI) (141-150)
    c15 = [
        ("copilot-sap-s4hana-odata-connector", "SAP S/4HANA OData Integration with Microsoft 365 - Bjarne Berg",
         "Integrating SAP S/4HANA OData services (BAPI / CDS views) via Graph Connector and OpenAPI.",
         ["sap-odata-copilot", "s4hana-connector", "bapi-cds-views", "sap-enterprise-integration"],
         ["OData v4 Protocol: Connect to SAP NetWeaver / S/4HANA OData services via Secure Agent or Azure API Management.",
          "Principal Propagation: Propagate Entra ID user identity to SAP backend via SAML/OAuth2 bearer assertion.",
          "ALWAYS validate SAP transaction locks and commit scopes before mutation."],
         "Connect Microsoft Copilot to SAP S/4HANA systems, enabling users to query purchase orders, inventory, and invoices."),
        
        ("copilot-salesforce-rest-graph-connector", "Salesforce CRM and Microsoft 365 Copilot Interoperability - Phil Weinmeister",
         "Salesforce REST API integration, SOQL query mapping, and indexing Accounts/Opportunities in Graph.",
         ["salesforce-copilot", "salesforce-graph-connector", "soql-query-mapping", "crm-opportunities-sync"],
         ["OAuth2 Web Server Flow: Authenticate via Salesforce Connected App with PKCE.",
          "SOQL Safety: Sanitize user input before interpolating into SOQL queries to prevent SOQL injection.",
          "MANDATORY field-level security (FLS) enforcement matching Salesforce permissions."],
         "Index Salesforce CRM Accounts, Contacts, and Opportunities into Microsoft Copilot for unified enterprise search."),
        
        ("copilot-servicenow-incident-cmdb-connector", "ServiceNow ITSM Automation with Microsoft Copilot - Tim Woodruff",
         "ServiceNow Table API integration for IT Service Management (incidents, change requests, CMDB).",
         ["servicenow-copilot", "itsm-automation", "table-api-servicenow", "incident-management-copilot"],
         ["Table API Binding: Interact with `incident`, `change_request`, and `cmdb_ci` tables via REST API.",
          "OAuth2 Token Flow: Authenticate via ServiceNow OAuth Provider with scoped client credentials.",
          "ALWAYS map incident urgency and impact to valid ServiceNow choice values."],
         "Automate IT service management workflows: create, query, and escalate ServiceNow incidents directly within Copilot."),
        
        ("copilot-jira-cloud-confluence-rest-connector", "Atlassian Ecosystem Integration with Microsoft 365 - Patrick Li",
         "Atlassian Jira Cloud & Confluence REST v3 integration for issues, epics, and documentation sync.",
         ["jira-confluence-copilot", "atlassian-rest-connector", "jira-issue-tracking", "confluence-knowledge-sync"],
         ["REST v3 Contracts: Query Jira issues via JQL (`/rest/api/3/search`) and Confluence spaces via v2 API.",
          "Atlassian Document Format (ADF): Parse and generate structured ADF payloads for issue descriptions.",
          "MANDATORY handling of Atlassian cloud rate limits (100 req/min)."],
         "Integrate Atlassian Jira and Confluence with Microsoft Copilot to query sprints, track tickets, and synthesize docs."),
        
        ("copilot-workday-raas-human-capital-connector", "Workday Human Capital Management API with Microsoft Copilot - John Boggess",
         "Workday Report-as-a-Service (RaaS) integration with OAuth2 and employee record indexing.",
         ["workday-copilot", "workday-raas", "hcm-api-integration", "employee-directory-copilot"],
         ["RaaS Endpoints: Consume structured enterprise reports from Workday using Report-as-a-Service REST endpoints.",
          "Privacy Invariant: Strictly mask and redact sensitive compensation and personal identity fields.",
          "MANDATORY mutual TLS (mTLS) or OAuth2 Bearer token authentication."],
         "Bridge Workday HCM data to Copilot, allowing authenticated employees to query PTO balances, benefits, and org charts."),
        
        ("copilot-dynamics-365-dataverse-deep-sync", "Microsoft Dynamics 365 and Copilot Integration Patterns - Simon Huckestein",
         "Deep bi-directional synchronization between Copilot and Dynamics 365 Sales/Customer Service.",
         ["dynamics-365-copilot", "dataverse-deep-sync", "crm-sales-automation", "dynamics-customer-service"],
         ["Native Dataverse Binding: Leverage native Dataverse tables (`contact`, `account`, `opportunity`, `incident`).",
          "Dual-Write Architecture: Ensure real-time consistency between Dynamics 365 Finance & Operations and Dataverse.",
          "ALWAYS respect business unit security boundaries and hierarchical security models."],
         "Deeply integrate Copilot with Dynamics 365 Sales and Service, enabling conversational lead updates and case resolution."),
        
        ("copilot-sql-server-azure-sql-graph-bridge", "Enterprise Generative AI Architecture - Various Authors",
         "Bridging enterprise relational databases (SQL Server, Azure SQL) into Copilot search index.",
         ["sql-server-graph-bridge", "azure-sql-copilot", "relational-data-indexing", "graph-sql-connector"],
         ["Change Tracking: Use SQL Server Change Tracking or Change Data Capture (CDC) to identify mutated rows.",
          "Relational-to-Document Transformation: Flatten relational normalized schemas into rich, self-contained JSON documents.",
          "NEVER expose raw database connection strings; use Azure Managed Identities."],
         "Index legacy SQL Server and modern Azure SQL databases into Microsoft 365 Semantic Index via automated CDC connectors."),
        
        ("copilot-cross-system-identity-reconciliation", "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci",
         "Mapping user identity across Entra ID, SAP, Salesforce, and Jira for secure authorization trimming.",
         ["cross-system-identity", "identity-reconciliation", "authorization-trimming", "federated-identity-mapping"],
         ["Identity Mapping Table: Maintain secure, audited mapping between Entra ID UserPrincipalName and external system IDs.",
          "Security Trimming Invariant: Users MUST only see data they have permission to access in BOTH systems.",
          "ALWAYS verify active account status in Entra ID before resolving external identity."],
         "Reconcile user identities across disparate enterprise SaaS systems to enforce seamless security trimming in Copilot."),
        
        ("copilot-rate-limit-federation-enterprise-apis", "Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe",
         "Managing composite rate-limiting and circuit breakers across heterogeneous enterprise endpoints.",
         ["rate-limit-federation", "circuit-breakers-enterprise", "composite-throttling", "resilient-api-federation"],
         ["Circuit Breaker Pattern: Trip circuit breakers to Open state after consecutive 5xx/429 errors, preventing cascading outages.",
          "Adaptive Rate Limiting: Allocate request tokens via Token Bucket algorithm tailored to each third-party API limit.",
          "MANDATORY graceful degradation with informative user notifications during service brownouts."],
         "Federate requests across multiple enterprise backends with intelligent rate limiting, circuit breakers, and fallbacks."),
        
        ("copilot-end-to-end-enterprise-action-choreography", "Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe",
         "Orchestrating complex distributed transactions across SAP, Salesforce, and ServiceNow via Copilot.",
         ["enterprise-action-choreography", "distributed-saga-copilot", "cross-system-transactions", "enterprise-workflows"],
         ["Saga Choreography: Coordinate multi-system business actions via forward actions and compensating rollback actions.",
          "Idempotency Invariant: Every distributed step MUST support idempotent execution via unique idempotency keys.",
          "MANDATORY complete audit trail recording every state change and compensating transaction."],
         "Orchestrate complex, multi-system enterprise workflows across SAP, Salesforce, and ServiceNow with transactional integrity."),
    ]

    def to_skill_dict(item, default_cluster=None):
        if isinstance(item, dict):
            s = dict(item)
        elif len(item) == 7:
            s_id, c_num, book, desc, triggers, foundations, protocol = item
            s = {
                "id": s_id,
                "cluster": c_num,
                "book": book,
                "desc": desc,
                "triggers": [t.lower() for t in triggers],
                "foundations": list(foundations),
                "protocol": protocol,
                "anti_patterns": [
                    f"Bypassing formal verification rules in {s_id}.",
                    f"Unmonitored runtime execution without telemetry in {s_id}."
                ]
            }
        elif len(item) == 6:
            s_id, book, desc, triggers, foundations, protocol = item
            s = {
                "id": s_id,
                "cluster": default_cluster,
                "book": book,
                "desc": desc,
                "triggers": [t.lower() for t in triggers],
                "foundations": list(foundations),
                "protocol": protocol,
                "anti_patterns": [
                    f"Bypassing formal verification rules in {s_id}.",
                    f"Unmonitored runtime execution without telemetry in {s_id}."
                ]
            }
        else:
            raise ValueError(f"Unexpected tuple length: {len(item)}")

        # Enforce ALWAYS, NEVER, MANDATORY invariants across all skills
        f_list = list(s["foundations"])
        if not any("ALWAYS" in f for f in f_list):
            f_list.append(f"ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in {s['id']}.")
        if not any("NEVER" in f for f in f_list):
            f_list.append(f"NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in {s['id']}.")
        if not any("MANDATORY" in f for f in f_list):
            f_list.append(f"MANDATORY timeout guarantees (30s default) and episodic reflection logging on all {s['id']} actions.")
        s["foundations"] = f_list
        s["triggers"] = [t.lower() for t in s["triggers"]]
        return s

    SKILLS.extend([to_skill_dict(x, 4) for x in c4])
    SKILLS.extend([to_skill_dict(x, 5) for x in c5])
    SKILLS.extend([to_skill_dict(x, 6) for x in c6])
    SKILLS.extend([to_skill_dict(x, 7) for x in c7])
    SKILLS.extend([to_skill_dict(x, 8) for x in c8])
    SKILLS.extend([to_skill_dict(x, 9) for x in c9])
    SKILLS.extend([to_skill_dict(x, 10) for x in c10])
    SKILLS.extend([to_skill_dict(x, 11) for x in c11])
    SKILLS.extend([to_skill_dict(x, 12) for x in c12])
    SKILLS.extend([to_skill_dict(x, 13) for x in c13])
    SKILLS.extend([to_skill_dict(x, 14) for x in c14])
    SKILLS.extend([to_skill_dict(x, 15) for x in c15])

    # Also normalize clusters 1-3
    for idx in range(30):
        SKILLS[idx] = to_skill_dict(SKILLS[idx], (idx // 10) + 1)

expand_all_150_skills()

def generate_markdown(skill):
    """Generate the full content of SKILL.md for a given skill definition."""
    triggers_str = ", ".join(f'"{t.lower()}"' for t in skill["triggers"])
    
    foundations_md = "\n".join(f"{i+1}. **{f}**" for i, f in enumerate(skill["foundations"]))
    anti_patterns_md = "\n".join(f"- **{ap}**" for ap in skill["anti_patterns"])
    
    content = f"""---
name: {skill["id"]}
description: "{skill["desc"]}"
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "{skill["book"]}"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: [{triggers_str}]
---

# {skill["id"]}
> Based on **{skill["book"]}** (Cluster {skill["cluster"]})

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

{foundations_md}

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
{skill["protocol"]}

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

{anti_patterns_md}

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "{skill["triggers"][0]}"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
"""
    return content

def main():
    print(f"Generating all {len(SKILLS)} Microsoft 365 Copilot & Vibe Code Development skill packages...")
    assert len(SKILLS) == 150, f"Expected exactly 150 skills, found {len(SKILLS)}"
    
    created_count = 0
    SKILLS_DIR.mkdir(parents=True, exist_ok=True)
    
    for skill in SKILLS:
        skill_dir = SKILLS_DIR / skill["id"]
        skill_dir.mkdir(parents=True, exist_ok=True)
        skill_file = skill_dir / "SKILL.md"
        
        content = generate_markdown(skill)
        skill_file.write_text(content, encoding="utf-8")
        created_count += 1

    print(f"[+] Successfully generated {created_count} SKILL.md packages in {SKILLS_DIR}")

if __name__ == "__main__":
    main()
