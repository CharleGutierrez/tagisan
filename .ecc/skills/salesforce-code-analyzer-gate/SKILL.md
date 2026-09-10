---
name: salesforce-code-analyzer-gate
description: 'The Salesforce Code Analyzer check on a pull request, and what to do when it is red.
  TRIGGER when: an analyze job fails or is red on a PR, a finding looks pre-existing
  or unrelated to the change, ApexCRUDViolation appears, analyzer suppressions are
  being edited, a workflow change does not seem to reach an open PR, or a Critical
  UnexpectedEngineError appears running it locally on Windows.'
---
# salesforce-code-analyzer-gate

Running Salesforce Code Analyzer as a PR check: what actually fails the job, why a finding you did not
write shows up on your branch, and the four mechanics that make a red or green result mean something
other than it looks like.

Assumes a workflow that runs the `Recommended` rule set (PMD, ESLint, and Flow, with no org and no
secrets) over the **delta** a PR changes, posts a severity summary, annotates findings inline via
Actions workflow commands, and uploads a results artifact.

Point-in-time: measured during 2026 on Code Analyzer v5 and flow engine 0.38.0. Re-check counts and
engine behavior after a plugin update.

---

## First move when the check is red: diff the config, not the code

```bash
git diff origin/<base-branch> -- code-analyzer.yml
```

Suppressions live in a root `code-analyzer.yml`, and **every branch carries a different vintage of
it**. A branch cut before a justification landed cannot see that justification, so the finding
re-reports and looks like a new defect in the author's own change.

**Check it first because it costs one `git diff` and, when it hits, no code was wrong at all.** That is
the reason rather than a claim about frequency. In the recorded case a PR failed on six findings
already justified on the base branch, purely because the branch predated the commit carrying those
entries. Nothing about the code was wrong and nothing needed suppressing; bringing the branch up to
date was the whole fix.

The same mechanism hits any long-lived branch that periodically re-syncs from the trunk: it re-reports
whatever was justified on the trunk since it last caught up. If you maintain such a branch, carry the
current config into it as a deliberate step.

---

## Should it be a required check?

**The test to apply before making any check required is who trips it and whether they can clear it
themselves.**

One team made it required and took it back off 90 minutes later. In that window it blocked a PR on
six findings that were already reviewed and justified, and that the author had not introduced. She had
no way to diagnose or clear it. The rollout note that already existed, report-only until the
changed-file baseline is clean, was right, and enforcing early is what ignored it.

Two mechanical preconditions if you do require it:

- **A `paths:` filter makes it impossible to require.** A `paths:` filter stops the workflow
  *triggering*, so a PR touching nothing scannable posts no status at all, and a required context that
  never arrives stalls the PR forever. Drop the filter and gate on cost instead (below).
- **Check the workflow and its config exist on every base branch you require it for.** A branch cut
  from a base with minimal CI inherits that minimal CI.

`ENFORCE_SEVERITY_THRESHOLD` as a repo Actions variable makes a finding at or above that severity fail
the job itself, independently of whether the check blocks the merge. Unset means report-only.

### Dropping the `paths:` filter costs almost nothing

Compute the delta **before** any toolchain is installed, and gate the setup steps on whether there are
targets. A PR with nothing scannable then runs a checkout, a `git diff`, and a `grep`: **7 to 14
seconds, green**, against 1m44s for a real scan.

It also buys no extra coverage in practice. In the measured repo, outside the Salesforce source
directory the scannable extensions matched seven tracked files (three jest mocks, a jest config, two
CSS files, and one HTML mockup), and the `.apex` anonymous scripts directory did not match the pattern
at all.

---

## Which rules can actually fail the job

**112 of the 310 `Recommended` rules fail rather than report** (4 Critical, 108 High, counted 2026-08-08):

```bash
sf code-analyzer rules --rule-selector Recommended --view detail
```

Most only reach LWC and JavaScript. Three bite Apex: **`ApexCRUDViolation`**,
`AvoidOldSalesforceApiVersions`, and `AvoidGetHeapSizeInLoop`.

### `ApexCRUDViolation` is the one you will meet

