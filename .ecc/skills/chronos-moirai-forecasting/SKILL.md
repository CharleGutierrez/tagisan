---
name: chronos-moirai-forecasting
description: Universal time series foundation models (Amazon Chronos, Salesforce MOIRAI), zero-shot probabilistic forecasting, patch-based multi-resolution architectures, and quantile Value-at-Risk (VaR / CVaR) estimation. Triggers: chronos, moirai, zero-shot time series, probabilistic forecasting, quantile prediction, value-at-risk, var, cvar, patch forecasting, universal time series.
triggers:
  - chronos
  - moirai
  - zero-shot time series
  - zero-shot time series forecasting with chronos
  - probabilistic forecasting
  - quantile prediction
  - patch forecasting
  - value at risk
  - var
  - cvar
---

# Chronos & MOIRAI Time Series Foundation Models Skill

## 1. Architectural Foundations

Foundation models for time series (Amazon Chronos and Salesforce MOIRAI) replace bespoke per-ticker statistical models with universal pre-trained architectures trained across billions of diverse observations.

### Architectural Distinctions:
1. **Amazon Chronos**: Tokenizes real-valued time series via non-parametric scaling and quantization into fixed vocabularies, enabling language model backbones (T5, GPT) to autoregressively forecast probability distributions.
2. **Salesforce MOIRAI**: Employs an Any-Variate, Multi-Patch Size Transformer architecture designed specifically for heterogeneous frequencies (seconds, minutes, days) with masked multi-patch attention.
3. **Zero-Shot Probabilistic Forecasting**: Outputs complete quantile trajectories $\hat{y}_t^{(q)}$ rather than point forecasts, enabling direct calculation of Value-at-Risk (VaR) and Expected Shortfall (CVaR).
4. **Scale Invariance & Patch Aggregation**: Automatic affine de-trending and instance normalization preserving relative percentage volatility across high-beta crypto and low-volatility sovereign debt.

---

## 2. Mathematical Rigor & Risk Formulations

### 2.1 Patching & Multi-Frequency Projection
Given an input univariate or multivariate series $\mathbf{Y} \in \mathbb{R}^{C \times L}$, patch projection chunks the sequence into non-overlapping or overlapping patches of length $P$:
$$\mathbf{p}_{c, i} = \mathbf{Y}_{c, (i-1)S : (i-1)S + P} \in \mathbb{R}^P$$
Projected into latent space:
$$\mathbf{h}_{c, i} = \mathbf{W}_{\text{patch}}^{(P)} \mathbf{p}_{c, i} + \mathbf{b} \in \mathbb{R}^D$$
In MOIRAI, multiple patch sizes $P \in \{8, 16, 32, 64, 128\}$ are dynamically routed to accommodate variable sampling frequencies.

### 2.2 Probabilistic Quantile Prediction & Pinball Loss
Predictions are generated for quantile levels $q \in \mathcal{Q} = \{0.01, 0.05, 0.10, 0.50, 0.90, 0.95, 0.99\}$.
Quantile loss (Pinball loss) for target $y$ and predicted quantile $\hat{y}^{(q)}$:
$$\mathcal{L}_q(y, \hat{y}^{(q)}) = \max\left(q(y - \hat{y}^{(q)}), (1-q)(\hat{y}^{(q)} - y)\right)$$

### 2.3 Value-at-Risk (VaR) and Expected Shortfall (CVaR)
For a portfolio value $V_0$ and confidence level $(1 - \alpha)$ (e.g. $\alpha = 0.05$ or $0.01$):
1. **Zero-Shot Quantile Value-at-Risk**:
   $$\text{VaR}_\alpha(t) = -\hat{y}_t^{(\alpha)}$$
   representing the maximum expected loss at probability $\alpha$.
2. **Conditional Value-at-Risk (Expected Shortfall)**:
   $$\text{CVaR}_\alpha(t) = -\frac{1}{\alpha} \int_0^\alpha \hat{y}_t^{(u)} du \approx -\frac{1}{K} \sum_{k=1}^K \hat{y}_t^{(u_k)}, \quad u_k < \alpha$$

---

## 3. PyTorch Reference Implementation

```python
import torch
import torch.nn as nn
from typing import List, Dict

class TimeSeriesPatchEmbedder(nn.Module):
    """
    Multi-patch linear projection layer for MOIRAI-style patch tokenization.
    """
    def __init__(self, patch_sizes: List[int], d_model: int):
        super().__init__()
        self.projections = nn.ModuleDict({
            str(p): nn.Linear(p, d_model) for p in patch_sizes
        })

    def forward(self, x: torch.Tensor, patch_size: int) -> torch.Tensor:
        # x: (B, Channels, Length)
        B, C, L = x.shape
        num_patches = L // patch_size
        x_trimmed = x[:, :, :num_patches * patch_size]
        # Reshape to (B * C, num_patches, patch_size)
        patches = x_trimmed.view(B * C, num_patches, patch_size)
        proj_layer = self.projections[str(patch_size)]
        tokens = proj_layer(patches)  # (B * C, num_patches, d_model)
        return tokens

class QuantileRiskLoss(nn.Module):
    """
    Pinball loss for calibrated zero-shot quantile forecasting and VaR estimation.
    """
    def __init__(self, quantiles: List[float] = [0.01, 0.05, 0.1, 0.5, 0.9, 0.95, 0.99]):
        super().__init__()
        self.register_buffer("quantiles", torch.tensor(quantiles))

    def forward(self, y_pred_quantiles: torch.Tensor, y_true: torch.Tensor) -> torch.Tensor:
        # y_pred_quantiles: (B, T, num_quantiles)
        # y_true: (B, T, 1)
        diff = y_true - y_pred_quantiles
        q = self.quantiles.view(1, 1, -1)
        loss = torch.max(q * diff, (q - 1.0) * diff)
        return loss.mean()
```

---

## 4. Quantitative Constraints & Risk Directives

1. **Reversible Instance Normalization (RevIN)**: Always apply affine normalizations with learned or rolling statistics $\mu_t, \sigma_t$ before tokenization, and de-normalize predictions at the output layer to maintain price-level realism.
2. **Quantile Monotonicity Invariant**: Enforce non-crossing quantile constraints:
   $$\hat{y}^{(q_1)} \le \hat{y}^{(q_2)} \quad \forall q_1 < q_2$$
   Use sorting or non-negative softmax incremental projections to prevent crossing quantiles.
3. **Fat-Tail & Extreme Value Stress Testing**: In addition to Gaussian-equivalent quantiles, evaluate against historical black-swan drawdowns (e.g., 2008 GFC, March 2020 COVID shock).
