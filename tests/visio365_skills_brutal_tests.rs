//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Visio 365 & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

pub const VISIO365_SKILLS_350: [&str; 350] = [
    // Cluster 1: ShapeSheet Architecture, Cell Formulas & Dependencies
    "visio-ss-core-formula-evaluation-order",
    "visio-ss-core-guard-formula-protection",
    "visio-ss-core-dependson-recalculation",
    "visio-ss-core-settarget-cross-shape-assignment",
    "visio-ss-core-cell-inheritance-hierarchies",
    "visio-ss-core-formula-recursion-prevention",
    "visio-ss-core-trigonometric-shape-math",
    "visio-ss-core-ref-error-recovery",
    "visio-ss-core-action-section-triggers",
    "visio-ss-core-parametric-dimension-binding",
    // Cluster 2: 2D/1D Geometry, Coordinate Systems & Transformation Matrices
    "visio-geom-pin-locpin-coordinate-transforms",
    "visio-geom-spline-nurbs-geometry-rows",
    "visio-geom-1d-vs-2d-behavior-switching",
    "visio-geom-multi-geometry-visibility",
    "visio-geom-affine-transformation-matrices",
    "visio-geom-bounding-box-auto-wrapping",
    "visio-geom-bezier-curvature-tension",
    "visio-geom-relative-vs-absolute-coords",
    "visio-geom-group-transform-propagation",
    "visio-geom-composite-multi-layer-shapes",
    // Cluster 3: Connection Points, Glue Mechanics & Dynamic Connectors
    "visio-connect-point-directions-and-types",
    "visio-connect-dynamic-routing-pathfinding",
    "visio-connect-walking-connectors-graph-traversal",
    "visio-connect-glueto-automation",
    "visio-connect-jumps-crossovers-and-bridges",
    "visio-connect-orthogonal-vs-curved-routing",
    "visio-connect-programmatic-rerouting-avoidance",
    "visio-connect-multi-point-bus-trunk-routing",
    "visio-connect-snap-glue-engine-configuration",
    "visio-connect-flow-direction-enforcement",
    // Cluster 4: Custom Properties, Shape Data & User-Defined Cells
    "visio-data-prop-schema-modeling",
    "visio-data-prop-user-defined-cells",
    "visio-data-prop-types-and-format-masks",
    "visio-data-prop-cascading-dropdown-dependencies",
    "visio-data-prop-inheritance-master-sync",
    "visio-data-prop-validation-input-sanitization",
    "visio-data-prop-harvesting-to-json-sql",
    "visio-data-prop-hidden-properties-ip-lock",
    "visio-data-prop-hyperlinks-section-engineering",
    "visio-data-prop-smart-tags-context-actions",
    // Cluster 5: Masters, Stencils (.vssx/.vstx) & Document Lifecycle
    "visio-stencil-vssx-vstm-package-format",
    "visio-stencil-master-inheritance-sync",
    "visio-stencil-enterprise-catalog-design",
    "visio-stencil-document-stencil-cleanup",
    "visio-stencil-multipage-background-overlays",
    "visio-stencil-vstx-template-standards",
    "visio-stencil-drawing-scales-metric-imperial",
    "visio-stencil-poster-printing-tiling",
    "visio-stencil-version-control-deprecations",
    "visio-stencil-msi-installer-packaging",
    // Cluster 6: Visio VBA Automation & Rapid Prototyping
    "visio-vba-core-object-model-navigation",
    "visio-vba-core-drop-draw-automation",
    "visio-vba-core-selection-spatial-queries",
    "visio-vba-core-master-dropping-coordinates",
    "visio-vba-core-shapesheet-cell-manipulation",
    "visio-vba-core-batch-csv-diagram-generation",
    "visio-vba-core-setformulas-getresults-arrays",
    "visio-vba-core-multipage-workbook-assembly",
    "visio-vba-core-userform-dialog-wizards",
    "visio-vba-core-legacy-macro-modernization",
    // Cluster 7: Automated Layout, Routing & Directed Graph Algorithms
    "visio-layout-directed-acyclic-graphs",
    "visio-layout-page-layout-built-in-engine",
    "visio-layout-custom-tree-generation",
    "visio-layout-force-directed-physics-simulation",
    "visio-layout-sugiyama-layered-layout",
    "visio-layout-orthogonal-astar-pathfinding",
    "visio-layout-radial-circular-topologies",
    "visio-layout-sankey-dynamic-connector-widths",
    "visio-layout-squarified-treemaps",
    "visio-layout-aesthetic-criteria-optimization",
    // Cluster 8: Events, Application Event Sinks & Marker Events
    "visio-event-iviseventproc-sink-dispatch",
    "visio-event-queuemarkerevent-async-tasks",
    "visio-event-mouse-interaction-hooks",
    "visio-event-cellchanged-formulachanged",
    "visio-event-window-selection-monitoring",
    "visio-event-connect-disconnect-validation",
    "visio-event-querycancel-deletion-guard",
    "visio-event-document-page-lifecycle",
    "visio-event-custom-undo-scopes",
    "visio-event-throttling-and-debouncing",
    // Cluster 9: C# / .NET VSTO Add-ins & High-Performance Interop
    "visio-vsto-add-in-lifecycle-architecture",
    "visio-vsto-com-memory-garbage-collection",
    "visio-vsto-wpf-custom-taskpane-binding",
    "visio-vsto-ribbonx-fluent-xml-integration",
    "visio-vsto-net-core-com-interop",
    "visio-vsto-multithreaded-background-queries",
    "visio-vsto-async-ui-dispatcher-marshaling",
    "visio-vsto-unit-testing-moq-isolation",
    "visio-vsto-clickonce-msi-packaging",
    "visio-vsto-diagnostic-telemetry-logging",
    // Cluster 10: XML Drawing Format (.vsdx) & Open Packaging Conventions (OPC)
    "visio-vsdx-xml-opc-zip-container-structure",
    "visio-vsdx-xml-page-xml-schema-deep-dive",
    "visio-vsdx-xml-headless-openxml-generation",
    "visio-vsdx-xml-raw-shapesheet-cell-patching",
    "visio-vsdx-xml-extracting-masters-and-stencils",
    "visio-vsdx-xml-streaming-search-and-replace",
    "visio-vsdx-xml-package-corruption-recovery",
    "visio-vsdx-xml-custom-xml-parts-embedding",
    "visio-vsdx-xml-legacy-vsd-migration-cli",
    "visio-vsdx-xml-package-minification-compression",
    // Cluster 11: Visio JavaScript API for Visio Online / Web
    "visio-js-web-application-page-classes",
    "visio-js-web-async-context-sync-queue",
    "visio-js-web-iframe-token-handshake",
    "visio-js-web-shape-selection-viewport-zoom",
    "visio-js-web-dynamic-highlight-badging",
    "visio-js-web-shape-data-hyperlinks-reading",
    "visio-js-web-selection-change-event-listeners",
    "visio-js-web-session-error-recovery",
    "visio-js-web-plan1-vs-plan2-boundaries",
    "visio-js-web-automated-walkthrough-presentation",
    // Cluster 12: Embedding Visio in React, Angular & Fluent UI
    "visio-embed-react-component-wrappers",
    "visio-embed-fluent-ui-react-panels",
    "visio-embed-bidirectional-state-sync",
    "visio-embed-angular-rxjs-event-streams",
    "visio-embed-msal-bearer-tokens",
    "visio-embed-responsive-breakpoint-scaling",
    "visio-embed-floating-html-tooltip-anchors",
    "visio-embed-nextjs-ssr-dashboards",
    "visio-embed-micro-frontend-visual-portals",
    "visio-embed-iframe-performance-profiling",
    // Cluster 13: SVG Generation, Path Optimization & Vector Rendering
    "visio-svg-export-viewbox-calibration",
    "visio-svg-path-sanitization-cleanup",
    "visio-svg-css-dark-mode-theming",
    "visio-svg-d3js-data-binding",
    "visio-svg-smil-css-flow-animations",
    "visio-svg-responsive-mobile-scaling",
    "visio-svg-font-inlining-woff2-assets",
    "visio-svg-headless-chromium-pdf-render",
    "visio-svg-svgo-path-decimation-opt",
    "visio-svg-aria-semantic-accessibility",
    // Cluster 14: Interactive Web Diagrams: Overlays, Tooltips & Click Handlers
    "visio-overlay-interactive-visual-interfaces",
    "visio-overlay-html-popovers-coordinate-anchors",
    "visio-overlay-canvas-svg-heatmaps",
    "visio-overlay-status-badging-pulse-indicators",
    "visio-overlay-multilevel-drilldown-navigation",
    "visio-overlay-minimap-viewport-synchronization",
    "visio-overlay-click-to-action-cloud-triggers",
    "visio-overlay-wcag-keyboard-tab-navigation",
    "visio-overlay-realtime-collaborative-cursors",
    "visio-overlay-mobile-touch-gesture-controls",
    // Cluster 15: Headless Diagram Synthesis via Node.js, Python & Canvas
    "visio-headless-python-vsdx-parsing",
    "visio-headless-nodejs-openxml-microservices",
    "visio-headless-powershell-server-automation",
    "visio-headless-mermaid-flowchart-translation",
    "visio-headless-plantuml-graphviz-dot-migration",
    "visio-headless-docker-containerized-rendering",
    "visio-headless-terraform-state-topology-maps",
    "visio-headless-sql-schema-to-erd-generation",
    "visio-headless-ci-cd-diagram-linting-gates",
    "visio-headless-massive-graph-streaming-opt",
    // Cluster 16: Data Linking, Link Data to Shapes & Recordsets
    "visio-data-link-recordset-architecture",
    "visio-data-link-autolink-matching-criteria",
    "visio-data-link-sql-server-oracle-odbc",
    "visio-data-link-excel-csv-tabular-sources",
    "visio-data-link-automatic-refresh-timers",
    "visio-data-link-primary-key-stability",
    "visio-data-link-rest-api-json-intermediaries",
    "visio-data-link-multisource-metric-merging",
    "visio-data-link-programmatic-creation-api",
    "visio-data-link-connection-timeout-troubleshooting",
    // Cluster 17: Data Graphics: Callouts, Data Bars & Color by Value
    "visio-data-gfx-visual-information-design",
    "visio-data-gfx-custom-callout-master-design",
    "visio-data-gfx-data-bars-progress-gauges",
    "visio-data-gfx-icon-sets-status-flags",
    "visio-data-gfx-color-by-value-heatmaps",
    "visio-data-gfx-programmatic-setdatagraphic",
    "visio-data-gfx-multimetric-corner-panels",
    "visio-data-gfx-conditional-number-format-masks",
    "visio-data-gfx-dynamic-legend-generation",
    "visio-data-gfx-high-volume-rendering-optimization",
    // Cluster 18: Visio Visual in Power BI: Interactive Drilldowns & Cross-Filtering
    "visio-pbi-visual-integration-architecture",
    "visio-pbi-bidirectional-cross-filtering",
    "visio-pbi-onedrive-sharepoint-hosting",
    "visio-pbi-floorplan-desk-occupancy-maps",
    "visio-pbi-supply-chain-warehouse-bottlenecks",
    "visio-pbi-manufacturing-assembly-line-scada",
    "visio-pbi-row-level-security-governance",
    "visio-pbi-mobile-report-layout-optimization",
    "visio-pbi-multipage-diagram-bookmark-sync",
    "visio-pbi-frame-rate-query-optimization",
    // Cluster 19: Excel to Visio: Data Visualizer & Automated Org Charts
    "visio-excel-dv-automated-flowchart-generation",
    "visio-excel-dv-custom-template-schema-design",
    "visio-excel-dv-swimlane-phase-mapping",
    "visio-excel-dv-automated-org-chart-hierarchies",
    "visio-excel-dv-python-workbook-builder",
    "visio-excel-dv-bidirectional-refresh-relinking",
    "visio-excel-dv-complex-looping-logic-patterns",
    "visio-excel-dv-enterprise-template-distribution",
    "visio-excel-dv-batch-process-map-cli-rendering",
    "visio-excel-dv-text-procedure-to-flowchart-ai",
    // Cluster 20: Real-Time IoT & Telemetry Visual Overlays
    "visio-iot-digital-twin-vector-displays",
    "visio-iot-datacenter-temperature-heatmaps",
    "visio-iot-scada-industrial-pipeline-alerts",
    "visio-iot-hvac-energy-consumption-dashboards",
    "visio-iot-high-frequency-websocket-streams",
    "visio-iot-alarm-threshold-escalation-states",
    "visio-iot-historical-telemetry-timeline-replay",
    "visio-iot-azure-digital-twins-dtdl-mapping",
    "visio-iot-soc-cybersecurity-attack-surfaces",
    "visio-iot-failsafe-telemetry-offline-caching",
    // Cluster 21: BPMN 2.0 & Workflow Diagram Modeling
    "visio-bpmn-modeling-and-style-foundations",
    "visio-bpmn-visio-native-stencil-components",
    "visio-bpmn-validation-rules-engine",
    "visio-bpmn-cross-functional-swimlane-pools",
    "visio-bpmn-exporting-to-camunda-power-automate",
    "visio-bpmn-subprocess-hierarchical-decomposition",
    "visio-bpmn-compensation-error-boundary-events",
    "visio-bpmn-conformance-checking-event-logs",
    "visio-bpmn-corporate-styleguide-enforcement",
    "visio-bpmn-spaghetti-flowchart-refactoring",
    // Cluster 22: Cloud Architecture Diagrams: Azure, AWS & GCP Topologies
    "visio-cloud-multiregion-system-design",
    "visio-cloud-azure-resource-graph-automation",
    "visio-cloud-azure-stencil-kit-standards",
    "visio-cloud-aws-well-architected-topologies",
    "visio-cloud-gcp-system-architecture-hubs",
    "visio-cloud-kubernetes-pod-service-ingress-mesh",
    "visio-cloud-terraform-state-infrastructure-maps",
    "visio-cloud-expressroute-directconnect-peering",
    "visio-cloud-finops-monthly-spend-badging",
    "visio-cloud-zero-trust-microsegmentation-nsgs",
    // Cluster 23: Enterprise IT & Network Topology Mapping
    "visio-net-logical-vs-physical-infrastructure",
    "visio-net-server-rack-elevation-plans",
    "visio-net-cisco-switch-ports-patch-panels",
    "visio-net-snmp-lldp-discovery-auto-mapping",
    "visio-net-datacenter-raised-floor-cable-trays",
    "visio-net-fiber-optic-splice-tray-schematics",
    "visio-net-active-directory-entra-forest-trusts",
    "visio-net-san-fabric-multipath-storage-luns",
    "visio-net-disaster-recovery-failover-routes",
    "visio-net-barcode-asset-tag-rack-dropping",
    // Cluster 24: UML 2.5, Software Architecture & Database ERDs
    "visio-uml-erd-standard-object-modeling",
    "visio-uml-erd-class-stereotypes-interfaces",
    "visio-uml-erd-reverse-engineering-sql-catalogs",
    "visio-uml-erd-crows-foot-cardinality-markers",
    "visio-uml-erd-sequence-lifelines-activation-boxes",
    "visio-uml-erd-microservices-component-deployment",
    "visio-uml-erd-composite-state-machine-guards",
    "visio-uml-erd-use-case-boundary-systems",
    "visio-uml-erd-exporting-erd-to-ddl-scripts",
    "visio-uml-erd-ddd-bounded-context-maps",
    // Cluster 25: Engineering & CAD: Floor Plans, P&ID & Electrical Schematics
    "visio-cad-eng-pid-instrumentation-diagrams",
    "visio-cad-eng-autocad-dwg-dxf-scaling-layers",
    "visio-cad-eng-architectural-walls-space-plans",
    "visio-cad-eng-electrical-single-line-circuits",
    "visio-cad-eng-hvac-ductwork-volume-calculations",
    "visio-cad-eng-security-cctv-sight-cone-plans",
    "visio-cad-eng-fire-safety-evacuation-routes",
    "visio-cad-eng-civil-boundary-landscape-plans",
    "visio-cad-eng-layer-isolation-complex-drawings",
    "visio-cad-eng-raster-blueprint-to-vector-shapes",
    // Cluster 26: Power Automate Workflows for Diagram Automation
    "visio-pa-flow-event-triggered-diagram-generation",
    "visio-pa-flow-sharepoint-list-modification-updates",
    "visio-pa-flow-microsoft-forms-survey-to-nodes",
    "visio-pa-flow-rpa-desktop-client-automation",
    "visio-pa-flow-inspecting-vsdx-xml-inside-flows",
    "visio-pa-flow-workflow-routing-driven-by-diagram",
    "visio-pa-flow-servicenow-incident-topology-maps",
    "visio-pa-flow-concurrency-locks-retry-policies",
    "visio-pa-flow-onedrive-vsdx-to-pdf-conversion",
    "visio-pa-flow-dlp-governance-service-accounts",
    // Cluster 27: Microsoft Graph API & Visio File Metadata
    "visio-graph-driveitem-diagram-crud-ops",
    "visio-graph-v1-drive-upload-download-versions",
    "visio-graph-thumbnail-preview-extraction",
    "visio-graph-kql-indexing-search-queries",
    "visio-graph-coauthoring-file-lock-management",
    "visio-graph-delta-queries-change-detection",
    "visio-graph-sharing-link-permission-scopes",
    "visio-graph-purview-sensitivity-retention-labels",
    "visio-graph-webhook-change-notifications",
    "visio-graph-dotnet-sdk-diagram-portals",
    // Cluster 28: SharePoint Server / Online Visio Services & Web Parts
    "visio-sp-serv-vds-rendering-engine-architecture",
    "visio-sp-serv-modern-page-file-viewer-webpart",
    "visio-sp-serv-spfx-react-visio-js-webparts",
    "visio-sp-serv-corporate-intranet-org-directories",
    "visio-sp-serv-secure-store-service-credentials",
    "visio-sp-serv-migrating-legacy-vwd-silverlight",
    "visio-sp-serv-shape-data-to-list-column-sync",
    "visio-sp-serv-user-interaction-audit-logging",
    "visio-sp-serv-multilingual-intranet-translations",
    "visio-sp-serv-multigeo-tenant-disaster-recovery",
    // Cluster 29: Microsoft Teams Meeting Collaboration & Whiteboard Bridges
    "visio-teams-channel-tab-coauthoring-sync",
    "visio-teams-meeting-live-share-synchronized-view",
    "visio-teams-whiteboard-canvas-export-bridge",
    "visio-teams-ai-architecture-review-chatbot",
    "visio-teams-adaptive-card-approval-prompts",
    "visio-teams-planner-todo-task-assignment",
    "visio-teams-threaded-node-conversations",
    "visio-teams-mobile-client-blueprint-reviews",
    "visio-teams-private-channel-permission-fences",
    "visio-teams-collaborative-inking-pointer-stream",
    // Cluster 30: AI Copilots & Natural Language to Visio Diagram Synthesis
    "visio-ai-copilot-prompt-to-vector-spec-pipeline",
    "visio-ai-copilot-copilot-studio-architecture-bot",
    "visio-ai-copilot-transcript-to-bpmn-translation",
    "visio-ai-copilot-layout-overlap-auto-alignment",
    "visio-ai-copilot-technical-manual-to-knowledge-graph",
    "visio-ai-copilot-audio-summary-alt-text-generation",
    "visio-ai-copilot-rag-over-shape-topology-vectors",
    "visio-ai-copilot-vision-schematic-anomaly-detection",
    "visio-ai-copilot-autonomous-subpage-decomposition",
    "visio-ai-copilot-flow-state-visual-workbench",
    // Cluster 31: Diagram Validation Rules, Custom Rule Sets & Issues
    "visio-valid-rulesets-and-issues-architecture",
    "visio-valid-bpmn-custom-ruleset-authoring",
    "visio-valid-network-schematic-rule-enforcement",
    "visio-valid-document-page-shape-scopes",
    "visio-valid-programmatic-issue-auto-healing",
    "visio-valid-ci-cd-quality-gate-pipeline",
    "visio-valid-issue-overrides-compliance-waivers",
    "visio-valid-dynamic-canvas-error-rings",
    "visio-valid-excel-compliance-report-export",
    "visio-valid-ieee-circuit-standards-validation",
    // Cluster 32: Enterprise Stencil Governance, Brand Compliance & Asset CDNs
    "visio-gov-centralized-catalog-lifecycle",
    "visio-gov-azure-cdn-workstation-distribution",
    "visio-gov-brand-colors-typography-locks",
    "visio-gov-shapesheet-ip-protection-locks",
    "visio-gov-legacy-master-guid-migration",
    "visio-gov-rbac-security-cleared-schematics",
    "visio-gov-multilingual-global-translations",
    "visio-gov-nightly-stencil-integrity-tests",
    "visio-gov-search-keyword-index-optimization",
    "visio-gov-self-service-gallery-portal",
    // Cluster 33: Git Version Control for Diagrams: Diffing, Merging & CI/CD
    "visio-git-xml-deconstruction-architecture",
    "visio-git-visual-pr-image-diff-overlays",
    "visio-git-lfs-attributes-binary-tracking",
    "visio-git-github-actions-png-preview-render",
    "visio-git-modular-page-splitting-branches",
    "visio-git-semver-release-tagging-blueprints",
    "visio-git-precommit-metadata-scrubbing-hooks",
    "visio-git-docusaurus-doc-portal-sync",
    "visio-git-deconstructed-xml-conflict-resolution",
    "visio-git-confluence-wiki-auto-deploy",
    // Cluster 34: Vibe Coding for Visio: Spec-Driven Generation & Flow-State Prototyping
    "visio-vibe-flow-state-visual-programming",
    "visio-vibe-spec-driven-markdown-scaffolding",
    "visio-vibe-prompt-engineering-visual-topologies",
    "visio-vibe-conversational-voice-refinement",
    "visio-vibe-napkin-sketch-to-visio-in-30-mins",
    "visio-vibe-legacy-drawing-modernization-agent",
    "visio-vibe-stride-threat-model-generation",
    "visio-vibe-opentelemetry-sequence-diagrams",
    "visio-vibe-semantic-kernel-diagramming-agents",
    "visio-vibe-zen-effortless-architecture-flow",
    // Cluster 35: Enterprise Deployment, Licensing, Packaging & High-Volume Export
    "visio-deploy-headless-batch-pdf-svg-export",
    "visio-deploy-high-dpi-600-raster-rendering",
    "visio-deploy-pdf-book-assembly-hyperlink-trees",
    "visio-deploy-cmyk-rgb-color-calibration-press",
    "visio-deploy-automated-security-watermarking",
    "visio-deploy-office-deployment-tool-odt-silent",
    "visio-deploy-licensing-tier-cost-optimization",
    "visio-deploy-kubernetes-diagram-microservices",
    "visio-deploy-pdfa-iso-19005-archival-standards",
    "visio-deploy-cad-omnigraffle-legacy-migration",
];

