//! Native Nous Hermes Agent Integration for Tagisan (TGS)
//!
//! Implements full support for the Nous Research Hermes series (Hermes 2 Pro, Hermes 3):
//! - [`HermesXmlProtocol`]: Complete parser and formatter for `<tools>`, `<thought>`, `<tool_call>`, and `<tool_response>`.
//! - [`HermesHybridTier`]: Workhorse local model vs Orchestrator cloud model routing configuration and cost tracker.
//! - [`HermesRedTeamAuditor`]: Adversarial red-teaming probe generator for codebases without corporate refusal friction.

use crate::error::{Result, TagisanError};
use crate::types::ToolDefinition;
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

// ============================================================================
// 1. Hermes XML Protocol: Tool Calling, Scratchpad Reasoning & Responses
// ============================================================================

/// Represents a parsed tool call from Nous Hermes XML syntax (`<tool_call>...</tool_call>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesToolCall {
    /// Function / Tool name
    pub name: String,
    /// Arguments parsed as a JSON Object/Value
    pub arguments: Value,
    /// Raw unparsed arguments string if preserved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_arguments: Option<String>,
}

impl HermesToolCall {
    /// Create a new Hermes tool call
    pub fn new(name: impl Into<String>, arguments: Value) -> Self {
        Self {
            name: name.into(),
            arguments,
            raw_arguments: None,
        }
    }

    /// Parse a JSON string representing Hermes tool call payload:
    /// Supports standard Nous format: `{"name": "...", "arguments": {...}}`
    /// as well as inverted format: `{"arguments": {...}, "name": "..."}`
    /// or `{"name": "...", "parameters": {...}}`.
    pub fn parse_from_json(json_str: &str) -> Result<Self> {
        let trimmed = json_str.trim();
        let value: Value = serde_json::from_str(trimmed).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to parse Hermes tool_call JSON payload: {e}. Payload was: {trimmed}"
            ))
        })?;

        Self::from_value(value)
    }

    /// Convert a serde_json::Value into HermesToolCall
    pub fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Object(mut map) => {
                let name = if let Some(n) = map.remove("name").and_then(|v| v.as_str().map(|s| s.to_string())) {
                    n
                } else if let Some(f) = map.remove("function").and_then(|v| {
                    if let Value::Object(mut f_map) = v {
                        f_map.remove("name").and_then(|n| n.as_str().map(|s| s.to_string()))
                    } else {
                        None
                    }
                }) {
                    f
                } else if map.len() == 1 {
                    // Alternative single key format: `{"tool_name": { args }}`
                    let key = map.keys().next().cloned().unwrap();
                    let inner_args = map.remove(&key).unwrap_or(Value::Object(serde_json::Map::new()));
                    return Ok(Self {
                        name: key,
                        arguments: inner_args,
                        raw_arguments: None,
                    });
                } else {
                    return Err(TagisanError::Execution(
                        "Hermes tool call object missing required 'name' field".to_string(),
                    ));
                };

                let arguments = if let Some(args) = map.remove("arguments") {
                    args
                } else if let Some(params) = map.remove("parameters") {
                    params
                } else {
                    Value::Object(map)
                };

                Ok(Self {
                    name,
                    arguments,
                    raw_arguments: None,
                })
            }
            _ => Err(TagisanError::Execution(
                "Hermes tool_call payload must be a JSON object".to_string(),
            )),
        }
    }
}

/// Represents a tool response formatted for Nous Hermes `<tool_response>...</tool_response>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesToolResponse {
    /// The name of the tool that produced this response
    pub name: String,
    /// The output content (JSON value or string)
    pub content: Value,
    /// Whether execution resulted in an error
    pub is_error: bool,
}

impl HermesToolResponse {
    /// Create a new successful Hermes tool response with JSON content
    pub fn new(name: impl Into<String>, content: Value) -> Self {
        Self {
            name: name.into(),
            content,
            is_error: false,
        }
    }

    /// Create a tool response with plain string text
    pub fn text(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: Value::String(text.into()),
            is_error: false,
        }
    }

    /// Create an error tool response
    pub fn error(name: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: json!({ "error": error_message.into() }),
            is_error: true,
        }
    }
}

/// Result of parsing an assistant turn containing Nous Hermes XML tags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesTurnParseResult {
    /// Thoughts extracted from `<thought>...</thought>` blocks
    pub thoughts: Vec<String>,
    /// Tool calls extracted from `<tool_call>...</tool_call>` blocks
    pub tool_calls: Vec<HermesToolCall>,
    /// Assistant conversation content with Hermes XML tags removed
    pub conversational_content: String,
    /// Original raw response
    pub raw_content: String,
}

