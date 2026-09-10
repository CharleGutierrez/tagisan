---
name: jira-rest-writes
description: 'Writing to Jira Cloud from the CLI: create, edit, comment, transition, assign, link,
  sprint, Story Points. TRIGGER when: composing a Jira description or comment, assigning
  or @-mentioning, adding to a sprint, setting points, linking a duplicate, or verifying
  a write landed. Every trap here returns a success code while storing the wrong thing.'
---
# jira-rest-writes

Endpoints, recipes, and verification patterns for writing to Jira Cloud over REST. What makes this
worth a skill is that **most of the failures return `200`, `201`, or `204`** while storing something
other than what you meant, so the whole file is organized around reading the write back.

Point-in-time: verified against Jira Cloud during 2026. Confirm anything volatile against your own
site.

## Setup and auth

Read and write through the REST API with `curl --netrc`, authenticated by a classic Atlassian **API
token** in `~/.netrc` for your site's machine. A box without it fails
`curl: (26) .netrc error: no such file`, which is the one thing to check before diagnosing anything
else:

```bash
test -f ~/.netrc && grep -q your-site.atlassian.net ~/.netrc && echo ok
```

**A SCOPED API token must call `https://api.atlassian.com/ex/jira/<cloudId>`, never
`https://<site>.atlassian.net`.** The site URL simply does not serve scoped tokens, and it returns
*inconsistent* status codes (401 on one endpoint, 404 on another), which reads like a credential
problem and is not. A **classic** unscoped token does work against the site URL, which is why a
colleague's own token succeeds on the identical path while yours fails. This one cost weeks to
diagnose. Get your cloudId from `GET https://<site>.atlassian.net/_edge/tenant_info`.

**`acli` is not the route.** It stays installed and authenticated, which is the trap, because it looks
like it works. It omits the assignee from its output, silently does nothing when told to assign, strips
`mention` nodes, flattens prose into a single ADF node, and cannot upload an attachment at all. Every
one of those reports success. Everything it does, REST does verifiably.

## Endpoint map

Base: `S=https://your-site.atlassian.net`, every call `curl --netrc`. Platform API v3 for issues, the
agile API for boards and sprints.

| Operation | Call | Success |
|---|---|---|
| Create | `POST $S/rest/api/3/issue` | `201`, key in `.key` |
| Edit fields | `PUT $S/rest/api/3/issue/<KEY>` | `204` |
| Comment | `POST $S/rest/api/3/issue/<KEY>/comment` | `201` |
| Transition | `POST $S/rest/api/3/issue/<KEY>/transitions` | `204` |
| Assign | `PUT $S/rest/api/3/issue/<KEY>/assignee` | `204` |
| Link | `POST $S/rest/api/3/issueLink` | `201` |
| Attach a file | `POST $S/rest/api/3/issue/<KEY>/attachments` | `200` |
| Read anything | `GET $S/rest/api/3/issue/<KEY>?fields=…` | `200` |

**Every write is confirmed by reading it back, never by the status code.** A `204` means the call was
accepted, and several of the traps below produce a `204` alongside the wrong stored value.

**But a read-back fired a second or two after a transition can return the PRE-automation state.** The
`GET` is truthful and already stale, which is how a transition that an Automation for Jira rule
reverted two seconds later was first reported as a success. Wait about 20 seconds anywhere a rule
might fire, and where you need to know what actually happened rather than what is true now, read the
**changelog** rather than the current status:

```bash
curl -s --netrc "$S/rest/api/3/issue/PROJ-177/changelog" \
  | jq -r '.values[] | .created + "  " + (.items[] | "\(.field): \(.fromString) -> \(.toString)")'
```

## Common operations

