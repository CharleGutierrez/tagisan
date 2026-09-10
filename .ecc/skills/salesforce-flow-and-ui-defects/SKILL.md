---
name: salesforce-flow-and-ui-defects
description: 'Salesforce Flow and Lightning defects that deploy clean and fail only on screen.
  TRIGGER when: a Lightning page, tab, or component renders blank or empty, a component
  is missing on a phone, a newly deployed field reads as missing, a Flow decision
  never matches, a Flow merge field renders blank, an orchestration errors ''Invalid
  Resource reference'', inline edit silently truncates a value, a required field keeps
  its error after being filled, a record cannot be created because a validation rule
  refused a flow''s own update, or a screen flow needs testing without firing a destructive
  Submit. Not for a deploy that fails or reports a component error.'
---
# salesforce-flow-and-ui-defects

Defects whose metadata is valid, so the deploy has nothing to report and the failure lives in what the
user sees. They are grouped because they share that shape: none of them are reachable from an error
message, and none of them are caught by a check-only validation.

The case that set this collection: three Flow defects shipped a clean 188 of 188 component validation,
threw nothing at runtime, and passed every rule-level check run in anonymous Apex. All three were found
only by opening the screens and using them, and one of them meant no approver could ever approve.

Point-in-time: measured on Salesforce orgs during 2026. Verify against your own org before asserting
any of it as current fact.

---

## Lightning inline edit truncates instead of erroring

`STRING_TOO_LONG` is what the **API** returns when a value exceeds a field's length. The **UI does not
behave that way.** The Lightning input carries the field's `maxlength`, so an over-long paste is cut to
the cap inside the box before anything reaches the server. The save then succeeds. There is no toast,
no field-level error, and nothing in the console.

Measured on a `Url` field: 275 characters pasted, 255 stored, no error.

For a link field this is the worst possible failure, because the value renders as a normal clickable
link and simply does not open. Nothing downstream can distinguish it from a link somebody typed wrong.

A `Url` field cannot be widened out of the problem. It is fixed at 255 and refuses a length, verified
by check-only deploy:

```
Can not specify 'length' for a CustomField of type Url
```

So exceeding 255 characters means changing the **type**, which costs filterability in SOQL, reports,
and list views. What restores a real error is a **validation rule at the cap**:

```
LEN(My_Link_Field__c) >= 1300
```

**Put the threshold at the cap, not above it.** A stored value can never exceed the field length, so a
rule at `length + 1` can never fire, and it is dead metadata that reads like a safeguard. Verified end
to end on a 1300-character Long Text Area: a 1400-character paste is capped at 1300 by the textarea,
the save is then refused with `FIELD_CUSTOM_VALIDATION_EXCEPTION`, and the previously stored value is
left untouched.

One useful side effect of the type change: **Lightning auto-links a URL held in a Long Text Area**,
rendering it as a real anchor with the correct `href`. Clickability survives the conversion, so the
only thing actually given up is filtering, sorting, grouping, and formula use.

---

## An already-open session serves a stale field describe

After a field-type conversion deployed, a browser session that had been open **before** the deploy
still rendered the field as a 255-character `INPUT`, and still capped a paste at 255, exactly as the
pre-conversion field had. A session opened after the deploy reported the truth:

```json
{ "tag": "TEXTAREA", "maxlength": "1300" }
```

This is more dangerous than an obviously broken page. The field is present, correctly labeled, and in
the right position, so a UI check passes visual inspection while measuring the *previous* definition.
The whole point of verifying in the UI is to catch what a green deploy cannot, and a stale session
quietly defeats that.

**A reload is not sufficient.** Open a session created after the deploy: a fresh
`sf org open --url-only` frontdoor into a new browser session. Driving a browser automation tool means
closing the named session and opening a new one, since the frontdoor URL is single-use and the existing
browser keeps the cached metadata.

**The same cache explains a report that a newly deployed field is missing.** Somebody with a tab open
from before the deploy sees the old page. Treat "I can't see the field" as a cache question first,
before chasing access, FLS, page assignment, or the deploy itself. In the recorded case the field was
confirmed rendering in two fresh sessions across both record types while it was still being reported
missing, and a refresh resolved it.

---

## FlexiPage component visibility by custom permission

Gating a Lightning page component on a **custom permission** requires the visibility rule's `leftValue`
to be `{!$Permission.CustomPermission.<ApiName>}` (operator `EQUAL`, `rightValue` `true`). **The
`.CustomPermission.` segment is mandatory.** The shorter `{!$Permission.<ApiName>}`, which is valid for
*standard* user permissions, is wrong for custom permissions, and the deploy rejects it with the
misleading:

