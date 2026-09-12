---
name: math-networks-crowds-markets
description: Graph theory, network centrality, PageRank, information cascades, agent swarm topologies, and market dynamics based on 'Networks, Crowds, and Markets' by David Easley and Jon Kleinberg. Triggers: networks-crowds-markets, easley-kleinberg, pagerank, network-centrality, betweenness-centrality, information-cascades, agent-swarm-topologies, graph-rag, epidemic-sir, structural-balance.
triggers:
  - networks-crowds-markets
  - easley-kleinberg
  - pagerank
  - network-centrality
  - betweenness-centrality
  - information-cascades
  - agent-swarm-topologies
  - graph-rag
  - epidemic-sir
  - structural-balance
---

# Networks & Swarms: PageRank, Centrality & Cascades

## 1. Mathematical Foundations & Formal Principles

### 1.1 Graph Topology & Centrality Measures
A network is a graph $G = (V, E)$ with adjacency matrix $\mathbf{A}$.
- **Degree Centrality**: $C_D(v) = \deg(v)$
- **Closeness Centrality**: $C_C(v) = \frac{1}{\sum_{u \neq v} d(v, u)}$
- **Betweenness Centrality**:
  $$C_B(v) = \sum_{s \neq v \neq t} \frac{\sigma_{st}(v)}{\sigma_{st}}$$
  Where $\sigma_{st}$ is the number of shortest paths from $s$ to $t$, and $\sigma_{st}(v)$ pass through $v$. Critical for detecting communication choke-points.

### 1.2 PageRank (Random Walk with Restart)
Let $\mathbf{P}$ be the column-stochastic transition matrix: $P_{ij} = \frac{A_{ji}}{\deg_{\text{out}}(j)}$.
With damping factor $d \in (0, 1)$ (typically $d = 0.85$):
$$\mathbf{r} = d \mathbf{P} \mathbf{r} + \frac{1 - d}{N} \mathbf{1}$$
Power iteration solves the principal eigenvector corresponding to $\lambda_1 = 1$:
$$\mathbf{r}^{(t+1)} = d \mathbf{P} \mathbf{r}^{(t)} + \frac{1 - d}{N} \mathbf{1}$$

### 1.3 Information Cascades & Viral Thresholds
In a population where agents make sequential decisions under noisy private signals:
When public evidence accumulated from previous agents outweighs an individual's private signal, an **information cascade** occurs: rational agents ignore their private data and mimic the herd.

---

## 2. The Vibe Coding Superpower

1. **Multi-Agent Swarm Topology Design**: Prevent message explosion in agent networks. Use betweenness centrality to identify bottlenecks and PageRank to select consensus coordinators.
2. **Graph-RAG Retrieval**: Rank knowledge graph entities by PageRank importance before querying vector embeddings.
3. **Cascade Control in Agent Debates**: Prevent groupthink in Mixture-of-Agents swarms by blinding intermediate agents from early votes.

---

## 3. Production Code Implementations

### 3.1 PageRank Power Iteration in Python
```python
import numpy as np

def compute_pagerank(
    adj_matrix: np.ndarray,
    damping: float = 0.85,
    max_iter: int = 100,
    tol: float = 1e-6
) -> np.ndarray:
    """
    Compute PageRank vector using power iteration.
    """
    n = adj_matrix.shape[0]
    out_degree = np.sum(adj_matrix, axis=1)

    # Transition probability matrix (row-stochastic)
    p = np.zeros_like(adj_matrix, dtype=float)
    for i in range(n):
        if out_degree[i] > 0:
            p[i, :] = adj_matrix[i, :] / out_degree[i]
        else:
            p[i, :] = 1.0 / n  # Teleport if sink

    rank = np.ones(n) / n
    teleport = np.ones(n) / n

    for _ in range(max_iter):
        new_rank = damping * np.dot(p.T, rank) + (1.0 - damping) * teleport
        if np.linalg.norm(new_rank - rank, 1) < tol:
            break
        rank = new_rank

    return rank
```

### 3.2 Betweenness Centrality Tracker in Rust
```rust
use std::collections::{HashMap, VecDeque};

pub struct GraphTopology {
    pub adj: HashMap<usize, Vec<usize>>,
}

impl GraphTopology {
    pub fn new() -> Self {
        Self { adj: HashMap::new() }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.adj.entry(u).or_default().push(v);
        self.adj.entry(v).or_default().push(u);
    }

    /// Brandes algorithm for betweenness centrality
    pub fn betweenness_centrality(&self, n: usize) -> Vec<f32> {
        let mut cb = vec![0.0f32; n];

        for s in 0..n {
            let mut stack = Vec::new();
            let mut paths = vec![Vec::new(); n];
            let mut sigma = vec![0.0f32; n];
            sigma[s] = 1.0;
            let mut dist = vec![-1i32; n];
            dist[s] = 0;

            let mut queue = VecDeque::new();
            queue.push_back(s);

            while let Some(v) = queue.pop_front() {
                stack.push(v);
                if let Some(neighbors) = self.adj.get(&v) {
                    for &w in neighbors {
                        if dist[w] < 0 {
                            dist[w] = dist[v] + 1;
                            queue.push_back(w);
                        }
                        if dist[w] == dist[v] + 1 {
                            sigma[w] += sigma[v];
                            paths[w].push(v);
                        }
                    }
                }
            }

            let mut delta = vec![0.0f32; n];
            while let Some(w) = stack.pop() {
                for &v in &paths[w] {
                    delta[v] += (sigma[v] / sigma[w].max(1e-8)) * (1.0 + delta[w]);
                }
                if w != s {
                    cb[w] += delta[w];
                }
            }
        }

        cb
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Agent Swarm Topologies
```markdown
When organizing multi-agent architectures:
- Identify bridge nodes that connect disparate knowledge silos using betweenness centrality.
- If information cascades or premature consensus is detected, inject a contrarian red-team agent.
- Prioritize memory retrieval using PageRank on entity-relationship graphs.
```
