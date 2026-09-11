use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::swarm::harmony::blackboard::SwarmBlackboard;
use crate::swarm::harmony::types::{ExtractedCodeBlock, HarmonyRole, HarmonyRoleConfig, RoleArtifact};
use crate::types::CompletionRequest;

/// Helper: Extracts code blocks delimited by markdown backticks.
pub fn extract_markdown_code_blocks(text: &str) -> Vec<ExtractedCodeBlock> {
    let mut blocks = Vec::new();
    let mut current_lang = String::new();
    let mut current_code = String::new();
    let mut in_block = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_block {
                // End of block
                blocks.push(ExtractedCodeBlock {
                    language: if current_lang.is_empty() {
                        "text".to_string()
                    } else {
                        current_lang.clone()
                    },
                    code: current_code.trim().to_string(),
                });
                current_code.clear();
                current_lang.clear();
                in_block = false;
            } else {
                // Start of block
                in_block = true;
                current_lang = trimmed.trim_start_matches('`').trim().to_string();
            }
        } else if in_block {
            current_code.push_str(line);
            current_code.push('\n');
        }
    }

    blocks
}

/// Generic configurable role implementing `HarmonyRole`.
#[derive(Clone)]
pub struct StandardHarmonyRole {
    pub config: HarmonyRoleConfig,
    pub prompt_builder: Arc<dyn Fn(&SwarmBlackboard, &str) -> String + Send + Sync>,
    pub auto_skills: bool,
    pub base_system_contract: String,
    pub domain_query: String,
}

impl std::fmt::Debug for StandardHarmonyRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StandardHarmonyRole")
            .field("config", &self.config)
            .field("auto_skills", &self.auto_skills)
            .finish()
    }
}

impl StandardHarmonyRole {
    pub fn new(
        config: HarmonyRoleConfig,
        prompt_builder: impl Fn(&SwarmBlackboard, &str) -> String + Send + Sync + 'static,
    ) -> Self {
        let base_contract = config.system_contract.clone();
        Self {
            config,
            prompt_builder: Arc::new(prompt_builder),
            auto_skills: false,
            base_system_contract: base_contract,
            domain_query: String::new(),
        }
    }

    pub fn new_with_skills(
        config: HarmonyRoleConfig,
        prompt_builder: impl Fn(&SwarmBlackboard, &str) -> String + Send + Sync + 'static,
        auto_skills: bool,
        base_system_contract: String,
        domain_query: String,
    ) -> Self {
        Self {
            config,
            prompt_builder: Arc::new(prompt_builder),
            auto_skills,
            base_system_contract,
            domain_query,
        }
    }

    /// Helper to execute an LLM prompt and return a structured `RoleArtifact`.
    pub async fn execute_prompt(
        &self,
        blackboard: &SwarmBlackboard,
        provider: Arc<dyn LlmProvider>,
        ctx: &EngineContext,
        extra_critique: Option<&str>,
    ) -> Result<RoleArtifact> {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let mut user_prompt = (self.prompt_builder)(blackboard, &blackboard.user_objective);
        if let Some(critique) = extra_critique {
            user_prompt.push_str(&format!(
                "\n\n[VALIDATION FEEDBACK / CORRECTION REQUIRED]:\n{}\nPlease correct this in your output.",
                critique
            ));
        }

        let mut req = CompletionRequest::new(self.config.model.clone(), user_prompt)
            .with_system(self.config.system_contract.clone())
            .with_temperature(self.config.temperature)
            .with_cancellation(ctx.cancellation_token.clone());

        if self.config.max_tokens > 0 {
            req.max_tokens = Some(self.config.max_tokens);
        }

        let t0 = Instant::now();
        let resp = provider.complete(req).await?;
        let elapsed = t0.elapsed();

        let raw_text = resp.message.extract_text();
        let code_blocks = extract_markdown_code_blocks(&raw_text);

        // Record budget / tokens (zero-cost for Ollama or local providers)
        if self.config.provider.eq_ignore_ascii_case("ollama")
            || self.config.provider.eq_ignore_ascii_case("local")
            || resp.usage.estimated_cost_usd == Some(0.0)
        {
            ctx.budget_tracker.record_micro_usd(0)?;
        } else {
            ctx.budget_tracker.record_usage(&self.config.model, &resp.usage)?;
        }

        let artifact = RoleArtifact {
            role_id: self.config.role_id.clone(),
            role_title: self.config.role_title.clone(),
            provider: self.config.provider.clone(),
            model: self.config.model.clone(),
            raw_output: raw_text,
            code_blocks,
            latency_secs: elapsed.as_secs_f64(),
            tokens_used: resp.usage.prompt_tokens + resp.usage.completion_tokens,
            failover_event: None,
        };

        Ok(artifact)
    }