```
Field <ApiName> does not exist. Check spelling.
```

The parser reads the name as a missing field rather than as an unknown permission. `{!$User.X}` and
`{!Record.X}` visibility deploy fine with the same criteria shape, so the trap is specific to the
`$Permission` path. Custom-permission visibility is fully deployable once the syntax is right; the
custom permission must already exist in the org.

- **To learn the exact format when unsure:** set the filter once in Lightning App Builder, then
  `sf project retrieve` the page and copy what it generated.
- **Stale-cache red herring:** repeated *failed* deploys of a given FlexiPage DeveloperName keep
  returning a cached "Field X does not exist" error even after the file is corrected. Deploy under a
  **fresh name** to see the true validation state.
- **A page whose components are ALL gated on custom permissions renders BLANK for anyone holding none
  of them.** Only ungated components still show, so the page looks broken rather than empty by design.
  This landed in production on a persona-gated org-default home page where every card was gated on one
  of two persona permissions: **admins are not a persona**, held neither, and fell through to a blank
  main region. A sandbox masked it, because its admin permission set had been hand-edited in Setup to
  grant both and the edit was never captured to source. The fix pattern is a small dedicated permission
  set added to the admin group, rather than hand-editing a large shared permission set, since those
  replace wholesale on deploy. These permissions gate only page visibility, so the grant is cosmetic
  and carries no CRUD or FLS.

### The read-only half of a two-instance field needs `uiBehavior: readonly`

Gating one field on a permission takes two `itemInstances`: the same `fieldItem` twice with opposite
conditions. The pair looks correct in App Builder either way, and **the visibility rule alone does not
make the second instance read-only**. Leaving it at `uiBehavior: none` gives an unauthorized user an
edit pencil, so they change the value and only the **save** refuses them, which reads as a broken field
rather than as a restriction.

In the recorded case one page was hand-built in App Builder in a sandbox and left at `none` there while
the branch source correctly said `readonly`, and a tester hit it inside 20 minutes. Two
consequences: the QA step that checks for the absence of a pencil passes or fails on this one property,
and **deploying the branch is what gets it right**. Check both instances after any hand edit:

```bash
grep -B6 '<identifier>Record.*_cField' <page>.flexipage-meta.xml | grep -A1 uiBehavior
```

### And `readonly` is not a control

The `Edit Read Only Fields` user permission (`PermissionsEditReadonlyFields`) **overrides it**, so the
pencil comes back for everyone who holds that permission, every System Administrator among them.

Measured with the read-only instance correctly set to `readonly`: an admin holding
`EditReadOnlyFields=true` still got a real edit button at every status, while a standard-profile user
with the identical edit FLS on the field correctly got none. Neither held the gating custom permission,
so the gate itself was working and only the read-only half failed. In that org, 11 active users held
the permission through their profile alone.

Three consequences, and the first costs QA time. **A step that reads "confirm there is no edit pencil"
is unpassable for an admin tester and is not evidence of anything.** Assert the refused **save** and its
message instead, which is the guarantee the validation rule actually delivers and which holds for admins
too. **No `uiBehavior` value can fix this**, so do not go back to the page looking for a property you
missed: the page is already correct. And when a read-only field renders editable, **check
`PermissionsEditReadonlyFields` before re-examining the page**, since the page will look right and the
permission is invisible from it:

```bash
sf data query -q "SELECT Profile.Name, Profile.PermissionsEditReadonlyFields FROM User WHERE Username = '<user>'" --json
```

Where the affordance genuinely has to disappear, the only display that survives the permission is one
nobody can edit: a formula field mirroring the value, shown to non-holders in place of the real field.
That costs a field per object, so weigh it against simply letting the rule refuse the save.

---

## A record-page LWC is missing from the phone unless it declares `Small`

An LWC on a Lightning record page renders on desktop and is **absent from the Salesforce mobile app**
unless its `*.js-meta.xml` declares the phone form factor. Nothing errors, the deploy has nothing to
report, and the person on the phone simply does not see the component. The only place the platform says
a word about it is **Lightning App Builder's phone view**, which draws a grey bar over the component:

```
Unsupported form factor
<Component Label> supports only the desktop form factor.
```

So the symptom that reaches you is a person rather than an error. In the recorded case it was an
approver saying they could not cast a vote from their phone.

The declaration lives inside the `targetConfig`:

```xml
<targetConfig targets="lightning__RecordPage">
    <objects>
        <object>My_Object__c</object>
    </objects>
    <supportedFormFactors>
        <supportedFormFactor type="Small"/>
        <supportedFormFactor type="Large"/>
    </supportedFormFactors>
</targetConfig>
```

