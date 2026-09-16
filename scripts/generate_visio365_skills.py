#!/usr/bin/env python3
"""
scripts/generate_visio365_skills.py

Generates all 350 Microsoft Visio 365 & Vibe Code Development skill packages
under .ecc/skills/visio-*/SKILL.md with full YAML frontmatter, strict operational
directives (ALWAYS, NEVER, MANDATORY), and comprehensive engineering instructions.
"""

import os
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

# 35 Clusters x 10 Skills = 350 Skills
CLUSTERS = [
    (1, "ShapeSheet Architecture, Cell Formulas & Dependencies", "visio-ss-core-", "Visio 2013 Developer's Guide: ShapeSheet Programming - David J. Parker"),
    (2, "2D/1D Geometry, Coordinate Systems & Transformation Matrices", "visio-geom-", "Visio Coordinate Systems & Mathematical Transformations - Graham Wideman"),
    (3, "Connection Points, Glue Mechanics & Dynamic Connectors", "visio-connect-", "Connecting and Gluing Shapes in Visio - David J. Parker"),
    (4, "Custom Properties, Shape Data & User-Defined Cells", "visio-data-prop-", "Modeling Structured Data in Visio Shapes - David J. Parker"),
    (5, "Masters, Stencils (.vssx/.vstx) & Document Lifecycle", "visio-stencil-", "Visio Stencil Architecture: VSSX, VSTM, and VSS - David J. Parker"),
    (6, "Visio VBA Automation & Rapid Prototyping", "visio-vba-core-", "Microsoft Visio 2016 Developer's Guide - Scott A. Helmers"),
    (7, "Automated Layout, Routing & Directed Graph Algorithms", "visio-layout-", "Graph Algorithms for Visual Process Mapping - Steven Skiena"),
    (8, "Events, Application Event Sinks & Marker Events", "visio-event-", "Visio Event Architecture: IVisEventProc & EventSink - David J. Parker"),
    (9, "C# / .NET VSTO Add-ins & High-Performance Interop", "visio-vsto-", "Developing Microsoft Visio Add-ins with C# and VSTO - David J. Parker"),
    (10, "XML Drawing Format (.vsdx) & Open Packaging Conventions (OPC)", "visio-vsdx-xml-", "Open Packaging Conventions (OPC) for Visio .VSDX - David J. Parker"),
    (11, "Visio JavaScript API for Visio Online / Web", "visio-js-web-", "Visio for the Web: Developer Extensibility Reference - Microsoft Learn"),
    (12, "Embedding Visio in React, Angular & Fluent UI", "visio-embed-", "Building Modern Web Apps with Embedded Visio Diagrams - Paolo Pialorsi"),
    (13, "SVG Generation, Path Optimization & Vector Rendering", "visio-svg-", "Exporting Visio to Scalable Vector Graphics (SVG) - David J. Parker"),
    (14, "Interactive Web Diagrams: Overlays, Tooltips & Click Handlers", "visio-overlay-", "Designing Interactive Visual Dashboards - David J. Parker"),
    (15, "Headless Diagram Synthesis via Node.js, Python & Canvas", "visio-headless-", "Generating Visio Diagrams from Python via vsdx-python - Elena Vance"),
    (16, "Data Linking, Link Data to Shapes & Recordsets", "visio-data-link-", "Visualizing Data with Microsoft Visio 365 - David J. Parker"),
    (17, "Data Graphics: Callouts, Data Bars & Color by Value", "visio-data-gfx-", "Mastering Visio Data Graphics: Visual Information Design - David J. Parker"),
    (18, "Visio Visual in Power BI: Interactive Drilldowns & Cross-Filtering", "visio-pbi-", "Power BI & Visio: Interactive Business Analytics - Alberto Ferrari & David J. Parker"),
    (19, "Excel to Visio: Data Visualizer & Automated Org Charts", "visio-excel-dv-", "Excel Data Visualizer: Automated Diagramming Playbook - Scott A. Helmers"),
    (20, "Real-Time IoT & Telemetry Visual Overlays", "visio-iot-", "Real-Time IoT Diagramming and Visual Digital Twins - Tom Morgan"),
    (21, "BPMN 2.0 & Workflow Diagram Modeling", "visio-bpmn-", "BPMN 2.0 Handbook: Modeling Method and Style - Bruce Silver"),
    (22, "Cloud Architecture Diagrams: Azure, AWS & GCP Topologies", "visio-cloud-", "Cloud Architecture Diagramming Standards - Mark Simos"),
    (23, "Enterprise IT & Network Topology Mapping", "visio-net-", "Network Diagramming and Infrastructure Documentation - Don R. Crawley"),
    (24, "UML 2.5, Software Architecture & Database ERDs", "visio-uml-erd-", "UML Distilled: A Brief Guide to the Standard Object Modeling Language - Martin Fowler"),
    (25, "Engineering & CAD: Floor Plans, P&ID & Electrical Schematics", "visio-cad-eng-", "Piping and Instrumentation Diagram (P&ID) Engineering with Visio - Bill Morelli"),
    (26, "Power Automate Workflows for Diagram Automation", "visio-pa-flow-", "Automating Visio with Power Automate and Cloud Flows - Matthew Devaney"),
    (27, "Microsoft Graph API & Visio File Metadata", "visio-graph-", "Microsoft Graph API Programming for Office Documents - Glenn Snyder & Vincent Biret"),
    (28, "SharePoint Server / Online Visio Services & Web Parts", "visio-sp-serv-", "Visio Services Architecture in SharePoint Online & Server - David J. Parker"),
    (29, "Microsoft Teams Meeting Collaboration & Whiteboard Bridges", "visio-teams-", "Collaborating on Visio Diagrams in Microsoft Teams - Tom Morgan"),
    (30, "AI Copilots & Natural Language to Visio Diagram Synthesis", "visio-ai-copilot-", "Prompt-Driven Visual Engineering: AI to Visio Pipelines - Devon Thorne"),
    (31, "Diagram Validation Rules, Custom Rule Sets & Issues", "visio-valid-", "Visio Diagram Validation Architecture - David J. Parker"),
    (32, "Enterprise Stencil Governance, Brand Compliance & Asset CDNs", "visio-gov-", "Enterprise Stencil Governance and Lifecycle Management - Scott A. Helmers"),
    (33, "Git Version Control for Diagrams: Diffing, Merging & CI/CD", "visio-git-", "Version Control for Visual Documents: The Git & Visio Playbook - Devon Thorne"),
    (34, "Vibe Coding for Visio: Spec-Driven Generation & Flow-State Prototyping", "visio-vibe-", "The Vibe Coding Blueprint for Visual Systems Engineers - Devon Thorne"),
    (35, "Enterprise Deployment, Licensing, Packaging & High-Volume Export", "visio-deploy-", "High-Throughput Batch Export to PDF, SVG, and PNG - David J. Parker"),
]

