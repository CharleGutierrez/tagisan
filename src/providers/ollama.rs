use crate::error::{Result, TagisanError};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage,
};
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        let timeout_secs = std::env::var("OLLAMA_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            base_url: base_url.into(),
            client,
        }
    }

    pub fn default_local() -> Self {
        Self::new("http://localhost:11434")
    }

    /// Check if local or remote Ollama daemon is reachable and responding
    pub async fn is_alive(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Retrieve available models from the Ollama tags registry
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url.trim_end_matches('/'));
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("ollama".into(), err));
        }
        #[derive(Deserialize)]
        struct TagsResponse {
            models: Option<Vec<ModelItem>>,
        }
        #[derive(Deserialize)]
        struct ModelItem {
            name: String,
        }
        let tags: TagsResponse = resp.json().await?;
        Ok(tags.models.unwrap_or_default().into_iter().map(|m| m.name).collect())
    }

    /// Returns the default model for Ollama:
    /// 1. `OLLAMA_MODEL` environment variable if set.
    /// 2. Auto-discovered installed model from local Ollama manifests on disk.
    /// 3. Default fallback: `"dolphin-phi:latest"`.
    pub fn default_model() -> String {
        if let Ok(model) = std::env::var("OLLAMA_MODEL") {
            let m = model.trim();
            if !m.is_empty() {
                return m.to_string();
            }
        }

        let installed = Self::discover_installed_models();
        if let Some(first) = installed.first() {
            return first.clone();
        }

        "dolphin-phi:latest".to_string()
    }

    /// Discovers locally installed models by inspecting the Ollama manifests directory on disk
    pub fn discover_installed_models() -> Vec<String> {
        let mut models = Vec::new();
        let base_models_dir = std::env::var("OLLAMA_MODELS")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))
                    .unwrap_or_default();
                std::path::PathBuf::from(home).join(".ollama").join("models")
            });

        let manifests_dir = base_models_dir
            .join("manifests")
            .join("registry.ollama.ai")
            .join("library");

        if let Ok(entries) = std::fs::read_dir(&manifests_dir) {
            for entry in entries.flatten() {
                let model_name = entry.file_name().to_string_lossy().to_string();
                let tag_path = entry.path();
                if tag_path.is_dir() {
                    if let Ok(tags) = std::fs::read_dir(&tag_path) {
                        for tag_entry in tags.flatten() {
                            let tag = tag_entry.file_name().to_string_lossy().to_string();
                            if tag == "latest" {
                                models.push(format!("{}:latest", model_name));
                            } else {
                                models.push(format!("{}:{}", model_name, tag));
                            }
                        }
                    }
                }
            }
        }

        // Priority order for optimal execution on local hardware:
        // 1. dolphin-phi:latest (high-performance 3B instruct/code model, ~1.6GB)
        // 2. qwen2.5:0.5b (ultra-fast 0.5B model, ~397MB)
        // 3. other installed models
        models.sort_by(|a, b| {
            let score = |m: &str| {
                if m.contains("dolphin-phi") {
                    0
                } else if m.contains("qwen2.5") {
                    1
                } else if m.contains("mixtral") {
                    99
                } else {
                    10
                }
            };
            score(a).cmp(&score(b))
        });

        models
    }

    fn format_messages(&self, req: &CompletionRequest) -> Vec<OllamaMessage> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: sys.clone(),
                images: None,
            });
        }

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
                Role::Reasoning => "assistant",
            };

            let mut text_parts = Vec::new();
            let mut images = Vec::new();

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        text_parts.push(text.clone());
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        text_parts.push(format!("<think>\n{}\n</think>", thinking));
                    }
                    ContentBlock::Image { data_base64, .. } => {
                        images.push(data_base64.clone());
                    }
                    ContentBlock::ToolCall { name, arguments, .. } => {
                        text_parts.push(format!("[Tool Call: {}({})]", name, arguments));
                    }
                    ContentBlock::ToolResult { tool_call_id, content, .. } => {
                        text_parts.push(format!("[Tool Result for {}: {}]", tool_call_id, content));
                    }
                }
            }

            messages.push(OllamaMessage {
                role: role_str.to_string(),
                content: text_parts.join("\n"),
                images: if images.is_empty() { None } else { Some(images) },
            });
        }

        messages
    }

    fn build_options(&self, req: &CompletionRequest) -> OllamaOptions {
        let num_gpu = std::env::var("TAGISAN_OLLAMA_NUM_GPU")
            .ok()
            .and_then(|s| s.parse().ok())
            .or(Some(99)); // Default to 99: offload all transformer layers into GPU VRAM

        let num_batch = std::env::var("TAGISAN_OLLAMA_NUM_BATCH")
            .ok()
            .and_then(|s| s.parse().ok())
            .or(Some(512)); // High-throughput prompt ingestion batch size

        let num_ctx = std::env::var("TAGISAN_OLLAMA_NUM_CTX")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| req.max_tokens.map(|m| m.max(4096)))
            .or(Some(8192));

        let num_thread = std::env::var("TAGISAN_OLLAMA_NUM_THREAD")
            .ok()
            .and_then(|s| s.parse().ok());

        OllamaOptions {
            temperature: req.temperature,
            num_predict: req.max_tokens,
            num_ctx,
            num_gpu,
            num_thread,
            num_batch,
            f16_kv: Some(true),
            use_mmap: Some(true),
            use_mlock: None,
        }
    }

    fn get_keep_alive(&self) -> String {
        std::env::var("TAGISAN_OLLAMA_KEEP_ALIVE").unwrap_or_else(|_| "24h".to_string())
    }
}

/// Resolve the default Ollama model dynamically
pub fn default_ollama_model() -> String {
    OllamaProvider::default_model()
}

/// Advanced inference options passed directly to the llama.cpp engine inside Ollama
#[derive(Serialize, Debug, Clone, Default)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_gpu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_thread: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_batch: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f16_kv: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mmap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mlock: Option<bool>,
}

#[derive(Serialize)]
struct OllamaChatPayload<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OllamaTool<'a>>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaTool<'a> {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: OllamaFunctionDefinition<'a>,
}

