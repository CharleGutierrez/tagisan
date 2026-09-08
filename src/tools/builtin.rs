use super::ToolHandler;
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

// =========================================================================
// 1. ReadFileTool
// =========================================================================

/// Tool for reading file contents safely from local disk
#[derive(Debug, Default, Clone)]
pub struct ReadFileTool;

impl ReadFileTool {
    pub fn new() -> Self {
        Self
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
        if !path.exists() {
            return Err(TagisanError::Execution(format!("File does not exist: {path_str}")));
        }

        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| TagisanError::Execution(format!("Failed to read file '{path_str}': {e}")))
    }
}

// =========================================================================
// 2. WriteFileTool
// =========================================================================

/// Tool for writing/creating files safely on the local filesystem
#[derive(Debug, Default, Clone)]
pub struct WriteFileTool;

impl WriteFileTool {
    pub fn new() -> Self {
        Self
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
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    TagisanError::Execution(format!("Failed to create parent directory '{parent:?}': {e}"))
                })?;
            }
        }

        tokio::fs::write(path, content)
            .await
            .map_err(|e| TagisanError::Execution(format!("Failed to write to file '{path_str}': {e}")))?;

        Ok(format!(
            "Successfully wrote {} bytes to '{}'.",
            content.len(),
            path_str
        ))
    }
}

// =========================================================================
// 3. RunCommandTool
// =========================================================================

/// Tool for executing shell commands with timeout and output capture
#[derive(Debug, Clone)]
pub struct RunCommandTool {
    pub default_timeout: Duration,
}

impl Default for RunCommandTool {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
        }
    }
}

impl RunCommandTool {
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            default_timeout: timeout_duration,
        }
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
