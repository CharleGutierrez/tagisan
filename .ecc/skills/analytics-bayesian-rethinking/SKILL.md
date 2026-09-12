---
name: analytics-bayesian-rethinking
description: Bayesian modeling: DAG causal graphs, prior predictive simulation, MCMC sampling, Highest Posterior Density Intervals (HPDI), and collider conditioning avoidance. Triggers: bayesian-rethinking, bayesian-modeling, prior-posterior, directed-acyclic-graphs, collider-bias, mcmc-sampling, hpdi, posterior-predictive.
triggers:
  - bayesian-rethinking
  - bayesian-modeling
  - prior-posterior
  - directed-acyclic-graphs
  - collider-bias
  - mcmc-sampling
  - hpdi
  - posterior-predictive
---

# Analytics Bayesian Rethinking
> Based on **Statistical Rethinking - Richard McElreath**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Bayesian Model Parameter Posterior Summary
CREATE TABLE bayesian_posteriors (
    model_name VARCHAR(100) NOT NULL,
    parameter_name VARCHAR(100) NOT NULL,
    mean DOUBLE PRECISION NOT NULL,
    std_dev DOUBLE PRECISION NOT NULL,
    hpdi_lower_95 DOUBLE PRECISION NOT NULL,
    hpdi_upper_95 DOUBLE PRECISION NOT NULL,
    r_hat DOUBLE PRECISION NOT NULL CHECK (r_hat < 1.05),
    PRIMARY KEY (model_name, parameter_name)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Bayes' Theorem Invariant
$$P(\theta | D) = \frac{P(D | \theta) P(\theta)}{P(D)} = \frac{P(D | \theta) P(\theta)}{\int P(D | \theta) P(\theta) d\theta}$$

### 2.2 Collider Bias Invariant
In DAG $X \to C \leftarrow Y$, conditioning on collider $C$ creates spurious association:
$$X \perp Y \quad \text{but} \quad X \not\perp Y \mid C$$
Rule: NEVER condition on a collider when estimating the causal effect of $X$ on $Y$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Prior[Define Generative Prior P(theta)] --> PriorPred[Prior Predictive Simulation]
    PriorPred --> Likelihood[Formulate Data Likelihood P(D|theta)]
    Likelihood --> HMC[Run Hamiltonian Monte Carlo Sampling]
    HMC --> CheckConv[Check R-hat < 1.01 and ESS > 400]
    CheckConv --> Posterior[Analyze HPDI & Posterior Predictive Checks]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# PyMC Generative Model
import pymc as pm

with pm.Model() as model:
    alpha = pm.Normal("alpha", mu=0, sigma=10)
    beta = pm.Normal("beta", mu=0, sigma=5)
    sigma = pm.Exponential("sigma", lam=1)
    mu = alpha + beta * x_obs
    y = pm.Normal("y", mu=mu, sigma=sigma, observed=y_obs)
    # trace = pm.sample(draws=2000, tune=1000, target_accept=0.95)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Perform prior predictive checks before observing data to verify realistic parameter ranges.
- Never condition on a collider (X -> C <- Y); it creates spurious correlations.
- Convergence requires Gelman-Rubin R-hat < 1.05 and high Effective Sample Size (ESS).
- Report 89% or 95% HPDI (Highest Posterior Density Interval) rather than point estimates.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement Bayesian statistical modeling workflows:
1. Formulate causal DAGs identifying backdoor adjustment paths and excluding colliders.
2. Execute Hamiltonian Monte Carlo (HMC) sampling with convergence verification (R-hat, divergences).
3. Generate posterior predictive distributions to validate model calibration against empirical data.
```
