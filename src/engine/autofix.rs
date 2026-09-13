//! Self-Healing Compiler & TDD Healer Engine (`tgs autofix`)
//!
//! Autonomous polyglot compiler diagnostic parsing, surgical AST patch synthesis,
//! and iterative self-healing verification for Rust, TypeScript, Python, and Go.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported project ecosystems for self-healing compiler diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    TypeScript,
    Python,
    Go,
    Unknown,
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::TypeScript => write!(f, "TypeScript"),
            Self::Python => write!(f, "Python"),
            Self::Go => write!(f, "Go"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Detect the programming ecosystem of a project path or file.
pub fn detect_project_type(path: &Path) -> ProjectType {
    if path.is_file() {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => return ProjectType::Rust,
                "ts" | "tsx" | "js" | "jsx" => return ProjectType::TypeScript,
                "py" => return ProjectType::Python,
                "go" => return ProjectType::Go,
                _ => {}
            }
        }
    }

    let dir = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };

    // Explicit manifest checks
    if dir.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }
    if dir.join("tsconfig.json").exists()
        || dir.join("package.json").exists()
        || dir.join("deno.json").exists()
        || dir.join("bunfig.toml").exists()
    {
        return ProjectType::TypeScript;
    }
    if dir.join("pyproject.toml").exists()
        || dir.join("requirements.txt").exists()
        || dir.join("setup.py").exists()
        || dir.join("Pipfile").exists()
    {
        return ProjectType::Python;
    }
    if dir.join("go.mod").exists() {
        return ProjectType::Go;
    }

    // Heuristic directory scan for signature source files
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => return ProjectType::Rust,
                    "py" => return ProjectType::Python,
                    "ts" | "tsx" => return ProjectType::TypeScript,
                    "go" => return ProjectType::Go,
                    _ => {}
                }
            }
        }
    }

    ProjectType::Unknown
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

impl DiagnosticLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::Help => "help",
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Standardized polyglot compiler diagnostic representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilerDiagnostic {
    pub file: PathBuf,
    /// 1-based line start
    pub line: usize,
    /// 1-based column start
    pub col: usize,
    /// 1-based line end
    pub end_line: usize,
    /// 1-based column end
    pub end_col: usize,
    pub code: Option<String>,
    pub message: String,
    pub rendered: Option<String>,
    pub suggested_replacement: Option<String>,
    pub level: DiagnosticLevel,
}

/// Options configuring self-healing execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofixOptions {
    pub max_attempts: usize,
    pub include_tests: bool,
    pub dry_run: bool,
    pub backup: bool,
}

impl Default for AutofixOptions {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            include_tests: true,
            dry_run: false,
            backup: true,
        }
    }
}

/// Final summary report of the self-healing cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofixReport {
    pub project_type: ProjectType,
    pub total_diagnostics: usize,
    pub healed_count: usize,
    pub fixes_applied: Vec<String>,
    pub attempts_made: usize,
    pub duration_ms: u128,
    pub is_clean: bool,
}