```bash
S=https://your-site.atlassian.net

# Create. Build the description as ADF first (see below) — the helper script emits the
# {"fields": …} envelope this endpoint wants, so add the other fields into that same object.
curl -s --netrc -X POST -H "Content-Type: application/json" "$S/rest/api/3/issue" \
  --data @payload.json | jq -r '.key'
# Sub-task: add  "parent":{"key":"PROJ-177"}  and  "issuetype":{"name":"Sub-task"}  to .fields

# Transition. Resolve the id first — names are not accepted, and the ids are per-workflow:
curl -s --netrc "$S/rest/api/3/issue/PROJ-177/transitions" | jq -r '.transitions[]|"\(.id) \(.to.name)"'
curl -s --netrc -X POST -H "Content-Type: application/json" \
  "$S/rest/api/3/issue/PROJ-177/transitions" --data '{"transition":{"id":"31"}}'

# Comment (ADF body)
curl -s --netrc -X POST -H "Content-Type: application/json" \
  "$S/rest/api/3/issue/PROJ-177/comment" --data @payload.json

# Attach a file. The X-Atlassian-Token header is REQUIRED — without it the call is refused as XSRF:
curl -s --netrc -X POST -H "X-Atlassian-Token: no-check" \
  -F "file=@screenshot.png" "$S/rest/api/3/issue/PROJ-177/attachments"

# Confirm exact transition and issue-type names when unsure:
curl -s --netrc "$S/rest/api/3/issue/PROJ-177/transitions" | jq -r '.transitions[].to.name'
curl -s --netrc "$S/rest/api/3/project/PROJ?expand=issueTypes" | jq -r '.issueTypes[]|"\(.name) subtask=\(.subtask)"'
```

---

## Build a description as an ADF document, never as prose

**A description or comment sent as a plain string lands as a SINGLE ADF text node.** Blank lines and
`- ` bullets survive only as literal `\n` characters inside one paragraph, so the item renders as an
unbroken blob under a real person's name. Nothing signals it, because the flattened form is equally
valid ADF.

This bites harder than it sounds, because good practice for anything a system reflows is to write one
continuous line per paragraph with a blank line between, which is exactly the input that flattens.

Use [`jira-text-to-adf.py`](jira-text-to-adf.py), included here. It turns blank-line-separated text into
paragraphs, `- ` runs into a `bulletList`, and numbered runs into an `orderedList`, and it emits the
`{"fields": …}` payload the REST endpoints take, so pass it unmodified:

```bash
# fix a description (PUT answers 204, no body)
python jira-text-to-adf.py body.txt --field description > payload.json
curl -sS --netrc -X PUT -H 'Content-Type: application/json' --data @payload.json \
    "$S/rest/api/3/issue/PROJ-510"

# post a structured comment (POST answers 201)
python jira-text-to-adf.py note.txt --field body > payload.json
curl -sS --netrc -X POST -H 'Content-Type: application/json' --data @payload.json \
    "$S/rest/api/3/issue/PROJ-510/comment"
```

**Confirm by reading the ADF back:**

```bash
curl -sS --netrc "$S/rest/api/3/issue/PROJ-510?fields=description" \
  | jq -r '.fields.description.content | length, ([.[].type] | join(", "))'
```

More than one block, with `bulletList` where you wrote bullets, means it landed. A length of `1` means
you are looking at the flattened form.

---

## The `qm:` dual-account assignment trap

On a site that also runs Jira Service Management, staff often have **two** Atlassian accounts: a JSM
**customer** account (`accountId qm:…`, `accountType customer`, which **cannot be assigned issues**)
and a real Jira **user** account (`accountId 712020:…`, `accountType atlassian`, assignable).

**Assigning or `@`-mentioning by email resolves to the wrong one** and fails with
`User '<qm:…>' cannot be assigned issues`.

Resolve the `atlassian` accountId first, then assign by it:

