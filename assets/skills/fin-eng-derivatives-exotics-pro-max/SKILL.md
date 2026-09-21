---
name: fin-eng-derivatives-exotics-pro-max
description: Master Derivative Pricing, Volatility Surface & Exotics Engine. Covers Black-Scholes-Merton, Local Volatility (Dupire), Heston Stochastic Volatility, SABR, SVI surface parameterization, Barrier/Lookback options, and Adjoint Algorithmic Differentiation (AAD). Based on Hull, Gatheral, Taleb, Kahl, Wilmott, Brigo-Mercurio, Savine, and Books 51–100. Triggers: derivatives-pricing, black-scholes, volatility-surface, heston-model, sabr-model, svi-surface, local-volatility, exotic-options, barrier-options, adjoint-algorithmic-differentiation, aad-greeks, dynamic-hedging.
version: 1.0.0
tags:
  - derivatives-pricing
  - black-scholes
  - volatility-surface
  - heston-model
  - sabr-model
  - svi
  - exotic-options
  - aad
triggers:
  - derivatives-pricing
  - black-scholes
  - volatility-surface
  - heston-model
  - sabr-model
  - svi-surface
  - local-volatility
  - exotic-options
  - barrier-options
  - adjoint-algorithmic-differentiation
  - aad-greeks
  - dynamic-hedging
compatibility: ">=0.2.0"
---

# Derivative Pricing, Volatility & Exotics Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Institutional derivatives trading requires precision pricing of vanilla and exotic structures, complete implied volatility smile modeling, and rapid calculation of high-order Greeks (Delta, Gamma, Vega, Theta, Vanna, Volga) with constant-time sensitivity via Adjoint Algorithmic Differentiation (AAD).

