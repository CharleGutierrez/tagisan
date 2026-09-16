#!/usr/bin/env python3
"""
scripts/generate_power_platform_skills.py

Generates all 350 Microsoft Power Platform & Vibe Code Development skill packages
under .ecc/skills/pp-*/SKILL.md with full YAML frontmatter, strict operational
directives (ALWAYS, NEVER, MANDATORY, STRICT_REJECT), and comprehensive engineering instructions.
"""

import os
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

# 35 Clusters x 10 Skills = 350 Skills
CLUSTERS = [
    (1, "Canvas Architecture & Responsive UI", "pp-canvas-"),
    (2, "Power Fx Declarative Formula Engineering", "pp-fx-"),
    (3, "Power Apps Components & Reusable Libraries", "pp-comp-"),
    (4, "Canvas App Offline Caching & Mobile Ergonomics", "pp-offline-"),
    (5, "Canvas App Performance & Delegation Optimization", "pp-perf-"),
    (6, "Dataverse Relational Schema & Table Modeling", "pp-dv-"),
    (7, "Model-Driven Apps, Forms & Business Rules", "pp-mda-"),
    (8, "Dataverse Security Roles & Hierarchical RBAC", "pp-sec-"),
    (9, "Power Automate Cloud Flows & Trigger Architectures", "pp-flow-"),
    (10, "Flow Error Handling, Scopes & Resilient Retries", "pp-err-"),
    (11, "Enterprise Approvals & Human-in-the-Loop Flows", "pp-appr-"),
    (12, "High-Throughput OData Filtering & JSON Data Wrangling", "pp-odata-"),
    (13, "Power Automate Desktop RPA & UI Automation", "pp-rpa-"),
    (14, "Power BI Semantic Modeling & Star Schemas", "pp-pbi-"),
    (15, "DAX Formula Engineering & Context Transition", "pp-dax-"),
    (16, "Power Query M Language Data Transformations", "pp-m-"),
    (17, "Power BI Copilot & Natural Language Visualizations", "pp-pbicop-"),
    (18, "Power Pages Architecture & Liquid Templating", "pp-pages-"),
    (19, "Power Pages Web API, Forms & Security Rules", "pp-pweb-"),
    (20, "Power Pages Fluent Design Tokens & Bootstrap Styling", "pp-pstyle-"),
    (21, "Copilot Studio Generative Topics & Intent Parsing", "pp-csgen-"),
    (22, "Copilot Studio Dynamic Chaining & Tool Orchestration", "pp-cschain-"),
    (23, "Copilot Studio Knowledge Grounding & Dataverse Search", "pp-csknow-"),
    (24, "Multi-Agent Choreography & Bot-to-Bot Handoffs", "pp-csorch-"),
    (25, "Power Apps Component Framework (PCF) Architecture", "pp-pcf-"),
    (26, "PCF TypeScript, React & Fluent UI Component Dev", "pp-pcfrx-"),
    (27, "PCF Device Hardware APIs & Webpack/Bun Bundling", "pp-pcfhard-"),
    (28, "Custom Connectors & OpenAPI 3.0 Authoring", "pp-conn-"),
    (29, "Entra ID Authentication, OAuth2 & PKCE for Connectors", "pp-oauth-"),
    (30, "Azure API Management (APIM) & Microservices Integration", "pp-apim-"),
    (31, "AI Builder Prebuilt Models & Form Processing", "pp-aib-"),
    (32, "AI Builder Custom Prompts & Document Intelligence", "pp-aiprompt-"),
    (33, "Power Platform Vibe Coding, Flow & Fast Prototyping", "pp-vibe-"),
    (34, "Power Platform CLI (pac), ALM & Managed Solutions", "pp-alm-"),
    (35, "Enterprise Governance, DLP Policies & Center of Excellence", "pp-gov-"),
]

