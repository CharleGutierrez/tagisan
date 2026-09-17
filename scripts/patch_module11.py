#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Patch tagisan/scripts/regenerate_module11_scenarios.py to provide clear,
human-understandable In-Platform Triggers (Power Fx, Cloud Flows, Copilot Prompts)
alongside the TGS Sovereign CLI Commands.
"""

import re
from pathlib import Path

script_path = Path("tagisan/scripts/regenerate_module11_scenarios.py")
with open(script_path, "r", encoding="utf-8") as f:
    code = f.read()

# 1. Triggers List
triggers_code = '''# In-Platform Triggers (Power Fx, Cloud Flows, Copilot Prompts, Dataverse Plugins, PCF)
CATEGORY_IN_PLATFORM_TRIGGERS = [
    "Power Fx: Set(varAudit, TGSConnector.AuditPleading({ Doc: UploadedPdf.Value }))",
    "PAC CLI: pac solution import --path TagisanRtcCdms_solution.zip",
    "PCF Manifest: <property name=\\"astCallGraph\\" of-type=\\"SingleLine.Text\\" usage=\\"bound\\"/>",
    "Fluent UI React: <PrimaryButton text=\\"Audit via TGS\\" onClick={() => triggerTgsDebate()}/>",
    "TypeScript: public updateView(context: ComponentFramework.Context<IInputs>): void",
    "Flow Action: Call TGS HTTP Webhook (POST /api/v1/copilot/debate)",
    "Dataverse Trigger: When a row is added to 'tgs_pleadings' (Scope: Organization)",
    "Teams Action: Post Adaptive Card and wait for response (Target: Presiding Judge)",
    "Flow Expression: triggerOutputs()['headers']['X-TGS-Signature'] == hmacSha256(body, secret)",
    "Flow Exception Policy: PT30S exponential backoff (Count: 5) -> Dead-Letter Queue",
    "OData GET: /api/data/v9.2/tgs_pleadings?$filter=tgs_status eq 'PendingAudit'",
    "OData POST: /api/data/v9.2/tgs_architecturaldecisions { \\"tgs_title\\": \\"CIV-2026-0891 TRO Grant\\" }",
    "Dataverse Batch: POST /api/data/v9.2/tgs_codegraphs (Batch Ingestion of AST Nodes)",
    "Dataverse Event: OnUpdate of 'tgs_incident' (Severity eq 1) -> Auto-Dispatch Autofix",
    "HTTP Batch: POST /api/data/v9.2/$batch (Multipart/mixed changeset of 500 rows)",
    "Delta Token: /api/data/v9.2/tgs_pleadings?$deltatoken=a1b2c3d4e5f6",
    "Security Role: Branch Clerk of Court (User-Level Read/Write, BU-Level AppendTo)",
    "C# Plugin: Stage 20 (Pre-Operation) on Create of 'tgs_pleading' -> TgsPlugin.Execute()",
    "DAX Measure: ComplianceRate = DIVIDE(CALCULATE(COUNTROWS(Pleadings), Status=\\"Approved\\"), COUNTROWS(Pleadings))",
    "Power BI Push: POST https://api.powerbi.com/v1.0/myorg/datasets/{id}/rows",
    "Power BI REST: POST /v1.0/myorg/reports/{id}/ExportTo { \\"format\\": \\"PDF\\" }",
    "DAX RLS: [BranchCode] = LOOKUPVALUE(UserBranches[BranchCode], UserBranches[Email], USERPRINCIPALNAME())",
    "Fabric Direct Lake: Zero-ETL Delta Lake Parquet streaming to Power BI visualizer",
    "Delta Lake API: deltaTable.write().format(\\"delta\\").mode(\\"append\\").save(\\"abfss://onelake@...\\")",
    "Fabric Spark: %python mssparkutils.notebook.run(\\"Tagisan_Judicial_Audit_Batch\\", 90)",
    "Copilot Plugin: declarativeAgent.json -> actions: [ { \\"id\\": \\"AuditLegalPleading\\" } ]",
    "Topic Phrase: User prompts \\"Copilot, audit the urgent TRO pleading in case CIV-2026-0891\\"",
    "Knowledge Grounding: Dataverse Table 'tgs_pleadings' & Supreme Court Reports Annotated (SCRA)",
    "WebSocket Stream: wss://directline.botframework.com/v3/directline/conversations/{id}/stream",
    "Dynamic Chaining: Copilot autonomously plans: ParsePDF -> VerifyRule7 -> CalculateDeadlines",
    "MSAL Auth: PublicClientApplication.AcquireTokenWithDeviceCode([\\"api://tagisan-gateway/Consensus.Execute\\"])",
    "OBO Exchange: ConfidentialClientApplication.AcquireTokenOnBehalfOf(userAssertion)",
    "CAE Validation: claims={\\"access_token\\":{\\"capolids\\\":{\\"values\\\":[\\"court-strict-location\\"]}}}",
    "Windows WAM: Silent token acquisition via WebAuthenticationCoreManager broker",
    "Purview MIP: SetSensitivityLabel(\\"Supreme Court - Highly Confidential / In Camera\\")",
    "DLP Policy: Tagisan Custom Connector assigned to 'Business' group; HTTP endpoints restricted",
    "AgentShield NER: Regex & ML token replacement: [Victim/Minor Name] -> 'AAA'",
    "Office 365 API: /api/v1.0/{tenant}/activity/feed/subscriptions/content?contentType=Audit.General",
    "Teams Webhook: POST /webhook/v2/incident-warroom { \\"@type\\": \\"MessageCard\\", \\"themeColor\\": \\"FF0000\\" }",
    "Teams Manifest: composeExtensions: [ { \\"id\\": \\"searchTgsPleadings\\", \\"type\\": \\"query\\" } ]",
    "Viva Goals API: PATCH /api/v1/goals/{id}/check_ins { \\"value\\": 100.0, \\"note\\": \\"0 procedural dismissals\\" }",
    "Graph Insights: GET /v1.0/me/insights/used?$filter=ResourceVisualization/Type eq 'CaseDocket'",
    "DevOps REST: POST /_apis/wit/workitems/$Bug?api-version=7.1 [ { \\"op\\": \\"add\\", \\"path\\": \\"/fields/System.Title\\", ... } ]",
    "WIQL Query: SELECT [System.Id] FROM WorkItems WHERE [System.WorkItemType] = 'Incident' AND [State] = 'Active'",
    "Azure Repos: Branch Policy requires status check 'Tagisan-AST-Consensus' to pass before merge",
    "IcM Incident: Alert Severity 1 -> TGS correlates call stack with git commit log in 400ms",
    "Word Online: Populate Word template with incident timeline, root cause, and action items",
    "PAC CLI: pac solution import --path TagisanRtcCdms_Upgrade.zip --stage-and-upgrade",
    "PowerShell CLI: pac auth create --url https://court.crm.dynamics.com; pac solution export",
    "GitHub Actions: .github/workflows/deploy-powerplatform.yml: on: [push] -> pac-action-runner",
]
'''

if "CATEGORY_IN_PLATFORM_TRIGGERS = [" not in code:
    code = code.replace("CATEGORIES = [", triggers_code + "\nCATEGORIES = [")

# 2. Update generate_scenario_row
old_trigger_block = """    # Trigger command syntax tailored to surface
    trigger_cmd = f\"{cat_cmd} --scenario {scen_id_str.lower()} --domain {domain['domain_id'].lower()}\""""

