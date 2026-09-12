---
name: analytics-intuitive-statistics
description: Foundational intuitive statistics: Central Limit Theorem (CLT), Law of Large Numbers, standard errors, Type I/II errors, statistical power, and Simpson's Paradox. Triggers: intuitive-statistics, central-limit-theorem, clt, law-of-large-numbers, standard-error, simpsons-paradox, type-1-type-2-errors, statistical-power.
triggers:
  - intuitive-statistics
  - central-limit-theorem
  - clt
  - law-of-large-numbers
  - standard-error
  - simpsons-paradox
  - type-1-type-2-errors
  - statistical-power
---

# Analytics Intuitive Statistics
> Based on **Naked Statistics - Charles Wheelan**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Metric Sampling Distribution Summary
CREATE TABLE sampling_distribution_runs (
    run_id UUID PRIMARY KEY,
    population_size BIGINT NOT NULL,
    sample_size INT NOT NULL,
    mean_of_means DOUBLE PRECISION NOT NULL,
    empirical_se DOUBLE PRECISION NOT NULL,
    theoretical_se DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Central Limit Theorem Invariant
For independent, identically distributed random variables with mean $\mu$ and variance $\sigma^2$:
$$\bar{X}_n = \frac{1}{n}\sum_{i=1}^n X_i \xrightarrow{d} \mathcal{N}\left(\mu, \frac{\sigma^2}{n}\right) \quad \text{as } n \to \infty$$
$$\text{Standard Error } (SE) = \frac{s}{\sqrt{n}}$$

### 2.2 Simpson's Paradox Invariant
An observed correlation in aggregate can reverse when conditioned on confounding variable $Z$:
$$\text{sgn}\left(\frac{\partial E[Y|X]}{\partial X}\right) \neq \text{sgn}\left(\frac{\partial E[Y|X, Z]}{\partial X}\right)$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Population[Non-Normal Population] --> Draw[Draw K Samples of size N]
    Draw --> SampleMeans[Compute Mean for each Sample]
    SampleMeans --> Distribution[Plot Means Distribution]
    Distribution --> BellCurve[Distribution converges to Gaussian N(mu, sigma^2/N)]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# CLT Verification in Python
import numpy as np

def verify_clt(population: np.ndarray, sample_size: int = 100, num_samples: int = 1000):
    sample_means = [np.mean(np.random.choice(population, size=sample_size)) for _ in range(num_samples)]
    theoretical_se = np.std(population) / np.sqrt(sample_size)
    empirical_se = np.std(sample_means)
    return {"mean": np.mean(sample_means), "theoretical_se": theoretical_se, "empirical_se": empirical_se}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Standard Error of the Mean is s / sqrt(n); increasing sample size 4x cuts SE by half.
- Always segment data to detect Simpson's Paradox where aggregate trends contradict sub-cohort trends.
- Balance Type I error alpha (false positive) and Type II error beta (false negative).
- Statistical significance is not practical significance: tiny effects become significant at massive n.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design foundational statistical safeguards for analytics reporting:
1. Implement automated checks detecting Simpson's paradox across high-cardinality dimensions.
2. Calculate power curves and minimum sample sizes to prevent underpowered experiments.
3. Formulate intuitive standard error and confidence interval displays for executive stakeholders.
```
