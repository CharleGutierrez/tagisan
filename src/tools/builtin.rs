use super::ToolHandler;
use crate::engine::graph::{BlastRisk, CodebaseGraph};
use crate::error::{Result, TagisanError};
use crate::types::ContentBlock;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

// =========================================================================
// 1. ReadFileTool
// =========================================================================

/// Tool for reading file contents safely from local disk
#[derive(Debug, Default, Clone)]
pub struct ReadFileTool {
    pub working_dir: Option<std::path::PathBuf>,
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read the text contents of a file from the local filesystem."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute or relative path to the file to read."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'path'".to_string()))?;

        let path = Path::new(path_str);
        let target_path = if path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(path)
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        if !target_path.exists() {
            return Err(TagisanError::Execution(format!("File does not exist: {}", target_path.display())));
        }

        tokio::fs::read_to_string(&target_path)
            .await
            .map_err(|e| TagisanError::Execution(format!("Failed to read file '{}': {e}", target_path.display())))
    }
}

// =========================================================================
// 2. WriteFileTool
// =========================================================================

/// Tool for writing/creating files safely on the local filesystem
#[derive(Debug, Default, Clone)]
pub struct WriteFileTool {
    pub working_dir: Option<std::path::PathBuf>,
}

impl WriteFileTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Write text content to a file on the local filesystem. Creates parent directories automatically if they do not exist."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The target file path to create or overwrite."
                },
                "content": {
                    "type": "string",
                    "description": "The text content to write into the file."
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'path'".to_string()))?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'content'".to_string()))?;

        let path = Path::new(path_str);
        let target_path = if path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(path)
            } else {
                path.to_path_buf()
            }
        } else {
            path.to_path_buf()
        };

        if let Some(parent) = target_path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    TagisanError::Execution(format!("Failed to create parent directory '{parent:?}': {e}"))
                })?;
            }
        }

        tokio::fs::write(&target_path, content).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to write to file '{}': {e}", target_path.display()))
        })?;

        Ok(format!("Successfully wrote {} bytes to {}", content.len(), target_path.display()))
    }
}

// =========================================================================
// Helpers for File CRUD Operations
// =========================================================================

fn resolve_target_path(working_dir: &Option<PathBuf>, path: &Path) -> PathBuf {
    if path.is_relative() {
        if let Some(ref base) = working_dir {
            base.join(path)
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    }
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let bytes = s.as_bytes();
    let len = bytes.len();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

fn format_bytes_human(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB ({} bytes)", bytes as f64 / 1024.0, format_with_commas(bytes))
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB ({} bytes)", bytes as f64 / (1024.0 * 1024.0), format_with_commas(bytes))
    } else {
        format!("{:.2} GB ({} bytes)", bytes as f64 / (1024.0 * 1024.0 * 1024.0), format_with_commas(bytes))
    }
}

async fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dst).await?;
    let mut entries = tokio::fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let file_type = entry.file_type().await?;
        let target_entry = dst.join(entry.file_name());
        if file_type.is_dir() {
            Box::pin(copy_dir_recursive(&entry_path, &target_entry)).await?;
        } else {
            tokio::fs::copy(&entry_path, &target_entry).await?;
        }
    }
    Ok(())
}

// =========================================================================
// 2a. EditFileTool
// =========================================================================

/// Tool for editing existing files by replacing exact content chunks, with line scoping, backup creation, and atomic writes.
#[derive(Debug, Default, Clone)]
pub struct EditFileTool {
    pub working_dir: Option<PathBuf>,
}

