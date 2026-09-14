use crate::error::{Result, TagisanError};
use crate::mcp::config::McpServerConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Individual plugin catalog entry representing an external MCP tool server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpCatalogEntry {
    #[serde(default)]
    pub index: usize,
    pub name: String,
    pub domain: String,
    #[serde(default)]
    pub domain_summary: String,
    pub authority: String,
    pub description: String,
    #[serde(default)]
    pub tgs_target: String,
    #[serde(default)]
    pub transport: String,
    #[serde(default)]
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawCatalog {
    #[serde(default)]
    pub total_count: usize,
    #[serde(default)]
    pub plugins: Vec<McpCatalogEntry>,
}

/// The 500-plugin sovereign MCP catalog index
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpCatalog {
    #[serde(default)]
    pub total_count: usize,
    #[serde(rename = "plugins", default)]
    pub entries: Vec<McpCatalogEntry>,
}

impl McpCatalog {
    /// Create a new empty MCP catalog
    pub fn new() -> Self {
        Self {
            total_count: 0,
            entries: Vec::new(),
        }
    }

    /// Load and parse the 500-plugin catalog from JSON string
    pub fn from_json(json_str: &str) -> Result<Self> {
        let raw: RawCatalog = serde_json::from_str(json_str).map_err(|e| {
            TagisanError::BadResponse(
                "mcp_catalog".to_string(),
                format!("Failed to parse MCP catalog JSON: {e}"),
            )
        })?;

        let count = if raw.total_count > 0 {
            raw.total_count
        } else {
            raw.plugins.len()
        };

        Ok(Self {
            total_count: count,
            entries: raw.plugins,
        })
    }

    /// Load and parse the catalog from a specific file path
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read MCP catalog file '{}': {e}",
                path_ref.display()
            ))
        })?;
        Self::from_json(&content)
    }

    /// Load the default 500-plugin MCP catalog from `.ecc/mcp_catalog.json` with robust fallbacks
    pub fn load_default() -> Result<Self> {
        let candidates = [
            PathBuf::from(".ecc/mcp_catalog.json"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".ecc").join("mcp_catalog.json"),
            PathBuf::from("../.ecc/mcp_catalog.json"),
            PathBuf::from("../../.ecc/mcp_catalog.json"),
            PathBuf::from("tagisan/.ecc/mcp_catalog.json"),
        ];

        for candidate in &candidates {
            if candidate.is_file() {
                return Self::from_file(candidate);
            }
        }

        // Try searching upward from current working directory
        if let Ok(cwd) = std::env::current_dir() {
            let mut cur = Some(cwd.as_path());
            while let Some(dir) = cur {
                let candidate = dir.join(".ecc").join("mcp_catalog.json");
                if candidate.is_file() {
                    return Self::from_file(&candidate);
                }
                cur = dir.parent();
            }
        }

        // Try searching upward from current executable
        if let Ok(exe) = std::env::current_exe() {
            let mut cur = exe.parent();
            while let Some(dir) = cur {
                let candidate = dir.join(".ecc").join("mcp_catalog.json");
                if candidate.is_file() {
                    return Self::from_file(&candidate);
                }
                cur = dir.parent();
            }
        }

        Err(TagisanError::Execution(
            "Could not locate '.ecc/mcp_catalog.json' in working directory, manifest dir, or parent directories".to_string()
        ))
    }

    /// Number of catalog entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the catalog is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retrieve an entry by exact name
    pub fn get_by_name(&self, name: &str) -> Option<&McpCatalogEntry> {
        self.entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Retrieve an entry by 1-based or 0-based index
    pub fn get_by_index(&self, index: usize) -> Option<&McpCatalogEntry> {
        self.entries.iter().find(|e| e.index == index)
            .or_else(|| self.entries.get(index))
    }

    /// Unique sorted list of domains present in the catalog
    pub fn domains(&self) -> Vec<String> {
        let set: HashSet<String> = self.entries.iter().map(|e| e.domain.clone()).collect();
        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        list
    }

    /// Unique sorted list of authorities present in the catalog
    pub fn authorities(&self) -> Vec<String> {
        let set: HashSet<String> = self.entries.iter().map(|e| e.authority.clone()).collect();
        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        list
    }

    /// Filter catalog entries by domain substring (case-insensitive)
    pub fn filter_by_domain(&self, domain_pattern: &str) -> Vec<&McpCatalogEntry> {
        let query = domain_pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.domain.to_lowercase().contains(&query) || e.domain_summary.to_lowercase().contains(&query))
            .collect()
    }

    /// Filter catalog entries by authority substring (case-insensitive)
    pub fn filter_by_authority(&self, authority_pattern: &str) -> Vec<&McpCatalogEntry> {
        let query = authority_pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.authority.to_lowercase().contains(&query))
            .collect()
    }

    /// Fast multi-keyword search with relevance scoring
    pub fn search(&self, query: &str) -> Vec<&McpCatalogEntry> {
        self.search_filtered(Some(query), None, None)
    }

    /// Search across catalog with optional domain and authority filters
    pub fn search_filtered(
        &self,
        query: Option<&str>,
        domain_filter: Option<&str>,
        authority_filter: Option<&str>,
    ) -> Vec<&McpCatalogEntry> {
        let domain_q = domain_filter.map(|d| d.to_lowercase());
        let auth_q = authority_filter.map(|a| a.to_lowercase());

        let raw_query = query.unwrap_or("").trim();
        let keywords: Vec<String> = raw_query
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        let mut scored_entries: Vec<(i64, &McpCatalogEntry)> = Vec::new();

        for entry in &self.entries {
            // Check domain filter
            if let Some(ref dq) = domain_q {
                if !entry.domain.to_lowercase().contains(dq)
                    && !entry.domain_summary.to_lowercase().contains(dq)
                {
                    continue;
                }
            }

            // Check authority filter
            if let Some(ref aq) = auth_q {
                if !entry.authority.to_lowercase().contains(aq) {
                    continue;
                }
            }

            // If no search query provided, include all matching domain/authority filters with default score
            if keywords.is_empty() {
                scored_entries.push((0, entry));
                continue;
            }

            let name_lower = entry.name.to_lowercase();
            let desc_lower = entry.description.to_lowercase();
            let domain_lower = entry.domain.to_lowercase();
            let summary_lower = entry.domain_summary.to_lowercase();
            let auth_lower = entry.authority.to_lowercase();

            let mut matched_keywords_count = 0;
            let mut score = 0i64;

            // Exact full query match
            let full_q = raw_query.to_lowercase();
            if name_lower == full_q {
                score += 500;
            } else if name_lower.starts_with(&full_q) {
                score += 250;
            } else if name_lower.contains(&full_q) {
                score += 150;
            }

            for kw in &keywords {
                let mut kw_matched = false;

                if name_lower == *kw {
                    score += 200;
                    kw_matched = true;
                } else if name_lower.contains(kw) {
                    score += 80;
                    kw_matched = true;
                }

                if desc_lower.contains(kw) {
                    score += 35;
                    kw_matched = true;
                }

                if domain_lower.contains(kw) || summary_lower.contains(kw) {
                    score += 20;
                    kw_matched = true;
                }

                if auth_lower.contains(kw) {
                    score += 15;
                    kw_matched = true;
                }

                if kw_matched {
                    matched_keywords_count += 1;
                }
            }

            // Must match at least one keyword
            if matched_keywords_count > 0 {
                // Bonus for matching all keywords
                if matched_keywords_count == keywords.len() {
                    score += 1000;
                } else {
                    score += (matched_keywords_count as i64) * 100;
                }
                scored_entries.push((score, entry));
            }
        }

        // Sort by score descending, then by catalog index ascending
        scored_entries.sort_by(|a, b| {
            b.0.cmp(&a.0).then_with(|| a.1.index.cmp(&b.1.index))
        });

        scored_entries.into_iter().map(|(_, e)| e).collect()
    }

    /// Generate real, runnable commands, arguments, environment placeholders,
    /// and AgentShield security allowances for an MCP catalog entry.
    pub fn generate_server_config(&self, entry: &McpCatalogEntry) -> McpServerConfig {
        Self::entry_to_server_config(entry)
    }

    /// Static constructor for transforming an McpCatalogEntry into a production McpServerConfig
    pub fn entry_to_server_config(entry: &McpCatalogEntry) -> McpServerConfig {
        let auth_lower = entry.authority.to_lowercase();
        let name_lower = entry.name.to_lowercase();

        let (command, args) = if auth_lower.contains("pypi") {
            ("uvx".to_string(), vec![entry.name.clone()])
        } else if auth_lower.contains("smithery") {
            (
                "npx".to_string(),
                vec![
                    "-y".to_string(),
                    "@smithery/cli".to_string(),
                    "run".to_string(),
                    entry.name.clone(),
                ],
            )
        } else if auth_lower.contains("npm") {
            ("npx".to_string(), vec!["-y".to_string(), entry.name.clone()])
        } else if auth_lower.contains("glama") {
            (
                "npx".to_string(),
                vec!["-y".to_string(), format!("@glama/{}", entry.name)],
            )
        } else if auth_lower.contains("cloudflare") {
            (
                "npx".to_string(),
                vec![
                    "-y".to_string(),
                    format!("@cloudflare/mcp-{}", entry.name.trim_end_matches("-mcp")),
                ],
            )
        } else if auth_lower.contains("composio") {
            (
                "uvx".to_string(),
                vec!["composio-core".to_string(), "mcp".to_string(), entry.name.clone()],
            )
        } else if auth_lower.contains("zapier") {
            (
                "npx".to_string(),
                vec!["-y".to_string(), "@zapier/mcp-server".to_string()],
            )
        } else if auth_lower.contains("postman") {
            (
                "npx".to_string(),
                vec!["-y".to_string(), "@postman/mcp-server".to_string()],
            )
        } else if auth_lower.contains("hugging face") {
            ("uvx".to_string(), vec!["mcp-server-hf".to_string()])
        } else if auth_lower.contains("ollama") {
            ("ollama".to_string(), vec!["run".to_string(), "mcp-bridge".to_string()])
        } else {
            ("npx".to_string(), vec!["-y".to_string(), entry.name.clone()])
        };

        // Populate environment variables and placeholders
        let mut env = HashMap::new();

        // AgentShield sandboxing and security allowances
        env.insert("TGS_AGENTSHIELD_PROFILE".to_string(), "sandboxed".to_string());
        env.insert("TGS_SECURITY_ALLOWANCE".to_string(), "mcp-stdio-subsystem".to_string());
        env.insert("TGS_AUTHORITY".to_string(), entry.authority.clone());
        env.insert("TGS_MCP_PLUGIN".to_string(), entry.name.clone());

        // Domain & service specific environment variables
        if name_lower.contains("brave") {
            env.insert("BRAVE_API_KEY".to_string(), "${BRAVE_API_KEY}".to_string());
        } else if name_lower.contains("exa") {
            env.insert("EXA_API_KEY".to_string(), "${EXA_API_KEY}".to_string());
        } else if name_lower.contains("tavily") {
            env.insert("TAVILY_API_KEY".to_string(), "${TAVILY_API_KEY}".to_string());
        } else if name_lower.contains("perplexity") {
            env.insert("PERPLEXITY_API_KEY".to_string(), "${PERPLEXITY_API_KEY}".to_string());
        } else if name_lower.contains("firecrawl") {
            env.insert("FIRECRAWL_API_KEY".to_string(), "${FIRECRAWL_API_KEY}".to_string());
        } else if name_lower.contains("scrapingbee") {
            env.insert("SCRAPINGBEE_API_KEY".to_string(), "${SCRAPINGBEE_API_KEY}".to_string());
        } else if name_lower.contains("brightdata") {
            env.insert("BRIGHTDATA_API_KEY".to_string(), "${BRIGHTDATA_API_KEY}".to_string());
        } else if name_lower.contains("diffbot") {
            env.insert("DIFFBOT_API_KEY".to_string(), "${DIFFBOT_API_KEY}".to_string());
        } else if name_lower.contains("serpapi") || name_lower.contains("serper") {
            env.insert("SERPAPI_API_KEY".to_string(), "${SERPAPI_API_KEY}".to_string());
        } else if name_lower.contains("postgres") || name_lower.contains("neon") || name_lower.contains("supabase") {
            env.insert("DATABASE_URL".to_string(), "${DATABASE_URL}".to_string());
        } else if name_lower.contains("redis") || name_lower.contains("upstash") {
            env.insert("REDIS_URL".to_string(), "${REDIS_URL}".to_string());
        } else if name_lower.contains("mongo") {
            env.insert("MONGODB_URI".to_string(), "${MONGODB_URI}".to_string());
        } else if name_lower.contains("snowflake") {
            env.insert("SNOWFLAKE_ACCOUNT".to_string(), "${SNOWFLAKE_ACCOUNT}".to_string());
            env.insert("SNOWFLAKE_USER".to_string(), "${SNOWFLAKE_USER}".to_string());
        } else if name_lower.contains("bigquery") {
            env.insert("BIGQUERY_PROJECT_ID".to_string(), "${BIGQUERY_PROJECT_ID}".to_string());
        } else if name_lower.contains("clickhouse") {
            env.insert("CLICKHOUSE_URL".to_string(), "${CLICKHOUSE_URL}".to_string());
        } else if name_lower.contains("pinecone") {
            env.insert("PINECONE_API_KEY".to_string(), "${PINECONE_API_KEY}".to_string());
        } else if name_lower.contains("qdrant") {
            env.insert("QDRANT_URL".to_string(), "${QDRANT_URL:-http://localhost:6333}".to_string());
            env.insert("QDRANT_API_KEY".to_string(), "${QDRANT_API_KEY:-}".to_string());
        } else if name_lower.contains("weaviate") {
            env.insert("WEAVIATE_URL".to_string(), "${WEAVIATE_URL:-http://localhost:8080}".to_string());
        } else if name_lower.contains("milvus") {
            env.insert("MILVUS_URI".to_string(), "${MILVUS_URI:-http://localhost:19530}".to_string());
        } else if name_lower.contains("chroma") {
            env.insert("CHROMA_URL".to_string(), "${CHROMA_URL:-http://localhost:8000}".to_string());
        } else if name_lower.contains("cloudflare") {
            env.insert("CLOUDFLARE_API_TOKEN".to_string(), "${CLOUDFLARE_API_TOKEN}".to_string());
        } else if name_lower.contains("aws") {
            env.insert("AWS_REGION".to_string(), "${AWS_REGION:-us-east-1}".to_string());
        } else if name_lower.contains("kubernetes") || name_lower.contains("k8s") {
            env.insert("KUBECONFIG".to_string(), "${KUBECONFIG:-~/.kube/config}".to_string());
        } else if name_lower.contains("docker") {
            env.insert("DOCKER_HOST".to_string(), "${DOCKER_HOST:-unix:///var/run/docker.sock}".to_string());
        } else if name_lower.contains("datadog") {
            env.insert("DD_API_KEY".to_string(), "${DD_API_KEY}".to_string());
        } else if name_lower.contains("sentry") {
            env.insert("SENTRY_AUTH_TOKEN".to_string(), "${SENTRY_AUTH_TOKEN}".to_string());
        } else if name_lower.contains("shodan") {
            env.insert("SHODAN_API_KEY".to_string(), "${SHODAN_API_KEY}".to_string());
        } else if name_lower.contains("virustotal") {
            env.insert("VIRUSTOTAL_API_KEY".to_string(), "${VIRUSTOTAL_API_KEY}".to_string());
        } else if name_lower.contains("github") {
            env.insert("GITHUB_PERSONAL_ACCESS_TOKEN".to_string(), "${GITHUB_TOKEN}".to_string());
        } else if name_lower.contains("slack") {
            env.insert("SLACK_BOT_TOKEN".to_string(), "${SLACK_BOT_TOKEN}".to_string());
        } else if name_lower.contains("jira") {
            env.insert("JIRA_API_TOKEN".to_string(), "${JIRA_API_TOKEN}".to_string());
        } else if name_lower.contains("notion") {
            env.insert("NOTION_API_KEY".to_string(), "${NOTION_API_KEY}".to_string());
        } else if name_lower.contains("linear") {
            env.insert("LINEAR_API_KEY".to_string(), "${LINEAR_API_KEY}".to_string());
        } else if name_lower.contains("openai") {
            env.insert("OPENAI_API_KEY".to_string(), "${OPENAI_API_KEY}".to_string());
        } else if name_lower.contains("anthropic") {
            env.insert("ANTHROPIC_API_KEY".to_string(), "${ANTHROPIC_API_KEY}".to_string());
        } else if name_lower.contains("gemini") || name_lower.contains("google") {
            env.insert("GEMINI_API_KEY".to_string(), "${GEMINI_API_KEY}".to_string());
        } else if name_lower.contains("huggingface") || name_lower.contains("hf-") {
            env.insert("HF_TOKEN".to_string(), "${HF_TOKEN}".to_string());
        }

        McpServerConfig {
            command,
            args,
            env,
            authority: Some(entry.authority.clone()),
            subsystem: Some(entry.domain.clone()),
            auto_approve: Some(false),
            description: Some(entry.description.clone()),
        }
    }
}