The `fin-eng-derivatives-exotics-pro-max` engine codifies the core theory and numerical algorithms from **Books 51–100** of the Financial Engineering Canon:
- *John C. Hull* (Options, Futures, and Other Derivatives)
- *Jim Gatheral* (The Volatility Surface: A Practitioner's Guide)
- *Nassim Nicholas Taleb* (Dynamic Hedging)
- *Antoine Savine* (Modern Computational Finance: AAD and Parallel Simulations)
- *Christian Kahl* (Option Valuation under Stochastic Volatility)
- *Paul Wilmott* (Paul Wilmott on Quantitative Finance)
- *Damiano Brigo & Fabio Mercurio* (Interest Rate Models)

---

## 1. Core Operational Invariants

### Invariant 1: Put-Call Parity Conservation
- For any European options on non-dividend paying underlying $S$:
  $$C(S, K, T) - P(S, K, T) = S - K e^{-rT}$$
- The violation margin $|(C - P) - (S - K e^{-rT})|$ must not exceed $10^{-12}$ in any numerical pricing engine.

### Invariant 2: Arbitrage-Free SVI Volatility Surface
- Total implied variance $w(k, T) = \sigma_{BS}^2(k, T) T$ parameterized by Gatheral's SVI:
  $$w(k) = a + b \left( \rho (k - m) + \sqrt{(k - m)^2 + \sigma^2} \right)$$
- Must satisfy:
  1. No Calendar Arbitrage: $\frac{\partial w}{\partial T} \ge 0$.
  2. No Butterfly Arbitrage: The risk-neutral density $g(k) = \left( 1 - \frac{k w'(k)}{2w(k)} \right)^2 - \frac{w'(k)^2}{4}\left(\frac{1}{w(k)} + \frac{1}{4}\right) + \frac{w''(k)}{2} \ge 0$ for all log-moneyness $k = \ln(K/F)$.

### Invariant 3: Dupire Local Volatility Uniqueness
- Under continuous frictionless diffusion, local volatility $\sigma_{loc}^2(K, T)$ is uniquely determined by the market call surface $C(K, T)$:
  $$\sigma_{loc}^2(K, T) = \frac{\frac{\partial C}{\partial T} + r K \frac{\partial C}{\partial K}}{\frac{1}{2} K^2 \frac{\partial^2 C}{\partial K^2}}$$
- The denominator $\frac{\partial^2 C}{\partial K^2} = e^{-rT} p(K)$ represents state price density and must remain strictly positive.

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: Complete Black-Scholes Greeks Engine with High-Order Sensitivities
```rust
pub const INV_SQRT_2PI: f64 = 0.3989422804014327;

#[inline(always)]
pub fn normal_pdf(x: f64) -> f64 {
    INV_SQRT_2PI * (-0.5 * x * x).exp()
}

#[inline(always)]
pub fn normal_cdf(x: f64) -> f64 {
    if x < -8.0 { return 0.0; }
    if x > 8.0 { return 1.0; }
    let k = 1.0 / (1.0 + 0.2316419 * x.abs());
    let poly = k * (0.319381530 + k * (-0.356563782 + k * (1.781477937 + k * (-1.821255978 + k * 1.330274429))));
    let approx = 1.0 - normal_pdf(x.abs()) * poly;
    if x >= 0.0 { approx } else { 1.0 - approx }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComprehensiveGreeks {
    pub call_price: f64,
    pub put_price: f64,
    pub delta_call: f64,
    pub delta_put: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta_call: f64,
    pub theta_put: f64,
    pub rho_call: f64,
    pub rho_put: f64,
    pub vanna: f64, // dVega / dS or dDelta / dSigma
    pub volga: f64, // dVega / dSigma
}

pub fn evaluate_complete_greeks(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> ComprehensiveGreeks {
    assert!(s > 0.0 && k > 0.0 && t > 0.0 && sigma > 0.0);
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let nd1 = normal_cdf(d1);
    let nd2 = normal_cdf(d2);
    let n_neg_d1 = normal_cdf(-d1);
    let n_neg_d2 = normal_cdf(-d2);
    let pdf_d1 = normal_pdf(d1);
    let disc = (-r * t).exp();

    let call_price = s * nd1 - k * disc * nd2;
    let put_price = k * disc * n_neg_d2 - s * n_neg_d1;

    let delta_call = nd1;
    let delta_put = nd1 - 1.0;
    let gamma = pdf_d1 / (s * sigma * sqrt_t);
    let vega = s * sqrt_t * pdf_d1;

    let theta_call = -(s * pdf_d1 * sigma) / (2.0 * sqrt_t) - r * k * disc * nd2;
    let theta_put = -(s * pdf_d1 * sigma) / (2.0 * sqrt_t) + r * k * disc * n_neg_d2;

    let rho_call = k * t * disc * nd2;
    let rho_put = -k * t * disc * n_neg_d2;

    let vanna = -pdf_d1 * d2 / sigma;
    let volga = vega * d1 * d2 / sigma;

    ComprehensiveGreeks {
        call_price,
        put_price,
        delta_call,
        delta_put,
        gamma,
        vega,
        theta_call,
        theta_put,
        rho_call,
        rho_put,
        vanna,
        volga,
    }
}

/// Gatheral's Stochastic Volatility Inspired (SVI) Raw Parameterization
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SviParameters {
    pub a: f64,     // Base variance level
    pub b: f64,     // Angle between asymptotes
    pub rho: f64,   // Skew / rotation (-1 < rho < 1)
    pub m: f64,     // Horizontal shift
    pub sigma: f64, // ATM curvature / smoothing
}

impl SviParameters {
    pub fn total_variance(&self, k: f64) -> f64 {
        let diff = k - self.m;
        self.a + self.b * (self.rho * diff + (diff * diff + self.sigma * self.sigma).sqrt())
    }

    /// Check Gatheral's no-arbitrage bounds
    pub fn is_arbitrage_free_bounds(&self) -> bool {
        if self.b < 0.0 || self.sigma <= 0.0 || self.rho.abs() >= 1.0 {
            return false;
        }
        if self.a + self.b * self.sigma * (1.0 - self.rho * self.rho).sqrt() < 0.0 {
            return false; // Negative minimum variance
        }
        true
    }
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: Volatility Surface Calibration
```markdown
You are a derivatives quant engineer calibrating an arbitrage-free volatility surface in Rust.
- Model: Gatheral's Quasi-Explicit SVI or SABR (Hagan 2002).
- Constraints: Ensure strictly zero butterfly arbitrage (density g(k) >= 0) and zero calendar arbitrage (dw/dT >= 0).
- Optimizer: Implement a bounded Levenberg-Marquardt or L-BFGS routine without heap allocations.
- Verification: Assert put-call parity to 1e-12 tolerance across all calibrated strikes and tenors.
```