    /// Create a copy of this role with provider and model overridden.
    pub fn with_provider_and_model(&self, provider: &str, model: &str) -> Self {
        let mut new_config = self.config.clone();
        new_config.provider = provider.to_string();
        new_config.model = model.to_string();

        if self.auto_skills && !self.domain_query.is_empty() {
            let dispatcher = crate::ecc::skills::global_dispatcher();
            let (equipped, _) = dispatcher.equip_prompt_for_provider(
                &self.base_system_contract,
                &self.domain_query,
                provider,
                None,
            );
            new_config.system_contract = equipped;
        }

        Self {
            config: new_config,
            prompt_builder: self.prompt_builder.clone(),
            auto_skills: self.auto_skills,
            base_system_contract: self.base_system_contract.clone(),
            domain_query: self.domain_query.clone(),
        }
    }

    /// Execute the role stage with an optional model override and explicit provider.
    pub async fn execute_stage_with_override(
        &self,
        blackboard: &SwarmBlackboard,
        provider: Arc<dyn LlmProvider>,
        model_override: Option<&str>,
        ctx: &EngineContext,
        critique: Option<&str>,
    ) -> Result<RoleArtifact> {
        if let Some(m) = model_override {
            let role = self.with_provider_and_model(provider.provider_id(), m);
            role.execute_prompt(blackboard, provider, ctx, critique).await
        } else if provider.provider_id() != self.config.provider {
            let role = self.with_provider_and_model(provider.provider_id(), &self.config.model);
            role.execute_prompt(blackboard, provider, ctx, critique).await
        } else {
            self.execute_prompt(blackboard, provider, ctx, critique).await
        }
    }
}

#[async_trait]
impl HarmonyRole for StandardHarmonyRole {
    fn config(&self) -> &HarmonyRoleConfig {
        &self.config
    }

    fn set_auto_skills(&mut self, enabled: bool) {
        self.auto_skills = enabled;
        if enabled && !self.domain_query.is_empty() {
            let dispatcher = crate::ecc::skills::global_dispatcher();
            let (equipped, _) = dispatcher.equip_prompt_for_provider(
                &self.base_system_contract,
                &self.domain_query,
                &self.config.provider,
                None,
            );
            self.config.system_contract = equipped;
        } else {
            self.config.system_contract = self.base_system_contract.clone();
        }
    }

    async fn execute_stage(
        &self,
        blackboard: &SwarmBlackboard,
        provider: Arc<dyn LlmProvider>,
        ctx: &EngineContext,
        critique: Option<&str>,
    ) -> Result<RoleArtifact> {
        self.execute_prompt(blackboard, provider, ctx, critique).await
    }

    async fn execute_stage_with_override(
        &self,
        blackboard: &SwarmBlackboard,
        provider: Arc<dyn LlmProvider>,
        model_override: Option<&str>,
        ctx: &EngineContext,
        critique: Option<&str>,
    ) -> Result<RoleArtifact> {
        StandardHarmonyRole::execute_stage_with_override(
            self,
            blackboard,
            provider,
            model_override,
            ctx,
            critique,
        )
        .await
    }
}

/// Factory functions for creating standard assembly line roles.
pub struct AssemblyRoles;

impl AssemblyRoles {
    /// 1. Systems Architect: Produces types, structs, interfaces, and signatures.
    pub fn architect(provider: &str, model: &str) -> Box<dyn HarmonyRole> {
        Self::architect_with_auto_skills(provider, model, true)
    }

