---
name: analytics-counterfactual-causality
description: Counterfactual causal theory: identifiability conditions (Exchangeability, Positivity, Consistency), Inverse Probability Weighting (IPW), marginal structural models, and g-computation. Triggers: counterfactual-causality, causal-inference-what-if, exchangeability, positivity, consistency, inverse-probability-weighting, ipw, g-methods.
triggers:
  - counterfactual-causality
  - causal-inference-what-if
  - exchangeability
  - positivity
  - consistency
  - inverse-probability-weighting
  - ipw
  - g-methods
---

# Analytics Counterfactual Causality
> Based on **Causal Inference: What If - Miguel Hernán & James Robins**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Propensity Scores & IPW Weights Table
CREATE TABLE propensity_weighted_cohort (
    subject_id BIGINT PRIMARY KEY,
    treatment INT NOT NULL CHECK (treatment IN (0, 1)),
    propensity_score DOUBLE PRECISION NOT NULL CHECK (propensity_score > 0.0 AND propensity_score < 1.0),
    ipw_weight DOUBLE PRECISION NOT NULL,
    outcome DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Identifiability Conditions
1. **Conditional Exchangeability**: $Y(a) \perp A \mid L$
2. **Positivity**: $P(A = a \mid L = l) > 0 \quad \forall a, l$
3. **Consistency**: If $A = a$, then $Y = Y(a)$

### 2.2 Inverse Probability Weighting (IPW) Invariant
Let $e(L) = P(A = 1 \mid L)$ be the propensity score:
$$W = \frac{A}{e(L)} + \frac{1 - A}{1 - e(L)}$$
In the pseudo-population weighted by $W$, treatment assignment is unconfounded by $L$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Confounders[Measure Confounders L] --> FitPropensity[Fit Logistic Regression: P(A=1|L)]
    FitPropensity --> CheckPositivity{Verify Positivity: 0.01 < e(L) < 0.99}
    CheckPositivity -->|Fail| Trim[Trim Non-Overlapping Support]
    CheckPositivity -->|Pass| CalcWeights[Calculate IPW Weights W]
    CalcWeights --> FitMSM[Fit Marginal Structural Model on Weighted Cohort]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def compute_ipw_weights(treatment: np.ndarray, prop_score: np.ndarray) -> np.ndarray:
    # Truncate to prevent extreme weights
    ps = np.clip(prop_score, 0.01, 0.99)
    weights = treatment / ps + (1 - treatment) / (1 - ps)
    return weights
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Identifiability requires 3 pillars: Exchangeability (no unmeasured confounding), Positivity, Consistency.
- Inverse Probability Weighting: W = A / e(L) + (1 - A) / (1 - e(L)).
- Clip extreme propensity scores (e.g. [0.01, 0.99]) to avoid high-variance weight explosion.
- Check covariate balance post-weighting: standardized mean difference should be < 0.1 for all features.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build counterfactual estimation pipelines:
1. Implement Inverse Probability Weighting (IPW) with stabilized weights and covariate balance checks.
2. Detect positivity violations and support overlap failures across high-dimensional confounders.
3. Formulate G-computation algorithms simulating multi-stage dynamic treatment regimes.
```
