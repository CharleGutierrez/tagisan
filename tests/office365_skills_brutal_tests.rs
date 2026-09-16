//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Office 365 & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

pub const OFFICE365_SKILLS_350: [&str; 350] = [
    // Cluster 1 (Skill o365-excel-lambda-spill-range-anchoring-hash)
    "o365-excel-lambda-spill-range-anchoring-hash",
    "o365-excel-lambda-lambda-recursive-reduction",
    "o365-excel-lambda-map-byrow-bycol-vectorization",
    "o365-excel-lambda-scan-reduce-cumulative-state",
    "o365-excel-lambda-makearray-index-synthesis",
    "o365-excel-lambda-let-variable-memoization",
    "o365-excel-lambda-filter-unique-sortby-pipelines",
    "o365-excel-lambda-xlookup-binary-search-opt",
    "o365-excel-lambda-choosocols-chooserows-projection",
    "o365-excel-lambda-vstack-hstack-matrix-consolidation",
    // Cluster 2 (Skill o365-excel-scripts-range-batch-read-write)
    "o365-excel-scripts-range-batch-read-write",
    "o365-excel-scripts-table-dynamic-column-binding",
    "o365-excel-scripts-conditional-formatting-rules",
    "o365-excel-scripts-chart-series-generation",
    "o365-excel-scripts-cloud-flow-parameter-contracts",
    "o365-excel-scripts-regex-data-cleansing-pipeline",
    "o365-excel-scripts-multi-sheet-consolidation-loop",
    "o365-excel-scripts-auto-filter-criteria-tuning",
    "o365-excel-scripts-custom-sorting-matrix",
    "o365-excel-scripts-cell-data-validation-rules",
    // Cluster 3 (Skill o365-excel-js-context-sync-queue-minimization)
    "o365-excel-js-context-sync-queue-minimization",
    "o365-excel-js-streaming-custom-functions",
    "o365-excel-js-custom-function-metadata-json",
    "o365-excel-js-volatile-custom-function-caching",
    "o365-excel-js-webassembly-math-acceleration",
    "o365-excel-js-worksheet-event-handlers",
    "o365-excel-js-range-untracked-garbage-collection",
    "o365-excel-js-shape-and-svg-canvas-injection",
    "o365-excel-js-pivot-table-layout-manipulation",
    "o365-excel-js-named-item-scope-management",
    // Cluster 4 (Skill o365-word-openxml-document-part-traversal)
    "o365-word-openxml-document-part-traversal",
    "o365-word-content-control-data-binding",
    "o365-word-range-search-and-replace",
    "o365-word-openxml-sdk-headless-assembly",
    "o365-word-tracked-changes-and-comments",
    "o365-word-nested-table-and-cell-formatting",
    "o365-word-header-footer-section-breaks",
    "o365-word-custom-xml-parts-binding",
    "o365-word-html-to-docx-conversion-fidelity",
    "o365-word-ai-clause-generation-taskpane",
    // Cluster 5 (Skill o365-ppt-slide-layout-insertion-matrix)
    "o365-ppt-slide-layout-insertion-matrix",
    "o365-ppt-shape-tree-positioning-calculus",
    "o365-ppt-presentationml-raw-slide-scaffolding",
    "o365-ppt-textframe-paragraph-formatting",
    "o365-ppt-svg-vector-shape-import",
    "o365-ppt-chart-and-table-slide-injection",
    "o365-ppt-speaker-notes-and-comments",
    "o365-ppt-automated-qbr-deck-synthesis",
    "o365-ppt-slide-transition-and-media-timing",
    "o365-ppt-conversational-pitch-deck-generator",
    // Cluster 6 (Skill o365-outlook-read-vs-compose-mode-contexts)
    "o365-outlook-read-vs-compose-mode-contexts",
    "o365-outlook-event-based-smart-alerts",
    "o365-outlook-internet-headers-extraction",
    "o365-outlook-actionable-messages-adaptive-cards",
    "o365-outlook-attachment-chunked-streaming",
    "o365-outlook-meeting-scheduling-attendee-slots",
    "o365-outlook-categories-and-followup-flags",
    "o365-outlook-shared-mailbox-delegated-handlers",
    "o365-outlook-mobile-touch-ergonomic-taskpanes",
    "o365-outlook-ai-email-triage-and-auto-drafting",
    // Cluster 7 (Skill o365-onenote-notebook-section-page-hierarchy)
    "o365-onenote-notebook-section-page-hierarchy",
    "o365-onenote-multipart-mime-page-creation",
    "o365-onenote-inkml-digital-handwriting-parsing",
    "o365-onenote-onenote-html-schema-conformance",
    "o365-onenote-meeting-transcript-auto-summary",
    "o365-onenote-embedded-ocr-text-mining",
    "o365-onenote-class-notebook-student-distribution",
    "o365-onenote-cross-notebook-knowledge-search",
    "o365-onenote-notebook-archival-pdf-export",
    "o365-onenote-ai-second-brain-autonomous-sync",
    // Cluster 8 (Skill o365-teams-teams-js-sdk-v2-capabilities)
    "o365-teams-teams-js-sdk-v2-capabilities",
    "o365-teams-app-manifest-v1-17-schema",
    "o365-teams-bot-framework-turn-context",
    "o365-teams-messaging-extensions-zero-install",
    "o365-teams-dialog-and-task-module-popups",
    "o365-teams-teams-sso-silent-token-acquisition",
    "o365-teams-channel-vs-chat-context-isolation",
    "o365-teams-proactive-conversation-reference",
    "o365-teams-teams-toolkit-environment-pipelines",
    "o365-teams-conversational-ai-agent-teams-lib",
    // Cluster 9 (Skill o365-teams-live-meeting-lifecycle-stage-routing)
    "o365-teams-live-meeting-lifecycle-stage-routing",
    "o365-teams-live-live-share-sdk-ephemeral-state",
    "o365-teams-live-meeting-stage-sharing-permission",
    "o365-teams-live-in-meeting-push-notification-banner",
    "o365-teams-live-participant-role-verification",
    "o365-teams-live-real-time-live-media-streaming",
    "o365-teams-live-breakout-rooms-orchestration",
    "o365-teams-live-meeting-transcription-webhooks",
    "o365-teams-live-end-to-end-media-encryption-audit",
    "o365-teams-live-collaborative-whiteboard-live-share",
    // Cluster 10 (Skill o365-viva-viva-connections-ace-card-views)
    "o365-viva-viva-connections-ace-card-views",
    "o365-viva-ace-quick-view-interaction",
    "o365-viva-audience-targeting-and-profile-sync",
    "o365-viva-viva-topics-semantic-discovery",
    "o365-viva-viva-insights-collaboration-metrics",
    "o365-viva-viva-engage-community-broadcasting",
    "o365-viva-viva-learning-lms-content-sync",
    "o365-viva-mobile-viva-card-ergonomics",
    "o365-viva-viva-dashboard-performance-auditing",
    "o365-viva-ai-employee-experience-copilot",
    // Cluster 11 (Skill o365-spfx-core-spfx-yeoman-scaffolding-manifest)
    "o365-spfx-core-spfx-yeoman-scaffolding-manifest",
    "o365-spfx-core-spfx-component-lifecycle-dom",
    "o365-spfx-core-spfx-build-pipeline-webpack-vite",
    "o365-spfx-core-isolated-web-parts-iframe-security",
    "o365-spfx-core-package-solution-sppkg-bundling",
    "o365-spfx-core-multi-host-spfx-targets",
    "o365-spfx-core-spfx-automated-unit-testing",
    "o365-spfx-core-spfx-dynamic-data-event-broker",
    "o365-spfx-core-tenant-app-catalog-alm-deployment",
    "o365-spfx-core-spfx-node-nvm-environment-hygiene",
    // Cluster 12 (Skill o365-spfx-react-fluent-ui-v9-react-styling)
    "o365-spfx-react-fluent-ui-v9-react-styling",
    "o365-spfx-react-react-hooks-state-management",
    "o365-spfx-react-property-pane-reactive-controls",
    "o365-spfx-react-custom-property-pane-fields",
    "o365-spfx-react-wcag-accessibility-landmarks",
    "o365-spfx-react-theme-variant-subscriber",
    "o365-spfx-react-zustand-in-memory-caching",
    "o365-spfx-react-responsive-css-modules-grid",
    "o365-spfx-react-drag-drop-file-upload-cards",
    "o365-spfx-react-vibe-coding-webparts-generative-ai",
    // Cluster 13 (Skill o365-spfx-ext-application-customizer-placeholders)
    "o365-spfx-ext-application-customizer-placeholders",
    "o365-spfx-ext-global-tenant-navigation-bar",
    "o365-spfx-ext-field-customizer-cell-renderers",
    "o365-spfx-ext-listview-command-set-actions",
    "o365-spfx-ext-form-customizer-modern-overrides",
    "o365-spfx-ext-search-extension-query-modifiers",
    "o365-spfx-ext-dom-reflow-performance-profiling",
    "o365-spfx-ext-tenant-wide-extension-distribution",
    "o365-spfx-ext-playwright-e2e-extension-testing",
    "o365-spfx-ext-security-sandbox-xss-defense",
    // Cluster 14 (Skill o365-sp-data-pnpjs-fluent-client-chaining)
    "o365-sp-data-pnpjs-fluent-client-chaining",
    "o365-sp-data-indexed-columns-5k-threshold",
    "o365-sp-data-caml-query-efficient-filtering",
    "o365-sp-data-chunked-file-upload-sessions",
    "o365-sp-data-managed-metadata-term-store",
    "o365-sp-data-unique-permissions-break-inheritance",
    "o365-sp-data-document-version-history-pruning",
    "o365-sp-data-folder-tree-hierarchy-traversal",
    "o365-sp-data-cross-site-content-aggregation",
    "o365-sp-data-headless-sharepoint-file-backend",
    // Cluster 15 (Skill o365-graph-core-graph-rest-endpoint-conventions)
    "o365-graph-core-graph-rest-endpoint-conventions",
    "o365-graph-core-json-batch-request-multiplexing",
    "o365-graph-core-http-429-exponential-backoff",
    "o365-graph-core-typescript-graph-sdk-client",
    "o365-graph-core-kiota-strongly-typed-sdk-generator",
    "o365-graph-core-delegated-vs-app-permissions",
    "o365-graph-core-page-iterator-cursor-loop",
    "o365-graph-core-correlation-headers-telemetry",
    "o365-graph-core-multi-tenant-graph-client-factory",
    "o365-graph-core-ai-prompt-to-graph-query-synthesis",
    // Cluster 16 (Skill o365-graph-events-webhook-subscription-handshake)
    "o365-graph-events-webhook-subscription-handshake",
    "o365-graph-events-delta-query-incremental-sync",
    "o365-graph-events-jwe-encrypted-payload-decryption",
    "o365-graph-events-azure-event-hubs-streaming",
    "o365-graph-events-serverless-webhook-receiver-queue",
    "o365-graph-events-user-lifecycle-deprovisioning-flow",
    "o365-graph-events-driveitem-delta-change-processing",
    "o365-graph-events-subscription-auto-renewal-timer",
    "o365-graph-events-dead-letter-queue-reconciliation",
    "o365-graph-events-cryptographic-origin-verification",
    // Cluster 17 (Skill o365-graph-conn-connection-schema-registration)
    "o365-graph-conn-connection-schema-registration",
    "o365-graph-conn-external-item-put-ingestion",
    "o365-graph-conn-external-group-and-acl-crawling",
    "o365-graph-conn-incremental-content-crawler-engine",
    "o365-graph-conn-property-semantic-relevance-tuning",
    "o365-graph-conn-connector-test-harness-debugging",
    "o365-graph-conn-sql-and-jira-external-crawlers",
    "o365-graph-conn-ingestion-rate-limit-budgeting",
    "o365-graph-conn-admin-center-indexing-telemetry",
    "o365-graph-conn-copilot-grounding-external-items",
    // Cluster 18 (Skill o365-addin-core-unified-manifest-json-schema)
    "o365-addin-core-unified-manifest-json-schema",
    "o365-addin-core-office-onready-runtime-bootstrap",
    "o365-addin-core-shared-runtime-cross-component-state",
    "o365-addin-core-custom-ribbon-tabs-and-buttons",
    "o365-addin-core-webview2-desktop-vs-safari-web",
    "o365-addin-core-auto-open-taskpane-binding",
    "o365-addin-core-taskpane-dom-recycling-memory",
    "o365-addin-core-office-addin-debugging-tooling",
    "o365-addin-core-fluent-command-surface-ergonomics",
    "o365-addin-core-json-manifest-ci-cd-validation",
    // Cluster 19 (Skill o365-addin-auth-entra-id-app-registration-scopes)
    "o365-addin-auth-entra-id-app-registration-scopes",
    "o365-addin-auth-nested-app-auth-naa-broker",
    "o365-addin-auth-msal-browser-naa-integration",
    "o365-addin-auth-obo-token-exchange-flow",
    "o365-addin-auth-fallback-dialog-auth-flow",
    "o365-addin-auth-continuous-access-evaluation-cae",
    "o365-addin-auth-jwt-token-claims-validation",
    "o365-addin-auth-safari-itp-cookie-workarounds",
    "o365-addin-auth-pkce-public-client-exchange",
    "o365-addin-auth-zero-trust-addin-security",
    // Cluster 20 (Skill o365-addin-deploy-centralized-deployment-admin-center)
    "o365-addin-deploy-centralized-deployment-admin-center",
    "o365-addin-deploy-appsource-commercial-marketplace",
    "o365-addin-deploy-appsource-certification-validation",
    "o365-addin-deploy-transact-saas-offer-monetization",
    "o365-addin-deploy-zero-downtime-addin-versioning",
    "o365-addin-deploy-private-tenant-app-catalogs",
    "o365-addin-deploy-telemetry-and-crash-analytics",
    "o365-addin-deploy-multi-language-localization-bundles",
    "o365-addin-deploy-soc-2-publisher-attestation",
    "o365-addin-deploy-azure-static-web-apps-delivery",
    // Cluster 21 (Skill o365-fluid-fluid-distributed-data-structures)
    "o365-fluid-fluid-distributed-data-structures",
    "o365-fluid-azure-fluid-relay-service",
    "o365-fluid-collaborative-cursor-presence",
    "o365-fluid-conflict-free-collaborative-editing",
    "o365-fluid-fluid-container-lifecycle-management",
    "o365-fluid-fluid-in-spfx-and-teams",
    "o365-fluid-hierarchical-tree-state-modeling",
    "o365-fluid-intermittent-connection-resiliency",
    "o365-fluid-token-provider-and-security",
    "o365-fluid-multiplayer-vibe-coding-canvas",
    // Cluster 22 (Skill o365-loop-loop-component-file-architecture)
    "o365-loop-loop-component-file-architecture",
    "o365-loop-adaptive-card-loop-components",
    "o365-loop-cross-host-bi-directional-sync",
    "o365-loop-loop-workspace-graph-apis",
    "o365-loop-collaborative-tables-and-trackers",
    "o365-loop-purview-governance-on-loop",
    "o365-loop-third-party-widget-embedding",
    "o365-loop-b2b-guest-sharing-security-rules",
    "o365-loop-loop-page-template-scaffolding",
    "o365-loop-autonomous-agents-in-loop-canvases",
    // Cluster 23 (Skill o365-search-kql-query-syntax-mastery)
    "o365-search-kql-query-syntax-mastery",
    "o365-search-search-rest-api-query-execution",
    "o365-search-syntex-document-understanding-models",
    "o365-search-adaptive-card-search-display-templates",
    "o365-search-neural-semantic-index-mechanics",
    "o365-search-automated-retention-label-tagging",
    "o365-search-search-refiners-and-managed-properties",
    "o365-search-acronym-and-bookmark-management",
    "o365-search-security-trimmed-search-results",
    "o365-search-enterprise-rag-search-agent",
    // Cluster 24 (Skill o365-forms-forms-power-automate-trigger)
    "o365-forms-forms-power-automate-trigger",
    "o365-forms-survey-branching-logic-architecture",
    "o365-forms-forms-rest-api-inspection",
    "o365-forms-automated-quiz-grading-algorithms",
    "o365-forms-responsive-iframe-embedding",
    "o365-forms-forms-file-upload-onedrive-routing",
    "o365-forms-power-bi-streaming-forms-data",
    "o365-forms-multilingual-forms-routing",
    "o365-forms-customer-voice-nps-automation",
    "o365-forms-conversational-survey-scaffolding",
    // Cluster 25 (Skill o365-tasks-unified-tasks-graph-model)
    "o365-tasks-unified-tasks-graph-model",
    "o365-tasks-automated-kanban-board-builder",
    "o365-tasks-etag-optimistic-concurrency-planner",
    "o365-tasks-todo-personal-list-synchronization",
    "o365-tasks-teams-tasks-by-planner-integration",
    "o365-tasks-planner-velocity-and-burndown-pbi",
    "o365-tasks-automated-task-escalation-flow",
    "o365-tasks-checklist-items-and-attachments",
    "o365-tasks-group-owned-plan-security",
    "o365-tasks-autonomous-project-manager-agent",
    // Cluster 26 (Skill o365-bookings-bookings-schema-and-endpoints)
    "o365-bookings-bookings-schema-and-endpoints",
    "o365-bookings-automated-calendar-appointment-scheduling",
    "o365-bookings-custom-nextjs-booking-frontend",
    "o365-bookings-staff-working-hours-and-conflict-guards",
    "o365-bookings-sms-email-reminder-dispatch",
    "o365-bookings-virtual-visit-teams-meeting-links",
    "o365-bookings-custom-intake-questions-mapping",
    "o365-bookings-multi-location-branch-architecture",
    "o365-bookings-payment-deposit-verification",
    "o365-bookings-ai-conversational-booking-concierge",
    // Cluster 27 (Skill o365-lists-json-column-formatting-ast)
    "o365-lists-json-column-formatting-ast",
    "o365-lists-view-formatting-kanban-boards",
    "o365-lists-status-pill-badges-and-color-ramps",
    "o365-lists-executeflow-action-buttons",
    "o365-lists-in-line-svg-donut-charts",
    "o365-lists-customcardprops-hover-flyouts",
    "o365-lists-json-form-section-formatting",
    "o365-lists-pnp-list-formatting-repository",
    "o365-lists-formatter-rendering-performance",
    "o365-lists-screenshot-to-json-formatter-ai",
    // Cluster 28 (Skill o365-exchange-exchange-powershell-v3-rest)
    "o365-exchange-exchange-powershell-v3-rest",
    "o365-exchange-mail-flow-transport-rules",
    "o365-exchange-spf-dkim-dmarc-authentication",
    "o365-exchange-anti-malware-and-zap-policies",
    "o365-exchange-smtp-relay-and-direct-send",
    "o365-exchange-shared-mailbox-and-resource-routing",
    "o365-exchange-mailbox-auditing-and-litigation-hold",
    "o365-exchange-cross-premises-hybrid-mail-flow",
    "o365-exchange-message-trace-delivery-telemetry",
    "o365-exchange-autonomous-quarantine-triage-agent",
    // Cluster 29 (Skill o365-purview-sensitivity-label-taxonomy)
    "o365-purview-sensitivity-label-taxonomy",
    "o365-purview-data-loss-prevention-policy-rules",
    "o365-purview-mip-sdk-programmatic-labeling",
    "o365-purview-exact-data-match-sit-hashing",
    "o365-purview-office-js-sensitivity-label-api",
    "o365-purview-defender-cloud-apps-session-proxy",
    "o365-purview-auto-labeling-at-rest-sharepoint",
    "o365-purview-double-key-encryption-dke-service",
    "o365-purview-unified-audit-log-label-downgrade",
    "o365-purview-ai-trainable-classifiers-purview",
    // Cluster 30 (Skill o365-sec-zero-trust-architecture-principles)
    "o365-sec-zero-trust-architecture-principles",
    "o365-sec-entra-id-pim-just-in-time",
    "o365-sec-conditional-access-signal-policy",
    "o365-sec-workload-identity-oidc-federation",
    "o365-sec-app-consent-and-permission-reviews",
    "o365-sec-microsoft-sentinel-m365-ingestion",
    "o365-sec-intune-mam-app-protection-policies",
    "o365-sec-threat-hunting-and-identity-alerts",
    "o365-sec-defender-air-automated-playbooks",
    "o365-sec-autonomous-permission-drift-watchdog",
    // Cluster 31 (Skill o365-compliance-ediscovery-premium-case-management)
    "o365-compliance-ediscovery-premium-case-management",
    "o365-compliance-unified-audit-log-forensic-queries",
    "o365-compliance-retention-policies-and-preservation-locks",
    "o365-compliance-insider-risk-data-theft-detection",
    "o365-compliance-information-barriers-isolation",
    "o365-compliance-graph-ediscovery-api-automation",
    "o365-compliance-compliance-manager-assessment-templates",
    "o365-compliance-inactive-mailbox-litigation-retention",
    "o365-compliance-compliance-power-bi-audit-dashboard",
    "o365-compliance-autonomous-compliance-verifier-agent",
    // Cluster 32 (Skill o365-admin-microsoft-graph-powershell-sdk-core)
    "o365-admin-microsoft-graph-powershell-sdk-core",
    "o365-admin-automated-user-onboarding-scripts",
    "o365-admin-dynamic-group-membership-rules",
    "o365-admin-license-reclamation-and-sku-optimization",
    "o365-admin-tenant-to-tenant-cross-migration",
    "o365-admin-b2b-direct-connect-shared-channels",
    "o365-admin-powershell-parallel-loop-performance",
    "o365-admin-service-health-outage-alerts",
    "o365-admin-custom-domain-dns-automation",
    "o365-admin-conversational-tenant-devops",
    // Cluster 33 (Skill o365-vibe-conversational-vibe-coding-workflow)
    "o365-vibe-conversational-vibe-coding-workflow",
    "o365-vibe-tracer-bullet-office-apps",
    "o365-vibe-rapid-feedback-hot-module-reload",
    "o365-vibe-conversational-tdd-office-addins",
    "o365-vibe-mock-graph-data-exploratory-spikes",
    "o365-vibe-dialectical-code-review-agents",
    "o365-vibe-keyboard-first-developer-flow",
    "o365-vibe-fail-fast-webview-error-surfacing",
    "o365-vibe-human-in-the-loop-architectural-guardrails",
    "o365-vibe-spec-driven-solution-scaffolding",
    // Cluster 34 (Skill o365-alm-m365-cli-cross-platform-automation)
    "o365-alm-m365-cli-cross-platform-automation",
    "o365-alm-pnp-powershell-site-provisioning",
    "o365-alm-site-designs-and-json-site-scripts",
    "o365-alm-github-actions-ci-cd-pipelines",
    "o365-alm-pnp-provisioning-engine-templates",
    "o365-alm-microsoft365-dsc-infrastructure-as-code",
    "o365-alm-teams-template-channel-scaffolding",
    "o365-alm-azure-key-vault-secret-rotation-alm",
    "o365-alm-static-analysis-and-bundle-size-gates",
    "o365-alm-disaster-recovery-and-tenant-rollback",
    // Cluster 35 (Skill o365-saas-multi-tenant-data-partitioning)
    "o365-saas-multi-tenant-data-partitioning",
    "o365-saas-admin-consent-url-architecture",
    "o365-saas-marketplace-saas-fulfillment-api",
    "o365-saas-sovereign-cloud-boundary-compliance",
    "o365-saas-metering-and-credit-billing-engine",
    "o365-saas-publisher-attestation-security-controls",
    "o365-saas-partner-center-offer-lifecycle",
    "o365-saas-azure-front-door-global-cdn-caching",
    "o365-saas-multi-tenant-telemetry-isolation",
    "o365-saas-autonomous-multi-agent-saas-swarms",
];

