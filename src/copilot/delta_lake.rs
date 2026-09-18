//! Microsoft OneLake Delta Lake Parquet Engine
//!
//! Subsystem 4: Microsoft Advanced Systems for Tagisan (`tgs`).
//!
//! Direct synthesis and reading of Delta Lake transaction commit logs (`_delta_log/00000000000000000000.json`),
//! protocol, metadata, statistics (min/max/nullCount), and multi-version time-travel without Apache Spark,
//! with native OneLake ADLS Gen2 endpoint mapping (`https://onelake.dfs.fabric.microsoft.com/{workspace}/{item}`).

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// =========================================================================
// 1. Data Models & Delta Protocol Specifications
// =========================================================================

pub const ONELAKE_DFS_BASE_URL: &str = "https://onelake.dfs.fabric.microsoft.com";
pub const ONELAKE_BLOB_BASE_URL: &str = "https://onelake.blob.fabric.microsoft.com";
pub const DEFAULT_ENGINE_INFO: &str = "Tagisan-DeltaLake-1.0";

/// Commit Info entry in Delta transaction log
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaCommitInfo {
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    pub operation: String,
    pub operation_parameters: HashMap<String, Value>,
    pub engine_info: String,
    pub is_blind_append: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_version: Option<u64>,
}

/// Protocol specification action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaProtocol {
    pub min_reader_version: u32,
    pub min_writer_version: u32,
}

impl Default for DeltaProtocol {
    fn default() -> Self {
        Self {
            min_reader_version: 1,
            min_writer_version: 2,
        }
    }
}

/// Format specification inside metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaFormatSpec {
    pub provider: String,
    pub options: HashMap<String, String>,
}

impl Default for DeltaFormatSpec {
    fn default() -> Self {
        Self {
            provider: "parquet".to_string(),
            options: HashMap::new(),
        }
    }
}

/// MetaData action defining table schema & partitioning
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaMetaData {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub format: DeltaFormatSpec,
    pub schema_string: String,
    pub partition_columns: Vec<String>,
    pub created_time: i64,
    pub configuration: HashMap<String, String>,
}

/// File Statistics (stringified JSON inside Add action)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaFileStats {
    pub num_records: u64,
    pub min_values: HashMap<String, Value>,
    pub max_values: HashMap<String, Value>,
    pub null_count: HashMap<String, u64>,
}

/// Add action registering a Parquet data file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaAddAction {
    pub path: String,
    pub size: i64,
    pub modification_time: i64,
    pub data_change: bool,
    pub partition_values: HashMap<String, String>,
    pub stats: String,
}

/// Remove action tombstoning a Parquet data file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaRemoveAction {
    pub path: String,
    pub deletion_timestamp: i64,
    pub data_change: bool,
}

/// Single Action inside a Delta commit log
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaActionEnvelope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_info: Option<DeltaCommitInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<DeltaProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<DeltaMetaData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<DeltaAddAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<DeltaRemoveAction>,
}

/// Reconstructed Snapshot of a Delta Table at Version N
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaTableSnapshot {
    pub table_name: String,
    pub version: u64,
    pub active_files: Vec<DeltaAddAction>,
    pub total_records: u64,
    pub total_bytes: i64,
    pub partition_columns: Vec<String>,
    pub schema_string: String,
    pub commit_history: Vec<DeltaCommitInfo>,
}

/// OneLake Path & URI Mapping Specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OneLakePathMapping {
    pub workspace: String,
    pub item_name: String,
    pub item_type: String, // e.g. "Lakehouse"
    pub table_name: String,
    pub adls_https_url: String,
    pub blob_https_url: String,
    pub abfss_url: String,
    pub local_relative_path: String,
}

// =========================================================================
// 2. DeltaLakeEngine Implementation
// =========================================================================

/// Microsoft OneLake Delta Lake Engine
pub struct DeltaLakeEngine {
    table_name: String,
    commits: RwLock<HashMap<u64, Vec<DeltaActionEnvelope>>>,
    checkpoints: RwLock<HashMap<u64, Vec<DeltaActionEnvelope>>>,
    base_dir: Option<PathBuf>,
}

