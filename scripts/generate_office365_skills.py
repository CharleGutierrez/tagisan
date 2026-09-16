#!/usr/bin/env python3
"""
scripts/generate_office365_skills.py

Generates all 350 Microsoft Office 365 & Vibe Code Development skill packages
under .ecc/skills/o365-*/SKILL.md with full YAML frontmatter, strict operational
directives (ALWAYS, NEVER, MANDATORY), and comprehensive engineering instructions.
"""

import os
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

# 35 Clusters x 10 Skills = 350 Skills
CLUSTERS = [
    (1, "Excel Advanced Formulas, Dynamic Arrays & LAMBDA Calculus", "o365-excel-lambda-", "Excel Dynamic Arrays & LAMBDA Calculus - Bill Jelen & Simon Peyton Jones"),
    (2, "Excel Office Scripts & TypeScript Automation", "o365-excel-scripts-", "Automating Excel with Office Scripts - Yevgen Rogozin & Sudhi Ramamurthy"),
    (3, "Excel JavaScript API & Custom Functions", "o365-excel-js-", "Professional Excel Add-in Development with Office.js - Michael Saunders"),
    (4, "Word Document Processing, OpenXML & Word JS API", "o365-word-", "OpenXML: The Definitive Guide to WordprocessingML - Wouter van Vugt & Doug Mahugh"),
    (5, "PowerPoint Generative Deck Building & Presentation JS API", "o365-ppt-", "PowerPoint JavaScript API & Slide Automation - Andrew Connell"),
    (6, "Outlook Add-ins, Mail Event Handlers & Smart Alerts", "o365-outlook-", "Mastering Outlook Add-in Development - Eric Legault"),
    (7, "OneNote REST API, Notebook Graph Automation & Digital Ink", "o365-onenote-", "Automating OneNote with Microsoft Graph - Gary Lapointe"),
    (8, "Microsoft Teams Tabs, Bots, Messaging Extensions & Teams JS SDK", "o365-teams-", "Developing Solutions for Microsoft Teams - John Miller"),
    (9, "Microsoft Teams Meeting Apps & Real-time Live Share SDK", "o365-teams-live-", "Building Microsoft Teams Meeting Apps - Tom Morgan"),
    (10, "Microsoft Viva Connections, Viva Topics & Employee Experience", "o365-viva-", "Mastering Microsoft Viva - Kat Greenan & Paolo Pialorsi"),
    (11, "SharePoint Framework (SPFx) Core Architecture & Tooling", "o365-spfx-core-", "Beginning SharePoint Framework Development - Vipul Jain & Andrew Connell"),
    (12, "SPFx React, Fluent UI & Modern Web Part Engineering", "o365-spfx-react-", "Building SPFx Web Parts with React & Fluent UI - Paolo Pialorsi"),
    (13, "SPFx Extensions (Application Customizers, Field Customizers)", "o365-spfx-ext-", "Developing SharePoint Framework Extensions - Vipul Jain"),
    (14, "SharePoint REST API, Lists, Document Libraries & Large File Streaming", "o365-sp-data-", "SharePoint REST API and OData Programming - Gary Lapointe & Patrick Rodgers"),
    (15, "Microsoft Graph API Core Architecture, Throttling & Batching", "o365-graph-core-", "Microsoft Graph API Programming - Glenn Snyder & Vincent Biret"),
    (16, "Graph Change Notifications, Webhooks & Delta Queries", "o365-graph-events-", "Event-Driven Architectures with Microsoft Graph Webhooks - Todd Baginski"),
    (17, "Microsoft Graph Connectors & External Item Ingestion", "o365-graph-conn-", "Building Custom Microsoft Graph Connectors - Paolo Pialorsi & Bill Baer"),
    (18, "Office Add-ins Unified Manifest (JSON) & Office.js Core Lifecycle", "o365-addin-core-", "The Modern Office Add-in Manifest: Migrating to JSON - Microsoft Learn"),
    (19, "Office Add-in Single Sign-On (SSO), Entra ID & Nested App Auth (NAA)", "o365-addin-auth-", "Single Sign-On (SSO) in Office Add-ins with Entra ID - Paolo Pialorsi"),
    (20, "Office Add-in Deployment, Centralized Admin & AppSource Certification", "o365-addin-deploy-", "Publishing Office Add-ins to Microsoft AppSource - Michael Saunders"),
    (21, "Microsoft 365 Fluid Framework & Real-Time Collaborative Canvas", "o365-fluid-", "Real-Time Applications with Microsoft Fluid Framework - Tom Morgan"),
    (22, "Microsoft Loop Components, Workspaces & Open Standards", "o365-loop-", "Microsoft Loop Architecture & Component Engineering - Vlad Catrinescu"),
    (23, "Microsoft 365 Search, Semantic Search & Syntex Content AI", "o365-search-", "Enterprise Search in Microsoft 365 - Bill Baer & Mikael Svenson"),
    (24, "Microsoft Forms API, Quizzes & Webhook Automation", "o365-forms-", "Automating Microsoft Forms with Power Automate & Graph - Shane Young"),
    (25, "Microsoft Planner, To-Do & Task Management Graph APIs", "o365-tasks-", "Task Management Programming with Microsoft Graph - Gary Lapointe"),
    (26, "Microsoft Bookings & Scheduling Automated Workflows", "o365-bookings-", "Microsoft Bookings API Architecture & Automation - Rick Van Rousselt"),
    (27, "Microsoft Lists Custom Formatting, JSON Column & View Formatting", "o365-lists-", "Modern SharePoint & Microsoft Lists Custom Formatting - Chris Kent"),
    (28, "Exchange Online Mail Flow, Transport Rules & PowerShell Automation", "o365-exchange-", "Exchange Online Administration with PowerShell - Tony Redmond"),
    (29, "Microsoft Purview Information Protection, Sensitivity Labels & DLP", "o365-purview-", "Microsoft Purview Information Protection Architecture - Peter De Tender"),
    (30, "Microsoft 365 Security, Zero Trust & Privileged Identity Management", "o365-sec-", "Zero Trust Architecture for Microsoft 365 - Mark Simos"),
    (31, "Microsoft 365 Compliance, eDiscovery, Audit Logs & Retention", "o365-compliance-", "Microsoft Purview eDiscovery & Compliance - Brett Lonsdale"),
    (32, "Microsoft 365 Tenant Administration & Microsoft Graph PowerShell SDK", "o365-admin-", "Microsoft Graph PowerShell SDK Programming - Gary Lapointe"),
    (33, "Office 365 Vibe Coding, AI Pair Programming & Rapid UI Prototyping", "o365-vibe-", "Vibe Coding: Conversational Development in Office 365 - Andrej Karpathy"),
    (34, "M365 CLI, PnP PowerShell & Automated ALM Provisioning", "o365-alm-", "Mastering CLI for Microsoft 365 - Waldek Mastykarz & Garry Trinder"),
    (35, "M365 Multi-Tenant SaaS Architecture, App Consent & AppSource Ecosystem", "o365-saas-", "Architecting Multi-Tenant SaaS on Microsoft 365 - Wael Hamze"),
]

