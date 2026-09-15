use crate::ecc::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::Result;
use crate::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, RequestId};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::RwLock;

pub const LSP_PROTOCOL_VERSION: &str = "3.17";
pub const LSP_SERVER_NAME: &str = "tagisan-lsp";
pub const LSP_SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Decodes percent-encoded characters in a URI string
pub fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(c1), Some(c2)) = (h1, h2) {
                let hex_str = format!("{}{}", c1, c2);
                if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                    out.push(byte as char);
                    continue;
                }
                out.push('%');
                out.push(c1);
                out.push(c2);
            } else {
                out.push('%');
                if let Some(c1) = h1 {
                    out.push(c1);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Converts an LSP document URI (`file:///...`) into a normalized `PathBuf`
pub fn uri_to_path(uri: &str) -> PathBuf {
    let decoded = percent_decode(uri);
    if let Some(stripped) = decoded.strip_prefix("file:///") {
        // Windows drive letters like C:/...
        if stripped.len() >= 2 && stripped.chars().nth(1) == Some(':') {
            PathBuf::from(stripped)
        } else {
            PathBuf::from(format!("/{}", stripped))
        }
    } else if let Some(stripped) = decoded.strip_prefix("file://") {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(decoded)
    }
}

/// Language Server Protocol (LSP) Engine for IDEs and Editor Integrations
pub struct LspServer {
    documents: Arc<RwLock<HashMap<String, String>>>,
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
}

impl Default for LspServer {
    fn default() -> Self {
        Self::new()
    }
}

impl LspServer {
    /// Create a new instance of the Tagisan LSP Server
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            workspace_root: Arc::new(RwLock::new(None)),
        }
    }

    /// Retrieve in-memory document content or read from disk
    pub async fn get_document_content(&self, uri: &str) -> String {
        {
            let docs = self.documents.read().await;
            if let Some(content) = docs.get(uri) {
                return content.clone();
            }
        }

        let path = uri_to_path(uri);
        if path.exists() {
            std::fs::read_to_string(&path).unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// Extract word/token at given 0-indexed line and character
    pub async fn extract_token_at(&self, uri: &str, line_idx: usize, char_idx: usize) -> Option<String> {
        let content = self.get_document_content(uri).await;
        if content.is_empty() {
            return None;
        }

        let lines: Vec<&str> = content.lines().collect();
        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];
        if line.is_empty() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        let target_pos = char_idx.min(chars.len().saturating_sub(1));

        let is_ident_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';

        let mut start = target_pos;
        while start > 0 && is_ident_char(chars[start - 1]) {
            start -= 1;
        }

        let mut end = target_pos;
        while end < chars.len() && is_ident_char(chars[end]) {
            end += 1;
        }

        if start < end {
            let token: String = chars[start..end].iter().collect();
            if !token.trim().is_empty() {
                return Some(token);
            }
        }

        None
    }

    /// Build rich Markdown hover documentation querying skills and AST symbol knowledge
    pub async fn build_hover_documentation(&self, query: &str) -> String {
        let clean_query = query.trim();
        if clean_query.is_empty() {
            return "### 🛡️ Tagisan IDE Integration\nAgentic AI & Language Server Protocol active.".to_string();
        }

        let dispatcher = crate::ecc::skills::global_dispatcher();

        // 1. Exact or normalized skill name lookup from the 100 Agentic and 50 Architecture skills
        if let Some(skill) = dispatcher.get_skill(clean_query) {
            let mut md = format!("### 🧠 Tagisan Agentic Skill: `{}`\n\n", skill.name);
            md.push_str(&format!("{}\n\n", skill.description));
            if !skill.instructions.is_empty() {
                let preview: String = skill
                    .instructions
                    .lines()
                    .take(12)
                    .collect::<Vec<_>>()
                    .join("\n");
                md.push_str(&format!("#### 📋 Operational Guidelines:\n```markdown\n{}\n```\n", preview));
            }
            return md;
        }

        // 2. High-speed TF-IDF dispatch matching against the skill catalog
        let dispatched = dispatcher.dispatch(clean_query, 2, None);
        if let Some(first) = dispatched.first() {
            let mut md = format!("### 🧠 Tagisan Skill: `{}` (Relevance: {:.2})\n\n", first.skill.name, first.score);
            md.push_str(&format!("**Domain:** `{}`\n\n", first.domain));
            md.push_str(&format!("{}\n\n", first.skill.description));
            if !first.matched_triggers.is_empty() {
                md.push_str(&format!("**Matched Triggers:** `{}`\n\n", first.matched_triggers.join("`, `")));
            }
            if !first.skill.instructions.is_empty() {
                let preview: String = first
                    .skill
                    .instructions
                    .lines()
                    .take(8)
                    .collect::<Vec<_>>()
                    .join("\n");
                md.push_str(&format!("#### 📋 Instructions Snippet:\n```markdown\n{}\n```\n", preview));
            }
            return md;
        }

        // 3. Fallback contextual Tagisan AST & Agent information
        format!(
            "### 🛡️ Tagisan IDE Integration\n**Symbol:** `{}`\n\nLanguage Server Protocol & Agentic Engine connected.\nUse code action `Tagisan: Run Dialectical Debate` or `tgs ground` for formal verification.",
            clean_query
        )
    }

    /// Process a single incoming JSON-RPC 2.0 LSP request and return response if required
    pub async fn handle_lsp_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        match req.method.as_str() {
            "initialize" => {
                let root_path = req.params.as_ref().and_then(|p| {
                    p.get("rootUri")
                        .and_then(|u| u.as_str())
                        .map(uri_to_path)
                        .or_else(|| p.get("rootPath").and_then(|r| r.as_str()).map(PathBuf::from))
                });

                if let Some(r) = root_path {
                    let verdict = AgentShieldScanner::scan_file_path(&r.to_string_lossy());
                    if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
                        return Some(JsonRpcResponse::error(
                            Some(req.id),
                            -32603,
                            format!("AgentShield blocked workspace root [{:?}]: {}", threat_level, reason),
                        ));
                    }
                    let mut lock = self.workspace_root.write().await;
                    *lock = Some(r);
                }

                let result = json!({
                    "capabilities": {
                        "textDocumentSync": 1,
                        "hoverProvider": true,
                        "codeActionProvider": true,
                        "diagnosticProvider": {
                            "interFileDependencies": false,
                            "workspaceDiagnostics": false
                        }
                    },
                    "serverInfo": {
                        "name": LSP_SERVER_NAME,
                        "version": LSP_SERVER_VERSION
                    }
                });
                Some(JsonRpcResponse::success(req.id, result))
            }

            "initialized" => None,

            "shutdown" => Some(JsonRpcResponse::success(req.id, json!(null))),

            "textDocument/didOpen" => {
                if let Some(params) = req.params {
                    if let Some(doc) = params.get("textDocument") {
                        if let (Some(uri), Some(text)) = (
                            doc.get("uri").and_then(|v| v.as_str()),
                            doc.get("text").and_then(|v| v.as_str()),
                        ) {
                            let mut docs = self.documents.write().await;
                            docs.insert(uri.to_string(), text.to_string());
                        }
                    }
                }
                None
            }

            "textDocument/didChange" => {
                if let Some(params) = req.params {
                    if let Some(uri) = params.get("textDocument").and_then(|d| d.get("uri")).and_then(|v| v.as_str()) {
                        if let Some(changes) = params.get("contentChanges").and_then(|c| c.as_array()) {
                            if let Some(last_change) = changes.last().and_then(|c| c.get("text")).and_then(|v| v.as_str()) {
                                let mut docs = self.documents.write().await;
                                docs.insert(uri.to_string(), last_change.to_string());
                            }
                        }
                    }
                }
                None
            }

            "textDocument/hover" => {
                let params = req.params.unwrap_or(json!({}));
                let uri = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !uri.is_empty() {
                    let path = uri_to_path(uri);
                    let verdict = AgentShieldScanner::scan_file_path(&path.to_string_lossy());
                    if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
                        return Some(JsonRpcResponse::error(
                            Some(req.id),
                            -32603,
                            format!("AgentShield blocked document hover [{:?}]: {}", threat_level, reason),
                        ));
                    }
                }

                let direct_word = params.get("word").or_else(|| params.get("query")).and_then(|v| v.as_str());
                let word = if let Some(w) = direct_word {
                    w.to_string()
                } else {
                    let line = params.get("position").and_then(|p| p.get("line")).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let character = params.get("position").and_then(|p| p.get("character")).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    self.extract_token_at(uri, line, character)
                        .await
                        .unwrap_or_else(|| "tagisan".to_string())
                };

                let hover_markdown = self.build_hover_documentation(&word).await;
                let result = json!({
                    "contents": {
                        "kind": "markdown",
                        "value": hover_markdown
                    }
                });
                Some(JsonRpcResponse::success(req.id, result))
            }

            "textDocument/codeAction" => {
                let params = req.params.unwrap_or(json!({}));
                let uri = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !uri.is_empty() {
                    let path = uri_to_path(uri);
                    let verdict = AgentShieldScanner::scan_file_path(&path.to_string_lossy());
                    if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
                        return Some(JsonRpcResponse::error(
                            Some(req.id),
                            -32603,
                            format!("AgentShield blocked codeAction [{:?}]: {}", threat_level, reason),
                        ));
                    }
                }

                let actions = json!([
                    {
                        "title": "Tagisan: Refactor with Clean Architecture",
                        "kind": "quickfix",
                        "isPreferred": true,
                        "command": {
                            "title": "Tagisan: Refactor with Clean Architecture",
                            "command": "tagisan.refactor",
                            "arguments": [uri]
                        }
                    },
                    {
                        "title": "Tagisan: Run Dialectical Debate",
                        "kind": "refactor",
                        "command": {
                            "title": "Tagisan: Run Dialectical Debate",
                            "command": "tagisan.debate",
                            "arguments": [uri]
                        }
                    },
                    {
                        "title": "Tagisan: Verify Invariants",
                        "kind": "source.fixAll",
                        "command": {
                            "title": "Tagisan: Verify Invariants",
                            "command": "tagisan.verify",
                            "arguments": [uri]
                        }
                    },
                    {
                        "title": "Tagisan: ECC Multi-Agent Engineering Pipeline",
                        "kind": "refactor.rewrite",
                        "command": {
                            "title": "Tagisan: ECC Multi-Agent Engineering Pipeline",
                            "command": "tagisan.ecc_pipeline",
                            "arguments": [uri]
                        }
                    },
                    {
                        "title": "Tagisan: AgentShield Cyber Defense Audit",
                        "kind": "quickfix",
                        "command": {
                            "title": "Tagisan: AgentShield Cyber Defense Audit",
                            "command": "tagisan.shield_scan",
                            "arguments": [uri]
                        }
                    }
                ]);

                Some(JsonRpcResponse::success(req.id, actions))
            }

            "textDocument/diagnostic" => {
                let params = req.params.unwrap_or(json!({}));
                let uri = params
                    .get("textDocument")
                    .and_then(|d| d.get("uri"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                let mut items = Vec::new();

                if !uri.is_empty() {
                    let path = uri_to_path(uri);
                    let path_str = path.to_string_lossy();
                    let path_verdict = AgentShieldScanner::scan_file_path(&path_str);

                    if let AgentShieldVerdict::Block { reason, threat_level } = path_verdict {
                        items.push(json!({
                            "range": {
                                "start": { "line": 0, "character": 0 },
                                "end": { "line": 0, "character": 80 }
                            },
                            "severity": 1,
                            "source": "AgentShield",
                            "message": format!("🚨 AgentShield Cyber Defense: Access blocked [{:?}]: {}", threat_level, reason)
                        }));
                    } else {
                        let content = self.get_document_content(uri).await;
                        if !content.is_empty() {
                            let code_verdict = AgentShieldScanner::scan_code(&content);
                            if let AgentShieldVerdict::Block { reason, threat_level } = code_verdict {
                                items.push(json!({
                                    "range": {
                                        "start": { "line": 0, "character": 0 },
                                        "end": { "line": 0, "character": 80 }
                                    },
                                    "severity": 1,
                                    "source": "AgentShield",
                                    "message": format!("🚨 AgentShield Code Safety Violation [{:?}]: {}", threat_level, reason)
                                }));
                            }

                            let pi_verdict = AgentShieldScanner::scan_prompt_injection(&content);
                            if let AgentShieldVerdict::Block { reason, threat_level } = pi_verdict {
                                items.push(json!({
                                    "range": {
                                        "start": { "line": 0, "character": 0 },
                                        "end": { "line": 0, "character": 80 }
                                    },
                                    "severity": 2,
                                    "source": "AgentShield",
                                    "message": format!("⚠️ AgentShield Prompt Injection Indicator [{:?}]: {}", threat_level, reason)
                                }));
                            }
                        }
                    }
                }

                let result = json!({
                    "kind": "full",
                    "items": items
                });
                Some(JsonRpcResponse::success(req.id, result))
            }

            unknown => Some(JsonRpcResponse::error(
                Some(req.id),
                -32601,
                format!("Method not found: '{unknown}'"),
            )),
        }
    }

    /// Run the LSP standard I/O loop processing messages from `reader` and writing to `writer`
    pub async fn run_stdio<R, W>(&self, mut reader: R, mut writer: W) -> Result<()>
    where
        R: AsyncBufRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut line_buf = String::new();

        loop {
            line_buf.clear();
            let bytes_read = reader.read_line(&mut line_buf).await?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line_buf.trim();
            if trimmed.is_empty() {
                continue;
            }

            let payload = if trimmed.to_lowercase().starts_with("content-length:") {
                let len_str = trimmed[15..].trim();
                let content_len: usize = match len_str.parse() {
                    Ok(l) => l,
                    Err(_) => continue,
                };

                // Read headers until empty separator line
                loop {
                    line_buf.clear();
                    let h_read = reader.read_line(&mut line_buf).await?;
                    if h_read == 0 || line_buf.trim().is_empty() {
                        break;
                    }
                }

                let mut body = vec![0u8; content_len];
                tokio::io::AsyncReadExt::read_exact(&mut reader, &mut body).await?;
                String::from_utf8_lossy(&body).to_string()
            } else {
                trimmed.to_string()
            };

            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&payload) {
                if let Some(resp) = self.handle_lsp_request(req).await {
                    let resp_str = serde_json::to_string(&resp)?;
                    let framed = format!("Content-Length: {}\r\n\r\n{}", resp_str.len(), resp_str);
                    writer.write_all(framed.as_bytes()).await?;
                    writer.flush().await?;
                }
            }
        }

        Ok(())
    }

    /// Run the default standard I/O server loop reading stdin and writing to stdout
    pub async fn run_default_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let reader = tokio::io::BufReader::new(stdin);
        self.run_stdio(reader, stdout).await
    }
}