impl DeltaLakeEngine {
    /// Creates a new in-memory Delta table engine
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            commits: RwLock::new(HashMap::new()),
            checkpoints: RwLock::new(HashMap::new()),
            base_dir: None,
        }
    }

    /// Sets local filesystem root path for `_delta_log` persistence
    pub fn with_base_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(path.into());
        self
    }

    pub fn current_timestamp_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    /// Formats a version number as 20-digit zero-padded file name
    pub fn format_commit_filename(version: u64) -> String {
        format!("{version:020}.json")
    }

    /// Formats a checkpoint file name
    pub fn format_checkpoint_filename(version: u64) -> String {
        format!("{version:020}.checkpoint.json")
    }

    /// Initializes a Delta Lake table with initial Commit 0
    pub async fn create_table(
        &self,
        schema_fields: &[(&str, &str, bool)], // (name, type, nullable)
        partition_cols: &[&str],
        description: Option<&str>,
    ) -> Result<Vec<DeltaActionEnvelope>> {
        let now = Self::current_timestamp_ms();

        // Synthesize StructType JSON schema
        let fields_json: Vec<Value> = schema_fields
            .iter()
            .map(|(name, typ, nullable)| {
                json!({
                    "name": name,
                    "type": typ,
                    "nullable": nullable,
                    "metadata": {}
                })
            })
            .collect();

        let schema_json = json!({
            "type": "struct",
            "fields": fields_json
        });

        let schema_string = serde_json::to_string(&schema_json)
            .map_err(|e| TagisanError::Execution(format!("Schema serialization error: {e}")))?;

        let commit_info = DeltaCommitInfo {
            timestamp: now,
            user_id: Some("tagisan-copilot".to_string()),
            user_name: Some("Tagisan Systems Architect".to_string()),
            operation: "CREATE TABLE".to_string(),
            operation_parameters: {
                let mut m = HashMap::new();
                m.insert("isManaged".to_string(), json!("true"));
                m.insert("description".to_string(), json!(description));
                m
            },
            engine_info: DEFAULT_ENGINE_INFO.to_string(),
            is_blind_append: true,
            read_version: None,
        };

        let protocol = DeltaProtocol::default();

        let meta_data = DeltaMetaData {
            id: format!("delta-tbl-{}", blake3::hash(self.table_name.as_bytes()).to_hex()),
            name: Some(self.table_name.clone()),
            description: description.map(|d| d.to_string()),
            format: DeltaFormatSpec::default(),
            schema_string,
            partition_columns: partition_cols.iter().map(|s| s.to_string()).collect(),
            created_time: now,
            configuration: {
                let mut conf = HashMap::new();
                conf.insert("delta.minReaderVersion".to_string(), "1".to_string());
                conf.insert("delta.minWriterVersion".to_string(), "2".to_string());
                conf
            },
        };

        let actions = vec![
            DeltaActionEnvelope {
                commit_info: Some(commit_info),
                protocol: None,
                meta_data: None,
                add: None,
                remove: None,
            },
            DeltaActionEnvelope {
                commit_info: None,
                protocol: Some(protocol),
                meta_data: None,
                add: None,
                remove: None,
            },
            DeltaActionEnvelope {
                commit_info: None,
                protocol: None,
                meta_data: Some(meta_data),
                add: None,
                remove: None,
            },
        ];

        let mut commits = self.commits.write().await;
        commits.insert(0, actions.clone());

        // Write to local disk if base_dir is set
        if let Some(ref base) = self.base_dir {
            Self::write_commit_log_to_disk(base, 0, &actions)?;
        }

        Ok(actions)
    }

    /// Appends Parquet files and records to a new version commit
    pub async fn commit_transaction(
        &self,
        operation: &str,
        added_files: Vec<DeltaAddAction>,
        removed_files: Vec<DeltaRemoveAction>,
    ) -> Result<(u64, Vec<DeltaActionEnvelope>)> {
        let now = Self::current_timestamp_ms();

        let mut commits = self.commits.write().await;
        let next_version = commits.keys().max().map(|v| v + 1).unwrap_or(0);

        let mut actions = Vec::new();

        let commit_info = DeltaCommitInfo {
            timestamp: now,
            user_id: Some("tagisan-copilot".to_string()),
            user_name: Some("Tagisan Systems Architect".to_string()),
            operation: operation.to_string(),
            operation_parameters: {
                let mut m = HashMap::new();
                m.insert("numAddedFiles".to_string(), json!(added_files.len()));
                m.insert("numRemovedFiles".to_string(), json!(removed_files.len()));
                m
            },
            engine_info: DEFAULT_ENGINE_INFO.to_string(),
            is_blind_append: removed_files.is_empty(),
            read_version: if next_version > 0 { Some(next_version - 1) } else { None },
        };

        actions.push(DeltaActionEnvelope {
            commit_info: Some(commit_info),
            protocol: None,
            meta_data: None,
            add: None,
            remove: None,
        });

        for add in added_files {
            actions.push(DeltaActionEnvelope {
                commit_info: None,
                protocol: None,
                meta_data: None,
                add: Some(add),
                remove: None,
            });
        }

        for rem in removed_files {
            actions.push(DeltaActionEnvelope {
                commit_info: None,
                protocol: None,
                meta_data: None,
                add: None,
                remove: Some(rem),
            });
        }

        commits.insert(next_version, actions.clone());

        if let Some(ref base) = self.base_dir {
            Self::write_commit_log_to_disk(base, next_version, &actions)?;
        }

        Ok((next_version, actions))
    }

    /// Reconstructs the exact table snapshot at `target_version` using Time Travel
    pub async fn read_version(&self, target_version: u64) -> Result<DeltaTableSnapshot> {
        let commits = self.commits.read().await;
        if commits.is_empty() {
            return Err(TagisanError::Execution("Delta table has no commits".to_string()));
        }

        let max_ver = commits.keys().max().copied().unwrap_or(0);
        if target_version > max_ver {
            return Err(TagisanError::Execution(format!(
                "Time travel version {target_version} exceeds latest table version {max_ver}"
            )));
        }

        // Check if there is a checkpoint <= target_version
        let checkpoints = self.checkpoints.read().await;
        let mut active_files: HashMap<String, DeltaAddAction> = HashMap::new();
        let mut schema_string = String::new();
        let mut partition_columns = Vec::new();
        let mut commit_history = Vec::new();

        let start_version = if let Some(&chk_ver) = checkpoints.keys().filter(|&&v| v <= target_version).max() {
            if let Some(chk_actions) = checkpoints.get(&chk_ver) {
                for act in chk_actions {
                    if let Some(ref meta) = act.meta_data {
                        schema_string = meta.schema_string.clone();
                        partition_columns = meta.partition_columns.clone();
                    }
                    if let Some(ref add) = act.add {
                        active_files.insert(add.path.clone(), add.clone());
                    }
                }
            }
            chk_ver + 1
        } else {
            0
        };

        // Replay commits from start_version up to target_version
        for ver in start_version..=target_version {
            if let Some(actions) = commits.get(&ver) {
                for act in actions {
                    if let Some(ref ci) = act.commit_info {
                        commit_history.push(ci.clone());
                    }
                    if let Some(ref meta) = act.meta_data {
                        schema_string = meta.schema_string.clone();
                        partition_columns = meta.partition_columns.clone();
                    }
                    if let Some(ref add) = act.add {
                        active_files.insert(add.path.clone(), add.clone());
                    }
                    if let Some(ref rem) = act.remove {
                        active_files.remove(&rem.path);
                    }
                }
            }
        }

        let mut total_records = 0u64;
        let mut total_bytes = 0i64;
        for file in active_files.values() {
            total_bytes += file.size;
            if let Ok(stats) = serde_json::from_str::<DeltaFileStats>(&file.stats) {
                total_records += stats.num_records;
            }
        }

        let mut files_vec: Vec<DeltaAddAction> = active_files.into_values().collect();
        files_vec.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(DeltaTableSnapshot {
            table_name: self.table_name.clone(),
            version: target_version,
            active_files: files_vec,
            total_records,
            total_bytes,
            partition_columns,
            schema_string,
            commit_history,
        })
    }

    /// Compacts commit history up to `version` into a Delta Checkpoint
    pub async fn create_checkpoint(&self, version: u64) -> Result<String> {
        let snapshot = self.read_version(version).await?;

        let mut chk_actions = Vec::new();
        chk_actions.push(DeltaActionEnvelope {
            commit_info: None,
            protocol: Some(DeltaProtocol::default()),
            meta_data: None,
            add: None,
            remove: None,
        });

        chk_actions.push(DeltaActionEnvelope {
            commit_info: None,
            protocol: None,
            meta_data: Some(DeltaMetaData {
                id: format!("chk-{}", version),
                name: Some(self.table_name.clone()),
                description: Some(format!("Compacted checkpoint at version {version}")),
                format: DeltaFormatSpec::default(),
                schema_string: snapshot.schema_string.clone(),
                partition_columns: snapshot.partition_columns.clone(),
                created_time: Self::current_timestamp_ms(),
                configuration: HashMap::new(),
            }),
            add: None,
            remove: None,
        });

        for file in snapshot.active_files {
            chk_actions.push(DeltaActionEnvelope {
                commit_info: None,
                protocol: None,
                meta_data: None,
                add: Some(file),
                remove: None,
            });
        }

        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.insert(version, chk_actions.clone());

        let filename = Self::format_checkpoint_filename(version);
        if let Some(ref base) = self.base_dir {
            let log_dir = base.join("_delta_log");
            let _ = std::fs::create_dir_all(&log_dir);
            let mut content = String::new();
            for act in &chk_actions {
                content.push_str(&serde_json::to_string(act).map_err(|e| TagisanError::Execution(e.to_string()))?);
                content.push('\n');
            }
            let _ = std::fs::write(log_dir.join(&filename), content);
        }

        Ok(filename)
    }

    /// Maps OneLake ADLS Gen2, Blob, ABFSS, and local paths
    pub fn map_onelake_path(
        workspace: &str,
        lakehouse_name: &str,
        table_name: &str,
    ) -> OneLakePathMapping {
        let adls_https = format!(
            "{ONELAKE_DFS_BASE_URL}/{workspace}/{lakehouse_name}.Lakehouse/Tables/{table_name}"
        );
        let blob_https = format!(
            "{ONELAKE_BLOB_BASE_URL}/{workspace}/{lakehouse_name}.Lakehouse/Tables/{table_name}"
        );
        let abfss = format!(
            "abfss://{workspace}@onelake.dfs.fabric.microsoft.com/{lakehouse_name}.Lakehouse/Tables/{table_name}"
        );
        let local_rel = format!("{lakehouse_name}/Tables/{table_name}");

        OneLakePathMapping {
            workspace: workspace.to_string(),
            item_name: lakehouse_name.to_string(),
            item_type: "Lakehouse".to_string(),
            table_name: table_name.to_string(),
            adls_https_url: adls_https,
            blob_https_url: blob_https,
            abfss_url: abfss,
            local_relative_path: local_rel,
        }
    }

    /// Serializes commit action list to NDJSON format
    pub fn serialize_commit_ndjson(actions: &[DeltaActionEnvelope]) -> Result<String> {
        let mut lines = Vec::new();
        for act in actions {
            let line = serde_json::to_string(act)
                .map_err(|e| TagisanError::Execution(format!("NDJSON serialization error: {e}")))?;
            lines.push(line);
        }
        Ok(lines.join("\n"))
    }

    /// Writes commit JSON file to `_delta_log` directory
    fn write_commit_log_to_disk(
        base_dir: &Path,
        version: u64,
        actions: &[DeltaActionEnvelope],
    ) -> Result<()> {
        let log_dir = base_dir.join("_delta_log");
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| TagisanError::Execution(format!("Failed to create _delta_log directory: {e}")))?;

        let filename = Self::format_commit_filename(version);
        let file_path = log_dir.join(filename);
        let ndjson = Self::serialize_commit_ndjson(actions)?;

        std::fs::write(&file_path, ndjson)
            .map_err(|e| TagisanError::Execution(format!("Failed to write commit log {file_path:?}: {e}")))?;
        Ok(())
    }
}

