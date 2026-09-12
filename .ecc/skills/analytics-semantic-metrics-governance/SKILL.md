---
name: analytics-semantic-metrics-governance
description: Centralized metric layers and Headless BI: Cube, dbt Semantic Layer, MetricFlow, Single Source of Truth, and preventing metric drift across tools. Triggers: semantic-metrics-governance, winning-with-data, metric-layer, headless-bi, single-source-of-truth, cube-semantic-layer, metricflow.
triggers:
  - semantic-metrics-governance
  - winning-with-data
  - metric-layer
  - headless-bi
  - single-source-of-truth
  - cube-semantic-layer
  - metricflow
---

# Analytics Semantic Metrics Governance
> Based on **Winning with Data - Tomasz Tunguz & Frank Bien**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Centralized Metric Definition Repository
CREATE TABLE semantic_metric_definitions (
    metric_name VARCHAR(100) PRIMARY KEY,
    metric_type VARCHAR(30) NOT NULL CHECK (metric_type IN ('SIMPLE', 'RATIO', 'CUMULATIVE', 'DERIVED')),
    sql_formula TEXT NOT NULL,
    underlying_table VARCHAR(100) NOT NULL,
    owner_team VARCHAR(50) NOT NULL,
    version INT NOT NULL DEFAULT 1,
    is_certified BOOLEAN NOT NULL DEFAULT TRUE
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Metric Expression Uniqueness Invariant
For any metric identifier $M$:
$$|\text{Definitions}(M)| \equiv 1$$
Every BI tool, API endpoint, and dashboard must resolve metric $M$ by querying the semantic layer API:
$$\text{Value}(M) \equiv \text{Eval}(\text{Formula}(M), \text{Dataset})$$
Eliminates contradictory revenue figures between Finance and Sales.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    DB[Data Warehouse Tables] --> Semantic[Semantic Layer: Cube / MetricFlow]
    Semantic --> Formula[Define Revenue = SUM(net_amount)]
    Formula --> BI[Tableau / Looker: Queries Semantic API]
    Formula --> App[Internal Portal: Queries Semantic API]
    Formula --> AI[LLM Agents: Queries Semantic API]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# MetricFlow / dbt Semantic Layer Spec
# metric:
#   name: monthly_recurring_revenue
#   type: simple
#   type_params:
#     measure: subscription_amount
#   filter: |
#     subscription_status = 'ACTIVE' 
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Define metrics once in the semantic layer; never hardcode aggregation SQL in BI dashboards.
- A semantic layer prevents 'metric drift' where different departments report conflicting numbers.
- Differentiate metric types: simple sums, ratios, cumulative lifetime metrics, derived formulas.
- Treat metrics as code: store definitions in Git, test changes, and automate deployment.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design and deploy enterprise headless BI semantic layers:
1. Implement centralized metric stores (Cube / dbt MetricFlow) exposing governed GraphQL/SQL APIs.
2. Eliminate redundant ad-hoc logic by defining business dimensions and measures as version-controlled code.
3. Integrate AI agent querying directly with semantic layer APIs for hallucination-free analytics.
```
