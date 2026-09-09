use serde::{Deserialize, Serialize};

/// Severity level of a TypeScript compiler or Bun runtime diagnostic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// Structured diagnostic parsed from Bun or TypeScript compiler output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TsDiagnostic {
    /// TypeScript error code (e.g. "TS2322", "TS2304", "TS2345")
    pub code: Option<String>,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Path to the file containing the error
    pub file: Option<String>,
    /// Line number (1-based)
    pub line: Option<usize>,
    /// Column number (1-based)
    pub column: Option<usize>,
    /// Human-readable diagnostic description
    pub message: String,
    /// Code snippet exhibiting the diagnostic
    pub snippet: Option<String>,
    /// Recommended automated repair hint
    pub repair_hint: Option<String>,
}

/// Parser for extracting structured diagnostics from compiler/runtime stderr outputs
pub struct TsDiagnosticParser;

impl TsDiagnosticParser {
    /// Parses Bun runtime or TypeScript compiler error text into structured diagnostics
    pub fn parse(raw_output: &str) -> Vec<TsDiagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = raw_output.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Pattern 1: `path/to/file.ts:line:col - (error|warning|info) TS2322: message`
            let level_match = if let Some(pos) = line.find(" - error ") {
                Some((pos, 9, DiagnosticSeverity::Error))
            } else if let Some(pos) = line.find(" - warning ") {
                Some((pos, 11, DiagnosticSeverity::Warning))
            } else if let Some(pos) = line.find(" - info ") {
                Some((pos, 8, DiagnosticSeverity::Info))
            } else {
                None
            };

            if let Some((pos, len, severity)) = level_match {
                let prefix = &line[..pos];
                let rest = &line[pos + len..];

                let (file, line_num, col_num) = Self::parse_file_line_col(prefix);
                let (code, message) = Self::parse_code_and_message(rest);

                // Collect following snippet lines if present
                let mut snippet_lines = Vec::new();
                let mut j = i + 1;
                while j < lines.len()
                    && !lines[j].contains(" - error ")
                    && !lines[j].contains(" - warning ")
                    && !lines[j].contains(" - info ")
                    && !lines[j].starts_with("error:")
                {
                    let s_line = lines[j];
                    if s_line.trim().is_empty() && !snippet_lines.is_empty() {
                        break;
                    }
                    if !s_line.trim().is_empty() {
                        snippet_lines.push(s_line);
                    }
                    j += 1;
                }

                let snippet = if snippet_lines.is_empty() {
                    None
                } else {
                    Some(snippet_lines.join("\n"))
                };

                let repair_hint = code.as_deref().map(Self::suggest_repair);

                diagnostics.push(TsDiagnostic {
                    code,
                    severity,
                    file,
                    line: line_num,
                    column: col_num,
                    message,
                    snippet,
                    repair_hint,
                });

                i = j;
                continue;
            }

            // Pattern 2: `error: message [TS2322]`
            if line.starts_with("error:") || line.starts_with("TypeError:") || line.starts_with("SyntaxError:") {
                let rest = line.trim_start_matches("error:").trim();
                let (code, message) = Self::parse_bracketed_code(rest);

                let mut file = None;
                let mut line_num = None;
                let mut col_num = None;
                let mut snippet_lines = Vec::new();

                // Look ahead for `--> file:line:col`
                let mut j = i + 1;
                while j < lines.len() && j <= i + 5 {
                    let next_line = lines[j].trim();
                    if next_line.starts_with("-->") {
                        let path_part = next_line.trim_start_matches("-->").trim();
                        let (f, l, c) = Self::parse_file_line_col(path_part);
                        file = f;
                        line_num = l;
                        col_num = c;
                    } else if next_line.starts_with('|') || next_line.contains('|') {
                        snippet_lines.push(lines[j]);
                    } else if next_line.starts_with("error:") {
                        break;
                    }
                    j += 1;
                }

                let snippet = if snippet_lines.is_empty() {
                    None
                } else {
                    Some(snippet_lines.join("\n"))
                };

                let repair_hint = code.as_deref().map(Self::suggest_repair);

                diagnostics.push(TsDiagnostic {
                    code,
                    severity: DiagnosticSeverity::Error,
                    file,
                    line: line_num,
                    column: col_num,
                    message,
                    snippet,
                    repair_hint,
                });

                i = j.max(i + 1);
                continue;
            }