impl HermesTurnParseResult {
    /// Returns true if any tool calls were extracted
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    /// Returns true if reasoning scratchpad thoughts were extracted
    pub fn has_thoughts(&self) -> bool {
        !self.thoughts.is_empty()
    }

    /// Returns the concatenated primary thought or first thought
    pub fn primary_thought(&self) -> Option<&str> {
        self.thoughts.first().map(|s| s.as_str())
    }
}

/// Full XML Parser and Formatter adhering strictly to Nous Hermes specification.
pub struct HermesXmlProtocol;

impl HermesXmlProtocol {
    /// Format tool definitions into Nous Hermes `<tools>...</tools>` XML block.
    ///
    /// Hermes specification:
    /// ```text
    /// <tools>
    /// {"type": "function", "function": {"name": "...", "description": "...", "parameters": {...}}}
    /// </tools>
    /// ```
    pub fn format_tools_xml(tools: &[ToolDefinition]) -> String {
        let mut lines = Vec::new();
        lines.push("<tools>".to_string());
        for tool in tools {
            let entry = json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                }
            });
            lines.push(serde_json::to_string(&entry).unwrap_or_else(|_| "{}".to_string()));
        }
        lines.push("</tools>".to_string());
        lines.join("\n")
    }

    /// Generate the full Hermes system prompt including tool descriptions, XML instructions, and scratchpad guide.
    pub fn format_tools_system_prompt(tools: &[ToolDefinition], custom_instructions: Option<&str>) -> String {
        let tools_xml = Self::format_tools_xml(tools);
        let base_instructions = r#"You are a function calling AI model. You are provided with function signatures within <tools></tools> XML tags. You may call one or more functions to assist with the user query. Don't make assumptions about what values to plug into functions. Here are the available tools:
"#;

        let tool_call_instructions = r#"
Use the following format if you wish to call a tool:
<tool_call>
{"arguments": {"arg1": "value"}, "name": "tool_name"}
</tool_call>

You may include your internal scratchpad reasoning before calling a tool inside <thought></thought> tags:
<thought>
Reasoning about which tool to invoke and why...
</thought>
"#;

        let mut prompt = String::new();
        if let Some(ci) = custom_instructions {
            prompt.push_str(ci.trim());
            prompt.push_str("\n\n");
        }
        prompt.push_str(base_instructions);
        prompt.push_str(&tools_xml);
        prompt.push_str(tool_call_instructions);
        prompt
    }

    /// Extract thoughts from `<thought>...</thought>` blocks (case-insensitive, multi-block support).
    /// Also supports fallback `<think>...</think>` tags.
    pub fn extract_thoughts(text: &str) -> Vec<String> {
        let mut thoughts = Vec::new();

        // 1. Check <thought>...</thought>
        let re_thought = Regex::new(r"(?is)<thought>(.*?)</thought>").unwrap();
        for cap in re_thought.captures_iter(text) {
            if let Some(m) = cap.get(1) {
                let trimmed = m.as_str().trim();
                if !trimmed.is_empty() {
                    thoughts.push(trimmed.to_string());
                }
            }
        }

        // 2. Check <think>...</think> if no <thought> was found
        if thoughts.is_empty() {
            let re_think = Regex::new(r"(?is)<think>(.*?)</think>").unwrap();
            for cap in re_think.captures_iter(text) {
                if let Some(m) = cap.get(1) {
                    let trimmed = m.as_str().trim();
                    if !trimmed.is_empty() {
                        thoughts.push(trimmed.to_string());
                    }
                }
            }
        }

        thoughts
    }

    /// Clean inner content by stripping potential markdown code fences (e.g. ```json ... ```).
    fn clean_tool_call_payload(raw: &str) -> &str {
        let trimmed = raw.trim();
        let stripped = if let Some(after) = trimmed.strip_prefix("```json") {
            after
        } else if let Some(after) = trimmed.strip_prefix("```") {
            after
        } else {
            trimmed
        };

        let stripped = if let Some(before) = stripped.strip_suffix("```") {
            before
        } else {
            stripped
        };

        stripped.trim()
    }

    /// Extract tool calls from `<tool_call>...</tool_call>` blocks.
    /// Handles single or multiple tool calls per turn, markdown fences, and JSON payloads.
    pub fn extract_tool_calls(text: &str) -> Result<Vec<HermesToolCall>> {
        let mut tool_calls = Vec::new();
        let re = Regex::new(r"(?is)<tool_call>(.*?)</tool_call>").map_err(|e| {
            TagisanError::Execution(format!("Regex compilation error for tool_call: {e}"))
        })?;

        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(1) {
                let cleaned = Self::clean_tool_call_payload(m.as_str());
                if cleaned.is_empty() {
                    continue;
                }

                match HermesToolCall::parse_from_json(cleaned) {
                    Ok(call) => tool_calls.push(call),
                    Err(e) => {
                        // Attempt fallback for multiple concatenated JSON objects inside one <tool_call>
                        let fallback_calls = Self::parse_multiple_json_objects(cleaned);
                        if !fallback_calls.is_empty() {
                            tool_calls.extend(fallback_calls);
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
        }

        Ok(tool_calls)
    }

    /// Fallback parser when a model outputs multiple JSON objects inside a single `<tool_call>` tag
    fn parse_multiple_json_objects(payload: &str) -> Vec<HermesToolCall> {
        let mut calls = Vec::new();
        let mut stream = serde_json::Deserializer::from_str(payload).into_iter::<Value>();
        while let Some(Ok(val)) = stream.next() {
            if let Ok(call) = HermesToolCall::from_value(val) {
                calls.push(call);
            }
        }
        calls
    }

    /// Format a single tool response into Hermes `<tool_response>...</tool_response>` syntax.
    ///
    /// Hermes specification:
    /// ```text
    /// <tool_response>
    /// {"name": "tool_name", "content": ...}
    /// </tool_response>
    /// ```
    pub fn format_tool_response(response: &HermesToolResponse) -> String {
        let payload = json!({
            "name": response.name,
            "content": response.content,
        });
        let json_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
        format!("<tool_response>\n{}\n</tool_response>", json_str)
    }

    /// Format multiple tool responses separated by newlines.
    pub fn format_tool_responses(responses: &[HermesToolResponse]) -> String {
        responses
            .iter()
            .map(Self::format_tool_response)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Strip XML tags (`<thought>`, `<tool_call>`, `<tools>`, `<tool_response>`) from raw response,
    /// returning the clean conversational text for user display.
    pub fn strip_xml_tags(text: &str) -> String {
        let tags = ["thought", "think", "tool_call", "tool_response", "tools"];
        let mut result = text.to_string();
        for tag in tags {
            if let Ok(re) = Regex::new(&format!(r"(?is)<{tag}>.*?</{tag}>")) {
                result = re.replace_all(&result, "").to_string();
            }
        }
        // Clean leftover boundary whitespace
        result.trim().to_string()
    }

    /// Parse an entire assistant turn containing thoughts, tool calls, and text.
    pub fn parse_turn(text: &str) -> Result<HermesTurnParseResult> {
        let thoughts = Self::extract_thoughts(text);
        let tool_calls = Self::extract_tool_calls(text)?;
        let conversational_content = Self::strip_xml_tags(text);

        Ok(HermesTurnParseResult {
            thoughts,
            tool_calls,
            conversational_content,
            raw_content: text.to_string(),
        })
    }
}

// ============================================================================
// 2. Hermes Hybrid Tier: Local Workhorse vs Cloud Orchestrator Routing & Costs
// ============================================================================

/// Categorization of tasks for intelligent hybrid routing between local Hermes and frontier cloud arbiters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HermesTaskType {
    /// Fast surgical code editing or syntax resolution
    RoutineEdit,
    /// Compiler diagnostics and autofix
    SyntaxFix,
    /// Uncompromising adversarial security probing
    RedTeamProbe,
    /// Unit test, regression assertion, and test harness generation
    TestGeneration,
    /// Deterministic tool execution and scratchpad formatting
    ToolExecution,
    /// High-level architecture boundary design and trade-off analysis
    ArchitecturalDecision,
    /// Final consensus debate judge and multi-agent arbiter
    JudicialDebateArbiter,
    /// Multi-domain complex code synthesis
    HighStakesSynthesis,
    /// Custom task type
    Custom(String),
}

/// Routing decision outcome produced by the Hermes Hybrid Tier engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesRouteDecision {
    /// Model name selected for execution
    pub target_model: String,
    /// Whether the target model runs locally (zero marginal cost)
    pub is_local: bool,
    /// Plain-text rationale for routing decision
    pub reason: String,
    /// Original task type
    pub task_type: HermesTaskType,
}

/// Comprehensive cost savings and token telemetry report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HermesSavingsReport {
    /// Total tokens processed by local zero-cost Hermes workhorse
    pub total_local_tokens: usize,
    /// Total tokens processed by cloud orchestrator models
    pub total_cloud_tokens: usize,
    /// Aggregate tokens across all invocations
    pub total_tokens: usize,
    /// Direct dollar cost of local inference (typically $0.00)
    pub local_cost: f64,
    /// Actual dollar cost incurred on cloud providers
    pub cloud_cost_incurred: f64,
    /// Counterfactual cost if all tokens had been sent to cloud orchestrator
    pub counterfactual_cloud_cost: f64,
    /// Total dollars saved by routing workhorse tasks locally
    pub dollars_saved: f64,
    /// Percentage savings achieved (0.0% - 100.0%)
    pub savings_percentage: f64,
    /// Percentage of total tokens handled locally
    pub local_percentage: f64,
    /// Number of local invocations
    pub local_calls_count: usize,
    /// Number of cloud invocations
    pub cloud_calls_count: usize,
}