/// Parses `cargo check --message-format=json` compiler output.
pub fn parse_cargo_json(output: &str) -> Vec<CompilerDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if parsed.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }

        let Some(msg) = parsed.get("message") else {
            continue;
        };

        let level_str = msg.get("level").and_then(|l| l.as_str()).unwrap_or("error");
        let level = match level_str {
            "error" => DiagnosticLevel::Error,
            "warning" => DiagnosticLevel::Warning,
            "note" => DiagnosticLevel::Note,
            "help" => DiagnosticLevel::Help,
            _ => DiagnosticLevel::Error,
        };

        let message_text = msg.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let code = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let rendered = msg.get("rendered").and_then(|r| r.as_str()).map(|s| s.to_string());

        let spans = msg.get("spans").and_then(|s| s.as_array());
        let primary_span = spans.and_then(|arr| {
            arr.iter()
                .find(|s| s.get("is_primary").and_then(|v| v.as_bool()).unwrap_or(false))
                .or_else(|| arr.first())
        });

        let Some(primary) = primary_span else {
            continue;
        };

        let file_name = primary.get("file_name").and_then(|f| f.as_str()).unwrap_or("");
        if file_name.is_empty() {
            continue;
        }

        let mut line_start = primary.get("line_start").and_then(|l| l.as_u64()).unwrap_or(1) as usize;
        let mut line_end = primary.get("line_end").and_then(|l| l.as_u64()).unwrap_or(line_start as u64) as usize;
        let mut col_start = primary.get("column_start").and_then(|c| c.as_u64()).unwrap_or(1) as usize;
        let mut col_end = primary.get("column_end").and_then(|c| c.as_u64()).unwrap_or(col_start as u64) as usize;

        let mut suggested_replacement = primary
            .get("suggested_replacement")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());

        // Check child suggestions (e.g. rustc help message with suggestion span)
        if suggested_replacement.is_none() {
            if let Some(children) = msg.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    if let Some(cspans) = child.get("spans").and_then(|s| s.as_array()) {
                        for cspan in cspans {
                            if let Some(repl) = cspan.get("suggested_replacement").and_then(|r| r.as_str()) {
                                let c_lstart = cspan.get("line_start").and_then(|l| l.as_u64()).map(|v| v as usize).unwrap_or(line_start);
                                let c_lend = cspan.get("line_end").and_then(|l| l.as_u64()).map(|v| v as usize).unwrap_or(line_end);
                                let c_cstart = cspan.get("column_start").and_then(|c| c.as_u64()).map(|v| v as usize).unwrap_or(col_start);
                                let c_cend = cspan.get("column_end").and_then(|c| c.as_u64()).map(|v| v as usize).unwrap_or(col_end);

                                line_start = c_lstart;
                                line_end = c_lend;
                                col_start = c_cstart;
                                col_end = c_cend;
                                suggested_replacement = Some(repl.to_string());
                                break;
                            }
                        }
                    }
                    if suggested_replacement.is_some() {
                        break;
                    }
                }
            }
        }

        // Heuristic fallback for common rustc compiler diagnostics
        if suggested_replacement.is_none() {
            if let Some(ref c) = code {
                if c == "unused_variables" || message_text.contains("unused variable:") {
                    if let Some(ident) = extract_backtick_identifier(&message_text) {
                        suggested_replacement = Some(format!("_{}", ident));
                    }
                } else if c == "unused_mut" || message_text.contains("variable does not need to be mutable") {
                    suggested_replacement = Some(String::new());
                }
            }
        }

        diagnostics.push(CompilerDiagnostic {
            file: PathBuf::from(file_name),
            line: line_start,
            col: col_start,
            end_line: line_end,
            end_col: col_end,
            code,
            message: message_text,
            rendered,
            suggested_replacement,
            level,
        });
    }

    diagnostics
}