```bash
S=https://your-site.atlassian.net

# Find the assignable (atlassian) accountId, scoped to the project:
curl -s --netrc "$S/rest/api/3/user/assignable/search?project=PROJ&query=Jordan" \
  | jq -r '.[]|"\(.accountId)\t\(.accountType)\t\(.displayName)"'

# Assign by accountId (204 = accepted):
curl -s --netrc -X PUT -H "Content-Type: application/json" \
  "$S/rest/api/3/issue/PROJ-179/assignee" --data '{"accountId":"712020:…"}'

# Verify: confirm the name AND a fresh `updated` stamp
curl -s --netrc "$S/rest/api/3/issue/PROJ-179?fields=assignee,updated" | jq '.fields'
```

`@`-mentions in a comment are an ADF `mention` node carrying that same `atlassian` accountId. Read the
stored comment back and confirm the node survived, the same way you confirm a description.

---

## Board and sprint operations

```bash
S=https://your-site.atlassian.net
B=<boardId>

# Active sprint for a board:
curl -s --netrc "$S/rest/agile/1.0/board/$B/sprint?state=active" | jq -r '.values[]|"\(.id) \(.name)"'

# Add an issue to a sprint (204 = ok; sub-tasks follow their parent automatically):
curl -s --netrc -X POST -H "Content-Type: application/json" \
  "$S/rest/agile/1.0/sprint/<id>/issue" --data '{"issues":["PROJ-177"]}'

# Set Story Points via the board — works for every issue type, unlike a field PUT:
curl -s --netrc -X PUT -H "Content-Type: application/json" \
  "$S/rest/agile/1.0/issue/PROJ-177/estimation?boardId=$B" --data '{"value":"2"}'

# Which bucket a sprint's issues fall in (velocity counts completedIssues ONLY):
curl -s --netrc "$S/rest/greenhopper/1.0/rapid/charts/sprintreport?rapidViewId=$B&sprintId=<id>" \
  | jq -r '.contents | {completed:.completedIssuesEstimateSum.text,
      elsewhere:.issuesCompletedInAnotherSprintEstimateSum.text}'
```

Find which field your board estimates on with
`GET /rest/agile/1.0/board/<boardId>/configuration | jq .estimation`. The Story Points custom field id
is per-site, so never copy one from documentation.

### Story Points: the field PUT works per issue type, and `/editmeta` lies about it

A plain `PUT /rest/api/3/issue/<key>` naming the Story Points custom field succeeded on a **Story** and
was refused on both a **Bug** and a **Sub-task** with:

```
Field 'customfield_XXXXX' cannot be set. It is not on the appropriate screen, or unknown.
```

**Treat the per-issue-type answer as volatile and confirm it against the issue type in front of you.**
An earlier version of this note asserted the opposite for Story, verified two days before, and whether
the screen configuration changed or the earlier check was mistaken was never established.

**`/editmeta` UNDER-REPORTS this field, so it cannot be used to decide whether the PUT will work.**
This is the real trap, because the reflex on a 400 is to consult editmeta and it gives a confidently
wrong answer in the reassuring direction. Verified across three issues: `GET /rest/api/3/issue/<key>/editmeta`
on a **Story** did not list the field anywhere in `.fields`, and the PUT succeeded anyway. On a **Bug**
editmeta agreed with the refusal, so the two sources match on one type and disagree on another, which
means agreement proves nothing either. This contradicts the general "check `/editmeta` first" advice
that works for other fields.

**The board estimation endpoint works regardless of screen configuration**, so reach for it whenever a
field PUT is refused. It is the only route for a sub-task, though a sub-task appears in no velocity
bucket at all, so where its work needs to count, point the parent.

**Never batch Story Points with other fields in one PUT.** A refused custom field silently discards
everything sent alongside it, so set it in its own call and read it back.

### Velocity cannot be back-dated in either direction

A **completed** sprint refuses any new issue, both routes: `You must specify a sprint which has not
been completed.` from the agile API, and `Issue can be assigned only active or future sprints.` from a
field PUT. Verified against two different closed sprint ids, so it is a platform rule rather than a
workflow quirk.

