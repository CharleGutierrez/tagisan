use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata associated with an individual text or code chunk
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkMetadata {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub language: String,
    pub token_estimate: usize,
}

/// An individual chunk of code or text with identifier and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    pub id: String,
    pub text: String,
    pub metadata: ChunkMetadata,
}

/// Configurable chunker that splits source files into overlapping line-based chunks
#[derive(Debug, Clone)]
pub struct CodeChunker {
    pub chunk_size_lines: usize,
    pub overlap_lines: usize,
}

impl Default for CodeChunker {
    fn default() -> Self {
        Self {
            chunk_size_lines: 60,
            overlap_lines: 10,
        }
    }
}

impl CodeChunker {
    pub fn new(chunk_size_lines: usize, overlap_lines: usize) -> Self {
        let overlap = if overlap_lines >= chunk_size_lines {
            chunk_size_lines.saturating_sub(1)
        } else {
            overlap_lines
        };
        Self {
            chunk_size_lines: chunk_size_lines.max(1),
            overlap_lines: overlap,
        }
    }

    /// Detect programming or markup language from file extension
    pub fn detect_language(path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
            Some("rs") => "rust".to_string(),
            Some("py") => "python".to_string(),
            Some("js") => "javascript".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("tsx") => "typescript_react".to_string(),
            Some("jsx") => "javascript_react".to_string(),
            Some("go") => "go".to_string(),
            Some("c") | Some("h") => "c".to_string(),
            Some("cpp") | Some("hpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
            Some("java") => "java".to_string(),
            Some("kt") | Some("kts") => "kotlin".to_string(),
            Some("json") => "json".to_string(),
            Some("toml") => "toml".to_string(),
            Some("yaml") | Some("yml") => "yaml".to_string(),
            Some("md") | Some("markdown") => "markdown".to_string(),
            Some("sh") | Some("bash") | Some("zsh") => "bash".to_string(),
            Some("sql") => "sql".to_string(),
            Some("html") | Some("htm") => "html".to_string(),
            Some("css") | Some("scss") => "css".to_string(),
            _ => "text".to_string(),
        }
    }

    /// Split file content into overlapping chunks with precise line numbering
    pub fn chunk_text(&self, file_path: &str, content: &str) -> Vec<Chunk> {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        if total_lines == 0 {
            return Vec::new();
        }

        let language = Self::detect_language(Path::new(file_path));

        // If file is smaller than chunk size, return single chunk
        if total_lines <= self.chunk_size_lines {
            let text = content.to_string();
            let token_estimate = text.split_whitespace().count() * 4 / 3 + 1;
            return vec![Chunk {
                id: format!("{}#L1-L{}", file_path, total_lines),
                text,
                metadata: ChunkMetadata {
                    file_path: file_path.to_string(),
                    start_line: 1,
                    end_line: total_lines,
                    language,
                    token_estimate,
                },
            }];
        }

        let mut chunks = Vec::new();
        let step = self.chunk_size_lines.saturating_sub(self.overlap_lines).max(1);
        let mut start_idx = 0;

        while start_idx < total_lines {
            let end_idx = (start_idx + self.chunk_size_lines).min(total_lines);
            let chunk_lines = &lines[start_idx..end_idx];
            let chunk_text = chunk_lines.join("\n");

            let start_line = start_idx + 1;
            let end_line = end_idx;
            let token_estimate = chunk_text.split_whitespace().count() * 4 / 3 + 1;

            chunks.push(Chunk {
                id: format!("{}#L{}-L{}", file_path, start_line, end_line),
                text: chunk_text,
                metadata: ChunkMetadata {
                    file_path: file_path.to_string(),
                    start_line,
                    end_line,
                    language: language.clone(),
                    token_estimate,
                },
            });

            if end_idx >= total_lines {
                break;
            }
            start_idx += step;
        }

        chunks
    }
}