    /// 1b. Systems Architect with explicit auto-skills flag
    pub fn architect_with_auto_skills(provider: &str, model: &str, auto_skills: bool) -> Box<dyn HarmonyRole> {
        let base_contract = 
            "You are the Lead Systems Architect in an automated software assembly line.\n\
            STRICT CONTRACT:\n\
            1. Output ONLY data structures, types, enums, error definitions, and method signatures.\n\
            2. Do NOT write conversational greetings, explanations, or function bodies.\n\
            3. Enclose all code strictly in markdown code fences (e.g. ```rust ... ```).\n\
            4. Focus on modular boundary design, safety invariants, and strict type safety.";

        let domain_query = "structured analysis yourdon modular coupling cohesion domain driven design page jones";
        let system_contract = if auto_skills {
            let dispatcher = crate::ecc::skills::global_dispatcher();
            let (equipped, _) = dispatcher.equip_prompt_for_provider(
                base_contract,
                domain_query,
                provider,
                None,
            );
            equipped
        } else {
            base_contract.to_string()
        };

        let config = HarmonyRoleConfig::new("architect", "Lead Systems Architect", provider, model, system_contract)
            .with_temperature(0.2);

        let role = StandardHarmonyRole::new_with_skills(
            config,
            |_, objective| {
                format!(
                    "OBJECTIVE:\n\"{}\"\n\n\
                    TASK: Design the complete data structures, type signatures, interfaces, and error types. \
                    Output only the types and function declarations in markdown code blocks.",
                    objective
                )
            },
            auto_skills,
            base_contract.to_string(),
            domain_query.to_string(),
        );

        Box::new(role)
    }

    /// 2. Senior Implementer: Implements function bodies based on Architect's types.
    pub fn implementer(provider: &str, model: &str) -> Box<dyn HarmonyRole> {
        Self::implementer_with_auto_skills(provider, model, true)
    }

    /// 2b. Senior Implementer with explicit auto-skills flag
    pub fn implementer_with_auto_skills(provider: &str, model: &str, auto_skills: bool) -> Box<dyn HarmonyRole> {
        let base_contract =
            "You are the Senior Systems Implementer in an automated software assembly line.\n\
            STRICT CONTRACT:\n\
            1. Given the types from the Systems Architect, implement the complete algorithm and method bodies.\n\
            2. Do NOT redefine structs or enums established by the Architect.\n\
            3. Do NOT write conversational greetings or filler text. Output ONLY compilable implementation code.\n\
            4. Enclose all code strictly in markdown code fences.";

        let domain_query = "rust tokio concurrency design by contract tokio async tuning";
        let system_contract = if auto_skills {
            let dispatcher = crate::ecc::skills::global_dispatcher();
            let (equipped, _) = dispatcher.equip_prompt_for_provider(
                base_contract,
                domain_query,
                provider,
                None,
            );
            equipped
        } else {
            base_contract.to_string()
        };

        let config = HarmonyRoleConfig::new("implementer", "Senior Systems Implementer", provider, model, system_contract)
            .with_temperature(0.3);

        let role = StandardHarmonyRole::new_with_skills(
            config,
            |blackboard, objective| {
                let architect_spec = blackboard
                    .get_artifact("architect")
                    .map(|a| a.raw_output)
                    .unwrap_or_default();

                format!(
                    "OBJECTIVE:\n\"{}\"\n\n\
                    ARCHITECT SPECIFICATION:\n\"\"\"\n{}\n\"\"\"\n\n\
                    TASK: Implement the complete execution logic and method bodies for the specification above. \
                    Output only the implementation code inside markdown code blocks.",
                    objective, architect_spec
                )
            },
            auto_skills,
            base_contract.to_string(),
            domain_query.to_string(),
        );

        Box::new(role)
    }

    /// 3. QA & Test Specialist: Generates comprehensive unit tests with assertions.
    pub fn qa(provider: &str, model: &str) -> Box<dyn HarmonyRole> {
        Self::qa_with_auto_skills(provider, model, true)
    }

