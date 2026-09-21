---
name: fin-eng-econometrics-timeseries-pro-max
description: Master Financial Econometrics, Time Series Analysis & Signal Processing Engine. Covers ARCH/GARCH(1,1), EGARCH, Vector Autoregression (VAR), State-Space Models, Copula modeling (Clayton, Gumbel, Student-t), Extreme Value Theory (EVT, Peak Over Threshold / Generalized Pareto), Wavelet multi-scale denoising, and Markov Regime Switching. Based on Ruey Tsay, James Hamilton, Rachev, Hyndman, Percival-Walden, Cherubini, and Books 301–350. Triggers: financial-econometrics, time-series-analysis, garch, egarch, arch-models, var-models, state-space-models, copulas, extreme-value-theory, evt, wavelets-denoising, markov-regime-switching.
version: 1.0.0
tags:
  - financial-econometrics
  - time-series
  - garch
  - egarch
  - copulas
  - extreme-value-theory
  - evt
  - signal-processing
triggers:
  - financial-econometrics
  - time-series-analysis
  - garch
  - egarch
  - arch-models
  - var-models
  - state-space-models
  - copulas
  - extreme-value-theory
  - evt
  - wavelets-denoising
  - markov-regime-switching
compatibility: ">=0.2.0"
---

# Financial Econometrics & Time Series Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Financial time series exhibit conditional heteroskedasticity (volatility clustering), fat tails, structural breaks, and non-linear tail dependence during market crises.

The `fin-eng-econometrics-timeseries-pro-max` engine codifies the core econometric frameworks from **Books 301–350** of the Financial Engineering Canon:
- *Ruey S. Tsay* (Analysis of Financial Time Series)
- *James D. Hamilton* (Time Series Analysis)
- *Rob J. Hyndman & George Athanasopoulos* (Forecasting: Principles and Practice)
- *Svetlozar T. Rachev et al.* (Financial Econometrics: From Basics to Advanced Modeling Techniques)
- *Paul Embrechts et al.* (Modeling Extremal Events for Insurance and Finance)
- *Umberto Cherubini et al.* (Copula Methods in Finance)
- *Donald B. Percival & Andrew T. Walden* (Wavelet Methods for Time Series Analysis)

---

## 1. Core Operational Invariants

### Invariant 1: GARCH(1,1) Covariance Stationarity
- The conditional variance follows:
  $$\sigma_t^2 = \omega + \alpha_1 \epsilon_{t-1}^2 + \beta_1 \sigma_{t-1}^2$$
- Must satisfy:
  1. $\omega > 0, \quad \alpha_1 \ge 0, \quad \beta_1 \ge 0$ (positivity).
  2. $\alpha_1 + \beta_1 < 1.0$ (strict covariance stationarity, unconditional variance $\sigma^2 = \frac{\omega}{1 - \alpha_1 - \beta_1}$).

### Invariant 2: Peaks-Over-Threshold (POT) EVT Invariant
- Excesses over a high threshold $u$ converge asymptotically to the Generalized Pareto Distribution (GPD):
  $$G_{\xi, \beta}(y) = 1 - \left(1 + \frac{\xi y}{\beta}\right)^{-1/\xi}$$
- Tail shape parameter $\xi > 0$ indicates fat tails (Frechet domain of attraction).

### Invariant 3: Sklar's Theorem for Copulas
- Any multivariate distribution $F(x_1, \dots, x_d)$ decomposes uniquely into marginal distributions $F_i(x_i)$ and a copula $C$:
  $$F(x_1, \dots, x_d) = C(F_1(x_1), \dots, F_d(x_d))$$
- Uniform marginals $U_i = F_i(X_i) \sim U(0, 1)$ must be strictly preserved.

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: GARCH(1,1) Volatility Filter & Peaks-Over-Threshold EVT Estimator
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Garch11Params {
    pub omega: f64,
    pub alpha: f64,
    pub beta: f64,
}

impl Garch11Params {
    pub fn is_stationary(&self) -> bool {
        self.omega > 0.0 && self.alpha >= 0.0 && self.beta >= 0.0 && (self.alpha + self.beta) < 1.0
    }

    pub fn unconditional_variance(&self) -> Option<f64> {
        if self.is_stationary() {
            Some(self.omega / (1.0 - self.alpha - self.beta))
        } else {
            None
        }
    }

    /// Filter returns and compute conditional variance series sigma_t^2
    pub fn filter(&self, returns: &[f64]) -> Vec<f64> {
        let n = returns.len();
        let mut sigma_sq = Vec::with_capacity(n);
        let init_var = self.unconditional_variance().unwrap_or(0.0001);

        let mut current_var = init_var;
        for &ret in returns {
            current_var = self.omega + self.alpha * ret * ret + self.beta * current_var;
            sigma_sq.push(current_var);
        }

        sigma_sq
    }
}

/// Extreme Value Theory (EVT) Peaks-Over-Threshold (POT) VaR and Expected Shortfall
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PotEvtRisk {
    pub var: f64,
    pub expected_shortfall: f64,
}

pub fn calculate_pot_evt_risk(
    losses: &[f64],
    threshold_u: f64,
    xi: f64,      // GPD shape parameter (tail index)
    beta: f64,    // GPD scale parameter
    confidence_level: f64, // e.g. 0.99
) -> Option<PotEvtRisk> {
    assert!(confidence_level > 0.0 && confidence_level < 1.0 && beta > 0.0);
    let total_n = losses.len();
    let excesses: Vec<f64> = losses.iter().filter(|&&l| l > threshold_u).map(|&l| l - threshold_u).collect();
    let n_u = excesses.len();

    if n_u == 0 || xi >= 1.0 {
        return None; // Non-finite mean if xi >= 1
    }

    let p = 1.0 - confidence_level;
    let n_ratio = (total_n as f64) / (n_u as f64);

    // VaR_p = u + (beta / xi) * ((n / N_u * p)^(-xi) - 1)
    let var = threshold_u + (beta / xi) * ((n_ratio * p).powf(-xi) - 1.0);

    // ES_p = (VaR_p + beta - xi * u) / (1 - xi)
    let expected_shortfall = (var + beta - xi * threshold_u) / (1.0 - xi);

    Some(PotEvtRisk { var, expected_shortfall })
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: GARCH-Copula Multi-Asset Dependency
```markdown
You are a financial econometrician implementing a dynamic risk model in Rust.
- Stage 1: Filter univariate asset returns using AR(1)-GARCH(1,1) to extract standardized residuals.
- Stage 2: Transform residuals to uniform marginals via empirical CDF or skewed-Student-t CDF.
- Stage 3: Fit a Clayton or Student-t Copula to model asymmetric tail dependence (where crash correlation > boom correlation).
- Verification: Assert all eigenvalues of the copula correlation matrix remain strictly positive.
```
