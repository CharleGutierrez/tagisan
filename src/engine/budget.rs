use crate::error::{Result, TagisanError};
use crate::types::TokenUsage;
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

    /// Record token usage with prompt caching discount and return current total USD spent
    pub fn record_with_cache(
        &self,
        model: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        cached_prompt_tokens: u32,
    ) -> Result<f64> {
        let (prompt_rate, completion_rate) = match model.to_lowercase() {
            m if m.contains("claude-3-5-sonnet") || m.contains("claude-3.5-sonnet") => (3.0, 15.0),
            m if m.contains("claude-3-5-haiku") => (0.8, 4.0),
            m if m.contains("grok-2") || m.contains("grok-3") => (2.0, 10.0),
            m if m.contains("gpt-4o-mini") => (0.15, 0.60),
            m if m.contains("gpt-4o") => (2.5, 10.0),
            m if m.contains("gemini-2.0-flash") || m.contains("gemini-1.5-flash") => (0.10, 0.40),
            m if m.contains("gemini-1.5-pro") || m.contains("gemini-2.0-pro") => (1.25, 5.00),
            m if m.contains("deepseek-r1") || m.contains("deepseek-chat") || m.contains("deepseek-v3") => (0.55, 2.19),
            m if m.contains("ollama")
                || m.contains("local")
                || m.contains("qwen")
                || m.contains("dolphin")
                || m.contains("llama")
                || m.contains("mistral")
                || m.contains("phi") => (0.0, 0.0),
            _ => (1.0, 3.0), // fallback estimate
        };

        // Cached tokens receive a 90% discount (0.1x of prompt rate)
        let uncached_prompt = prompt_tokens.saturating_sub(cached_prompt_tokens);
        let prompt_cost = (uncached_prompt as f64 * prompt_rate)
            + (cached_prompt_tokens as f64 * prompt_rate * 0.1);
        let completion_cost = completion_tokens as f64 * completion_rate;

        let cost_micro = (prompt_cost + completion_cost).round() as u64;
        self.record_micro_usd(cost_micro)
    }

    /// Record token usage without caching discount and return current total USD spent
    pub fn record(&self, model: &str, prompt_tokens: u32, completion_tokens: u32) -> Result<f64> {
        self.record_with_cache(model, prompt_tokens, completion_tokens, 0)
    }

    /// Convenient helper to record usage from a TokenUsage struct
    pub fn record_usage(&self, model: &str, usage: &TokenUsage) -> Result<f64> {
        if usage.estimated_cost_usd == Some(0.0) {
            return self.record_micro_usd(0);
        }
        if let Some(cached) = usage.cached_prompt_tokens {
            self.record_with_cache(model, usage.prompt_tokens, usage.completion_tokens, cached)
        } else {
            self.record(model, usage.prompt_tokens, usage.completion_tokens)
        }
    }

    /// Record an exact amount in micro-USD (1 micro-USD = $0.000001)
    pub fn record_micro_usd(&self, cost_micro: u64) -> Result<f64> {
        if cost_micro == 0 {
            return Ok(self.current_spent_usd());
        }

        let max_micro = if self.max_budget_usd < 0.0 {
            0
        } else {
            (self.max_budget_usd * 1_000_000.0).round() as u64
        };

        let mut current = self.total_micro_usd_spent.load(Ordering::SeqCst);
        loop {
            let new_micro = current.saturating_add(cost_micro);
            if new_micro > max_micro {
                let total_usd = new_micro as f64 / 1_000_000.0;
                return Err(TagisanError::BudgetExceeded {
                    max_budget: self.max_budget_usd,
                    current_spent: total_usd,
                });
            }
            match self.total_micro_usd_spent.compare_exchange_weak(
                current,
                new_micro,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    let total_usd = new_micro as f64 / 1_000_000.0;
                    return Ok(total_usd);
                }
                Err(actual) => current = actual,
            }
        }
    }

    /// Record direct cost in USD
    pub fn record_cost_usd(&self, cost_usd: f64) -> Result<f64> {
        if cost_usd <= 0.0 {
            return Ok(self.current_spent_usd());
        }
        let cost_micro = (cost_usd * 1_000_000.0).round() as u64;
        self.record_micro_usd(cost_micro)
    }

    pub fn current_spent_usd(&self) -> f64 {
        self.total_micro_usd_spent.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn max_budget_usd(&self) -> f64 {
        self.max_budget_usd
    }

    /// Check if the total budget has been exhausted
    pub fn is_exhausted(&self) -> bool {
        let max_micro = if self.max_budget_usd < 0.0 {
            0
        } else {
            (self.max_budget_usd * 1_000_000.0).round() as u64
        };
        self.total_micro_usd_spent.load(Ordering::Relaxed) >= max_micro
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_cost_never_exceeds_budget() {
        let tracker = TokenBudgetTracker::new(0.05);
        assert!(!tracker.is_exhausted());

        // Spend entire budget
        let spent = tracker.record_micro_usd(50_000).expect("exact budget spent");
        assert_eq!(spent, 0.05);
        assert!(tracker.is_exhausted());

        // Further cloud call must fail
        let err = tracker.record_micro_usd(1).unwrap_err();
        assert!(matches!(err, TagisanError::BudgetExceeded { .. }));

        // Zero-cost operations must NEVER fail, even when exhausted
        let zero_res = tracker.record_micro_usd(0);
        assert!(zero_res.is_ok(), "Zero-cost micro_usd must succeed when exhausted");
        assert_eq!(zero_res.unwrap(), 0.05);

        // Recording zero cost via model lookup (ollama / local)
        let local_res = tracker.record("ollama", 1000, 1000);
        assert!(local_res.is_ok(), "Ollama zero-cost tokens must succeed when exhausted");
        assert_eq!(local_res.unwrap(), 0.05);

        let local_cache_res = tracker.record_with_cache("local-model", 500, 500, 100);
        assert!(local_cache_res.is_ok());
    }

    #[test]
    fn test_zero_initial_budget_permits_zero_cost() {
        let tracker = TokenBudgetTracker::new(0.0);
        assert!(tracker.is_exhausted());

        // Zero cost succeeds
        assert!(tracker.record_micro_usd(0).is_ok());
        assert!(tracker.record("ollama", 100, 100).is_ok());

        // Non-zero fails
        assert!(tracker.record_micro_usd(1).is_err());
    }
}

