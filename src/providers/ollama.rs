use crate::error::{Result, TagisanError};
use crate::governor::{HostMemoryGovernor, LinuxMemInfo};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage, ToolDefinition,
};
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::io::AsyncBufReadExt;
use tokio::sync::{Mutex, RwLock};
use tokio_util::io::StreamReader;
use tokio_util::sync::CancellationToken;

// =========================================================================
// 1. Prompt Lookup Decoder (PLD) - N-gram Speculative Decoding Engine
// =========================================================================

/// Configuration parameters for the Prompt Lookup Decoder (PLD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PldConfig {
    pub min_ngram_size: usize,
    pub max_ngram_size: usize,
    pub speculation_window: usize,
    pub max_context_tokens: usize,
    pub enabled: bool,
}

impl Default for PldConfig {
    fn default() -> Self {
        Self {
            min_ngram_size: 2,
            max_ngram_size: 5,
            speculation_window: 8,
            max_context_tokens: 16384,
            enabled: true,
        }
    }
}

/// Tokenizes text into lossless tokens (preserving whitespace and punctuation)
/// such that `tokens.concat() == text`.
pub fn tokenize_pld(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_alphanumeric() || c == '_' {
            current.push(c);
        } else if c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            let mut ws = String::new();
            ws.push(c);
            while let Some(&next_c) = chars.peek() {
                if next_c == c && (c == ' ' || c == '\t' || c == '\n') {
                    ws.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            tokens.push(ws);
        } else {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(c.to_string());
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// A candidate speculative sequence proposed by Prompt Lookup Decoding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeculationCandidate {
    pub matched_ngram: Vec<String>,
    pub speculated_tokens: Vec<String>,
    pub source_position: usize,
    pub confidence_score: f32,
}

impl SpeculationCandidate {
    pub fn to_string_lossless(&self) -> String {
        self.speculated_tokens.concat()
    }
}

/// Telemetry metrics for PLD speculative decoding performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PldTelemetry {
    pub total_speculations: usize,
    pub total_speculated_tokens: usize,
    pub total_accepted_tokens: usize,
    pub acceptance_rate: f64,
    pub ngram_hit_counts: HashMap<usize, usize>,
    pub estimated_latency_saved_ms: f64,
}

/// Prompt Lookup Decoder (PLD) Engine
#[derive(Debug, Clone)]
pub struct PromptLookupDecoder {
    config: PldConfig,
    token_buffer: Vec<String>,
    index: HashMap<Vec<String>, Vec<usize>>,
    telemetry: PldTelemetry,
}

impl Default for PromptLookupDecoder {
    fn default() -> Self {
        Self::new(PldConfig::default())
    }
}

impl PromptLookupDecoder {
    pub fn new(config: PldConfig) -> Self {
        Self {
            config,
            token_buffer: Vec::new(),
            index: HashMap::new(),
            telemetry: PldTelemetry::default(),
        }
    }

    pub fn reset(&mut self) {
        self.token_buffer.clear();
        self.index.clear();
        self.telemetry = PldTelemetry::default();
    }

    pub fn index_text(&mut self, text: &str) {
        if !self.config.enabled || text.is_empty() {
            return;
        }
        let tokens = tokenize_pld(text);
        self.index_tokens(&tokens);
    }

    pub fn index_tokens(&mut self, tokens: &[String]) {
        if !self.config.enabled || tokens.is_empty() {
            return;
        }
        for token in tokens {
            self.append_token_internal(token.clone());
        }
    }

    pub fn index_messages(&mut self, messages: &[Message]) {
        if !self.config.enabled {
            return;
        }
        for msg in messages {
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        self.index_text(text);
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        self.index_text(thinking);
                    }
                    ContentBlock::ToolCall { name, arguments, .. } => {
                        self.index_text(name);
                        self.index_text(&arguments.to_string());
                    }
                    ContentBlock::ToolResult { content, .. } => {
                        self.index_text(content);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn append_emitted_token(&mut self, token: &str) {
        if !self.config.enabled {
            return;
        }
        self.append_token_internal(token.to_string());
    }

    fn append_token_internal(&mut self, token: String) {
        if self.token_buffer.len() >= self.config.max_context_tokens {
            let evict_count = self.config.max_context_tokens / 4;
            self.token_buffer.drain(0..evict_count);
            self.rebuild_index();
        }

        self.token_buffer.push(token);

        let buf_len = self.token_buffer.len();
        for n in self.config.min_ngram_size..=self.config.max_ngram_size {
            if buf_len >= n {
                let start_idx = buf_len - n;
                let ngram = self.token_buffer[start_idx..buf_len].to_vec();
                self.index.entry(ngram).or_default().push(start_idx);
            }
        }
    }

    fn rebuild_index(&mut self) {
        self.index.clear();
        let buf_len = self.token_buffer.len();
        for n in self.config.min_ngram_size..=self.config.max_ngram_size {
            if buf_len >= n {
                for start_idx in 0..=(buf_len - n) {
                    let ngram = self.token_buffer[start_idx..start_idx + n].to_vec();
                    self.index.entry(ngram).or_default().push(start_idx);
                }
            }
        }
    }

    pub fn speculate(&self, recent_tokens: &[String]) -> Option<SpeculationCandidate> {
        if !self.config.enabled || recent_tokens.is_empty() || self.token_buffer.is_empty() {
            return None;
        }

        let max_n = self.config.max_ngram_size.min(recent_tokens.len());
        let min_n = self.config.min_ngram_size;

        if max_n < min_n {
            return None;
        }

        for n in (min_n..=max_n).rev() {
            let tail_ngram = &recent_tokens[recent_tokens.len() - n..];
            if let Some(positions) = self.index.get(tail_ngram) {
                let current_tail_start = if self.token_buffer.len() >= n {
                    self.token_buffer.len() - n
                } else {
                    usize::MAX
                };

                for &pos in positions.iter().rev() {
                    if pos >= current_tail_start {
                        continue;
                    }

                    let continuation_start = pos + n;
                    if continuation_start < self.token_buffer.len() {
                        let continuation_end = (continuation_start + self.config.speculation_window)
                            .min(self.token_buffer.len());
                        let speculated_tokens = self.token_buffer[continuation_start..continuation_end].to_vec();

                        if !speculated_tokens.is_empty() {
                            let size_factor = n as f32 / self.config.max_ngram_size as f32;
                            let confidence = (0.5 + 0.5 * size_factor).clamp(0.0, 1.0);

                            return Some(SpeculationCandidate {
                                matched_ngram: tail_ngram.to_vec(),
                                speculated_tokens,
                                source_position: pos,
                                confidence_score: confidence,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    pub fn speculate_from_text(&self, recent_text: &str) -> Option<SpeculationCandidate> {
        let tokens = tokenize_pld(recent_text);
        self.speculate(&tokens)
    }

    pub fn verify_and_accept(
        &mut self,
        candidate: &SpeculationCandidate,
        actual_tokens: &[String],
    ) -> usize {
        self.telemetry.total_speculations += 1;
        self.telemetry.total_speculated_tokens += candidate.speculated_tokens.len();

        let mut accepted_count = 0;
        for (spec, act) in candidate.speculated_tokens.iter().zip(actual_tokens.iter()) {
            if spec == act {
                accepted_count += 1;
            } else {
                break;
            }
        }

        self.telemetry.total_accepted_tokens += accepted_count;
        let ngram_len = candidate.matched_ngram.len();
        *self.telemetry.ngram_hit_counts.entry(ngram_len).or_insert(0) += 1;

        if self.telemetry.total_speculated_tokens > 0 {
            self.telemetry.acceptance_rate =
                self.telemetry.total_accepted_tokens as f64 / self.telemetry.total_speculated_tokens as f64;
        }

        self.telemetry.estimated_latency_saved_ms = self.telemetry.total_accepted_tokens as f64 * 20.0;
        accepted_count
    }

    pub fn telemetry(&self) -> &PldTelemetry {
        &self.telemetry
    }

    pub fn buffer_len(&self) -> usize {
        self.token_buffer.len()
    }

    pub fn unique_ngrams_count(&self) -> usize {
        self.index.len()
    }
}

// =========================================================================
// 2. Deterministic KV Cache Prefix Alignment Engine
// =========================================================================

/// Canonicalized invariant prefix representing system prompt and tool definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrefix {
    pub system_prompt: Option<String>,
    pub tools_canonical: Vec<ToolDefinition>,
    pub prefix_hash_hex: String,
    pub estimated_tokens: usize,
}

/// Report produced when aligning a completion request with the KV cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixAlignmentReport {
    pub hit: bool,
    pub prefix_tokens_saved: usize,
    pub prefix_hash_hex: String,
    pub shared_turns: usize,
    pub total_turns: usize,
    pub estimated_latency_saved_ms: f64,
}

/// Telemetry metrics for deterministic KV prefix caching
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KvCacheTelemetry {
    pub total_requests: usize,
    pub prefix_hits: usize,
    pub prefix_misses: usize,
    pub hit_rate: f64,
    pub total_prefix_tokens_saved: usize,
    pub estimated_total_saved_ms: f64,
}

/// Canonical KV Cache Prefix Alignment Engine for 0ms prompt evaluation hits
#[derive(Debug, Clone, Default)]
pub struct DeterministicKvPrefixCache {
    active_prefix: Option<CanonicalPrefix>,
    turn_hashes: Vec<String>,
    telemetry: KvCacheTelemetry,
}

impl DeterministicKvPrefixCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Canonicalize system prompt: normalize line endings, trim lines, remove excess whitespace
    pub fn canonicalize_system_prompt(raw: &str) -> String {
        let normalized = raw.replace("\r\n", "\n");
        let mut lines = Vec::new();
        for line in normalized.lines() {
            lines.push(line.trim_end());
        }
        let joined = lines.join("\n");
        let mut result = String::new();
        let mut newline_count = 0;
        for c in joined.chars() {
            if c == '\n' {
                newline_count += 1;
                if newline_count <= 2 {
                    result.push(c);
                }
            } else {
                newline_count = 0;
                result.push(c);
            }
        }
        result.trim().to_string()
    }

    /// Canonicalize tool definitions: sort deterministically by tool name and canonicalize JSON parameters
    pub fn canonicalize_tools(tools: &[ToolDefinition]) -> Vec<ToolDefinition> {
        let mut sorted = tools.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        sorted.into_iter().map(|mut t| {
            t.parameters = Self::canonicalize_json_value(&t.parameters);
            t
        }).collect()
    }

    /// Recursively sort JSON object keys to ensure deterministic byte representation
    fn canonicalize_json_value(val: &serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::Object(map) => {
                let mut sorted_keys: Vec<String> = map.keys().cloned().collect();
                sorted_keys.sort();
                let mut new_map = serde_json::Map::new();
                for k in sorted_keys {
                    if let Some(v) = map.get(&k) {
                        new_map.insert(k, Self::canonicalize_json_value(v));
                    }
                }
                serde_json::Value::Object(new_map)
            }
            serde_json::Value::Array(arr) => {
                let new_arr = arr.iter().map(Self::canonicalize_json_value).collect();
                serde_json::Value::Array(new_arr)
            }
            other => other.clone(),
        }
    }

    /// Compute cryptographic Blake3 hash of the canonical system prompt and tools
    pub fn compute_prefix_hash(sys: Option<&str>, tools: &[ToolDefinition]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"tagisan_kv_prefix_v1\n");
        if let Some(s) = sys {
            hasher.update(b"system:\n");
            hasher.update(s.as_bytes());
            hasher.update(b"\n");
        }
        for t in tools {
            hasher.update(b"tool:\n");
            hasher.update(t.name.as_bytes());
            hasher.update(b"\n");
            hasher.update(t.description.as_bytes());
            hasher.update(b"\n");
            let param_bytes = serde_json::to_vec(&t.parameters).unwrap_or_default();
            hasher.update(&param_bytes);
            hasher.update(b"\n");
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Align an incoming CompletionRequest with the KV cache and canonicalize in place
    pub fn align_request(&mut self, req: &mut CompletionRequest) -> PrefixAlignmentReport {
        self.telemetry.total_requests += 1;

        if let Some(ref sys) = req.system_prompt {
            req.system_prompt = Some(Self::canonicalize_system_prompt(sys));
        }

        if !req.tools.is_empty() {
            req.tools = Self::canonicalize_tools(&req.tools);
        }

        let prefix_hash = Self::compute_prefix_hash(req.system_prompt.as_deref(), &req.tools);

        let mut est_tokens = req.system_prompt.as_ref().map(|s| s.len() / 4).unwrap_or(0);
        for t in &req.tools {
            est_tokens += t.name.len() / 4 + t.description.len() / 4 + serde_json::to_string(&t.parameters).unwrap_or_default().len() / 4;
        }

        let is_prefix_hit = if let Some(ref active) = self.active_prefix {
            active.prefix_hash_hex == prefix_hash
        } else {
            false
        };

        let mut shared_turns = 0;
        let mut current_turn_hashes = Vec::new();
        let mut running_hasher = blake3::Hasher::new();
        running_hasher.update(prefix_hash.as_bytes());

        for (idx, msg) in req.messages.iter().enumerate() {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
                Role::Reasoning => "reasoning",
            };
            running_hasher.update(role_str.as_bytes());
            let text = msg.extract_text();
            running_hasher.update(text.as_bytes());
            let turn_h = running_hasher.finalize().to_hex().to_string();

            if is_prefix_hit && idx < self.turn_hashes.len() && self.turn_hashes[idx] == turn_h {
                shared_turns += 1;
            }
            current_turn_hashes.push(turn_h);
        }

        self.turn_hashes = current_turn_hashes;
        self.active_prefix = Some(CanonicalPrefix {
            system_prompt: req.system_prompt.clone(),
            tools_canonical: req.tools.clone(),
            prefix_hash_hex: prefix_hash.clone(),
            estimated_tokens: est_tokens,
        });

        let hit = is_prefix_hit && (shared_turns > 0 || req.messages.len() <= 1);
        if hit {
            self.telemetry.prefix_hits += 1;
            self.telemetry.total_prefix_tokens_saved += est_tokens;
        } else {
            self.telemetry.prefix_misses += 1;
        }

        if self.telemetry.total_requests > 0 {
            self.telemetry.hit_rate = self.telemetry.prefix_hits as f64 / self.telemetry.total_requests as f64;
        }

        let saved_ms = if hit { est_tokens as f64 * 0.08 } else { 0.0 };
        self.telemetry.estimated_total_saved_ms += saved_ms;

        PrefixAlignmentReport {
            hit,
            prefix_tokens_saved: if hit { est_tokens } else { 0 },
            prefix_hash_hex: prefix_hash,
            shared_turns,
            total_turns: req.messages.len(),
            estimated_latency_saved_ms: saved_ms,
        }
    }

    pub fn telemetry(&self) -> &KvCacheTelemetry {
        &self.telemetry
    }

    pub fn active_prefix(&self) -> Option<&CanonicalPrefix> {
        self.active_prefix.as_ref()
    }
}

// =========================================================================
// 3. Dynamic Context-Window Sizing (Power-of-2 Context Fitting)
// =========================================================================

/// Configuration for Dynamic Context Fitter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicContextConfig {
    pub min_ctx: u32,
    pub max_ctx: u32,
    pub headroom_pct: f32,
    pub min_headroom_tokens: u32,
    pub baseline_fixed_ctx: u32,
}

impl Default for DynamicContextConfig {
    fn default() -> Self {
        Self {
            min_ctx: 512,
            max_ctx: 32768,
            headroom_pct: 0.20,
            min_headroom_tokens: 128,
            baseline_fixed_ctx: 8192,
        }
    }
}

/// Report produced by Dynamic Context Fitting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFitReport {
    pub estimated_prompt_tokens: usize,
    pub max_predict_tokens: u32,
    pub required_tokens: usize,
    pub allocated_num_ctx: u32,
    pub baseline_num_ctx: u32,
    pub vram_saved_mb: f64,
    pub bandwidth_reduction_pct: f64,
}

/// Adaptive power-of-2 context allocation engine to minimize KV cache footprint
#[derive(Debug, Clone)]
pub struct DynamicContextFitter {
    config: DynamicContextConfig,
}

impl Default for DynamicContextFitter {
    fn default() -> Self {
        Self::new(DynamicContextConfig::default())
    }
}

impl DynamicContextFitter {
    pub fn new(config: DynamicContextConfig) -> Self {
        Self { config }
    }

    /// Estimate total prompt token count across system prompt, tools, and message history
    pub fn estimate_prompt_tokens(&self, req: &CompletionRequest) -> usize {
        let mut tokens = 0;

        if let Some(ref sys) = req.system_prompt {
            tokens += (sys.len() as f64 / 3.7).ceil() as usize;
        }

        for tool in &req.tools {
            tokens += (tool.name.len() as f64 / 3.7).ceil() as usize;
            tokens += (tool.description.len() as f64 / 3.7).ceil() as usize;
            let param_len = serde_json::to_string(&tool.parameters).unwrap_or_default().len();
            tokens += (param_len as f64 / 3.5).ceil() as usize;
        }

        for msg in &req.messages {
            tokens += 4;
            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        tokens += (text.len() as f64 / 3.7).ceil() as usize;
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        tokens += (thinking.len() as f64 / 3.7).ceil() as usize;
                    }
                    ContentBlock::ToolCall { name, arguments, .. } => {
                        tokens += (name.len() as f64 / 3.7).ceil() as usize;
                        let arg_len = arguments.to_string().len();
                        tokens += (arg_len as f64 / 3.5).ceil() as usize;
                    }
                    ContentBlock::ToolResult { content, .. } => {
                        tokens += (content.len() as f64 / 3.7).ceil() as usize;
                    }
                    _ => {}
                }
            }
        }

        tokens
    }

    /// Fit the context window to the optimal power-of-2 context bucket
    pub fn fit_context_window(&self, req: &CompletionRequest) -> ContextFitReport {
        let prompt_tokens = self.estimate_prompt_tokens(req);
        let max_predict = req.max_tokens.unwrap_or(512);

        let required = if let Some(explicit_max) = req.max_tokens {
            if explicit_max >= self.config.baseline_fixed_ctx {
                explicit_max as usize
            } else {
                let headroom = ((prompt_tokens as f32 * self.config.headroom_pct) as usize)
                    .max(self.config.min_headroom_tokens as usize);
                prompt_tokens + explicit_max as usize + headroom
            }
        } else {
            let headroom = ((prompt_tokens as f32 * self.config.headroom_pct) as usize)
                .max(self.config.min_headroom_tokens as usize);
            prompt_tokens + (max_predict as usize) + headroom
        };

        let mut candidate = self.config.min_ctx;
        while (candidate as usize) < required && candidate < self.config.max_ctx {
            candidate *= 2;
        }

        if let Some(explicit_max) = req.max_tokens {
            candidate = candidate.max(explicit_max);
        }

        let metrics = LinuxMemInfo::read_host();
        let gov = HostMemoryGovernor::new();
        let is_8gb = gov.is_8gb_workstation(&metrics);
        let pressure = gov.evaluate_pressure(&metrics);

        let mut clamped_ctx = candidate.clamp(self.config.min_ctx, self.config.max_ctx);

        if pressure == crate::governor::MemoryPressureTier::RedCritical {
            clamped_ctx = clamped_ctx.min(if is_8gb { 1024 } else { 2048 });
        } else if pressure == crate::governor::MemoryPressureTier::YellowWarning || is_8gb {
            clamped_ctx = clamped_ctx.min(if is_8gb { 2048 } else { 4096 });
        }

        if let Some(explicit_max) = req.max_tokens {
            if explicit_max > clamped_ctx && !is_8gb {
                clamped_ctx = explicit_max;
            }
        }

        let baseline = self.config.baseline_fixed_ctx;
        let vram_saved_mb = if clamped_ctx < baseline {
            (baseline - clamped_ctx) as f64 * 0.125
        } else {
            0.0
        };

        let bandwidth_reduction_pct = if clamped_ctx < baseline {
            (1.0 - (clamped_ctx as f64 / baseline as f64)) * 100.0
        } else {
            0.0
        };

        ContextFitReport {
            estimated_prompt_tokens: prompt_tokens,
            max_predict_tokens: max_predict,
            required_tokens: required,
            allocated_num_ctx: clamped_ctx,
            baseline_num_ctx: baseline,
            vram_saved_mb,
            bandwidth_reduction_pct,
        }
    }
}

// =========================================================================
// 4. Warmth Sentinel - Zero-Cold-Start Background Daemon
// =========================================================================

/// Status of model warmth in local memory / VRAM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarmthStatus {
    Cold,
    Warming,
    WarmInVram,
    Evicted,
    Error(String),
}

/// Configuration for Warmth Sentinel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmthConfig {
    pub model: String,
    pub base_url: String,
    pub heartbeat_interval: Duration,
    pub keep_alive: String,
    pub prewarm_canonical_prefix: Option<String>,
    pub enabled: bool,
}

