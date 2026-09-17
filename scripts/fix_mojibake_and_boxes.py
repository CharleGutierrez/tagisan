#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Fixes all control characters, empty tags, and mojibake boxes in TGS_MS_POWER_PLATFORM_MASTERCLASS_COURSE.html.
Restores all missing instructional statements, headings, code tags, and navigation paths.
"""

import re
import os

HTML_PATH = os.path.join(os.path.dirname(__file__), "..", "docs", "TGS_MS_POWER_PLATFORM_MASTERCLASS_COURSE.html")

def fix_html():
    with open(HTML_PATH, "r", encoding="utf-8") as f:
        html = f.read()

    initial_bad_count = html.count("\x01")
    print(f"Initial \\x01 count: {initial_bad_count}")

    # Explicit replacements for all corrupted lines in Modules 1-10

    # Module 1
    html = html.replace(
        "<h3>\x01</h3>\n<p>In modern Global 2000 enterprises, software engineering operates at the critical intersection of developer toolchains and low-code business operations. Mission-critical business processes, compliance approvals, financial transactions, and operational incident war-rooms reside within the <strong>\x01</strong> (Power Apps, Power Automate, Copilot Studio, and Power BI).</p>",
        "<h3>1.1 The Sovereign Paradigm: Invoking TGS Inside Microsoft Power Platform</h3>\n<p>In modern Global 2000 enterprises, software engineering operates at the critical intersection of developer toolchains and low-code business operations. Mission-critical business processes, compliance approvals, financial transactions, and operational incident war-rooms reside within the <strong>Microsoft Power Platform</strong> (Power Apps, Power Automate, Copilot Studio, and Power BI).</p>"
    )

    html = html.replace(
        "<p>However, low-code platforms traditionally lack deterministic mathematical verification, dialectical multi-agent consensus, and deep abstract syntax tree (AST) code analysis. <strong>\x01</strong> bridges this divide by enabling low-code makers, professional developers, and enterprise cloud architects to <strong>\x01</strong>.</p>",
        "<p>However, low-code platforms traditionally lack deterministic mathematical verification, dialectical multi-agent consensus, and deep abstract syntax tree (AST) code analysis. <strong>Tagisan (<code>tgs</code>) Subsystem 5</strong> bridges this divide by enabling low-code makers, professional developers, and enterprise cloud architects to <strong>call Tagisan directly from inside Microsoft Power Platform tools</strong>.</p>"
    )

    html = html.replace(
        "  <li><strong>\x01</strong> in Canvas Apps trigger real-time Hegelian dialectical debates on code and architectural proposals via concise Power Fx formulas.</li>\n"
        "  <li><strong>\x01</strong> in Power Automate automate pull request audits, blast-radius mitigation, and incident escalations with sub-second TGS evaluations.</li>\n"
        "  <li><strong>\x01</strong> in Microsoft Dataverse enforce transaction-level gates using C# Plugins and Custom APIs that query TGS before database commit.</li>\n"
        "  <li><strong>\x01</strong> in Microsoft Copilot Studio invoke TGS tools via Declarative AI Plugins, generating conversational responses strictly grounded in verified architectural decision records (ADRs).</li>\n"
        "  <li><strong>\x01</strong> on Power Pages sovereign portals verify Ed25519 asymmetric cryptographic receipts issued by TGS for every automated change.</li>",
        "  <li><strong>Low-Code Makers</strong> in Canvas Apps trigger real-time Hegelian dialectical debates on code and architectural proposals via concise Power Fx formulas.</li>\n"
        "  <li><strong>Enterprise SREs</strong> in Power Automate automate pull request audits, blast-radius mitigation, and incident escalations with sub-second TGS evaluations.</li>\n"
        "  <li><strong>Enterprise Architects</strong> in Microsoft Dataverse enforce transaction-level gates using C# Plugins and Custom APIs that query TGS before database commit.</li>\n"
        "  <li><strong>AI Operators</strong> in Microsoft Copilot Studio invoke TGS tools via Declarative AI Plugins, generating conversational responses strictly grounded in verified architectural decision records (ADRs).</li>\n"
        "  <li><strong>Compliance Auditors</strong> on Power Pages sovereign portals verify Ed25519 asymmetric cryptographic receipts issued by TGS for every automated change.</li>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n<p>Subsystem 5 establishes six distinct in-platform invocation patterns, connecting low-code Microsoft cloud services to the sovereign Tagisan engine:</p>",
        "<h3>1.2 In-Platform Calling Topologies &amp; The 6 Invocation Modes</h3>\n<p>Subsystem 5 establishes six distinct in-platform invocation patterns, connecting low-code Microsoft cloud services to the sovereign Tagisan engine:</p>"
    )

    html = html.replace(
        "  <li><strong>\x01</strong>: Makers write declarative formulas invoking <code class=\"inline-code\">\x01</code> or <code class=\"inline-code\">\x01</code>, populating in-memory collections (<code class=\"inline-code\">\x01</code>) and driving reactive galleries.</li>\n"
        "  <li><strong>\x01</strong>: Flow triggers (GitHub PRs, Dataverse events, Jira webhooks) invoke the TGS Custom Connector action or authenticated HTTP endpoints with HMAC validation, parsing TGS response JSON and branching on consensus scores.</li>\n"
        "  <li><strong>\x01</strong>: AI plugins bind TGS tools (<code class=\"inline-code\">\x01</code>, <code class=\"inline-code\">\x01</code>, <code class=\"inline-code\">\x01</code>) to Copilot's Generative Orchestrator, enabling conversational prompts grounded in verified TGS audit trails.</li>\n"
        "  <li><strong>\x01</strong>: Synchronous C# Plugins (<code class=\"inline-code\">\x01</code>), Custom APIs, and Virtual Tables invoke TGS REST endpoints via <code class=\"inline-code\">\x01</code> using Entra ID service principal tokens to gate transactions before database commit.</li>\n"
        "  <li><strong>\x01</strong>: External sovereign portals execute portal Web API JavaScript and Liquid templates calling TGS endpoints to verify cryptographic signatures and display live compliance attestations.</li>\n"
        "  <li><strong>\x01</strong>: Power Query M scripts call TGS REST endpoints (<code class=\"inline-code\">\x01</code>), while Direct Lake semantic models read real-time Parquet Delta Lake tables with zero ETL delay.</li>",
        "  <li><strong>Power Apps Canvas Studio (Power Fx)</strong>: Makers write declarative formulas invoking <code class=\"inline-code\">TGSConnector.ExecuteDebate(...)</code> or <code class=\"inline-code\">TGSConnector.AnalyzeAST(...)</code>, populating in-memory collections (<code class=\"inline-code\">ClearCollect(colTgsNodes, ...)</code>) and driving reactive galleries.</li>\n"
        "  <li><strong>Power Automate Cloud Flows</strong>: Flow triggers (GitHub PRs, Dataverse events, Jira webhooks) invoke the TGS Custom Connector action or authenticated HTTP endpoints with HMAC validation, parsing TGS response JSON and branching on consensus scores.</li>\n"
        "  <li><strong>Microsoft Copilot Studio AI Plugins</strong>: AI plugins bind TGS tools (<code class=\"inline-code\">CallTGSEngine</code>, <code class=\"inline-code\">AnalyzeCodeAST</code>, <code class=\"inline-code\">ResolveArchitecturalDebate</code>) to Copilot's Generative Orchestrator, enabling conversational prompts grounded in verified TGS audit trails.</li>\n"
        "  <li><strong>Microsoft Dataverse Native Extensions</strong>: Synchronous C# Plugins (<code class=\"inline-code\">IPlugin</code>), Custom APIs, and Virtual Tables invoke TGS REST endpoints via <code class=\"inline-code\">HttpClient</code> using Entra ID service principal tokens to gate transactions before database commit.</li>\n"
        "  <li><strong>Power Pages Compliance Portals</strong>: External sovereign portals execute portal Web API JavaScript and Liquid templates calling TGS endpoints to verify cryptographic signatures and display live compliance attestations.</li>\n"
        "  <li><strong>Power BI &amp; Microsoft Fabric OneLake</strong>: Power Query M scripts call TGS REST endpoints (<code class=\"inline-code\">Json.Document(Web.Contents(...))</code>), while Direct Lake semantic models read real-time Parquet Delta Lake tables with zero ETL delay.</li>"
    )

    # Module 1.3
    html = html.replace(
        "  <li><strong>\x01</strong>: Power Platform never stores raw passwords. All invocations leverage Microsoft Entra ID Managed Identities, Continuous Access Evaluation (CAE), or Azure Workload Identity Federation (WIF).</li>\n"
        "  <li><strong>\x01</strong>: High blast-radius code modifications are blocked from reaching production Dataverse tables unless a 5-stage Hegelian multi-agent debate reaches an approved consensus score.</li>\n"
        "  <li><strong>\x01</strong>: Every execution emits an Ed25519 digital signature verified against sovereign public keys, establishing non-repudiable audit trails.</li>",
        "  <li><strong>Zero Plaintext Secrets</strong>: Power Platform never stores raw passwords. All invocations leverage Microsoft Entra ID Managed Identities, Continuous Access Evaluation (CAE), or Azure Workload Identity Federation (WIF).</li>\n"
        "  <li><strong>Dialectical Verification</strong>: High blast-radius code modifications are blocked from reaching production Dataverse tables unless a 5-stage Hegelian multi-agent debate reaches an approved consensus score.</li>\n"
        "  <li><strong>Cryptographic Receipt Ledger</strong>: Every execution emits an Ed25519 digital signature verified against sovereign public keys, establishing non-repudiable audit trails.</li>"
    )

    # Module 2
    html = html.replace(
        "<h2>Module 2: Complete Installation, Toolchains & Entra ID Setup for Calling TGS</h2>\n\n<h3>\x01</h3>\n<p>To call TGS from inside Microsoft Power Platform, professional developers and administrators must establish the dual CLI orchestration environment:</p>",
        "<h2>Module 2: Complete Installation, Toolchains &amp; Entra ID Setup for Calling TGS</h2>\n\n<h3>2.1 Developer Toolchain Installation &amp; Prerequisites</h3>\n<p>To call TGS from inside Microsoft Power Platform, professional developers and administrators must establish the dual CLI orchestration environment:</p>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n<p>To enable secure service-to-service communication between Power Platform and the TGS API Gateway, configure an Entra ID Application Registration using Azure CLI (<code class=\"inline-code\">\x01</code>):</p>",
        "<h3>2.2 Microsoft Entra ID App Registration for Calling TGS</h3>\n<p>To enable secure service-to-service communication between Power Platform and the TGS API Gateway, configure an Entra ID Application Registration using Azure CLI (<code class=\"inline-code\">az ad</code>):</p>"
    )

    # Asterisks in secret
    html = html.replace(
        "\"  Client Secret : <strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong><strong>\x01</strong>\"",
        "\"  Client Secret : ****************************************\""
    )

    html = html.replace(
        "<h3>\x01</h3>\n<p>In sovereign enterprise topologies where the TGS engine runs within private corporate datacenters or isolated Azure Virtual Networks:</p>\n<ul>\n"
        "  <li><strong>\x01</strong>: Install the OPDG cluster on a dedicated Linux/Windows host with outbound HTTPS (port 443) to the Power Platform cloud. The gateway proxies inbound Custom Connector requests directly to <code class=\"inline-code\">\x01</code> without opening public inbound firewall ports.</li>\n"
        "  <li><strong>\x01</strong>: For Azure-hosted containerized TGS clusters, deploy an Azure VNet Data Gateway in the delegated subnet to route Power Platform traffic over private IP backbones with zero public ingress.</li>\n"
        "</ul>\n\n<h3>\x01</h3>",
        "<h3>2.3 Hybrid Connectivity: On-Premises Data Gateway &amp; Azure VNet Gateways</h3>\n<p>In sovereign enterprise topologies where the TGS engine runs within private corporate datacenters or isolated Azure Virtual Networks:</p>\n<ul>\n"
        "  <li><strong>On-Premises Data Gateway (OPDG)</strong>: Install the OPDG cluster on a dedicated Linux/Windows host with outbound HTTPS (port 443) to the Power Platform cloud. The gateway proxies inbound Custom Connector requests directly to <code class=\"inline-code\">http://localhost:8080</code> without opening public inbound firewall ports.</li>\n"
        "  <li><strong>Azure VNet Data Gateway</strong>: For Azure-hosted containerized TGS clusters, deploy an Azure VNet Data Gateway in the delegated subnet to route Power Platform traffic over private IP backbones with zero public ingress.</li>\n"
        "</ul>\n\n<h3>2.4 PAC CLI Authentication Handshake</h3>"
    )

    # Module 3
    html = html.replace(
        "<h2>Module 3: Building & Certifying the TGS Custom Connector to Enable In-Platform Calls</h2>\n\n<h3>\x01</h3>\n<p>Power Apps, Power Automate, and Copilot Studio interact with external REST services through <strong>\x01</strong>. A production-grade connector wraps the Tagisan REST Gateway using an OpenAPI 3.0 (Swagger) specification annotated with Microsoft vendor extensions (<code class=\"inline-code\">\x01</code>).</p>",
        "<h2>Module 3: Building &amp; Certifying the TGS Custom Connector to Enable In-Platform Calls</h2>\n\n<h3>3.1 Certified Custom Connectors: The Bridge for Calling TGS</h3>\n<p>Power Apps, Power Automate, and Copilot Studio interact with external REST services through <strong>Custom Connectors</strong>. A production-grade connector wraps the Tagisan REST Gateway using an OpenAPI 3.0 (Swagger) specification annotated with Microsoft vendor extensions (<code class=\"inline-code\">x-ms-*</code>).</p>"
    )

    # Module 3.2
    m3_2_pattern = re.compile(
        r"<h3>\x01</h3>\s*<ol>\s*<li>Sign in to the <strong>\x01</strong> \(<code class=\"inline-code\">\x01</code>\)\.</li>.*?"
        r"<li>Verify the HTTP <code class=\"inline-code\">\x01</code> response returning the synthesized verdict, confidence score, and Ed25519 signature proof\.</li>\s*</ol>",
        re.DOTALL
    )
    m3_2_replacement = (
        "<h3>3.2 Step-by-Step Maker Portal Configuration &amp; Live Test</h3>\n"
        "<ol>\n"
        "  <li>Sign in to the <strong>Power Apps Maker Portal</strong> (<code class=\"inline-code\">https://make.powerapps.com</code>).</li>\n"
        "  <li>In the left navigation, select <strong>Dataverse</strong> &rarr; <strong>Custom Connectors</strong> &rarr; <strong>New custom connector</strong> &rarr; <strong>Import an OpenAPI file</strong>.</li>\n"
        "  <li>In <strong>General information</strong>, set Host to <code class=\"inline-code\">api.tagisan.internal</code> and Base URL to <code class=\"inline-code\">/v1</code>.</li>\n"
        "  <li>In <strong>Security</strong>, select <strong>OAuth 2.0</strong> with <strong>Azure Active Directory</strong>:\n"
        "    <ul>\n"
        "      <li>Client ID: <code class=\"inline-code\">c83f1204-7491-4d32-9b12-45e6789012ab</code></li>\n"
        "      <li>Client Secret: <code class=\"inline-code\">[From Entra ID App Registration]</code></li>\n"
        "      <li>Authorization URL: <code class=\"inline-code\">https://login.microsoftonline.com/{tenantId}/oauth2/v2.0/authorize</code></li>\n"
        "      <li>Token URL: <code class=\"inline-code\">https://login.microsoftonline.com/{tenantId}/oauth2/v2.0/token</code></li>\n"
        "      <li>Refresh URL: <code class=\"inline-code\">https://login.microsoftonline.com/{tenantId}/oauth2/v2.0/token</code></li>\n"
        "      <li>Resource URL: <code class=\"inline-code\">https://api.tagisan.internal</code>.</li>\n"
        "      <li>Scope: <code class=\"inline-code\">api://tagisan.internal/Consensus.Execute</code>.</li>\n"
        "    </ul>\n"
        "  </li>\n"
        "  <li>In <strong>Test</strong>, create a new connection and execute <code class=\"inline-code\">ExecuteDialecticalDebate</code> with a sample payload.</li>\n"
        "  <li>Verify the HTTP <code class=\"inline-code\">200 OK</code> response returning the synthesized verdict, confidence score, and Ed25519 signature proof.</li>\n"
        "</ol>"
    )
    html = m3_2_pattern.sub(m3_2_replacement, html)

    # Module 4: Canvas Apps & Power Fx (The exact user screenshot location!)
    m4_pattern = re.compile(
        r"<h2>Module 4: Calling TGS Inside Power Apps Canvas Apps &amp; Model-Driven Apps via Power Fx</h2>\s*"
        r"<h3>\x01</h3>\s*"
        r"<p>Enterprise governance requires all low-code components, Custom Connectors, and PCF widgets to be bundled into managed, versioned Solution packages \(<code class=\"inline-code\">\x01</code>\) containing <code class=\"inline-code\">\x01</code> and <code class=\"inline-code\">\x01</code>\. Tagisan automates this assembly, ensuring seamless deployment across DEV, TEST, and PROD environments\.</p>\s*"
        r"<h3>\x01</h3>\s*"
        r"<p>Citizen developers and SRE engineers invoke TGS directly from Canvas App button clicks, screen transitions, and timer ticks using declarative <strong>\x01</strong>\.</p>\s*"
        r"<p>Step-by-Step Power Fx Implementation:</p>\s*"
        r"<ol>\s*"
        r"<li>Open the Canvas App in Power Apps Studio\.</li>\s*"
        r"<li>In the <strong>\x01</strong> pane, click <strong>\x01</strong> and select the <code class=\"inline-code\">\x01</code>\.</li>\s*"
        r"<li>Add a Multi-line Text Input \(<code class=\"inline-code\">\x01</code>\), a Slider \(<code class=\"inline-code\">\x01</code>\), an \"Execute Audit\" Button \(<code class=\"inline-code\">\x01</code>\), and a Gallery \(<code class=\"inline-code\">\x01</code>\)\.</li>\s*"
        r"<li>Set the <code class=\"inline-code\">\x01</code> property of <code class=\"inline-code\">\x01</code> to call TGS:</li>\s*"
        r"</ol>",
        re.DOTALL
    )
    m4_replacement = (
        "<h2>Module 4: Calling TGS Inside Power Apps Canvas Apps &amp; Model-Driven Apps via Power Fx</h2>\n\n"
        "<h3>4.1 Solution Packaging Engine &amp; Power Apps Canvas Studio</h3>\n"
        "<p>Enterprise governance requires all low-code components, Custom Connectors, and PCF widgets to be bundled into managed, versioned Solution packages (<code class=\"inline-code\">.zip</code>) containing <code class=\"inline-code\">solution.xml</code> and <code class=\"inline-code\">customizations.xml</code>. Tagisan automates this assembly, ensuring seamless deployment across DEV, TEST, and PROD environments.</p>\n\n"
        "<h3>4.2 Calling TGS Inside Canvas Studio via Power Fx</h3>\n"
        "<p>Citizen developers and SRE engineers invoke TGS directly from Canvas App button clicks, screen transitions, and timer ticks using declarative <strong>Power Fx formulas</strong>.</p>\n\n"
        "<p>Step-by-Step Power Fx Implementation:</p>\n"
        "<ol>\n"
        "  <li>Open the Canvas App in Power Apps Studio.</li>\n"
        "  <li>In the <strong>Data</strong> pane, click <strong>Add data</strong> and select the <code class=\"inline-code\">TGSConnector</code>.</li>\n"
        "  <li>Add a Multi-line Text Input (<code class=\"inline-code\">txtCodeInput</code>), a Slider (<code class=\"inline-code\">sldRiskLimit</code>), an \"Execute Audit\" Button (<code class=\"inline-code\">btnExecuteAudit</code>), and a Gallery (<code class=\"inline-code\">galArguments</code>).</li>\n"
        "  <li>Set the <code class=\"inline-code\">OnSelect</code> property of <code class=\"inline-code\">btnExecuteAudit</code> to call TGS:</li>\n"
        "</ol>"
    )
    html = m4_pattern.sub(m4_replacement, html)

    # Module 4.3
    m4_3_pattern = re.compile(
        r"<h3>\x01</h3>\s*<ul>\s*"
        r"<li><strong>\x01</strong>: Set <code class=\"inline-code\">\x01</code> of <code class=\"inline-code\">\x01</code> to <code class=\"inline-code\">\x01</code>, binding <code class=\"inline-code\">\x01</code> to <code class=\"inline-code\">\x01</code> and <code class=\"inline-code\">\x01</code> to <code class=\"inline-code\">\x01</code>\.</li>\s*"
        r"<li><strong>\x01</strong>: Bind a circular SVG progress meter to <code class=\"inline-code\">\x01</code>\.</li>\s*"
        r"<li><strong>\x01</strong>: Render a badge displaying <code class=\"inline-code\">\x01</code> with a verified green checkmark\.</li>\s*"
        r"</ul>",
        re.DOTALL
    )
    m4_3_replacement = (
        "<h3>4.3 Binding TGS Results to Visual Gallery &amp; Gauges</h3>\n"
        "<ul>\n"
        "  <li><strong>AST Nodes Gallery</strong>: Set <code class=\"inline-code\">Items</code> of <code class=\"inline-code\">galArguments</code> to <code class=\"inline-code\">colThesisPoints</code>, binding <code class=\"inline-code\">Title</code> to <code class=\"inline-code\">ThisItem.ArgumentTitle</code> and <code class=\"inline-code\">Subtitle</code> to <code class=\"inline-code\">ThisItem.EvidenceCodeSnippet</code>.</li>\n"
        "  <li><strong>Hegelian Confidence Gauge</strong>: Bind a circular SVG progress meter to <code class=\"inline-code\">varTgsResponse.synthesis.confidence_pct</code>.</li>\n"
        "  <li><strong>Cryptographic Stamp</strong>: Render a badge displaying <code class=\"inline-code\">Left(varCryptoProof, 16) &amp; \"...\"</code> with a verified green checkmark.</li>\n"
        "</ul>"
    )
    html = m4_3_pattern.sub(m4_3_replacement, html)

    # Module 5: PCF Controls
    html = html.replace(
        "<h2>Module 5: Modern PCF Controls Calling TGS Inside Dataverse Forms (React 18 & Fluent UI v9)</h2>\n\n"
        "<h3>\x01</h3>\n"
        "<p>While Canvas Apps handle low-code forms, Model-Driven Apps and Dataverse standard entity forms require high-performance, responsive custom UI components. The <strong>\x01</strong> allows pro-developers to write TypeScript controls that call TGS endpoints directly and render complex interactive visualizations.</p>\n\n"
        "<h3>\x01</h3>\n"
        "<p>Initialize the PCF field control using PAC CLI:</p>",
        "<h2>Module 5: Modern PCF Controls Calling TGS Inside Dataverse Forms (React 18 &amp; Fluent UI v9)</h2>\n\n"
        "<h3>5.1 Modern PCF Custom Controls with React 18 &amp; Fluent UI v9</h3>\n"
        "<p>While Canvas Apps handle low-code forms, Model-Driven Apps and Dataverse standard entity forms require high-performance, responsive custom UI components. The <strong>Power Apps Component Framework (PCF)</strong> allows pro-developers to write TypeScript controls that call TGS endpoints directly and render complex interactive visualizations.</p>\n\n"
        "<h3>5.2 Scaffolding &amp; Manifest Modeling</h3>\n"
        "<p>Initialize the PCF field control using PAC CLI:</p>"
    )

    html = html.replace(
        "<p>Configure <code class=\"inline-code\">\x01</code> to declare bound Dataverse columns and Web API capabilities:</p>",
        "<p>Configure <code class=\"inline-code\">ControlManifest.Input.xml</code> to declare bound Dataverse columns and Web API capabilities:</p>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n<p>The component uses React 18 and Fluent UI v9 (<code class=\"inline-code\">\x01</code>) to call TGS and render an interactive AST dependency tree:</p>",
        "<h3>5.3 Calling TGS Inside <code>TagisanAstConsensusWidget.tsx</code></h3>\n<p>The component uses React 18 and Fluent UI v9 (<code class=\"inline-code\">@fluentui/react-components</code>) to call TGS and render an interactive AST dependency tree:</p>"
    )

    # Module 5.4
    html = html.replace(
        "<p>Tagisan implements Georg Wilhelm Friedrich Hegel's dialectical triad (<strong>\x01</strong>) as an autonomous multi-agent consensus loop. In Power Automate and PCF controls, this ensures that every code change is rigorously stress-tested before being merged or approved.</p>",
        "<p>Tagisan implements Georg Wilhelm Friedrich Hegel's dialectical triad (<strong>Thesis &rarr; Antithesis &rarr; Synthesis</strong>) as an autonomous multi-agent consensus loop. In Power Automate and PCF controls, this ensures that every code change is rigorously stress-tested before being merged or approved.</p>"
    )

    # Module 6
    html = html.replace(
        "<h2>Module 6: Calling TGS Inside Power Automate Cloud Flows & Desktop Flows</h2>\n\n"
        "<h3>\x01</h3>\n"
        "<p>Tagisan can automatically transpile complex engineering pipelines into native Power Automate Cloud Flow JSON definitions (<code class=\"inline-code\">\x01</code>).</p>",
        "<h2>Module 6: Calling TGS Inside Power Automate Cloud Flows &amp; Desktop Flows</h2>\n\n"
        "<h3>6.1 Power Automate Cloud Flow Transpilation &amp; Calling TGS</h3>\n"
        "<p>Tagisan can automatically transpile complex engineering pipelines into native Power Automate Cloud Flow JSON definitions (<code class=\"inline-code\">workflowDefinition.json</code>).</p>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n"
        "<p>The Power Automate Run History page provides complete visibility into flow execution, showing execution timing, HTTP status codes, and raw JSON input/output payloads.</p>",
        "<h3>6.2 Live Power Automate Flow Execution &amp; Run History</h3>\n"
        "<p>The Power Automate Run History page provides complete visibility into flow execution, showing execution timing, HTTP status codes, and raw JSON input/output payloads.</p>"
    )

    # Module 7
    html = html.replace(
        "<h2>Module 7: Calling TGS Inside Microsoft Dataverse (C# Plugins, Custom APIs & Virtual Tables)</h2>\n\n"
        "<h3>\x01</h3>\n"
        "<p>Deep backend enterprise integration allows Microsoft Dataverse itself to invoke TGS synchronously or asynchronously:</p>\n\n"
        "<h4>\x01</h4>\n"
        "<p>When an architectural record is created or updated in Dataverse, a synchronous C# plugin can call the TGS API Gateway to execute pre-validation before committing the transaction:</p>",
        "<h2>Module 7: Calling TGS Inside Microsoft Dataverse (C# Plugins, Custom APIs &amp; Virtual Tables)</h2>\n\n"
        "<h3>7.1 Calling TGS from Microsoft Dataverse: C# Plugins, Custom APIs &amp; Virtual Tables</h3>\n"
        "<p>Deep backend enterprise integration allows Microsoft Dataverse itself to invoke TGS synchronously or asynchronously:</p>\n\n"
        "<h4>7.1.1 Synchronous C# Plugin Execution Pipeline (<code>IPlugin</code>)</h4>\n"
        "<p>When an architectural record is created or updated in Dataverse, a synchronous C# plugin can call the TGS API Gateway to execute pre-validation before committing the transaction:</p>"
    )

    html = html.replace(
        "<h4>\x01</h4>\n<p>Expose <code class=\"inline-code\">\x01</code> directly as an unmanaged/managed Custom API in Dataverse, making it callable via OData v4 Web API (<code class=\"inline-code\">\x01</code>).</p>\n\n<h4>\x01</h4>",
        "<h4>7.1.2 Custom API Declaration for Calling TGS</h4>\n<p>Expose <code class=\"inline-code\">tgs_ExecuteAstAudit</code> directly as an unmanaged/managed Custom API in Dataverse, making it callable via OData v4 Web API (<code class=\"inline-code\">POST /api/data/v9.2/tgs_ExecuteAstAudit</code>).</p>\n\n<h4>7.1.3 Virtual Table Data Provider</h4>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n"
        "<p>External auditors and third-party compliance assessors verify Tagisan architectural decisions and Ed25519 cryptographic receipts through a hardened, read-only Power Pages sovereign portal:</p>\n"
        "<ul>\n"
        "  <li>Authenticated via Azure B2B / Entra External ID.</li>\n"
        "  <li>Displays live compliance certificates (SOC2 Type II, ISO 27001, FedRAMP High).</li>\n"
        "  <li>Portal Web API JavaScript calls <code class=\"inline-code\">\x01</code> to validate signature authenticity in real time.</li>\n"
        "</ul>",
        "<h3>7.2 Power Pages Sovereign Compliance Portal</h3>\n"
        "<p>External auditors and third-party compliance assessors verify Tagisan architectural decisions and Ed25519 cryptographic receipts through a hardened, read-only Power Pages sovereign portal:</p>\n"
        "<ul>\n"
        "  <li>Authenticated via Azure B2B / Entra External ID.</li>\n"
        "  <li>Displays live compliance certificates (SOC2 Type II, ISO 27001, FedRAMP High).</li>\n"
        "  <li>Portal Web API JavaScript calls <code class=\"inline-code\">tgs_VerifyReceipt</code> to validate signature authenticity in real time.</li>\n"
        "</ul>"
    )

    # Module 8
    html = html.replace(
        "<h2>Module 8: Calling TGS Inside Power BI: Direct Lake, M Query & Real-Time SRE Observability</h2>\n\n"
        "<h3>\x01</h3>\n"
        "<p>Power BI allows business leaders and SRE teams to analyze multi-agent consensus telemetry. Power Query M functions call the TGS REST API Gateway directly to pull real-time metrics:</p>",
        "<h2>Module 8: Calling TGS Inside Power BI: Direct Lake, M Query &amp; Real-Time SRE Observability</h2>\n\n"
        "<h3>8.1 Ingesting TGS Telemetry into Power BI via Power Query M</h3>\n"
        "<p>Power BI allows business leaders and SRE teams to analyze multi-agent consensus telemetry. Power Query M functions call the TGS REST API Gateway directly to pull real-time metrics:</p>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n<p>Tagisan streams high-velocity telemetry into <strong>\x01</strong>:</p>\n<ul>\n"
        "  <li><strong>\x01</strong>: Power BI reports read Parquet Delta Lake files directly from OneLake storage with zero ETL latency and sub-second query speeds.</li>\n"
        "  <li><strong>\x01</strong>: SRE measures compute rolling MTTR, Hegelian consensus confidence percentiles (P50, P95, P99), and Purview DLP compliance scores:</li>",
        "<h3>8.2 Microsoft Fabric OneLake Direct Lake SRE Observability</h3>\n<p>Tagisan streams high-velocity telemetry into <strong>Microsoft Fabric OneLake</strong>:</p>\n<ul>\n"
        "  <li><strong>Direct Lake Semantic Models</strong>: Power BI reports read Parquet Delta Lake files directly from OneLake storage with zero ETL latency and sub-second query speeds.</li>\n"
        "  <li><strong>Real-Time SRE Metrics</strong>: SRE measures compute rolling MTTR, Hegelian consensus confidence percentiles (P50, P95, P99), and Purview DLP compliance scores:</li>"
    )

    # Module 9
    html = html.replace(
        "<h2>Module 9: Calling TGS Inside Microsoft Copilot Studio: Declarative AI Plugins & Generative Grounding</h2>\n\n"
        "<h3>\x01</h3>\n"
        "<p>Tagisan acts as an autonomous knowledge and execution engine for <strong>\x01</strong>:</p>\n"
        "<ul>\n"
        "  <li><strong>\x01</strong>: Exposing Tagisan tools directly to Copilot using OpenAPI 3.0 plugin manifests (<code class=\"inline-code\">\x01</code>).</li>\n"
        "  <li><strong>\x01</strong>: Binding <code class=\"inline-code\">\x01</code>, <code class=\"inline-code\">\x01</code>, and <code class=\"inline-code\">\x01</code> to the Copilot Generative Orchestrator.</li>\n"
        "  <li><strong>\x01</strong>: Grounding Copilot responses in verified Dataverse ADR tables to completely eliminate LLM hallucinations.</li>\n"
        "</ul>",
        "<h2>Module 9: Calling TGS Inside Microsoft Copilot Studio: Declarative AI Plugins &amp; Generative Grounding</h2>\n\n"
        "<h3>9.1 Microsoft Copilot Studio Declarative AI Plugins Calling TGS</h3>\n"
        "<p>Tagisan acts as an autonomous knowledge and execution engine for <strong>Microsoft Copilot Studio</strong>:</p>\n"
        "<ul>\n"
        "  <li><strong>OpenAPI 3.0 Tool Binding</strong>: Exposing Tagisan tools directly to Copilot using OpenAPI 3.0 plugin manifests (<code class=\"inline-code\">ai-plugin.json</code>).</li>\n"
        "  <li><strong>Orchestrator Tool Calling</strong>: Binding <code class=\"inline-code\">CallTGSEngine</code>, <code class=\"inline-code\">AnalyzeCodeAST</code>, and <code class=\"inline-code\">ResolveArchitecturalDebate</code> to the Copilot Generative Orchestrator.</li>\n"
        "  <li><strong>ADR Table Grounding</strong>: Grounding Copilot responses in verified Dataverse ADR tables to completely eliminate LLM hallucinations.</li>\n"
        "</ul>"
    )

    html = html.replace(
        "<h3>\x01</h3>\n"
        "<ol>\n"
        "  <li><strong>\x01</strong>: SRE Engineer asks: *\"Copilot, audit the checkout microservice refactoring proposal for high-risk AST dependencies using Tagisan.\"*</li>\n"
        "  <li><strong>\x01</strong>: Copilot identifies the intent and invokes the <code class=\"inline-code\">\x01</code> tool with the source repository URL.</li>\n"
        "  <li><strong>\x01</strong>: TGS parses syntax trees, computes blast-radius score (3.2 / Low), and returns synthesized evidence.</li>\n"
        "  <li><strong>\x01</strong>: Copilot renders an interactive Adaptive Card with verified green status badges, blast radius breakdown, and an \"Approve Merge\" action button.</li>\n"
        "</ol>",
        "<h3>9.2 Conversational Prompt Lifecycle Calling TGS Inside Copilot</h3>\n"
        "<ol>\n"
        "  <li><strong>User Prompt</strong>: SRE Engineer asks: <em>\"Copilot, audit the checkout microservice refactoring proposal for high-risk AST dependencies using Tagisan.\"</em></li>\n"
        "  <li><strong>Intent &amp; Tool Selection</strong>: Copilot identifies the intent and invokes the <code class=\"inline-code\">AnalyzeCodeAST</code> tool with the source repository URL.</li>\n"
        "  <li><strong>TGS Autonomous Execution</strong>: TGS parses syntax trees, computes blast-radius score (3.2 / Low), and returns synthesized evidence.</li>\n"
        "  <li><strong>Adaptive Card Grounded Output</strong>: Copilot renders an interactive Adaptive Card with verified green status badges, blast radius breakdown, and an \"Approve Merge\" action button.</li>\n"
        "</ol>"
    )

    # Module 10
    html = html.replace(
        "<h2>Module 10: Enterprise Security, ALM Pipelines & Purview Data Loss Prevention (DLP)</h2>\n\n"
        "<h3>\x01</h3>\n"
        "<p>Calling TGS from inside Power Platform requires enterprise-grade security and automated lifecycle management:</p>\n\n"
        "<h4>\x01</h4>\n"
        "<p>To ensure sensitive corporate source code sent to TGS never leaks to unauthorized external connectors:</p>\n"
        "<ul>\n"
        "  <li>Classify the <code class=\"inline-code\">\x01</code> into the <strong>\x01</strong> alongside Microsoft Dataverse, Azure DevOps, and Office 365.</li>\n"
        "  <li>Isolate social and personal cloud connectors (Twitter, Google Drive, Personal Dropbox) in the <strong>\x01</strong> or <strong>\x01</strong> groups.</li>\n"
        "  <li>If a maker attempts to create a flow passing TGS audit data to an unapproved connector, Power Platform DLP immediately blocks flow activation.</li>\n"
        "</ul>\n\n"
        "<h4>\x01</h4>\n"
        "<ul>\n"
        "  <li>In multi-tier Canvas App calls, the user's Entra ID identity is preserved using OAuth 2.0 On-Behalf-Of (OBO) token exchange, ensuring user-level RBAC is respected inside TGS.</li>\n"
        "  <li>Real-time token revocation (CAE) terminates sessions within seconds if user risk changes or device compliance lapses.</li>\n"
        "</ul>\n\n"
        "<h4>\x01</h4>\n"
        "<ul>\n"
        "  <li>Manage automated multi-stage deployment environments (DEV &rarr; TEST &rarr; PROD) directly in the Power Platform Admin Center.</li>\n"
        "  <li>Automated PAC CLI solution export/import with pre-deployment automated regression test validation.</li>\n"
        "</ul>",
        "<h2>Module 10: Enterprise Security, ALM Pipelines &amp; Purview Data Loss Prevention (DLP)</h2>\n\n"
        "<h3>10.1 Enterprise ALM Pipelines &amp; Purview Data Loss Prevention (DLP)</h3>\n"
        "<p>Calling TGS from inside Power Platform requires enterprise-grade security and automated lifecycle management:</p>\n\n"
        "<h4>10.1.1 Microsoft Purview Data Loss Prevention (DLP) Policy Configuration</h4>\n"
        "<p>To ensure sensitive corporate source code sent to TGS never leaks to unauthorized external connectors:</p>\n"
        "<ul>\n"
        "  <li>Classify the <code class=\"inline-code\">TGSConnector</code> into the <strong>Business</strong> group alongside Microsoft Dataverse, Azure DevOps, and Office 365.</li>\n"
        "  <li>Isolate social and personal cloud connectors (Twitter, Google Drive, Personal Dropbox) in the <strong>Non-Business</strong> or <strong>Blocked</strong> groups.</li>\n"
        "  <li>If a maker attempts to create a flow passing TGS audit data to an unapproved connector, Power Platform DLP immediately blocks flow activation.</li>\n"
        "</ul>\n\n"
        "<h4>10.1.2 Continuous Access Evaluation &amp; Zero-Trust Authentication</h4>\n"
        "<ul>\n"
        "  <li>In multi-tier Canvas App calls, the user's Entra ID identity is preserved using OAuth 2.0 On-Behalf-Of (OBO) token exchange, ensuring user-level RBAC is respected inside TGS.</li>\n"
        "  <li>Real-time token revocation (CAE) terminates sessions within seconds if user risk changes or device compliance lapses.</li>\n"
        "</ul>\n\n"
        "<h4>10.1.3 Automated ALM Pipelines via GitHub Actions &amp; PAC CLI</h4>\n"
        "<ul>\n"
        "  <li>Manage automated multi-stage deployment environments (DEV &rarr; TEST &rarr; PROD) directly in the Power Platform Admin Center.</li>\n"
        "  <li>Automated PAC CLI solution export/import with pre-deployment automated regression test validation.</li>\n"
        "</ul>"
    )

    # General replacements for any LaTeX \to or tab artifacts
    html = html.replace("$\\to$", "&rarr;")
    html = html.replace("$\to$", "&rarr;")
    html = html.replace("$\t", "&rarr; ")

    # Check remaining \x01 count
    remaining_bad = html.count("\x01")
    print(f"Remaining \\x01 count after all fixes: {remaining_bad}")

    with open(HTML_PATH, "w", encoding="utf-8") as f:
        f.write(html)
    print("Successfully wrote updated HTML file.")

if __name__ == "__main__":
    fix_html()
