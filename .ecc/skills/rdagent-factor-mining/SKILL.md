---
name: rdagent-factor-mining
description: Microsoft RD-Agent autonomous quantitative R&D, automated alpha factor mining, market anomaly hypothesis formulation, symbolic factor expression trees, collinearity filtering, and factor half-life decay management. Triggers: rdagent, rd-agent, autonomous alpha mining, rd-agent alpha mining, factor mining, hypothesis generation, factor expression tree, automated quant r&d, factor decay.
triggers:
  - rdagent
  - rd-agent
  - rdagent factor mining
  - rd-agent alpha mining
  - autonomous alpha mining
  - factor mining
  - hypothesis generation
  - factor expression tree
  - automated quant r&d
  - factor decay
---

# Microsoft RD-Agent Autonomous Alpha Mining Skill

## 1. Architectural Foundations

Microsoft RD-Agent operationalizes scientific discovery in quantitative finance through multi-agent collaboration loops consisting of Hypothesis Generators, Factor Programmers, Backtest Critics, and Factor Archive Curators.

### Core Autonomous R&D Loop:
1. **Financial Hypothesis Formulation**: Rather than generating brute-force data-mined noise, agents formulate economically sound hypotheses grounded in market anomalies:
   - Liquidity & order flow asymmetry
   - Post-earnings announcement drift (PEAD)
   - Idiosyncratic volatility puzzle
   - Institutional retail order imbalance
2. **Symbolic Factor Expression Tree Generation**: Map qualitative economic hypotheses into executable domain-specific expression trees using composable mathematical primitives.
3. **Rigorous Out-of-Sample (OOS) Validation**: Test synthesized alpha factors across bull, bear, high-volatility, and stagflation regimes.
4. **Collinearity Pruning & Orthogonal Factor Library**: Discard candidate factors exhibiting high cross-sectional correlation ($\rho > 0.60$) with existing library factors.
5. **Factor Half-Life & Decay Monitoring**: Continuously track alpha decay curves, automatically triggering retirement when predictive power fades.

---

## 2. Mathematical Formulations & Expression Trees

### 2.1 Factor Expression Grammar & Grammar Trees
Factors are represented as directed acyclic expression trees $\mathcal{T}$ over operators:
- **Unary Time-Series Operators**: $\text{Ts\_Mean}(X, d)$, $\text{Ts\_Std}(X, d)$, $\text{Ts\_Rank}(X, d)$, $\text{Ts\_Max}(X, d)$, $\text{Ts\_DecayLinear}(X, d)$.
- **Binary Cross-Sectional Operators**: $\text{Cs\_Rank}(X)$, $\text{Cs\_ZScore}(X)$, $\text{Cs\_Neutralize}(X, \text{Industry})$.
- **Arithmetic Operators**: $+$, $-$, $\times$, $\div$, $\log(|X| + \epsilon)$.

Example Factor Expression Tree:
$$\text{Factor} = \text{Cs\_Rank}\left(\frac{\text{Ts\_Mean}(\$close, 5) - \$close}{\text{Ts\_Std}(\$close, 20) + \epsilon}\right) \times \text{Ts\_DecayLinear}\left(\frac{\$volume}{\text{Ts\_Mean}(\$volume, 20)}, 10\right)$$

### 2.2 Factor Collinearity & Diversity Filtering
For candidate factor vector $\mathbf{f}_{\text{cand}}$ and existing factor archive $\{\mathbf{f}_1, \dots, \mathbf{f}_M\}$:
$$\rho_{\max} = \max_{m \in [1, M]} \left|\text{SpearmanCorr}(\mathbf{f}_{\text{cand}}, \mathbf{f}_m)\right|$$
If $\rho_{\max} > 0.60$, the candidate factor is rejected to protect against portfolio risk concentration and redundant turnover.

### 2.3 Factor Decay & Half-Life Estimation
The longitudinal predictive capability of an alpha factor decays as market participants arbitrage the signal:
$$\text{RankIC}(t) = \text{RankIC}_0 \cdot 2^{-t / \tau} + \epsilon_t$$
where $\tau$ is the empirical half-life in trading days:
$$\tau = -\frac{\ln(2)}{\beta_{\text{decay}}}, \quad \text{estimated via } \ln(\text{RankIC}(t)) = \alpha - \beta_{\text{decay}} \cdot t$$
Deprecate factors when $\text{ICIR}_{\text{trailing 60d}} < 0.30$ or turnover exceeds 40% daily.

---

## 3. Python Factor Tree Generation Engine

```python
import numpy as np
import pandas as pd
from typing import Callable, Dict, Any

class FactorNode:
    """
    Symbolic Expression Tree Node for RD-Agent factor generation.
    """
    def __init__(self, op_name: str, func: Callable, children: list = None):
        self.op_name = op_name
        self.func = func
        self.children = children or []

    def evaluate(self, df: pd.DataFrame) -> pd.Series:
        child_vals = [c.evaluate(df) if isinstance(c, FactorNode) else df[c] for c in self.children]
        return self.func(*child_vals)

    def to_string(self) -> str:
        child_strs = [c.to_string() if isinstance(c, FactorNode) else str(c) for c in self.children]
        return f"{self.op_name}({', '.join(child_strs)})"

# Primitive Quantitative Operators
def ts_mean(s: pd.Series, window: int) -> pd.Series:
    return s.rolling(window).mean()

def ts_std(s: pd.Series, window: int) -> pd.Series:
    return s.rolling(window).std()

def cs_rank(s: pd.Series) -> pd.Series:
    return s.rank(pct=True)

def sub_op(a: pd.Series, b: pd.Series) -> pd.Series:
    return a - b

def div_op(a: pd.Series, b: pd.Series) -> pd.Series:
    return a / (b + 1e-8)

def construct_mean_reversion_factor_tree() -> FactorNode:
    """
    Constructs: Cs_Rank( Div( Sub( Ts_Mean(close, 5), close ), Ts_Std(close, 20) ) )
    """
    mean_node = FactorNode("Ts_Mean", ts_mean, ["close", 5])
    sub_node = FactorNode("Sub", sub_op, [mean_node, "close"])
    std_node = FactorNode("Ts_Std", ts_std, ["close", 20])
    div_node = FactorNode("Div", div_op, [sub_node, std_node])
    return FactorNode("Cs_Rank", cs_rank, [div_node])
```

---

## 4. Quantitative Safety & R&D Governance

1. **Multiple Testing Overfitting Correction**: Enforce Holm-Bonferroni or Deflated Sharpe Ratio (DSR) adjustments to p-values to account for the total number of trial hypotheses tested by autonomous agents.
2. **Formula Complexity Penalty**: Penalize trees with depth $> 6$ or node count $> 15$ to avoid brittle data-mined curve-fitting.
3. **Execution Cost Reality Check**: Require candidate alphas to achieve a net annualized Sharpe $> 1.5$ after 15 bps two-way friction before passing to production paper-trading.
