---
name: salesforce-data-loading
description: 'Bulk data loads in Salesforce. TRIGGER when: designing or running an sf data import/upsert/update,
  building a load CSV, counting rows in a CSV export, loading records owned by a departed
  user, or DUPLICATE_VALUE / INACTIVE_OWNER_OR_USER / LineEnding errors. Not for deleting
  records.'
---
# salesforce-data-loading

The field flags to read before designing a load, the permissions the job needs, the ownership pattern
for departed users, and four traps that report success while doing the wrong thing.

Point-in-time: measured on Salesforce orgs during 2026. Verify against your own org before asserting
any of it as current fact.

---

## Set the audit fields on ANY import of records that already existed

**Default to setting `OwnerId`, `CreatedById`, and `CreatedDate` from the source on every load that
represents something which existed before it**: a migration, a backfill, a re-load of records that did
not land, an import from another system. Not only the departed-owner case below.

Records that carry the loading user and the load date sit visibly apart from their peers, and they
misreport who did the work and when to anyone reading the record or reporting on it.

**Decide it BEFORE the insert, because audit fields cannot be retrofitted.** `CreatedById` and
`CreatedDate` are writable **only on insert**, and only with the **Set Audit Fields upon Record
Creation** user permission. They are `updateable=false` forever after, so the only way to correct them
later is to delete the record and insert it again. `OwnerId` is the exception: it is updateable, so
owner alone can be fixed after the fact.

**Pre-flight, one line, before building the load file:**

```bash
sf sobject describe --sobject <API> --json | jq -r '.result.fields[] | select(.name=="CreatedDate") | "createable=\(.createable) updateable=\(.updateable)"'
```

`createable=false` means the permission is not held and the audit fields will silently default to the
loading user and the load date. **Do not assume the permission is still on because a previous load used
it** — it is commonly enabled for a migration and turned off afterwards, and one production org was
found in exactly that state for a full administrator, months after a migration that had used it.
Granting it is a permission-set change and belongs in the plan, before the load rather than after.

**What it costs to skip:** in the recorded case two records loaded without it carry the loading user
and the load date against 9,862 peers carrying the real ones. Owner was corrected afterwards by update;
the audit fields were not correctable at all.

---

## Assigning records to an inactive user

Some loads need a record **owned by a person who has left**, for instance a migrated agreement whose
requestor is a departed employee. This is an **ownership** problem rather than a writable-field one.

The mechanism worth recognizing: where a display field is a formula that falls back to the record
**Owner** when a lookup is blank, and the load leaves that lookup blank, the only way to preserve the
true historical person is for that now-inactive user to own the record. **Read the fallback direction
precisely, because it also governs the reverse case:** transferring such a record *away* from the
departed owner silently rewrites the displayed person, which is why deprovisioning should not blanket
reassign closed records.

**Two user permissions are required together:**

- **Set Audit Fields upon Record Creation** lets an INSERT set `CreatedById`, `CreatedDate`,
  `LastModifiedById`, and `LastModifiedDate`.
- **Update Records with Inactive Owners** lets `OwnerId` be, or become, an inactive user.

**Prefer the INSERT path.** With both permissions, a single insert can set `OwnerId` directly to an
**inactive** user *and* backdate the audit fields, including `CreatedById` and `LastModifiedById` to
that same inactive user, so "Created By" reads as the real person. No activate-then-deactivate round
trip is needed: create the user inactive up front and set everything at insert. Verified on a load of
9,838 records.

**The UPDATE and transfer path is the one that fails, and its errors mislead:**

- Transferring an existing record to an **already-inactive** user via update returns
  `INACTIVE_OWNER_OR_USER`.
- Transferring to a user that lacks read on the object returns `TRANSFER_REQUIRES_READ`, which is easy
  to miss.
- So handing an *existing* record to a departed person means: create the user **active**, grant it
  object access, set `OwnerId`, then **deactivate**. Ownership persists on inactive users. For a fresh
  load, skip all of that and use the insert path.

Two practical notes when creating placeholder users for this. **Reuse any pre-existing inactive user
for that name rather than duplicating it**, found with a name-only lookup across both active and
inactive. And **watch compound surnames**, since a name like "Austin Paul-Orecchio" can normalize to
"Austin Orecchio" and miss on a later run; alias it.

**Load mechanics.** Hold a bypass permission set granting whatever custom permission your
record-triggered flows check, so the load does not fire a notification burst, then remove it after. Set
Audit Fields does **not** override validation rules, so a rule that requires a field for a given status
still fires; satisfy it in the data.

---

## Do not ask for a value the org can already tell you, and score the answer before loading

**When a load needs a value nobody has decided, look first at what comparable records already carry.**
In the recorded case that answered two of five outstanding groups without anyone ruling on them: 227
records took a value because the same counterparties carry it on 145 of the 159 that resolve elsewhere,
which also retired a standing request for a new picklist value that had already been put to two people.
Comparable usually means same counterparty, same project, or same document type, and the query is a
`GROUP BY` over the records that are already populated.