Moving the work into the **active** sprint does not credit it either: an item that reached Done before
the sprint opened lands in `issuesCompletedInAnotherSprint`, which the velocity chart excludes.
Measured on one sprint, three items carrying 3 points sat in that bucket while `completedIssuesEstimateSum`
stayed at 19. The only mechanism that would move them is reopening and re-closing inside the sprint,
which misdates when the work shipped.

**So raise a missing sprint assignment promptly rather than late — the window is one-way.**

---

## Closing an issue with a specific resolution: the transition cannot carry it

**`resolution` cannot be set through a transition, and batching it into the transition POST rejects the
whole call.** Verified closing a Bug as a duplicate: a transition POST carrying
`{"fields":{"resolution":{"name":"Duplicate"}}}` answered **`HTTP 400`**, `Field 'resolution' cannot be
set. It is not on the appropriate screen, or unknown.`, and **the issue did not move**. That atomic
refusal is the good failure mode, since a partial success would have closed the issue carrying the
wrong resolution.

Check whether any transition on your workflow exposes fields at all before hunting for another one:
`GET /rest/api/3/issue/<key>/transitions?expand=transitions.fields` returned an empty `fields` object
for every transition on the workflow measured.

**The working shape is two calls: transition first, then `resolution` as its own atomic field PUT.**

```bash
S=https://your-site.atlassian.net
# 1. move it
curl -s --netrc -X POST -H "Content-Type: application/json" \
  "$S/rest/api/3/issue/PROJ-695/transitions" --data '{"transition":{"id":"31"}}'
# 2. correct the resolution, on its own
curl -s --netrc -X PUT -H "Content-Type: application/json" \
  "$S/rest/api/3/issue/PROJ-695" --data '{"fields":{"resolution":{"name":"Duplicate"}}}'
```

**A bare transition to Done auto-sets resolution `Done`, so the second call is a CORRECTION rather than
a first write.** Skipping it leaves a plausible-looking `Done` on an issue you meant to close as
`Duplicate` or `Won't Do`, and the transition's own `204` says nothing about it. Read the resolution
back. Your project's values come from `GET /rest/api/3/resolution`.

---

## Linking a duplicate, and reading any link back

**The issue that IS the duplicate goes in `inwardIssue`.** `GET /rest/api/3/issueLinkType` gives
`Duplicate` as `outward="duplicates"` and `inward="is duplicated by"`, and the **inward** issue is the
one that displays the outward description:

```bash
curl -s --netrc -X POST -H "Content-Type: application/json" "$S/rest/api/3/issueLink" \
  --data '{"type":{"name":"Duplicate"},"inwardIssue":{"key":"PROJ-695"},"outwardIssue":{"key":"PROJ-680"}}'
```

which reads `PROJ-695 duplicates PROJ-680` from one side and `PROJ-680 is duplicated by PROJ-695` from
the other.

**Swapping the two answers `201` just the same and records the opposite claim.** Alongside a
`Duplicate` resolution on the closed ticket, that says the *surviving* ticket is the copy, which is
worse than no link at all. Nothing errors either way, so the only way to catch it is to read the link
back from both sides.

**There is no flip operation: fix a reversed link by deleting and recreating it.** Take the `.id` off
the issue's own `issuelinks`, `DELETE /rest/api/3/issueLink/<id>` (answers `204`), then POST the
reverse.

**Reading a link back needs a direction-aware `jq`, and the obvious shape is silently wrong.**
`.type.outward // .type.inward` takes `outward` whether or not this issue is the outward side, so an
**inbound** link renders as its own opposite. In the recorded case that printed `blocks PROJ-693` for a
link that actually said `is blocked by PROJ-693`, and the inverted reading was stated as fact and acted
on, which reversed the order two pieces of work had to happen in. Branch on which key is present:

```bash
curl -s --netrc "$S/rest/api/3/issue/<KEY>?fields=issuelinks" | jq -r '.fields.issuelinks[]
  | if .outwardIssue then "\(.type.outward) \(.outwardIssue.key)"
    else "\(.type.inward) \(.inwardIssue.key)" end'
```
