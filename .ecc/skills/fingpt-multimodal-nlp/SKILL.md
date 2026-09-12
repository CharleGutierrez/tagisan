---
name: fingpt-multimodal-nlp
description: Multimodal financial NLP (FinGPT) and deep reinforcement learning (FinRL) for SEC 10-K/10-Q filing analysis, sentiment alpha factor extraction, and optimal trade execution (PPO/DDPG). Triggers: fingpt, finrl, financial nlp, sec filing sentiment, 10-k analysis, 10-q filings, sentiment factors, reinforcement learning execution, almgren-chriss, ppo trade execution.
triggers:
  - fingpt
  - finrl
  - financial nlp
  - sec filing sentiment with fingpt
  - 10-k filing analysis
  - 10-q filing analysis
  - sentiment factors
  - reinforcement learning execution
  - ppo trade execution
  - almgren chriss
---

# FinGPT & FinRL Multimodal Financial NLP Skill

## 1. Architectural Foundations

FinGPT and FinRL bridge natural language unstructured financial disclosures (SEC 10-K, 10-Q, 8-K filings, earnings call transcripts, real-time Bloomberg/Reuters news) with continuous-time quantitative decision making and deep reinforcement learning execution policies.

### Architectural Pillars:
1. **Domain-Adapted Low-Rank Financial LLMs (FinGPT)**: Parameter-Efficient Fine-Tuning (LoRA / QLoRA) on curated financial corpora (EDGAR SEC filings, financial news, Social sentiment) overcoming generic LLM hallucinations.
2. **Hierarchical Item Parsing**: Systematic extraction and longitudinal comparison of regulatory sections:
   - **Item 1A**: Risk Factors (identification of novel emerging operational/legal liabilities).
   - **Item 7**: Management's Discussion & Analysis (MD&A) of Financial Condition.
   - **Item 7A**: Quantitative and Qualitative Disclosures About Market Risk.
3. **Sentiment Alpha Factor Construction**: Formulate differential sentiment indices $\Delta S_t$ with exponential half-life decay, orthogonalized against market sentiment beta.
4. **Deep RL Execution Optimization (FinRL)**: Formulate algorithmic execution (liquidation, TWAP/VWAP optimization) as a Markov Decision Process (MDP) with Almgren-Chriss market impact penalties.

---

## 2. Mathematical Formulations & Execution MDP

### 2.1 SEC Filing Sentiment & Divergence Metrics
Let $\mathcal{D}_t$ be the parsed text tokens of a firm's 10-K filing at reporting period $t$:
1. **Loughran-McDonald Financial Sentiment Polarity**:
   $$\text{Score}_t = \frac{N_{\text{pos}} - N_{\text{neg}}}{N_{\text{pos}} + N_{\text{neg}} + \epsilon} \in [-1, 1]$$
2. **FinGPT Dense Sentiment Logits**:
   $$\mathbf{p}_t = \text{Softmax}\left(\mathbf{W}_{\text{cls}} \mathbf{h}_{\text{[CLS]}}\right)$$
   $$\text{Sentiment Factor } S_t = \mathbf{p}_t(\text{Bullish}) - \mathbf{p}_t(\text{Bearish})$$
3. **Filing-over-Filing Semantic Drift (Cosine Divergence)**:
   $$\Delta_{\text{Risk}} = 1 - \frac{\mathbf{e}_{t}^{\text{Item1A}} \cdot \mathbf{e}_{t-1}^{\text{Item1A}}}{\|\mathbf{e}_{t}^{\text{Item1A}}\| \|\mathbf{e}_{t-1}^{\text{Item1A}}\|}$$
   Spikes in $\Delta_{\text{Risk}}$ historically precede negative earnings surprises and credit rating downgrades.

### 2.2 FinRL Execution MDP & Almgren-Chriss Reward
Executing total quantity $Q$ over discrete intervals $k = 0, \dots, N$:
1. **State Space $\mathbf{s}_k$**:
   $$\mathbf{s}_k = \left(r_k, \text{Spread}_k, \text{Imbalance}_k, \frac{q_k}{Q}, \frac{k}{N}\right)$$
   where $q_k$ is the remaining unsold inventory.
2. **Action Space $a_k \in [0, 1]$**: Fraction of remaining shares to execute in interval $k$: $\Delta q_k = a_k \cdot q_k$.
3. **Almgren-Chriss Reward Function**:
   $$R_k = \left(P_k - P_{\text{benchmark}}\right) \Delta q_k - \eta (\Delta q_k)^2 - \lambda \sigma^2 q_k^2 \tau$$
   where $\eta$ governs temporary market impact, and $\lambda$ penalizes holding inventory variance risk.

---

## 3. Python Reference Implementation

```python
import torch
import torch.nn as nn
from transformers import AutoTokenizer, AutoModelForSequenceClassification

class FinGPTSentimentAnalyzer:
    """
    Financial sentiment scoring using domain-adapted LoRA representations.
    """
    def __init__(self, model_name: str = "FinGPT/fingpt-sentiment_llama2-13b_lora"):
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModelForSequenceClassification.from_pretrained(model_name).to(self.device)

    def score_mda_section(self, text_chunks: list[str]) -> float:
        """
        Score parsed MD&A text chunks and compute mean sentiment polarity.
        """
        scores = []
        for chunk in text_chunks:
            inputs = self.tokenizer(chunk, return_tensors="pt", truncation=True, max_length=512).to(self.device)
            with torch.no_grad():
                logits = self.model(**inputs).logits
                probs = torch.softmax(logits, dim=-1)
                # Label indices: 0: Bearish, 1: Neutral, 2: Bullish
                polarity = (probs[0, 2] - probs[0, 0]).item()
                scores.append(polarity)
        return float(sum(scores) / max(len(scores), 1))

class ExecutionEnvState:
    """
    FinRL State representation for optimal order execution.
    """
    def __init__(self, total_shares: float, num_steps: int):
        self.total_shares = total_shares
        self.remaining_shares = total_shares
        self.num_steps = num_steps
        self.current_step = 0

    def step(self, action_fraction: float, current_price: float, volatility: float, eta: float = 1e-4, lam: float = 1e-5):
        shares_to_sell = min(action_fraction * self.remaining_shares, self.remaining_shares)
        self.remaining_shares -= shares_to_sell
        self.current_step += 1
        
        # Almgren-Chriss reward calculation
        market_impact = eta * (shares_to_sell ** 2)
        risk_penalty = lam * (volatility ** 2) * (self.remaining_shares ** 2)
        reward = (current_price * shares_to_sell) - market_impact - risk_penalty
        done = (self.current_step >= self.num_steps) or (self.remaining_shares <= 0)
        return reward, done
```

---

## 4. Quantitative Constraints & Governance

1. **Filing Filing-Date Alignment**: Always index 10-K/10-Q by `acceptance_datetime` (EDGAR timestamp), NEVER the fiscal period end date (e.g. `2025-12-31`), preventing 60-day look-ahead leakage.
2. **Vocabulary Overfitting Prevention**: Do not rely exclusively on general sentiment lexicons (e.g. VADER); always employ specialized financial taxonomies (Loughran-McDonald).
3. **Execution Liquidity Boundaries**: Cap instantaneous execution action $a_k \cdot q_k \le 0.05 \times \text{Volume}_k$ to avoid predatory high-frequency counterparty front-running.
