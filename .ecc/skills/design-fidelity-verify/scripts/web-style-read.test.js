#!/usr/bin/env node
// Unit tests for the pure normalizers in web-style-read.js.
// Runs in plain node — no browser. `readAll`/`readSpec` need a DOM and are
// exercised in a real page, not here.

const t = require('./web-style-read.js')
const assert = require('node:assert')

let failed = 0
function check(label, got, want) {
  try {
    assert.deepStrictEqual(got, want)
    console.log(`PASS  ${label}`)
  } catch (e) {
    failed++
    console.log(`FAIL  ${label} — got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`)
  }
}

// normalizeColor — the false-FAIL guard: rgb() and hex are the same value.
check('rgb spaced -> hex', t.normalizeColor('rgb(51, 102, 204)'), '#3366cc')
check('rgb tight -> hex', t.normalizeColor('rgb(51,102,204)'), '#3366cc')
check('fully transparent -> transparent', t.normalizeColor('rgba(0, 0, 0, 0)'), 'transparent')
check('alpha -> 8-digit hex', t.normalizeColor('rgba(0,0,0,0.5)'), '#00000080')
check('space/slash syntax', t.normalizeColor('rgba(51 102 204 / 0.5)'), '#3366cc80')
check('3-digit hex expands', t.normalizeColor('#ABC'), '#aabbcc')
check('hex lowercased', t.normalizeColor('#3366CC'), '#3366cc')
check('transparent passthrough', t.normalizeColor('transparent'), 'transparent')
check('named color left raw', t.normalizeColor('currentcolor'), 'currentcolor')

// roundPx — fractional rects are noise; round them.
check('px string rounds', t.roundPx('12.5px'), 13)
check('bare fractional rounds', t.roundPx('123.5'), 124)
check('zero', t.roundPx('0px'), 0)
check('normal -> null', t.roundPx('normal'), null)
check('number passthrough', t.roundPx(16), 16)

// letterSpacing — "normal" (font decides) is not the same as 0 (explicitly none).
check('ls normal preserved', t.normalizeLetterSpacing('normal'), 'normal')
check('ls px', t.normalizeLetterSpacing('-0.36px'), -0.36)
check('ls zero', t.normalizeLetterSpacing('0px'), 0)

// figmaLetterSpacingToPx — the silent-false-PASS trap: Figma's -2 is a percent.
check('figma -2% @ 18px', t.figmaLetterSpacingToPx(-2, 18), -0.36)
check('figma -2% @ 24px', t.figmaLetterSpacingToPx(-2, 24), -0.48)
check('figma -2% @ 16px', t.figmaLetterSpacingToPx(-2, 16), -0.32)
check('figma -1% @ 18px', t.figmaLetterSpacingToPx(-1, 18), -0.18)

// detectGapDrift — the container that spaces with marginTop looks right, measures wrong.
const drift = t.detectGapDrift({ display: 'flex', gap: 'normal' }, ['8px', '8px'])
check('gap drift detected', typeof drift === 'string' && /marginTop/.test(drift), true)
check('real gap -> no drift', t.detectGapDrift({ display: 'flex', gap: '8px' }, ['0px', '0px']), null)
check('block layout -> no drift', t.detectGapDrift({ display: 'block', gap: 'normal' }, ['8px']), null)

console.log(failed === 0 ? '\nALL PASS' : `\n${failed} FAILED`)
process.exit(failed === 0 ? 0 : 1)