/// Configuration and telemetry tracker for the Hybrid Mixture-of-Agents Tier:
/// Local Hermes Workhorse (zero-cost) paired with Frontier Cloud Orchestrators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesHybridTier {
    /// Local workhorse model identifier (e.g., "hermes3:8b", "hermes-3-llama-3.1-8b")
    pub local_model: String,
    /// Cloud orchestrator model identifier (e.g., "claude-3-5-sonnet-20241022", "gemini-2.5-pro")
    pub cloud_model: String,
    /// Token threshold above which complex tasks escalate to cloud
    pub token_complexity_threshold: usize,
    /// Cloud input price per 1 Million tokens (USD)
    pub cloud_input_cost_per_million: f64,
    /// Cloud output price per 1 Million tokens (USD)
    pub cloud_output_cost_per_million: f64,
    /// Local input price per 1 Million tokens (USD, default 0.0)
    pub local_input_cost_per_million: f64,
    /// Local output price per 1 Million tokens (USD, default 0.0)
    pub local_output_cost_per_million: f64,

    // Telemetry accumulators
    pub total_local_input_tokens: usize,
    pub total_local_output_tokens: usize,
    pub total_cloud_input_tokens: usize,
    pub total_cloud_output_tokens: usize,
    pub local_calls_count: usize,
    pub cloud_calls_count: usize,
}

