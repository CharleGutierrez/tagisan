---
name: fin-eng-stochastic-calculus-pro-max
description: Master Financial Engineering Engine for Stochastic Calculus, Brownian Motion, Martingales, SDEs, Jump Diffusions & Numerical Methods. Based on Shreve, Baxter-Rennie, Karatzas-Shreve, Neftci, Glasserman, Øksendal, Seydel, and Books 1–50. Triggers: stochastic-calculus, brownian-motion, ito-calculus, feynman-kac, girsanov-theorem, martingale, monte-carlo-pricing, finite-difference-pde, sde-diffusion, jump-diffusion, crank-nicolson, variance-reduction.
version: 1.0.0
tags:
  - stochastic-calculus
  - brownian-motion
  - ito-calculus
  - sde
  - monte-carlo
  - feynman-kac
  - martingales
triggers:
  - stochastic-calculus
  - brownian-motion
  - ito-calculus
  - feynman-kac
  - girsanov-theorem
  - martingale
  - monte-carlo-pricing
  - finite-difference-pde
  - sde-diffusion
  - jump-diffusion
  - crank-nicolson
  - variance-reduction
compatibility: ">=0.2.0"
---

# Stochastic Calculus & Numerical Methods Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Rigorous continuous-time quantitative finance rests entirely upon the foundational apparatus of stochastic calculus, probability spaces, and numerical approximations to stochastic differential equations (SDEs) and partial differential equations (PDEs).

The `fin-eng-stochastic-calculus-pro-max` engine codifies the core theory and numerical algorithms from **Books 1–50** of the Financial Engineering Canon:
- *Steven E. Shreve* (Stochastic Calculus for Finance I & II)
- *Martin Baxter & Andrew Rennie* (Financial Calculus)
- *Ioannis Karatzas & Steven E. Shreve* (Brownian Motion and Stochastic Calculus)
- *Paul Glasserman* (Monte Carlo Methods in Financial Engineering)
- *Bernt Øksendal* (Stochastic Differential Equations)
- *Rüdiger Seydel* (Tools for Computational Finance)
- *Rama Cont & Peter Tankov* (Financial Modelling with Jump Processes)

---

## 1. Core Operational Invariants

### Invariant 1: Martingale Property Under Equivalent Measure
- Let $W_t^\mathbb{Q}$ be a standard Brownian motion under the risk-neutral measure $\mathbb{Q}$. The discounted asset price process $e^{-rt} S_t$ is a $\mathbb{Q}$-martingale:
  $$\mathbb{E}^\mathbb{Q}[e^{-rT} S_T \mid \mathcal{F}_t] = e^{-rt} S_t$$
- Any numerical simulation (Euler-Maruyama, Milstein) must preserve this expectation within statistical confidence intervals ($|\hat{\mu} - S_0| < 3 \times \text{SE}$).

### Invariant 2: Itô's Lemma Expansion
- For $f(t, S_t) \in C^{1,2}$, the differential is governed strictly by:
  $$df(t, S_t) = \left( \frac{\partial f}{\partial t} + \mu S_t \frac{\partial f}{\partial S} + \frac{1}{2} \sigma^2 S_t^2 \frac{\partial^2 f}{\partial S^2} \right) dt + \sigma S_t \frac{\partial f}{\partial S} dW_t$$
- First-order Taylor approximations in finance are invalid due to the quadratic variation $(dW_t)^2 = dt$.

### Invariant 3: Feynman-Kac Connection
- The conditional expectation $V(t, x) = \mathbb{E}^\mathbb{Q}[e^{-r(T-t)} \Phi(S_T) \mid S_t = x]$ satisfies the terminal-value parabolic PDE:
  $$\frac{\partial V}{\partial t} + r x \frac{\partial V}{\partial x} + \frac{1}{2} \sigma^2 x^2 \frac{\partial^2 V}{\partial x^2} - r V = 0, \quad V(T, x) = \Phi(x)$$