impl EditFileTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for EditFileTool {
    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Edit a file by replacing an exact chunk of target text with replacement text. Supports line-range scoping, multiple replacement toggle, automatic backup creation, and atomic writes."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The target file to edit (absolute or relative to working directory)."
                },
                "target_content": {
                    "type": "string",
                    "description": "The exact substring or text chunk to be replaced."
                },
                "replacement_content": {
                    "type": "string",
                    "description": "The new replacement content."
                },
                "start_line": {
                    "type": "integer",
                    "description": "Optional 1-indexed line number to start scoped search range."
                },
                "end_line": {
                    "type": "integer",
                    "description": "Optional 1-indexed line number to end scoped search range."
                },
                "allow_multiple": {
                    "type": "boolean",
                    "description": "If true, replaces all occurrences in the target scope. If false (default), errors if multiple matches found."
                },
                "create_backup": {
                    "type": "boolean",
                    "description": "If true (default), writes '<path>.bak' before modifying."
                }
            },
            "required": ["path", "target_content", "replacement_content"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'path'".to_string()))?;

        let target_content = arguments
            .get("target_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'target_content'".to_string()))?;

        let replacement_content = arguments
            .get("replacement_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'replacement_content'".to_string()))?;

        if target_content.is_empty() {
            return Err(TagisanError::Execution("Parameter 'target_content' cannot be empty.".to_string()));
        }

        let start_line = arguments.get("start_line").and_then(|v| v.as_u64());
        let end_line = arguments.get("end_line").and_then(|v| v.as_u64());
        let allow_multiple = arguments.get("allow_multiple").and_then(|v| v.as_bool()).unwrap_or(false);
        let create_backup = arguments.get("create_backup").and_then(|v| v.as_bool()).unwrap_or(true);

        let target_path = resolve_target_path(&self.working_dir, Path::new(path_str));

        if !target_path.exists() {
            return Err(TagisanError::Execution(format!("File does not exist: '{}'", target_path.display())));
        }
        if target_path.is_dir() {
            return Err(TagisanError::Execution(format!("Target path is a directory, not a file: '{}'", target_path.display())));
        }

        let original_content = tokio::fs::read_to_string(&target_path)
            .await
            .map_err(|e| TagisanError::Execution(format!("Failed to read file '{}': {e}", target_path.display())))?;

        let lines: Vec<&str> = original_content.split_inclusive('\n').collect();
        let total_lines = lines.len();

        let (prefix, slice_content, suffix, scope_desc) = if start_line.is_some() || end_line.is_some() {
            let s = start_line.unwrap_or(1);
            if s < 1 {
                return Err(TagisanError::Execution("Parameter 'start_line' must be >= 1".to_string()));
            }
            if total_lines == 0 && s > 1 {
                return Err(TagisanError::Execution(format!("File is empty, start_line {s} exceeds line count 0")));
            }
            if s as usize > total_lines && total_lines > 0 {
                return Err(TagisanError::Execution(format!(
                    "Parameter 'start_line' ({s}) exceeds total line count ({total_lines})"
                )));
            }

            let e = match end_line {
                Some(val) => {
                    if val < 1 {
                        return Err(TagisanError::Execution("Parameter 'end_line' must be >= 1".to_string()));
                    }
                    if val < s {
                        return Err(TagisanError::Execution(format!(
                            "Parameter 'end_line' ({val}) cannot be less than 'start_line' ({s})"
                        )));
                    }
                    (val as usize).min(total_lines)
                }
                None => total_lines,
            };

            let start_idx = (s as usize) - 1;
            let end_idx = e;

            let prefix = lines[..start_idx].join("");
            let slice = lines[start_idx..end_idx].join("");
            let suffix = lines[end_idx..].join("");
            let scope_desc = format!("lines {s}..={e}");
            (prefix, slice, suffix, scope_desc)
        } else {
            (String::new(), original_content.clone(), String::new(), "entire file".to_string())
        };

        let match_count = slice_content.matches(target_content).count();
        if match_count == 0 {
            return Err(TagisanError::Execution(format!(
                "Target content not found within {scope_desc} in file '{}'",
                target_path.display()
            )));
        }
        if match_count > 1 && !allow_multiple {
            return Err(TagisanError::Execution(format!(
                "Target content found {match_count} times within {scope_desc} in file '{}'. Set 'allow_multiple: true' to replace all matches, or narrow the search scope using 'start_line' and 'end_line'.",
                target_path.display()
            )));
        }

        let modified_slice = if allow_multiple {
            slice_content.replace(target_content, replacement_content)
        } else {
            slice_content.replacen(target_content, replacement_content, 1)
        };

        let new_content = format!("{prefix}{modified_slice}{suffix}");

        let mut backup_info = "Backup disabled".to_string();
        if create_backup {
            let backup_path = PathBuf::from(format!("{}.bak", target_path.display()));
            tokio::fs::write(&backup_path, &original_content).await.map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to write backup file '{}': {e}",
                    backup_path.display()
                ))
            })?;
            backup_info = format!("Backup saved to '{}'", backup_path.display());
        }

        // Atomic file write using temporary file in same parent directory
        let parent_dir = target_path.parent().unwrap_or_else(|| Path::new("."));
        let temp_file_name = format!(
            ".tmp_edit_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let temp_path = parent_dir.join(temp_file_name);

        if let Err(e) = tokio::fs::write(&temp_path, &new_content).await {
            return Err(TagisanError::Execution(format!(
                "Failed to write temporary file '{}': {e}",
                temp_path.display()
            )));
        }

        if let Err(rename_err) = tokio::fs::rename(&temp_path, &target_path).await {
            let _ = tokio::fs::remove_file(&temp_path).await;
            tokio::fs::write(&target_path, &new_content).await.map_err(|write_err| {
                TagisanError::Execution(format!(
                    "Failed to write target file '{}' (rename failed: {rename_err}; direct write failed: {write_err})",
                    target_path.display()
                ))
            })?;
        }

        let original_bytes = original_content.len();
        let new_bytes = new_content.len();
        let delta_bytes = (new_bytes as i64) - (original_bytes as i64);
        let new_lines_count = new_content.split_inclusive('\n').count();

        Ok(format!(
            "Successfully edited file '{}'.\n\
             - Scope: {}\n\
             - Replacements: {} occurrence(s)\n\
             - Size: {} bytes -> {} bytes ({:+})\n\
             - Line count: {} -> {}\n\
             - Backup: {}",
            target_path.display(),
            scope_desc,
            match_count,
            format_with_commas(original_bytes as u64),
            format_with_commas(new_bytes as u64),
            delta_bytes,
            total_lines,
            new_lines_count,
            backup_info
        ))
    }
}

// =========================================================================
// 2b. DeleteFileTool
// =========================================================================

/// Tool for safely deleting files or directories, with safety trash bin support.
#[derive(Debug, Default, Clone)]
pub struct DeleteFileTool {
    pub working_dir: Option<PathBuf>,
}

