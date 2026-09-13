//! Native Web Search Tool for Tagisan (tgs)
//!
//! Provides zero-configuration web search via DuckDuckGo (HTML scraping & API fallback),
//! with optional acceleration via Tavily (`TAVILY_API_KEY`) or Brave Search (`BRAVE_API_KEY`),
//! and direct webpage content fetching and readable text extraction.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

/// Search result item containing title, URL, and snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Native web search tool with zero-API-key DuckDuckGo fallback and multi-engine support
#[derive(Clone, Debug)]
pub struct WebSearchTool {
    client: reqwest::Client,
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchTool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// Perform web search with automatic engine selection:
    /// 1. Tavily if `TAVILY_API_KEY` is present.
    /// 2. Brave if `BRAVE_API_KEY` is present.
    /// 3. Zero-config DuckDuckGo HTML search with API fallback.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<WebSearchResult>> {
        if let Ok(api_key) = std::env::var("TAVILY_API_KEY") {
            if !api_key.trim().is_empty() {
                if let Ok(results) = self.search_tavily(query, &api_key, limit).await {
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
            }
        }

        if let Ok(api_key) = std::env::var("BRAVE_API_KEY") {
            if !api_key.trim().is_empty() {
                if let Ok(results) = self.search_brave(query, &api_key, limit).await {
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
            }
        }

        // Default zero-key DuckDuckGo search
        self.search_duckduckgo(query, limit).await
    }

    /// Search DuckDuckGo using HTML scraping with instant-answer API fallback
    pub async fn search_duckduckgo(&self, query: &str, limit: usize) -> Result<Vec<WebSearchResult>> {
        let url = match reqwest::Url::parse_with_params("https://html.duckduckgo.com/html/", &[("q", query)]) {
            Ok(u) => u,
            Err(e) => return Err(TagisanError::Execution(format!("Invalid search query URL: {e}"))),
        };

        let resp = self
            .client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await;

        if let Ok(res) = resp {
            if res.status().is_success() {
                let html = res.text().await.unwrap_or_default();
                let results = self.parse_duckduckgo_html(&html, limit);
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }

        // Fallback to DuckDuckGo Instant Answer JSON API
        self.search_duckduckgo_api(query, limit).await
    }

    /// Parse organic search results from DuckDuckGo HTML
    pub fn parse_duckduckgo_html(&self, html: &str, limit: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();

        let title_re = match Regex::new(r#"<a[^>]*class="[^"]*result__a[^"]*"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#) {
            Ok(r) => r,
            Err(_) => return results,
        };

        let snippet_re = match Regex::new(r#"<a[^>]*class="result__snippet"[^>]*>(.*?)</a>"#) {
            Ok(r) => r,
            Err(_) => return results,
        };

        let tag_re = Regex::new(r"<[^>]+>").ok();

        // Collect snippets
        let mut snippets = Vec::new();
        for cap in snippet_re.captures_iter(html) {
            if let Some(m) = cap.get(1) {
                let clean = if let Some(ref tr) = tag_re {
                    tr.replace_all(m.as_str(), "").to_string()
                } else {
                    m.as_str().to_string()
                };
                snippets.push(Self::decode_html_entities(&clean).trim().to_string());
            }
        }

        let mut idx = 0;
        for cap in title_re.captures_iter(html) {
            let raw_url = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let title_html = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Extract real destination URL from uddg= parameter
            let mut actual_url = raw_url.to_string();
            if let Some(start_pos) = raw_url.find("uddg=") {
                let sub = &raw_url[start_pos + 5..];
                let end_pos = sub.find('&').unwrap_or(sub.len());
                let encoded = &sub[..end_pos];
                // Percent-decode using URL parser or unescape
                let dummy_url = format!("http://dummy/?x={}", encoded);
                if let Ok(parsed) = reqwest::Url::parse(&dummy_url) {
                    for (k, v) in parsed.query_pairs() {
                        if k == "x" {
                            actual_url = v.into_owned();
                            break;
                        }
                    }
                }
            } else if actual_url.starts_with("//") {
                actual_url = format!("https:{}", actual_url);
            }

            // Skip ads
            if actual_url.contains("duckduckgo.com/y.js")
                || actual_url.contains("bing.com/aclick")
                || actual_url.contains("googleadservices")
            {
                continue;
            }

            let clean_title = if let Some(ref tr) = tag_re {
                tr.replace_all(title_html, "").to_string()
            } else {
                title_html.to_string()
            };
            let decoded_title = Self::decode_html_entities(&clean_title).trim().to_string();

            let snippet = if idx < snippets.len() {
                snippets[idx].clone()
            } else {
                String::new()
            };

            if !decoded_title.is_empty() && !actual_url.is_empty() {
                results.push(WebSearchResult {
                    title: decoded_title,
                    url: actual_url,
                    snippet,
                });
            }

            idx += 1;
            if results.len() >= limit {
                break;
            }
        }

        results
    }

    /// Search DuckDuckGo Instant Answer API
    async fn search_duckduckgo_api(&self, query: &str, limit: usize) -> Result<Vec<WebSearchResult>> {
        let url = match reqwest::Url::parse_with_params(
            "https://api.duckduckgo.com/",
            &[("q", query), ("format", "json"), ("no_html", "1"), ("skip_disambig", "0")],
        ) {
            Ok(u) => u,
            Err(e) => return Err(TagisanError::Execution(format!("Invalid URL: {e}"))),
        };

        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(TagisanError::BadResponse(
                "duckduckgo".to_string(),
                format!("HTTP error {}", resp.status()),
            ));
        }

        let data: Value = resp.json().await?;
        let mut results = Vec::new();

        if let (Some(heading), Some(abstract_text), Some(abstract_url)) = (
            data.get("Heading").and_then(|v| v.as_str()),
            data.get("AbstractText").and_then(|v| v.as_str()),
            data.get("AbstractURL").and_then(|v| v.as_str()),
        ) {
            if !heading.is_empty() && !abstract_url.is_empty() {
                results.push(WebSearchResult {
                    title: heading.to_string(),
                    url: abstract_url.to_string(),
                    snippet: abstract_text.to_string(),
                });
            }
        }

        if let Some(topics) = data.get("RelatedTopics").and_then(|v| v.as_array()) {
            for t in topics {
                if let (Some(text), Some(url)) = (
                    t.get("Text").and_then(|v| v.as_str()),
                    t.get("FirstURL").and_then(|v| v.as_str()),
                ) {
                    let title = text.split(" - ").next().unwrap_or(text);
                    results.push(WebSearchResult {
                        title: title.to_string(),
                        url: url.to_string(),
                        snippet: text.to_string(),
                    });
                }
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Search via Tavily API
    async fn search_tavily(&self, query: &str, api_key: &str, limit: usize) -> Result<Vec<WebSearchResult>> {
        let body = json!({
            "api_key": api_key,
            "query": query,
            "max_results": limit,
            "search_depth": "basic"
        });

        let resp = self
            .client
            .post("https://api.tavily.com/search")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TagisanError::BadResponse("tavily".into(), format!("HTTP {}", resp.status())));
        }

        let json: Value = resp.json().await?;
        let mut results = Vec::new();

        if let Some(arr) = json.get("results").and_then(|v| v.as_array()) {
            for item in arr {
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let snippet = item.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if !title.is_empty() && !url.is_empty() {
                    results.push(WebSearchResult { title, url, snippet });
                }
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Search via Brave Search API
    async fn search_brave(&self, query: &str, api_key: &str, limit: usize) -> Result<Vec<WebSearchResult>> {
        let url = match reqwest::Url::parse_with_params(
            "https://api.search.brave.com/res/v1/web/search",
            &[("q", query), ("count", &limit.to_string())],
        ) {
            Ok(u) => u,
            Err(e) => return Err(TagisanError::Execution(format!("Invalid URL: {e}"))),
        };

        let resp = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .header("X-Subscription-Token", api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TagisanError::BadResponse("brave".into(), format!("HTTP {}", resp.status())));
        }

        let json: Value = resp.json().await?;
        let mut results = Vec::new();

        if let Some(arr) = json.pointer("/web/results").and_then(|v| v.as_array()) {
            for item in arr {
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let snippet = item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if !title.is_empty() && !url.is_empty() {
                    results.push(WebSearchResult { title, url, snippet });
                }
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Fetch and clean webpage text for readability
    pub async fn fetch_webpage(&self, url: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("Failed to fetch webpage '{url}': {e}")))?;

        if !resp.status().is_success() {
            return Err(TagisanError::BadResponse(
                url.to_string(),
                format!("HTTP error {}", resp.status()),
            ));
        }

        let html = resp
            .text()
            .await
            .map_err(|e| TagisanError::Execution(format!("Failed to read webpage content: {e}")))?;

        // Clean HTML: Remove scripts, styles, navigation, and footers
        let clean_html = Regex::new(r"(?is)<script[^>]*>.*?</script>")
            .map(|r| r.replace_all(&html, "").to_string())
            .unwrap_or_else(|_| html.clone());

        let clean_html = Regex::new(r"(?is)<style[^>]*>.*?</style>")
            .map(|r| r.replace_all(&clean_html, "").to_string())
            .unwrap_or(clean_html);

        let clean_html = Regex::new(r"(?is)<nav[^>]*>.*?</nav>")
            .map(|r| r.replace_all(&clean_html, "").to_string())
            .unwrap_or(clean_html);

        let clean_html = Regex::new(r"(?is)<footer[^>]*>.*?</footer>")
            .map(|r| r.replace_all(&clean_html, "").to_string())
            .unwrap_or(clean_html);

        // Strip HTML tags
        let text = Regex::new(r"<[^>]+>")
            .map(|r| r.replace_all(&clean_html, " ").to_string())
            .unwrap_or(clean_html);

        // Collapse whitespace
        let cleaned = Regex::new(r"\s+")
            .map(|r| r.replace_all(&text, " ").to_string())
            .unwrap_or(text);

        let mut output = Self::decode_html_entities(&cleaned);
        if output.len() > 4000 {
            output.truncate(4000);
            output.push_str("... [truncated]");
        }

        Ok(output.trim().to_string())
    }

    /// Decode common HTML entities without external crate dependencies
    fn decode_html_entities(text: &str) -> String {
        text.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
            .replace("&#x27;", "'")
            .replace("&nbsp;", " ")
    }
}

#[async_trait]
impl ToolHandler for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the live web for real-time information, documentation, news, or technical questions using DuckDuckGo (zero-config), Tavily, or Brave. Also supports directly fetching and summarizing webpage content via 'fetch_url'."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query or keywords to look up on the web."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of search results to return (1-10, default: 5)."
                },
                "fetch_url": {
                    "type": "string",
                    "description": "Optional direct URL to fetch, clean, and extract readable text from instead of searching."
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        if let Some(url) = arguments.get("fetch_url").and_then(|v| v.as_str()) {
            if !url.trim().is_empty() {
                let content = self.fetch_webpage(url).await?;
                return Ok(format!("## 📄 Content from [{url}]({url})\n\n{content}"));
            }
        }

        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'query' or 'fetch_url'".to_string()))?;

        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .clamp(1, 10) as usize;

        let results = self.search(query, limit).await?;

        if results.is_empty() {
            return Ok(format!("No web search results found for query: \"{query}\""));
        }

        let mut out = format!("## 🔍 Web Search Results for \"{query}\"\n\n");
        for (i, r) in results.iter().enumerate() {
            out.push_str(&format!(
                "{}. **[{}]({})**\n   > {}\n\n",
                i + 1,
                r.title,
                r.url,
                if r.snippet.is_empty() { "No snippet available." } else { &r.snippet }
            ));
        }

        Ok(out)
    }
}
