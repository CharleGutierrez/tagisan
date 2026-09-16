//! Power BI Tabular Model Definition Language (TMDL), DAX Semantic Modeling & TMSL XMLA Engine
//!
//! Subsystem 4: Microsoft Ecosystem Expansion for Tagisan (`tgs`).
//!
//! Exposes:
//! 1. Tabular Model Definition Language (TMDL) generator:
//!    - Generates compliant TMDL code for databases, tables, columns, relationships (1-to-many, active/inactive), and partitions.
//! 2. Native Tagisan Telemetry DAX Measures:
//!    - Generates DAX measures:
//!      `[Total Code Churn]`, `[Blast Radius Index]`, `[Invariant Pass Rate]`,
//!      `[Consensus Convergence Time]`, `[Carbon Savings (kg CO2e)]`, `[AI Cost per PR]`.
//! 3. TMDL Syntax & DAX Expression Validator:
//!    - Validates indentation, keywords, and DAX expression bracket balancing (`[`, `]`, `(`, `)`).
//! 4. Tabular Model Scripting Language (`TmslEngine`):
//!    - Generates Tabular Model Scripting Language (TMSL) JSON execution payloads (`createOrReplace`, `refresh`, `alter`)
//!      for direct automated deployment over Fabric/Power BI XMLA endpoints (`powerbi://api.powerbi.com`).
//!    - Formats XMLA SOAP envelopes and HTTP POST payloads for Power BI Premium / Fabric capacity endpoints.
//! 5. `CopilotPowerBiTool`:
//!    - Implements `ToolHandler` exposing these capabilities with full JSON schemas.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::Arc;

// =========================================================================
// 1. Data Models
// =========================================================================

/// TMDL Column Definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlColumn {
    pub name: String,
    pub data_type: String, // int64, string, dateTime, double, boolean
    pub format_string: Option<String>,
    pub summarize_by: Option<String>, // sum, count, average, none
    pub source_column: String,
    pub description: Option<String>,
}

/// TMDL Measure Definition with DAX Formula
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlMeasure {
    pub name: String,
    pub expression: String, // DAX formula
    pub format_string: Option<String>,
    pub display_folder: Option<String>,
    pub description: Option<String>,
}

/// TMDL Partition Definition (Power Query M expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlPartition {
    pub name: String,
    pub mode: String, // import, directQuery
    pub source_expression: String,
}

/// TMDL Table Definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlTable {
    pub name: String,
    pub lineage_tag: String,
    pub description: Option<String>,
    pub columns: Vec<TmdlColumn>,
    pub measures: Vec<TmdlMeasure>,
    pub partitions: Vec<TmdlPartition>,
}

/// TMDL Relationship Definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlRelationship {
    pub name: String,
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
    pub cardinality: String, // oneToMany, manyToOne, oneToOne
    pub cross_filtering_behavior: String, // bothDirections, singleDirection
    pub is_active: bool,
}

/// TMDL Semantic Database / Model Definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlDatabase {
    pub name: String,
    pub compatibility_level: u32,
    pub tables: Vec<TmdlTable>,
    pub relationships: Vec<TmdlRelationship>,
}

/// Validation Result for TMDL and DAX expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmdlValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub table_count: usize,
    pub column_count: usize,
    pub measure_count: usize,
    pub relationship_count: usize,
    pub summary: String,
}

/// Target Object Identifier in Tabular Model Scripting Language (TMSL)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TmslTargetObject {
    pub database: String,
    pub table: Option<String>,
    pub partition: Option<String>,
}

/// TMSL Refresh Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TmslRefreshType {
    Full,
    ClearValues,
    Calculate,
    DataOnly,
    Defragment,
    Add,
}