impl DeleteFileTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for DeleteFileTool {
    fn name(&self) -> &'static str {
        "delete_file"
    }

    fn description(&self) -> &'static str {
        "Delete a file or directory. Moves items to .tagisan/trash/<timestamp>_<filename> by default for safety, or permanently deletes if trash is set to false."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target file or directory path to delete."
                },
                "recursive": {
                    "type": "boolean",
                    "description": "If true, recursively deletes directories and contents. Required when deleting a directory (default: false)."
                },
                "trash": {
                    "type": "boolean",
                    "description": "If true (default), moves to '.tagisan/trash/<timestamp>_<filename>' for safety. If false, permanently deletes."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'path'".to_string()))?;

        let recursive = arguments.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let trash = arguments.get("trash").and_then(|v| v.as_bool()).unwrap_or(true);

        let target_path = resolve_target_path(&self.working_dir, Path::new(path_str));

        let metadata = match tokio::fs::symlink_metadata(&target_path).await {
            Ok(m) => m,
            Err(_) => {
                return Err(TagisanError::Execution(format!(
                    "Target path does not exist: '{}'",
                    target_path.display()
                )));
            }
        };

        let is_dir = metadata.is_dir();
        if is_dir && !recursive {
            return Err(TagisanError::Execution(format!(
                "Path '{}' is a directory. Set 'recursive: true' to delete directories.",
                target_path.display()
            )));
        }

        if trash {
            let base_dir = self.working_dir.clone().unwrap_or_else(|| PathBuf::from("."));
            let trash_dir = base_dir.join(".tagisan").join("trash");
            tokio::fs::create_dir_all(&trash_dir).await.map_err(|e| {
                TagisanError::Execution(format!("Failed to create trash directory '{}': {e}", trash_dir.display()))
            })?;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            let file_name = target_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unnamed".to_string());
            let trash_dest = trash_dir.join(format!("{}_{}", timestamp, file_name));

            if let Err(_rename_err) = tokio::fs::rename(&target_path, &trash_dest).await {
                // Fallback if cross-device move occurs
                if is_dir {
                    copy_dir_recursive(&target_path, &trash_dest).await.map_err(|e| {
                        TagisanError::Execution(format!("Failed to copy directory to trash: {e}"))
                    })?;
                    tokio::fs::remove_dir_all(&target_path).await.map_err(|e| {
                        TagisanError::Execution(format!("Failed to clean up original directory after trash move: {e}"))
                    })?;
                } else {
                    tokio::fs::copy(&target_path, &trash_dest).await.map_err(|e| {
                        TagisanError::Execution(format!("Failed to copy file to trash: {e}"))
                    })?;
                    tokio::fs::remove_file(&target_path).await.map_err(|e| {
                        TagisanError::Execution(format!("Failed to clean up original file after trash move: {e}"))
                    })?;
                }
            }

            Ok(format!(
                "Safely moved {} '{}' to trash at '{}'.",
                if is_dir { "directory" } else { "file" },
                target_path.display(),
                trash_dest.display()
            ))
        } else {
            if is_dir {
                tokio::fs::remove_dir_all(&target_path).await.map_err(|e| {
                    TagisanError::Execution(format!("Failed to permanently delete directory '{}': {e}", target_path.display()))
                })?;
                Ok(format!(
                    "Permanently deleted directory '{}' (recursive).",
                    target_path.display()
                ))
            } else {
                let size = metadata.len();
                tokio::fs::remove_file(&target_path).await.map_err(|e| {
                    TagisanError::Execution(format!("Failed to permanently delete file '{}': {e}", target_path.display()))
                })?;
                Ok(format!(
                    "Permanently deleted file '{}' ({} bytes).",
                    target_path.display(),
                    format_with_commas(size)
                ))
            }
        }
    }
}

// =========================================================================
// 2c. ListDirTool
// =========================================================================

/// Tool for listing directory contents with recursive depth control, hidden file filtering, pattern matching, and formatted tabular outputs.
#[derive(Debug, Default, Clone)]
pub struct ListDirTool {
    pub working_dir: Option<PathBuf>,
}

impl ListDirTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

struct ListedEntry {
    rel_path: String,
    entry_type: String,
    size_bytes: u64,
    item_count: Option<usize>,
    modified: String,
}

