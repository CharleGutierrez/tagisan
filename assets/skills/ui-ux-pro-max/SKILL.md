---
name: ui-ux-pro-max
description: Autonomous UI/UX Design System Intelligence, WCAG 2.2 Accessibility Auditing, 8-Point Spatial Grid Enforcement, Multi-Brand Token Synthesis, and 500-Rule HCI Heuristics Engine. Triggers: ui, ux, design system, wcag, contrast ratio, accessibility, a11y, spatial grid, tokens, tailwind, color palette, cls, forms, micro-interactions, genai ui, heuristics.
version: 1.0.0
tags:
  - ui
  - ux
  - design-systems
  - wcag-2-2
  - accessibility
  - tokens
  - tailwind
  - spatial-grid
  - heuristics
  - dark-mode
compatibility: ">=0.2.0"
---

# UI/UX Pro Max: Production Design Intelligence & Accessibility Engine

## Purpose & Scope
Modern user interfaces frequently suffer from subtle accessibility regressions, visual disharmony, layout shifts (CLS), inconsistent spacing, and broken keyboard navigation. 

The `ui-ux-pro-max` skill equips Tagisan with an industrial-grade UI/UX design intelligence engine, static source code analyzer, multi-brand token synthesizer, and pre-delivery release gating system. It codifies 500 HCI rules across 12 disciplines, implements exact IEEE/IEC 61966-2-1 and WCAG 2.2 color science, and provides CLI automation via `tgs ux`.

---

## 1. Exact Color Science & WCAG 2.2 Mathematics

### 1.1 sRGB Channel Linearization
Human ocular perception of luminance is non-linear (gamma curve ~2.2). Under IEC 61966-2-1 and WCAG 2.2, sRGB color channels $C \in [0.0, 1.0]$ are linearized:

$$C_{\text{linear}} = \begin{cases} \frac{C}{12.92} & \text{if } C \le 0.04045 \\ \left(\frac{C + 0.055}{1.055}\right)^{2.4} & \text{if } C > 0.04045 \end{cases}$$

### 1.2 Relative Luminance ($L$)
Calculated from the linearized channels using the standard Rec. 709 spectral weighting coefficients:
$$L = 0.2126 \times R_{\text{linear}} + 0.7152 \times G_{\text{linear}} + 0.0722 \times B_{\text{linear}}$$
- Pure white (`#ffffff`): $L = 1.0$
- Pure black (`#000000`): $L = 0.0$

### 1.3 WCAG 2.2 Contrast Ratio
$$\text{Contrast Ratio} = \frac{L_1 + 0.05}{L_2 + 0.05}$$
where $L_1 = \max(L_{\text{fg}}, L_{\text{bg}})$ and $L_2 = \min(L_{\text{fg}}, L_{\text{bg}})$. Range is $[1.0, 21.0]$.

### 1.4 WCAG 2.2 Compliance Thresholds
- **Normal Text** (< 18pt or < 14pt bold):
  - **Level AA (SC 1.4.3)**: Ratio $\ge 4.5:1$
  - **Level AAA (SC 1.4.6)**: Ratio $\ge 7.0:1$
- **Large Text** ($\ge 18\text{pt}$ or $\ge 14\text{pt}$ bold):
  - **Level AA**: Ratio $\ge 3.0:1$
  - **Level AAA**: Ratio $\ge 4.5:1$
- **UI Components & Graphical Objects** (SC 1.4.11):
  - **Level AA**: Ratio $\ge 3.0:1$
- **Automated Remediation**: When a contrast ratio fails Level AA, the engine adjusts HSL lightness via binary search to synthesize the minimal perceptually compliant color while locking hue and saturation.

---

## 2. Static UI AST & Pattern Linter Rules

The static linter inspects `.tsx`, `.jsx`, `.html`, `.vue`, `.svelte`, `.css`, and `.rs` UI files:

| Rule ID | Severity | Focus Area | Violation Trigger | Remediation |
|---|---|---|---|---|
| `UX-A11Y-001` | **Error** | Accessibility | Icon button lacking `aria-label`, `aria-labelledby`, or `title` | Add descriptive `aria-label="Action description"` |
| `UX-A11Y-002` | **Error** | Keyboard / A11y | `<div>` or `<span>` with click handler lacking `role="button"` or `tabIndex` | Use `<button>` or add `role="button" tabIndex={0} onKeyDown={...}` |
| `UX-A11Y-003` | **Error** | Accessibility | `<img>` or `<Image>` lacking `alt` attribute | Add descriptive `alt="..."` or `alt=""` for decorative images |
| `UX-FORMS-001`| **Warning** | Forms | `<button>` in form lacking explicit `type="button"` | Add `type="button"` to avoid unintended form submission |
| `UX-GRID-001` | **Warning** | Spatial Scale | Arbitrary non-8pt pixel spacing (e.g. `p-[7px]`, `margin: 13px`) | Align to 8pt scale (4px, 8px, 12px, 16px, 24px, 32px) |
| `UX-TOKEN-001`| **Warning** | Design System | Hardcoded hex literals (e.g. `bg-[#1a2b3c]`) | Reference semantic tokens (e.g. `bg-primary`, `text-muted`) |
| `UX-CLS-001`  | **Warning** | Performance | Media tags lacking explicit dimensions or aspect-ratio | Provide `width`, `height`, or `aspect-ratio` to eliminate layout shift |

