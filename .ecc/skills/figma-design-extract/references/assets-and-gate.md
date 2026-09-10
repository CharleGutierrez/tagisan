# Assets, and enforcing "no raw values"

Read this when the node contains icons or illustrations to export, or when you're ready to make "never hardcode" a real gate rather than a hope.

## Assets

Pull icons and images out of the node with `download_assets` (Figma MCP) rather than exporting by hand — it takes the same `nodeId` you've been using and writes the files locally.

Then, before committing:

- **Optimize.** Run SVGs through SVGO. Never commit a raw editor export — they carry editor metadata, absolute ids, and often a `<style>` block that leaks into the page.
- **Ensure an explicit `viewBox`.** Without it the SVG can't scale, and a `width`/`height` pair alone will fight your layout.
- **Strip hardcoded fills** from icons meant to inherit color. An icon with `fill="#333"` baked in cannot take your theme's token, and will silently fail dark mode. Replace with `fill="currentColor"`.

## Enforce it — a no-raw-values gate

"Never hardcode" is a rule the model can forget mid-build. Back it with a deterministic check that doesn't depend on the model's discipline: after building, grep the diff for raw values that should be tokens.

**This is a starting point, not a drop-in.** Any regex over a diff will have both false positives and false negatives on some stack. Tune it, then run it in CI.

```bash
#!/usr/bin/env bash
# Flag raw hex / rgb() / px / numeric font-weight literals in ADDED lines.
# Excluded: token-definition files (where the literals belong), custom-property
# declarations, and media queries (breakpoints are not design tokens).
set -uo pipefail

hits=$(
  git diff --unified=0 -- . \
      ':(exclude)**/tokens/**' ':(exclude)**/*.tokens.*' ':(exclude)**/theme/**' \
    | grep -E '^\+[^+]' \
    | grep -vE '@media|^\+\s*--[a-zA-Z0-9-]+\s*:' \
    | grep -inE '#[0-9a-f]{3}([0-9a-f]{3}([0-9a-f]{2})?)?\b|rgba?\([^)]*\)|[0-9]+(\.[0-9]+)?px\b|font-?weight["'"'"']?\s*[:=]\s*["'"'"']?[1-9]00\b'
)

if [ -n "$hits" ]; then
  printf 'raw values found — map them to tokens:\n%s\n' "$hits"
  exit 1
fi
echo "clean"
```

Notes on what it catches and misses:

- Matches `#abc`, `#aabbcc`, `#aabbccdd`, `rgb(...)`, `rgba(...)`, any `16px`, and both `font-weight: 600` (CSS) and `fontWeight: 600` / `fontWeight="600"` (JSX) — the last is the case a naive `font-weight:\s*[0-9]{3}` pattern misses entirely on the stack this skill targets.
- Skips `--space-4: 16px` inside a custom-property declaration and `@media (min-width: 768px)`, both of which are legitimate.
- Will still fire on `1px` borders and on `0px`. Either allow those explicitly, or accept the noise and let the reviewer wave it through — a gate that never fires is not a gate.
- A linter is the better long-term home: `stylelint` (`declaration-property-value-allowed-list`) for CSS, or an ESLint rule keyed to your token module for JSX. The grep exists so you have a gate *today*, in any repo, with no install.

## How this pairs with the drift log

Two different findings, two different fixes:

- A **bound** Figma value whose token resolves differently in your repo → **drift**. Log it; keep using the token *name*. Fix the token package separately.
- An **unbound** raw literal in your diff → tokenize it, or flag the missing Figma variable to the designer.

Never "fix" the first by hardcoding the Figma value. That converts a one-line package fix into permanent, invisible debt.