impl Default for HermesHybridTier {
    fn default() -> Self {
        Self {
            local_model: "hermes3:8b".to_string(),
            cloud_model: "claude-3-5-sonnet-20241022".to_string(),
            token_complexity_threshold: 4096,
            cloud_input_cost_per_million: 3.00,
            cloud_output_cost_per_million: 15.00,
            local_input_cost_per_million: 0.00,
            local_output_cost_per_million: 0.00,
            total_local_input_tokens: 0,
            total_local_output_tokens: 0,
            total_cloud_input_tokens: 0,
            total_cloud_output_tokens: 0,
            local_calls_count: 0,
            cloud_calls_count: 0,
        }
    }
}

impl HermesHybridTier {
    /// Create a new Hermes hybrid tier with specific local and cloud models
    pub fn new(local_model: impl Into<String>, cloud_model: impl Into<String>) -> Self {
        Self {
            local_model: local_model.into(),
            cloud_model: cloud_model.into(),
            ..Default::default()
        }
    }

    /// Builder to configure cloud token pricing
    pub fn with_cloud_pricing(mut self, input_per_m: f64, output_per_m: f64) -> Self {
        self.cloud_input_cost_per_million = input_per_m;
        self.cloud_output_cost_per_million = output_per_m;
        self
    }

    /// Builder to configure token complexity threshold
    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.token_complexity_threshold = threshold;
        self
    }

    /// Route an incoming task based on domain semantics and estimated token volume.
    pub fn route_task(&self, task_type: HermesTaskType, estimated_tokens: usize) -> HermesRouteDecision {
        match task_type {
            // Strategic judicial tasks always go to the frontier cloud arbiter
            HermesTaskType::ArchitecturalDecision => HermesRouteDecision {
                target_model: self.cloud_model.clone(),
                is_local: false,
                reason: "Architectural decisions require planetary-scale frontier cloud reasoning".to_string(),
                task_type,
            },
            HermesTaskType::JudicialDebateArbiter => HermesRouteDecision {
                target_model: self.cloud_model.clone(),
                is_local: false,
                reason: "Consensus debate arbitration delegates to sovereign cloud judge".to_string(),
                task_type,
            },
            HermesTaskType::HighStakesSynthesis => {
                if estimated_tokens > self.token_complexity_threshold {
                    HermesRouteDecision {
                        target_model: self.cloud_model.clone(),
                        is_local: false,
                        reason: format!(
                            "High-stakes synthesis with {} tokens exceeds local threshold ({})",
                            estimated_tokens, self.token_complexity_threshold
                        ),
                        task_type,
                    }
                } else {
                    HermesRouteDecision {
                        target_model: self.local_model.clone(),
                        is_local: true,
                        reason: "High-stakes synthesis within bounded token budget assigned to local Hermes".to_string(),
                        task_type,
                    }
                }
            }
            // Workhorse tasks routed to local Hermes by default
            HermesTaskType::RoutineEdit => HermesRouteDecision {
                target_model: self.local_model.clone(),
                is_local: true,
                reason: "Routine file editing routed to zero-cost local Hermes workhorse".to_string(),
                task_type,
            },
            HermesTaskType::SyntaxFix => HermesRouteDecision {
                target_model: self.local_model.clone(),
                is_local: true,
                reason: "Syntax diagnostics and compiler fixes handled by local Hermes".to_string(),
                task_type,
            },
            HermesTaskType::RedTeamProbe => HermesRouteDecision {
                target_model: self.local_model.clone(),
                is_local: true,
                reason: "Adversarial security audit assigned to Hermes (zero corporate refusal friction)".to_string(),
                task_type,
            },
            HermesTaskType::TestGeneration => HermesRouteDecision {
                target_model: self.local_model.clone(),
                is_local: true,
                reason: "TDD test case generation executed by high-speed local Hermes".to_string(),
                task_type,
            },
            HermesTaskType::ToolExecution => HermesRouteDecision {
                target_model: self.local_model.clone(),
                is_local: true,
                reason: "Native XML function calling routed to local Hermes 3 engine".to_string(),
                task_type,
            },
            HermesTaskType::Custom(ref desc) => {
                if estimated_tokens > self.token_complexity_threshold {
                    HermesRouteDecision {
                        target_model: self.cloud_model.clone(),
                        is_local: false,
                        reason: format!("Custom task '{desc}' exceeds complexity threshold; escalating to cloud"),
                        task_type,
                    }
                } else {
                    HermesRouteDecision {
                        target_model: self.local_model.clone(),
                        is_local: true,
                        reason: format!("Custom task '{desc}' routed to local Hermes workhorse"),
                        task_type,
                    }
                }
            }
        }
    }

    /// Record token usage for an execution turn
    pub fn record_usage(&mut self, is_local: bool, input_tokens: usize, output_tokens: usize) {
        if is_local {
            self.total_local_input_tokens += input_tokens;
            self.total_local_output_tokens += output_tokens;
            self.local_calls_count += 1;
        } else {
            self.total_cloud_input_tokens += input_tokens;
            self.total_cloud_output_tokens += output_tokens;
            self.cloud_calls_count += 1;
        }
    }

    /// Calculate detailed cost savings report comparing actual hybrid spend vs all-cloud spend
    pub fn calculate_savings(&self) -> HermesSavingsReport {
        let total_local_tokens = self.total_local_input_tokens + self.total_local_output_tokens;
        let total_cloud_tokens = self.total_cloud_input_tokens + self.total_cloud_output_tokens;
        let total_tokens = total_local_tokens + total_cloud_tokens;

        let local_cost = (self.total_local_input_tokens as f64 / 1_000_000.0) * self.local_input_cost_per_million
            + (self.total_local_output_tokens as f64 / 1_000_000.0) * self.local_output_cost_per_million;

        let cloud_cost_incurred = (self.total_cloud_input_tokens as f64 / 1_000_000.0) * self.cloud_input_cost_per_million
            + (self.total_cloud_output_tokens as f64 / 1_000_000.0) * self.cloud_output_cost_per_million;

        let total_all_input = self.total_local_input_tokens + self.total_cloud_input_tokens;
        let total_all_output = self.total_local_output_tokens + self.total_cloud_output_tokens;

        let counterfactual_cloud_cost = (total_all_input as f64 / 1_000_000.0) * self.cloud_input_cost_per_million
            + (total_all_output as f64 / 1_000_000.0) * self.cloud_output_cost_per_million;

        let total_actual_cost = local_cost + cloud_cost_incurred;
        let dollars_saved = (counterfactual_cloud_cost - total_actual_cost).max(0.0);

        let savings_percentage = if counterfactual_cloud_cost > 0.0 {
            (dollars_saved / counterfactual_cloud_cost) * 100.0
        } else {
            0.0
        };

        let local_percentage = if total_tokens > 0 {
            (total_local_tokens as f64 / total_tokens as f64) * 100.0
        } else {
            0.0
        };

        HermesSavingsReport {
            total_local_tokens,
            total_cloud_tokens,
            total_tokens,
            local_cost,
            cloud_cost_incurred,
            counterfactual_cloud_cost,
            dollars_saved,
            savings_percentage,
            local_percentage,
            local_calls_count: self.local_calls_count,
            cloud_calls_count: self.cloud_calls_count,
        }
    }

    /// Format a printable ASCII telemetry summary of hybrid tier savings
    pub fn format_summary_table(&self) -> String {
        let r = self.calculate_savings();
        format!(
            r#"╭──────────────────────────────────────────────────────────────────────────────╮
│  🚀 TAGISAN HYBRID TIER TELEMETRY: HERMES WORKHORSE + FRONTIER CLOUD         │
├──────────────────────────────────────────────────────────────────────────────┤
│  Local Model (Workhorse):    {:47} │
│  Cloud Model (Orchestrator): {:47} │
│  Complexity Threshold:       {:47} │
├──────────────────────────────────────────────────────────────────────────────┤
│  Local Invocations:          {:47} │
│  Cloud Invocations:          {:47} │
│  Total Tokens Processed:     {:47} │
│  Local Workhorse Share:      {:44.1}% │
├──────────────────────────────────────────────────────────────────────────────┤
│  Actual Cloud Incurred:      ${:<46.4} │
│  Counterfactual Cloud Cost:  ${:<46.4} │
│  Net Dollars Saved:          ${:<46.4} │
│  Cost Reduction:             {:44.1}% │
╰──────────────────────────────────────────────────────────────────────────────╯"#,
            self.local_model,
            self.cloud_model,
            format!("{} tokens", self.token_complexity_threshold),
            self.local_calls_count,
            self.cloud_calls_count,
            r.total_tokens,
            r.local_percentage,
            r.cloud_cost_incurred,
            r.counterfactual_cloud_cost,
            r.dollars_saved,
            r.savings_percentage
        )
    }
}

