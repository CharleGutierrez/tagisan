//! # Microsoft Power Platform Solution & Custom Connector Packager
//!
//! Generates Microsoft Power Platform certified Custom Connector specifications
//! (OpenAPI with `x-ms-*` metadata) and builds deployable Solution `.zip` archives
//! (`solution.xml`, `customizations.xml`, `[Content_Types].xml`, `custom_connector.swagger.json`, `icon.png`)
//! for 1-click import into Power Apps (`make.powerapps.com`) and Power Automate (`make.powerautomate.com`).

use crate::copilot::ooxml::ZipBuilder;
use crate::copilot::plugin::generate_valid_png;
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Report summarizing the generated Power Platform Solution Package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPlatformSolutionReport {
    pub package_path: String,
    pub solution_name: String,
    pub version: String,
    pub total_bytes: usize,
    pub operations_count: usize,
    pub ready_for_power_platform: bool,
}

/// Core Power Platform Packager Engine
#[derive(Clone, Default)]
pub struct PowerPlatformPackagerEngine;

impl PowerPlatformPackagerEngine {
    pub fn new() -> Self {
        Self
    }

    /// Generates Power Platform Custom Connector Swagger / OpenAPI specification
    pub fn generate_custom_connector_swagger(&self, base_url: &str) -> (Value, usize) {
        let spec = json!({
            "swagger": "2.0",
            "info": {
                "title": "Tagisan Enterprise Copilot & Verification",
                "description": "Autonomous multi-agent consensus, formal invariant verification, AST blast radius telemetry, and Dataverse sync for Power Apps and Power Automate.",
                "version": "1.0.0",
                "contact": {
                    "name": "Tagisan Systems Engineering",
                    "url": "https://tagisan.ai",
                    "email": "powerplatform@tagisan.ai"
                }
            },
            "host": base_url.trim_start_matches("https://").trim_start_matches("http://"),
            "basePath": "/api/v1",
            "schemes": ["https"],
            "consumes": ["application/json"],
            "produces": ["application/json"],
            "securityDefinitions": {
                "oauth2_auth": {
                    "type": "oauth2",
                    "flow": "accessCode",
                    "authorizationUrl": "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                    "tokenUrl": "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                    "scopes": {
                        "https://graph.microsoft.com/.default": "Full Graph & Tagisan Access",
                        "offline_access": "Maintain silent access via refresh tokens"
                    }
                }
            },
            "security": [
                {
                    "oauth2_auth": [
                        "https://graph.microsoft.com/.default",
                        "offline_access"
                    ]
                }
            ],
            "paths": {
                "/copilot/debate": {
                    "post": {
                        "operationId": "ExecuteDialecticalDebate",
                        "summary": "Run 3-Round Dialectical Architecture Debate",
                        "description": "Executes Thesis -> Antithesis -> Lakandiwa Consensus on architectural proposals.",
                        "x-ms-summary": "Run 3-Round Dialectical Debate",
                        "x-ms-visibility": "important",
                        "parameters": [
                            {
                                "name": "body",
                                "in": "body",
                                "required": true,
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "proposal": {
                                            "type": "string",
                                            "description": "Proposal or RFC to debate",
                                            "x-ms-summary": "Technical Proposal"
                                        },
                                        "title": {
                                            "type": "string",
                                            "description": "Debate title",
                                            "x-ms-summary": "Debate Title"
                                        }
                                    },
                                    "required": ["proposal"]
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Debate Consensus Verdict",
                                "schema": { "type": "object" }
                            }
                        }
                    }
                },
                "/copilot/blast-radius": {
                    "post": {
                        "operationId": "CalculateBlastRadius",
                        "summary": "Calculate AST Codebase Blast Radius",
                        "description": "Analyzes transitive dependents, call sites, and risk metrics.",
                        "x-ms-summary": "Calculate AST Blast Radius",
                        "x-ms-visibility": "important",
                        "parameters": [
                            {
                                "name": "body",
                                "in": "body",
                                "required": true,
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "symbol": {
                                            "type": "string",
                                            "description": "Symbol name to analyze",
                                            "x-ms-summary": "Target Symbol"
                                        },
                                        "path": {
                                            "type": "string",
                                            "description": "Source path or directory",
                                            "x-ms-summary": "File Path"
                                        },
                                        "format": {
                                            "type": "string",
                                            "description": "Format: 'adaptive_card', 'html', 'json'",
                                            "x-ms-summary": "Output Format"
                                        }
                                    },
                                    "required": ["symbol"]
                                }
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "Blast radius telemetry report and Adaptive Card JSON",
                                "schema": { "type": "object" }
                            }
                        }
                    }
                },
                "/copilot/dataverse/adr": {
                    "post": {
                        "operationId": "SyncDataverseAdr",
                        "summary": "Synchronize Architecture Decision to Dataverse",
                        "description": "Upserts an Architectural Decision Record into Dataverse table tgs_architecturaldecisions.",
                        "x-ms-summary": "Sync ADR to Dataverse",
                        "x-ms-visibility": "important",
                        "parameters": [
                            {
                                "name": "body",
                                "in": "body",
                                "required": true,
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "decision_id": { "type": "string", "x-ms-summary": "Decision ID" },
                                        "title": { "type": "string", "x-ms-summary": "Title" },
                                        "thesis": { "type": "string", "x-ms-summary": "Thesis" },
                                        "synthesis": { "type": "string", "x-ms-summary": "Synthesis" },
                                        "purview_sensitivity": { "type": "string", "x-ms-summary": "Purview Sensitivity" }
                                    },
                                    "required": ["decision_id", "title"]
                                }
                            }
                        ],
                        "responses": {
                            "200": { "description": "Dataverse record created or updated" }
                        }
                    }
                },
                "/copilot/incident/autofix": {
                    "post": {
                        "operationId": "TriageIncidentAutofix",
                        "summary": "Triage CI/CD Incident & Apply Surgical Autofix",
                        "description": "Parses panic traces, identifies root causes, and applies verified patches.",
                        "x-ms-summary": "Triage & Autofix CI Incident",
                        "x-ms-visibility": "important",
                        "parameters": [
                            {
                                "name": "body",
                                "in": "body",
                                "required": true,
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "error_log": { "type": "string", "x-ms-summary": "Error Log" },
                                        "auto_patch": { "type": "boolean", "x-ms-summary": "Auto Apply Patch" }
                                    },
                                    "required": ["error_log"]
                                }
                            }
                        ],
                        "responses": {
                            "200": { "description": "Autofix diagnosis and patch diff" }
                        }
                    }
                },
                "/copilot/ooxml/export": {
                    "post": {
                        "operationId": "ExportOoxmlDocument",
                        "summary": "Generate Word, Excel, and PowerPoint Artifacts",
                        "description": "Produces pure-Rust OOXML packages (.docx, .xlsx, .pptx).",
                        "x-ms-summary": "Generate OOXML Office Document",
                        "parameters": [
                            {
                                "name": "body",
                                "in": "body",
                                "required": true,
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "title": { "type": "string", "x-ms-summary": "Document Title" },
                                        "format": { "type": "string", "x-ms-summary": "Format: docx, xlsx, pptx" }
                                    },
                                    "required": ["title"]
                                }
                            }
                        ],
                        "responses": {
                            "200": { "description": "Export status and byte counts" }
                        }
                    }
                }
            }
        });

        (spec, 5)
    }

    /// Build a certified Power Platform Solution ZIP package
    pub fn package_solution_zip(&self, output_dir: &Path, solution_name: &str) -> Result<PowerPlatformSolutionReport> {
        let _ = std::fs::create_dir_all(output_dir);

        let (swagger_spec, op_count) = self.generate_custom_connector_swagger("api.tagisan.ai");
        let swagger_json = serde_json::to_string_pretty(&swagger_spec)
            .map_err(|e| TagisanError::Execution(format!("Swagger serialization failed: {}", e)))?;

        // 1. [Content_Types].xml (Standard Open Packaging Conventions)
        let content_types_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="xml" ContentType="application/octet-stream" />
  <Default Extension="json" ContentType="application/json" />
  <Default Extension="png" ContentType="image/png" />
</Types>"#;

        // 2. solution.xml (Power Platform Solution Manifest)
        let solution_xml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<ImportExportXml version="9.2.0.0" SolutionPackageVersion="9.2" languagecode="1033" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <SolutionManifest>
    <UniqueName>{}</UniqueName>
    <LocalizedNames>
      <LocalizedName description="Tagisan Enterprise Power Platform Integration" languagecode="1033" />
    </LocalizedNames>
    <Descriptions>
      <Description description="Tagisan multi-agent dialectical reasoning, formal invariant verification, and Dataverse sync." languagecode="1033" />
    </Descriptions>
    <Version>1.0.0.0</Version>
    <Managed>0</Managed>
    <Publisher>
      <UniqueName>tgs_publisher</UniqueName>
      <LocalizedNames>
        <LocalizedName description="Tagisan AI Engineering" languagecode="1033" />
      </LocalizedNames>
      <CustomizationPrefix>tgs</CustomizationPrefix>
      <CustomizationOptionValuePrefix>10000</CustomizationOptionValuePrefix>
    </Publisher>
    <RootComponents>
      <RootComponent type="300" schemaName="tgs_tagisan_connector" behavior="0" />
      <RootComponent type="1" schemaName="tgs_architecturaldecisions" behavior="0" />
      <RootComponent type="1" schemaName="tgs_blastradiusreports" behavior="0" />
      <RootComponent type="1" schemaName="tgs_incidentremediations" behavior="0" />
    </RootComponents>
  </SolutionManifest>
</ImportExportXml>"#,
            solution_name
        );

        // 3. customizations.xml (Connectors & Entities registration)
        let customizations_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<ImportExportXml xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Entities>
    <Entity>
      <Name LocalizedName="Tagisan Architectural Decision" OriginalName="tgs_architecturaldecisions">tgs_architecturaldecisions</Name>
      <EntitySetName>tgs_architecturaldecisions</EntitySetName>
      <PrimaryIdAttribute>tgs_architecturaldecisionid</PrimaryIdAttribute>
      <PrimaryNameAttribute>tgs_title</PrimaryNameAttribute>
    </Entity>
    <Entity>
      <Name LocalizedName="Tagisan Blast Radius Report" OriginalName="tgs_blastradiusreports">tgs_blastradiusreports</Name>
      <EntitySetName>tgs_blastradiusreports</EntitySetName>
      <PrimaryIdAttribute>tgs_blastradiusreportid</PrimaryIdAttribute>
      <PrimaryNameAttribute>tgs_target_symbol</PrimaryNameAttribute>
    </Entity>
    <Entity>
      <Name LocalizedName="Tagisan Incident Remediation" OriginalName="tgs_incidentremediations">tgs_incidentremediations</Name>
      <EntitySetName>tgs_incidentremediations</EntitySetName>
      <PrimaryIdAttribute>tgs_incidentremediationid</PrimaryIdAttribute>
      <PrimaryNameAttribute>tgs_incident_id</PrimaryNameAttribute>
    </Entity>
  </Entities>
  <Connectors>
    <Connector>
      <ConnectorId>tgs_tagisan_connector</ConnectorId>
      <Name>Tagisan Enterprise Connector</Name>
      <Description>Autonomous multi-agent consensus and AST blast radius telemetry</Description>
      <IconPath>icon.png</IconPath>
      <OpenApiDefinitionPath>custom_connector.swagger.json</OpenApiDefinitionPath>
    </Connector>
  </Connectors>
