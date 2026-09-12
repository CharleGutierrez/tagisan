---
name: analytics-d3-interactive-graphics
description: Interactive web visualization: D3.js data binding pattern (enter, update, exit), mathematical scales, SVG/Canvas rendering, transitions, and force layouts. Triggers: d3-interactive-graphics, d3js, enter-update-exit, d3-scales, svg-visualization, interactive-dashboards, data-joins.
triggers:
  - d3-interactive-graphics
  - d3js
  - enter-update-exit
  - d3-scales
  - svg-visualization
  - interactive-dashboards
  - data-joins
---

# Analytics D3 Interactive Graphics
> Based on **Interactive Data Visualization for the Web - Scott Murray**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Dynamic Dashboard Widget Layout Registry
CREATE TABLE d3_widget_configs (
    widget_id VARCHAR(50) PRIMARY KEY,
    chart_type VARCHAR(50) NOT NULL,
    width INT NOT NULL,
    height INT NOT NULL,
    margin_json JSONB NOT NULL,
    animation_duration_ms INT NOT NULL DEFAULT 750
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 D3 Linear Scale Mapping Invariant
For domain $[x_{\text{min}}, x_{\text{max}}]$ and screen range $[y_{\text{min}}, y_{\text{max}}]$:
$$f(x) = y_{\text{min}} + \frac{x - x_{\text{min}}}{x_{\text{max}} - x_{\text{min}}} (y_{\text{max}} - y_{\text{min}})$$
Scale function preserves order and linearity:
$$f(x_1) < f(x_2) \iff x_1 < x_2 \quad (\text{for positive range gradient})$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[New Data Array] --> Join[selection.data(data, d => d.id)]
    Join --> Enter[enter(): Create new DOM nodes for new items]
    Join --> Update[update: Transition existing DOM elements to new positions]
    Join --> Exit[exit(): Remove DOM elements with no matching data]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// D3.js General Update Pattern
function updateBars(svg, data, xScale, yScale, height) {
    const bars = svg.selectAll("rect.bar")
        .data(data, d => d.id);

    // Enter
    bars.enter()
        .append("rect")
        .attr("class", "bar")
        .attr("x", d => xScale(d.key))
        .attr("y", height)
        .attr("width", xScale.bandwidth())
        .attr("height", 0)
        .transition().duration(750)
        .attr("y", d => yScale(d.value))
        .attr("height", d => height - yScale(d.value));

    // Update
    bars.transition().duration(750)
        .attr("x", d => xScale(d.key))
        .attr("y", d => yScale(d.value))
        .attr("height", d => height - yScale(d.value));

    // Exit
    bars.exit()
        .transition().duration(750)
        .attr("height", 0)
        .attr("y", height)
        .remove();
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- The D3 data join matches DOM elements to data items using key functions (d => d.id).
- Master the Enter-Update-Exit pattern: enter() appends, update transitions, exit() cleans up DOM.
- Use d3.scaleLinear and d3.scaleBand to map mathematical domains to SVG pixel coordinates.
- Separate SVG margins pattern: wrapper <g> offset by margin.left and margin.top.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build bespoke interactive data visualization applications:
1. Implement responsive SVG/Canvas visual graphics leveraging D3.js general update patterns.
2. Build interactive brush, pan, zoom, and force-directed graph physics simulations.
3. Optimize DOM rendering performance using virtual canvases for datasets exceeding 50,000 points.
```