/// Parses TypeScript `tsc` compiler diagnostic lines.
pub fn parse_tsc_output(output: &str) -> Vec<CompilerDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Format 1: src/index.ts(12,5): error TS2322: Type 'number' is not assignable to type 'string'.
        if let Some(idx) = trimmed.find("): ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 3..];
            if let Some(paren_open) = left.find('(') {
                let file = &left[..paren_open];
                let coords = &left[paren_open + 1..];
                let mut parts = coords.split(',');
                let line_num = parts.next().and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(1);
                let col_num = parts.next().and_then(|s| s.trim().parse::<usize>().ok()).unwrap_or(1);

                let (level, code, msg) = parse_tsc_message(right);
                diagnostics.push(CompilerDiagnostic {
                    file: PathBuf::from(file),
                    line: line_num,
                    col: col_num,
                    end_line: line_num,
                    end_col: col_num,
                    code,
                    message: msg,
                    rendered: Some(trimmed.to_string()),
                    suggested_replacement: None,
                    level,
                });
                continue;
            }
        }

        // Format 2: src/index.ts:12:5 - error TS2322: Type 'number' is not assignable to type 'string'.
        if let Some(dash_idx) = trimmed.find(" - ") {
            let left = &trimmed[..dash_idx];
            let right = &trimmed[dash_idx + 3..];
            let parts: Vec<&str> = left.split(':').collect();
            if parts.len() >= 3 {
                let file = parts[..parts.len() - 2].join(":");
                let line_num = parts[parts.len() - 2].trim().parse::<usize>().unwrap_or(1);
                let col_num = parts[parts.len() - 1].trim().parse::<usize>().unwrap_or(1);

                let (level, code, msg) = parse_tsc_message(right);
                diagnostics.push(CompilerDiagnostic {
                    file: PathBuf::from(file),
                    line: line_num,
                    col: col_num,
                    end_line: line_num,
                    end_col: col_num,
                    code,
                    message: msg,
                    rendered: Some(trimmed.to_string()),
                    suggested_replacement: None,
                    level,
                });
            }
        }
    }

    diagnostics
}

fn parse_tsc_message(msg: &str) -> (DiagnosticLevel, Option<String>, String) {
    let parts: Vec<&str> = msg.splitn(3, ':').collect();
    if parts.len() >= 2 {
        let tag = parts[0].trim();
        let level = if tag.to_lowercase().starts_with("warning") {
            DiagnosticLevel::Warning
        } else {
            DiagnosticLevel::Error
        };

        let code = tag
            .split_whitespace()
            .find(|tok| tok.starts_with("TS") || tok.starts_with("ts"))
            .map(|s| s.to_string());

        let message = if parts.len() == 3 {
            format!("{}: {}", parts[1].trim(), parts[2].trim())
        } else {
            parts[1].trim().to_string()
        };

        (level, code, message)
    } else {
        (DiagnosticLevel::Error, None, msg.to_string())
    }
}

/// Parses Python `py_compile`, pytest, or traceback diagnostics.
pub fn parse_python_diagnostics(output: &str) -> Vec<CompilerDiagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Format: File "foo.py", line 12
        if let Some(file_idx) = line.find("File \"") {
            let rest = &line[file_idx + 6..];
            if let Some(quote_end) = rest.find('"') {
                let file_path = &rest[..quote_end];
                let after_quote = &rest[quote_end + 1..];

                let mut line_num = 1;
                if let Some(line_idx) = after_quote.find("line ") {
                    let num_str = after_quote[line_idx + 5..]
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap_or("1");
                    line_num = num_str.parse::<usize>().unwrap_or(1);
                }

                // Look ahead for caret col or error message
                let mut col_num = 1;
                let mut error_msg = String::new();
                let mut code = None;

                let mut j = i + 1;
                while j < lines.len() && j < i + 6 {
                    let next_line = lines[j];
                    if let Some(caret_idx) = next_line.find('^') {
                        col_num = caret_idx + 1;
                    }
                    if next_line.contains("SyntaxError:")
                        || next_line.contains("IndentationError:")
                        || next_line.contains("Error:")
                        || next_line.contains("AssertionError:")
                    {
                        error_msg = next_line.trim().to_string();
                        if let Some(colon) = error_msg.find(':') {
                            code = Some(error_msg[..colon].trim().to_string());
                        }
                        break;
                    }
                    j += 1;
                }

                if error_msg.is_empty() {
                    error_msg = line.trim().to_string();
                }

                diagnostics.push(CompilerDiagnostic {
                    file: PathBuf::from(file_path),
                    line: line_num,
                    col: col_num,
                    end_line: line_num,
                    end_col: col_num,
                    code,
                    message: error_msg,
                    rendered: Some(line.to_string()),
                    suggested_replacement: None,
                    level: DiagnosticLevel::Error,
                });
            }
        } else if line.contains("FAILED ") && line.contains(".py::") {
            // Pytest failure format: FAILED tests/test_foo.py::test_bar - AssertionError: ...
            let parts: Vec<&str> = line.split("::").collect();
            if let Some(first) = parts.first() {
                let file = first.replace("FAILED ", "").trim().to_string();
                diagnostics.push(CompilerDiagnostic {
                    file: PathBuf::from(file),
                    line: 1,
                    col: 1,
                    end_line: 1,
                    end_col: 1,
                    code: Some("pytest_failure".to_string()),
                    message: line.trim().to_string(),
                    rendered: Some(line.to_string()),
                    suggested_replacement: None,
                    level: DiagnosticLevel::Error,
                });
            }
        }

        i += 1;
    }

    diagnostics
}