// =========================================================================
// 3. CopilotDeltaLakeTool (ToolHandler)
// =========================================================================

/// Tool for Microsoft OneLake Delta Lake Parquet Engine & Time-Travel Versioning
#[derive(Clone)]
pub struct CopilotDeltaLakeTool {
    engine: Arc<DeltaLakeEngine>,
}

impl Default for CopilotDeltaLakeTool {
    fn default() -> Self {
        Self {
            engine: Arc::new(DeltaLakeEngine::new("default_telemetry")),
        }
    }
}

impl CopilotDeltaLakeTool {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ToolHandler for CopilotDeltaLakeTool {
    fn name(&self) -> &str {
        "copilot_delta_lake"
    }

    fn description(&self) -> &str {
        "Manages Microsoft OneLake Delta Lake tables, writes atomic transaction logs (_delta_log), generates Parquet metadata & statistics, executes time-travel version queries, and maps ADLS Gen2 Fabric endpoints."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action"],
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "create_table",
                        "commit_transaction",
                        "read_version",
                        "create_checkpoint",
                        "map_onelake_path",
                        "get_table_history"
                    ],
                    "description": "Delta Lake engine action to execute."
                },
                "table_name": {
                    "type": "string",
                    "description": "Delta table name."
                },
                "schema_fields": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "type": { "type": "string" },
                            "nullable": { "type": "boolean" }
                        }
                    },
                    "description": "Schema field definitions for create_table."
                },
                "partition_columns": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Partition column names."
                },
                "version": {
                    "type": "integer",
                    "description": "Target version for time-travel read or checkpoint."
                },
                "added_files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of DeltaAddAction parquet files to register."
                },
                "removed_files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of DeltaRemoveAction parquet files to remove."
                },
                "workspace": {
                    "type": "string",
                    "description": "Microsoft Fabric OneLake workspace name or GUID."
                },
                "lakehouse_name": {
                    "type": "string",
                    "description": "Microsoft Fabric Lakehouse container name."
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|a| a.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        match action {
            "create_table" => {
                let schema_fields: Vec<(&str, &str, bool)> = if let Some(arr) = arguments.get("schema_fields").and_then(|a| a.as_array()) {
                    let mut fields = Vec::new();
                    for item in arr {
                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("col");
                        let typ = item.get("type").and_then(|t| t.as_str()).unwrap_or("string");
                        let nullable = item.get("nullable").and_then(|b| b.as_bool()).unwrap_or(true);
                        fields.push((name, typ, nullable));
                    }
                    fields
                } else {
                    vec![
                        ("id", "long", false),
                        ("symbol", "string", false),
                        ("blast_score", "double", true),
                        ("timestamp", "timestamp", false),
                    ]
                };

                let partition_cols = ["date"];
                let actions = self
                    .engine
                    .create_table(&schema_fields, &partition_cols, Some("Tagisan Telemetry Table"))
                    .await?;

                let ndjson = DeltaLakeEngine::serialize_commit_ndjson(&actions)?;
                let result = json!({
                    "status": "table_created",
                    "version": 0,
                    "commit_file": DeltaLakeEngine::format_commit_filename(0),
                    "action_count": actions.len(),
                    "ndjson": ndjson
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
            "commit_transaction" => {
                let added: Vec<DeltaAddAction> = if let Some(add_val) = arguments.get("added_files") {
                    serde_json::from_value(add_val.clone()).unwrap_or_default()
                } else {
                    vec![DeltaAddAction {
                        path: format!("part-00000-{}.parquet", blake3::hash(b"seed").to_hex()),
                        size: 40960,
                        modification_time: DeltaLakeEngine::current_timestamp_ms(),
                        data_change: true,
                        partition_values: {
                            let mut m = HashMap::new();
                            m.insert("date".to_string(), "2026-09-16".to_string());
                            m
                        },
                        stats: serde_json::to_string(&DeltaFileStats {
                            num_records: 1500,
                            min_values: {
                                let mut m = HashMap::new();
                                m.insert("id".to_string(), json!(1));
                                m
                            },
                            max_values: {
                                let mut m = HashMap::new();
                                m.insert("id".to_string(), json!(1500));
                                m
                            },
                            null_count: {
                                let mut m = HashMap::new();
                                m.insert("blast_score".to_string(), 0);
                                m
                            },
                        })
                        .unwrap_or_default(),
                    }]
                };

                let removed: Vec<DeltaRemoveAction> = if let Some(rem_val) = arguments.get("removed_files") {
                    serde_json::from_value(rem_val.clone()).unwrap_or_default()
                } else {
                    Vec::new()
                };

                let (ver, actions) = self.engine.commit_transaction("WRITE", added, removed).await?;
                let result = json!({
                    "status": "committed",
                    "version": ver,
                    "commit_file": DeltaLakeEngine::format_commit_filename(ver),
                    "action_count": actions.len()
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
            "read_version" => {
                let ver = arguments.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
                let snapshot = self.engine.read_version(ver).await?;
                Ok(serde_json::to_string_pretty(&snapshot)?)
            }
            "create_checkpoint" => {
                let ver = arguments.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
                let chk_name = self.engine.create_checkpoint(ver).await?;
                let result = json!({
                    "status": "checkpoint_created",
                    "version": ver,
                    "checkpoint_file": chk_name
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
            "map_onelake_path" => {
                let ws = arguments.get("workspace").and_then(|w| w.as_str()).unwrap_or("Production-Fabric-Workspace");
                let lh = arguments.get("lakehouse_name").and_then(|l| l.as_str()).unwrap_or("CoreLakehouse");
                let tbl = arguments.get("table_name").and_then(|t| t.as_str()).unwrap_or("ASTTelemetry");

                let mapping = DeltaLakeEngine::map_onelake_path(ws, lh, tbl);
                Ok(serde_json::to_string_pretty(&mapping)?)
            }
            "get_table_history" => {
                let snap = self.engine.read_version(0).await?;
                let result = json!({
                    "table_name": snap.table_name,
                    "commits": snap.commit_history
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
            _ => Err(TagisanError::Execution(format!("Unsupported action '{action}' for copilot_delta_lake"))),
        }
    }
}
