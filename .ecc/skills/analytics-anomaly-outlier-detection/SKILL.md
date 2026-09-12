---
name: analytics-anomaly-outlier-detection
description: Multi-dimensional anomaly detection: Isolation Forests, Local Outlier Factor (LOF), Mahalanobis distance, extreme value metrics, and anomaly scoring. Triggers: anomaly-outlier-detection, isolation-forest, local-outlier-factor, outlier-analysis, extreme-value-detection, lof-score, anomaly-scores.
triggers:
  - anomaly-outlier-detection
  - isolation-forest
  - local-outlier-factor
  - outlier-analysis
  - extreme-value-detection
  - lof-score
  - anomaly-scores
---

# Analytics Anomaly Outlier Detection
> Based on **Outlier Analysis - Charu C. Aggarwal**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Anomaly Detection Alert Incidents
CREATE TABLE anomaly_incidents (
    incident_id UUID PRIMARY KEY,
    entity_id VARCHAR(100) NOT NULL,
    detector_algorithm VARCHAR(50) NOT NULL,
    anomaly_score DOUBLE PRECISION NOT NULL CHECK (anomaly_score >= 0.0 AND anomaly_score <= 1.0),
    is_anomaly BOOLEAN NOT NULL DEFAULT FALSE,
    feature_contributions JSONB NOT NULL,
    detected_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Isolation Forest Anomaly Score Invariant
For sample size $n$ and average path length $E(h(x))$ across isolation trees:
$$c(n) = 2 \ln(n - 1) + 0.5772156649 - \frac{2(n - 1)}{n}$$
$$s(x, n) = 2^{-\frac{E(h(x))}{c(n)}}$$
- If $s \to 1.0$ (path length $E(h(x)) \to 0$): Definite anomaly (isolated rapidly).
- If $s < 0.5$: Normal observation.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    DataPoint[Incoming Multi-Dimensional Record] --> Forest[Pass through N Isolation Trees]
    Forest --> PathLength[Measure Tree Depth to Isolate Point]
    PathLength --> AvgDepth[Compute Mean Path Length E(h(x))]
    AvgDepth --> Score[Compute Anomaly Score s = 2^(-E(h)/c(n))]
    Score --> ThresholdCheck{Score > 0.60?}
    ThresholdCheck -->|Yes| Alert[Raise High-Priority Anomaly Incident]
    ThresholdCheck -->|No| Normal[Pass Record as Nominal]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from sklearn.ensemble import IsolationForest
import numpy as np

def detect_outliers_isolation_forest(X: np.ndarray, contamination: float = 0.01):
    clf = IsolationForest(contamination=contamination, random_state=42)
    preds = clf.fit_predict(X) # -1 for outlier, 1 for inlier
    scores = -clf.score_samples(X) # Higher score = more anomalous
    return {"outlier_mask": preds == -1, "scores": scores}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Isolation Forest isolates anomalies using random partitioning; outliers have short path lengths.
- Anomaly score s(x) > 0.6 indicates strong anomaly; s(x) < 0.5 indicates normal data.
- Local Outlier Factor (LOF) compares local density of an entity to its k-nearest neighbors.
- Use Mahalanobis distance for multivariate Gaussian data to account for inter-feature covariance.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Deploy real-time multi-variate anomaly detection microservices:
1. Train Isolation Forest and Local Outlier Factor models across high-dimensional feature spaces.
2. Decompose anomaly contributions to provide explainable root-cause attribution JSON payloads.
3. Build continuous streaming anomaly monitors alerting on operational metric drifts.
```