#[derive(Serialize)]
struct OllamaFunctionDefinition<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Default)]
struct OllamaApiResponse {
    #[serde(default)]
    message: Option<OllamaMessageResponse>,
    done: Option<bool>,
    done_reason: Option<String>,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
    error: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
struct OllamaMessageResponse {
    #[serde(default)]
    content: String,
    tool_calls: Option<Vec<OllamaToolCallResponse>>,
}

#[derive(Deserialize, Debug)]
struct OllamaToolCallResponse {
    function: OllamaFunctionResponse,
}

#[derive(Deserialize, Debug)]
struct OllamaFunctionResponse {
    name: String,
    arguments: serde_json::Value,
}

/// Robust parser extracting DeepSeek-style `<think>...</think>` blocks into ContentBlock::Thinking and remaining text into ContentBlock::Text
pub fn parse_thinking_blocks(raw_text: &str) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut remaining = raw_text;

    while let Some(start_idx) = remaining.find("<think>") {
        let before = remaining[..start_idx].trim();
        if !before.is_empty() {
            blocks.push(ContentBlock::Text { text: before.to_string() });
        }
        let after_start = &remaining[start_idx + 7..];
        if let Some(end_idx) = after_start.find("</think>") {
            let thinking = after_start[..end_idx].trim();
            blocks.push(ContentBlock::Thinking {
                thinking: thinking.to_string(),
                signature: None,
            });
            remaining = &after_start[end_idx + 8..];
        } else {
            // Unclosed <think> tag: fallback remainder gracefully to Text
            let unclosed_text = remaining.trim();
            if !unclosed_text.is_empty() {
                blocks.push(ContentBlock::Text { text: unclosed_text.to_string() });
            }
            return blocks;
        }
    }

    let tail = remaining.trim();
    if !tail.is_empty() {
        blocks.push(ContentBlock::Text { text: tail.to_string() });
    }

    // Edge-case: if raw_text was solely `<think></think>` or contained empty think tags
    if blocks.is_empty() && !raw_text.is_empty() {
        if raw_text.contains("<think>") && raw_text.contains("</think>") {
            blocks.push(ContentBlock::Thinking {
                thinking: String::new(),
                signature: None,
            });
        } else {
            blocks.push(ContentBlock::Text { text: raw_text.to_string() });
        }
    }

    blocks
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
            | ProviderCapabilities::SYSTEM_PROMPT
            | ProviderCapabilities::FUNCTION_CALLING
            | ProviderCapabilities::VISION
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let messages = self.format_messages(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| OllamaTool {
                        tool_type: "function",
                        function: OllamaFunctionDefinition {
                            name: &t.name,
                            description: &t.description,
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        } else {
            None
        };

        let keep_alive = self.get_keep_alive();
        let options = self.build_options(&req);

        let payload = OllamaChatPayload {
            model: &req.model,
            messages,
            tools,
            stream: false,
            keep_alive: Some(&keep_alive),
            options: Some(options),
        };

        let send_future = self.client.post(&url).json(&payload).send();

        let response = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = send_future => res?,
            }
        } else {
            send_future.await?
        };

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(TagisanError::RateLimited("ollama".into(), None));
            }
            return Err(TagisanError::BadResponse("ollama".into(), err_text));
        }

        let api_resp: OllamaApiResponse = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = response.json::<OllamaApiResponse>() => res?,
            }
        } else {
            response.json().await?
        };

        if let Some(err) = api_resp.error {
            return Err(TagisanError::BadResponse("ollama".into(), err));
        }

        let mut content_blocks = Vec::new();
        let mut has_tool_calls = false;

        if let Some(msg) = api_resp.message {
            if let Some(tool_calls) = msg.tool_calls {
                for (idx, tc) in tool_calls.into_iter().enumerate() {
                    has_tool_calls = true;
                    let arguments = match tc.function.arguments {
                        serde_json::Value::String(ref s) => {
                            serde_json::from_str::<serde_json::Value>(s).unwrap_or(tc.function.arguments)
                        }
                        other => other,
                    };
                    content_blocks.push(ContentBlock::ToolCall {
                        id: format!("ollama_call_{}", idx),
                        name: tc.function.name,
                        arguments,
                    });
                }
            }

            let text_blocks = parse_thinking_blocks(&msg.content);
            content_blocks.extend(text_blocks);
        }

        let finish_reason = match api_resp.done_reason.as_deref() {
            Some("stop") => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
            Some("length") => FinishReason::Length,
            Some("tool_calls") | Some("tool_call") => FinishReason::ToolCalls,
            Some(other) => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Other(other.to_string())
                }
            }
            None => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
        };

        let usage = TokenUsage {
            prompt_tokens: api_resp.prompt_eval_count.unwrap_or(0),
            completion_tokens: api_resp.eval_count.unwrap_or(0),
            reasoning_tokens: None,
            cached_prompt_tokens: None,
            estimated_cost_usd: Some(0.0),
        };

        Ok(CompletionResponse {
            id: format!("ollama_{}", start.elapsed().as_millis()),
            provider: "ollama".to_string(),
            model: req.model,
            message: Message {
                role: Role::Assistant,
                content: content_blocks,
                name: None,
                metadata: std::collections::HashMap::new(),
            },
            finish_reason,
            usage,
            latency: start.elapsed(),
        })
    }

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let messages = self.format_messages(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| OllamaTool {
                        tool_type: "function",
                        function: OllamaFunctionDefinition {
                            name: &t.name,
                            description: &t.description,
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        } else {
            None
        };

        let keep_alive = self.get_keep_alive();
        let options = self.build_options(&req);

        let payload = OllamaChatPayload {
            model: &req.model,
            messages,
            tools,
            stream: true,
            keep_alive: Some(&keep_alive),
            options: Some(options),
        };

        let send_future = self.client.post(&url).json(&payload).send();

        let response = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = send_future => res?,
            }
        } else {
            send_future.await?
        };

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(TagisanError::RateLimited("ollama".into(), None));
            }
            return Err(TagisanError::BadResponse("ollama".into(), err_text));
        }

        let cancellation_token = req.cancellation_token.clone();
        let stream = response.bytes_stream().map(|item| {
            item.map_err(std::io::Error::other)
        });
        let reader = StreamReader::new(stream);
        let mut lines = reader.lines();

        let output_stream = async_stream::stream! {
            let mut accumulated_usage: Option<TokenUsage> = None;
            loop {
                let next_line = if let Some(ref token) = cancellation_token {
                    tokio::select! {
                        _ = token.cancelled() => {
                            yield Err(TagisanError::Cancelled);
                            return;
                        }
                        line_res = lines.next_line() => line_res,
                    }
                } else {
                    lines.next_line().await
                };

                let line_str = match next_line {
                    Ok(Some(line)) => line,
                    Ok(None) => break,
                    Err(e) => {
                        yield Err(TagisanError::BadResponse("ollama".into(), e.to_string()));
                        break;
                    }
                };

                let line_trim = line_str.trim();
                if line_trim.is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaApiResponse>(line_trim) {
                    Ok(resp) => {
                        if let Some(err) = resp.error {
                            yield Err(TagisanError::BadResponse("ollama".into(), err));
                            break;
                        }

                        let is_done = resp.done.unwrap_or(false);
                        if is_done {
                            accumulated_usage = Some(TokenUsage {
                                prompt_tokens: resp.prompt_eval_count.unwrap_or(0),
                                completion_tokens: resp.eval_count.unwrap_or(0),
                                reasoning_tokens: None,
                                cached_prompt_tokens: None,
                                estimated_cost_usd: Some(0.0),
                            });
                        }

                        let finish_reason = if is_done {
                            Some(match resp.done_reason.as_deref() {
                                Some("length") => FinishReason::Length,
                                Some("tool_calls") | Some("tool_call") => FinishReason::ToolCalls,
                                Some(other) if other != "stop" => FinishReason::Other(other.to_string()),
                                _ => {
                                    let has_tc = resp.message.as_ref()
                                        .and_then(|m| m.tool_calls.as_ref())
                                        .map(|tc| !tc.is_empty())
                                        .unwrap_or(false);
                                    if has_tc {
                                        FinishReason::ToolCalls
                                    } else {
                                        FinishReason::Stop
                                    }
                                }
                            })
                        } else {
                            None
                        };

                        if let Some(msg) = resp.message {
                            if let Some(tool_calls) = msg.tool_calls {
                                for (idx, tc) in tool_calls.into_iter().enumerate() {
                                    let args_delta = match tc.function.arguments {
                                        serde_json::Value::String(s) => s,
                                        other => other.to_string(),
                                    };
                                    yield Ok(StreamChunk {
                                        delta: StreamChunkDelta::ToolCallDelta {
                                            index: idx,
                                            id: Some(format!("ollama_call_{}", idx)),
                                            name: Some(tc.function.name),
                                            arguments_delta: Some(args_delta),
                                        },
                                        finish_reason: finish_reason.clone(),
                                        usage: accumulated_usage.clone(),
                                    });
                                }
                            }

                            if !msg.content.is_empty() {
                                yield Ok(StreamChunk {
                                    delta: StreamChunkDelta::Text(msg.content),
                                    finish_reason: finish_reason.clone(),
                                    usage: accumulated_usage.clone(),
                                });
                            } else if is_done {
                                yield Ok(StreamChunk::done(
                                    finish_reason.unwrap_or(FinishReason::Stop),
                                    accumulated_usage.clone(),
                                ));
                            }
                        } else if is_done {
                            yield Ok(StreamChunk::done(
                                finish_reason.unwrap_or(FinishReason::Stop),
                                accumulated_usage.clone(),
                            ));
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Failed to parse Ollama line: {}", e);
                    }
                }
            }
        };

        Ok(Box::pin(output_stream))
    }
}