// =========================================================================
// 1. Discovery of all 350 Office 365 Skills on Disk
// =========================================================================

#[test]
fn test_all_350_office365_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &OFFICE365_SKILLS_350 {
        assert!(
            loaded_map.contains(*skill),
            "Office 365 skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 350 Office 365 Skills (assert >= 1345 total skills)
// =========================================================================

#[test]
fn test_all_350_office365_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 1345,
        "Expected at least 1345 total built-in skills including Office 365 suite, found {}",
        all_skills.len()
    );

    for skill_name in &OFFICE365_SKILLS_350 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Office 365 skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty(), "Skill name cannot be empty for '{}'", skill_name);
        assert!(!s.description.is_empty(), "Description cannot be empty for '{}'", skill_name);
        assert!(!s.instructions.is_empty(), "Instructions cannot be empty for '{}'", skill_name);
        assert!(
            s.instructions.len() > 100,
            "Skill '{}' must contain concrete engineering instructions, got length {}",
            skill_name,
            s.instructions.len()
        );
        assert!(
            s.instructions.contains("## 1. Core Mathematical"),
            "Skill '{}' must contain Core Mathematical & Architectural Foundations",
            skill_name
        );
        assert!(
            s.instructions.contains("## 2. Concrete Agent Specification"),
            "Skill '{}' must contain Concrete Agent Specification & Prompt Contract",
            skill_name
        );
        assert!(
            s.instructions.contains("## 3. Anti-Patterns"),
            "Skill '{}' must contain Anti-Patterns & Hallucination Mitigations",
            skill_name
        );
    }
}

