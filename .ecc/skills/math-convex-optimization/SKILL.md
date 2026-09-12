---
name: math-convex-optimization
description: Convex optimization, convex sets and functions, duality, Slater's condition, Karush-Kuhn-Tucker (KKT) optimality conditions, and projected gradient descent based on 'Convex Optimization' by Stephen Boyd and Lieven Vandenberghe. Triggers: convex-optimization, boyd-vandenberghe, convex-sets, kkt-conditions, lagrangian-duality, slater-condition, projected-gradient-descent, interior-point, cost-latency-tradeoff, constrained-optimization.
triggers:
  - convex-optimization
  - boyd-vandenberghe
  - convex-sets
  - kkt-conditions
  - lagrangian-duality
  - slater-condition
  - projected-gradient-descent
  - interior-point
  - cost-latency-tradeoff
  - constrained-optimization
---

# Convex Optimization: Duality, KKT & Resource Allocation

## 1. Mathematical Foundations & Formal Principles

### 1.1 Convex Sets & Convex Functions
A set $C \subseteq \mathbb{R}^n$ is convex if for all $\mathbf{x}, \mathbf{y} \in C$ and $\theta \in [0, 1]$:
$$\theta \mathbf{x} + (1 - \theta)\mathbf{y} \in C$$
A function $f: \mathbb{R}^n \to \mathbb{R}$ is convex if its domain is convex and Jensen's inequality holds:
$$f(\theta \mathbf{x} + (1 - \theta)\mathbf{y}) \le \theta f(\mathbf{x}) + (1 - \theta)f(\mathbf{y})$$
Second-order condition: $\nabla^2 f(\mathbf{x}) \succeq 0$ (Hessian is positive semi-definite everywhere).

### 1.2 Standard Optimization Problem
$$\begin{aligned}
\text{minimize} \quad & f_0(\mathbf{x}) \\
\text{subject to} \quad & f_i(\mathbf{x}) \le 0, \quad i = 1, \dots, m \\
& h_i(\mathbf{x}) = \mathbf{a}_i^T \mathbf{x} - b_i = 0, \quad i = 1, \dots, p
\end{aligned}$$

### 1.3 Lagrangian & Dual Problem
The Lagrangian $\mathcal{L}: \mathbb{R}^n \times \mathbb{R}^m \times \mathbb{R}^p \to \mathbb{R}$:
$$\mathcal{L}(\mathbf{x}, \boldsymbol{\lambda}, \boldsymbol{\nu}) = f_0(\mathbf{x}) + \sum_{i=1}^m \lambda_i f_i(\mathbf{x}) + \sum_{i=1}^p \nu_i h_i(\mathbf{x})$$
The Lagrange dual function: $g(\boldsymbol{\lambda}, \boldsymbol{\nu}) = \inf_{\mathbf{x}} \mathcal{L}(\mathbf{x}, \boldsymbol{\lambda}, \boldsymbol{\nu})$.
**Weak Duality**: $g(\boldsymbol{\lambda}, \boldsymbol{\nu}) \le p^*$ for all $\boldsymbol{\lambda} \ge \mathbf{0}$.
**Strong Duality** ($d^* = p^*$): Holds if Slater's condition is satisfied (strictly feasible point exists).

### 1.4 Karush-Kuhn-Tucker (KKT) Optimality Conditions
For convex problems with strong duality, $\mathbf{x}^*$ and $(\boldsymbol{\lambda}^*, \boldsymbol{\nu}^*)$ are optimal if and only if:
1. **Primal Feasibility**: $f_i(\mathbf{x}^*) \le 0, \quad h_i(\mathbf{x}^*) = 0$
2. **Dual Feasibility**: $\lambda_i^* \ge 0$
3. **Complementary Slackness**: $\lambda_i^* f_i(\mathbf{x}^*) = 0$ for all $i$
4. **Stationarity**: $\nabla f_0(\mathbf{x}^*) + \sum_{i=1}^m \lambda_i^* \nabla f_i(\mathbf{x}^*) + \sum_{i=1}^p \nu_i^* \nabla h_i(\mathbf{x}^*) = \mathbf{0}$

---

## 2. The Vibe Coding Superpower

1. **Optimal Agent Budget Allocation**: Solve the tradeoff between token cost and response latency: minimize cost subject to quality $\ge Q_{\text{min}}$ and latency $\le T_{\text{max}}$.
2. **Guaranteed Global Convergence**: Recognizing whether an optimization problem is convex ensures you never get trapped in sub-optimal local minima.
3. **Projected Gradient Solvers**: Build instant in-browser constrained solvers for financial rebalancing or load dispatch without heavy third-party solvers.

---

## 3. Production Code Implementations

### 3.1 Projected Gradient Descent on Simplex in Python
```python
import numpy as np

def project_onto_probability_simplex(v: np.ndarray) -> np.ndarray:
    """
    Project vector v onto probability simplex sum(x) = 1, x >= 0.
    Exact O(n log n) algorithm.
    """
    n = len(v)
    u = np.sort(v)[::-1]
    cssv = np.cumsum(u)
    rho = np.nonzero(u * np.arange(1, n + 1) > (cssv - 1.0))[0][-1]
    theta = (cssv[rho] - 1.0) / (rho + 1.0)
    return np.maximum(v - theta, 0.0)

def allocate_agent_budget(
    costs: np.ndarray,
    latencies: np.ndarray,
    budget: float,
    lr: float = 0.01,
    steps: int = 500
) -> np.ndarray:
    """
    Find model mixture weights x on simplex minimizing latency subject to cost <= budget.
    """
    n = len(costs)
    x = np.ones(n) / n  # start at centroid

    for _ in range(steps):
        # Gradient of latency objective
        grad = latencies.copy()
        # Penalty if cost exceeds budget
        current_cost = np.dot(x, costs)
        if current_cost > budget:
            grad += 10.0 * (current_cost - budget) * costs

        # Descent step
        x = x - lr * grad
        # Project onto simplex
        x = project_onto_probability_simplex(x)

    return x
```

### 3.2 KKT Complementary Slackness Verifier in Rust
```rust
pub struct KktValidator;

impl KktValidator {
    pub fn check_complementary_slackness(
        lambda: &[f64],
        constraints: &[f64],
        tol: f64
    ) -> bool {
        assert_eq!(lambda.len(), constraints.len());
        for (&lam, &g) in lambda.iter().zip(constraints.iter()) {
            if lam < -tol {
                return false; // Dual feasibility violation
            }
            if (lam * g).abs() > tol {
                return false; // Complementary slackness violation
            }
        }
        true
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Constraint Optimization
```markdown
When solving resource allocation or routing tradeoffs:
- Clearly define the primal objective, inequality constraints, and equality constraints.
- Formulate the problem in convex standard form (minimize convex f_0, s.t. f_i <= 0).
- State the KKT optimality conditions to verify global optimality.
```
