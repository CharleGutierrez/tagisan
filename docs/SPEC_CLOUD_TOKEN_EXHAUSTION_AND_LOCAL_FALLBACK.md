# RFC-004: Cloud Token Exhaustion Handling & Zero-Cost Local Ollama Failover Architecture

- **Status**: Draft / Proposed for Implementation
- **Author**: Tagisan Core Engineering Team
- **Date**: 2026-09-11
- **Component**: `src/engine/budget.rs`, `src/providers/cascade.rs`, `src/swarm/harmony/`, `src/cli.rs`
- **Related Specs**: [RFC-003: Structured Role Harmony Swarm](SPEC_STRUCTURED_ROLE_HARMONY_SWARM.md), [SPEC_LOCAL_LLM_SKILL_DISPATCHING](SPEC_LOCAL_LLM_SKILL_DISPATCHING.md)

---

## 1. Executive Summary

When executing complex, multi-agent LLM workflows (such as the `StructuredRoleHarmonySwarm` assembly line) using Non-Local Cloud LLMs (Anthropic Claude, OpenAI GPT-4o, DeepSeek-V3, Google Gemini), token exhaustion can occur across three distinct dimensions:

1. **User Financial Budget Exhaustion**: The user-configured spending cap (USD) is reached mid-session.
2. **Model Output Limit Exhaustion**: The model exceeds its maximum generation length (`max_tokens`), truncating code or JSON mid-syntax (`finish_reason: "length"`).
3. **Cloud Provider API Quota / Rate Limits**: The cloud provider returns HTTP 429 (Tokens-Per-Minute / RateLimited), HTTP 402/403 (Insufficient Credits / Quota Exhausted), or HTTP 400 (Context Length Overflow).

This specification outlines Tagisan's multi-layered protection mechanism and specifies the **Zero-Cost Evacuation Architecture**, which automatically redirects stalled or exhausted cloud requests to locally running LLMs in Ollama (e.g. `qwen2.5:0.5b`, `dolphin-phi:latest`, `dolphin-mixtral:latest`).

---

## 2. The 3-Tier Protection Architecture

```
                                [ Incoming User Objective ]
                                             │
                       ┌─────────────────────┴─────────────────────┐
                       ▼                                           ▼
              [ Cloud LLM Stage ]                         [ Local LLM Stage ]
                       │                                           │
         ┌─────────────┼─────────────┐                             │
         ▼             ▼             ▼                             │
    [Tier 1: USD] [Tier 2: Length] [Tier 3: Rate/Quota]           │
      Exhausted     Truncated      HTTP 429 / 402                  │
         │             │             │                             │
         └─────────────┼─────────────┘                             │
                       ▼                                           │
         [ Zero-Cost Evacuation Gate ]                             │
                       │                                           │
                       └─────────────► [ Ollama Local Provider ] ◄─┘
                                             │
                                     ($0.00 Incremental Cost)
                                             │
                                             ▼
                                  [ Assembly Completed ]
```

### 2.1 Tier 1: Financial Budget Guardrail (`TokenBudgetTracker`)
- **Module**: `src/engine/budget.rs`
- **Mechanism**:
  - `TokenBudgetTracker` uses atomic integers (`AtomicU64`) to track spent micro-USD ($0.000001 precision).
  - Tracks `prompt_tokens`, `completion_tokens`, and applies 90% prompt caching discounts.
  - Prior Behavior: When cumulative micro-USD spent exceeds `max_budget_usd`, `record_micro_usd()` returns `Err(TagisanError::BudgetExceeded)`. The pipeline aborts immediately to prevent unexpected cloud billing runaways.
  - Enhanced Behavior under RFC-004: Cloud budget exhaustion triggers an **Evacuation Policy**, dynamically transferring pending stages to $0-cost local models instead of abandoning the task.