#[async_trait]
impl ToolHandler for ListDirTool {
    fn name(&self) -> &'static str {
        "list_dir"
    }

    fn description(&self) -> &'static str {
        "List files and subdirectories in a structured table, including entry type, size, item count, and modification timestamp."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (default: '.')."
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum depth to traverse (1 = direct children only, default: 1)."
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "Whether to include hidden files and directories (starting with '.'). Default: false."
                },
                "pattern": {
                    "type": "string",
                    "description": "Optional substring or extension pattern filter (e.g. '.rs', 'src')."
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let max_depth = arguments
            .get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .max(1) as usize;

        let include_hidden = arguments
            .get("include_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let pattern = arguments
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let target_path = resolve_target_path(&self.working_dir, Path::new(path_str));

        if !target_path.exists() {
            return Err(TagisanError::Execution(format!("Path does not exist: '{}'", target_path.display())));
        }
        if !target_path.is_dir() {
            return Err(TagisanError::Execution(format!("Path is not a directory: '{}'", target_path.display())));
        }

        let mut entries_collected = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((target_path.clone(), String::new(), 1usize));

        while let Some((curr_dir, rel_prefix, depth)) = queue.pop_front() {
            let mut reader = match tokio::fs::read_dir(&curr_dir).await {
                Ok(r) => r,
                Err(e) => {
                    return Err(TagisanError::Execution(format!(
                        "Failed to read directory '{}': {e}",
                        curr_dir.display()
                    )));
                }
            };

            while let Ok(Some(entry)) = reader.next_entry().await {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if !include_hidden && file_name.starts_with('.') {
                    continue;
                }

                let entry_path = entry.path();
                let rel_path = if rel_prefix.is_empty() {
                    file_name.clone()
                } else {
                    format!("{rel_prefix}/{file_name}")
                };

                let meta = match tokio::fs::symlink_metadata(&entry_path).await {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_symlink = meta.file_type().is_symlink();
                let is_dir = meta.is_dir();
                let entry_type = if is_symlink {
                    "[SYMLINK]".to_string()
                } else if is_dir {
                    "[DIR]".to_string()
                } else {
                    "[FILE]".to_string()
                };

                let modified = match meta.modified() {
                    Ok(t) => {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    }
                    Err(_) => "unknown".to_string(),
                };

                let (size_bytes, item_count) = if is_dir {
                    let mut count = 0usize;
                    if let Ok(mut sub_reader) = tokio::fs::read_dir(&entry_path).await {
                        while let Ok(Some(_)) = sub_reader.next_entry().await {
                            count += 1;
                        }
                    }
                    (0u64, Some(count))
                } else {
                    (meta.len(), None)
                };

                entries_collected.push(ListedEntry {
                    rel_path: if is_dir { format!("{rel_path}/") } else { rel_path.clone() },
                    entry_type,
                    size_bytes,
                    item_count,
                    modified,
                });

                if is_dir && depth < max_depth && !is_symlink {
                    queue.push_back((entry_path, rel_path, depth + 1));
                }
            }
        }

        if let Some(ref pat) = pattern {
            let pat_lower = pat.to_lowercase();
            entries_collected.retain(|e| e.rel_path.to_lowercase().contains(&pat_lower));
        }

        entries_collected.sort_by(|a, b| a.rel_path.to_lowercase().cmp(&b.rel_path.to_lowercase()));

        let mut output = format!(
            "Directory listing for: '{}' (max_depth: {}, entries: {})\n\n",
            target_path.display(),
            max_depth,
            entries_collected.len()
        );

        if entries_collected.is_empty() {
            output.push_str("No files or directories found matching the criteria.\n");
            return Ok(output);
        }

        output.push_str(&format!(
            "{:<10} {:<16} {:<20} {}\n",
            "TYPE", "SIZE / ITEMS", "MODIFIED", "PATH"
        ));
        output.push_str(&format!("{}\n", "-".repeat(78)));

        let mut total_files = 0usize;
        let mut total_dirs = 0usize;
        let mut total_bytes = 0u64;

        for e in &entries_collected {
            let size_or_items = if let Some(items) = e.item_count {
                total_dirs += 1;
                format!("{} item(s)", items)
            } else {
                total_files += 1;
                total_bytes += e.size_bytes;
                format!("{} B", format_with_commas(e.size_bytes))
            };

            output.push_str(&format!(
                "{:<10} {:<16} {:<20} {}\n",
                e.entry_type, size_or_items, e.modified, e.rel_path
            ));
        }

        output.push_str(&format!("{}\n", "-".repeat(78)));
        output.push_str(&format!(
            "Total: {} file(s) ({}), {} directory(ies)\n",
            total_files,
            format_bytes_human(total_bytes),
            total_dirs
        ));

        Ok(output)
    }
}

// =========================================================================
// 3. RunCommandTool
// =========================================================================

/// Tool for executing shell commands with timeout and output capture
#[derive(Debug, Clone)]
pub struct RunCommandTool {
    pub default_timeout: Duration,
    pub working_dir: Option<std::path::PathBuf>,
}

impl Default for RunCommandTool {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            working_dir: None,
        }
    }
}

impl RunCommandTool {
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            default_timeout: timeout_duration,
            working_dir: None,
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for RunCommandTool {
    fn name(&self) -> &'static str {
        "run_command"
    }

    fn description(&self) -> &'static str {
        "Run a terminal command (e.g. 'cargo test', 'git status') and capture its exit code, stdout, and stderr."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 30)."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let command_str = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'command'".to_string()))?;

        let timeout_secs = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd.exe");
            c.args(["/C", command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command_str]);
            c
        };

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.kill_on_drop(true);

        let output_fut = cmd.output();
        let output = timeout(timeout_secs, output_fut)
            .await
            .map_err(|_| {
                TagisanError::Execution(format!(
                    "Command '{}' timed out after {:.1}s",
                    command_str,
                    timeout_secs.as_secs_f32()
                ))
            })?
            .map_err(|e| TagisanError::Execution(format!("Failed to execute command '{command_str}': {e}")))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = format!("Exit Code: {exit_code}\n");
        if !stdout.is_empty() {
            result.push_str("--- STDOUT ---\n");
            result.push_str(&stdout);
            if !stdout.ends_with('\n') {
                result.push('\n');
            }
        }
        if !stderr.is_empty() {
            result.push_str("--- STDERR ---\n");
            result.push_str(&stderr);
            if !stderr.ends_with('\n') {
                result.push('\n');
            }
        }

        Ok(result)
    }
}

// =========================================================================
// 4. CalculatorTool
// =========================================================================

/// Tool for evaluating mathematical expressions
#[derive(Debug, Default, Clone)]
pub struct CalculatorTool;

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for CalculatorTool {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn description(&self) -> &'static str {
        "Evaluate mathematical expressions. Supports basic arithmetic (+, -, *, /, %, ^), functions (sqrt, abs, sin, cos, tan, ln, log, exp, floor, ceil, round), and constants (pi, e)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate (e.g., '2 + 2 * (3 ^ 2)', 'sqrt(144) + 5')."
                }
            },
            "required": ["expression"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let expr = arguments
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'expression'".to_string()))?;

        match eval_math_expression(expr) {
            Ok(val) => {
                if val.fract() == 0.0 && val.abs() < 1e15 {
                    Ok(format!("{}", val as i64))
                } else {
                    Ok(format!("{val}"))
                }
            }
            Err(e) => Err(TagisanError::Execution(format!("Math evaluation error: {e}"))),
        }
    }
}

// =========================================================================
// 5. ViewImageTool
// =========================================================================

/// Tool for inspecting an image file, returning its format, dimensions/size, and base64 preview
#[derive(Debug, Default, Clone)]
pub struct ViewImageTool;

