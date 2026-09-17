#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Tagisan MS Power Platform MasterClass Course: Line-by-Line Prose Restorer
Restores all corrupted instructional text, headings, code tags, and navigation paths
that previously showed up as unreadable 'little boxes' (ASCII 0x01 tofu characters).
"""

import os
import sys

HTML_PATH = os.path.join(os.path.dirname(__file__), "..", "docs", "TGS_MS_POWER_PLATFORM_MASTERCLASS_COURSE.html")

# Map of 1-indexed line numbers to their authentic, human-understandable HTML replacement
REPLACEMENTS = {
    247: "<h3>1.1 The Sovereign Paradigm: Invoking TGS Inside Microsoft Power Platform</h3>\n",
    248: "<p>In modern Global 2000 enterprises, software engineering operates at the critical intersection of developer toolchains and low-code business operations. Mission-critical business processes, compliance approvals, financial transactions, and operational incident war-rooms reside within the <strong>Microsoft Power Platform</strong> (Power Apps, Power Automate, Copilot Studio, and Power BI).</p>\n",
    250: "<p>However, low-code platforms traditionally lack deterministic mathematical verification, dialectical multi-agent consensus, and deep abstract syntax tree (AST) code analysis. <strong>Tagisan (<code>tgs</code>) Subsystem 5</strong> bridges this divide by enabling low-code makers, professional developers, and enterprise cloud architects to <strong>call Tagisan directly from inside Microsoft Power Platform tools</strong>.</p>\n",
    254: "  <li><strong>Low-Code Makers</strong> in Canvas Apps trigger real-time Hegelian dialectical debates on code and architectural proposals via concise Power Fx formulas.</li>\n",
    255: "  <li><strong>Enterprise SREs</strong> in Power Automate automate pull request audits, blast-radius mitigation, and incident escalations with sub-second TGS evaluations.</li>\n",
    256: "  <li><strong>Enterprise Architects</strong> in Microsoft Dataverse enforce transaction-level gates using C# Plugins and Custom APIs that query TGS before database commit.</li>\n",
    257: "  <li><strong>AI Operators</strong> in Microsoft Copilot Studio invoke TGS tools via Declarative AI Plugins, generating conversational responses strictly grounded in verified architectural decision records (ADRs).</li>\n",
    258: "  <li><strong>Compliance Auditors</strong> on Power Pages sovereign portals verify Ed25519 asymmetric cryptographic receipts issued by TGS for every automated change.</li>\n",
    261: "<h3>1.2 In-Platform Calling Topologies &amp; The 6 Invocation Modes</h3>\n",
    264: "  <li><strong>Power Apps Canvas Studio (Power Fx)</strong>: Makers write declarative formulas invoking <code class=\"inline-code\">TGSConnector.ExecuteDebate(...)</code> or <code class=\"inline-code\">TGSConnector.AnalyzeAST(...)</code>, populating in-memory collections (<code class=\"inline-code\">ClearCollect(colTgsNodes, ...)</code>) and driving reactive galleries.</li>\n",
    265: "  <li><strong>Power Automate Cloud Flows</strong>: Flow triggers (GitHub PRs, Dataverse events, Jira webhooks) invoke the TGS Custom Connector action or authenticated HTTP endpoints with HMAC validation, parsing TGS response JSON and branching on consensus scores.</li>\n",
    266: "  <li><strong>Microsoft Copilot Studio AI Plugins</strong>: AI plugins bind TGS tools (<code class=\"inline-code\">CallTGSEngine</code>, <code class=\"inline-code\">AnalyzeCodeAST</code>, <code class=\"inline-code\">ResolveArchitecturalDebate</code>) to Copilot's Generative Orchestrator, enabling conversational prompts grounded in verified TGS audit trails.</li>\n",
    267: "  <li><strong>Microsoft Dataverse Native Extensions</strong>: Synchronous C# Plugins (<code class=\"inline-code\">IPlugin</code>), Custom APIs, and Virtual Tables invoke TGS REST endpoints via <code class=\"inline-code\">HttpClient</code> using Entra ID service principal tokens to gate transactions before database commit.</li>\n",
    268: "  <li><strong>Power Pages Compliance Portals</strong>: External sovereign portals execute portal Web API JavaScript and Liquid templates calling TGS endpoints to verify cryptographic signatures and display live compliance attestations.</li>\n",
    269: "  <li><strong>Power BI &amp; Microsoft Fabric OneLake</strong>: Power Query M scripts call TGS REST endpoints (<code class=\"inline-code\">Json.Document(Web.Contents(...))</code>), while Direct Lake semantic models read real-time Parquet Delta Lake tables with zero ETL delay.</li>\n",
    402: "  <li><strong>Zero Plaintext Secrets</strong>: Power Platform never stores raw passwords. All invocations leverage Microsoft Entra ID Managed Identities, Continuous Access Evaluation (CAE), or Azure Workload Identity Federation (WIF).</li>\n",
    403: "  <li><strong>Dialectical Verification</strong>: High blast-radius code modifications are blocked from reaching production Dataverse tables unless a 5-stage Hegelian multi-agent debate reaches an approved consensus score.</li>\n",
    404: "  <li><strong>Cryptographic Receipt Ledger</strong>: Every execution emits an Ed25519 digital signature verified against sovereign public keys, establishing non-repudiable audit trails.</li>\n",
    409: "<h3>2.1 Developer Toolchain Installation &amp; Prerequisites</h3>\n",
    458: "<h3>2.2 Microsoft Entra ID App Registration for Calling TGS</h3>\n",
    459: "<p>To enable secure service-to-service communication between Power Platform and the TGS API Gateway, configure an Entra ID Application Registration using Azure CLI (<code class=\"inline-code\">az ad</code>):</p>\n",
    532: "<tr><td class=\"line-num\">62</td><td class=\"line-code\"><span style=\"color: #A6E22E\">echo</span><span style=\"color: #F8F8F2\"> </span><span style=\"color: #E6DB74\">\"  Client Secret : ****************************************\"</span></td></tr>\n",
    538: "<h3>2.3 Hybrid Connectivity: On-Premises Data Gateway &amp; Azure VNet Gateways</h3>\n",
    541: "  <li><strong>On-Premises Data Gateway (OPDG)</strong>: Install the OPDG cluster on a dedicated Linux/Windows host with outbound HTTPS (port 443) to the Power Platform cloud. The gateway proxies inbound Custom Connector requests directly to <code class=\"inline-code\">http://localhost:8080</code> without opening public inbound firewall ports.</li>\n",
    542: "  <li><strong>Azure VNet Data Gateway</strong>: For Azure-hosted containerized TGS clusters, deploy an Azure VNet Data Gateway in the delegated subnet to route Power Platform traffic over private IP backbones with zero public ingress.</li>\n",
    545: "<h3>2.4 PAC CLI Authentication Handshake</h3>\n",
    615: "<h3>3.1 Certified Custom Connectors: The Bridge for Calling TGS</h3>\n",
    616: "<p>Power Apps, Power Automate, and Copilot Studio interact with external REST services through <strong>Custom Connectors</strong>. A production-grade connector wraps the Tagisan REST Gateway using an OpenAPI 3.0 (Swagger) specification annotated with Microsoft vendor extensions (<code class=\"inline-code\">x-ms-*</code>).</p>\n",
    727: "<h3>3.2 Step-by-Step Maker Portal Configuration &amp; Live Test</h3>\n",
    729: "  <li>Sign in to the <strong>Power Apps Maker Portal</strong> (<code class=\"inline-code\">https://make.powerapps.com</code>).</li>\n",
    730: "  <li>In the left navigation, select <strong>Dataverse</strong> &rarr; <strong>Custom Connectors</strong> &rarr; <strong>New custom connector</strong> &rarr; <strong>Import an OpenAPI file</strong>.</li>\n",
    731: "  <li>In <strong>General information</strong>, set Host to <code class=\"inline-code\">api.tagisan.internal</code> and Base URL to <code class=\"inline-code\">/v1</code>.</li>\n",
    732: "  <li>In <strong>Security</strong>, select <strong>OAuth 2.0</strong> with <strong>Azure Active Directory</strong>:</li>\n",
    735: "  <li>Resource URL: <code class=\"inline-code\">https://api.tagisan.internal</code>.</li>\n",
    736: "  <li>Scope: <code class=\"inline-code\">api://tagisan.internal/Consensus.Execute</code>.</li>\n",
    737: "  <li>In <strong>Test</strong>, create a new connection and execute <code class=\"inline-code\">ExecuteDialecticalDebate</code> with a sample payload.</li>\n",
    738: "  <li>Verify the HTTP <code class=\"inline-code\">200 OK</code> response returning the synthesized verdict, confidence score, and Ed25519 signature proof.</li>\n",
    827: "<h3>4.1 Solution Packaging Engine &amp; Power Apps Canvas Studio</h3>\n",
    828: "<p>Enterprise governance requires all low-code components, Custom Connectors, and PCF widgets to be bundled into managed, versioned Solution packages (<code class=\"inline-code\">.zip</code>) containing <code class=\"inline-code\">solution.xml</code> and <code class=\"inline-code\">customizations.xml</code>. Tagisan automates this assembly, ensuring seamless deployment across DEV, TEST, and PROD environments.</p>\n",
    830: "<h3>4.2 Calling TGS Inside Canvas Studio via Power Fx</h3>\n",
    831: "<p>Citizen developers and SRE engineers invoke TGS directly from Canvas App button clicks, screen transitions, and timer ticks using declarative <strong>Power Fx formulas</strong>.</p>\n",
    836: "  <li>In the <strong>Data</strong> pane, click <strong>Add data</strong> and select the <code class=\"inline-code\">TGSConnector</code>.</li>\n",
    837: "  <li>Add a Multi-line Text Input (<code class=\"inline-code\">txtCodeInput</code>), a Slider (<code class=\"inline-code\">sldRiskLimit</code>), an \"Execute Audit\" Button (<code class=\"inline-code\">btnExecuteAudit</code>), and a Gallery (<code class=\"inline-code\">galArguments</code>).</li>\n",
    838: "  <li>Set the <code class=\"inline-code\">OnSelect</code> property of <code class=\"inline-code\">btnExecuteAudit</code> to call TGS:</li>\n",
    944: "<h3>4.3 Binding TGS Results to Visual Gallery &amp; Gauges</h3>\n",
    946: "  <li><strong>AST Nodes Gallery</strong>: Set <code class=\"inline-code\">Items</code> of <code class=\"inline-code\">galArguments</code> to <code class=\"inline-code\">colThesisPoints</code>, binding <code class=\"inline-code\">Title</code> to <code class=\"inline-code\">ThisItem.ArgumentTitle</code> and <code class=\"inline-code\">Subtitle</code> to <code class=\"inline-code\">ThisItem.EvidenceCodeSnippet</code>.</li>\n",
    947: "  <li><strong>Hegelian Confidence Gauge</strong>: Bind a circular SVG progress meter to <code class=\"inline-code\">varTgsResponse.synthesis.confidence_pct</code>.</li>\n",
    948: "  <li><strong>Cryptographic Stamp</strong>: Render a badge displaying <code class=\"inline-code\">Left(varCryptoProof, 16) &amp; \"...\"</code> with a verified green checkmark.</li>\n",
    1042: "<h3>5.1 Modern PCF Custom Controls with React 18 &amp; Fluent UI v9</h3>\n",
    1043: "<p>While Canvas Apps handle low-code forms, Model-Driven Apps and Dataverse standard entity forms require high-performance, responsive custom UI components. The <strong>Power Apps Component Framework (PCF)</strong> allows pro-developers to write TypeScript controls that call TGS endpoints directly and render complex interactive visualizations.</p>\n",
    1045: "<h3>5.2 Scaffolding &amp; Manifest Modeling</h3>\n",
    1062: "<p>Configure <code class=\"inline-code\">ControlManifest.Input.xml</code> to declare bound Dataverse columns and Web API capabilities:</p>\n",
    1097: "<h3>5.3 Calling TGS Inside <code>TagisanAstConsensusWidget.tsx</code></h3>\n",
    1098: "<p>The component uses React 18 and Fluent UI v9 (<code class=\"inline-code\">@fluentui/react-components</code>) to call TGS and render an interactive AST dependency tree:</p>\n",
    1358: "<p>Tagisan implements Georg Wilhelm Friedrich Hegel's dialectical triad (<strong>Thesis &rarr; Antithesis &rarr; Synthesis</strong>) as an autonomous multi-agent consensus loop. In Power Automate and PCF controls, this ensures that every code change is rigorously stress-tested before being merged or approved.</p>\n",
    1453: "<h3>6.1 Power Automate Cloud Flow Transpilation &amp; Calling TGS</h3>\n",
    1454: "<p>Tagisan can automatically transpile complex engineering pipelines into native Power Automate Cloud Flow JSON definitions (<code class=\"inline-code\">workflowDefinition.json</code>).</p>\n",
    1599: "<h3>6.2 Live Power Automate Flow Execution &amp; Run History</h3>\n",
    1685: "<h3>7.1 Calling TGS from Microsoft Dataverse: C# Plugins, Custom APIs &amp; Virtual Tables</h3>\n",
    1688: "<h4>7.1.1 Synchronous C# Plugin Execution Pipeline (<code>IPlugin</code>)</h4>\n",
    1814: "<h4>7.1.2 Custom API Declaration for Calling TGS</h4>\n",
    1815: "<p>Expose <code class=\"inline-code\">tgs_ExecuteAstAudit</code> directly as an unmanaged/managed Custom API in Dataverse, making it callable via OData v4 Web API (<code class=\"inline-code\">POST /api/data/v9.2/tgs_ExecuteAstAudit</code>).</p>\n",
    1817: "<h4>7.1.3 Virtual Table Data Provider</h4>\n",
    1820: "<h3>7.2 Power Pages Sovereign Compliance Portal</h3>\n",
    1825: "  <li>Portal Web API JavaScript calls <code class=\"inline-code\">tgs_VerifyReceipt</code> to validate signature authenticity in real time.</li>\n",
    1912: "<h3>8.1 Ingesting TGS Telemetry into Power BI via Power Query M</h3>\n",
    1987: "<h3>8.2 Microsoft Fabric OneLake Direct Lake SRE Observability</h3>\n",
    1988: "<p>Tagisan streams high-velocity telemetry into <strong>Microsoft Fabric OneLake</strong>:</p>\n",
    1990: "  <li><strong>Direct Lake Semantic Models</strong>: Power BI reports read Parquet Delta Lake files directly from OneLake storage with zero ETL latency and sub-second query speeds.</li>\n",
    1991: "  <li><strong>Real-Time SRE Metrics</strong>: SRE measures compute rolling MTTR, Hegelian consensus confidence percentiles (P50, P95, P99), and Purview DLP compliance scores:</li>\n",
    2111: "<h3>9.1 Microsoft Copilot Studio Declarative AI Plugins Calling TGS</h3>\n",
    2112: "<p>Tagisan acts as an autonomous knowledge and execution engine for <strong>Microsoft Copilot Studio</strong>:</p>\n",
    2114: "  <li><strong>OpenAPI 3.0 Tool Binding</strong>: Exposing Tagisan tools directly to Copilot using OpenAPI 3.0 plugin manifests (<code class=\"inline-code\">ai-plugin.json</code>).</li>\n",
    2115: "  <li><strong>Orchestrator Tool Calling</strong>: Binding <code class=\"inline-code\">CallTGSEngine</code>, <code class=\"inline-code\">AnalyzeCodeAST</code>, and <code class=\"inline-code\">ResolveArchitecturalDebate</code> to the Copilot Generative Orchestrator.</li>\n",
    2116: "  <li><strong>ADR Table Grounding</strong>: Grounding Copilot responses in verified Dataverse ADR tables to completely eliminate LLM hallucinations.</li>\n",
    2187: "<h3>9.2 Conversational Prompt Lifecycle Calling TGS Inside Copilot</h3>\n",
    2189: "  <li><strong>User Prompt</strong>: SRE Engineer asks: <em>\"Copilot, audit the checkout microservice refactoring proposal for high-risk AST dependencies using Tagisan.\"</em></li>\n",
    2190: "  <li><strong>Intent &amp; Tool Selection</strong>: Copilot identifies the intent and invokes the <code class=\"inline-code\">AnalyzeCodeAST</code> tool with the source repository URL.</li>\n",
    2191: "  <li><strong>TGS Autonomous Execution</strong>: TGS parses syntax trees, computes blast-radius score (3.2 / Low), and returns synthesized evidence.</li>\n",
    2192: "  <li><strong>Adaptive Card Grounded Output</strong>: Copilot renders an interactive Adaptive Card with verified green status badges, blast radius breakdown, and an \"Approve Merge\" action button.</li>\n",
    2275: "<h3>10.1 Enterprise ALM Pipelines &amp; Purview Data Loss Prevention (DLP)</h3>\n",
    2278: "<h4>10.1.1 Microsoft Purview Data Loss Prevention (DLP) Policy Configuration</h4>\n",
    2281: "  <li>Classify the <code class=\"inline-code\">TGSConnector</code> into the <strong>Business</strong> group alongside Microsoft Dataverse, Azure DevOps, and Office 365.</li>\n",
    2282: "  <li>Isolate social and personal cloud connectors (Twitter, Google Drive, Personal Dropbox) in the <strong>Non-Business</strong> or <strong>Blocked</strong> groups.</li>\n",
    2286: "<h4>10.1.2 Automated ALM via GitHub Actions &amp; PAC CLI</h4>\n",
    2292: "<h4>10.1.3 Continuous Security Attestation &amp; Rollback Circuit Breakers</h4>\n",
    2294: "  <li>Manage automated multi-stage deployment environments (DEV &rarr; TEST &rarr; PROD) directly in the Power Platform Admin Center.</li>\n"
}

def restore_masterclass():
    with open(HTML_PATH, "r", encoding="utf-8") as f:
        lines = f.readlines()

    print(f"Total lines in HTML: {len(lines)}")
    initial_bad_lines = sum(1 for l in lines if "\x01" in l)
    print(f"Initial bad lines containing \\x01: {initial_bad_lines}")

    replaced_count = 0
    for line_num, new_content in REPLACEMENTS.items():
        idx = line_num - 1
        if idx < len(lines):
            old_line = lines[idx]
            lines[idx] = new_content
            replaced_count += 1
        else:
            print(f"Warning: Line {line_num} out of range")

    print(f"Successfully applied {replaced_count} line replacements.")

    full_html = "".join(lines)
    # Replace any leftover LaTeX to symbols
    full_html = full_html.replace("$\\to$", "&rarr;")
    full_html = full_html.replace("$\to$", "&rarr;")
    full_html = full_html.replace("$\t", "&rarr; ")

    remaining_bad_chars = full_html.count("\x01")
    print(f"Remaining \\x01 characters in full HTML: {remaining_bad_chars}")

    if remaining_bad_chars > 0:
        print("Warning: Still found \\x01 in the file!")
        for idx, line in enumerate(full_html.splitlines()):
            if "\x01" in line:
                print(f"Line {idx+1}: {repr(line)}")
    else:
        print("PERFECT: Exactly ZERO \\x01 characters remain in the document!")

    with open(HTML_PATH, "w", encoding="utf-8") as f:
        f.write(full_html)
    print(f"Wrote {len(full_html)} characters to {HTML_PATH}")

if __name__ == "__main__":
    restore_masterclass()