### 2.2 Tier 2: Model Output Truncation (`max_tokens` / Syntax Gate)
- **Module**: `src/swarm/harmony/gates.rs`
- **Mechanism**:
  - When cloud models hit generation limits, outputs are severed mid-syntax.
  - `SyntaxValidationGate` analyzes the output for unclosed markdown fences (```) and unbalanced delimiters (`{}`, `()`, `[]`).
  - Upon failure, the pipeline does not write bad code to disk. It triggers a structured critique retry:
    > *"Syntax validation failed: Unclosed code block or unbalanced braces. Output was truncated due to token limits. Output a modular, concise implementation."*
  - If retries fail, the stage is eligible for handoff to a local model with unconstrained local token limits.

### 2.3 Tier 3: Cloud Provider API & Rate Limits (HTTP 429 / 402 / 400)
- **Module**: `src/providers/cascade.rs` & `src/error.rs`
- **Mechanism**:
  - HTTP 429 maps to `TagisanError::RateLimited`, flagged as `is_retryable() == true`.
  - HTTP 402 / 500 maps to `TagisanError::BadResponse`, flagged as `is_retryable() == true`.
  - HTTP 400 (context length) maps to `TagisanError::ContextLengthExceeded`.
  - `CascadeProvider` catches retryable errors and transparently invokes the next provider in the chain.

---

## 3. Zero-Cost Evacuation to Local Ollama Specification

### 3.1 Economics of Local Failover
Local models hosted via Ollama report `estimated_cost_usd: Some(0.0)` in `TokenUsage`. Therefore:
- Transitioning from Cloud $\rightarrow$ Local adds **$0.00** to the user's financial budget.
- Local LLMs have no Tokens-Per-Minute (TPM) throttling or credit balances.

### 3.2 Provider-Level Cascade Chain
When configuring a provider, users can chain cloud models with a terminal local fallback:

```rust
let cascade = CascadeProvider::new(vec![
    CascadeEntry::new(anthropic_provider, Some("claude-3-5-sonnet-20241022".into())),
    CascadeEntry::new(deepseek_provider, Some("deepseek-chat".into())),
    CascadeEntry::new(ollama_provider, Some("qwen2.5:0.5b".into())), // Terminal zero-cost fallback
]);
```

### 3.3 Enhanced Failover Condition in `CascadeProvider`
Update `CascadeProvider::is_retryable` to allow failover on `BudgetExceeded` **if and only if** subsequent entries in the cascade chain are zero-cost local providers:

```rust
impl CascadeProvider {
    fn can_failover(&self, error: &TagisanError, next_entry_idx: usize) -> bool {
        match error {
            TagisanError::RateLimited(_, _) => true,
            TagisanError::BadResponse(_, _) => true,
            TagisanError::Network(_) => true,
            TagisanError::Authentication(_, _) => true,
            TagisanError::ProviderNotFound(_) => true,
            TagisanError::ContextLengthExceeded(_, _, _) => true,
            TagisanError::BudgetExceeded { .. } => {
                // If the next provider is Ollama/local (zero-cost), failover is permitted!
                if let Some(next_entry) = self.entries.get(next_entry_idx) {
                    next_entry.provider.provider_id() == "ollama"
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
```

---

## 4. Swarm Stage-Level Dynamic Hot-Swapping

In `StructuredRoleHarmonySwarm`, stages can independently hot-swap models mid-pipeline:

```
[ Stage 1: Lead Architect ]   ---> Ran on Claude 3.5 Sonnet (Success, used $0.02)
[ Stage 2: Implementer ]      ---> DeepSeek V3 hits HTTP 429 RateLimit!
                                    └── Dynamic Stage Failover: Rerouted to Ollama `dolphin-phi:latest`
[ Stage 3: QA Specialist ]    ---> Budget exhausted ($0 remaining)!
                                    └── Zero-Cost Evacuation: Dispatched to Ollama `qwen2.5:0.5b`
[ Stage 4: Documentation ]    ---> Zero-Cost Evacuation: Dispatched to Ollama `qwen2.5:0.5b`
                                    └── Swarm Assembly Successfully Finalized on Blackboard!
```

### 4.1 Swarm Stage Handoff Protocol
1. Each stage records its completed artifact to the shared `SwarmBlackboard`.
2. If stage $N$ fails due to token or rate exhaustion, the pipeline queries `OllamaProvider::discover_installed_models()`.
3. The failed stage is re-instantiated with the best matching local model:
   - Coding / Architecture: `dolphin-phi:latest` or `qwen2.5:0.5b`
   - QA / Documentation: `qwen2.5:0.5b`
4. The stage executes locally using the existing blackboard context without losing work from previous stages.

---

## 5. CLI & Configuration Flags

Add CLI options to control fallback behavior:

```bash
# Enable automatic local fallback if cloud hits rate limits or token exhaustion
tgs harmony "Build a high-performance LRU cache" --fallback-to-local

# Enable zero-cost evacuation if USD budget limit is reached
tgs harmony "Build a microservice" --budget 0.50 --evacuate-on-budget

# Explicitly specify hybrid cloud-to-local cascade chain
tgs harmony "Build a compiler" --architect anthropic:claude-3-5-sonnet --implementer deepseek:deepseek-chat --qa ollama:qwen2.5:0.5b --doc ollama:dolphin-phi:latest
```

---

## 6. Implementation Roadmap

1. **Step 1**: Update `CascadeProvider` with zero-cost detection for `BudgetExceeded` failover.
2. **Step 2**: Implement `--fallback-to-local` and `--evacuate-on-budget` flags in `src/cli.rs`.
3. **Step 3**: Update `StructuredHarmonyPipeline::execute_stage_with_retries` with dynamic stage hot-swapping into Ollama.
4. **Step 4**: Add comprehensive integration tests in `tests/cloud_local_failover_tests.rs`.
