---
name: fin-eng-portfolio-factor-investing-pro-max
description: Master Quantitative Portfolio Management, Factor Investing & Asset Allocation Engine. Covers Markowitz Mean-Variance Optimization, Black-Litterman model, Risk Parity, Hierarchical Risk Parity (HRP), Barra multi-factor models, Fundamental Law of Active Management, and Downside Risk (Sortino). Based on Grinold-Kahn, Chincarini-Kim, Swensen, Markowitz, Qian, Ilmanen, Meucci, and Books 101–150. Triggers: portfolio-management, factor-investing, markowitz-optimization, black-litterman, risk-parity, hierarchical-risk-parity, barra-factors, mean-variance, fundamental-law-active-management, smart-beta, information-ratio.
version: 1.0.0
tags:
  - portfolio-management
  - factor-investing
  - markowitz
  - black-litterman
  - risk-parity
  - hrp
  - smart-beta
triggers:
  - portfolio-management
  - factor-investing
  - markowitz-optimization
  - black-litterman
  - risk-parity
  - hierarchical-risk-parity
  - barra-factors
  - mean-variance
  - fundamental-law-active-management
  - smart-beta
  - information-ratio
compatibility: ">=0.2.0"
---

# Quantitative Portfolio Management & Factor Investing Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Systematic asset allocation balances alpha generation, factor exposure risk, and diversification across non-correlated asset classes under rigorous transaction cost and turnover constraints.

The `fin-eng-portfolio-factor-investing-pro-max` engine codifies the core theory and optimization algorithms from **Books 101–150** of the Financial Engineering Canon:
- *Richard C. Grinold & Ronald N. Kahn* (Active Portfolio Management)
- *Harry M. Markowitz* (Portfolio Selection: Efficient Diversification of Investments)
- *Edward E. Qian* (Risk Parity Fundamentals)
- *David F. Swensen* (Pioneering Portfolio Management)
- *Antti Ilmanen* (Expected Returns)
- *Attilio Meucci* (Risk and Asset Allocation)
- *Ludwig B. Chincarini & Daehwan Kim* (Quantitative Equity Portfolio Management)

---

## 1. Core Operational Invariants

### Invariant 1: Budget & Solvency Constraint
- All asset weights must strictly sum to unity:
  $$\sum_{i=1}^N w_i = 1.0, \quad \left| \sum w_i - 1.0 \right| < 10^{-12}$$
- In long-only mandates: $w_i \ge 0 \quad \forall i$.

### Invariant 2: Fundamental Law of Active Management
- The Information Ratio ($IR$) achieved by an active manager is governed by the Information Coefficient ($IC$) and strategy Breadth ($BR$):
  $$IR = IC \times \sqrt{BR}$$
- Transfer Coefficient ($TC$) discounts real-world constrained implementations: $IR = TC \times IC \times \sqrt{BR}$.

### Invariant 3: Equal Risk Contribution (Risk Parity)
- In a risk-parity allocation, each asset's marginal contribution to portfolio risk must be equal:
  $$RC_i = w_i \frac{(\Sigma w)_i}{\sigma_p} = \frac{\sigma_p}{N} \quad \forall i$$
- Total risk decomposes completely via Euler's homogeneous function theorem: $\sum_{i=1}^N RC_i = \sigma_p$.

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: Risk Parity (Equal Risk Contribution) Coordinate Descent Solver
```rust
pub const MAX_ASSETS: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioWeights {
    pub weights: [f64; MAX_ASSETS],
    pub asset_count: usize,
    pub portfolio_volatility: f64,
}

/// Equal Risk Contribution (Risk Parity) cyclical coordinate descent solver
pub fn solve_risk_parity(cov: &[[f64; MAX_ASSETS]; MAX_ASSETS], n: usize, max_iter: usize, tol: f64) -> Option<PortfolioWeights> {
    assert!(n <= MAX_ASSETS && n > 0);

    // Initial equal weights
    let mut w = [1.0 / (n as f64); MAX_ASSETS];
    let target_rc = 1.0 / (n as f64);

    for _ in 0..max_iter {
        let mut max_diff = 0.0f64;

        // Compute portfolio variance w^T * Sigma * w
        let mut port_var = 0.0;
        let mut sigma_w = [0.0f64; MAX_ASSETS];
        for i in 0..n {
            for j in 0..n {
                sigma_w[i] += cov[i][j] * w[j];
            }
            port_var += w[i] * sigma_w[i];
        }
        let port_vol = port_var.sqrt();

        for i in 0..n {
            // Marginal risk contribution: w_i * (Sigma * w)_i / sigma_p
            let rc_i = w[i] * sigma_w[i] / (port_var + 1e-15);
            let diff = (rc_i - target_rc).abs();
            if diff > max_diff {
                max_diff = diff;
            }

            // Newton update for weight w_i
            let a = cov[i][i];
            let b = sigma_w[i] - w[i] * cov[i][i];
            let c = -target_rc * port_var;

            // Solve a * w_i^2 + b * w_i + c = 0
            let discriminant = b * b - 4.0 * a * c;
            if discriminant >= 0.0 {
                let w_new = (-b + discriminant.sqrt()) / (2.0 * a);
                if w_new > 0.0 {
                    w[i] = w_new;
                }
            }
        }

        // Renormalize weights
        let sum_w: f64 = w[0..n].iter().sum();
        for i in 0..n {
            w[i] /= sum_w;
        }

        if max_diff < tol {
            let mut final_var = 0.0;
            for i in 0..n {
                let mut row_sum = 0.0;
                for j in 0..n {
                    row_sum += cov[i][j] * w[j];
                }
                final_var += w[i] * row_sum;
            }

            return Some(PortfolioWeights {
                weights: w,
                asset_count: n,
                portfolio_volatility: final_var.sqrt(),
            });
        }
    }

    None // Did not converge within tolerance
}

/// Calculate Grinold-Kahn Information Ratio & Sizing
#[inline(always)]
pub fn calculate_fundamental_law(ic: f64, breadth: f64, transfer_coeff: f64) -> f64 {
    transfer_coeff * ic * breadth.sqrt()
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: Hierarchical Risk Parity (HRP)
```markdown
You are a quantitative portfolio architect implementing Marcos López de Prado's Hierarchical Risk Parity (HRP).
- Inputs: Correlation matrix C (NxN) and variance vector V (N).
- Stage 1 (Tree Clustering): Compute distance matrix D_i,j = sqrt(0.5 * (1 - C_i,j)). Form hierarchical cluster tree using Single Linkage.
- Stage 2 (Quasi-Diagonalization): Reorder the covariance matrix rows/columns to place correlated assets along the diagonal.
- Stage 3 (Recursive Bisection): Recursively bisect the tree and allocate weights inversely proportional to cluster variances.
- Invariants: Zero allocations in inner loops, output weights sum strictly to 1.0.
```