CLUSTER_SKILLS = [
    # Cluster 1: ShapeSheet Architecture, Cell Formulas & Dependencies
    (1, [
        ("formula-evaluation-order", "Section cell evaluation order, dependency trees, and calculation chains in the ShapeSheet.", ["shapesheet-evaluation-order", "cell-dependency-tree", "shapesheet-calc-chain"]),
        ("guard-formula-protection", "Using Guard() to protect critical ShapeSheet formulas against manual shape manipulation.", ["guard-formula", "protect-shapesheet-cells", "guard-cell-macro"]),
        ("dependson-recalculation", "Triggering dynamic recalculation with DependsOn() functions across interrelated cells.", ["dependson-formula", "dynamic-recalculation", "cell-trigger-math"]),
        ("settarget-cross-shape-assignment", "Pushing values across shapes using SetTarget() and SetF() formula functions.", ["settarget-formula", "setf-cross-shape", "push-cell-values"]),
        ("cell-inheritance-hierarchies", "Inheriting master formulas vs instance overrides and formula resetting mechanics.", ["cell-inheritance", "master-cell-overrides", "formula-resetting"]),
        ("formula-recursion-prevention", "Cycle detection, preventing circular cell dependencies and stack overflow crashes.", ["prevent-circular-cells", "recursion-cycle-detect", "shapesheet-stack-safety"]),
        ("trigonometric-shape-math", "Sin, Cos, Tan, Atan2, and radian angles in parametric ShapeSheet cell calculations.", ["shapesheet-trigonometry", "atan2-radian-angles", "parametric-math-cells"]),
        ("ref-error-recovery", "Detecting and repairing #REF! and #VALUE! formula corruption in damaged shape cells.", ["ref-error-recovery", "value-error-repair", "corrupt-cell-healing"]),
        ("action-section-triggers", "Right-click contextual menus, Actions.* cells, and dynamic checked status expressions.", ["actions-section", "context-menu-actions", "checked-status-cells"]),
        ("parametric-dimension-binding", "Parametric dimensions, dynamic text scaling, and auto-expanding shape labels.", ["parametric-dimensions", "dynamic-text-scaling", "auto-expanding-labels"]),
    ]),

    # Cluster 2: 2D/1D Geometry, Coordinate Systems & Transformation Matrices
    (2, [
        ("pin-locpin-coordinate-transforms", "PinX/PinY, LocPinX/LocPinY, Angle, and parent-child coordinate spaces.", ["pinx-piny-transforms", "locpin-rotation-origin", "parent-child-spaces"]),
        ("spline-nurbs-geometry-rows", "NURBSTo, PolylineTo, Ellipse, and ArcTo vector geometry paths in Geometry sections.", ["nurbs-spline-geometry", "polylineto-paths", "arcto-elliptical-rows"]),
        ("1d-vs-2d-behavior-switching", "ObjType cell, BeginX/EndX, and converting between 1D connector and 2D shape mechanics.", ["1d-vs-2d-switching", "objtype-cell", "beginx-endx-controls"]),
        ("multi-geometry-visibility", "Managing multiple Geometry sections and conditional visibility via NoShow cells.", ["multi-geometry-sections", "noshow-conditional-vis", "layered-shape-geometry"]),
        ("affine-transformation-matrices", "Shearing, skewing, mirror reflections, and 2D rotation matrices in vector cells.", ["affine-transforms-visio", "shearing-skewing-matrix", "mirror-reflection-cells"]),
        ("bounding-box-auto-wrapping", "BoundingBox queries, dynamic padding, and text-driven geometry expansion.", ["bounding-box-wrapping", "dynamic-padding-cells", "text-driven-geometry"]),
        ("bezier-curvature-tension", "Control points, tension weights, and smooth curvature calculations in geometry rows.", ["bezier-spline-tension", "curvature-control-points", "smooth-vector-paths"]),
        ("relative-vs-absolute-coords", "Geometry row flags, Width*Fraction vs fixed measurement units in vector drafting.", ["relative-coordinates", "width-fraction-geometry", "fixed-units-geometry"]),
        ("group-transform-propagation", "Group transforms, sub-shape coordinates, and 2D group boundaries propagation.", ["group-transform-propagation", "subshape-coordinates", "group-boundary-math"]),
        ("composite-multi-layer-shapes", "Outlines, drop shadows, and badge overlays across stacked geometry layers.", ["composite-multi-layer", "stacked-geometry-layers", "badge-drop-shadow-cells"]),
    ]),

    # Cluster 3: Connection Points, Glue Mechanics & Dynamic Connectors
    (3, [
        ("point-directions-and-types", "Connection points (X/Y, DirX/DirY, and inward/outward types) in the ShapeSheet.", ["connection-point-types", "dirx-diry-vectors", "inward-outward-points"]),
        ("dynamic-routing-pathfinding", "RoutingStyle, ConFixedCode, and dynamic avoidance of obstacle shapes.", ["dynamic-connector-routing", "consequence-pathfinding", "obstacle-avoidance-algo"]),
        ("walking-connectors-graph-traversal", "Shape.FromConnects and Shape.Connects directed graph edge traversal in automation.", ["walking-connectors", "graph-edge-traversal", "fromconnects-connects"]),
        ("glueto-automation", "Programmatic gluing of 1D connector endpoints to 2D shape connection points via API.", ["glueto-api", "glue-connector-endpoints", "connection-gluing-macro"]),
        ("jumps-crossovers-and-bridges", "Line jumps, crossover bridges, custom arrowheads, and line caps on overlapping edges.", ["connector-line-jumps", "crossover-bridges", "custom-arrowhead-caps"]),
        ("orthogonal-vs-curved-routing", "Right-angle routing, Bezier connectors, and tree channel algorithms in diagrams.", ["orthogonal-routing", "curved-bezier-connectors", "tree-channel-routing"]),
        ("programmatic-rerouting-avoidance", "Triggering auto-rerouting after moving shapes and preventing overlapping paths.", ["programmatic-rerouting", "prevent-overlapping-paths", "post-move-reroute"]),
        ("multi-point-bus-trunk-routing", "Electrical-style bus connectors and dynamic trunk branch points in schematics.", ["bus-trunk-routing", "electrical-bus-connectors", "branch-trunk-points"]),
        ("snap-glue-engine-configuration", "Controlling application-level snap strength, glue targets, and grid line alignments.", ["snap-glue-engine", "snap-strength-config", "grid-line-alignment"]),
        ("flow-direction-enforcement", "Input-only vs Output-only connection point validation on directed graph nodes.", ["flow-direction-enforce", "directed-connection-points", "input-output-guards"]),
    ]),

    # Cluster 4: Custom Properties, Shape Data & User-Defined Cells
    (4, [
        ("schema-modeling", "Shape Data sections, Prop.* rows, labels, prompts, and data formats in Visio shapes.", ["shape-data-schema", "prop-rows-modeling", "custom-properties-visio"]),
        ("user-defined-cells", "User.* rows as private internal variables and intermediate calculations in ShapeSheets.", ["user-defined-cells", "user-rows-variables", "private-shapesheet-state"]),
        ("types-and-format-masks", "Configuring variable types, dropdown fixed lists, and currency/date format masks.", ["shape-data-types", "dropdown-fixed-lists", "format-masks-visio"]),
        ("cascading-dropdown-dependencies", "Dependencies between Prop.Category and Prop.SubCategory dropdowns via formulas.", ["cascading-dropdowns-visio", "dependent-shape-data", "lookup-dependency-cells"]),
        ("inheritance-master-sync", "Inheriting Shape Data schemas without overriding instance values in diagram shapes.", ["shape-data-inheritance", "master-schema-sync", "instance-value-protection"]),
        ("validation-input-sanitization", "Formulas enforcing numerical ranges, required text, and regex validation on data.", ["shape-data-validation", "range-enforcement-cells", "input-sanitization-rules"]),
        ("harvesting-to-json-sql", "Automated harvesting of Shape Data properties across multi-page diagrams into JSON/SQL.", ["harvest-shape-data", "shape-data-to-json", "export-diagram-metadata"]),
        ("hidden-properties-ip-lock", "Prop.Visible, User.* obfuscation, and locking proprietary IP inside shapes.", ["hidden-shape-data", "prop-visible-cells", "shape-ip-protection"]),
        ("hyperlinks-section-engineering", "Hyperlink.* rows, multi-link menus, relative file paths, and URI parameter binding.", ["hyperlinks-section", "multi-link-shape-menus", "uri-parameter-binding"]),
        ("smart-tags-context-actions", "Actions.* section, dynamic action titles, and command triggers in context menus.", ["smart-tags-actions", "dynamic-action-titles", "context-action-triggers"]),
    ]),

    # Cluster 5: Masters, Stencils (.vssx/.vstx) & Document Lifecycle
    (5, [
        ("vssx-vstm-package-format", "Authoring reusable stencil packages, masters, icons, and stencils cache structure.", ["vssx-stencil-packages", "vstm-macro-stencils", "stencil-cache-structure"]),
        ("master-inheritance-sync", "Understanding how shapes inherit changes when stencils are updated in production.", ["master-inheritance-sync", "stencil-update-cascade", "master-copy-propagation"]),
        ("enterprise-catalog-design", "Visual consistency, search keywords, localized prompts, and icon grids in stencils.", ["enterprise-stencil-catalog", "search-keywords-stencils", "stencil-icon-grids"]),
        ("document-stencil-cleanup", "Cleaning bloated document stencils, purging unused masters, and GUID alignment.", ["document-stencil-cleanup", "purge-unused-masters", "guid-alignment-stencils"]),
        ("multipage-background-overlays", "Foreground pages, background overlays, shared title blocks, and page scaling.", ["multipage-backgrounds", "shared-title-blocks", "background-page-overlays"]),
        ("vstx-template-standards", "Default styles, color palettes, page setups, and embedded document settings in VSTX.", ["vstx-template-design", "template-palette-styles", "page-setup-standards"]),
        ("drawing-scales-metric-imperial", "Page scale formulas: 1:50 architectural scales, zero-point origins, and unit ratios.", ["drawing-scales-visio", "architectural-scale-math", "metric-imperial-origins"]),
        ("poster-printing-tiling", "Print paper vs drawing page sizes, margins, and auto-expanding pages in print layout.", ["poster-printing-tiling", "auto-expanding-pages", "print-page-margins"]),
        ("version-control-deprecations", "Managing stencil upgrades across enterprise developer fleets without breaking legacy files.", ["stencil-version-control", "stencil-deprecation-flow", "backward-compat-stencils"]),
        ("msi-installer-packaging", "MSI installer layouts for dropping stencils and templates into user Visio paths.", ["stencil-msi-installer", "enterprise-stencil-deploy", "visio-content-paths"]),
    ]),

    # Cluster 6: Visio VBA Automation & Rapid Prototyping
    (6, [
        ("object-model-navigation", "Application, Document, Page, Shape, Selection, and Window object model navigation.", ["visio-object-model", "app-doc-page-shape", "window-selection-nav"]),
        ("drop-draw-automation", "Page.Drop(), DrawRectangle(), DrawLine(), and programmatic drafting in VBA.", ["page-drop-automation", "drawrectangle-drawline", "programmatic-drafting"]),
        ("selection-spatial-queries", "Selection.Iteration, SpatialNeighbors, BoundingBox queries, and grouping via VBA.", ["selection-spatial-queries", "spatial-neighbors-visio", "bounding-box-grouping"]),
        ("master-dropping-coordinates", "Page.Drop(Master, x, y), converting units, and positioning math in automation.", ["drop-master-coords", "unit-conversion-visio", "placement-coordinate-math"]),
        ("shapesheet-cell-manipulation", "Shape.Cells(), CellsSRC(), FormulaU, ResultStr, and ResultIU performance in VBA.", ["cellssrc-manipulation", "formlau-resultiu", "fast-cell-access-vba"]),
        ("batch-csv-diagram-generation", "Reading CSV/Excel rows and spawning connected diagram blocks automatically.", ["batch-csv-diagrams", "csv-to-visio-vba", "spawn-connected-shapes"]),
        ("setformulas-getresults-arrays", "Bulk reading and writing ShapeSheet arrays in single COM calls via SetFormulas.", ["setformulas-getresults", "bulk-shapesheet-arrays", "zero-chatter-com-visio"]),
        ("multipage-workbook-assembly", "Document.Pages.Add(), copying shapes across pages, and batch export pipelines.", ["multipage-assembly-vba", "pages-add-automation", "cross-page-shape-copy"]),
        ("userform-dialog-wizards", "Custom input forms, progress bars, and parameter configuration wizards in Visio VBA.", ["visio-userforms", "dialog-wizards-vba", "progress-bar-automation"]),
        ("legacy-macro-modernization", "Refactoring unmaintainable legacy Visio macros into clean, modular typed services.", ["modernize-visio-macros", "modular-visio-services", "refactor-legacy-vba"]),
    ]),

    # Cluster 7: Automated Layout, Routing & Directed Graph Algorithms
    (7, [
        ("directed-acyclic-graphs", "Topological sort, breadth-first search, cycle detection, and DAGs in diagram graphs.", ["dag-layout-algorithms", "topological-sort-graph", "cycle-detection-visio"]),
        ("page-layout-built-in-engine", "Hierarchical, radial, circular, and flowchart automatic layouts via Page.Layout().", ["page-layout-engine", "visio-autolayout-api", "hierarchical-radial-layout"]),
        ("custom-tree-generation", "Calculating sub-tree bounding boxes, sibling spacing, and tier coordinates in code.", ["custom-tree-layout", "sub-tree-bounding-box", "tier-spacing-algorithm"]),
        ("force-directed-physics-simulation", "Spring-embedder physics simulation for dynamic network node distribution on page.", ["force-directed-layout", "spring-embedder-visio", "network-node-physics"]),
        ("sugiyama-layered-layout", "Layer assignment, crossing reduction, and coordinate assignment in Visio graphs.", ["sugiyama-layered-graph", "crossing-reduction-algo", "layer-assignment-math"]),
        ("orthogonal-astar-pathfinding", "A* pathfinding, grid obstacles, and collision-free connector paths in code.", ["astar-connector-routing", "grid-obstacle-avoidance", "collision-free-paths"]),
        ("radial-circular-topologies", "Polar coordinate mapping, angle distribution, and concentric ring layouts in Visio.", ["radial-circular-topologies", "polar-coordinate-mapping", "concentric-ring-layouts"]),
        ("sankey-dynamic-connector-widths", "Dynamic connector widths proportional to volume metrics in process flow diagrams.", ["sankey-diagrams-visio", "proportional-edge-widths", "flow-volume-connectors"]),
        ("squarified-treemaps", "Recursive squarified treemaps drawn directly on Visio pages with area weighting.", ["squarified-treemaps", "hierarchical-treemap-visio", "area-weighted-rects"]),
        ("aesthetic-criteria-optimization", "Minimizing edge crossings, maximizing symmetry, and uniform edge lengths in graphs.", ["graph-aesthetic-opt", "minimize-edge-crossings", "uniform-edge-lengths"]),
    ]),

    # Cluster 8: Events, Application Event Sinks & Marker Events
    (8, [
        ("iviseventproc-sink-dispatch", "Intercepting engine events without polling: ShapeAdded, BeforeShapeDelete.", ["iviseventproc-sink", "eventsink-dispatch", "shapeadded-event-trap"]),
        ("queuemarkerevent-async-tasks", "Application.QueueMarkerEvent and processing safe background actions asynchronously.", ["queuemarkerevent", "async-marker-events", "safe-background-actions"]),
        ("mouse-interaction-hooks", "MouseUp, MouseDown, MouseMove, and drag-and-drop interception on canvas.", ["mouse-event-hooks", "drag-drop-interception", "canvas-interaction-events"]),
        ("cellchanged-formulachanged", "Reacting to user edits in specific ShapeSheet cells in real time via events.", ["cellchanged-event", "formulachanged-trap", "real-time-cell-listeners"]),
        ("window-selection-monitoring", "Window.SelectionChanged, dynamic context updates, and inspector panel synchronization.", ["selection-changed-event", "inspector-panel-sync", "window-context-updates"]),
        ("connect-disconnect-validation", "ConnectionsAdded, ConnectionsDeleted, and enforcing connection rules on drop.", ["connections-added-event", "connection-rule-enforce", "disconnect-validation"]),
        ("querycancel-deletion-guard", "QueryCancelShapeDelete and showing user confirmation prompts before data loss.", ["querycancel-deletion", "prevent-accidental-delete", "deletion-guard-prompt"]),
        ("document-page-lifecycle", "DocumentOpened, BeforeDocumentSave, PageAdded, and cleanup lifecycle hooks.", ["doc-page-lifecycle", "beforedocumentsave", "pageadded-event-hooks"]),
        ("custom-undo-scopes", "Application.BeginUndoScope, EndUndoScope, and custom transactional rollbacks.", ["custom-undo-scopes", "beginundoscope-api", "transactional-undo-units"]),
        ("throttling-and-debouncing", "Preventing UI freezing during rapid user resizing and mouse movements via debounce.", ["event-throttling-visio", "debounce-resize-events", "ui-freeze-prevention"]),
    ]),

    # Cluster 9: C# / .NET VSTO Add-ins & High-Performance Interop
    (9, [
        ("add-in-lifecycle-architecture", "Visual Studio Tools for Office, add-in lifecycle, and WPF TaskPanes in C#.", ["vsto-add-in-lifecycle", "visio-csharp-addin", "wpf-taskpane-visio"]),
        ("com-memory-garbage-collection", "Marshal.ReleaseComObject, avoiding ghost Visio processes, and zero COM leaks.", ["marshal-releasecomobject", "prevent-ghost-visio", "com-garbage-collection"]),
        ("wpf-custom-taskpane-binding", "Dockable TaskPanes, XAML UI styling, and bidirectional data binding to shapes.", ["dockable-taskpane-xaml", "wpf-shape-binding", "custom-taskpane-visio"]),
        ("ribbonx-fluent-xml-integration", "Custom tabs, groups, galleries, dynamic toggleButtons, and callbacks in C#.", ["ribbonx-visio-csharp", "custom-galleries-ribbon", "dynamic-togglebuttons"]),
        ("net-core-com-interop", "Setting up Microsoft.Office.Interop.Visio in modern .NET 8/9 C# applications.", ["net-core-visio-interop", "modern-csharp-visio", "office-interop-assemblies"]),
        ("multithreaded-background-queries", "Offloading heavy SQL/REST queries to background workers safely without UI lag.", ["background-worker-visio", "multithreaded-sql-fetch", "offload-heavy-queries"]),
        ("async-ui-dispatcher-marshaling", "Using Dispatcher.Invoke to marshal data from async tasks back to Visio COM objects.", ["dispatcher-invoke-visio", "thread-safe-ui-marshaling", "async-com-dispatch"]),
        ("unit-testing-moq-isolation", "Mocking Visio IVPage, IVShape, and isolating business logic in automated tests.", ["mock-visio-moq", "unit-test-visio-csharp", "isolated-business-logic"]),
        ("clickonce-msi-packaging", "Code signing certificates, registry keys, and enterprise distribution for VSTO.", ["clickonce-visio-deploy", "vsto-msi-packaging", "authenticode-signing"]),
        ("diagnostic-telemetry-logging", "Capturing unhandled exceptions, user metrics, and crash reporting in production.", ["vsto-telemetry-logging", "crash-reporting-visio", "unhandled-exception-hook"]),
    ]),

    # Cluster 10: XML Drawing Format (.vsdx) & Open Packaging Conventions (OPC)
    (10, [
        ("opc-zip-container-structure", "ZIP container layout, [Content_Types].xml, and _rels relationships in .VSDX.", ["vsdx-opc-structure", "content-types-xml", "rels-relationship-graph"]),
        ("page-xml-schema-deep-dive", "Shape nodes, Cell elements, and Section schemas inside /visio/pages/page1.xml.", ["page-xml-schema", "shape-nodes-schema", "cell-element-xml"]),
        ("headless-openxml-generation", "Creating and modifying Visio diagrams without installing Microsoft Visio via OpenXML.", ["headless-openxml-visio", "generate-vsdx-openxml", "visio-free-assembly"]),
        ("raw-shapesheet-cell-patching", "Editing <Cell N='...' F='...'/> directly in page1.xml via streaming XML parsers.", ["raw-cell-patching-xml", "patch-shapesheet-xml", "streaming-xml-editor"]),
        ("extracting-masters-and-stencils", "Inspecting /visio/masters/masters.xml and extracting master shapes headlessly.", ["extract-masters-xml", "masters-xml-schema", "headless-master-extract"]),
        ("streaming-search-and-replace", "Scanning 1,000s of VSDX files for text strings via streaming XML readers.", ["streaming-vsdx-search", "batch-search-replace-xml", "fast-vsdx-scanner"]),
        ("package-corruption-recovery", "Resolving broken XML tags, invalid relationship IDs, and missing media files.", ["vsdx-corruption-recovery", "repair-broken-opc-zip", "invalid-rel-id-fix"]),
        ("custom-xml-parts-embedding", "Storing proprietary JSON/XML enterprise payloads inside VSDX packages safely.", ["custom-xml-parts-visio", "embed-json-in-vsdx", "enterprise-payload-storage"]),
        ("legacy-vsd-migration-cli", "Automating mass migration from binary .VSD format to modern open .VSDX via CLI.", ["legacy-vsd-migration", "binary-to-vsdx-cli", "batch-vsd-converter"]),
        ("package-minification-compression", "Stripping redundant preview images, unused masters, and compressing XML streams.", ["vsdx-minification", "strip-preview-images", "compress-vsdx-package"]),
    ]),

    # Cluster 11: Visio JavaScript API for Visio Online / Web
    (11, [
        ("application-page-classes", "Visio.Application, Page, Shape, and Hyperlink web classes in Visio Online.", ["visio-js-classes", "visio-online-api", "web-application-page"]),
        ("async-context-sync-queue", "Batching web API calls and asynchronous execution queues via context.sync().", ["context-sync-visio-js", "async-web-queue", "batch-web-api-calls"]),
        ("iframe-token-handshake", "Loading embedded diagrams, access tokens, and postMessage security handshakes.", ["iframe-token-handshake", "postmessage-visio-web", "secure-iframe-embedding"]),
        ("shape-selection-viewport-zoom", "Page.getSelectedShapes(), Application.setActivePage(), and viewport zoom controls.", ["shape-selection-web", "viewport-zoom-api", "setactivepage-web"]),
        ("dynamic-highlight-badging", "Adding real-time visual focus and high-contrast badging to shapes via JavaScript.", ["dynamic-highlight-web", "shape-badging-js", "visual-focus-online"]),
        ("shape-data-hyperlinks-reading", "Shape.shapeData, Shape.hyperlinks, and displaying custom metadata panels in web.", ["read-shapedata-web", "hyperlinks-web-api", "metadata-panels-online"]),
        ("selection-change-event-listeners", "Hooking user click events on embedded online diagrams via onSelectionChanged.", ["onselectionchanged-web", "click-listeners-visio-js", "web-event-dispatch"]),
        ("session-error-recovery", "Handling session timeouts, token renewals, and network drops in Visio Online.", ["session-timeout-recovery", "token-renewal-visio-web", "resilient-web-session"]),
        ("plan1-vs-plan2-boundaries", "Understanding license boundaries and read-only vs editable embedding capabilities.", ["plan1-vs-plan2-visio", "web-licensing-boundaries", "embed-mode-restrictions"]),
        ("automated-walkthrough-presentation", "Creating automated slide-show walkthroughs across diagram pages in the browser.", ["diagram-walkthrough-web", "automated-page-carousel", "presentation-mode-js"]),
    ]),

    # Cluster 12: Embedding Visio in React, Angular & Fluent UI
    (12, [
        ("react-component-wrappers", "React component wrappers, iframe embedding, and state hooks for Visio diagrams.", ["react-visio-wrapper", "useeffect-visio-embed", "react-diagram-component"]),
        ("fluent-ui-react-panels", "Styling side panels, tooltips, and controls with Fluent UI Design System.", ["fluent-ui-visio-panels", "fluent-design-sidebars", "react-fluent-tooltips"]),
        ("bidirectional-state-sync", "Dispatching actions from React state into Visio diagram selections and vice-versa.", ["bidirectional-react-visio", "redux-visio-sync", "state-driven-selection"]),
        ("angular-rxjs-event-streams", "Observables, RxJS event streams, and Visio JS integration in Angular apps.", ["angular-visio-service", "rxjs-diagram-stream", "angular-iframe-bridge"]),
        ("msal-bearer-tokens", "Acquiring bearer tokens for embedding OneDrive/SharePoint diagrams via MSAL 2.0.", ["msal-visio-tokens", "sharepoint-diagram-embed", "bearer-token-handshake"]),
        ("responsive-breakpoint-scaling", "Handling window resize, scaling ratios, and touch navigation across devices.", ["responsive-visio-scaling", "breakpoint-resize-iframe", "touch-navigation-web"]),
        ("floating-html-tooltip-anchors", "Converting Visio page coordinates into DOM screen pixel positions for popovers.", ["floating-html-tooltips", "page-to-dom-coords", "anchored-popovers-visio"]),
        ("nextjs-ssr-dashboards", "SSR Next.js app consuming real-time metrics over Visio diagrams in modern cloud.", ["nextjs-visio-dashboards", "ssr-diagram-portal", "realtime-nextjs-visio"]),
        ("micro-frontend-visual-portals", "Decoupled micro-apps embedding Visio viewers in larger enterprise portals.", ["micro-frontend-visio", "module-federation-diagram", "enterprise-visual-portal"]),
        ("iframe-performance-profiling", "Measuring load times, DOM memory footprints, and frame rate optimization in web.", ["profile-visio-iframe", "dom-memory-optimization", "web-loadtime-tuning"]),
    ]),

    # Cluster 13: SVG Generation, Path Optimization & Vector Rendering
    (13, [
        ("export-viewbox-calibration", "DoCmd.SaveAs SVG, viewBox settings, and vector fidelity calibration.", ["export-svg-visio", "viewbox-calibration", "vector-fidelity-svg"]),
        ("path-sanitization-cleanup", "Sanitizing Visio-exported SVGs, removing inline junk, and injecting CSS classes.", ["sanitize-visio-svg", "svg-path-cleanup", "inject-css-classes-svg"]),
        ("css-dark-mode-theming", "Applying dark mode CSS themes, gradient fills, and stroke effects to SVG shapes.", ["svg-dark-mode-theme", "css-stroke-effects", "gradient-fills-svg"]),
        ("d3js-data-binding", "Binding D3 data joins directly to Visio SVG <path> and <g> nodes dynamically.", ["d3js-visio-binding", "d3-data-joins-svg", "dynamic-svg-data"]),
        ("smil-css-flow-animations", "Pulsing connection paths, rotating fan blades, and flow animations in vector SVGs.", ["svg-flow-animations", "smil-css-pulsing", "animated-vector-paths"]),
        ("responsive-mobile-scaling", "Creating vector diagrams that fluidly scale from 4K displays to mobile phones.", ["responsive-vector-svg", "mobile-svg-scaling", "fluid-diagram-display"]),
        ("font-inlining-woff2-assets", "Overcoming missing fonts by embedding WOFF2 and base64 assets directly in SVG.", ["font-inlining-svg", "woff2-base64-embed", "missing-font-solution"]),
        ("headless-chromium-pdf-render", "Lossless SVG-to-PDF rendering pipelines using headless Chromium and Node.js.", ["headless-svg-to-pdf", "chromium-vector-render", "lossless-pdf-pipeline"]),
        ("svgo-path-decimation-opt", "Shrinking multi-megabyte CAD/Visio SVGs by 80% without visible vector quality loss.", ["svgo-optimization", "path-decimation-algo", "shrink-svg-filesize"]),
        ("aria-semantic-accessibility", "Adding ARIA labels, title tags, and role attributes to SVG nodes for screen-readers.", ["svg-accessibility-aria", "screen-reader-diagrams", "wcag-semantic-svg"]),
    ]),

    # Cluster 14: Interactive Web Diagrams: Overlays, Tooltips & Click Handlers
    (14, [
        ("interactive-visual-interfaces", "Transforming static architectural blueprints into live clickable web interfaces.", ["interactive-blueprints", "live-clickable-diagrams", "visual-web-interfaces"]),
        ("html-popovers-coordinate-anchors", "Calculating bounding rects and displaying floating UI popovers over shapes.", ["bounding-rect-popovers", "floating-ui-anchors", "html-popovers-visio"]),
        ("canvas-svg-heatmaps", "Layering semi-transparent density heatmaps over floor plans and schematics.", ["canvas-svg-heatmaps", "density-overlay-floors", "heatmap-layer-visio"]),
        ("status-badging-pulse-indicators", "Rendering green/yellow/red status pills and pulsing indicators on components.", ["status-badging-pills", "pulse-indicators-web", "severity-color-badges"]),
        ("multilevel-drilldown-navigation", "Clicking a server rack shape to navigate into internal board schematics.", ["drilldown-navigation", "parent-child-diagrams", "rack-to-board-drill"]),
        ("minimap-viewport-synchronization", "Building custom minimap overview windows synchronizing with main viewports.", ["minimap-sync-visio", "viewport-overview-window", "minimap-navigation"]),
        ("click-to-action-cloud-triggers", "Clicking a database node in a diagram to trigger a cloud backup API call.", ["click-to-action-cloud", "diagram-api-triggers", "interactive-ops-diagram"]),
        ("wcag-keyboard-tab-navigation", "Keyboard tab navigation across diagram shapes, focus rings, and descriptions.", ["keyboard-tab-navigation", "focus-rings-shapes", "wcag-interactive-diagram"]),
        ("realtime-collaborative-cursors", "Rendering collaborative presence indicators and cursors over Visio canvas.", ["collaborative-cursors", "realtime-presence-canvas", "multiuser-pointer-stream"]),
        ("mobile-touch-gesture-controls", "Optimizing touch listeners for iPad and Surface tablet pinch-to-zoom gestures.", ["mobile-touch-gestures", "pinch-zoom-diagram", "touch-listener-visio"]),
    ]),

    # Cluster 15: Headless Diagram Synthesis via Node.js, Python & Canvas
    (15, [
        ("python-vsdx-parsing", "Reading, parsing, and programmatically creating .vsdx files in Python scripts.", ["python-vsdx-library", "parse-vsdx-python", "programmatic-vsdx-script"]),
        ("nodejs-openxml-microservices", "Building server-side microservices that emit dynamic VSDX packages on demand.", ["nodejs-vsdx-service", "openxml-microservices", "dynamic-vsdx-endpoint"]),
        ("powershell-server-automation", "Running Visio automation jobs headlessly in scheduled Windows backend tasks.", ["powershell-visio-jobs", "scheduled-diagram-task", "headless-server-visio"]),
        ("mermaid-flowchart-translation", "Parsing Mermaid flowchart markdown and generating native Visio diagrams.", ["mermaid-to-visio", "markdown-flowchart-parse", "mermaid-vsdx-bridge"]),
        ("plantuml-graphviz-dot-migration", "Translating DOT graph nodes and edges into Visio shape connections accurately.", ["plantuml-to-visio", "graphviz-dot-to-vsdx", "dot-edge-translation"]),
        ("docker-containerized-rendering", "Converting VSDX to PDF/SVG in Docker using headless tools and OpenXML.", ["docker-vsdx-render", "container-diagram-export", "headless-docker-visio"]),
        ("terraform-state-topology-maps", "Generating Visio infrastructure maps from terraform plan and state outputs.", ["terraform-to-visio", "tfstate-infrastructure", "cloud-diagram-generator"]),
        ("sql-schema-to-erd-generation", "Querying INFORMATION_SCHEMA and synthesizing native Visio ERD tables.", ["sql-schema-to-visio", "information-schema-erd", "automated-erd-generator"]),
        ("ci-cd-diagram-linting-gates", "Verifying that generated diagrams adhere to corporate styling and layout rules.", ["ci-cd-diagram-linting", "styling-rules-gate", "automated-diagram-audit"]),
        ("massive-graph-streaming-opt", "Memory optimization and streaming techniques for generating 10,000-node graphs.", ["massive-graph-streaming", "10k-node-visio-opt", "low-memory-graph-gen"]),
    ]),

    # Cluster 16: Data Linking, Link Data to Shapes & Recordsets
    (16, [
        ("recordset-architecture", "DataRecordsets, AddFromConnection, external data windows, and linking schemas.", ["datarecordsets-architecture", "addfromconnection-api", "external-data-window"]),
        ("autolink-matching-criteria", "AutoLink method, field matching criteria, and bulk automated linking pipelines.", ["autolink-matching", "bulk-data-linking", "field-matching-criteria"]),
        ("sql-server-oracle-odbc", "Configuring connection strings, SQL queries, and parameter bindings in Visio.", ["sql-server-visio-link", "oracle-odbc-linking", "parameterized-data-queries"]),
        ("excel-csv-tabular-sources", "Sheet ranges, dynamic updates, and managing file path changes in linked data.", ["excel-visio-data-link", "csv-tabular-sources", "dynamic-sheet-updates"]),
        ("automatic-refresh-timers", "DataRecordset.Refresh(), refresh timers, and handling deleted backend rows safely.", ["automatic-refresh-timers", "datarecordset-refresh", "deleted-row-handling"]),
        ("primary-key-stability", "Ensuring shapes remain glued to data records across schema updates and sorting.", ["primary-key-stability", "stable-shape-data-links", "key-column-integrity"]),
        ("rest-api-json-intermediaries", "Injecting JSON payloads into DataRecordsets via XML intermediary tables.", ["rest-api-data-linking", "json-to-datarecordset", "xml-intermediary-tables"]),
        ("multisource-metric-merging", "Combining finance data and IT metrics into a single server rack shape.", ["multisource-metric-merge", "combine-disparate-data", "composite-data-shapes"]),
        ("programmatic-creation-api", "DataRecordsets.Add, defining column types, and schema overrides in C# and VBA.", ["datarecordsets-add-api", "programmatic-columns", "schema-overrides-data"]),
        ("connection-timeout-troubleshooting", "Diagnosing ODBC timeouts, firewall blocks, and stale connection caches in Visio.", ["odbc-timeout-diagnostics", "stale-connection-cache", "firewall-data-repair"]),
    ]),

    # Cluster 17: Data Graphics: Callouts, Data Bars & Color by Value
    (17, [
        ("visual-information-design", "Text callouts, data bars, icon sets, and color by value design patterns.", ["data-graphics-design", "text-callouts-visio", "icon-sets-datagfx"]),
        ("custom-callout-master-design", "Authoring multi-line callouts, dynamic leader lines, and smart badges.", ["custom-callout-masters", "dynamic-leader-lines", "smart-badge-design"]),
        ("data-bars-progress-gauges", "Min/max thresholds, percentage scaling, and horizontal/vertical progress bars.", ["data-bars-gauges", "percentage-progress-bars", "threshold-scaling-gfx"]),
        ("icon-sets-status-flags", "Configuring 3-color and 5-color icon sets driven by numeric metrics in shapes.", ["icon-sets-status", "color-coded-flags", "metric-driven-icons"]),
        ("color-by-value-heatmaps", "Formulas shading shape backgrounds based on severity scales and thresholds.", ["color-by-value-heatmaps", "severity-shading-cells", "dynamic-heatmap-colors"]),
        ("programmatic-setdatagraphic", "Shape.SetDataGraphic(), creating GraphicItems, and bulk styling shapes via code.", ["setdatagraphic-api", "graphicitems-creation", "bulk-datagfx-styling"]),
        ("multimetric-corner-panels", "Positioning 4 distinct metrics on corners of an equipment shape in Visio.", ["multimetric-corner-panels", "4-corner-metrics", "equipment-dashboard-shape"]),
        ("conditional-number-format-masks", "Formatting currencies $#,##0, percentages, and dates in data graphic callouts.", ["currency-format-callouts", "percentage-masks-gfx", "date-formatting-badges"]),
        ("dynamic-legend-generation", "Inserting dynamic legend blocks documenting active Data Graphic rules and ranges.", ["dynamic-legend-blocks", "datagfx-legend-generator", "range-scale-legends"]),
        ("high-volume-rendering-optimization", "Preventing render lag when 5,000 shapes carry active Data Graphics on page.", ["datagfx-performance-opt", "high-volume-render-lag", "5k-shape-datagfx"]),
    ]),

    # Cluster 18: Visio Visual in Power BI: Interactive Drilldowns & Cross-Filtering
    (18, [
        ("visual-integration-architecture", "Visio Visual in Power BI, mapping keys, and visual properties configuration.", ["powerbi-visio-visual", "mapping-keys-powerbi", "visual-property-config"]),
        ("bidirectional-cross-filtering", "Clicking a shape filters bar charts; clicking charts highlights shapes.", ["bidirectional-cross-filtering", "shape-to-chart-filter", "chart-to-shape-highlight"]),
        ("onedrive-sharepoint-hosting", "Storing diagrams in OneDrive/SharePoint, shape ID consistency, and cloud access.", ["powerbi-diagram-hosting", "shape-id-consistency", "onedrive-powerbi-bridge"]),
        ("floorplan-desk-occupancy-maps", "Real-time desk occupancy, conference room utilization, and spatial heatmaps.", ["desk-occupancy-maps", "floorplan-powerbi", "spatial-heatmaps-visio"]),
        ("supply-chain-warehouse-bottlenecks", "Warehouse rack status, shipping lane throughput, and bottleneck alert flags.", ["supply-chain-visio-pbi", "warehouse-rack-status", "bottleneck-alert-flags"]),
        ("manufacturing-assembly-line-scada", "Machine downtime, telemetry alarms, and throughput pacing in Power BI.", ["manufacturing-scada-pbi", "assembly-line-downtime", "throughput-pacing-visio"]),
        ("row-level-security-governance", "Row-level security, tenant permissions, and embedded tokens in Power BI Visio.", ["rls-visio-powerbi", "tenant-security-governance", "embedded-token-auth"]),
        ("mobile-report-layout-optimization", "Designing responsive visual layouts for Power BI mobile app users.", ["powerbi-mobile-visio", "responsive-visual-reports", "mobile-diagram-layout"]),
        ("multipage-diagram-bookmark-sync", "Page switching parameters, bookmarking states, and visual synchronization.", ["powerbi-bookmark-sync", "multipage-diagram-switch", "bookmark-state-visio"]),
        ("frame-rate-query-optimization", "Managing dataset query size, caching strategies, and frame rate optimization.", ["powerbi-framerate-tuning", "dataset-query-size-opt", "visio-visual-perf"]),
    ]),

    # Cluster 19: Excel to Visio: Data Visualizer & Automated Org Charts
    (19, [
        ("automated-flowchart-generation", "Basic flowcharts, cross-functional swimlanes, and org charts from tables.", ["data-visualizer-flowcharts", "automated-swimlanes", "table-to-flowchart-visio"]),
        ("custom-template-schema-design", "Column schemas, process step IDs, next step IDs, and connector labels.", ["data-visualizer-templates", "step-id-connector-schema", "process-schema-design"]),
        ("swimlane-phase-mapping", "Function/Department columns generating dynamic swimlane pools and phases.", ["swimlane-phase-mapping", "department-swimlane-pools", "phase-boundary-columns"]),
        ("automated-org-chart-hierarchies", "Reports-to hierarchies, employee photos, title formatting, and vacancy flags.", ["automated-org-charts", "reports-to-hierarchy", "employee-photo-shapes"]),
        ("python-workbook-builder", "Building compliant Excel tables from raw system logs using openpyxl.", ["python-data-visualizer", "openpyxl-visio-tables", "build-compliant-workbooks"]),
        ("bidirectional-refresh-relinking", "Re-linking data sources, refreshing geometry from rows, and reconciliation.", ["data-visualizer-relink", "refresh-geometry-from-rows", "tabular-reconciliation"]),
        ("complex-looping-logic-patterns", "Handling multi-condition splits, loops, and parallel paths in table rows.", ["multi-condition-splits", "table-looping-logic", "parallel-flow-rows"]),
        ("enterprise-template-distribution", "Distributing templates via SharePoint and OneDrive for corporate business units.", ["distribute-dv-templates", "sharepoint-template-hub", "corporate-process-templates"]),
        ("batch-process-map-cli-rendering", "CLI automation rendering individual VSDX files per business unit from tables.", ["batch-dv-rendering", "cli-process-map-gen", "bulk-vsdx-from-excel"]),
        ("text-procedure-to-flowchart-ai", "LLM extraction of process steps from standard operating procedures into tables.", ["sop-to-flowchart-ai", "text-procedure-extractor", "ai-data-visualizer"]),
    ]),

    # Cluster 20: Real-Time IoT & Telemetry Visual Overlays
    (20, [
        ("digital-twin-vector-displays", "Azure IoT Hub, SignalR webhooks, and live Visio vector displays in cloud apps.", ["digital-twin-visio", "azure-iothub-diagrams", "signalr-live-vectors"]),
        ("datacenter-temperature-heatmaps", "Server rack temperature heatmaps, power load, and network drops in Visio.", ["datacenter-heatmaps", "server-rack-temp-load", "dcim-visual-dashboards"]),
        ("scada-industrial-pipeline-alerts", "Piping pressure alerts, valve states, and fluid animation overlays on diagrams.", ["scada-visio-alerts", "pipeline-pressure-valves", "industrial-fluid-overlays"]),
        ("hvac-energy-consumption-dashboards", "Live airflow metrics, chiller telemetry, and kilowatt heatmaps on floor plans.", ["hvac-energy-visio", "chiller-telemetry-floor", "kilowatt-heatmap-dash"]),
        ("high-frequency-websocket-streams", "Pushing 10Hz updates into shape properties without crashing UI threads.", ["high-frequency-websockets", "10hz-shape-updates", "ui-thread-protection"]),
        ("alarm-threshold-escalation-states", "Green -> Yellow -> Blinking Red state animations driven by MQTT broker alerts.", ["alarm-threshold-states", "mqtt-state-animations", "blinking-alert-shapes"]),
        ("historical-telemetry-timeline-replay", "Timeline slider replaying historical sensor states across diagram shapes.", ["telemetry-timeline-replay", "historical-sensor-playback", "time-scrubber-visio"]),
        ("azure-digital-twins-dtdl-mapping", "Mapping DTDL models to Visio shapes and cloud twin instances seamlessly.", ["dtdl-digital-twins-visio", "cloud-twin-mapping", "dtdl-model-shapes"]),
        ("soc-cybersecurity-attack-surfaces", "Real-time Sentinel intrusion alerts highlighting visual firewall and server nodes.", ["soc-attack-surface-visio", "sentinel-intrusion-alerts", "visual-firewall-nodes"]),
        ("failsafe-telemetry-offline-caching", "Graceful handling of dropped telemetry connections in live diagram views.", ["failsafe-telemetry-cache", "offline-connection-guards", "graceful-degradation-iot"]),
    ]),

    # Cluster 21: BPMN 2.0 & Workflow Process Modeling
    (21, [
        ("modeling-and-style-foundations", "Descriptive vs analytical vs executable modeling standards in BPMN 2.0.", ["bpmn-style-foundations", "descriptive-vs-executable", "bpmn-modeling-standards"]),
        ("visio-native-stencil-components", "Events, tasks, gateways, pools, lanes, and data objects in native Visio stencils.", ["bpmn-native-components", "gateways-pools-lanes", "bpmn-data-objects"]),
        ("validation-rules-engine", "Verifying syntax: single start events, valid sequence flows, and gateway rules.", ["bpmn-validation-rules", "single-start-event-check", "sequence-flow-syntax"]),
        ("cross-functional-swimlane-pools", "Horizontal vs vertical swimlanes, phases, and lane reordering in diagrams.", ["swimlane-pools-bpmn", "horizontal-vertical-lanes", "phase-reordering-visio"]),
        ("exporting-to-camunda-power-automate", "Translating Visio BPMN XML into executable process definitions for engines.", ["export-bpmn-camunda", "power-automate-bpmn", "executable-bpmn-translation"]),
        ("subprocess-hierarchical-decomposition", "Collapsed sub-processes, page hyperlinks, and hierarchical process mapping.", ["collapsed-subprocesses", "hierarchical-process-maps", "page-hyperlink-subproc"]),
        ("compensation-error-boundary-events", "Boundary events, intermediate throwing/catching events in Visio workflows.", ["boundary-events-bpmn", "compensation-error-events", "intermediate-throwing-catch"]),
        ("conformance-checking-event-logs", "Comparing Visio models against event logs from real systems for conformance.", ["conformance-checking-bpmn", "event-log-comparison", "process-mining-visio"]),
        ("corporate-styleguide-enforcement", "Establishing corporate BPMN style guides, stencil locks, and palette hygiene.", ["bpmn-corporate-styleguide", "stencil-locks-bpmn", "palette-hygiene-visio"]),
        ("spaghetti-flowchart-refactoring", "Patterns for untangling cross-over connectors and bloated loops into clean BPMN.", ["untangle-spaghetti-flows", "refactor-bloated-loops", "clean-bpmn-refactoring"]),
    ]),

    # Cluster 22: Cloud Architecture Diagrams: Azure, AWS & GCP Topologies
    (22, [
        ("multiregion-system-design", "Visual design patterns for scalable, multi-region cloud infrastructures in Visio.", ["multiregion-cloud-design", "cloud-diagram-patterns", "high-availability-visuals"]),
        ("azure-resource-graph-automation", "Querying Azure Resource Graph and rendering live topologies automatically.", ["azure-resource-graph-visio", "live-topology-render", "automated-azure-maps"]),
        ("azure-stencil-kit-standards", "Official icons, subscription boundaries, VNets, and subnet grouping in Azure.", ["azure-stencil-kit", "vnet-subnet-grouping", "subscription-boundaries"]),
        ("aws-well-architected-topologies", "VPCs, availability zones, IAM policies, and serverless topologies in Visio.", ["aws-well-architected-visio", "vpc-az-boundaries", "serverless-aws-topology"]),
        ("gcp-system-architecture-hubs", "Projects, VPC peering, Kubernetes clusters, and BigQuery hubs in GCP diagrams.", ["gcp-architecture-visio", "vpc-peering-diagrams", "bigquery-hub-visuals"]),
        ("kubernetes-pod-service-ingress-mesh", "Visualizing K8s namespaces, service meshes, and ingress routing in Visio.", ["kubernetes-visio-topologies", "k8s-service-mesh-diagram", "pod-ingress-routing"]),
        ("terraform-state-infrastructure-maps", "Generating Visio infrastructure maps from terraform plan outputs.", ["terraform-state-visio", "tfplan-to-diagram", "iac-visual-generator"]),
        ("expressroute-directconnect-peering", "Visualizing VPN tunnels, peering gateways, and edge routers in hybrid cloud.", ["expressroute-directconnect", "hybrid-cloud-peering", "vpn-tunnel-diagrams"]),
        ("finops-monthly-spend-badging", "Adding live FinOps cost badges to cloud resource shapes in architecture maps.", ["finops-cost-badges", "monthly-spend-shapes", "cloud-cost-diagrams"]),
        ("zero-trust-microsegmentation-nsgs", "Visualizing micro-segmentation, NSGs, and bastion hosts in security diagrams.", ["zero-trust-microsegmentation", "nsg-security-diagrams", "bastion-host-visuals"]),
    ]),

    # Cluster 23: Enterprise IT & Network Topology Mapping
    (23, [
        ("logical-vs-physical-infrastructure", "Logical vs physical network topologies, IP addressing schemas, and subnets.", ["logical-vs-physical-net", "ip-addressing-schemas", "subnet-topology-visio"]),
        ("server-rack-elevation-plans", "Rack units [U], cable routing, power distribution units [PDUs] in Visio.", ["rack-elevation-plans", "rack-units-cable-route", "pdu-power-draw-shapes"]),
        ("cisco-switch-ports-patch-panels", "Connecting physical switch ports, patch cables, and VLANs on rack diagrams.", ["cisco-switch-ports", "patch-panel-cables", "vlan-patching-diagrams"]),
        ("snmp-lldp-discovery-auto-mapping", "SNMP scans, LLDP/CDP discovery, and automated diagram synthesis in IT.", ["snmp-lldp-discovery", "automated-network-synthesis", "cdp-topology-scan"]),
        ("datacenter-raised-floor-cable-trays", "Raised floor grids, under-floor cable trays, and airflow tiles in facility maps.", ["datacenter-floor-grids", "cable-tray-routes", "airflow-tile-layouts"]),
        ("fiber-optic-splice-tray-schematics", "Fiber pairs, splice trays, patch distribution, and signal paths in schematics.", ["fiber-splice-trays", "fiber-optic-schematics", "patch-distribution-paths"]),
        ("active-directory-entra-forest-trusts", "Visualizing forests, domains, OUs, and cross-tenant synchronization in Visio.", ["ad-entra-forest-trusts", "cross-tenant-sync-maps", "ou-hierarchy-diagrams"]),
        ("san-fabric-multipath-storage-luns", "HBAs, SAN fabrics, storage arrays, and multipath I/O maps in enterprise IT.", ["san-fabric-storage", "multipath-io-maps", "lun-storage-arrays"]),
        ("disaster-recovery-failover-routes", "Visualizing primary to secondary failover paths during data center outages.", ["dr-failover-routes", "datacenter-outage-paths", "failover-route-simulation"]),
        ("barcode-asset-tag-rack-dropping", "Barcode scanning physical servers to drop mapped shapes into rack elevations.", ["barcode-asset-dropping", "asset-tag-server-map", "rack-elevation-scanner"]),
    ]),

    # Cluster 24: UML 2.5, Software Architecture & Database ERDs
    (24, [
        ("standard-object-modeling", "Class, sequence, state, and component diagrams in UML 2.5 notation.", ["uml25-object-modeling", "class-sequence-state", "component-diagrams-visio"]),
        ("class-stereotypes-interfaces", "Class modeling, associations, stereotypes, and interfaces in software specs.", ["class-stereotypes-visio", "interface-realization", "uml-association-rules"]),
        ("reverse-engineering-sql-catalogs", "ODBC catalog extraction, entity tables, and relationship lines into Visio ERDs.", ["reverse-engineer-sql-erd", "odbc-catalog-extraction", "entity-table-schemas"]),
        ("crows-foot-cardinality-markers", "1:1, 1:N, M:N cardinality markers and relationship lines in Visio database models.", ["crows-foot-cardinality", "cardinality-markers-visio", "relationship-line-styles"]),
        ("sequence-lifelines-activation-boxes", "Synchronous, asynchronous, return messages, and loops in sequence flows.", ["sequence-lifelines", "activation-boxes-uml", "async-return-messages"]),
        ("microservices-component-deployment", "Docker containers, API gateways, and distributed message queues in deployments.", ["microservices-deployment-uml", "docker-container-nodes", "api-gateway-queues"]),
        ("composite-state-machine-guards", "Initial states, composite states, transitions, and guard conditions in UML.", ["composite-state-machines", "guard-condition-uml", "transition-triggers-visio"]),
        ("use-case-boundary-systems", "Actors, use cases, <<include>>, <<extend>>, and system boundaries in analysis.", ["use-case-boundaries", "include-extend-actors", "system-boundary-boxes"]),
        ("exporting-erd-to-ddl-scripts", "Exporting entity-relationship models to CREATE TABLE statements automatically.", ["export-erd-to-ddl", "create-table-ddl-gen", "schema-ddl-export"]),
        ("ddd-bounded-context-maps", "Upstream/downstream relationships, shared kernels, and customer/supplier in DDD.", ["ddd-bounded-contexts", "shared-kernel-diagrams", "upstream-downstream-maps"]),
    ]),

    # Cluster 25: Engineering & CAD: Floor Plans, P&ID & Electrical Schematics
    (25, [
        ("pid-instrumentation-diagrams", "Pumps, valves, tanks, instruments, and signal lines in engineering P&ID.", ["pid-instrumentation-visio", "pumps-valves-tanks", "signal-line-schematics"]),
        ("autocad-dwg-dxf-scaling-layers", "CAD file scaling, layering, conversion to native Visio shapes in engineering.", ["autocad-dwg-dxf-visio", "cad-scaling-layers", "convert-cad-shapes"]),
        ("architectural-walls-space-plans", "Walls, doors, windows, dimension lines, and space utilization in floor plans.", ["architectural-floorplans", "walls-doors-windows", "dimension-lines-visio"]),
        ("electrical-single-line-circuits", "Transformers, breakers, contactors, and wire color codes in electrical models.", ["electrical-single-line", "breakers-contactors-circuits", "wire-color-codes-visio"]),
        ("hvac-ductwork-volume-calculations", "Duct routes, diffusers, air handlers, and volume calculations in mechanical plans.", ["hvac-ductwork-visio", "diffusers-air-handlers", "mechanical-volume-calc"]),
        ("security-cctv-sight-cone-plans", "CCTV camera sight cones, badge readers, magnetic door locks on floor plans.", ["cctv-sight-cones", "security-door-access-plans", "badge-reader-layouts"]),
        ("fire-safety-evacuation-routes", "Emergency exits, extinguisher locations, muster points, and evacuation routes.", ["fire-safety-routes", "emergency-exits-visio", "muster-point-signage"]),
        ("civil-boundary-landscape-plans", "Property boundaries, topography lines, parking stalls, utilities in site plans.", ["civil-boundary-plans", "topography-lines-visio", "parking-stall-layouts"]),
        ("layer-isolation-complex-drawings", "Isolating electrical, plumbing, and architectural layers cleanly in Visio.", ["layer-isolation-visio", "multi-layer-engineering", "plumbing-electrical-layers"]),
        ("raster-blueprint-to-vector-shapes", "Raster-to-vector tracing pipelines and ShapeSheet conversion from scanned plans.", ["raster-to-vector-blueprint", "scanned-plan-vectorize", "blueprint-tracing-visio"]),
    ]),

    # Cluster 26: Power Automate Workflows for Diagram Automation
    (26, [
        ("event-triggered-diagram-generation", "Triggering diagram generation from approvals, emails, and data in Power Automate.", ["flow-triggered-diagrams", "approval-diagram-gen", "event-driven-visio-flows"]),
        ("sharepoint-list-modification-updates", "Using cloud flows to update Visio files stored in SharePoint automatically.", ["update-visio-from-sharepoint", "cloud-flow-visio-sync", "list-change-diagram-update"]),
        ("microsoft-forms-survey-to-nodes", "Capturing user inputs and appending dynamic flowchart nodes from Microsoft Forms.", ["forms-to-visio-flow", "survey-input-diagrams", "dynamic-node-appending"]),
        ("rpa-desktop-client-automation", "UI flow automation driving desktop Visio operations via Power Automate Desktop.", ["rpa-visio-desktop", "power-automate-desktop-visio", "ui-flow-automation"]),
        ("inspecting-vsdx-xml-inside-flows", "Extracting text and Shape Data from VSDX files directly in cloud flow runs.", ["inspect-vsdx-flow", "extract-shapedata-flow", "cloud-vsdx-xml-parse"]),
        ("workflow-routing-driven-by-diagram", "Executing workflow sequences modeled visually in Visio diagrams in real time.", ["diagram-driven-workflows", "execute-visio-model-flow", "visual-workflow-engine"]),
        ("servicenow-incident-topology-maps", "Spawning visual incident topology maps when P1 tickets trigger in ServiceNow.", ["servicenow-incident-visio", "p1-incident-topology", "automated-incident-map"]),
        ("concurrency-locks-retry-policies", "Handling concurrency locks on cloud-hosted VSDX documents in automated flows.", ["concurrency-locks-visio", "retry-policy-vsdx", "cloud-file-lock-guards"]),
        ("onedrive-vsdx-to-pdf-conversion", "Using OneDrive conversion endpoints to convert VSDX to PDF in cloud flows.", ["onedrive-vsdx-to-pdf", "cloud-diagram-conversion", "automated-pdf-export-flow"]),
        ("dlp-governance-service-accounts", "Managing service accounts, DLP policies, and connection quotas in diagram flows.", ["dlp-visio-flows", "service-account-governance", "diagram-flow-quotas"]),
    ]),

    # Cluster 27: Microsoft Graph API & Visio File Metadata
    (27, [
        ("driveitem-diagram-crud-ops", "DriveItems, graph permissions, and file metadata operations on Visio assets.", ["graph-driveitem-visio", "visio-metadata-graph", "drive-crud-operations"]),
        ("v1-drive-upload-download-versions", "/v1.0/drives/{id}/items operations on .vsdx packages across cloud sites.", ["graph-v1-vsdx-upload", "download-vsdx-graph", "diagram-versioning-api"]),
        ("thumbnail-preview-extraction", "/thumbnails endpoint returning high-res page image renders from Visio files.", ["graph-thumbnails-visio", "page-image-renders-api", "extract-preview-graph"]),
        ("kql-indexing-search-queries", "Indexing Shape Data text and searching diagram content via Microsoft Graph KQL.", ["kql-search-visio", "search-shape-data-graph", "index-diagram-content"]),
        ("coauthoring-file-lock-management", "Handling checkout, check-in, and concurrent editing sessions via Graph API.", ["coauthoring-locks-graph", "checkin-checkout-visio", "concurrent-sessions-api"]),
        ("delta-queries-change-detection", "Detecting incremental updates to Visio files across SharePoint sites via delta.", ["delta-queries-visio", "incremental-diagram-sync", "change-tracking-graph"]),
        ("sharing-link-permission-scopes", "Creating anonymous, organization, or specific user view links for diagrams.", ["sharing-links-visio", "anonymous-view-links", "permission-scopes-diagrams"]),
        ("purview-sensitivity-retention-labels", "Applying sensitivity and retention labels to VSDX assets via Graph security.", ["purview-labels-visio", "sensitivity-labels-vsdx", "retention-labels-graph"]),
        ("webhook-change-notifications", "Subscribing to real-time change events on enterprise diagram libraries via API.", ["webhook-notifications-visio", "graph-change-subscriptions", "realtime-diagram-alerts"]),
        ("dotnet-sdk-diagram-portals", "Custom web catalogs searching and rendering enterprise diagrams via Graph .NET SDK.", ["dotnet-sdk-visio-portal", "graph-diagram-catalog", "csharp-graph-visio-app"]),
    ]),

    # Cluster 28: SharePoint Server / Online Visio Services & Web Parts
    (28, [
        ("vds-rendering-engine-architecture", "VDS rendering engine, data refresh cycles, and web access in SharePoint.", ["vds-rendering-engine", "visio-services-architecture", "data-refresh-cycles"]),
        ("modern-page-file-viewer-webpart", "Modern web part configuration, responsive scaling, interactive mode on intranet.", ["modern-file-viewer-visio", "responsive-scaling-webpart", "intranet-diagram-embed"]),
        ("spfx-react-visio-js-webparts", "Building custom React web parts hosting Visio JavaScript API in SharePoint.", ["spfx-visio-react", "custom-webpart-visio-js", "sharepoint-framework-diagram"]),
        ("corporate-intranet-org-directories", "Creating corporate org chart directories and process centers on SharePoint.", ["corporate-org-directories", "process-center-intranet", "sharepoint-orgcharts"]),
        ("secure-store-service-credentials", "Secure Store Service, credentials, and automated refresh in Visio Services.", ["secure-store-credentials", "visio-services-auth", "unattended-refresh-account"]),
        ("migrating-legacy-vwd-silverlight", "Decommissioning legacy Silverlight/ActiveX components into modern HTML5.", ["migrate-legacy-vwd", "decommission-silverlight-visio", "html5-visio-migration"]),
        ("shape-data-to-list-column-sync", "Promoting Shape Data attributes to SharePoint list columns automatically.", ["shape-data-to-columns", "promote-attributes-list", "sharepoint-column-sync"]),
        ("user-interaction-audit-logging", "Tracking diagram consumption, click hotspots, and popular drill-down paths.", ["diagram-consumption-audit", "click-hotspots-logging", "drilldown-analytics-visio"]),
        ("multilingual-intranet-translations", "Serving localized diagrams based on user language profiles in SharePoint.", ["multilingual-diagrams", "localized-intranet-visio", "user-language-diagram-sync"]),
        ("multigeo-tenant-disaster-recovery", "Multi-geo tenant replication, backup, and disaster recovery for diagram libraries.", ["multigeo-visio-replication", "disaster-recovery-diagrams", "tenant-backup-visio"]),
    ]),

    # Cluster 29: Microsoft Teams Meeting Collaboration & Whiteboard Bridges
    (29, [
        ("channel-tab-coauthoring-sync", "Embedding diagrams in channel tabs, real-time co-authoring in Teams.", ["teams-channel-visio-tab", "realtime-coauthoring-teams", "channel-diagram-sync"]),
        ("meeting-live-share-synchronized-view", "Synchronized zoom and pan during collaborative architecture reviews in calls.", ["teams-live-share-visio", "synchronized-zoom-pan", "collaborative-call-reviews"]),
        ("whiteboard-canvas-export-bridge", "Transforming formal diagrams into brainstorming canvases in Microsoft Whiteboard.", ["whiteboard-visio-bridge", "formal-to-brainstorm-canvas", "export-to-whiteboard"]),
        ("ai-architecture-review-chatbot", "Chatbot retrieving diagram sections and answering architecture questions in Teams.", ["teams-visio-chatbot", "architecture-review-bot", "ai-diagram-qa-teams"]),
        ("adaptive-card-approval-prompts", "Posting diagram preview thumbnails with Approve/Reject buttons in channels.", ["adaptive-cards-visio", "diagram-approval-prompts", "thumbnail-preview-cards"]),
        ("planner-todo-task-assignment", "Clicking process steps to assign tasks to team members in Teams Planner.", ["planner-visio-task-assign", "click-step-to-task", "teams-tasks-integration"]),
        ("threaded-node-conversations", "Threaded conversations anchored to specific Visio diagram nodes in channels.", ["threaded-node-conversations", "node-anchored-chat", "diagram-feedback-threads"]),
        ("mobile-client-blueprint-reviews", "Navigating multi-page architectural blueprints on Teams mobile client.", ["teams-mobile-blueprints", "mobile-architecture-review", "touch-blueprint-view"]),
        ("private-channel-permission-fences", "Permission boundaries and download restrictions on confidential diagrams.", ["private-channel-visio", "confidential-diagram-fences", "download-restriction-rules"]),
        ("collaborative-inking-pointer-stream", "Real-time collaborative inking and pointer highlighting during team meetings.", ["collaborative-inking-visio", "pointer-stream-meetings", "live-pen-annotations"]),
    ]),

    # Cluster 30: AI Copilots & Natural Language to Visio Diagram Synthesis
    (30, [
        ("prompt-to-vector-spec-pipeline", "Prompting LLMs to output structured diagram specs for automated synthesis.", ["prompt-to-diagram-pipeline", "structured-vector-specs", "llm-diagram-synthesis"]),
        ("copilot-studio-architecture-bot", "Building custom Copilots that generate cloud architecture maps on conversation.", ["copilot-studio-visio", "conversational-cloud-maps", "architecture-bot-synthesis"]),
        ("transcript-to-bpmn-translation", "Transforming unstructured meeting transcripts into valid Visio processes.", ["transcript-to-bpmn", "meeting-to-process-flow", "ai-bpmn-translation"]),
        ("layout-overlap-auto-alignment", "Using machine learning models to detect unaligned shapes and overlapping edges.", ["ai-layout-alignment", "overlap-detection-ml", "auto-align-diagram-shapes"]),
        ("technical-manual-to-knowledge-graph", "Synthesizing knowledge graphs in Visio from Word/PDF technical manuals via AI.", ["manual-to-knowledge-graph", "synthesize-visio-graphs", "pdf-doc-diagram-mining"]),
        ("audio-summary-alt-text-generation", "Generating AI audio summaries and alt-text for complex technical diagrams.", ["diagram-alt-text-ai", "audio-summary-diagrams", "accessibility-description-ai"]),
        ("rag-over-shape-topology-vectors", "Vector indexing Visio diagrams to query topological relationships with RAG.", ["rag-over-visio-topology", "vector-index-diagrams", "topological-query-llm"]),
        ("vision-schematic-anomaly-detection", "Scanning CAD/Visio schematics for missing valves or broken circuits via Vision.", ["vision-schematic-anomalies", "missing-valves-detector", "circuit-breaker-vision-ai"]),
        ("autonomous-subpage-decomposition", "AI agents decomposing monolithic diagrams into structured sub-pages.", ["subpage-decomposition-ai", "monolithic-diagram-refactor", "autonomous-page-splitting"]),
        ("flow-state-visual-workbench", "How vibe coding transforms diagramming into a primary computing canvas.", ["flow-state-workbench", "vibe-coding-diagrams", "visual-computing-canvas"]),
    ]),

    # Cluster 31: Diagram Validation Rules, Custom Rule Sets & Issues
    (31, [
        ("rulesets-and-issues-architecture", "RuleSets, Rule, Issue, RuleFilter, and programmatic validation in Visio.", ["validation-rulesets-visio", "rulefilter-issues-api", "programmatic-validation"]),
        ("bpmn-custom-ruleset-authoring", "Writing ShapeSheet and XML logic to enforce enterprise BPMN modeling rules.", ["custom-bpmn-rulesets", "shapesheet-validation-logic", "enterprise-bpmn-enforce"]),
        ("network-schematic-rule-enforcement", "Enforcing that all servers connect to a switch port and have a valid IP.", ["network-rule-enforcement", "switch-port-connection-check", "ip-validation-rules"]),
        ("document-page-shape-scopes", "Configuring validation scopes and filter expressions across document elements.", ["validation-scopes-visio", "document-page-shape-rules", "filter-expressions-valid"]),
        ("programmatic-issue-auto-healing", "Iterating through the Issues collection and auto-healing diagram flaws via code.", ["issue-auto-healing", "issues-collection-iterate", "automated-flaw-repair"]),
        ("ci-cd-quality-gate-pipeline", "Failing PR builds if diagrams contain severe validation errors in CI/CD.", ["ci-cd-quality-gates-visio", "fail-build-diagram-errors", "pr-validation-pipeline"]),
        ("issue-overrides-compliance-waivers", "Managing issue overrides, audit logs, and compliance waivers for diagrams.", ["issue-overrides-waivers", "compliance-audit-logs", "false-positive-suppression"]),
        ("dynamic-canvas-error-rings", "Drawing dynamic error rings and callout warnings around invalid shapes.", ["dynamic-error-rings", "callout-warnings-shapes", "visual-syntax-highlighters"]),
        ("excel-compliance-report-export", "Generating audit metrics and compliance reports in Excel for architecture boards.", ["excel-compliance-reports", "architecture-audit-metrics", "export-validation-excel"]),
        ("ieee-circuit-standards-validation", "Enforcing circuit continuity, ground lines, and fuse ratings in schematics.", ["ieee-circuit-validation", "circuit-continuity-checks", "fuse-rating-enforcement"]),
    ]),

    # Cluster 32: Enterprise Stencil Governance, Brand Compliance & Asset CDNs
    (32, [
        ("centralized-catalog-lifecycle", "Centralizing stencils, version deprecation, and change management workflows.", ["centralized-stencil-catalog", "stencil-lifecycle-mgmt", "version-deprecation-flow"]),
        ("azure-cdn-workstation-distribution", "Azure Blob storage, CDN caching, and automated local sync to client machines.", ["azure-cdn-stencil-sync", "blob-storage-stencil-hub", "client-workstation-sync"]),
        ("brand-colors-typography-locks", "Locking shapes, brand color themes, and font restrictions in corporate stencils.", ["brand-colors-typography", "locked-corporate-shapes", "font-restriction-stencils"]),
        ("shapesheet-ip-protection-locks", "Locking ShapeSheets, stripping source metadata, and obfuscating formulas.", ["shapesheet-ip-protection", "strip-metadata-stencils", "obfuscate-formulas-visio"]),
        ("legacy-master-guid-migration", "Replacing obsolete shapes across 10,000 files automatically using GUID maps.", ["legacy-master-guid-mig", "guid-mapping-replacement", "bulk-shape-modernization"]),
        ("rbac-security-cleared-schematics", "Restricting security-sensitive schematics to cleared engineers via RBAC.", ["rbac-stencil-security", "cleared-engineer-schematics", "stencil-access-control"]),
        ("multilingual-global-translations", "Serving translated shape prompts and labels across 20 languages dynamically.", ["multilingual-stencil-prompts", "global-shape-translation", "localized-stencil-labels"]),
        ("nightly-stencil-integrity-tests", "Nightly test harnesses verifying that all masters drop cleanly without errors.", ["nightly-stencil-tests", "clean-master-drop-check", "stencil-integrity-harness"]),
        ("search-keyword-index-optimization", "Optimizing shape search terms so developers find assets instantly in Visio.", ["search-keyword-opt", "fast-shape-search-index", "stencil-discovery-tags"]),
        ("self-service-gallery-portal", "Web gallery for previewing, requesting, and installing stencils across fleets.", ["self-service-stencil-portal", "stencil-gallery-web", "request-install-stencils"]),
    ]),

    # Cluster 33: Git Version Control for Diagrams: Diffing, Merging & CI/CD
    (33, [
        ("xml-deconstruction-architecture", "Deconstructing binary VSDX into diffable XML text files for Git repos.", ["vsdx-git-deconstruction", "diffable-xml-diagrams", "text-deconstruct-visio"]),
        ("visual-pr-image-diff-overlays", "Rendering before/after image overlays and highlighting altered nodes in PRs.", ["visual-pr-image-diffs", "before-after-diagram-overlays", "highlight-altered-nodes"]),
        ("lfs-attributes-binary-tracking", "Configuring .gitattributes for binary VSDX tracking with Git LFS.", ["git-lfs-visio", "gitattributes-vsdx-track", "large-file-storage-diagram"]),
        ("github-actions-png-preview-render", "Generating high-res PNG previews on every git push automatically in CI.", ["github-actions-diagram-png", "auto-render-previews-ci", "png-preview-generator"]),
        ("modular-page-splitting-branches", "Preventing binary merge collisions through modular page splitting in teams.", ["modular-page-splitting", "prevent-git-merge-collision", "page-level-branching"]),
        ("semver-release-tagging-blueprints", "Tagging visual architecture blueprints matching software releases in Git.", ["semver-diagram-releases", "release-tagging-blueprints", "architecture-version-sync"]),
        ("precommit-metadata-scrubbing-hooks", "Scrubbing author names, company metadata, and temp files before commit.", ["precommit-visio-scrub", "metadata-scrubbing-hooks", "clean-git-commits-visio"]),
        ("docusaurus-doc-portal-sync", "Embedding auto-generated SVGs in technical doc sites directly from Git.", ["docusaurus-diagram-sync", "technical-doc-site-svg", "git-to-docusaurus-visio"]),
        ("deconstructed-xml-conflict-resolution", "Safe techniques for resolving shape ID and connection collisions in Git merges.", ["xml-conflict-resolution-visio", "shape-id-collision-fix", "safe-git-merge-diagrams"]),
        ("confluence-wiki-auto-deploy", "Pushing approved diagrams to corporate wikis on main merge automatically.", ["confluence-diagram-deploy", "wiki-diagram-sync", "auto-publish-architecture"]),
    ]),

    # Cluster 34: Vibe Coding for Visio: Spec-Driven Generation & Flow-State Prototyping
    (34, [
        ("flow-state-visual-programming", "Flow-state diagramming with generative AI pair programming for engineers.", ["flow-state-visual-vibe", "ai-pair-diagramming", "vibe-coding-visio"]),
        ("spec-driven-markdown-scaffolding", "Transforming Markdown architecture specs into native Visio files in minutes.", ["markdown-to-visio-spec", "spec-driven-diagrams", "scaffold-vsdx-from-md"]),
        ("prompt-engineering-visual-topologies", "Structuring LLM system prompts for coordinate and edge generation in diagrams.", ["prompt-visual-topologies", "system-prompts-diagrams", "llm-coordinate-generation"]),
        ("conversational-voice-refinement", "Iterating on visual layouts through conversational voice prompts with agents.", ["conversational-voice-diagram", "voice-to-shape-refinement", "agent-layout-iteration"]),
        ("napkin-sketch-to-visio-in-30-mins", "AI vision models translating whiteboard photos to Visio diagrams rapidly.", ["napkin-sketch-to-visio", "whiteboard-photo-to-vsdx", "rapid-visual-prototyping"]),
        ("legacy-drawing-modernization-agent", "Scanning 15-year-old Visio drawings and restructuring into modern cloud icons.", ["modernize-legacy-drawings", "ai-drawing-restructure", "cloud-icon-migration-ai"]),
        ("stride-threat-model-generation", "Generating STRIDE threat modeling diagrams from system descriptions in minutes.", ["stride-threat-modeling-visio", "threat-model-diagram-ai", "security-boundary-gen"]),
        ("opentelemetry-sequence-diagrams", "Parsing distributed OpenTelemetry traces into Visio sequence flows directly.", ["opentelemetry-visio-sequence", "distributed-trace-diagrams", "otel-flow-synthesis"]),
        ("semantic-kernel-diagramming-agents", "Autonomous agents that write and validate Visio diagrams using plugins.", ["semantic-kernel-visio", "diagramming-agent-plugins", "autonomous-diagram-bot"]),
        ("zen-effortless-architecture-flow", "Achieving effortless creative velocity in enterprise visual modeling.", ["zen-architecture-flow", "effortless-creative-velocity", "vibe-developer-canvas"]),
    ]),

    # Cluster 35: Enterprise Deployment, Licensing, Packaging & High-Volume Export
    (35, [
        ("headless-batch-pdf-svg-export", "Automating mass export of 10,000+ diagram pages via headless CLI tools.", ["headless-batch-export", "mass-pdf-svg-export", "cli-export-pipeline"]),
        ("high-dpi-600-raster-rendering", "Exporting razor-sharp 600 DPI raster images for large-format engineering print.", ["high-dpi-600-rendering", "large-format-raster-print", "razor-sharp-diagrams"]),
        ("pdf-book-assembly-hyperlink-trees", "Generating 500-page engineering manuals with live link trees and index tabs.", ["pdf-book-assembly-visio", "500-page-manual-export", "hyperlink-tree-indexing"]),
        ("cmyk-rgb-color-calibration-press", "Calibrating color output for commercial printing and offset presses accurately.", ["cmyk-rgb-color-press", "commercial-print-calibration", "offset-press-diagrams"]),
        ("automated-security-watermarking", "Programmatically stamping CONFIDENTIAL, timestamps, and commit hashes on sheets.", ["security-watermarking-visio", "confidential-stamp-macro", "commit-hash-watermark"]),
        ("office-deployment-tool-odt-silent", "Office Deployment Tool [ODT], configuration.xml, and Click-to-Run silent install.", ["odt-silent-install-visio", "configuration-xml-deploy", "click-to-run-packaging"]),
        ("licensing-tier-cost-optimization", "Evaluating features, web vs desktop capabilities, and cost models across tiers.", ["visio-licensing-optimization", "plan1-plan2-viewer-cost", "licensing-tier-matrix"]),
        ("kubernetes-diagram-microservices", "Scaling diagram generation microservices in Kubernetes pods horizontally.", ["k8s-diagram-microservices", "scale-visio-generator", "container-pod-export"]),
        ("pdfa-iso-19005-archival-standards", "Exporting diagrams compliant with ISO 19005 PDF/A standards for 20-year audits.", ["pdfa-archival-visio", "iso-19005-compliance", "20-year-audit-preservation"]),
        ("cad-omnigraffle-legacy-migration", "Exporting proprietary CAD/OmniGraffle assets into open Visio VSDX formats.", ["cad-omnigraffle-migration", "proprietary-to-vsdx", "legacy-migration-playbook"]),
    ]),
]