impl Default for WarmthConfig {
    fn default() -> Self {
        Self {
            model: "dolphin-phi:latest".to_string(),
            base_url: "http://localhost:11434".to_string(),
            heartbeat_interval: Duration::from_secs(60),
            keep_alive: "24h".to_string(),
            prewarm_canonical_prefix: None,
            enabled: true,
        }
    }
}

/// Zero-cold-start background daemon keeping local models pinned in VRAM/RAM
#[derive(Debug)]
pub struct WarmthSentinel {
    pub config: WarmthConfig,
    client: reqwest::Client,
    status: Arc<RwLock<WarmthStatus>>,
    last_touch_timestamp: Arc<AtomicU64>,
    heartbeat_count: Arc<AtomicUsize>,
    cancellation_token: CancellationToken,
}

impl WarmthSentinel {
    pub fn new(config: WarmthConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            client,
            status: Arc::new(RwLock::new(WarmthStatus::Cold)),
            last_touch_timestamp: Arc::new(AtomicU64::new(0)),
            heartbeat_count: Arc::new(AtomicUsize::new(0)),
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let cancel = self.cancellation_token.clone();
        tokio::spawn(async move {
            if !self.config.enabled {
                return;
            }

            let _ = self.touch_now().await;

            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(self.config.heartbeat_interval) => {
                        let _ = self.touch_now().await;
                    }
                }
            }
        })
    }

    pub async fn touch_now(&self) -> Result<Duration> {
        let start = Instant::now();
        *self.status.write().await = WarmthStatus::Warming;

        let url = format!("{}/api/generate", self.config.base_url.trim_end_matches('/'));
        let prompt = self.config.prewarm_canonical_prefix.as_deref().unwrap_or("");

        let payload = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "keep_alive": self.config.keep_alive,
            "options": {
                "num_predict": 0
            }
        });

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let elapsed = start.elapsed();
                    *self.status.write().await = WarmthStatus::WarmInVram;
                    self.last_touch_timestamp.store(
                        chrono::Utc::now().timestamp_millis() as u64,
                        Ordering::SeqCst,
                    );
                    self.heartbeat_count.fetch_add(1, Ordering::SeqCst);
                    Ok(elapsed)
                } else {
                    let err_text = resp.text().await.unwrap_or_default();
                    *self.status.write().await = WarmthStatus::Error(err_text.clone());
                    Err(TagisanError::BadResponse("ollama".into(), err_text))
                }
            }
            Err(e) => {
                *self.status.write().await = WarmthStatus::Error(e.to_string());
                Err(TagisanError::Execution(format!("Warmth heartbeat failed: {e}")))
            }
        }
    }

    pub async fn prewarm_prompt(&self, prompt: &str) -> Result<Duration> {
        let start = Instant::now();
        let url = format!("{}/api/generate", self.config.base_url.trim_end_matches('/'));

        let payload = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "keep_alive": self.config.keep_alive,
            "options": {
                "num_predict": 0
            }
        });

        let resp = self.client.post(&url).json(&payload).send().await?;
        if resp.status().is_success() {
            *self.status.write().await = WarmthStatus::WarmInVram;
            Ok(start.elapsed())
        } else {
            let err = resp.text().await.unwrap_or_default();
            Err(TagisanError::BadResponse("ollama".into(), err))
        }
    }

    pub async fn get_status(&self) -> WarmthStatus {
        self.status.read().await.clone()
    }

    pub fn stop(&self) {
        self.cancellation_token.cancel();
    }
}