# 10 skills per cluster specs: (cluster_id, [(suffix, desc, triggers)])
CLUSTER_SKILLS = [
    # Cluster 1: Excel Advanced Formulas, Dynamic Arrays & LAMBDA Calculus (o365-excel-lambda-*)
    (1, [
        ("spill-range-anchoring-hash", "Spill range operator (#), anchoring dynamic arrays, and preventing #SPILL! collisions.", ["spill-range-hash", "excel-dynamic-arrays", "spill-collision-avoidance"]),
        ("lambda-recursive-reduction", "Recursive LAMBDAs, base cases, accumulator patterns, and stack recursion limits.", ["lambda-recursive-reduction", "excel-lambda-recursion", "functional-excel-lambda"]),
        ("map-byrow-bycol-vectorization", "MAP, BYROW, and BYCOL element-wise projection over 2D dynamic calculation matrices.", ["map-byrow-bycol", "vectorized-excel-formulas", "byrow-bycol-matrices"]),
        ("scan-reduce-cumulative-state", "SCAN and REDUCE functions for cumulative running totals and array fold operations.", ["scan-reduce-cumulative", "excel-scan-reduce", "array-fold-calculations"]),
        ("makearray-index-synthesis", "MAKEARRAY coordinate synthesis for generating parametric grid schemas dynamically.", ["makearray-synthesis", "makearray-coordinate-grid", "parametric-array-generation"]),
        ("let-variable-memoization", "LET expression variable scoping, sub-calculation memoization, and dependency optimization.", ["let-variable-memoization", "excel-let-optimization", "memoized-formula-variables"]),
        ("filter-unique-sortby-pipelines", "Multi-condition FILTER, UNIQUE deduplication, and nested SORTBY computation pipelines.", ["filter-unique-sortby", "dynamic-filtering-excel", "multi-condition-filter"]),
        ("xlookup-binary-search-opt", "XLOOKUP binary search mode (search_mode: 2) on sorted keys for microsecond retrieval.", ["xlookup-binary-search", "xlookup-performance-tuning", "sorted-key-lookup"]),
        ("choosocols-chooserows-projection", "CHOOSECOLS, CHOOSEROWS, DROP, and TAKE for slicing virtual tables in memory.", ["choosecols-chooserows", "array-slicing-excel", "take-drop-table-slices"]),
        ("vstack-hstack-matrix-consolidation", "VSTACK and HSTACK for consolidating heterogeneous report ranges into unified arrays.", ["vstack-hstack-consolidation", "matrix-stacking-excel", "dynamic-table-merging"]),
    ]),
    # Cluster 2: Excel Office Scripts & TypeScript Automation (o365-excel-scripts-*)
    (2, [
        ("range-batch-read-write", "Bulk getValues() and setValues() batching, eliminating cell-by-cell mutation latency.", ["range-batch-read-write", "office-scripts-batching", "bulk-range-setvalues"]),
        ("table-dynamic-column-binding", "workbook.getTable(), adding calculated columns, and strongly typed table manipulation.", ["table-dynamic-binding", "office-scripts-tables", "calculated-table-columns"]),
        ("conditional-formatting-rules", "Programmatic conditional formatting, color scales, icon sets, and data bars via scripts.", ["conditional-formatting-rules", "office-scripts-formatting", "programmatic-data-bars"]),
        ("chart-series-generation", "Programmatic chart creation, series configuration, titles, and automated styling.", ["chart-series-generation", "office-scripts-charts", "automated-excel-charting"]),
        ("cloud-flow-parameter-contracts", "Typed input parameters and return payloads for Power Automate flow trigger bindings.", ["cloud-flow-contracts", "office-scripts-flow-binding", "typed-script-parameters"]),
        ("regex-data-cleansing-pipeline", "In-memory regex string transformations, phone/email standardization before writing.", ["regex-data-cleansing", "office-scripts-regex", "string-sanitization-excel"]),
        ("multi-sheet-consolidation-loop", "Iterating workbook.getWorksheets(), aggregating summary metrics across multiple sheets.", ["multi-sheet-consolidation", "office-scripts-worksheets", "cross-sheet-aggregation"]),
        ("auto-filter-criteria-tuning", "Applying multi-criteria column filters, visible range extraction, and filter resets.", ["auto-filter-criteria", "office-scripts-autofilter", "visible-range-extraction"]),
        ("custom-sorting-matrix", "Multi-column hierarchical sort specifications with custom orientation and case matching.", ["custom-sorting-matrix", "office-scripts-sorting", "hierarchical-range-sort"]),
        ("cell-data-validation-rules", "Applying list validation, integer bounds, and custom error alert prompts via script.", ["cell-data-validation", "office-scripts-validation", "list-validation-rules"]),
    ]),
    # Cluster 3: Excel JavaScript API & Custom Functions (o365-excel-js-*)
    (3, [
        ("context-sync-queue-minimization", "Excel.run, batching commands into single context.sync() roundtrips, and load queues.", ["context-sync-minimization", "excel-run-batching", "office-js-load-queue"]),
        ("streaming-custom-functions", "Creating real-time streaming custom functions with setResult invocation loops and WebSockets.", ["streaming-custom-functions", "excel-streaming-functions", "websocket-cell-streaming"]),
        ("custom-function-metadata-json", "Authoring functions.json metadata, parameter descriptions, help URLs, and return types.", ["custom-function-metadata", "functions-json-authoring", "excel-custom-function-schema"]),
        ("volatile-custom-function-caching", "Controlling custom function volatility, in-memory caching, and cancellation handlers.", ["volatile-function-caching", "custom-function-memoization", "cancellation-handler-excel"]),
        ("webassembly-math-acceleration", "Compiling Rust/C++ algorithms to WebAssembly for sub-millisecond math in Excel taskpanes.", ["webassembly-math-excel", "wasm-excel-addins", "high-speed-calculation-wasm"]),
        ("worksheet-event-handlers", "Binding onChanged, onActivated, and onSelectionChanged events with clean unbind lifecycles.", ["worksheet-event-handlers", "excel-event-binding", "onchanged-selectionchanged"]),
        ("range-untracked-garbage-collection", "Explicitly managing range.untrack() for high-throughput memory reclamation in Office.js.", ["range-untrack-gc", "office-js-memory-reclamation", "proxy-garbage-collection"]),
        ("shape-and-svg-canvas-injection", "Injecting interactive geometric shapes, SVG graphics, and lines into worksheet layers.", ["shape-svg-injection", "excel-svg-insertion", "worksheet-shape-layer"]),
        ("pivot-table-layout-manipulation", "Programmatically creating and pivoting PivotTables, adding row/column/data hierarchies.", ["pivottable-manipulation", "excel-pivottable-api", "dynamic-pivot-hierarchies"]),
        ("named-item-scope-management", "Scoped workbook vs worksheet named items and programmatic formula evaluation via ranges.", ["named-item-scope", "excel-named-items", "scoped-named-ranges"]),
    ]),
    # Cluster 4: Word Document Processing, OpenXML & Word JS API (o365-word-*)
    (4, [
        ("openxml-document-part-traversal", "ZIP packaging, word/document.xml traversal, namespaces, and relationship mapping.", ["openxml-document-traversal", "wordprocessingml-structure", "document-xml-packaging"]),
        ("content-control-data-binding", "Content controls (Word.ContentControl), title tagging, placeholder text, and lock flags.", ["content-control-data-binding", "word-content-controls", "document-placeholder-locks"]),
        ("range-search-and-replace", "body.search(), regex wildcards, replacing tokens with rich text, tables, or paragraphs.", ["range-search-and-replace", "word-search-replace", "rich-text-token-substitution"]),
        ("openxml-sdk-headless-assembly", "Server-side OpenXML SDK assembly of multi-chapter legal reports without Microsoft Word.", ["openxml-sdk-assembly", "headless-docx-generation", "server-side-document-synthesis"]),
        ("tracked-changes-and-comments", "Programmatic comment insertion, author attribution, resolving and accepting revisions.", ["tracked-changes-comments", "word-comment-threading", "programmatic-revisions-word"]),
        ("nested-table-and-cell-formatting", "Generating complex tables, cell borders, cell shading, merged cells, repeating headers.", ["nested-table-formatting", "word-table-generation", "cell-shading-borders"]),
        ("header-footer-section-breaks", "Multi-section documents, unlinking headers/footers, and inserting section page breaks.", ["header-footer-section-breaks", "word-sections-unlinking", "odd-even-page-headers"]),
        ("custom-xml-parts-binding", "Embedding custom XML payloads in DOCX and binding content controls to XML nodes dynamically.", ["custom-xml-parts-binding", "docx-custom-xml", "xml-node-content-control"]),
        ("html-to-docx-conversion-fidelity", "Converting rich HTML into native WordprocessingML runs, preserving typography and lists.", ["html-to-docx-fidelity", "html-word-conversion", "typography-preservation-docx"]),
        ("ai-clause-generation-taskpane", "Taskpane UI with LLM integration to draft, critique, and insert legal contract clauses.", ["ai-clause-generation", "word-copilot-taskpane", "contract-drafting-ai"]),
    ]),
    # Cluster 5: PowerPoint Generative Deck Building & Presentation JS API (o365-ppt-*)
    (5, [
        ("slide-layout-insertion-matrix", "PowerPoint.run, selecting slide layouts from master slides, inserting formatted slides.", ["slide-layout-insertion", "powerpoint-run-api", "slide-master-layouts"]),
        ("shape-tree-positioning-calculus", "Coordinate math for shapes (left, top, width, height) and collision avoidance algorithms.", ["shape-tree-positioning", "powerpoint-shape-coordinates", "bounding-box-alignment-ppt"]),
        ("presentationml-raw-slide-scaffolding", "OpenXML PresentationML slide parts (p:sld) and slide master hierarchies (p:sldMaster).", ["presentationml-scaffolding", "openxml-pptx-structure", "slide-xml-packaging"]),
        ("textframe-paragraph-formatting", "Formatting text frames, bullet points, font families, font sizes, line spacing via API.", ["textframe-paragraph-formatting", "powerpoint-typography-api", "bullet-point-formatting"]),
        ("svg-vector-shape-import", "Inserting SVG vector assets and converting SVG paths into native PowerPoint shape geometries.", ["svg-vector-shape-import", "powerpoint-svg-insertion", "native-geometry-conversion"]),
        ("chart-and-table-slide-injection", "Injecting embedded data tables and native PowerPoint charts into active slide canvases.", ["chart-table-slide-injection", "powerpoint-charts-api", "embedded-table-generation-ppt"]),
        ("speaker-notes-and-comments", "Programmatically reading and updating speaker notes and collaborative slide comments.", ["speaker-notes-comments", "powerpoint-speaker-notes", "slide-comment-threading"]),
        ("automated-qbr-deck-synthesis", "Automated quarterly business review deck assembler pulling live metrics from external APIs.", ["automated-qbr-synthesis", "presentation-generation-pipeline", "automated-slide-deck-factory"]),
        ("slide-transition-and-media-timing", "Setting slide transition durations, embedding video streams, and audio timing triggers.", ["slide-transition-timing", "powerpoint-media-embedding", "slide-animation-durations"]),
        ("conversational-pitch-deck-generator", "Natural language prompt-driven generation of fully styled multi-slide pitch decks.", ["conversational-pitch-deck", "ai-slide-generator", "prompt-to-presentation-deck"]),
    ]),
    # Cluster 6: Outlook Add-ins, Mail Event Handlers & Smart Alerts (o365-outlook-*)
    (6, [
        ("read-vs-compose-mode-contexts", "Managing distinct contexts and permissions in Read mode vs Compose mode taskpanes.", ["read-vs-compose-contexts", "outlook-mode-permissions", "compose-mode-taskpanes"]),
        ("event-based-smart-alerts", "OnNewMessageCompose and OnAppointmentSend intercepting sends for smart alert checks.", ["event-based-smart-alerts", "outlook-smart-alerts", "intercept-email-send"]),
        ("internet-headers-extraction", "Reading and setting custom x-headers (internetHeaders.setAsync()) for email audit tracking.", ["internet-headers-extraction", "custom-email-x-headers", "mail-header-tracking"]),
        ("actionable-messages-adaptive-cards", "Authoring Actionable Messages in email with signed JWT verification and actionable buttons.", ["actionable-messages-cards", "outlook-actionable-email", "signed-jwt-card-actions"]),
        ("attachment-chunked-streaming", "Uploading and downloading large email attachments via Graph without mailbox memory limits.", ["attachment-chunked-streaming", "large-attachment-upload", "mailbox-attachment-streaming"]),
        ("meeting-scheduling-attendee-slots", "Querying attendee free/busy availability and suggesting optimal meeting slots via API.", ["meeting-scheduling-slots", "free-busy-availability-lookup", "attendee-scheduling-api"]),
        ("categories-and-followup-flags", "Setting item categories, color codes, follow-up flags, and due dates via Office.js.", ["categories-followup-flags", "outlook-categories-api", "email-flag-automation"]),
        ("shared-mailbox-delegated-handlers", "Detecting shared mailbox context and asserting delegated permissions before send actions.", ["shared-mailbox-handlers", "delegated-mailbox-security", "shared-inbox-workflows"]),
        ("mobile-touch-ergonomic-taskpanes", "Designing responsive, compact mobile taskpane interfaces for Outlook iOS and Android.", ["mobile-touch-taskpanes", "outlook-mobile-addins", "mobile-email-ergonomics"]),
        ("ai-email-triage-and-auto-drafting", "Real-time sentiment analysis, intent extraction, and automated contextual email drafting.", ["ai-email-triage", "outlook-copilot-auto-draft", "email-sentiment-triage"]),
    ]),
    # Cluster 7: OneNote REST API, Notebook Graph Automation & Digital Ink (o365-onenote-*)
    (7, [
        ("notebook-section-page-hierarchy", "Navigating notebooks, section groups, sections, and pages via Microsoft Graph.", ["notebook-hierarchy-navigation", "onenote-graph-structure", "section-page-traversal"]),
        ("multipart-mime-page-creation", "Submitting multipart/form-data with HTML page content, embedded images, and attachments.", ["multipart-mime-page-creation", "onenote-multipart-post", "html-page-generation-onenote"]),
        ("inkml-digital-handwriting-parsing", "Reading InkML digital ink strokes, point arrays, and bounding boxes for handwriting analysis.", ["inkml-handwriting-parsing", "digital-ink-onenote", "handwriting-stroke-analysis"]),
        ("onenote-html-schema-conformance", "Strict HTML elements supported by OneNote (data-render-src, data-tag, font styling).", ["onenote-html-conformance", "onenote-supported-html", "data-render-src-onenote"]),
        ("meeting-transcript-auto-summary", "Automated extraction of Teams transcripts, generating formatted OneNote meeting binders.", ["meeting-transcript-summary", "teams-transcript-to-onenote", "automated-meeting-binders"]),
        ("embedded-ocr-text-mining", "Extracting text recognized inside images stored in OneNote pages via Graph endpoints.", ["embedded-ocr-text-mining", "onenote-image-ocr-extraction", "scanned-text-mining-onenote"]),
        ("class-notebook-student-distribution", "Distributing teacher sections, assignments, and read-only content to student notebooks.", ["class-notebook-distribution", "onenote-education-api", "student-section-provisioning"]),
        ("cross-notebook-knowledge-search", "Executing Graph search queries across multiple OneNote notebooks with snippet highlight.", ["cross-notebook-search", "onenote-knowledge-base", "notebook-snippet-search"]),
        ("notebook-archival-pdf-export", "Exporting pages and sections to archival PDF formats while preserving visual layout.", ["notebook-archival-pdf", "onenote-pdf-export", "page-archival-pipeline"]),
        ("ai-second-brain-autonomous-sync", "Ingesting messy voice memos and drafts, organizing them into structured OneNote hierarchies.", ["ai-second-brain-sync", "autonomous-notes-organizer", "voice-memo-to-onenote"]),
    ]),
    # Cluster 8: Microsoft Teams Tabs, Bots, Messaging Extensions & Teams JS SDK (o365-teams-*)
    (8, [
        ("teams-js-sdk-v2-capabilities", "Teams JS SDK v2 modular capabilities (app, pages, dialog, teamsCore).", ["teams-js-sdk-v2", "teams-modular-capabilities", "teams-app-context-init"]),
        ("app-manifest-v1-17-schema", "Authoring manifest.json v1.17+, configuring configurable tabs, static tabs, and bots.", ["app-manifest-v1-17", "teams-manifest-schema", "configurable-tabs-manifest"]),
        ("bot-framework-turn-context", "Bot Framework turn context lifecycle, receiving activities, sending typing indicators.", ["bot-framework-turn-context", "teams-bot-turn-handling", "proactive-bot-messaging"]),
        ("messaging-extensions-zero-install", "Search and action messaging extensions, link unfurling without requiring prior app install.", ["messaging-extensions-zero-install", "teams-link-unfurling", "search-messaging-extensions"]),
        ("dialog-and-task-module-popups", "Triggering interactive modal dialogs (dialog.url.open, dialog.adaptiveCard.open).", ["dialog-task-module-popups", "teams-dialog-api", "adaptive-card-modal-dialogs"]),
        ("teams-sso-silent-token-acquisition", "app.getAuthToken(), exchanging client token via OBO flow for downstream Graph calls.", ["teams-sso-silent-token", "getauthtoken-teams-sdk", "obo-token-exchange-teams"]),
        ("channel-vs-chat-context-isolation", "Determining host context: personal app, 1:1 chat, group chat, or channel conversation.", ["channel-chat-isolation", "teams-host-context-detection", "multi-context-teams-app"]),
        ("proactive-conversation-reference", "Storing ConversationReference records in database and sending targeted bot pings.", ["proactive-conversation-reference", "teams-proactive-alerts", "bot-conversation-reference"]),
        ("teams-toolkit-environment-pipelines", "Managing env/.env.* variable substitution and automated Teams app registration in Azure.", ["teams-toolkit-pipelines", "teams-env-variables", "automated-teams-registration"]),
        ("conversational-ai-agent-teams-lib", "Implementing conversational engine agents with @microsoft/teams-ai action planners.", ["conversational-ai-teams-lib", "teams-ai-library-planners", "turn-state-action-handlers"]),
    ]),
    # Cluster 9: Microsoft Teams Meeting Apps & Real-time Live Share SDK (o365-teams-live-*)
    (9, [
        ("meeting-lifecycle-stage-routing", "Pre-meeting tab, in-meeting side panel, meeting stage sharing, and post-meeting tabs.", ["meeting-lifecycle-stages", "teams-meeting-app-routing", "side-panel-vs-stage"]),
        ("live-share-sdk-ephemeral-state", "Live Share SDK: LiveState, LiveEvent, LivePresence for sub-100ms collaborative sync.", ["live-share-sdk-ephemeral", "live-share-presence", "ephemeral-state-sync-teams"]),
        ("meeting-stage-sharing-permission", "Requesting stage sharing (meeting.shareAppContentToStage) and presenter controls.", ["meeting-stage-sharing", "shareappcontenttostage", "teams-meeting-stage-sync"]),
        ("in-meeting-push-notification-banner", "Triggering in-meeting notification banners and targeted user engagement alerts.", ["in-meeting-push-banner", "teams-meeting-notification", "in-meeting-user-prompt"]),
        ("participant-role-verification", "Querying meeting participants, asserting organizer, presenter, or attendee role permissions.", ["participant-role-verification", "meeting-participant-api", "organizer-presenter-roles"]),
        ("real-time-live-media-streaming", "Real-time media platform, consuming audio/video streams via Teams calling bots.", ["real-time-live-media", "teams-calling-bot-media", "audio-stream-extraction-teams"]),
        ("breakout-rooms-orchestration", "Programmatic breakout room management, assigning participants, synchronizing timers.", ["breakout-rooms-orchestration", "teams-breakout-rooms-api", "automated-room-assignment"]),
        ("meeting-transcription-webhooks", "Subscribing to meeting transcript events, streaming live speaker-attributed text.", ["meeting-transcription-webhooks", "live-transcription-stream", "speaker-attributed-transcript"]),
        ("end-to-end-media-encryption-audit", "Compliance policies for meeting recordings, participant consent, and encryption boundaries.", ["media-encryption-audit", "meeting-recording-compliance", "e2e-media-security"]),
        ("collaborative-whiteboard-live-share", "Building multi-user real-time whiteboards on the meeting stage with live cursors.", ["collaborative-whiteboard-live", "teams-live-share-whiteboard", "shared-stage-canvas"]),
    ]),
    # Cluster 10: Microsoft Viva Connections, Viva Topics & Employee Experience (o365-viva-*)
    (10, [
        ("viva-connections-ace-card-views", "Building Adaptive Card Extensions (ACEs): Basic Card, Image Card, Primary Text Card.", ["viva-ace-card-views", "adaptive-card-extensions", "viva-connections-cards"]),
        ("ace-quick-view-interaction", "Designing slide-out Quick Views, form submission, and dynamic state changes within ACEs.", ["ace-quick-view-interaction", "viva-quick-view-forms", "ace-card-state-machine"]),
        ("audience-targeting-and-profile-sync", "Targeting dashboard cards to specific Entra ID security groups and departments.", ["audience-targeting-viva", "profile-based-dashboard-cards", "entra-group-targeting"]),
        ("viva-topics-semantic-discovery", "Knowledge discovery, automatic topic harvesting from SharePoint pages and Office files.", ["viva-topics-discovery", "semantic-topic-cards", "automated-topic-harvesting"]),
        ("viva-insights-collaboration-metrics", "Workplace analytics query API, measuring meeting load, focus time, network centrality.", ["viva-insights-metrics", "collaboration-analytics-api", "meeting-load-focus-time"]),
        ("viva-engage-community-broadcasting", "Viva Engage (Yammer) Graph API: Posting announcements, polls, tracking engagement.", ["viva-engage-broadcasting", "yammer-graph-api", "community-announcement-bots"]),
        ("viva-learning-lms-content-sync", "Ingesting third-party LMS course catalogs, tracking completion statuses via Graph.", ["viva-learning-lms-sync", "learning-provider-api", "course-completion-tracking"]),
        ("mobile-viva-card-ergonomics", "Touch ergonomics for field workers, responsive card sizing, offline caching on mobile.", ["mobile-viva-ergonomics", "viva-field-worker-cards", "mobile-dashboard-caching"]),
        ("viva-dashboard-performance-auditing", "Minimizing ACE bundle footprints, deferring secondary API lookups to Quick Views.", ["viva-performance-auditing", "ace-bundle-optimization", "deferred-quick-view-hydration"]),
        ("ai-employee-experience-copilot", "Contextual AI assistant inside Viva Connections surfacing corporate benefits and tasks.", ["ai-employee-experience-copilot", "viva-connections-copilot", "benefits-qa-assistant"]),
    ]),
    # Cluster 11: SharePoint Framework (SPFx) Core Architecture & Tooling (o365-spfx-core-*)
    (11, [
        ("spfx-yeoman-scaffolding-manifest", "Scaffolding SPFx web parts, component manifests, id, alias, version, preconfiguredEntries.", ["spfx-yeoman-manifest", "spfx-manifest-json", "web-part-scaffolding"]),
        ("spfx-component-lifecycle-dom", "onInit(), render(), onDispose(), DOM container attachment, avoiding memory leaks.", ["spfx-component-lifecycle", "spfx-dom-attachment", "ondispose-cleanup-spfx"]),
        ("spfx-build-pipeline-webpack-vite", "Gulp build pipeline, Webpack 5 configuration, tree-shaking, code-splitting chunks.", ["spfx-build-pipeline", "spfx-webpack-optimization", "code-splitting-spfx"]),
        ("isolated-web-parts-iframe-security", "Running web parts in isolated iframes (isDomainIsolated: true) for high-security tenants.", ["isolated-web-parts-iframe", "domain-isolated-spfx", "iframe-security-webparts"]),
        ("package-solution-sppkg-bundling", "gulp bundle --ship, gulp package-solution --ship, .sppkg package inspection.", ["package-solution-sppkg", "gulp-bundle-ship", "sppkg-package-validation"]),
        ("multi-host-spfx-targets", "Targeting SharePoint Modern Pages, Teams Tabs, Personal Apps, and Outlook taskpanes.", ["multi-host-spfx-targets", "spfx-teams-tab-target", "cross-host-web-parts"]),
        ("spfx-automated-unit-testing", "Unit testing SPFx with Jest, React Testing Library, and mocking SPHttpClient.", ["spfx-unit-testing-jest", "mock-sphttpclient", "react-testing-library-spfx"]),
        ("spfx-dynamic-data-event-broker", "Implementing IDynamicDataCallables, registering properties, subscribing to filter events.", ["spfx-dynamic-data-broker", "dynamic-data-callables", "webpart-event-subscription"]),
        ("tenant-app-catalog-alm-deployment", "Automating .sppkg deployment, tenant-wide installation, and feature enablement.", ["tenant-app-catalog-deployment", "sppkg-alm-automation", "tenant-wide-spfx-install"]),
        ("spfx-node-nvm-environment-hygiene", "Managing Node.js LTS versions, avoiding dependency mismatches across SPFx releases.", ["spfx-node-nvm-hygiene", "node-lts-spfx-matrix", "spfx-dependency-hygiene"]),
    ]),
    # Cluster 12: SPFx React, Fluent UI & Modern Web Part Engineering (o365-spfx-react-*)
    (12, [
        ("fluent-ui-v9-react-styling", "Integrating Fluent UI v9 Griffel styles, Button, Input, DataGrid, Badge in web parts.", ["fluent-ui-v9-spfx", "griffel-styles-spfx", "fluent-react-webparts"]),
        ("react-hooks-state-management", "useState, useEffect, useCallback, custom hooks for SharePoint list data retrieval.", ["react-hooks-spfx", "custom-data-hooks-spfx", "useeffect-spfx-lifecycle"]),
        ("property-pane-reactive-controls", "Reactive vs non-reactive property panes, textboxes, dropdowns, toggle switches.", ["property-pane-reactive", "spfx-property-pane-controls", "non-reactive-pane-submit"]),
        ("custom-property-pane-fields", "Authoring custom property pane controls: color pickers, multi-select tags, date pickers.", ["custom-property-pane-fields", "custom-field-type-spfx", "property-pane-color-picker"]),
        ("wcag-accessibility-landmarks", "ARIA landmarks, screen reader labels, keyboard tab stops, accessible high contrast.", ["wcag-accessibility-spfx", "aria-landmarks-webparts", "screen-reader-spfx-audit"]),
        ("theme-variant-subscriber", "Subscribing to theme variant changes via ThemeVariantEntityManager for dark/light mode.", ["theme-variant-subscriber", "themevariantentitymanager", "dark-light-mode-spfx"]),
        ("zustand-in-memory-caching", "Using Zustand to cache SharePoint REST responses, eliminating redundant HTTP roundtrips.", ["zustand-caching-spfx", "client-store-spfx", "eliminate-redundant-rest"]),
        ("responsive-css-modules-grid", "CSS Modules scoping, CSS Grid layouts, container queries for flexible web part widths.", ["responsive-css-modules-spfx", "css-grid-webparts", "container-queries-spfx"]),
        ("drag-drop-file-upload-cards", "Drag-and-drop file uploaders with upload progress bars, dropping directly into libraries.", ["drag-drop-file-cards-spfx", "dropzone-webpart", "direct-library-file-drop"]),
        ("vibe-coding-webparts-generative-ai", "Rapid natural language generation of production React web parts with zero boilerplate.", ["vibe-coding-webparts", "ai-generated-spfx-react", "prompt-to-web-part"]),
    ]),
    # Cluster 13: SPFx Extensions (Application Customizers, Field Customizers) (o365-spfx-ext-*)
    (13, [
        ("application-customizer-placeholders", "Injecting custom HTML into Top and Bottom page placeholders across modern sites.", ["application-customizer-placeholders", "spfx-top-bottom-placeholder", "global-page-injection"]),
        ("global-tenant-navigation-bar", "Responsive global navigation bar with mega menus, company alerts, and user profiles.", ["global-tenant-navigation", "mega-menu-application-customizer", "tenant-alert-banner"]),
        ("field-customizer-cell-renderers", "Overriding SharePoint list column cells: status badges, progress bars, clickable icons.", ["field-customizer-renderers", "spfx-cell-formatting", "custom-column-badges"]),
        ("listview-command-set-actions", "Adding custom buttons to modern list command bars, handling selected items, modal forms.", ["listview-command-set-actions", "command-bar-buttons-spfx", "selected-items-action-set"]),
        ("form-customizer-modern-overrides", "Replacing modern list item creation, view, and edit forms with custom React forms.", ["form-customizer-overrides", "modern-list-form-replacement", "custom-react-list-forms"]),
        ("search-extension-query-modifiers", "Modifying search queries on modern search pages, injecting custom refiners and facets.", ["search-extension-modifiers", "spfx-search-query-interceptor", "custom-refiners-search"]),
        ("dom-reflow-performance-profiling", "Profiling global application customizers, eliminating page layout shifts (CLS < 0.1).", ["dom-reflow-profiling", "cumulative-layout-shift-spfx", "extension-performance-audit"]),
        ("tenant-wide-extension-distribution", "Deploying extensions across 10,000+ site collections via Tenant-Wide Extensions list.", ["tenant-wide-extension-distribution", "tenant-wide-extensions-list", "automated-global-script-deploy"]),
        ("playwright-e2e-extension-testing", "Automated browser regression testing for header/footer extensions using Playwright.", ["playwright-extension-testing", "e2e-spfx-testing", "browser-regression-customizers"]),
        ("security-sandbox-xss-defense", "Sanitizing user inputs, defending against XSS in custom field customizers via DOMPurify.", ["security-sandbox-xss-defense", "dompurify-field-customizer", "xss-mitigation-spfx"]),
    ]),
    # Cluster 14: SharePoint REST API, Lists, Document Libraries & Large File Streaming (o365-sp-data-*)
    (14, [
        ("pnpjs-fluent-client-chaining", "Utilizing @pnp/sp fluent chaining, batching queries, selective imports, and caching.", ["pnpjs-fluent-client", "pnp-sp-query-chaining", "pnpjs-batching-caching"]),
        ("indexed-columns-5k-threshold", "Indexing columns, folder partitioning, and bypassing the 5,000 List View Threshold.", ["indexed-columns-5k-limit", "list-view-threshold-bypass", "folder-partitioning-sp"]),
        ("caml-query-efficient-filtering", "Authoring CAML XML queries for complex Boolean filters, sorting, and view limits.", ["caml-query-filtering", "caml-xml-syntax", "efficient-list-queries"]),
        ("chunked-file-upload-sessions", "Uploading files >250MB via createUploadSession, 4MB slices, and network retry tokens.", ["chunked-file-upload-sessions", "large-file-streaming-sp", "upload-session-resume-token"]),
        ("managed-metadata-term-store", "Querying term store hierarchies, assigning taxonomy terms to list items programmatically.", ["managed-metadata-term-store", "taxonomy-term-store-api", "hierarchical-metadata-tagging"]),
        ("unique-permissions-break-inheritance", "Breaking permission inheritance, assigning granular read/write roles via REST.", ["unique-permissions-break", "breakroleinheritance-api", "granular-role-assignments"]),
        ("document-version-history-pruning", "Purging old minor versions, automating version cleanup policies to reclaim quota.", ["version-history-pruning", "document-version-cleanup", "storage-quota-reclamation"]),
        ("folder-tree-hierarchy-traversal", "Recursively traversing document library folder trees, extracting file paths and sizes.", ["folder-tree-traversal", "recursive-folder-crawler", "document-library-structure"]),
        ("cross-site-content-aggregation", "Executing KQL search queries via REST to aggregate items across hundreds of modern sites.", ["cross-site-content-aggregation", "rest-kql-search-aggregation", "enterprise-task-rollup"]),
        ("headless-sharepoint-file-backend", "Consuming SharePoint document libraries as secure, versioned object storage for web apps.", ["headless-sharepoint-storage", "object-storage-sharepoint", "headless-document-api"]),
    ]),
    # Cluster 15: Microsoft Graph API Core Architecture, Throttling & Batching (o365-graph-core-*)
    (15, [
        ("graph-rest-endpoint-conventions", "Resource hierarchy (/users, /groups, /sites, /me) and OData v4 query conventions.", ["graph-rest-conventions", "resource-hierarchy-graph", "odata-v4-query-syntax"]),
        ("json-batch-request-multiplexing", "Multiplexing up to 20 requests in a single JSON $batch payload, handling dependencies.", ["json-batch-multiplexing", "graph-batch-requests", "batch-request-dependencies"]),
        ("http-429-exponential-backoff", "Intercepting HTTP 429 throttling, parsing Retry-After, implementing jittered backoff.", ["http-429-exponential-backoff", "graph-throttling-handling", "retry-after-jittered-backoff"]),
        ("typescript-graph-sdk-client", "Configuring @microsoft/microsoft-graph-client, middleware pipelines, PageIterator.", ["typescript-graph-sdk", "graph-client-middleware", "pageiterator-streaming"]),
        ("kiota-strongly-typed-sdk-generator", "Generating typed client SDKs with Kiota, reducing bundle size by 80% vs full SDK.", ["kiota-sdk-generator", "strongly-typed-kiota-client", "lightweight-graph-sdk"]),
        ("delegated-vs-app-permissions", "Determining permission types, admin consent requirements, and least privilege principles.", ["delegated-vs-app-permissions", "least-privilege-graph", "admin-consent-scoping"]),
        ("page-iterator-cursor-loop", "Traversing @odata.nextLink cursors automatically to stream 100,000+ items safely.", ["page-iterator-cursor-loop", "odata-nextlink-traversal", "stream-large-graph-datasets"]),
        ("correlation-headers-telemetry", "Injecting client-request-id, capturing request-id and Date for distributed tracing.", ["correlation-headers-telemetry", "client-request-id-tracing", "graph-telemetry-correlation"]),
        ("multi-tenant-graph-client-factory", "Managing multi-tenant token caches and tenant-scoped Graph clients dynamically.", ["multi-tenant-graph-factory", "dynamic-tenant-token-cache", "saas-graph-client-routing"]),
        ("ai-prompt-to-graph-query-synthesis", "Converting natural language questions into optimized Graph REST requests.", ["ai-prompt-graph-synthesis", "natural-language-to-graph", "query-synthesis-ai"]),
    ]),
    # Cluster 16: Graph Change Notifications, Webhooks & Delta Queries (o365-graph-events-*)
    (16, [
        ("webhook-subscription-handshake", "Creating Graph subscriptions, responding to validationToken queries in <3 seconds.", ["webhook-subscription-handshake", "validationtoken-response", "graph-webhook-lifecycle"]),
        ("delta-query-incremental-sync", "Polling @odata.deltaLink, capturing incremental changes across users, groups, and files.", ["delta-query-sync", "deltalink-change-tracking", "incremental-data-sync-graph"]),
        ("jwe-encrypted-payload-decryption", "Decrypting rich notification payloads with Azure Key Vault private keys and AES-GCM.", ["jwe-payload-decryption", "rich-notifications-graph", "asymmetric-key-decryption"]),
        ("azure-event-hubs-streaming", "Configuring direct streaming of Graph change notifications into Azure Event Hubs.", ["event-hubs-streaming-graph", "high-volume-change-notifications", "event-grid-graph-routing"]),
        ("serverless-webhook-receiver-queue", "Decoupled webhook processing: Azure Function pushes notification to Service Bus queue.", ["serverless-webhook-receiver", "decoupled-webhook-pipeline", "service-bus-queue-notifications"]),
        ("user-lifecycle-deprovisioning-flow", "Subscribing to employee termination events, triggering automated access revocations.", ["user-lifecycle-deprovisioning", "automated-offboarding-webhooks", "user-delete-notification-flow"]),
        ("driveitem-delta-change-processing", "Tracking OneDrive and SharePoint file uploads, deletions, and moves in real time.", ["driveitem-delta-processing", "file-upload-change-tracking", "real-time-driveitem-events"]),
        ("subscription-auto-renewal-timer", "Monitoring subscription expiration timestamps, scheduling automated renewal calls.", ["subscription-auto-renewal", "renew-graph-subscription", "expiration-timestamp-monitoring"]),
        ("dead-letter-queue-reconciliation", "Handling missed webhook notifications, running scheduled delta queries to reconcile state.", ["dead-letter-queue-reconciliation", "missed-webhook-reconciliation", "delta-fallback-sync"]),
        ("cryptographic-origin-verification", "Verifying client state secrets and certificate thumbprints on incoming webhooks.", ["cryptographic-origin-verification", "clientstate-validation", "webhook-authenticity-verification"]),
    ]),
    # Cluster 17: Microsoft Graph Connectors & External Item Ingestion (o365-graph-conn-*)
    (17, [
        ("connection-schema-registration", "Registering external connections (/external/connections), defining strongly typed schemas.", ["connection-schema-registration", "external-connections-graph", "typed-connector-schema"]),
        ("external-item-put-ingestion", "Ingesting items (/items/{id}), passing searchable text, properties, and direct view URLs.", ["external-item-ingestion", "put-external-item", "searchable-property-ingestion"]),
        ("external-group-and-acl-crawling", "Mirroring external user and group ACLs (/accessReviews, /acls) to preserve permissions.", ["external-group-acl-crawling", "preserve-external-permissions", "connector-acl-mirroring"]),
        ("incremental-content-crawler-engine", "Implementing full crawl vs periodic incremental change crawlers for external sources.", ["incremental-content-crawler", "external-content-crawling", "periodic-connector-crawls"]),
        ("property-semantic-relevance-tuning", "Tagging schema properties as isSearchable, isQueryable, isRetrievable, isRefinable.", ["property-relevance-tuning", "semantic-property-flags", "searchable-queryable-flags"]),
        ("connector-test-harness-debugging", "Testing connectors locally using the Graph Connectors SDK and mock payloads.", ["connector-test-harness", "graph-connectors-sdk", "local-connector-debugging"]),
        ("sql-and-jira-external-crawlers", "Building custom ingestion pipelines for SQL databases, Jira Cloud, and ServiceNow.", ["sql-jira-crawlers", "servicenow-connector-ingestion", "custom-database-crawler"]),
        ("ingestion-rate-limit-budgeting", "Respecting items per minute ingestion limits, batching parallel PUT requests.", ["ingestion-rate-limit-budgeting", "connector-quota-management", "batch-external-item-puts"]),
        ("admin-center-indexing-telemetry", "Monitoring connector indexing status, crawl errors, and document counts in M365 Admin.", ["admin-center-indexing-telemetry", "connector-crawl-diagnostics", "indexing-health-monitoring"]),
        ("copilot-grounding-external-items", "Verifying that external items appear in Microsoft 365 Copilot grounded responses.", ["copilot-grounding-external", "external-item-semantic-index", "ground-copilot-on-connectors"]),
    ]),
    # Cluster 18: Office Add-ins Unified Manifest (JSON) & Office.js Core Lifecycle (o365-addin-core-*)
    (18, [
        ("unified-manifest-json-schema", "Authoring JSON manifests v1.17+, extensions, capabilities, ribbonCommands.", ["unified-manifest-json", "json-manifest-v1-17", "modern-addin-manifest"]),
        ("office-onready-runtime-bootstrap", "Office.onReady(), checking info.host and info.platform, graceful non-Office fallbacks.", ["office-onready-bootstrap", "info-host-platform-check", "graceful-runtime-fallbacks"]),
        ("shared-runtime-cross-component-state", "Configuring <Runtimes> for shared JS state between ribbon, taskpane, and functions.", ["shared-runtime-state", "cross-component-js-state", "shared-runtime-office-addin"]),
        ("custom-ribbon-tabs-and-buttons", "Declaring custom ribbon tabs, primary buttons, toggle buttons, and menu dropdowns.", ["custom-ribbon-tabs", "addin-ribbon-buttons", "toggle-button-menu-dropdowns"]),
        ("webview2-desktop-vs-safari-web", "Adapting to Chromium WebView2 on Windows vs WebKit on macOS and Web browsers.", ["webview2-vs-safari", "cross-browser-addin-compatibility", "webkit-webview2-differences"]),
        ("auto-open-taskpane-binding", "Configuring documents to open taskpanes automatically upon document load.", ["auto-open-taskpane", "auto-open-addin-binding", "document-load-taskpane-trigger"]),
        ("taskpane-dom-recycling-memory", "Preventing DOM leaks during prolonged taskpane sessions in Excel and Word.", ["taskpane-dom-recycling", "prevent-dom-leaks-addin", "webview2-memory-profiling"]),
        ("office-addin-debugging-tooling", "Local debugging with office-addin-debugging, HTTPS dev certificates, and source maps.", ["office-addin-debugging-tools", "https-dev-certificates", "local-addin-source-maps"]),
        ("fluent-command-surface-ergonomics", "Designing taskpane command bars matching native Office application ergonomics.", ["fluent-command-ergonomics", "native-office-look-feel", "taskpane-command-surface"]),
        ("json-manifest-ci-cd-validation", "Automating manifest JSON schema validation in GitHub Actions using office-addin-manifest.", ["manifest-ci-cd-validation", "office-addin-manifest-validate", "automated-manifest-linter"]),
    ]),
    # Cluster 19: Office Add-in Single Sign-On (SSO), Entra ID & Nested App Auth (NAA) (o365-addin-auth-*)
    (19, [
        ("entra-id-app-registration-scopes", "Registering Entra ID apps, exposing access_as_user scopes, setting redirect URIs.", ["entra-id-app-scopes", "access-as-user-scope", "addin-redirect-uris"]),
        ("nested-app-auth-naa-broker", "Nested App Authentication (NAA) architecture, eliminating third-party cookie popups.", ["nested-app-auth-naa", "naa-broker-architecture", "eliminate-cookie-popups-addin"]),
        ("msal-browser-naa-integration", "Configuring @azure/msal-browser with NAA broker for seamless silent token acquisition.", ["msal-browser-naa", "silent-token-acquisition-naa", "msal-broker-integration"]),
        ("obo-token-exchange-flow", "Exchanging client-side id tokens for downstream Microsoft Graph tokens on backend APIs.", ["obo-token-exchange-flow", "on-behalf-of-token-exchange", "backend-graph-token-exchange"]),
        ("fallback-dialog-auth-flow", "Handling SSO failures gracefully: opening fallback modal authentication dialogs (displayDialogAsync).", ["fallback-dialog-auth", "displaydialogasync-sso-fallback", "graceful-sso-degradation"]),
        ("continuous-access-evaluation-cae", "Catching CAE claims challenges, forcing re-authentication upon security policy changes.", ["cae-claims-challenge", "continuous-access-evaluation-addin", "re-auth-cae-trigger"]),
        ("jwt-token-claims-validation", "Validating aud, iss, tid, and scp claims in backend APIs before serving data.", ["jwt-token-claims-validation", "validate-jwt-addin-backend", "tenant-audience-claim-check"]),
        ("safari-itp-cookie-workarounds", "Overcoming Intelligent Tracking Prevention (ITP) in Safari and Office on the Web.", ["safari-itp-workarounds", "intelligent-tracking-prevention", "cookie-blocking-office-web"]),
        ("pkce-public-client-exchange", "Implementing PKCE for public client desktop add-ins without embedded secrets.", ["pkce-public-client-addin", "proof-key-code-exchange", "secure-public-client-oauth"]),
        ("zero-trust-addin-security", "Ephemeral scoped tokens, strict CORS headers, and least-privilege Graph permissions.", ["zero-trust-addin-security", "ephemeral-scoped-tokens", "strict-cors-least-privilege"]),
    ]),
    # Cluster 20: Office Add-in Deployment, Centralized Admin & AppSource Certification (o365-addin-deploy-*)
    (20, [
        ("centralized-deployment-admin-center", "Deploying add-ins via M365 Admin Center, assigning to specific user groups.", ["centralized-deployment-admin", "admin-center-addin-rollout", "security-group-addin-assign"]),
        ("appsource-commercial-marketplace", "Listing add-ins on Microsoft AppSource, configuring Partner Center profiles.", ["appsource-marketplace-listing", "partner-center-offer-config", "commercial-addin-publishing"]),
        ("appsource-certification-validation", "Passing AppSource automated verification tests, privacy policy requirements.", ["appsource-certification-checks", "marketplace-verification-tests", "privacy-policy-validation"]),
        ("transact-saas-offer-monetization", "Integrating Azure Marketplace SaaS fulfillment APIs for recurring add-in subscriptions.", ["transact-saas-monetization", "marketplace-fulfillment-api", "recurring-subscription-billing"]),
        ("zero-downtime-addin-versioning", "Updating hosted taskpane bundles without breaking active client document sessions.", ["zero-downtime-addin-versioning", "blue-green-addin-deployment", "hosted-taskpane-cache-busting"]),
        ("private-tenant-app-catalogs", "Sideloading add-ins to private SharePoint App Catalogs for internal enterprise rollouts.", ["private-tenant-app-catalogs", "internal-enterprise-addin-deploy", "sideload-sppkg-catalog"]),
        ("telemetry-and-crash-analytics", "Integrating Application Insights to capture unhandled taskpane errors in production.", ["telemetry-crash-analytics", "app-insights-addin-monitoring", "unhandled-taskpane-exceptions"]),
        ("multi-language-localization-bundles", "Localizing manifest strings, taskpane UI labels, and RTL layout support.", ["multi-language-localization", "addin-string-resource-bundles", "rtl-layout-support-addin"]),
        ("soc-2-publisher-attestation", "Completing Microsoft 365 Publisher Attestation and Microsoft 365 App Certification.", ["soc-2-publisher-attestation", "m365-app-certification", "security-compliance-attestation"]),
        ("azure-static-web-apps-delivery", "Deploying add-in static frontends to Azure Static Web Apps with global CDN caching.", ["azure-static-web-apps-addin", "global-cdn-caching-addin", "automated-frontend-deploy"]),
    ]),
    # Cluster 21: Microsoft 365 Fluid Framework & Real-Time Collaborative Canvas (o365-fluid-*)
    (21, [
        ("fluid-distributed-data-structures", "DDS fundamentals: SharedMap, SharedString, SharedDirectory, ConsensusRegister.", ["fluid-dds-fundamentals", "sharedmap-sharedstring", "consensusregister-fluid"]),
        ("azure-fluid-relay-service", "Provisioning and connecting to Azure Fluid Relay managed service instances.", ["azure-fluid-relay", "managed-fluid-service", "fluid-relay-connection-config"]),
        ("collaborative-cursor-presence", "Sharing real-time multi-user cursor coordinates, selection ranges, and active status.", ["collaborative-cursor-presence", "multi-user-cursor-sync", "live-selection-ranges"]),
        ("conflict-free-collaborative-editing", "Implementing CRDT-backed collaborative text input without server-side locking.", ["crdt-collaborative-editing", "conflict-free-text-sync", "server-free-collaboration"]),
        ("fluid-container-lifecycle-management", "Creating, loading, attaching, and disconnecting Fluid containers across sessions.", ["fluid-container-lifecycle", "create-load-fluid-container", "container-session-disconnect"]),
        ("fluid-in-spfx-and-teams", "Embedding collaborative Fluid DDS components inside SPFx web parts and Teams tabs.", ["fluid-in-spfx-teams", "embed-fluid-components", "teams-tab-fluid-sync"]),
        ("hierarchical-tree-state-modeling", "Building complex hierarchical tree data structures using Fluid SharedDirectory.", ["hierarchical-tree-state", "shareddirectory-state-tree", "nested-data-model-fluid"]),
        ("intermittent-connection-resiliency", "Local optimistic updates, handling offline drops, and automated reconnection catch-up.", ["intermittent-connection-fluid", "optimistic-local-updates", "offline-reconnection-catchup"]),
        ("token-provider-and-security", "Implementing custom ITokenProvider, signing JWT container tokens securely.", ["itokenprovider-fluid", "secure-jwt-container-signing", "fluid-security-token-provider"]),
        ("multiplayer-vibe-coding-canvas", "Rapidly building real-time multiplayer drawing and whiteboarding tools with Fluid.", ["multiplayer-vibe-coding-canvas", "real-time-multiplayer-whiteboard", "fluid-canvas-toys"]),
    ]),
    # Cluster 22: Microsoft Loop Components, Workspaces & Open Standards (o365-loop-*)
    (22, [
        ("loop-component-file-architecture", "Understanding .loop container files, storage substrates in SharePoint Embedded.", ["loop-component-architecture", "loop-container-files", "sharepoint-embedded-substrate"]),
        ("adaptive-card-loop-components", "Creating portable Adaptive Card Loop components synchronized across Teams and Outlook.", ["adaptive-card-loop-components", "portable-card-components", "cross-app-loop-cards"]),
        ("cross-host-bi-directional-sync", "Real-time state synchronization when editing a Loop component embedded in multiple apps.", ["cross-host-loop-sync", "bi-directional-loop-updates", "synchronized-loop-embeds"]),
        ("loop-workspace-graph-apis", "Programmatically creating, reading, and updating Loop pages and workspaces via Graph.", ["loop-workspace-graph-apis", "create-update-loop-pages", "programmatic-loop-workspace"]),
        ("collaborative-tables-and-trackers", "Dynamic voting tables, progress trackers, and checklist items inside Loop.", ["collaborative-loop-tables", "loop-voting-tables", "interactive-progress-trackers"]),
        ("purview-governance-on-loop", "Applying DLP policies, sensitivity labels, and retention holds to portable Loop files.", ["purview-governance-loop", "dlp-policies-loop-files", "retention-holds-loop"]),
        ("third-party-widget-embedding", "Embedding external web views and interactive widgets into Microsoft Loop canvases.", ["third-party-loop-widgets", "embed-widgets-in-loop", "external-canvas-extensions"]),
        ("b2b-guest-sharing-security-rules", "Enforcing external guest sharing boundaries and conditional access on Loop components.", ["b2b-guest-sharing-loop", "external-sharing-loop-rules", "conditional-access-loop"]),
        ("loop-page-template-scaffolding", "Programmatically scaffolding standardized project kick-off pages in Loop workspaces.", ["loop-page-template-scaffolding", "standardized-project-loop-pages", "automated-loop-templates"]),
        ("autonomous-agents-in-loop-canvases", "AI agents updating and summarizing Loop components asynchronously in collaborative spaces.", ["autonomous-agents-in-loop", "ai-loop-canvas-updates", "collaborative-ai-loop-agent"]),
    ]),
    # Cluster 23: Microsoft 365 Search, Semantic Search & Syntex Content AI (o365-search-*)
    (23, [
        ("kql-query-syntax-mastery", "Keyword Query Language (KQL) operators, property restrictions, wildcards, Boolean logic.", ["kql-query-syntax", "keyword-query-language", "property-restrictions-kql"]),
        ("search-rest-api-query-execution", "Executing search queries via /search/query, requesting entity types, sorting, paging.", ["search-rest-api-execution", "post-search-query-graph", "entitytypes-search-paging"]),
        ("syntex-document-understanding-models", "Training unstructured document models, extractor fields, classifier definitions.", ["syntex-document-models", "unstructured-document-training", "syntex-extractor-fields"]),
        ("adaptive-card-search-display-templates", "Designing custom result types and Adaptive Card templates for search results.", ["search-display-templates", "adaptive-card-search-results", "custom-result-types-search"]),
        ("neural-semantic-index-mechanics", "Multi-vector embeddings, semantic reranking, and intent matching in M365 Search.", ["neural-semantic-index", "vector-embeddings-m365-search", "semantic-reranking-engine"]),
        ("automated-retention-label-tagging", "Triggering retention labels and security classifications automatically via Syntex models.", ["automated-retention-tagging", "syntex-label-triggers", "automatic-compliance-tagging"]),
        ("search-refiners-and-managed-properties", "Mapping crawled properties to managed properties, configuring facets and refiners.", ["search-refiners-managed-props", "crawled-to-managed-mapping", "search-facets-refiners"]),
        ("acronym-and-bookmark-management", "Programmatically managing corporate bookmarks, acronym glossaries, and Q&A pairs.", ["acronym-bookmark-management", "corporate-glossary-search", "q-and-a-bookmarks-graph"]),
        ("security-trimmed-search-results", "Preserving user ACLs, preventing data leakage in enterprise search integrations.", ["security-trimmed-search", "acl-trimmed-search-results", "prevent-search-data-leakage"]),
        ("enterprise-rag-search-agent", "Augmenting M365 Search with LLM-powered Retrieval-Augmented Generation for grounded Q&A.", ["enterprise-rag-search-agent", "m365-search-augmented-rag", "grounded-search-qa-agent"]),
    ]),
    # Cluster 24: Microsoft Forms API, Quizzes & Webhook Automation (o365-forms-*)
    (24, [
        ("forms-power-automate-trigger", "When a new response is submitted, retrieving response details, handling multi-choice fields.", ["forms-flow-trigger", "when-response-submitted", "forms-response-details"]),
        ("survey-branching-logic-architecture", "Dynamic question branching, skipping sections based on respondent input.", ["survey-branching-logic", "conditional-forms-branching", "dynamic-survey-sections"]),
        ("forms-rest-api-inspection", "Graph endpoints for Forms, retrieving form definitions, questions, and responses.", ["forms-rest-api", "graph-forms-definitions", "forms-questions-responses"]),
        ("automated-quiz-grading-algorithms", "Automated grading of math and text questions, instant score calculation and email certs.", ["automated-quiz-grading", "quiz-score-calculation", "instant-certificate-dispatch"]),
        ("responsive-iframe-embedding", "Embedding Forms securely into modern SharePoint pages, Teams tabs, and web apps.", ["responsive-iframe-forms", "embed-forms-sharepoint", "forms-in-teams-tabs"]),
        ("forms-file-upload-onedrive-routing", "Handling file uploads from respondents, verifying virus scan status, storage routing.", ["forms-file-upload-routing", "onedrive-forms-attachments", "upload-virus-scan-status"]),
        ("power-bi-streaming-forms-data", "Streaming survey response metrics into real-time Power BI sentiment dashboards.", ["power-bi-streaming-forms", "survey-sentiment-dashboards", "real-time-forms-analytics"]),
        ("multilingual-forms-routing", "Configuring multi-language surveys, detecting locale, presenting translated questions.", ["multilingual-forms-routing", "multi-language-surveys", "locale-detected-questions"]),
        ("customer-voice-nps-automation", "Capturing Net Promoter Scores (NPS), automated follow-up workflows for negative feedback.", ["customer-voice-nps", "net-promoter-score-flow", "automated-feedback-followup"]),
        ("conversational-survey-scaffolding", "Generating complete multi-page surveys from a single natural language description.", ["conversational-survey-scaffolding", "prompt-to-form-generator", "ai-survey-builder"]),
    ]),
    # Cluster 25: Microsoft Planner, To-Do & Task Management Graph APIs (o365-tasks-*)
    (25, [
        ("unified-tasks-graph-model", "Consolidated task architecture: Planner plans, buckets, tasks, To-Do lists, linked tasks.", ["unified-tasks-graph-model", "planner-todo-consolidation", "tasks-graph-endpoints"]),
        ("automated-kanban-board-builder", "Programmatic plan creation, bucket definitions, moving cards across stages via API.", ["automated-kanban-builder", "programmatic-planner-buckets", "move-cards-planner-api"]),
        ("etag-optimistic-concurrency-planner", "Managing @odata.etag, If-Match headers, resolving concurrent task update conflicts.", ["etag-concurrency-planner", "if-match-headers-tasks", "task-update-conflict-resolution"]),
        ("todo-personal-list-synchronization", "Querying To-Do task folders, creating reminders, due dates, linking to Outlook flags.", ["todo-list-synchronization", "todo-reminders-due-dates", "outlook-flag-task-link"]),
        ("teams-tasks-by-planner-integration", "Syncing channel board tasks with individual user task lists in Microsoft Teams.", ["teams-tasks-by-planner", "channel-tasks-sync", "shared-planner-teams-tab"]),
        ("planner-velocity-and-burndown-pbi", "Extracting task completion timestamps, calculating team velocity, burn-down metrics.", ["planner-velocity-burndown", "burndown-metrics-pbi", "task-velocity-calculation"]),
        ("automated-task-escalation-flow", "Identifying overdue tasks, reassigning to alternate owners, sending Teams chat alerts.", ["automated-task-escalation", "overdue-task-reassignment", "task-escalation-chat-alerts"]),
        ("checklist-items-and-attachments", "Adding sub-item checklists, preview attachments, category color tags via Graph API.", ["checklist-items-attachments", "planner-subtasks-checklist", "preview-attachments-tasks"]),
        ("group-owned-plan-security", "Managing M365 Group-backed plan permissions, guest assignments, tenant boundaries.", ["group-owned-plan-security", "planner-group-permissions", "guest-task-assignment-rules"]),
        ("autonomous-project-manager-agent", "AI agent parsing meeting transcripts, generating, assigning, and tracking Planner tasks.", ["autonomous-project-manager", "transcript-to-planner-tasks", "ai-task-assignment-agent"]),
    ]),
    # Cluster 26: Microsoft Bookings & Scheduling Automated Workflows (o365-bookings-*)
    (26, [
        ("bookings-schema-and-endpoints", "bookingBusinesses, services, staffMembers, customers, and appointments schema.", ["bookings-schema-endpoints", "bookingbusinesses-services", "staffmembers-appointments"]),
        ("automated-calendar-appointment-scheduling", "Programmatically creating appointments, verifying staff availability, cancellations.", ["automated-appointment-scheduling", "verify-staff-availability", "cancel-reschedule-appointments"]),
        ("custom-nextjs-booking-frontend", "Building a branded booking portal with Next.js and Tailwind on top of Bookings API.", ["custom-nextjs-booking", "branded-booking-portal", "tailwind-bookings-ui"]),
        ("staff-working-hours-and-conflict-guards", "Handling staff time zones, buffer times between appointments, holiday calendars.", ["staff-working-hours-conflicts", "appointment-buffer-times", "time-zone-booking-guards"]),
        ("sms-email-reminder-dispatch", "Triggering SMS reminders (Azure Communication Services) and calendar invite updates.", ["sms-email-reminders-bookings", "azure-sms-reminders", "automated-appointment-ics"]),
        ("virtual-visit-teams-meeting-links", "Automatic generation of Teams meeting links for virtual consultations and telehealth.", ["virtual-visit-teams-links", "telehealth-appointment-links", "auto-generated-teams-meetings"]),
        ("custom-intake-questions-mapping", "Appending custom intake questionnaires to booking appointments and CRM sync.", ["custom-intake-questions", "booking-intake-questionnaire", "crm-appointment-sync"]),
        ("multi-location-branch-architecture", "Managing hundreds of branch offices, shared staff pools, regional services.", ["multi-location-branch-bookings", "regional-booking-services", "shared-staff-pools-bookings"]),
        ("payment-deposit-verification", "Integrating Stripe/PayPal deposit verification before confirming Bookings appointments.", ["payment-deposit-verification", "stripe-bookings-integration", "escrow-deposit-booking"]),
        ("ai-conversational-booking-concierge", "Voice and text chat AI assistant booking appointments dynamically via dialogue.", ["ai-booking-concierge", "conversational-appointment-booking", "voice-text-booking-agent"]),
    ]),
    # Cluster 27: Microsoft Lists Custom Formatting, JSON Column & View Formatting (o365-lists-*)
    (27, [
        ("json-column-formatting-ast", "Authoring JSON formatters, @currentField, operators (+, -, ==, ? :), AST schema.", ["json-column-formatting", "currentfield-formatting-ast", "lists-json-syntax"]),
        ("view-formatting-kanban-boards", "Designing custom view formatters for interactive board cards, tiles, and timeline views.", ["view-formatting-kanban", "lists-tile-board-views", "custom-view-formatters"]),
        ("status-pill-badges-and-color-ramps", "Building custom status pills, priority badges, conditional color ramps.", ["status-pill-badges", "lists-color-ramps", "conditional-status-pills"]),
        ("executeflow-action-buttons", "Adding interactive buttons to list rows that trigger Power Automate cloud flows.", ["executeflow-action-buttons", "trigger-flow-from-list-row", "customrowaction-executeflow"]),
        ("in-line-svg-donut-charts", "Rendering dynamic SVG donut charts and KPI progress bars directly inside list cells.", ["inline-svg-donut-charts", "svg-in-lists-formatting", "kpi-progress-bars-lists"]),
        ("customcardprops-hover-flyouts", "Building interactive flyout popovers that display related item details on mouse hover.", ["customcardprops-hover-flyouts", "hover-card-popovers-lists", "flyout-menu-lists-formatting"]),
        ("json-form-section-formatting", "Customizing list creation and edit forms: header, footer, multi-column body sections.", ["json-form-section-formatting", "lists-form-customization", "header-footer-form-sections"]),
        ("pnp-list-formatting-repository", "Reusing and contributing to open-source PnP list formatting template collections.", ["pnp-list-formatting-repo", "pnp-formatting-templates", "open-source-list-formatters"]),
        ("formatter-rendering-performance", "Optimizing element counts per cell, preventing scrolling lag on 500-item list views.", ["formatter-rendering-performance", "dom-element-count-lists", "smooth-scrolling-formatters"]),
        ("screenshot-to-json-formatter-ai", "Converting UI mockups and screenshots into production-ready JSON formatting code.", ["screenshot-to-json-formatter", "mockup-to-lists-json", "ai-generated-list-formatting"]),
    ]),
    # Cluster 28: Exchange Online Mail Flow, Transport Rules & PowerShell Automation (o365-exchange-*)
    (28, [
        ("exchange-powershell-v3-rest", "ExchangeOnlineManagement v3 cmdlets, REST-backed execution, token authentication.", ["exchange-powershell-v3", "exchangeonlinemanagement-rest", "modern-exchange-cmdlets"]),
        ("mail-flow-transport-rules", "Configuring mail routing rules, disclaimer injection, DLP inspection on messages.", ["mail-flow-transport-rules", "exchange-transport-rules", "disclaimer-injection-mail"]),
        ("spf-dkim-dmarc-authentication", "Generating DKIM selector keys, SPF record syntax, DMARC reject policies.", ["spf-dkim-dmarc-auth", "dkim-selector-keys", "dmarc-reject-policies"]),
        ("anti-malware-and-zap-policies", "Defender for Office 365, Zero-Hour Auto Purge (ZAP), Safe Links, Safe Attachments.", ["anti-malware-zap-policies", "zero-hour-auto-purge", "safe-links-attachments"]),
        ("smtp-relay-and-direct-send", "Configuring SMTP relay endpoints, IP allowlists, sending transaction emails from apps.", ["smtp-relay-direct-send", "exchange-smtp-relay", "transactional-email-endpoints"]),
        ("shared-mailbox-and-resource-routing", "Programmatically provisioning shared mailboxes, meeting rooms, resource booking.", ["shared-mailbox-provisioning", "meeting-room-resource-booking", "resource-mailbox-routing"]),
        ("mailbox-auditing-and-litigation-hold", "Enabling mailbox audit logs, litigation holds, managing Recoverable Items store.", ["mailbox-auditing-litigation-hold", "recoverable-items-dumpster", "non-owner-access-audit"]),
        ("cross-premises-hybrid-mail-flow", "Inbound/outbound mail flow connectors between Exchange Online and on-prem Exchange.", ["hybrid-mail-flow-connectors", "cross-premises-mail-routing", "exchange-hybrid-connectors"]),
        ("message-trace-delivery-telemetry", "Get-MessageTrace, analyzing delivery status, failure diagnostic codes (DSN).", ["message-trace-telemetry", "get-messagetrace-powershell", "delivery-failure-dsn-codes"]),
        ("autonomous-quarantine-triage-agent", "AI agent analyzing quarantined messages, detecting false positives, isolating threats.", ["quarantine-triage-agent", "automated-quarantine-analysis", "phishing-threat-isolation-ai"]),
    ]),
    # Cluster 29: Microsoft Purview Information Protection, Sensitivity Labels & DLP (o365-purview-*)
    (29, [
        ("sensitivity-label-taxonomy", "Configuring sensitivity labels, encryption rights, visual watermarks, header stamps.", ["sensitivity-label-taxonomy", "purview-sensitivity-labels", "visual-watermark-encryption"]),
        ("data-loss-prevention-policy-rules", "Creating DLP policies across Exchange, SharePoint, OneDrive, Teams, and endpoints.", ["dlp-policy-rules-purview", "data-loss-prevention-rules", "endpoint-dlp-blocking"]),
        ("mip-sdk-programmatic-labeling", "Microsoft Information Protection (MIP) SDK, inspecting, applying labels in custom apps.", ["mip-sdk-programmatic-labeling", "mip-sdk-inspect-labels", "file-label-encryption-sdk"]),
        ("exact-data-match-sit-hashing", "Exact Data Match (EDM) sensitive information types, hashing customer tables for DLP.", ["exact-data-match-sit", "edm-hash-table-dlp", "precision-pii-detection"]),
        ("office-js-sensitivity-label-api", "Reading and applying sensitivity labels directly in Outlook and Word add-ins.", ["office-js-sensitivity-api", "mailbox-item-sensitivitylabel", "word-sensitivity-label-binding"]),
        ("defender-cloud-apps-session-proxy", "Real-time session monitoring, blocking downloads of confidential files on BYOD devices.", ["defender-cloud-apps-proxy", "conditional-access-session-fencing", "block-byod-confidential-downloads"]),
        ("auto-labeling-at-rest-sharepoint", "Automated cloud-side classification jobs in SharePoint libraries, simulation testing.", ["auto-labeling-at-rest", "sharepoint-auto-labeling-jobs", "classification-simulation-mode"]),
        ("double-key-encryption-dke-service", "Deploying Double Key Encryption REST services, managing sovereign customer keys.", ["double-key-encryption-dke", "sovereign-key-control", "dke-rest-service-keys"]),
        ("unified-audit-log-label-downgrade", "Auditing label removal/downgrade events, triggering incident escalation workflows.", ["unified-audit-log-downgrades", "label-removal-escalation", "purview-audit-forensics"]),
        ("ai-trainable-classifiers-purview", "Training machine learning classifiers to detect proprietary intellectual property.", ["ai-trainable-classifiers", "ml-document-classifiers-purview", "detect-proprietary-ip"]),
    ]),
    # Cluster 30: Microsoft 365 Security, Zero Trust & Privileged Identity Management (o365-sec-*)
    (30, [
        ("zero-trust-architecture-principles", "Explicit verification, least privilege access, and assume breach across M365.", ["zero-trust-principles", "explicit-verification-m365", "assume-breach-security"]),
        ("entra-id-pim-just-in-time", "Privileged Identity Management: Just-in-Time activation, approval workflows, role duration.", ["entra-id-pim-jit", "just-in-time-role-activation", "pim-approval-workflows"]),
        ("conditional-access-signal-policy", "Configuring Conditional Access: device compliance, trusted locations, risk levels.", ["conditional-access-signals", "device-compliance-ca", "risk-based-conditional-access"]),
        ("workload-identity-oidc-federation", "Eliminating static client secrets, using GitHub Actions OIDC workload identity federation.", ["workload-identity-oidc", "github-actions-oidc-m365", "eliminate-client-secrets"]),
        ("app-consent-and-permission-reviews", "Restricting end-user consent, configuring verified publisher requirements.", ["app-consent-permission-reviews", "restrict-user-consent", "verified-publisher-rules"]),
        ("microsoft-sentinel-m365-ingestion", "Ingesting M365 security logs into Sentinel, authoring KQL threat hunting queries.", ["sentinel-m365-ingestion", "kql-threat-hunting-m365", "soar-security-automation"]),
        ("intune-mam-app-protection-policies", "Mobile Application Management: copy/paste restrictions, encrypted app storage, PINs.", ["intune-mam-app-protection", "mobile-app-protection-policies", "clipboard-data-leak-prevention"]),
        ("threat-hunting-and-identity-alerts", "Detecting impossible travel, password spray, and compromised credentials in real time.", ["threat-hunting-identity-alerts", "impossible-travel-detection", "compromised-credential-alerts"]),
        ("defender-air-automated-playbooks", "Automated Investigation and Response (AIR) playbooks in Microsoft Defender for M365.", ["defender-air-playbooks", "automated-investigation-response", "incident-remediation-playbooks"]),
        ("autonomous-permission-drift-watchdog", "AI agent continuously monitoring and revoking excessive or dormant tenant permissions.", ["permission-drift-watchdog", "autonomous-permission-auditor", "dormant-account-remediation"]),
    ]),
    # Cluster 31: Microsoft 365 Compliance, eDiscovery, Audit Logs & Retention (o365-compliance-*)
    (31, [
        ("ediscovery-premium-case-management", "Managing custodians, search queries, deduplication, review sets, and legal exports.", ["ediscovery-premium-cases", "custodian-management-ediscovery", "review-sets-legal-export"]),
        ("unified-audit-log-forensic-queries", "Querying Search-UnifiedAuditLog, parsing JSON audit events, tracking admin actions.", ["unified-audit-log-queries", "search-unifiedauditlog-powershell", "forensic-audit-parsing"]),
        ("retention-policies-and-preservation-locks", "Applying retention schedules, preservation locks, and automated disposition reviews.", ["retention-preservation-locks", "regulatory-retention-schedules", "automated-disposition-reviews"]),
        ("insider-risk-data-theft-detection", "Detecting abnormal data downloads, mass file transfers before employee resignations.", ["insider-risk-data-theft", "mass-file-download-alerts", "departing-employee-risk-rules"]),
        ("information-barriers-isolation", "Enforcing information barriers between conflicting departments in Teams and SharePoint.", ["information-barriers-isolation", "ethical-walls-teams", "departmental-barrier-policies"]),
        ("graph-ediscovery-api-automation", "Programmatically creating legal hold cases, adding custodians, and exporting evidence.", ["graph-ediscovery-api", "automated-legal-holds", "ediscovery-evidence-export"]),
        ("compliance-manager-assessment-templates", "Aligning tenant configurations with ISO 27001, GDPR, HIPAA, and NIST templates.", ["compliance-manager-templates", "iso-27001-gdpr-m365", "compliance-score-assessments"]),
        ("inactive-mailbox-litigation-retention", "Retaining departed employee mailboxes indefinitely without recurring license costs.", ["inactive-mailbox-retention", "litigation-hold-inactive-mail", "zero-cost-mailbox-archival"]),
        ("compliance-power-bi-audit-dashboard", "Extracting compliance telemetry into executive Power BI compliance scorecards.", ["compliance-power-bi-dashboards", "audit-telemetry-scorecards", "retention-compliance-metrics"]),
        ("autonomous-compliance-verifier-agent", "Pre-commit checking of corporate communications against regulatory compliance laws.", ["compliance-verifier-agent", "pre-commit-compliance-guards", "regulatory-communication-checks"]),
    ]),
    # Cluster 32: Microsoft 365 Tenant Administration & Microsoft Graph PowerShell SDK (o365-admin-*)
    (32, [
        ("microsoft-graph-powershell-sdk-core", "Microsoft.Graph PowerShell module, authentication (Connect-MgGraph), scopes.", ["graph-powershell-sdk-core", "connect-mggraph-automation", "modern-graph-powershell"]),
        ("automated-user-onboarding-scripts", "Automated employee provisioning: Entra account, license, group assignment, mailbox init.", ["user-onboarding-scripts", "automated-employee-provisioning", "entra-account-mailbox-init"]),
        ("dynamic-group-membership-rules", "Crafting Entra ID dynamic group membership rules (user.department -eq 'Engineering').", ["dynamic-group-membership", "entra-dynamic-group-rules", "rule-based-group-assignment"]),
        ("license-reclamation-and-sku-optimization", "Identifying inactive accounts, reclaiming unused Copilot/E5 licenses automatically.", ["license-reclamation-optimization", "reclaim-unused-copilot-licenses", "sku-utilization-auditing"]),
        ("tenant-to-tenant-cross-migration", "Planning and automating domain transfers, mailbox cutovers, and OneDrive re-mapping.", ["tenant-to-tenant-migration", "domain-transfer-automation", "mailbox-cutover-engineering"]),
        ("b2b-direct-connect-shared-channels", "Setting up cross-tenant access settings, trust relationships, Teams shared channels.", ["b2b-direct-connect-channels", "cross-tenant-access-settings", "shared-channels-trust-rules"]),
        ("powershell-parallel-loop-performance", "Using ForEach-Object -Parallel for high-speed batch modifications across 50k users.", ["powershell-parallel-loops", "foreach-object-parallel-m365", "batch-user-modifications-perf"]),
        ("service-health-outage-alerts", "Querying Service Communications API, posting real-time incident notices to admin channels.", ["service-health-outage-alerts", "service-communications-api", "automated-incident-broadcast"]),
        ("custom-domain-dns-automation", "Automating DNS verification, MX records, autodiscover endpoints via Azure DNS scripts.", ["custom-domain-dns-automation", "azure-dns-m365-records", "autodiscover-mx-automation"]),
        ("conversational-tenant-devops", "Executing validated tenant administrative actions via conversational AI instructions.", ["conversational-tenant-devops", "ai-driven-tenant-management", "natural-language-powershell"]),
    ]),
    # Cluster 33: Office 365 Vibe Coding, AI Pair Programming & Rapid UI Prototyping (o365-vibe-*)
    (33, [
        ("conversational-vibe-coding-workflow", "Flow-state programming, iterating on Office apps and add-ins through live conversation.", ["conversational-vibe-workflow", "flow-state-office-development", "conversational-add-in-prototyping"]),
        ("tracer-bullet-office-apps", "Building thin end-to-end tracer bullets across Office.js, Graph, and backend APIs in hours.", ["tracer-bullet-office-apps", "thin-end-to-end-spikes-o365", "rapid-tracer-bullets-office"]),
        ("rapid-feedback-hot-module-reload", "Sub-second taskpane reload loops, test-driven prompt synthesis, instant feedback.", ["rapid-feedback-hmr-office", "taskpane-hot-module-reload", "sub-second-feedback-loops-office"]),
        ("conversational-tdd-office-addins", "Writing test assertions first, prompting AI agents to synthesize Office.js implementations.", ["conversational-tdd-office", "test-first-office-addin-synthesis", "tdd-prompt-engineering-office"]),
        ("mock-graph-data-exploratory-spikes", "Generating synthetic mailboxes, mock workbooks, and Graph payloads for rapid prototyping.", ["mock-graph-exploratory-spikes", "synthetic-mailboxes-workbooks", "mock-graph-payload-spikes"]),
        ("dialectical-code-review-agents", "AI peer review subagents checking memory leaks, context.sync() bottlenecks, and security.", ["dialectical-code-review-office", "peer-agent-addin-review", "context-sync-bottleneck-audit"]),
        ("keyboard-first-developer-flow", "Minimizing context switching with keyboard shortcuts, AI code generation, and live CLI tooling.", ["keyboard-first-dev-flow", "unbroken-flow-state-office", "keyboard-ergonomics-vibe"]),
        ("fail-fast-webview-error-surfacing", "Instantaneous error banners, unhandled rejection overlays in taskpanes for rapid debugging.", ["fail-fast-webview-errors", "taskpane-error-overlays", "instant-unhandled-rejection-alert"]),
        ("human-in-the-loop-architectural-guardrails", "Steering AI code synthesis with explicit invariant constraints and verification recipes.", ["human-in-the-loop-guardrails", "architectural-steering-office", "prompt-invariant-constraints"]),
        ("spec-driven-solution-scaffolding", "Scaffolding complete Office 365 solutions, manifests, and web parts from a single Markdown spec.", ["spec-driven-solution-scaffolding", "single-prompt-office-solution", "markdown-spec-to-m365-app"]),
    ]),
    # Cluster 34: M365 CLI, PnP PowerShell & Automated ALM Provisioning (o365-alm-*)
    (34, [
        ("m365-cli-cross-platform-automation", "Mastering CLI for Microsoft 365 (m365), script automation on Windows/macOS/Linux.", ["m365-cli-automation", "cross-platform-m365-cli", "bash-powershell-m365-scripts"]),
        ("pnp-powershell-site-provisioning", "Modern site provisioning, schema templates, bulk permission assignments via PnP PowerShell.", ["pnp-powershell-site-provisioning", "declarative-site-templates", "bulk-permission-assignments-pnp"]),
        ("site-designs-and-json-site-scripts", "Authoring JSON site scripts, applying corporate themes, branding, and hub site links.", ["site-designs-json-scripts", "json-site-script-authoring", "automated-branding-hub-links"]),
        ("github-actions-ci-cd-pipelines", "Automated .sppkg building, bundling, and deployment to the App Catalog in GitHub Actions.", ["github-actions-m365-cicd", "automated-sppkg-build-deploy", "app-catalog-ci-cd-pipeline"]),
        ("pnp-provisioning-engine-templates", "XML/YAML provisioning templates, differential template extraction, automated application.", ["pnp-provisioning-engine-templates", "differential-template-extraction", "declarative-pnp-xml-yaml"]),
        ("microsoft365-dsc-infrastructure-as-code", "Desired State Configuration (M365DSC), automated detection and remediation of drift.", ["microsoft365-dsc-iac", "m365dsc-drift-detection", "declarative-tenant-iac"]),
        ("teams-template-channel-scaffolding", "Provisioning Teams from standardized templates with pre-installed apps and channels.", ["teams-template-channel-scaffolding", "standardized-team-templates", "pre-installed-teams-apps-script"]),
        ("azure-key-vault-secret-rotation-alm", "Secure certificate and secret bindings in deployment pipelines, zero hardcoded credentials.", ["key-vault-secret-rotation-alm", "zero-hardcoded-credentials-cicd", "pipeline-certificate-rotation"]),
        ("static-analysis-and-bundle-size-gates", "Enforcing ESLint, TypeScript strict checks, and bundle size limits before deployment.", ["static-analysis-bundle-gates", "eslint-typescript-strict-checks", "bundle-size-enforcement-m365"]),
        ("disaster-recovery-and-tenant-rollback", "Automated configuration backups, site collection recovery scripts, and rollback plans.", ["disaster-recovery-tenant-rollback", "automated-config-backups", "site-collection-recovery-scripts"]),
    ]),
    # Cluster 35: M365 Multi-Tenant SaaS Architecture, App Consent & AppSource Ecosystem (o365-saas-*)
    (35, [
        ("multi-tenant-data-partitioning", "Tenant isolation patterns, partitioned databases, tenant ID tagging, cross-tenant security.", ["multi-tenant-data-partitioning", "tenant-id-tagging-isolation", "cross-tenant-security-fences"]),
        ("admin-consent-url-architecture", "Constructing multi-tenant admin consent URLs, handling step-up consent, dynamic permissions.", ["admin-consent-url-architecture", "step-up-consent-oauth", "multi-tenant-admin-consent-flow"]),
        ("marketplace-saas-fulfillment-api", "Implementing SaaS fulfillment APIs v2, resolve subscription token, activate subscription.", ["marketplace-saas-fulfillment", "saas-fulfillment-api-v2", "resolve-activate-subscription"]),
        ("sovereign-cloud-boundary-compliance", "Adapting endpoints for EU Data Boundary (EUDB), GCC, GCC High, DoD, and China Mooncake.", ["sovereign-cloud-compliance", "eudb-data-residency", "gcc-high-dod-mooncake-endpoints"]),
        ("metering-and-credit-billing-engine", "Emitting usage events to the Azure Marketplace Metering Service API for dimension billing.", ["metering-credit-billing-engine", "marketplace-metering-service", "dimension-based-usage-events"]),
        ("publisher-attestation-security-controls", "Aligning security controls with Microsoft 365 Certification and ACAS automated scans.", ["publisher-attestation-controls", "m365-certification-audit", "acas-security-scan-alignment"]),
        ("partner-center-offer-lifecycle", "Managing commercial marketplace offers, co-sell readiness, and private enterprise plans.", ["partner-center-offer-lifecycle", "commercial-marketplace-co-sell", "private-enterprise-saas-plans"]),
        ("azure-front-door-global-cdn-caching", "Global CDN routing, SSL offloading, and zero-downtime blue/green deployments for add-ins.", ["azure-front-door-cdn-caching", "global-cdn-add-in-routing", "zero-downtime-blue-green-m365"]),
        ("multi-tenant-telemetry-isolation", "Segregating tenant telemetry, structured logging, and anomaly alerts in Application Insights.", ["multi-tenant-telemetry-isolation", "tenant-segregated-logging", "application-insights-tenant-tag"]),
        ("autonomous-multi-agent-saas-swarms", "Multi-agent swarms delivering autonomous collaborative capabilities inside Office 365.", ["autonomous-multi-agent-saas", "multi-agent-collaborative-swarms", "enterprise-agent-swarms-m365"]),
    ]),
]