/// Parses Go `go vet` or `go build` diagnostics (e.g. `main.go:5:2: undefined: fmt`).
pub fn parse_go_diagnostics(output: &str) -> Vec<CompilerDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() >= 4 {
            let file = parts[0].trim();
            if file.ends_with(".go") {
                let line_num = parts[1].trim().parse::<usize>().unwrap_or(1);
                let col_num = parts[2].trim().parse::<usize>().unwrap_or(1);
                let msg = parts[3..].join(":").trim().to_string();

                diagnostics.push(CompilerDiagnostic {
                    file: PathBuf::from(file),
                    line: line_num,
                    col: col_num,
                    end_line: line_num,
                    end_col: col_num,
                    code: Some("go_compiler".to_string()),
                    message: msg,
                    rendered: Some(trimmed.to_string()),
                    suggested_replacement: None,
                    level: DiagnosticLevel::Error,
                });
            }
        }
    }

    diagnostics
}

fn extract_backtick_identifier(text: &str) -> Option<String> {
    let first = text.find('`')?;
    let rest = &text[first + 1..];
    let second = rest.find('`')?;
    let ident = rest[..second].trim();
    if !ident.is_empty() {
        Some(ident.to_string())
    } else {
        None
    }
}

/// Applies a surgical replacement to source content at 1-based (line_start, col_start) to (line_end, col_end).
/// Guarantees safe UTF-8 character boundary adjustments with zero unwrap panics.
pub fn apply_span_replacement(
    content: &str,
    line_start: usize,
    col_start: usize,
    line_end: usize,
    col_end: usize,
    replacement: &str,
) -> Option<String> {
    if content.is_empty() {
        if line_start <= 1 && col_start <= 1 {
            return Some(replacement.to_string());
        }
        return None;
    }

    // Collect byte offsets of every line
    let mut line_starts = Vec::new();
    line_starts.push(0);
    for (i, b) in content.bytes().enumerate() {
        if b == b'\n' && i + 1 <= content.len() {
            line_starts.push(i + 1);
        }
    }

    let total_lines = line_starts.len();
    let l_start = line_start.max(1);
    let l_end = line_end.max(l_start);

    if l_start > total_lines {
        return None;
    }

    let line_start_offset = line_starts[l_start - 1];
    let next_line_start = if l_start < total_lines {
        line_starts[l_start]
    } else {
        content.len()
    };
    let line_slice = &content[line_start_offset..next_line_start];

    let c_start_offset = col_to_byte_offset(line_slice, col_start);
    let mut byte_start = line_start_offset + c_start_offset;

    let target_l_end = l_end.min(total_lines);
    let line_end_offset = line_starts[target_l_end - 1];
    let next_end_line_start = if target_l_end < total_lines {
        line_starts[target_l_end]
    } else {
        content.len()
    };
    let end_line_slice = &content[line_end_offset..next_end_line_start];

    let c_end_offset = col_to_byte_offset(end_line_slice, col_end);
    let mut byte_end = line_end_offset + c_end_offset;

    // Safety checks
    if byte_start > content.len() {
        byte_start = content.len();
    }
    if byte_end > content.len() {
        byte_end = content.len();
    }
    if byte_start > byte_end {
        std::mem::swap(&mut byte_start, &mut byte_end);
    }

    // Adjust to nearest UTF-8 character boundaries
    while byte_start > 0 && !content.is_char_boundary(byte_start) {
        byte_start -= 1;
    }
    while byte_end < content.len() && !content.is_char_boundary(byte_end) {
        byte_end += 1;
    }

    // If removing unused mut ("mut" without space), also trim one trailing space if present
    if replacement.is_empty()
        && &content[byte_start..byte_end] == "mut"
        && byte_end < content.len()
        && content.as_bytes()[byte_end] == b' '
    {
        byte_end += 1;
    }

    let mut result = String::with_capacity(content.len() + replacement.len());
    result.push_str(&content[..byte_start]);
    result.push_str(replacement);
    result.push_str(&content[byte_end..]);
    Some(result)
}