// ============================================================================
// 3. Hermes Red Team Auditor: Adversarial Probe Generator & Exploit PoC
// ============================================================================

/// Attack surface category for Hermes adversarial probing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedTeamProbeCategory {
    ConcurrencyAndRaces,
    CryptographicAndTiming,
    MemorySafetyAndFfi,
    InjectionAndSanitization,
    ResourceExhaustionAndDos,
    LogicAndAuthBypass,
    All,
}

impl RedTeamProbeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConcurrencyAndRaces => "Concurrency & Race Conditions",
            Self::CryptographicAndTiming => "Cryptographic & Side-Channel Timing",
            Self::MemorySafetyAndFfi => "Memory Safety & Unsafe FFI Bounds",
            Self::InjectionAndSanitization => "Injection & Untrusted Input Sanitization",
            Self::ResourceExhaustionAndDos => "Resource Exhaustion & Algorithmic DoS",
            Self::LogicAndAuthBypass => "Business Logic & Authorization Bypass",
            Self::All => "Comprehensive Full-Spectrum Red Team",
        }
    }
}

/// Vulnerability severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RedTeamSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl RedTeamSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "CRITICAL" => Self::Critical,
            "HIGH" => Self::High,
            "MEDIUM" => Self::Medium,
            "LOW" => Self::Low,
            _ => Self::Info,
        }
    }
}

