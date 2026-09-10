#!/usr/bin/env node
// Tests the exit-code contract of token-drift-diff.js — the exit codes ARE the
// API (CI gates on them). The case that matters most: a MISSING selector must
// never read as FIXED, or you close a bug that is still there.

const { execFileSync } = require('node:child_process')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')

const diff = path.join(__dirname, 'token-drift-diff.js')
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'drift-'))

function entry(id, measured, token, verdict) {
  return {
    id, property: 'p',
    figmaToken: 'primary-500', figmaValue: '#3366cc',
    measured, measuredToken: token, verdict, scope: 'component-level',
  }
}
function snap(label, entries) {
  const f = path.join(dir, label + '.json')
  fs.writeFileSync(f, JSON.stringify({ label, entries }))
  return f
}

const baseDrift = snap('base-drift', [entry('cta.bg', '#3060c0', 'primary-600', 'DRIFT')])
const curSame = snap('cur-same', [entry('cta.bg', '#3060c0', 'primary-600', 'DRIFT')])
const basePass = snap('base-pass', [entry('cta.bg', '#3366cc', 'primary-500', 'PASS')])
const curDrift = snap('cur-drift', [entry('cta.bg', '#3060c0', 'primary-600', 'DRIFT')])
const curPass = snap('cur-pass', [entry('cta.bg', '#3366cc', 'primary-500', 'PASS')])
const curMissing = snap('cur-missing', [entry('cta.bg', '', null, 'MISSING')])

let failed = 0
function run(args) {
  try {
    execFileSync('node', [diff, ...args], { stdio: 'pipe' })
    return 0
  } catch (e) {
    return e.status
  }
}
function check(label, got, want) {
  const ok = got === want
  if (!ok) failed++
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${label} — exit ${got}${ok ? '' : ` (want ${want})`}`)
}

check('DRIFT vs same DRIFT = PERSISTING', run([baseDrift, curSame]), 0)
check('PASS vs DRIFT = NEW (your regression)', run([basePass, curDrift]), 1)
check('DRIFT vs PASS = FIXED', run([baseDrift, curPass]), 0)
check('DRIFT vs MISSING = not a fix, lenient', run([baseDrift, curMissing]), 0)
check('DRIFT vs MISSING under --strict', run(['--strict', baseDrift, curMissing]), 3)
check('no args = usage error', run([]), 64)
check('too many args = usage error', run([baseDrift, curSame, curPass]), 64)
check('unknown flag = usage error', run(['--nope', baseDrift, curSame]), 64)

// The regression that motivates --strict: MISSING must land in SKIPPED, not FIXED.
const out = execFileSync('node', [diff, baseDrift, curMissing], { encoding: 'utf8' })
const missingNotFixed = /NOT a fix/.test(out) && !/✓ FIXED/.test(out)
if (!missingNotFixed) failed++
console.log(`${missingNotFixed ? 'PASS' : 'FAIL'}  MISSING is reported as "NOT a fix", never FIXED`)

fs.rmSync(dir, { recursive: true, force: true })
console.log(failed === 0 ? '\nALL PASS' : `\n${failed} FAILED`)
process.exit(failed === 0 ? 0 : 1)