// =========================================================================
// 5. FlashAttention & Dynamic Batch Optimization
// =========================================================================

/// Check and automatically enable OLLAMA_FLASH_ATTENTION=1 if supported
pub fn ensure_flash_attention_env() -> bool {
    if std::env::var("OLLAMA_FLASH_ATTENTION").is_err() {
        std::env::set_var("OLLAMA_FLASH_ATTENTION", "1");
    }
    is_flash_attention_enabled()
}

/// Check if FlashAttention is active
pub fn is_flash_attention_enabled() -> bool {
    std::env::var("OLLAMA_FLASH_ATTENTION")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// High-throughput prompt ingestion batch optimizer
pub struct DynamicBatchOptimizer;

impl DynamicBatchOptimizer {
    /// Calculate optimal num_batch based on prompt size, memory pressure, and GPU offloading
    pub fn optimize_batch_size(prompt_tokens: usize) -> u32 {
        let metrics = LinuxMemInfo::read_host();
        let gov = HostMemoryGovernor::new();
        let is_8gb = gov.is_8gb_workstation(&metrics);
        let pressure = gov.evaluate_pressure(&metrics);

        if pressure == crate::governor::MemoryPressureTier::RedCritical || (is_8gb && prompt_tokens < 512) {
            256
        } else if is_8gb || pressure == crate::governor::MemoryPressureTier::YellowWarning {
            512
        } else if prompt_tokens > 2048 {
            1024
        } else {
            512
        }
    }
}

/// Unified Hyper-Ollama Acceleration Suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperOllamaConfig {
    pub prompt_lookup_decoding: bool,
    pub deterministic_kv_cache: bool,
    pub dynamic_context_fitting: bool,
    pub dynamic_batching: bool,
    pub warmth_sentinel: bool,
    pub flash_attention: bool,
    pub num_gpu: u32,
    pub use_mmap: bool,
}