    /// 3b. QA & Test Specialist with explicit auto-skills flag
    pub fn qa_with_auto_skills(provider: &str, model: &str, auto_skills: bool) -> Box<dyn HarmonyRole> {
        let base_contract =
            "You are the Quality & Test Verification Specialist in an automated software assembly line.\n\
            STRICT CONTRACT:\n\
            1. Given the types and implementation, generate comprehensive unit tests covering edge cases, null boundaries, scale limits, and happy paths.\n\
            2. Do NOT write greetings or chat. Output ONLY test code in markdown code fences.\n\
            3. Write deterministic assertions that verify correctness under stress.";

        let domain_query = "tdd workflow data intensive architecture unit test assertions edge cases";
        let system_contract = if auto_skills {
            let dispatcher = crate::ecc::skills::global_dispatcher();
            let (equipped, _) = dispatcher.equip_prompt_for_provider(
                base_contract,
                domain_query,
                provider,
                None,
            );
            equipped
        } else {
            base_contract.to_string()
        };

        let config = HarmonyRoleConfig::new("qa", "QA & Verification Specialist", provider, model, system_contract)
            .with_temperature(0.2);

        let role = StandardHarmonyRole::new_with_skills(
            config,
            |blackboard, objective| {
                let arch_spec = blackboard.get_artifact("architect").map(|a| a.raw_output).unwrap_or_default();
                let impl_code = blackboard.get_artifact("implementer").map(|a| a.raw_output).unwrap_or_default();

                format!(
                    "OBJECTIVE:\n\"{}\"\n\n\
                    TYPES SPECIFICATION:\n\"\"\"\n{}\n\"\"\"\n\n\
                    IMPLEMENTATION CODE:\n\"\"\"\n{}\n\"\"\"\n\n\
                    TASK: Write complete unit tests covering edge cases, concurrent access, and boundary checks. \
                    Enclose tests in markdown code blocks.",
                    objective, arch_spec, impl_code
                )
            },
            auto_skills,
            base_contract.to_string(),
            domain_query.to_string(),
        );

        Box::new(role)
    }

    /// 4. Documentation & Packaging: Produces usage documentation and examples.
    pub fn documentation(provider: &str, model: &str) -> Box<dyn HarmonyRole> {
        Self::documentation_with_auto_skills(provider, model, true)
    }

    /// 4b. Documentation & Packaging with explicit auto-skills flag
    pub fn documentation_with_auto_skills(provider: &str, model: &str, auto_skills: bool) -> Box<dyn HarmonyRole> {
        let base_contract =
            "You are the Documentation & Packaging Specialist in an automated software assembly line.\n\
            STRICT CONTRACT:\n\
            1. Synthesize concise technical documentation, quickstart usage examples, and complexity notes.\n\
            2. Output cleanly formatted markdown.";

        let domain_query = "documentation quickstart api usage clean standards";
        let system_contract = if auto_skills {
            let dispatcher = crate::ecc::skills::global_dispatcher();
            let (equipped, _) = dispatcher.equip_prompt_for_provider(
                base_contract,
                domain_query,
                provider,
                None,
            );
            equipped
        } else {
            base_contract.to_string()
        };

        let config = HarmonyRoleConfig::new("doc", "Documentation & Packaging Specialist", provider, model, system_contract)
            .with_temperature(0.4);

        let role = StandardHarmonyRole::new_with_skills(
            config,
            |blackboard, objective| {
                let arch_spec = blackboard.get_artifact("architect").map(|a| a.raw_output).unwrap_or_default();
                let impl_code = blackboard.get_artifact("implementer").map(|a| a.raw_output).unwrap_or_default();

                format!(
                    "OBJECTIVE:\n\"{}\"\n\n\
                    TYPES:\n\"\"\"\n{}\n\"\"\"\n\n\
                    IMPLEMENTATION:\n\"\"\"\n{}\n\"\"\"\n\n\
                    TASK: Write clean markdown documentation, quickstart API usage examples, and time/space complexity notes.",
                    objective, arch_spec, impl_code
                )
            },
            auto_skills,
            base_contract.to_string(),
            domain_query.to_string(),
        );

        Box::new(role)
    }
}

/// Model assignment overrides from user CLI flags.
#[derive(Debug, Clone, Default)]
pub struct RoleModelOverrides {
    pub architect: Option<String>,
    pub implementer: Option<String>,
    pub qa: Option<String>,
    pub doc: Option<String>,
    pub profile: Option<crate::swarm::harmony::types::HarmonyTierProfile>,
}

/// Helper: Parses a string like "ollama:qwen2.5:0.5b", "anthropic:claude-3-5-sonnet", "mock:model", or "qwen2.5:0.5b" into (provider_id, model_name).
pub fn parse_provider_and_model(input: &str, default_provider: &str) -> (String, String) {
    let trimmed = input.trim();
    if let Some((prov, model)) = trimmed.split_once(':') {
        let prov_lower = prov.to_lowercase();
        match prov_lower.as_str() {
            "ollama" | "anthropic" | "openai" | "gemini" | "xai" | "deepseek" | "groq" | "mock" => {
                (prov_lower, model.to_string())
            }
            _ => (default_provider.to_string(), trimmed.to_string()),
        }
    } else {
        (default_provider.to_string(), trimmed.to_string())
    }
}