impl ViewImageTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for ViewImageTool {
    fn name(&self) -> &'static str {
        "view_image"
    }

    fn description(&self) -> &'static str {
        "Inspect an image file on disk, validating format (.png, .jpg, .jpeg, .webp, .gif), computing file size, and encoding to Base64 for multimodal LLM vision analysis."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The local filesystem path to the image file (.png, .jpg, .jpeg, .webp, or .gif)."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'path'".to_string()))?;

        let block = ContentBlock::from_image_file(path_str)?;
        if let ContentBlock::Image { media_type, data_base64 } = block {
            let byte_count = std::fs::metadata(path_str)
                .map(|m| m.len())
                .unwrap_or(0);
            Ok(format!(
                "Image validated and loaded successfully from '{}'. Format: {}, Size: {} bytes, Base64 payload: {} chars.",
                path_str,
                media_type,
                byte_count,
                data_base64.len()
            ))
        } else {
            Err(TagisanError::Execution("Failed to process image content block".to_string()))
        }
    }
}

// =========================================================================
// 6. SearchMemoryTool
// =========================================================================

/// Tool that searches long-term vector memory and indexed codebase chunks
#[derive(Clone)]
pub struct SearchMemoryTool {
    store: Arc<crate::memory::VectorStore>,
    provider: Arc<dyn crate::memory::EmbeddingProvider>,
}

impl SearchMemoryTool {
    pub fn new(store: Arc<crate::memory::VectorStore>, provider: Arc<dyn crate::memory::EmbeddingProvider>) -> Self {
        Self { store, provider }
    }
}

#[async_trait]
impl ToolHandler for SearchMemoryTool {
    fn name(&self) -> &'static str {
        "search_memory"
    }

    fn description(&self) -> &'static str {
        "Search long-term vector memory and indexed codebase chunks using semantic similarity"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query or concept to locate in codebase chunks or episodic memory"
                },
                "top_k": {
                    "type": "integer",
                    "description": "Maximum number of top matches to return (default: 5)"
                },
                "threshold": {
                    "type": "number",
                    "description": "Minimum similarity score between 0.0 and 1.0 (default: 0.15)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'query'".to_string()))?;

        let top_k = arguments
            .get("top_k")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let threshold = arguments
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.15) as f32;

        let query_embedding = self.provider.embed_text(query).await?;
        let hits = self.store.search(&query_embedding, top_k, threshold);

        if hits.is_empty() {
            return Ok(format!("No relevant memory chunks found matching query: \"{query}\" (threshold: {threshold:.2})"));
        }

        let mut output = format!("Found {} relevant memory chunks for \"{}\":\n\n", hits.len(), query);
        for (i, hit) in hits.iter().enumerate() {
            let doc = &hit.document;
            let file_path = doc.metadata.get("file_path").map(|s| s.as_str()).unwrap_or(&doc.id);
            let start = doc.metadata.get("start_line").map(|s| s.as_str()).unwrap_or("?");
            let end = doc.metadata.get("end_line").map(|s| s.as_str()).unwrap_or("?");
            let lang = doc.metadata.get("language").map(|s| s.as_str()).unwrap_or("text");

            output.push_str(&format!(
                "[{}] {} (Lines {}-{}, Score: {:.3}, Lang: {})\n```{}\n{}\n```\n\n",
                i + 1,
                file_path,
                start,
                end,
                hit.score,
                lang,
                lang,
                doc.text.trim()
            ));
        }

        Ok(output.trim_end().to_string())
    }
}

// =========================================================================
// 7. SaveMemoryTool
// =========================================================================

/// Tool that saves an insight, decision, or summary into long-term episodic memory
#[derive(Clone)]
pub struct SaveMemoryTool {
    store: Arc<crate::memory::VectorStore>,
    provider: Arc<dyn crate::memory::EmbeddingProvider>,
    persistence_path: Option<std::path::PathBuf>,
}

impl SaveMemoryTool {
    pub fn new(store: Arc<crate::memory::VectorStore>, provider: Arc<dyn crate::memory::EmbeddingProvider>) -> Self {
        Self {
            store,
            provider,
            persistence_path: Some(crate::memory::VectorStore::default_path()),
        }
    }

    pub fn with_persistence_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.persistence_path = Some(path.into());
        self
    }
}

#[async_trait]
impl ToolHandler for SaveMemoryTool {
    fn name(&self) -> &'static str {
        "save_memory"
    }

    fn description(&self) -> &'static str {
        "Save a key architectural decision, insight, debugging solution, or summary into long-term episodic memory"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "tag": {
                    "type": "string",
                    "description": "Category tag: decision, architecture, debugging, insight, summary"
                },
                "summary": {
                    "type": "string",
                    "description": "Concise 1-2 sentence description of what to remember"
                },
                "details": {
                    "type": "string",
                    "description": "Detailed explanation, code snippet, or steps to persist"
                }
            },
            "required": ["tag", "summary"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let tag = arguments
            .get("tag")
            .and_then(|v| v.as_str())
            .unwrap_or("insight");

        let summary = arguments
            .get("summary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'summary'".to_string()))?;

        let details = arguments
            .get("details")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let episodic = crate::memory::EpisodicMemory::new();
        let doc_id = episodic.record(&self.store, self.provider.as_ref(), tag, summary, details).await?;

        if let Some(ref path) = self.persistence_path {
            let _ = self.store.save_to_file(path);
        }

        Ok(format!(
            "Successfully saved episodic memory item [{}] under tag '{}'. Memory id: {}",
            summary, tag, doc_id
        ))
    }
}

// =========================================================================
// 8. SearchSkillsTool
// =========================================================================

/// Tool that searches and discovers engineering skills from the 3,840+ skill catalog
#[derive(Clone, Default)]
pub struct SearchSkillsTool {
    dispatcher: Option<Arc<crate::ecc::skills::SkillDispatcher>>,
}