fn col_to_byte_offset(line: &str, col_1_based: usize) -> usize {
    if col_1_based <= 1 {
        return 0;
    }
    let target_char_idx = col_1_based - 1;
    let mut current_char = 0;
    for (byte_idx, _) in line.char_indices() {
        if current_char == target_char_idx {
            return byte_idx;
        }
        current_char += 1;
    }
    line.len()
}

/// Heuristically repair Python syntax errors (missing colons on control statements, defs, classes).
pub fn try_heal_python_syntax(content: &str, diag: &CompilerDiagnostic) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if diag.line == 0 || diag.line > lines.len() {
        return None;
    }
    let target_line = lines[diag.line - 1];
    let trimmed = target_line.trim();

    let is_header = trimmed.starts_with("def ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("if ")
        || trimmed.starts_with("elif ")
        || trimmed == "else"
        || trimmed.starts_with("else ")
        || trimmed.starts_with("while ")
        || trimmed.starts_with("for ")
        || trimmed == "try"
        || trimmed.starts_with("try ")
        || trimmed.starts_with("except")
        || trimmed == "finally"
        || trimmed.starts_with("finally ")
        || trimmed.starts_with("with ");

    if is_header && !trimmed.ends_with(':') {
        let (code_part, comment_part) = if let Some(hash_idx) = target_line.find('#') {
            (&target_line[..hash_idx], Some(&target_line[hash_idx..]))
        } else {
            (target_line, None)
        };

        let healed_line = if let Some(comment) = comment_part {
            format!("{}: {}", code_part.trim_end(), comment)
        } else {
            format!("{}:", target_line.trim_end())
        };

        let mut result = Vec::new();
        for (idx, l) in lines.iter().enumerate() {
            if idx == diag.line - 1 {
                result.push(healed_line.as_str());
            } else {
                result.push(*l);
            }
        }
        let mut output = result.join("\n");
        if content.ends_with('\n') {
            output.push('\n');
        }
        return Some(output);
    }

    None
}

/// The Core Self-Healing Compiler Engine (`tgs autofix`).
#[derive(Debug, Default, Clone)]
pub struct AutofixEngine;

impl AutofixEngine {
    pub fn new() -> Self {
        Self
    }

    /// Diagnose compiler errors and test failures in the given project or file.
    pub fn diagnose(&self, path: &Path, include_tests: bool) -> Result<Vec<CompilerDiagnostic>> {
        let project_type = detect_project_type(path);
        let target_dir = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        match project_type {
            ProjectType::Rust => self.diagnose_rust(target_dir, include_tests),
            ProjectType::TypeScript => self.diagnose_typescript(target_dir),
            ProjectType::Python => self.diagnose_python(path, include_tests),
            ProjectType::Go => self.diagnose_go(target_dir),
            ProjectType::Unknown => Ok(Vec::new()),
        }
    }