---

## 3. Design System Tokens & Brand Presets

Tagisan provides 5 curated, mathematically balanced design system presets:

1. **Linear**: Ultra-dark charcoal (`#08090a`), signature electric indigo (`#5e6ad2`), refined slate borders (`#282a30`).
2. **Apple**: Crisp porcelain canvas (`#ffffff`), Cupertino system blue (`#0071e3`), SF Pro geometric typography, smooth corner curves (`0.75rem`).
3. **Stripe**: High-clarity white background (`#ffffff`), developer blurple (`#635bff`), mint accent (`#00d4aa`), Söhne typography.
4. **Cyberpunk**: Dystopian pitch (`#08080c`), radioactive neon yellow (`#fcee0a`), electric cyan (`#00f0ff`), warning magenta (`#ff003c`).
5. **Nord**: Arctic polar night (`#2e3440`), frost blues (`#88c0d0`, `#81a1c1`), snow storm neutrals (`#eceff4`).

### Export Targets:
- **Tailwind Config**: `tailwind.config.js` theme extension.
- **CSS Custom Properties**: `:root` and `.dark` variables.
- **W3C Design Tokens JSON**: DTCG standard format for cross-platform consumption.

---

## 4. Pre-Delivery UX Readiness Scoring Engine

Evaluates source repositories across 5 weighted pillars:
- **Accessibility (`a11y`)** [30%]: Zero WCAG 2.2 errors, all interactive elements labeled and keyboard navigable.
- **8-Point Spatial Grid (`grid`)** [15%]: Elimination of arbitrary pixel nudges and misaligned gutters.
- **Design Token Discipline (`tokens`)** [15%]: Centralized color/typography token adoption.
- **Layout Shift (CLS) Immunity (`cls`)** [20%]: Unsized image and dynamic iframe protection.
- **Forms & Inputs (`forms`)** [20%]: Clear labels, explicit button types, error states, and keyboard submit affordance.

### Verdict Gating:
- **Score $\ge 90$**: `✔ SHIP READY` - Approved for production deployment.
- **Score $75 - 89$**: `▲ NEEDS MINOR POLISH` - Minor warnings permitted, non-blocking for staging.
- **Score $< 75$ or Any Critical Error**: `✖ BLOCKED FOR RELEASE` - Hard CI failure.

---

## 5. The 500 UX Rules Catalog (12 Clusters)

The engine houses 500 structured rules indexed across 12 disciplines:
1. **A11Y (45 rules)**: WCAG 2.2 AA/AAA, focus rings (min 3:1 contrast, $\ge 2\text{px}$), ARIA attributes, screen reader text, live regions.
2. **TYPO (40 rules)**: Typographic scale (1.25 major third / 1.333 perfect fourth), line length (45-75 characters), line-height ($1.4-1.6\times$), font pairing.
3. **COLOR (40 rules)**: 60-30-10 distribution rule, semantic status palettes (success, warning, error, info), dark mode elevation desaturation.
4. **GRID (40 rules)**: 8pt baseline grid, 4pt micro-spacing, 12-column responsive layouts, fluid gutters, container padding.
5. **NAV (40 rules)**: Sticky navigation thresholds, breadcrumb trails, mobile drawer ergonomics, active tab indicators, skip links.
6. **FORM (45 rules)**: Top-aligned labels, inline validation on blur, autofocus discipline, field grouping, input masks, submit button loading states.
7. **CTRL (40 rules)**: Minimum touch targets ($44\times44\text{px}$ or $48\times48\text{px}$ Android), button hierarchy (primary, secondary, tertiary, destructive), disabled state tooltips.
8. **FEED (40 rules)**: Toast notification timeouts (4-6s), optimistic UI updates, non-intrusive banner alerts, empty state illustrations with action buttons.
9. **MOTO (40 rules)**: Physics-based spring easings, duration bounds (150ms-300ms for micro-interactions), `prefers-reduced-motion` overrides.
10. **PERF (45 rules)**: Cumulative Layout Shift (CLS $\le 0.1$), Largest Contentful Paint (LCP $\le 2.5\text{s}$), skeleton screens, font-display: swap.
11. **COPY (40 rules)**: Plain language, front-loaded sentence structures, actionable error messages (problem + solution), conversational tone consistency.
12. **GENAI (45 rules)**: Streaming token animations, prompt suggestion chips, cancel generation affordance, hallucination disclaimer badges, citation linkouts.

---

## 6. CLI Usage & Automation

```bash
# 1. Audit color contrast against WCAG 2.2
tgs ux contrast --fg "#5e6ad2" --bg "#ffffff" --target aa

# 2. Static lint code for UX/a11y defects
tgs ux scan --path ./src --strict

# 3. Generate design tokens for a preset
tgs ux tokens --preset linear --format css --output ./src/styles/tokens.css

# 4. Compute pre-delivery UX readiness score
tgs ux readiness --path . --threshold 90

# 5. Query the 500 UX rules catalog
tgs ux rules --cluster a11y --search "touch target"
```
