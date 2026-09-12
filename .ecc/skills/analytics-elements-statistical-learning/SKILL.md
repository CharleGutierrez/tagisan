---
name: analytics-elements-statistical-learning
description: Advanced statistical learning theory: Support Vector Machines (maximal margin hyperplanes), KKT optimality conditions, kernel methods, boosting, and cost-complexity pruning. Triggers: elements-statistical-learning, esl, statistical-learning-theory, svm-margin, kernel-tricks, boosting-trees, cost-complexity-pruning.
triggers:
  - elements-statistical-learning
  - esl
  - statistical-learning-theory
  - svm-margin
  - kernel-tricks
  - boosting-trees
  - cost-complexity-pruning
---

# Analytics Elements Statistical Learning
> Based on **The Elements of Statistical Learning - Trevor Hastie, Robert Tibshirani, Jerome Friedman**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Statistical Theory Algorithm Benchmarks
CREATE TABLE learning_theory_benchmarks (
    algorithm VARCHAR(50) PRIMARY KEY,
    loss_function VARCHAR(50) NOT NULL,
    generalization_bound DOUBLE PRECISION NOT NULL,
    empirical_risk DOUBLE PRECISION NOT NULL,
    structural_risk DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Support Vector Machine Maximal Margin Formulation
$$\min_{w, b, \xi} \frac{1}{2} \|w\|^2 + C \sum_{i=1}^n \xi_i$$
subject to:
$$y_i (w^T \phi(x_i) + b) \ge 1 - \xi_i, \quad \xi_i \ge 0 \quad \forall i$$
Karush-Kuhn-Tucker (KKT) complementary slackness condition:
$$\alpha_i [y_i(w^T \phi(x_i) + b) - 1 + \xi_i] = 0$$

### 2.2 Cost-Complexity Tree Pruning
$$C_\alpha(T) = R(T) + \alpha |T|$$
where $R(T)$ is misclassification risk, $|T|$ is terminal leaf count, and $\alpha$ is complexity penalty.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[Input Space X] --> Kernel[Kernel Mapping: Inner Product in Hilbert Space K(x, x')]
    Kernel --> SolveDual[Solve Dual Quadratic Optimization for alpha_i]
    SolveDual --> SupportVectors[Identify Support Vectors: alpha_i > 0]
    SupportVectors --> DecisionBoundary[Construct Optimal Separating Hyperplane]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Radial Basis Function (RBF) Kernel in NumPy
import numpy as np

def rbf_kernel(X1: np.ndarray, X2: np.ndarray, gamma: float = 0.1) -> np.ndarray:
    # K(x, y) = exp(-gamma * ||x - y||^2)
    dist_sq = np.sum(X1**2, 1).reshape(-1, 1) + np.sum(X2**2, 1) - 2 * np.dot(X1, X2.T)
    return np.exp(-gamma * dist_sq)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Support Vector Machines maximize the geometric margin 2 / ||w|| between classes.
- Only data points lying on the margin (support vectors, alpha > 0) influence the decision boundary.
- The Kernel Trick computes inner products in high-dimensional Hilbert spaces without explicit mapping.
- Tree cost-complexity pruning balances in-sample error R(T) against tree size alpha * |T|.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Deploy advanced statistical learning algorithms:
1. Implement maximal-margin quadratic programming solvers with soft-margin slack penalties.
2. Build Mercer-compliant kernel functions (RBF, Polynomial) enabling non-linear boundary separation.
3. Formulate cost-complexity pruning routines optimizing decision tree generalization limits.
```