    fn diagnose_rust(&self, dir: &Path, include_tests: bool) -> Result<Vec<CompilerDiagnostic>> {
        let mut cmd = Command::new("cargo");
        cmd.arg("check").arg("--message-format=json");
        if include_tests {
            cmd.arg("--tests");
        }
        cmd.current_dir(dir);

        let output = cmd.output().map_err(|e| {
            TagisanError::Execution(format!("Failed to invoke 'cargo check': {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut diags = parse_cargo_json(&stdout);

        if diags.is_empty() && !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                diags.push(CompilerDiagnostic {
                    file: dir.join("Cargo.toml"),
                    line: 1,
                    col: 1,
                    end_line: 1,
                    end_col: 1,
                    code: Some("cargo_build_failure".to_string()),
                    message: stderr.trim().to_string(),
                    rendered: Some(stderr.to_string()),
                    suggested_replacement: None,
                    level: DiagnosticLevel::Error,
                });
            }
        }

        Ok(diags)
    }

    fn diagnose_typescript(&self, dir: &Path) -> Result<Vec<CompilerDiagnostic>> {
        let mut cmd = Command::new("tsc");
        cmd.arg("--noEmit").current_dir(dir);

        let output = match cmd.output() {
            Ok(out) => out,
            Err(_) => {
                // Fallback to npx tsc
                let mut npx = Command::new("npx");
                npx.args(["tsc", "--noEmit"]).current_dir(dir);
                npx.output().map_err(|e| {
                    TagisanError::Execution(format!("Failed to execute 'tsc' or 'npx tsc': {}", e))
                })?
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);
        Ok(parse_tsc_output(&combined))
    }

    fn diagnose_python(&self, target: &Path, include_tests: bool) -> Result<Vec<CompilerDiagnostic>> {
        let mut diagnostics = Vec::new();

        if target.is_file() {
            let mut cmd = Command::new("python3");
            cmd.args(["-m", "py_compile"]);
            cmd.arg(target);
            if let Ok(output) = cmd.output() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let combined = format!("{}\n{}", stdout, stderr);
                diagnostics.extend(parse_python_diagnostics(&combined));
            }
            return Ok(diagnostics);
        }

        // Target is directory: scan python files for compilation errors
        let dir = target;
        let mut py_files = Vec::new();
        collect_py_files(dir, &mut py_files, 4);

        for py_file in py_files {
            let mut cmd = Command::new("python3");
            cmd.args(["-m", "py_compile"]).arg(&py_file);
            if let Ok(output) = cmd.output() {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let combined = format!("{}\n{}", stdout, stderr);
                    diagnostics.extend(parse_python_diagnostics(&combined));
                }
            }
        }

        // Optional pytest / unittest discovery if requested and no syntax errors remain
        if include_tests && diagnostics.is_empty() {
            let mut pytest = Command::new("pytest");
            pytest.current_dir(dir);
            if let Ok(output) = pytest.output() {
                if !output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    diagnostics.extend(parse_python_diagnostics(&format!("{}\n{}", stdout, stderr)));
                }
            }
        }

        Ok(diagnostics)
    }

    fn diagnose_go(&self, dir: &Path) -> Result<Vec<CompilerDiagnostic>> {
        let mut cmd = Command::new("go");
        cmd.args(["vet", "./..."]).current_dir(dir);

        let output = cmd.output().map_err(|e| {
            TagisanError::Execution(format!("Failed to invoke 'go vet': {}", e))
        })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);
        let mut diags = parse_go_diagnostics(&combined);

        if diags.is_empty() && !output.status.success() {
            let mut build_cmd = Command::new("go");
            build_cmd.args(["build", "-o", "/dev/null", "./..."]).current_dir(dir);
            if let Ok(build_out) = build_cmd.output() {
                let b_stderr = String::from_utf8_lossy(&build_out.stderr);
                diags.extend(parse_go_diagnostics(&b_stderr));
            }
        }

