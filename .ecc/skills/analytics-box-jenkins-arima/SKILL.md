---
name: analytics-box-jenkins-arima
description: Box-Jenkins time series methodology: ARIMA/SARIMA modeling, stationary differencing, ACF/PACF diagnostics, and Ljung-Box residual white-noise testing. Triggers: box-jenkins-arima, arima-modeling, stationarity-differencing, acf-pacf, ljung-box-test, white-noise-residuals, sarima.
triggers:
  - box-jenkins-arima
  - arima-modeling
  - stationarity-differencing
  - acf-pacf
  - ljung-box-test
  - white-noise-residuals
  - sarima
---

# Analytics Box Jenkins Arima
> Based on **Time Series Analysis: Forecasting and Control - George Box et al.**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- ARIMA Model Diagnostic Registry
CREATE TABLE arima_model_diagnostics (
    series_id VARCHAR(50) PRIMARY KEY,
    p INT NOT NULL, -- AR order
    d INT NOT NULL, -- Differencing order
    q INT NOT NULL, -- MA order
    aic DOUBLE PRECISION NOT NULL,
    bic DOUBLE PRECISION NOT NULL,
    ljung_box_stat DOUBLE PRECISION NOT NULL,
    ljung_box_p_value DOUBLE PRECISION NOT NULL,
    residuals_are_white_noise BOOLEAN NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 ARIMA(p, d, q) Mathematical Formulation
$$(1 - \sum_{i=1}^p \phi_i B^i) (1 - B)^d Y_t = c + (1 + \sum_{j=1}^q \theta_j B^j) \epsilon_t$$
where $B$ is the backshift operator ($B^k Y_t = Y_{t-k}$) and $\epsilon_t \sim \mathcal{N}(0, \sigma^2)$.

### 2.2 Ljung-Box White Noise Diagnostic Invariant
$$Q = n(n + 2) \sum_{k=1}^h \frac{\hat{\rho}_k^2}{n - k} \sim \chi^2(h - p - q)$$
Residuals are pure white noise if and only if $p = P(\chi^2 \ge Q) > 0.05$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Raw[Raw Time Series] --> StationarityCheck{ADF Test: Is Series Stationary?}
    StationarityCheck -->|No| Difference[Difference Series d times: (1-B)^d Y_t]
    Difference --> StationarityCheck
    StationarityCheck -->|Yes| InspectACF[Inspect ACF & PACF to identify p, q]
    InspectACF --> Estimate[Estimate Parameters via MLE]
    Estimate --> Diagnostic{Ljung-Box p-value > 0.05?}
    Diagnostic -->|No: Autocorrelated| Refit[Adjust p, q Orders]
    Diagnostic -->|Yes: White Noise| Forecast[Generate Production Forecast]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from statsmodels.tsa.stattools import acf
import numpy as np

def ljung_box_test(residuals: np.ndarray, lags: int = 10) -> tuple[float, float]:
    from statsmodels.stats.diagnostic import acorr_ljungbox
    res = acorr_ljungbox(residuals, lags=[lags], return_df=True)
    stat = res['lb_stat'].iloc[0]
    p_val = res['lb_pvalue'].iloc[0]
    return float(stat), float(p_val)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Box-Jenkins 3-stage iterative cycle: 1. Identification -> 2. Estimation -> 3. Diagnostics.
- Difference the series d times until stationary (verify with Augmented Dickey-Fuller test).
- PACF cuts off at lag p for AR(p); ACF cuts off at lag q for MA(q).
- Diagnostic rule: residuals must be uncorrelated white noise (Ljung-Box test p-value > 0.05).
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build enterprise ARIMA/SARIMA econometric forecasting modules:
1. Implement automated Box-Jenkins pipelines running ADF unit-root tests and optimal differencing.
2. Search hyperparameter space (p, d, q, P, D, Q) minimizing AIC/BIC information criteria.
3. Validate residual white-noise compliance with automated Ljung-Box and normality tests.
```
