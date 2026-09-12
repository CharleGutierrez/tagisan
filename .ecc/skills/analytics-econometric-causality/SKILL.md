---
name: analytics-econometric-causality
description: Applied econometrics: Difference-in-Differences (DiD), parallel trends testing, Two-Way Fixed Effects (TWFE), omitted variable bias formula, and cluster-robust standard errors. Triggers: econometric-causality, mostly-harmless-econometrics, difference-in-differences, did-estimator, omitted-variable-bias, 2sls, parallel-trends.
triggers:
  - econometric-causality
  - mostly-harmless-econometrics
  - difference-in-differences
  - did-estimator
  - omitted-variable-bias
  - 2sls
  - parallel-trends
---

# Analytics Econometric Causality
> Based on **Mostly Harmless Econometrics - Joshua Angrist & Jörn-Steffen Pischke**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Panel Data Table for Difference-in-Differences
CREATE TABLE panel_observations (
    entity_id BIGINT NOT NULL,
    time_period INT NOT NULL,
    is_treated_group INT NOT NULL CHECK (is_treated_group IN (0, 1)),
    is_post_period INT NOT NULL CHECK (is_post_period IN (0, 1)),
    outcome DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (entity_id, time_period)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Difference-in-Differences (DiD) Estimator
Let $\bar{Y}_{G, T}$ be the sample mean of group $G \in \{T, C\}$ at time $T \in \{1, 2\}$:
$$\hat{\delta}_{\text{DiD}} = (\bar{Y}_{T, 2} - \bar{Y}_{T, 1}) - (\bar{Y}_{C, 2} - \bar{Y}_{C, 1})$$
Regression specification:
$$Y_{it} = \alpha + \beta \cdot \text{Treated}_i + \gamma \cdot \text{Post}_t + \delta_{\text{DiD}} (\text{Treated}_i \times \text{Post}_t) + \epsilon_{it}$$

### 2.2 Omitted Variable Bias (OVB) Invariant
$$\hat{\beta}_{\text{short}} = \beta_{\text{long}} + \gamma \frac{\text{Cov}(X, Z)}{\text{Var}(X)}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    PrePeriod[Pre-Treatment Periods 1..k] --> TestParallel{Parallel Pre-Trends Test}
    TestParallel -->|Reject: Non-parallel| Invalidate[DiD Invalid: Trends diverging before treatment]
    TestParallel -->|Accept: Parallel trends| Intervention[Intervention Occurs at Period k+1]
    Intervention --> PostPeriod[Post-Treatment Periods k+1..T]
    PostPeriod --> EstimateDiD[Compute (Delta Y_treat) - (Delta Y_control)]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def calculate_did(y_t1: float, y_t2: float, y_c1: float, y_c2: float) -> float:
    # DiD = (Y_T2 - Y_T1) - (Y_C2 - Y_C1)
    treat_diff = y_t2 - y_t1
    control_diff = y_c2 - y_c1
    return treat_diff - control_diff
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- DiD formula: (Y_treatment_post - Y_treatment_pre) - (Y_control_post - Y_control_pre).
- The fundamental identifying assumption is Parallel Trends: treatment and control must track identically in pre-period.
- Always cluster standard errors at the state/entity level to prevent deflated p-values from autocorrelation.
- Omitted Variable Bias = (Relationship of omitted with Y) * (Regression of omitted on X).
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement an econometric causal analysis suite:
1. Build automated Difference-in-Differences estimators with event-study pre-trend diagnostic tests.
2. Formulate Two-Way Fixed Effects regressions with cluster-robust sandwich covariance estimators.
3. Validate parallel trends robustness against staggered rollouts using Callaway-Sant'Anna estimators.
```