// =========================================================================
// 1. Discovery of all 350 Visio 365 Skills on Disk
// =========================================================================

#[test]
fn test_all_350_visio365_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &VISIO365_SKILLS_350 {
        assert!(
            loaded_map.contains(*skill),
            "Visio 365 skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 350 Visio 365 Skills (assert >= 2045 total skills)
// =========================================================================

#[test]
fn test_all_350_visio365_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 2045,
        "Expected at least 2045 total built-in skills including Visio 365 suite, found {}",
        all_skills.len()
    );

    for skill_name in &VISIO365_SKILLS_350 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Visio 365 skill '{}' must be registered in built-in skills",
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
        ("shapesheet-evaluation-order", "visio-ss-core-formula-evaluation-order"),
        ("pinx-piny-transforms", "visio-geom-pin-locpin-coordinate-transforms"),
        ("connection-point-types", "visio-connect-point-directions-and-types"),
        ("shape-data-schema", "visio-data-prop-schema-modeling"),
        ("vssx-stencil-packages", "visio-stencil-vssx-vstm-package-format"),
        ("visio-object-model", "visio-vba-core-object-model-navigation"),
        ("dag-layout-algorithms", "visio-layout-directed-acyclic-graphs"),
        ("iviseventproc-sink", "visio-event-iviseventproc-sink-dispatch"),
        ("vsto-add-in-lifecycle", "visio-vsto-add-in-lifecycle-architecture"),
        ("vsdx-opc-structure", "visio-vsdx-xml-opc-zip-container-structure"),
        ("visio-js-classes", "visio-js-web-application-page-classes"),
        ("react-visio-wrapper", "visio-embed-react-component-wrappers"),
        ("export-svg-visio", "visio-svg-export-viewbox-calibration"),
        ("interactive-blueprints", "visio-overlay-interactive-visual-interfaces"),
        ("python-vsdx-library", "visio-headless-python-vsdx-parsing"),
        ("datarecordsets-architecture", "visio-data-link-recordset-architecture"),
        ("data-graphics-design", "visio-data-gfx-visual-information-design"),
        ("powerbi-visio-visual", "visio-pbi-visual-integration-architecture"),
        ("data-visualizer-flowcharts", "visio-excel-dv-automated-flowchart-generation"),
        ("digital-twin-visio", "visio-iot-digital-twin-vector-displays"),
        ("bpmn-style-foundations", "visio-bpmn-modeling-and-style-foundations"),
        ("multiregion-cloud-design", "visio-cloud-multiregion-system-design"),
        ("logical-vs-physical-net", "visio-net-logical-vs-physical-infrastructure"),
        ("uml25-object-modeling", "visio-uml-erd-standard-object-modeling"),
        ("pid-instrumentation-visio", "visio-cad-eng-pid-instrumentation-diagrams"),
        ("flow-triggered-diagrams", "visio-pa-flow-event-triggered-diagram-generation"),
        ("graph-driveitem-visio", "visio-graph-driveitem-diagram-crud-ops"),
        ("vds-rendering-engine", "visio-sp-serv-vds-rendering-engine-architecture"),
        ("teams-channel-visio-tab", "visio-teams-channel-tab-coauthoring-sync"),
        ("prompt-to-diagram-pipeline", "visio-ai-copilot-prompt-to-vector-spec-pipeline"),
        ("validation-rulesets-visio", "visio-valid-rulesets-and-issues-architecture"),
        ("centralized-stencil-catalog", "visio-gov-centralized-catalog-lifecycle"),
        ("vsdx-git-deconstruction", "visio-git-xml-deconstruction-architecture"),
        ("flow-state-visual-vibe", "visio-vibe-flow-state-visual-programming"),
        ("headless-batch-export", "visio-deploy-headless-batch-pdf-svg-export"),
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
    for skill_name in &VISIO365_SKILLS_350 {
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
        ("shapesheet-evaluation-order", "visio-ss-core-formula-evaluation-order"),
        ("pinx-piny-transforms", "visio-geom-pin-locpin-coordinate-transforms"),
        ("connection-point-types", "visio-connect-point-directions-and-types"),
        ("shape-data-schema", "visio-data-prop-schema-modeling"),
        ("vssx-stencil-packages", "visio-stencil-vssx-vstm-package-format"),
        ("visio-object-model", "visio-vba-core-object-model-navigation"),
        ("dag-layout-algorithms", "visio-layout-directed-acyclic-graphs"),
        ("iviseventproc-sink", "visio-event-iviseventproc-sink-dispatch"),
        ("vsto-add-in-lifecycle", "visio-vsto-add-in-lifecycle-architecture"),
        ("vsdx-opc-structure", "visio-vsdx-xml-opc-zip-container-structure"),
        ("visio-js-classes", "visio-js-web-application-page-classes"),
        ("react-visio-wrapper", "visio-embed-react-component-wrappers"),
        ("export-svg-visio", "visio-svg-export-viewbox-calibration"),
        ("interactive-blueprints", "visio-overlay-interactive-visual-interfaces"),
        ("python-vsdx-library", "visio-headless-python-vsdx-parsing"),
        ("datarecordsets-architecture", "visio-data-link-recordset-architecture"),
        ("data-graphics-design", "visio-data-gfx-visual-information-design"),
        ("powerbi-visio-visual", "visio-pbi-visual-integration-architecture"),
        ("data-visualizer-flowcharts", "visio-excel-dv-automated-flowchart-generation"),
        ("digital-twin-visio", "visio-iot-digital-twin-vector-displays"),
        ("bpmn-style-foundations", "visio-bpmn-modeling-and-style-foundations"),
        ("multiregion-cloud-design", "visio-cloud-multiregion-system-design"),
        ("logical-vs-physical-net", "visio-net-logical-vs-physical-infrastructure"),
        ("uml25-object-modeling", "visio-uml-erd-standard-object-modeling"),
        ("pid-instrumentation-visio", "visio-cad-eng-pid-instrumentation-diagrams"),
        ("flow-triggered-diagrams", "visio-pa-flow-event-triggered-diagram-generation"),
        ("graph-driveitem-visio", "visio-graph-driveitem-diagram-crud-ops"),
        ("vds-rendering-engine", "visio-sp-serv-vds-rendering-engine-architecture"),
        ("teams-channel-visio-tab", "visio-teams-channel-tab-coauthoring-sync"),
        ("prompt-to-diagram-pipeline", "visio-ai-copilot-prompt-to-vector-spec-pipeline"),
        ("validation-rulesets-visio", "visio-valid-rulesets-and-issues-architecture"),
        ("centralized-stencil-catalog", "visio-gov-centralized-catalog-lifecycle"),
        ("vsdx-git-deconstruction", "visio-git-xml-deconstruction-architecture"),
        ("flow-state-visual-vibe", "visio-vibe-flow-state-visual-programming"),
        ("headless-batch-export", "visio-deploy-headless-batch-pdf-svg-export"),
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
        "shapesheet-evaluation-order",
        "pinx-piny-transforms",
        "connection-point-types",
        "shape-data-schema",
        "vssx-stencil-packages",
        "visio-object-model",
        "dag-layout-algorithms",
        "iviseventproc-sink",
        "vsto-add-in-lifecycle",
        "vsdx-opc-structure",
        "visio-js-classes",
        "react-visio-wrapper",
        "export-svg-visio",
        "interactive-blueprints",
        "python-vsdx-library",
        "datarecordsets-architecture",
        "data-graphics-design",
        "powerbi-visio-visual",
        "data-visualizer-flowcharts",
        "digital-twin-visio",
        "bpmn-style-foundations",
        "multiregion-cloud-design",
        "logical-vs-physical-net",
        "uml25-object-modeling",
        "pid-instrumentation-visio",
        "flow-triggered-diagrams",
        "graph-driveitem-visio",
        "vds-rendering-engine",
        "teams-channel-visio-tab",
        "prompt-to-diagram-pipeline",
        "validation-rulesets-visio",
        "centralized-stencil-catalog",
        "vsdx-git-deconstruction",
        "flow-state-visual-vibe",
        "headless-batch-export",
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
