use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::swarm::harmony::blackboard::SwarmBlackboard;
use crate::swarm::harmony::gates::{GateResult, ValidationGate};
use crate::swarm::harmony::types::{FailoverEvent, HarmonyRole, RoleArtifact};
use crate::types::{CompletionRequest, TokenUsage};

/// Pipeline stage configuration containing the executing role and attached validation gates.
pub struct HarmonyStage {
    pub role: Box<dyn HarmonyRole>,
    pub gates: Vec<Box<dyn ValidationGate>>,
}

impl HarmonyStage {
    pub fn new(role: Box<dyn HarmonyRole>) -> Self {
        Self {
            role,
            gates: Vec::new(),
        }
    }

    pub fn with_gate(mut self, gate: Box<dyn ValidationGate>) -> Self {
        self.gates.push(gate);
        self
    }
}

/// The complete execution outcome of a Structured Role-Based Harmony Swarm pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonyExecutionResult {
    pub objective: String,
    pub artifacts: Vec<RoleArtifact>,
    pub complete_project: String,
    pub audit_verdict: Option<String>,
    pub total_latency: Duration,
    pub total_usage: TokenUsage,
    pub total_cost_usd: f64,
}

/// Renders a standardized ANSI failover alert banner to stderr when a cloud LLM evacuates to local Ollama.
pub fn render_failover_banner(
    original_provider: &str,
    original_model: &str,
    trigger_reason: &str,
    target_provider: &str,
    target_model: &str,
    stage_idx: usize,
    stage_title: &str,
) {
    use colored::Colorize;
    eprintln!("\n{}", "┌───────────────────────────── ⚠️  FAILOVER NOTICE ─────────────────────────────┐".yellow().bold());
    let print_row = |content: &str| {
        let char_count = content.chars().count();
        let pad = if char_count < 77 { 77 - char_count } else { 0 };
        eprintln!("│ {}{} │", content, " ".repeat(pad));
    };
    print_row(&format!("Cloud Provider : {} ({})", original_provider, original_model));
    print_row(&format!("Trigger Reason : {}", trigger_reason));
    print_row(&format!("Action Taken   : 🔄 Evacuating to Local LLM ({}: {})", target_provider, target_model));
    print_row("Cost Delta     : +$0.00 (Zero incremental cost on local hardware)");
    print_row(&format!("Current Stage  : Stage {} [{}]", stage_idx + 1, stage_title));
    print_row("Context Retained: 100% (Architecture & Types preserved on Blackboard)");
    eprintln!("{}\n", "└───────────────────────────────────────────────────────────────────────────────┘".yellow().bold());
}

/// Emits an OS-native desktop notification alerting that failover occurred.
pub fn send_failover_desktop_notification(
    original_provider: &str,
    target_model: &str,
    trigger_reason: &str,
) {
    let summary = "Tagisan Harmony Swarm: Failover to Local LLM";
    let body = format!(
        "Cloud provider '{}' triggered failover ({}) -> Evacuated to local model '{}'.",
        original_provider, trigger_reason, target_model
    );

    #[cfg(target_family = "unix")]
    {
        let _ = std::process::Command::new("notify-send")
            .arg("-u")
            .arg("critical")
            .arg("-a")
            .arg("Tagisan")
            .arg(&summary)
            .arg(&body)
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
             $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
             $textNodes = $template.GetElementsByTagName('text'); \
             $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) > $null; \
             $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) > $null; \
             $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Tagisan'); \
             $notification = [Windows.UI.Notifications.ToastNotification]::new($template); \
             $notifier.Show($notification);",
            summary, body
        );
        let _ = std::process::Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&script)
            .spawn();
    }
}

/// The orchestrator executing the multi-stage assembly line across local and cloud LLMs.
pub struct StructuredHarmonyPipeline {
    pub blackboard: Arc<SwarmBlackboard>,
    stages: Vec<HarmonyStage>,
    audit_adversary: Option<(String, String)>,
    max_stage_retries: usize,
    /// When true, Stage 3 (QA) and Stage 4 (Doc) are executed concurrently in parallel.
    pub parallel_qa_doc: bool,
    /// When true, cloud rate limits, network faults, context errors, and budget exhaustion trigger evacuation to Ollama.
    pub fallback_to_local: bool,
    /// When true, reaching the financial spending cap automatically evacuates pending stages to zero-cost local Ollama.
    pub evacuate_on_budget: bool,
    /// When true, emits a native OS desktop notification when dynamic failover occurs.
    pub notify_on_failover: bool,
}

