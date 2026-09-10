# Verify UI — Web

The web mechanics for steps **B2 (capture)** and **B5 (numeric pass)** of the verify loop. The loop itself lives in `../SKILL.md`.

The numeric pass runs the bundled reader: [`../scripts/web-style-read.js`](../scripts/web-style-read.js).

**Tooling (examples — use what you have connected):** a browser-driving MCP such as Playwright MCP, Chrome MCP / "Claude in Chrome", or any headless-browser control that can run JavaScript in the page and take screenshots. The method is tool-agnostic; only the call names change.

## B1 — record the navigation as a flow

Script the path to the screen as a reusable sequence (navigate to URL → click → fill → wait for selector), so every re-verify lands on the identical state. With Playwright this is a short script or a recorded trace; with a browser MCP it's the fixed list of navigate/click calls. Re-run the *same* flow each iteration.

## B2 — capture full-resolution

Capture at **devicePixelRatio ≥ 2** so text edges and hairline borders are crisp.

- Playwright: launch the context with `deviceScaleFactor: 2` (or emulate a 2x device), then `page.screenshot({ path, fullPage: false })` of the target region.
- Browser MCP: set a 2x/retina viewport before the screenshot call.

A 1x capture hides sub-pixel spacing and color-banding drift — don't measure from it, and don't trust a vision pass done on it.

## B5 — numeric pass: read computed styles

Read the **rendered** values directly from the DOM. `getComputedStyle` returns the resolved, post-cascade value (px, resolved color, actual font), which is exactly what you compare against the spec table.

Paste [`../scripts/web-style-read.js`](../scripts/web-style-read.js) into the page (Playwright `page.evaluate`, or the browser MCP's JS-eval tool), then call it with your selectors:

```js
// After pasting the file once, the page holds globalThis.__webStyleRead.
JSON.stringify(__webStyleRead(['[data-testid="pricing-card"]', '[data-testid="pricing-card"] h3']))
```

It returns, per selector, the normalized rendered style (`color` and `backgroundColor` already converted to hex, box values rounded to the nearest px) plus the raw values so you can audit the normalization. The same file runs in plain node as a module, which is how its normalizers are unit-tested.

Then for each spec-table row — classify with [`drift-classification.md`](drift-classification.md), not a bare delta:

- **Color:** computed colors come back as `rgb()` / `rgba()`. The reader converts to hex before you compare; `#3366cc` vs `rgb(51,102,204)` is a match, `rgb(51,102,200)` is not. A fully transparent `rgba(0,0,0,0)` normalizes to `transparent`, which is *not* the same finding as a wrong color.
- **Spacing / radius / font-size:** compare the `px` numbers directly to the spec.
- **Gap:** read `gap` (or `rowGap` / `columnGap`) on the flex/grid *container*, not the children. **A named drift pattern:** a `Stack` / `Box` that spaces its children with `marginTop` instead of `gap` reports `gap: "normal"` while looking correct. The reader flags this.
- **Sizing / placement:** use `getBoundingClientRect()` width/height and x/y.
- **Tolerance:** `getBoundingClientRect()` returns *fractional* pixels (zoom / devicePixelRatio / sub-pixel layout), so a literal `123.5 === 124` check produces noisy FAILs. Compare box/position values with a small tolerance — the reader rounds to the nearest px; allow ±1px. Apply the same rounding to spacing/gap deltas.
- **letterSpacing:** Figma's value is a **percent**, the computed value is px — see `figma-design-extract` step 2. Convert the spec side before comparing, or every letterSpacing row is a silent false PASS. Note computed `letterSpacing` is the string `"normal"` when unset, not `"0px"`.
- **Token check (bonus):** if your app exposes CSS custom properties, read `getComputedStyle(document.documentElement).getPropertyValue('--primary-500')` and confirm the element actually resolved to the *token*, not a hardcoded literal. This is the cheap version of the reverse-lookup that turns a `FAIL` into a `DRIFT` vs `HARDCODED` verdict.

## Graceful degradation

If JS-eval is blocked (CSP, no devtools access), you can still measure **placement and sizing** from `getBoundingClientRect` via accessibility/DOM snapshots, and read colors from a full-res capture qualitatively. Record any property you couldn't read computationally as an explicit **residual** in the report rather than claiming a full numeric pass.

## Map back to source

Use the element's stable selector / `data-*` attribute / component name to jump from an on-screen discrepancy to the `file:line` that renders it. Many browser MCPs expose the DOM node's source; otherwise grep the component by its text/test id.

## Bonus — structural check with ARIA snapshots

Before (or alongside) the per-property numeric pass, a cheap structural assertion catches "wrong element / wrong order / missing label" fast and stays stable across layout refactors. With Playwright:

```js
await expect(page.locator('[data-testid="pricing-card"]')).toMatchAriaSnapshot(`
  - heading "Pro" [level=3]
  - text: /\\$\\d+/
  - button "Choose Pro"
`);
```

It verifies role / name / order / hierarchy, not pixels — complementary to the computed-style diff, and a natural extension of mapping elements to code (B3).