impl TmslRefreshType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::ClearValues => "clearValues",
            Self::Calculate => "calculate",
            Self::DataOnly => "dataOnly",
            Self::Defragment => "defragment",
            Self::Add => "add",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "clearvalues" => Self::ClearValues,
            "calculate" => Self::Calculate,
            "dataonly" => Self::DataOnly,
            "defragment" => Self::Defragment,
            "add" => Self::Add,
            _ => Self::Full,
        }
    }
}

/// Full XMLA Deployment Payload Package
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlaDeploymentPackage {
    pub endpoint_url: String,
    pub database_name: String,
    pub tmsl_command: Value,
    pub soap_envelope_xml: String,
    pub content_type: String,
    pub command_type: String,
}

// =========================================================================
// 2. TMSL Engine
// =========================================================================

/// Tabular Model Scripting Language (TMSL) Payload Synthesizer
#[derive(Clone, Default)]
pub struct TmslEngine;

impl TmslEngine {
    pub fn new() -> Self {
        Self
    }

    /// Converts a TmdlDatabase into a TMSL model JSON definition
    pub fn database_to_tmsl_model(database: &TmdlDatabase) -> Value {
        let mut tables = Vec::new();
        for table in &database.tables {
            let mut columns = Vec::new();
            for col in &table.columns {
                let mut col_obj = json!({
                    "name": col.name,
                    "dataType": col.data_type,
                    "sourceColumn": col.source_column
                });
                if let Some(ref fmt) = col.format_string {
                    col_obj["formatString"] = json!(fmt);
                }
                if let Some(ref sum) = col.summarize_by {
                    col_obj["summarizeBy"] = json!(sum);
                }
                if let Some(ref desc) = col.description {
                    col_obj["description"] = json!(desc);
                }
                columns.push(col_obj);
            }

            let mut measures = Vec::new();
            for m in &table.measures {
                let mut m_obj = json!({
                    "name": m.name,
                    "expression": m.expression
                });
                if let Some(ref fmt) = m.format_string {
                    m_obj["formatString"] = json!(fmt);
                }
                if let Some(ref folder) = m.display_folder {
                    m_obj["displayFolder"] = json!(folder);
                }
                if let Some(ref desc) = m.description {
                    m_obj["description"] = json!(desc);
                }
                measures.push(m_obj);
            }

            let mut partitions = Vec::new();
            for p in &table.partitions {
                partitions.push(json!({
                    "name": p.name,
                    "mode": p.mode,
                    "source": {
                        "type": "m",
                        "expression": p.source_expression
                    }
                }));
            }

            let mut table_obj = json!({
                "name": table.name,
                "lineageTag": table.lineage_tag,
                "columns": columns,
                "measures": measures,
                "partitions": partitions
            });
            if let Some(ref desc) = table.description {
                table_obj["description"] = json!(desc);
            }
            tables.push(table_obj);
        }

        let mut relationships = Vec::new();
        for rel in &database.relationships {
            relationships.push(json!({
                "name": rel.name,
                "fromTable": rel.from_table,
                "fromColumn": rel.from_column,
                "toTable": rel.to_table,
                "toColumn": rel.to_column,
                "cardinality": rel.cardinality,
                "crossFilteringBehavior": rel.cross_filtering_behavior,
                "isActive": rel.is_active
            }));
        }

        json!({
            "name": database.name,
            "compatibilityLevel": database.compatibility_level,
            "model": {
                "culture": "en-US",
                "tables": tables,
                "relationships": relationships
            }
        })
    }

    /// Generates TMSL `createOrReplace` execution payload for direct deployment over Fabric/Power BI XMLA endpoints
    pub fn generate_create_or_replace(&self, database: &TmdlDatabase) -> Value {
        let db_payload = Self::database_to_tmsl_model(database);
        json!({
            "createOrReplace": {
                "object": {
                    "database": database.name
                },
                "database": db_payload
            }
        })
    }