It fires on **any SOQL or DML with no explicit mode**, so essentially every new Apex class that touches
the database trips the gate until it says which mode it wants.

Verified on a brand-new invocable: it failed on that single severity-2 finding, while 10 severity-3
and severity-4 findings in the same two files (ApexDoc, a cyclomatic complexity of 11, and
`ApexUnitTestClassShouldHaveRunAs`) were reported and ignored.

```apex
List<Account> accts = [SELECT Id FROM Account WHERE Name = :n WITH SYSTEM_MODE];
```

**`WITH SYSTEM_MODE` clears it and changes nothing at runtime**, since Apex SOQL already defaults to
system mode for CRUD and FLS. The declaration is documentation the rule can read, which is the point:
it forces the author to state whether the query should respect the running user.

- `WITH USER_MODE` when it genuinely should respect the running user.
- `WITH SYSTEM_MODE` **with a written reason** when it should not. Org configuration read on behalf of
  automation is the usual honest case.
- Reserve an inline suppression for a real false positive rather than reaching for it first.

---

## It scans whole changed files

A pre-existing finding in a file the PR merely touches can turn somebody's unrelated PR red. **Judge a
finding on what the PR actually changed.** Expect this on any edit to an older class.

---

## A green result on an older run is not evidence the code is clean

A **conflicting** PR runs no `pull_request` workflow at all, so its findings surface only once the
conflict is resolved. Compare the head-commit date against the last run before trusting a green result:

```bash
gh pr view <n> --json commits --jq '.commits[-1].committedDate'
gh run list --branch <head> --limit 1 --json createdAt
```

The risk here runs toward believing a stale pass rather than chasing a false failure, which is why it
is worth the two commands.

---

## A landed workflow change reaches an open PR only when GitHub recomputes its merge ref

GitHub does that lazily, so a retry before it happens looks broken. In the recorded case, reopening
three PRs fired every *other* workflow and not this one, twice, which read as a defect. The merge ref
still held the old file, and the same reopen worked first time once it caught up.

```bash
MSYS_NO_PATHCONV=1 git cat-file -p refs/pull/<n>/merge:<path>
```

**Both halves matter.** Without `MSYS_NO_PATHCONV=1`, git-bash mangles `<ref>:<path>` into a Windows
path and reports a missing revision, which is a false *absent* in the reassuring direction. And a
`2>/dev/null` on it makes an unreadable ref look like a clean result. A claim of "not on that branch"
derived from a command whose stderr was discarded has already been written down as fact and been wrong.

---

## Close-and-reopen forces a check onto an open PR without a push

And it does **not** dismiss an approval, where a push would. Verified on a PR whose `APPROVED` review
kept its `submittedAt` across a close and reopen.

`reopened` is a default `pull_request` activity type. `ready_for_review` is **not**, so toggling draft
does nothing.

---

## Running it locally on Windows: the `flow` engine needs `PYTHONUTF8=1`

The vendored scanner's `quick_validate` reads flow files with no explicit encoding, so Python falls
back to the Windows locale codec (cp1252), fails on the UTF-8 bytes in the flow XML, drops every
affected flow, exits with *"No flow files found to scan"*, and writes no results file.

That surfaces as a **Critical `UnexpectedEngineError`** (ENOENT on `flowScannerResultsFile.json`),
never as a flow finding, which is what makes it hard to place.

**It is the locale, not the Python version**, so changing Python does not fix it. CI never hits it,
since `ubuntu-latest` is already UTF-8. Set `PYTHONUTF8=1` in your environment or your agent settings so
every session inherits it. Reproduce the raw error outside the wrapper with:

```bash
cd <plugin>/code-analyzer-flow-engine/FlowScanner && python -m flow_scanner --help
```

The underlying bug is upstream, so re-check after a plugin update.

---

## If GitHub Advanced Security is off, the SARIF upload does nothing

Keep the SARIF-to-code-scanning step `continue-on-error`, because without the license it silently does
nothing. The inline annotations, the summary, and the artifact are the real output and need no license.

Lighting up code scanning needs the **GitHub Code Security** add-on. It is available on the Team plan,
so no Enterprise upgrade is required.

```bash
gh api repos/<owner>/<repo> --jq '.security_and_analysis'
```
