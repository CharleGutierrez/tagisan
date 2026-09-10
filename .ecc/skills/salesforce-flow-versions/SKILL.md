---
name: salesforce-flow-versions
description: 'Salesforce Flow VERSIONS and activation: which version an org is really running,
  and why a deployed flow is not a live one. TRIGGER when: a flow deployed but its
  fix is not live, a version reads InvalidDraft, activating or deactivating a flow,
  deciding whether a flow is dead, deleting a flow or a field a flow references, comparing
  an org''s flow against source, an org behaving like an older version of a flow,
  or a running orchestration keeping stages that no longer exist. Not for a flow defect
  that renders wrong on screen.'
---
# salesforce-flow-versions

Which version of a flow an org is actually running, and why almost nothing you can read from source or
from a deploy result answers that.

**The shape every item here shares: a version is a separate object from the flow, and the tools all
report on a different one than you assume.** Source reports intent. A deploy result reports the
component. A retrieve reports the **latest** version. `FlowDefinitionView.IsActive` reports the
**active** one. Reading any of them as the others is where the wrong answers come from.

Point-in-time: measured on Salesforce orgs during 2026, API v66.0. Verify against your own org before
asserting any of it as current fact.

---

## The queries worth having before anything else

```sql
-- every version of a flow, with its status  (Tooling API)
SELECT Id, VersionNumber, Status, CreatedDate FROM Flow
  WHERE Definition.DeveloperName = '<ApiName>' ORDER BY VersionNumber DESC

-- which version is ACTIVE  (standard API)
SELECT ApiName, ActiveVersionId, IsActive FROM FlowDefinitionView WHERE ApiName = '<ApiName>'
```

```bash
# the content of ONE specific version (Metadata is not queryable through SOQL)
sf api request rest "services/data/v66.0/tooling/sobjects/Flow/<301...>"
```

Three gotchas on those:

- **`FlowDefinitionView` is not supported with `--use-tooling-api`** and answers
  `INVALID_TYPE: sObject type 'FlowDefinitionView' is not supported`. Use the standard API.
- **A flow absent from the org returns no row rather than a null**, so compare the row count against
  the names you asked for, or an absence reads as an inactive flow.
- **The Tooling `Flow.Metadata` field is retrievable one row at a time only** (`WHERE Id = '...'`). An
  `IN (...)` errors with *"must specify no more than one row"*.

---

## A deploy to production lands every flow inactive

Whatever tool deployed it. This is not a quirk of one pipeline: it holds for a managed deploy tool with
no activate-on-deploy setting at all, and for a direct `sf project deploy quick` run by a human
administrator, including flows whose source said `Active`.

**So every release needs a post-release activation step**, and the safe pipeline shape is
deploy-inactive-then-activate. Production also requires at least 75% flow-test coverage to deploy an
*active* record-triggered, scheduled, or autolaunched flow, which is a second reason to ship inactive.

**Activate an already-deployed version without creating a new one** by deploying a `FlowDefinition`
file carrying `<activeVersionNumber>N</activeVersionNumber>`. A CI service account often cannot do this
(activation needs View All Data, which a deploy-only account should not have), so it is usually an
administrator's step.

### Do NOT blindly bulk-activate everything inactive

Cross-check each flow's **source** `<status>` first, because it encodes intent that the org's inactive
state does not. One naive "activate all 40 latest-inactive" wrongly turned on three flows and had to be
reverted.

| Source status and type | Meaning | Action |
|---|---|---|
| `Active` + record-triggered | Meant to be live | Activate |
| `Draft` + record-triggered or autolaunched | Staged, not ready | Do **not** activate |
| `Draft` + scheduled, or a screen flow needing per-org setup | Shipped Draft by convention, because the CI account cannot activate it | Activate per org |
| `Obsolete` | Retired | Never activate |

**Also exclude flows that are inactive because somebody deliberately turned them off.** Search your
tracker for the flow name to catch an open bug, and treat a first-time activation whose production
history already contains an `Obsolete` version as a prior deliberate off-switch rather than a missed
deploy.

**Watch what a batch activation actually starts.** A scheduled flow begins running the moment it is
activated.

**A large production deploy can outrun a two-minute tool timeout.** The local `sf` is killed while the
server-side deploy keeps going. Recover with `sf project deploy report --use-most-recent --json`, then
`sf project deploy resume --job-id <id> --wait N`. `resume` takes no `-o` flag, since it uses the
cached job's org. Never re-submit, or you double-deploy.

---

## A flow's source `<status>` does not indicate whether it is live

**`<status>Draft</status>` in a tracked flow file is not evidence the flow is dead.** Measured against
one production org: of the nine flows whose source said Draft or Obsolete, **six were ACTIVE in
production**.

