//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Access 365 & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

pub const ACCESS365_SKILLS_350: [&str; 350] = [
    // Cluster 1 (Skill acc-schema-relational-theory-3nf)
    "acc-schema-relational-theory-3nf",
    "acc-schema-surrogate-vs-natural-keys",
    "acc-schema-domain-integrity-constraints",
    "acc-schema-table-normalization-patterns",
    "acc-schema-null-handling-antipatterns",
    "acc-schema-order-inventory-design",
    "acc-schema-functional-dependency-isolation",
    "acc-schema-reusable-entity-templates",
    "acc-schema-evolutionary-refactoring",
    "acc-schema-candidate-key-enforcement",
    // Cluster 2 (Skill acc-ace-4kb-storage-page-geometry)
    "acc-ace-4kb-storage-page-geometry",
    "acc-ace-2gb-boundary-navigation",
    "acc-ace-database-bloat-mitigation",
    "acc-ace-index-b-tree-defragmentation",
    "acc-ace-lob-memo-page-chains",
    "acc-ace-laccdb-lock-mechanics",
    "acc-ace-unicode-ucs2-compression",
    "acc-ace-synchronous-disk-writes",
    "acc-ace-accdb-binary-header-forensics",
    "acc-ace-multi-valued-complex-types",
    // Cluster 3 (Skill acc-sql-ddl-create-table-constraints)
    "acc-sql-ddl-create-table-constraints",
    "acc-sql-ddl-alter-table-migrations",
    "acc-sql-ddl-querydef-parameter-compilation",
    "acc-sql-ddl-action-query-transactions",
    "acc-sql-ddl-jet-syntax-conventions",
    "acc-sql-ddl-expression-builtins-mastery",
    "acc-sql-ddl-showplan-execution-optimizer",
    "acc-sql-ddl-composite-unique-indexes",
    "acc-sql-ddl-runtime-injection-defense",
    "acc-sql-ddl-table-validation-rules",
    // Cluster 4 (Skill acc-sql-adv-correlated-subqueries)
    "acc-sql-adv-correlated-subqueries",
    "acc-sql-adv-transform-pivot-crosstab",
    "acc-sql-adv-union-heterogeneous-merging",
    "acc-sql-adv-domain-aggregates-vs-sql",
    "acc-sql-adv-top-n-ranking-subqueries",
    "acc-sql-adv-cartesian-tally-tables",
    "acc-sql-adv-string-concatenation-aggregation",
    "acc-sql-adv-pass-through-sql-queries",
    "acc-sql-adv-gap-and-island-detection",
    "acc-sql-adv-temporal-fiscal-rollups",
    // Cluster 5 (Skill acc-rel-foreign-key-referential-integrity)
    "acc-rel-foreign-key-referential-integrity",
    "acc-rel-cascading-updates-and-deletes",
    "acc-rel-junction-table-patterns",
    "acc-rel-self-referencing-hierarchies",
    "acc-rel-table-level-lookup-elimination",
    "acc-rel-multi-user-lock-contention",
    "acc-rel-unmatched-record-forensics",
    "acc-rel-composite-foreign-keys",
    "acc-rel-schema-topology-documentation",
    "acc-rel-autonumber-propagation-safety",
    // Cluster 6 (Skill acc-vba-core-modular-code-organization)
    "acc-vba-core-modular-code-organization",
    "acc-vba-core-variable-typing-and-scoping",
    "acc-vba-core-control-flow-and-iteration",
    "acc-vba-core-array-memory-processing",
    "acc-vba-core-screen-echo-suppression",
    "acc-vba-core-docmd-clean-abstractions",
    "acc-vba-core-vibe-prompt-pair-programming",
    "acc-vba-core-string-builder-optimizations",
    "acc-vba-core-date-time-serial-arithmetic",
    "acc-vba-core-legacy-macro-modernization",
    // Cluster 7 (Skill acc-vba-err-structured-error-handlers)
    "acc-vba-err-structured-error-handlers",
    "acc-vba-err-global-telemetry-logging",
    "acc-vba-err-call-stack-tracking-erl",
    "acc-vba-err-assert-precondition-contracts",
    "acc-vba-err-network-drop-graceful-recovery",
    "acc-vba-err-transaction-rollback-guards",
    "acc-vba-err-silent-error-reporting-emails",
    "acc-vba-err-type-mismatch-coercion-safety",
    "acc-vba-err-self-healing-compact-repair",
    "acc-vba-err-user-friendly-exception-dialogs",
    // Cluster 8 (Skill acc-vba-oop-property-get-let-set)
    "acc-vba-oop-property-get-let-set",
    "acc-vba-oop-implements-polymorphism",
    "acc-vba-oop-custom-collections-dictionaries",
    "acc-vba-oop-withevents-ui-decoupling",
    "acc-vba-oop-factory-and-builder-patterns",
    "acc-vba-oop-state-machine-workflows",
    "acc-vba-oop-teardown-circular-references",
    "acc-vba-oop-model-view-controller-mvc",
    "acc-vba-oop-custom-control-wrappers",
    "acc-vba-oop-dependency-injection-testing",
    // Cluster 9 (Skill acc-win32-ptrsafe-64bit-declarations)
    "acc-win32-ptrsafe-64bit-declarations",
    "acc-win32-file-open-save-dialogs",
    "acc-win32-window-styles-and-subclassing",
    "acc-win32-high-resolution-timers",
    "acc-win32-registry-access-advapi32",
    "acc-win32-gdi32-graphics-rendering",
    "acc-win32-rtlmovememory-byte-copy",
    "acc-win32-process-execution-monitoring",
    "acc-win32-system-metrics-monitor-dpi",
    "acc-win32-credential-manager-dpapi",
    // Cluster 10 (Skill acc-str-json-regex-pattern-matching)
    "acc-str-json-regex-pattern-matching",
    "acc-str-json-vba-json-serialization",
    "acc-str-json-high-speed-string-builder",
    "acc-str-json-csv-delimited-stream-parsing",
    "acc-str-json-utf8-unicode-conversions",
    "acc-str-json-msxml2-dom-xpath-queries",
    "acc-str-json-pii-data-masking",
    "acc-str-json-binary-stream-io",
    "acc-str-json-fuzzy-soundex-levenshtein",
    "acc-str-json-mustache-token-templating",
    // Cluster 11 (Skill acc-dao-cursor-selection-dynaset-snapshot)
    "acc-dao-cursor-selection-dynaset-snapshot",
    "acc-dao-batch-inserts-append-only",
    "acc-dao-programmatic-tabledef-schema",
    "acc-dao-querydef-parameter-bindings",
    "acc-dao-seek-vs-findfirst-benchmarks",
    "acc-dao-recordset-clone-and-bookmarks",
    "acc-dao-attachment-and-multivalued-fields",
    "acc-dao-custom-workspace-isolation",
    "acc-dao-recordset-reentrant-traversal",
    "acc-dao-com-reference-garbage-collection",
    // Cluster 12 (Skill acc-ado-disconnected-recordsets)
    "acc-ado-disconnected-recordsets",
    "acc-ado-command-parameters-stored-procs",
    "acc-ado-binding-forms-to-recordsets",
    "acc-ado-stream-object-blob-storage",
    "acc-ado-asynchronous-query-execution",
    "acc-ado-in-memory-filter-and-sort",
    "acc-ado-hierarchical-data-shaping",
    "acc-ado-native-provider-error-collection",
    "acc-ado-transaction-savepoints",
    "acc-ado-migrating-dao-to-ado-matrix",
    // Cluster 13 (Skill acc-odbc-dsn-less-connection-strings)
    "acc-odbc-dsn-less-connection-strings",
    "acc-odbc-connection-pooling-mechanics",
    "acc-odbc-automated-tabledef-relinking",
    "acc-odbc-tls13-encrypted-connections",
    "acc-odbc-pass-through-query-optimization",
    "acc-odbc-query-and-connection-timeouts",
    "acc-odbc-heterogeneous-cloud-backends",
    "acc-odbc-identity-autonumber-handling",
    "acc-odbc-zero-config-startup-relinker",
    "acc-odbc-sql-profiler-trace-auditing",
    // Cluster 14 (Skill acc-oledb-ace-provider-architecture)
    "acc-oledb-ace-provider-architecture",
    "acc-oledb-connecting-sql-server-msoledbsql",
    "acc-oledb-universal-connection-catalog",
    "acc-oledb-reading-excel-spreadsheets",
    "acc-oledb-keyset-vs-dynamic-cursors",
    "acc-oledb-querying-csv-and-parquet",
    "acc-oledb-windows-integrated-security",
    "acc-oledb-schema-rowset-catalog-mining",
    "acc-oledb-64bit-office-driver-registry",
    "acc-oledb-bulk-row-copy-pipelines",
    // Cluster 15 (Skill acc-concurr-no-locks-vs-edited-record)
    "acc-concurr-no-locks-vs-edited-record",
    "acc-concurr-optimistic-write-conflict-7787",
    "acc-concurr-laccdb-lock-contention-auditing",
    "acc-concurr-sql-server-timestamp-rowversion",
    "acc-concurr-deadlock-detection-retry-loops",
    "acc-concurr-smb3-directory-leasing-bugs",
    "acc-concurr-custom-soft-locking-sessions",
    "acc-concurr-programmatic-conflict-overwrites",
    "acc-concurr-50-user-multi-threaded-stress",
    "acc-concurr-read-only-snapshot-isolation",
    // Cluster 16 (Skill acc-form-life-event-sequence-architecture)
    "acc-form-life-event-sequence-architecture",
    "acc-form-life-beforeupdate-cancellation",
    "acc-form-life-afterupdate-dirty-tracking",
    "acc-form-life-keyboard-hotkey-interception",
    "acc-form-life-search-as-you-type-filters",
    "acc-form-life-modal-dialog-wizard-flows",
    "acc-form-life-thin-form-code-behind",
    "acc-form-life-tab-control-lazy-loading",
    "acc-form-life-dirty-flag-close-protection",
    "acc-form-life-custom-navigation-steppers",
    // Cluster 17 (Skill acc-subform-master-child-link-fields)
    "acc-subform-master-child-link-fields",
    "acc-subform-continuous-conditional-formatting",
    "acc-subform-emulated-grid-actions",
    "acc-subform-nested-three-tier-editing",
    "acc-subform-skeleton-screen-delayed-bind",
    "acc-subform-multi-select-checkbox-arrays",
    "acc-subform-aggregate-footer-summaries",
    "acc-subform-datasheet-view-customization",
    "acc-subform-split-form-orientation",
    "acc-subform-api-continuous-scrolling",
    // Cluster 18 (Skill acc-report-section-layout-mastery)
    "acc-report-section-layout-mastery",
    "acc-report-two-pass-page-n-of-m",
    "acc-report-dynamic-runtime-grouping",
    "acc-report-conditional-formatting-print",
    "acc-report-nested-subreports-binding",
    "acc-report-pdf-and-excel-export-pipelines",
    "acc-report-multi-page-invoicing-layouts",
    "acc-report-barcode-and-qr-code-printing",
    "acc-report-cangrow-blank-line-suppression",
    "acc-report-interactive-drilldown-views",
    // Cluster 19 (Skill acc-ribbon-customui-xml-schema)
    "acc-ribbon-customui-xml-schema",
    "acc-ribbon-usysribbons-table-storage",
    "acc-ribbon-vba-callbacks-invalidation",
    "acc-ribbon-custom-icons-image-mso",
    "acc-ribbon-contextual-tabs-activation",
    "acc-ribbon-custom-shortcut-context-menus",
    "acc-ribbon-backstage-view-customization",
    "acc-ribbon-rbac-dynamic-button-security",
    "acc-ribbon-migrating-commandbars-to-xml",
    "acc-ribbon-schema-validation-tooling",
    // Cluster 20 (Skill acc-ui-modern-fluent-design-system)
    "acc-ui-modern-fluent-design-system",
    "acc-ui-modern-true-dark-mode-theming",
    "acc-ui-modern-high-dpi-per-monitor-v2",
    "acc-ui-modern-responsive-control-anchors",
    "acc-ui-modern-custom-toast-notifications",
    "acc-ui-modern-svg-vector-iconography",
    "acc-ui-modern-animated-loading-spinners",
    "acc-ui-modern-styled-modal-message-boxes",
    "acc-ui-modern-office-theme-integration",
    "acc-ui-modern-tablet-touch-ergonomics",
    // Cluster 21 (Skill acc-azure-sql-ssma-migration-architecture)
    "acc-azure-sql-ssma-migration-architecture",
    "acc-azure-sql-data-type-conversions",
    "acc-azure-sql-latency-minimization",
    "acc-azure-sql-firewall-and-private-endpoints",
    "acc-azure-sql-case-sensitivity-collation",
    "acc-azure-sql-views-with-pseudo-indexes",
    "acc-azure-sql-stored-procedure-delegation",
    "acc-azure-sql-powershell-ssma-automation",
    "acc-azure-sql-post-migration-checksum-audit",
    "acc-azure-sql-cost-and-tier-sizing",
    // Cluster 22 (Skill acc-dataverse-linking-tables-architecture)
    "acc-dataverse-linking-tables-architecture",
    "acc-dataverse-migration-to-teams",
    "acc-dataverse-power-apps-hybrid-frontends",
    "acc-dataverse-power-automate-event-triggers",
    "acc-dataverse-solution-lifecycle-management",
    "acc-dataverse-row-level-security-roles",
    "acc-dataverse-power-bi-directquery-models",
    "acc-dataverse-polymorphic-lookups-choice-sets",
    "acc-dataverse-offline-mobile-sync-patterns",
    "acc-dataverse-licensing-tco-evaluation",
    // Cluster 23 (Skill acc-split-db-fe-be-partitioning)
    "acc-split-db-fe-be-partitioning",
    "acc-split-db-automated-environment-relinker",
    "acc-split-db-client-auto-updater-scripts",
    "acc-split-db-scheduled-midnight-compact-backup",
    "acc-split-db-multi-backend-partitioning",
    "acc-split-db-vpn-performance-remediation",
    "acc-split-db-azure-virtual-desktop-hosting",
    "acc-split-db-in-memory-lookup-caching",
    "acc-split-db-single-instance-enforcement",
    "acc-split-db-disaster-recovery-restoration",
    // Cluster 24 (Skill acc-open-db-postgresql-psqlodbc-integration)
    "acc-open-db-postgresql-psqlodbc-integration",
    "acc-open-db-mysql-mariadb-connector",
    "acc-open-db-boolean-timestamp-translations",
    "acc-open-db-postgres-jsonb-querying",
    "acc-open-db-amazon-rds-google-cloud-sql",
    "acc-open-db-serial-sequence-alignment",
    "acc-open-db-plpgsql-stored-routines",
    "acc-open-db-ssl-tls-certificate-pinning",
    "acc-open-db-cloud-data-warehouse-queries",
    "acc-open-db-zero-license-enterprise-tier",
    // Cluster 25 (Skill acc-rest-api-serverxmlhttp-client)
    "acc-rest-api-serverxmlhttp-client",
    "acc-rest-api-reusable-http-wrapper-class",
    "acc-rest-api-nested-json-relational-insert",
    "acc-rest-api-webhook-polling-and-listening",
    "acc-rest-api-payment-gateway-integration",
    "acc-rest-api-openai-claude-llm-calls",
    "acc-rest-api-windows-curl-cli-execution",
    "acc-rest-api-s3-blob-storage-uploads",
    "acc-rest-api-rate-limit-exponential-backoff",
    "acc-rest-api-graphql-queries-from-vba",
    // Cluster 26 (Skill acc-excel-com-automation-application)
    "acc-excel-com-automation-application",
    "acc-excel-copyfromrecordset-fast-export",
    "acc-excel-power-query-m-ingestion",
    "acc-excel-headless-oledb-sheet-queries",
    "acc-excel-dynamic-pivottable-generation",
    "acc-excel-bidirectional-data-reconciliation",
    "acc-excel-large-dataset-chunking",
    "acc-excel-template-driven-reporting",
    "acc-excel-orphan-com-process-cleanup",
    "acc-excel-power-bi-gateway-integration",
    // Cluster 27 (Skill acc-outlook-com-automation-mailitems)
    "acc-outlook-com-automation-mailitems",
    "acc-outlook-html-body-styled-templates",
    "acc-outlook-throttled-batch-dispatch",
    "acc-outlook-inbox-scraping-attachment-import",
    "acc-outlook-calendar-appointment-sync",
    "acc-outlook-contacts-and-task-sync",
    "acc-outlook-mapi-security-dialog-bypass",
    "acc-outlook-exchange-direct-graph-mail",
    "acc-outlook-msg-file-extraction-and-storage",
    "acc-outlook-shared-mailbox-ticket-routing",
    // Cluster 28 (Skill acc-word-com-automation-documents)
    "acc-word-com-automation-documents",
    "acc-word-advanced-mail-merge-pipelines",
    "acc-word-dynamic-contract-assembly",
    "acc-word-headless-openxml-docx-generation",
    "acc-word-dynamic-image-signature-injection",
    "acc-word-export-to-pdf-archival",
    "acc-word-custom-xml-parts-binding",
    "acc-word-programmatic-table-formatting",
    "acc-word-batch-certificate-generation",
    "acc-word-template-corruption-sanitization",
    // Cluster 29 (Skill acc-sharepoint-linked-lists-architecture)
    "acc-sharepoint-linked-lists-architecture",
    "acc-sharepoint-offline-caching-and-sync",
    "acc-sharepoint-5000-item-threshold-bypass",
    "acc-sharepoint-complex-field-type-handling",
    "acc-sharepoint-document-library-attachments",
    "acc-sharepoint-table-export-migration",
    "acc-sharepoint-fine-grained-permissions",
    "acc-sharepoint-power-automate-triggers",
    "acc-sharepoint-rest-api-batch-operations",
    "acc-sharepoint-azure-sql-comparison-matrix",
    // Cluster 30 (Skill acc-graph-rest-api-access-desktop)
    "acc-graph-rest-api-access-desktop",
    "acc-graph-oauth2-msal-authentication",
    "acc-graph-sendmail-rest-pipeline",
    "acc-graph-onedrive-file-sync-and-shares",
    "acc-graph-user-profile-entra-id-lookups",
    "acc-graph-teams-channel-notifications",
    "acc-graph-planner-task-management",
    "acc-graph-certificate-based-client-auth",
    "acc-graph-silent-token-refresh-loop",
    "acc-graph-delta-queries-and-change-tracking",
    // Cluster 31 (Skill acc-sec-accdb-database-password-aes)
    "acc-sec-accdb-database-password-aes",
    "acc-sec-accde-executable-compilation",
    "acc-sec-bitlocker-volume-encryption",
    "acc-sec-allowbypasskey-shift-lockdown",
    "acc-sec-legacy-mdw-workgroup-removal",
    "acc-sec-role-based-access-control-rbac",
    "acc-sec-vba-obfuscation-and-ip-protection",
    "acc-sec-sql-injection-prevention-audit",
    "acc-sec-windows-credential-manager-dpapi",
    "acc-sec-comprehensive-audit-logging",
    // Cluster 32 (Skill acc-git-rd-source-control-architecture)
    "acc-git-rd-source-control-architecture",
    "acc-git-rd-git-workflow-branch-hygiene",
    "acc-git-rd-msaccess-vcs-integration",
    "acc-git-rd-rubberduck-vba-static-code-analysis",
    "acc-git-rd-saveastext-loadfromtext-automation",
    "acc-git-rd-pre-commit-hook-linting",
    "acc-git-rd-semantic-versioning-tagging",
    "acc-git-rd-multi-developer-collaboration",
    "acc-git-rd-vba-code-review-standards",
    "acc-git-rd-resolving-form-layout-conflicts",
    // Cluster 33 (Skill acc-test-ci-tdd-rubberduck-testing)
    "acc-test-ci-tdd-rubberduck-testing",
    "acc-test-ci-pure-vba-unit-test-harness",
    "acc-test-ci-integration-testing-crud",
    "acc-test-ci-github-actions-automation",
    "acc-test-ci-winappdriver-ui-automation",
    "acc-test-ci-mocking-database-repositories",
    "acc-test-ci-regression-testing-data-parity",
    "acc-test-ci-code-coverage-and-dead-code",
    "acc-test-ci-performance-regression-benchmarks",
    "acc-test-ci-smoke-testing-32bit-64bit",
    // Cluster 34 (Skill acc-vibe-flow-state-desktop-development)
    "acc-vibe-flow-state-desktop-development",
    "acc-vibe-prompt-engineering-for-vba",
    "acc-vibe-conversational-tdd-access",
    "acc-vibe-spec-driven-schema-scaffolding",
    "acc-vibe-legacy-code-archeology",
    "acc-vibe-autonomous-access-copilot-taskpane",
    "acc-vibe-automated-data-dictionary-generation",
    "acc-vibe-napkin-sketch-to-app-prototyping",
    "acc-vibe-cryptic-error-decoding-agents",
    "acc-vibe-future-desktop-database-workbench",
    // Cluster 35 (Skill acc-deploy-free-access-runtime-packaging)
    "acc-deploy-free-access-runtime-packaging",
    "acc-deploy-bulletproof-client-auto-updater",
    "acc-deploy-inno-setup-desktop-installers",
    "acc-deploy-trusted-locations-macro-security",
    "acc-deploy-side-by-side-office-coexistence",
    "acc-deploy-group-policy-gpo-distribution",
    "acc-deploy-application-crash-telemetry",
    "acc-deploy-digital-code-signing-authenticode",
    "acc-deploy-multi-language-localization",
    "acc-deploy-decommissioning-and-data-archival",
];

