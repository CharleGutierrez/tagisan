---
name: kronos-kline-modeling
description: Financial Candlestick (K-line) discrete tokenization using Binary Spherical Quantization (BSQ), autoregressive Transformer architectures, hierarchical dual decoding (s1 price tokens, s2 volume/residual tokens), and synthetic market generation. Triggers: kronos, k-line, candlestick, bsq, binary spherical quantization, dual decoding, market simulation, synthetic market generation, ohlcv sliding window.
triggers:
  - kronos
  - k-line
  - kline
  - candlestick
  - candlestick prediction with kronos bsq
  - bsq
  - binary spherical quantization
  - dual decoding
  - synthetic market generation
  - ohlcv normalization
---

# Kronos Financial Candlestick Modeling Skill

## 1. Architectural Foundations

Kronos represents financial market candlestick (K-line) intervals as discrete token sequences, treating multi-interval OHLCV patches analogously to discrete tokens in foundation models. By projecting continuous non-stationary market feeds into discrete spherical codebooks, Kronos prevents distribution collapse and non-stationarity drift.

### Core Architectural Pillars:
1. **Sliding-Window Invariant Normalization**: Remove secular price trends by normalizing OHLC values relative to the initial window price $P_{\text{ref}} = C_{t-W}$, and standardizing volume against a rolling Exponential Moving Average (EMA).
2. **Binary Spherical Quantization (BSQ)**: Map high-dimensional continuous latent candlestick embeddings onto a unit hypersphere $\mathbb{S}^{D-1}$, followed by sign binarization to produce compact bit-packed discrete codebook tokens.
3. **Hierarchical Dual Decoding**: Autoregressively decompose the market interval into:
   - **$s_1$ Tokens**: Macro price direction, trend velocity, and high-low spread.
   - **$s_2$ Tokens**: Microstructure volume, order flow intensity, and residual ticks conditioned on $s_1$.
4. **No-Arbitrage & Market Physics Conservation**: Reconstructed synthetic series strictly enforce candlestick invariants: $L_t \le \min(O_t, C_t) \le \max(O_t, C_t) \le H_t$ and $V_t \ge 0$.

---

## 2. Mathematical Rigor & Normalization

### 2.1 Sliding Window Normalization
For a historical context window of size $W$ with raw bars $\mathbf{X}_t = [O_t, H_t, L_t, C_t, V_t]$:
1. **Reference Price Anchor**:
   $$P_{\text{base}} = C_{t-W}$$
2. **Log-Relative Price Scaling**:
   $$o_\tau = \ln\left(\frac{O_\tau}{P_{\text{base}}}\right), \quad h_\tau = \ln\left(\frac{H_\tau}{P_{\text{base}}}\right), \quad l_\tau = \ln\left(\frac{L_\tau}{P_{\text{base}}}\right), \quad c_\tau = \ln\left(\frac{C_\tau}{P_{\text{base}}}\right)$$
   for all $\tau \in [t-W+1, t]$.
3. **Volume Standardization**:
   $$\tilde{V}_\tau = \ln\left(1 + \frac{V_\tau}{\text{EMA}(V, W)_\tau + \epsilon}\right)$$

### 2.2 Binary Spherical Quantization (BSQ)
Given an encoder latent embedding $\mathbf{z}_\tau \in \mathbb{R}^D$:
1. **Hypersphere Projection**:
   $$\tilde{\mathbf{z}}_\tau = \frac{\mathbf{z}_\tau}{\|\mathbf{z}_\tau\|_2 + \epsilon} \in \mathbb{S}^{D-1}$$
2. **Coordinate Sign Quantization**:
   $$q_b = \text{sign}(\tilde{z}_{\tau, b}) \in \{-1, +1\}, \quad b \in \{0, \dots, B-1\}$$
3. **Token ID Construction**:
   $$\text{Token ID } k_\tau = \sum_{b=0}^{B-1} 2^b \cdot \mathbb{I}(q_b > 0) \in [0, 2^B - 1]$$