| Source status | Production state | Count |
|---|---|---|
| Draft | **ACTIVE** | **6** |
| Draft | inactive | 2 |
| Obsolete | absent from every org | 1 |

**Why:** a production deploy lands every flow inactive, an administrator activates it in the org
afterwards, and the source status is never reconciled back. So source drifts to Draft permanently while
the flow runs. Scheduled and always-system-context flows are additionally shipped Draft on purpose.

**Before retiring, deleting, or counting "dead" flows, check the org, not the file.** In the recorded
case a tech-debt document had listed the source-Draft flows as retirement candidates, and acting on it
would have deleted working automation.

---

## After a release, compare latest-against-active for every flow in the payload

**An activation checker that flags only flows whose source ships `<status>Active</status>` cannot
detect the most common missed activation, and it reports the org CLEAN while it happens.** Most flows
sit at `Draft` in source for the reason above, so they are invisible to such a check by construction.

Measured the day after one release: the payload carried **16** flows, the checker answered *"all
source-Active flows are active"*, and **4 of the 16** were still running their previous version. Two of
those backed items that had already been announced to the whole company that morning, so the release
notes described behavior that was not live.

**`FlowDefinitionView.IsActive` will not answer this.** It reports on whichever version is active, so
it reads `true` while the newly deployed version sits Draft. Take the `Flow` members straight out of the
release manifest and compare per name:

```bash
sf data query --use-tooling-api -q "SELECT Definition.DeveloperName, VersionNumber, Status FROM Flow WHERE Definition.DeveloperName IN (...) ORDER BY Definition.DeveloperName, VersionNumber"
```

Then group by name and assert `max(VersionNumber).Status == 'Active'`.

**Diff two flow versions with the Tooling API, not a retrieve**, since a retrieve returns only the
latest. Query `SELECT Metadata FROM Flow WHERE Id = '<301...>'` one row at a time, compare the `name`
sets per element collection (`assignments`, `decisions`, `formulas`, `recordLookups`) to see structural
change, then compare same-named elements field by field to see repoints. That is what distinguished a
two-step subflow repoint, safe to activate immediately, from a full routing rewrite that was being held
back deliberately.

---

## Deactivation: a Draft deploy is not an off-switch

**Deploying the Flow with `<status>Draft</status>` does NOT deactivate an already-active version.** It
adds a new inactive version while the active one keeps running and firing.

To actually deactivate, deploy a `FlowDefinition` file with `<activeVersionNumber>0</activeVersionNumber>`
at `force-app/main/default/flowDefinitions/<Name>.flowDefinition-meta.xml`. The artifact does not need
to be committed: build it in a scratch directory and deploy that directory.

A *fresh* deploy of `status=Draft` to an org that lacks the flow does land it inactive at v1, which is
fine for shipping inactive to production. The two cases look identical in source and are not.

**`FlowDefinitionView` LAGS after the deploy, so do not use it to verify.** `IsActive` and
`ActiveVersionId` can keep showing the flow active, with the old `ActiveVersionId`, for several minutes
after a deactivation deploy has already reported Succeeded. Verify against the Tooling
`FlowDefinition.ActiveVersionId` (null means inactive) or per-version `Flow.Status`. Those go fresh
immediately.

**Once a flow has been deactivated, a later content deploy of Active-status source lands INACTIVE.**
Reactivation always needs its own explicit `FlowDefinition` step; the deploy never restores it for you.

**A flow deploy reporting `changed=false` can still have created a real new active version.** Never
trust the deploy's `changed` flag for flows. Query the versions.

---

## `InvalidDraft`: three routes to a green deploy that cannot be activated

`InvalidDraft` is distinct from `Draft` and shows only in a Tooling query on `Flow.Status`. Attempting
activation gives *"You can't activate version N of this flow because it has errors"*, which names
nothing. **So check the version's Status after deploying a flow, rather than reading the deploy
result.**

**Isolate any `InvalidDraft` the same way: deploy the UNMODIFIED file first and see which status it
lands in.** That separates "my edit broke it" from "this flow was already invalid".

### 1. The target org lacks a field the flow reads

**Salesforce does not fail the component.** It silently rewrites the unresolved merge reference to the
literal placeholder **`null__NotFound`** and marks the new version `InvalidDraft`. The deploy reports
the flow in `componentSuccesses`, so nothing in the release output hints at it.

Live case: a flow deployed in a 105-component, zero-error release and landed `InvalidDraft`. The org's
stored copy read `<assignToReference>varSuggested.null__NotFound</assignToReference>` where source read
`varSuggested.Environmental_Lead__c`, and the same for a second field. Root cause was an incomplete
capture: the flow shipped alongside 14 discipline configuration records against only 11 matching lead
fields, and two of the fields existed in no org and in no source.

