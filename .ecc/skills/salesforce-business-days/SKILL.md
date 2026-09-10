---
name: salesforce-business-days
description: 'Business Hours, holidays, and business-day math in Salesforce. TRIGGER when: a requirement
  says ''after N business days'', deploying or editing BusinessHours settings or the
  holiday list (a deploy REPLACES holidays), adding a company holiday, or working
  out whether an org''s Business Hours are real business hours at all.'
---
# salesforce-business-days

Business Hours look like a solved problem and are not. The default set in a fresh org is frequently
24/7 wearing the name "Default", the holiday-to-set association is invisible to the API, and a deploy
replaces your entire holiday list without saying so.

Point-in-time: measured on Salesforce orgs during 2026. Verify against your own org before asserting
any of it as current fact.

---

## First, measure what the org actually has. You cannot read it off the XML

`00:00 -> 00:00` serializes **identically** for "open 24 hours" and for "closed", so the metadata
cannot tell you which one an org has. Neither can a glance at Setup. Measure it:

```apex
BusinessHours bh = [SELECT Id FROM BusinessHours WHERE IsDefault = true LIMIT 1];
Long ms = BusinessHours.diff(bh.Id,
    Datetime.newInstanceGmt(2026, 8, 3, 7, 0, 0),    // Mon 00:00 in your timezone
    Datetime.newInstanceGmt(2026, 8, 10, 7, 0, 0));  // the following Mon
System.debug(ms / 3600000.0 + ' business hours in a calendar week');
```

- `168` means fully 24/7.
- A Mon-Fri 08:00-17:00 set returns `45`.

**With 168, "5 business days" resolves five *calendar* days out, straight through the weekend**, so the
weekend exclusion everyone assumes they are getting does not exist. Two orgs measured for this each had
exactly one set, named `Default`, at 24/7, with zero `Holiday` records.

**The same measurement is the only way to confirm a holiday is attached.** A record count cannot
distinguish a holiday attached to the set from one that merely exists, because the association is
invisible to the API. Run `diff()` over a week containing a holiday: a nine-hour drop against an
ordinary week is the proof, and 36 against 45 is what a single holiday looks like.

### Check the timezone in the same pass

A set on the wrong timezone is the kind of defect that **appears to fix itself for half the year**. One
org ran `America/Los_Angeles` for a company in Phoenix. Phoenix is UTC-7 all year; Pacific is UTC-7
only during daylight saving. They agree all summer and diverge by an hour from the November changeover.

---

## Holidays attach to a SET, not to the org

That is the whole mechanism, and it is what lets holiday-aware and holiday-blind automation coexist
without either side compromising. Two sets:

| Set | Timezone | Hours | Holidays | Org default |
| --- | --- | --- | --- | --- |
| `Corporate` | your timezone | Mon-Fri 08:00-17:00 | all attached | yes |
| `Ops 24-7` | your timezone | every day, 24h | none | no |

Make the holiday-aware one the **org default**, so anything that does not name a set gets the safe
behavior. Ops-facing automation opts into the 24/7 set explicitly.

Measured with both sets loaded and 24 holidays attached to Corporate only:

| Question | Corporate | Ops 24-7 |
| --- | --- | --- |
| Hours in a normal week | 45 | 168 |
| Hours in Thanksgiving week (2 holidays) | 27 | 168 |
| 5 business days from Wed 2026-11-25 09:00 | Fri 2026-12-04 09:00 | Thu 2026-11-27 06:00 |

The last row is the design working: the same request skips both holidays and two weekends on Corporate,
and runs straight through on Ops.

**The 08:00-17:00 span is an assumption unless somebody has confirmed it.** Every number in that table
scales with it, so make it one named constant rather than a literal spread across formulas.

---

## Holidays live in metadata, even though the API cannot see the association

`Holiday` is data, but it also serializes into the `BusinessHoursSettings` metadata type:

```bash
sf project retrieve start -m "Settings:BusinessHours"
```

That writes `force-app/main/default/settings/BusinessHours.settings-meta.xml`, holding one
`<businessHours>` block per set and one `<holidays>` block per holiday:

```xml
<holidays>
    <activityDate>2026-11-26</activityDate>
    <businessHours>Corporate</businessHours>
    <isRecurring>false</isRecurring>
    <name>Thanksgiving Day</name>
</holidays>
```

The `<businessHours>` association round-trips through deploy and retrieve. **That is worth stating
plainly, because the association is invisible to the API:** there is no junction sObject, no
`BusinessHoursHoliday`, and no `Holiday` child relationship on `BusinessHours`. Metadata is the only
programmatic view of it.