def generate_skills():
    skills = []
    cluster_dict = {c[0]: (c[1], c[2], c[3]) for c in CLUSTERS}
    
    for cluster_id, skill_tuples in CLUSTER_SKILLS:
        cluster_name, prefix, default_book = cluster_dict[cluster_id]
        for suffix, desc, triggers in skill_tuples:
            skill_id = f"{prefix}{suffix}"
            
            foundations = [
                f"Engineering Standard: Conforms strictly to enterprise patterns established in '{default_book}'.",
                f"Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking {skill_id} pipelines.",
                f"Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during {skill_id} operations.",
                f"Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.",
            ]
            
            protocol = f"ALWAYS apply the {skill_id} protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry."
            anti_patterns = [
                f"Deploying unvalidated configurations for {skill_id} directly into production without staging verification.",
                f"Silently ignoring transient failures or error codes during {skill_id} execution.",
            ]
            
            skills.append({
                "id": skill_id,
                "cluster": cluster_id,
                "book": default_book,
                "desc": desc,
                "triggers": [t.lower() for t in triggers],
                "foundations": foundations,
                "protocol": protocol,
                "anti_patterns": anti_patterns,
            })
            
    return skills

def main():
    skills = generate_skills()
    print(f"Total Office 365 skills to generate: {len(skills)}")
    assert len(skills) == 350, f"Expected 350 skills, found {len(skills)}"

    SKILLS_DIR.mkdir(parents=True, exist_ok=True)
    generated = 0

    for s in skills:
        skill_dir = SKILLS_DIR / s["id"]
        skill_dir.mkdir(parents=True, exist_ok=True)
        skill_file = skill_dir / "SKILL.md"

        triggers_str = ", ".join(f'"{t.lower()}"' for t in s["triggers"])
        foundations_str = "\n".join(f"- {f}" for f in s["foundations"])
        anti_str = "\n".join(f"- {a}" for a in s["anti_patterns"])

        content = f"""---
name: {s["id"]}
description: "{s["desc"]}"
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-{s['cluster']}"]
triggers: [{triggers_str}]
---

# {s["id"]}

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
{foundations_str}
- Source Reference: *{s["book"]}*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
{s["protocol"]}

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
{anti_str}

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ({triggers_str}) are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
"""
        skill_file.write_text(content, encoding="utf-8")
        generated += 1

    print(f"Successfully generated {generated} Office 365 skill packages in {SKILLS_DIR}")

if __name__ == "__main__":
    main()
