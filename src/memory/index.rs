use crate::error::{Result, TagisanError};
use crate::memory::chunking::CodeChunker;
use crate::memory::embedding::EmbeddingProvider;
use crate::memory::store::{VectorDocument, VectorStore};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};

/// An individual rule parsed from a .gitignore file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitIgnoreRule {
    /// The glob pattern (without negation prefix or trailing slash)
    pub pattern: String,
    /// Whether this is a negation rule (starts with '!')
    pub is_negation: bool,
    /// Whether this rule only matches directories (ended with '/')
    pub directory_only: bool,
    /// Whether this rule is anchored to the root (started with '/' or contains a '/' in the middle)
    pub anchored: bool,
}

impl GitIgnoreRule {
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        let (is_negation, raw_pattern) = if let Some(stripped) = trimmed.strip_prefix('!') {
            (true, stripped)
        } else if let Some(stripped) = trimmed.strip_prefix(r"\#") {
            (false, stripped)
        } else if let Some(stripped) = trimmed.strip_prefix(r"\!") {
            (false, stripped)
        } else {
            (false, trimmed)
        };

        if raw_pattern.is_empty() {
            return None;
        }

        let (directory_only, pattern_without_slash) = if let Some(stripped) = raw_pattern.strip_suffix('/') {
            (true, stripped)
        } else {
            (false, raw_pattern)
        };

        let (anchored, final_pattern) = if let Some(stripped) = pattern_without_slash.strip_prefix('/') {
            (true, stripped)
        } else if pattern_without_slash.contains('/') {
            (true, pattern_without_slash)
        } else {
            (false, pattern_without_slash)
        };

        if final_pattern.is_empty() {
            return None;
        }

        Some(Self {
            pattern: final_pattern.to_string(),
            is_negation,
            directory_only,
            anchored,
        })
    }
}

