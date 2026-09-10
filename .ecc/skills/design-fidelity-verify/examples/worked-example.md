# Worked example — verifying the "Pricing card"

Continues the [`figma-design-extract` worked example](../../figma-design-extract/examples/worked-example.md). We built the Pricing card from its spec table; now we prove the running app matches — once on **web**, once on **mobile**.

The spec table (abridged). We verify it against **two** codebases — a web app and a React Native app — so the `repo token` column is per-stack where the two differ:

```
element        | property | spec value            | repo token (web / mobile)
---------------+----------+-----------------------+--------------------------
Card container | radius   | 12                    | rounded-xl
Card container | padding  | 16                    | p-4
Plan name      | size/wt  | 18 / 600              | text-lg font-semibold
CTA            | bg       | #3366cc (primary/500) | bg-primary-500
CTA            | radius   | 8                     | rounded-lg
CTA            | height   | 48                    | h-12
Feature list   | gap      | 8                     | gap-2 (web) / gap-xsmall (mobile)
```

The gap split is not a typo: the web repo is a `rem`-16 Tailwind scale where `gap-2` = 8px, the mobile repo a named scale where `8` is `gap-xsmall`. Same design value, different token — see the mobile run below for what happens when you carry the wrong one across.

---

## Web run (browser/Playwright MCP)

**B0 preflight:** dev server up, card visible on `/pricing`, JS-eval available. ✅
**B1 flow:** `navigate /pricing` → `waitFor [data-testid=pricing-card]`. Recorded.
**B2 capture:** screenshot at `deviceScaleFactor: 2`.
**B5 numeric pass** — pasted `scripts/web-style-read.js`, then called it with the card's selectors. State parity checked first: the live card is the "Most popular" variant, same as the Figma node. Not a `VARIANT`.

```
element        | property | spec token / value   | measured / resolves to  | delta | verdict   | scope
---------------+----------+----------------------+-------------------------+-------+-----------+----------------
Card container | radius   | rounded-xl 12px      | 12px (rounded-xl)       | 0     | PASS      | —
Card container | padding  | p-4 16px             | 16px (p-4)              | 0     | PASS      | —
Plan name      | size     | text-lg 18px         | 18px (text-lg)          | 0     | PASS      | —
Plan name      | weight   | font-semibold 600    | 400 (font-normal)       | -200  | DRIFT     | component-level
CTA            | bg       | primary-500 #3366cc  | #3366cc (primary-500)   | —     | PASS      | —
CTA            | radius   | rounded-lg 8px       | 6px (rounded-md)        | -2    | DRIFT     | component-level
CTA            | height   | h-12 48px            | 48px (h-12)             | 0     | PASS      | —
Feature list   | gap      | gap-2 8px            | 8px (gap-2)             | 0     | PASS      | —
```

Note the CTA background: computed came back `rgb(51,102,204)`, normalized to `#3366cc` before comparing. Raw `rgb()` vs spec hex is not a mismatch.

Both failures are `DRIFT`, not `HARDCODED` — the component reached for a real token, just the wrong one. That's the difference between "someone typed a hex" and "someone typed `rounded-md`", and it changes the fix.

**B7 fix (iteration 1):** plan name `font-normal` → `font-semibold`; CTA `rounded-md` → `rounded-lg`. Replay flow, re-capture, re-measure → both PASS. Two fixes, one iteration. Done.

---

## Mobile run (Argent, iOS simulator)

**B0 preflight:** simulator force-booted through the MCP, app authenticated, test account has ≥2 plans so the card list actually renders. JS-eval channel confirmed attached. ✅
**B1 flow:** launch app → tap "Plans" tab → wait for card. Recorded as a replayable flow.
**B2 capture:** `screenshot scale: 1.0`.
**B5 numeric pass** — installed `scripts/rn-style-read.js` once, then:

```js
JSON.stringify(__rnStyleRead({ targets: ["rounded-xl", "bg-primary-500", "text-lg"], maxNodes: 250 }))
```

The fiber read returns the resolved style *and* an absolute rect for each host node — colors already hex, `fontWeight` normalized to a string. No native devtools injection required, so the "injection won't attach" problem never arises.

**One thing about this repo matters for the gap row below.** The web app maps the design's `8px` gap to Tailwind's `gap-2` (a `rem`-16 scale, so `gap-2` = 8px — correct). This React Native app uses a **named** spacing scale where `8` is `gap-xsmall`; its numeric NativeWind classes resolve through the default `inlineRem` of **14**, so `gap-2` here is `7px`, not `8`. Same design value, different token in each stack — exactly the kind of thing `figma-design-extract`'s [`stack-mapping.md`](../../figma-design-extract/references/stack-mapping.md) warns to check per repo. The generic spec table said `gap-2`; on this stack that's a bug, and verify is what catches it.

```
element        | property | spec token / value   | measured / resolves to  | delta | verdict   | scope
---------------+----------+----------------------+-------------------------+-------+-----------+----------------
Card container | radius   | rounded-xl 12        | 12 (rounded-xl)         | 0     | PASS      | —
Card container | width    | 343                  | 343                     | 0     | PASS      | —
Plan name      | weight   | font-semibold 600    | "600" (font-semibold)   | 0     | PASS      | —
CTA            | bg       | primary-500 #3366cc  | #3366cc (primary-500)   | —     | PASS      | —
CTA            | height   | h-12 48              | 44 (h-11)               | -4    | DRIFT     | component-level
Feature list   | gap      | gap-xsmall 8         | 7 (no token)            | -1    | HARDCODED | component-level
```

Two findings worth reading closely:

- **CTA height** is a plain `DRIFT` — someone wrote `h-11`.
- **Feature-list gap** measured `7`, which resolves to *no* token. The component used the generic table's `gap-2`; on this named-scale repo that resolves through `inlineRem` 14 to `7px` — off the 4px grid and mapping to nothing, where the design's `8` should have been `gap-xsmall`. That is `HARDCODED`, not `DRIFT`, and the fix is the *class*, not the number.

`debugger-inspect-element` confirms the CTA maps to `components/PricingCard.tsx:42` — the live UI is the code we think it is.

**B7 fix (iteration 1):** `h-11` → `h-12`; `gap-2` → `gap-xsmall`. Replay flow, re-measure → both PASS.

### If the eval channel had been unavailable

Then the accessibility/view tree still yields frames, so placement, bounds, sizing, and gap remain measurable. Hex, radius, and font-weight would be carried as an explicit **residual** ("not measured: no JS-eval channel"), checked qualitatively against the sharp `scale: 1.0` capture plus a source audit. Never a silent full-pass claim.

On **Android**, native paint readback is `N/A` by definition — record it as a stated residual every time, and remember `adb reverse tcp:8081 tcp:8081` before connecting, or there is no CDP target at all.

---

## Takeaway

Same loop on both platforms; only the capture and read mechanics differed. The numeric pass caught a `-200` font weight, a `-2px` radius, a `-4px` height, and a `-1px` gap that a vision-only check would have waved through.

More importantly, it named *what* each one was. A report that says `gap: expected 8, got 7` gets shrugged at. A report that says `gap resolves to no token — the utility class silently used a 14px rem base` gets fixed, once, for the whole app.
