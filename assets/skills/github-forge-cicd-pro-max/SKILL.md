---
name: github-forge-cicd-pro-max
description: Autonomous Master Engine for GitHub Forge Integration, Pull Request Automation, Issue Triage, CI/CD Pipeline Telemetry, and Offline Git Repository Fallbacks. Enforces deterministic API authentication, GitHub CLI (gh) orchestration, git branch & diff analysis, commit history mining, workflow log inspection, and resilient air-gapped local VCS operations across Windows, Linux, and macOS. Triggers: github, git, forge, pull-request, pr-review, issue-tracker, ci-status, github-actions, git-diff, commit-history, github-pro-max.
version: 1.0.0
tags:
  - github
  - git
  - pr
  - issues
  - cicd
  - github-actions
  - diff
  - vcs
  - offline-git
compatibility: ">=0.2.0"
---

# GitHub Forge & CI/CD Pro Max: Autonomous Repository, PR & Workflow Orchestration Engine

## Purpose & Scope
Modern autonomous engineering agents operate within collaborative Git codebases: opening pull requests, reviewing diffs, triaging issue tickets, validating CI build statuses, and diagnosing test pipeline failures. Inability to connect to GitHub or failing when network tokens are missing halts developer momentum.

The `github-forge-cicd-pro-max` skill establishes Tagisan's authoritative GitHub and Git forge orchestration architecture. It codifies the 6 architectural pillars governing GitHub REST/GraphQL API integration, GitHub CLI (`gh`) execution, resilient local Git repository fallback operations, multi-file diff inspection, CI log streaming, and offline development invariants.

---

## Pillar FORGE-01: Dual-Tier Cloud & Local Repository Orchestration

### 1. Unified Cloud API & Local VCS Fallback
The tool adapts seamlessly across online and offline environments:
- **Cloud Tier**: Authenticates via `GITHUB_TOKEN` or `GH_TOKEN` environment variables, or through active `gh auth status` session credentials. Queries GitHub v3 REST API / GitHub Actions API endpoints.
- **Local Git Repository Tier**: When working offline, air-gapped, or without credentials:
  - Automatically queries the local Git repository via native `git` CLI operations.
  - Resolves branch topology, remote heads (`refs/remotes/origin/*`), local commits, and staged/unstaged changes.
  - Guarantees that commands like `pr_list`, `diff_view`, and `commit_history` function with full fidelity regardless of network connectivity.

---

## Pillar FORGE-02: Pull Request Lifecycle & Review Automation

### 1. PR Mining & Review Inspection
- `pr_list`: Retrieves open, closed, or merged pull requests with author, labels, review status, and branch head/base info. Local fallback lists active development branches compared against the base branch.
- `pr_view`: Inspects PR description, conversation comments, assigned reviewers, CI checks summary, and changed files list. Local fallback extracts git log messages and commits between `main` and feature branch.

---

## Pillar FORGE-03: Issue Tracking & Requirement Traceability

### 1. Issue Triage & Metadata Extraction
- `issue_list`: Lists active tickets filtered by state (`open`, `closed`), labels, or milestone. Local fallback parses `.github/ISSUE_TEMPLATE`, Markdown issue specs, or commits matching `#<issue_id>`.
- `issue_view`: Retrieves ticket problem descriptions, acceptance criteria, reproduction steps, and discussion threads.

---

## Pillar FORGE-04: CI/CD Pipeline Telemetry & Failure Log Inspection

### 1. GitHub Actions Run Diagnostics
- `ci_status`: Queries the status of the latest GitHub Actions workflow runs (Success, Failure, In Progress, Queued). Local fallback validates `.github/workflows/*.yml` syntax, checks local git commit status, and inspects test exit codes.
- `ci_logs`: Streams failed job logs, extracting compilation errors, broken unit test tracebacks, and lint warnings directly into agent context for self-healing.

---

## Pillar FORGE-05: High-Performance Git Diff & Tree Analysis

### 1. Structured Patch & Diff Synthesis
- `diff_view`: Generates unified diffs comparing:
  - Working tree vs. HEAD
  - Feature branch vs. target base (`git diff main...feature`)
  - Specific commit SHA diffs
- Includes file stat summaries (`+added, -deleted` lines) and changed file path lists.

---

## Pillar FORGE-06: Commit History Mining & Blame Archeology

### 1. Deterministic Commit Log Inspection
- `commit_history`: Queries recent repository commits with structured JSON output:
  - Commit SHA (full and abbreviated)
  - Author name and email
  - Timestamp (ISO 8601)
  - Commit subject and message body
  - List of modified files

---

## Autonomous Tool-Use Protocols
1. **Pre-PR Diff Verification**: Agent runs `action: "diff_view"` to review changes against `main` before opening PR.
2. **CI Pipeline Health Check**: Following push, agent calls `action: "ci_status"`; if failure occurs, agent runs `action: "ci_logs"` to immediately remediate the error.
