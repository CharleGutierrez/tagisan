---
name: fin-eng-algo-trading-statarb-pro-max
description: Master Algorithmic Trading, Statistical Arbitrage & Quantitative Execution Engine. Covers Cointegration (Engle-Granger & Johansen), Ornstein-Uhlenbeck mean-reversion drift fitting, Kalman Filter dynamic hedge ratio estimation, Almgren-Chriss optimal execution, VWAP/TWAP schedules, and trend/momentum filters. Based on Ernie Chan, Narang, Pole, Vidyamurthy, Davey, Carver, Cartea-Jaimungal, and Books 151–200. Triggers: algo-trading, statistical-arbitrage, statarb, pairs-trading, cointegration, engle-granger, johansen-test, ornstein-uhlenbeck, kalman-filter-hedge-ratio, mean-reversion, momentum-trading, execution-algorithms.
version: 1.0.0
tags:
  - algo-trading
  - statistical-arbitrage
  - pairs-trading
  - cointegration
  - kalman-filter
  - ornstein-uhlenbeck
  - almgren-chriss
triggers:
  - algo-trading
  - statistical-arbitrage
  - statarb
  - pairs-trading
  - cointegration
  - engle-granger
  - johansen-test
  - ornstein-uhlenbeck
  - kalman-filter-hedge-ratio
  - mean-reversion
  - momentum-trading
  - execution-algorithms
compatibility: ">=0.2.0"
---

# Algorithmic Trading & Statistical Arbitrage Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Statistical arbitrage extracts alpha from temporary pricing discrepancies across cointegrated economic assets, mean-reverting spreads, and microstructural inefficiencies while strictly managing inventory risk and market impact during trade execution.

The `fin-eng-algo-trading-statarb-pro-max` engine codifies the core theory and algorithmic execution frameworks from **Books 151–200** of the Financial Engineering Canon:
- *Ernie Chan* (Quantitative Trading & Algorithmic Trading)
- *Ganapathy Vidyamurthy* (Pairs Trading: Quantitative Methods and Analysis)
- *Andrew Pole* (Statistical Arbitrage: Algorithmic Trading Insights and Techniques)
- *Rishi K. Narang* (Inside the Black Box)
- *Álvaro Cartea, Sebastian Jaimungal, & José Penalva* (Algorithmic and High-Frequency Trading)
- *Robert Carver* (Systematic Trading)
- *Perry J. Kaufman* (Trading Systems and Methods)

---

## 1. Core Operational Invariants

### Invariant 1: Spread Stationarity via Cointegration
- Two non-stationary price series $Y_t \sim I(1)$ and $X_t \sim I(1)$ are cointegrated if there exists a cointegrating vector $[1, -\beta]$ such that:
  $$Z_t = Y_t - \beta X_t - \alpha \sim I(0)$$
- Spreads must pass the Augmented Dickey-Fuller (ADF) test at $p < 0.05$ before capital allocation.

### Invariant 2: Ornstein-Uhlenbeck Mean Reversion
- Spread dynamics follow the continuous SDE:
  $$dZ_t = \theta (\mu - Z_t) dt + \sigma dW_t$$
- The mean-reversion parameter $\theta$ must be strictly positive ($\theta > 0$), yielding a finite half-life:
  $$t_{\text{half}} = \frac{\ln(2)}{\theta}$$