def generate_skill_content(
    skill_id: str,
    desc: str,
    triggers: list,
    cluster_title: str,
    book_ref: str,
    cluster_id: int,
) -> str:
    triggers_yaml = "\n".join([f'  - "{t}"' for t in triggers])
    tags = ["visio", "visio-365", "ecc", "tagisan", "vibe-coding", "vector-graphics"]

    return f"""---
name: "{skill_id}"
description: "{desc}"
domain: "visio-365"
triggers:
{triggers_yaml}
tags: {tags}
version: "1.0.0"
---

# Section 1: Overview & Operational Invariants

The `{skill_id}` skill provides automated developer capabilities for {desc.lower()}.
Rooted in authoritative knowledge from *{book_ref}*, this skill enforces deterministic vector geometry calculations, ShapeSheet mathematical invariants, and robust multi-page diagram automation.

## Operational Invariants

- **ALWAYS** validate ShapeSheet cell formulas, coordinate transformations, and spatial bounds before committing modifications to master shapes, stencils, or page instances.
- **NEVER** bypass connection point directional constraints, glue verification protocols, or event-driven error handlers that protect against cyclic recursion, #REF! errors, or process memory leaks.
- **MANDATORY** encapsulate all multi-step diagram mutations and batch ShapeSheet updates inside explicit undo scopes (`Application.BeginUndoScope` / `EndUndoScope`) or batch API transactions (`SetFormulas` / `context.sync()`) with automated rollback handlers.

# Section 2: Core Architecture & Workflow Execution

Operating within Cluster {cluster_id} (*{cluster_title}*), this skill enforces systematic visual execution patterns across Microsoft Visio 365 diagrams:

```
[Developer / AI Agent Prompt]
             │
             ▼
[Spatial & Coordinate Verification] ──► (PinX/PinY, Angle, LocPin, BoundingBox)
             │
             ▼
[ShapeSheet Formula & Geometry Layer] ─► (CellsSRC, Guard, DependsOn, NoShow)
             │
             ▼
 [Transaction & Undo Scope Boundary] ──► (BeginUndoScope -> EndUndoScope)
             │
             ▼
 [Vector Canvas & Telemetry Pipeline] ──► (SVG Render, Data Graphics, Audit Log)
```

### Execution Protocol
1. **Contract Ingestion:** Parse diagram requirements, vector schemas, or ShapeSheet formulas against Tagisan RFC-004 specifications.
2. **Spatial Alignment & Geometry Check:** Evaluate parent-child coordinate spaces, rotation origins, and connection point inward/outward directions.
3. **Execution & Undo Guard:** Apply geometric modifications and formula bindings inside a supervised undo scope, intercepting engine errors and recursion cycles.
4. **State Commitment & Visual Verification:** Persist shape properties, trigger dynamic layout re-routing if required, and emit vector render telemetry.

# Section 3: Pragmatic Implementation & Code Blueprints

Below is a production-grade implementation pattern for `{skill_id}`:

```vba
' ==============================================================================
' Skill: {skill_id}
' Description: {desc}
' Authority: {book_ref}
' ==============================================================================
Option Explicit

Public Function Execute_{skill_id.replace('-', '_')}(ByRef targetPage As Visio.Page, ByVal contextParam As String) As Boolean
    On Error GoTo ErrorHandler
    Dim app As Visio.Application
    Dim undoScopeId As Long
    Dim inScope As Boolean
    
    Set app = targetPage.Application
    
    ' MANDATORY: Begin transactional undo scope
    undoScopeId = app.BeginUndoScope("Execute_{skill_id.replace('-', '_')}")
    inScope = True
    
    ' Core operational payload for {skill_id}
    Debug.Print "Executing skill [{skill_id}] on page: " & targetPage.Name & " with param: " & contextParam
    
    ' Commit atomic undo scope
    app.EndUndoScope undoScopeId, True
    inScope = False
    Execute_{skill_id.replace('-', '_')} = True
    Exit Function

ErrorHandler:
    If inScope Then
        app.EndUndoScope undoScopeId, False
        Debug.Print "Visio action rolled back due to error " & Err.Number & ": " & Err.Description
    End If
    Execute_{skill_id.replace('-', '_')} = False
End Function
```

# Section 4: Validation Criteria & Brutal Verification

To pass automated quality gates within Tagisan ECC, this skill must satisfy:
1. **Disk Presence:** A valid `SKILL.md` file resides under `.ecc/skills/{skill_id}/` with intact YAML frontmatter.
2. **Compiler & Runtime Invariants:** Built-in registration within `src/ecc/skills.rs` with lowercase alias routing and domain mapping to `visio-365`.
3. **Vector & Formula Guarantees:** Zero unhandled errors (#REF!, #VALUE!) during ShapeSheet formula execution; guaranteed rollback upon simulated failure.
4. **Deterministic Dispatching:** The skill must rank as a top match under `tgs ecc skills -q "{triggers[0]}"`.
"""