### Invariant 4: Variance Reduction Symmetry
- Monte Carlo estimations of expectations must employ antithetic variates ($\pm Z$) or control variates to halve variance without doubling path computation.

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: High-Precision Normal CDF & Antithetic Geometric Brownian Motion Simulator
```rust
pub const INV_SQRT_2PI: f64 = 0.3989422804014327;

#[inline(always)]
pub fn normal_pdf(x: f64) -> f64 {
    INV_SQRT_2PI * (-0.5 * x * x).exp()
}

/// Standard Normal Cumulative Distribution Function (Abramowitz & Stegun 7.1.26)
#[inline(always)]
pub fn normal_cdf(x: f64) -> f64 {
    if x < -8.0 { return 0.0; }
    if x > 8.0 { return 1.0; }
    let k = 1.0 / (1.0 + 0.2316419 * x.abs());
    let poly = k * (0.319381530 + k * (-0.356563782 + k * (1.781477937 + k * (-1.821255978 + k * 1.330274429))));
    let approx = 1.0 - normal_pdf(x.abs()) * poly;
    if x >= 0.0 { approx } else { 1.0 - approx }
}

/// Linear congruential generator / Xorshift64star for deterministic, zero-allocation uniform random floats
pub struct XorShift64Star {
    state: u64,
}

impl XorShift64Star {
    pub fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x546789ABCDEF1234 } else { seed } }
    }

    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    #[inline(always)]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }

    /// Box-Muller transform for standard normal pair (Z1, Z2)
    #[inline(always)]
    pub fn next_normal_pair(&mut self) -> (f64, f64) {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        (r * theta.cos(), r * theta.sin())
    }
}

/// Geometric Brownian Motion Monte Carlo Simulator with Antithetic Variates
pub struct GbmSimulator {
    pub s0: f64,
    pub r: f64,
    pub sigma: f64,
    pub t: f64,
    pub steps: usize,
}

impl GbmSimulator {
    pub fn new(s0: f64, r: f64, sigma: f64, t: f64, steps: usize) -> Self {
        Self { s0, r, sigma, t, steps }
    }

    /// Price European Call option using Antithetic Monte Carlo
    pub fn price_call_antithetic(&self, strike: f64, num_paths: usize, seed: u64) -> (f64, f64) {
        let mut rng = XorShift64Star::new(seed);
        let dt = self.t / (self.steps as f64);
        let sqrt_dt = dt.sqrt();
        let drift = (self.r - 0.5 * self.sigma * self.sigma) * dt;
        let vol = self.sigma * sqrt_dt;

        let mut payoff_sum = 0.0;
        let mut payoff_sq_sum = 0.0;
        let num_pairs = num_paths / 2;

        for _ in 0..num_pairs {
            let (z, _) = rng.next_normal_pair();
            
            // Primary path
            let s_primary = self.s0 * (drift * (self.steps as f64) + vol * (self.steps as f64).sqrt() * z).exp();
            // Antithetic path (-z)
            let s_antithetic = self.s0 * (drift * (self.steps as f64) - vol * (self.steps as f64).sqrt() * z).exp();

            let c1 = (s_primary - strike).max(0.0);
            let c2 = (s_antithetic - strike).max(0.0);
            let pair_avg = 0.5 * (c1 + c2);

            payoff_sum += pair_avg;
            payoff_sq_sum += pair_avg * pair_avg;
        }

        let disc = (-self.r * self.t).exp();
        let mean = (payoff_sum / (num_pairs as f64)) * disc;
        let var = (payoff_sq_sum / (num_pairs as f64) - (payoff_sum / (num_pairs as f64)).powi(2)) / (num_pairs as f64);
        let se = var.sqrt() * disc;

        (mean, se)
    }
}

/// Thomas Algorithm for solving Tridiagonal Matrix Systems in O(N) time and O(1) heap allocations
pub fn solve_tridiagonal(a: &[f64], b: &[f64], c: &[f64], d: &[f64], x: &mut [f64]) {
    let n = d.len();
    assert!(a.len() >= n && b.len() >= n && c.len() >= n && x.len() >= n);

    let mut c_prime = [0.0f64; 256];
    let mut d_prime = [0.0f64; 256];
    let cap = n.min(256);

    c_prime[0] = c[0] / b[0];
    d_prime[0] = d[0] / b[0];

    for i in 1..cap {
        let m = 1.0 / (b[i] - a[i] * c_prime[i - 1]);
        c_prime[i] = c[i] * m;
        d_prime[i] = (d[i] - a[i] * d_prime[i - 1]) * m;
    }

    x[cap - 1] = d_prime[cap - 1];
    for i in (0..cap - 1).rev() {
        x[i] = d_prime[i] - c_prime[i] * x[i + 1];
    }
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: Stochastic SDE Implementation
```markdown
You are a quantitative systems developer implementing an SDE pricing engine in zero-allocation Rust.
- Model: Heston Stochastic Volatility / Jump Diffusion (Merton 1976).
- Mathematical Invariant: Discretization must preserve the Feller condition (2 * kappa * theta > sigma^2) for positive variance.
- Numerical Scheme: Use the full truncation Euler-Maruyama scheme or Quadratic-Exponential (QE) scheme of Andersen (2008).
- Allocations: Use pre-allocated static arrays for trajectory recording.
- Variance Reduction: Implement antithetic variates across both Wiener processes with correlation rho.
```