impl SearchSkillsTool {
    pub fn new(dispatcher: Arc<crate::ecc::skills::SkillDispatcher>) -> Self {
        Self {
            dispatcher: Some(dispatcher),
        }
    }

    pub fn with_default() -> Self {
        Self { dispatcher: None }
    }
}

#[async_trait]
impl ToolHandler for SearchSkillsTool {
    fn name(&self) -> &'static str {
        "search_skills"
    }

    fn description(&self) -> &'static str {
        "Search and discover available engineering skills from the 3,840+ skill repository based on task objectives, technology keywords, or triggers. Returns skill names, descriptions, relevance scores, and operational instructions."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Technical task, framework, SDK, or architecture concept to find skills for."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of skills to return (default: 3, max: 10)."
                },
                "domain": {
                    "type": "string",
                    "description": "Optional domain filter (e.g., 'azure', 'rust', 'bun', 'security', 'sap', 'test')."
                },
                "include_instructions": {
                    "type": "boolean",
                    "description": "Whether to include full operational instructions in the output (default: true)."
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'query'".to_string()))?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(3)
            .clamp(1, 10) as usize;

        let domain = arguments.get("domain").and_then(|v| v.as_str());
        let include_instructions = arguments
            .get("include_instructions")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if let Some(ref disp) = self.dispatcher {
            Ok(disp.search_and_format(query, limit, domain, include_instructions))
        } else {
            Ok(crate::ecc::skills::global_dispatcher().search_and_format(query, limit, domain, include_instructions))
        }
    }
}

// =========================================================================
// 8b. FetchSkillTool
// =========================================================================

/// Tool that retrieves full specification, directives, and verification checklists for a specific skill
#[derive(Clone, Default)]
pub struct FetchSkillTool {
    dispatcher: Option<Arc<crate::ecc::skills::SkillDispatcher>>,
}

impl FetchSkillTool {
    pub fn new(dispatcher: Arc<crate::ecc::skills::SkillDispatcher>) -> Self {
        Self {
            dispatcher: Some(dispatcher),
        }
    }

    pub fn with_default() -> Self {
        Self { dispatcher: None }
    }
}

#[async_trait]
impl ToolHandler for FetchSkillTool {
    fn name(&self) -> &'static str {
        "fetch_skill"
    }

    fn description(&self) -> &'static str {
        "Retrieve the complete specification, operational directives, invariant rules, and verification checklists for a specific engineering skill from the ECC catalog by exact or partial name."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The exact or partial skill name to fetch (e.g., 'ba-state-machine-lifecycle-modeling', 'security-review', 'tokio-async-tuning')."
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'name'".to_string()))?;

        let spec = if let Some(ref disp) = self.dispatcher {
            disp.get_skill_spec(name)
        } else {
            crate::ecc::skills::global_dispatcher().get_skill_spec(name)
        };

        match spec {
            Some(content) => Ok(content),
            None => Ok(format!("Skill '{}' not found in ECC catalog. Use 'search_skills' to discover available skills.", name)),
        }
    }
}

// =========================================================================
// Math Expression Parser & Evaluator (Pratt / Recursive Descent)
// =========================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    LParen,
    RParen,
    Comma,
}

