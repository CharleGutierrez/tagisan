---
name: salesforce-files-queries
description: 'Querying Salesforce Files. TRIGGER when: any ContentVersion or ContentDocumentLink
  query, counting files or building a file population, files that look missing or
  under-counted, or ERROR_HTTP_414 on a long IN list. An unfiltered ContentVersion
  sweep under-returns and its COUNT corroborates the wrong number.'
---
# salesforce-files-queries

Query files through `ContentDocumentLink`, never through a `ContentVersion` sweep. You need this once
you are already querying files, and the failure it prevents is a confidently wrong answer rather than
an error.

Point-in-time: measured on a Salesforce production org in 2026. Verify against your own org before
asserting any of it as current fact.

## An unfiltered sweep under-returns, and its COUNT agrees with the wrong number

`SELECT ... FROM ContentVersion WHERE IsLatest = true`, run with no other filter, **silently returns
fewer rows than the org holds**, and `SELECT COUNT(Id) FROM ContentVersion WHERE IsLatest = true`
returns the same short number.

**The count corroborates nothing.** Both queries travel the same access path, so a matching count
reads as independent confirmation and is not. That is the whole trap: the natural way to sanity-check
the sweep is the one check guaranteed to agree with it.

Measured: the sweep and its `COUNT` agreed exactly, while pulling the same files filtered by the
`ContentDocumentId` values gathered from `ContentDocumentLink` returned **99 additional rows**. Those
99 are real, present, and `IsLatest = true`. A targeted `WHERE ContentDocumentId IN (...)` returns
them; the sweep does not.

## What to do instead

- **Drive file work from the links.** Query `ContentDocumentLink` for the records you care about
  (`WHERE LinkedEntityId IN (...)`), then pull `ContentVersion` filtered by the resulting
  `ContentDocumentId` values. Never take a `ContentVersion` sweep as the population.

- **A gap between the two is not deleted files.** That was the first theory here and it was wrong
  twice: `ContentDocument` returned every one of the supposedly missing ids, including under
  `--all-rows`. In that case the missing rows clustered into a single category of file, so the sweep
  did not just undercount, it made a whole category read as zero. Check what the missing rows have in
  common before theorizing about deletion.

- **Both queries need chunking, and the limit is the URI, not the SOQL.** The CLI sends SOQL as a GET
  parameter, so roughly 1,000 ids in an `IN` list returns `ERROR_HTTP_414: URI Too Long` while 400 is
  comfortable. `sf data query` has no `--bulk` flag in the current CLI; it paginates a plain query
  itself, which is exactly what makes the unfiltered shortcut look attractive in the first place.

## Reading the symptom

If a file count came from a `ContentVersion` sweep, treat it as a floor rather than a total, and
re-derive it from `ContentDocumentLink` before anyone acts on the number. The symptom that should send
you here is a category of file that reads as zero, or a total that is close to right and slightly low.
