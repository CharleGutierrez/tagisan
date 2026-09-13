use crate::error::{Result, TagisanError};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

/// Classification of a source code symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Interface,
    TypeAlias,
    Module,
    Constant,
    Macro,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Interface => "interface",
            SymbolKind::TypeAlias => "type_alias",
            SymbolKind::Module => "module",
            SymbolKind::Constant => "constant",
            SymbolKind::Macro => "macro",
        }
    }
}

/// Visibility scope of a code symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolVisibility {
    Public,
    Private,
    Crate,
    Protected,
}

impl SymbolVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolVisibility::Public => "public",
            SymbolVisibility::Private => "private",
            SymbolVisibility::Crate => "crate",
            SymbolVisibility::Protected => "protected",
        }
    }
}

/// Structural and semantic relationships between symbols in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolRelation {
    Calls,
    Defines,
    Implements,
    Imports,
    References,
}

impl SymbolRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolRelation::Calls => "calls",
            SymbolRelation::Defines => "defines",
            SymbolRelation::Implements => "implements",
            SymbolRelation::Imports => "imports",
            SymbolRelation::References => "references",
        }
    }
}

/// Blast risk classification based on caller centrality and architectural depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BlastRisk {
    Low,
    Medium,
    High,
    Critical,
}

impl BlastRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlastRisk::Low => "LOW",
            BlastRisk::Medium => "MEDIUM",
            BlastRisk::High => "HIGH",
            BlastRisk::Critical => "CRITICAL",
        }
    }
}

/// An individual parsed code symbol in the knowledge graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: SymbolKind,
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub visibility: SymbolVisibility,
    pub signature: String,
    pub doc: Option<String>,
}

/// Directed edge representing relationships between symbols
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolEdge {
    pub relation: SymbolRelation,
    pub weight: f32,
    pub call_site_line: Option<usize>,
}

/// Transitive blast radius report for a targeted symbol modification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    pub target_symbol: String,
    pub target_file: PathBuf,
    pub direct_callers: Vec<String>,
    pub transitive_callers: Vec<String>,
    pub implementing_types: Vec<String>,
    pub affected_files: Vec<PathBuf>,
    pub total_affected_symbols: usize,
    pub risk_level: BlastRisk,
    pub recommendations: Vec<String>,
}

/// Global statistical summary of the codebase knowledge graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub functions_count: usize,
    pub types_count: usize,
    pub modules_count: usize,
    pub files_count: usize,
    pub top_central_symbols: Vec<(String, usize)>,
}

/// Codebase knowledge graph containing nodes and directed dependency edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseGraph {
    pub graph: DiGraph<CodeSymbol, SymbolEdge>,
    pub symbol_index: HashMap<String, NodeIndex>,
    pub file_index: HashMap<PathBuf, Vec<NodeIndex>>,
}

impl Default for CodebaseGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw parsed edge connecting symbol names prior to index resolution
#[derive(Debug, Clone)]
struct RawEdge {
    caller_key: String,
    callee_name: String,
    relation: SymbolRelation,
    call_site_line: Option<usize>,
}

/// Raw file parse output containing extracted symbols and edges
#[derive(Debug, Clone, Default)]
struct FileParseResult {
    file: PathBuf,
    symbols: Vec<CodeSymbol>,
    edges: Vec<RawEdge>,
}

