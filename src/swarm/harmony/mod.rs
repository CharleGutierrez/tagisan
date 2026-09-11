pub mod blackboard;
pub mod gates;
pub mod pipeline;
pub mod roles;
pub mod types;

pub use blackboard::SwarmBlackboard;
pub use gates::{AgentShieldSecurityGate, GateResult, SyntaxValidationGate, ValidationGate};
pub use pipeline::{
    render_failover_banner, send_failover_desktop_notification,
    HarmonyExecutionResult, HarmonyStage, StructuredHarmonyPipeline,
};
pub use roles::{
    extract_markdown_code_blocks, parse_provider_and_model, resolve_harmony_models,
    AssemblyRoles, RoleModelOverrides, StandardHarmonyRole,
};
pub use types::{
    ExtractedCodeBlock, FailoverEvent, HarmonyRole, HarmonyRoleConfig, HarmonyTierProfile, RoleArtifact,
};

use crate::engine::EngineContext;

/// Builds a production-ready 4-stage assembly line with syntax & security gates.
pub fn build_standard_harmony_pipeline(
    objective: &str,
    ctx: &EngineContext,
    overrides: &RoleModelOverrides,
    audit: bool,
) -> StructuredHarmonyPipeline {
    let (arch, imp, qa, doc) = resolve_harmony_models(ctx, overrides);

    let mut pipeline = StructuredHarmonyPipeline::new(objective);

    // Automatically enable concurrent parallel execution of Stage 3 (QA) and Stage 4 (Doc)
    // when using Non-Local Cloud LLMs to slash pipeline turnaround latency.
    let is_qa_cloud = matches!(qa.0.as_str(), "anthropic" | "openai" | "gemini" | "deepseek" | "xai" | "groq");
    let is_doc_cloud = matches!(doc.0.as_str(), "anthropic" | "openai" | "gemini" | "deepseek" | "xai" | "groq");
    if is_qa_cloud && is_doc_cloud {
        pipeline = pipeline.with_parallel(true);
    }

    // Stage 1: Lead Systems Architect
    let stage1 = HarmonyStage::new(AssemblyRoles::architect(&arch.0, &arch.1))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()))
        .with_gate(Box::new(AgentShieldSecurityGate::new()));
    pipeline = pipeline.add_stage(stage1);

    // Stage 2: Senior Systems Implementer
    let stage2 = HarmonyStage::new(AssemblyRoles::implementer(&imp.0, &imp.1))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()))
        .with_gate(Box::new(AgentShieldSecurityGate::new()));
    pipeline = pipeline.add_stage(stage2);

    // Stage 3: QA & Test Specialist
    let stage3 = HarmonyStage::new(AssemblyRoles::qa(&qa.0, &qa.1))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    pipeline = pipeline.add_stage(stage3);

    // Stage 4: Documentation & Packaging
    let stage4 = HarmonyStage::new(AssemblyRoles::documentation(&doc.0, &doc.1))
        .with_gate(Box::new(SyntaxValidationGate::permissive()));
    pipeline = pipeline.add_stage(stage4);

    // Optional Adversarial Audit
    if audit {
        pipeline = pipeline.with_audit((qa.0.clone(), qa.1.clone()));
    }

    pipeline
}
