---
name: salesforce-user-automation
description: 'Automation on the Salesforce User object. TRIGGER when: building a flow, trigger,
  or Queueable that touches User plus any non-setup object, MIXED_DML_OPERATION, CANNOT_EXECUTE_FLOW_TRIGGER
  on user activation, Chatter group membership DML, a User field that deploys but
  reads as nonexistent, or filtering for real staff accounts. Not for creating or
  de-provisioning users.'
---
# salesforce-user-automation

`User` is a **setup** object and most things you want to touch alongside it are not, so automation on
`User` carries traps that a clean deploy will not catch. Every item here deploys green, passes tests,
and fails somewhere else.

Point-in-time: these were observed on Salesforce orgs during 2026. Verify against your own org before
asserting any of it as current fact.

## Scope

| | |
|---|---|
| **In scope** | Mixed DML on `User` paths (flow and Apex), async-path and chained-Queueable patterns, the testing traps, FLS-less `User` fields, Chatter group membership DML and queries, filtering for real staff accounts |
| **Out of scope** | Creating, reactivating, or de-provisioning users; access errors on a working user; permission set authoring |

## The traps

- **Put the non-setup DML on an ASYNCHRONOUS path (`<pathType>AsyncAfterCommit</pathType>`), not the
  immediate path.** Creating a `CollaborationGroupMember` in the triggering transaction works on
  **insert** but throws `MIXED_DML_OPERATION` on **update**, and it surfaces as
  `CANNOT_EXECUTE_FLOW_TRIGGER` (*"We can't save this record"*), so **activating a user fails
  outright**. The insert-path pass is what makes this dangerous: the flow looks correct until someone
  activates a user. An async path runs after commit in its own transaction, which removes the
  restriction and also means a failure there can never block a user save. A `<label>` on an async
  `scheduledPaths` is **rejected at deploy** (`Label ... cannot be set for ScheduledPath of PathType`):
  `name`, `connector`, and `pathType` only.

- **An async path is not enough, because two DML types in ONE job collide the same way, and
  `allOrNone: false` makes the collision SILENT.** A job that reconciles a setup object (say
  `UserPackageLicense`) and a non-setup object (say `CollaborationGroupMember`) fails the second DML
  with `MIXED_DML_OPERATION`, and under `Database.insert(records, false)` that arrives as a **failed
  `SaveResult`, never a thrown exception**. So nothing throws, a `Finalizer` never retries, and
  `AsyncApexJob` reads `Status=Completed, NumberOfErrors=0` while the work simply did not happen. One
  such defect shipped with 13 green tests, a clean production check-only validation, and 95% coverage.
  **Put the setup DML in its OWN chained Queueable**, and treat any reconcile job touching both a
  setup and a non-setup object as suspect until you have watched it run.

- **A test that dodges the real-world precondition stays green forever.** The tests above passed
  because they ran against a Chatter group name that did not exist, so the reconcile returned early
  and the two DML types never met. **The one combination that occurs in every real org was the one
  never tested**, and the test class documented the dodge as though it were a design choice. Write the
  regression test with the precondition **present**. Apex forbids chaining a queueable from a queueable
  inside a test, so assert the chain by calling the reconcile **directly** and checking
  `Limits.getQueueableJobs()`, rather than by enqueuing it.

- **Testing this in anonymous Apex will lie to you if you update the RUNNING user.** DML on your own
  `User` record is exempt from the mixed-DML restriction, so
  `update [SELECT ... WHERE Id = :UserInfo.getUserId()]` followed by a `CollaborationGroupMember`
  insert **succeeds**, and reads as proof that mixed DML is fine here. Test against a different user.
  The reverse order trips too (`CollaborationGroup` then `User` in one transaction), so split cleanup
  scripts into separate executions.

- **A custom field on `User` with NO FLS grant is invisible to `sf sobject describe` AND uncompilable
  in Apex** (*"Field does not exist"*), even for a System Administrator, while
  `sf org list metadata --metadata-type CustomField` still lists it. So "the deploy succeeded but the
  field does not exist" means **no FLS**, not a failed deploy. This is the mirror of the
  `FieldDefinition`-is-FLS-filtered trap, and it is worth recognizing quickly because the obvious
  reading (redeploy the field) never fixes it.

- **`CollaborationGroupMember.NotificationFrequency` inherits the member's own
  `User.DefaultGroupNotificationFrequency` when you leave it unset**, which is commonly `P` (email on
  every post). An unset insert therefore emails those people on every future post to the group. Set it
  explicitly to `N`. The field describe reports a default of `N`, which is misleading: the describe
  default is not what an unset insert gets. A Chatter announcement's "Email all group members" option
  overrides the setting either way.

- **`IsActive = true AND FederationIdentifier != null` is NOT "only humans".** An integration or
  vendor service account provisioned through SSO holds a Federation Identifier too, and one of those
  inside a "notify all staff" automation is how a service account ends up in a Chatter group or an
  all-staff email. **No single field discriminates reliably, so measure your own org before picking a
  marker**: count how many active users have each of `UserRoleId`, `EmployeeNumber`, `ManagerId`, and
  `Department` populated, and choose whichever your provisioning process actually fills every time. In
  one org measured for this, `UserRoleId != null` was the only marker that covered every staff account
  while excluding the sole service account holding a Federation Identifier; `EmployeeNumber` was
  populated for well under 1% of active users, and several real people shared the System Administrator
  profile, so neither of those worked. Where a role is the marker, clearing a role is also how you take
  a service account out of such automation by hand. **The residual risk runs one way**: a staff user
  provisioned without the marker is silently skipped, which is itself a provisioning defect worth
  catching rather than working around.

## Counting Chatter membership through the `Member` relationship silently drops rows

`WHERE Member.LastName LIKE 'x%'` returned **three** of a group's **four** real rows, and
`SELECT Member.Name ...` under-returned against `COUNT(Id)` on the same group. The misses are
**inactive users**, whom the relationship traversal filters out.

It fails **silently and in the direction that looks like success**: a deactivated member reads as
"already removed", so the query will confirm a removal that never happened. **Query `MemberId` alone
and join to `User` locally.** The same rule applies to any membership existence check whose answer
you are about to act on.