            i += 1;
        }

        // Fallback: if no patterns matched but output contains error indicators
        if diagnostics.is_empty() && (raw_output.contains("error") || raw_output.contains("Error")) {
            diagnostics.push(TsDiagnostic {
                code: None,
                severity: DiagnosticSeverity::Error,
                file: None,
                line: None,
                column: None,
                message: raw_output.trim().to_string(),
                snippet: None,
                repair_hint: Some("Review syntax and module exports".to_string()),
            });
        }

        diagnostics
    }

    /// Evaluates or type-checks code using Bun and returns any parsed diagnostics
    pub async fn check_and_parse(code: &str) -> crate::error::Result<Vec<TsDiagnostic>> {
        let runtime = crate::bun::runtime::BunRuntime::default();
        let res = runtime.eval(code, std::time::Duration::from_secs(5), None, None).await?;
        if res.is_success() {
            Ok(Vec::new())
        } else {
            Ok(Self::parse(&res.stderr))
        }
    }

    /// Generates structured repair context and guidance for Autonomous Agent self-healing loops
    pub fn generate_self_healing_prompt(diagnostics: &[TsDiagnostic], original_code: &str) -> String {
        let mut prompt = String::new();
        prompt.push_str("## 🛠️ TypeScript Self-Healing Compilation Report\n\n");
        prompt.push_str("The executed TypeScript code failed compilation. Please repair the code based on the following diagnostic details:\n\n");

        for (idx, diag) in diagnostics.iter().enumerate() {
            prompt.push_str(&format!("### Error #{}: {}\n", idx + 1, diag.code.as_deref().unwrap_or("CompilerError")));
            prompt.push_str(&format!("- **Message**: {}\n", diag.message));
            if let Some(ref f) = diag.file {
                prompt.push_str(&format!("- **File**: `{f}`"));
                if let Some(l) = diag.line {
                    prompt.push_str(&format!(" (Line {l}"));
                    if let Some(c) = diag.column {
                        prompt.push_str(&format!(", Col {c}"));
                    }
                    prompt.push(')');
                }
                prompt.push('\n');
            }
            if let Some(ref hint) = diag.repair_hint {
                prompt.push_str(&format!("- **Surgical Action**: {}\n", hint));
            }
            if let Some(ref snippet) = diag.snippet {
                prompt.push_str(&format!("```typescript\n{}\n```\n", snippet));
            }
            prompt.push('\n');
        }

        prompt.push_str("### Original Code:\n```typescript\n");
        prompt.push_str(original_code.trim());
        prompt.push_str("\n```\n\n");
        prompt.push_str("### Instructions:\n");
        prompt.push_str("1. Correct the type mismatch, declaration, or import issue identified above.\n");
        prompt.push_str("2. Return only the complete, corrected TypeScript code in a ```typescript fenced code block.\n");

        prompt
    }

    fn parse_file_line_col(s: &str) -> (Option<String>, Option<usize>, Option<usize>) {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() >= 3 {
            let file = parts[..parts.len() - 2].join(":");
            let line = parts[parts.len() - 2].parse().ok();
            let col = parts[parts.len() - 1].parse().ok();
            (Some(file), line, col)
        } else if parts.len() == 2 {
            let file = parts[0].to_string();
            let line = parts[1].parse().ok();
            (Some(file), line, None)
        } else if !s.trim().is_empty() {
            (Some(s.trim().to_string()), None, None)
        } else {
            (None, None, None)
        }
    }

    fn parse_code_and_message(s: &str) -> (Option<String>, String) {
        let trimmed = s.trim();
        if trimmed.starts_with("TS") {
            if let Some(colon_idx) = trimmed.find(':') {
                let code = trimmed[..colon_idx].trim().to_string();
                let msg = trimmed[colon_idx + 1..].trim().to_string();
                return (Some(code), msg);
            }
        }
        (None, trimmed.to_string())
    }

    fn parse_bracketed_code(s: &str) -> (Option<String>, String) {
        if let Some(open) = s.rfind('[') {
            if let Some(close) = s[open..].find(']') {
                let code_cand = &s[open + 1..open + close];
                if code_cand.starts_with("TS") {
                    let msg = s[..open].trim().to_string();
                    return (Some(code_cand.to_string()), msg);
                }
            }
        }
        (None, s.to_string())
    }

    fn suggest_repair(code: &str) -> String {
        match code {
            "TS2322" => "Type mismatch: align the assigned value's type with the target variable, or perform explicit casting ('as TargetType').".to_string(),
            "TS2304" => "Name resolution failure: add missing import statement or declare the identifier in scope.".to_string(),
            "TS2345" => "Argument type mismatch: check function signature parameter types and convert supplied argument.".to_string(),
            "TS2339" => "Property missing: verify interface/type declaration and add missing field or use optional chaining (?.).".to_string(),
            "TS2741" => "Missing required properties in object literal: supply all non-optional interface fields.".to_string(),
            "TS7006" => "Implicit 'any' parameter: add explicit type annotation to function parameter.".to_string(),
            "TS1378" => "Top-level await issue: ensure module format is ESM ('export {}' or 'module': 'esnext').".to_string(),
            _ => format!("Review TypeScript error code '{code}' and refine type annotations."),
        }
    }
}
