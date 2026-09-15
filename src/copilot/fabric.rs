//! Microsoft Fabric & Power BI DAX Semantic Layer Integration Engine
//!
//! Connects Tagisan swarms directly to enterprise data in Microsoft Fabric OneLake,
//! executing authenticated DAX queries against Power BI Semantic Models and querying
//! Delta Lake parquet tables via Fabric REST APIs.

use crate::copilot::graph::GraphClient;
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

/// Microsoft Fabric Workspace item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricWorkspace {
    pub id: String,
    pub name: String,
    pub capacity_id: String,
    pub type_name: String,
}

/// OneLake Delta Lake table representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneLakeTable {
    pub table_name: String,
    pub format: String,
    pub location_uri: String,
    pub column_count: usize,
    pub row_count_estimate: u64,
}

/// DAX query execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaxQueryRequest {
    pub dataset_id: String,
    pub query: String,
}

/// DAX query execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaxQueryResult {
    pub dataset_id: String,
    pub row_count: usize,
    pub columns: Vec<String>,
    pub rows: Vec<Value>,
    pub execution_duration_ms: u64,
}

/// Core Fabric and Power BI DAX engine
pub struct FabricEngine {
    client: Arc<GraphClient>,
}

impl FabricEngine {
    pub fn new(client: Arc<GraphClient>) -> Self {
        Self { client }
    }

    /// Executes an authenticated DAX query against a target Power BI dataset
    pub async fn execute_dax_query(
        &self,
        dataset_id: &str,
        dax_query: &str,
    ) -> Result<DaxQueryResult> {
        let start = std::time::Instant::now();

        if self.client.is_mock() {
            // High-fidelity enterprise mock response (e.g. Court Docket Statistics or Financial KPIs)
            let columns = vec![
                "BranchName".to_string(),
                "CaseClassification".to_string(),
                "TotalCasesFiled".to_string(),
                "LegalFeesCollectedPHP".to_string(),
                "ClearanceTurnaroundDays".to_string(),
            ];

            let rows = vec![
                json!({
                    "BranchName": "Branch 101 (Civil)",
                    "CaseClassification": "CV",
                    "TotalCasesFiled": 420,
                    "LegalFeesCollectedPHP": 2840500.50,
                    "ClearanceTurnaroundDays": 1.2
                }),
                json!({
                    "BranchName": "Branch 102 (Special Commercial)",
                    "CaseClassification": "SP",
                    "TotalCasesFiled": 185,
                    "LegalFeesCollectedPHP": 5120800.00,
                    "ClearanceTurnaroundDays": 1.8
                }),
                json!({
                    "BranchName": "Branch 103 (Family Court)",
                    "CaseClassification": "FC",
                    "TotalCasesFiled": 310,
                    "LegalFeesCollectedPHP": 945200.00,
                    "ClearanceTurnaroundDays": 0.9
                }),
            ];

            let duration_ms = start.elapsed().as_millis() as u64;

            Ok(DaxQueryResult {
                dataset_id: dataset_id.to_string(),
                row_count: rows.len(),
                columns,
                rows,
                execution_duration_ms: duration_ms,
            })
        } else {
            // Live Power BI REST endpoint: https://api.powerbi.com/v1.0/myorg/datasets/{datasetId}/executeQueries
            let payload = json!({
                "queries": [{ "query": dax_query }],
                "serializerSettings": { "includeNulls": true }
            });

            let endpoint = format!("https://api.powerbi.com/v1.0/myorg/datasets/{}/executeQueries", dataset_id);
            let resp = self.client.post(&endpoint, &payload).await?;

            let mut columns = Vec::new();
            let mut rows = Vec::new();

            if let Some(results) = resp.get("results").and_then(|r| r.as_array()) {
                if let Some(first) = results.first() {
                    if let Some(tables) = first.get("tables").and_then(|t| t.as_array()) {
                        if let Some(tbl) = tables.first() {
                            if let Some(rows_arr) = tbl.get("rows").and_then(|r| r.as_array()) {
                                if let Some(first_row) = rows_arr.first().and_then(|fr| fr.as_object()) {
                                    columns = first_row.keys().cloned().collect();
                                }
                                rows = rows_arr.clone();
                            }
                        }
                    }
                }
            }

            let duration_ms = start.elapsed().as_millis() as u64;

            Ok(DaxQueryResult {
                dataset_id: dataset_id.to_string(),
                row_count: rows.len(),
                columns,
                rows,
                execution_duration_ms: duration_ms,
            })
        }
    }