**Band the answer by how decisive the evidence is, and load only the strong band.** The same technique
on a second group resolved 318 of 390, but the org genuinely used both values, so a single answer for
the group would have been wrong. Splitting by margin loaded 237 where the evidence pointed one way
decisively and held 81. The held rows clustered on four counterparties whose work spans both
categories. **That clustering is the tell that the split is real variation rather than noise**, and a
uniform answer there would be a coin flip wearing evidence.

**Score a candidate rule against the records that already carry the value, and let the number decide.**
A third field went the other way and the scoring is what stopped it: five rules for reconstructing a
cumulative total were each scored against the 355 values production already held, and the best
reproduced 32%. The misses were more useful than the score, showing a constant offset across
consecutive change orders, which says the accumulation was right while the grouping and the starting
figure were wrong. **A rule that reproduces a third of the known-good answers is a rule to abandon, not
to tune.** The honest output was 2,221 records where no chain exists, plus a specific question for the
business.

**Re-measure a stated blocker before you treat it as one.** One field sat recorded as blocked on
records that did not exist, measured when production held 108 of them. It held 1,103 four weeks later
and 788 rows linked on exact name match, so the block had quietly lifted and the note describing it had
not. **A blocker written down weeks ago is a claim with a date on it.**

---

## `sf data ... bulk` needs `--line-ending LF` on Windows

The CLI defaults the *job* to CRLF from the host OS, so an LF data file is rejected outright:

```
ClientInputError : LineEnding is invalid on user data. Current LineEnding setting is CRLF
```

Nothing is applied. It fails loudly, which is the good case; the trap is reading it as a data problem
and editing the CSV. If your generators write pure LF, the flag is needed every time rather than
occasionally.

---

## Blanking a field in a bulk update needs `#N/A`

An empty cell means "leave it alone", and the job reports success either way. This is the quiet twin of
the line-ending trap and the worse one, because nothing in the output distinguishes "wrote your value"
from "ignored your column".

Verified on `ContentVersion`: five rows with empty cells returned `processed=5 ok=5 failed=0` and **all
five still held their old values**; the same five rows with `#N/A` cleared to NULL.

So a success count is not evidence a field was cleared, and **a clearing load has to be confirmed by
reading the records back.**

It cuts the other way too, and that half is useful: an empty cell is how you update one column without
disturbing the rest of the row, so a load file should carry only the columns it intends to write.

---

## Count CSV rows with a parser, never `wc -l`

Long-text fields carry embedded newlines, so a CSV holds more physical lines than records. An
881-record export spanned **1,015 lines**.

Line-counting reports a surplus that does not exist, and it errs in the alarming direction: the count
reads as a defect in the export or the load, so the reflex is to go hunting for rows that were never
there. It bites wherever a row count is the evidence — sizing a load file, reconciling an export
against a `COUNT()`, or confirming what a job processed.

```bash
python -c "import csv,sys; print(sum(1 for _ in csv.reader(open(sys.argv[1], newline='', encoding='utf-8'))) - 1)" <file.csv>
```

Pass the path as `sys.argv[1]` rather than inlining it. A Windows path inside the `-c` string makes
Python read the `\U` of `C:\Users` as a unicode escape.

---

## Read `unique` on the describe before designing any load that repeats a value

Not just `updateable` and `length`. A unique field accepts the first record and rejects every other
with `DUPLICATE_VALUE`, and because a bulk job reports that per record rather than up front, the
constraint surfaces only after thousands of rows have been attempted.

In the recorded case a file backfill checked `createable`, `updateable`, and `length`, read all three as
green, and loaded 18,513 records. **8,604 failed**, because the external-id field was `unique=true`
while 6,412 source documents carry more than one file, one of them 56. **The flag decided the whole
approach and was the one not read.**

**A tracing or grouping field is almost always wrong as Unique.** Repeated values are the point, and
they are what lets the field double as a duplicate indicator. The original design had made both tracing
fields Unique on the assumption of one file per document.

Removing Unique costs the upsert key, since a non-unique external id fails an upsert whenever more than
one record matches, so such loads have to key off `Id` instead. Decide that consciously rather than
discovering it.

---

## If you generate import templates

Two conventions worth keeping whatever generates them.

**Templates are insert-focused, not update-focused.** Do not include the `Id` or "Record ID" column, and
do not add comments framing the template as "leave blank to insert, populate to update". If an update
template is genuinely needed, that is a separate artifact.

**Reference related records by human-friendly Name, not by Id.** Whoever fills the sheet has names, not
18-character keys.

- **Lookups and master-detail:** the column holds the related record's *name*, and the header gets a
  " Name" suffix (for example "Site Name", "Account Name") to signal that match-by-name is the
  mechanism.
- **Record Type:** use the record type *name*, not `RecordTypeId`, with a dropdown of active assignable
  names. Record type names are org-independent metadata, identical between sandbox and production, so
  source them from any org.
- If an Id genuinely must appear and it differs by org, source it from the org that will actually be
  loaded.

Two mechanical notes if you build the generator: filter the field list by what is on the object's
canonical Lightning record page and drop anything an approval or workflow auto-populates, and put
dropdown values on a hidden sheet so they can contain commas and exceed Excel's 255-character
inline-validation limit.
