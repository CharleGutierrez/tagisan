---
name: analytics-practical-statistics
description: Practical statistical methods: bootstrap resampling, permutation testing, robust statistics, median absolute deviation (MAD), and sampling distribution validation. Triggers: practical-statistics, bootstrap-resampling, permutation-test, robust-statistics, median-absolute-deviation, mad, trimmed-mean, sampling-variability.
triggers:
  - practical-statistics
  - bootstrap-resampling
  - permutation-test
  - robust-statistics
  - median-absolute-deviation
  - mad
  - trimmed-mean
  - sampling-variability
---

# Analytics Practical Statistics
> Based on **Practical Statistics for Data Scientists - Peter Bruce, Andrew Bruce, Peter Gedeck**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Statistical Test Results Tracking
CREATE TABLE experiment_hypothesis_tests (
    test_id UUID PRIMARY KEY,
    metric_name VARCHAR(100) NOT NULL,
    test_type VARCHAR(50) NOT NULL, -- BOOTSTRAP, PERMUTATION
    observed_diff DOUBLE PRECISION NOT NULL,
    p_value DOUBLE PRECISION NOT NULL,
    ci_lower DOUBLE PRECISION NOT NULL,
    ci_upper DOUBLE PRECISION NOT NULL,
    iterations INT NOT NULL DEFAULT 10000
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Median Absolute Deviation (MAD)
For sample $X$:
$$MAD = \text{median}(|X_i - \text{median}(X)|)$$
$$\hat{\sigma}_{\text{robust}} = 1.4826 \times MAD$$

### 2.2 Bootstrap Standard Error
Given $B$ bootstrap samples $\hat{\theta}^*_1, \dots, \hat{\theta}^*_B$:
$$SE_{\text{boot}} = \sqrt{\frac{1}{B-1}\sum_{b=1}^B (\hat{\theta}^*_b - \bar{\theta}^*)^2}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[Sample Groups A & B] --> CalcObs[Calculate Observed Delta]
    CalcObs --> Pool[Pool Data & Strip Group Labels]
    Pool --> Shuffle[Permute / Shuffle Pooled Data]
    Shuffle --> Reassign[Reassign to Synthetic A & B]
    Reassign --> Recalc[Compute Permuted Delta]
    Recalc --> Iterate{Repeat 10,000 times}
    Iterate --> CalcPVal[p = Count(|Delta_perm| >= |Delta_obs|) / 10000]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def permutation_test(group_a: np.ndarray, group_b: np.ndarray, n_iter: int = 10000) -> float:
    obs_diff = np.abs(np.mean(group_a) - np.mean(group_b))
    combined = np.concatenate([group_a, group_b])
    n_a = len(group_a)
    count = 0
    for _ in range(n_iter):
        np.random.shuffle(combined)
        perm_diff = np.abs(np.mean(combined[:n_a]) - np.mean(combined[n_a:]))
        if perm_diff >= obs_diff:
            count += 1
    return count / n_iter
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Use bootstrap resampling to estimate standard errors and confidence intervals without normality assumptions.
- Calculate robust scale via MAD: sigma_est = 1.4826 * median(|x - median(x)|).
- Apply permutation tests to determine exact p-values for difference in means or medians.
- Rely on trimmed mean (e.g. 10% trim) to protect location metrics from heavy-tailed outliers.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect a production statistical inference library:
1. Build non-parametric bootstrap and permutation testing harnesses executing 10,000+ iterations.
2. Implement robust estimators (Huber loss, trimmed mean, MAD) resistant to non-Gaussian noise.
3. Validate sample size adequacy and empirical sampling distributions across continuous KPIs.
```
