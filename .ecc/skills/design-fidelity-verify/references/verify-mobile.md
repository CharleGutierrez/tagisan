# Verify UI — Mobile (iOS / Android)

The mobile mechanics for steps **B2 (capture)** and **B5 (numeric pass)** of the verify loop. The loop itself lives in `../SKILL.md`.

The numeric pass runs the bundled reader: [`../scripts/rn-style-read.js`](../scripts/rn-style-read.js). Don't re-derive the fiber walk each run.

## Tooling

**Primary recommendation: Argent** — an MCP server for driving iOS simulators / Android emulators and React Native apps.
- Site: https://argent.swmansion.com
- Repo: https://github.com/software-mansion/argent

Argent is the strongest fit because it can evaluate JavaScript in the app's JS runtime over Metro/CDP — the channel the numeric pass needs — as well as drive the device, record replayable flows, and inspect the RN tree, all from one server.

**Alternatives (the method is tool-agnostic):**
- **metro-mcp** — exposes `evaluate_js` / `inspect_at_point` over the same CDP channel; the reader runs unchanged.
- **agent-device** — https://github.com/callstackincubator/agent-device — agent-driven device control.
- **Lighter iOS-simulator MCP servers** — several community servers wrap `simctl` for boot/screenshot/tap. Good enough for capture and accessibility-tree placement, but without a JS-eval channel you cannot run the numeric pass (see *Graceful degradation*).

Pick whatever you have. What matters: **read the rendered value**, don't eyeball it. The calls below use Argent-style names — substitute your tool's equivalents.

## B1 — record the navigation as a flow

Record the path to the screen once and replay it each iteration so the measured state is identical. With Argent, capture the tap/scroll/type sequence as a flow and execute it on every re-verify. (If your tool has no flow recorder, keep the exact ordered list of interactions and replay them verbatim.)

**Tap coordinates and measured coordinates are different spaces.** CDP tap coordinates can differ from accessibility-tree and screenshot coordinates — don't assume they align. Drive taps from the accessibility tree; measure from the fiber read.

## B2 — capture full-resolution

**Capture at `scale: 1.0`.** Device-screenshot defaults are often downscaled (~0.3), and that blur is the #1 reason a "looks fine" capture hides real drift. Bump the scale to 1.0 every time.

> **Boot the simulator/emulator through your device tool, first.** A simulator left running from a prior `run:ios` may not be controllable by the MCP — gestures can silently no-op (returning `{tapped: true}` while nothing moves) and the accessibility tree can come back empty. Boot (or force-reboot) it through the tool up front so taps register and the tree is populated. Tell-tale: an action "succeeds" but the screenshot is identical.

## B5 — numeric pass: read the resolved style off the rendered fiber (PRIMARY)

This is the rigor a screenshot can't give you. The **primary** mobile channel is a **CDP fiber read** — no native injection needed, identical on iOS and Android.

**Why the fiber, not the native view.** NativeWind (and plain `StyleSheet`) resolve `className` / `style` into a concrete style object *in the JS layer*. That object lands on the **host** fiber's `memoizedProps.style`. Read it over the JS-runtime debugger (Argent `debugger-evaluate`, metro-mcp `evaluate_js`) and one read yields, per host `View` / `Text`:

- resolved **color** (already hex under recent NativeWind), **font** (family / size / weight / letterSpacing), **radius**, **padding / gap / margin**.
- an **absolute rect** `{x, y, w, h}`.
- the source `className` and component name, so you can jump to `file:line`.

### Run the bundled reader

```
1. Install once per app session — paste the whole of scripts/rn-style-read.js as the eval expression.
   The JS runtime persists between evals, so subsequent calls are cheap.
      argent     debugger-evaluate { expression: <contents of the file> }
      metro-mcp  evaluate_js       { code:       <contents of the file> }

2. Call it — in a separate eval, and WRAP THE CALL IN JSON.stringify:
      JSON.stringify(globalThis.__rnStyleRead({ targets: ["rounded-lg", "text-primary-500"], maxNodes: 250 }))
```

Then **walk every spec-table row** → spec value vs measured value → verdict + delta. Classify each row per [`drift-classification.md`](drift-classification.md); a bare hex/px delta is not a finding.

### Gotchas the reader already handles (and why)

Each of these silently corrupts a hand-rolled walk:

- **Most eval wrappers return only primitives.** Argent and metro-mcp marshal a primitive back, so an object result is *silently dropped* — `JSON.stringify(...)` the call, or you get nothing back and no error. (Raw CDP `Runtime.evaluate` returns a RemoteObject and wouldn't need this, but stringify anyway — it's harmless.)
- **The host fiber carries no `className`.** It lives 2–7 hops up; the reader climbs `.return` to find it.
- **`stateNode.canonical.publicInstance` is lazily created and usually absent** on the New Architecture (Fabric). Use `nativeFabricUIManager.measureInWindow(stateNode.node, cb)` — the callback fires **synchronously** inside the eval, so no Promise plumbing is needed.
- **Style may be a nested array.** Flatten it, last-wins.
- **`fontWeight` may be the number `600` or the string `"600"`.** The reader normalizes to a string.
- **`rounded-full` surfaces as a huge sentinel radius**, not `9999`. Normalized.
- **The fiber walk includes other mounted tabs.** Disambiguate by text, rect, or route — not by className alone.
- **Multiple devtools renderers** may be registered; the reader picks the app's.

### Two perf levers

- `measure: false` unless a target actually reads a rect — each returned node otherwise costs a native `measureInWindow`, the dominant cost of a large read.
- `styleKeys: [...]` trims each node's style to the keys you compare, shrinking the CDP payload and the parse.

### letterSpacing

Figma's `letterSpacing` is a **percent**, not px — see `figma-design-extract` step 2. A spec of `-2` means `-0.02em`, whose rendered value is font-size dependent (18px → `-0.36`). Convert before comparing, or every letterSpacing row is a silent false PASS.

## Android — CDP is the ONLY style source

Validated live on an Android emulator (RN 0.85, Fabric/Hermes). `native-find-views` does not exist on Android, so the CDP fiber read above is the *only* style source; the accessibility tree (`describe` / uiautomator) gives geometry and text only. The reader runs **unchanged** and returns the same resolved style and rect as iOS, byte-identical across repeated reads.

- **`adb -s <serial> reverse tcp:8081 tcp:8081` BEFORE connecting to Metro/CDP.** Without it the app has no route to Metro and there is no CDP target at all. Re-run if adb drops.
- **Rect:** `nativeFabricUIManager.measureInWindow` fires synchronously in-eval here too — absolute **window** coordinates in **dp** (screen width = px / density). There is no `publicInstance` fallback.
- **Taps come from the accessibility tree, never the RN tree** — RN-tree coordinates collapse to `0.50,0.50` on Android.
- **Geometry corroboration:** uiautomator x-centers match the CDP rects exactly; y is offset by the status-bar height (CDP is window-relative, uiautomator display-relative). Expected, not a bug.
- **Determinism:** scope targets to *static* classNames. An animated element's rect changes between reads — that's an animation artifact, not a reader bug.
- **Parity:** Android resolved values should equal the iOS baseline (same JS layer). A divergence is a real platform bug worth reporting.
- **Native paint readback is N/A.** Record it as a stated residual; corroborate bounds via uiautomator only.

## iOS native paint props — BONUS cross-check, not the primary pass

`native-find-views` returns native `backgroundColor` / `frame` / `cornerRadius` / `font`. Use it to **double-confirm** background and frame, not as the numeric pass:

- It needs native devtools injected — confirm with a devtools-status call **first, in preflight (B0)**, not at measurement time. On many dev-client builds injection never connects (`connected: false`).
- **Even when connected, on Fabric it surfaces less than the CDP read** — typically `backgroundColor` + `frame`, but no `cornerRadius` (on `RCTViewComponentView`) and no font or text color (those live on the `NSAttributedString`). The CDP fiber read is the more complete source.

**Single-element spot-check** (no fiber walk): `debugger-inspect-element { x, y }` (Argent) or `inspect_at_point { x, y }` (metro-mcp) returns `file:line` + props for one on-screen coordinate — useful for confirming the live element is the code you think it is.

## Graceful degradation

If the CDP eval channel is unavailable, the **accessibility / view tree** still returns normalized frames `(x, y, w, h)` and element *order* — a genuine numeric source for placement, bounds, sizing, and spacing. What remains unmeasurable is hex color, radius, and font weight. Record those as an explicit **residual** ("hex not measured: no JS-eval channel"); cover them qualitatively with a sharp `scale: 1.0` capture and a source-level audit. Do **not** silently claim a full pass.

## Map back to source

Use the tool's coordinate→source inspector (e.g. `debugger-inspect-element`) to jump from an on-screen discrepancy straight to the `file:line` and component that renders it, instead of hunting through the tree.

**`componentName` is the nearest *named ancestor*, not your file's component.** A `ProductCard` that renders through a shared `Card` wrapper reports `componentName: "Card"`. Key your targets on that, plus a distinctive className, plus a text anchor.