### Invariant 3: Almgren-Chriss Terminal Liquidation
- An execution schedule for $Q_0$ shares over horizon $T$ divided into $N$ intervals must strictly satisfy:
  $$q_0 = Q_0, \quad q_N = 0, \quad \sum_{k=1}^N \Delta n_k = Q_0$$

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: Online Kalman Filter Dynamic Hedge Ratio & OU Parameter Calibrator
```rust
/// Online 2D Kalman Filter for Dynamic Hedge Ratio (Alpha, Beta) Tracking
/// State: x_t = [alpha, beta]^T, Observation: y_t = alpha + beta * x_t + v_t
#[derive(Debug, Clone)]
pub struct OnlineKalmanFilter {
    pub state: [f64; 2],       // [alpha, beta]
    pub cov: [[f64; 2]; 2],    // Error covariance P
    pub r: f64,                // Measurement noise variance
    pub q: [[f64; 2]; 2],      // Process noise covariance
}

impl OnlineKalmanFilter {
    pub fn new(delta: f64, r: f64) -> Self {
        Self {
            state: [0.0, 1.0], // Initial guess: intercept = 0, beta = 1
            cov: [[1.0, 0.0], [0.0, 1.0]],
            r,
            q: [[delta / (1.0 - delta), 0.0], [0.0, delta / (1.0 - delta)]],
        }
    }

    /// Update filter with new observation (x_t, y_t), returning predicted spread
    pub fn update(&mut self, x: f64, y: f64) -> f64 {
        // 1. Predict state: x_pred = x_{t-1}, P_pred = P_{t-1} + Q
        let p_pred = [
            [self.cov[0][0] + self.q[0][0], self.cov[0][1]],
            [self.cov[1][0], self.cov[1][1] + self.q[1][1]],
        ];

        // 2. Observation matrix: H = [1.0, x]
        let h = [1.0, x];

        // 3. Innovation: y_pred = H * x_pred
        let y_pred = h[0] * self.state[0] + h[1] * self.state[1];
        let error = y - y_pred;

        // 4. Innovation variance: S = H * P_pred * H^T + R
        let hp0 = h[0] * p_pred[0][0] + h[1] * p_pred[1][0];
        let hp1 = h[0] * p_pred[0][1] + h[1] * p_pred[1][1];
        let s = hp0 * h[0] + hp1 * h[1] + self.r;

        // 5. Kalman gain: K = P_pred * H^T / S
        let k = [
            (p_pred[0][0] * h[0] + p_pred[0][1] * h[1]) / s,
            (p_pred[1][0] * h[0] + p_pred[1][1] * h[1]) / s,
        ];

        // 6. Update state: x = x_pred + K * error
        self.state[0] += k[0] * error;
        self.state[1] += k[1] * error;

        // 7. Update covariance: P = (I - K * H) * P_pred
        let kh = [
            [k[0] * h[0], k[0] * h[1]],
            [k[1] * h[0], k[1] * h[1]],
        ];
        self.cov[0][0] = (1.0 - kh[0][0]) * p_pred[0][0] - kh[0][1] * p_pred[1][0];
        self.cov[0][1] = (1.0 - kh[0][0]) * p_pred[0][1] - kh[0][1] * p_pred[1][1];
        self.cov[1][0] = -kh[1][0] * p_pred[0][0] + (1.0 - kh[1][1]) * p_pred[1][0];
        self.cov[1][1] = -kh[1][0] * p_pred[0][1] + (1.0 - kh[1][1]) * p_pred[1][1];

        // Return current spread
        y - (self.state[0] + self.state[1] * x)
    }
}

/// Almgren-Chriss Optimal Execution Trajectory Generator
#[derive(Debug, Clone)]
pub struct AlmgrenChrissTrajectory {
    pub holding: Vec<f64>,
    pub trades: Vec<f64>,
}

pub fn calculate_almgren_chriss(
    total_shares: f64,
    num_intervals: usize,
    lambda: f64, // Risk aversion parameter
    sigma: f64,  // Asset price volatility
    eta: f64,    // Temporary market impact parameter
    gamma: f64,  // Permanent market impact parameter
    tau: f64,    // Time step dt
) -> AlmgrenChrissTrajectory {
    assert!(num_intervals > 0 && total_shares > 0.0 && eta > 0.0);
    let t_total = (num_intervals as f64) * tau;
    let kappa = ((lambda * sigma * sigma) / eta).sqrt();

    let mut holding = Vec::with_capacity(num_intervals + 1);
    let mut trades = Vec::with_capacity(num_intervals);

    for j in 0..=num_intervals {
        let t_j = (j as f64) * tau;
        let num = (kappa * (t_total - t_j)).sinh();
        let den = (kappa * t_total).sinh();
        let q_j = total_shares * (num / den);
        holding.push(q_j);
    }

    for j in 0..num_intervals {
        trades.push(holding[j] - holding[j + 1]);
    }

    AlmgrenChrissTrajectory { holding, trades }
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: Cointegration Arbitrage System
```markdown
You are a quantitative statistical arbitrage engineer writing an automated pairs trading module in Rust.
- Modeling: Fit an Ornstein-Uhlenbeck SDE to the cointegrated residual z_t = y_t - beta * x_t.
- Verification: Compute half-life t_half = ln(2) / theta. Reject pairs with half-life < 1 day or > 30 days.
- Signal Generation: Compute running Z-score z_score = (z_t - mu) / sigma. Enter long spread at z_score <= -2.0, short at z_score >= +2.0, exit at z_score == 0.0.
- Risk Controls: Enforce hard stop loss if |z_score| >= 3.5 (indicating structural break in cointegration).
```