**List both factors.** Declaring `supportedFormFactors` replaces the default rather than adding to it,
so `Small` on its own takes the component off the desktop page where it is already working, which is a
fix that breaks the half that worked.

Two things this does not fix, both worth checking in the same pass:

- **The record page has to be assigned for the phone form factor too**, which lives on the object rather
  than in the FlexiPage: an `actionOverrides` entry with `<formFactor>Small</formFactor>` naming the
  page. Check it per object rather than assuming.
- **Declared support is not usable support.** App Builder's phone view stops complaining the moment the
  factor is declared, which is not the same as the component being usable at around 360px, so open it on
  a real phone. The banner disappearing is not the verification.

---

## "Related List - Single" is page-layout-dependent, and a missing list renders the tab blank

The standard **Related List - Single** component (`force:relatedListSingleContainer`) shows one related
list, but it sources that list, its presence **and** its columns, from the object's **page layout**:
specifically the legacy layout assigned to the viewing user's profile plus record type. If the related
list is not on the applicable layout, the tab renders **empty at runtime with nothing in the browser
console**, and only Lightning App Builder flags it, with *"this related list isn't on the page layout."*

The tab lives in the **FlexiPage** but its content depends on a **Layout**, which is what makes it easy
to misdiagnose. A managed-package grid override on the component does not exempt it, since the override
only changes how rows render.

- **A blank tab in one org only is usually a page-layout *assignment* difference** (per profile and
  record type), not a package version or FLS. Same-version orgs differ because a different layout is
  assigned. **Confirm which layout the failing user's profile plus record type actually resolves to**
  before checking its related lists; finding the list on a similarly-named layout proves nothing.
- **Two fixes, one durable.** Fragile: add the related list to the assigned page layout, which re-breaks
  on any layout reassignment. Durable and preferred: in App Builder, **Upgrade** the component from
  Related List - Single to **Dynamic Related List - Single** (`lst:dynamicRelatedList`), which is
  configured on the FlexiPage itself and is not layout-dependent.
- **What feeds which.** The FlexiPage defines tabs, regions, and components. Page layouts still feed the
  **Record Detail** fields unless Dynamic Forms is on, the **Related Lists** (all) component, and a
  **Related List - Single**'s list and columns. When a component misbehaves, first decide whether it is
  layout-fed or FlexiPage-fed.

The recorded case broke twice on the same tab: once fixed by patching the layout, and again four days
later in production after a layout reassignment, fixed durably by upgrading to the dynamic component.

---

## `$User.Id` is 15 characters in Flow; a Lookup field stores 18

The worst defect in this file, because it fails totally and silently. A decision comparing a User
lookup field to `{!$User.Id}` **never matches for anyone**, and the flow takes the other branch with no
error.

Proved by making the flow echo both values onto its own screen:

```
DIAG running user    = 005dy00000LrqGg      (15)
DIAG stored approver = 005dy00000LrqGgAAJ   (18)
```

Fix with a Boolean formula, and test the formula in the decision rather than comparing the fields
directly:

```
CASESAFEID({!$User.Id}) = {!varRecord.Some_User_Lookup__c}
```

This is not a `Login As` artifact. `$User.Id` is 15 characters in an ordinary session too, so the same
comparison fails for every real user. **If a decision on a User lookup "just never fires", check the Id
lengths before anything else.** In the recorded case the named approver was told there was nothing for
them to approve, while the stored approver Id and their own user Id were the same person.

---

## A cross-object hop off an automatically-stored Get Records renders blank

`{!GET_Running_User.Manager.Name}` renders as empty text with no error. Automatic output storage gives
you the queried record's **own** fields; traversing a lookup off it does not populate. The tell is a
sentence with a hole in it, for example "This request goes to your manager, , for approval."

Query the related record as its own Get Records, filtered on `GET_Running_User.ManagerId`, and reference
`{!GET_Manager.Name}`. A field that lives directly on the queried record, like `ManagerId` itself, works
fine, which is why a surrounding visibility rule can keep working while only the name is blank.

### The same blank appears spanning `$Record` to an audit user

In a record-triggered flow, spanning from `$Record` to an audit-user relationship for its name renders
**blank** in emails and text templates, even though the underlying Id field is set. The related User is
not hydrated on the triggering record. It hits `{!$Record.LastModifiedBy.Name}`,
`{!$Record.CreatedBy.Name}`, and `{!$Record.Owner.Name}` alike, while custom-lookup spans such as
`{!$Record.My_Parent__r.Name}` resolve fine, which is what makes it look arbitrary.

