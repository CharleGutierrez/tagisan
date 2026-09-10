---
name: qa-handoff
description: 'Creating and handing off a QA sub-task in Jira for Salesforce work. TRIGGER when:
  writing QA test steps, creating a ''QA:'' sub-task, capturing QA screenshots, assigning
  a tester, moving a story or bug to a QA status, or posting a handoff. Not for clearing
  QA, which an independent tester does.'
---
# qa-handoff

Writing a QA sub-task a tester can act on without you, and handing it over without tripping the
automation on the way.

Assumes Jira with `Build:` and `QA:` sub-tasks under a story or bug, and a Salesforce org the tester
signs into. The step-writing craft transfers anywhere; the Jira mechanics are specific to that setup.

**The one rule that is not negotiable: QA is cleared by somebody other than the builder.** Read-only
verification to tee QA up does not count as clearing it, and neither does the person who wrote the
change signing it off.

Point-in-time: measured during 2026. Verify anything volatile against your own instance.

---

## Before you write anything

- **Check whether the QA sub-task already exists.** A Jira automation rule commonly creates `Build:`
  and `QA:` sub-tasks on issue creation. Amend the existing one rather than adding a second, because
  a gate that aggregates every QA sub-task on the key now has two lanes to satisfy.
- **A change to what a story already SHIPPED gets its own work item, never a second QA sub-task on the
  closed story.** Once a story is `Done` it is a historical record. File a **Bug** if the shipped
  behavior is wrong, a **Story** if the requirement changed, link it to the original rather than
  parenting under it, and key the branch off the new item. Attaching the change to a `Done` story
  loses the tester's start signal and leaves a closed ticket describing behavior that is no longer
  live. While the story is still open the opposite applies: amend in place, same key.
- **Confirm the tester can actually log in to the target org.** Having a user there is not having
  access. Query `LastLoginDate` and `Email` alongside the profile check, since a sandbox refresh
  appends `.invalid` to every address and a user can be active, correctly profiled, and locked out.
  If they are, change the org or hand it over rather than editing their account.

---

## Writing the steps

### Summary line

`QA: <what to verify> (<PARENT-KEY>)`

Start with `QA:` so it matches a start-anchored pattern like `^QA\b` if a merge gate reads sub-task
summaries. **Append the parent key rather than prefixing it**, so the anchor still matches, and the
sub-task stays self-identifying in notification emails, on mobile, and in flat search.

### One check per number

**The testing guidance is a numbered list, one check per number.** The numbers are what the tester
records pass, fail, and findings against, so prose paragraphs, bullets, or several checks folded into
one step all leave them nothing to anchor a result to.

Each step is one action plus the result that counts as a pass. Anything that is not a check, such as
setup context or the target-org callout, sits outside the numbering.

### Give the field combination, not fixture record Ids

**This is the single most useful rule here.** Naming specific records looks concrete and is the more
fragile choice: those are live business records, somebody moves them, and the step becomes untestable
with nobody noticing. In the recorded case all three records named in one sub-task drifted within
three days.

State the criteria the component or automation keys off, in page-facing labels, so the tester can find
a current record themselves. Cite a record as a convenience example if you like, but the criteria are
the instruction. Best of all is a fixture the **tester themselves owns**.

The same drift hits your own re-verification. And **a record you cite should still isolate the fix**:
check the other criteria have not shifted to hide the behavior for an unrelated reason, or a passing
result proves nothing.

### Page-facing labels, verified live

Name every object and field by the **label shown on the tester's record page**, not the API name and
not a logical name you assumed. Verify with a live `sf sobject describe`. A label mismatch reads to
the tester as a missing field and produces a false QA gap.

### Name the org, and say so when it is production

**A fix that only exists in production has to be QA'd in production.** A tester who exercises it in a
sandbox gets a **false pass**, because the change is not there.

Put that in the steps as an explicit callout rather than an aside. If your team's default assumption
is a particular QA sandbox, silence reads as "test there."

---

## Exercise every path before you write about it

Click through each new or changed user-facing path end to end: each screen, each branch, as the
persona who will actually use it, with `Login As` for a second persona.

**The screenshot is the byproduct of that run, never the deliverable.** Capturing one proves a field
renders; it does not prove the feature works. In the recorded case a record-page capture read as the
requirement met while neither of the story's two screen flows had been opened, and running them found
three defects past a clean 188-component validation, including that nobody could approve anything.

---

## Screenshots

### Place each one at the step it illustrates

**Attaching is not placing.** A file the tester has to match up against a step themselves looks like
compliance and is not, which is exactly how a gap slips through. Placement puts visual acceptance
criteria at each step.

Producing it is also what puts the builder in the UI, which is the only place a user-visible symptom
is confirmed fixed. One fix was confirmed by SOQL while the tab the user had complained about still
showed the old state; opening the tab is what caught it.

**Jira's REST API cannot embed an image inline**, so placement means a **named callout at the step**,
not a `media` node. Upload with `POST /rest/api/3/issue/<key>/attachments`, then put an ADF `panel`
immediately after the step naming the exact filename and what it shows.