    /// Discovers OneLake Delta Lake tables in target Fabric Lakehouse
    pub async fn list_lakehouse_tables(
        &self,
        workspace_id: &str,
        lakehouse_id: &str,
    ) -> Result<Vec<OneLakeTable>> {
        if self.client.is_mock() {
            Ok(vec![
                OneLakeTable {
                    table_name: "dim_court_branches".to_string(),
                    format: "Delta".to_string(),
                    location_uri: format!("abfss://{}@onelake.dfs.fabric.microsoft.com/{}/Tables/dim_court_branches", workspace_id, lakehouse_id),
                    column_count: 8,
                    row_count_estimate: 54,
                },
                OneLakeTable {
                    table_name: "fact_case_filings".to_string(),
                    format: "Delta".to_string(),
                    location_uri: format!("abfss://{}@onelake.dfs.fabric.microsoft.com/{}/Tables/fact_case_filings", workspace_id, lakehouse_id),
                    column_count: 24,
                    row_count_estimate: 142850,
                },
                OneLakeTable {
                    table_name: "fact_legal_fees_rule141".to_string(),
                    format: "Delta".to_string(),
                    location_uri: format!("abfss://{}@onelake.dfs.fabric.microsoft.com/{}/Tables/fact_legal_fees_rule141", workspace_id, lakehouse_id),
                    column_count: 14,
                    row_count_estimate: 189200,
                },
            ])
        } else {
            let endpoint = format!(
                "https://api.fabric.microsoft.com/v1/workspaces/{}/lakehouses/{}/tables",
                workspace_id, lakehouse_id
            );
            let resp = self.client.get(&endpoint).await?;
            let items = resp.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();

            let mut tables = Vec::new();
            for item in items {
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("table").to_string();
                let fmt = item.get("format").and_then(|v| v.as_str()).unwrap_or("Delta").to_string();
                let loc = item.get("location").and_then(|v| v.as_str()).unwrap_or("").to_string();

                tables.push(OneLakeTable {
                    table_name: name,
                    format: fmt,
                    location_uri: loc,
                    column_count: 12,
                    row_count_estimate: 1000,
                });
            }
            Ok(tables)
        }
    }
}

// =========================================================================
// CopilotFabricQueryTool (copilot_fabric_query)
// =========================================================================

/// Autonomous tool for executing DAX queries against Power BI Semantic Models and Fabric Lakehouses
#[derive(Clone)]
pub struct CopilotFabricQueryTool {
    engine: Arc<FabricEngine>,
}

impl Default for CopilotFabricQueryTool {
    fn default() -> Self {
        Self {
            engine: Arc::new(FabricEngine::new(Arc::new(GraphClient::mock()))),
        }
    }
}

impl CopilotFabricQueryTool {
    pub fn new(engine: Arc<FabricEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl ToolHandler for CopilotFabricQueryTool {
    fn name(&self) -> &str {
        "copilot_fabric_query"
    }

    fn description(&self) -> &str {
        "Query Microsoft Fabric OneLake Delta Lake tables and execute authenticated DAX queries against Power BI Semantic Models."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["execute_dax", "list_tables"],
                    "description": "Operation: 'execute_dax' or 'list_tables'"
                },
                "dataset_id": {
                    "type": "string",
                    "description": "Power BI Semantic Model / Dataset GUID for DAX execution"
                },
                "dax_query": {
                    "type": "string",
                    "description": "DAX query expression (e.g. 'EVALUATE SUMMARIZECOLUMNS(...)')"
                },
                "workspace_id": {
                    "type": "string",
                    "description": "Fabric Workspace GUID"
                },
                "lakehouse_id": {
                    "type": "string",
                    "description": "Fabric Lakehouse GUID"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let op = arguments
            .get("operation")
            .or_else(|| arguments.get("action"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing 'operation'".to_string()))?;

        match op {
            "execute_dax" => {
                let dataset_id = arguments
                    .get("dataset_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ds-rtcocc-cdms-prod-01");

                let query = arguments
                    .get("dax_query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("EVALUATE SUMMARIZECOLUMNS('DimBranch'[BranchName], \"TotalCases\", [Total Cases Filed])");

                let result = self.engine.execute_dax_query(dataset_id, query).await?;

                Ok(format!(
                    "### 📊 Power BI DAX Query Execution Result\n\n\
                    - **Dataset Target:** `{}`\n\
                    - **Rows Returned:** {}\n\
                    - **Columns:** {}\n\
                    - **Execution Latency:** {} ms\n\n\
                    #### Tabular Result Matrix:\n```json\n{}\n```\n",
                    result.dataset_id,
                    result.row_count,
                    result.columns.join(", "),
                    result.execution_duration_ms,
                    serde_json::to_string_pretty(&result.rows)?
                ))
            }
            "list_tables" => {
                let ws_id = arguments.get("workspace_id").and_then(|v| v.as_str()).unwrap_or("ws-judiciary-01");
                let lh_id = arguments.get("lakehouse_id").and_then(|v| v.as_str()).unwrap_or("lh-court-dockets-01");

                let tables = self.engine.list_lakehouse_tables(ws_id, lh_id).await?;

                Ok(format!(
                    "### 🌊 Microsoft Fabric OneLake Tables Discovered\n\n\
                    - **Workspace:** `{}`\n\
                    - **Lakehouse:** `{}`\n\
                    - **Tables Found:** {}\n\n\
                    #### Delta Table Catalog:\n{}\n",
                    ws_id,
                    lh_id,
                    tables.len(),
                    tables
                        .iter()
                        .map(|t| format!("- **{}** (Format: `{}`) | ~{} rows | Location: `{}`", t.table_name, t.format, t.row_count_estimate, t.location_uri))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            _ => Err(TagisanError::Execution(format!("Unsupported Fabric operation '{}'", op))),
        }
    }
}