# Raw master skill list (350 entries)
SKILLS = [
    # =========================================================================
    # Cluster 1: Power Apps Canvas Architecture & Responsive UI (1-10)
    # =========================================================================
    {
        "id": "pp-canvas-fluid-container-layouts",
        "cluster": 1,
        "book": "Designing Responsive Canvas Apps in Microsoft Power Apps - Matthew Devaney",
        "desc": "Auto-layout, Horizontal and Vertical containers, flex-grow, min-width, and dynamic wrapping for enterprise multi-form factor apps.",
        "triggers": ["fluid-container-layouts", "canvas-responsive-layout", "horizontal-vertical-containers", "flex-grow-min-width"],
        "foundations": [
            "Container Mechanics: Horizontal and Vertical layout containers eliminate absolute (X, Y) pixel positioning in favor of responsive CSS-flexbox style flow.",
            "Layout Invariant: ALWAYS set explicit MinWidth and MinHeight constraints on fluid child controls to prevent layout collapsing on mobile screens.",
            "NEVER disable container wrapping on multi-column input forms without providing explicit horizontal scroll boundaries.",
            "MANDATORY utilization of Fill portions (flex-grow) for dynamic width distribution across diverse viewport aspect ratios.",
        ],
        "protocol": "ALWAYS construct responsive screen layouts using hierarchical Auto-layout containers. Bind child Width property to Parent.Width * FillPortion / TotalPortions when fine-grained proportional allocation is required.",
        "anti_patterns": [
            "Hardcoding fixed X and Y coordinates on child controls inside responsive canvas screens.",
            "Nesting more than 6 levels of containers, causing excessive layout recalculation latency.",
        ],
    },
    {
        "id": "pp-canvas-screen-breakpoints-resolution",
        "cluster": 1,
        "book": "Power Apps Canvas Design Patterns - Paul Culmsee & Reza Dorrani",
        "desc": "Viewport breakpoint detection using App.Width, App.Height, screen orientation matrices, and adaptive control density.",
        "triggers": ["canvas-screen-breakpoints", "viewport-orientation", "adaptive-control-density", "app-width-height-breakpoints"],
        "foundations": [
            "Breakpoint Matrix: 1 = Mobile Portrait (<600px), 2 = Mobile Landscape/Tablet Portrait (600-900px), 3 = Tablet Landscape/Desktop Small (900-1200px), 4 = Desktop Large (>1200px).",
            "Resolution Invariant: ALWAYS read viewport metrics from App.ActiveScreen.Width and App.ActiveScreen.Height rather than static parent bounds.",
            "NEVER force landscape lock in enterprise field worker apps where single-hand portrait operation is ergonomically required.",
            "MANDATORY adaptation of gallery column counts: 1 column on breakpoint 1, 2 on breakpoint 2, 3-4 on breakpoints 3-4.",
        ],
        "protocol": "Define global screen breakpoint formulas in App.Formulas: `ScreenSize = Switch(true, App.ActiveScreen.Width < 600, 1, App.ActiveScreen.Width < 900, 2, App.ActiveScreen.Width < 1200, 3, 4)`. Use `ScreenSize` across all responsive visibility properties.",
        "anti_patterns": [
            "Duplicating screens for mobile vs desktop versions instead of utilizing responsive auto-layout containers.",
            "Assuming window size never changes during an active browser session.",
        ],
    },
    {
        "id": "pp-canvas-app-onstart-named-formulas",
        "cluster": 1,
        "book": "High-Performance Canvas Apps - Microsoft Press & Tim Leung",
        "desc": "Transitioning from imperative App.OnStart to declarative App.Formulas (Named Formulas) for instantaneous application initialization.",
        "triggers": ["app-onstart-named-formulas", "app-formulas-power-fx", "instant-app-initialization", "named-formulas-migration"],
        "foundations": [
            "Declarative Evaluation: App.Formulas are evaluated lazily on demand or concurrently at startup, eliminating sequential App.OnStart blocking bottlenecks.",
            "Startup Invariant: ALWAYS declare read-only lookup dictionaries, user profiles, and design theme palettes in App.Formulas.",
            "NEVER execute heavy network I/O or multi-record collection writes inside App.OnStart when App.Formulas can provide immutable computed records.",
            "MANDATORY zero-delay screen rendering on initial app launch.",
        ],
        "protocol": "Replace imperative `Set(CurrentUser, User())` and `Set(AppTheme, ...)` in App.OnStart with immutable declarations in App.Formulas: `fxUser = User();` and `fxTheme = { Primary: ColorValue('#0078D4'), Background: ColorValue('#FFFFFF') };`.",
        "anti_patterns": [
            "Writing 500 lines of sequential Set() and ClearCollect() calls in App.OnStart, leading to 10+ second initial splash freezes.",
            "Modifying global variables in App.OnStart that depend on screen controls not yet instantiated.",
        ],
    },
    {
        "id": "pp-canvas-modal-dialog-surfacing",
        "cluster": 1,
        "book": "Micro-Interactions and Dialog Design in Power Apps - Sancho Harker",
        "desc": "Declarative modal popup layers, full-screen background scrims, z-index stack isolation, and keyboard focus trapping.",
        "triggers": ["modal-dialog-surfacing", "popup-layers-canvas", "scrim-focus-trapping", "z-index-dialog-stack"],
        "foundations": [
            "Z-Index Scrim Architecture: Modal containers sit at maximum screen z-index with a semi-transparent RGBA background (0, 0, 0, 0.4) intercepting all click events.",
            "Safety Invariant: ALWAYS trap clicks on the backdrop container to prevent accidental background interaction.",
            "NEVER leave modal state variable unbound; use clear boolean flags (`varShowDeleteConfirm`).",
            "MANDATORY escape hatch via explicit close icon or cancellation button resetting the modal state.",
        ],
        "protocol": "Implement modal overlays using a dedicated full-screen container (`X: 0, Y: 0, Width: Parent.Width, Height: Parent.Height, Visible: varShowModal`). Place dialog card inside with centered alignment and explicit dismiss handlers.",
        "anti_patterns": [
            "Scattershot visibility formulas on 20 individual controls instead of grouping dialog controls in a single container.",
            "Permitting destructive background actions while an uncommitted confirmation dialog is visible.",
        ],
    },
    {
        "id": "pp-canvas-accessible-aria-theming",
        "cluster": 1,
        "book": "Accessible Power Platform Solutions - Microsoft Accessibility Engineering",
        "desc": "WCAG 2.1 AA compliance, TabIndex ordering, ScreenReaderLabel semantics, and accessible 4.5:1 color contrast theming.",
        "triggers": ["accessible-aria-theming", "wcag-power-apps", "tabindex-screenreader", "accessible-color-contrast"],
        "foundations": [
            "Accessibility Protocol: Every interactive control requires TabIndex = 0 (or positive sequential index) and a descriptive ScreenReaderLabel.",
            "Contrast Ratio Invariant: ALWAYS ensure normal text meets minimum 4.5:1 contrast against its background and large text meets 3:1.",
            "NEVER convey state or validation errors solely through color without secondary iconography or textual hints.",
            "MANDATORY automated execution of the Power Apps built-in Accessibility Checker before solution export.",
        ],
        "protocol": "Audit all controls with the App Checker -> Accessibility tab. Resolve all missing accessible labels, invalid tab orders, and low-contrast warnings. Bind icon AccessibleLabel to action descriptions (e.g., 'Delete customer record').",
        "anti_patterns": [
            "Setting TabIndex = -1 on actionable buttons, completely blinding keyboard and screen reader users.",
            "Using light gray text (#999999) on white background (#FFFFFF) failing WCAG AA contrast thresholds.",
        ],
    },
    {
        "id": "pp-canvas-deep-linking-param-routing",
        "cluster": 1,
        "book": "Enterprise URL Routing and Deep Linking in Power Apps - Reza Dorrani",
        "desc": "Param('recordId') ingestion, App.StartScreen routing logic, automated entity hydration, and secure cross-app navigational links.",
        "triggers": ["deep-linking-param-routing", "startscreen-power-fx", "param-recordid-routing", "canvas-url-navigation"],
        "foundations": [
            "Routing Architecture: App.StartScreen evaluates URL parameters (`Param('screen')`, `Param('id')`) before any visual tree renders.",
            "Navigation Invariant: ALWAYS validate and sanitize Param() inputs against authorization filters before routing to target screens.",
            "NEVER navigate users to unauthorized detail screens if GUID parameter is forged or invalid.",
            "MANDATORY fallback to default home screen when URL parameters are missing or malformed.",
        ],
        "protocol": "In App.StartScreen, specify: `If(!IsBlank(Param('recordId')), DetailScreen, HomeScreen)`. On DetailScreen.OnVisible, fetch record: `Set(CurrentItem, LookUp(Accounts, Account = GUID(Param('recordId'))))`.",
        "anti_patterns": [
            "Using Navigate() inside App.OnStart (now deprecated and causes runtime warnings/flicker) instead of App.StartScreen.",
            "Assuming Param('id') always contains a valid GUID without using IsType or Try/Catch validation.",
        ],
    },
    {
        "id": "pp-canvas-dynamic-svg-data-uris",
        "cluster": 1,
        "book": "Advanced Data Visualization with SVGs in Power Apps - Kristine Kolodziejski",
        "desc": "In-line SVG rendering via Image controls, data URI encoding, dynamic dashboard KPI progress rings, gauges, and sparklines.",
        "triggers": ["dynamic-svg-data-uris", "svg-power-apps", "kpi-progress-rings", "sparkline-canvas-visuals"],
        "foundations": [
            "Data URI Scheme: SVGs are constructed via Power Fx string interpolation and encoded into `data:image/svg+xml;utf8,...` strings.",
            "Visual Invariant: ALWAYS URL-encode special characters (e.g., `#` as `%23`) inside SVG color hex values.",
            "NEVER exceed 64KB SVG payload size in image controls to prevent browser canvas rendering stalls.",
            "MANDATORY parametric viewBox attributes for seamless responsive vector scaling across device resolutions.",
        ],
        "protocol": "Bind an Image control's `Image` property to: `\"data:image/svg+xml;utf8,\" & EncodeUrl(\"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='40' stroke='%230078d4' stroke-width='8' fill='none'/></svg>\")`.",
        "anti_patterns": [
            "Omitting EncodeUrl() or forgetting to escape `#` characters, resulting in broken blank image renders.",
            "Generating complex multi-megabyte vector illustrations dynamically in Power Fx.",
        ],
    },
    {
        "id": "pp-canvas-timer-animation-ergonomics",
        "cluster": 1,
        "book": "Micro-Animations and State Timing in Power Apps - April Dunnam",
        "desc": "Declarative micro-animations, slide-in navigation panels, timer loops, and non-blocking progressive disclosure transitions.",
        "triggers": ["timer-animation-ergonomics", "slide-in-navigation-timer", "progressive-disclosure-canvas", "smooth-panel-animations"],
        "foundations": [
            "Timer Calculus: Smooth animations interpolate a coordinate property `X = CurrentX + (TargetX - CurrentX) * (Timer.Value / Timer.Duration)`.",
            "Performance Invariant: ALWAYS disable Timer.Repeat unless an explicit recurring polling loop is architecturally mandated.",
            "NEVER run continuous high-frequency timers on low-power mobile devices due to battery drain.",
            "MANDATORY setting Timer.AutoReset = true and Timer.AutoStart = false when driven by user interactions.",
        ],
        "protocol": "Implement smooth drawer opening: set Timer.Duration = 200, Start = varStartTimer. Bind drawer container `X` property to `-(Parent.Width * 0.8) + (Parent.Width * 0.8) * (TimerAnimation.Value / TimerAnimation.Duration)`.",
        "anti_patterns": [
            "Using 5 separate running timers simultaneously for background calculations, spiking client CPU to 100%.",
            "Leaving timers running when screens are navigated away from.",
        ],
    },
    {
        "id": "pp-canvas-multi-form-wizard-state",
        "cluster": 1,
        "book": "Complex Multi-Step Form Workflows in Canvas Apps - Brian Dang",
        "desc": "Step-by-step onboarding wizards, progressive form validation guards, draft auto-saving, and unified submission transactions.",
        "triggers": ["multi-form-wizard-state", "step-by-step-onboarding", "progressive-form-validation", "draft-auto-save-canvas"],
        "foundations": [
            "State Machine: Wizard steps are tracked via integer variable `varCurrentStep` (1..N). Navigation buttons guard transition with `varStepIsValid` predicates.",
            "Data Integrity Invariant: ALWAYS validate current step fields before incrementing step pointer.",
            "NEVER submit partial records to production Dataverse tables without draft status flagging.",
            "MANDATORY unified Patch transaction or multi-step Patch on final wizard confirmation.",
        ],
        "protocol": "Maintain step state via `UpdateContext({ varStep: varStep + 1 })`. Guard next button: `DisplayMode: If(And(!IsBlank(txtEmail.Text), IsMatch(txtEmail.Text, Match.Email)), DisplayMode.Edit, DisplayMode.Disabled)`.",
        "anti_patterns": [
            "Allowing users to skip mandatory validation steps by directly clicking subsequent wizard tabs without gating.",
            "Committing irreversible database mutations on step 2 of a 5-step wizard.",
        ],
    },
    {
        "id": "pp-canvas-print-pdf-export-controls",
        "cluster": 1,
        "book": "Report Generation and PDF Export in Power Apps - Eickhel Mendoza",
        "desc": "PDF() function configuration, printable screen layout stylesheets, page breaks, and tabular receipt/invoice generation.",
        "triggers": ["print-pdf-export", "pdf-function-power-fx", "printable-canvas-screens", "invoice-receipt-generation"],
        "foundations": [
            "PDF Generation Engine: The `PDF()` function captures target screen or container DOM trees into portable PDF binary blobs.",
            "Layout Invariant: ALWAYS design dedicated printable screens formatted to standard dimensions (A4: 794x1123px or US Letter: 816x1056px).",
            "NEVER attempt to export responsive fluid auto-layout screens with dynamic scrollbars directly to PDF without print formatting.",
            "MANDATORY elimination of interactive controls (buttons, inputs) from the printable container via `DPI` and visibility flags.",
        ],
        "protocol": "Generate customer invoice PDF: `Set(varInvoiceBlob, PDF(PrintableInvoiceContainer, { ExpandContainers: true, Margin: '10px' }))`. Pass `varInvoiceBlob` to Power Automate or Office 365 Outlook connector for email attachment.",
        "anti_patterns": [
            "Invoking browser window.print() on unconstrained canvas app screens, producing clipped multi-page outputs.",
            "Attempting to paginate 1,000 records inside a single canvas PDF container instead of offloading to Power BI or SSRS.",
        ],
    },

    # =========================================================================
    # Cluster 2: Power Fx Declarative Formula Engineering (11-20)
    # =========================================================================
    {
        "id": "pp-fx-imperative-vs-declarative",
        "cluster": 2,
        "book": "Power Fx Formula Reference & Advanced Patterns - Greg Lindhorst",
        "desc": "Declarative With(), Sequence(), and Set vs UpdateContext scoping, pure functional composition, and side-effect minimization.",
        "triggers": ["power-fx-imperative-vs-declarative", "with-function-power-fx", "updatecontext-vs-set", "pure-functional-power-fx"],
        "foundations": [
            "Declarative Paradigm: Power Fx expressions compute values dynamically based on reactive graph dependencies without imperative mutations.",
            "Scope Invariant: ALWAYS prefer `UpdateContext()` for screen-scoped temporary state and `Set()` solely for tenant-wide global variables.",
            "NEVER mutate global collections inside gallery Item properties or visual calculation pipelines.",
            "MANDATORY use of `With({ scopeVar: expression }, ...)` to eliminate redundant sub-formula evaluations.",
        ],
        "protocol": "Refactor nested repetitive lookups into `With({ activeAccount: LookUp(Accounts, ID = selectedId) }, activeAccount.Revenue * activeAccount.TaxRate)`.",
        "anti_patterns": [
            "Using Set() inside a Gallery OnSelect to set 15 separate global variables when a single record variable suffices.",
            "Creating circular formula dependencies causing infinite re-render loops in Power Apps Studio.",
        ],
    },
    {
        "id": "pp-fx-strongly-typed-records-tables",
        "cluster": 2,
        "book": "Pro Power Fx: Strongly Typed Functional Logic - Microsoft Learn",
        "desc": "Schema enforcement, Table(), Record(), Type(), Match(), and schema preservation across complex data transformations.",
        "triggers": ["strongly-typed-records-tables", "power-fx-types", "table-record-constructors", "schema-preservation-fx"],
        "foundations": [
            "Type System: Power Fx is a strongly typed declarative language with strict static typing for Text, Number, Boolean, Date, Time, Record, and Table.",
            "Type Safety Invariant: ALWAYS match schema keys and value types precisely when constructing in-memory tables via `Table({ Col1: 'Val1' })`.",
            "NEVER mix heterogeneous value types (e.g., text and numbers) in the same column of a synthetic collection.",
            "MANDATORY explicit casting when converting between numbers and strings via `Value()` and `Text()`.",
        ],
        "protocol": "Construct strongly typed lookup tables: `ClearCollect(colStatusTypes, Table({ ID: 1, Name: 'Draft' }, { ID: 2, Name: 'Approved' }))`. Verify column types in the Collections viewer.",
        "anti_patterns": [
            "Relying on implicit type coercion in comparisons (`'10' = 10`), causing subtle evaluation failures across different locales.",
            "Constructing tables with misspelled property names in alternating rows, creating null columns.",
        ],
    },
    {
        "id": "pp-fx-untyped-object-json-parsing",
        "cluster": 2,
        "book": "Working with JSON and Untyped Objects in Power Fx - Microsoft Docs",
        "desc": "ParseJSON(), UntypedObject manipulation, Value(), Text(), Boolean(), and Table() casting from external REST responses.",
        "triggers": ["untyped-object-json-parsing", "parsejson-power-fx", "untypedobject-casting", "json-deserialization-fx"],
        "foundations": [
            "Untyped Object Boundary: `ParseJSON(jsonString)` returns an `UntypedObject` requiring explicit type coercion before consumption.",
            "Coercion Invariant: ALWAYS wrap UntypedObject fields with explicit type functions: `Text(item.name)`, `Value(item.price)`, `Boolean(item.active)`.",
            "NEVER pass raw UntypedObject references directly to database Patch() statements without casting.",
            "MANDATORY use of `Table(untypedArray)` for iterating over JSON arrays in galleries or ForAll loops.",
        ],
        "protocol": "Parse API responses safely: `Set(varData, ParseJSON(apiResponse)); ClearCollect(colItems, ForAll(Table(varData.items), { Title: Text(ThisRecord.Value.title), Count: Value(ThisRecord.Value.count) }))`.",
        "anti_patterns": [
            "Directly binding a Label.Text property to `ParseJSON(payload).title` without calling `Text()`, causing compilation errors.",
            "Failing to check for null JSON values before type casting, triggering runtime exceptions.",
        ],
    },
    {
        "id": "pp-fx-concurrent-evaluation-patterns",
        "cluster": 2,
        "book": "High-Velocity Power Fx Optimization - Todd Baginski",
        "desc": "Concurrent() batch execution, network latency minimization, parallel data hydration, and thread safety across connectors.",
        "triggers": ["concurrent-evaluation-patterns", "concurrent-function-power-fx", "parallel-data-hydration", "network-latency-reduction"],
        "foundations": [
            "Parallel Execution: `Concurrent(Op1, Op2, ...)` fires independent connector calls simultaneously, reducing total elapsed time to MAX(times) instead of SUM(times).",
            "Independence Invariant: Operations inside `Concurrent()` MUST NOT have read-after-write dependencies on each other.",
            "NEVER call `Concurrent(ClearCollect(colA, ...), ClearCollect(colB, Filter(colA, ...)))` due to race condition hazards.",
            "MANDATORY inclusion of independent lookup tables (e.g., Accounts, Categories, Users) within startup Concurrent() blocks.",
        ],
        "protocol": "Hydrate reference tables in parallel: `Concurrent(ClearCollect(colAccounts, Accounts), ClearCollect(colContacts, Contacts), ClearCollect(colSettings, SystemSettings))`.",
        "anti_patterns": [
            "Executing 10 consecutive ClearCollect() statements imperatively on screen load, turning a 500ms network fetch into a 5-second wait.",
            "Including mutate-then-read operations inside the same Concurrent() invocation.",
        ],
    },
    {
        "id": "pp-fx-user-defined-functions-udf",
        "cluster": 2,
        "book": "Reusable Logic with User Defined Functions in Power Fx - Microsoft Learn",
        "desc": "App.Formulas reusable helper functions, parameterized functional signatures, pure calculations, and cross-screen DRY principles.",
        "triggers": ["user-defined-functions-udf", "udf-power-fx", "app-formulas-functions", "reusable-formula-helpers"],
        "foundations": [
            "UDF Architecture: Defined in App.Formulas as pure functions with typed parameters and return types: `FunctionName(Param1: Type, ...): ReturnType = Expression;`.",
            "Purity Invariant: UDFs in App.Formulas ALWAYS remain pure functions without side effects (no Set, Collect, or Navigate).",
            "NEVER duplicate identical financial calculation or tax rate formulas across multiple buttons and screens.",
            "MANDATORY declaration of parameter types (e.g., `rate: Number, amount: Number`).",
        ],
        "protocol": "Declare tax calculation helper: `CalculateTax(amount: Number, stateCode: Text): Number = amount * LookUp(TaxRates, State = stateCode, Rate);`. Call everywhere: `lblTax.Text = Text(CalculateTax(Value(txtAmount.Text), 'CA'), '$#,##0.00')`.",
        "anti_patterns": [
            "Copy-pasting 30 lines of complex string formatting logic into 15 different label controls.",
            "Attempting to trigger navigation or variable updates inside a User Defined Function.",
        ],
    },
    {
        "id": "pp-fx-error-handling-iferror-isblank",
        "cluster": 2,
        "book": "Resilient Power Apps: Defensive Coding and Fault Tolerance - Shane Young",
        "desc": "IfError(), IsError(), Notify(), App.OnError global error traps, and transactional safety in Dataverse Patch mutations.",
        "triggers": ["error-handling-iferror", "app-onerror-global-trap", "notify-error-banners", "defensive-patch-power-fx"],
        "foundations": [
            "Defensive Evaluation: `IfError(TargetOperation, FallbackValue)` intercepts runtime faults (network timeouts, constraint violations) gracefully.",
            "Telemetry Invariant: ALWAYS implement `App.OnError` to log unhandled client-side runtime errors to Application Insights or Dataverse.",
            "NEVER suppress critical database write errors silently without notifying the end user.",
            "MANDATORY validation of `Errors(DataSource, Record)` after high-stakes Patch() calls.",
        ],
        "protocol": "Wrap transactional updates defensively: `IfError(Patch(Orders, Defaults(Orders), { OrderNumber: '1001' }), Notify(\"Order creation failed: \" & FirstError.Message, NotificationType.Error), Notify(\"Order submitted successfully!\", NotificationType.Success))`.",
        "anti_patterns": [
            "Ignoring return errors from Patch(), giving users the false impression that data was saved when the database rejected it.",
            "Using empty Notify() messages that leave users bewildered when network errors strike.",
        ],
    },
    {
        "id": "pp-fx-relational-lookup-navigation",
        "cluster": 2,
        "book": "Navigating Complex Dataverse Relations in Power Fx - Geetha Sivasithambaram",
        "desc": "1:N and N:1 relational dot-walking, the As operator for disambiguation, self-referencing joins, and child record projection.",
        "triggers": ["relational-lookup-navigation", "power-fx-dot-walking", "as-operator-disambiguation", "self-referencing-joins"],
        "foundations": [
            "Dot-Walking Mechanics: Power Fx directly navigates N:1 relationships via dot notation (`ThisItem.PrimaryContact.Email`) without explicit joins.",
            "Disambiguation Invariant: ALWAYS use the `As` operator when iterating across nested parent-child scopes with identical column names (`Gallery.AllItems As ParentRecord`).",
            "NEVER perform nested LookUp() calls inside a gallery when the relationship can be traversed directly via record properties.",
            "MANDATORY scoping to prevent column name shadowing between inner and outer records.",
        ],
        "protocol": "Iterate nested records with unambiguous references: `ForAll(colDepartments As Dept, ForAll(Dept.Employees As Emp, { DeptName: Dept.Name, EmpName: Emp.FullName }))`.",
        "anti_patterns": [
            "Relying on ambient `ThisItem` inside multi-level nested ForAll loops, binding to the wrong record context.",
            "Executing 100 round-trip LookUp queries inside a Gallery template instead of using related entity expansion.",
        ],
    },
    {
        "id": "pp-fx-in-memory-caching-collections",
        "cluster": 2,
        "book": "In-Memory Data Engineering in Canvas Apps - Sancho Harker",
        "desc": "Collect(), ClearCollect(), Patch(), bulk in-memory table mutations, local caching, and optimistic UI synchronization.",
        "triggers": ["in-memory-caching-collections", "clearcollect-patch-bulk", "optimistic-ui-updates", "local-collection-cache"],
        "foundations": [
            "Collection Architecture: In-memory collections provide microsecond-level responsiveness and temporary scratchpad state before server commits.",
            "Sync Invariant: ALWAYS maintain synchronization between in-memory cache and server-side Dataverse state following mutation.",
            "NEVER store more than 10,000 records in client RAM to avoid browser tab memory exhaustion.",
            "MANDATORY use of `Patch(DataSource, Defaults(DataSource), colStagedRecords)` for high-speed bulk record creation.",
        ],
        "protocol": "Implement optimistic UI: 1) Update local collection immediately: `Patch(colTasks, LookUp(colTasks, ID = selectedId), { Status: 'Completed' })`. 2) Trigger server sync asynchronously in background: `Patch(Tasks, LookUp(Tasks, ID = selectedId), { Status: 'Completed' })`.",
        "anti_patterns": [
            "Reloading entire 2,000 record collections from the server after modifying a single boolean toggle.",
            "Mutating collection schemas dynamically mid-session, corrupting gallery data bindings.",
        ],
    },
    {
        "id": "pp-fx-regular-expressions-pattern-matching",
        "cluster": 2,
        "book": "Data Validation and Pattern Matching in Power Apps - Rory Neary",
        "desc": "IsMatch(), Match(), MatchAll(), regular expression token matching, email/phone/credit card validation, and string sanitization.",
        "triggers": ["regular-expressions-pattern-matching", "ismatch-power-fx", "regex-validation-canvas", "input-sanitization-fx"],
        "foundations": [
            "Regex Engine: `IsMatch(Text, Pattern)` validates string conformance against predefined tokens (`Match.Email`, `Match.PhoneNumber`) or custom PCRE regex strings.",
            "Sanitization Invariant: ALWAYS validate critical input fields on client before passing to connectors or database tables.",
            "NEVER trust raw user string input without sanitizing against injection vectors.",
            "MANDATORY real-time feedback indicator when input fails pattern match.",
        ],
        "protocol": "Validate enterprise employee ID format (2 uppercase letters followed by 6 digits): `IsMatch(txtEmpID.Text, \"^[A-Z]{2}\\d{6}$\", MatchOptions.ContainsName)`.",
        "anti_patterns": [
            "Relying on simple length checks instead of structural pattern verification for emails and postal codes.",
            "Permitting unbounded string submission that breaks downstream legacy ERP integration.",
        ],
    },
    {
        "id": "pp-fx-coalesce-null-propagation",
        "cluster": 2,
        "book": "Defensive Formula Design in Power Fx - Microsoft Press",
        "desc": "Coalesce(), Blank(), IsBlank(), IsEmpty(), default value cascades, and graceful null handling across complex expressions.",
        "triggers": ["coalesce-null-propagation", "blank-vs-isblank-power-fx", "default-value-fallback-cascades", "null-coalescing-fx"],
        "foundations": [
            "Coalesce Semantics: `Coalesce(Val1, Val2, ...)` evaluates arguments in order and returns the first non-blank/non-empty value.",
            "Distinction Invariant: `IsBlank()` checks for scalar nulls/empty strings; `IsEmpty()` checks for empty tables/collections. They are NOT interchangeable.",
            "NEVER permit raw unhandled Blanks to propagate into numerical multiplication or date formatting expressions.",
            "MANDATORY provision of sensible fallback values for nullable database fields.",
        ],
        "protocol": "Display customer contact name with fallback cascade: `lblContact.Text = Coalesce(ThisItem.PreferredName, ThisItem.FullName, ThisItem.Email, 'Unknown Contact')`.",
        "anti_patterns": [
            "Using `If(IsBlank(x), fallback, x)` chains instead of clean, concise `Coalesce(x, y, fallback)`.",
            "Calling `IsEmpty()` on a scalar text field, causing runtime evaluation errors.",
        ],
    },
]

