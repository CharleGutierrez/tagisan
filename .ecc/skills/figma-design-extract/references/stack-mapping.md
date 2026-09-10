# Map Figma variables to your stack

Read this when filling the **repo token / class** column of the design-spec table.

The mechanism is identical everywhere; only the token syntax changes. These are **examples to adapt** — use whatever your repo already uses. The rule that never changes: a value bound to a Figma variable maps to a *named token* in your code, never to a hardcoded literal.

## Tailwind / NativeWind (utility classes)

```
var(--primary/500)  →  bg-primary-500
radius 12           →  rounded-xl
gap 8               →  gap-2
```

For light/dark, lean on your config's dark variant: `bg-white dark:bg-neutral-900`.

> **Check whether your spacing scale is named before assuming it's numeric.** A repo that defines `p-medium = 16` has *no* token at `p-4` — and NativeWind will happily resolve `p-4` through its default `inlineRem` (14, not 16), silently landing ~12.5% off the design's 4px grid. That renders as a `HARDCODED` verdict at verify time with no obvious cause. Read your config, not the Tailwind defaults.

## Restyle / theme objects (React Native)

Map to theme keys: `backgroundColor="primary500"`, `borderRadius="l"`, `spacing="s"`.

## CSS variables / vanilla

```css
background: var(--primary-500);
border-radius: var(--radius-md);
```

## Theme objects with a non-linear scale (MUI and friends)

A `theme.spacing` may be a **custom array** — e.g. `[0, 4, 8, 12, 16, 20, 24, 28, 32, 40, 48, 64]` — rather than `n × 8`. Resolve a measured px value through the array index, not by dividing. Similarly, a theme's weight vocabulary may not have a key for every Figma weight: if `600` is aliased as "Medium" and there is no `fontWeightSemiBold`, say so in the table rather than inventing the key.

## Color-only props

SVG/icon strokes and native props that can't take a class → read the resolved color from your theme/token module rather than pasting a hex.

## Typography

Figma's `letterSpacing` is a **percent**. `-2` means `-0.02em`; the rendered px depends on font size (24px → `-0.48`, 18px → `-0.36`, 16px → `-0.32`). Convert before you write the token, and again before you compare at verify time.

Figma's line-height may be a percent, a px value, or `auto`. `auto` is the font's own metrics — don't write a literal.

## When nothing maps

If your repo has none of these, that's the real finding — surface it; don't invent ad-hoc values. A Figma variable with no counterpart in your token system is either (a) a token you're missing, or (b) a design that reached for something outside the system. Both are worth a sentence to the designer; neither is worth a hardcoded hex.
