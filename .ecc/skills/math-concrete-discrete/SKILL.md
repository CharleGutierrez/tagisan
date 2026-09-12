---
name: math-concrete-discrete
description: Concrete mathematics: sums and recurrences, binomial coefficients, generating functions, Stirling numbers, and context window asymptotic analysis based on 'Concrete Mathematics' by Graham, Knuth, and Patashnik. Triggers: concrete-discrete, concrete-mathematics, knuth, recurrence-relations, generating-functions, binomial-coefficients, stirling-numbers, finite-calculus, asymptotic-analysis, token-budget-recurrence.
triggers:
  - concrete-discrete
  - concrete-mathematics
  - knuth
  - recurrence-relations
  - generating-functions
  - binomial-coefficients
  - stirling-numbers
  - finite-calculus
  - asymptotic-analysis
  - token-budget-recurrence
---

# Concrete Mathematics: Recurrences & Generating Functions

## 1. Mathematical Foundations & Formal Principles

### 1.1 Recurrence Relations & The Repertoire Method
A linear recurrence of order $k$:
$$a_n = c_1 a_{n-1} + c_2 a_{n-2} + \dots + c_k a_{n-k} + g(n)$$
Divide-and-conquer recurrences (Master Theorem):
$$T(n) = a T(n/b) + f(n)$$
Solved via expansion, recursion trees, or repertoire substitution.

### 1.2 Ordinary Generating Functions (OGF)
The generating function of a sequence $\langle g_0, g_1, g_2, \dots \rangle$ is the formal power series:
$$G(z) = \sum_{n=0}^\infty g_n z^n$$
Operations:
- **Right Shift**: $z G(z) = \sum_{n=1}^\infty g_{n-1} z^n$
- **Differentiation**: $G'(z) = \sum_{n=1}^\infty n g_n z^{n-1}$
- **Convolution**: $A(z) B(z) = \sum_{n=0}^\infty \left(\sum_{k=0}^n a_k b_{n-k}\right) z^n$

### 1.3 Binomial Coefficients & Identities
$$\binom{n}{k} = \frac{n!}{k! (n-k)!} = \frac{n(n-1)\dots(n-k+1)}{k!}$$
- **Symmetry**: $\binom{n}{k} = \binom{n}{n-k}$
- **Pascal's Identity**: $\binom{n}{k} = \binom{n-1}{k} + \binom{n-1}{k-1}$
- **Vandermonde Convolution**:
  $$\sum_{k=0}^r \binom{m}{k}\binom{n}{r-k} = \binom{m+n}{r}$$

---

## 2. The Vibe Coding Superpower

1. **Exact Token Budget Modeling**: Calculate exact token accumulation in recursive agent chains: $T(d) = b \cdot T(d-1) + C$ before launching runaway recursive queries.
2. **Combinatoric Tree Search Bounds**: Compute state-space size for Monte Carlo Tree Search (MCTS) in agentic code generation.
3. **Generating Function Caching**: Precompute combinatorial probability tables using polynomial convolution.

---

## 3. Production Code Implementations

### 3.1 Token Consumption Recurrence Estimator in Python
```python
def estimate_recursive_token_usage(
    depth: int,
    branching_factor: int,
    tokens_per_step: int,
    history_multiplier: float = 1.2
) -> dict:
    """
    Solve closed-form recurrence for recursive agent tree token consumption.
    T(d) = branching_factor * T(d-1) * history_multiplier + tokens_per_step
    """
    total_tokens = 0
    current_tokens_per_call = tokens_per_step
    calls_at_level = 1

    level_breakdown = []
    for d in range(depth + 1):
        tokens_at_level = calls_at_level * int(current_tokens_per_call)
        total_tokens += tokens_at_level
        level_breakdown.append({
            "depth": d,
            "agent_calls": calls_at_level,
            "tokens": tokens_at_level
        })
        calls_at_level *= branching_factor
        current_tokens_per_call *= history_multiplier

    return {
        "total_tokens": total_tokens,
        "levels": level_breakdown
    }
```

### 3.2 Binomial Coefficient with Overflow Guard in Rust
```rust
pub fn binomial_coefficient(n: u64, mut k: u64) -> Option<u64> {
    if k > n {
        return Some(0);
    }
    if k == 0 || k == n {
        return Some(1);
    }
    if k > n - k {
        k = n - k; // Symmetry identity
    }

    let mut result: u64 = 1;
    for i in 1..=k {
        result = result.checked_mul(n - k + i)?;
        result /= i;
    }
    Some(result)
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Algorithmic Complexity
```markdown
When analyzing recursive agent loops or data structures:
- Provide exact closed-form recurrences rather than vague Big-O approximations.
- Set hard upper bounds on recursion depth to prevent exponential token consumption.
- Use generating functions to model multi-stage token decay.
```