**Consecutive `orderedList` nodes restart at 1**, so a panel splitting your numbered list silently
renumbers everything after it. Set `attrs.order` on each following list and read the numbering back.

### Two captures per changed screen: desktop and phone

Anything rendering on a record page, an app page, or a home page — an LWC, a field on a layout, a
quick action, a screen flow — has to be shown working on a phone as well. Mobile failures deploy
clean, error nowhere, and reach you as a person saying they cannot do something from their phone
rather than as anything you can query.

**When the change genuinely has no mobile surface, write that line into the sub-task** rather than
leaving the capture out, so the tester reads a decision instead of a gap.

### The phone capture needs no phone

Take it from **Lightning App Builder's phone view**, which runs in an ordinary desktop session,
renders the record page at the phone form factor, and names any component the phone will drop. Get the
FlexiPage id from a Tooling query, then open the App Builder URL and switch the preview device:

```
/visualEditor/appBuilder.app?id=<flexipage-id>
```

A desktop-only component draws a grey **Unsupported form factor** bar naming itself, which is how you
catch an LWC that is simply absent from the mobile app before a user reports it.

**Emulating a phone in the browser does not work.** Measured in two orgs: a mobile-emulation session
is redirected to Salesforce Classic from the frontdoor URL, from a Lightning record URL, and from
`/one/one.app` alike, while the identical sequence in a desktop session reaches Lightning. **Why was
not established**, so read it as observed behavior rather than a settled rule and re-test if it
matters.

**A real phone is only needed when the step is about the feature working on the device** rather than
about a component being present: a screen flow completed by touch, an approval actually cast. Then the
tester captures in the mobile app themselves.

### Capture mechanics

Drive a headless browser rather than a logged-in one, because **a frontdoor URL auto-authenticates**,
so it needs no SSO and no MFA handoff:

```bash
FD=$(sf org open --target-org <qa-org> --url-only --json | jq -r '.result.url')
HOST=$(echo "$FD" | sed -E 's#^(https://[^/]+)/.*#\1#')
```

Three things that each cost a wasted capture:

- **The frontdoor URL is single-use.** Open it once to establish the session, then navigate by path.
- **Resize to something like 1600x1200 first**, or a record page's lower half is cut off.
- **Reach Lightning checkboxes and buttons through `getByRole`**, which pierces the shadow DOM, and
  allow 15 to 20 seconds for the page to settle.

**Set up as much as possible through the API so the browser only does what must be seen.** An
error-path capture needs a real save attempt, but the record it acts on can be created by
`sf data create record` in the state that leaves one field to change. Grant the viewing user FLS on
any new field first so it renders, then revert.

---

## Handing it over

**Assign the tester when you create the sub-task. The parent's status is what gates them.** Testers
know not to start a QA sub-task until its parent reaches the QA status, so an assignment on a sub-task
whose parent is still `To Do` is a queue entry rather than a request to begin. Do not hold the
assignment back on the reasoning that the change is not testable yet.

### The two automation rules that pull in opposite directions

Knowing only one of them produces the failure the other was meant to prevent.

- **A rule transitions a parent to `Done` once every sub-task is Done.** On a story whose only
  sub-task is the `Build:` one, finishing the build sends the parent to `Done` with
  `resolution=Done` instead of to QA.
- **A rule reverts a parent out of the QA status while a `Build:` sub-task is not Done.** Measured
  once at two seconds: the parent reached the QA status at 01:44:22 and was put back at 01:44:24.

**So the working order is: create the `QA:` sub-task, then move the `Build:` one to Done, then move
the parent.** Creating the QA sub-task first leaves one open, so the first rule does not fire.
Moving the parent first simply does not stick.

Recovery from a false `Done` does work, since transitioning to the QA status clears the resolution,
but it costs a `Done` that stays in the changelog for good plus a status notification to every
watcher. **Ordering is the whole fix, so do not reach for the recovery.** If the QA sub-task is not
ready to write yet, that is a reason to ask before finishing the build sub-task, not a reason to
proceed and correct afterwards.

### Post the handoff as a comment on the ticket

Scenarios, target org, screenshots, and who should test, as a comment on the story or its QA sub-task,
with an `@`-mention of the tester, so it lives with the work and they are notified where they track
it. Do not compose it as chat text for somebody else to paste.

A mention is an ADF `mention` node carrying the assignable account id, which on a site also running
Service Management is not the one an email lookup returns.

### Verify it landed

Read every write back through REST rather than off a status code. A `204` says the call was accepted,
not that it wrote what you meant, and a flattened description is equally valid ADF.

**Wait about 20 seconds before reading back a transition.** A read fired a second or two after one can
return the pre-automation state: the `GET` is truthful and already stale, which is how a reverted
transition was first reported as a success. Where you need to know what actually happened, prefer the
changelog over the current status.

---

## Drafting is not posting

Assignment, the transition, and the handoff comment all land in a system under a real person's name.
Each is an outward action needing its own go-ahead from whoever is directing the work, and approval of
the **content** is not approval to **post** it.
