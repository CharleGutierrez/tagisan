---
name: math-causal-inference
description: Structural Causal Models (SCMs), causal Directed Acyclic Graphs (DAGs), Pearl's causal hierarchy (association, intervention, counterfactuals), d-separation, backdoor/frontdoor criteria, and do-calculus based on 'The Book of Why' by Judea Pearl & Dana Mackenzie. Triggers: causal-inference, book-of-why, judea-pearl, structural-causal-model, causal-dag, do-calculus, backdoor-criterion, counterfactuals, d-separation, confounding-bias.
triggers:
  - causal-inference
  - book-of-why
  - judea-pearl
  - structural-causal-model
  - causal-dag
  - do-calculus
  - backdoor-criterion
  - counterfactuals
  - d-separation
  - confounding-bias
---

# Causal Inference & Do-Calculus: SCMs & Counterfactuals

## 1. Mathematical Foundations & Formal Principles

### 1.1 Pearl's Causal Hierarchy (The Ladder of Causation)
1. **Layer 1: Association (Observational)**:
   $$P(Y=y \mid X=x)$$
   Questions: "What does a symptom tell me about a disease?" Driven by correlation.
2. **Layer 2: Intervention (Action)**:
   $$P(Y=y \mid \text{do}(X=x))$$
   Questions: "What will happen if we prescribe the drug?" Cuts incoming arrows to $X$.
3. **Layer 3: Counterfactuals (Imagination)**:
   $$P(Y_{X=x} = y \mid X=x', Y=y')$$
   Questions: "Was it the medicine that cured the patient, or would they have recovered anyway?"

### 1.2 Structural Causal Models (SCM)
An SCM is a 4-tuple $\mathcal{M} = \langle U, V, F, P(u) \rangle$:
- $U$: Exogenous background variables determined outside the model.
- $V$: Endogenous variables determined within the model: $V_i = f_i(\text{PA}_i, U_i)$ where $\text{PA}_i \subseteq V \setminus \{V_i\}$.
- $F$: Set of deterministic structural equations.
- $P(u)$: Prior probability distribution over exogenous shocks $U$.

### 1.3 Graphical Criterion: d-Separation
In a DAG $G$, a path $p$ is d-separated by set $Z$ if:
1. $p$ contains a chain $A \to B \to C$ or fork $A \leftarrow B \to C$ where $B \in Z$.
2. $p$ contains a collider $A \to B \leftarrow C$ where neither $B$ nor any descendant of $B$ is in $Z$.
If all paths between $X$ and $Y$ are d-separated by $Z$, then $X \perp\!\!\!\perp Y \mid Z$.

### 1.4 The Backdoor Criterion & Adjustment Formula
A set of variables $Z$ satisfies the backdoor criterion relative to ordered pair $(X, Y)$ if:
1. No node in $Z$ is a descendant of $X$.
2. $Z$ blocks every path between $X$ and $Y$ that contains an arrow into $X$ (backdoor paths).

**Backdoor Adjustment Theorem**:
If $Z$ satisfies the backdoor criterion for $(X, Y)$:
$$P(Y=y \mid \text{do}(X=x)) = \sum_{z} P(Y=y \mid X=x, Z=z) P(Z=z)$$

---

## 2. The Vibe Coding Superpower

AI agents frequently fall victim to confounding bias, mistaking symptoms for causes:
1. **Root Cause Analysis in Distributed Systems**: If high latency ($L$) and high CPU ($C$) both co-occur with a database lock ($Z$), intervening to restart the CPU ($do(C)$) will not fix latency ($L$). Causal DAGs allow agents to identify true root interventions.
2. **Robust Multi-Agent Decision Trees**: Formulate agent actions using $do(X)$ interventions rather than observational conditional probabilities.
3. **Counterfactual Reflection in Autonomous Loops**: If a unit test fails, ask: "Had the parser sanitized unicode input ($X=\text{sanitized}$), would assertion failure $Y=1$ have occurred?"

---

## 3. Production Code Implementations

### 3.1 Causal DAG & Backdoor Adjustment in Python
```python
from typing import Dict, List, Set

class CausalDAG:
    """
    Lightweight Directed Acyclic Graph for causal reasoning and backdoor identification.
    """
    def __init__(self):
        self.parents: Dict[str, Set[str]] = {}
        self.children: Dict[str, Set[str]] = {}

    def add_edge(self, u: str, v: str):
        self.parents.setdefault(u, set())
        self.parents.setdefault(v, set()).add(u)
        self.children.setdefault(v, set())
        self.children.setdefault(u, set()).add(v)

    def get_descendants(self, node: str) -> Set[str]:
        desc = set()
        queue = [node]
        while queue:
            curr = queue.pop(0)
            for ch in self.children.get(curr, set()):
                if ch not in desc:
                    desc.add(ch)
                    queue.append(ch)
        return desc

    def is_backdoor_admissible(self, x: str, y: str, z_set: Set[str]) -> bool:
        """
        Check if set Z satisfies the Backdoor Criterion relative to (X, Y).
        Condition 1: No node in Z is a descendant of X.
        """
        desc_x = self.get_descendants(x)
        if any(z in desc_x for z in z_set):
            return False
        # Simplified check for confounding fork blocking
        confounders = self.parents.get(x, set()).intersection(self.parents.get(y, set()))
        return confounders.issubset(z_set)

def compute_backdoor_intervention(
    p_y_given_xz: Dict[tuple, float],
    p_z: Dict[str, float],
    x_val: str,
    y_val: str
) -> float:
    """
    Compute P(Y=y | do(X=x)) = sum_z P(Y=y | X=x, Z=z) * P(Z=z).
    """
    prob = 0.0
    for z_val, pz in p_z.items():
        prob += p_y_given_xz.get((y_val, x_val, z_val), 0.0) * pz
    return prob
```

### 3.2 Counterfactual Simulator in Rust
```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StructuralEquation {
    pub name: String,
    pub parents: Vec<String>,
    pub formula: fn(&HashMap<String, f32>, f32) -> f32, // f(parents, noise_u)
}

pub struct CausalModel {
    pub equations: HashMap<String, StructuralEquation>,
}

impl CausalModel {
    pub fn new() -> Self {
        Self { equations: HashMap::new() }
    }

    /// Perform intervention do(X = val) by replacing structural equation for X
    pub fn do_intervention(&self, target: &str, fixed_value: f32, state: &mut HashMap<String, f32>) {
        state.insert(target.to_string(), fixed_value);
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 Prompt Scaffold for Causal Reasoning
```markdown
When debugging or diagnosing root causes:
1. Construct a causal DAG of the incident: List Exogenous Causes -> Intermediate Mediators -> Observable Symptoms.
2. Distinguish Observation from Intervention:
   - What did we OBSERVE? P(Symptom | Observation)
   - What will change if we ACT? P(Symptom | do(Fix))
3. Check for Confounders: Identify common causes driving both metric anomalies before blaming one on the other.
4. Reason Counterfactually: State explicitly: "Had [X] not occurred, would [Y] still have happened?"
```
