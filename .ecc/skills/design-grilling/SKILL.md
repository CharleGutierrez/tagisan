---
name: design-grilling
description: 'Settling a design before building it. TRIGGER when: picking up a story or bug whose
  approach is open, a requirement or design doc leaves choices unstated, a new object,
  field, relationship, or flow is proposed, or a plan rests on something nobody has
  confirmed. Ask the whole frontier in one round, recommend an answer to each, and
  build once the frontier is empty.'
---
# design-grilling

Interview whoever is directing the work until the design is settled, then build. The method is a
**design tree** worked in **rounds**, adapted from Matt Pocock's `grilling` skill
(https://github.com/mattpocock/skills, MIT). What it prevents is the most expensive shape of failure
in configuration-heavy platform work: a build that is internally coherent, carefully specified, and
answering a question nobody ever confirmed.

The examples lean Salesforce because that is where it was hardened, but the method is
platform-agnostic. Substitute your own standing frontier.

## When a round earns its cost

Grill when a wrong answer costs a rebuild: a new object, field, relationship, or flow; an open
approach; a design document; anything reaching into a managed package's territory. For mechanical work
with one obvious shape (a label fix, a description backfill, a picklist value the ticket names
outright), say in one line that you are skipping the interview and get on with it.

**An empty frontier is a valid first result.** When the ticket, the org, and the requirement already
answer everything, say so in one sentence and start building. Manufacturing questions to look thorough
is the mirror image of the failure this skill exists to stop.

## The tree and the frontier

Model the work as a tree of decisions, each one branching into the decisions that depend on it. The
**frontier** is every decision whose prerequisites are already settled, so you can put it without
guessing at an answer you have not heard yet.

Work the frontier in rounds:

1. **Compute the frontier.** A question whose answer depends on another question still open in this
   round belongs to a later round, not this one.
2. **Settle every factual question yourself first**, per the section on facts.
3. **Put the whole frontier in one round**, numbered, each question carrying your recommended answer.
4. **Wait for the answers.** They reshape the tree: settled decisions push the frontier outward and
   unblock what depended on them.
5. **Recompute and repeat** until the frontier is empty.

Where your harness offers a structured multiple-choice prompt, use it for a round of four or fewer
decisions with discrete options, so they render as pickable choices. Past that, write the block:

```
Q1 - <decision title>: <what is being decided, and the options>
     Recommendation: <your answer, and what it rests on>
```

**Lead with a recommendation every time.** A symmetric menu that offloads an easy call is competence
theater: give the strongest-supported answer, label it as inference where it is one, and say what
would change it.

## Fog of war: the unknown you cannot put as a question yet

Not every unknown is a frontier decision. **Fog of war** is something you know is unresolved and
cannot yet state precisely. The test is whether you can state the question now, never whether you can
answer it now. State it, and it is a frontier question or a later-round one. Cannot, and it is fog.
(The term is Matt Pocock's, from his `wayfinder` skill.)

**Carry fog in a named list rather than dropping it or dressing it up as a question.** Forcing a
precise-sounding question out of fog is the failure to avoid, because it draws a confident answer to a
question nobody actually asked. That is how a plausible reading hardens into a requirement and
everything downstream gets built on it. A fog item recorded as fog invites "we do not know that yet"
and costs nothing.

Re-test each item as rounds settle, since settled decisions routinely turn fog into a question you can
put. **The frontier being empty is not the same as the fog being clear**, so say which of the two you
mean before building: an empty frontier with fog outstanding means start, and expect to come back.
Record what is still fogged under a "not yet specified" heading wherever the implementation plan
lives.

## Facts are yours to find; decisions are theirs to make

**Look the facts up rather than spending a round on them.** A question the environment can answer is
never a question for a person. Every question you spend on a lookup is a question you did not get to
ask about something only they know.

- The **org** says what exists: `sf sobject describe`, a SOQL count, `FieldPermissions`,
  `FlowDefinitionView`.
- **git and the PR host** say what shipped and what is in flight, including whether someone else is
  already changing the components you are about to touch.
- The **tracker** says what was actually requested: the ticket text, its sub-tasks, the comments.
- **A prior AI-authored artifact is not a fact.** A design document, an earlier analysis, or a memory
  note is a prior inference, so re-verify it against the ticket, the stakeholder, or the live org
  before a round rests on it. The recorded case: a *recommendation* in an AI-authored design document
  was promoted to a *requirement* one session later, and a confident "conflicting requirements"
  finding was framed on top of it, when the story text had always said something plainer and the
  stakeholder settled it in one line.

**A running lookup is an unsettled prerequisite, not a stop.** Only the questions downstream of it
wait for it. Put the rest of the frontier now.

## A standing frontier for Salesforce work

Certain decisions get assumed rather than asked, over and over. Put every one of these that bears on
the work. Replace the list with your own platform's version as your incidents teach you what it
should hold.

- **What does the managed package, or the existing config, already do here?** Ask this before "what
  field do I need?". The recorded case cost five stories, five independent QA sign-offs, and a merged
  PR building a custom lookup plus a two-way sync, most of it reversed once it emerged the package
  already modeled the relationship and already displayed the records. One column added to an existing
  managed field set replaced a field, three flows, and the sync.
- **Is the cardinality really 1:1?** Where every record has exactly one parent and every parent has
  exactly one of them, the fields belong on the parent. A separate object has to clear a stated
  exception, and "it is a distinct concept" is not one, because a schema models cardinality rather
  than vocabulary. The tell is the compensating machinery: when most of a design's automation exists
  to work around the split, the split is the defect.
- **Which org, and is it production?** Take it from the resolved target plus `Organization.IsSandbox`,
  never from the alias.
- **Who else is editing these components?** Check before any deploy to a shared org, and check every
  path the deploy carries rather than the interesting ones.
- **Who verifies this, and can they log in to that org?** An independent tester, never the builder.
  Check `LastLoginDate` and `Email` in the same breath as the profile, because having a user in an org
  is not the same as being able to sign in to it.
- **What do the org's own labels and values say?** Page-facing labels for anything a tester will
  follow, and the live picklist values for anything a rule, formula, or flow names.
- **What did the requirement literally ask for?** Quote the ticket line. Where it is genuinely silent,
  that is a question for the stakeholder rather than a gap to fill with a plausible guess.

## Where the settled decisions land

Write the agreed approach into wherever your team keeps the live implementation plan, and keep the
discussion that reached it in the comments. If posting it is an outward action under someone's name,
show the draft and get a go-ahead before it goes up.

Then build.