new_trigger_block = """    # Authentic In-Platform Trigger Expression & TGS Engine Command
    in_platform_trigger = CATEGORY_IN_PLATFORM_TRIGGERS[cat_idx]
    
    # Formatted Markdown and HTML triggers
    trigger_md = f"**In-Platform:** `{in_platform_trigger}`<br>**TGS Engine:** `{cat_cmd}`"
    trigger_html = (
        f"<div style=\\"font-size:7pt; margin-bottom:4px;\\"><strong style=\\"color:#0284c7;\\">In-Platform:</strong> <code style=\\"background:#e0f2fe; color:#0369a1; padding:1px 4px; border-radius:3px;\\">{in_platform_trigger}</code></div>"
        f"<div style=\\"font-size:7pt;\\"><strong style=\\"color:#7c3aed;\\">TGS Engine:</strong> <span class=\\"scen-cmd\\">{cat_cmd}</span></div>"
    )"""

if old_trigger_block in code:
    code = code.replace(old_trigger_block, new_trigger_block)
    code = code.replace('"trigger": trigger_cmd,', '"trigger_md": trigger_md,\n        "trigger_html": trigger_html,')

# 3. Update table headers
code = code.replace(
    'md_output.append("| ID | Operational Scenario Title | In-Platform Trigger Command | Real-World Action, TGS Verification & Expected In-Platform Result | Rollback & Cryptographic Proof |\\n")',
    'md_output.append("| ID | Operational Scenario Title | In-Platform Trigger & Sovereign TGS Command | Real-World Action, TGS Verification & In-Platform Result | Rollback & Cryptographic Proof |\\n")'
)
code = code.replace(
    'html_output.append("<th style=\\"width: 8%;\\">ID</th><th style=\\"width: 25%;\\">Operational Scenario Title</th><th style=\\"width: 25%;\\">In-Platform Trigger Command</th><th style=\\"width: 27%;\\">Real-World Action, TGS Verification &amp; In-Platform Result</th><th style=\\"width: 15%;\\\">Rollback &amp; Proof</th>\\n")',
    'html_output.append("<th style=\\"width: 8%;\\">ID</th><th style=\\"width: 23%;\\">Operational Scenario Title</th><th style=\\"width: 27%;\\">In-Platform Trigger &amp; TGS Engine Command</th><th style=\\"width: 27%;\\">Real-World Action, TGS Verification &amp; In-Platform Result</th><th style=\\"width: 15%;\\\">Rollback &amp; Proof</th>\\n")'
)

# 4. Update row additions
code = code.replace(
    "f\"| **{row['id']}** | {row['title']} | `{row['trigger']}` | {row['action_md']} | {row['proof_md']} |\\n\"",
    "f\"| **{row['id']}** | {row['title']} | {row['trigger_md']} | {row['action_md']} | {row['proof_md']} |\\n\""
)
code = code.replace(
    'html_output.append(f"<td><span class=\\"scen-cmd\\">{row[\'trigger\']}</span></td>\\n")',
    'html_output.append(f"<td>{row[\'trigger_html\']}</td>\\n")'
)

with open(script_path, "w", encoding="utf-8") as f:
    f.write(code)

print("Successfully patched tagisan/scripts/regenerate_module11_scenarios.py!")
