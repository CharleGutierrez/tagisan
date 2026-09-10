---
name: sf-stakeholder-questionnaire
description: 'Getting answers out of a colleague who holds knowledge you do not. TRIGGER when:
  a decision needs input from Legal, a project manager, or a business owner, a story
  stalls on something only a stakeholder can answer, several open questions are piling
  up for one person, or a meeting needs an agenda of questions. Interview about the
  send, not the subject.'
---
# stakeholder-questionnaire

Turn questions you cannot answer into a **questionnaire**: one document a named colleague fills in
async, or that you both work through in a meeting.

Adapted from Matt Pocock's `to-questionnaire` skill (https://github.com/mattpocock/skills, MIT), with
the output moved from a bare Markdown file in the working directory to a formatted document, since a
raw `.md` is not something a Legal reviewer or a business owner is going to open and fill in.

## Grill the send, not the subject

Interview whoever is directing the work about the **send**, which they can always answer: who it goes
to, and what they need back. The questions in the document then target the **gap** between what the
recipient knows and what your side does not.

Asking them about the subject instead is how a questionnaire ends up asking its own author's questions
back at them.

## Two exchanges, then write it

1. **Who is it going to?** Their role, what they own, and what they know that you do not. This sets
   the tone and how much context the document has to carry. A Legal reviewer and a project manager
   need different framings of the same question.
2. **What do you need back?** The specific decisions or facts you cannot resolve alone. Done when you
   have a concrete list of what you must walk away able to decide.

Then write it, and cover every item from the second exchange.

## Never spend a question on something you can look up

The recipient gets one pass, so every question the system, the repository, or the tracker could have
answered is a question you did not get to ask. Settle the facts yourself first and put only genuine
gaps in the document.

**Ask nothing that can only be answered in a tool the recipient does not use.** A question mentioning
a branch, a PR, a source file, or a merge conflict is not a question to someone who has never used
git, it is a description of somewhere they cannot go. Every question has to be answerable from the
application's own admin screens, from a record page, or from what they know about the business. Where
the answer really does live in source, find it yourself and put the finding in the document as
context.

## The document

Order questions most-important-first, since an async send may only get one pass, and group them under
headings by theme once there are more than a handful. One idea per question, never compound, with an
answer stub directly beneath it, and a short line on why it matters only where the question could be
misread or invite a throwaway answer.

Open with:

- the purpose and the decision riding on it,
- one paragraph of context for a recipient who was not in your head,
- how to answer: an explicit deadline date, the rough effort, and that partial answers and "I don't
  know" are useful.

Close with a catch-all asking what you should have asked.

Produce it in a format the recipient will actually open, with a lowercase kebab-case filename, since
the filename becomes the document's visible title once it is shared.

## Write it for the person, not for the repo

- **No em dashes**, since this is addressed to another person and they render badly in most of the
  places it will be pasted.
- **Plain, calm language and a measured tone.** Prefer "I can" or "could" over "I'll" or "will" before
  anything is agreed, and hedge an unverified inference as one.
- **Do not presume the recipient's workflow.** Neutral phrasing when you are inferring what they do,
  and name each object and field by the label on their own screen rather than the API name.

## Drafting it is not sending it

Show the draft and stop. Approval of the content is not approval to send, and a questionnaire reaches
a real person under your name, so the send needs its own go-ahead. That covers posting it to the
tracker, attaching it to a work item, and pasting it into chat or an email.

Related: use `design-grilling` when the decisions are yours to settle rather than someone else's to
answer, and record what comes back wherever the implementation plan lives.
