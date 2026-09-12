---
name: qlib-alpha-engineering
description: Microsoft Qlib AI-driven quantitative investment pipeline, alpha factor engineering, Barra risk factor neutralization (market beta, size, industry, volatility), turnover cost/slippage modeling, and Top-K backtesting with RankIC evaluation. Triggers: qlib, alpha factors, barra risk neutralization, top-k backtesting, rankic, information coefficient, icir, portfolio turnover, slippage modeling.
triggers:
  - qlib
  - alpha factors
  - barra factor neutralization in qlib
  - barra risk neutralization
  - top-k backtesting
  - rankic
  - information coefficient
  - icir
  - turnover modeling
  - factor neutralization
---

# Microsoft Qlib Alpha Engineering Skill

## 1. Architectural Foundations

Microsoft Qlib provides an end-to-end AI-oriented quantitative investment framework spanning data preprocessing, automated feature derivation, model training (LightGBM, DoubleEnsemble, Trajectory Transformers), Barra risk neutralization, and order execution backtesting.

### Core Quant Tenets:
1. **Point-in-Time Data Integrity**: Strict elimination of look-ahead bias and survivorship bias across cross-sectional universes.
2. **Barra Multi-Factor Risk Neutralization**: Orthogonalize raw alpha factor predictions against systematic risk exposures (Market Beta, Log Market Cap Size, Industry Dummies, Volatility, Liquidity) to extract genuine idiosyncratic alpha.
3. **Turnover & Cost Realism**: Model explicit turnover drag and non-linear market execution slippage: every factor must demonstrate net-of-fee profitability under realistic commission, stamp tax, and slippage constraints.
4. **Rank Information Coefficient (RankIC & ICIR)**: Evaluate alpha quality using non-parametric rank correlation across cross-sectional time steps.

---

## 2. Mathematical Formulations

### 2.1 Barra Risk Factor Neutralization
Given a cross-sectional raw alpha prediction vector $\mathbf{f}_t \in \mathbb{R}^N$ for $N$ instruments at date $t$, and a risk factor matrix $\mathbf{X}_t \in \mathbb{R}^{N \times K}$ containing $K$ risk drivers (e.g., Market Beta, Size, 30 Industry Dummy columns, Volatility):
1. **Cross-Sectional Ordinary Least Squares (OLS)**:
   $$\mathbf{f}_t = \mathbf{X}_t \boldsymbol{\beta}_t + \boldsymbol{\epsilon}_t$$
2. **Projection / Annihilator Matrix**:
   $$\mathbf{M}_t = \mathbf{I}_N - \mathbf{X}_t \left(\mathbf{X}_t^T \mathbf{X}_t + \lambda \mathbf{I}_K\right)^{-1} \mathbf{X}_t^T$$
   where $\lambda \ge 0$ provides Ridge regularization when industry indicator matrices are near-singular.
3. **Neutralized Alpha Vector**:
   $$\tilde{\mathbf{f}}_t = \mathbf{M}_t \mathbf{f}_t = \boldsymbol{\epsilon}_t$$
   Properties: $\mathbf{X}_t^T \tilde{\mathbf{f}}_t = \mathbf{0}$, ensuring zero collinearity with systematic risk drivers.

### 2.2 Alpha Evaluation Metrics (RankIC, ICIR, Sharpe)
1. **Cross-Sectional RankIC**:
   $$\text{RankIC}_t = \text{Corr}\left(\text{rank}(\tilde{\mathbf{f}}_t), \text{rank}(\mathbf{r}_{t+1})\right)$$
   where $\mathbf{r}_{t+1}$ represents the forward rebalancing interval returns.
2. **Information Ratio of IC (ICIR)**:
   $$\text{ICIR} = \frac{\mathbb{E}[\text{RankIC}_t]}{\sigma(\text{RankIC}_t)} \times \sqrt{252}$$
   Healthy alphas require $\text{Mean}(\text{RankIC}) > 0.05$ and $\text{ICIR} > 0.8$.
3. **Turnover & Cost Drag**:
   $$\text{Turnover}_t = \frac{1}{2} \sum_{i=1}^N \left|w_{i, t} - w_{i, t^-}\right|$$
   $$\text{Cost}_t = \text{Turnover}_t \cdot \left(c_{\text{fee}} + c_{\text{tax}} + \text{Slippage}(V_t)\right)$$

---

## 3. Python & Qlib Implementation

```python
import numpy as np
import pandas as pd
from typing import Tuple

def barra_factor_neutralize(
    raw_alpha: np.ndarray, 
    risk_factors: np.ndarray, 
    ridge_lambda: float = 1e-6
) -> np.ndarray:
    """
    Orthogonalize raw alpha factor predictions against Barra risk exposures.
    
    Parameters:
        raw_alpha: (N,) array of alpha scores for N stocks.
        risk_factors: (N, K) matrix of risk exposures (size, beta, industry dummies).
        ridge_lambda: L2 regularization parameter for numerical stability.
    Returns:
        neutralized_alpha: (N,) orthogonalized idiosyncratic alpha vector.
    """
    N, K = risk_factors.shape
    # Add intercept column if not present
    if not np.allclose(risk_factors[:, 0], 1.0):
        X = np.hstack([np.ones((N, 1)), risk_factors])
        K += 1
    else:
        X = risk_factors

    # Compute regularized projection: (X^T X + lambda I)^(-1) X^T
    XtX = X.T @ X + ridge_lambda * np.eye(K)
    beta = np.linalg.solve(XtX, X.T @ raw_alpha)
    
    # Residual idiosyncratic alpha
    neutralized = raw_alpha - X @ beta
    
    # Cross-sectional z-score standardization
    z_neutralized = (neutralized - np.nanmean(neutralized)) / (np.nanstd(neutralized) + 1e-8)
    return z_neutralized

def compute_rank_ic(factor_series: pd.Series, forward_returns: pd.Series) -> float:
    """
    Calculate Spearman Rank Correlation between alpha factor and forward returns.
    """
    df = pd.DataFrame({"alpha": factor_series, "target": forward_returns}).dropna()
    if len(df) < 5:
        return 0.0
    rank_alpha = df["alpha"].rank()
    rank_target = df["target"].rank()
    return float(np.corrcoef(rank_alpha, rank_target)[0, 1])
```

---

## 4. Quantitative Safety & Backtest Rules

1. **Zero Look-Ahead Bias Enforcement**:
   - Alpha factors computed at bar $t$ must only use information available strictly at or before $t$.
   - Trades rebalance at $t+1$ Open or TWAP; never rebalance at bar $t$ Close price without explicit half-spread slippage.
2. **Strict Transaction Cost & Slippage Accounting**:
   - Zero-fee backtests are strictly prohibited in production pipelines. Minimum fee model: 5 bps commission + 10 bps stamp duty + 5 bps bid-ask half-spread.
3. **Industry Dummy Rank Defect**: Always condition industry indicator matrices using pseudo-inverse or Ridge regularization to prevent singular matrix inversion crashes during stock suspension periods.
4. **Capacity & Liquidity Constraints**: Exclude bottom 20% liquid names ($V_{20\text{d}} < \text{Threshold}$) or enforce max order participation $< 2.5\%$ of Average Daily Volume (ADV).
