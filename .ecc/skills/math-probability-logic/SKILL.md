---
name: math-probability-logic
description: Probability as extended Aristotelian logic, Cox's theorems, principle of maximum entropy (MaxEnt), and epistemic uncertainty grounding based on 'Probability Theory: The Logic of Science' by E.T. Jaynes. Triggers: probability-logic, jaynes-probability, probability-extended-logic, cox-theorems, maximum-entropy-maxent, epistemic-uncertainty, plausibility-reasoning, bayesian-consistency, prior-assignment.
triggers:
  - probability-logic
  - jaynes-probability
  - probability-extended-logic
  - cox-theorems
  - maximum-entropy-maxent
  - epistemic-uncertainty
  - plausibility-reasoning
  - bayesian-consistency
  - prior-assignment
---

# Probability as Logic: Cox's Theorems & Maximum Entropy

## 1. Mathematical Foundations & Formal Principles

### 1.1 Probability as Extended Logic
Aristotelian Boolean logic handles propositions with truth values in $\{0, 1\}$ (True, False).
E.T. Jaynes formalizes probability $P(A \mid X) \in [0, 1]$ as the unique consistent numerical measure of the **degree of plausibility** of proposition $A$ given background information $X$.

### 1.2 Cox's Theorems
Richard Cox proved that any numerical measure of plausibility satisfying three fundamental desiderata:
1. **Representation by Real Numbers**: Plausibility is represented by a single real number.
2. **Qualitative Correspondence with Common Sense**: If plausibility increases, its numerical value increases.
3. **Self-Consistency**:
   - If a conclusion can be arrived at in more than one way, every way must lead to the same result.
   - All available background evidence must always be taken into account.
   - Equivalent states of knowledge must be represented by equivalent plausibilities.

Necessarily obeys the **Product Rule** and **Sum Rule**:
- **Product Rule**: $P(AB \mid X) = P(A \mid BX) P(B \mid X) = P(B \mid AX) P(A \mid X)$
- **Sum Rule**: $P(A \mid X) + P(\bar{A} \mid X) = 1$

### 1.3 Principle of Maximum Entropy (MaxEnt)
When assigning a prior distribution $p(x)$ subject to testable expectation constraints $\mathbb{E}[f_k(X)] = \alpha_k$:
Select the distribution that maximizes Shannon entropy $H(p) = -\sum p(x) \ln p(x)$ while satisfying the constraints.
$$p^*(x) = \frac{1}{Z} \exp\left(-\sum_k \lambda_k f_k(x)\right)$$
Any other distribution assumes information not justified by the stated evidence.

---

## 2. The Vibe Coding Superpower

1. **Epistemic Calibration in AI Outputs**: Condition every statement on background information: never output raw probabilities without conditioning context $P(A \mid \text{Evidence})$.
2. **Unbiased Default Initialization**: Use MaxEnt priors (uniform for bounded intervals, exponential for positive means, Gaussian for given mean and variance) to initialize zero-shot agent weights.
3. **Contradiction Detection**: If two subagents give contradictory probabilities, trace back to differing background knowledge $X_1 \neq X_2$.

---

## 3. Production Code Implementations

### 3.1 Maximum Entropy Prior Solver in Python
```python
from scipy.optimize import minimize
import numpy as np

def maxent_discrete_prior(n_states: int, constraints: list[tuple[np.ndarray, float]]) -> np.ndarray:
    """
    Find MaxEnt distribution p over n_states satisfying sum(p * f_k) = alpha_k.
    """
    def neg_entropy(p):
        p_safe = np.clip(p, 1e-12, 1.0)
        return float(np.sum(p_safe * np.log(p_safe)))

    # Constraints: sum(p) == 1, plus user expectations
    cons = [{'type': 'eq', 'fun': lambda p: np.sum(p) - 1.0}]
    for feature, expected_val in constraints:
        cons.append({'type': 'eq', 'fun': lambda p, f=feature, target=expected_val: np.dot(p, f) - target})

    bounds = [(1e-8, 1.0) for _ in range(n_states)]
    init_guess = np.ones(n_states) / n_states

    res = minimize(neg_entropy, init_guess, method='SLSQP', bounds=bounds, constraints=cons)
    return res.x
```

### 3.2 Plausibility Reasoner in Rust
```rust
pub struct JaynesianBelief {
    pub log_odds: f64, // psi = ln(P / (1 - P))
}

impl JaynesianBelief {
    pub fn from_prob(p: f64) -> Self {
        let p_safe = p.clamp(1e-8, 1.0 - 1e-8);
        Self { log_odds: (p_safe / (1.0 - p_safe)).ln() }
    }

    pub fn to_prob(&self) -> f64 {
        1.0 / (1.0 + (-self.log_odds).exp())
    }

    /// Add weight of evidence in decibels (dB)
    pub fn update_with_evidence(&mut self, log_likelihood_ratio: f64) {
        self.log_odds += log_likelihood_ratio;
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Jaynesian Extended Logic
```markdown
When reasoning under incomplete information:
- Treat probability strictly as a measure of plausibility conditioned on stated assumptions: P(Hypothesis | Context).
- Apply Cromwell's Rule: Never assign probability 0 or 1 to non-tautological propositions.
- State explicitly what evidence would be required to shift your plausibility score.
```
