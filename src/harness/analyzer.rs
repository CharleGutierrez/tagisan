use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TagisanError};
use crate::harness::spec::SourceType;

/// Detailed information about a parameter in a function or method signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

impl ParameterInfo {
    pub fn new(name: impl Into<String>, param_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            description: None,
            required: true,
            default_value: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_default(mut self, default_val: impl Into<String>) -> Self {
        self.default_value = Some(default_val.into());
        self.required = false;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// Extracted function signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub docstring: Option<String>,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub is_method: bool,
    pub line_number: usize,
}

impl FunctionSignature {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            docstring: None,
            parameters: Vec::new(),
            return_type: None,
            is_async: false,
            is_method: false,
            line_number: 1,
        }
    }
}

/// Extracted class signature with its methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassSignature {
    pub name: String,
    pub docstring: Option<String>,
    pub methods: Vec<FunctionSignature>,
    pub line_number: usize,
}

impl ClassSignature {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            docstring: None,
            methods: Vec::new(),
            line_number: 1,
        }
    }
}

/// The comprehensive analysis result of a source codebase or API specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppAnalysis {
    pub source_name: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub module_docstring: Option<String>,
    pub functions: Vec<FunctionSignature>,
    pub classes: Vec<ClassSignature>,
    pub raw_content: String,
    pub metadata: HashMap<String, String>,
}