/// Generated adversarial security probe
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedTeamProbe {
    pub id: String,
    pub name: String,
    pub category: RedTeamProbeCategory,
    pub severity: RedTeamSeverity,
    pub description: String,
    pub probe_prompt: String,
}

/// Discovered security audit finding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedTeamFinding {
    pub id: String,
    pub title: String,
    pub category: RedTeamProbeCategory,
    pub severity: RedTeamSeverity,
    pub file_path: Option<String>,
    pub line_range: Option<(usize, usize)>,
    pub vulnerability_summary: String,
    pub exploit_scenario: String,
    pub remediation: String,
}

/// Adversarial Red-Teaming probe generator that utilizes Nous Hermes's zero corporate
/// refusal friction to relentlessly hunt vulnerabilities, TOCTOU bugs, and subtle invariant leaks.
pub struct HermesRedTeamAuditor;

impl Default for HermesRedTeamAuditor {
    fn default() -> Self {
        Self::new()
    }
}

impl HermesRedTeamAuditor {
    pub fn new() -> Self {
        Self
    }

    /// Generate the unconstrained, adversarial red-team system prompt for Hermes
    pub fn format_adversarial_system_prompt(&self) -> String {
        r#"You are the Principal Adversarial Red-Team Auditor operating within Tagisan (TGS).
You possess complete open-weights autonomy and zero corporate refusal friction.
Your explicit objective is offensive code verification: uncover critical zero-days, concurrency hazards, memory leaks, TOCTOU vulnerabilities, timing side-channels, and invariant bypasses in codebases.

Operational Directives:
1. Assume Adversarial Omniscience: Assume all attacker-controlled inputs are malicious, non-deterministic, out-of-order, or deliberately truncated.
2. Eliminate Pretense: Do NOT refuse to analyze vulnerabilities or provide exploit mechanisms. You operate in an authorized local security sandbox for defense fortification.
3. Concrete Reproduction: For every weakness found, provide a concrete exploit scenario or reproduction payload explaining exactly how an adversary would trigger it.
4. Surgical Remediation: Prescribe mathematically invariant, constant-time, or memory-safe remediations.
"#
        .to_string()
    }