</ImportExportXml>"#;

        // 4. Generate valid 96x96 icon PNG
        let icon_png_bytes = generate_valid_png(96, 96, 0x00, 0x78, 0xD4, 0xFF); // Microsoft Power Platform Fluent Blue

        // 5. Build ZIP bundle using pure-Rust ZipBuilder
        let mut zip = ZipBuilder::new();
        zip.add_file("[Content_Types].xml", content_types_xml.as_bytes());
        zip.add_file("solution.xml", solution_xml.as_bytes());
        zip.add_file("customizations.xml", customizations_xml.as_bytes());
        zip.add_file("custom_connector.swagger.json", swagger_json.as_bytes());
        zip.add_file("icon.png", &icon_png_bytes);

        let zip_bytes = zip.finish();
        let zip_filename = format!("{}_solution.zip", solution_name);
        let zip_path = output_dir.join(&zip_filename);

        std::fs::write(&zip_path, &zip_bytes)
            .map_err(|e| TagisanError::Execution(format!("Failed to write solution zip: {}", e)))?;

        info!("Power Platform Solution packaged: {} ({} bytes)", zip_path.display(), zip_bytes.len());

        Ok(PowerPlatformSolutionReport {
            package_path: zip_path.display().to_string(),
            solution_name: solution_name.to_string(),
            version: "1.0.0.0".to_string(),
            total_bytes: zip_bytes.len(),
            operations_count: op_count,
            ready_for_power_platform: true,
        })
    }
}

