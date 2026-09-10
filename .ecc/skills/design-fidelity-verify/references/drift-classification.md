# Classifying a failing row

Read this before writing the B6 discrepancy report. It turns a measurement into a **finding** — something with a named cause and a known place to fix it.

## The rule

> **Resolve every measured value back to its named token before you label the row. A bare hex/px delta is not a finding.**

`CTA bg: spec #3366cc, app #3060c0, delta ✗` tells the reader nothing actionable. It doesn't say whether someone typed a hex by hand, reached for the wrong token, or whether the token itself is wrong. Those are three different fixes in three different files — and one of them isn't even in this repo.

So: take the measured value, look it up in your design-token package, and report *which token it is*. Then the row classifies itself.

## The five verdicts

| Verdict | Meaning | Where the fix lands |
|---|---|---|
| **PASS** | Measured value equals the spec value. | — |
| **DRIFT** | Measured value resolves to a token, but **not the spec's token**. Name *both*. | The component: it reached for the wrong token. |
| **HARDCODED** | Measured value resolves to **no token at all**. | The component: a literal was typed in. |
| **VARIANT** | The rendered element is in a **different state** than the Figma node you're comparing against. | Nothing. Re-pick the node. |
| **MISSING** | The selector matched nothing. | Your targets — not the app. |

**`DRIFT` names two tokens.** `"Title color: spec text-primary (#1e293b), app resolves to text-secondary (#334155)"`. That's a one-line fix and the reviewer can see it's a fix.

**`HARDCODED` names none.** `"Card bg: spec surface-0 (#ffffff), app #fdfdfd — resolves to no token"`. Also a one-line fix, but a different one.

A well-tokenized app produces mostly `DRIFT`, not `HARDCODED`. Say which — the ratio tells the team whether they have a discipline problem or a vocabulary problem.

## VARIANT-match *before* you call drift

**Check state parity first.** If the live screen shows an *unpaid* invoice and the Figma node you pulled is the *paid* variant, then every content, weight, and color delta downstream of that is noise. The element isn't drifting; you're comparing two different designs.

Symptoms: several rows fail together, the copy doesn't match, a badge is a different color *and* a different word. Don't fix any of it. Re-pick the Figma node for the state that's actually on screen, then re-run.

This check is cheap and it comes first. A verify run that "found 9 drifts" and turns out to have been reading the wrong variant costs more credibility than one that found nothing.

## Tag every drift `theme-level` or `component-level`

Where does the fix go?

- **`theme-level`** — the element inherited the wrong value from a theme default, with no local override. The fix is **upstream, in the design-system package**, and it is **cross-app**. Do not propose a local patch for a page-wide default; you'd be papering over it in one place and leaving it broken in ten.
- **`component-level`** — the component explicitly set the value (a local `style`, `sx`, `styled`, or an inline class). The fix is here, in this repo.

**State the target repo in the report.** A `theme-level` finding that reads like a local bug will get "fixed" locally, and the drift survives.

How to tell: read the source. If the property is set nowhere in the component and its ancestors, it came from the theme.

## Disambiguate the token category before reverse-lookup

`16` is a spacing token **and** a fontSize token **and** a border radius. Reverse-lookup on the bare number and you'll match the wrong one, then report a `DRIFT` against a token nobody used.

Pin the category from the *resolved-style key* you read it from — `paddingLeft` → spacing, `fontSize` → fontSize, `borderRadius` → radius — and search only that category.

## MISSING is an error, not a pass

If a selector matches nothing, the row is `MISSING`. Never let it fall through as `PASS`, and never let it silently vanish from the report.

The specific trap: on a re-verify against a baseline, a `MISSING` row makes the differ see a drift in the baseline and nothing in the current run — which reads as **`FIXED`**. You then close a bug that is still there, because you renamed a class.

Treat `MISSING` as "retarget your selector and run again." A run with any `MISSING` row has not verified the screen. If you're gating CI, fail the run.

## The discrepancy report

```
element  | property | spec token / value        | measured / resolves to     | delta | verdict    | scope
---------+----------+---------------------------+----------------------------+-------+------------+----------------
CTA      | bg       | primary-500  #3366cc      | #3060c0  (primary-600)     | —     | DRIFT      | component-level
Title    | color    | text-primary #1e293b      | #334155  (text-secondary)  | —     | DRIFT      | theme-level
Card     | bg       | surface-0    #ffffff      | #fdfdfd  (no token)        | —     | HARDCODED  | component-level
Row gap  | gap      | space-2      8            | 12       (space-3)         | +4    | DRIFT      | component-level
Badge    | label    | "Paid"                    | "Outstanding"              | —     | VARIANT    | re-pick node
Footer   | radius   | rounded-md   8            | selector matched nothing   | —     | MISSING    | retarget
```

`delta` is a number or `—`. Numeric properties get a signed delta; colors and strings don't. Keep `verdict` and `scope` in their own columns — the point of the table is that a reader can sort by where the fix goes.

## The baseline (B9) — the entry schema

The same classification serializes into a snapshot so the *next* run can tell **your** regression from the pile the app already had.

```json
{
  "label": "pricing-page",
  "route": "/pricing",
  "viewport": "1280x800",
  "figmaFile": "AbC123",
  "capturedFrom": "scripts/web-style-read.js",
  "entries": [
    {
      "id": "cta.bg",
      "node": "210:540",
      "component": "PricingCTA",
      "property": "background color",
      "figmaToken": "primary-500",
      "figmaValue": "#3366cc",
      "measured": "#3060c0",
      "measuredToken": "primary-600",
      "verdict": "DRIFT",
      "scope": "component-level",
      "note": ""
    }
  ]
}
```

Eleven fields, platform-agnostic. `id` is a stable key (`<element>.<property>`) — it's what the differ joins on, so it must survive a re-render.

Then [`../scripts/token-drift-diff.js`](../scripts/token-drift-diff.js) compares two snapshots:

| Outcome | Meaning | Exit |
|---|---|---|
| **NEW** | Drift present now, absent in the baseline. **This is yours.** | `1` |
| **CHANGED** | Drift in both, but the measured value moved. | `0` |
| **PERSISTING** | Drift in both, unchanged. Pre-existing; not introduced by this change. | `0` |
| **FIXED** | Drift in the baseline, `PASS` now. | `0` |
| **SKIPPED** | In the baseline, absent from this run's targets. | `0` |
| **VARIANT** | Rendered state ≠ the Figma node. Informational — re-pick the node. | `0` |
| **MISSING** | Targeted, matched nothing. | `3` under `--strict` |

Only **NEW** fails the run (and **MISSING** under `--strict`). A `CHANGED` value that is still drift is still pre-existing — it doesn't fail, but it's worth a look.

Without a baseline, every fidelity run re-reports the app's existing drift as noise, and the one row you actually introduced is buried in it.

**Three rules for living with a baseline:**

1. **Seed it by *running* the capturer, never by hand.** Measured numerics come back bare — `"14"`, not `"14px"`. A hand-typed value diffs spuriously forever.
2. **`--strict` before you re-baseline.** A `MISSING` row silently converts a real drift into a `FIXED`. Fail on it.
3. **After a fidelity fix lands, flip that entry to `PASS`** and commit the updated baseline — otherwise the next run reports your fix as pre-existing drift, forever.
