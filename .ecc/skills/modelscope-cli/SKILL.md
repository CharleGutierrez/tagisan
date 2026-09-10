---
name: modelscope-cli
description: Execute ModelScope Hub login, create, download, and upload tasks from plain-language
  requests with the official CLI.
...
---
# ModelScope CLI

Use this skill when the task is fundamentally a repository operation and the official `modelscope` CLI is the most direct path.

## Request Style

- Accept requests like:
  - `Download this model locally.`
  - `创建一个新的数据集仓库并上传 data/ 目录。`
- Translate the request into explicit CLI commands with explicit repo types and paths.

## Core Uses

- login
- create model or dataset repo
- download full or partial snapshots
- upload files or folders

## Workflow

1. Determine the repo type.
2. Check whether authentication is already available.
3. Keep local output directories explicit.
4. Keep upload target paths explicit.
5. If credentials are missing, still prepare the exact command sequence.

## When Not To Use

- heavy remote compute
- dataset transformation logic that deserves a script
- complex metadata normalization

## Guardrails

- Never hardcode tokens.
- Never guess the repo type.
- Never use ambiguous destination paths.
- Prefer deterministic `--local_dir` downloads.