fn tokenize(expr: &str) -> std::result::Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        match ch {
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    tokens.push(Token::Pow);
                    i += 2;
                } else {
                    tokens.push(Token::Mul);
                    i += 1;
                }
            }
            '/' => { tokens.push(Token::Div); i += 1; }
            '%' => { tokens.push(Token::Mod); i += 1; }
            '^' => { tokens.push(Token::Pow); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    num_str.push(chars[i]);
                    i += 1;
                }
                // Support scientific notation (e.g., 1e5, 2.5e-3, 1E+6)
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    if i + 1 < chars.len() && (chars[i + 1].is_ascii_digit() || chars[i + 1] == '+' || chars[i + 1] == '-') {
                        num_str.push(chars[i]);
                        i += 1;
                        if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                            num_str.push(chars[i]);
                            i += 1;
                        }
                        while i < chars.len() && chars[i].is_ascii_digit() {
                            num_str.push(chars[i]);
                            i += 1;
                        }
                    }
                }
                let num: f64 = num_str.parse().map_err(|_| format!("Invalid number: '{num_str}'"))?;
                tokens.push(Token::Number(num));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    ident.push(chars[i]);
                    i += 1;
                }
                tokens.push(Token::Ident(ident.to_lowercase()));
            }
            _ => return Err(format!("Unexpected character: '{ch}'")),
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn parse_expression(&mut self) -> std::result::Result<f64, String> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> std::result::Result<f64, String> {
        let mut left = self.parse_multiplication()?;

        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    let right = self.parse_multiplication()?;
                    left += right;
                }
                Token::Minus => {
                    self.next();
                    let right = self.parse_multiplication()?;
                    left -= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> std::result::Result<f64, String> {
        let mut left = self.parse_power()?;

        while let Some(tok) = self.peek() {
            match tok {
                Token::Mul => {
                    self.next();
                    let right = self.parse_power()?;
                    left *= right;
                }
                Token::Div => {
                    self.next();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left /= right;
                }
                Token::Mod => {
                    self.next();
                    let right = self.parse_power()?;
                    if right == 0.0 {
                        return Err("Modulo by zero".to_string());
                    }
                    left %= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> std::result::Result<f64, String> {
        let left = self.parse_unary()?;

        if let Some(Token::Pow) = self.peek() {
            self.next();
            let right = self.parse_power()?; // right-associative
            Ok(left.powf(right))
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> std::result::Result<f64, String> {
        if let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    self.parse_unary()
                }
                Token::Minus => {
                    self.next();
                    let val = self.parse_unary()?;
                    Ok(-val)
                }
                _ => self.parse_primary(),
            }
        } else {
            Err("Unexpected end of expression".to_string())
        }
    }

    fn parse_primary(&mut self) -> std::result::Result<f64, String> {
        match self.next() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::Ident(ident)) => match ident.as_str() {
                "pi" => Ok(std::f64::consts::PI),
                "e" => Ok(std::f64::consts::E),
                "sqrt" => self.parse_func_arg(|v| {
                    if v < 0.0 {
                        Err("Square root of negative number".to_string())
                    } else {
                        Ok(v.sqrt())
                    }
                }),
                "abs" => self.parse_func_arg(|v| Ok(v.abs())),
                "sin" => self.parse_func_arg(|v| Ok(v.sin())),
                "cos" => self.parse_func_arg(|v| Ok(v.cos())),
                "tan" => self.parse_func_arg(|v| Ok(v.tan())),
                "asin" => self.parse_func_arg(|v| {
                    if v < -1.0 || v > 1.0 {
                        Err("Domain error: asin argument must be in [-1, 1]".to_string())
                    } else {
                        Ok(v.asin())
                    }
                }),
                "acos" => self.parse_func_arg(|v| {
                    if v < -1.0 || v > 1.0 {
                        Err("Domain error: acos argument must be in [-1, 1]".to_string())
                    } else {
                        Ok(v.acos())
                    }
                }),
                "atan" => self.parse_func_arg(|v| Ok(v.atan())),
                "ln" => self.parse_func_arg(|v| {
                    if v <= 0.0 {
                        Err("Logarithm of non-positive number".to_string())
                    } else {
                        Ok(v.ln())
                    }
                }),
                "log" | "log10" => self.parse_func_arg(|v| {
                    if v <= 0.0 {
                        Err("Logarithm of non-positive number".to_string())
                    } else {
                        Ok(v.log10())
                    }
                }),
                "exp" => self.parse_func_arg(|v| Ok(v.exp())),
                "floor" => self.parse_func_arg(|v| Ok(v.floor())),
                "ceil" => self.parse_func_arg(|v| Ok(v.ceil())),
                "round" => self.parse_func_arg(|v| Ok(v.round())),
                _ => Err(format!("Unknown function or constant: '{ident}'")),
            },
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("Missing closing parenthesis ')'".to_string()),
                }
            }
            Some(tok) => Err(format!("Unexpected token: {tok:?}")),
            None => Err("Unexpected end of expression".to_string()),
        }
    }

    fn parse_func_arg<F>(&mut self, f: F) -> std::result::Result<f64, String>
    where
        F: FnOnce(f64) -> std::result::Result<f64, String>,
    {
        match self.next() {
            Some(Token::LParen) => {
                let arg = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => f(arg),
                    _ => Err("Missing closing parenthesis for function call".to_string()),
                }
            }
            _ => Err("Expected '(' after function name".to_string()),
        }
    }
}

pub fn eval_math_expression(expr: &str) -> std::result::Result<f64, String> {
    let tokens = tokenize(expr)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut parser = Parser::new(tokens);
    let result = parser.parse_expression()?;
    if parser.pos < parser.tokens.len() {
        return Err(format!("Unexpected trailing token: {:?}", parser.tokens[parser.pos]));
    }
    Ok(result)
}

// =========================================================================
// QueryCodeGraphTool
// =========================================================================

/// Tool for querying codebase AST knowledge graph for symbols, callers, and callees
#[derive(Debug, Default, Clone)]
pub struct QueryCodeGraphTool {
    pub working_dir: Option<PathBuf>,
}

impl QueryCodeGraphTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for QueryCodeGraphTool {
    fn name(&self) -> &'static str {
        "query_code_graph"
    }

    fn description(&self) -> &'static str {
        "Query the AST codebase knowledge graph for symbols, callers, or callees across Rust, Python, TypeScript, and Go."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Symbol name or pattern to search for (e.g. 'TokenBudgetTracker', 'build_from_dir', 'execute')."
                },
                "direction": {
                    "type": "string",
                    "enum": ["definition", "callers", "callees"],
                    "description": "Query mode: 'definition' to locate symbol, 'callers' for incoming callers, 'callees' for outgoing calls (default: 'definition')."
                },
                "path": {
                    "type": "string",
                    "description": "Root codebase path to index or search (default: '.')."
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'query'".to_string()))?;

        let direction = arguments
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("definition");

        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let root_path = if Path::new(path_str).is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(path_str)
            } else {
                PathBuf::from(path_str)
            }
        } else {
            PathBuf::from(path_str)
        };

        let q = query.to_string();
        let dir = direction.to_string();
        tokio::task::spawn_blocking(move || {
            let graph = CodebaseGraph::build_from_dir(&root_path, 10_000)?;

            match dir.as_str() {
                "callers" => {
                    let callers = graph.find_callers(&q);
                    if callers.is_empty() {
                        return Ok(format!("No callers found for symbol '{q}'."));
                    }

                    let mut out = format!("### Incoming Callers of `{q}` (Total: {})\n\n", callers.len());
                    out.push_str("| Caller Symbol | Kind | File | Line | Relation |\n");
                    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

                    for (caller, edge) in callers {
                        let line_str = edge
                            .call_site_line
                            .map(|l| l.to_string())
                            .unwrap_or_else(|| caller.line.to_string());
                        out.push_str(&format!(
                            "| `{}` | {} | `{}` | {} | {} |\n",
                            caller.qualified_name,
                            caller.kind.as_str(),
                            caller.file.display(),
                            line_str,
                            edge.relation.as_str()
                        ));
                    }

                    Ok(out)
                }
                "callees" => {
                    let callees = graph.find_callees(&q);
                    if callees.is_empty() {
                        return Ok(format!("No outgoing calls found for symbol '{q}'."));
                    }

                    let mut out = format!("### Outgoing Calls from `{q}` (Total: {})\n\n", callees.len());
                    out.push_str("| Callee Symbol | Kind | File | Call Site Line |\n");
                    out.push_str("| :--- | :--- | :--- | :--- |\n");

                    for (callee, edge) in callees {
                        let line_str = edge
                            .call_site_line
                            .map(|l| l.to_string())
                            .unwrap_or_else(|| "-".to_string());
                        out.push_str(&format!(
                            "| `{}` | {} | `{}` | {} |\n",
                            callee.qualified_name,
                            callee.kind.as_str(),
                            callee.file.display(),
                            line_str
                        ));
                    }

                    Ok(out)
                }
                _ => {
                    let symbols = graph.find_symbol(&q);
                    if symbols.is_empty() {
                        return Ok(format!("No symbols found matching query '{q}'."));
                    }

                    let mut out = format!("### Symbol Definitions Matching `{q}` (Total: {})\n\n", symbols.len());
                    out.push_str("| Symbol Name | Kind | Visibility | File | Line | Signature |\n");
                    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");

                    for sym in symbols {
                        out.push_str(&format!(
                            "| `{}` | {} | {} | `{}` | {} | `{}` |\n",
                            sym.qualified_name,
                            sym.kind.as_str(),
                            sym.visibility.as_str(),
                            sym.file.display(),
                            sym.line,
                            sym.signature.replace('|', "\\|")
                        ));
                    }

                    Ok(out)
                }
            }
        })
        .await
        .map_err(|e| TagisanError::Execution(format!("QueryCodeGraph execution panic: {e}")))?
    }
}