// =========================================================================
// 3. High-Leverage Alias Resolution Across All 35 Clusters
// =========================================================================

#[test]
fn test_high_leverage_alias_lookups() {
    let alias_cases = [
        ("spill-range-hash", "o365-excel-lambda-spill-range-anchoring-hash"),
        ("range-batch-read-write", "o365-excel-scripts-range-batch-read-write"),
        ("context-sync-minimization", "o365-excel-js-context-sync-queue-minimization"),
        ("openxml-document-traversal", "o365-word-openxml-document-part-traversal"),
        ("slide-layout-insertion", "o365-ppt-slide-layout-insertion-matrix"),
        ("read-vs-compose-contexts", "o365-outlook-read-vs-compose-mode-contexts"),
        ("notebook-hierarchy-navigation", "o365-onenote-notebook-section-page-hierarchy"),
        ("teams-js-sdk-v2", "o365-teams-teams-js-sdk-v2-capabilities"),
        ("meeting-lifecycle-stages", "o365-teams-live-meeting-lifecycle-stage-routing"),
        ("viva-ace-card-views", "o365-viva-viva-connections-ace-card-views"),
        ("spfx-yeoman-manifest", "o365-spfx-core-spfx-yeoman-scaffolding-manifest"),
        ("fluent-ui-v9-spfx", "o365-spfx-react-fluent-ui-v9-react-styling"),
        ("application-customizer-placeholders", "o365-spfx-ext-application-customizer-placeholders"),
        ("pnpjs-fluent-client", "o365-sp-data-pnpjs-fluent-client-chaining"),
        ("graph-rest-conventions", "o365-graph-core-graph-rest-endpoint-conventions"),
        ("webhook-subscription-handshake", "o365-graph-events-webhook-subscription-handshake"),
        ("connection-schema-registration", "o365-graph-conn-connection-schema-registration"),
        ("unified-manifest-json", "o365-addin-core-unified-manifest-json-schema"),
        ("entra-id-app-scopes", "o365-addin-auth-entra-id-app-registration-scopes"),
        ("centralized-deployment-admin", "o365-addin-deploy-centralized-deployment-admin-center"),
        ("fluid-dds-fundamentals", "o365-fluid-fluid-distributed-data-structures"),
        ("loop-component-architecture", "o365-loop-loop-component-file-architecture"),
        ("kql-query-syntax", "o365-search-kql-query-syntax-mastery"),
        ("forms-flow-trigger", "o365-forms-forms-power-automate-trigger"),
        ("unified-tasks-graph-model", "o365-tasks-unified-tasks-graph-model"),
        ("bookings-schema-endpoints", "o365-bookings-bookings-schema-and-endpoints"),
        ("json-column-formatting", "o365-lists-json-column-formatting-ast"),
        ("exchange-powershell-v3", "o365-exchange-exchange-powershell-v3-rest"),
        ("sensitivity-label-taxonomy", "o365-purview-sensitivity-label-taxonomy"),
        ("zero-trust-principles", "o365-sec-zero-trust-architecture-principles"),
        ("ediscovery-premium-cases", "o365-compliance-ediscovery-premium-case-management"),
        ("graph-powershell-sdk-core", "o365-admin-microsoft-graph-powershell-sdk-core"),
        ("conversational-vibe-workflow", "o365-vibe-conversational-vibe-coding-workflow"),
        ("m365-cli-automation", "o365-alm-m365-cli-cross-platform-automation"),
        ("multi-tenant-data-partitioning", "o365-saas-multi-tenant-data-partitioning"),
    ];

    for (alias, canonical) in &alias_cases {
        let res = find_ecc_skill(alias);
        assert!(
            res.is_some(),
            "Alias '{}' must resolve to canonical skill '{}'",
            alias,
            canonical
        );
        assert_eq!(
            res.unwrap().name,
            *canonical,
            "Alias '{}' resolved to unexpected skill name",
            alias
        );
    }
}