/// Auto-discovers and resolves role models for Local and Non-Local providers.
pub fn resolve_harmony_models(
    ctx: &EngineContext,
    overrides: &RoleModelOverrides,
) -> ((String, String), (String, String), (String, String), (String, String)) {
    use crate::swarm::harmony::types::HarmonyTierProfile;

    // 1. Determine baseline provider and available models
    let has_anthropic = ctx.get_provider("anthropic").is_ok();
    let has_openai = ctx.get_provider("openai").is_ok();
    let has_gemini = ctx.get_provider("gemini").is_ok();
    let has_deepseek = ctx.get_provider("deepseek").is_ok();
    let has_ollama = ctx.get_provider("ollama").is_ok();
    let has_mock = ctx.get_provider("mock").is_ok();

    let profile = overrides.profile.unwrap_or(HarmonyTierProfile::Smart);

    // Default pair defaults
    let (mut arch_p, mut arch_m);
    let (mut impl_p, mut impl_m);
    let (mut qa_p, mut qa_m);
    let (mut doc_p, mut doc_m);

    let has_any_cloud = has_anthropic || has_openai || has_gemini || has_deepseek;

    if has_any_cloud {
        match profile {
            HarmonyTierProfile::Smart => {
                // Smart tier: Top reasoning/code for Architect & QA, cost-effective high-throughput for Doc
                // Architect
                if has_anthropic {
                    arch_p = "anthropic".to_string(); arch_m = "claude-3-5-sonnet-20241022".to_string();
                } else if has_openai {
                    arch_p = "openai".to_string(); arch_m = "gpt-4o".to_string();
                } else if has_deepseek {
                    arch_p = "deepseek".to_string(); arch_m = "deepseek-chat".to_string();
                } else {
                    arch_p = "gemini".to_string(); arch_m = "gemini-2.0-flash".to_string();
                }

                // Implementer: DeepSeek V3 if present, else Anthropic/OpenAI/Gemini
                if has_deepseek {
                    impl_p = "deepseek".to_string(); impl_m = "deepseek-chat".to_string();
                } else if has_anthropic {
                    impl_p = "anthropic".to_string(); impl_m = "claude-3-5-sonnet-20241022".to_string();
                } else if has_openai {
                    impl_p = "openai".to_string(); impl_m = "gpt-4o".to_string();
                } else {
                    impl_p = "gemini".to_string(); impl_m = "gemini-2.0-flash".to_string();
                }

                // QA: Best testing/verification
                if has_anthropic {
                    qa_p = "anthropic".to_string(); qa_m = "claude-3-5-sonnet-20241022".to_string();
                } else if has_openai {
                    qa_p = "openai".to_string(); qa_m = "gpt-4o".to_string();
                } else if has_deepseek {
                    qa_p = "deepseek".to_string(); qa_m = "deepseek-chat".to_string();
                } else {
                    qa_p = "gemini".to_string(); qa_m = "gemini-2.0-flash".to_string();
                }

                // Doc: Gemini Flash (fastest, near-zero cost) if available, else DeepSeek/Anthropic/OpenAI
                if has_gemini {
                    doc_p = "gemini".to_string(); doc_m = "gemini-2.0-flash".to_string();
                } else if has_deepseek {
                    doc_p = "deepseek".to_string(); doc_m = "deepseek-chat".to_string();
                } else if has_anthropic {
                    doc_p = "anthropic".to_string(); doc_m = "claude-3-5-sonnet-20241022".to_string();
                } else {
                    doc_p = "openai".to_string(); doc_m = "gpt-4o".to_string();
                }
            }

            HarmonyTierProfile::Flagship => {
                let (p, m) = if has_anthropic {
                    ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
                } else if has_openai {
                    ("openai".to_string(), "gpt-4o".to_string())
                } else if has_deepseek {
                    ("deepseek".to_string(), "deepseek-chat".to_string())
                } else {
                    ("gemini".to_string(), "gemini-2.0-flash".to_string())
                };
                arch_p = p.clone(); arch_m = m.clone();
                impl_p = p.clone(); impl_m = m.clone();
                qa_p = p.clone(); qa_m = m.clone();
                doc_p = p; doc_m = m;
            }

            HarmonyTierProfile::Economy => {
                let (p, m) = if has_gemini {
                    ("gemini".to_string(), "gemini-2.0-flash".to_string())
                } else if has_deepseek {
                    ("deepseek".to_string(), "deepseek-chat".to_string())
                } else if has_openai {
                    ("openai".to_string(), "gpt-4o".to_string())
                } else {
                    ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
                };
                arch_p = p.clone(); arch_m = m.clone();
                impl_p = p.clone(); impl_m = m.clone();
                qa_p = p.clone(); qa_m = m.clone();
                doc_p = p; doc_m = m;
            }
        }
    } else if has_ollama {
        // Local Ollama intelligent rotation
        let installed = crate::providers::ollama::OllamaProvider::discover_installed_models();
        let has_qwen = installed.iter().any(|m| m.contains("qwen2.5:0.5b") || m.contains("qwen"));
        let has_dolphin = installed.iter().any(|m| m.contains("dolphin-phi") || m.contains("phi"));

        if has_qwen && has_dolphin {
            arch_p = "ollama".to_string(); arch_m = "qwen2.5:0.5b".to_string();
            impl_p = "ollama".to_string(); impl_m = "dolphin-phi:latest".to_string();
            qa_p = "ollama".to_string(); qa_m = "qwen2.5:0.5b".to_string();
            doc_p = "ollama".to_string(); doc_m = "dolphin-phi:latest".to_string();
        } else if let Some(first) = installed.first() {
            arch_p = "ollama".to_string(); arch_m = first.clone();
            impl_p = "ollama".to_string(); impl_m = first.clone();
            qa_p = "ollama".to_string(); qa_m = first.clone();
            doc_p = "ollama".to_string(); doc_m = first.clone();
        } else {
            let def = crate::providers::ollama::default_ollama_model();
            arch_p = "ollama".to_string(); arch_m = def.clone();
            impl_p = "ollama".to_string(); impl_m = def.clone();
            qa_p = "ollama".to_string(); qa_m = def.clone();
            doc_p = "ollama".to_string(); doc_m = def;
        }
    } else if has_mock {
        let def = "mock".to_string();
        arch_p = "mock".to_string(); arch_m = def.clone();
        impl_p = "mock".to_string(); impl_m = def.clone();
        qa_p = "mock".to_string(); qa_m = def.clone();
        doc_p = "mock".to_string(); doc_m = def;
    } else {
        // Fallback default
        let def = "qwen2.5:0.5b".to_string();
        arch_p = "ollama".to_string(); arch_m = def.clone();
        impl_p = "ollama".to_string(); impl_m = def.clone();
        qa_p = "ollama".to_string(); qa_m = def.clone();
        doc_p = "ollama".to_string(); doc_m = def;
    }

    // Helper closure to resolve overrides with ctx awareness
    let resolve_one = |o: &str, def_p: &str| -> (String, String) {
        let trimmed = o.trim();
        if let Some((prov, model)) = trimmed.split_once(':') {
            let prov_lower = prov.to_lowercase();
            if ctx.get_provider(prov).is_ok()
                || ctx.get_provider(&prov_lower).is_ok()
                || matches!(prov_lower.as_str(), "ollama" | "anthropic" | "openai" | "gemini" | "xai" | "deepseek" | "groq" | "mock")
            {
                (prov_lower, model.to_string())
            } else {
                (def_p.to_string(), trimmed.to_string())
            }
        } else {
            (def_p.to_string(), trimmed.to_string())
        }
    };

    // 2. Apply user-specified overrides
    if let Some(ref o) = overrides.architect {
        let (p, m) = resolve_one(o, &arch_p);
        arch_p = p; arch_m = m;
    }
    if let Some(ref o) = overrides.implementer {
        let (p, m) = resolve_one(o, &impl_p);
        impl_p = p; impl_m = m;
    }
    if let Some(ref o) = overrides.qa {
        let (p, m) = resolve_one(o, &qa_p);
        qa_p = p; qa_m = m;
    }
    if let Some(ref o) = overrides.doc {
        let (p, m) = resolve_one(o, &doc_p);
        doc_p = p; doc_m = m;
    }

    ((arch_p, arch_m), (impl_p, impl_m), (qa_p, qa_m), (doc_p, doc_m))
}