// =========================================================================
// CalculateBlastRadiusTool
// =========================================================================

/// Tool for calculating transitive blast radius and refactoring risk analysis
#[derive(Debug, Default, Clone)]
pub struct CalculateBlastRadiusTool {
    pub working_dir: Option<PathBuf>,
}

impl CalculateBlastRadiusTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for CalculateBlastRadiusTool {
    fn name(&self) -> &'static str {
        "calculate_blast_radius"
    }

    fn description(&self) -> &'static str {
        "Calculate the transitive blast radius and impact risk of modifying or refactoring a code symbol."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target symbol name or qualified path to evaluate (e.g. 'TokenBudgetTracker', 'execute')."
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum transitive depth to traverse callers and references (default: 3)."
                },
                "path": {
                    "type": "string",
                    "description": "Root codebase path to index or search (default: '.')."
                }
            },
            "required": ["target"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let target = arguments
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'target'".to_string()))?;

        let max_depth = arguments
            .get("max_depth")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(3);

        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let root_path = if Path::new(path_str).is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(path_str)
            } else {
                PathBuf::from(path_str)
            }
        } else {
            PathBuf::from(path_str)
        };

        let t = target.to_string();
        tokio::task::spawn_blocking(move || {
            let graph = CodebaseGraph::build_from_dir(&root_path, 10_000)?;
            let report = graph.calculate_blast_radius(&t, max_depth)?;

            let risk_badge = match report.risk_level {
                BlastRisk::Low => "🟢 LOW RISK",
                BlastRisk::Medium => "🟡 MEDIUM RISK",
                BlastRisk::High => "🟠 HIGH RISK",
                BlastRisk::Critical => "🔴 CRITICAL RISK",
            };

            let mut out = format!("# Blast Radius Analysis: `{}`\n\n", report.target_symbol);
            out.push_str(&format!("- **Target File**: `{}`\n", report.target_file.display()));
            out.push_str(&format!("- **Assessed Risk Level**: {}\n", risk_badge));
            out.push_str(&format!("- **Total Affected Symbols**: {}\n", report.total_affected_symbols));
            out.push_str(&format!("- **Affected Files Count**: {}\n\n", report.affected_files.len()));

            // Direct callers
            out.push_str(&format!("### Direct Callers ({})\n", report.direct_callers.len()));
            if report.direct_callers.is_empty() {
                out.push_str("*None identified*\n\n");
            } else {
                for c in &report.direct_callers {
                    out.push_str(&format!("- `{c}`\n"));
                }
                out.push('\n');
            }

            // Transitive callers
            out.push_str(&format!("### Transitive Callers ({})\n", report.transitive_callers.len()));
            if report.transitive_callers.is_empty() {
                out.push_str("*None (within depth limit)*\n\n");
            } else {
                for c in &report.transitive_callers {
                    out.push_str(&format!("- `{c}`\n"));
                }
                out.push('\n');
            }

            // Implementing types
            if !report.implementing_types.is_empty() {
                out.push_str(&format!("### Implementing Types ({})\n", report.implementing_types.len()));
                for imp in &report.implementing_types {
                    out.push_str(&format!("- `{imp}`\n"));
                }
                out.push('\n');
            }

            // Affected files
            out.push_str(&format!("### Affected Files ({})\n", report.affected_files.len()));
            for f in &report.affected_files {
                out.push_str(&format!("- `{}`\n", f.display()));
            }
            out.push('\n');

            // Recommendations
            out.push_str("### Refactoring Recommendations\n");
            for rec in &report.recommendations {
                out.push_str(&format!("- {rec}\n"));
            }

            Ok(out)
        })
        .await
        .map_err(|e| TagisanError::Execution(format!("CalculateBlastRadius execution panic: {e}")))?
    }
}
