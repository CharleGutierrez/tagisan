---
name: git-worktree-merge-arbiter
description: Autonomous Git worktree isolation and semantic merge arbiter. Provisions isolated sandboxes for risky exploratory coding, enforces clean working tree invariants, runs automated verification before commits, and executes semantic AST conflict resolution.
version: 1.0.0
tags:
  - git-worktree
  - worktree-isolation
  - merge-arbiter
  - conflict-resolution
  - sandbox
  - ast-merge
triggers:
  - git-worktree
  - worktree-isolation
  - merge-arbiter
  - conflict-resolution
  - isolated-sandbox
  - ast-merge
  - clean-tree-invariant
compatibility: ">=0.2.0"
---

# Git Worktree Merge Arbiter: Autonomous Sandbox & Conflict Resolution

## Purpose & Scope
Autonomous coding agents operating directly on a developer's primary checkout risk dirtying unstaged edits, destroying uncommitted work, or leaving the workspace in an unbuildable state during exploratory refactorings.

This skill provides ironclad worktree isolation and semantic merge arbitration. All exploratory, high-blast-radius, or multi-agent parallel workflows execute in ephemeral, dedicated Git worktrees (`git worktree add`). Changes undergo mandatory pre-commit verification before being merged back, and conflict resolution leverages AST structural awareness rather than brittle text markers.

---

## 1. Operational Invariants (Worktree Protocol)

### Invariant 1: Clean Tree Preservation Invariant
- **NEVER**: Perform destructive edits, speculative rewrites, or heavy refactoring in a dirty repository working tree where user changes are unstaged.
- **MANDATORY**: If `git status --porcelain` indicates uncommitted user modifications, or if the task is exploratory, the agent MUST immediately provision an isolated Git worktree sandbox via `git_worktree(action: "create", ...)`.

### Invariant 2: Ephemeral Sandbox Isolation
- Every autonomous sub-task or parallel agent must run within its own isolated worktree branch (e.g. `tagisan/sandbox-<timestamp>`).
- Worktrees are hosted outside the primary source tree (in temporary directories or designated sandbox storage) to ensure zero pollution of the developer's IDE indexer or active watcher.

### Invariant 3: Pre-Commit Verification Gate
- **NEVER**: Commit code or declare a task complete inside a worktree without running full automated verification:
  1. Syntax & typecheck (`cargo check`, `tsc --noEmit`, etc.)
  2. Test suite (`cargo test`, `bun test`, `pytest`)
  3. Linter & formatting (`cargo clippy`, `eslint`)
- Verification must produce exit code 0 before invoking `git_worktree(action: "commit", message: "...")`.

### Invariant 4: Semantic AST Conflict Resolution
- When integrating or rebasing worktree branches, line-based Git merge conflicts (`<<<<<<<`, `=======`, `>>>>>>>`) must be resolved with semantic AST awareness:
  - **Imports/Uses**: Merge both sets, deduplicate, and sort alphabetically.
  - **Enum Variants / Match Arms**: Append new variants without dropping existing ones.
  - **Non-Overlapping Functions**: Retain both implementations.
  - **Overlapping Logic**: Verify against the invariant specification and select the version that preserves contractual guarantees.

### Invariant 5: Deterministic Lifecycle & Cleanup
- **ALWAYS**: Upon task success (after merging) or task abort/rollback, invoke `git_worktree(action: "cleanup", worktree_id: "...")` to prune the temporary directory and delete the ephemeral branch. Never leak abandoned worktree references.

---

## 2. Tool Integration & Operational Recipes

### Provisioning an Isolated Worktree
```json
{
  "action": "create",
  "repo_path": ".",
  "branch": "feature/async-worker-pool"
}
```

### Executing Verification Inside Sandbox
```json
{
  "action": "run",
  "worktree_id": "feature/async-worker-pool",
  "command": "cargo test --lib -- --nocapture"
}
```

### Inspecting Sandbox Diff
```json
{
  "action": "diff",
  "worktree_id": "feature/async-worker-pool"
}
```

### Committing & Cleaning Up
```json
{
  "action": "commit",
  "worktree_id": "feature/async-worker-pool",
  "message": "feat(worker): implement lock-free work-stealing queue with verification"
}
```
