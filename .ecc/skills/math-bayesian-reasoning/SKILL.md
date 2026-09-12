---
name: math-bayesian-reasoning
description: Bayesian reasoning, prior and posterior distributions, Beta-Binomial conjugate updating, Thompson Sampling for multi-armed bandits, and dynamic agent prompt routing based on 'Bayesian Statistics the Fun Way' by Will Kurt. Triggers: bayesian-reasoning, bayesian-statistics-fun-way, will-kurt, bayes-theorem, beta-binomial, thompson-sampling, multi-armed-bandit, prior-posterior, credible-intervals, agent-routing.
triggers:
  - bayesian-reasoning
  - bayesian-statistics-fun-way
  - will-kurt
  - bayes-theorem
  - beta-binomial
  - thompson-sampling
  - multi-armed-bandit
  - prior-posterior
  - credible-intervals
  - agent-routing
---

# Bayesian Reasoning & Bandits: Posterior Belief Updating

## 1. Mathematical Foundations & Formal Principles

### 1.1 Bayes' Theorem
Given hypothesis $H$ and evidence $E$:
$$P(H \mid E) = \frac{P(E \mid H) P(H)}{P(E)}$$
Where:
- $P(H)$: Prior probability of hypothesis before seeing evidence.
- $P(E \mid H)$: Likelihood of observing evidence given hypothesis is true.
- $P(E) = \sum_{i} P(E \mid H_i) P(H_i)$: Marginal likelihood (normalizing constant).
- $P(H \mid E)$: Posterior probability after assimilating evidence.

### 1.2 Beta-Binomial Conjugate Model
For a Bernoulli trial with success probability $p \in [0, 1]$:
If prior is a Beta distribution $\text{Beta}(\alpha, \beta)$ with PDF:
$$f(p; \alpha, \beta) = \frac{1}{\text{B}(\alpha, \beta)} p^{\alpha - 1} (1 - p)^{\beta - 1}$$
After observing $k$ successes and $n - k$ failures, the posterior is conjugate:
$$p \mid \text{data} \sim \text{Beta}(\alpha + k, \; \beta + n - k)$$
- Prior Mean: $\mathbb{E}[p] = \frac{\alpha}{\alpha + \beta}$
- Prior Variance: $\text{Var}(p) = \frac{\alpha \beta}{(\alpha + \beta)^2 (\alpha + \beta + 1)}$

### 1.3 Thompson Sampling (Bayesian Multi-Armed Bandit)
For $K$ arms where each arm $k$ has unknown reward probability $\theta_k \sim \text{Beta}(\alpha_k, \beta_k)$:
1. Sample $\hat{\theta}_k \sim \text{Beta}(\alpha_k, \beta_k)$ for each $k \in \{1, \dots, K\}$.
2. Select arm $k^* = \arg\max_k \hat{\theta}_k$.
3. Observe reward $r \in \{0, 1\}$.
4. Update posterior:
   - If $r = 1$: $\alpha_{k^*} \leftarrow \alpha_{k^*} + 1$
   - If $r = 0$: $\beta_{k^*} \leftarrow \beta_{k^*} + 1$
Thompson Sampling achieves asymptotically optimal logarithmic regret without explicit epsilon exploration hyperparameter tuning.

---

## 2. The Vibe Coding Superpower

1. **Adaptive Agent Prompt Routing**: Dynamically route queries across model tiers (Haiku vs Sonnet vs Opus) using Thompson Sampling to minimize cost while maximizing response quality.
2. **Online Confidence Calibration**: Maintain Bayesian beliefs over tool reliability; if an API starts returning errors, its posterior drops smoothly.
3. **A/B Testing with Small Samples**: Evaluate feature variants without waiting for thousands of samples; determine true probabilities with credible intervals.

---

## 3. Production Code Implementations

### 3.1 Thompson Sampling Multi-Armed Bandit in Python
```python
import numpy as np

class ThompsonSamplingRouter:
    """
    Adaptive Bayesian Router for LLM models or agent tools.
    Balances exploration and exploitation using Beta-Binomial posteriors.
    """
    def __init__(self, arm_names: list[str], prior_alpha: float = 1.0, prior_beta: float = 1.0):
        self.arms = arm_names
        # Prior parameters (alpha = successes + 1, beta = failures + 1)
        self.alphas = {arm: prior_alpha for arm in arm_names}
        self.betas = {arm: prior_beta for arm in arm_names}

    def select_arm(self) -> str:
        """
        Sample from each arm's posterior Beta distribution and select the argmax.
        """
        samples = {
            arm: np.random.beta(self.alphas[arm], self.betas[arm])
            for arm in self.arms
        }
        return max(samples, key=samples.get)

    def record_feedback(self, arm: str, success: bool):
        """
        Conjugate update of posterior distribution.
        """
        if success:
            self.alphas[arm] += 1.0
        else:
            self.betas[arm] += 1.0

    def get_stats(self) -> dict:
        return {
            arm: {
                "mean": self.alphas[arm] / (self.alphas[arm] + self.betas[arm]),
                "trials": int(self.alphas[arm] + self.betas[arm] - 2.0),
            }
            for arm in self.arms
        }
```

### 3.2 Bayesian Belief Updater in Rust
```rust
#[derive(Debug, Clone)]
pub struct BetaDistribution {
    pub alpha: f64,
    pub beta: f64,
}

impl BetaDistribution {
    pub fn uniform_prior() -> Self {
        Self { alpha: 1.0, beta: 1.0 }
    }

    pub fn update(&mut self, successes: u32, failures: u32) {
        self.alpha += successes as f64;
        self.beta += failures as f64;
    }

    pub fn expected_value(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    pub fn variance(&self) -> f64 {
        let sum = self.alpha + self.beta;
        (self.alpha * self.beta) / (sum * sum * (sum + 1.0))
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Bayesian Reasoning
```markdown
When answering questions under uncertainty:
- Always state the prior hypothesis and how the newly presented evidence modifies that probability.
- Never declare 0% or 100% certainty (Cromwell's rule) unless mathematically proven impossible.
- Express confidence as a Bayesian probability interval: "Given observations X, posterior belief is 85% [Credible Interval: 78%-92%]".
```
