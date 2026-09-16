//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Power Platform & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

pub const POWER_PLATFORM_SKILLS_350: [&str; 350] = [
    // Cluster 1 (Skill pp-canvas-fluid-container-layouts)
    "pp-canvas-fluid-container-layouts",
    "pp-canvas-screen-breakpoints-resolution",
    "pp-canvas-app-onstart-named-formulas",
    "pp-canvas-modal-dialog-surfacing",
    "pp-canvas-accessible-aria-theming",
    "pp-canvas-deep-linking-param-routing",
    "pp-canvas-dynamic-svg-data-uris",
    "pp-canvas-timer-animation-ergonomics",
    "pp-canvas-multi-form-wizard-state",
    "pp-canvas-print-pdf-export-controls",
    // Cluster 2 (Skill pp-fx-imperative-vs-declarative)
    "pp-fx-imperative-vs-declarative",
    "pp-fx-strongly-typed-records-tables",
    "pp-fx-untyped-object-json-parsing",
    "pp-fx-concurrent-evaluation-patterns",
    "pp-fx-user-defined-functions-udf",
    "pp-fx-error-handling-iferror-isblank",
    "pp-fx-relational-lookup-navigation",
    "pp-fx-in-memory-caching-collections",
    "pp-fx-regular-expressions-pattern-matching",
    "pp-fx-coalesce-null-propagation",
    // Cluster 3 (Skill pp-comp-custom-properties-input-output)
    "pp-comp-custom-properties-input-output",
    "pp-comp-component-libraries-versioning",
    "pp-comp-fluent-navigation-header-bars",
    "pp-comp-custom-data-grid-pagination",
    "pp-comp-global-theme-token-provider",
    "pp-comp-notification-toast-overlay",
    "pp-comp-media-file-uploader-dropzone",
    "pp-comp-treeview-hierarchical-navigator",
    "pp-comp-kpi-metric-card-widgets",
    "pp-comp-signature-capture-canvas",
    // Cluster 4 (Skill pp-offline-loaddata-savedata-local)
    "pp-offline-loaddata-savedata-local",
    "pp-offline-dataverse-offline-profiles",
    "pp-offline-two-way-sync-conflict-resolution",
    "pp-offline-device-hardware-sensors",
    "pp-offline-camera-photo-compression",
    "pp-offline-mobile-push-notifications",
    "pp-offline-nfc-tag-reading-rfid",
    "pp-offline-network-connectivity-status",
    "pp-offline-background-sync-reconciliation",
    "pp-offline-biometric-auth-gateways",
    // Cluster 5 (Skill pp-perf-delegation-limits-workarounds)
    "pp-perf-delegation-limits-workarounds",
    "pp-perf-odata-delegation-matrix",
    "pp-perf-n-plus-one-query-elimination",
    "pp-perf-monitor-tool-network-telemetry",
    "pp-perf-app-bundle-asset-trimming",
    "pp-perf-lazy-loading-screen-caching",
    "pp-perf-control-count-dom-overhead",
    "pp-perf-sql-stored-procedure-delegation",
    "pp-perf-dataverse-indexed-columns",
    "pp-perf-concurrent-data-hydration",
    // Cluster 6 (Skill pp-dv-standard-custom-activity-tables)
    "pp-dv-standard-custom-activity-tables",
    "pp-dv-relational-cardinality-relationships",
    "pp-dv-polymorphic-lookup-architecture",
    "pp-dv-alternate-keys-upsert-idempotency",
    "pp-dv-calculated-and-rollup-columns",
    "pp-dv-formula-columns-power-fx",
    "pp-dv-schema-prefix-publisher-governance",
    "pp-dv-auditing-retention-lifecycle",
    "pp-dv-elastic-tables-azure-cosmos-db",
    "pp-dv-file-and-image-column-streaming",
    // Cluster 7 (Skill pp-mda-modern-app-designer-sitemaps)
    "pp-mda-modern-app-designer-sitemaps",
    "pp-mda-main-quick-create-card-forms",
    "pp-mda-business-rules-declarative-logic",
    "pp-mda-business-process-flows-bpf",
    "pp-mda-modern-command-bar-power-fx",
    "pp-mda-subgrid-view-customization",
    "pp-mda-client-scripting-xrm-page-api",
    "pp-mda-web-resources-javascript-html",
    "pp-mda-dashboard-charts-power-bi-embedding",
    "pp-mda-embedded-canvas-pcf-controls",
    // Cluster 8 (Skill pp-sec-privilege-depth-matrix)
    "pp-sec-privilege-depth-matrix",
    "pp-sec-business-units-security-hierarchy",
    "pp-sec-entra-id-group-teams",
    "pp-sec-field-level-security-profiles",
    "pp-sec-record-ownership-and-sharing",
    "pp-sec-hierarchy-security-models",
    "pp-sec-service-principal-app-users",
    "pp-sec-modern-role-based-app-licensing",
    "pp-sec-data-masking-and-encryption",
    "pp-sec-audit-logging-purview-integration",
    // Cluster 9 (Skill pp-flow-trigger-types-matrix)
    "pp-flow-trigger-types-matrix",
    "pp-flow-dataverse-trigger-filtering",
    "pp-flow-trigger-conditions-expressions",
    "pp-flow-concurrency-and-degree-of-parallelism",
    "pp-flow-dynamic-content-and-json-schemas",
    "pp-flow-child-flows-reusable-subroutines",
    "pp-flow-webhook-callback-patterns",
    "pp-flow-batch-pagination-chunking",
    "pp-flow-timeout-and-run-duration-limits",
    "pp-flow-service-account-connection-references",
    // Cluster 10 (Skill pp-err-try-catch-finally-scope-patterns)
    "pp-err-try-catch-finally-scope-patterns",
    "pp-err-run-after-configuration-matrices",
    "pp-err-retry-policy-exponential-backoff",
    "pp-err-result-function-error-inspection",
    "pp-err-dead-letter-queue-logging",
    "pp-err-circuit-breaker-transient-failures",
    "pp-err-compensating-transactions-rollback",
    "pp-err-alerting-teams-adaptive-cards",
    "pp-err-flow-analytics-run-history-api",
    "pp-err-idempotent-replay-safety",
    // Cluster 11 (Skill pp-appr-approval-types-matrix)
    "pp-appr-approval-types-matrix",
    "pp-appr-sequential-and-parallel-approvals",
    "pp-appr-approvals-in-teams-adaptive-cards",
    "pp-appr-escalation-and-timeout-reassignment",
    "pp-appr-dataverse-approvals-core-tables",
    "pp-appr-delegation-and-out-of-office-routing",
    "pp-appr-digital-signature-audit-trails",
    "pp-appr-guest-user-external-approvals",
    "pp-appr-cancellation-and-revocation-logic",
    "pp-appr-ai-summarized-approval-packets",
    // Cluster 12 (Skill pp-odata-filter-query-syntax)
    "pp-odata-filter-query-syntax",
    "pp-odata-expand-query-related-records",
    "pp-odata-select-column-pruning",
    "pp-odata-orderby-and-top-pagination",
    "pp-odata-xml-fetchxml-conversions",
    "pp-odata-json-xpath-xml-transformations",
    "pp-odata-array-filter-select-operations",
    "pp-odata-chunked-batching-changesets",
    "pp-odata-delta-token-incremental-sync",
    "pp-odata-base64-binary-handling",
    // Cluster 13 (Skill pp-rpa-attended-vs-unattended-execution)
    "pp-rpa-attended-vs-unattended-execution",
    "pp-rpa-desktop-flow-machine-groups",
    "pp-rpa-ui-element-selector-tuning",
    "pp-rpa-web-automation-browser-drivers",
    "pp-rpa-legacy-win32-terminal-emulation",
    "pp-rpa-credential-cyberark-azure-keyvault",
    "pp-rpa-subflows-exception-handling",
    "pp-rpa-ocr-screen-scraping-ai-vision",
    "pp-rpa-excel-advanced-vba-macros",
    "pp-rpa-cloud-to-desktop-flow-orchestration",
    // Cluster 14 (Skill pp-pbi-star-schema-dimensional-modeling)
    "pp-pbi-star-schema-dimensional-modeling",
    "pp-pbi-relationship-cardinality-filter-direction",
    "pp-pbi-role-playing-dimensions-active-keys",
    "pp-pbi-slowly-changing-dimensions-scd",
    "pp-pbi-composite-models-directquery-import",
    "pp-pbi-incremental-refresh-range-partitions",
    "pp-pbi-row-level-security-rls-roles",
    "pp-pbi-calculation-groups-time-intelligence",
    "pp-pbi-field-parameters-dynamic-axes",
    "pp-pbi-vertipaq-engine-compression-tuning",
    // Cluster 15 (Skill pp-dax-row-context-vs-filter-context)
    "pp-dax-row-context-vs-filter-context",
    "pp-dax-calculate-context-transition",
    "pp-dax-filter-modifiers-all-allexcept",
    "pp-dax-time-intelligence-functions",
    "pp-dax-iterator-functions-sumx-filter",
    "pp-dax-semi-additive-measures-inventory",
    "pp-dax-virtual-table-creation-summarize",
    "pp-dax-variables-var-evaluation-scope",
    "pp-dax-dax-studio-performance-profiling",
    "pp-dax-dynamic-ranking-topn-pareto",
    // Cluster 16 (Skill pp-m-query-folding-diagnostics)
    "pp-m-query-folding-diagnostics",
    "pp-m-let-in-evaluation-laziness",
    "pp-m-custom-functions-and-recursion",
    "pp-m-nested-lists-records-tables",
    "pp-m-web-api-json-ingestion-paging",
    "pp-m-fuzzy-matching-and-clustering",
    "pp-m-error-handling-try-otherwise",
    "pp-m-data-type-coercion-locales",
    "pp-m-unpivot-transpose-matrix-wrangling",
    "pp-m-dataflow-gen2-fabric-integration",
    // Cluster 17 (Skill pp-pbicop-linguistic-schema-synonyms)
    "pp-pbicop-linguistic-schema-synonyms",
    "pp-pbicop-copilot-report-page-generation",
    "pp-pbicop-narrative-visual-summaries",
    "pp-pbicop-dax-formula-generation-ai",
    "pp-pbicop-featured-questions-and-qa",
    "pp-pbicop-data-storytelling-infographics",
    "pp-pbicop-fabric-workspace-capacity-admin",
    "pp-pbicop-copilot-security-sensitivity-labels",
    "pp-pbicop-semantic-model-metadata-enrichment",
    "pp-pbicop-mobile-bi-report-ergonomics",
    // Cluster 18 (Skill pp-pages-site-architecture-web-templates)
    "pp-pages-site-architecture-web-templates",
    "pp-pages-liquid-syntax-control-flow",
    "pp-pages-liquid-entity-query-fetchxml",
    "pp-pages-portal-user-authentication-contacts",
    "pp-pages-web-roles-table-permissions",
    "pp-pages-multilingual-content-snippets",
    "pp-pages-site-settings-and-custom-domains",
    "pp-pages-caching-invalidation-mechanisms",
    "pp-pages-head-meta-seo-optimization",
    "pp-pages-progressive-web-app-pwa-features",
    // Cluster 19 (Skill pp-pweb-dataverse-web-api-portal-client)
    "pp-pweb-dataverse-web-api-portal-client",
    "pp-pweb-site-setting-api-column-permissions",
    "pp-pweb-csrf-token-request-validation",
    "pp-pweb-basic-forms-and-multistep-forms",
    "pp-pweb-client-side-form-validation-js",
    "pp-pweb-file-attachments-azure-blob-storage",
    "pp-pweb-subgrid-actions-and-modals",
    "pp-pweb-anti-scraping-waf-cloud-security",
    "pp-pweb-custom-portal-web-services",
    "pp-pweb-audit-trail-and-portal-telemetry",
    // Cluster 20 (Skill pp-pstyle-bootstrap-5-customization)
    "pp-pstyle-bootstrap-5-customization",
    "pp-pstyle-fluent-2-design-tokens-integration",
    "pp-pstyle-styling-workspace-code-studio",
    "pp-pstyle-responsive-navigation-mega-menus",
    "pp-pstyle-accessible-form-controls-wcag",
    "pp-pstyle-interactive-data-tables-datatables",
    "pp-pstyle-css-grid-flexbox-card-layouts",
    "pp-pstyle-custom-svg-iconography-icons",
    "pp-pstyle-dark-mode-toggle-state",
    "pp-pstyle-micro-animations-css-transitions",
    // Cluster 21 (Skill pp-csgen-intent-recognition-trigger-phrases)
    "pp-csgen-intent-recognition-trigger-phrases",
    "pp-csgen-conversation-boosting-answers",
    "pp-csgen-system-prompt-persona-tuning",
    "pp-csgen-entity-extraction-slot-filling",
    "pp-csgen-clarification-and-disambiguation",
    "pp-csgen-multi-turn-context-memory",
    "pp-csgen-generative-node-custom-instructions",
    "pp-csgen-topic-level-moderation-filters",
    "pp-csgen-user-input-validation-repair",
    "pp-csgen-fallback-topic-graceful-recovery",
    // Cluster 22 (Skill pp-cschain-dynamic-chaining-generative-actions)
    "pp-cschain-dynamic-chaining-generative-actions",
    "pp-cschain-action-input-output-parameter-schemas",
    "pp-cschain-power-automate-flow-action-plugins",
    "pp-cschain-connector-actions-openapi-plugins",
    "pp-cschain-confirmation-and-safety-checkpoints",
    "pp-cschain-dynamic-plan-inspection-debugging",
    "pp-cschain-action-chaining-multi-step-execution",
    "pp-cschain-error-handling-and-retry-actions",
    "pp-cschain-custom-api-plugin-manifests",
    "pp-cschain-latency-budgeting-streaming",
    // Cluster 23 (Skill pp-csknow-dataverse-search-knowledge-indexing)
    "pp-csknow-dataverse-search-knowledge-indexing",
    "pp-csknow-sharepoint-onedrive-grounding",
    "pp-csknow-public-website-url-grounding",
    "pp-csknow-uploaded-file-knowledge-stores",
    "pp-csknow-semantic-citations-referencing",
    "pp-csknow-knowledge-filters-metadata-tagging",
    "pp-csknow-knowledge-refresh-cycle-cadence",
    "pp-csknow-synonym-dictionaries-and-acronyms",
    "pp-csknow-grounding-evals-fidelity-metrics",
    "pp-csknow-zero-knowledge-fallback-polite-refusal",
    // Cluster 24 (Skill pp-csorch-hub-and-spoke-agent-architecture)
    "pp-csorch-hub-and-spoke-agent-architecture",
    "pp-csorch-bot-to-bot-context-passing",
    "pp-csorch-live-agent-handoff-omnichannel",
    "pp-csorch-third-party-live-agent-handoff",
    "pp-csorch-autonomous-subagent-consensus",
    "pp-csorch-voice-telephony-ivr-integration",
    "pp-csorch-conversation-transcripts-export-synapse",
    "pp-csorch-agent-lifecycle-solution-deployment",
    "pp-csorch-channel-security-directline-tokens",
    "pp-csorch-multi-lingual-agent-routing",
    // Cluster 25 (Skill pp-pcf-control-manifest-input-output-schema)
    "pp-pcf-control-manifest-input-output-schema",
    "pp-pcf-lifecycle-init-updateview-destroy",
    "pp-pcf-field-controls-vs-dataset-controls",
    "pp-pcf-context-parameters-formatting",
    "pp-pcf-virtual-controls-vs-standard",
    "pp-pcf-webapi-client-crud-operations",
    "pp-pcf-utility-navigation-lookup-dialogs",
    "pp-pcf-local-harness-debugging-test",
    "pp-pcf-solution-packaging-pac-pcf-push",
    "pp-pcf-security-sandboxing-iframe-restrictions",
    // Cluster 26 (Skill pp-pcfrx-fluent-ui-v9-react-components)
    "pp-pcfrx-fluent-ui-v9-react-components",
    "pp-pcfrx-typescript-strict-type-definitions",
    "pp-pcfrx-react-hooks-state-management",
    "pp-pcfrx-dataset-grid-custom-cell-renderers",
    "pp-pcfrx-custom-event-dispatching",
    "pp-pcfrx-theming-palette-sync-container",
    "pp-pcfrx-modal-drawer-overlay-management",
    "pp-pcfrx-complex-charting-recharts-d3",
    "pp-pcfrx-drag-and-drop-kanban-board",
    "pp-pcfrx-rich-text-markdown-editor",
    // Cluster 27 (Skill pp-pcfhard-barcode-scanner-camera-capture)
    "pp-pcfhard-barcode-scanner-camera-capture",
    "pp-pcfhard-geo-location-hardware-access",
    "pp-pcfhard-audio-recording-microphone",
    "pp-pcfhard-device-capabilities-feature-flags",
    "pp-pcfhard-webpack-bundle-splitting-optimization",
    "pp-pcfhard-css-isolation-shadow-dom",
    "pp-pcfhard-unit-testing-jest-react-testing-lib",
    "pp-pcfhard-bundle-analysis-tree-shaking",
    "pp-pcfhard-canvas-vs-model-driven-rendering-diffs",
    "pp-pcfhard-production-build-minification",
    // Cluster 28 (Skill pp-conn-openapi-swagger-v2-v3-schemas)
    "pp-conn-openapi-swagger-v2-v3-schemas",
    "pp-conn-x-ms-summary-visibility-metadata",
    "pp-conn-dynamic-values-and-dynamic-schema",
    "pp-conn-webhook-trigger-definitions",
    "pp-conn-polling-trigger-configuration",
    "pp-conn-csharp-policy-templates-transforms",
    "pp-conn-connector-error-codes-mapping",
    "pp-conn-pac-connector-cli-management",
    "pp-conn-connector-certification-pipeline",
    "pp-conn-mock-server-postman-testing",
    // Cluster 29 (Skill pp-oauth-oauth2-code-grant-flow)
    "pp-oauth-oauth2-code-grant-flow",
    "pp-oauth-entra-id-app-registration-scopes",
    "pp-oauth-pkce-proof-key-exchange",
    "pp-oauth-on-behalf-of-obo-token-exchange",
    "pp-oauth-api-key-and-basic-authentication",
    "pp-oauth-custom-token-refresh-handling",
    "pp-oauth-multi-tenant-connector-auth",
    "pp-oauth-mtls-client-certificates",
    "pp-oauth-service-principal-connection-sharing",
    "pp-oauth-credential-rotation-automation",
    // Cluster 30 (Skill pp-apim-export-to-power-platform-native)
    "pp-apim-export-to-power-platform-native",
    "pp-apim-inbound-outbound-xml-policies",
    "pp-apim-jwt-validation-claims-inspection",
    "pp-apim-mocking-responses-fast-prototyping",
    "pp-apim-caching-policies-redis-store",
    "pp-apim-backend-circuit-breaking-failover",
    "pp-apim-soap-to-rest-transformation",
    "pp-apim-request-correlation-tracking",
    "pp-apim-openapi-schema-normalization",
    "pp-apim-developer-portal-onboarding",
    // Cluster 31 (Skill pp-aib-prebuilt-invoice-receipt-processor)
    "pp-aib-prebuilt-invoice-receipt-processor",
    "pp-aib-id-reader-passport-license",
    "pp-aib-business-card-contact-extractor",
    "pp-aib-text-recognition-ocr-prebuilt",
    "pp-aib-sentiment-analysis-key-phrases",
    "pp-aib-language-detection-translation",
    "pp-aib-custom-document-model-training",
    "pp-aib-table-extraction-multi-page-docs",
    "pp-aib-confidence-score-threshold-routing",
    "pp-aib-credit-capacity-governance-admin",
    // Cluster 32 (Skill pp-aiprompt-prompt-engineering-studio)
    "pp-aiprompt-prompt-engineering-studio",
    "pp-aiprompt-grounding-with-dataverse-records",
    "pp-aiprompt-json-output-mode-guarantees",
    "pp-aiprompt-multimodal-vision-prompts",
    "pp-aiprompt-classification-categorization",
    "pp-aiprompt-entity-extraction-custom-entities",
    "pp-aiprompt-summarization-condensed-briefs",
    "pp-aiprompt-sentiment-and-intent-triage",
    "pp-aiprompt-safety-moderation-guardrails",
    "pp-aiprompt-alm-packaging-prompts-solutions",
    // Cluster 33 (Skill pp-vibe-conversational-prototyping-canvas)
    "pp-vibe-conversational-prototyping-canvas",
    "pp-vibe-tracer-bullet-solution-spikes",
    "pp-vibe-rapid-feedback-run-history",
    "pp-vibe-copilot-assisted-formula-synthesis",
    "pp-vibe-exploratory-spikes-mock-data",
    "pp-vibe-dialectical-design-agent-reviews",
    "pp-vibe-conversational-flow-state-ergonomics",
    "pp-vibe-fail-fast-diagnostic-telemetry",
    "pp-vibe-human-in-the-loop-steering",
    "pp-vibe-context-scaffolding-solutions",
    // Cluster 34 (Skill pp-alm-pac-cli-core-commands-auth)
    "pp-alm-pac-cli-core-commands-auth",
    "pp-alm-managed-vs-unmanaged-solutions",
    "pp-alm-solution-pack-unpack-source-control",
    "pp-alm-environment-variables-secrets",
    "pp-alm-connection-reference-binding-pipelines",
    "pp-alm-azure-devops-github-actions-pipelines",
    "pp-alm-solution-upgrade-vs-update",
    "pp-alm-solution-checker-static-analysis",
    "pp-alm-multi-environment-strategy-dev-test-prod",
    "pp-alm-configuration-data-migration-tool",
    // Cluster 35 (Skill pp-gov-dlp-policy-tiering-business-nonbusiness)
    "pp-gov-dlp-policy-tiering-business-nonbusiness",
    "pp-gov-connector-endpoint-filtering",
    "pp-gov-connector-action-control",
    "pp-gov-coe-starter-kit-core-components",
    "pp-gov-environment-lifecycle-management",
    "pp-gov-managed-environments-admin-controls",
    "pp-gov-tenant-isolation-inbound-outbound",
    "pp-gov-audit-logging-and-activity-reporting",
    "pp-gov-advisor-and-security-recommendations",
    "pp-gov-capacity-storage-quota-allocation",
];

