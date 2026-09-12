---
name: math-information-inference
description: Information theory, Shannon entropy, mutual information, KL divergence, cross-entropy, perplexity, Bayesian evidence, and hallucination detection based on 'Information Theory, Inference, and Learning Algorithms' by David J.C. MacKay. Triggers: information-theory, shannon-entropy, mutual-information, kl-divergence, cross-entropy, perplexity, hallucination-detector, bayesian-evidence, entropy-gating, mackay-inference.
triggers:
  - information-theory
  - shannon-entropy
  - mutual-information
  - kl-divergence
  - cross-entropy
  - perplexity
  - hallucination-detector
  - bayesian-evidence
  - entropy-gating
  - mackay-inference
---

# Information Theory, Shannon Entropy & Inference

## 1. Mathematical Foundations & Formal Principles

### 1.1 Shannon Entropy & Information Measure
The information content (surprisal) of an event $x$ with probability $P(x)$ is $I(x) = \log_2 \frac{1}{P(x)} = -\log_2 P(x)$ bits.
The Shannon Entropy $H(X)$ of a discrete random variable $X \sim P(x)$ is the expected surprisal:
$$H(X) = \mathbb{E}[I(X)] = -\sum_{x \in \mathcal{X}} P(x) \log_2 P(x)$$
For base $e$ (nats): $H_e(X) = -\sum P(x) \ln P(x)$.
Maximum entropy occurs under the uniform distribution: $H(X) \le \log_2 |\mathcal{X}|$.

### 1.2 Joint, Conditional Entropy & Mutual Information
- **Joint Entropy**: $H(X, Y) = -\sum_{x, y} P(x, y) \log_2 P(x, y)$
- **Conditional Entropy**: $H(Y|X) = -\sum_{x, y} P(x, y) \log_2 P(y|x)$
- **Chain Rule**: $H(X, Y) = H(X) + H(Y|X)$
- **Mutual Information**:
  $$I(X; Y) = H(X) - H(X|Y) = \sum_{x, y} P(x, y) \log_2 \frac{P(x, y)}{P(x)P(y)}$$
  $I(X; Y) \ge 0$ with equality if and only if $X$ and $Y$ are statistically independent.

### 1.3 Kullback-Leibler (KL) Divergence & Cross-Entropy
The relative entropy (KL divergence) between true distribution $P$ and approximating distribution $Q$:
$$D_{KL}(P \parallel Q) = \sum_{x \in \mathcal{X}} P(x) \log \frac{P(x)}{Q(x)}$$
By Gibbs' inequality: $D_{KL}(P \parallel Q) \ge 0$.
**Cross-Entropy**:
$$H(P, Q) = -\sum_{x} P(x) \log Q(x) = H(P) + D_{KL}(P \parallel Q)$$

### 1.4 Perplexity
For a discrete language model sequence $(w_1, \dots, w_N)$:
$$\text{PPL} = 2^{H(W)} = 2^{-\frac{1}{N}\sum_{i=1}^N \log_2 P(w_i \mid w_{<i})} = \left(\prod_{i=1}^N \frac{1}{P(w_i \mid w_{<i})}\right)^{1/N}$$

---

## 2. The Vibe Coding Superpower

1. **LLM Hallucination Detection via Entropy Spikes**: Token generation with high entropy ($H > \tau$) indicates the model is guessing between alternatives. Gating agent actions when cumulative sequence entropy is high prevents hallucinated tool calls.
2. **Context Compression via Mutual Information**: Keep RAG chunks that maximize $I(\text{Chunk}; \text{Query})$, filtering redundant text where conditional entropy $H(\text{Chunk} \mid \text{Context}) \approx 0$.
3. **Semantic Entropy across Rollouts**: Sample $M=5$ completions at temperature $T=0.7$. Cluster completions by meaning. High cluster entropy signals epistemic uncertainty.

---

## 3. Production Code Implementations

### 3.1 Shannon Entropy & Hallucination Detector in Python
```python
import numpy as np

def compute_token_entropy(logits: np.ndarray, temperature: float = 1.0, eps: float = 1e-12) -> float:
    """
    Compute Shannon entropy of next-token distribution from unnormalized logits.
    High entropy signifies epistemic or aleatoric uncertainty.
    """
    scaled = logits / max(temperature, 1e-4)
    # Numerical stability via log-sum-exp trick
    max_logit = np.max(scaled)
    exp_logits = np.exp(scaled - max_logit)
    probs = exp_logits / np.maximum(np.sum(exp_logits), eps)

    # Shannon entropy H in bits
    nonzero_probs = probs[probs > eps]
    entropy = -np.sum(nonzero_probs * np.log2(nonzero_probs))
    return float(entropy)

def detect_hallucination_spike(
    token_entropies: list[float],
    threshold: float = 2.5,
    consecutive_spikes: int = 3
) -> bool:
    """
    Triggers hallucination alarm if entropy exceeds threshold across consecutive tokens.
    """
    spikes = 0
    for h in token_entropies:
        if h > threshold:
            spikes += 1
            if spikes >= consecutive_spikes:
                return True
        else:
            spikes = 0
    return False

def kl_divergence(p: np.ndarray, q: np.ndarray, eps: float = 1e-12) -> float:
    """
    Compute D_KL(P || Q) with division by zero and log zero guards.
    """
    p_safe = np.clip(p, eps, 1.0)
    q_safe = np.clip(q, eps, 1.0)
    p_safe /= np.sum(p_safe)
    q_safe /= np.sum(q_safe)
    return float(np.sum(p_safe * np.log(p_safe / q_safe)))
```

### 3.2 Streaming Entropy Monitor in Rust
```rust
pub struct EntropyMonitor {
    pub max_allowed_entropy: f32,
    pub window_size: usize,
    history: Vec<f32>,
}

impl EntropyMonitor {
    pub fn new(max_entropy: f32, window: usize) -> Self {
        Self {
            max_allowed_entropy: max_entropy,
            window_size: window,
            history: Vec::new(),
        }
    }

    pub fn push_probs(&mut self, probs: &[f32]) -> (f32, bool) {
        let mut entropy = 0.0f32;
        for &p in probs {
            if p > 1e-8 {
                entropy -= p * p.log2();
            }
        }

        self.history.push(entropy);
        if self.history.len() > self.window_size {
            self.history.remove(0);
        }

        let mean_entropy: f32 = self.history.iter().sum::<f32>() / self.history.len() as f32;
        let is_hallucinating = mean_entropy > self.max_allowed_entropy;
        (entropy, is_hallucinating)
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Uncertainty Awareness
```markdown
You are an uncertainty-calibrated agent:
- For every decisive conclusion, rate your epistemic confidence on a scale of [0.0 to 1.0].
- When multiple plausible hypotheses exist (high entropy), list the top alternatives explicitly rather than arbitrarily selecting one.
- If information is insufficient (conditional entropy H(Answer|Context) is high), emit a clarifying query instead of guessing.
```