    /// Generates TMSL `refresh` execution payload for tables, partitions, or entire model
    pub fn generate_refresh(
        &self,
        database_name: &str,
        refresh_type: TmslRefreshType,
        objects: &[TmslTargetObject],
    ) -> Value {
        let refresh_objects: Vec<Value> = if objects.is_empty() {
            vec![json!({ "database": database_name })]
        } else {
            objects
                .iter()
                .map(|obj| {
                    let mut m = json!({ "database": obj.database });
                    if let Some(ref tbl) = obj.table {
                        m["table"] = json!(tbl);
                    }
                    if let Some(ref part) = obj.partition {
                        m["partition"] = json!(part);
                    }
                    m
                })
                .collect()
        };

        json!({
            "refresh": {
                "type": refresh_type.as_str(),
                "objects": refresh_objects
            }
        })
    }

    /// Generates TMSL `alter` execution payload to modify a specific table, measure, or database
    pub fn generate_alter_table(&self, database_name: &str, table: &TmdlTable) -> Value {
        let mut columns = Vec::new();
        for col in &table.columns {
            columns.push(json!({
                "name": col.name,
                "dataType": col.data_type,
                "sourceColumn": col.source_column
            }));
        }

        let mut measures = Vec::new();
        for m in &table.measures {
            measures.push(json!({
                "name": m.name,
                "expression": m.expression
            }));
        }

        json!({
            "alter": {
                "object": {
                    "database": database_name,
                    "table": table.name
                },
                "table": {
                    "name": table.name,
                    "columns": columns,
                    "measures": measures
                }
            }
        })
    }

    /// Builds the standard XMLA SOAP Execution Envelope for Fabric / Power BI Premium endpoints
    pub fn generate_xmla_envelope(
        &self,
        workspace_name: &str,
        database_name: &str,
        tmsl_json: &Value,
        command_type: &str,
    ) -> XmlaDeploymentPackage {
        let tmsl_str = serde_json::to_string(tmsl_json).unwrap_or_default();
        let endpoint_url = format!("powerbi://api.powerbi.com/v1.0/myorg/{}", workspace_name);

        let soap_envelope_xml = format!(
            "<Envelope xmlns=\"http://schemas.xmlsoap.org/soap/envelope/\">\n\
             \t<Header/>\n\
             \t<Body>\n\
             \t\t<Execute xmlns=\"urn:schemas-microsoft-com:xml-analysis\">\n\
             \t\t\t<Command>\n\
             \t\t\t\t<Statement>\n\
             \t\t\t\t\t{}\n\
             \t\t\t\t</Statement>\n\
             \t\t\t</Command>\n\
             \t\t\t<Properties>\n\
             \t\t\t\t<PropertyList>\n\
             \t\t\t\t\t<Catalog>{}</Catalog>\n\
             \t\t\t\t</PropertyList>\n\
             \t\t\t</Properties>\n\
             \t\t</Execute>\n\
             \t</Body>\n\
             </Envelope>",
            tmsl_str.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"),
            database_name
        );

        XmlaDeploymentPackage {
            endpoint_url,
            database_name: database_name.to_string(),
            tmsl_command: tmsl_json.clone(),
            soap_envelope_xml,
            content_type: "text/xml; charset=utf-8".to_string(),
            command_type: command_type.to_string(),
        }
    }
}

// =========================================================================
// 3. Power BI Engine (Existing Core + TMDL + TMSL)
// =========================================================================

/// Power BI TMDL & DAX Semantic Modeling Engine
#[derive(Clone, Default)]
pub struct PowerBiEngine {
    tmsl: TmslEngine,
}

impl PowerBiEngine {
    pub fn new() -> Self {
        Self {
            tmsl: TmslEngine::new(),
        }
    }

    pub fn tmsl_engine(&self) -> &TmslEngine {
        &self.tmsl
    }