/// Single segment matching for wildcard characters ('*' and '?')
fn match_single_segment(pat: &str, text: &str) -> bool {
    let p_chars: Vec<char> = pat.chars().collect();
    let t_chars: Vec<char> = text.chars().collect();

    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while t_idx < t_chars.len() {
        if p_idx < p_chars.len() && (p_chars[p_idx] == '?' || p_chars[p_idx] == t_chars[t_idx]) {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p_chars.len() && p_chars[p_idx] == '*' {
            star_idx = Some(p_idx);
            match_idx = t_idx;
            p_idx += 1;
        } else if let Some(star) = star_idx {
            p_idx = star + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_chars.len() && p_chars[p_idx] == '*' {
        p_idx += 1;
    }

    p_idx == p_chars.len()
}

/// Match a sequence of pattern segments against a sequence of path segments.
/// Supports `**` (zero or more directories).
fn match_segments(p_segs: &[&str], t_segs: &[&str]) -> bool {
    if p_segs.is_empty() {
        return t_segs.is_empty();
    }

    if p_segs[0] == "**" {
        for i in 0..=t_segs.len() {
            if match_segments(&p_segs[1..], &t_segs[i..]) {
                return true;
            }
        }
        return false;
    }

    if t_segs.is_empty() {
        return false;
    }

    if match_single_segment(p_segs[0], t_segs[0]) {
        match_segments(&p_segs[1..], &t_segs[1..])
    } else {
        false
    }
}

/// Standard .gitignore matcher implementing git wildcard, negation, comment,
/// directory slash, and recursive traversal rules.
#[derive(Debug, Clone, Default)]
pub struct GitIgnoreMatcher {
    pub rules: Vec<GitIgnoreRule>,
}

impl GitIgnoreMatcher {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Standard default ignore rules (target/, node_modules/, .git/, etc.)
    pub fn default_rules() -> Self {
        let mut matcher = Self::new();
        matcher.add_rule(".*").ok();
        matcher.add_rule("target/").ok();
        matcher.add_rule("node_modules/").ok();
        matcher.add_rule("dist/").ok();
        matcher.add_rule("build/").ok();
        matcher.add_rule("bin/").ok();
        matcher.add_rule("obj/").ok();
        matcher.add_rule("vendor/").ok();
        matcher
    }

    pub fn add_rule(&mut self, rule_str: &str) -> Result<()> {
        if let Some(rule) = GitIgnoreRule::parse(rule_str) {
            self.rules.push(rule);
        }
        Ok(())
    }

    pub fn parse_str(&mut self, content: &str) {
        for line in content.lines() {
            if let Some(rule) = GitIgnoreRule::parse(line) {
                self.rules.push(rule);
            }
        }
    }

    pub fn parse_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let p = path.as_ref();
        if p.exists() {
            let content = fs::read_to_string(p).map_err(TagisanError::Io)?;
            self.parse_str(&content);
        }
        Ok(())
    }

    /// Checks whether a normalized relative path is ignored according to gitignore rules.
    pub fn is_ignored(&self, rel_path: &str, is_dir: bool) -> bool {
        let clean_path = rel_path.replace('\\', "/").trim_matches('/').to_string();
        if clean_path.is_empty() {
            return false;
        }

        let segments: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return false;
        }

        let mut ignored = false;

        for rule in &self.rules {
            if rule.directory_only && !is_dir {
                continue;
            }

            let matches = if rule.anchored {
                let p_segs: Vec<&str> = rule.pattern.split('/').filter(|s| !s.is_empty()).collect();
                if match_segments(&p_segs, &segments) {
                    true
                } else if !rule.is_negation {
                    (1..segments.len()).any(|k| match_segments(&p_segs, &segments[..k]))
                } else {
                    false
                }
            } else {
                let filename = segments[segments.len() - 1];
                if match_single_segment(&rule.pattern, filename) {
                    true
                } else if !rule.is_negation {
                    if rule.directory_only {
                        let dir_segs = if is_dir { &segments[..] } else { &segments[..segments.len() - 1] };
                        dir_segs.iter().any(|seg| match_single_segment(&rule.pattern, seg))
                    } else {
                        segments.iter().any(|seg| match_single_segment(&rule.pattern, seg))
                    }
                } else {
                    false
                }
            };

            if matches {
                ignored = !rule.is_negation;
            }
        }

        ignored
    }

    /// Convenience wrapper for Path references
    pub fn is_path_ignored(&self, path: &Path, is_dir: bool) -> bool {
        self.is_ignored(&path.to_string_lossy(), is_dir)
    }
}

/// Scans source trees, splits code into overlapping chunks, generates embeddings,
/// and indexes them into an active VectorStore for semantic retrieval.
#[derive(Clone)]
pub struct CodebaseIndexer {
    pub chunker: CodeChunker,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub max_file_size_bytes: usize,
    pub ignore_matcher: GitIgnoreMatcher,
}

impl CodebaseIndexer {
    pub fn new(embedding_provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self {
            chunker: CodeChunker::default(),
            embedding_provider,
            max_file_size_bytes: 512 * 1024, // 512 KB
            ignore_matcher: GitIgnoreMatcher::default_rules(),
        }
    }

    pub fn with_ignore_matcher(mut self, matcher: GitIgnoreMatcher) -> Self {
        self.ignore_matcher = matcher;
        self
    }

    pub fn with_chunker(mut self, chunker: CodeChunker) -> Self {
        self.chunker = chunker;
        self
    }

    pub fn with_max_file_size(mut self, max_bytes: usize) -> Self {
        self.max_file_size_bytes = max_bytes;
        self
    }

    /// Check if a directory or file path should be ignored during indexing using default rules
    pub fn is_ignored_path(path: &Path) -> bool {
        let matcher = GitIgnoreMatcher::default_rules();
        matcher.is_path_ignored(path, path.is_dir())
    }

    /// Check if file is a known binary or non-source asset
    pub fn is_binary_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref(),
            Some("exe") | Some("dll") | Some("so") | Some("dylib") | Some("bin") | Some("iso")
            | Some("zip") | Some("tar") | Some("gz") | Some("7z") | Some("rar")
            | Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("ico")
            | Some("pdf") | Some("lock") | Some("pyc") | Some("wasm") | Some("o") | Some("a")
        )
    }

    /// Recursively discover all indexable files under root
    pub fn discover_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut matcher = self.ignore_matcher.clone();
        let root_gitignore = root.join(".gitignore");
        if root_gitignore.exists() {
            let _ = matcher.parse_file(&root_gitignore);
        }

        let mut files = Vec::new();
        let mut dirs = vec![root.to_path_buf()];

        while let Some(current_dir) = dirs.pop() {
            let entries = match fs::read_dir(&current_dir) {
                Ok(e) => e,
                Err(err) => {
                    debug!("Skipping unreadable directory {:?}: {err}", current_dir);
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let is_dir = path.is_dir();
                let rel_path = path.strip_prefix(root).unwrap_or(&path);

                if matcher.is_path_ignored(rel_path, is_dir) {
                    continue;
                }

                if is_dir {
                    let nested_gitignore = path.join(".gitignore");
                    if nested_gitignore.exists() {
                        let _ = matcher.parse_file(&nested_gitignore);
                    }
                    dirs.push(path);
                } else if path.is_file() {
                    if Self::is_binary_file(&path) {
                        continue;
                    }

                    if let Ok(meta) = entry.metadata() {
                        if meta.len() as usize <= self.max_file_size_bytes {
                            files.push(path);
                        }
                    }
                }
            }
        }

        files.sort();
        files
    }

    /// Index all eligible source files under `root` into the specified `VectorStore`
    pub async fn index_directory(&self, root: impl AsRef<Path>, store: &VectorStore) -> Result<usize> {
        let root_ref = root.as_ref();
        if !root_ref.exists() {
            return Err(TagisanError::Execution(format!(
                "Directory does not exist: {:?}",
                root_ref
            )));
        }

        let files = self.discover_files(root_ref);
        info!("Discovered {} indexable source files under {:?}", files.len(), root_ref);

        let mut total_chunks = 0;

        for file_path in &files {
            let content = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => {
                    // Non-UTF-8 or inaccessible file, skip
                    continue;
                }
            };

            let rel_path = file_path
                .strip_prefix(root_ref)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "/");

            let chunks = self.chunker.chunk_text(&rel_path, &content);
            if chunks.is_empty() {
                continue;
            }

            for chunk in chunks {
                let embedding = self.embedding_provider.embed_text(&chunk.text).await?;

                let mut metadata = HashMap::new();
                metadata.insert("file_path".to_string(), chunk.metadata.file_path);
                metadata.insert("start_line".to_string(), chunk.metadata.start_line.to_string());
                metadata.insert("end_line".to_string(), chunk.metadata.end_line.to_string());
                metadata.insert("language".to_string(), chunk.metadata.language);
                metadata.insert("token_estimate".to_string(), chunk.metadata.token_estimate.to_string());
                metadata.insert("type".to_string(), "code_chunk".to_string());

                let doc = VectorDocument::new(chunk.id, chunk.text, embedding).with_metadata(metadata);
                store.add_document(doc)?;
                total_chunks += 1;
            }
        }

        info!("Successfully indexed {} total chunks into VectorStore", total_chunks);
        Ok(total_chunks)
    }
}