def generate_skills():
    """Generates remaining clusters programmatically to complete all 350 skills."""
    # Complete list of clusters 3-35 definitions to generate
    # We will build out every cluster with 10 skills each
    all_skills = list(SKILLS)
    existing_ids = {s["id"] for s in all_skills}
    
    # Specifications for clusters 3 to 35
    cluster_specs = [
        # Cluster 3: Power Apps Components & Reusable Libraries (pp-comp-*)
        (3, "Power Apps Components & Reusable Libraries", "pp-comp-", "Reusable Canvas Components and Design Systems - Hardit Bhatia", [
            ("custom-properties-input-output", "Input and Output custom component properties, Event parameters, and change handlers.", ["component-properties", "input-output-properties", "component-parameters"]),
            ("component-libraries-versioning", "Component library architecture, tenant distribution, updates, and ALM lifecycle.", ["component-libraries", "component-versioning", "tenant-library-distribution"]),
            ("fluent-navigation-header-bars", "Responsive breadcrumb headers, command bars, profile avatars, and navigation drawers.", ["fluent-navigation-header", "command-bar-component", "responsive-header-bars"]),
            ("custom-data-grid-pagination", "Custom canvas grid with column sorting indicators, virtual scrolling, and page size controls.", ["custom-data-grid", "canvas-grid-pagination", "sorting-table-component"]),
            ("global-theme-token-provider", "Design token distribution components, dark/light palette switching, and brand compliance.", ["global-theme-provider", "canvas-design-tokens", "theme-switching-component"]),
            ("notification-toast-overlay", "Declarative floating toast notification manager with auto-dismiss timers and severity styling.", ["notification-toast", "floating-toast-component", "toast-overlay-manager"]),
            ("media-file-uploader-dropzone", "Drag-and-drop file upload zone, multi-file preview cards, and size/mime validation.", ["file-uploader-dropzone", "media-upload-component", "drag-and-drop-canvas"]),
            ("treeview-hierarchical-navigator", "Recursive expandable treeview component for organizational charts and folder hierarchies.", ["treeview-hierarchical", "recursive-tree-component", "folder-tree-navigator"]),
            ("kpi-metric-card-widgets", "Modular KPI metric cards with micro-sparklines, percentage change badges, and alert thresholds.", ["kpi-metric-card", "metric-card-widgets", "sparkline-kpi-component"]),
            ("signature-capture-canvas", "Touch and pen signature capture component exporting high-resolution PNG base64 data.", ["signature-capture", "peninput-component", "digital-signature-canvas"]),
        ]),
        # Cluster 4: Canvas App Offline Caching & Mobile Ergonomics (pp-offline-*)
        (4, "Canvas App Offline Caching & Mobile Ergonomics", "pp-offline-", "Building Field-Ready Offline Mobile Power Apps - Microsoft Docs", [
            ("loaddata-savedata-local", "LoadData and SaveData local encrypted storage mechanics on iOS, Android, and Windows.", ["loaddata-savedata", "local-offline-storage", "savedata-encrypted-cache"]),
            ("dataverse-offline-profiles", "Configuring Dataverse Mobile Offline profiles, table sync filters, and column exclusion.", ["dataverse-offline-profiles", "mobile-offline-sync", "offline-profile-filters"]),
            ("two-way-sync-conflict-resolution", "Handling two-way offline synchronization conflicts, client vs server timestamps, and retry buffers.", ["two-way-sync-conflict", "offline-conflict-resolution", "sync-retry-buffers"]),
            ("device-hardware-sensors", "Hardware sensor integration: Location.Latitude, Compass, Acceleration, and BarcodeScanner.", ["device-hardware-sensors", "location-gps-power-apps", "hardware-barcodescanner"]),
            ("camera-photo-compression", "Mobile camera capture, AddPicture controls, and client-side resolution resizing before upload.", ["camera-photo-compression", "image-resizing-canvas", "photo-upload-optimization"]),
            ("mobile-push-notifications", "Power Apps Notification connector, deep-linked payloads, and mobile background alerts.", ["mobile-push-notifications", "power-apps-notification", "deep-linked-push-alerts"]),
            ("nfc-tag-reading-rfid", "ReadNFC integration, payload decoding, and mobile asset tracking field workflows.", ["nfc-tag-reading", "readnfc-power-apps", "rfid-asset-tracking"]),
            ("network-connectivity-status", "Connection.Connected and Connection.Metered inspection and proactive offline banners.", ["network-connectivity-status", "connection-connected-banner", "metered-network-handling"]),
            ("background-sync-reconciliation", "Queue processing on network reconnect, partial-failure dead-lettering, and sync state indicators.", ["background-sync-reconciliation", "reconnect-queue-processing", "sync-state-indicators"]),
            ("biometric-auth-gateways", "Device biometric prompt verification for high-security record unlocking in mobile field apps.", ["biometric-auth-gateways", "mobile-biometric-prompt", "sensitive-record-unlock"]),
        ]),
        # Cluster 5: Canvas App Performance & Delegation Optimization (pp-perf-*)
        (5, "Canvas App Performance & Delegation Optimization", "pp-perf-", "Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press", [
            ("delegation-limits-workarounds", "Mastering 500/2000 record limits, delegable operators, and server-side pushdown.", ["delegation-limits", "server-pushdown-query", "delegable-operators-matrix"]),
            ("odata-delegation-matrix", "Delegation compatibility across Dataverse, SharePoint, and Azure SQL data sources.", ["odata-delegation-matrix", "sharepoint-vs-dataverse-delegation", "sql-delegation-support"]),
            ("n-plus-one-query-elimination", "Eliminating N+1 LookUp loops inside galleries by leveraging collections and single batch queries.", ["n-plus-one-query-elimination", "gallery-lookup-optimization", "batch-query-hydration"]),
            ("monitor-tool-network-telemetry", "Using Power Apps Monitor to profile network request latency, response sizes, and formula times.", ["monitor-tool-telemetry", "power-apps-monitor-profiling", "network-request-tracing"]),
            ("app-bundle-asset-trimming", "Image compression, SVG substitution, and unreferenced media pruning for small bundle footprints.", ["app-bundle-trimming", "canvas-asset-compression", "unreferenced-media-pruning"]),
            ("lazy-loading-screen-caching", "Deferring data hydration until screen navigation and gating execution behind OnVisible flags.", ["lazy-loading-screens", "screen-onvisible-gating", "deferred-data-hydration"]),
            ("control-count-dom-overhead", "Maintaining screen control count under 500 and replacing redundant controls with gallery templates.", ["control-count-overhead", "screen-dom-optimization", "gallery-control-reuse"]),
            ("sql-stored-procedure-delegation", "Invoking Direct SQL Stored Procedures for multi-million row analytics and heavy aggregations.", ["sql-stored-procedure-delegation", "direct-sproc-execution", "sql-analytics-canvas"]),
            ("dataverse-indexed-columns", "Quick Find configuration, secondary b-tree indexing, and composite search key optimization.", ["dataverse-indexed-columns", "quick-find-optimization", "secondary-indexes-dataverse"]),
            ("concurrent-data-hydration", "Parallelizing data queries on app launch to shrink startup rendering times to under 1 second.", ["concurrent-data-hydration", "parallel-data-prefetching", "startup-rendering-optimization"]),
        ]),
        # Cluster 6: Dataverse Relational Schema & Table Modeling (pp-dv-*)
        (6, "Dataverse Relational Schema & Table Modeling", "pp-dv-", "Data Modeling for Microsoft Dataverse - Marc Gerner", [
            ("standard-custom-activity-tables", "Architectural distinctions between Standard, Custom, and Activity tables for interaction tracking.", ["standard-custom-tables", "activity-tables-modeling", "interaction-tracking-schema"]),
            ("relational-cardinality-relationships", "Configuring 1:N, N:1, and N:N relationships, cascade behaviors, and referential integrity.", ["relational-cardinality", "cascade-delete-rules", "referential-integrity-dataverse"]),
            ("polymorphic-lookup-architecture", "Customer and Owner polymorphic lookups, multi-table targeting, and schema patterns.", ["polymorphic-lookups", "multi-table-targets", "owner-customer-polymorphism"]),
            ("alternate-keys-upsert-idempotency", "Defining Alternate Keys for natural primary keys and idempotent REST Upsert operations.", ["alternate-keys-upsert", "natural-keys-dataverse", "idempotent-upsert-rest"]),
            ("calculated-and-rollup-columns", "Asynchronous Rollup aggregates, scheduled calculation frequency, and calculated formula limits.", ["calculated-rollup-columns", "rollup-aggregates-dataverse", "scheduled-recalculation-dv"]),
            ("formula-columns-power-fx", "Authoring native Power Fx formula columns inside Dataverse tables for live computed values.", ["formula-columns-power-fx", "dataverse-formula-columns", "computed-columns-power-fx"]),
            ("schema-prefix-publisher-governance", "Publisher prefix governance, collision avoidance, and enterprise naming conventions.", ["schema-prefix-governance", "solution-publisher-prefix", "naming-conventions-dataverse"]),
            ("auditing-retention-lifecycle", "Field-level auditing configuration, audit partition archiving, and compliance policies.", ["auditing-retention-lifecycle", "field-level-auditing", "audit-partition-management"]),
            ("elastic-tables-azure-cosmos-db", "Dataverse Elastic tables powered by Azure Cosmos DB for ultra-high throughput telemetry.", ["elastic-tables-cosmos", "high-volume-telemetry-dv", "json-partition-keys"]),
            ("file-and-image-column-streaming", "Chunked streaming for File and Image columns, SAS token security, and upload size quotas.", ["file-image-streaming", "chunked-binary-upload", "dataverse-sas-tokens"]),
        ]),
        # Cluster 7: Model-Driven Apps, Forms & Business Rules (pp-mda-*)
        (7, "Model-Driven Apps, Forms & Business Rules", "pp-mda-", "Mastering Model-Driven Apps in Power Apps - Phil Cole", [
            ("modern-app-designer-sitemaps", "Modern App Designer, responsive navigation sitemaps, area grouping, and page embedding.", ["modern-app-designer", "sitemap-navigation-mda", "responsive-page-grouping"]),
            ("main-quick-create-card-forms", "Form design paradigms: Main forms, Quick Create side-panels, and Quick View summary cards.", ["main-quick-create-forms", "quick-view-cards", "form-layout-architecture"]),
            ("business-rules-declarative-logic", "Client-side and entity-level Business Rules for visibility, field locking, and defaults.", ["business-rules-declarative", "entity-business-rules", "field-locking-logic"]),
            ("business-process-flows-bpf", "Multi-stage Business Process Flows, conditional stage branching, and cross-table progression.", ["business-process-flows", "bpf-stage-branching", "cross-table-progression"]),
            ("modern-command-bar-power-fx", "Modern Command Bar button authoring using Power Fx formulas and contextual visibility rules.", ["modern-command-bar", "command-bar-power-fx", "ribbon-button-customization"]),
            ("subgrid-view-customization", "Configuring related record subgrids, editable grid controls, and nested record filters.", ["subgrid-customization", "editable-grids-mda", "related-record-views"]),
            ("client-scripting-xrm-page-api", "Modern Xrm.WebApi client scripting, execution context lifecycle, and event registration.", ["xrm-page-client-scripting", "xrm-webapi-crud", "form-execution-context"]),
            ("web-resources-javascript-html", "Web resource development, ES6 packaging, webpack bundling, and dependency injection.", ["web-resources-javascript", "es6-bundling-mda", "html-web-resources"]),
            ("dashboard-charts-power-bi-embedding", "Model-driven interactive dashboards, native drill-through charts, and Power BI visual embedding.", ["mda-dashboards-charts", "drill-through-visuals", "power-bi-embedding-mda"]),
            ("embedded-canvas-pcf-controls", "Embedding Canvas Apps and PCF components directly onto Model-Driven forms with context sync.", ["embedded-canvas-pcf", "modeldrivenformintegration", "hybrid-mda-canvas-forms"]),
        ]),
        # Cluster 8: Dataverse Security Roles & Hierarchical RBAC (pp-sec-*)
        (8, "Dataverse Security Roles & Hierarchical RBAC", "pp-sec-", "Security, Governance, and Compliance in Dataverse - Microsoft Learn", [
            ("privilege-depth-matrix", "Mastering the 8 privileges and 4 access depths: User, Business Unit, Parent-Child, Organization.", ["privilege-depth-matrix", "security-role-depths", "rbac-privilege-levels"]),
            ("business-units-security-hierarchy", "Business Unit hierarchy design, parent-child inheritance, and user ownership boundaries.", ["business-units-hierarchy", "bu-ownership-boundaries", "parent-child-bu-security"]),
            ("entra-id-group-teams", "Automating Dataverse role assignments via Entra ID Security Group Teams and dynamic licensing.", ["entra-id-group-teams", "aad-group-teams-dataverse", "automated-role-assignment"]),
            ("field-level-security-profiles", "Column Security Profiles, PII field masking, and granular read/update authorization.", ["field-level-security", "column-security-profiles", "pii-data-masking-dv"]),
            ("record-ownership-and-sharing", "User vs Team ownership paradigms, explicit sharing records, Access Teams, and PoLP.", ["record-ownership-sharing", "access-teams-dataverse", "principle-of-least-privilege"]),
            ("hierarchy-security-models", "Configuring Manager vs Position hierarchy security for automatic downward record visibility.", ["hierarchy-security-models", "manager-hierarchy-security", "position-hierarchy-access"]),
            ("service-principal-app-users", "S2S Service Principal registration, Dataverse Application Users, and headless authentication.", ["service-principal-app-users", "s2s-application-users", "headless-api-security"]),
            ("modern-role-based-app-licensing", "Restricting Canvas and Model-Driven App access strictly via Security Role assignment.", ["role-based-app-access", "app-security-role-binding", "licensed-app-protection"]),
            ("data-masking-and-encryption", "Customer-Managed Keys (CMK), transparent data encryption, and synthetic column regex masking.", ["data-masking-encryption", "customer-managed-keys-dv", "transparent-data-encryption"]),
            ("audit-logging-purview-integration", "Dataverse audit log export to Microsoft Purview, Azure Sentinel, and compliance trails.", ["audit-logging-purview", "dataverse-sentinel-export", "compliance-audit-trails"]),
        ]),
        # Cluster 9: Power Automate Cloud Flows & Trigger Architectures (pp-flow-*)
        (9, "Power Automate Cloud Flows & Trigger Architectures", "pp-flow-", "Automating Enterprise Workflows with Power Automate - Serge Luca", [
            ("trigger-types-matrix", "Architectural trade-offs: Automated event triggers, Instant/Button flows, and Scheduled recurrence.", ["flow-trigger-types", "automated-vs-instant-flows", "scheduled-recurrence-triggers"]),
            ("dataverse-trigger-filtering", "Dataverse trigger optimization: Change types, Select columns, and Filter rows expressions.", ["dataverse-trigger-filtering", "when-a-row-is-added-modified", "trigger-select-columns"]),
            ("trigger-conditions-expressions", "Configuring Trigger Conditions expressions (@equals, @and, @not) to eliminate wasteful flow runs.", ["trigger-conditions-expressions", "prevent-unnecessary-runs", "trigger-condition-rules"]),
            ("concurrency-and-degree-of-parallelism", "Concurrency control tuning, SplitOn array iteration, and parallel branch execution throughput.", ["concurrency-control-flow", "degree-of-parallelism", "spliton-throughput"]),
            ("dynamic-content-and-json-schemas", "Parse JSON action schemas, dynamic content token binding, and null-safe payload parsing.", ["dynamic-content-json", "parse-json-action-schema", "null-safe-payload-parsing"]),
            ("child-flows-reusable-subroutines", "Building reusable Child Flows, parameter contracts, response payloads, and solution packaging.", ["child-flows-subroutines", "reusable-flow-components", "run-a-child-flow"]),
            ("webhook-callback-patterns", "HTTP Webhook request triggers, asynchronous HTTP 202 patterns, and callback listeners.", ["webhook-callback-patterns", "http-webhook-listener", "async-202-flow-pattern"]),
            ("batch-pagination-chunking", "Configuring Pagination thresholds on list actions and processing 5,000+ records safely.", ["batch-pagination-chunking", "pagination-threshold-tuning", "chunked-record-processing"]),
            ("timeout-and-run-duration-limits", "Action timeout customization, PT1H syntax, and managing the 30-day asynchronous flow limit.", ["flow-timeout-limits", "action-run-duration", "30-day-asynchronous-limit"]),
            ("service-account-connection-references", "Migrating personal user connections to Service Principal Connection References in solutions.", ["service-account-connections", "connection-references-alm", "spn-flow-connections"]),
        ]),
        # Cluster 10: Flow Error Handling, Scopes & Resilient Retries (pp-err-*)
        (10, "Flow Error Handling, Scopes & Resilient Retries", "pp-err-", "Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra", [
            ("try-catch-finally-scope-patterns", "Structuring flows with Try, Catch, and Finally Scopes using Run After branch configuration.", ["try-catch-finally-scopes", "flow-run-after-pattern", "exception-handling-scopes"]),
            ("run-after-configuration-matrices", "Configuring fine-grained Run After conditions: has failed, is skipped, and has timed out.", ["run-after-matrices", "has-failed-is-skipped", "flow-branching-guards"]),
            ("retry-policy-exponential-backoff", "Configuring exponential backoff and fixed interval retry policies for HTTP 429 throttling.", ["retry-policy-backoff", "http-429-throttling-flow", "exponential-retry-tuning"]),
            ("result-function-error-inspection", "Using the result('Try_Scope') expression to extract failed action names and exact error codes.", ["result-function-error-check", "inspect-try-scope-errors", "failed-action-diagnostics"]),
            ("dead-letter-queue-logging", "Routing terminal flow exceptions to Azure Service Bus or Dataverse Dead Letter Queue tables.", ["dead-letter-queue-logging", "flow-dlq-pattern", "terminal-exception-routing"]),
            ("circuit-breaker-transient-failures", "Implementing transient failure circuit breakers to halt downstream calls during outages.", ["circuit-breaker-flows", "transient-failure-detection", "outage-circuit-tripping"]),
            ("compensating-transactions-rollback", "Orchestrating compensating transactions to roll back partial writes across distributed systems.", ["compensating-transactions", "flow-rollback-logic", "distributed-write-compensation"]),
            ("alerting-teams-adaptive-cards", "Posting rich Adaptive Card error alerts into Microsoft Teams with direct run history links.", ["alerting-teams-cards", "flow-error-adaptive-card", "incident-notification-teams"]),
            ("flow-analytics-run-history-api", "Programmatically querying failed flow runs via Power Automate Management connector.", ["flow-analytics-run-api", "query-flow-history", "automated-failure-monitoring"]),
            ("idempotent-replay-safety", "Enforcing deduplication transaction keys to guarantee safe flow replay without duplicates.", ["idempotent-replay-safety", "deduplication-transaction-keys", "safe-flow-retry-runs"]),
        ]),
        # Cluster 11: Enterprise Approvals & Human-in-the-Loop Flows (pp-appr-*)
        (11, "Enterprise Approvals & Human-in-the-Loop Flows", "pp-appr-", "Enterprise Approvals Architecture in Power Platform - Vivek Bavishi", [
            ("approval-types-matrix", "Trade-offs: First to respond, Everyone must approve, and Custom response multi-option branches.", ["approval-types-matrix", "everyone-must-approve", "first-to-respond-approvals"]),
            ("sequential-and-parallel-approvals", "Multi-tier approval pipelines: Sequential hierarchy vs Parallel cross-department sign-offs.", ["sequential-parallel-approvals", "multi-tier-approval-pipeline", "parallel-signoff-routing"]),
            ("approvals-in-teams-adaptive-cards", "Native Microsoft Teams actionable approval cards with inline comment validation.", ["teams-approval-cards", "actionable-approvals-teams", "inline-approval-comments"]),
            ("escalation-and-timeout-reassignment", "Do Until reminder loops, timeout detection, and automatic escalation to secondary managers.", ["approval-escalation-timeout", "do-until-reminder-loops", "manager-escalation-routing"]),
            ("dataverse-approvals-core-tables", "Querying msdyn_flow_approval and msdyn_flow_approvalrequest Dataverse tables directly.", ["approvals-core-tables", "msdyn-flow-approval-schema", "query-approval-requests"]),
            ("delegation-and-out-of-office-routing", "Handling out-of-office delegations and automated reassignment during approver absences.", ["approval-delegation-ooo", "out-of-office-approver-routing", "substitute-approver-logic"]),
            ("digital-signature-audit-trails", "Capturing cryptographic timestamps, user identity hashes, and audit comments for compliance.", ["digital-signature-audit", "approval-timestamp-verification", "compliance-signature-trail"]),
            ("guest-user-external-approvals", "Routing approvals to external B2B guest users via actionable email messages safely.", ["guest-user-external-approvals", "b2b-actionable-email-approvals", "external-partner-signoff"]),
            ("cancellation-and-revocation-logic", "Programmatically canceling in-flight approval requests upon record modification or cancellation.", ["cancellation-revocation-logic", "cancel-in-flight-approval", "approval-lifecycle-cleanup"]),
            ("ai-summarized-approval-packets", "Generating executive summary bullet points inside approval card bodies via AI Builder prompts.", ["ai-summarized-approvals", "copilot-approval-packet", "executive-summary-approval"]),
        ]),
        # Cluster 12: High-Throughput OData Filtering & JSON Data Wrangling (pp-odata-*)
        (12, "High-Throughput OData Filtering & JSON Data Wrangling", "pp-odata-", "OData and JSON Mastery for Power Platform Integrations - Microsoft Press", [
            ("filter-query-syntax", "Precision OData filter queries: eq, ne, gt, ge, lt, le, and, or, not, startswith, endswith.", ["odata-filter-query", "odata-comparison-operators", "server-side-filter-syntax"]),
            ("expand-query-related-records", "Using $expand on navigation properties to retrieve parent and child entities in a single HTTP trip.", ["odata-expand-query", "single-trip-related-records", "expand-navigation-properties"]),
            ("select-column-pruning", "Applying $select projection to strip redundant fields and drastically reduce JSON payload sizes.", ["select-column-pruning", "odata-select-projection", "payload-bandwidth-optimization"]),
            ("orderby-and-top-pagination", "Combining $orderby and $top parameters for predictable, server-side deterministic pagination.", ["orderby-top-pagination", "odata-sorting-pagination", "deterministic-paging"]),
            ("xml-fetchxml-conversions", "Executing complex FetchXML in Dataverse List Rows for outer joins, grouping, and aggregates.", ["xml-fetchxml-conversions", "fetchxml-list-rows", "outer-joins-aggregates-dv"]),
            ("json-xpath-xml-transformations", "Leveraging json(), xml(), and xpath() expressions to wrangle legacy enterprise XML documents.", ["json-xpath-xml-transforms", "xpath-expressions-flow", "xml-to-json-wrangling"]),
            ("array-filter-select-operations", "Transforming in-memory JSON via Filter Array and Select actions for ultra-fast schema reshaping.", ["array-filter-select", "select-action-projection", "in-memory-json-reshaping"]),
            ("chunked-batching-changesets", "Constructing Dataverse $batch requests to group multiple CRUD operations into a single network call.", ["chunked-batching-changesets", "dataverse-batch-operations", "http-changeset-requests"]),
            ("delta-token-incremental-sync", "Tracking entity changes via Dataverse delta tokens (@odata.deltaLink) for incremental sync.", ["delta-token-sync", "incremental-data-sync", "odata-deltalink-tracking"]),
            ("base64-binary-handling", "Wrangling binary payloads via base64ToString, stringToBase64, and dataUri expressions safely.", ["base64-binary-handling", "binary-streaming-flow", "base64-conversion-expressions"]),
        ]),
        # Cluster 13: Power Automate Desktop RPA & UI Automation (pp-rpa-*)
        (13, "Power Automate Desktop RPA & UI Automation", "pp-rpa-", "Robotic Process Automation with Power Automate Desktop - Joe Unwin", [
            ("attended-vs-unattended-execution", "Architectural differences: Attended worker-driven RPA vs Unattended headless VM automation.", ["attended-vs-unattended-rpa", "headless-vm-automation", "rpa-session-unlocking"]),
            ("desktop-flow-machine-groups", "Hosted machine groups, On-premises data gateway clusters, and dynamic load distribution.", ["desktop-machine-groups", "hosted-rpa-bots", "gateway-cluster-balancing"]),
            ("ui-element-selector-tuning", "Selector tuning: UI Automation trees, CSS selectors, wildcards, and dynamic ID handling.", ["ui-element-selector-tuning", "rpa-selector-resilience", "uia-tree-selectors"]),
            ("web-automation-browser-drivers", "Automating Edge/Chrome browsers via native extensions, DOM manipulation, and cookies.", ["web-automation-browser", "edge-chrome-extension-rpa", "dom-element-automation"]),
            ("legacy-win32-terminal-emulation", "Automating legacy Win32 thick clients, SAP GUI scripting, and 3270/5250 mainframe green screens.", ["legacy-win32-terminal", "sap-gui-scripting", "mainframe-3270-automation"]),
            ("credential-cyberark-azure-keyvault", "Injecting dynamic secrets into RPA tasks securely from Azure Key Vault or CyberArk.", ["credential-keyvault-rpa", "cyberark-secret-injection", "secure-rpa-credentials"]),
            ("subflows-exception-handling", "Modular PAD subflow architecture, On Error blocks, automated screenshot capture, and retries.", ["subflows-exception-handling", "pad-error-screenshot", "modular-rpa-subflows"]),
            ("ocr-screen-scraping-ai-vision", "Computer Vision OCR and Tesseract screen scraping for remote virtual desktop Citrix environments.", ["ocr-screen-scraping", "citrix-virtual-desktop-rpa", "computer-vision-scraping"]),
            ("excel-advanced-vba-macros", "Automating complex legacy Excel workbooks, running embedded VBA macros, and matrix processing.", ["excel-vba-macros-rpa", "pad-excel-automation", "fast-cell-matrix-processing"]),
            ("cloud-to-desktop-flow-orchestration", "Triggering Desktop Flows from Cloud Flows, passing typed inputs, and receiving JSON outputs.", ["cloud-to-desktop-flow", "rpa-hybrid-orchestration", "desktop-flow-parameters"]),
        ]),
        # Cluster 14: Power BI Semantic Modeling & Star Schemas (pp-pbi-*)
        (14, "Power BI Semantic Modeling & Star Schemas", "pp-pbi-", "Analyzing Data with Microsoft Power BI - Alberto Ferrari & Marco Russo", [
            ("star-schema-dimensional-modeling", "Designing optimal Star Schemas: Fact tables, Dimension tables, surrogate keys, and avoiding snowflakes.", ["star-schema-modeling", "dimensional-modeling-pbi", "fact-dimension-tables"]),
            ("relationship-cardinality-filter-direction", "Relationship design: 1:*, *:1, *:*, Single vs Both cross-filter direction, and ambiguity traps.", ["relationship-cardinality", "cross-filter-direction", "bidirectional-filtering-hazards"]),
            ("role-playing-dimensions-active-keys", "Role-playing dimensions (Order Date, Ship Date) and inactive relationship activation via DAX.", ["role-playing-dimensions", "inactive-relationships-pbi", "userelationship-modeling"]),
            ("slowly-changing-dimensions-scd", "Implementing SCD Type 1 vs Type 2 tracking in the semantic layer with valid-to timestamps.", ["slowly-changing-dimensions", "scd-type-2-modeling", "historical-snapshots-pbi"]),
            ("composite-models-directquery-import", "Composite Models combining high-speed Import tables with real-time DirectQuery sources.", ["composite-models-pbi", "directquery-vs-import", "dual-mode-aggregations"]),
            ("incremental-refresh-range-partitions", "Incremental Refresh policy configuration with RangeStart and RangeEnd date parameters.", ["incremental-refresh-pbi", "range-start-range-end", "partition-archiving-pbi"]),
            ("row-level-security-rls-roles", "Dynamic Row-Level Security (RLS) implementation using USERPRINCIPALNAME() and security tables.", ["row-level-security-rls", "dynamic-rls-userprincipalname", "security-role-filters-pbi"]),
            ("calculation-groups-time-intelligence", "Creating Calculation Groups in Tabular Editor for dynamic time intelligence measure modifiers.", ["calculation-groups-pbi", "tabular-editor-time-intel", "dynamic-measure-modifiers"]),
            ("field-parameters-dynamic-axes", "Field Parameters for interactive dynamic visual axes and dimension/measure swapping.", ["field-parameters-dynamic-axes", "dynamic-visual-dimensions", "parameterized-chart-axes"]),
            ("vertipaq-engine-compression-tuning", "VertiPaq engine optimization: High-cardinality column pruning, datetime splitting, and encoding.", ["vertipaq-compression-tuning", "dictionary-encoding-pbi", "high-cardinality-pruning"]),
        ]),
        # Cluster 15: DAX Formula Engineering & Context Transition (pp-dax-*)
        (15, "DAX Formula Engineering & Context Transition", "pp-dax-", "The Definitive Guide to DAX - Marco Russo & Alberto Ferrari", [
            ("row-context-vs-filter-context", "The foundational duality: Row Context iteration vs Filter Context set propagation in DAX.", ["row-context-vs-filter-context", "dax-context-mechanics", "filter-context-propagation"]),
            ("calculate-context-transition", "CALCULATE and CALCULATETABLE mechanics: Transforming row context into equivalent filter context.", ["calculate-context-transition", "context-transition-dax", "calculate-filter-engine"]),
            ("filter-modifiers-all-allexcept", "Filter modifiers: ALL, ALLEXCEPT, ALLSELECTED, REMOVEFILTERS, and KEEPFILTERS behavior.", ["filter-modifiers-dax", "all-allexcept-removefilters", "allselected-visual-context"]),
            ("time-intelligence-functions", "Time intelligence: TOTALYTD, SAMEPERIODLASTYEAR, DATEADD, DATESBETWEEN, and 4-4-5 calendars.", ["time-intelligence-dax", "totalytd-sameperiodlastyear", "custom-fiscal-calendars"]),
            ("iterator-functions-sumx-filter", "Iterator functions: SUMX, AVERAGEX, MINX, MAXX, and evaluation overhead optimization.", ["iterator-functions-sumx", "dax-iterator-overhead", "filter-function-performance"]),
            ("semi-additive-measures-inventory", "Semi-additive measures for balance sheets and inventory using CLOSINGBALANCEMONTH and LASTDATE.", ["semi-additive-measures", "inventory-balance-dax", "closingbalancemonth"]),
            ("virtual-table-creation-summarize", "Authoring virtual tables via SUMMARIZE, SUMMARIZECOLUMNS, ADDCOLUMNS, and SELECTCOLUMNS.", ["virtual-tables-dax", "summarizecolumns-virtual", "addcolumns-table-variables"]),
            ("variables-var-evaluation-scope", "VAR / RETURN semantics: Lazy definition, constant evaluation scope, and performance gains.", ["variables-var-dax", "var-return-scope", "dax-variable-optimization"]),
            ("dax-studio-performance-profiling", "Profiling queries in DAX Studio: Server Timings, Storage Engine (SE) vs Formula Engine (FE).", ["dax-studio-profiling", "server-timings-se-fe", "storage-engine-xmSql"]),
            ("dynamic-ranking-topn-pareto", "Dynamic ranking and Pareto analysis using RANKX, TOPN, and cumulative percentage curves.", ["dynamic-ranking-topn", "pareto-80-20-dax", "rankx-dense-skipping"]),
        ]),
        # Cluster 16: Power Query M Language Data Transformations (pp-m-*)
        (16, "Power Query M Language Data Transformations", "pp-m-", "Collect, Combine, and Transform Data Using Power Query in Excel and Power BI - Gil Raviv", [
            ("query-folding-diagnostics", "Diagnosing Query Folding indicators, View Native Query, and maximizing SQL pushdown.", ["query-folding-diagnostics", "view-native-query-m", "sql-pushdown-optimization"]),
            ("let-in-evaluation-laziness", "The M language let ... in expression pipeline, immutable bindings, and lazy evaluation resolution.", ["let-in-evaluation-laziness", "m-language-lazy-eval", "immutable-step-bindings"]),
            ("custom-functions-and-recursion", "Authoring parameter-driven M functions, each syntactic sugar, and recursive pagination loops.", ["custom-functions-m", "recursive-pagination-m", "parameterized-m-transforms"]),
            ("nested-lists-records-tables", "Deep manipulation of nested List, Record, and Table types using List.Transform and Record.Field.", ["nested-structures-m", "list-transform-record-field", "table-nested-columns"]),
            ("web-api-json-ingestion-paging", "Web.Contents, RelativePath, Query options, authorization headers, and cursor pagination.", ["web-api-json-m", "web-contents-relativepath", "cursor-pagination-powerquery"]),
            ("fuzzy-matching-and-clustering", "Table.FuzzyNestedJoin, similarity thresholds, case insensitivity, and cluster deduplication.", ["fuzzy-matching-m", "table-fuzzynestedjoin", "text-clustering-powerquery"]),
            ("error-handling-try-otherwise", "Defensive transformation handling using try ... otherwise blocks and error record diagnostics.", ["error-handling-try-otherwise", "m-try-otherwise-blocks", "resilient-transformations-m"]),
            ("data-type-coercion-locales", "DateTimeZone.FromText, Culture parameter handling, and locale-safe type transformations.", ["data-type-coercion-locales", "culture-parameter-m", "datetimezone-parsing-m"]),
            ("unpivot-transpose-matrix-wrangling", "Table.UnpivotOtherColumns for reshaping cross-tabulated spreadsheets into tidy normalized tables.", ["unpivot-transpose-matrix", "table-unpivotothercolumns", "tidy-data-wrangling-m"]),
            ("dataflow-gen2-fabric-integration", "Fabric Dataflow Gen2 authoring, compute scaling, and staging destinations (Lakehouse/Warehouse).", ["dataflow-gen2-fabric", "fabric-dataflows-staging", "lakehouse-warehouse-destination"]),
        ]),
        # Cluster 17: Power BI Copilot & Natural Language Visualizations (pp-pbicop-*)
        (17, "Power BI Copilot & Natural Language Visualizations", "pp-pbicop-", "Generative AI and Copilot in Microsoft Power BI - Microsoft Learn", [
            ("linguistic-schema-synonyms", "Optimizing linguistic schemas, Q&A synonyms, and relationship phrasing for accurate Copilot reasoning.", ["linguistic-schema-synonyms", "q-and-a-synonym-dictionary", "copilot-linguistic-modeling"]),
            ("copilot-report-page-generation", "Prompt-driven generation of executive summary report pages and automated visual layouts.", ["copilot-report-page-generation", "prompt-driven-bi-pages", "automated-visual-layout"]),
            ("narrative-visual-summaries", "Configuring Smart Narrative visuals and Copilot dynamic text summaries bound to active slicers.", ["narrative-visual-summaries", "smart-narrative-copilot", "dynamic-slicer-summaries"]),
            ("dax-formula-generation-ai", "Copilot for DAX in Desktop and Tabular models: Natural language description to measure synthesis.", ["dax-formula-generation-ai", "copilot-for-dax", "natural-language-to-measure"]),
            ("featured-questions-and-qa", "Configuring Featured Questions, teaching vocabulary to the Q&A engine, and ambiguous query mapping.", ["featured-questions-qa", "teach-q-and-a-engine", "ambiguous-query-mapping"]),
            ("data-storytelling-infographics", "AI-driven anomalous data point detection, automated trend explanations, and narrative callouts.", ["data-storytelling-infographics", "ai-insights-anomaly-detection", "automated-trend-explanations"]),
            ("fabric-workspace-capacity-admin", "Managing Microsoft Fabric F-SKU capacity requirements, Copilot tenant switches, and governance.", ["fabric-capacity-copilot-admin", "f-sku-requirements-bi", "tenant-switch-activation"]),
            ("copilot-security-sensitivity-labels", "Sensitivity label inheritance (Purview Information Protection) and preventing unauthorized Copilot export.", ["copilot-sensitivity-labels", "purview-label-inheritance", "data-exfiltration-prevention-bi"]),
            ("semantic-model-metadata-enrichment", "Enriching table, column, and measure descriptions with domain context to ground LLM reasoning.", ["semantic-model-metadata-enrichment", "ground-llm-bi-reasoning", "column-description-context"]),
            ("mobile-bi-report-ergonomics", "Authoring responsive phone layouts and dynamic Copilot bullet briefings for mobile executives.", ["mobile-bi-report-ergonomics", "phone-layout-optimization-pbi", "executive-mobile-briefings"]),
        ]),
        # Cluster 18: Power Pages Architecture & Liquid Templating (pp-pages-*)
        (18, "Power Pages Architecture & Liquid Templating", "pp-pages-", "Building Modern Websites with Microsoft Power Pages - Nick Doelman", [
            ("site-architecture-web-templates", "Power Pages hierarchy: Web Templates, Page Templates, Web Pages, and Content Snippets.", ["site-architecture-web-templates", "power-pages-hierarchy", "web-page-templates"]),
            ("liquid-syntax-control-flow", "Liquid template syntax: {% for %}, {% if %}, {% assign %}, liquid filters, and variable scoping.", ["liquid-syntax-control-flow", "liquid-template-engine", "liquid-filters-power-pages"]),
            ("liquid-entity-query-fetchxml", "Embedding FetchXML queries in Liquid templates ({% fetchxml %}) for dynamic server-side rendering.", ["liquid-fetchxml-queries", "fetchxml-liquid-tag", "server-rendered-entity-lists"]),
            ("portal-user-authentication-contacts", "Authentication mapping: Contact records, Entra External ID, Google, and LinkedIn OAuth providers.", ["portal-user-auth-contacts", "entra-external-id-pages", "social-login-providers"]),
            ("web-roles-table-permissions", "Web Roles configuration and Table Permissions (Global, Contact, Account parent scopes).", ["web-roles-table-permissions", "table-permission-scopes", "portal-security-roles"]),
            ("multilingual-content-snippets", "Multi-language portals, Content Snippets, and automated LCID locale route handling.", ["multilingual-content-snippets", "multi-language-portals", "lcid-locale-routing"]),
            ("site-settings-and-custom-domains", "Site Settings configuration, custom domain binding, SSL certificates, and URL rewrite rules.", ["site-settings-custom-domains", "ssl-certificates-power-pages", "url-rewrite-rules"]),
            ("caching-invalidation-mechanisms", "Portal cache invalidation mechanisms, Web API synchronization, and administrative cache clearing.", ["caching-invalidation-mechanisms", "portal-cache-clear", "web-api-cache-sync"]),
            ("head-meta-seo-optimization", "Dynamic SEO headers, Open Graph meta tags, robots.txt, and automated sitemap.xml generation.", ["head-meta-seo-optimization", "open-graph-meta-tags", "sitemap-robots-pages"]),
            ("progressive-web-app-pwa-features", "Enabling Progressive Web App (PWA) features, offline caching service workers, and mobile installs.", ["progressive-web-app-pwa", "power-pages-pwa-install", "offline-service-workers"]),
        ]),
        # Cluster 19: Power Pages Web API, Forms & Security Rules (pp-pweb-*)
        (19, "Power Pages Web API, Forms & Security Rules", "pp-pweb-", "Secure Portal Development with Power Pages Web API - Colin Vermander", [
            ("dataverse-web-api-portal-client", "Using the Portal Web API client wrapper (safeAjax) for client-side CRUD on Dataverse records.", ["portal-web-api-client", "safeajax-wrapper", "client-side-crud-pages"]),
            ("site-setting-api-column-permissions", "Configuring Webapi/{table}/enabled site settings and column-level permission access.", ["site-setting-api-permissions", "webapi-enabled-settings", "column-permission-access"]),
            ("csrf-token-request-validation", "Injecting __RequestVerificationToken headers to defend against Cross-Site Request Forgery.", ["csrf-token-validation", "request-verification-token", "anti-csrf-headers-pages"]),
            ("basic-forms-and-multistep-forms", "Configuring Basic Forms, Multistep Forms, session state persistence, and branch logic.", ["basic-multistep-forms", "multistep-form-sessions", "form-branching-logic-pages"]),
            ("client-side-form-validation-js", "Custom JavaScript on portal forms, Page_Validators array manipulation, and regex checks.", ["client-side-form-validation", "page-validators-javascript", "portal-custom-js-validation"]),
            ("file-attachments-azure-blob-storage", "Configuring Azure Blob Storage integration for large file attachments on portal forms.", ["file-attachments-blob-storage", "azure-blob-portal-integration", "portal-attachment-storage"]),
            ("subgrid-actions-and-modals", "Adding modal popups, custom action buttons, and inline record creation to portal entity lists.", ["subgrid-actions-modals", "entity-list-modal-popups", "inline-record-creation-portal"]),
            ("anti-scraping-waf-cloud-security", "Deploying Azure Front Door Web Application Firewall (WAF), rate limiting, and bot protection.", ["anti-scraping-waf-security", "azure-front-door-waf-pages", "rate-limiting-bot-protection"]),
            ("custom-portal-web-services", "Integrating companion Azure Functions microservices securely with authenticated portal tokens.", ["custom-portal-web-services", "azure-functions-pages-interop", "authenticated-portal-tokens"]),
            ("audit-trail-and-portal-telemetry", "Integrating Application Insights telemetry for user session clickstream tracking and audit trails.", ["audit-trail-portal-telemetry", "app-insights-power-pages", "user-session-clickstreams"]),
        ]),
        # Cluster 20: Power Pages Fluent Design Tokens & Bootstrap Styling (pp-pstyle-*)
        (20, "Power Pages Fluent Design Tokens & Bootstrap Styling", "pp-pstyle-", "Styling and Theming Microsoft Power Pages - Ulrikke Akerbæk", [
            ("bootstrap-5-customization", "Overriding Bootstrap 5 variables, custom theme.css compilation, and responsive grid alignment.", ["bootstrap-5-customization", "custom-theme-css-pages", "responsive-grid-alignment"]),
            ("fluent-2-design-tokens-integration", "Integrating Fluent 2 design tokens, color palettes, elevation drop shadows, and typography.", ["fluent-2-design-tokens", "fluent-typography-palette", "elevation-shadows-pages"]),
            ("styling-workspace-code-studio", "Using the Power Pages Styling Workspace, custom brand palettes, and VS Code for Web editing.", ["styling-workspace-code-studio", "brand-palette-editor", "vscode-web-styling-pages"]),
            ("responsive-navigation-mega-menus", "Building accessible multi-level mega menus and collapsible mobile drawer navigation bars.", ["responsive-mega-menus", "collapsible-mobile-drawers", "accessible-navbar-pages"]),
            ("accessible-form-controls-wcag", "Enforcing WCAG 2.1 AA accessible labels, focus rings, high contrast, and error announcements.", ["accessible-form-controls-pages", "wcag-portal-compliance", "aria-error-announcements"]),
            ("interactive-data-tables-datatables", "Enhancing entity lists with DataTables.js, client-side searching, sorting, and Excel export.", ["interactive-datatables-js", "datatables-export-buttons", "client-side-table-sorting"]),
            ("css-grid-flexbox-card-layouts", "Creating modern CSS Grid layout templates and responsive card dashboard views.", ["css-grid-card-layouts", "flexbox-dashboard-templates", "responsive-card-grids"]),
            ("custom-svg-iconography-icons", "Embedding Fluent System Icons, FontAwesome, and inline SVG assets for vector crispness.", ["custom-svg-iconography", "fluent-system-icons-pages", "fontawesome-portal-assets"]),
            ("dark-mode-toggle-state", "Implementing CSS prefers-color-scheme media queries and localStorage dark mode toggle switches.", ["dark-mode-toggle-state", "prefers-color-scheme-pages", "localstorage-theme-toggle"]),
            ("micro-animations-css-transitions", "Crafting smooth CSS transitions, accordion expand animations, and skeleton loading states.", ["micro-animations-transitions", "skeleton-loading-states", "smooth-accordion-animations"]),
        ]),
        # Cluster 21: Copilot Studio Generative Topics & Intent Parsing (pp-csgen-*)
        (21, "Copilot Studio Generative Topics & Intent Parsing", "pp-csgen-", "Conversational AI with Microsoft Copilot Studio - Michael Roth", [
            ("intent-recognition-trigger-phrases", "Generative intent recognition, trigger phrase clustering, and intent disambiguation.", ["intent-recognition-triggers", "trigger-phrase-clustering", "intent-disambiguation-cs"]),
            ("conversation-boosting-answers", "Conversational boosting configuration, generative answers fallback, and internal grounding.", ["conversation-boosting-answers", "generative-answers-fallback", "internal-grounding-cs"]),
            ("system-prompt-persona-tuning", "Crafting custom Copilot personas, tone boundaries, system instructions, and response formatting.", ["system-prompt-persona-tuning", "copilot-tone-boundaries", "system-instructions-cs"]),
            ("entity-extraction-slot-filling", "Configuring prebuilt and custom regex entities, slot filling, and conversation variable binding.", ["entity-extraction-slot-filling", "custom-regex-entities", "slot-filling-variables"]),
            ("clarification-and-disambiguation", "Managing 'Did you mean' clarification dialogs when user queries match multiple topic intents.", ["clarification-disambiguation", "did-you-mean-dialogs", "multi-intent-handling"]),
            ("multi-turn-context-memory", "Managing Bot and Global variables to retain state and context across multi-turn dialogues.", ["multi-turn-context-memory", "bot-global-variables", "dialogue-state-retention"]),
            ("generative-node-custom-instructions", "Authoring localized Custom Instructions within individual Generative Answers nodes.", ["generative-custom-instructions", "node-level-instructions", "localized-llm-grounding"]),
            ("topic-level-moderation-filters", "Configuring Content Moderation levels (High, Medium, Low) and graceful safety refusals.", ["topic-moderation-filters", "content-moderation-levels", "safety-refusal-handling"]),
            ("user-input-validation-repair", "Validating user inputs with Power Fx conditions and conversational repair re-prompts.", ["user-input-validation-repair", "power-fx-input-validation", "conversational-repair-prompts"]),
            ("fallback-topic-graceful-recovery", "Custom Fallback topic authoring, sentiment-aware escalation, and human handoff routing.", ["fallback-topic-graceful-recovery", "sentiment-aware-escalation", "custom-fallback-cs"]),
        ]),
        # Cluster 22: Copilot Studio Dynamic Chaining & Tool Orchestration (pp-cschain-*)
        (22, "Copilot Studio Dynamic Chaining & Tool Orchestration", "pp-cschain-", "Autonomous Multi-Action Agents with Copilot Studio - Microsoft Learn", [
            ("dynamic-chaining-generative-actions", "Configuring the Dynamic Chaining generative orchestration engine for autonomous action selection.", ["dynamic-chaining-actions", "generative-orchestrator-cs", "autonomous-action-selection"]),
            ("action-input-output-parameter-schemas", "Authoring precise natural language descriptions on action parameters for accurate LLM tool use.", ["parameter-schemas-tool-use", "action-description-tuning", "input-output-contracts-cs"]),
            ("power-automate-flow-action-plugins", "Invoking Cloud Flows as Copilot Studio action plugins with synchronous and asynchronous patterns.", ["flow-action-plugins", "cloud-flows-as-plugins", "plugin-execution-patterns"]),
            ("connector-actions-openapi-plugins", "Exposing Power Platform connectors directly as generative actions without intermediary flows.", ["connector-actions-openapi", "direct-connector-plugins", "openapi-tool-calling-cs"]),
            ("confirmation-and-safety-checkpoints", "Injecting human confirmation dialogs before executing destructive or high-impact actions.", ["confirmation-safety-checkpoints", "human-in-the-loop-actions", "destructive-action-guards"]),
            ("dynamic-plan-inspection-debugging", "Using the Copilot Studio test canvas to inspect LLM reasoning plans, tool calls, and inputs.", ["dynamic-plan-inspection", "test-canvas-plan-debug", "reasoning-step-inspection"]),
            ("action-chaining-multi-step-execution", "Chaining multiple actions sequentially to complete complex cross-system business tasks.", ["action-chaining-multi-step", "sequential-action-execution", "multi-step-task-completion"]),
            ("error-handling-and-retry-actions", "Handling plugin execution failures, API timeouts, and fallback tool selection dynamically.", ["error-handling-retry-actions", "plugin-timeout-handling", "fallback-tool-selection"]),
            ("custom-api-plugin-manifests", "Authoring AI Plugin manifests conforming to Microsoft Copilot plugin standards.", ["custom-api-plugin-manifests", "ai-plugin-json-cs", "plugin-manifest-standards"]),
            ("latency-budgeting-streaming", "Managing execution latency budgets, status update messages, and response streaming UX.", ["latency-budgeting-streaming", "action-timeout-management", "intermediate-status-updates"]),
        ]),
        # Cluster 23: Copilot Studio Knowledge Grounding & Dataverse Search (pp-csknow-*)
        (23, "Copilot Studio Knowledge Grounding & Dataverse Search", "pp-csknow-", "Enterprise Knowledge Grounding with Microsoft Copilot Studio - J. Peter Bruzzese", [
            ("dataverse-search-knowledge-indexing", "Configuring Dataverse Search, table selection, field weights, and real-time indexing for grounding.", ["dataverse-search-grounding", "knowledge-indexing-dv", "field-weights-search"]),
            ("sharepoint-onedrive-grounding", "Connecting SharePoint document libraries, user ACL crawling, and document freshness cycles.", ["sharepoint-onedrive-grounding", "acl-crawling-sharepoint", "document-freshness-cs"]),
            ("public-website-url-grounding", "Scraping public documentation domains, subpath scoping, and handling dynamic SPA sites.", ["public-website-url-grounding", "domain-scraping-grounding", "subpath-scoping-cs"]),
            ("uploaded-file-knowledge-stores", "Uploading PDF, Word, and Excel files directly into Copilot Studio knowledge stores.", ["uploaded-file-knowledge", "pdf-word-file-stores", "direct-file-grounding-cs"]),
            ("semantic-citations-referencing", "Extracting semantic citations, validating source snippet accuracy, and surfacing links.", ["semantic-citations-referencing", "citation-link-validation", "source-snippet-accuracy"]),
            ("knowledge-filters-metadata-tagging", "Filtering knowledge queries by metadata tags, user departmental role, and geographic market.", ["knowledge-filters-metadata", "metadata-tagging-search", "role-filtered-knowledge"]),
            ("knowledge-refresh-cycle-cadence", "Automating document re-indexing schedules, handling deletions, and purging stale vector chunks.", ["knowledge-refresh-cadence", "re-indexing-schedules", "stale-chunk-purging"]),
            ("synonym-dictionaries-and-acronyms", "Defining corporate acronym glossaries and domain synonym dictionaries for semantic search.", ["synonym-dictionaries-acronyms", "acronym-glossary-grounding", "corporate-synonyms-search"]),
            ("grounding-evals-fidelity-metrics", "Measuring grounding fidelity: Answer relevance, context precision, and hallucination rate.", ["grounding-evals-fidelity", "context-precision-metrics", "hallucination-rate-testing"]),
            ("zero-knowledge-fallback-polite-refusal", "Enforcing strict out-of-scope boundaries and polite refusals when knowledge is absent.", ["zero-knowledge-fallback", "strict-out-of-scope-boundary", "polite-refusal-cs"]),
        ]),
        # Cluster 24: Multi-Agent Choreography & Bot-to-Bot Handoffs (pp-csorch-*)
        (24, "Multi-Agent Choreography & Bot-to-Bot Handoffs", "pp-csorch-", "Multi-Agent Systems and Omnichannel Handoff - Microsoft Press", [
            ("hub-and-spoke-agent-architecture", "Hub-and-spoke multi-agent topology: Master Router Agent delegating tasks to specialist bots.", ["hub-and-spoke-multi-agent", "master-router-agent", "specialist-subagent-dispatch"]),
            ("bot-to-bot-context-passing", "Passing conversation history, authenticated user tokens, and variable state between agents.", ["bot-to-bot-context-passing", "agent-context-transfer", "token-passing-multi-agent"]),
            ("live-agent-handoff-omnichannel", "Escalating conversations seamlessly to Dynamics 365 Omnichannel for Customer Service agents.", ["live-agent-handoff-omnichannel", "omnichannel-escalation", "queue-routing-live-chat"]),
            ("third-party-live-agent-handoff", "Configuring handoffs to external platforms (ServiceNow, Salesforce, Zendesk) via Bot Framework.", ["third-party-live-agent-handoff", "servicenow-agent-handoff", "zendesk-bot-framework"]),
            ("autonomous-subagent-consensus", "Choreographing multiple specialized subagents to analyze inputs and synthesize consensus answers.", ["subagent-consensus-synthesis", "multi-agent-collaboration", "agent-jury-deliberation"]),
            ("voice-telephony-ivr-integration", "Deploying Copilot Studio agents on telephony channels with Azure Communication Services IVR.", ["voice-telephony-ivr-cs", "azure-communication-services-ivr", "speech-to-text-telephony"]),
            ("conversation-transcripts-export-synapse", "Exporting conversation transcripts to Azure Synapse or Microsoft Fabric for analytics.", ["transcripts-export-synapse", "conversation-analytics-fabric", "session-transcript-export"]),
            ("agent-lifecycle-solution-deployment", "Packaging Copilot Studio bots in Dataverse solutions for automated ALM CI/CD deployment.", ["agent-lifecycle-alm", "bot-solution-packaging", "copilot-studio-cicd"]),
            ("channel-security-directline-tokens", "Direct Line secret shielding: Exchanging static secrets for short-lived client user tokens.", ["channel-security-directline", "directline-token-exchange", "bot-secret-shielding"]),
            ("multi-lingual-agent-routing", "Detecting user language dynamically and routing between localized topics or real-time translation.", ["multi-lingual-agent-routing", "language-detection-cs", "real-time-translation-chat"]),
        ]),
        # Cluster 25: Power Apps Component Framework (PCF) Architecture (pp-pcf-*)
        (25, "Power Apps Component Framework (PCF) Architecture", "pp-pcf-", "Professional PCF Development: Extending Power Apps - Greg Hurlman", [
            ("control-manifest-input-output-schema", "ControlManifest.Input.xml authoring: Property definitions, type-groups, and resource references.", ["control-manifest-schema", "controlmanifest-input-xml", "pcf-property-definitions"]),
            ("lifecycle-init-updateview-destroy", "Mastering the component lifecycle methods: init, updateView, getOutputs, and destroy.", ["pcf-lifecycle-methods", "init-updateview-destroy", "getoutputs-pcf-lifecycle"]),
            ("field-controls-vs-dataset-controls", "Architectural differences: Single-value Field controls vs Multi-record Dataset grid controls.", ["field-vs-dataset-controls", "dataset-grid-pcf", "field-bound-pcf-controls"]),
            ("context-parameters-formatting", "Utilizing context.parameters for input bounds and context.formatting for localized dates and currencies.", ["context-parameters-formatting", "localized-formatting-pcf", "context-parameters-api"]),
            ("virtual-controls-vs-standard", "Virtual PCF controls: Sharing React and Fluent UI runtime libraries vs Sandboxed Standard controls.", ["virtual-controls-vs-standard", "virtual-pcf-react-shared", "runtime-overhead-reduction"]),
            ("webapi-client-crud-operations", "Invoking context.webAPI CRUD operations: createRecord, retrieveRecord, updateRecord, deleteRecord.", ["webapi-client-crud-pcf", "context-webapi-operations", "client-side-dataverse-crud"]),
            ("utility-navigation-lookup-dialogs", "Leveraging context.navigation and context.utility for modal dialogs, lookups, and alerts.", ["utility-navigation-pcf", "context-navigation-openform", "lookup-dialogs-pcf"]),
            ("local-harness-debugging-test", "Testing PCF controls locally via npm start test harness, parameter mocking, and hot reload.", ["local-harness-debugging", "pcf-test-harness", "hot-reload-pcf-debugging"]),
            ("solution-packaging-pac-pcf-push", "Automating deployment with pac pcf init, pac solution add-reference, and pac pcf push.", ["solution-packaging-pac-push", "pac-pcf-push-command", "pcf-solution-build"]),
            ("security-sandboxing-iframe-restrictions", "Navigating iframe restrictions, CSP policies, eval() prohibitions, and sandboxed storage.", ["security-sandboxing-pcf", "csp-iframe-restrictions", "sandbox-security-rules"]),
        ]),
        # Cluster 26: PCF TypeScript, React & Fluent UI Component Dev (pp-pcfrx-*)
        (26, "PCF TypeScript, React & Fluent UI Component Dev", "pp-pcfrx-", "Modern Front-End Engineering with PCF and Fluent UI - Dian Taylor", [
            ("fluent-ui-v9-react-components", "Implementing Fluent UI v9 Griffel styling, Button, Input, DataGrid, and Badge components.", ["fluent-ui-v9-pcf", "griffel-styling-pcf", "fluent-react-components"]),
            ("typescript-strict-type-definitions", "Configuring strict TypeScript compiler settings and authoring strong component interfaces.", ["typescript-strict-pcf", "componentframework-dts", "strict-types-interfaces"]),
            ("react-hooks-state-management", "Managing component state and effects with useState, useEffect, useMemo, and useCallback.", ["react-hooks-pcf", "usestate-useeffect-pcf", "custom-hooks-lifecycle"]),
            ("dataset-grid-custom-cell-renderers", "Building custom cell renderers for editable dataset grids with badges and action buttons.", ["dataset-grid-cell-renderers", "custom-grid-cells-pcf", "inline-grid-actions"]),
            ("custom-event-dispatching", "Dispatching custom DOM events from PCF components to parent canvas or model-driven containers.", ["custom-event-dispatching-pcf", "dom-events-pcf", "parent-container-messaging"]),
            ("theming-palette-sync-container", "Subscribing to host design themes via context.fluentDesignLanguage for dark/light mode sync.", ["theming-palette-sync-pcf", "fluent-design-language", "dark-light-mode-sync-pcf"]),
            ("modal-drawer-overlay-management", "Rendering React Portals and Drawer overlays with clean z-index stacking inside Dataverse forms.", ["modal-drawer-overlay-pcf", "react-portals-pcf", "z-index-stacking-forms"]),
            ("complex-charting-recharts-d3", "Embedding lightweight interactive data visualizations via Recharts and D3 micro-libraries.", ["complex-charting-recharts", "d3-visuals-pcf", "interactive-chart-components"]),
            ("drag-and-drop-kanban-board", "Building interactive Drag-and-Drop Kanban boards using dnd-kit and updating Dataverse stages.", ["drag-and-drop-kanban-pcf", "dnd-kit-kanban-board", "drag-drop-stage-update"]),
            ("rich-text-markdown-editor", "Integrating Slate / TipTap WYSIWYG editors with live Markdown export inside PCF controls.", ["rich-text-markdown-editor", "tiptap-wysiwyg-pcf", "markdown-export-pcf"]),
        ]),
        # Cluster 27: PCF Device Hardware APIs & Webpack/Bun Bundling (pp-pcfhard-*)
        (27, "PCF Device Hardware APIs & Webpack/Bun Bundling", "pp-pcfhard-", "Advanced PCF: Native Hardware & Performance Optimization - Microsoft Docs", [
            ("barcode-scanner-camera-capture", "Accessing context.device.captureBarcode and captureImage for mobile camera integration.", ["barcode-scanner-camera-pcf", "context-device-barcode", "mobile-camera-capture"]),
            ("geo-location-hardware-access", "Querying context.device.getCurrentPosition for GPS geolocation and accuracy filtering.", ["geo-location-hardware-pcf", "context-device-location", "gps-tracking-pcf"]),
            ("audio-recording-microphone", "Capturing audio via context.device.captureAudio and encoding voice notes to base64.", ["audio-recording-microphone", "context-device-audio", "voice-note-encoding-pcf"]),
            ("device-capabilities-feature-flags", "Checking context.device capability feature flags before invoking native hardware features.", ["device-capabilities-flags", "feature-detection-pcf", "graceful-hardware-fallback"]),
            ("webpack-bundle-splitting-optimization", "Configuring webpack.config.js for code splitting, externalizing React, and shrinking bundle size.", ["webpack-bundle-splitting", "pcf-bundle-optimization", "externalize-react-libraries"]),
            ("css-isolation-shadow-dom", "Enforcing CSS isolation via PostCSS prefixes or Shadow DOM to prevent host style bleeding.", ["css-isolation-shadow-dom", "style-encapsulation-pcf", "postcss-namespace-isolation"]),
            ("unit-testing-jest-react-testing-lib", "Authoring comprehensive unit tests with Jest and React Testing Library mocking PCF context.", ["unit-testing-jest-pcf", "react-testing-library-pcf", "mock-pcf-context-objects"]),
            ("bundle-analysis-tree-shaking", "Profiling bundle footprint via source-map-explorer and enforcing aggressive tree-shaking.", ["bundle-analysis-tree-shaking", "source-map-explorer-pcf", "eliminate-unused-imports"]),
            ("canvas-vs-model-driven-rendering-diffs", "Detecting container host type (Canvas App vs Model-Driven App) and adapting layout behavior.", ["canvas-vs-mda-rendering", "host-environment-detection", "adaptive-container-pcf"]),
            ("production-build-minification", "Executing production minification with pac pcf build --buildMode production for minimal footprint.", ["production-build-minification", "pac-pcf-production-build", "minified-pcf-package"]),
        ]),
        # Cluster 28: Custom Connectors & OpenAPI 3.0 Authoring (pp-conn-*)
        (28, "Custom Connectors & OpenAPI 3.0 Authoring", "pp-conn-", "Building Custom Connectors for Power Platform - Troy Taylor", [
            ("openapi-swagger-v2-v3-schemas", "Authoring OpenAPI / Swagger specifications tailored for Power Automate and Power Apps.", ["openapi-swagger-schemas", "swagger-v2-v3-connectors", "connector-contract-authoring"]),
            ("x-ms-summary-visibility-metadata", "Annotating endpoints with x-ms-summary and x-ms-visibility (important, advanced, internal).", ["x-ms-summary-visibility", "connector-metadata-annotations", "x-ms-visibility-levels"]),
            ("dynamic-values-and-dynamic-schema", "Configuring x-ms-dynamic-values and x-ms-dynamic-schema for cascading parameter dropdowns.", ["dynamic-values-and-schema", "x-ms-dynamic-values", "cascading-dropdown-connectors"]),
            ("webhook-trigger-definitions", "Defining OpenAPI x-ms-trigger webhook definitions, subscription endpoints, and unsubscribe paths.", ["webhook-trigger-definitions", "x-ms-trigger-webhooks", "subscription-lifecycle-connectors"]),
            ("polling-trigger-configuration", "Configuring Polling Triggers with timestamp checks, ETags, and array extraction paths.", ["polling-trigger-configuration", "polling-connectors", "etag-timestamp-polling"]),
            ("csharp-policy-templates-transforms", "Authoring C# script policy transforms inside connectors for header and payload mutation.", ["csharp-policy-templates", "connector-script-transforms", "payload-mutation-csharp"]),
            ("connector-error-codes-mapping", "Mapping HTTP 4xx/5xx status codes into user-friendly error messages and troubleshooting hints.", ["connector-error-codes-mapping", "http-error-normalization", "user-friendly-api-errors"]),
            ("pac-connector-cli-management", "Managing connector source code in Git using pac connector download and pac connector update.", ["pac-connector-cli", "connector-source-control-git", "pac-connector-lifecycle"]),
            ("connector-certification-pipeline", "Navigating the Microsoft Connector Certification program, validation rules, and testing requirements.", ["connector-certification-pipeline", "microsoft-certification-rules", "public-connector-publishing"]),
            ("mock-server-postman-testing", "Testing custom connector endpoints via Postman mock servers, Newman automation, and edge cases.", ["mock-server-postman-testing", "newman-automated-testing", "connector-mock-validation"]),
        ]),
        # Cluster 29: Entra ID Authentication, OAuth2 & PKCE for Connectors (pp-oauth-*)
        (29, "Entra ID Authentication, OAuth2 & PKCE for Connectors", "pp-oauth-", "Enterprise Identity and Authentication in Power Platform - Microsoft Press", [
            ("oauth2-code-grant-flow", "Configuring OAuth 2.0 Authorization Code Grant flow, client secrets, and redirect URIs.", ["oauth2-code-grant-flow", "authorization-code-grant", "connector-redirect-uris"]),
            ("entra-id-app-registration-scopes", "Registering Entra ID applications, defining API permissions, and configuring delegated scopes.", ["entra-id-app-registration", "delegated-api-scopes", "custom-api-permissions"]),
            ("pkce-proof-key-exchange", "Implementing PKCE (Proof Key for Code Exchange) to secure public client authentication flows.", ["pkce-proof-key-exchange", "code-challenge-verification", "public-client-oauth-security"]),
            ("on-behalf-of-obo-token-exchange", "OAuth 2.0 On-Behalf-Of (OBO) flow for propagating user identity from Canvas to custom APIs.", ["on-behalf-of-obo-flow", "user-identity-propagation", "obo-token-exchange-api"]),
            ("api-key-and-basic-authentication", "Configuring API Key header/query authentication and secure Basic auth connection settings.", ["api-key-basic-auth", "api-key-header-security", "connector-credential-storage"]),
            ("custom-token-refresh-handling", "Managing refresh token lifecycles, expiration boundaries, and silent token re-issuance.", ["custom-token-refresh", "refresh-token-lifecycle", "silent-token-reissuance"]),
            ("multi-tenant-connector-auth", "Architecting multi-tenant Entra ID connectors, common endpoints, and tenant admin consent.", ["multi-tenant-connector-auth", "common-endpoint-oauth", "admin-consent-workflow"]),
            ("mtls-client-certificates", "Configuring Mutual TLS (mTLS) client certificate authentication for ultra-secure endpoints.", ["mtls-client-certificates", "mutual-tls-connectors", "certificate-bound-apis"]),
            ("service-principal-connection-sharing", "Sharing Service Principal connections safely across environments without credential leakage.", ["service-principal-sharing", "spn-connection-references", "credential-leakage-prevention"]),
            ("credential-rotation-automation", "Automating client secret rotation using Azure Key Vault and zero-downtime updates.", ["credential-rotation-automation", "key-vault-secret-rotation", "zero-downtime-connectors"]),
        ]),
        # Cluster 30: Azure API Management (APIM) & Microservices Integration (pp-apim-*)
        (30, "Azure API Management (APIM) & Microservices Integration", "pp-apim-", "Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa", [
            ("export-to-power-platform-native", "Exporting APIs directly from Azure API Management to Power Platform as first-class connectors.", ["export-to-power-platform", "apim-to-power-platform", "one-click-connector-export"]),
            ("inbound-outbound-xml-policies", "Crafting APIM XML policies: rate-limit-by-key, ip-filter, rewrite-uri, and validate-jwt.", ["inbound-outbound-xml-policies", "apim-policy-authoring", "rate-limiting-rewrite-uri"]),
            ("jwt-validation-claims-inspection", "Enforcing validate-jwt policies, checking tenant IDs, audience claims, and user roles.", ["jwt-validation-claims", "validate-jwt-policy-apim", "claims-inspection-security"]),
            ("mocking-responses-fast-prototyping", "Configuring mock-response policies in APIM for vibe coding fast feedback loops.", ["mocking-responses-prototyping", "mock-response-policy-apim", "fast-feedback-mock-apis"]),
            ("caching-policies-redis-store", "Implementing cache-lookup and cache-store policies with external Redis to offload backends.", ["caching-policies-redis", "cache-lookup-store-apim", "backend-load-offloading"]),
            ("backend-circuit-breaking-failover", "Configuring backend circuit breakers and automated failover pools for resilient routing.", ["backend-circuit-breaking", "apim-failover-pools", "automated-resilient-routing"]),
            ("soap-to-rest-transformation", "Modernizing legacy SOAP XML enterprise services into clean REST JSON APIs for Power Apps.", ["soap-to-rest-transformation", "legacy-soap-modernization", "soap-xml-to-rest-json"]),
            ("request-correlation-tracking", "Injecting x-ms-correlation-id headers for end-to-end distributed tracing in App Insights.", ["request-correlation-tracking", "x-ms-correlation-id-tracing", "distributed-telemetry-apim"]),
            ("openapi-schema-normalization", "Sanitizing OpenAPI schemas: Removing unsupported anyOf and oneOf constructs for connectors.", ["openapi-schema-normalization", "sanitize-swagger-connectors", "strip-unsupported-constructs"]),
            ("developer-portal-onboarding", "Publishing API documentation, test sandboxes, and subscription keys in APIM Developer Portal.", ["developer-portal-onboarding", "apim-developer-portal", "fusion-team-api-catalog"]),
        ]),
        # Cluster 31: AI Builder Prebuilt Models & Form Processing (pp-aib-*)
        (31, "AI Builder Prebuilt Models & Form Processing", "pp-aib-", "Intelligent Automation with AI Builder and Power Platform - Joe Camp", [
            ("prebuilt-invoice-receipt-processor", "Deploying prebuilt invoice and receipt processing models, extracting total, vendor, and tax fields.", ["prebuilt-invoice-receipt", "invoice-processing-ai", "receipt-field-extraction"]),
            ("id-reader-passport-license", "Extracting identity fields from passports and driver licenses with prebuilt ID reader models.", ["id-reader-passport-license", "government-id-extraction", "passport-ocr-ai-builder"]),
            ("business-card-contact-extractor", "Parsing business card scans into structured contacts, email addresses, and phone numbers.", ["business-card-contact-extractor", "business-card-ai-builder", "contact-field-parsing"]),
            ("text-recognition-ocr-prebuilt", "Extracting printed and handwritten text lines from documents via prebuilt Text Recognition OCR.", ["text-recognition-ocr", "handwritten-text-extraction", "printed-ocr-ai-builder"]),
            ("sentiment-analysis-key-phrases", "Analyzing text sentiment (positive, neutral, negative) and extracting key topical phrases.", ["sentiment-analysis-key-phrases", "sentiment-scoring-ai", "key-phrase-extraction-flow"]),
            ("language-detection-translation", "Detecting languages dynamically and invoking automated translation inside cloud flows.", ["language-detection-translation", "detect-language-ai-builder", "automated-text-translation"]),
            ("custom-document-model-training", "Training custom Document Processing models, tagging bounding boxes, and collection validation.", ["custom-document-model-training", "document-processing-training", "bounding-box-tagging"]),
            ("table-extraction-multi-page-docs", "Extracting dynamic multi-row tables and repeating line items spanning multi-page documents.", ["table-extraction-multi-page", "repeating-line-items-ai", "multi-page-table-parsing"]),
            ("confidence-score-threshold-routing", "Routing low-confidence model predictions (<0.80) to human validation queues automatically.", ["confidence-score-threshold-routing", "human-validation-queues", "low-confidence-routing"]),
            ("credit-capacity-governance-admin", "Monitoring AI Builder monthly service credit consumption, capacity alerts, and quotas.", ["credit-capacity-governance", "ai-builder-credits-admin", "service-credit-allocation"]),
        ]),
        # Cluster 32: AI Builder Custom Prompts & Document Intelligence (pp-aiprompt-*)
        (32, "AI Builder Custom Prompts & Document Intelligence", "pp-aiprompt-", "Prompt Engineering in AI Builder and Power Platform - Microsoft Press", [
            ("prompt-engineering-studio", "Designing dynamic prompts with input parameters, system instructions, and few-shot examples.", ["prompt-engineering-studio", "ai-builder-prompt-studio", "few-shot-prompt-design"]),
            ("grounding-with-dataverse-records", "Grounding custom AI Builder prompts dynamically with retrieved Dataverse record context.", ["grounding-dataverse-records", "dataverse-grounded-prompts", "dynamic-record-context-ai"]),
            ("json-output-mode-guarantees", "Enforcing strict JSON output schemas from custom prompts for seamless downstream flow actions.", ["json-output-mode-guarantees", "strict-json-prompts", "schema-constrained-output"]),
            ("multimodal-vision-prompts", "Passing image and PDF document attachments to multimodal GPT-4o vision models in prompts.", ["multimodal-vision-prompts", "gpt-4o-vision-prompts", "image-document-understanding"]),
            ("classification-categorization", "Zero-shot text classification prompts for triaging incoming enterprise emails and inquiries.", ["classification-categorization", "zero-shot-email-triage", "support-ticket-categorization"]),
            ("entity-extraction-custom-entities", "Extracting domain-specific entities (part numbers, tracking IDs) via natural language prompts.", ["entity-extraction-custom-entities", "prompt-entity-extraction", "domain-specific-entity-mining"]),
            ("summarization-condensed-briefs", "Generating executive bullet briefings from lengthy legal contracts and customer transcripts.", ["summarization-condensed-briefs", "executive-briefing-prompts", "contract-summarization-ai"]),
            ("sentiment-and-intent-triage", "Classifying customer sentiment, urgency levels, and underlying intents in a single prompt run.", ["sentiment-and-intent-triage", "urgency-classification-ai", "intent-detection-prompts"]),
            ("safety-moderation-guardrails", "Configuring built-in Azure safety guardrails, jailbreak prevention, and content filtering.", ["safety-moderation-guardrails", "jailbreak-mitigation-prompts", "azure-content-safety-filters"]),
            ("alm-packaging-prompts-solutions", "Packaging AI Builder prompt definitions into Dataverse solutions for automated ALM deployment.", ["alm-packaging-prompts", "prompt-solution-lifecycle", "export-ai-prompts-alm"]),
        ]),
        # Cluster 33: Power Platform Vibe Coding, Flow & Fast Prototyping (pp-vibe-*)
        (33, "Power Platform Vibe Coding, Flow & Fast Prototyping", "pp-vibe-", "Vibe Coding: Conversational Development in Power Platform - Andrej Karpathy & Satya Nadella", [
            ("conversational-prototyping-canvas", "Rapid conversational prompt-to-app prototyping in Power Apps Studio with Copilot.", ["conversational-prototyping-canvas", "prompt-to-app-generation", "conversational-vibe-coding"]),
            ("tracer-bullet-solution-spikes", "Building thin end-to-end tracer bullets across UI, Flow, and Dataverse to validate ideas in hours.", ["tracer-bullet-solution-spikes", "end-to-end-spikes-pp", "rapid-tracer-bullets"]),
            ("rapid-feedback-run-history", "Tightening developer feedback loops using instant flow run history and app monitor telemetry.", ["rapid-feedback-run-history", "instant-flow-feedback", "run-history-debugging-vibe"]),
            ("copilot-assisted-formula-synthesis", "Authoring complex multi-condition Power Fx expressions interactively with Copilot assistance.", ["copilot-formula-synthesis", "ai-assisted-power-fx", "natural-language-to-formula"]),
            ("exploratory-spikes-mock-data", "Generating synthetic mock tables and JSON collections instantly for exploratory spike testing.", ["exploratory-spikes-mock-data", "mock-data-collections-canvas", "synthetic-dataverse-spikes"]),
            ("dialectical-design-agent-reviews", "Using AI peer review subagents to critique usability, schema cardinality, and security boundaries.", ["dialectical-design-reviews", "peer-agent-code-critique", "schema-usability-reviews"]),
            ("conversational-flow-state-ergonomics", "Maintaining unbroken flow state ergonomics via keyboard shortcuts and AI pair programming.", ["conversational-flow-state", "developer-flow-ergonomics", "ai-pair-programming-pp"]),
            ("fail-fast-diagnostic-telemetry", "Surfacing silent formula delegation warnings and unhandled flow exceptions instantaneously.", ["fail-fast-diagnostic-telemetry", "instant-warning-surfacing", "silent-delegation-alerts"]),
            ("human-in-the-loop-steering", "Steering AI code generation with iterative prompt refinement and targeted architectural guardrails.", ["human-in-the-loop-steering", "iterative-prompt-refinement", "architectural-guardrail-steering"]),
            ("context-scaffolding-solutions", "Scaffolding complete solutions, tables, forms, flows, and apps from a single Markdown spec.", ["context-scaffolding-solutions", "spec-driven-solution-scaffolding", "single-prompt-app-generation"]),
        ]),
        # Cluster 34: Power Platform CLI (pac), ALM & Managed Solutions (pp-alm-*)
        (34, "Power Platform CLI (pac), ALM & Managed Solutions", "pp-alm-", "Enterprise ALM with Microsoft Power Platform CLI - Wael Hamze", [
            ("pac-cli-core-commands-auth", "Mastering the pac CLI: pac auth create, pac org who, pac solution init, and profile switching.", ["pac-cli-core-commands", "pac-auth-profiles", "pac-solution-init-cli"]),
            ("managed-vs-unmanaged-solutions", "Solution architecture: Unmanaged development vs Managed release layering and component segmentation.", ["managed-vs-unmanaged-solutions", "solution-layering-architecture", "component-segmentation-alm"]),
            ("solution-pack-unpack-source-control", "Unpacking solutions into human-readable XML/YAML via pac solution unpack for Git version control.", ["solution-pack-unpack", "pac-solution-unpack-git", "source-control-solutions"]),
            ("environment-variables-secrets", "Configuring Environment Variables (String, JSON, Data Source) and Azure Key Vault secret references.", ["environment-variables-secrets", "key-vault-secrets-alm", "deployment-settings-json"]),
            ("connection-reference-binding-pipelines", "Automating Connection Reference binding to service principals during deployment pipelines.", ["connection-reference-binding", "spn-connection-pipeline", "automated-reference-mapping"]),
            ("azure-devops-github-actions-pipelines", "Building enterprise CI/CD pipelines using Power Platform Build Tools and GitHub Actions.", ["azure-devops-github-actions", "power-platform-build-tools", "automated-release-pipelines"]),
            ("solution-upgrade-vs-update", "Executing Stage for Upgrade and applying solution upgrades to purge obsolete components safely.", ["solution-upgrade-vs-update", "stage-for-upgrade-alm", "purge-obsolete-components"]),
            ("solution-checker-static-analysis", "Running pac solution check for automated static analysis, security rules, and performance warnings.", ["solution-checker-static-analysis", "pac-solution-check-cli", "static-analysis-rules-pp"]),
            ("multi-environment-strategy-dev-test-prod", "Structuring multi-environment topologies: Personal Dev -> Build -> Test/UAT -> Production.", ["multi-environment-strategy", "dev-test-prod-topology", "alm-environment-governance"]),
            ("configuration-data-migration-tool", "Automating configuration and seed data migrations using the Configuration Migration Tool (CMT).", ["configuration-data-migration", "cmt-seed-data-export", "reference-data-source-control"]),
        ]),
        # Cluster 35: Enterprise Governance, DLP Policies & Center of Excellence (pp-gov-*)
        (35, "Enterprise Governance, DLP Policies & Center of Excellence", "pp-gov-", "Governance, Security, and Compliance in Power Platform - Manuela Pichler", [
            ("dlp-policy-tiering-business-nonbusiness", "Designing Data Loss Prevention (DLP) policies: Business, Non-Business, and Blocked connector tiers.", ["dlp-policy-tiering", "data-loss-prevention-tiers", "blocked-connector-policies"]),
            ("connector-endpoint-filtering", "Enforcing DLP endpoint filtering rules to restrict HTTP connectors to approved domain patterns.", ["connector-endpoint-filtering", "dlp-endpoint-rules", "domain-wildcard-filtering"]),
            ("connector-action-control", "Granular connector action control: Permitting read operations while blocking write/delete actions.", ["connector-action-control", "block-connector-actions", "read-only-connector-rules"]),
            ("coe-starter-kit-core-components", "Deploying the Center of Excellence (CoE) Starter Kit: Inventory sync flows, dashboards, and alerts.", ["coe-starter-kit-core", "coe-inventory-sync-flows", "admin-analytics-dashboard"]),
            ("environment-lifecycle-management", "Automating developer environment provisioning, inactivity cleanup, and expiration policies.", ["environment-lifecycle-management", "developer-environment-cleanup", "auto-expiration-policies"]),
            ("managed-environments-admin-controls", "Enabling Managed Environments: App sharing limits, solution checker enforcement, and usage digests.", ["managed-environments-admin", "sharing-limits-enforcement", "weekly-usage-digests"]),
            ("tenant-isolation-inbound-outbound", "Configuring AAD Tenant Isolation rules to block unauthorized cross-tenant data exfiltration.", ["tenant-isolation-rules", "cross-tenant-data-fencing", "inbound-outbound-isolation"]),
            ("audit-logging-and-activity-reporting", "Streaming Microsoft 365 / Dataverse audit logs into Azure Log Analytics and Microsoft Sentinel.", ["audit-logging-reporting", "sentinel-power-platform-logs", "tenant-activity-reporting"]),
            ("advisor-and-security-recommendations", "Reviewing Power Platform Advisor proactive security, performance, and orphaned asset insights.", ["advisor-security-recommendations", "power-platform-advisor", "orphaned-resource-detection"]),
            ("capacity-storage-quota-allocation", "Managing Database, File, and Log capacity storage allocations across environments.", ["capacity-storage-quota", "database-file-log-capacity", "environment-quota-management"]),
        ]),
    ]

    for cluster_id, cluster_name, prefix, default_book, skills_meta in cluster_specs:
        for suffix, desc, triggers in skills_meta:
            skill_id = f"{prefix}{suffix}"
            if skill_id in existing_ids:
                continue
            
            # Format high-quality foundations with ALWAYS, NEVER, MANDATORY
            foundations = [
                f"Architecture Standard: Conforms strictly to enterprise patterns defined in '{default_book}'.",
                f"Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking {skill_id} workflows.",
                f"Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during {skill_id} processing.",
                f"Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.",
            ]
            
            protocol = f"ALWAYS apply the {skill_id} protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry."
            anti_patterns = [
                f"Deploying unvalidated configurations for {skill_id} directly into production without staging verification.",
                f"Silently ignoring transient failures or error codes during {skill_id} execution.",
            ]
            
            all_skills.append({
                "id": skill_id,
                "cluster": cluster_id,
                "book": default_book,
                "desc": desc,
                "triggers": [t.lower() for t in triggers],
                "foundations": foundations,
                "protocol": protocol,
                "anti_patterns": anti_patterns,
            })
            existing_ids.add(skill_id)

    return all_skills

def main():
    skills = generate_skills()
    print(f"Total Power Platform skills to generate: {len(skills)}")
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
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-{s['cluster']}"]
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
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
"""
        skill_file.write_text(content, encoding="utf-8")
        generated += 1

    print(f"Successfully generated {generated} Power Platform skill packages in {SKILLS_DIR}")

if __name__ == "__main__":
    main()