    /// Generates native Tagisan Telemetry Semantic Model
    pub fn create_default_telemetry_model() -> TmdlDatabase {
        let git_commits_table = TmdlTable {
            name: "GitCommits".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000001".to_string(),
            description: Some("Git commits telemetry recorded during autonomous agent execution.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "CommitSha".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "CommitSha".to_string(),
                    description: Some("Git commit SHA hash".to_string()),
                },
                TmdlColumn {
                    name: "LinesAdded".to_string(),
                    data_type: "int64".to_string(),
                    format_string: Some("#,##0".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "LinesAdded".to_string(),
                    description: Some("Total lines inserted".to_string()),
                },
                TmdlColumn {
                    name: "LinesDeleted".to_string(),
                    data_type: "int64".to_string(),
                    format_string: Some("#,##0".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "LinesDeleted".to_string(),
                    description: Some("Total lines removed".to_string()),
                },
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "GitCommits".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_GitCommits = Source{[Schema=\"dbo\",Item=\"GitCommits\"]}[Data]\nin\n\tdbo_GitCommits".to_string(),
            }],
        };

        let symbol_changes_table = TmdlTable {
            name: "SymbolChanges".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000002".to_string(),
            description: Some("AST symbols modified and their transitive caller blast radius.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "SymbolId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "SymbolId".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "CommitSha".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "CommitSha".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "TransitiveCallersCount".to_string(),
                    data_type: "int64".to_string(),
                    format_string: Some("#,##0".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "TransitiveCallersCount".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "RiskWeight".to_string(),
                    data_type: "double".to_string(),
                    format_string: Some("0.00".to_string()),
                    summarize_by: Some("average".to_string()),
                    source_column: "RiskWeight".to_string(),
                    description: None,
                },
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "SymbolChanges".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_SymbolChanges = Source{[Schema=\"dbo\",Item=\"SymbolChanges\"]}[Data]\nin\n\tdbo_SymbolChanges".to_string(),
            }],
        };

        let invariant_checks_table = TmdlTable {
            name: "InvariantChecks".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000003".to_string(),
            description: Some("Formal invariant checks evaluated via Z3 SMT and Kani.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "CheckId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "CheckId".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "CommitSha".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "CommitSha".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "Status".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "Status".to_string(),
                    description: Some("Passed or Failed".to_string()),
                },
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "InvariantChecks".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_InvariantChecks = Source{[Schema=\"dbo\",Item=\"InvariantChecks\"]}[Data]\nin\n\tdbo_InvariantChecks".to_string(),
            }],
        };

        let debate_rounds_table = TmdlTable {
            name: "DebateRounds".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000004".to_string(),
            description: Some("Multi-agent dialectical debate convergence metrics.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "DebateId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "DebateId".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "CommitSha".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "CommitSha".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "ConvergenceTimeMs".to_string(),
                    data_type: "int64".to_string(),
                    format_string: Some("#,##0".to_string()),
                    summarize_by: Some("average".to_string()),
                    source_column: "ConvergenceTimeMs".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "AiCostUsd".to_string(),
                    data_type: "double".to_string(),
                    format_string: Some("$#,##0.000".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "AiCostUsd".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "CarbonGrams".to_string(),
                    data_type: "double".to_string(),
                    format_string: Some("0.00".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "CarbonGrams".to_string(),
                    description: None,
                },
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "DebateRounds".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_DebateRounds = Source{[Schema=\"dbo\",Item=\"DebateRounds\"]}[Data]\nin\n\tdbo_DebateRounds".to_string(),
            }],
        };

        let measures_table = TmdlTable {
            name: "TelemetryMeasures".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000005".to_string(),
            description: Some("Core KPI DAX measures for Tagisan enterprise telemetry.".to_string()),
            columns: vec![],
            measures: vec![
                TmdlMeasure {
                    name: "Total Code Churn".to_string(),
                    expression: "SUM(GitCommits[LinesAdded]) + SUM(GitCommits[LinesDeleted])".to_string(),
                    format_string: Some("#,##0".to_string()),
                    display_folder: Some("Engineering KPIs".to_string()),
                    description: Some("Total code lines inserted and deleted across all commits.".to_string()),
                },
                TmdlMeasure {
                    name: "Blast Radius Index".to_string(),
                    expression: "AVERAGE(SymbolChanges[TransitiveCallersCount]) * AVERAGE(SymbolChanges[RiskWeight])".to_string(),
                    format_string: Some("0.00".to_string()),
                    display_folder: Some("Risk Analysis".to_string()),
                    description: Some("Transitive caller blast radius weighted by symbol criticality.".to_string()),
                },
                TmdlMeasure {
                    name: "Invariant Pass Rate".to_string(),
                    expression: "DIVIDE(CALCULATE(COUNTROWS(InvariantChecks), InvariantChecks[Status] = \"Passed\"), COUNTROWS(InvariantChecks), 1.0)".to_string(),
                    format_string: Some("0.0%".to_string()),
                    display_folder: Some("Verification".to_string()),
                    description: Some("Percentage of formal invariant checks that passed Z3 verification.".to_string()),
                },
                TmdlMeasure {
                    name: "Consensus Convergence Time".to_string(),
                    expression: "AVERAGE(DebateRounds[ConvergenceTimeMs]) / 1000.0".to_string(),
                    format_string: Some("0.00s".to_string()),
                    display_folder: Some("Debate Swarm".to_string()),
                    description: Some("Average seconds required for multi-agent debate to reach consensus.".to_string()),
                },
                TmdlMeasure {
                    name: "Carbon Savings (kg CO2e)".to_string(),
                    expression: "(SUM(GitCommits[LinesAdded]) * 0.05) - (SUM(DebateRounds[CarbonGrams]) / 1000.0)".to_string(),
                    format_string: Some("0.00".to_string()),
                    display_folder: Some("Sustainability".to_string()),
                    description: Some("Net carbon footprint saved through automated defect prevention.".to_string()),
                },
                TmdlMeasure {
                    name: "AI Cost per PR".to_string(),
                    expression: "DIVIDE(SUM(DebateRounds[AiCostUsd]), DISTINCTCOUNT(GitCommits[CommitSha]), 0)".to_string(),
                    format_string: Some("$#,##0.00".to_string()),
                    display_folder: Some("FinOps".to_string()),
                    description: Some("Total LLM inference expenditure divided by pull request count.".to_string()),
                },
            ],
            partitions: vec![TmdlPartition {
                name: "TelemetryMeasures".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = #table({\"Dummy\"}, {{1}})\nin\n\tSource".to_string(),
            }],
        };

        let relationships = vec![
            TmdlRelationship {
                name: "rel_git_symbols".to_string(),
                from_table: "SymbolChanges".to_string(),
                from_column: "CommitSha".to_string(),
                to_table: "GitCommits".to_string(),
                to_column: "CommitSha".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
            TmdlRelationship {
                name: "rel_git_invariants".to_string(),
                from_table: "InvariantChecks".to_string(),
                from_column: "CommitSha".to_string(),
                to_table: "GitCommits".to_string(),
                to_column: "CommitSha".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
            TmdlRelationship {
                name: "rel_git_debates".to_string(),
                from_table: "DebateRounds".to_string(),
                from_column: "CommitSha".to_string(),
                to_table: "GitCommits".to_string(),
                to_column: "CommitSha".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
        ];

        TmdlDatabase {
            name: "TagisanTelemetryDb".to_string(),
            compatibility_level: 1600,
            tables: vec![
                git_commits_table,
                symbol_changes_table,
                invariant_checks_table,
                debate_rounds_table,
                measures_table,
            ],
            relationships,
        }
    }

    /// Generates pure Tabular Model Definition Language (TMDL) output for a complete model
    pub fn generate_tmdl(&self, database: &TmdlDatabase) -> String {
        let mut out = String::new();

        out.push_str(&format!("database {}\n", database.name));
        out.push_str(&format!("\tcompatibilityLevel: {}\n\n", database.compatibility_level));

        out.push_str("model Model\n");
        out.push_str("\tculture: en-US\n\n");

        for table in &database.tables {
            out.push_str(&format!("table {}\n", table.name));
            out.push_str(&format!("\tlineageTag: {}\n", table.lineage_tag));
            if let Some(ref desc) = table.description {
                out.push_str(&format!("\tdescription: \"{}\"\n", desc.replace('"', "\\\"")));
            }
            out.push('\n');

            for col in &table.columns {
                out.push_str(&format!("\tcolumn {}\n", col.name));
                out.push_str(&format!("\t\tdataType: {}\n", col.data_type));
                if let Some(ref fmt) = col.format_string {
                    out.push_str(&format!("\t\tformatString: {}\n", fmt));
                }
                if let Some(ref sum) = col.summarize_by {
                    out.push_str(&format!("\t\tsummarizeBy: {}\n", sum));
                }
                out.push_str(&format!("\t\tsourceColumn: {}\n", col.source_column));
                if let Some(ref desc) = col.description {
                    out.push_str(&format!("\t\tdescription: \"{}\"\n", desc.replace('"', "\\\"")));
                }
                out.push('\n');
            }

            for measure in &table.measures {
                out.push_str(&format!("\tmeasure '{}' = {}\n", measure.name, measure.expression));
                if let Some(ref fmt) = measure.format_string {
                    out.push_str(&format!("\t\tformatString: {}\n", fmt));
                }
                if let Some(ref folder) = measure.display_folder {
                    out.push_str(&format!("\t\tdisplayFolder: \"{}\"\n", folder));
                }
                if let Some(ref desc) = measure.description {
                    out.push_str(&format!("\t\tdescription: \"{}\"\n", desc.replace('"', "\\\"")));
                }
                out.push('\n');
            }

            for part in &table.partitions {
                out.push_str(&format!("\tpartition {} = m\n", part.name));
                out.push_str(&format!("\t\tmode: {}\n", part.mode));
                out.push_str("\t\tsource =\n");
                for line in part.source_expression.lines() {
                    out.push_str(&format!("\t\t\t{}\n", line));
                }
                out.push('\n');
            }
        }

        for rel in &database.relationships {
            out.push_str(&format!("relationship {}\n", rel.name));
            out.push_str(&format!("\tfromColumn: {}.{}\n", rel.from_table, rel.from_column));
            out.push_str(&format!("\ttoColumn: {}.{}\n", rel.to_table, rel.to_column));
            out.push_str(&format!("\tcardinality: {}\n", rel.cardinality));
            out.push_str(&format!("\tcrossFilteringBehavior: {}\n", rel.cross_filtering_behavior));
            out.push_str(&format!("\tisActive: {}\n\n", rel.is_active));
        }

        out
    }

    /// Validates TMDL text syntax, indentation consistency, and DAX bracket balancing
    pub fn validate_tmdl(&self, tmdl_code: &str) -> TmdlValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut table_count = 0;
        let mut column_count = 0;
        let mut measure_count = 0;
        let mut relationship_count = 0;

        let lines: Vec<&str> = tmdl_code.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("table ") {
                table_count += 1;
            } else if trimmed.starts_with("column ") {
                column_count += 1;
            } else if trimmed.starts_with("measure ") {
                measure_count += 1;
                if let Some(eq_idx) = trimmed.find('=') {
                    let dax_expr = &trimmed[eq_idx + 1..];
                    Self::validate_dax_brackets(dax_expr, line_num, &mut errors);
                }
            } else if trimmed.starts_with("relationship ") {
                relationship_count += 1;
            }

            if !line.starts_with('\t') && !line.starts_with("  ") {
                let first_word = trimmed.split_whitespace().next().unwrap_or("");
                let valid_roots = ["model", "database", "table", "relationship", "role", "createOrReplace", "///"];
                if !valid_roots.contains(&first_word) {
                    warnings.push(format!("Line {line_num}: Root-level identifier '{first_word}' is not a standard TMDL root keyword."));
                }
            }
        }

        if table_count == 0 {
            errors.push("TMDL document does not contain any table declarations.".to_string());
        }

        let is_valid = errors.is_empty();
        let summary = format!(
            "TMDL Validation: {} (Tables: {}, Columns: {}, Measures: {}, Relationships: {}, Errors: {}, Warnings: {})",
            if is_valid { "VALID" } else { "INVALID" },
            table_count,
            column_count,
            measure_count,
            relationship_count,
            errors.len(),
            warnings.len()
        );

        TmdlValidationResult {
            is_valid,
            errors,
            warnings,
            table_count,
            column_count,
            measure_count,
            relationship_count,
            summary,
        }
    }

    /// Verifies that all parenthesis (), square brackets [], and quotes in a DAX expression are balanced
    pub fn validate_dax_brackets(dax_expr: &str, line_num: usize, errors: &mut Vec<String>) {
        let mut paren_count: i32 = 0;
        let mut bracket_count: i32 = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        let chars: Vec<char> = dax_expr.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
            } else if ch == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
            } else if !in_single_quote && !in_double_quote {
                if ch == '(' {
                    paren_count += 1;
                } else if ch == ')' {
                    paren_count -= 1;
                    if paren_count < 0 {
                        errors.push(format!("Line {line_num}: DAX expression has unmatched closing parenthesis ')'"));
                    }
                } else if ch == '[' {
                    bracket_count += 1;
                } else if ch == ']' {
                    bracket_count -= 1;
                    if bracket_count < 0 {
                        errors.push(format!("Line {line_num}: DAX expression has unmatched closing square bracket ']'"));
                    }
                }
            }
            i += 1;
        }

        if paren_count > 0 {
            errors.push(format!("Line {line_num}: DAX expression has {} unclosed opening parenthesis '('", paren_count));
        }
        if bracket_count > 0 {
            errors.push(format!("Line {line_num}: DAX expression has {} unclosed opening square bracket '['", bracket_count));
        }
        if in_single_quote {
            errors.push(format!("Line {line_num}: DAX expression has unclosed single quote '''"));
        }
        if in_double_quote {
            errors.push(format!("Line {line_num}: DAX expression has unclosed double quote '\"'"));
        }
    }
}

// =========================================================================
// 4. CopilotPowerBiTool (ToolHandler Implementation)
// =========================================================================

/// Autonomous Tool exposing Power BI TMDL Generation, DAX Measures, Syntax Validation, and TMSL XMLA Payloads
#[derive(Clone, Default)]
pub struct CopilotPowerBiTool {
    engine: Arc<PowerBiEngine>,
}

impl CopilotPowerBiTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_engine(engine: PowerBiEngine) -> Self {
        Self {
            engine: Arc::new(engine),
        }
    }
}

#[async_trait]
impl ToolHandler for CopilotPowerBiTool {
    fn name(&self) -> &str {
        "copilot_powerbi"
    }

    fn description(&self) -> &str {
        "Power BI Tabular Model Definition Language (TMDL) & DAX Semantic Modeling Engine: generates TMDL database definitions, native Tagisan telemetry measures, validates TMDL/DAX syntax, and synthesizes TMSL execution payloads (createOrReplace, refresh, alter) over Fabric/Power BI XMLA endpoints."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "generate_tmdl",
                        "generate_measures",
                        "validate_tmdl",
                        "export_semantic_model",
                        "generate_tmsl",
                        "generate_xmla_envelope"
                    ],
                    "description": "The specific Power BI TMDL / TMSL capability to execute."
                },
                "database": {
                    "type": "object",
                    "description": "TmdlDatabase structure to serialize."
                },
                "tmdl_code": {
                    "type": "string",
                    "description": "TMDL code snippet or file content to validate."
                },
                "tmsl_type": {
                    "type": "string",
                    "enum": ["createOrReplace", "refresh", "alter"],
                    "description": "TMSL operation type for generate_tmsl (default: 'createOrReplace')."
                },
                "refresh_type": {
                    "type": "string",
                    "enum": ["full", "clearValues", "calculate", "dataOnly", "defragment"],
                    "description": "Refresh mode for TMSL refresh."
                },
                "workspace_name": {
                    "type": "string",
                    "description": "Fabric/Power BI workspace name for XMLA endpoint (default: 'TagisanEnterpriseWorkspace')."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|a| a.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        match action {
            "generate_tmdl" | "export_semantic_model" => {
                let db = if let Some(db_val) = arguments.get("database") {
                    serde_json::from_value::<TmdlDatabase>(db_val.clone())
                        .map_err(|e| TagisanError::Execution(format!("Invalid database schema: {e}")))?
                } else {
                    PowerBiEngine::create_default_telemetry_model()
                };

                let tmdl = self.engine.generate_tmdl(&db);
                Ok(tmdl)
            }
            "generate_measures" => {
                let default_model = PowerBiEngine::create_default_telemetry_model();
                let measures_table = default_model.tables.iter().find(|t| t.name == "TelemetryMeasures")
                    .ok_or_else(|| TagisanError::Execution("Measures table not found".to_string()))?;
                Ok(serde_json::to_string_pretty(&measures_table.measures)?)
            }
            "validate_tmdl" => {
                let tmdl_code = arguments
                    .get("tmdl_code")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing required parameter 'tmdl_code'".to_string()))?;

                let validation = self.engine.validate_tmdl(tmdl_code);
                Ok(serde_json::to_string_pretty(&validation)?)
            }
            "generate_tmsl" => {
                let tmsl_type = arguments.get("tmsl_type").and_then(|t| t.as_str()).unwrap_or("createOrReplace");
                let db = if let Some(db_val) = arguments.get("database") {
                    serde_json::from_value::<TmdlDatabase>(db_val.clone())
                        .map_err(|e| TagisanError::Execution(format!("Invalid database schema: {e}")))?
                } else {
                    PowerBiEngine::create_default_telemetry_model()
                };

                let tmsl_json = match tmsl_type {
                    "refresh" => {
                        let r_type_str = arguments.get("refresh_type").and_then(|r| r.as_str()).unwrap_or("full");
                        let r_type = TmslRefreshType::from_str(r_type_str);
                        self.engine.tmsl_engine().generate_refresh(&db.name, r_type, &[])
                    }
                    "alter" => {
                        let table = db.tables.first().ok_or_else(|| TagisanError::Execution("No tables in database".to_string()))?;
                        self.engine.tmsl_engine().generate_alter_table(&db.name, table)
                    }
                    _ => self.engine.tmsl_engine().generate_create_or_replace(&db),
                };

                Ok(serde_json::to_string_pretty(&tmsl_json)?)
            }
            "generate_xmla_envelope" => {
                let workspace = arguments.get("workspace_name").and_then(|w| w.as_str()).unwrap_or("TagisanEnterpriseWorkspace");
                let db = if let Some(db_val) = arguments.get("database") {
                    serde_json::from_value::<TmdlDatabase>(db_val.clone())
                        .map_err(|e| TagisanError::Execution(format!("Invalid database schema: {e}")))?
                } else {
                    PowerBiEngine::create_default_telemetry_model()
                };

                let tmsl_cmd = self.engine.tmsl_engine().generate_create_or_replace(&db);
                let pkg = self.engine.tmsl_engine().generate_xmla_envelope(workspace, &db.name, &tmsl_cmd, "createOrReplace");

                Ok(serde_json::to_string_pretty(&pkg)?)
            }
            _ => Err(TagisanError::Execution(format!(
                "Unsupported action '{action}' for copilot_powerbi"
            ))),
        }
    }
}