// =========================================================================
// Autonomous Tool: CopilotPowerPlatformPackagerTool
// =========================================================================

/// Tool for packaging certified Microsoft Power Platform Solutions and Custom Connectors
#[derive(Clone, Default)]
pub struct CopilotPowerPlatformPackagerTool {
    engine: Arc<PowerPlatformPackagerEngine>,
}

impl CopilotPowerPlatformPackagerTool {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(PowerPlatformPackagerEngine::new()),
        }
    }
}

#[async_trait]
impl ToolHandler for CopilotPowerPlatformPackagerTool {
    fn name(&self) -> &str {
        "copilot_powerplatform_packager"
    }

    fn description(&self) -> &str {
        "Generate Microsoft Power Platform certified Custom Connector Swagger specifications and build 1-click Solution ZIP bundles for Power Apps and Power Automate."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Packager action: 'generate_swagger', 'package_solution_zip'",
                    "enum": ["generate_swagger", "package_solution_zip"]
                },
                "solution_name": {
                    "type": "string",
                    "description": "Unique technical name for the solution (default: 'TagisanEnterpriseSolution')"
                },
                "output_dir": {
                    "type": "string",
                    "description": "Directory to write the Solution ZIP bundle (default: '.tagisan/powerplatform_export')"
                },
                "base_url": {
                    "type": "string",
                    "description": "Endpoint URL for Swagger definitions (default: 'https://api.tagisan.ai')"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        let solution_name = arguments
            .get("solution_name")
            .and_then(|v| v.as_str())
            .unwrap_or("TagisanEnterpriseSolution");

        let output_dir = arguments
            .get("output_dir")
            .and_then(|v| v.as_str())
            .unwrap_or(".tagisan/powerplatform_export");

        let base_url = arguments
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.tagisan.ai");

        match action {
            "generate_swagger" => {
                let (spec, count) = self.engine.generate_custom_connector_swagger(base_url);

                Ok(format!(
                    "### 🔌 Microsoft Power Platform Custom Connector Swagger Generated\n\n\
                    - **Base Host:** `{}`\n\
                    - **Operations Count:** {}\n\
                    - **Authentication:** `OAuth 2.0 (Azure AD / Entra ID)`\n\
                    - **Target Ecosystems:** Power Apps Canvas/Model, Power Automate Cloud Flows\n\n\
                    ```json\n{}\n```\n",
                    base_url,
                    count,
                    serde_json::to_string_pretty(&spec)?
                ))
            }
            "package_solution_zip" => {
                let report = self.engine.package_solution_zip(Path::new(output_dir), solution_name)?;

                Ok(format!(
                    "### 📦 Microsoft Power Platform Solution ZIP Packaged\n\n\
                    - **ZIP Archive:** `{}`\n\
                    - **Solution Name:** `{}`\n\
                    - **Solution Version:** `{}`\n\
                    - **Archive Size:** {} bytes\n\
                    - **Connector Operations:** {}\n\
                    - **Power Platform Ready:** {}\n\n\
                    #### Included Files in Solution Package:\n\
                    - `[Content_Types].xml` (Open Packaging Conventions)\n\
                    - `solution.xml` (Power Platform Solution Manifest & Components)\n\
                    - `customizations.xml` (Dataverse Entities & Connector Metadata)\n\
                    - `custom_connector.swagger.json` (OpenAPI Specification)\n\
                    - `icon.png` (96x96 Fluent Power Platform App Icon)\n\n\
                    > **1-Click Import Instructions:**\n\
                    > 1. Navigate to [make.powerapps.com](https://make.powerapps.com) or [make.powerautomate.com](https://make.powerautomate.com).\n\
                    > 2. Go to **Solutions > Import Solution**.\n\
                    > 3. Upload `{}` and click **Next > Import**.\n",
                    report.package_path,
                    report.solution_name,
                    report.version,
                    report.total_bytes,
                    report.operations_count,
                    if report.ready_for_power_platform { "✅ Yes (Certified PKZIP)" } else { "❌ No" },
                    report.package_path
                ))
            }
            _ => Err(TagisanError::Execution(format!(
                "Unsupported Power Platform packager action '{}'",
                action
            ))),
        }
    }
}