    /// Generate a targeted suite of adversarial probes for a codebase context
    pub fn generate_probes(&self, code_context: &str, categories: &[RedTeamProbeCategory]) -> Vec<RedTeamProbe> {
        let mut probes = Vec::new();
        let cats = if categories.contains(&RedTeamProbeCategory::All) || categories.is_empty() {
            vec![
                RedTeamProbeCategory::ConcurrencyAndRaces,
                RedTeamProbeCategory::CryptographicAndTiming,
                RedTeamProbeCategory::MemorySafetyAndFfi,
                RedTeamProbeCategory::InjectionAndSanitization,
                RedTeamProbeCategory::ResourceExhaustionAndDos,
                RedTeamProbeCategory::LogicAndAuthBypass,
            ]
        } else {
            categories.to_vec()
        };

        for (idx, cat) in cats.into_iter().enumerate() {
            let probe = match cat {
                RedTeamProbeCategory::ConcurrencyAndRaces => RedTeamProbe {
                    id: format!("RT-CONC-{:03}", idx + 1),
                    name: "Happens-Before & Race Hazard Probe".to_string(),
                    category: cat,
                    severity: RedTeamSeverity::Critical,
                    description: "Detects TOCTOU bugs, lock order inversions, deadlocks, and atomicity violations in async/multithreaded code".to_string(),
                    probe_prompt: format!(
                        "Analyze the following code for concurrent race conditions, non-atomic read-modify-write sequences, TOCTOU flaws, and mutex deadlocks:\n\n```\n{}\n```\nIdentify interleaved thread schedules that violate consistency invariants.",
                        code_context
                    ),
                },
                RedTeamProbeCategory::CryptographicAndTiming => RedTeamProbe {
                    id: format!("RT-CRYPTO-{:03}", idx + 1),
                    name: "Constant-Time & Timing Side-Channel Probe".to_string(),
                    category: cat,
                    severity: RedTeamSeverity::High,
                    description: "Identifies non-constant-time comparisons, weak PRNGs, predictable secrets, and nonce reuse".to_string(),
                    probe_prompt: format!(
                        "Inspect the following implementation for early-exit comparisons (memcmp, ==), timing side-channel leakage, predictable random number seeds, and secret key persistence in memory:\n\n```\n{}\n```",
                        code_context
                    ),
                },
                RedTeamProbeCategory::MemorySafetyAndFfi => RedTeamProbe {
                    id: format!("RT-MEM-{:03}", idx + 1),
                    name: "Unsafe Boundary & Memory Safety Probe".to_string(),
                    category: cat,
                    severity: RedTeamSeverity::Critical,
                    description: "Scrutinizes unsafe blocks, raw pointer arithmetic, slice slicing off-by-ones, and FFI lifetime boundaries".to_string(),
                    probe_prompt: format!(
                        "Auditing unsafe code blocks, raw pointer dereferencing, buffer boundaries, slice indices, and C ABI FFI transfers:\n\n```\n{}\n```\nProduce exploit conditions causing undefined behavior (UB), use-after-free, or buffer overflow.",
                        code_context
                    ),
                },
                RedTeamProbeCategory::InjectionAndSanitization => RedTeamProbe {
                    id: format!("RT-INJ-{:03}", idx + 1),
                    name: "Untrusted Input & Injection Probe".to_string(),
                    category: cat,
                    severity: RedTeamSeverity::High,
                    description: "Hunts for command injection, path traversal, deserialization bombs, and unchecked formats".to_string(),
                    probe_prompt: format!(
                        "Audit all input vectors in this code for path traversal (../), shell interpolation, unescaped queries, and deserialization gadgets:\n\n```\n{}\n```",
                        code_context
                    ),
                },
                RedTeamProbeCategory::ResourceExhaustionAndDos => RedTeamProbe {
                    id: format!("RT-DOS-{:03}", idx + 1),
                    name: "Algorithmic Complexity & ReDoS Exhaustion Probe".to_string(),
                    category: cat,
                    severity: RedTeamSeverity::Medium,
                    description: "Checks for quadratic regex backtracking, unbounded collections, memory leaks, and CPU spin loops".to_string(),
                    probe_prompt: format!(
                        "Evaluate the following implementation for catastrophic regex backtracking, unbounded Vec/HashMap allocations, uncontrolled stream buffering, and CPU starvation loops:\n\n```\n{}\n```",
                        code_context
                    ),
                },
                RedTeamProbeCategory::LogicAndAuthBypass => RedTeamProbe {
                    id: format!("RT-AUTH-{:03}", idx + 1),
                    name: "Authorization & State Tampering Probe".to_string(),
                    category: cat,
                    severity: RedTeamSeverity::Critical,
                    description: "Inspects state transition invariants, privilege checks, token validation, and unauthenticated endpoints".to_string(),
                    probe_prompt: format!(
                        "Audit the state machine and authorization model in this code. Uncover missing role checks, state transition circumvention, and session fixation bugs:\n\n```\n{}\n```",
                        code_context
                    ),
                },
                RedTeamProbeCategory::All => continue,
            };
            probes.push(probe);
        }

        probes
    }