impl CodebaseGraph {
    /// Initialize an empty codebase knowledge graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            symbol_index: HashMap::new(),
            file_index: HashMap::new(),
        }
    }

    /// Build a polyglot AST knowledge graph from a project directory
    pub fn build_from_dir(path: &Path, max_files: usize) -> Result<Self> {
        let root = if path.is_relative() {
            std::env::current_dir()
                .map_err(|e| TagisanError::Execution(format!("Cannot resolve current dir: {e}")))?
                .join(path)
        } else {
            path.to_path_buf()
        };

        if !root.exists() {
            return Err(TagisanError::Execution(format!(
                "Path does not exist: {}",
                root.display()
            )));
        }

        let mut eligible_files = Vec::new();
        let mut dirs = vec![root.clone()];

        // Common directories to skip
        let ignore_dirs: HashSet<&'static str> = [
            ".git", "target", "node_modules", "dist", "build", ".venv", "venv",
            "__pycache__", ".tagisan", "coverage", ".next", ".nuxt", "vendor",
            ".turbo", ".cargo", "out", "bin", "obj",
        ]
        .iter()
        .cloned()
        .collect();

        while let Some(dir) = dirs.pop() {
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let p = entry.path();
                let fname = match p.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n,
                    None => continue,
                };

                if p.is_dir() {
                    if !ignore_dirs.contains(fname) && !fname.starts_with('.') {
                        dirs.push(p);
                    }
                } else if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        let is_supported = matches!(
                            ext,
                            "rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "go" | "mjs" | "cjs"
                        );
                        if is_supported {
                            eligible_files.push(p);
                            if max_files > 0 && eligible_files.len() >= max_files {
                                break;
                            }
                        }
                    }
                }
            }

            if max_files > 0 && eligible_files.len() >= max_files {
                break;
            }
        }

        // Rayon parallel file parsing
        let parse_results: Vec<FileParseResult> = eligible_files
            .into_par_iter()
            .map(|f| Self::parse_file(&f, &root))
            .collect();

        // Assemble graph
        let mut codebase_graph = CodebaseGraph::new();
        let mut all_raw_edges = Vec::new();

        for file_res in parse_results {
            let mut file_node_indices = Vec::new();

            for sym in file_res.symbols {
                let sym_id = sym.id.clone();
                let sym_name = sym.name.clone();
                let qual_name = sym.qualified_name.clone();

                let idx = codebase_graph.graph.add_node(sym);
                codebase_graph.symbol_index.insert(sym_id, idx);
                codebase_graph.symbol_index.insert(qual_name, idx);

                // Store primary name if not already taken, or allow overwrite
                codebase_graph
                    .symbol_index
                    .entry(sym_name)
                    .or_insert(idx);

                file_node_indices.push(idx);
            }

            codebase_graph
                .file_index
                .insert(file_res.file, file_node_indices);
            all_raw_edges.extend(file_res.edges);
        }

        // Resolve edges
        for raw in all_raw_edges {
            let caller_idx = codebase_graph.resolve_symbol_node(&raw.caller_key);
            let callee_idx = codebase_graph.resolve_symbol_node(&raw.callee_name);

            if let (Some(from), Some(to)) = (caller_idx, callee_idx) {
                if from != to {
                    // Check if edge already exists to prevent duplicate edges
                    let exists = codebase_graph
                        .graph
                        .edges_connecting(from, to)
                        .any(|e| e.weight().relation == raw.relation);

                    if !exists {
                        codebase_graph.graph.add_edge(
                            from,
                            to,
                            SymbolEdge {
                                relation: raw.relation,
                                weight: 1.0,
                                call_site_line: raw.call_site_line,
                            },
                        );
                    }
                }
            }
        }

        Ok(codebase_graph)
    }

    /// Resolve a symbol query string to a NodeIndex in the graph
    fn resolve_symbol_node(&self, key: &str) -> Option<NodeIndex> {
        if let Some(&idx) = self.symbol_index.get(key) {
            return Some(idx);
        }

        // Fallback: search by symbol simple name or suffix match
        for (idx, sym) in self.graph.node_indices().map(|i| (i, &self.graph[i])) {
            if sym.name == key || sym.qualified_name == key || sym.id == key {
                return Some(idx);
            }
            if sym.qualified_name.ends_with(&format!("::{key}"))
                || sym.qualified_name.ends_with(&format!(".{key}"))
            {
                return Some(idx);
            }
        }

        None
    }

    /// Parse a single source file into symbols and raw relational edges
    fn parse_file(file: &Path, root: &Path) -> FileParseResult {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => return FileParseResult::default(),
        };

        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        let rel_file = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_path_buf();

        match ext {
            "rs" => Self::parse_rust(&content, &rel_file),
            "py" => Self::parse_python(&content, &rel_file),
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => {
                Self::parse_typescript_javascript(&content, &rel_file)
            }
            "go" => Self::parse_go(&content, &rel_file),
            _ => FileParseResult::default(),
        }
    }

    // =========================================================================
    // Polyglot AST Parsers
    // =========================================================================

    /// Rust AST extraction
    fn parse_rust(content: &str, file: &Path) -> FileParseResult {
        let mut symbols = Vec::new();
        let mut edges = Vec::new();

        let fn_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:const\s+)?fn\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s*(?:<[^>]+>)?\s*\((?P<params>[^)]*)\)(?:\s*->\s*(?P<ret>[^{;]+))?",
        )
        .unwrap();

        let struct_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?struct\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)",
        )
        .unwrap();

        let enum_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?enum\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)",
        )
        .unwrap();

        let trait_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?trait\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)",
        )
        .unwrap();

        let impl_trait_regex = Regex::new(
            r"^impl(?:<[^>]+>)?\s+(?P<trait>[a-zA-Z_][a-zA-Z0-9_:]*)\s+for\s+(?P<type>[a-zA-Z_][a-zA-Z0-9_:]*)",
        )
        .unwrap();

        let impl_type_regex =
            Regex::new(r"^impl(?:<[^>]+>)?\s+(?P<type>[a-zA-Z_][a-zA-Z0-9_:]*)").unwrap();

        let type_alias_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?type\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s*=",
        )
        .unwrap();

        let mod_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?mod\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)",
        )
        .unwrap();

        let macro_regex =
            Regex::new(r"^macro_rules!\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        let const_regex = Regex::new(
            r"^(?P<vis>pub(?:\([^)]+\))?\s+)?const\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s*:",
        )
        .unwrap();

        let call_regex =
            Regex::new(r"(?:[a-zA-Z_][a-zA-Z0-9_]*::)*(?P<callee>[a-zA-Z_][a-zA-Z0-9_]*)\s*\(")
                .unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_doc = Vec::new();
        let mut current_impl_type: Option<String> = None;
        let mut current_impl_trait: Option<String> = None;
        let mut current_fn_scope: Option<(String, usize, usize)> = None; // (name, start_line, depth)
        let mut brace_depth = 0;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Collect doc comments
            if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                let doc_line = trimmed
                    .trim_start_matches("///")
                    .trim_start_matches("//!")
                    .trim();
                current_doc.push(doc_line.to_string());
                continue;
            }

            if trimmed.starts_with("//") {
                continue;
            }

            let doc_text = if !current_doc.is_empty() {
                let d = current_doc.join("\n");
                current_doc.clear();
                Some(d)
            } else {
                None
            };

            // Parse mod declarations
            if let Some(caps) = mod_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:mod:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Module,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse struct declarations
            if let Some(caps) = struct_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:struct:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Struct,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse enum declarations
            if let Some(caps) = enum_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:enum:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Enum,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse trait declarations
            if let Some(caps) = trait_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:trait:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Trait,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse type alias
            if let Some(caps) = type_alias_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:type:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::TypeAlias,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse macro declarations
            if let Some(caps) = macro_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:macro:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Macro,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: SymbolVisibility::Public,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse constant declarations
            if let Some(caps) = const_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:const:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Constant,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text.clone(),
                });
            }

            // Parse impl blocks
            if let Some(caps) = impl_trait_regex.captures(trimmed) {
                let tr = caps["trait"].to_string();
                let ty = caps["type"].to_string();
                current_impl_trait = Some(tr.clone());
                current_impl_type = Some(ty.clone());

                edges.push(RawEdge {
                    caller_key: ty.clone(),
                    callee_name: tr,
                    relation: SymbolRelation::Implements,
                    call_site_line: Some(line_num),
                });
            } else if let Some(caps) = impl_type_regex.captures(trimmed) {
                let ty = caps["type"].to_string();
                current_impl_trait = None;
                current_impl_type = Some(ty);
            }

            // Parse use / import statements
            if trimmed.starts_with("use ") {
                let path_part = trimmed.trim_start_matches("use ").trim_end_matches(';');
                let clean_part = path_part.split('{').next().unwrap_or("").trim();
                let imported = clean_part.split("::").last().unwrap_or("").trim();
                if !imported.is_empty() && imported != "*" {
                    edges.push(RawEdge {
                        caller_key: file.display().to_string(),
                        callee_name: imported.to_string(),
                        relation: SymbolRelation::Imports,
                        call_site_line: Some(line_num),
                    });
                }
            }

            // Parse function/method declarations
            if let Some(caps) = fn_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = Self::parse_rust_visibility(caps.name("vis").map(|m| m.as_str()));
                let kind = if current_impl_type.is_some() {
                    SymbolKind::Method
                } else {
                    SymbolKind::Function
                };

                let qual = if let Some(ref ty) = current_impl_type {
                    format!("{ty}::{name}")
                } else {
                    name.clone()
                };

                let sym = CodeSymbol {
                    id: format!("{}:{}:fn:{}", file.display(), line_num, qual),
                    name: name.clone(),
                    qualified_name: qual.clone(),
                    kind,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: doc_text,
                };
                symbols.push(sym);

                current_fn_scope = Some((qual, line_num, brace_depth));
            }

            // Call site detection within functions
            if let Some((ref caller_name, _, _)) = current_fn_scope {
                for cap in call_regex.captures_iter(trimmed) {
                    let callee = cap["callee"].to_string();
                    let rust_keywords = [
                        "if", "match", "while", "for", "return", "let", "mut", "Some", "Ok",
                        "Err", "println", "eprintln", "vec", "format", "panic", "assert",
                        "assert_eq", "assert_ne", "todo", "unimplemented", "unreachable",
                    ];
                    if !rust_keywords.contains(&callee.as_str()) && callee != *caller_name {
                        edges.push(RawEdge {
                            caller_key: caller_name.clone(),
                            callee_name: callee,
                            relation: SymbolRelation::Calls,
                            call_site_line: Some(line_num),
                        });
                    }
                }
            }

            // Track brace depth
            let open_braces = line.matches('{').count();
            let close_braces = line.matches('}').count();
            brace_depth += open_braces;

            if let Some((_, start_l, fn_depth)) = current_fn_scope.clone() {
                if brace_depth <= fn_depth && close_braces > 0 && line_num > start_l {
                    if let Some(last_sym) = symbols.last_mut() {
                        last_sym.end_line = line_num;
                        last_sym.end_col = line.len();
                    }
                    current_fn_scope = None;
                }
            }

            if brace_depth >= close_braces {
                brace_depth -= close_braces;
            } else {
                brace_depth = 0;
            }

            if brace_depth == 0 {
                current_impl_type = None;
                current_impl_trait = None;
            }
        }

        FileParseResult {
            file: file.to_path_buf(),
            symbols,
            edges,
        }
    }

    fn parse_rust_visibility(vis: Option<&str>) -> SymbolVisibility {
        match vis.map(|v| v.trim()) {
            Some(v) if v.starts_with("pub(crate)") || v.starts_with("pub(super)") => {
                SymbolVisibility::Crate
            }
            Some(v) if v.starts_with("pub") => SymbolVisibility::Public,
            _ => SymbolVisibility::Private,
        }
    }

    /// Python AST extraction
    fn parse_python(content: &str, file: &Path) -> FileParseResult {
        let mut symbols = Vec::new();
        let mut edges = Vec::new();

        let def_regex = Regex::new(
            r"^(?P<indent>\s*)(?:async\s+)?def\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s*\((?P<params>[^)]*)\)",
        )
        .unwrap();

        let class_regex = Regex::new(
            r"^(?P<indent>\s*)class\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)(?:\((?P<bases>[^)]*)\))?:",
        )
        .unwrap();

        let import_regex = Regex::new(r"^(?:from\s+[a-zA-Z0-9_.]+\s+)?import\s+(?P<names>[^#]+)")
            .unwrap();

        let call_regex = Regex::new(r"(?P<callee>[a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_class: Option<(String, usize)> = None; // (name, indent)
        let mut current_fn: Option<(String, usize)> = None; // (qual_name, indent)

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indent = line.len() - line.trim_start().len();

            // Check if class scope ended
            if let Some((_, class_indent)) = current_class.clone() {
                if indent <= class_indent && !trimmed.starts_with("class ") {
                    current_class = None;
                }
            }

            // Check if function scope ended
            if let Some((_, fn_indent)) = current_fn.clone() {
                if indent <= fn_indent
                    && !trimmed.starts_with("def ")
                    && !trimmed.starts_with("async def ")
                {
                    current_fn = None;
                }
            }

            // Parse class
            if let Some(caps) = class_regex.captures(line) {
                let name = caps["name"].to_string();
                let bases = caps
                    .name("bases")
                    .map(|b| b.as_str())
                    .unwrap_or("")
                    .split(',')
                    .map(|b| b.trim().to_string())
                    .filter(|b| !b.is_empty());

                let vis = if name.starts_with('_') {
                    SymbolVisibility::Private
                } else {
                    SymbolVisibility::Public
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:class:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Struct,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });

                for base in bases {
                    edges.push(RawEdge {
                        caller_key: name.clone(),
                        callee_name: base,
                        relation: SymbolRelation::Implements,
                        call_site_line: Some(line_num),
                    });
                }

                current_class = Some((name, indent));
                continue;
            }

            // Parse def
            if let Some(caps) = def_regex.captures(line) {
                let name = caps["name"].to_string();
                let vis = if name.starts_with("__") && !name.ends_with("__") {
                    SymbolVisibility::Private
                } else if name.starts_with('_') && !name.ends_with("__") {
                    SymbolVisibility::Protected
                } else {
                    SymbolVisibility::Public
                };

                let (kind, qual) = if let Some((ref cls, _)) = current_class {
                    (SymbolKind::Method, format!("{cls}.{name}"))
                } else {
                    (SymbolKind::Function, name.clone())
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:def:{}", file.display(), line_num, qual),
                    name: name.clone(),
                    qualified_name: qual.clone(),
                    kind,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });

                current_fn = Some((qual, indent));
                continue;
            }

            // Parse imports
            if let Some(caps) = import_regex.captures(line) {
                let names_str = caps["names"].to_string();
                for n in names_str.split(',') {
                    let clean = n.split(" as ").next().unwrap_or("").trim();
                    if !clean.is_empty() {
                        edges.push(RawEdge {
                            caller_key: file.display().to_string(),
                            callee_name: clean.to_string(),
                            relation: SymbolRelation::Imports,
                            call_site_line: Some(line_num),
                        });
                    }
                }
            }

            // Call sites inside functions
            if let Some((ref caller_name, _)) = current_fn {
                for cap in call_regex.captures_iter(trimmed) {
                    let callee = cap["callee"].to_string();
                    let py_builtins = [
                        "print", "len", "range", "int", "str", "float", "bool", "list", "dict",
                        "set", "tuple", "isinstance", "issubclass", "super", "enumerate", "zip",
                        "map", "filter", "min", "max", "sum", "abs", "round", "open", "type",
                    ];
                    if !py_builtins.contains(&callee.as_str()) && callee != *caller_name {
                        edges.push(RawEdge {
                            caller_key: caller_name.clone(),
                            callee_name: callee,
                            relation: SymbolRelation::Calls,
                            call_site_line: Some(line_num),
                        });
                    }
                }
            }
        }

        FileParseResult {
            file: file.to_path_buf(),
            symbols,
            edges,
        }
    }

    /// TypeScript / JavaScript AST extraction
    fn parse_typescript_javascript(content: &str, file: &Path) -> FileParseResult {
        let mut symbols = Vec::new();
        let mut edges = Vec::new();

        let fn_regex = Regex::new(
            r"^(?P<export>export\s+)?(?:default\s+)?(?:async\s+)?function\s+(?P<name>[a-zA-Z_$][a-zA-Z0-9_$]*)\s*(?:<[^>]+>)?\s*\(",
        )
        .unwrap();

        let arrow_regex = Regex::new(
            r"^(?P<export>export\s+)?(?:const|let|var)\s+(?P<name>[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:async\s+)?(?:\([^)]*\)|[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=>",
        )
        .unwrap();

        let class_regex = Regex::new(
            r"^(?P<export>export\s+)?(?:default\s+)?class\s+(?P<name>[a-zA-Z_$][a-zA-Z0-9_$]*)(?:\s+extends\s+(?P<base>[a-zA-Z_$][a-zA-Z0-9_$]*))?(?:\s+implements\s+(?P<interfaces>[^{]+))?",
        )
        .unwrap();

        let interface_regex = Regex::new(
            r"^(?P<export>export\s+)?interface\s+(?P<name>[a-zA-Z_$][a-zA-Z0-9_$]*)",
        )
        .unwrap();

        let type_alias_regex =
            Regex::new(r"^(?P<export>export\s+)?type\s+(?P<name>[a-zA-Z_$][a-zA-Z0-9_$]*)\s*=")
                .unwrap();

        let import_regex = Regex::new(
            r#"^import\s+(?:(?:\{(?P<named>[^}]+)\})|(?P<default>[a-zA-Z_$][a-zA-Z0-9_$]*))\s+from"#,
        )
        .unwrap();

        let call_regex = Regex::new(r"(?P<callee>[a-zA-Z_$][a-zA-Z0-9_$]*)\s*\(").unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_class: Option<(String, usize)> = None;
        let mut current_fn: Option<(String, usize)> = None;
        let mut brace_depth = 0;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Parse interface
            if let Some(caps) = interface_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if caps.name("export").is_some() {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:interface:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Interface,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });
            }

            // Parse type alias
            if let Some(caps) = type_alias_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if caps.name("export").is_some() {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };
                symbols.push(CodeSymbol {
                    id: format!("{}:{}:type:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::TypeAlias,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });
            }

            // Parse class
            if let Some(caps) = class_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if caps.name("export").is_some() {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:class:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Struct,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });

                if let Some(base) = caps.name("base").map(|b| b.as_str().to_string()) {
                    edges.push(RawEdge {
                        caller_key: name.clone(),
                        callee_name: base,
                        relation: SymbolRelation::References,
                        call_site_line: Some(line_num),
                    });
                }

                if let Some(interfaces) = caps.name("interfaces").map(|i| i.as_str()) {
                    for iface in interfaces.split(',') {
                        let iface_clean = iface.trim();
                        if !iface_clean.is_empty() {
                            edges.push(RawEdge {
                                caller_key: name.clone(),
                                callee_name: iface_clean.to_string(),
                                relation: SymbolRelation::Implements,
                                call_site_line: Some(line_num),
                            });
                        }
                    }
                }

                current_class = Some((name, brace_depth));
            }

            // Parse function
            if let Some(caps) = fn_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if caps.name("export").is_some() {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };

                let (kind, qual) = if let Some((ref cls, _)) = current_class {
                    (SymbolKind::Method, format!("{cls}.{name}"))
                } else {
                    (SymbolKind::Function, name.clone())
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:fn:{}", file.display(), line_num, qual),
                    name: name.clone(),
                    qualified_name: qual.clone(),
                    kind,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });

                current_fn = Some((qual, brace_depth));
            } else if let Some(caps) = arrow_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if caps.name("export").is_some() {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:fn:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Function,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });

                current_fn = Some((name, brace_depth));
            }

            // Parse imports
            if let Some(caps) = import_regex.captures(trimmed) {
                if let Some(named) = caps.name("named") {
                    for n in named.as_str().split(',') {
                        let clean = n.split(" as ").next().unwrap_or("").trim();
                        if !clean.is_empty() {
                            edges.push(RawEdge {
                                caller_key: file.display().to_string(),
                                callee_name: clean.to_string(),
                                relation: SymbolRelation::Imports,
                                call_site_line: Some(line_num),
                            });
                        }
                    }
                }
                if let Some(def) = caps.name("default") {
                    let d = def.as_str().trim();
                    if !d.is_empty() {
                        edges.push(RawEdge {
                            caller_key: file.display().to_string(),
                            callee_name: d.to_string(),
                            relation: SymbolRelation::Imports,
                            call_site_line: Some(line_num),
                        });
                    }
                }
            }

            // Call sites
            if let Some((ref caller_name, _)) = current_fn {
                for cap in call_regex.captures_iter(trimmed) {
                    let callee = cap["callee"].to_string();
                    let js_keywords = [
                        "if", "while", "for", "switch", "catch", "return", "require", "import",
                        "console", "setTimeout", "setInterval", "clearTimeout", "clearInterval",
                        "fetch", "Promise", "Object", "Array", "String", "Number", "Boolean",
                    ];
                    if !js_keywords.contains(&callee.as_str()) && callee != *caller_name {
                        edges.push(RawEdge {
                            caller_key: caller_name.clone(),
                            callee_name: callee,
                            relation: SymbolRelation::Calls,
                            call_site_line: Some(line_num),
                        });
                    }
                }
            }

            // Braces
            let open = line.matches('{').count();
            let close = line.matches('}').count();
            brace_depth += open;

            if let Some((_, fn_depth)) = current_fn.clone() {
                if brace_depth <= fn_depth && close > 0 {
                    current_fn = None;
                }
            }

            if let Some((_, cls_depth)) = current_class.clone() {
                if brace_depth <= cls_depth && close > 0 {
                    current_class = None;
                }
            }

            if brace_depth >= close {
                brace_depth -= close;
            } else {
                brace_depth = 0;
            }
        }

        FileParseResult {
            file: file.to_path_buf(),
            symbols,
            edges,
        }
    }

    /// Go AST extraction
    fn parse_go(content: &str, file: &Path) -> FileParseResult {
        let mut symbols = Vec::new();
        let mut edges = Vec::new();

        let func_regex = Regex::new(
            r"^func\s+(?:\((?P<recv>[^)]+)\)\s+)?(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s*\(",
        )
        .unwrap();

        let type_struct_regex =
            Regex::new(r"^type\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s+struct\s*\{").unwrap();

        let type_interface_regex =
            Regex::new(r"^type\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s+interface\s*\{").unwrap();

        let import_regex = Regex::new(r#"import\s+(?:\(\s*)?"(?P<pkg>[^"]+)""#).unwrap();

        let call_regex =
            Regex::new(r"(?:[a-zA-Z_][a-zA-Z0-9_]*\.)?(?P<callee>[a-zA-Z_][a-zA-Z0-9_]*)\s*\(")
                .unwrap();

        let lines: Vec<&str> = content.lines().collect();
        let mut current_fn: Option<(String, usize)> = None;
        let mut brace_depth = 0;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // Parse struct
            if let Some(caps) = type_struct_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:struct:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Struct,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });
            }

            // Parse interface
            if let Some(caps) = type_interface_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:interface:{}", file.display(), line_num, name),
                    name: name.clone(),
                    qualified_name: name.clone(),
                    kind: SymbolKind::Interface,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });
            }

            // Parse func / receiver method
            if let Some(caps) = func_regex.captures(trimmed) {
                let name = caps["name"].to_string();
                let vis = if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    SymbolVisibility::Public
                } else {
                    SymbolVisibility::Private
                };

                let (kind, qual) = if let Some(recv) = caps.name("recv") {
                    let recv_str = recv.as_str().trim();
                    let recv_type = recv_str
                        .split_whitespace()
                        .last()
                        .unwrap_or("")
                        .trim_start_matches('*');
                    (SymbolKind::Method, format!("{recv_type}.{name}"))
                } else {
                    (SymbolKind::Function, name.clone())
                };

                symbols.push(CodeSymbol {
                    id: format!("{}:{}:func:{}", file.display(), line_num, qual),
                    name: name.clone(),
                    qualified_name: qual.clone(),
                    kind,
                    file: file.to_path_buf(),
                    line: line_num,
                    col: line.find(&name).unwrap_or(0) + 1,
                    end_line: line_num,
                    end_col: line.len(),
                    visibility: vis,
                    signature: trimmed.to_string(),
                    doc: None,
                });

                current_fn = Some((qual, brace_depth));
            }

            // Parse imports
            if let Some(caps) = import_regex.captures(trimmed) {
                let pkg = caps["pkg"].to_string();
                let short_pkg = pkg.split('/').last().unwrap_or(&pkg).to_string();
                edges.push(RawEdge {
                    caller_key: file.display().to_string(),
                    callee_name: short_pkg,
                    relation: SymbolRelation::Imports,
                    call_site_line: Some(line_num),
                });
            }

            // Call sites
            if let Some((ref caller_name, _)) = current_fn {
                for cap in call_regex.captures_iter(trimmed) {
                    let callee = cap["callee"].to_string();
                    let go_keywords = [
                        "if", "for", "switch", "select", "return", "go", "defer", "make", "new",
                        "append", "len", "cap", "close", "panic", "recover", "delete", "copy",
                    ];
                    if !go_keywords.contains(&callee.as_str()) && callee != *caller_name {
                        edges.push(RawEdge {
                            caller_key: caller_name.clone(),
                            callee_name: callee,
                            relation: SymbolRelation::Calls,
                            call_site_line: Some(line_num),
                        });
                    }
                }
            }

            let open = line.matches('{').count();
            let close = line.matches('}').count();
            brace_depth += open;

            if let Some((_, fn_depth)) = current_fn.clone() {
                if brace_depth <= fn_depth && close > 0 {
                    current_fn = None;
                }
            }

            if brace_depth >= close {
                brace_depth -= close;
            } else {
                brace_depth = 0;
            }
        }

        FileParseResult {
            file: file.to_path_buf(),
            symbols,
            edges,
        }
    }

    // =========================================================================
    // Query & Traversal Engine
    // =========================================================================

    /// Find symbols matching a query (exact name, qualified name, or substring)
    pub fn find_symbol(&self, query: &str) -> Vec<&CodeSymbol> {
        let mut results = Vec::new();

        // 1. Fast exact lookup in symbol_index
        if let Some(&idx) = self.symbol_index.get(query) {
            results.push(&self.graph[idx]);
        }

        // 2. Scan for exact matches on name or qualified name
        for node in self.graph.node_weights() {
            if (node.name == query || node.qualified_name == query)
                && !results.iter().any(|s| s.id == node.id)
            {
                results.push(node);
            }
        }

        // 3. If empty, fall back to case-insensitive substring search
        if results.is_empty() {
            let lower_q = query.to_lowercase();
            for node in self.graph.node_weights() {
                if node.name.to_lowercase().contains(&lower_q)
                    || node.qualified_name.to_lowercase().contains(&lower_q)
                {
                    results.push(node);
                }
            }
        }

        results
    }

    /// Find direct callers of a symbol (incoming Calls or References edges)
    pub fn find_callers(&self, query: &str) -> Vec<(&CodeSymbol, &SymbolEdge)> {
        let mut callers = Vec::new();
        let target_symbols = self.find_symbol(query);

        for target in target_symbols {
            if let Some(target_idx) = self.resolve_symbol_node(&target.id) {
                for edge in self.graph.edges_directed(target_idx, Direction::Incoming) {
                    let source_idx = edge.source();
                    let caller_sym = &self.graph[source_idx];
                    let sym_edge = edge.weight();
                    if sym_edge.relation == SymbolRelation::Calls
                        || sym_edge.relation == SymbolRelation::References
                        || sym_edge.relation == SymbolRelation::Implements
                    {
                        callers.push((caller_sym, sym_edge));
                    }
                }
            }
        }

        callers
    }

    /// Find direct callees of a symbol (outgoing Calls edges)
    pub fn find_callees(&self, query: &str) -> Vec<(&CodeSymbol, &SymbolEdge)> {
        let mut callees = Vec::new();
        let source_symbols = self.find_symbol(query);

        for source in source_symbols {
            if let Some(source_idx) = self.resolve_symbol_node(&source.id) {
                for edge in self.graph.edges_directed(source_idx, Direction::Outgoing) {
                    let target_idx = edge.target();
                    let callee_sym = &self.graph[target_idx];
                    let sym_edge = edge.weight();
                    if sym_edge.relation == SymbolRelation::Calls {
                        callees.push((callee_sym, sym_edge));
                    }
                }
            }
        }

        callees
    }

    /// Calculate transitive blast radius and refactoring risk classification
    pub fn calculate_blast_radius(
        &self,
        target_query: &str,
        max_depth: usize,
    ) -> Result<BlastRadiusReport> {
        let target_candidates = self.find_symbol(target_query);
        let target = target_candidates.into_iter().next().ok_or_else(|| {
            TagisanError::Execution(format!("Target symbol not found in graph: '{target_query}'"))
        })?;

        let target_idx = self
            .resolve_symbol_node(&target.id)
            .ok_or_else(|| TagisanError::Execution("Target node resolution failure".to_string()))?;

        let mut direct_callers = Vec::new();
        let mut transitive_callers = Vec::new();
        let mut implementing_types = Vec::new();
        let mut affected_files_set = HashSet::new();
        affected_files_set.insert(target.file.clone());

        let mut visited = HashSet::new();
        visited.insert(target_idx);

        let mut queue = VecDeque::new();
        // Queue elements: (NodeIndex, current_depth)
        queue.push_back((target_idx, 0));

        while let Some((curr_idx, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for edge in self.graph.edges_directed(curr_idx, Direction::Incoming) {
                let pred_idx = edge.source();
                let pred_sym = &self.graph[pred_idx];
                let relation = edge.weight().relation;

                if relation == SymbolRelation::Implements && depth == 0 {
                    implementing_types.push(pred_sym.qualified_name.clone());
                }

                if !visited.contains(&pred_idx) {
                    visited.insert(pred_idx);
                    affected_files_set.insert(pred_sym.file.clone());

                    if depth == 0 {
                        direct_callers.push(pred_sym.qualified_name.clone());
                    } else {
                        transitive_callers.push(pred_sym.qualified_name.clone());
                    }

                    queue.push_back((pred_idx, depth + 1));
                }
            }
        }

        let total_affected = visited.len();
        let callers_count = direct_callers.len() + transitive_callers.len();

        // Risk classification:
        // Low: 0-2 callers
        // Medium: 3-5 callers
        // High: 6-12 callers
        // Critical: 13+ callers or critical architectural traits with implementations
        let risk_level = if callers_count >= 13
            || (matches!(target.kind, SymbolKind::Trait | SymbolKind::Interface)
                && implementing_types.len() >= 3)
        {
            BlastRisk::Critical
        } else if callers_count >= 6 || !implementing_types.is_empty() {
            BlastRisk::High
        } else if callers_count >= 3 {
            BlastRisk::Medium
        } else {
            BlastRisk::Low
        };

        // Synthesize actionable refactoring recommendations
        let mut recommendations = Vec::new();
        match risk_level {
            BlastRisk::Low => {
                recommendations.push(format!(
                    "Low blast radius ({} dependents). Localized edits in '{}' are safe.",
                    callers_count,
                    target.file.display()
                ));
                recommendations.push(
                    "Execute unit tests covering the immediate module after refactoring."
                        .to_string(),
                );
            }
            BlastRisk::Medium => {
                recommendations.push(format!(
                    "Medium blast radius ({} callers across {} files).",
                    callers_count,
                    affected_files_set.len()
                ));
                recommendations.push(
                    "Inspect all direct call sites before modifying function parameter signatures."
                        .to_string(),
                );
                recommendations
                    .push("Run integration tests for all affected parent modules.".to_string());
            }
            BlastRisk::High => {
                recommendations.push(format!(
                    "HIGH RISK: Wide ripple effect across {} files and {} total symbols.",
                    affected_files_set.len(),
                    total_affected
                ));
                if !implementing_types.is_empty() {
                    recommendations.push(format!(
                        "Modifying this interface directly impacts implementing types: [{}].",
                        implementing_types.join(", ")
                    ));
                }
                recommendations.push(
                    "Consider adding a new overload/method instead of mutating existing signature."
                        .to_string(),
                );
                recommendations
                    .push("Execute full end-to-end regression test suite.".to_string());
            }
            BlastRisk::Critical => {
                recommendations.push(format!(
                    "CRITICAL ARCHITECTURAL HUB: {} callers transitively dependent across {} files.",
                    callers_count,
                    affected_files_set.len()
                ));
                recommendations.push(
                    "Requires RFC deprecation cycle or backwards-compatible facade adapter."
                        .to_string(),
                );
                recommendations.push(
                    "Stage the migration across distinct commits and run continuous integration."
                        .to_string(),
                );
            }
        }

        let mut affected_files: Vec<PathBuf> = affected_files_set.into_iter().collect();
        affected_files.sort();

        Ok(BlastRadiusReport {
            target_symbol: target.qualified_name.clone(),
            target_file: target.file.clone(),
            direct_callers,
            transitive_callers,
            implementing_types,
            affected_files,
            total_affected_symbols: total_affected,
            risk_level,
            recommendations,
        })
    }

    /// Compute summary statistics and centrality rankings
    pub fn stats(&self) -> GraphStats {
        let total_nodes = self.graph.node_count();
        let total_edges = self.graph.edge_count();

        let mut functions_count = 0;
        let mut types_count = 0;
        let mut modules_count = 0;

        let mut centrality: Vec<(String, usize)> = Vec::new();

        for idx in self.graph.node_indices() {
            let sym = &self.graph[idx];
            match sym.kind {
                SymbolKind::Function | SymbolKind::Method => functions_count += 1,
                SymbolKind::Struct
                | SymbolKind::Enum
                | SymbolKind::Trait
                | SymbolKind::Interface
                | SymbolKind::TypeAlias => types_count += 1,
                SymbolKind::Module => modules_count += 1,
                _ => {}
            }

            let in_degree = self
                .graph
                .edges_directed(idx, Direction::Incoming)
                .count();
            if in_degree > 0 {
                centrality.push((sym.qualified_name.clone(), in_degree));
            }
        }

        centrality.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        centrality.truncate(10);

        GraphStats {
            total_nodes,
            total_edges,
            functions_count,
            types_count,
            modules_count,
            files_count: self.file_index.len(),
            top_central_symbols: centrality,
        }
    }

    /// Export the codebase knowledge graph in Graphviz DOT format
    pub fn export_dot(&self) -> String {
        let mut out = String::from("digraph CodebaseGraph {\n");
        out.push_str("    rankdir=LR;\n");
        out.push_str("    node [shape=box, style=\"rounded,filled\", fillcolor=\"#F3F4F6\", fontname=\"Helvetica\"];\n");
        out.push_str("    edge [fontname=\"Helvetica\", fontsize=10];\n\n");

        for idx in self.graph.node_indices() {
            let sym = &self.graph[idx];
            let label = format!(
                "{} {}\\n{}:{}",
                sym.kind.as_str(),
                sym.name,
                sym.file.display(),
                sym.line
            );
            out.push_str(&format!(
                "    node_{} [label=\"{}\"];\n",
                idx.index(),
                label.replace('"', "\\\"")
            ));
        }

        out.push('\n');

        for edge in self.graph.edge_references() {
            out.push_str(&format!(
                "    node_{} -> node_{} [label=\"{}\"];\n",
                edge.source().index(),
                edge.target().index(),
                edge.weight().relation.as_str()
            ));
        }

        out.push_str("}\n");
        out
    }

    /// Export the codebase knowledge graph in structured JSON format
    pub fn export_json(&self) -> Result<String> {
        let nodes: Vec<&CodeSymbol> = self.graph.node_weights().collect();
        let edges: Vec<serde_json::Value> = self
            .graph
            .edge_references()
            .map(|e| {
                let from = &self.graph[e.source()];
                let to = &self.graph[e.target()];
                serde_json::json!({
                    "from": from.qualified_name,
                    "to": to.qualified_name,
                    "relation": e.weight().relation.as_str(),
                    "weight": e.weight().weight,
                    "call_site_line": e.weight().call_site_line,
                })
            })
            .collect();

        let root = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "stats": self.stats(),
        });

        serde_json::to_string_pretty(&root)
            .map_err(|e| TagisanError::Execution(format!("Failed to serialize graph JSON: {e}")))
    }
}
