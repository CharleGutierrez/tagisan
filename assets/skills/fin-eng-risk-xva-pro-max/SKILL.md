---
name: fin-eng-risk-xva-pro-max
description: Master Quantitative Risk Management, Stress Testing & XVA Engine. Covers Value at Risk (VaR), Expected Shortfall (CVaR/ES), Basel III/IV Fundamental Review of the Trading Book (FRTB), Credit Value Adjustment (CVA), Debit Value Adjustment (DVA), Funding Value Adjustment (FVA), Extreme Value Theory, and Fat-Tailed Distribution analysis. Based on McNeil-Frey-Embrechts, Hull, Crépey-Bielecki-Brigo, Jorion, Taleb, Bluhm-Overbeck, Gregory, and Books 351–400. Triggers: quantitative-risk-management, xva, cva, dva, fva, basel-iii, frtb, expected-shortfall, value-at-risk, fat-tails, stress-testing, counterparty-credit-risk, extreme-value-risk.
version: 1.0.0
tags:
  - quantitative-risk
  - xva
  - cva
  - frtb
  - expected-shortfall
  - value-at-risk
  - fat-tails
triggers:
  - quantitative-risk-management
  - xva
  - cva
  - dva
  - fva
  - basel-iii
  - frtb
  - expected-shortfall
  - value-at-risk
  - fat-tails
  - stress-testing
  - counterparty-credit-risk
  - extreme-value-risk
compatibility: ">=0.2.0"
---

# Quantitative Risk Management & XVA Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Modern prudential risk management requires coherent risk metrics under Basel III/IV FRTB, non-linear counterparty credit exposure adjustments (XVA: CVA, DVA, FVA, MVA, KVA), and robust statistical defense against fat-tailed black swan events.

The `fin-eng-risk-xva-pro-max` engine codifies the core theory and risk modeling apparatus from **Books 351–400** of the Financial Engineering Canon:
- *Alexander J. McNeil, Rüdiger Frey, & Paul Embrechts* (Quantitative Risk Management: Concepts, Techniques and Tools)
- *John C. Hull* (Risk Management and Financial Institutions)
- *Stéphane Crépey, Tomasz R. Bielecki, & Damiano Brigo* (Counterparty Risk and Funding: A Tale of Two Puzzles - XVA)
- *Philippe Jorion* (Value at Risk: The New Benchmark for Managing Financial Risk)
- *Nassim Nicholas Taleb* (Statistical Consequences of Fat Tails & Dynamic Hedging)
- *Jon Gregory* (The xVA Challenge: Counterparty Credit Risk, Funding, Collateral and Capital)

---

## 1. Core Operational Invariants

### Invariant 1: Coherence of Risk Measures (Artzner Axioms)
- Any regulatory risk measure $\rho$ must satisfy:
  1. Monotonicity: $X \le Y \implies \rho(X) \ge \rho(Y)$.
  2. Subadditivity: $\rho(X + Y) \le \rho(X) + \rho(Y)$ (diversification must never increase risk).
  3. Positive Homogeneity: $\rho(h X) = h \rho(X) \quad \forall h > 0$.
  4. Translation Invariance: $\rho(X + m) = \rho(X) - m$.
- Value at Risk (VaR) violates subadditivity for fat-tailed distributions. Expected Shortfall ($ES$) is coherent.

### Invariant 2: Expected Shortfall Dominance
- At any confidence level $\alpha \in (0, 1)$:
  $$ES_\alpha(X) \ge VaR_\alpha(X)$$
- Basel III FRTB replaces 99% 10-day VaR with 97.5% Expected Shortfall across liquidity horizons.

### Invariant 3: Unilateral CVA Formulation
- For counterparty default time $\tau$ with hazard rate $\lambda(t)$ and Recovery Rate $R$:
  $$\text{CVA} = (1 - R) \int_0^T D(0, t) \mathbb{E}^\mathbb{Q}[(V_t)^+] d\mathbb{Q}(\tau \le t)$$
- Netting sets with collateral threshold $H$ replace $(V_t)^+$ with $(V_t - H)^+$.

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: Multi-Model VaR, Expected Shortfall & CVA Simulator
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskMetrics {
    pub parametric_var: f64,
    pub historical_var: f64,
    pub expected_shortfall: f64,
}

pub fn calculate_portfolio_risk(
    losses: &mut [f64],
    confidence_level: f64, // e.g. 0.99
) -> RiskMetrics {
    assert!(losses.len() > 10 && confidence_level > 0.5 && confidence_level < 1.0);
    let n = losses.len();

    // 1. Mean and standard deviation
    let mean = losses.iter().sum::<f64>() / (n as f64);
    let var = losses.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / ((n - 1) as f64);
    let std_dev = var.sqrt();

    // Standard normal inverse for confidence level (e.g. 2.326 for 0.99)
    let z = if confidence_level >= 0.99 { 2.3263 } else { 1.64485 };
    let parametric_var = mean + z * std_dev;

    // 2. Historical VaR & Expected Shortfall
    losses.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let cutoff_idx = ((n as f64) * confidence_level).floor() as usize;
    let safe_idx = cutoff_idx.min(n - 1);
    let historical_var = losses[safe_idx];

    let tail_losses = &losses[safe_idx..n];
    let expected_shortfall = if !tail_losses.is_empty() {
        tail_losses.iter().sum::<f64>() / (tail_losses.len() as f64)
    } else {
        historical_var
    };

    RiskMetrics {
        parametric_var,
        historical_var,
        expected_shortfall,
    }
}

/// Counterparty Credit Value Adjustment (CVA) Calculator
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CvaResult {
    pub cva: f64,
    pub expected_exposure_profile: [f64; 16],
}

pub fn calculate_cva(
    simulated_exposures: &[[f64; 16]], // Paths x TimeSteps
    default_probabilities: &[f64; 16], // Marginal default prob for each step
    recovery_rate: f64,                 // e.g. 0.40
    discount_factors: &[f64; 16],
) -> CvaResult {
    assert!(recovery_rate >= 0.0 && recovery_rate <= 1.0);
    let num_paths = simulated_exposures.len();
    assert!(num_paths > 0);

    let lgd = 1.0 - recovery_rate;
    let mut ee_profile = [0.0f64; 16];
    let mut cva = 0.0f64;

    for t in 0..16 {
        let mut positive_exposure_sum = 0.0;
        for path in simulated_exposures {
            positive_exposure_sum += path[t].max(0.0);
        }
        let ee = positive_exposure_sum / (num_paths as f64);
        ee_profile[t] = ee;

        // CVA contribution = LGD * DF(t) * EE(t) * PD(t)
        cva += lgd * discount_factors[t] * ee * default_probabilities[t];
    }

    CvaResult { cva, expected_exposure_profile: ee_profile }
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: FRTB Risk Engine Implementation
```markdown
You are a regulatory quantitative risk architect implementing Basel III FRTB rules in Rust.
- Scope: Compute Expected Shortfall at 97.5% confidence for non-modellable and modellable risk factors.
- Stressed Period: Calibrate historical shock scenarios to the 2008 Lehman crisis and March 2020 liquidity shock.
- Liquidity Horizons: Apply regulatory scale factors sqrt(LH_j / 10) across 10, 20, 40, 60, and 120-day liquidity horizons.
- Invariant: Guarantee subadditivity across all multi-desk netting sets.
```
