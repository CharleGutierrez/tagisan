---
name: salesforce-develop
description: "Salesforce development workflow and guardrails. Use when Codex is asked to develop,\
  \ modify, debug, refactor, review, or maintain Salesforce code or metadata in an\
  \ existing project, especially Apex classes, triggers, controllers, schedulers,\
  \ object fields, approval-related logic, OA integration, or other business process\
  \ code. This skill requires reading the matching project-root '*-development-doc.md'\
  \ or '*-\u5F00\u53D1\u6587\u6863.md' before editing, preserves user-owned progress\
  \ documentation, applies Chinese code comments and debug conventions, supports controlled\
  \ source backups, and forbids running any sfdx commands."
---
# Salesforce Develop

This is the default public skill file for the GitHub version.

For the Chinese version, see [SKILL_CN.md](SKILL_CN.md).

## What This Skill Enforces

1. Read the matching project-root `*-开发文档.md` or `*-development-doc.md` before making edits.
2. Do not update that document unless the user explicitly asks for documentation maintenance.
3. Never run any `sfdx` command.
4. Use Chinese comments in Salesforce code.
5. Add Chinese comments to every new or modified class, trigger, interface, enum, and method.
6. Add `System.debug` only at meaningful checkpoints and avoid redundant duplicate logs.
7. If the user explicitly asks for a backup, create it under the project-root `备份` directory and keep at most three backups per source file.

## Resources

- Chinese skill guide: [SKILL_CN.md](SKILL_CN.md)
- Chinese development document template: [references/开发文档模板.md](references/开发文档模板.md)
- English development document template: [references/development-doc-template.md](references/development-doc-template.md)
- Backup helper script: [scripts/backup_source_file.py](scripts/backup_source_file.py)

## Required Workflow

### 1. Identify the project root

Use the user-provided path or repository markers such as `.git/`, `sfdx-project.json`, `force-app/`, `README.md`, and `package.json`.

### 2. Read the matching development document first

Before editing, find the most relevant project-root `*-开发文档.md` or `*-development-doc.md`.

Priority:

1. Choose the file whose name best matches the current task.
2. If multiple files are plausible, present the candidates and ask for confirmation.
3. If none exists, stop editing and ask the user to provide or approve a relaxed workflow.

### 3. Extract the current implementation context

At minimum, capture:

- current progress
- implemented scope
- confirmed business rules
- expected end state
- known risks and pending items

### 4. Edit conservatively

- read existing logic first
- prefer minimal, controlled edits
- preserve the current project style
- do not perform unrelated refactors
- do not remove unrelated logic, comments, debug statements, tests, or configuration

## Documentation Maintenance

Only update the development document when the user explicitly asks for it.

When updating it:

1. Start from the high-level structure first, then describe flow and methods.
2. Focus on core classes and core methods only if the codebase is large.
3. Write both a business flow section and a method call chain section.
4. In the method call chain, every line should include:
   - parameters
   - return value
   - purpose

Use the public templates:

- Chinese: [references/开发文档模板.md](references/开发文档模板.md)
- English: [references/development-doc-template.md](references/development-doc-template.md)

## Comment Rules

In Salesforce code:

- use Chinese comments
- write class-level comments for each new or modified class or trigger
- write method-level comments for each new or modified method
- add comments to important branches, state transitions, batch logic, aggregation logic, recursion guards, async logic, and external integration points
- do not add low-value comments for trivial assignments or obvious returns

## Debug Rules

Add `System.debug` only at important checkpoints:

- after entering a meaningful branch
- after key parameter preparation
- before DML with important state snapshots
- after important DML or external calls
- inside exception handling
- after aggregation or state transitions

Avoid duplicate logs that say the same thing before and after entering the same block.

## Backup Rules

If the user explicitly asks for a backup:

1. Use the project-root `备份` directory.
2. Create it if missing.
3. Name each backup as `original-file-name-timestamp.ext`.
4. Keep at most three backups for the same source file.
5. On the fourth backup, delete the oldest one first.
6. Back up the original file before changes.
7. Prefer the helper script [scripts/backup_source_file.py](scripts/backup_source_file.py).

## Delivery Notes

When finishing a task, report:

- which development document was read
- which files, classes, triggers, or methods were changed
- where Chinese comments were added
- where debug logs were added
- whether the development document was intentionally left untouched