4. **Straight-Through Estimator (STE)**:
   $$\mathbf{z}_q = \tilde{\mathbf{z}} + \text{sg}[q(\tilde{\mathbf{z}}) - \tilde{\mathbf{z}}]$$
   where $\text{sg}[\cdot]$ is the `stop_gradient` operator ensuring stable gradient descent.

### 2.3 Hierarchical Dual Decoding Likelihood
The joint probability of an autoregressive sequence over $T$ steps is:
$$P(\mathbf{S}_{1:T}) = \prod_{t=1}^T P(s_1^t \mid s_1^{<t}, s_2^{<t}) \cdot P(s_2^t \mid s_1^{\le t}, s_2^{<t})$$
This prevents volatile volume noise from perturbing directional price trends while allowing volume decoding to condition on the realized directional movement.

---

## 3. PyTorch Reference Implementation

```python
import torch
import torch.nn as nn
import torch.nn.functional as F

class BinarySphericalQuantizer(nn.Module):
    """
    Binary Spherical Quantization (BSQ) module with Straight-Through Estimator (STE).
    Projects continuous candlestick features onto S^{D-1} and binarizes into discrete tokens.
    """
    def __init__(self, in_dim: int, num_bits: int = 10):
        super().__init__()
        self.num_bits = num_bits
        self.proj = nn.Linear(in_dim, num_bits, bias=False)
        self.register_buffer("powers_of_two", 2 ** torch.arange(num_bits))

    def forward(self, z: torch.Tensor):
        h = self.proj(z)
        h_norm = F.normalize(h, p=2, dim=-1, eps=1e-8)
        b = torch.where(h_norm >= 0, torch.ones_like(h_norm), -torch.ones_like(h_norm))
        q = h_norm + (b - h_norm).detach()
        bits_binary = (b > 0).long()
        token_indices = torch.sum(bits_binary * self.powers_of_two, dim=-1)
        return q, token_indices, h_norm

class HierarchicalDualDecoderTransformer(nn.Module):
    """
    Autoregressive dual-stage transformer generating s1 (price) and s2 (volume) tokens.
    """
    def __init__(self, vocab_size: int = 1024, d_model: int = 256, nhead: int = 8, num_layers: int = 6):
        super().__init__()
        self.s1_embed = nn.Embedding(vocab_size, d_model)
        self.s2_embed = nn.Embedding(vocab_size, d_model)
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=d_model, nhead=nhead, dim_feedforward=d_model * 4,
            dropout=0.1, activation="gelu", batch_first=True
        )
        self.backbone = nn.TransformerEncoder(encoder_layer, num_layers=num_layers)
        self.s1_head = nn.Linear(d_model, vocab_size)
        self.s2_head = nn.Linear(d_model * 2, vocab_size)

    def forward(self, s1_seq: torch.Tensor, s2_seq: torch.Tensor):
        B, T = s1_seq.shape
        e1 = self.s1_embed(s1_seq)
        e2 = self.s2_embed(s2_seq)
        h = self.backbone(e1 + e2)
        logits_s1 = self.s1_head(h)
        h_cond = torch.cat([h, e1], dim=-1)
        logits_s2 = self.s2_head(h_cond)
        return logits_s1, logits_s2
```

---

## 4. Quantitative Safety & Risk Rules

1. **Strict Causal Masking**: Attention matrices must be lower triangular ($M_{ij} = -\infty$ for $j > i$). Zero look-ahead leakage.
2. **Physical Boundary Invariants**:
   - Reconstructed bars must verify: $\hat{H}_t \ge \max(\hat{O}_t, \hat{C}_t)$, $\hat{L}_t \le \min(\hat{O}_t, \hat{C}_t)$, $\hat{V}_t \ge 0$.
3. **Entropy & Perplexity Tracking**: Monitor codebook perplexity $P = \exp\left(-\sum_k p_k \ln p_k\right)$. If perplexity drops below $0.7 \times 2^B$, apply codebook perturbation.
4. **Realistic Market Execution**: Backtesting synthetic predictions must include non-linear slippage models: $\Delta P_{\text{slippage}} = \eta \cdot \text{Spread} + \gamma \left(\frac{\text{OrderSize}}{\text{Volume}}\right)^{0.5}$.
