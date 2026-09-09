use super::ToolHandler;
use crate::error::{Result, TagisanError};
use crate::types::ContentBlock;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
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
