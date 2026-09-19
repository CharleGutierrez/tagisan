---
name: github-planning-with-files
description: GitHub Planning with Files for issue decomposition, strict file manifests, pre-flight blast radius auditing, and self-healing PR synthesis.
tools:
  - read_file
  - write_file
  - run_command
  - reach_search
  - calculator
---

# GitHub Planning with Files (TGS Skill Guide)

## Overview
GitHub Planning with Files transforms high-level GitHub issues, milestones, and project cards into deterministic, repository-grounded file execution manifests persisted in `.tgs/plans/issue-{N}.md`.

## Core Capabilities
1. **Issue Decomposition & Grounding**: Scrapes or queries GitHub issue URLs or numbers and maps user stories to concrete codebase target files.
2. **Strict File Whitelist**: Enforces a non-bypassable file-access whitelist (`GhFileGovernor`) to eliminate hallucination drift and out-of-scope edits.
3. **Pre-flight Blast Radius Auditing**: Evaluates dependency and architectural impact before touching core infrastructural files (`Cargo.toml`, `src/lib.rs`).
4. **Context Window & RAM Preservation**: Scopes LLM attention to declared target files, keeping memory footprint under 150MB and eliminating 8GB laptop freezes.
5. **Atomic Conventional Commits**: Maps checklist completion to verified commits (`feat(scope): task summary (refs #N)`).
6. **Production PR Synthesis**: Generates full Markdown pull requests with file mutation matrices, blast radius risk tiers, test pass evidence, and Hermes red-team security audits.

## REPL Commands
- `/plan fetch <issue_url_or_num>`: Scaffolds `.tgs/plans/issue-<N>.md` from GitHub issue with strict file whitelist
- `/plan status`: Inspect active plan manifest, completion percentage, and whitelist
- `/plan list`: List all `.tgs/plans/*.md` manifests in the repository
- `/plan pr`: Synthesize production GitHub PR with blast radius matrix & test logs
- `/plan <objective>`: Run autonomous planning prompt on workspace objective