// =========================================================================
// 1. Discovery of all 350 Access 365 Skills on Disk
// =========================================================================

#[test]
fn test_all_350_access365_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &ACCESS365_SKILLS_350 {
        assert!(
            loaded_map.contains(*skill),
            "Access 365 skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 350 Access 365 Skills (assert >= 1695 total skills)
// =========================================================================

#[test]
fn test_all_350_access365_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 1695,
        "Expected at least 1695 total built-in skills including Access 365 suite, found {}",
        all_skills.len()
    );

    for skill_name in &ACCESS365_SKILLS_350 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Access 365 skill '{}' must be registered in built-in skills",
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
    }
}

// =========================================================================
// 3. High-Leverage Alias Resolution Across All 35 Clusters
// =========================================================================

#[test]
fn test_high_leverage_alias_lookups() {
    let alias_cases = [
        ("relational-3nf", "acc-schema-relational-theory-3nf"),
        ("storage-pages", "acc-ace-4kb-storage-page-geometry"),
        ("jet-ddl-create", "acc-sql-ddl-create-table-constraints"),
        ("correlated-subqueries", "acc-sql-adv-correlated-subqueries"),
        ("foreign-keys", "acc-rel-foreign-key-referential-integrity"),
        ("option-explicit", "acc-vba-core-modular-code-organization"),
        ("structured-error-handling", "acc-vba-err-structured-error-handlers"),
        ("property-get-let", "acc-vba-oop-property-get-let-set"),
        ("ptrsafe-64bit", "acc-win32-ptrsafe-64bit-declarations"),
        ("vbscript-regexp", "acc-str-json-regex-pattern-matching"),
        ("dao-cursors", "acc-dao-cursor-selection-dynaset-snapshot"),
        ("disconnected-recordsets", "acc-ado-disconnected-recordsets"),
        ("dsn-less-connections", "acc-odbc-dsn-less-connection-strings"),
        ("ace-oledb-provider", "acc-oledb-ace-provider-architecture"),
        ("no-locks-property", "acc-concurr-no-locks-vs-edited-record"),
        ("form-event-sequence", "acc-form-life-event-sequence-architecture"),
        ("linkmasterfields", "acc-subform-master-child-link-fields"),
        ("report-sections", "acc-report-section-layout-mastery"),
        ("customui-xml", "acc-ribbon-customui-xml-schema"),
        ("fluent-design-access", "acc-ui-modern-fluent-design-system"),
        ("ssma-migration", "acc-azure-sql-ssma-migration-architecture"),
        ("dataverse-linked-tables", "acc-dataverse-linking-tables-architecture"),
        ("split-database-architecture", "acc-split-db-fe-be-partitioning"),
        ("postgresql-access", "acc-open-db-postgresql-psqlodbc-integration"),
        ("serverxmlhttp-vba", "acc-rest-api-serverxmlhttp-client"),
        ("excel-com-automation", "acc-excel-com-automation-application"),
        ("outlook-com-automation", "acc-outlook-com-automation-mailitems"),
        ("word-com-automation", "acc-word-com-automation-documents"),
        ("sharepoint-linked-lists", "acc-sharepoint-linked-lists-architecture"),
        ("graph-api-access", "acc-graph-rest-api-access-desktop"),
        ("accdb-encryption", "acc-sec-accdb-database-password-aes"),
        ("access-source-control", "acc-git-rd-source-control-architecture"),
        ("tdd-rubberduck", "acc-test-ci-tdd-rubberduck-testing"),
        ("flow-state-vibe", "acc-vibe-flow-state-desktop-development"),
        ("access-runtime-packaging", "acc-deploy-free-access-runtime-packaging"),
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
    for skill_name in &ACCESS365_SKILLS_350 {
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
        ("relational-3nf", "acc-schema-relational-theory-3nf"),
        ("storage-pages", "acc-ace-4kb-storage-page-geometry"),
        ("jet-ddl-create", "acc-sql-ddl-create-table-constraints"),
        ("correlated-subqueries", "acc-sql-adv-correlated-subqueries"),
        ("foreign-keys", "acc-rel-foreign-key-referential-integrity"),
        ("option-explicit", "acc-vba-core-modular-code-organization"),
        ("structured-error-handling", "acc-vba-err-structured-error-handlers"),
        ("property-get-let", "acc-vba-oop-property-get-let-set"),
        ("ptrsafe-64bit", "acc-win32-ptrsafe-64bit-declarations"),
        ("vbscript-regexp", "acc-str-json-regex-pattern-matching"),
        ("dao-cursors", "acc-dao-cursor-selection-dynaset-snapshot"),
        ("disconnected-recordsets", "acc-ado-disconnected-recordsets"),
        ("dsn-less-connections", "acc-odbc-dsn-less-connection-strings"),
        ("ace-oledb-provider", "acc-oledb-ace-provider-architecture"),
        ("no-locks-property", "acc-concurr-no-locks-vs-edited-record"),
        ("form-event-sequence", "acc-form-life-event-sequence-architecture"),
        ("linkmasterfields", "acc-subform-master-child-link-fields"),
        ("report-sections", "acc-report-section-layout-mastery"),
        ("customui-xml", "acc-ribbon-customui-xml-schema"),
        ("fluent-design-access", "acc-ui-modern-fluent-design-system"),
        ("ssma-migration", "acc-azure-sql-ssma-migration-architecture"),
        ("dataverse-linked-tables", "acc-dataverse-linking-tables-architecture"),
        ("split-database-architecture", "acc-split-db-fe-be-partitioning"),
        ("postgresql-access", "acc-open-db-postgresql-psqlodbc-integration"),
        ("serverxmlhttp-vba", "acc-rest-api-serverxmlhttp-client"),
        ("excel-com-automation", "acc-excel-com-automation-application"),
        ("outlook-com-automation", "acc-outlook-com-automation-mailitems"),
        ("word-com-automation", "acc-word-com-automation-documents"),
        ("sharepoint-linked-lists", "acc-sharepoint-linked-lists-architecture"),
        ("graph-api-access", "acc-graph-rest-api-access-desktop"),
        ("accdb-encryption", "acc-sec-accdb-database-password-aes"),
        ("access-source-control", "acc-git-rd-source-control-architecture"),
        ("tdd-rubberduck", "acc-test-ci-tdd-rubberduck-testing"),
        ("flow-state-vibe", "acc-vibe-flow-state-desktop-development"),
        ("access-runtime-packaging", "acc-deploy-free-access-runtime-packaging"),
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
        "relational-3nf",
        "storage-pages",
        "jet-ddl-create",
        "correlated-subqueries",
        "foreign-keys",
        "option-explicit",
        "structured-error-handling",
        "property-get-let",
        "ptrsafe-64bit",
        "vbscript-regexp",
        "dao-cursors",
        "disconnected-recordsets",
        "dsn-less-connections",
        "ace-oledb-provider",
        "no-locks-property",
        "form-event-sequence",
        "linkmasterfields",
        "report-sections",
        "customui-xml",
        "fluent-design-access",
        "ssma-migration",
        "dataverse-linked-tables",
        "split-database-architecture",
        "postgresql-access",
        "serverxmlhttp-vba",
        "excel-com-automation",
        "outlook-com-automation",
        "word-com-automation",
        "sharepoint-linked-lists",
        "graph-api-access",
        "accdb-encryption",
        "access-source-control",
        "tdd-rubberduck",
        "flow-state-vibe",
        "access-runtime-packaging",
    ]);

    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    let num_threads = 50;
    let iterations_per_thread = 20;

    let start = Instant::now();

    for t in 0..num_threads {
        let d = Arc::clone(&dispatcher);
        let q = Arc::clone(&queries);
        let s = Arc::clone(&success_count);

        let handle = thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let query = q[(t * iterations_per_thread + i) % q.len()];
                let res = d.dispatch(query, 3, None);
                if !res.is_empty() {
                    s.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked in stress test");
    }

    let elapsed = start.elapsed();
    let total = success_count.load(Ordering::Relaxed);

    println!(
        "Concurrent stress test: {} dispatches across {} threads completed in {:?} ({:.2} us/dispatch)",
        total,
        num_threads,
        elapsed,
        (elapsed.as_micros() as f64) / (total as f64)
    );

    assert_eq!(
        total,
        num_threads * iterations_per_thread,
        "All {} dispatches must succeed",
        num_threads * iterations_per_thread
    );
}