**Diagnose by retrieving the org's stored flow and diffing it against source.**

```bash
sf project retrieve start -m "Flow:<Name>" --output-dir <dir>
```

The `null__NotFound` tokens name the broken references directly. **This is the only step that gives a
straight answer.** Extracting field references from the source XML and checking each against the org
found nothing, because the references that mattered were on a **flow variable** rather than inside a
`recordLookups` or `recordUpdates` block or a `__r` traversal, so a scan of those element types misses
them entirely.

**Do not conclude the flow is fine elsewhere because another org reports Active.** One org read
`IsActive=true` while its latest version was `InvalidDraft` with byte-identical `null__NotFound`
content; it was simply still running an older version.

### 2. A subflow's new input variable, deployed in the same payload

One deploy carried a subflow gaining a new input and an orchestration passing that input to it. The
deploy reported `status: Succeeded`, `numberComponentErrors: 0`, every component in
`componentSuccesses`, and left the orchestration `InvalidDraft`. Re-deploying **only** the
orchestration, with the subflow already active, turned that same version into plain `Draft`, which then
activated normally. No new version was created by the second pass; a Draft is overwritten in place.

**The compile-order explanation is inference, not proven.** The only thing that changed between the two
deploys was that the subflow's new input already existed; it was not isolated further.

It is the same shape as other two-pass bootstraps: a payload cannot always resolve a reference to
something else in its own payload.

### 3. `$GlobalConstant.EmptyString` assigned to a Long Text Area

Clearing a Long Text Area field that way compiled to `InvalidDraft`; an empty
`<stringValue></stringValue>` compiled to `Draft` and activated.

More generally, **clear a field in a Flow record update with an empty `<stringValue></stringValue>`,
not `$GlobalConstant.EmptyString`.** The `<elementReference>$GlobalConstant.EmptyString</elementReference>`
form passes the metadata **deploy** clean but **fails activation** with
`field integrity exception: unknown (The assignment that uses the field "X" has an invalid reference in
"$GlobalConstant.EmptyString".)`. The global constant is fine on ordinary Text, so the Long Text Area
case reads as a type limit rather than a general rule, and it is not proven beyond the one field.

---

## The retrieve-returns-latest trap defeats every source-versus-org comparison tool

Because a retrieve returns the **latest** version, **any tool that compares a retrieve against source
is comparing the wrong version whenever the org's active version is not its latest.** A
drift-detection script is exactly such a tool, and it misfiles the case in the reassuring direction:
the flow reads as "diverged, the org's own content, not actionable", when the truth is that the org is
running a version predating a merged change.

Live case: a notification flow was Active at **v6**, which read a retired custom-metadata field and did
not reference its replacement at all. **v7 carried the repoint and was Obsolete.** A retrieve returned
v7, so a source-versus-org diff reported the flow as carrying the repoint. Reading v6 directly settles
it, with the single-version REST call above, then grep the field names.

**And v6 was active because a person deliberately put it back**, so it was not drift to fix. The audit
trail showed a colleague in Legal creating v7, activating it, then reactivating v6 and deactivating v7
two hours later. Activating v7 to "fix" the divergence would have reversed their call.

Since `Section` cannot be filtered on `SetupAuditTrail`, select `CreatedDate, CreatedBy.Username,
Action, Section, Display` over a time window and grep the `Display` text.

**It also blocks field deletion, twice over.** Deleting a field needs every referencing flow version
gone, obsolete ones included, and **`MetadataComponentDependency` reports zero dependents because it
only sees current versions**. So a dependency query returning nothing is NOT evidence a field is safe
to delete. The destructive validate is what names the blocking versions, and its error text is
truncated by Salesforce mid-sentence, so it under-reports how many there are.

---

## Deleting a flow

**Check EVERY org first, because "inactive in production" is not "dead".** Verified the hard way: a
tech-debt plan listed three flows as retirement candidates on the strength of a production-only check,
and one of them turned out **active in three other orgs** while inactive in production. Deleting it
would have removed working automation from all three.

This is the mirror of the source-status section: source status lies in one direction, a single org's
status lies in the other. One query per org settles it, then enumerate versions per org with the
Tooling `Flow` object before deleting anything. Also check whether an open PR touches the flow, which
on its own disqualifies it as a clean deletion.

**Delete the `Flow` versions via the Tooling API, not via MDAPI `destructiveChanges.xml`.** A
destructive deploy naming the flow fails check-only with **`insufficient access rights on cross-reference
id`**. Nothing in that message points at flows, versions, or what the cross-reference is, and in the
measured case the flow was neither active, managed, a template, nor holding any `FlowInterview`. **Do
not chase permissions or references on it.**

```bash
sf data delete record --use-tooling-api --sobject Flow --record-id <301...>
```

