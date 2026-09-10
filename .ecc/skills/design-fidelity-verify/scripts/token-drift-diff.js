#!/usr/bin/env node
// =============================================================================
// token-drift-diff.js — separate YOUR drift from the pile the app already had
// =============================================================================
//
// Diffs two design-fidelity snapshots and reports only what CHANGED:
//   NEW        drift now, none in the baseline           → your regression (exit 1)
//   CHANGED    drift in both, but the value moved
//   FIXED      drift in the baseline, PASS now
//   PERSISTING drift in both, unchanged                  → pre-existing, not yours
//   VARIANT    rendered state ≠ the Figma node           → re-pick the node
//   SKIPPED    in the baseline, not re-measured this run
//
// Without a baseline, every fidelity run re-reports the app's existing drift as
// noise, and the one row you actually introduced is buried in it.
//
// Pure fs + JSON. Platform-agnostic: the same snapshot schema comes off the web
// reader and the React Native reader.
//
// Usage:
//   token-drift-diff.js [--strict] <baseline.json> <current.json>
//
//   --strict  Fail (exit 3) if the current run has any MISSING entry.
//             A MISSING row is a selector that matched nothing. Left unchecked,
//             it makes a real drift in the baseline look FIXED — you close a bug
//             that is still there, because you renamed a class. Always pass
//             --strict before you re-baseline.
//
// Snapshot schema:
//   { label, route, viewport, figmaFile, capturedFrom, entries: [ {
//       id,            // stable key, e.g. "balance-card.title.color" — the join key
//       node,          // Figma node id
//       component,     // resolved component name
//       property,      // "title color"
//       figmaToken, figmaValue,     // spec side (token name + value)
//       measured, measuredToken,    // rendered value + the token it resolves to (null = HARDCODED)
//       verdict,       // PASS | DRIFT | HARDCODED | VARIANT | MISSING
//       scope,         // theme-level | component-level  (where the fix lands)
//       note
//   } ] }
//
// Seed the baseline by RUNNING the capturer, never by hand: measured numerics
// come back bare ("14", not "14px"), so a typed value diffs spuriously forever.
//
// Exit: 0 = no new drift · 1 = at least one NEW drift · 3 = --strict and a
//       MISSING entry · 64 = usage error. FIXED / PERSISTING / VARIANT never fail.
// =============================================================================

const fs = require('fs')

const argv = process.argv.slice(2)
const strict = argv.includes('--strict')
const positional = argv.filter((a) => !a.startsWith('--'))
const unknownFlags = argv.filter((a) => a.startsWith('--') && a !== '--strict')

// Exit code is this tool's API, so a mistyped extra file or unknown flag is a
// usage error — never silently ignored.
if (positional.length !== 2 || unknownFlags.length) {
  console.error('usage: token-drift-diff.js [--strict] <baseline.json> <current.json>')
  process.exit(64)
}
const [baseFile, curFile] = positional

const base = JSON.parse(fs.readFileSync(baseFile, 'utf8'))
const cur = JSON.parse(fs.readFileSync(curFile, 'utf8'))

const isDrift = (e) => e && (e.verdict === 'DRIFT' || e.verdict === 'HARDCODED')
const byId = (snap) => Object.fromEntries((snap.entries || []).map((e) => [e.id, e]))
const b = byId(base)
const c = byId(cur)
const desc = (e) =>
  `${e.property} — spec ${e.figmaToken || '?'}=${e.figmaValue || '?'} vs measured ${e.measured}` +
  `${e.measuredToken ? ` (${e.measuredToken})` : ' (no token)'}${e.scope ? ` [${e.scope}]` : ''}`

const line = (s) => console.log(s)
line(`TOKEN-DRIFT DIFF — baseline "${base.label || baseFile}" → current "${cur.label || curFile}"`)

const newDrift = []
const fixed = []
const persisting = []
const changed = []
const variants = []
const missing = []
const skipped = []
let regressions = 0

for (const id of Object.keys(c)) {
  const ce = c[id]
  if (ce.verdict === 'MISSING') {
    missing.push(`  ! MISSING ${id}: ${ce.property} — selector matched nothing; retarget, do not read as fixed`)
    continue
  }
  if (ce.verdict === 'VARIANT') {
    variants.push(`  ~ VARIANT ${id}: ${ce.property} — ${ce.note || 'rendered state ≠ Figma node'}`)
    continue
  }
  if (!isDrift(ce)) continue

  const be = b[id]
  if (!be || !isDrift(be)) {
    regressions++
    newDrift.push(`  ✗ NEW ${ce.verdict} ${id}: ${desc(ce)}`)
  } else if (be.measured !== ce.measured || be.verdict !== ce.verdict) {
    changed.push(`  … CHANGED ${id}: ${be.verdict} ${be.measured} → ${ce.verdict} ${ce.measured}`)
  } else {
    persisting.push(`  · PERSISTING ${ce.verdict} ${id}: ${desc(ce)}`)
  }
}

for (const id of Object.keys(b)) {
  if (!isDrift(b[id])) continue
  const ce = c[id]
  if (!ce) skipped.push(`  ○ ${id}: ${b[id].property} — in baseline, not re-measured (scoped run?)`)
  else if (ce.verdict === 'MISSING')
    skipped.push(`  ○ ${id}: ${b[id].property} — MISSING in current (stale selector / not rendered), NOT a fix`)
  else if (!isDrift(ce)) fixed.push(`  ✓ FIXED ${id}: ${b[id].property} (${b[id].verdict} → ${ce.verdict})`)
}

const section = (title, arr) => {
  if (arr.length) {
    line(`\n${title}`)
    arr.forEach(line)
  }
}
section('NEW DRIFT (introduced by this change):', newDrift)
section('CHANGED (drift value moved):', changed)
section('FIXED:', fixed)
section('PERSISTING (pre-existing — not this change):', persisting)
section('VARIANT (content-state mismatch — re-pick the Figma node):', variants)
section('MISSING (selector matched nothing — this run verified less than it looks):', missing)
section('SKIPPED (baseline entries not re-measured):', skipped)

line(
  regressions
    ? `\n→ ${regressions} NEW drift. ${persisting.length} pre-existing carried over.`
    : `\n→ No new drift. ${persisting.length} pre-existing, ${fixed.length} fixed.`,
)

if (strict && missing.length) {
  line(`\n→ --strict: ${missing.length} MISSING entr${missing.length === 1 ? 'y' : 'ies'}. Retarget before diffing or re-baselining.`)
  process.exit(3)
}
process.exit(regressions ? 1 : 0)
