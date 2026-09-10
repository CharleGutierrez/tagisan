---
name: sf-session-handoff
description: 'Carrying a multi-session task across sessions. TRIGGER when: picking up work in an
  existing task folder, resuming something a previous session started, wrapping up
  with work unfinished, or the conversation has been summarized and the work is still
  going. Read the handoff before starting; write one before stopping.'
---
# session-handoff

A **handoff** is one dated Markdown file per task that lets a fresh session continue the work without
rediscovering it. Read one before you start; write one before you stop.

Adapted from Matt Pocock's `handoff` skill (https://github.com/mattpocock/skills, MIT), with the
storage location changed. His saves to the OS temporary directory; here the file has to survive,
because the value shows up days later.

## Two reasons this pays, and the second is the one nobody feels

- **A new session re-solves what the last one settled.** Recorded case: a session rebuilt a
  parent-record derivation from scratch, reached 91.7% accuracy, and called the remainder unsafe to
  automate, while a script in the same task folder already sat at 98.8% with the decision encoded in
  its docstring. It had been solved before, in that folder, by an earlier session.

- **A long session costs many times what a fresh one does.** Re-read context is billed every turn, so
  cost compounds with position. Measured across 309 transcripts of one team's usage in 2026: a request
  at turns 201-400 re-read 619K tokens against 24.8K at turn 1, and **47.7% of all spend sat past turn
  100**. A handoff plus a genuinely new session resets that baseline. Letting the conversation be
  summarized instead keeps you in the same session, so the compounding continues.

## Before you start: read, then work

When work touches an existing task folder:

1. **List the folder.** Read the newest `HANDOFF-*.md` in full.
2. **Read the docstring of every script whose name is near your problem.** The handoff is the map; the
   scripts are the answers, and their docstrings carry the decisions and the measured accuracy.
3. **Only then start building.** A derivation, classifier, load, or query you are about to write from
   scratch is the exact thing that folder most likely already holds.

## Before you stop: write one

Write `<task-folder>/HANDOFF-<YYYY-MM-DD>.md`. A new dated file each time, never an edit of
yesterday's, so the sequence stays readable.

Four sections, and the last two are what make it worth reading:

```
# <task> - handoff, <date>

## State
What is true now, with each figure marked as verified live against a named environment and date.
The branch, the working tree path, the PR number, and the target environment.

## Open
The decisions still unsettled and who owns each. When nothing is open, say NOTHING outright:
an explicit empty list is information, and a missing section reads as an omission.

## Traps this task has already paid for (do not rediscover)
Each wrong turn already taken, and what it cost. This is the section that saves the next session.

## Next
The first thing to do, specific enough to act on without re-deriving why.
```

**Reference rather than restate.** Anything already carried by the branch, the PR, the tracker item,
the project instructions, or a script in the folder gets a path or a number, not a copy. A handoff
that restates the diff goes stale the moment either changes, and length is what stops it being read.

**Redact.** No credentials, tokens, or personal data.

**Say when a figure was verified and against which environment.** A number carried forward with no
date turns into a fact nobody can check, which is how a stale count reaches a stakeholder as current.

## Name a handoff to a PERSON with its task folder, never the bare filename

Say `<task-folder>/HANDOFF-2026-08-20.md`, or link the full path, and never `HANDOFF-2026-08-20.md` on
its own. All the identity is in the **folder** and only the date is in the filename, so every task
folder holds a file with the same name and the bare one identifies nothing. Measured on one repo: 12
handoff files across six task folders, six of them in a single folder. Reading the bare name back to
somebody makes them redo the folder lookup you already did, and date-suffixed variants
(`HANDOFF-2026-08-19-afternoon.md`) are harder still to place.

The convention itself is right and stays as it is, so this is a rule about how a handoff is REFERRED
to and never a reason to rename one. Inside the file the folder is already the context, so it does not
apply there.

## Then actually start a new session

The saving comes from the fresh context, not from the file. Hand off, end the session, and open a new
one pointed at the handoff. Carrying on in the same conversation after writing one pays the full
re-read and gains nothing.

If your harness can surface the per-turn token figure, wire it to say so rather than to block. What
that removes is the part where nobody could see the number; the decision to stop stays the user's.

**Three absolute token thresholds work well, each firing once and escalating:** 200K as a notice,
300K to offer a handoff, and 400K to say plainly that continuing is the expensive choice and stop.
Make them absolute counts rather than a percentage of anything, so they mean the same thing for a
light user and a heavy one.

To check them against your own usage, the curve they came from averaged 199K at turn 50, 287K at
turn 100, and 435K at turn 200, which puts the three lines near turns 50, 110, and 180. Growth ran
about 1.5K per turn, so 100K of headroom is roughly 65 turns.

**Measure your own before adopting these.** They come from one person's sessions, and that person is
the heaviest user of the repo they were measured in. The mean and median tracked closely at every
turn sampled, so they are not distorted by outliers, but they still describe one curve.

Two things that measurement settled, and both were wrong beforehand. **A first threshold at 150K was
too early**: it landed around turn 25, before most sessions have done anything, so the honest
response to it was always "nothing to do yet". And **a top threshold at 500K was far too late**, at
somewhere past turn 250, by which point the expensive turns have already been paid for.