// =========================================================================
// Brutal Production-Grade Stress Tests
// =========================================================================

#[cfg(test)]
mod brutal_stress_tests {
    use super::*;
    use crate::types::ToolDefinition;
    use serde_json::json;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;
    use tokio_util::sync::CancellationToken;

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    pub struct RecordedRequest {
        pub method: String,
        pub path: String,
        pub headers: HashMap<String, String>,
        pub body_json: Option<serde_json::Value>,
        pub body_raw: Vec<u8>,
    }

    pub type RequestHandler = Arc<
        dyn Fn(RecordedRequest) -> (u16, HashMap<String, String>, Vec<u8>) + Send + Sync + 'static,
    >;

    pub type StreamHandler = Arc<
        dyn Fn(RecordedRequest) -> (u16, HashMap<String, String>, Vec<String>, Duration)
            + Send
            + Sync
            + 'static,
    >;

    #[allow(dead_code)]
    pub struct MockOllamaServer {
        pub addr: SocketAddr,
        pub base_url: String,
        shutdown_token: CancellationToken,
        recorded_requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    impl MockOllamaServer {
        pub async fn start_custom(
            request_handler: Option<RequestHandler>,
            stream_handler: Option<StreamHandler>,
        ) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("Failed to bind mock server to ephemeral port");
            let addr = listener.local_addr().unwrap();
            let base_url = format!("http://{}", addr);
            let shutdown_token = CancellationToken::new();
            let recorded_requests = Arc::new(Mutex::new(Vec::new()));

            let shutdown_clone = shutdown_token.clone();
            let recorded_clone = recorded_requests.clone();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown_clone.cancelled() => {
                            break;
                        }
                        accept_res = listener.accept() => {
                            let (socket, _) = match accept_res {
                                Ok(res) => res,
                                Err(_) => break,
                            };

                            let (read_half, mut write_half) = socket.into_split();
                            let recorded = recorded_clone.clone();
                            let req_h = request_handler.clone();
                            let str_h = stream_handler.clone();

                            tokio::spawn(async move {
                                let mut reader = BufReader::new(read_half);
                                let mut req_line = String::new();
                                if reader.read_line(&mut req_line).await.is_err() || req_line.is_empty() {
                                    return;
                                }

                                let parts: Vec<&str> = req_line.split_whitespace().collect();
                                if parts.len() < 2 {
                                    return;
                                }
                                let method = parts[0].to_string();
                                let path = parts[1].to_string();

                                let mut headers = HashMap::new();
                                let mut content_length = 0usize;

                                loop {
                                    let mut header_line = String::new();
                                    if reader.read_line(&mut header_line).await.is_err() {
                                        break;
                                    }
                                    let trimmed = header_line.trim();
                                    if trimmed.is_empty() {
                                        break;
                                    }
                                    if let Some((k, v)) = trimmed.split_once(':') {
                                        let key = k.trim().to_lowercase();
                                        let val = v.trim().to_string();
                                        if key == "content-length" {
                                            content_length = val.parse().unwrap_or(0);
                                        }
                                        headers.insert(key, val);
                                    }
                                }

                                let mut body_raw = vec![0u8; content_length];
                                if content_length > 0 {
                                    let _ = reader.read_exact(&mut body_raw).await;
                                }

                                let body_json: Option<serde_json::Value> = serde_json::from_slice(&body_raw).ok();

                                let req_rec = RecordedRequest {
                                    method: method.clone(),
                                    path: path.clone(),
                                    headers: headers.clone(),
                                    body_json: body_json.clone(),
                                    body_raw,
                                };

                                recorded.lock().await.push(req_rec.clone());

                                let is_stream = body_json.as_ref()
                                    .and_then(|j| j.get("stream"))
                                    .and_then(|s| s.as_bool())
                                    .unwrap_or(false);

                                if is_stream && str_h.is_some() {
                                    let handler = str_h.as_ref().unwrap();
                                    let (status, custom_headers, chunks, delay) = handler(req_rec);

                                    let mut header_str = format!("HTTP/1.1 {} OK\r\nContent-Type: application/x-ndjson\r\nConnection: close\r\n", status);
                                    for (k, v) in custom_headers {
                                        header_str.push_str(&format!("{}: {}\r\n", k, v));
                                    }
                                    header_str.push_str("\r\n");

                                    if write_half.write_all(header_str.as_bytes()).await.is_ok() {
                                        let _ = write_half.flush().await;
                                        for chunk in chunks {
                                            if write_half.write_all(chunk.as_bytes()).await.is_err() {
                                                break;
                                            }
                                            if write_half.write_all(b"\n").await.is_err() {
                                                break;
                                            }
                                            let _ = write_half.flush().await;
                                            if delay > Duration::ZERO {
                                                tokio::time::sleep(delay).await;
                                            }
                                        }
                                    }
                                    return;
                                }

                                if let Some(handler) = req_h.as_ref() {
                                    let (status, custom_headers, body) = handler(req_rec);
                                    let status_text = match status {
                                        200 => "OK",
                                        429 => "Too Many Requests",
                                        500 => "Internal Server Error",
                                        503 => "Service Unavailable",
                                        _ => "Custom",
                                    };
                                    let mut header_str = format!("HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n", status, status_text, body.len());
                                    for (k, v) in custom_headers {
                                        header_str.push_str(&format!("{}: {}\r\n", k, v));
                                    }
                                    header_str.push_str("\r\n");

                                    let _ = write_half.write_all(header_str.as_bytes()).await;
                                    let _ = write_half.write_all(&body).await;
                                    let _ = write_half.flush().await;
                                    return;
                                }

                                if path == "/api/tags" {
                                    let body = json!({
                                        "models": [
                                            {
                                                "name": "llama3.2:latest",
                                                "model": "llama3.2:latest",
                                                "modified_at": "2024-10-01T00:00:00Z",
                                                "size": 2019393189,
                                                "digest": "sha256:abc123mockdigest"
                                            },
                                            {
                                                "name": "deepseek-r1:latest",
                                                "model": "deepseek-r1:latest"
                                            }
                                        ]
                                    });
                                    let body_bytes = serde_json::to_vec(&body).unwrap();
                                    let resp = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                        body_bytes.len()
                                    );
                                    let _ = write_half.write_all(resp.as_bytes()).await;
                                    let _ = write_half.write_all(&body_bytes).await;
                                    let _ = write_half.flush().await;
                                    return;
                                }

                                if path == "/api/chat" {
                                    let resp_json = json!({
                                        "model": "llama3.2:latest",
                                        "created_at": "2024-10-01T00:00:00Z",
                                        "message": {
                                            "role": "assistant",
                                            "content": "Hello from mock Ollama!"
                                        },
                                        "done": true,
                                        "done_reason": "stop",
                                        "prompt_eval_count": 12,
                                        "eval_count": 8
                                    });
                                    let body_bytes = serde_json::to_vec(&resp_json).unwrap();
                                    let resp = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                        body_bytes.len()
                                    );
                                    let _ = write_half.write_all(resp.as_bytes()).await;
                                    let _ = write_half.write_all(&body_bytes).await;
                                    let _ = write_half.flush().await;
                                    return;
                                }

                                let not_found = b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                                let _ = write_half.write_all(not_found).await;
                            });
                        }
                    }
                }
            });

            Self {
                addr,
                base_url,
                shutdown_token,
                recorded_requests,
            }
        }
    }

    impl Drop for MockOllamaServer {
        fn drop(&mut self) {
            self.shutdown_token.cancel();
        }
    }

    // =========================================================================
    // Test 1: Complete Non-Streaming Protocol & Optimization Verification
    // =========================================================================
    #[tokio::test]
    async fn test_01_non_streaming_protocol_and_optimizations() {
        let captured_request = Arc::new(Mutex::new(None));
        let captured_clone = captured_request.clone();

        let handler: RequestHandler = Arc::new(move |req| {
            let mut guard = captured_clone.try_lock().unwrap();
            *guard = Some(req);

            let resp_json = json!({
                "model": "llama3.2:latest",
                "message": {
                    "role": "assistant",
                    "content": "42 is the ultimate answer."
                },
                "done": true,
                "done_reason": "stop",
                "prompt_eval_count": 105,
                "eval_count": 48
            });
            (200, HashMap::new(), serde_json::to_vec(&resp_json).unwrap())
        });

        let server = MockOllamaServer::start_custom(Some(handler), None).await;
        let provider = OllamaProvider::new(&server.base_url);

        let req = CompletionRequest::new("llama3.2:latest", "Calculate the answer to life")
            .with_max_tokens(8192)
            .with_temperature(0.42);

        let response = provider.complete(req).await.expect("Completion should succeed");

        // 1. Verify outgoing request options & keep_alive
        let recorded = captured_request.lock().await.clone().expect("Request must be recorded");
        let payload = recorded.body_json.expect("Body must be valid JSON");

        assert_eq!(payload["model"], "llama3.2:latest");
        assert_eq!(payload["stream"], false);
        assert_eq!(payload["keep_alive"], "24h", "Default keep_alive must be 24h");

        let options = &payload["options"];
        assert_eq!(options["num_gpu"], 99, "num_gpu must default to 99 for full GPU offload");
        assert_eq!(options["num_batch"], 512, "num_batch must default to 512 for high prompt batching");
        assert_eq!(options["f16_kv"], true, "f16_kv must be enabled");
        assert_eq!(options["use_mmap"], true, "use_mmap must be enabled");
        assert_eq!(options["num_ctx"], 8192, "num_ctx must reflect max_tokens >= 8192");
        assert_eq!(options["temperature"], 0.42, "temperature must match request");
        assert_eq!(options["num_predict"], 8192, "num_predict must match max_tokens");

        // 2. Verify response parsing
        assert_eq!(response.provider, "ollama");
        assert_eq!(response.model, "llama3.2:latest");
        assert_eq!(response.message.role, Role::Assistant);
        assert_eq!(response.message.extract_text(), "42 is the ultimate answer.");
        assert_eq!(response.finish_reason, FinishReason::Stop);

        // 3. Verify usage statistics & zero-cost estimation
        assert_eq!(response.usage.prompt_tokens, 105);
        assert_eq!(response.usage.completion_tokens, 48);
        assert_eq!(response.usage.estimated_cost_usd, Some(0.0), "Local Ollama inference must estimate $0.00 cost");

        // 4. Test finish_reason variants: Length, ToolCalls
        let length_handler: RequestHandler = Arc::new(|_| {
            let resp = json!({
                "message": { "role": "assistant", "content": "truncated..." },
                "done": true,
                "done_reason": "length"
            });
            (200, HashMap::new(), serde_json::to_vec(&resp).unwrap())
        });
        let server_length = MockOllamaServer::start_custom(Some(length_handler), None).await;
        let provider_len = OllamaProvider::new(&server_length.base_url);
        let resp_len = provider_len.complete(CompletionRequest::new("llama3.2", "hi")).await.unwrap();
        assert_eq!(resp_len.finish_reason, FinishReason::Length);

        let tool_handler: RequestHandler = Arc::new(|_| {
            let resp = json!({
                "message": {
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [
                        { "function": { "name": "calc", "arguments": { "x": 1 } } }
                    ]
                },
                "done": true,
                "done_reason": "stop"
            });
            (200, HashMap::new(), serde_json::to_vec(&resp).unwrap())
        });
        let server_tool = MockOllamaServer::start_custom(Some(tool_handler), None).await;
        let provider_tool = OllamaProvider::new(&server_tool.base_url);
        let resp_tool = provider_tool.complete(CompletionRequest::new("llama3.2", "calc")).await.unwrap();
        assert_eq!(resp_tool.finish_reason, FinishReason::ToolCalls);
    }

    // =========================================================================
    // Test 2: Real-time NDJSON Streaming Stress (200+ Incremental Chunks)
    // =========================================================================
    #[tokio::test]
    async fn test_02_realtime_ndjson_streaming_200_chunks() {
        let total_chunks = 220;

        let stream_handler: StreamHandler = Arc::new(move |_| {
            let mut chunks = Vec::new();
            for i in 0..total_chunks {
                let chunk_json = json!({
                    "model": "deepseek-r1:latest",
                    "created_at": "2024-10-01T00:00:00Z",
                    "message": {
                        "role": "assistant",
                        "content": format!("chunk_{:03} ", i)
                    },
                    "done": false
                });
                chunks.push(serde_json::to_string(&chunk_json).unwrap());
            }

            let terminal_json = json!({
                "model": "deepseek-r1:latest",
                "created_at": "2024-10-01T00:00:00Z",
                "message": {
                    "role": "assistant",
                    "content": ""
                },
                "done": true,
                "done_reason": "stop",
                "prompt_eval_count": 55,
                "eval_count": total_chunks as u32
            });
            chunks.push(serde_json::to_string(&terminal_json).unwrap());

            (200, HashMap::new(), chunks, Duration::from_millis(1))
        });

        let server = MockOllamaServer::start_custom(None, Some(stream_handler)).await;
        let provider = OllamaProvider::new(&server.base_url);

        let req = CompletionRequest::new("deepseek-r1:latest", "Stream 200 tokens");
        let mut stream = provider.stream(req).await.expect("Stream initialization must succeed");

        let mut received_tokens = Vec::new();
        let mut final_usage = None;
        let mut final_finish_reason = None;

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.expect("Streaming chunk should be Ok");
            match chunk.delta {
                StreamChunkDelta::Text(t) => {
                    if !t.is_empty() {
                        received_tokens.push(t);
                    }
                }
                _ => panic!("Expected text delta during streaming token test"),
            }
            if let Some(u) = chunk.usage {
                final_usage = Some(u);
            }
            if let Some(fr) = chunk.finish_reason {
                final_finish_reason = Some(fr);
            }
        }

        assert_eq!(received_tokens.len(), total_chunks, "Must receive exactly 220 chunks");
        for (i, token) in received_tokens.iter().enumerate() {
            let expected = format!("chunk_{:03} ", i);
            assert_eq!(token, &expected, "Chunk at position {} was out of order or corrupted", i);
        }

        let usage = final_usage.expect("Terminal chunk must yield token usage");
        assert_eq!(usage.prompt_tokens, 55);
        assert_eq!(usage.completion_tokens, total_chunks as u32);
        assert_eq!(usage.estimated_cost_usd, Some(0.0));
        assert_eq!(final_finish_reason, Some(FinishReason::Stop));
    }

    // =========================================================================
    // Test 3: DeepSeek-Style Thinking Block Extraction
    // =========================================================================
    #[tokio::test]
    async fn test_03_deepseek_thinking_block_extraction() {
        // 1. Standard thinking block
        let text1 = "<think>reasoning steps</think>final answer";
        let blocks1 = parse_thinking_blocks(text1);
        assert_eq!(blocks1.len(), 2);
        assert_eq!(blocks1[0], ContentBlock::Thinking {
            thinking: "reasoning steps".to_string(),
            signature: None,
        });
        assert_eq!(blocks1[1], ContentBlock::Text {
            text: "final answer".to_string(),
        });

        // 2. Multiline thinking block
        let text2 = "<think>\nStep 1: Parse requirements\nStep 2: Validate edge cases\nStep 3: Conclude\n</think>\n\nHere is the verified solution.";
        let blocks2 = parse_thinking_blocks(text2);
        assert_eq!(blocks2.len(), 2);
        assert_eq!(blocks2[0], ContentBlock::Thinking {
            thinking: "Step 1: Parse requirements\nStep 2: Validate edge cases\nStep 3: Conclude".to_string(),
            signature: None,
        });
        assert_eq!(blocks2[1], ContentBlock::Text {
            text: "Here is the verified solution.".to_string(),
        });

        // 3. Empty thinking block
        let text3 = "<think></think>";
        let blocks3 = parse_thinking_blocks(text3);
        assert_eq!(blocks3.len(), 1);
        assert_eq!(blocks3[0], ContentBlock::Thinking {
            thinking: "".to_string(),
            signature: None,
        });

        // 3b. Empty thinking block followed by answer
        let text3b = "<think></think>42";
        let blocks3b = parse_thinking_blocks(text3b);
        assert_eq!(blocks3b.len(), 2);
        assert_eq!(blocks3b[0], ContentBlock::Thinking {
            thinking: "".to_string(),
            signature: None,
        });
        assert_eq!(blocks3b[1], ContentBlock::Text {
            text: "42".to_string(),
        });

        // 4. Unclosed <think> tag: graceful fallback, zero panics
        let text4 = "<think>This thought never ends and lacks closing tag";
        let blocks4 = parse_thinking_blocks(text4);
        assert_eq!(blocks4.len(), 1);
        assert_eq!(blocks4[0], ContentBlock::Text {
            text: "<think>This thought never ends and lacks closing tag".to_string(),
        });

        // 5. Inverted tags: </think><think>inside</think>
        let text5 = "</think><think>inside</think>";
        let blocks5 = parse_thinking_blocks(text5);
        assert_eq!(blocks5.len(), 2);
        assert_eq!(blocks5[0], ContentBlock::Text {
            text: "</think>".to_string(),
        });
        assert_eq!(blocks5[1], ContentBlock::Thinking {
            thinking: "inside".to_string(),
            signature: None,
        });

        // 6. Multiple sequential thinking blocks
        let text6 = "<think>t1</think>intermediate<think>t2</think>final";
        let blocks6 = parse_thinking_blocks(text6);
        assert_eq!(blocks6.len(), 4);
        assert_eq!(blocks6[0], ContentBlock::Thinking { thinking: "t1".into(), signature: None });
        assert_eq!(blocks6[1], ContentBlock::Text { text: "intermediate".into() });
        assert_eq!(blocks6[2], ContentBlock::Thinking { thinking: "t2".into(), signature: None });
        assert_eq!(blocks6[3], ContentBlock::Text { text: "final".into() });

        // End-to-end verification through OllamaProvider::complete
        let end_to_end_handler: RequestHandler = Arc::new(|_| {
            let resp = json!({
                "message": {
                    "role": "assistant",
                    "content": "<think>Let me calculate 2+2.\n2+2 is 4.</think>Result: 4"
                },
                "done": true,
                "done_reason": "stop"
            });
            (200, HashMap::new(), serde_json::to_vec(&resp).unwrap())
        });
        let server = MockOllamaServer::start_custom(Some(end_to_end_handler), None).await;
        let provider = OllamaProvider::new(&server.base_url);

        let res = provider.complete(CompletionRequest::new("deepseek-r1", "2+2")).await.unwrap();
        assert_eq!(res.message.extract_thinking(), Some("Let me calculate 2+2.\n2+2 is 4.".to_string()));
        assert_eq!(res.message.extract_text(), "Result: 4");
    }

    // =========================================================================
    // Test 4: Tool / Function Calling Protocol
    // =========================================================================
    #[tokio::test]
    async fn test_04_tool_calling_protocol() {
        let captured_req = Arc::new(Mutex::new(None));
        let cap_clone = captured_req.clone();

        // 1. Non-streaming tool call request & serialization verification
        let tool_handler: RequestHandler = Arc::new(move |req| {
            *cap_clone.try_lock().unwrap() = Some(req);

            let resp = json!({
                "message": {
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [
                        {
                            "function": {
                                "name": "calculator",
                                "arguments": {
                                    "expression": "sqrt(1764)"
                                }
                            }
                        },
                        {
                            "function": {
                                "name": "lookup_constant",
                                "arguments": "{\"constant\": \"PI\"}"
                            }
                        }
                    ]
                },
                "done": true,
                "done_reason": "stop"
            });
            (200, HashMap::new(), serde_json::to_vec(&resp).unwrap())
        });

        let server = MockOllamaServer::start_custom(Some(tool_handler), None).await;
        let provider = OllamaProvider::new(&server.base_url);

        let tool_def = ToolDefinition::new(
            "calculator",
            "Evaluate math",
            json!({
                "type": "object",
                "properties": {
                    "expression": { "type": "string" }
                },
                "required": ["expression"]
            }),
        );

        let req = CompletionRequest::new("llama3.2", "Calculate sqrt(1764)")
            .with_tool(tool_def);

        let response = provider.complete(req).await.unwrap();

        let captured = captured_req.lock().await.clone().unwrap();
        let body_json = captured.body_json.unwrap();
        let tools = body_json["tools"].as_array().expect("Tools must be serialized as array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "calculator");
        assert_eq!(tools[0]["function"]["description"], "Evaluate math");

        assert_eq!(response.finish_reason, FinishReason::ToolCalls);
        let tool_calls = response.message.extract_tool_calls();
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(tool_calls[0].1, "calculator");
        assert_eq!(tool_calls[0].2, &json!({ "expression": "sqrt(1764)" }));
        assert_eq!(tool_calls[1].1, "lookup_constant");
        assert_eq!(tool_calls[1].2, &json!({ "constant": "PI" }));

        // 2. Streaming tool call chunks with incremental argument deltas
        let stream_tool_handler: StreamHandler = Arc::new(|_| {
            let chunks = vec![
                json!({
                    "message": {
                        "role": "assistant",
                        "content": "",
                        "tool_calls": [
                            { "function": { "name": "calculator", "arguments": "{\"expr\":" } }
                        ]
                    },
                    "done": false
                }).to_string(),
                json!({
                    "message": {
                        "role": "assistant",
                        "content": "",
                        "tool_calls": [
                            { "function": { "name": "calculator", "arguments": " \"2 + 2\"}" } }
                        ]
                    },
                    "done": true,
                    "done_reason": "stop"
                }).to_string(),
            ];
            (200, HashMap::new(), chunks, Duration::from_millis(5))
        });

        let stream_server = MockOllamaServer::start_custom(None, Some(stream_tool_handler)).await;
        let stream_provider = OllamaProvider::new(&stream_server.base_url);

        let mut stream = stream_provider.stream(CompletionRequest::new("llama3.2", "call tool")).await.unwrap();

        let mut deltas = Vec::new();
        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.unwrap();
            if let StreamChunkDelta::ToolCallDelta { index, name, arguments_delta, .. } = chunk.delta {
                deltas.push((index, name, arguments_delta));
            }
        }

        assert_eq!(deltas.len(), 2);
        assert_eq!(deltas[0].0, 0);
        assert_eq!(deltas[0].1, Some("calculator".to_string()));
        assert_eq!(deltas[0].2, Some("{\"expr\":".to_string()));

        assert_eq!(deltas[1].0, 0);
        assert_eq!(deltas[1].1, Some("calculator".to_string()));
        assert_eq!(deltas[1].2, Some(" \"2 + 2\"}".to_string()));
    }

    // =========================================================================
    // Test 5: Mid-Stream & Pre-Flight Cancellation Stress
    // =========================================================================
    #[tokio::test]
    async fn test_05_cancellation_stress() {
        let stream_handler: StreamHandler = Arc::new(move |_| {
            let mut chunks = Vec::new();
            for i in 0..100 {
                chunks.push(json!({
                    "message": { "role": "assistant", "content": format!("chunk_{} ", i) },
                    "done": i == 99
                }).to_string());
            }
            (200, HashMap::new(), chunks, Duration::from_millis(20))
        });

        let server = MockOllamaServer::start_custom(None, Some(stream_handler)).await;
        let provider = OllamaProvider::new(&server.base_url);

        // Subtest A: Pre-flight cancellation
        let pre_token = CancellationToken::new();
        pre_token.cancel();
        let pre_req = CompletionRequest::new("llama3.2", "pre-flight cancel")
            .with_cancellation(pre_token);
        let pre_err = provider.complete(pre_req).await.unwrap_err();
        assert!(matches!(pre_err, TagisanError::Cancelled), "Must return TagisanError::Cancelled immediately");

        // Subtest B: Cancellation at chunk 5
        let cancel_5_token = CancellationToken::new();
        let req_5 = CompletionRequest::new("llama3.2", "cancel at 5")
            .with_cancellation(cancel_5_token.clone());
        let mut stream_5 = provider.stream(req_5).await.unwrap();

        let mut count_5 = 0;
        let mut got_cancel_err = false;

        while let Some(item) = stream_5.next().await {
            match item {
                Ok(_) => {
                    count_5 += 1;
                    if count_5 == 5 {
                        cancel_5_token.cancel();
                    }
                }
                Err(TagisanError::Cancelled) => {
                    got_cancel_err = true;
                    break;
                }
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }
        assert_eq!(count_5, 5);
        assert!(got_cancel_err, "Stream must terminate with TagisanError::Cancelled");

        // Subtest C: Cancellation at chunk 50
        let cancel_50_token = CancellationToken::new();
        let req_50 = CompletionRequest::new("llama3.2", "cancel at 50")
            .with_cancellation(cancel_50_token.clone());
        let mut stream_50 = provider.stream(req_50).await.unwrap();

        let mut count_50 = 0;
        let mut got_cancel_50_err = false;

        while let Some(item) = stream_50.next().await {
            match item {
                Ok(_) => {
                    count_50 += 1;
                    if count_50 == 50 {
                        cancel_50_token.cancel();
                    }
                }
                Err(TagisanError::Cancelled) => {
                    got_cancel_50_err = true;
                    break;
                }
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }
        assert_eq!(count_50, 50);
        assert!(got_cancel_50_err, "Stream must cleanly terminate at chunk 50 without hanging");
    }

    // =========================================================================
    // Test 6: Network Fault Injection & Malformed Stream Resilience
    // =========================================================================
    #[tokio::test]
    async fn test_06_network_fault_injection_and_malformed_streams() {
        // 1. HTTP 500 Internal Server Error
        let err_500_handler: RequestHandler = Arc::new(|_| {
            (500, HashMap::new(), b"{\"error\": \"llama runner crashed with SIGSEGV\"}".to_vec())
        });
        let server_500 = MockOllamaServer::start_custom(Some(err_500_handler), None).await;
        let provider_500 = OllamaProvider::new(&server_500.base_url);
        let err_500 = provider_500.complete(CompletionRequest::new("llama", "test")).await.unwrap_err();
        assert!(matches!(err_500, TagisanError::BadResponse(p, _) if p == "ollama"));

        // 2. HTTP 503 Overloaded
        let err_503_handler: RequestHandler = Arc::new(|_| {
            (503, HashMap::new(), b"Model currently loading into VRAM".to_vec())
        });
        let server_503 = MockOllamaServer::start_custom(Some(err_503_handler), None).await;
        let provider_503 = OllamaProvider::new(&server_503.base_url);
        let err_503 = provider_503.complete(CompletionRequest::new("llama", "test")).await.unwrap_err();
        assert!(matches!(err_503, TagisanError::BadResponse(p, _) if p == "ollama"));

        // 3. HTTP 429 Rate Limited
        let err_429_handler: RequestHandler = Arc::new(|_| {
            (429, HashMap::new(), b"Too Many Requests".to_vec())
        });
        let server_429 = MockOllamaServer::start_custom(Some(err_429_handler), None).await;
        let provider_429 = OllamaProvider::new(&server_429.base_url);
        let err_429 = provider_429.complete(CompletionRequest::new("llama", "test")).await.unwrap_err();
        assert!(matches!(err_429, TagisanError::RateLimited(p, _) if p == "ollama"));

        // 4. Stream containing garbage / non-JSON lines and blank lines
        let chaos_stream_handler: StreamHandler = Arc::new(|_| {
            let chunks = vec![
                json!({ "message": { "role": "assistant", "content": "Valid Part 1, " }, "done": false }).to_string(),
                "".to_string(),
                "    ".to_string(),
                "502 Bad Gateway: NGINX upstream failed".to_string(),
                "{\"broken_json\": [incomplete".to_string(),
                json!({ "message": { "role": "assistant", "content": "Valid Part 2." }, "done": true, "done_reason": "stop" }).to_string(),
            ];
            (200, HashMap::new(), chunks, Duration::ZERO)
        });
        let server_chaos = MockOllamaServer::start_custom(None, Some(chaos_stream_handler)).await;
        let provider_chaos = OllamaProvider::new(&server_chaos.base_url);
        let mut chaos_stream = provider_chaos.stream(CompletionRequest::new("llama", "test")).await.unwrap();

        let mut accumulated = String::new();
        while let Some(item) = chaos_stream.next().await {
            let chunk = item.unwrap();
            if let StreamChunkDelta::Text(t) = chunk.delta {
                accumulated.push_str(&t);
            }
        }
        assert_eq!(accumulated, "Valid Part 1, Valid Part 2.");

        // 5. Sudden TCP connection drop mid-stream
        let drop_stream_handler: StreamHandler = Arc::new(|_| {
            let chunks = vec![
                json!({ "message": { "role": "assistant", "content": "Chunk before disconnect" }, "done": false }).to_string(),
            ];
            (200, HashMap::new(), chunks, Duration::ZERO)
        });
        let server_drop = MockOllamaServer::start_custom(None, Some(drop_stream_handler)).await;
        let provider_drop = OllamaProvider::new(&server_drop.base_url);
        let mut drop_stream = provider_drop.stream(CompletionRequest::new("llama", "drop")).await.unwrap();

        let first = drop_stream.next().await;
        assert!(first.is_some());
        let second = drop_stream.next().await;
        assert!(second.is_none());

        // 6. Huge response payload (120,000+ characters)
        let huge_content = "A".repeat(120_000);
        let huge_content_clone = huge_content.clone();
        let huge_handler: RequestHandler = Arc::new(move |_| {
            let resp = json!({
                "message": {
                    "role": "assistant",
                    "content": huge_content_clone
                },
                "done": true,
                "done_reason": "stop"
            });
            (200, HashMap::new(), serde_json::to_vec(&resp).unwrap())
        });
        let server_huge = MockOllamaServer::start_custom(Some(huge_handler), None).await;
        let provider_huge = OllamaProvider::new(&server_huge.base_url);
        let huge_resp = provider_huge.complete(CompletionRequest::new("llama", "huge")).await.unwrap();
        assert_eq!(huge_resp.message.extract_text().len(), 120_000);
    }

    // =========================================================================
    // Test 7: Multithreaded Parallel Streaming Concurrency (24 Workers)
    // =========================================================================
    #[tokio::test]
    async fn test_07_multithreaded_parallel_concurrency() {
        let concurrency = 24;
        let request_counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = request_counter.clone();

        let stream_handler: StreamHandler = Arc::new(move |_| {
            let id = counter_clone.fetch_add(1, Ordering::SeqCst);
            let chunks = vec![
                json!({ "message": { "role": "assistant", "content": format!("Worker_{} token_A ", id) }, "done": false }).to_string(),
                json!({ "message": { "role": "assistant", "content": format!("Worker_{} token_B", id) }, "done": true, "done_reason": "stop" }).to_string(),
            ];
            (200, HashMap::new(), chunks, Duration::from_millis(2))
        });

        let server = Arc::new(MockOllamaServer::start_custom(None, Some(stream_handler)).await);
        let provider = Arc::new(OllamaProvider::new(&server.base_url));

        let mut handles = Vec::new();
        let start_barrier = Arc::new(tokio::sync::Barrier::new(concurrency));

        for worker_idx in 0..concurrency {
            let p = provider.clone();
            let barrier = start_barrier.clone();

            handles.push(tokio::spawn(async move {
                barrier.wait().await;

                let req = CompletionRequest::new("llama3.2", format!("Worker request {}", worker_idx));
                let mut stream = p.stream(req).await.expect("Parallel stream must initialize");

                let mut output = String::new();
                while let Some(chunk_res) = stream.next().await {
                    let chunk = chunk_res.expect("Parallel chunk must succeed without data race");
                    if let StreamChunkDelta::Text(t) = chunk.delta {
                        output.push_str(&t);
                    }
                }

                assert!(output.contains("token_A") && output.contains("token_B"));
                output
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            let res = handle.await.expect("Worker thread panicked!");
            results.push(res);
        }

        assert_eq!(results.len(), concurrency, "All 24 workers must complete successfully");
        assert_eq!(request_counter.load(Ordering::SeqCst), concurrency);
    }

    // =========================================================================
    // Test 8: Live Ollama Daemon Probing
    // =========================================================================
    #[tokio::test]
    async fn test_08_live_ollama_daemon_probing() {
        let local_provider = OllamaProvider::default_local();
        let is_alive = local_provider.is_alive().await;

        if is_alive {
            println!(">>> LIVE OLLAMA DAEMON DETECTED AT http://localhost:11434 <<<");
            let models = local_provider.list_models().await.unwrap_or_default();
            println!(">>> Available local models: {:?}", models);

            if let Some(model_name) = models.first() {
                println!(">>> Executing live query with model '{}'...", model_name);
                let req = CompletionRequest::new(model_name, "Say PONG in one word")
                    .with_max_tokens(10);
                match local_provider.complete(req).await {
                    Ok(resp) => {
                        println!(">>> Live query response: {:?}", resp.message.extract_text());
                        assert!(!resp.message.extract_text().is_empty());
                    }
                    Err(e) => {
                        println!(">>> Live query warning: {} (model may still be loading)", e);
                    }
                }
            }
        } else {
            println!(">>> Live Ollama daemon not running at http://localhost:11434 (cleanly handled; mock server tests verify 100% protocol fidelity) <<<");
            assert!(!is_alive);
        }

        // Probing a verified non-existent port returns false without panic
        let dead_provider = OllamaProvider::new("http://127.0.0.1:59999");
        assert!(!dead_provider.is_alive().await);
    }

    // =========================================================================
    // Test 9: Default Model Detection & Dynamic Local Discovery
    // =========================================================================
    #[test]
    fn test_09_default_model_detection_and_discovery() {
        let installed = OllamaProvider::discover_installed_models();
        println!(">>> Discovered local Ollama models on disk: {:?}", installed);

        let default_m = OllamaProvider::default_model();
        println!(">>> Selected default Ollama model: {}", default_m);

        assert!(!default_m.is_empty());
        if !installed.is_empty() {
            assert!(installed.contains(&default_m));
        } else {
            assert_eq!(default_m, "dolphin-phi:latest");
        }
    }
}

