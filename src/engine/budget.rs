use crate::error::{Result, TagisanError};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct TokenBudgetTracker {
    max_budget_usd: f64,
    total_micro_usd_spent: AtomicU64,
}

impl TokenBudgetTracker {
    pub fn new(max_budget_usd: f64) -> Self {
        Self {
            max_budget_usd,
            total_micro_usd_spent: AtomicU64::new(0),
        }
    }

    /// Record token usage and return current total USD spent
    pub fn record(&self, model: &str, prompt_tokens: u32, completion_tokens: u32) -> Result<f64> {
        let (prompt_rate, completion_rate) = match model.to_lowercase() {
            m if m.contains("claude-3-5-sonnet") || m.contains("claude-3.5-sonnet") => (3.0, 15.0),
            m if m.contains("claude-3-5-haiku") => (0.8, 4.0),
            m if m.contains("grok-2") || m.contains("grok-3") => (2.0, 10.0),
            m if m.contains("gpt-4o-mini") => (0.15, 0.60),
            m if m.contains("gpt-4o") => (2.5, 10.0),
            m if m.contains("gemini-2.0-flash") || m.contains("gemini-1.5-flash") => (0.10, 0.40),
            m if m.contains("gemini-1.5-pro") || m.contains("gemini-2.0-pro") => (1.25, 5.00),
            m if m.contains("deepseek-r1") || m.contains("deepseek-chat") || m.contains("deepseek-v3") => (0.55, 2.19),
            m if m.contains("ollama") || m.contains("local") => (0.0, 0.0),
            _ => (1.0, 3.0), // fallback estimate
        };

        let cost_micro = ((prompt_tokens as f64 * prompt_rate) + (completion_tokens as f64 * completion_rate)) as u64;
        let prev = self.total_micro_usd_spent.fetch_add(cost_micro, Ordering::SeqCst);
        let total_usd = (prev + cost_micro) as f64 / 1_000_000.0;

        if total_usd > self.max_budget_usd {
            return Err(TagisanError::BudgetExceeded {
                max_budget: self.max_budget_usd,
                current_spent: total_usd,
            });
        }

        Ok(total_usd)
    }

    pub fn current_spent_usd(&self) -> f64 {
        self.total_micro_usd_spent.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }
}