impl StructuredHarmonyPipeline {
    /// Create a new pipeline for a specific user objective.
    pub fn new(objective: impl Into<String>) -> Self {
        let obj = objective.into();
        Self {
            blackboard: Arc::new(SwarmBlackboard::new(obj)),
            stages: Vec::new(),
            audit_adversary: None,
            max_stage_retries: 2,
            parallel_qa_doc: false,
            fallback_to_local: false,
            evacuate_on_budget: false,
            notify_on_failover: false,
        }
    }

    /// Add a stage with its role and validation gates.
    pub fn add_stage(mut self, stage: HarmonyStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Attach an optional adversarial audit model to critique the final assembly.
    pub fn with_audit(mut self, adversary: (String, String)) -> Self {
        self.audit_adversary = Some(adversary);
        self
    }

    /// Set maximum retries allowed if a validation gate triggers a critique.
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_stage_retries = retries;
        self
    }

    /// Enable or disable concurrent parallel execution of independent downstream stages (QA + Doc).
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel_qa_doc = parallel;
        self
    }

    /// Enable or disable automatic failover to local Ollama on cloud failure/rate limits.
    pub fn with_fallback_to_local(mut self, fallback: bool) -> Self {
        self.fallback_to_local = fallback;
        self
    }

    /// Enable or disable zero-cost local evacuation when the financial spending budget is reached.
    pub fn with_evacuate_on_budget(mut self, evacuate: bool) -> Self {
        self.evacuate_on_budget = evacuate;
        self
    }

    /// Enable or disable OS desktop notifications on stage failover events.
    pub fn with_notify_on_failover(mut self, notify: bool) -> Self {
        self.notify_on_failover = notify;
        self
    }

    /// Resolve a zero-cost local provider and matching model for a role during failover evacuation.
    fn resolve_local_fallback(
        ctx: &EngineContext,
        role_id: &str,
    ) -> Option<(Arc<dyn LlmProvider>, String)> {
        let prov = ctx.get_provider("ollama").or_else(|_| ctx.get_provider("local")).ok()?;
        let installed = crate::providers::ollama::OllamaProvider::discover_installed_models();

        let model = match role_id {
            "architect" | "implementer" => {
                if let Some(m) = installed.iter().find(|m| m.contains("dolphin-phi") || m.contains("phi")) {
                    m.clone()
                } else if let Some(m) = installed.iter().find(|m| m.contains("qwen2.5:0.5b") || m.contains("qwen")) {
                    m.clone()
                } else if let Some(first) = installed.first() {
                    first.clone()
                } else {
                    crate::providers::ollama::default_ollama_model()
                }
            }
            _ => {
                if let Some(m) = installed.iter().find(|m| m.contains("qwen2.5:0.5b") || m.contains("qwen")) {
                    m.clone()
                } else if let Some(m) = installed.iter().find(|m| m.contains("dolphin-phi") || m.contains("phi")) {
                    m.clone()
                } else if let Some(first) = installed.first() {
                    first.clone()
                } else {
                    crate::providers::ollama::default_ollama_model()
                }
            }
        };

        Some((prov, model))
    }

    /// Execute the complete assembly line pipeline.
    pub async fn execute(&self, ctx: &EngineContext) -> Result<HarmonyExecutionResult> {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let start_time = Instant::now();
        let mut total_tokens = TokenUsage::default();

        if self.parallel_qa_doc && self.stages.len() == 4 {
            // Stage 1: Lead Systems Architect
            let a1 = self
                .execute_stage_with_retries(
                    &self.stages[0],
                    &self.blackboard,
                    ctx,
                    self.max_stage_retries,
                    0,
                )
                .await?;
            total_tokens.prompt_tokens += a1.tokens_used / 2;
            total_tokens.completion_tokens += a1.tokens_used / 2;
            self.blackboard.append_artifact(a1);

            // Stage 2: Senior Implementer
            let a2 = self
                .execute_stage_with_retries(
                    &self.stages[1],
                    &self.blackboard,
                    ctx,
                    self.max_stage_retries,
                    1,
                )
                .await?;
            total_tokens.prompt_tokens += a2.tokens_used / 2;
            total_tokens.completion_tokens += a2.tokens_used / 2;
            self.blackboard.append_artifact(a2);

            // Stage 3 (QA) and Stage 4 (Doc) executed concurrently in parallel
            let fut_qa = self.execute_stage_with_retries(
                &self.stages[2],
                &self.blackboard,
                ctx,
                self.max_stage_retries,
                2,
            );
            let fut_doc = self.execute_stage_with_retries(
                &self.stages[3],
                &self.blackboard,
                ctx,
                self.max_stage_retries,
                3,
            );

            let (a3, a4) = tokio::try_join!(fut_qa, fut_doc)?;

            total_tokens.prompt_tokens += (a3.tokens_used + a4.tokens_used) / 2;
            total_tokens.completion_tokens += (a3.tokens_used + a4.tokens_used) / 2;
            self.blackboard.append_artifact(a3);
            self.blackboard.append_artifact(a4);
        } else {
            // Sequential execution loop
            for (idx, stage) in self.stages.iter().enumerate() {
                if ctx.cancellation_token.is_cancelled() {
                    return Err(TagisanError::Cancelled);
                }

                let final_artifact = self
                    .execute_stage_with_retries(
                        stage,
                        &self.blackboard,
                        ctx,
                        self.max_stage_retries,
                        idx,
                    )
                    .await?;

                total_tokens.prompt_tokens += final_artifact.tokens_used / 2;
                total_tokens.completion_tokens += final_artifact.tokens_used / 2;

                self.blackboard.append_artifact(final_artifact);
            }
        }

        let complete_project = self.blackboard.assemble_complete_project();

        // Optional Adversarial Audit Phase
        let mut audit_verdict = None;
        if let Some((ref adv_prov_id, ref adv_model)) = self.audit_adversary {
            if !ctx.cancellation_token.is_cancelled() {
                let adv_prov = ctx.get_provider(adv_prov_id)?;
                let audit_prompt = format!(
                    "You are a Principal Security and Architecture Auditor inspecting the assembled project.\n\n\
                    PROJECT OBJECTIVE:\n\"{}\"\n\n\
                    ASSEMBLED CODEBASE:\n\"\"\"\n{}\n\"\"\"\n\n\
                    TASK:\n\
                    1. Rigorously inspect this implementation for edge-case vulnerabilities, performance regressions, or type violations.\n\
                    2. Provide an executive summary and final verdict (APPROVED, CONDITIONAL, or REJECTED) with concrete reasoning.",
                    self.blackboard.user_objective, complete_project
                );

                let req = CompletionRequest::new(adv_model.clone(), audit_prompt)
                    .with_temperature(0.3)
                    .with_cancellation(ctx.cancellation_token.clone());

                let resp = adv_prov.complete(req).await?;
                total_tokens.prompt_tokens += resp.usage.prompt_tokens;
                total_tokens.completion_tokens += resp.usage.completion_tokens;
                audit_verdict = Some(resp.message.extract_text());
            }
        }

        let total_latency = start_time.elapsed();
        let total_cost_usd = ctx.budget_tracker.current_spent_usd();

        Ok(HarmonyExecutionResult {
            objective: self.blackboard.user_objective.clone(),
            artifacts: self.blackboard.get_artifacts(),
            complete_project,
            audit_verdict,
            total_latency,
            total_usage: total_tokens,
            total_cost_usd,
        })
    }

    /// Internal helper to execute a single stage through its validation gates with critique retries.
    async fn execute_stage_with_retries(
        &self,
        stage: &HarmonyStage,
        blackboard: &SwarmBlackboard,
        ctx: &EngineContext,
        max_retries: usize,
        stage_idx: usize,
    ) -> Result<RoleArtifact> {
        let role_cfg = stage.role.config();
        let is_already_local = role_cfg.provider.eq_ignore_ascii_case("ollama")
            || role_cfg.provider.eq_ignore_ascii_case("local");

        let mut current_provider = ctx.get_provider(&role_cfg.provider)?;
        let mut current_model_override: Option<String> = None;
        let mut current_failover_event: Option<FailoverEvent> = None;

        // Check if pre-emptive evacuation is warranted (budget already exhausted on entry)
        let can_evacuate_budget = self.evacuate_on_budget || self.fallback_to_local;
        if !is_already_local && can_evacuate_budget && ctx.budget_tracker.is_exhausted() {
            if let Some((loc_prov, loc_model)) = Self::resolve_local_fallback(ctx, &role_cfg.role_id) {
                let trigger_reason = format!(
                    "Token budget reached (${:.2} spent >= ${:.2} max)",
                    ctx.budget_tracker.current_spent_usd(),
                    ctx.budget_tracker.max_budget_usd()
                );
                let timestamp_epoch_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let event = FailoverEvent {
                    original_provider: role_cfg.provider.clone(),
                    original_model: role_cfg.model.clone(),
                    trigger_reason: trigger_reason.clone(),
                    evacuated_to_provider: loc_prov.provider_id().to_string(),
                    evacuated_to_model: loc_model.clone(),
                    timestamp_epoch_ms,
                    cost_at_failover_usd: ctx.budget_tracker.current_spent_usd(),
                };

                render_failover_banner(
                    &role_cfg.provider,
                    &role_cfg.model,
                    &trigger_reason,
                    loc_prov.provider_id(),
                    &loc_model,
                    stage_idx,
                    &role_cfg.role_title,
                );

                if self.notify_on_failover {
                    send_failover_desktop_notification(
                        &role_cfg.provider,
                        &loc_model,
                        &trigger_reason,
                    );
                }

                current_provider = loc_prov;
                current_model_override = Some(loc_model);
                current_failover_event = Some(event);
            }
        }

        let mut retries = 0usize;
        let mut last_critique: Option<String> = None;
        let mut committed_artifact: Option<RoleArtifact> = None;

        while retries <= max_retries {
            if ctx.cancellation_token.is_cancelled() {
                return Err(TagisanError::Cancelled);
            }

            let exec_result = stage
                .role
                .execute_stage_with_override(
                    blackboard,
                    current_provider.clone(),
                    current_model_override.as_deref(),
                    ctx,
                    last_critique.as_deref(),
                )
                .await;

            let mut artifact = match exec_result {
                Ok(art) => art,
                Err(err) => {
                    let is_trigger_error = matches!(
                        err,
                        TagisanError::BudgetExceeded { .. }
                            | TagisanError::RateLimited(_, _)
                            | TagisanError::BadResponse(_, _)
                            | TagisanError::ContextLengthExceeded(_, _, _)
                            | TagisanError::Network(_)
                    );

                    let can_evacuate = current_failover_event.is_none()
                        && !is_already_local
                        && ((self.fallback_to_local && is_trigger_error)
                            || (self.evacuate_on_budget && matches!(err, TagisanError::BudgetExceeded { .. })));

                    if can_evacuate {
                        if let Some((loc_prov, loc_model)) = Self::resolve_local_fallback(ctx, &role_cfg.role_id) {
                            let trigger_reason = match &err {
                                TagisanError::BudgetExceeded { max_budget, current_spent } => {
                                    format!("Token budget reached (${:.2} spent >= ${:.2} max)", current_spent, max_budget)
                                }
                                TagisanError::RateLimited(p, retry_after) => {
                                    format!("Rate limited (HTTP 429) on '{}' (retry after: {:?})", p, retry_after)
                                }
                                TagisanError::BadResponse(p, msg) => {
                                    format!("API error on '{}': {}", p, msg)
                                }
                                TagisanError::ContextLengthExceeded(p, cur, max) => {
                                    format!("Context length exceeded on '{}' ({} > {})", p, cur, max)
                                }
                                TagisanError::Network(msg) => {
                                    format!("Network fault: {}", msg)
                                }
                                _ => format!("Error: {}", err),
                            };

                            let timestamp_epoch_ms = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64;

                            let event = FailoverEvent {
                                original_provider: role_cfg.provider.clone(),
                                original_model: role_cfg.model.clone(),
                                trigger_reason: trigger_reason.clone(),
                                evacuated_to_provider: loc_prov.provider_id().to_string(),
                                evacuated_to_model: loc_model.clone(),
                                timestamp_epoch_ms,
                                cost_at_failover_usd: ctx.budget_tracker.current_spent_usd(),
                            };

                            render_failover_banner(
                                &role_cfg.provider,
                                &role_cfg.model,
                                &trigger_reason,
                                loc_prov.provider_id(),
                                &loc_model,
                                stage_idx,
                                &role_cfg.role_title,
                            );

                            if self.notify_on_failover {
                                send_failover_desktop_notification(
                                    &role_cfg.provider,
                                    &loc_model,
                                    &trigger_reason,
                                );
                            }

                            current_provider = loc_prov;
                            current_model_override = Some(loc_model);
                            current_failover_event = Some(event);

                            // Retry immediately with the local provider
                            continue;
                        }
                    }

                    return Err(err);
                }
            };

            // Attach failover telemetry event if one occurred
            if let Some(ref fo) = current_failover_event {
                artifact.failover_event = Some(fo.clone());
            }

            // Run all validation gates
            let mut all_passed = true;
            for gate in &stage.gates {
                match gate.validate(&artifact) {
                    GateResult::Pass => {}
                    GateResult::RetryWithCritique { critique } => {
                        last_critique = Some(critique);
                        all_passed = false;
                        break;
                    }
                    GateResult::HardFailure { reason } => {
                        return Err(TagisanError::Execution(format!(
                            "Stage {} ('{}') hard gate failure: {}",
                            stage_idx + 1,
                            role_cfg.role_title,
                            reason
                        )));
                    }
                }
            }

            if all_passed {
                committed_artifact = Some(artifact);
                break;
            } else {
                retries += 1;
            }
        }

        committed_artifact.ok_or_else(|| {
            TagisanError::Execution(format!(
                "Stage {} ('{}') failed to satisfy validation gates after {} retries. Last critique: {:?}",
                stage_idx + 1,
                role_cfg.role_title,
                max_retries,
                last_critique
            ))
        })
    }
}