For "who performed the action" in a record-triggered flow, use the running-user global
`{!$User.FirstName} {!$User.LastName}` instead. The running user is whoever did the DML that fired the
flow. This surfaced on a job-completed notification whose "Completed by" line was empty in every email.

---

## A picklist default applies only to records created after the field exists

A `<default>true</default>` value on a new picklist is applied on **insert**. Every row that already
existed holds `null`, and never the default: 10,194 records against zero holding the default, in the
recorded case.

So a FlexiPage visibility rule or list-view filter written as `= "<default value>"` matches **nothing
that already exists**, which usually means a brand-new button is invisible on every real record while
looking perfect on a freshly created test one.

Key the rule off the states that should *hide* the component instead:

```
Status__c            != Executed
Approval_Status__c   != Pending Approval
Approval_Status__c   != Approved
```

That covers `null`, the default, and whichever states you deliberately want to leave visible.

---

## Make a decision's default path the safe one

A Flow decision sends anything unmatched down the default connector. Pointing that at a real outcome
means an input the component failed to register silently records that outcome.

In the recorded case an approval decision defaulted to `Rejected`, so a click that did not register
would have rejected an approval rather than doing nothing. That was observed live, when a scripted click
set a radio's DOM state without firing the component's change event.

Give every genuine outcome its own explicit rule, and leave the default connector for a screen that
records nothing and says so.

---

## An orchestration's Resource assignee must hold a USERNAME, never a User Id

A Flow Orchestration approval or interactive step whose assignee is set by **Resource** takes a variable
holding the assignee's **Username**. Salesforce Help, "Step Flow Orchestration Resource": *"Resource -
The API name of the variable that contains the username of the user... to assign the step to."*

A variable holding a raw User **Id** deploys clean, passes static analysis and check-only validation,
and fails only when a person submits. The platform throws **`Invalid Resource reference`** on entering
the stage, **no work item is ever created** (`ApprovalWorkItem` and `FlowOrchestrationWorkItem` both
empty), and the record strands with no recall path, because Withdraw Approval Submission only finds
`InProgress` submissions and an `Errored` one is invisible to it.

**The trap is that the wrong value looks right.** The flow error email shows the Resource resolving to a
real, active user, because an Id decodes to one. The value is populated and names the intended person;
it is just the wrong kind of string. That reads as "the assignee is fine, the defect must be in the
stage" and points the diagnosis away from the actual cause.

**Fix: assign the backing variable from the lookup's `User__r.Username`, never `User__c`.** When adding
a new Resource-assignee variable, copy an existing working slot's pattern rather than reaching for the
Id. A related failure in the same family: a variable carrying a hardcoded default of its own literal
name, inert while the orchestration routed by participant count and fatal once each step was gated on
`varUserN Is null = false`. **Whatever string a Resource assignee resolves to must be a Username**, and
both an Id and a stray literal fail the same way at the same point.

---

## A validation rule cannot exempt an after-save flow with `ISCHANGED` on another field

The rule deploys clean, static analysis and check-only validation both pass, and the first person to
create a record loses it.

A rule gating who may change a field carried an arm meant to let an init flow through, and the rule
description said so in as many words:

```
AND(
    ISCHANGED(Approver_Category__c),
    NOT(ISCHANGED(Type__c)),            <-- never true when the flow writes
    OR( ... permission and status arms ... )
)
```

**It never reaches the flow.** The init flows are `RecordAfterSave` flows that issue their **own**
update, and in that update the Type has not changed. So the exemption evaluates false, the rule fires,
and the guard that exists specifically to admit the automation admits nothing.

**The cost is a lost record, not a refused update.** An unhandled fault in an after-save flow rolls back
the transaction that triggered it, so **creating the record fails outright**. The symptom that reaches
you is a `FlowApplication` fault email naming `FIELD_CUSTOM_VALIDATION_EXCEPTION` and the rule's own
message, not a save error on anyone's screen, so the person who created the record may just report that
nothing saved.

**A status window silently excludes BLANK, and two objects sharing a rule fail differently.** Where the
status picklist has **no default value**, a new record's status is blank, blank is in no listed window,
and even a holder of the gating permission is blocked. Where the status defaults to a value inside the
window, the same rule fails on the permission arm alone. **A fix verified on one of two objects proves
nothing about the other**, which is the trap that makes this look half-fixed.

**The exemption that works keys off the field's OWN prior value**, since an init flow's job is the first
stamp:

```
NOT(ISPICKVAL(PRIORVALUE(Approver_Category__c), ""))
```