// =========================================================================
// 4. Structural Completeness & Operational Invariants
// =========================================================================

#[test]
fn test_structural_completeness_and_operational_invariants() {
    for skill_name in &OFFICE365_SKILLS_350 {
        let skill = find_ecc_skill(skill_name).expect("Skill must be found");

        assert!(
            skill.instructions.contains("ALWAYS"),
            "Skill '{}' must enforce explicit ALWAYS directive",
            skill_name
        );
        assert!(
            skill.instructions.contains("NEVER"),
            "Skill '{}' must enforce explicit NEVER directive",
            skill_name
        );
        assert!(
            skill.instructions.contains("MANDATORY"),
            "Skill '{}' must enforce explicit MANDATORY directive",
            skill_name
        );
    }
}

// =========================================================================
// 5. Semantic Intent Dispatching & Routing Across Clusters
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_and_routing() {
    let dispatcher = global_ecc_dispatcher();

    let query_cases = [
        ("spill-range-hash", "o365-excel-lambda-spill-range-anchoring-hash"),
        ("range-batch-read-write", "o365-excel-scripts-range-batch-read-write"),
        ("context-sync-minimization", "o365-excel-js-context-sync-queue-minimization"),
        ("openxml-document-traversal", "o365-word-openxml-document-part-traversal"),
        ("slide-layout-insertion", "o365-ppt-slide-layout-insertion-matrix"),
        ("read-vs-compose-contexts", "o365-outlook-read-vs-compose-mode-contexts"),
        ("notebook-hierarchy-navigation", "o365-onenote-notebook-section-page-hierarchy"),
        ("teams-js-sdk-v2", "o365-teams-teams-js-sdk-v2-capabilities"),
        ("meeting-lifecycle-stages", "o365-teams-live-meeting-lifecycle-stage-routing"),
        ("viva-ace-card-views", "o365-viva-viva-connections-ace-card-views"),
        ("spfx-yeoman-manifest", "o365-spfx-core-spfx-yeoman-scaffolding-manifest"),
        ("fluent-ui-v9-spfx", "o365-spfx-react-fluent-ui-v9-react-styling"),
        ("application-customizer-placeholders", "o365-spfx-ext-application-customizer-placeholders"),
        ("pnpjs-fluent-client", "o365-sp-data-pnpjs-fluent-client-chaining"),
        ("graph-rest-conventions", "o365-graph-core-graph-rest-endpoint-conventions"),
        ("webhook-subscription-handshake", "o365-graph-events-webhook-subscription-handshake"),
        ("connection-schema-registration", "o365-graph-conn-connection-schema-registration"),
        ("unified-manifest-json", "o365-addin-core-unified-manifest-json-schema"),
        ("entra-id-app-scopes", "o365-addin-auth-entra-id-app-registration-scopes"),
        ("centralized-deployment-admin", "o365-addin-deploy-centralized-deployment-admin-center"),
        ("fluid-dds-fundamentals", "o365-fluid-fluid-distributed-data-structures"),
        ("loop-component-architecture", "o365-loop-loop-component-file-architecture"),
        ("kql-query-syntax", "o365-search-kql-query-syntax-mastery"),
        ("forms-flow-trigger", "o365-forms-forms-power-automate-trigger"),
        ("unified-tasks-graph-model", "o365-tasks-unified-tasks-graph-model"),
        ("bookings-schema-endpoints", "o365-bookings-bookings-schema-and-endpoints"),
        ("json-column-formatting", "o365-lists-json-column-formatting-ast"),
        ("exchange-powershell-v3", "o365-exchange-exchange-powershell-v3-rest"),
        ("sensitivity-label-taxonomy", "o365-purview-sensitivity-label-taxonomy"),
        ("zero-trust-principles", "o365-sec-zero-trust-architecture-principles"),
        ("ediscovery-premium-cases", "o365-compliance-ediscovery-premium-case-management"),
        ("graph-powershell-sdk-core", "o365-admin-microsoft-graph-powershell-sdk-core"),
        ("conversational-vibe-workflow", "o365-vibe-conversational-vibe-coding-workflow"),
        ("m365-cli-automation", "o365-alm-m365-cli-cross-platform-automation"),
        ("multi-tenant-data-partitioning", "o365-saas-multi-tenant-data-partitioning"),
    ];

    for (query, expected_skill) in &query_cases {
        let results = dispatcher.dispatch(query, 5, None);
        assert!(
            !results.is_empty(),
            "Dispatcher must return matches for query '{}'",
            query
        );

        let found = results.iter().any(|d| d.skill.name == *expected_skill);
        assert!(
            found,
            "Dispatcher failed to match query '{}' with expected skill '{}'. Got top matches: {:?}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );
    }
}

// =========================================================================
// 6. Concurrent Dispatching Stress Test (50 Threads, Zero Panic)
// =========================================================================

#[test]
fn test_concurrent_dispatching_stress_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "spill-range-hash",
        "range-batch-read-write",
        "context-sync-minimization",
        "openxml-document-traversal",
        "slide-layout-insertion",
        "read-vs-compose-contexts",
        "notebook-hierarchy-navigation",
        "teams-js-sdk-v2",
        "meeting-lifecycle-stages",
        "viva-ace-card-views",
        "spfx-yeoman-manifest",
        "fluent-ui-v9-spfx",
        "application-customizer-placeholders",
        "pnpjs-fluent-client",
        "graph-rest-conventions",
        "webhook-subscription-handshake",
        "connection-schema-registration",
        "unified-manifest-json",
        "entra-id-app-scopes",
        "centralized-deployment-admin",
        "fluid-dds-fundamentals",
        "loop-component-architecture",
        "kql-query-syntax",
        "forms-flow-trigger",
        "unified-tasks-graph-model",
        "bookings-schema-endpoints",
        "json-column-formatting",
        "exchange-powershell-v3",
        "sensitivity-label-taxonomy",
        "zero-trust-principles",
        "ediscovery-premium-cases",
        "graph-powershell-sdk-core",
        "conversational-vibe-workflow",
        "m365-cli-automation",
        "multi-tenant-data-partitioning",
    ]);

    let thread_count = 50;
    let iterations_per_thread = 20;
    let completed_queries = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(thread_count);

    let start = Instant::now();

    for t_idx in 0..thread_count {
        let d = Arc::clone(&dispatcher);
        let q = Arc::clone(&queries);
        let c = Arc::clone(&completed_queries);

        let handle = thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let query = q[(t_idx + i) % q.len()];
                let res = d.dispatch(query, 3, None);
                if !res.is_empty() && res[0].score > 0.0 {
                    c.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked during dispatch");
    }

    let elapsed = start.elapsed();
    let total = thread_count * iterations_per_thread;
    let completed = completed_queries.load(Ordering::SeqCst);
    assert_eq!(completed, total, "Expected all 1000 concurrent dispatches to succeed");
    println!(
        "Concurrent stress test: {} dispatches across 50 threads completed in {:?} ({:.2} us/dispatch)",
        total,
        elapsed,
        (elapsed.as_micros() as f64) / (total as f64)
    );
}
