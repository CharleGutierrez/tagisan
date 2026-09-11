---
name: "refactoring-ui"
description: "Tactical visual design and layout refactoring principles based on Refactoring UI by Adam Wathan and Steve Schoger. Use when polishing interfaces, establishing visual hierarchy, balancing whitespace and contrast, calibrating typography scales, designing multi-layer depth with shadows, or removing visual clutter in web and mobile applications."
---

# Refactoring UI Engineering Skill

Tactical, developer-first principles for transforming rough software layouts into clean, balanced, and aesthetically compelling user interfaces without relying on arbitrary aesthetic guesswork.

---

## 1. Core Principles

1. **Grayscale First**: Design layout, hierarchy, and spatial rhythm in monochrome before introducing color. If a layout doesn't work in black and white, adding color will only mask fundamental hierarchy flaws.
2. **Hierarchy Over Pure Sizing**: Avoid making important items gigantic and less important items tiny. Establish hierarchy through **weight** (e.g., `font-semibold` vs `font-normal`), **color contrast** (`text-slate-900` vs `text-slate-500`), and **spatial positioning**.
3. **Double or Half (Systematic Spacing)**: Never use arbitrary padding or margins (e.g. `13px`, `27px`). Stick strictly to an 8pt/4pt scale: `4px` (1), `8px` (2), `12px` (3), `16px` (4), `24px` (6), `32px` (8), `48px` (12), `64px` (16).
4. **Optical Balancing over Mathematical Centering**: Visual weight frequently deviates from geometric bounds. Shift icons, badges, and asymmetric shapes manually by 1–2px until they look visually centered to the human eye.
5. **Depth via Multi-Layered Elevation**: Realistic depth is achieved through two shadow layers: an **ambient shadow** (diffuse, soft, minimal Y-offset) and a **key-light shadow** (crisp, directional, larger Y-offset).

---

## 2. Typography Calibration

### Contrast and Role Scale
- **Display / H1**: Large, tight letter-spacing (`tracking-tight`), line height equal to font size (`leading-none` or `leading-tight`).
- **Headings (H2-H3)**: Medium weight (`font-semibold` or `font-medium`), slightly snug leading (`leading-snug`).
- **Body**: Regular weight (`font-normal`), relaxed leading (`leading-relaxed` ~ 1.5 to 1.6), line length constrained to 45–75 characters (`max-w-prose` or `max-w-[65ch]`).
- **Labels & Captions**: Small (`text-xs` / `text-sm`), elevated weight (`font-medium`), wide tracking (`tracking-wide` or `tracking-wider`) when uppercase.

```html
<!-- Balanced hierarchy without massive font scale differences -->
<div class="space-y-1">
  <span class="text-xs font-semibold uppercase tracking-wider text-indigo-600">Performance Index</span>
  <h2 class="text-xl font-semibold tracking-tight text-slate-900">Throughput Optimization Engine</h2>
  <p class="text-sm font-normal leading-relaxed text-slate-500 max-w-prose">
    Tagisan processes multi-model consensus adjudication in sub-50ms cycles using distributed lock-free pipelines.
  </p>
</div>
```

---

## 3. Depth, Borders, and Elevation

### Replacing Harsh Gray Borders
Harsh gray borders create excessive visual noise and section off elements too rigidly. Use these alternatives:
1. **Background Contrast**: Place a white card on a subtle gray background (`bg-slate-50`).
2. **Subtle Box Shadows**: Use soft, tinted shadows instead of outline strokes.
3. **Whitespace Dividers**: Separate distinct sections with generous padding (`py-8`) rather than explicit `<hr>` tags.

### Layered Shadow Recipe
```css
/* Multi-layer elevation for cards and floating modals */
.elevation-card {
  box-shadow: 
    0 1px 3px 0 rgba(15, 23, 42, 0.06),  /* Ambient light */
    0 1px 2px -1px rgba(15, 23, 42, 0.04); /* Edge definition */
}

.elevation-floating {
  box-shadow: 
    0 10px 15px -3px rgba(15, 23, 42, 0.08), /* Ambient shadow */
    0 4px 6px -4px rgba(15, 23, 42, 0.04);  /* Key-light shadow */
}
```

---

## 4. Color Palette Construction

1. **Greys with a Tint**: Avoid pure neutral grays (`#808080`). Use slate (cool blue tint), zinc (neutral cool), or stone (warm earthy tint) to give interfaces character.
2. **Semantic Distinction**:
   - **Primary**: Brand accent (use sparingly for primary actions and active states).
   - **Neutral**: 80% of the UI (surfaces, typography, dividers).
   - **Feedback States**: Red (destructive/error), Amber (warning/caution), Emerald (success/confirmation), Sky (informational).
3. **Accessible Text Contrast**: Ensure text passes WCAG AA contrast (>= 4.5:1 against background).

---

## 5. Vibe Coder Directives for AI Prompts

When directing AI agents to polish interfaces, use these exact instructions:
- *"De-clutter this card by removing 1px borders and replacing them with a layered ambient drop shadow (`shadow-sm`) and a 1-shade background contrast."*
- *"Enforce an 8pt spatial grid: ensure all margins, padding, and gaps use multiples of 4px/8px."*
- *"Adjust the typography hierarchy: reduce the H1 from 4xl to 2xl, but increase the weight to `font-bold` and darken the heading to `text-slate-900` while shifting secondary metadata to `text-slate-500`."*
- *"Constrain all explanatory paragraphs to `max-w-[65ch]` and set line-height to `leading-relaxed`."*