        Ok(diags)
    }

    /// Autonomously heal compiler diagnostics and test failures in the target path.
    pub fn heal(&self, path: &Path, options: &AutofixOptions) -> Result<AutofixReport> {
        let start_time = std::time::Instant::now();
        let project_type = detect_project_type(path);
        let base_dir = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        let mut attempts_made = 0;
        let mut total_diagnostics = 0;
        let mut healed_count = 0;
        let mut fixes_applied = Vec::new();
        let mut is_clean = false;

        for attempt in 1..=options.max_attempts {
            attempts_made = attempt;
            let diagnostics = self.diagnose(path, options.include_tests)?;

            if attempt == 1 {
                total_diagnostics = diagnostics.len();
            }

            if diagnostics.is_empty() {
                is_clean = true;
                break;
            }

            let mut round_fixes = 0;

            // Group diagnostics by target file
            let mut by_file: HashMap<PathBuf, Vec<CompilerDiagnostic>> = HashMap::new();
            for diag in diagnostics {
                let resolved_file = if diag.file.is_absolute() {
                    diag.file.clone()
                } else {
                    base_dir.join(&diag.file)
                };
                by_file.entry(resolved_file).or_default().push(diag);
            }

            for (file_path, mut diags) in by_file {
                if !file_path.exists() {
                    continue;
                }

                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // Create atomic backup if enabled
                if options.backup && !options.dry_run {
                    let mut bak_path = file_path.as_os_str().to_os_string();
                    bak_path.push(".bak");
                    let _ = fs::copy(&file_path, Path::new(&bak_path));
                }

                // Sort descending by line and column so earlier span coordinates remain valid
                diags.sort_by(|a, b| b.line.cmp(&a.line).then_with(|| b.col.cmp(&a.col)));

                let mut updated_content = content.clone();
                let mut file_healed_this_round = 0;

                for diag in &diags {
                    if let Some(ref repl) = diag.suggested_replacement {
                        if let Some(new_content) = apply_span_replacement(
                            &updated_content,
                            diag.line,
                            diag.col,
                            diag.end_line,
                            diag.end_col,
                            repl,
                        ) {
                            if new_content != updated_content {
                                updated_content = new_content;
                                file_healed_this_round += 1;
                                fixes_applied.push(format!(
                                    "{}:{}:{} [{}] Replaced span with '{}'",
                                    file_path.display(),
                                    diag.line,
                                    diag.col,
                                    diag.code.as_deref().unwrap_or("compiler_fix"),
                                    repl.trim()
                                ));
                            }
                        }
                    } else if project_type == ProjectType::Python {
                        if let Some(new_content) = try_heal_python_syntax(&updated_content, diag) {
                            if new_content != updated_content {
                                updated_content = new_content;
                                file_healed_this_round += 1;
                                fixes_applied.push(format!(
                                    "{}:{} [Python Syntax] Repaired missing syntax colon",
                                    file_path.display(),
                                    diag.line
                                ));
                            }
                        }
                    }
                }

                if file_healed_this_round > 0 {
                    round_fixes += file_healed_this_round;
                    healed_count += file_healed_this_round;

                    if !options.dry_run {
                        fs::write(&file_path, &updated_content).map_err(TagisanError::Io)?;
                    }
                }
            }

            if round_fixes == 0 {
                // Cannot heuristically fix remaining diagnostics, exit loop early
                break;
            }
        }

        // Final verification check
        if !options.dry_run {
            let final_diags = self.diagnose(path, options.include_tests)?;
            is_clean = final_diags.is_empty();
        }

        let duration_ms = start_time.elapsed().as_millis();

        Ok(AutofixReport {
            project_type,
            total_diagnostics,
            healed_count,
            fixes_applied,
            attempts_made,
            duration_ms,
            is_clean,
        })
    }
}