**Generate the file from a plain holiday list rather than maintaining it by hand**, and use the same
list anything else consumes (a scheduler skipping weekday crons, for instance) so the two cannot drift.
A `--check` mode that diffs against the file on disk without writing makes a usable CI gate.

**Prefer explicit dates to recurring ones.** Recurring entries cover indefinitely, which sounds better
until you need an *observed* holiday: a recurring July 4 cannot express `2026-07-03`. One org carried
four hand-made recurring holidays that were not from the company list at all, leaving 20 of 24 real
dates missing.

**A finite list runs out silently.** When the last date passes, every org loses holiday awareness with
no error and no warning. Put the end date somewhere it will be noticed, and extend before it.

---

## Deploy semantics: five traps

1. **Holidays are REPLACED wholesale; business-hours sets MERGE by name.** Asymmetric behavior inside a
   single file. A `<holidays>` entry omitted from the file is **deleted** from the org; a
   `<businessHours>` set omitted from the file is **left alone**. Deploying a partial holiday list
   silently removes the rest.
2. **Every deploy deletes and recreates the `Holiday` records**, with new Ids each time, even when
   nothing changed. Never reference a `Holiday` Id from anything.
3. **Renaming a set via metadata creates a NEW set** rather than renaming the existing one, and leaves
   the original behind, demoted if it held the default flag. Worse, **you cannot clean up the orphan
   programmatically**: `BusinessHours` reports `deletable=false` and the API answers
   `entity type cannot be deleted`, so removing it is a Setup-UI-only job. Rename in the Setup UI
   *first*, then deploy. Getting this wrong leaves a permanent extra row until someone clicks Del.
4. **Omitting a day's start and end elements means CLOSED**, while `00:00 -> 00:00` means open 24
   hours. Both look like "nothing set" at a glance.
5. **A holiday attached to business hours cannot be deleted through the API or Apex.** The error is
   `DELETE_FAILED: The holiday you are attempting to delete is in use by business hours.` A metadata
   deploy that omits the holiday is the only programmatic route, which is trap 1 being useful for once.

A note on stock sets: the `Default` set usually still exists after you introduce your own, demoted and
otherwise untouched, because sets merge by name and your file never mentions it.

---

## Who owns the list: git or Setup, never both

Trap 1 makes this a real decision rather than a preference. Once `BusinessHours.settings-meta.xml` is
tracked, **any deploy replaces the org's holidays with whatever that file says**, including a deploy
from a branch cut before somebody added a holiday in Setup. It fails silently, because a wholesale
replace is a successful deploy.

The recommended direction is **git to Salesforce, one-directional**, generated from a list. The reverse
is technically possible (retrieve, then convert back) but inverts the risk, since Setup edits become
the source and any stale branch overwrites them.

Until the settings file is actually tracked, Setup is the de facto owner and editing holidays there is
safe. Switch deliberately: retrieve the org's version into source, confirm it matches your generator's
output, then commit.

---

## Business-day math in Flow and Apex

**Flow has no native business-hours math, but "5 business days" does NOT need Apex.** `TODAY() + 7` is
exact, because five business days spans one calendar week from any weekday start. A plain Date formula
matches `BusinessHours.add()` without touching hours at all: both return `2026-08-10` from Monday
`2026-08-03`. **Reach for the formula first.**

**Holidays are the gap.** Each one falling inside the span pushes the true date a day later, and the
formula cannot see them.

**For an arbitrary N, or to be holiday-aware, you do need Apex:** an invocable wrapping
`BusinessHours.add()`, which takes elapsed business **milliseconds**, not days.

**The multiplier is the length of your business day.** A continuous nine-hour day with no lunch break
makes "5 business days" `5 * 9 * 3600 * 1000`. Not `5 * 8h`, which lands a day early, and not `5 * 24h`.
Since it changes with the configured hours, name the constant rather than inlining the literal.

---

## Before you promise holidays will affect scheduling, check what consumes Business Hours

Standard `Holiday` and `BusinessHours` feed Cases, Entitlements, Milestones, and anything that calls
`BusinessHours.diff()` or `.add()` itself. They do **not** automatically feed every scheduling feature
in the org.

Managed packages commonly carry their **own** parallel calendar model, with their own working-hours and
non-working-hours rule objects, unrelated to the standard ones. In one measured package the field
scheduling objects held raw start and end datetimes with no Business Hours reference at all, while a
separate packaged calendar model, holding zero records, fed only SLA clocks and timesheets.

So the question "will adding holidays disturb field scheduling?" is answered by looking at what the
scheduling objects actually reference, not by assuming the platform wires it up. Confirm before either
promising or denying it.
