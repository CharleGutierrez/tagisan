---
name: math-algorithmic-game-theory
description: Algorithmic game theory: Nash equilibria, Price of Anarchy, mechanism design, truthful VCG auctions, and Shapley values for multi-agent credit assignment based on 'Algorithmic Game Theory' by Nisan, Roughgarden, Tardos, and Vazirani. Triggers: algorithmic-game-theory, nisan-roughgarden, nash-equilibrium, shapley-value, mechanism-design, vcg-auction, price-of-anarchy, multi-agent-incentives, congestion-games, cooperative-game-theory.
triggers:
  - algorithmic-game-theory
  - nisan-roughgarden
  - nash-equilibrium
  - shapley-value
  - mechanism-design
  - vcg-auction
  - price-of-anarchy
  - multi-agent-incentives
  - congestion-games
  - cooperative-game-theory
---

# Algorithmic Game Theory: Nash, Shapley & Mechanism Design

## 1. Mathematical Foundations & Formal Principles

### 1.1 Strategic Games & Nash Equilibrium
A normal-form game consists of players $N = \{1, \dots, n\}$, strategy sets $S_i$, and payoff functions $u_i(s_1, \dots, s_n)$.
A strategy profile $s^* = (s_1^*, \dots, s_n^*)$ is a **Nash Equilibrium** if no player can unilaterally deviate to improve payoff:
$$u_i(s_i^*, s_{-i}^*) \ge u_i(s_i, s_{-i}^*) \quad \forall s_i \in S_i, \; \forall i \in N$$
**Nash's Theorem**: Every finite game possesses at least one mixed-strategy Nash equilibrium.

### 1.2 Price of Anarchy (PoA)
Quantifies efficiency loss due to selfish behavior compared to social optimum:
$$\text{PoA} = \frac{\max_{s} \sum_i u_i(s)}{\min_{s \in \text{NE}} \sum_i u_i(s)}$$
In congestion games and routing, PoA measures network degradation under selfish decentralized routing.

### 1.3 Truthful Mechanism Design & VCG Auctions
In a Vickrey-Clarke-Groves (VCG) auction:
1. Each agent reports value $v_i$.
2. Center picks allocation $x^* = \arg\max_x \sum_j v_j(x)$.
3. Agent $i$ pays the externality they impose on all other agents:
   $$p_i = \max_x \sum_{j \neq i} v_j(x) - \sum_{j \neq i} v_j(x^*)$$
**Dominant-Strategy Truthfulness**: Truth-telling ($v_i = \text{true\_value}$) is always the dominant strategy.

### 1.4 Cooperative Game Theory & Shapley Value
For coalition game $(N, v)$ with characteristic function $v(S)$, the unique fair payoff attribution satisfying Efficiency, Symmetry, Dummy Player, and Additivity axioms is the **Shapley Value**:
$$\phi_i(v) = \sum_{S \subseteq N \setminus \{i\}} \frac{|S|!(|N| - |S| - 1)!}{|N|!} (v(S \cup \{i\}) - v(S))$$
Where $v(S \cup \{i\}) - v(S)$ is agent $i$'s marginal contribution to coalition $S$.

---

## 2. The Vibe Coding Superpower

1. **Fair Multi-Agent Credit Assignment**: Attribute revenue or task tokens fairly among specialized subagents using Shapley values.
2. **Truthful Compute Auctions**: Prevent multi-agent bidding wars from starving resources by running second-price VCG auctions.
3. **Decentralized Coordination Without Deadlocks**: Structure incentives so that autonomous agents converge to Nash equilibrium without central supervision.

---

## 3. Production Code Implementations

### 3.1 Shapley Value Attribution for Multi-Agent Swarms in Python
```python
from itertools import combinations
import math
from typing import Callable

def compute_shapley_values(
    agents: list[str],
    characteristic_fn: Callable[[frozenset], float]
) -> dict[str, float]:
    """
    Compute exact Shapley value for each agent measuring marginal contribution across all coalitions.
    """
    n = len(agents)
    shapley = {agent: 0.0 for agent in agents}

    for agent in agents:
        other_agents = [a for a in agents if a != agent]
        # Sum over all subset sizes |S| = 0 to n-1
        for s_size in range(n):
            weight = (math.factorial(s_size) * math.factorial(n - s_size - 1)) / math.factorial(n)
            for subset in combinations(other_agents, s_size):
                s_without = frozenset(subset)
                s_with = frozenset(subset + (agent,))
                marginal_contrib = characteristic_fn(s_with) - characteristic_fn(s_without)
                shapley[agent] += weight * marginal_contrib

    return shapley
```

### 3.2 VCG Second-Price Auction Solver in Rust
```rust
pub struct Bid {
    pub bidder_id: String,
    pub amount: f64,
}

pub struct VcgAuctionResult {
    pub winner_id: String,
    pub price_paid: f64,
}

pub fn run_second_price_vcg_auction(bids: &[Bid]) -> Option<VcgAuctionResult> {
    if bids.is_empty() {
        return None;
    }
    let mut sorted_bids = bids.to_vec();
    sorted_bids.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());

    let winner = &sorted_bids[0];
    let second_price = if sorted_bids.len() > 1 {
        sorted_bids[1].amount
    } else {
        0.0 // Reserve price
    };

    Some(VcgAuctionResult {
        winner_id: winner.bidder_id.clone(),
        price_paid: second_price,
    })
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Multi-Agent Game Theory
```markdown
When designing multi-agent coordination and revenue splits:
- Assign credit to collaborative agents based on their marginal Shapley contribution v(S U {i}) - v(S).
- Structure auction bidding using VCG mechanisms so that reporting true computational cost is the strictly dominant strategy.
- Analyze agent failure loops as Nash equilibria: modify the payoff matrix to eliminate toxic defect-defect equilibria.
```