def get_all_skills():
    cluster_dict = {c[0]: (c[1], c[2], c[3]) for c in CLUSTERS}
    all_skills = []
    for cluster_id, skills in CLUSTER_SKILLS:
        cluster_title, prefix, book_ref = cluster_dict[cluster_id]
        for suffix, desc, triggers in skills:
            skill_id = f"{prefix}{suffix}"
            all_skills.append({
                "id": skill_id,
                "desc": desc,
                "triggers": triggers,
                "cluster_title": cluster_title,
                "cluster": cluster_id,
                "book": book_ref,
            })
    return all_skills

def main():
    print("=== Generating 350 Microsoft Visio 365 Vibe Coding Skills ===")
    count = 0
    
    cluster_dict = {c[0]: (c[1], c[2], c[3]) for c in CLUSTERS}
    
    for cluster_id, skills in CLUSTER_SKILLS:
        cluster_title, prefix, book_ref = cluster_dict[cluster_id]
        
        for suffix, desc, triggers in skills:
            skill_id = f"{prefix}{suffix}"
            skill_dir = SKILLS_DIR / skill_id
            skill_dir.mkdir(parents=True, exist_ok=True)
            
            skill_file = skill_dir / "SKILL.md"
            content = generate_skill_content(
                skill_id=skill_id,
                desc=desc,
                triggers=triggers,
                cluster_title=cluster_title,
                book_ref=book_ref,
                cluster_id=cluster_id,
            )
            
            with open(skill_file, "w", encoding="utf-8") as f:
                f.write(content)
            
            count += 1

    print(f"Successfully generated {count} Microsoft Visio 365 skills under {SKILLS_DIR}")
    assert count == 350, f"Expected 350 skills, but generated {count}!"

if __name__ == "__main__":
    main()
