---
name: fin-eng-machine-learning-alpha-pro-max
description: Master Financial Machine Learning, Deep Learning & AI Alpha Engine. Covers Marcos López de Prado's AFML methodology: Triple-Barrier Method, Meta-Labeling, Fractional Differentiation (stationarity with memory retention), Purged & Embargoed K-Fold Cross-Validation, Mean Decrease Accuracy (MDA) feature importance, and Financial Transformers/Deep RL. Based on López de Prado, Jansen, Dixon-Halperin-Bilokon, Sutton-Barto, and Books 251–300. Triggers: financial-machine-learning, machine-learning-alpha, lopez-de-prado, afml, purged-cross-validation, triple-barrier-method, meta-labeling, fractional-differentiation, feature-importance-mda, financial-reinforcement-learning, transformer-alpha.
version: 1.0.0
tags:
  - financial-machine-learning
  - afml
  - lopez-de-prado
  - purged-cv
  - triple-barrier
  - fractional-diff
  - meta-labeling
triggers:
  - financial-machine-learning
  - machine-learning-alpha
  - lopez-de-prado
  - afml
  - purged-cross-validation
  - triple-barrier-method
  - meta-labeling
  - fractional-differentiation
  - feature-importance-mda
  - financial-reinforcement-learning
  - transformer-alpha
compatibility: ">=0.2.0"
---

# Financial Machine Learning & AI Alpha Pro Max: The Sovereign 50-Book Engine

## Purpose & Scope
Standard machine learning techniques fail catastrophically in finance due to non-IID observations, low signal-to-noise ratios, non-stationarity, and cross-validation leakage.

The `fin-eng-machine-learning-alpha-pro-max` engine codifies the core theory and algorithmic methodologies from **Books 251–300** of the Financial Engineering Canon:
- *Marcos López de Prado* (Advances in Financial Machine Learning & Machine Learning for Asset Managers)
- *Stefan Jansen* (Machine Learning for Algorithmic Trading)
- *Matthew F. Dixon, Igor Halperin, & Paul Bilokon* (Machine Learning in Finance: From Theory to Practice)
- *Richard S. Sutton & Andrew G. Barto* (Reinforcement Learning: An Introduction)
- *Guillaume Coqueret & Tony Guida* (Financial Machine Learning: A Practice-Oriented Guide)
- *Aleksander Molak* (Causal Inference and Discovery in Python)

---

## 1. Core Operational Invariants

### Invariant 1: Purged & Embargoed Cross-Validation
- Standard K-Fold CV creates massive leakage because financial labels span forward in time $[t_i, t_{i+h}]$.
- **Purging**: Eliminate all training samples whose label evaluation window overlaps with test sample windows.
- **Embargoing**: Eliminate training samples immediately following the test set for duration $h_{\text{embargo}}$ to remove autoregressive correlation leakage.

### Invariant 2: Fractional Differentiation for Memory Preservation
- Integer differencing ($d=1$) removes non-stationarity but erases all long-term memory (mean-reversion and macro trends).
- Fractional differentiation $(1 - B)^d = \sum_{k=0}^\infty \omega_k B^k$ with $d \in (0, 1)$ achieves stationarity (passes ADF test) while maximizing the correlation with the original series.

### Invariant 3: Triple Barrier Labeling
- Each trade observation must be labeled deterministically by whichever barrier is touched first:
  1. Upper Barrier: Profit-taking limit ($+L \sigma$).
  2. Lower Barrier: Stop-loss limit ($-L \sigma$).
  3. Vertical Barrier: Holding period timeout ($t + \Delta t$).

---

## 2. Battle-Tested Production Blueprints

### Blueprint 1: Purged & Embargoed Cross-Validation Splitter & Fractional Differencing
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CvSplit {
    pub train_indices: Vec<usize>,
    pub test_indices: Vec<usize>,
}

