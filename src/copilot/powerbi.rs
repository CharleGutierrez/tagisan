//! Power BI Tabular Model Definition Language (TMDL) & DAX Semantic Modeling Engine
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
//! 4. `CopilotPowerBiTool`:
//!    - Implements `ToolHandler` exposing these capabilities with full JSON schemas.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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

// =========================================================================
// 2. Power BI Engine
// =========================================================================

/// Power BI TMDL & DAX Semantic Modeling Engine
#[derive(Clone, Default)]
pub struct PowerBiEngine;

impl PowerBiEngine {
    pub fn new() -> Self {
        Self
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
                    name: "ConvergenceTimeSeconds".to_string(),
                    data_type: "double".to_string(),
                    format_string: Some("0.0".to_string()),
                    summarize_by: Some("average".to_string()),
                    source_column: "ConvergenceTimeSeconds".to_string(),
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

        let npu_executions_table = TmdlTable {
            name: "NpuExecutions".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000005".to_string(),
            description: Some("Copilot+ PC NPU / DirectML offloaded inferences.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "ExecutionId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "ExecutionId".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "DirectML_FLOPs".to_string(),
                    data_type: "double".to_string(),
                    format_string: Some("#,##0".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "DirectML_FLOPs".to_string(),
                    description: None,
                },
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "NpuExecutions".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_NpuExecutions = Source{[Schema=\"dbo\",Item=\"NpuExecutions\"]}[Data]\nin\n\tdbo_NpuExecutions".to_string(),
            }],
        };

        let agent_invocations_table = TmdlTable {
            name: "AgentInvocations".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000006".to_string(),
            description: Some("Autonomous agent token consumption and cost.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "InvocationId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "InvocationId".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "PrId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "PrId".to_string(),
                    description: None,
                },
                TmdlColumn {
                    name: "CostUsd".to_string(),
                    data_type: "double".to_string(),
                    format_string: Some("$#,##0.0000".to_string()),
                    summarize_by: Some("sum".to_string()),
                    source_column: "CostUsd".to_string(),
                    description: None,
                },
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "AgentInvocations".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_AgentInvocations = Source{[Schema=\"dbo\",Item=\"AgentInvocations\"]}[Data]\nin\n\tdbo_AgentInvocations".to_string(),
            }],
        };

        let git_pull_requests_table = TmdlTable {
            name: "GitPullRequests".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000007".to_string(),
            description: Some("Pull Requests generated and reviewed by Tagisan.".to_string()),
            columns: vec![
                TmdlColumn {
                    name: "PrId".to_string(),
                    data_type: "string".to_string(),
                    format_string: None,
                    summarize_by: Some("none".to_string()),
                    source_column: "PrId".to_string(),
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
            ],
            measures: vec![],
            partitions: vec![TmdlPartition {
                name: "GitPullRequests".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Sql.Database(\"tagisan-sql.database.windows.net\", \"TelemetryDb\"),\n\tdbo_GitPullRequests = Source{[Schema=\"dbo\",Item=\"GitPullRequests\"]}[Data]\nin\n\tdbo_GitPullRequests".to_string(),
            }],
        };

        // Measures Table containing all 6 native Tagisan telemetry measures
        let telemetry_measures_table = TmdlTable {
            name: "TelemetryMeasures".to_string(),
            lineage_tag: "00000000-0000-0000-0000-000000000008".to_string(),
            description: Some("Key performance indicators and telemetry measures for Tagisan.".to_string()),
            columns: vec![],
            measures: vec![
                TmdlMeasure {
                    name: "Total Code Churn".to_string(),
                    expression: "SUM('GitCommits'[LinesAdded]) + SUM('GitCommits'[LinesDeleted])".to_string(),
                    format_string: Some("#,##0".to_string()),
                    display_folder: Some("Code Quality".to_string()),
                    description: Some("Total combined code churn in lines across commits.".to_string()),
                },
                TmdlMeasure {
                    name: "Blast Radius Index".to_string(),
                    expression: "AVERAGEX('SymbolChanges', 'SymbolChanges'[TransitiveCallersCount] * 'SymbolChanges'[RiskWeight])".to_string(),
                    format_string: Some("0.00".to_string()),
                    display_folder: Some("Architectural Blast".to_string()),
                    description: Some("Weighted blast radius score across affected symbols.".to_string()),
                },
                TmdlMeasure {
                    name: "Invariant Pass Rate".to_string(),
                    expression: "DIVIDE(CALCULATE(COUNTROWS('InvariantChecks'), 'InvariantChecks'[Status] = \"Passed\"), COUNTROWS('InvariantChecks'), 0)".to_string(),
                    format_string: Some("0.0%".to_string()),
                    display_folder: Some("Formal Invariants".to_string()),
                    description: Some("Proportion of formal invariant verification checks that succeeded.".to_string()),
                },
                TmdlMeasure {
                    name: "Consensus Convergence Time".to_string(),
                    expression: "AVERAGE('DebateRounds'[ConvergenceTimeSeconds])".to_string(),
                    format_string: Some("0.0s".to_string()),
                    display_folder: Some("Consensus & Debate".to_string()),
                    description: Some("Mean time required for dialectical multi-agent consensus convergence.".to_string()),
                },
                TmdlMeasure {
                    name: "Carbon Savings (kg CO2e)".to_string(),
                    expression: "SUM('NpuExecutions'[DirectML_FLOPs]) * 0.000000000045".to_string(),
                    format_string: Some("#,##0.000".to_string()),
                    display_folder: Some("Hardware & Carbon".to_string()),
                    description: Some("Estimated carbon footprint avoided by on-device NPU DirectML offloading.".to_string()),
                },
                TmdlMeasure {
                    name: "AI Cost per PR".to_string(),
                    expression: "DIVIDE(SUM('AgentInvocations'[CostUsd]), DISTINCTCOUNT('GitPullRequests'[PrId]), 0)".to_string(),
                    format_string: Some("$#,##0.00".to_string()),
                    display_folder: Some("Economics".to_string()),
                    description: Some("Mean LLM and compute cost expended per pull request merged.".to_string()),
                },
            ],
            partitions: vec![TmdlPartition {
                name: "TelemetryMeasures".to_string(),
                mode: "import".to_string(),
                source_expression: "let\n\tSource = Table.FromRows(Json.Document(Binary.Decompress(Binary.FromText(\"i44FAA==\", BinaryEncoding.Base64), Compression.Deflate)), let _t = ((type nullable text) meta [Serialized.Text = true]) in type table [Column1 = _t])\nin\n\tSource".to_string(),
            }],
        };

        // Relationships
        let relationships = vec![
            TmdlRelationship {
                name: "Rel_GitCommits_SymbolChanges".to_string(),
                from_table: "SymbolChanges".to_string(),
                from_column: "CommitSha".to_string(),
                to_table: "GitCommits".to_string(),
                to_column: "CommitSha".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
            TmdlRelationship {
                name: "Rel_GitCommits_InvariantChecks".to_string(),
                from_table: "InvariantChecks".to_string(),
                from_column: "CommitSha".to_string(),
                to_table: "GitCommits".to_string(),
                to_column: "CommitSha".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
            TmdlRelationship {
                name: "Rel_GitCommits_GitPullRequests".to_string(),
                from_table: "GitPullRequests".to_string(),
                from_column: "CommitSha".to_string(),
                to_table: "GitCommits".to_string(),
                to_column: "CommitSha".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
            TmdlRelationship {
                name: "Rel_GitPullRequests_AgentInvocations".to_string(),
                from_table: "AgentInvocations".to_string(),
                from_column: "PrId".to_string(),
                to_table: "GitPullRequests".to_string(),
                to_column: "PrId".to_string(),
                cardinality: "manyToOne".to_string(),
                cross_filtering_behavior: "bothDirections".to_string(),
                is_active: true,
            },
        ];

        TmdlDatabase {
            name: "TagisanTelemetryModel".to_string(),
            compatibility_level: 1600,
            tables: vec![
                git_commits_table,
                symbol_changes_table,
                invariant_checks_table,
                debate_rounds_table,
                npu_executions_table,
                agent_invocations_table,
                git_pull_requests_table,
                telemetry_measures_table,
            ],
            relationships,
        }
    }

    /// Serializes a complete database model to TMDL format
    pub fn generate_tmdl(&self, database: &TmdlDatabase) -> String {
        let mut out = String::new();

        // Model Header
        out.push_str(&format!("model {}\n", database.name));
        out.push_str(&format!("\tcompatibilityLevel: {}\n\n", database.compatibility_level));

        // Tables
        for table in &database.tables {
            out.push_str(&format!("table {}\n", table.name));
            out.push_str(&format!("\tlineageTag: {}\n", table.lineage_tag));

            if let Some(ref desc) = table.description {
                out.push_str(&format!("\tdescription: \"{}\"\n", desc.replace('"', "\\\"")));
            }
            out.push('\n');

            // Columns
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

            // Measures
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

            // Partitions
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

        // Relationships
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

            // Count entities
            if trimmed.starts_with("table ") {
                table_count += 1;
            } else if trimmed.starts_with("column ") {
                column_count += 1;
            } else if trimmed.starts_with("measure ") {
                measure_count += 1;

                // Validate DAX measure expression bracket balance
                if let Some(eq_idx) = trimmed.find('=') {
                    let dax_expr = &trimmed[eq_idx + 1..];
                    Self::validate_dax_brackets(dax_expr, line_num, &mut errors);
                }
            } else if trimmed.starts_with("relationship ") {
                relationship_count += 1;
            }

            // Check for illegal or malformed keywords
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
// 3. CopilotPowerBiTool (ToolHandler Implementation)
// =========================================================================

/// Autonomous Tool exposing Power BI TMDL Generation, DAX Measures, and Syntax Validation
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
        "Power BI Tabular Model Definition Language (TMDL) & DAX Semantic Modeling Engine: generates TMDL database definitions, native Tagisan telemetry measures, and validates TMDL/DAX syntax."
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
                        "export_semantic_model"
                    ],
                    "description": "The specific Power BI TMDL capability to execute."
                },
                "database": {
                    "type": "object",
                    "description": "TmdlDatabase structure to serialize to TMDL format."
                },
                "tmdl_code": {
                    "type": "string",
                    "description": "TMDL code snippet or file content to validate."
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
            _ => Err(TagisanError::Execution(format!(
                "Unsupported action '{action}' for copilot_powerbi"
            ))),
        }
    }
}