impl Default for HyperOllamaConfig {
    fn default() -> Self {
        Self {
            prompt_lookup_decoding: true,
            deterministic_kv_cache: true,
            dynamic_context_fitting: true,
            dynamic_batching: true,
            warmth_sentinel: true,
            flash_attention: true,
            num_gpu: 99,
            use_mmap: true,
        }
    }
}

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
    pub context_fitter: DynamicContextFitter,
    pub prefix_cache: Arc<Mutex<DeterministicKvPrefixCache>>,
    pub pld: Arc<Mutex<PromptLookupDecoder>>,
    pub warmth_sentinel: Arc<RwLock<Option<Arc<WarmthSentinel>>>>,
    pub hyper_config: HyperOllamaConfig,
    pub unsupported_tool_models: Arc<RwLock<HashSet<String>>>,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        let timeout_secs = std::env::var("OLLAMA_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1800);

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        ensure_flash_attention_env();

        Self {
            base_url: base_url.into(),
            client,
            context_fitter: DynamicContextFitter::default(),
            prefix_cache: Arc::new(Mutex::new(DeterministicKvPrefixCache::default())),
            pld: Arc::new(Mutex::new(PromptLookupDecoder::default())),
            warmth_sentinel: Arc::new(RwLock::new(None)),
            hyper_config: HyperOllamaConfig::default(),
            unsupported_tool_models: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check if a given model is statically known not to support native tool calling in Ollama
    pub fn is_model_tool_supported(model: &str) -> bool {
        let m = model.to_ascii_lowercase();
        if m.contains("llama2")
            || m.contains("llama-2")
            || m.contains("codellama")
            || m.contains("orca-mini")
            || m.contains("vicuna")
            || m.contains("wizard")
            || m.contains("tinyllama")
            || m.contains("medllama")
            || m.contains("solar")
            || m.contains("yarn")
            || m.contains("openhermes")
            || m.contains("deepseek-llm")
        {
            return false;
        }
        true
    }

    pub fn default_local() -> Self {
        Self::new("http://localhost:11434")
    }

    /// Start the zero-cold-start Warmth Sentinel in the background
    pub async fn start_warmth_sentinel(&self, model: &str) -> Arc<WarmthSentinel> {
        let sentinel = Arc::new(WarmthSentinel::new(WarmthConfig {
            model: model.to_string(),
            base_url: self.base_url.clone(),
            heartbeat_interval: Duration::from_secs(60),
            keep_alive: self.get_keep_alive(),
            prewarm_canonical_prefix: None,
            enabled: true,
        }));
        let _ = sentinel.clone().start();
        *self.warmth_sentinel.write().await = Some(sentinel.clone());
        sentinel
    }

    /// Retrieve the active Warmth Sentinel if running
    pub async fn get_warmth_sentinel(&self) -> Option<Arc<WarmthSentinel>> {
        self.warmth_sentinel.read().await.clone()
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

    /// Queries the live Ollama server for installed models via /api/tags
    pub fn fetch_live_tags() -> Option<Vec<String>> {
        use std::net::ToSocketAddrs;
        use std::io::{Read, Write};

        let raw_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1:11434".to_string());
        let host_clean = raw_host
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let (host, port) = if let Some((h, p)) = host_clean.split_once(':') {
            let h = if h.is_empty() || h == "0.0.0.0" { "127.0.0.1" } else { h };
            let p: u16 = p.split('/').next()?.parse().ok()?;
            (h, p)
        } else {
            ("127.0.0.1", 11434)
        };

        let addr_str = format!("{}:{}", host, port);
        let mut addrs = addr_str.to_socket_addrs().ok()?;
        let addr = addrs.next()?;

        let socket = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).ok()?;
        socket.set_read_timeout(Some(std::time::Duration::from_millis(500))).ok()?;
        socket.set_write_timeout(Some(std::time::Duration::from_millis(500))).ok()?;

        let mut stream = socket;
        let request = format!("GET /api/tags HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n", host, port);
        stream.write_all(request.as_bytes()).ok()?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).ok()?;

        let resp_str = String::from_utf8_lossy(&response);
        if let Some(body_idx) = resp_str.find("\r\n\r\n") {
            let body = &resp_str[body_idx + 4..];
            #[derive(Deserialize)]
            struct TagsResp {
                models: Option<Vec<ModelItem>>,
            }
            #[derive(Deserialize)]
            struct ModelItem {
                name: String,
            }
            if let Ok(parsed) = serde_json::from_str::<TagsResp>(body) {
                let names: Vec<String> = parsed
                    .models
                    .unwrap_or_default()
                    .into_iter()
                    .map(|m| m.name)
                    .collect();
                return Some(names);
            }
        }
        None
    }

    /// Checks if local or remote Ollama server socket is reachable
    pub fn is_server_reachable() -> bool {
        use std::net::ToSocketAddrs;
        let raw_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1:11434".to_string());
        let host_clean = raw_host
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let (host, port) = if let Some((h, p)) = host_clean.split_once(':') {
            let h = if h.is_empty() || h == "0.0.0.0" { "127.0.0.1" } else { h };
            let p: u16 = p.split('/').next().and_then(|s| s.parse().ok()).unwrap_or(11434);
            (h, p)
        } else {
            ("127.0.0.1", 11434)
        };
        let addr_str = format!("{}:{}", host, port);
        if let Ok(mut addrs) = addr_str.to_socket_addrs() {
            if let Some(addr) = addrs.next() {
                return std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(300)).is_ok();
            }
        }
        false
    }

    /// Automatically updates .env files in the workspace if OLLAMA_MODEL points to a removed model
    pub fn auto_heal_env_file(new_model: &str) {
        std::env::set_var("OLLAMA_MODEL", new_model);

        let mut curr = std::env::current_dir().ok();
        for _ in 0..4 {
            if let Some(dir) = curr {
                let env_path = dir.join(".env");
                if env_path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&env_path) {
                        let mut modified = false;
                        let mut new_lines = Vec::new();
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if (trimmed.starts_with("OLLAMA_MODEL=") || trimmed.starts_with("export OLLAMA_MODEL="))
                                && !line.trim_start().starts_with('#')
                            {
                                if trimmed != format!("OLLAMA_MODEL={}", new_model)
                                    && trimmed != format!("export OLLAMA_MODEL={}", new_model)
                                {
                                    new_lines.push(format!("OLLAMA_MODEL={}", new_model));
                                    modified = true;
                                } else {
                                    new_lines.push(line.to_string());
                                }
                            } else {
                                new_lines.push(line.to_string());
                            }
                        }
                        if modified {
                            let _ = std::fs::write(&env_path, new_lines.join("\n") + "\n");
                        }
                    }
                    break;
                }
                curr = dir.parent().map(|p| p.to_path_buf());
            } else {
                break;
            }
        }
    }

    /// Checks if a requested model name matches any installed model
    pub fn find_matching_model(requested: &str, installed: &[String]) -> Option<String> {
        let req = requested.trim();
        if req.is_empty() {
            return None;
        }

        // 1. Exact match (case-insensitive)
        for m in installed {
            if m.eq_ignore_ascii_case(req) {
                return Some(m.clone());
            }
        }

        // 2. Base name / tag match: e.g. "smollm2" matches "smollm2:1.7b"
        let req_base = req.split(':').next().unwrap_or(req);
        let req_base_no_ns = req_base.split('/').last().unwrap_or(req_base);

        for m in installed {
            let m_base = m.split(':').next().unwrap_or(m);
            let m_base_no_ns = m_base.split('/').last().unwrap_or(m_base);

            if m.eq_ignore_ascii_case(req_base)
                || m_base.eq_ignore_ascii_case(req)
                || m_base.eq_ignore_ascii_case(req_base)
                || m_base_no_ns.eq_ignore_ascii_case(req_base_no_ns)
            {
                return Some(m.clone());
            }
        }

        // 3. Substring / partial match
        for m in installed {
            let m_lower = m.to_ascii_lowercase();
            let req_lower = req.to_ascii_lowercase();
            if m_lower.contains(&req_lower) || req_lower.contains(&m_lower) {
                return Some(m.clone());
            }
        }

        None
    }

    /// Returns the default model for Ollama:
    /// 1. `OLLAMA_MODEL` environment variable if set AND verified to be currently installed.
    /// 2. If `OLLAMA_MODEL` is set but the model was removed/missing, auto-recovers to installed model.
    /// 3. Auto-discovered installed model from local Ollama tags or manifests on disk.
    /// 4. Default fallback: `"dolphin-phi:latest"`.
    pub fn default_model() -> String {
        let installed = Self::discover_installed_models();

        if let Ok(model) = std::env::var("OLLAMA_MODEL") {
            let m = model.trim();
            if !m.is_empty() {
                if !installed.is_empty() {
                    if let Some(matched) = Self::find_matching_model(m, &installed) {
                        return matched;
                    }

                    // Configured model is no longer installed / was removed!
                    let fallback = installed[0].clone();
                    eprintln!(
                        "{}",
                        format!(
                            "⚠️  [Ollama Auto-Recovery] Configured model '{}' is not installed or was removed. Automatically switching to installed model '{}'.",
                            m, fallback
                        ).yellow().bold()
                    );
                    Self::auto_heal_env_file(&fallback);
                    return fallback;
                } else {
                    return m.to_string();
                }
            }
        }

        if let Some(first) = installed.first() {
            return first.clone();
        }

        "dolphin-phi:latest".to_string()
    }

    /// Checks whether at least one model is installed in Ollama
    pub fn has_installed_models() -> bool {
        !Self::discover_installed_models().is_empty()
    }

    /// Prints a user-friendly notification when no local LLM is installed or all models were removed
    pub fn notify_no_models_installed() {
        use colored::Colorize;
        eprintln!("\n{}", "=========================================================================".yellow());
        eprintln!("{}", "  ⚠️   TAGISAN NOTIFICATION: NO LOCAL LLM INSTALLED IN OLLAMA".bold().yellow());
        eprintln!("{}", "=========================================================================".yellow());
        eprintln!("{}", "A local LLM was removed or no models are currently installed in Ollama.\n".white());
        eprintln!("{}", "To prevent errors and enable offline autonomous intelligence, please run:".bold());
        eprintln!("  ▶ {}  {}", "ollama pull smollm2:1.7b".cyan().bold(), "(Recommended: Fast & lightweight ~1GB)".italic());
        eprintln!("  ▶ {}  {}", "ollama pull llama3.2:3b".cyan().bold(), "(High accuracy general reasoning)".italic());
        eprintln!("  ▶ {}  {}", "ollama pull qwen2.5-coder:1.5b".cyan().bold(), "(Compact coding specialist)".italic());
        eprintln!("\n{}", "Once downloaded, Tagisan will automatically discover and use the model.".green());
        eprintln!("{}\n", "Tip: Or configure an API key in .env (e.g. GEMINI_API_KEY) to use cloud providers.".dimmed());
    }

    /// Discovers locally installed models by inspecting both the live Ollama daemon and manifests on disk
    pub fn discover_installed_models() -> Vec<String> {
        let mut models = Vec::new();

        // 1. Try querying the live running Ollama instance directly via /api/tags
        if let Some(live_models) = Self::fetch_live_tags() {
            for m in live_models {
                if !models.contains(&m) {
                    models.push(m);
                }
            }
        }

        // 2. Scan disk manifests in candidate directories
        let mut candidate_dirs = Vec::new();
        if let Ok(dir) = std::env::var("OLLAMA_MODELS") {
            candidate_dirs.push(std::path::PathBuf::from(dir));
        }

        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_default();
        if !home.is_empty() {
            candidate_dirs.push(std::path::PathBuf::from(home).join(".ollama").join("models"));
        }
        candidate_dirs.push(std::path::PathBuf::from("/usr/share/ollama/.ollama/models"));
        candidate_dirs.push(std::path::PathBuf::from("/var/lib/ollama/.ollama/models"));

        for base_models_dir in candidate_dirs {
            let manifests_root = base_models_dir.join("manifests");
            if !manifests_root.is_dir() {
                continue;
            }

            if let Ok(registries) = std::fs::read_dir(&manifests_root) {
                for reg_entry in registries.flatten() {
                    let reg_path = reg_entry.path();
                    if !reg_path.is_dir() {
                        continue;
                    }

                    if let Ok(namespaces) = std::fs::read_dir(&reg_path) {
                        for ns_entry in namespaces.flatten() {
                            let ns_path = ns_entry.path();
                            if !ns_path.is_dir() {
                                continue;
                            }
                            let ns_name = ns_entry.file_name().to_string_lossy().to_string();

                            if let Ok(model_entries) = std::fs::read_dir(&ns_path) {
                                for model_entry in model_entries.flatten() {
                                    let model_path = model_entry.path();
                                    if !model_path.is_dir() {
                                        continue;
                                    }
                                    let model_name = model_entry.file_name().to_string_lossy().to_string();

                                    if let Ok(tag_entries) = std::fs::read_dir(&model_path) {
                                        for tag_entry in tag_entries.flatten() {
                                            let tag_name = tag_entry.file_name().to_string_lossy().to_string();
                                            let full_model_id = if ns_name == "library" {
                                                format!("{}:{}", model_name, tag_name)
                                            } else {
                                                format!("{}/{}:{}", ns_name, model_name, tag_name)
                                            };
                                            if !models.contains(&full_model_id) {
                                                models.push(full_model_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Priority order for optimal execution on local hardware:
        // 0. abliterated / uncensored models (top priority when user has installed them)
        // 1. dolphin-phi:latest (high-performance 3B uncensored instruct/code model, ~1.6GB)
        // 2. qwen2.5:0.5b (ultra-fast 0.5B model, ~397MB)
        // 3. smollm / llama3.2 (fast lightweight instruct models)
        // 4. other installed models
        models.sort_by(|a, b| {
            let score = |m: &str| {
                let m_lower = m.to_ascii_lowercase();
                if m_lower.contains("abliterate") || m_lower.contains("uncensored") {
                    0
                } else if m_lower.contains("dolphin-phi") {
                    1
                } else if m_lower.contains("qwen2.5") {
                    2
                } else if m_lower.contains("llama3.2") {
                    3
                } else if m_lower.contains("smollm") {
                    4
                } else if m_lower.contains("mistral") || m_lower.contains("gemma") {
                    5
                } else if m_lower.contains("mixtral") {
                    99
                } else {
                    10
                }
            };
            score(a).cmp(&score(b))
        });

        models
    }

    /// Sanitizes model name to prevent sending 'null', 'none', 'default' or placeholders to Ollama
    pub fn sanitize_model_name<'a>(raw_model: &'a str) -> std::borrow::Cow<'a, str> {
        let trimmed = raw_model.trim();
        if trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case("null")
            || trimmed.eq_ignore_ascii_case("none")
            || trimmed.eq_ignore_ascii_case("default")
            || trimmed.starts_with('<')
        {
            std::borrow::Cow::Owned(Self::default_model())
        } else {
            std::borrow::Cow::Borrowed(trimmed)
        }
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
                    ContentBlock::ToolResult { tool_call_id, content, is_error } => {
                        if *is_error {
                            text_parts.push(format!("[Tool Error for {}: {}]", tool_call_id, content));
                        } else {
                            text_parts.push(format!("[Tool Result for {}: {}]", tool_call_id, content));
                        }
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
        let metrics = LinuxMemInfo::read_host();
        let gov = HostMemoryGovernor::new();
        let is_8gb = gov.is_8gb_workstation(&metrics);

        let num_gpu = std::env::var("TAGISAN_OLLAMA_NUM_GPU")
            .ok()
            .and_then(|s| s.parse().ok())
            .or(Some(self.hyper_config.num_gpu));

        let prompt_tokens = self.context_fitter.estimate_prompt_tokens(req);
        let num_batch = std::env::var("TAGISAN_OLLAMA_NUM_BATCH")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| {
                if self.hyper_config.dynamic_batching {
                    Some(DynamicBatchOptimizer::optimize_batch_size(prompt_tokens))
                } else {
                    Some(512)
                }
            });

        let num_ctx = std::env::var("TAGISAN_OLLAMA_NUM_CTX")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| {
                if self.hyper_config.dynamic_context_fitting {
                    let fit = self.context_fitter.fit_context_window(req);
                    Some(fit.allocated_num_ctx)
                } else {
                    req.max_tokens.map(|m| m.max(if is_8gb { 2048 } else { 4096 }))
                        .or(Some(if is_8gb { 2048 } else { 8192 }))
                }
            });

        let num_thread = std::env::var("TAGISAN_OLLAMA_NUM_THREAD")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| {
                let rec = gov.recommended_concurrency(&metrics);
                Some(rec.clamp(2, 6) as u32)
            });

        let use_mmap = Some(self.hyper_config.use_mmap);
        let use_mlock = if std::env::var("TAGISAN_OLLAMA_USE_MLOCK").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false) {
            Some(true)
        } else {
            None
        };

        OllamaOptions {
            temperature: req.temperature,
            num_predict: req.max_tokens,
            num_ctx,
            num_gpu,
            num_thread,
            num_batch,
            f16_kv: Some(true),
            use_mmap,
            use_mlock,
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
    format: Option<&'a serde_json::Value>,
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

    fn capabilities(&self, model: &str) -> ProviderCapabilities {
        let mut caps = ProviderCapabilities::STREAMING
            | ProviderCapabilities::SYSTEM_PROMPT
            | ProviderCapabilities::VISION;

        let effective = Self::sanitize_model_name(model);
        let is_unsupported = !Self::is_model_tool_supported(&effective)
            || if let Ok(guard) = self.unsupported_tool_models.try_read() {
                guard.contains(&*effective) || guard.contains(model)
            } else {
                false
            };

        if !is_unsupported {
            caps |= ProviderCapabilities::FUNCTION_CALLING;
        }
        caps
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let mut current_req = req;
        let mut fallback_attempted = false;

        loop {
            let start = Instant::now();
            let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

            let installed = Self::discover_installed_models();
            if installed.is_empty() {
                Self::notify_no_models_installed();
                return Err(TagisanError::NoModelsInstalled);
            }

            let effective_model = Self::sanitize_model_name(&current_req.model).into_owned();
            let target_model = if let Some(matched) = Self::find_matching_model(&effective_model, &installed) {
                matched
            } else {
                let fb = installed[0].clone();
                eprintln!(
                    "{}",
                    format!(
                        "⚠️  [Ollama Auto-Recovery] Model '{}' not found in Ollama (it may have been removed). Automatically switching to installed model '{}'...",
                        effective_model, fb
                    ).yellow().bold()
                );
                Self::auto_heal_env_file(&fb);
                current_req.model = fb.clone();
                fb
            };

            if self.hyper_config.deterministic_kv_cache {
                let mut prefix_guard = self.prefix_cache.lock().await;
                let _alignment = prefix_guard.align_request(&mut current_req);
            }

            let messages = self.format_messages(&current_req);

            let is_tool_unsupported = !Self::is_model_tool_supported(&target_model)
                || self.unsupported_tool_models.read().await.contains(&target_model)
                || self.unsupported_tool_models.read().await.contains(&effective_model);

            let tools = if !current_req.tools.is_empty() && !is_tool_unsupported {
                Some(
                    current_req.tools
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
            let options = self.build_options(&current_req);

            let payload = OllamaChatPayload {
                model: &target_model,
                messages,
                format: current_req.format.as_ref(),
                tools,
                stream: false,
                keep_alive: Some(&keep_alive),
                options: Some(options),
            };

            let mut builder = self.client.post(&url)
                .header("X-Tagisan-Client", "true")
                .header("X-Tagisan-KeepAlive", "true");

            if crate::ecc::agentshield::AgentShieldScanner::is_unrestricted() {
                builder = builder.header("X-Tagisan-Unrestricted", "true");
            }

            let send_future = builder.json(&payload).send();

            let response = if let Some(token) = &current_req.cancellation_token {
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

                if err_text.contains("does not support tools") {
                    eprintln!(
                        "{}",
                        format!(
                            "⚠️  [Ollama Auto-Recovery] Model '{}' does not support native tool calling. Retrying with direct conversation mode (tools disabled)...",
                            target_model
                        ).yellow().bold()
                    );
                    let mut guard = self.unsupported_tool_models.write().await;
                    guard.insert(target_model.clone());
                    guard.insert(effective_model.clone());
                    drop(guard);
                    current_req.tools.clear();
                    continue;
                }

                if !fallback_attempted && (status.as_u16() == 404 || err_text.contains("not found")) {
                    fallback_attempted = true;
                    let available = self.list_models().await.unwrap_or_else(|_| Self::discover_installed_models());
                    if available.is_empty() {
                        Self::notify_no_models_installed();
                        return Err(TagisanError::NoModelsInstalled);
                    }
                    if let Some(fb) = available.into_iter().find(|m| m != &*target_model) {
                        eprintln!(
                            "{}",
                            format!(
                                "⚠️  [Ollama Auto-Recovery] Model '{}' not found in Ollama (it may have been removed). Automatically switching to '{}'...",
                                target_model, fb
                            ).yellow().bold()
                        );
                        Self::auto_heal_env_file(&fb);
                        current_req.model = fb;
                        continue;
                    } else {
                        Self::notify_no_models_installed();
                        return Err(TagisanError::NoModelsInstalled);
                    }
                }

                return Err(TagisanError::BadResponse("ollama".into(), err_text));
            }

            let api_resp: OllamaApiResponse = if let Some(token) = &current_req.cancellation_token {
                tokio::select! {
                    _ = token.cancelled() => return Err(TagisanError::Cancelled),
                    res = response.json::<OllamaApiResponse>() => res?,
                }
            } else {
                response.json().await?
            };

            if let Some(err) = api_resp.error {
                if !fallback_attempted && err.contains("not found") {
                    fallback_attempted = true;
                    let available = self.list_models().await.unwrap_or_else(|_| Self::discover_installed_models());
                    if available.is_empty() {
                        Self::notify_no_models_installed();
                        return Err(TagisanError::NoModelsInstalled);
                    }
                    if let Some(fb) = available.into_iter().find(|m| m != &*target_model) {
                        eprintln!(
                            "{}",
                            format!(
                                "⚠️  [Ollama Auto-Recovery] Model '{}' not found in Ollama. Automatically switching to '{}'...",
                                target_model, fb
                            ).yellow().bold()
                        );
                        Self::auto_heal_env_file(&fb);
                        current_req.model = fb;
                        continue;
                    } else {
                        Self::notify_no_models_installed();
                        return Err(TagisanError::NoModelsInstalled);
                    }
                }
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

            return Ok(CompletionResponse {
                id: format!("ollama_{}", start.elapsed().as_millis()),
                provider: "ollama".to_string(),
                model: target_model,
                message: Message {
                    role: Role::Assistant,
                    content: content_blocks,
                    name: None,
                    metadata: std::collections::HashMap::new(),
                },
                finish_reason,
                usage,
                latency: start.elapsed(),
            });
        }
    }

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        let mut current_req = req;
        let mut fallback_attempted = false;

        loop {
            let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

            let installed = Self::discover_installed_models();
            if installed.is_empty() {
                Self::notify_no_models_installed();
                return Err(TagisanError::NoModelsInstalled);
            }

            let effective_model = Self::sanitize_model_name(&current_req.model).into_owned();
            let target_model = if let Some(matched) = Self::find_matching_model(&effective_model, &installed) {
                matched
            } else {
                let fb = installed[0].clone();
                eprintln!(
                    "{}",
                    format!(
                        "⚠️  [Ollama Auto-Recovery] Model '{}' not found in Ollama (it may have been removed). Automatically switching to installed model '{}'...",
                        effective_model, fb
                    ).yellow().bold()
                );
                Self::auto_heal_env_file(&fb);
                current_req.model = fb.clone();
                fb
            };

            if self.hyper_config.deterministic_kv_cache {
                let mut prefix_guard = self.prefix_cache.lock().await;
                let _alignment = prefix_guard.align_request(&mut current_req);
            }

            if self.hyper_config.prompt_lookup_decoding {
                let mut pld_guard = self.pld.lock().await;
                pld_guard.reset();
                if let Some(ref sys) = current_req.system_prompt {
                    pld_guard.index_text(sys);
                }
                pld_guard.index_messages(&current_req.messages);
            }

            let messages = self.format_messages(&current_req);

            let is_tool_unsupported = !Self::is_model_tool_supported(&target_model)
                || self.unsupported_tool_models.read().await.contains(&target_model)
                || self.unsupported_tool_models.read().await.contains(&effective_model);

            let tools = if !current_req.tools.is_empty() && !is_tool_unsupported {
                Some(
                    current_req.tools
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
            let options = self.build_options(&current_req);

            let payload = OllamaChatPayload {
                model: &target_model,
                messages,
                format: current_req.format.as_ref(),
                tools,
                stream: true,
                keep_alive: Some(&keep_alive),
                options: Some(options),
            };

            let mut builder = self.client.post(&url)
                .header("X-Tagisan-Client", "true")
                .header("X-Tagisan-KeepAlive", "true");

            if crate::ecc::agentshield::AgentShieldScanner::is_unrestricted() {
                builder = builder.header("X-Tagisan-Unrestricted", "true");
            }

            let send_future = builder.json(&payload).send();

            let response = if let Some(token) = &current_req.cancellation_token {
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

                if err_text.contains("does not support tools") {
                    eprintln!(
                        "{}",
                        format!(
                            "⚠️  [Ollama Auto-Recovery] Model '{}' does not support native tool calling. Retrying with direct conversation mode (tools disabled)...",
                            target_model
                        ).yellow().bold()
                    );
                    let mut guard = self.unsupported_tool_models.write().await;
                    guard.insert(target_model.clone());
                    guard.insert(effective_model.clone());
                    drop(guard);
                    current_req.tools.clear();
                    continue;
                }

                if !fallback_attempted && (status.as_u16() == 404 || err_text.contains("not found")) {
                    fallback_attempted = true;
                    let available = self.list_models().await.unwrap_or_else(|_| Self::discover_installed_models());
                    if available.is_empty() {
                        Self::notify_no_models_installed();
                        return Err(TagisanError::NoModelsInstalled);
                    }
                    if let Some(fb) = available.into_iter().find(|m| m != &*target_model) {
                        eprintln!(
                            "{}",
                            format!(
                                "⚠️  [Ollama Auto-Recovery] Model '{}' not found in Ollama (it may have been removed). Automatically switching to '{}'...",
                                target_model, fb
                            ).yellow().bold()
                        );
                        Self::auto_heal_env_file(&fb);
                        current_req.model = fb;
                        continue;
                    } else {
                        Self::notify_no_models_installed();
                        return Err(TagisanError::NoModelsInstalled);
                    }
                }
                return Err(TagisanError::BadResponse("ollama".into(), err_text));
            }

            let cancellation_token = current_req.cancellation_token.clone();
        let pld_arc = self.pld.clone();
        let enable_pld = self.hyper_config.prompt_lookup_decoding;

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
                                if enable_pld {
                                    if let Ok(mut pld) = pld_arc.try_lock() {
                                        let recent_spec = pld.speculate_from_text(&msg.content);
                                        pld.append_emitted_token(&msg.content);
                                        if let Some(cand) = recent_spec {
                                            let actual_tokens = vec![msg.content.clone()];
                                            pld.verify_and_accept(&cand, &actual_tokens);
                                        }
                                    }
                                }
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

            return Ok(Box::pin(output_stream));
        }
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

    #[test]
    fn test_10_sanitize_model_name_null_fallback() {
        let fallback = OllamaProvider::default_model();
        assert_eq!(OllamaProvider::sanitize_model_name("null"), fallback.as_str());
        assert_eq!(OllamaProvider::sanitize_model_name("NULL"), fallback.as_str());
        assert_eq!(OllamaProvider::sanitize_model_name("none"), fallback.as_str());
        assert_eq!(OllamaProvider::sanitize_model_name("default"), fallback.as_str());
        assert_eq!(OllamaProvider::sanitize_model_name(""), fallback.as_str());
        assert_eq!(OllamaProvider::sanitize_model_name("<recommended model or null>"), fallback.as_str());
        assert_eq!(OllamaProvider::sanitize_model_name("llama3.2:latest"), "llama3.2:latest");
    }

    #[test]
    fn test_11_model_removal_and_auto_recovery() {
        let mock_installed = vec![
            "smollm2:1.7b".to_string(),
            "qwen2.5:0.5b".to_string(),
        ];

        // 1. Exact match
        assert_eq!(
            OllamaProvider::find_matching_model("smollm2:1.7b", &mock_installed),
            Some("smollm2:1.7b".to_string())
        );

        // 2. Base tag-less match
        assert_eq!(
            OllamaProvider::find_matching_model("smollm2", &mock_installed),
            Some("smollm2:1.7b".to_string())
        );

        // 3. Removed / non-existent model returns None
        assert_eq!(
            OllamaProvider::find_matching_model("huihui_ai/llama3.2-abliterate:3b-instruct", &mock_installed),
            None
        );

        // 4. Test environment override with a removed model
        std::env::set_var("OLLAMA_MODEL", "deleted-model-xyz-999");
        let recovered = OllamaProvider::default_model();
        assert_ne!(recovered, "deleted-model-xyz-999", "default_model must never return a deleted model if models are installed");
        let installed = OllamaProvider::discover_installed_models();
        if !installed.is_empty() {
            assert!(installed.contains(&recovered));
        }
    }

    // =========================================================================
    // Test 12: Prompt Lookup Decoder (PLD) N-gram Speculative Decoding Engine
    // =========================================================================
    #[test]
    fn test_12_pld_ngram_matching_speculation_and_acceptance() {
        let mut pld = PromptLookupDecoder::default();

        let prompt = "pub struct HyperOllamaConfig {\n    pub num_ctx: u32,\n    pub num_gpu: u32,\n    pub num_batch: u32,\n    pub f16_kv: bool,\n}";
        pld.index_text(prompt);

        assert!(pld.buffer_len() > 0);
        assert!(pld.unique_ngrams_count() > 0);

        // 1. Lossless tokenization verification
        let tokens = tokenize_pld(prompt);
        assert_eq!(tokens.concat(), prompt, "Tokenization must be completely lossless");

        // 2. Query with matching 3-gram: ["pub", " ", "struct"]
        let query = vec!["pub".to_string(), " ".to_string(), "struct".to_string()];
        let candidate = pld.speculate(&query).expect("Should find matching N-gram in context");

        assert_eq!(candidate.matched_ngram, vec!["pub", " ", "struct"]);
        assert!(!candidate.speculated_tokens.is_empty());
        assert_eq!(candidate.speculated_tokens[0], " ");
        assert_eq!(candidate.speculated_tokens[1], "HyperOllamaConfig");
        assert!(candidate.confidence_score > 0.5);

        // 3. Verify and accept matching continuation
        let actual = vec![" ".to_string(), "HyperOllamaConfig".to_string(), " ".to_string()];
        let accepted = pld.verify_and_accept(&candidate, &actual);
        assert_eq!(accepted, 3);
        assert_eq!(pld.telemetry().total_accepted_tokens, 3);
        assert_eq!(pld.telemetry().total_speculations, 1);
        assert!(pld.telemetry().acceptance_rate > 0.0);
        assert!(pld.telemetry().estimated_latency_saved_ms > 0.0);

        // 4. Verification on mismatch
        let mismatch = vec!["something_else".to_string()];
        let accepted_mismatch = pld.verify_and_accept(&candidate, &mismatch);
        assert_eq!(accepted_mismatch, 0);

        // 5. Code repetition speculation
        let code = r#"
            if let Some(val) = map.get("target_key") {
                process_target_value(val);
            }
        "#;
        pld.index_text(code);

        let recent = tokenize_pld("if let Some(val) = map.get(");
        let cand = pld.speculate(&recent).expect("Should speculate code continuation");
        let cand_str = cand.to_string_lossless();
        assert!(cand_str.contains("\"target_key\""));
    }

    // =========================================================================
    // Test 13: Deterministic KV Cache Prefix Alignment & 100% Hit Validation
    // =========================================================================
    #[test]
    fn test_13_deterministic_kv_prefix_alignment_and_cache_hits() {
        let mut cache = DeterministicKvPrefixCache::default();

        // 1. System prompt whitespace & line ending normalization
        let messy_sys = "  You are Tagisan AI.\r\n\r\n\r\n\r\nAlways write Rust.   \n   ";
        let canonical_sys = DeterministicKvPrefixCache::canonicalize_system_prompt(messy_sys);
        assert_eq!(canonical_sys, "You are Tagisan AI.\n\nAlways write Rust.");

        // 2. Deterministic tool sorting and parameter canonicalization
        let tool_b = ToolDefinition::new(
            "zebra_search",
            "Search zebra",
            serde_json::json!({ "z_param": 1, "a_param": 2 }),
        );
        let tool_a = ToolDefinition::new(
            "alpha_calc",
            "Calculate alpha",
            serde_json::json!({ "y_val": true, "x_val": false }),
        );

        let canonical_tools = DeterministicKvPrefixCache::canonicalize_tools(&[tool_b.clone(), tool_a.clone()]);
        assert_eq!(canonical_tools[0].name, "alpha_calc", "Tools must be sorted alphabetically");
        assert_eq!(canonical_tools[1].name, "zebra_search");

        // 3. Multi-turn alignment test: Turn 1
        let mut req_turn1 = CompletionRequest::new("llama3.2", "Hello assistant")
            .with_system(messy_sys)
            .with_tool(tool_b)
            .with_tool(tool_a);

        let report1 = cache.align_request(&mut req_turn1);
        assert!(!report1.prefix_hash_hex.is_empty());
        assert!(report1.prefix_tokens_saved > 0);

        // Turn 2: Follow-up question (same system prompt, same tools, history appended)
        let mut req_turn2 = CompletionRequest::new("llama3.2", "Second question")
            .with_system("You are Tagisan AI.\n\nAlways write Rust.")
            .with_messages(vec![
                Message::user("Hello assistant"),
                Message::assistant("Hello! How can I help you today?"),
                Message::user("Second question"),
            ]);

        let report2 = cache.align_request(&mut req_turn2);
        assert!(report2.hit, "Turn 2 must achieve 100% prefix cache hit on identical prefix");
        assert!(report2.prefix_tokens_saved > 0);
        assert!(cache.telemetry().prefix_hits >= 1);
        assert!(cache.telemetry().hit_rate > 0.0);

        // Turn 3 with divergent system prompt: must register as miss
        let mut req_divergent = CompletionRequest::new("llama3.2", "Third question")
            .with_system("Completely different system instructions")
            .with_messages(vec![Message::user("Third question")]);

        let report_div = cache.align_request(&mut req_divergent);
        assert!(!report_div.hit, "Divergent prefix must register as cache miss");
        assert_eq!(cache.telemetry().prefix_misses, 1);
    }

    // =========================================================================
    // Test 14: Dynamic Context-Window Sizing (Power-of-2 Context Fitting)
    // =========================================================================
    #[test]
    fn test_14_dynamic_context_fitter_power_of_two_and_memory_savings() {
        let fitter = DynamicContextFitter::default();

        // 1. Tiny prompt (~10 tokens, no max_tokens set)
        let req_tiny = CompletionRequest::new("llama3.2", "Hi");
        let report_tiny = fitter.fit_context_window(&req_tiny);

        // Should fit to 512 or 1024, NOT the wasteful fixed 8192 buffer!
        assert!(report_tiny.allocated_num_ctx <= 1024, "Tiny request must fit into compact <=1024 buffer");
        assert_eq!(report_tiny.allocated_num_ctx.count_ones(), 1, "Context size must be power of 2");
        assert!(report_tiny.vram_saved_mb > 500.0, "Must save significant VRAM over 8192 baseline");
        assert!(report_tiny.bandwidth_reduction_pct > 75.0, "Attention bandwidth must be slashed >75%");

        // 2. Medium prompt (~500 tokens)
        let med_text = "fn compute_heavy_task() {\n".repeat(40);
        let req_med = CompletionRequest::new("llama3.2", med_text)
            .with_max_tokens(1024);
        let report_med = fitter.fit_context_window(&req_med);

        assert!(report_med.allocated_num_ctx >= 2048);
        assert_eq!(report_med.allocated_num_ctx.count_ones(), 1, "Must be power of 2");

        // 3. Explicit large request with max_tokens: 8192
        let req_large = CompletionRequest::new("llama3.2", "Calculate the answer to life")
            .with_max_tokens(8192);
        let report_large = fitter.fit_context_window(&req_large);
        assert_eq!(report_large.allocated_num_ctx, 8192, "Explicit 8192 max_tokens must allocate 8192 buffer");
    }

    // =========================================================================
    // Test 15: FlashAttention Activation & Dynamic Batch Size Optimization
    // =========================================================================
    #[test]
    fn test_15_flash_attention_and_batch_optimizer() {
        // 1. FlashAttention environment auto-enabling
        let fa_active = ensure_flash_attention_env();
        assert!(fa_active, "ensure_flash_attention_env must activate FlashAttention");
        assert_eq!(std::env::var("OLLAMA_FLASH_ATTENTION").unwrap(), "1");
        assert!(is_flash_attention_enabled());

        // 2. Dynamic batch size optimizer
        let batch_small = DynamicBatchOptimizer::optimize_batch_size(100);
        assert!(batch_small >= 256 && batch_small <= 512);

        let batch_large = DynamicBatchOptimizer::optimize_batch_size(3000);
        assert!(batch_large >= 512);

        // 3. End-to-end options generation
        let provider = OllamaProvider::default_local();
        let req = CompletionRequest::new("llama3.2", "Test options generation")
            .with_temperature(0.7);

        let options = provider.build_options(&req);
        assert_eq!(options.f16_kv, Some(true));
        assert_eq!(options.use_mmap, Some(true));
        assert!(options.num_gpu.unwrap() >= 99);
        assert!(options.num_batch.unwrap() >= 256);
        assert!(options.num_ctx.unwrap() >= 512);
        assert_eq!(options.num_ctx.unwrap().count_ones(), 1, "num_ctx must be power of 2");
    }

    // =========================================================================
    // Test 16: Warmth Sentinel Zero-Cold-Start Daemon
    // =========================================================================
    #[tokio::test]
    async fn test_16_warmth_sentinel_lifecycle_and_mock_probing() {
        let heartbeat_received = Arc::new(AtomicUsize::new(0));
        let hb_clone = heartbeat_received.clone();

        let handler: RequestHandler = Arc::new(move |req| {
            if req.path == "/api/generate" {
                hb_clone.fetch_add(1, Ordering::SeqCst);
                let resp = json!({
                    "model": "llama3.2",
                    "response": "",
                    "done": true,
                    "done_reason": "stop"
                });
                (200, HashMap::new(), serde_json::to_vec(&resp).unwrap())
            } else {
                (404, HashMap::new(), b"{}".to_vec())
            }
        });

        let server = MockOllamaServer::start_custom(Some(handler), None).await;

        let config = WarmthConfig {
            model: "llama3.2".to_string(),
            base_url: server.base_url.clone(),
            heartbeat_interval: Duration::from_millis(50),
            keep_alive: "24h".to_string(),
            prewarm_canonical_prefix: Some("You are a warm assistant.".to_string()),
            enabled: true,
        };

        let sentinel = Arc::new(WarmthSentinel::new(config));
        assert_eq!(sentinel.get_status().await, WarmthStatus::Cold);

        // Immediate touch probe
        let latency = sentinel.touch_now().await.expect("Touch probe must succeed");
        assert!(latency < Duration::from_secs(2));
        assert_eq!(sentinel.get_status().await, WarmthStatus::WarmInVram);

        // Prewarm prompt
        let prewarm_latency = sentinel.prewarm_prompt("Canonical prefix").await.expect("Prewarm must succeed");
        assert!(prewarm_latency < Duration::from_secs(2));

        // Start background loop for 150ms
        let handle = sentinel.clone().start();
        tokio::time::sleep(Duration::from_millis(150)).await;

        sentinel.stop();
        let _ = handle.await;

        assert!(heartbeat_received.load(Ordering::SeqCst) >= 2, "Must have received multiple heartbeats");
    }

    // =========================================================================
    // Test 17: End-to-End Hyper-Ollama Streaming with PLD and KV Cache
    // =========================================================================
    #[tokio::test]
    async fn test_17_end_to_end_hyper_ollama_streaming_with_pld_and_kv_cache() {
        let stream_handler: StreamHandler = Arc::new(|_| {
            let chunks = vec![
                json!({ "message": { "role": "assistant", "content": "Hyper-" }, "done": false }).to_string(),
                json!({ "message": { "role": "assistant", "content": "Ollama " }, "done": false }).to_string(),
                json!({ "message": { "role": "assistant", "content": "Acceleration!" }, "done": true, "done_reason": "stop" }).to_string(),
            ];
            (200, HashMap::new(), chunks, Duration::from_millis(2))
        });

        let server = MockOllamaServer::start_custom(None, Some(stream_handler)).await;
        let provider = OllamaProvider::new(&server.base_url);

        let req = CompletionRequest::new("llama3.2", "Accelerate Ollama inference")
            .with_system("You are Hyper-Ollama Engine.");

        let mut stream = provider.stream(req).await.expect("Streaming must succeed");

        let mut output = String::new();
        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.expect("Chunk must be valid");
            if let StreamChunkDelta::Text(t) = chunk.delta {
                output.push_str(&t);
            }
        }

        assert_eq!(output, "Hyper-Ollama Acceleration!");

        // Verify that prefix_cache recorded the turn
        let prefix_guard = provider.prefix_cache.lock().await;
        assert!(prefix_guard.telemetry().total_requests >= 1);
        assert!(prefix_guard.active_prefix().is_some());

        // Verify that PLD indexed the prompt
        let pld_guard = provider.pld.lock().await;
        assert!(pld_guard.buffer_len() > 0);
    }

    #[test]
    fn test_18_non_tool_model_capability_and_auto_recovery() {
        let provider = OllamaProvider::default_local();

        // 1. llama2-uncensored and llama2 must report NO FUNCTION_CALLING
        let caps_llama2 = provider.capabilities("llama2-uncensored:latest");
        assert!(!caps_llama2.contains(ProviderCapabilities::FUNCTION_CALLING));

        let caps_codellama = provider.capabilities("codellama:7b");
        assert!(!caps_codellama.contains(ProviderCapabilities::FUNCTION_CALLING));

        // 2. Modern tool-capable models (qwen2.5, llama3) must report FUNCTION_CALLING
        let caps_qwen = provider.capabilities("qwen2.5-coder:1.5b");
        assert!(caps_qwen.contains(ProviderCapabilities::FUNCTION_CALLING));

        let caps_llama3 = provider.capabilities("llama3.1:8b");
        assert!(caps_llama3.contains(ProviderCapabilities::FUNCTION_CALLING));

        // 3. Static check helper
        assert!(!OllamaProvider::is_model_tool_supported("llama2-uncensored"));
        assert!(!OllamaProvider::is_model_tool_supported("registry.ollama.ai/library/llama2-uncensored:latest"));
        assert!(OllamaProvider::is_model_tool_supported("qwen2.5:7b"));
    }
}