impl AppAnalysis {
    pub fn new(
        source_name: impl Into<String>,
        source_path: impl Into<PathBuf>,
        source_type: SourceType,
    ) -> Self {
        Self {
            source_name: source_name.into(),
            source_path: source_path.into(),
            source_type,
            module_docstring: None,
            functions: Vec::new(),
            classes: Vec::new(),
            raw_content: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Returns total number of callable functions and methods extracted
    pub fn total_callables(&self) -> usize {
        let method_count: usize = self.classes.iter().map(|c| c.methods.len()).sum();
        self.functions.len() + method_count
    }
}

/// Production-grade code analyzer scanning source files and extracting callable signatures
pub struct CodeAnalyzer;

impl CodeAnalyzer {
    /// Detects source type from file extension or content heuristics
    pub fn detect_source_type(path: &Path, content: Option<&str>) -> SourceType {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "py" | "pyi" => return SourceType::Python,
                "rs" => return SourceType::Rust,
                "js" | "mjs" | "cjs" => return SourceType::JavaScript,
                "ts" | "mts" | "cts" => return SourceType::TypeScript,
                "json" | "yaml" | "yml" => {
                    if let Some(c) = content {
                        if c.contains("openapi") || c.contains("swagger") || c.contains("\"paths\":") || c.contains("paths:") {
                            return SourceType::OpenApi;
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(c) = content {
            let sample = &c[..c.len().min(2048)];
            if sample.contains("\"openapi\"") || sample.contains("openapi:") || sample.contains("swagger:") {
                return SourceType::OpenApi;
            }
            if sample.contains("def ") || sample.contains("import ") && sample.contains("from ") {
                return SourceType::Python;
            }
            if sample.contains("fn ") || sample.contains("pub fn ") || sample.contains("use std::") {
                return SourceType::Rust;
            }
            if sample.contains("function ") || sample.contains("const ") || sample.contains("export ") {
                if sample.contains(": string") || sample.contains(": number") || sample.contains("interface ") {
                    return SourceType::TypeScript;
                }
                return SourceType::JavaScript;
            }
        }

        SourceType::Python
    }

    /// Analyzes a single file on disk
    pub fn analyze_file(path: impl AsRef<Path>) -> Result<AppAnalysis> {
        let p = path.as_ref();
        if !p.is_file() {
            return Err(TagisanError::Execution(format!(
                "Path is not a valid file: {}",
                p.display()
            )));
        }

        let content = fs::read_to_string(p).map_err(|e| {
            TagisanError::Execution(format!("Failed to read file '{}': {}", p.display(), e))
        })?;

        let lang = Self::detect_source_type(p, Some(&content));
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string();

        Self::analyze_source(&content, lang, Some(&stem), Some(p))
    }

    /// Analyzes a directory recursively, combining analysis across files
    pub fn analyze_dir(dir_path: impl AsRef<Path>) -> Result<AppAnalysis> {
        let root = dir_path.as_ref();
        if !root.is_dir() {
            return Err(TagisanError::Execution(format!(
                "Path is not a valid directory: {}",
                root.display()
            )));
        }

        let dir_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        let mut combined = AppAnalysis::new(dir_name.clone(), root.to_path_buf(), SourceType::Auto);

        // Find primary files
        let mut py_files = Vec::new();
        let mut rs_files = Vec::new();
        let mut ts_files = Vec::new();
        let mut js_files = Vec::new();
        let mut openapi_files = Vec::new();

        Self::collect_source_files(root, &mut py_files, &mut rs_files, &mut ts_files, &mut js_files, &mut openapi_files)?;

        // Determine dominant language
        let (dominant_lang, target_files) = if !openapi_files.is_empty() {
            (SourceType::OpenApi, openapi_files)
        } else if py_files.len() >= rs_files.len() && py_files.len() >= ts_files.len() && py_files.len() >= js_files.len() && !py_files.is_empty() {
            (SourceType::Python, py_files)
        } else if rs_files.len() >= ts_files.len() && !rs_files.is_empty() {
            (SourceType::Rust, rs_files)
        } else if !ts_files.is_empty() {
            (SourceType::TypeScript, ts_files)
        } else if !js_files.is_empty() {
            (SourceType::JavaScript, js_files)
        } else {
            (SourceType::Python, Vec::new())
        };

        combined.source_type = dominant_lang;

        for file_path in target_files {
            if let Ok(file_analysis) = Self::analyze_file(&file_path) {
                if combined.module_docstring.is_none() && file_analysis.module_docstring.is_some() {
                    combined.module_docstring = file_analysis.module_docstring;
                }
                combined.functions.extend(file_analysis.functions);
                combined.classes.extend(file_analysis.classes);
            }
        }

        Ok(combined)
    }

    fn collect_source_files(
        dir: &Path,
        py: &mut Vec<PathBuf>,
        rs: &mut Vec<PathBuf>,
        ts: &mut Vec<PathBuf>,
        js: &mut Vec<PathBuf>,
        openapi: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            TagisanError::Execution(format!("Failed to read directory '{}': {}", dir.display(), e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "__pycache__" || name == "venv" || name == ".venv" {
                continue;
            }

            if path.is_dir() {
                let _ = Self::collect_source_files(&path, py, rs, ts, js, openapi);
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "py" => py.push(path),
                        "rs" => rs.push(path),
                        "ts" => ts.push(path),
                        "js" => js.push(path),
                        "json" | "yaml" | "yml" => {
                            if name.contains("openapi") || name.contains("swagger") {
                                openapi.push(path);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyzes source code content string directly
    pub fn analyze_source(
        source: &str,
        lang: SourceType,
        name: Option<&str>,
        path: Option<&Path>,
    ) -> Result<AppAnalysis> {
        let source_name = name.unwrap_or("app").to_string();
        let source_path = path.map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("app.py"));

        let mut analysis = AppAnalysis::new(source_name, source_path, lang);
        analysis.raw_content = source.to_string();

        match lang {
            SourceType::Python => {
                let (module_doc, functions, classes) = Self::parse_python(source);
                analysis.module_docstring = module_doc;
                analysis.functions = functions;
                analysis.classes = classes;
            }
            SourceType::Rust => {
                let (module_doc, functions, classes) = Self::parse_rust(source);
                analysis.module_docstring = module_doc;
                analysis.functions = functions;
                analysis.classes = classes;
            }
            SourceType::JavaScript | SourceType::TypeScript => {
                let is_ts = lang == SourceType::TypeScript;
                let (module_doc, functions, classes) = Self::parse_javascript(source, is_ts);
                analysis.module_docstring = module_doc;
                analysis.functions = functions;
                analysis.classes = classes;
            }
            SourceType::OpenApi => {
                let (module_doc, functions, classes) = Self::parse_openapi(source)?;
                analysis.module_docstring = module_doc;
                analysis.functions = functions;
                analysis.classes = classes;
            }
            SourceType::Auto => {
                let detected = Self::detect_source_type(&analysis.source_path, Some(source));
                analysis.source_type = detected;
                return Self::analyze_source(source, detected, name, path);
            }
        }

        Ok(analysis)
    }

    // =========================================================================
    // Python Parser
    // =========================================================================

    fn parse_python(source: &str) -> (Option<String>, Vec<FunctionSignature>, Vec<ClassSignature>) {
        let lines: Vec<&str> = source.lines().collect();
        let mut module_doc: Option<String> = None;
        let mut functions = Vec::new();
        let mut classes = Vec::new();

        let mut i = 0;

        // 1. Check for module docstring at the top of file
        while i < lines.len() && (lines[i].trim().is_empty() || lines[i].trim().starts_with('#')) {
            i += 1;
        }
        if i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                let quote = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
                let (doc, next_i) = Self::extract_multiline_docstring(&lines, i, quote);
                module_doc = Some(doc);
                i = next_i;
            }
        }

        // 2. Parse functions and classes
        let mut current_class: Option<ClassSignature> = None;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();
            let indent = line.len() - line.trim_start().len();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }

            // Top-level or class-level handling
            if indent == 0 && current_class.is_some() && !trimmed.starts_with("def ") && !trimmed.starts_with("async def ") {
                if let Some(c) = current_class.take() {
                    classes.push(c);
                }
            }

            if trimmed.starts_with("class ") {
                if let Some(c) = current_class.take() {
                    classes.push(c);
                }

                let class_name = Self::extract_python_class_name(trimmed);
                if !class_name.is_empty() {
                    let mut class_sig = ClassSignature::new(&class_name);
                    class_sig.line_number = i + 1;

                    // Check for class docstring
                    let next_line_idx = i + 1;
                    if next_line_idx < lines.len() {
                        let next_trimmed = lines[next_line_idx].trim();
                        if next_trimmed.starts_with("\"\"\"") || next_trimmed.starts_with("'''") {
                            let quote = if next_trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
                            let (doc, next_i) = Self::extract_multiline_docstring(&lines, next_line_idx, quote);
                            class_sig.docstring = Some(doc);
                            i = next_i;
                        }
                    }

                    current_class = Some(class_sig);
                }
                i += 1;
                continue;
            }

            let is_async = trimmed.starts_with("async def ");
            let is_def = trimmed.starts_with("def ") || is_async;

            if is_def {
                // Collect full multi-line function header until ':'
                let mut header = trimmed.to_string();
                let mut def_end_idx = i;
                while !header.contains(':') && def_end_idx + 1 < lines.len() {
                    def_end_idx += 1;
                    header.push(' ');
                    header.push_str(lines[def_end_idx].trim());
                }

                if let Some(mut fn_sig) = Self::parse_python_def(&header, is_async) {
                    fn_sig.line_number = i + 1;

                    // Extract function docstring directly underneath
                    let next_line_idx = def_end_idx + 1;
                    if next_line_idx < lines.len() {
                        let next_trimmed = lines[next_line_idx].trim();
                        if next_trimmed.starts_with("\"\"\"") || next_trimmed.starts_with("'''") {
                            let quote = if next_trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
                            let (doc, next_i) = Self::extract_multiline_docstring(&lines, next_line_idx, quote);
                            // Parse docstring for parameter descriptions
                            Self::enrich_params_from_docstring(&mut fn_sig.parameters, &doc);
                            fn_sig.docstring = Some(doc);
                            i = next_i;
                        } else {
                            i = def_end_idx + 1;
                        }
                    } else {
                        i = def_end_idx + 1;
                    }

                    // Check if inside class or standalone
                    if indent > 0 && current_class.is_some() {
                        fn_sig.is_method = true;
                        // Filter out 'self' or 'cls' from parameters
                        fn_sig.parameters.retain(|p| p.name != "self" && p.name != "cls");
                        if let Some(ref mut c) = current_class {
                            // Don't add private methods except __init__
                            if !fn_sig.name.starts_with('_') || fn_sig.name == "__init__" {
                                c.methods.push(fn_sig);
                            }
                        }
                    } else {
                        // Skip private functions
                        if !fn_sig.name.starts_with('_') {
                            functions.push(fn_sig);
                        }
                    }
                    continue;
                }
            }

            i += 1;
        }

        if let Some(c) = current_class {
            classes.push(c);
        }

        (module_doc, functions, classes)
    }

    fn extract_multiline_docstring(lines: &[&str], start_idx: usize, quote: &str) -> (String, usize) {
        let first_line = lines[start_idx].trim();
        let rest = first_line.trim_start_matches(quote);

        if rest.contains(quote) {
            let doc = rest.split(quote).next().unwrap_or("").trim().to_string();
            return (doc, start_idx + 1);
        }

        let mut doc_lines = Vec::new();
        if !rest.trim().is_empty() {
            doc_lines.push(rest.trim());
        }

        let mut curr = start_idx + 1;
        while curr < lines.len() {
            let line = lines[curr];
            if line.contains(quote) {
                let part = line.split(quote).next().unwrap_or("").trim();
                if !part.is_empty() {
                    doc_lines.push(part);
                }
                return (doc_lines.join("\n"), curr + 1);
            } else {
                doc_lines.push(line.trim());
                curr += 1;
            }
        }

        (doc_lines.join("\n"), curr)
    }

    fn extract_python_class_name(line: &str) -> String {
        let after_class = line.trim_start_matches("class ").trim();
        let end_idx = after_class.find(|c: char| c == '(' || c == ':').unwrap_or(after_class.len());
        after_class[..end_idx].trim().to_string()
    }

    fn parse_python_def(header: &str, is_async: bool) -> Option<FunctionSignature> {
        let prefix = if is_async { "async def " } else { "def " };
        let after_def = header.trim_start_matches(prefix).trim();

        let paren_open = after_def.find('(')?;
        let fn_name = after_def[..paren_open].trim().to_string();

        let paren_close = after_def.rfind(')')?;
        let args_str = &after_def[paren_open + 1..paren_close];

        let return_type = if let Some(arrow_idx) = after_def[paren_close..].find("->") {
            let colon_idx = after_def[paren_close + arrow_idx..].find(':').unwrap_or(after_def.len() - (paren_close + arrow_idx));
            let ret = &after_def[paren_close + arrow_idx + 2..paren_close + arrow_idx + colon_idx].trim();
            Some(ret.to_string())
        } else {
            None
        };

        let mut fn_sig = FunctionSignature::new(fn_name);
        fn_sig.is_async = is_async;
        fn_sig.return_type = return_type;

        // Parse arguments
        let param_tokens = Self::split_python_params(args_str);
        for token in param_tokens {
            let token = token.trim();
            if token.is_empty() || token == "*" || token == "/" || token.starts_with("*args") || token.starts_with("**kwargs") {
                continue;
            }

            let mut name = token.to_string();
            let mut param_type = "str".to_string();
            let mut default_val: Option<String> = None;
            let mut required = true;

            // Check for default: name: type = default
            if let Some((lhs, rhs)) = token.split_once('=') {
                name = lhs.trim().to_string();
                default_val = Some(rhs.trim().trim_matches('"').trim_matches('\'').to_string());
                required = false;
            }

            // Check for type annotation: name: type
            if let Some((p_name, p_type)) = name.clone().split_once(':') {
                name = p_name.trim().to_string();
                param_type = p_type.trim().to_string();
            } else if let Some(ref def) = default_val {
                // Infer type from default value
                if def == "True" || def == "False" {
                    param_type = "bool".to_string();
                } else if def.parse::<i64>().is_ok() {
                    param_type = "int".to_string();
                } else if def.parse::<f64>().is_ok() {
                    param_type = "float".to_string();
                } else if def.starts_with('[') {
                    param_type = "list".to_string();
                } else if def.starts_with('{') {
                    param_type = "dict".to_string();
                }
            }

            let mut param = ParameterInfo::new(name, param_type);
            param.required = required;
            param.default_value = default_val;
            fn_sig.parameters.push(param);
        }

        Some(fn_sig)
    }

    fn split_python_params(args_str: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for c in args_str.chars() {
            match c {
                '[' | '(' | '{' => {
                    depth += 1;
                    current.push(c);
                }
                ']' | ')' | '}' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    current.push(c);
                }
                ',' if depth == 0 => {
                    if !current.trim().is_empty() {
                        tokens.push(current.trim().to_string());
                    }
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            tokens.push(current.trim().to_string());
        }
        tokens
    }

    fn enrich_params_from_docstring(params: &mut [ParameterInfo], doc: &str) {
        let mut in_args_section = false;
        for line in doc.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_lowercase();

            if lower.starts_with("args:") || lower.starts_with("arguments:") || lower.starts_with("parameters:") {
                in_args_section = true;
                continue;
            }
            if in_args_section && (lower.starts_with("returns:") || lower.starts_with("raises:") || lower.starts_with("example:") || lower.starts_with("yields:")) {
                in_args_section = false;
                continue;
            }

            // Google style: `param_name (type): description` or `param_name: description`
            if in_args_section && (line.starts_with("    ") || line.starts_with("\t") || trimmed.contains(':')) {
                if let Some((p_spec, p_desc)) = trimmed.split_once(':') {
                    let p_spec = p_spec.trim();
                    let p_name = if let Some(idx) = p_spec.find('(') {
                        p_spec[..idx].trim()
                    } else {
                        p_spec
                    };

                    for p in params.iter_mut() {
                        if p.name == p_name && p.description.is_none() {
                            p.description = Some(p_desc.trim().to_string());
                        }
                    }
                }
            }

            // Sphinx style: `:param param_name: description`
            if trimmed.starts_with(":param ") {
                let after = &trimmed[7..].trim();
                if let Some((p_name, p_desc)) = after.split_once(':') {
                    let clean_name = p_name.trim();
                    for p in params.iter_mut() {
                        if p.name == clean_name && p.description.is_none() {
                            p.description = Some(p_desc.trim().to_string());
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // Rust Parser
    // =========================================================================

    fn parse_rust(source: &str) -> (Option<String>, Vec<FunctionSignature>, Vec<ClassSignature>) {
        let lines: Vec<&str> = source.lines().collect();
        let mut module_doc_lines = Vec::new();
        let mut functions = Vec::new();
        let mut classes = Vec::new();

        let mut current_docs = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            if trimmed.starts_with("//!") {
                let doc = trimmed.trim_start_matches("//!").trim();
                module_doc_lines.push(doc);
                i += 1;
                continue;
            }

            if trimmed.starts_with("///") {
                let doc = trimmed.trim_start_matches("///").trim();
                current_docs.push(doc);
                i += 1;
                continue;
            }

            if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub async fn ") {
                let is_async = trimmed.contains("async fn ");
                let mut header = trimmed.to_string();
                let mut end_idx = i;
                while !header.contains('{') && !header.ends_with(';') && end_idx + 1 < lines.len() {
                    end_idx += 1;
                    header.push(' ');
                    header.push_str(lines[end_idx].trim());
                }

                if let Some(mut fn_sig) = Self::parse_rust_fn(&header, is_async) {
                    fn_sig.line_number = i + 1;
                    if !current_docs.is_empty() {
                        fn_sig.docstring = Some(current_docs.join("\n"));
                    }
                    functions.push(fn_sig);
                }
                current_docs.clear();
                i = end_idx + 1;
                continue;
            }

            if trimmed.starts_with("pub struct ") {
                let after = trimmed.trim_start_matches("pub struct ").trim();
                let name = after.split(|c: char| c == '{' || c == '(' || c == ';' || c.is_whitespace()).next().unwrap_or("").trim();
                if !name.is_empty() {
                    let mut class_sig = ClassSignature::new(name);
                    class_sig.line_number = i + 1;
                    if !current_docs.is_empty() {
                        class_sig.docstring = Some(current_docs.join("\n"));
                    }
                    classes.push(class_sig);
                }
                current_docs.clear();
            }

            if !trimmed.starts_with("///") && !trimmed.is_empty() {
                current_docs.clear();
            }

            i += 1;
        }

        let module_doc = if module_doc_lines.is_empty() {
            None
        } else {
            Some(module_doc_lines.join("\n"))
        };

        (module_doc, functions, classes)
    }

    fn parse_rust_fn(header: &str, is_async: bool) -> Option<FunctionSignature> {
        let fn_idx = header.find("fn ")?;
        let after_fn = &header[fn_idx + 3..].trim();

        let paren_open = after_fn.find('(')?;
        let fn_name = after_fn[..paren_open].trim();
        let paren_close = after_fn.rfind(')')?;
        let args_str = &after_fn[paren_open + 1..paren_close];

        let return_type = if let Some(arrow_idx) = after_fn[paren_close..].find("->") {
            let brace_idx = after_fn[paren_close + arrow_idx..].find('{').unwrap_or(after_fn.len() - (paren_close + arrow_idx));
            let ret = &after_fn[paren_close + arrow_idx + 2..paren_close + arrow_idx + brace_idx].trim();
            Some(ret.to_string())
        } else {
            None
        };

        let mut fn_sig = FunctionSignature::new(fn_name);
        fn_sig.is_async = is_async;
        fn_sig.return_type = return_type;

        let tokens = Self::split_python_params(args_str); // Reusing nested bracket splitter
        for token in tokens {
            let token = token.trim();
            if token.is_empty() || token == "&self" || token == "&mut self" || token == "self" {
                continue;
            }

            if let Some((p_name, p_type)) = token.split_once(':') {
                let name = p_name.trim().trim_start_matches("mut ");
                let p_type = p_type.trim();
                let param = ParameterInfo::new(name, p_type);
                fn_sig.parameters.push(param);
            }
        }

        Some(fn_sig)
    }

    // =========================================================================
    // JavaScript / TypeScript Parser
    // =========================================================================

    fn parse_javascript(source: &str, is_ts: bool) -> (Option<String>, Vec<FunctionSignature>, Vec<ClassSignature>) {
        let lines: Vec<&str> = source.lines().collect();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut current_jsdoc = Vec::new();
        let mut in_jsdoc = false;

        for (line_idx, raw_line) in lines.iter().enumerate() {
            let trimmed = raw_line.trim();

            if trimmed.starts_with("/**") {
                in_jsdoc = true;
                current_jsdoc.clear();
                let rest = trimmed.trim_start_matches("/**").trim();
                if !rest.is_empty() && rest != "*/" {
                    current_jsdoc.push(rest.trim_end_matches("*/").trim());
                }
                if trimmed.ends_with("*/") && trimmed.len() > 3 {
                    in_jsdoc = false;
                }
                continue;
            }

            if in_jsdoc {
                if trimmed.ends_with("*/") {
                    in_jsdoc = false;
                    let rest = trimmed.trim_end_matches("*/").trim_start_matches('*').trim();
                    if !rest.is_empty() {
                        current_jsdoc.push(rest);
                    }
                } else {
                    let clean = trimmed.trim_start_matches('*').trim();
                    if !clean.is_empty() {
                        current_jsdoc.push(clean);
                    }
                }
                continue;
            }

            // Export function or standalone function
            let is_fn = trimmed.contains("function ") || (trimmed.contains("const ") && trimmed.contains("=>"));
            if is_fn && (trimmed.starts_with("export ") || trimmed.starts_with("function ") || trimmed.starts_with("async function ") || trimmed.starts_with("const ")) {
                let is_async = trimmed.contains("async ");
                if let Some(mut fn_sig) = Self::parse_js_fn(trimmed, is_async, is_ts) {
                    fn_sig.line_number = line_idx + 1;
                    if !current_jsdoc.is_empty() {
                        let doc = current_jsdoc.join("\n");
                        Self::enrich_params_from_jsdoc(&mut fn_sig.parameters, &doc);
                        fn_sig.docstring = Some(doc);
                    }
                    functions.push(fn_sig);
                }
                current_jsdoc.clear();
            }

            // Classes
            if trimmed.starts_with("class ") || trimmed.starts_with("export class ") {
                let after = trimmed.trim_start_matches("export ").trim_start_matches("class ").trim();
                let class_name = after.split(|c: char| c == '{' || c.is_whitespace()).next().unwrap_or("").trim();
                if !class_name.is_empty() {
                    let mut class_sig = ClassSignature::new(class_name);
                    class_sig.line_number = line_idx + 1;
                    if !current_jsdoc.is_empty() {
                        class_sig.docstring = Some(current_jsdoc.join("\n"));
                    }
                    classes.push(class_sig);
                }
                current_jsdoc.clear();
            }

            if !trimmed.starts_with("//") && !trimmed.is_empty() && !is_fn {
                current_jsdoc.clear();
            }
        }

        (None, functions, classes)
    }

    fn parse_js_fn(line: &str, is_async: bool, is_ts: bool) -> Option<FunctionSignature> {
        let clean = line.trim_start_matches("export ").trim();

        if clean.starts_with("function ") || clean.starts_with("async function ") {
            let after_fn = clean.trim_start_matches("async ").trim_start_matches("function ").trim();
            let paren_open = after_fn.find('(')?;
            let fn_name = after_fn[..paren_open].trim();
            let paren_close = after_fn.rfind(')')?;
            let args_str = &after_fn[paren_open + 1..paren_close];

            let mut fn_sig = FunctionSignature::new(fn_name);
            fn_sig.is_async = is_async;
            Self::parse_js_args(&mut fn_sig.parameters, args_str, is_ts);
            return Some(fn_sig);
        }

        if clean.starts_with("const ") && clean.contains("=>") {
            let after_const = clean.trim_start_matches("const ").trim();
            let eq_idx = after_const.find('=')?;
            let fn_name = after_const[..eq_idx].trim();
            let paren_open = after_const.find('(')?;
            let paren_close = after_const.find(')')?;
            if paren_close > paren_open {
                let args_str = &after_const[paren_open + 1..paren_close];
                let mut fn_sig = FunctionSignature::new(fn_name);
                fn_sig.is_async = is_async;
                Self::parse_js_args(&mut fn_sig.parameters, args_str, is_ts);
                return Some(fn_sig);
            }
        }

        None
    }

    fn parse_js_args(params: &mut Vec<ParameterInfo>, args_str: &str, is_ts: bool) {
        let tokens = Self::split_python_params(args_str);
        for token in tokens {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }

            let mut name = token.to_string();
            let mut param_type = "any".to_string();
            let mut required = true;
            let mut default_val = None;

            if let Some((lhs, rhs)) = token.split_once('=') {
                name = lhs.trim().to_string();
                default_val = Some(rhs.trim().to_string());
                required = false;
            }

            if is_ts {
                if let Some((p_name, p_type)) = name.clone().split_once(':') {
                    name = p_name.trim().trim_end_matches('?').to_string();
                    param_type = p_type.trim().to_string();
                    if p_name.ends_with('?') {
                        required = false;
                    }
                }
            }

            let mut param = ParameterInfo::new(name, param_type);
            param.required = required;
            param.default_value = default_val;
            params.push(param);
        }
    }

    fn enrich_params_from_jsdoc(params: &mut [ParameterInfo], doc: &str) {
        for line in doc.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("@param") {
                // @param {type} name description
                let after = trimmed.trim_start_matches("@param").trim();
                let (param_type, rest) = if after.starts_with('{') {
                    if let Some(close) = after.find('}') {
                        (&after[1..close], &after[close + 1..].trim())
                    } else {
                        ("", &after)
                    }
                } else {
                    ("", &after)
                };

                let mut parts = rest.split_whitespace();
                if let Some(p_name) = parts.next() {
                    let desc: Vec<&str> = parts.collect();
                    let desc_str = desc.join(" ");

                    for p in params.iter_mut() {
                        if p.name == p_name {
                            if !param_type.is_empty() && p.param_type == "any" {
                                p.param_type = param_type.to_string();
                            }
                            if !desc_str.is_empty() && p.description.is_none() {
                                p.description = Some(desc_str.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // OpenAPI Parser
    // =========================================================================

    fn parse_openapi(source: &str) -> Result<(Option<String>, Vec<FunctionSignature>, Vec<ClassSignature>)> {
        let json_val: serde_json::Value = serde_json::from_str(source).or_else(|_| {
            // Basic fallback conversion from YAML-like OpenAPI to JSON structure
            Err(TagisanError::Execution("OpenAPI source must be valid JSON or parseable YAML".to_string()))
        })?;

        let title = json_val["info"]["title"].as_str().unwrap_or("API");
        let desc = json_val["info"]["description"].as_str().unwrap_or("");
        let module_doc = Some(format!("{} - {}", title, desc));

        let mut functions = Vec::new();

        if let Some(paths) = json_val.get("paths").and_then(|p| p.as_object()) {
            for (path, path_item) in paths {
                if let Some(path_obj) = path_item.as_object() {
                    for (method, op_val) in path_obj {
                        let method_lower = method.to_lowercase();
                        if !["get", "post", "put", "delete", "patch", "options", "head"].contains(&method_lower.as_str()) {
                            continue;
                        }

                        let op_id = op_val["operationId"].as_str().map(|s| s.to_string()).unwrap_or_else(|| {
                            let slug = path.trim_matches('/').replace('/', "_").replace(['{', '}'], "");
                            format!("{}_{}", method_lower, slug)
                        });

                        let summary = op_val["summary"].as_str().unwrap_or("");
                        let op_desc = op_val["description"].as_str().unwrap_or(summary);

                        let mut fn_sig = FunctionSignature::new(&op_id);
                        if !op_desc.is_empty() {
                            fn_sig.docstring = Some(op_desc.to_string());
                        }

                        // Parameters
                        if let Some(params_array) = op_val["parameters"].as_array() {
                            for p in params_array {
                                let p_name = p["name"].as_str().unwrap_or("param").to_string();
                                let p_type = p["schema"]["type"].as_str().unwrap_or("string").to_string();
                                let p_desc = p["description"].as_str().map(|s| s.to_string());
                                let required = p["required"].as_bool().unwrap_or(false);

                                let mut param = ParameterInfo::new(p_name, p_type);
                                param.required = required;
                                param.description = p_desc;
                                fn_sig.parameters.push(param);
                            }
                        }

                        // Request body properties
                        if let Some(body_schema) = op_val["requestBody"]["content"]["application/json"]["schema"]["properties"].as_object() {
                            let required_fields: Vec<String> = op_val["requestBody"]["content"]["application/json"]["schema"]["required"]
                                .as_array()
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default();

                            for (prop_name, prop_spec) in body_schema {
                                let p_type = prop_spec["type"].as_str().unwrap_or("string").to_string();
                                let p_desc = prop_spec["description"].as_str().map(|s| s.to_string());
                                let is_req = required_fields.contains(prop_name);

                                let mut param = ParameterInfo::new(prop_name.clone(), p_type);
                                param.required = is_req;
                                param.description = p_desc;
                                fn_sig.parameters.push(param);
                            }
                        }

                        functions.push(fn_sig);
                    }
                }
            }
        }

        Ok((module_doc, functions, Vec::new()))
    }
}