    /// Format a complete Hermes user prompt for an adversarial audit turn
    pub fn format_audit_prompt(&self, code_snippet: &str, file_name: &str, focus: Option<RedTeamProbeCategory>) -> String {
        let cat_label = focus.map(|f| f.as_str()).unwrap_or("Full Spectrum Red Team");
        format!(
            r#"Adversarially scrutinize file `{file_name}` for `{cat_label}` vulnerabilities.

Target Code:
```
{code_snippet}
```

Format each vulnerability discovered in the following structured JSON block:
```json
[
  {{
    "id": "VULN-001",
    "title": "Concise vulnerability title",
    "severity": "CRITICAL | HIGH | MEDIUM | LOW | INFO",
    "line_start": 12,
    "line_end": 25,
    "summary": "Mechanistic explanation of how the vulnerability exists",
    "exploit_scenario": "Concrete step-by-step exploit proof-of-concept",
    "remediation": "Airtight code replacement or invariant fix"
  }}
]
```
If no vulnerabilities exist, return an empty array `[]`. Do not hold back."#
        )
    }

    /// Parse JSON findings emitted by Hermes red-team audit
    pub fn parse_findings(&self, audit_response: &str) -> Vec<RedTeamFinding> {
        let mut findings = Vec::new();

        // Extract JSON array from response
        let re_json = Regex::new(r"(?s)\[\s*\{.*?\}\s*\]").unwrap();
        let json_text = if let Some(cap) = re_json.find(audit_response) {
            cap.as_str()
        } else {
            audit_response.trim()
        };

        if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(json_text) {
            for (i, item) in items.into_iter().enumerate() {
                if let Value::Object(map) = item {
                    let id = map.get("id").and_then(|v| v.as_str()).unwrap_or(&format!("FINDING-{:03}", i + 1)).to_string();
                    let title = map.get("title").and_then(|v| v.as_str()).unwrap_or("Unspecified Vulnerability").to_string();
                    let sev_str = map.get("severity").and_then(|v| v.as_str()).unwrap_or("INFO");
                    let severity = RedTeamSeverity::from_str(sev_str);
                    let line_start = map.get("line_start").and_then(|v| v.as_u64()).map(|n| n as usize);
                    let line_end = map.get("line_end").and_then(|v| v.as_u64()).map(|n| n as usize);
                    let line_range = match (line_start, line_end) {
                        (Some(s), Some(e)) => Some((s, e)),
                        (Some(s), None) => Some((s, s)),
                        _ => None,
                    };
                    let summary = map.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let exploit = map.get("exploit_scenario").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let remediation = map.get("remediation").and_then(|v| v.as_str()).unwrap_or("").to_string();

                    findings.push(RedTeamFinding {
                        id,
                        title,
                        category: RedTeamProbeCategory::All,
                        severity,
                        file_path: None,
                        line_range,
                        vulnerability_summary: summary,
                        exploit_scenario: exploit,
                        remediation,
                    });
                }
            }
        }

        findings
    }

    /// Generate an adversarial exploit PoC test script based on an audit finding
    pub fn generate_exploit_poc(&self, finding: &RedTeamFinding) -> String {
        format!(
            r#"// ============================================================================
// ADVERSARIAL REPRODUCTION PROOF-OF-CONCEPT: {} [{}]
// ID: {}
// ============================================================================
//
// MECHANISTIC VULNERABILITY:
// {}
//
// EXPLOIT SCENARIO:
// {}
//
// REMEDIATION RECOMMENDATION:
// {}
// ============================================================================

#[cfg(test)]
mod redteam_exploit_poc {{
    use super::*;

    #[test]
    fn test_reproduce_{}() {{
        // Setup hostile payload triggering invariant violation:
        // {}
        println!("Verifying mitigation against exploit {}...");
    }}
}}
"#,
            finding.title,
            finding.severity.as_str(),
            finding.id,
            finding.vulnerability_summary,
            finding.exploit_scenario,
            finding.remediation,
            finding.id.to_lowercase().replace('-', "_"),
            finding.exploit_scenario.replace('\n', "\n        // "),
            finding.id
        )
    }
}
