//! TagisanTracer — structured OpenTelemetry-compatible trace recorder.
//!
//! Uses the existing `tracing` crate for structured spans. When the `otel`
//! feature flag is enabled, spans are also exported via OTLP gRPC to backends
//! like Langfuse, Arize Phoenix, Datadog, or Jaeger.

use super::{journal::TraceJournal, span::TraceSpan};
use serde_json::json;
use tracing::{info, info_span};

/// The central telemetry facade for Tagisan. Records structured spans to both
/// the `tracing` crate and the persistent JSONL journal.
#[derive(Debug, Clone)]
pub struct TagisanTracer {
    pub service_name: String,
    pub otlp_endpoint: Option<String>,
    pub journal: TraceJournal,
}

impl TagisanTracer {
    pub fn new(
        service_name: impl Into<String>,
        otlp_endpoint: Option<String>,
    ) -> Self {
        let endpoint = otlp_endpoint.or_else(|| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok());
        Self {
            service_name: service_name.into(),
            otlp_endpoint: endpoint,
            journal: TraceJournal::default(),
        }
    }

    pub fn default_tracer() -> Self {
        Self::new("tgs", None)
    }

    pub fn with_journal(mut self, journal: TraceJournal) -> Self {
        self.journal = journal;
        self
    }

    /// Asynchronously export an event to an OTLP endpoint (HTTP or gRPC bridge)
    pub fn maybe_export_otlp(&self, span_name: &str, fields: &serde_json::Value, duration_ms: Option<u64>) {
        if let Some(ref endpoint) = self.otlp_endpoint {
            let endpoint_url = if endpoint.ends_with("/v1/traces") {
                endpoint.clone()
            } else {
                format!("{}/v1/traces", endpoint.trim_end_matches('/'))
            };

            let service_name = self.service_name.clone();
            let span_name_owned = span_name.to_string();
            let fields_owned = fields.clone();

            tokio::spawn(async move {
                let payload = json!({
                    "resourceSpans": [{
                        "resource": {
                            "attributes": [{
                                "key": "service.name",
                                "value": { "stringValue": service_name }
                            }]
                        },
                        "scopeSpans": [{
                            "scope": { "name": "tagisan.telemetry" },
                            "spans": [{
                                "name": span_name_owned,
                                "kind": 1,
                                "attributes": [{
                                    "key": "tagisan.payload",
                                    "value": { "stringValue": fields_owned.to_string() }
                                }],
                                "durationMs": duration_ms
                            }]
                        }]
                    }]
                });

                let client = reqwest::Client::new();
                let _ = client.post(&endpoint_url).json(&payload).send().await;
            });
        }
    }

    /// Start a new named span.
    pub fn span(&self, name: impl Into<String>) -> TraceSpan {
        TraceSpan::new(name)
    }

    /// Record a DAG node execution event.
    pub fn record_dag_node(
        &self,
        task_name: &str,
        provider: &str,
        input_tokens: u64,
        output_tokens: u64,
        latency_ms: u64,
    ) {
        let _span = info_span!(
            "dag.node.execute",
            task_name = task_name,
            provider = provider,
            input_tokens = input_tokens,
            output_tokens = output_tokens,
            latency_ms = latency_ms,
        )
        .entered();

        info!(
            task = task_name,
            provider = provider,
            input_tokens = input_tokens,
            output_tokens = output_tokens,
            latency_ms = latency_ms,
            "DAG node executed"
        );

        let mut span = self.span("dag.node.execute");
        span.set_field("task_name", json!(task_name));
        span.set_field("provider", json!(provider));
        span.set_field("input_tokens", json!(input_tokens));
        span.set_field("output_tokens", json!(output_tokens));
        span.set_field("latency_ms", json!(latency_ms));
        let fields = span.fields.clone();
        let _ = span.finish(&self.journal);
        self.maybe_export_otlp("dag.node.execute", &fields, Some(latency_ms));
    }

    /// Record an agent ReAct turn event.
    pub fn record_agent_turn(
        &self,
        thought: &str,
        tool_name: &str,
        args: &str,
        observation_ok: bool,
    ) {
        let _span = info_span!(
            "agent.react.turn",
            tool = tool_name,
            observation_ok = observation_ok,
        )
        .entered();

        info!(
            tool = tool_name,
            observation_ok = observation_ok,
            "Agent ReAct turn"
        );

        let mut span = self.span("agent.react.turn");
        span.set_field("thought", json!(thought));
        span.set_field("tool_name", json!(tool_name));
        span.set_field("args", json!(args));
        span.set_field("observation_ok", json!(observation_ok));
        let fields = span.fields.clone();
        let _ = span.finish(&self.journal);
        self.maybe_export_otlp("agent.react.turn", &fields, None);
    }

    /// Record a dialectical debate round event.
    pub fn record_debate_round(
        &self,
        round: u32,
        thesis: &str,
        antithesis: &str,
        synthesis: &str,
        borda_score: f32,
    ) {
        let _span = info_span!(
            "debate.round",
            round = round,
            borda_score = borda_score,
        )
        .entered();

        info!(round = round, borda_score = borda_score, "Debate round completed");

        let mut span = self.span("debate.round");
        span.set_field("round", json!(round));
        span.set_field("thesis", json!(thesis));
        span.set_field("antithesis", json!(antithesis));
        span.set_field("synthesis", json!(synthesis));
        span.set_field("borda_score", json!(borda_score));
        let fields = span.fields.clone();
        let _ = span.finish(&self.journal);
        self.maybe_export_otlp("debate.round", &fields, None);
    }

    /// Record a token cost event.
    pub fn record_cost(&self, provider: &str, usd: f64) {
        let _span = info_span!("cost.usd", provider = provider, usd = usd).entered();
        info!(provider = provider, usd = usd, "Token cost recorded");

        let mut span = self.span("cost.usd");
        span.set_field("provider", json!(provider));
        span.set_field("usd", json!(usd));
        let fields = span.fields.clone();
        let _ = span.finish(&self.journal);
        self.maybe_export_otlp("cost.usd", &fields, None);
    }

    /// Record an AgentShield interception event.
    pub fn record_agentshield(&self, threat_level: &str, command: &str, blocked: bool) {
        let _span = info_span!(
            "agentshield.interception",
            threat_level = threat_level,
            blocked = blocked,
        )
        .entered();

        info!(
            threat_level = threat_level,
            command = command,
            blocked = blocked,
            "AgentShield interception"
        );

        let mut span = self.span("agentshield.interception");
        span.set_field("threat_level", json!(threat_level));
        span.set_field("command", json!(command));
        span.set_field("blocked", json!(blocked));
        let fields = span.fields.clone();
        let _ = span.finish(&self.journal);
        self.maybe_export_otlp("agentshield.interception", &fields, None);
    }
}