/// Purged & Embargoed K-Fold Cross Validation Splitter
pub struct PurgedKFold {
    pub n_samples: usize,
    pub n_splits: usize,
    pub label_end_indices: Vec<usize>, // For each sample i, index when label horizon ends
    pub embargo_samples: usize,
}

impl PurgedKFold {
    pub fn new(n_samples: usize, n_splits: usize, label_end_indices: Vec<usize>, embargo_pct: f64) -> Self {
        assert!(n_samples >= n_splits && n_splits > 1);
        let embargo_samples = ((n_samples as f64) * embargo_pct).ceil() as usize;
        Self {
            n_samples,
            n_splits,
            label_end_indices,
            embargo_samples,
        }
    }

    pub fn split(&self) -> Vec<CvSplit> {
        let mut splits = Vec::with_capacity(self.n_splits);
        let fold_size = self.n_samples / self.n_splits;

        for k in 0..self.n_splits {
            let test_start = k * fold_size;
            let test_end = if k == self.n_splits - 1 { self.n_samples } else { (k + 1) * fold_size };

            let test_indices: Vec<usize> = (test_start..test_end).collect();
            let mut train_indices: Vec<usize> = Vec::with_capacity(self.n_samples - test_indices.len());

            // Max evaluation horizon of the test fold
            let max_test_end = test_indices.iter().map(|&idx| self.label_end_indices[idx]).max().unwrap_or(test_end);
            let embargo_limit = (max_test_end + self.embargo_samples).min(self.n_samples);

            for i in 0..self.n_samples {
                if i >= test_start && i < test_end {
                    continue; // Inside test set
                }

                // Purge: Train observation label ends inside or after test start, but started before test end
                let train_label_end = self.label_end_indices[i];
                let overlaps_test = !(train_label_end < test_start || i >= test_end);

                // Embargo: Train observation starts within embargo period following test set
                let in_embargo = i >= test_end && i < embargo_limit;

                if !overlaps_test && !in_embargo {
                    train_indices.push(i);
                }
            }

            splits.push(CvSplit { train_indices, test_indices });
        }

        splits
    }
}

/// Fractional Differentiation Weights Generator
pub fn compute_frac_diff_weights(d: f64, size: usize, threshold: f64) -> Vec<f64> {
    let mut weights = Vec::with_capacity(size);
    weights.push(1.0);

    for k in 1..size {
        let w_prev = weights[k - 1];
        let w_k = -w_prev / (k as f64) * (d - (k as f64) + 1.0);
        if w_k.abs() < threshold {
            break;
        }
        weights.push(w_k);
    }

    weights
}

/// Apply Fractional Differencing to a series
pub fn apply_frac_diff(series: &[f64], d: f64, threshold: f64) -> Vec<f64> {
    let weights = compute_frac_diff_weights(d, series.len(), threshold);
    let k_len = weights.len();
    let mut result = Vec::with_capacity(series.len());

    for i in 0..series.len() {
        if i < k_len - 1 {
            result.push(f64::NAN); // Insufficient history for convolution
        } else {
            let mut val = 0.0;
            for (j, &w) in weights.iter().enumerate() {
                val += w * series[i - j];
            }
            result.push(val);
        }
    }

    result
}
```

---

## 3. Frontier LLM Vibe Coding Prompt Engineering Guide

### Canonical Prompt Template: AFML Meta-Labeling Model
```markdown
You are a machine learning quant engineer implementing Marcos López de Prado's Meta-Labeling paradigm.
- Primary Model: Fast rule-based statistical arbitrage indicator (e.g. Bollinger Bands / Kalman filter) predicting trade direction {-1, 1}.
- Secondary (Meta) Model: LightGBM / XGBoost classifier predicting whether the primary model's trade touches profit-taking before stop-loss {0, 1}.
- Data Pipeline: Apply fractional differencing d=0.45 to features to preserve long-term memory.
- Cross-Validation: Train using PurgedKFold with 5 splits and 1% embargo. Ensure strictly zero leakage.
```
