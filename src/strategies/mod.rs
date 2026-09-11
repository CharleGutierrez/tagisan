pub mod debate;
pub mod harmony;
pub mod moa;

pub use harmony::StructuredHarmonyStrategy;

use crate::engine::EngineContext;
use crate::error::Result;
use crate::types::{Message, TokenUsage};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct StrategyInput {
    pub prompt: String,
    pub system_instruction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IntermediateStep {
    pub step_name: String,
    pub provider: String,
    pub model: String,
    pub message: Message,
    pub latency: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct StrategyOutput {
    pub strategy_name: &'static str,
    pub final_answer: String,
    pub intermediate_steps: Vec<IntermediateStep>,
    pub total_usage: TokenUsage,
    pub total_cost_usd: f64,
    pub total_latency: std::time::Duration,
}

#[async_trait]
pub trait CollaborationStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(&self, input: StrategyInput, ctx: &EngineContext) -> Result<StrategyOutput>;
}