**Deleting the last version removes the `FlowDefinition` automatically.** Confirmed at scale: after
removing nine versions across five orgs, a `FlowDefinition` query returned zero rows in every org. There
is no orphan cleanup step.

Recipe: query the versions, deactivate first if any is Active (remembering that a `status=Draft` deploy
is not deactivation), then Tooling-delete each version.

**On the pre-delete safety check:** `MetadataComponentDependency` **does** carry
`MetadataComponentName` and `RefMetadataComponentName`, and selecting both works. What they are not is
**filterable**. So filter on `RefMetadataComponentId`, using the `301…` version Ids, and select the
names for readability. Remember the limitation above: it sees current versions only.

**A sandbox-sync tool listing flows to delete is reporting the source-side delta, not the org side.**
In one case three flows were listed and only one existed in the sandbox; the others had never been
deployed there. Confirm each exists before reporting it deleted.

---

## Apex `Flow.Interview` can run a later Draft, not the active version, and drop its input silently

Measured while diagnosing five failing tests. The flow held **v2 Active**, carrying the input variable
the test passed, and **v3 Draft**, an older design without that variable.
`Flow.Interview.createInterview('<Name>', {recordVar => rec}).start()` ran **v3**, confirmed from the
debug log's `FLOW_CREATE_INTERVIEW_BEGIN` naming v3's Id while `FlowDefinitionView.ActiveVersionId`
named v2.

**The failure mode is silence.** v3 had no `recordVar`, so the input went nowhere: no exception, no
`FLOW_VALUE_ASSIGNMENT` line at all, the criteria check read a null field, took the default connector,
and ended. Zero DML, zero errors. **A `try/catch` around the interview counts no failure**, so batch
code that tallies exceptions reports a clean run while doing nothing.

**How the shadowing Draft got there, and this is the reusable part:** the trunk still carried the older
version of the flow, and a routine "sync my sandbox from the trunk" deployed it **on top of** the
branch's v2, landing as a later Draft. So **syncing a sandbox can shadow an unmerged branch's active
flow**, and every Apex test driving that flow starts failing for a reason that is nowhere in the branch.
The same branch validated with zero failures in a CI org that had no shadowing Draft.

**Why Apex chose the Draft is NOT established.** The `Flow.Interview` documentation states no
version-selection rule. The running user did hold **Manage Flow**
(`PermissionsManageInteraction`), which is the usual explanation offered for latest-over-active, but
that link was not verified. Treat it as an untested hypothesis.

**How to apply:** when an Apex test that launches a flow fails with everything reading null, or a
criteria check is inexplicably false, enumerate the flow's versions before touching the Apex and
compare the Id in `FLOW_CREATE_INTERVIEW_BEGIN` against `FlowDefinitionView.ActiveVersionId`. Flow
events need a DebugLevel with `Workflow=FINEST`; the org default omits them entirely, so the first log
looks as though the flow never ran.

---

## A running orchestration keeps the stage structure of the version it started on

Verified on six `FlowOrchestrationInstance` rows with `Status = InProgress`, sitting in stages that a
redesign had deleted months earlier. **Deploying and activating a new orchestration version does not
change anything already in flight.**

**The QA consequence is a mixed state that reads as a new defect.** A record-triggered before-save flow
is version-independent and applies to every new record immediately, while an orchestration change
reaches only rounds started afterwards. A tester's parked records will therefore show the fixed
behavior from one half alongside the old behavior from the other.

**Tell a tester to start a fresh submission after an orchestration fix, and do not let them resume a
parked record.**

`FlowOrchestrationInstance.FlowDefinitionVersionName` is null on every row, so it cannot tell you which
version an instance is on. `CurrentStage` against the current design is the usable signal.

---

## Two deploy-time gotchas worth knowing

**`emailSimple`**, the core Send Email action, requires
`<flowTransactionModel>CurrentTransaction</flowTransactionModel>`. `Automatic` fails the deploy with
*"The action 'EMAILSIMPLE' only supports 'CurrentTransaction' in 'FlowTransactionModel'"*.

**A record-triggered flow's entry-criteria FORMULA cannot reference a polymorphic reference field.**
`ISCHANGED({!$Record.AssignedToId})` on an object whose `AssignedToId` points to Group **or** User fails
check-only deploy with the misleading *"Field AssignedToId does not exist. Check spelling."* Use a
**structured** entry condition (`<filters>` with `IsNull` / `EqualTo`), which accepts the polymorphic
field. The consequence is that "run only when the assignee changes" is **not expressible** for a
polymorphic assignee; the closest is "assignee is set". Applies to `OwnerId`, `WhoId`, `WhatId`,
`RelatedRecordId`, and `AssignedToId`.
