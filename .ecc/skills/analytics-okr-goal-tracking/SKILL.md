---
name: analytics-okr-goal-tracking
description: Enterprise goal alignment: Objectives and Key Results (OKRs), committed vs aspirational goals, CFRs (Conversations, Feedback, Recognition), and mathematical scoring. Triggers: okr-goal-tracking, measure-what-matters, okrs, key-results, aspirational-vs-committed, cfr-alignment, goal-scoring.
triggers:
  - okr-goal-tracking
  - measure-what-matters
  - okrs
  - key-results
  - aspirational-vs-committed
  - cfr-alignment
  - goal-scoring
---

# Analytics Okr Goal Tracking
> Based on **Measure What Matters - John Doerr**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Enterprise OKR Tracking Table
CREATE TABLE okr_key_results (
    kr_id VARCHAR(50) PRIMARY KEY,
    objective_id VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    target_value DOUBLE PRECISION NOT NULL,
    current_value DOUBLE PRECISION NOT NULL,
    is_aspirational BOOLEAN NOT NULL DEFAULT FALSE,
    score DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (score >= 0.0 AND score <= 1.0)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Key Result Progress & Scoring Invariant
For Key Result with baseline $B$, target $T$, and current measurement $C$:
$$\text{Score} = \text{clamp}\left( \frac{C - B}{T - B}, 0.0, 1.0 \right)$$
For aspirational (moonshot) OKRs:
$$\text{Optimal Performance Sweet Spot} \in [0.6, 0.7]$$
A consistent score of 1.0 indicates under-ambitious goal setting.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Obj[Objective: Qualitative Inspiring Goal] --> KR1[Key Result 1: Quantitative Metric]
    Obj --> KR2[Key Result 2: Quantitative Metric]
    Obj --> KR3[Key Result 3: Quantitative Metric]
    KR1 --> ScoreKR[Score = (Current - Base) / (Target - Base)]
    ScoreKR --> AvgScore[Aggregate Objective Score: Sweet spot 0.7]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust OKR Scorer
pub fn score_key_result(baseline: f64, target: f64, current: f64) -> f64 {
    if (target - baseline).abs() < 1e-6 {
        return 1.0;
    }
    let raw = (current - baseline) / (target - baseline);
    raw.clamp(0.0, 1.0)
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Objectives are qualitative, inspiring, and time-bound; Key Results are strictly quantitative.
- Every Key Result must have a number, baseline, and target date: 'measure what matters'.
- Score KRs from 0.0 to 1.0: a score of 0.6 - 0.7 is the sweet spot for aspirational goals.
- Separate OKR performance evaluation from salary/compensation reviews to encourage risk-taking.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build automated OKR metric tracking architectures:
1. Connect enterprise OKR platforms directly to data warehouse analytical tables for real-time progress.
2. Implement mathematical scoring algorithms distinguishing committed (1.0 target) vs aspirational goals.
3. Build cascading alignment graphs tracing team key results directly to corporate objectives.
```