It deploys, and `PRIORVALUE` on a picklist stays maintainable in the Setup formula editor. What it does
**not** cover is a re-stamp on a record that already holds a value, so a flow that rewrites the field on
a later change is still refused. Where that matters, the discriminator is a field the flow always writes
in the same update and a person never does, which is a behavior decision rather than a mechanical fix.

**Verify both directions without persisting anything.** A savepoint probe proves the insert path and the
protection in one run: `Database.insert(rec, false)`, read the stamped value back, then
`Database.update` on a record past the window, then `Database.rollback`.

**Before shipping a rule on a field any automation writes, find the flows that actually write it**, and
check whether each writes in its own DML. Grep for the assignment rather than the name, since a name
match overstates the exposure. In the recorded case five other flows mentioned the field and only two
wrote it:

```bash
grep -l 'assignToReference>[^<]*Approver_Category__c' force-app/main/default/flows/*.xml
```

One incidental from the same build: a validation rule `<description>` is capped at **255 characters**,
and the deploy refuses the whole component past it with *"Validation rule description cannot be longer
than 255 characters long."*

---

## A required lookup keeps "A value is required" after you fill it

Once a `flowruntime:lookup` marked required has raised its error, **filling the box does not always
clear the message**. The user looks at a box holding a name and a red error at the same time, and it
stays that way until the next Submit click re-runs validation.

**The cause is not established.** A throwaway screen flow (one screen, one required lookup, no formulas,
no pre-filled values, no variables) reproduces the stale message in two separate orgs. But a real submit
screen using the same component **does not** reproduce it: Submit with a required lookup empty raises the
error, and selecting a person clears both the message and the red border. The wording differs between
the two as well, `Complete this field.` against `A value is required`, which points at two validation
paths rather than one component behaving one way everywhere.

**The methodological point is the transferable part.** A bare probe reproducing something proves the
component *can* misbehave. It does not prove every screen using that component does, and it does not
license the conclusion that there is nothing in a subscriber flow to fix. An earlier version of this
entry asserted a cause on exactly that reasoning and was wrong.

Two theories that read as obvious from the flow XML are both dead for the screen that did reproduce it,
which is worth knowing because each costs a session to chase: the `isRequired` flag (the message appears
on components with it both true and false) and the `recordId` binding re-seeding the box (a box that
renders blank has nothing to re-apply, and it keeps the message too).

**Do not "fix" it by removing `required` from the boxes.** That flag stops the submit inline, at the
box, before anything runs. Take it off and the empty box has to be caught afterwards with an error
screen, which is a worse experience than a stale message and a much bigger change.

---

## Testing a screen flow whose Submit is destructive

Two problems: firing the thing you are testing, and measuring a screen that will not hold still.

### Leave one Required box empty as an anchor

Screen-level required validation blocks navigation **client-side, before any flow element runs**. So if
at least one Required field is empty at every Submit click, the flow never advances and nothing is
written. That buys unlimited Submit clicks on a screen whose real Submit deletes records.

Check first that no DML runs before the first screen, rather than assuming it. In the recorded case 147
elements were reachable from `start` before the first screen and every one of them was a read. Confirm
the fixture afterwards by reading the records back, rather than trusting that nothing happened.

### Three measurement traps, each of which produced a wrong answer first

- **Screenshots time out on a large flow modal.** `Page.captureScreenshot` returned a 30-second CDP
  timeout repeatedly while the page was fully responsive to both the accessibility tree and JavaScript.
  A screenshot timeout is not evidence of a hung page.
- **The components sit behind roughly a thousand shadow roots**, so `document.querySelector` finds
  nothing. Walk the tree yourself and read state from the input's `aria-invalid`, not from the rendered
  message text, which is not in a queryable text node.
- **A read fired straight after a click returns the pre-render value.** This is the one that matters,
  because it invents behavior rather than failing. A clear that had worked read back as still populated,
  which looks exactly like a component re-seeding itself, and a read straight after Submit reported
  filled boxes as empty. Both were within a sentence of being reported as product defects. **Settle one
  to two seconds before reading, and re-read before believing anything surprising.**

```javascript
const walk = () => { const a = []; const w = (r, d) => { if (d > 40) return;
    for (const el of r.querySelectorAll('*')) { a.push(el); if (el.shadowRoot) w(el.shadowRoot, d + 1); } };
  w(document, 0); return a; };
walk().filter(e => e.tagName === 'INPUT' && e.getAttribute('role') === 'combobox')
      .map(i => ({ label: i.getAttribute('aria-label'), value: i.value,
                   invalid: i.getAttribute('aria-invalid') === 'true' }));
```