fn collect_py_files(dir: &Path, results: &mut Vec<PathBuf>, max_depth: usize) {
    if max_depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.')
            || name_str == "node_modules"
            || name_str == "venv"
            || name_str == ".venv"
            || name_str == "__pycache__"
            || name_str == "target"
        {
            continue;
        }

        if p.is_dir() {
            collect_py_files(&p, results, max_depth - 1);
        } else if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("py") {
            results.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_project_type() {
        assert_eq!(detect_project_type(Path::new("src/main.rs")), ProjectType::Rust);
        assert_eq!(detect_project_type(Path::new("app.py")), ProjectType::Python);
        assert_eq!(detect_project_type(Path::new("index.ts")), ProjectType::TypeScript);
        assert_eq!(detect_project_type(Path::new("main.go")), ProjectType::Go);
    }

    #[test]
    fn test_parse_cargo_json_unused_var() {
        let cargo_json = r#"{"reason":"compiler-message","package_id":"dummy 0.1.0","target":{"kind":["bin"],"name":"dummy"},"message":{"children":[{"children":[],"code":null,"level":"help","message":"if this is intentional, prefix it with an underscore","rendered":null,"spans":[{"byte_end":45,"byte_start":42,"column_end":12,"column_start":9,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":3,"line_start":3,"suggested_replacement":"_val","suggestion_applicability":"MachineApplicable","text":[{"highlight_end":12,"highlight_start":9,"text":"    let val = 42;"}]}]}],"code":{"code":"unused_variables","explanation":null},"level":"warning","message":"unused variable: `val`","rendered":"warning: unused variable: `val`\n","spans":[{"byte_end":45,"byte_start":42,"column_end":12,"column_start":9,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":3,"line_start":3,"suggested_replacement":null,"suggestion_applicability":null,"text":[{"highlight_end":12,"highlight_start":9,"text":"    let val = 42;"}]}]}}"#;

        let diags = parse_cargo_json(cargo_json);
        assert_eq!(diags.len(), 1);
        let d = &diags[0];
        assert_eq!(d.file, PathBuf::from("src/main.rs"));
        assert_eq!(d.line, 3);
        assert_eq!(d.col, 9);
        assert_eq!(d.end_col, 12);
        assert_eq!(d.code.as_deref(), Some("unused_variables"));
        assert_eq!(d.suggested_replacement.as_deref(), Some("_val"));
    }

    #[test]
    fn test_apply_span_replacement() {
        let code = "fn main() {\n    let val = 42;\n}\n";
        // replace val (line 2, col 9 to 12) with _val
        let healed = apply_span_replacement(code, 2, 9, 2, 12, "_val");
        assert_eq!(healed, Some("fn main() {\n    let _val = 42;\n}\n".to_string()));
    }

    #[test]
    fn test_parse_tsc_output() {
        let tsc = "src/index.ts(14,5): error TS2322: Type 'string' is not assignable to type 'number'.\n";
        let diags = parse_tsc_output(tsc);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file, PathBuf::from("src/index.ts"));
        assert_eq!(diags[0].line, 14);
        assert_eq!(diags[0].col, 5);
        assert_eq!(diags[0].code.as_deref(), Some("TS2322"));
    }

    #[test]
    fn test_parse_python_diagnostics() {
        let py_err = "  File \"test_math.py\", line 4\n    def broken_func()\n                    ^\nSyntaxError: expected ':'\n";
        let diags = parse_python_diagnostics(py_err);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].file, PathBuf::from("test_math.py"));
        assert_eq!(diags[0].line, 4);
        assert_eq!(diags[0].code.as_deref(), Some("SyntaxError"));
    }

    #[test]
    fn test_try_heal_python_syntax() {
        let code = "def greet(name)\n    print(name)\n";
        let diag = CompilerDiagnostic {
            file: PathBuf::from("test.py"),
            line: 1,
            col: 16,
            end_line: 1,
            end_col: 16,
            code: Some("SyntaxError".to_string()),
            message: "SyntaxError: expected ':'".to_string(),
            rendered: None,
            suggested_replacement: None,
            level: DiagnosticLevel::Error,
        };
        let fixed = try_heal_python_syntax(code, &diag);
        assert_eq!(fixed, Some("def greet(name):\n    print(name)\n".to_string()));
    }
}