// =========================================================================
// 1. Discovery of all 350 Power Platform Skills on Disk
// =========================================================================

#[test]
fn test_all_350_power_platform_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &POWER_PLATFORM_SKILLS_350 {
        assert!(
            loaded_map.contains(*skill),
            "Power Platform skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 350 Power Platform Skills (assert >= 995 total skills)
// =========================================================================

#[test]
fn test_all_350_power_platform_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 995,
        "Expected at least 995 total built-in skills including Power Platform suite, found {}",
        all_skills.len()
    );

    for skill_name in &POWER_PLATFORM_SKILLS_350 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Power Platform skill '{}' must be registered in built-in skills",
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
        ("fluid-container-layouts", "pp-canvas-fluid-container-layouts"),
        ("power-fx-imperative-vs-declarative", "pp-fx-imperative-vs-declarative"),
        ("component-properties", "pp-comp-custom-properties-input-output"),
        ("loaddata-savedata", "pp-offline-loaddata-savedata-local"),
        ("delegation-limits", "pp-perf-delegation-limits-workarounds"),
        ("standard-custom-tables", "pp-dv-standard-custom-activity-tables"),
        ("modern-app-designer", "pp-mda-modern-app-designer-sitemaps"),
        ("privilege-depth-matrix", "pp-sec-privilege-depth-matrix"),
        ("flow-trigger-types", "pp-flow-trigger-types-matrix"),
        ("try-catch-finally-scopes", "pp-err-try-catch-finally-scope-patterns"),
        ("approval-types-matrix", "pp-appr-approval-types-matrix"),
        ("odata-filter-query", "pp-odata-filter-query-syntax"),
        ("attended-vs-unattended-rpa", "pp-rpa-attended-vs-unattended-execution"),
        ("star-schema-modeling", "pp-pbi-star-schema-dimensional-modeling"),
        ("row-context-vs-filter-context", "pp-dax-row-context-vs-filter-context"),
        ("query-folding-diagnostics", "pp-m-query-folding-diagnostics"),
        ("linguistic-schema-synonyms", "pp-pbicop-linguistic-schema-synonyms"),
        ("site-architecture-web-templates", "pp-pages-site-architecture-web-templates"),
        ("portal-web-api-client", "pp-pweb-dataverse-web-api-portal-client"),
        ("bootstrap-5-customization", "pp-pstyle-bootstrap-5-customization"),
        ("intent-recognition-triggers", "pp-csgen-intent-recognition-trigger-phrases"),
        ("dynamic-chaining-actions", "pp-cschain-dynamic-chaining-generative-actions"),
        ("dataverse-search-grounding", "pp-csknow-dataverse-search-knowledge-indexing"),
        ("hub-and-spoke-multi-agent", "pp-csorch-hub-and-spoke-agent-architecture"),
        ("control-manifest-schema", "pp-pcf-control-manifest-input-output-schema"),
        ("fluent-ui-v9-pcf", "pp-pcfrx-fluent-ui-v9-react-components"),
        ("barcode-scanner-camera-pcf", "pp-pcfhard-barcode-scanner-camera-capture"),
        ("openapi-swagger-schemas", "pp-conn-openapi-swagger-v2-v3-schemas"),
        ("oauth2-code-grant-flow", "pp-oauth-oauth2-code-grant-flow"),
        ("export-to-power-platform", "pp-apim-export-to-power-platform-native"),
        ("prebuilt-invoice-receipt", "pp-aib-prebuilt-invoice-receipt-processor"),
        ("prompt-engineering-studio", "pp-aiprompt-prompt-engineering-studio"),
        ("conversational-prototyping-canvas", "pp-vibe-conversational-prototyping-canvas"),
        ("pac-cli-core-commands", "pp-alm-pac-cli-core-commands-auth"),
        ("dlp-policy-tiering", "pp-gov-dlp-policy-tiering-business-nonbusiness"),
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
    for skill_name in &POWER_PLATFORM_SKILLS_350 {
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
        ("fluid-container-layouts", "pp-canvas-fluid-container-layouts"),
        ("power-fx-imperative-vs-declarative", "pp-fx-imperative-vs-declarative"),
        ("component-libraries", "pp-comp-component-libraries-versioning"),
        ("loaddata-savedata", "pp-offline-loaddata-savedata-local"),
        ("delegation-limits", "pp-perf-delegation-limits-workarounds"),
        ("standard-custom-tables", "pp-dv-standard-custom-activity-tables"),
        ("modern-app-designer", "pp-mda-modern-app-designer-sitemaps"),
        ("privilege-depth-matrix", "pp-sec-privilege-depth-matrix"),
        ("flow-trigger-types", "pp-flow-trigger-types-matrix"),
        ("try-catch-finally-scopes", "pp-err-try-catch-finally-scope-patterns"),
        ("approval-types-matrix", "pp-appr-approval-types-matrix"),
        ("odata-filter-query", "pp-odata-filter-query-syntax"),
        ("attended-vs-unattended-rpa", "pp-rpa-attended-vs-unattended-execution"),
        ("star-schema-modeling", "pp-pbi-star-schema-dimensional-modeling"),
        ("row-context-vs-filter-context", "pp-dax-row-context-vs-filter-context"),
        ("query-folding-diagnostics", "pp-m-query-folding-diagnostics"),
        ("linguistic-schema-synonyms", "pp-pbicop-linguistic-schema-synonyms"),
        ("site-architecture-web-templates", "pp-pages-site-architecture-web-templates"),
        ("portal-web-api-client", "pp-pweb-dataverse-web-api-portal-client"),
        ("bootstrap-5-customization", "pp-pstyle-bootstrap-5-customization"),
        ("intent-recognition-triggers", "pp-csgen-intent-recognition-trigger-phrases"),
        ("dynamic-chaining-actions", "pp-cschain-dynamic-chaining-generative-actions"),
        ("dataverse-search-grounding", "pp-csknow-dataverse-search-knowledge-indexing"),
        ("hub-and-spoke-multi-agent", "pp-csorch-hub-and-spoke-agent-architecture"),
        ("control-manifest-schema", "pp-pcf-control-manifest-input-output-schema"),
        ("fluent-ui-v9-pcf", "pp-pcfrx-fluent-ui-v9-react-components"),
        ("barcode-scanner-camera-pcf", "pp-pcfhard-barcode-scanner-camera-capture"),
        ("openapi-swagger-schemas", "pp-conn-openapi-swagger-v2-v3-schemas"),
        ("oauth2-code-grant-flow", "pp-oauth-oauth2-code-grant-flow"),
        ("export-to-power-platform", "pp-apim-export-to-power-platform-native"),
        ("prebuilt-invoice-receipt", "pp-aib-prebuilt-invoice-receipt-processor"),
        ("prompt-engineering-studio", "pp-aiprompt-prompt-engineering-studio"),
        ("conversational-prototyping-canvas", "pp-vibe-conversational-prototyping-canvas"),
        ("pac-cli-core-commands", "pp-alm-pac-cli-core-commands-auth"),
        ("dlp-policy-tiering", "pp-gov-dlp-policy-tiering-business-nonbusiness"),
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
        "fluid-container-layouts",
        "power-fx-imperative-vs-declarative",
        "component-libraries",
        "loaddata-savedata",
        "delegation-limits",
        "standard-custom-tables",
        "modern-app-designer",
        "privilege-depth-matrix",
        "flow-trigger-types",
        "try-catch-finally-scopes",
        "approval-types-matrix",
        "odata-filter-query",
        "attended-vs-unattended-rpa",
        "star-schema-modeling",
        "row-context-vs-filter-context",
        "query-folding-diagnostics",
        "linguistic-schema-synonyms",
        "site-architecture-web-templates",
        "portal-web-api-client",
        "bootstrap-5-customization",
        "intent-recognition-triggers",
        "dynamic-chaining-actions",
        "dataverse-search-grounding",
        "hub-and-spoke-multi-agent",
        "control-manifest-schema",
        "fluent-ui-v9-pcf",
        "barcode-scanner-camera-pcf",
        "openapi-swagger-schemas",
        "oauth2-code-grant-flow",
        "export-to-power-platform",
        "prebuilt-invoice-receipt",
        "prompt-engineering-studio",
        "conversational-prototyping-canvas",
        "pac-cli-core-commands",
        "dlp-policy-tiering",
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
