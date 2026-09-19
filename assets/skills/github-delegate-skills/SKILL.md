---
name: github-delegate-skills
description: GitHub Delegate-Skills for Just-In-Time skill hydration, micro-agent task delegation, Landlock file-boundary sandboxing, CI log self-healing, and automatic post-task heap trimming.
tools:
  - read_file
  - write_file
  - run_command
  - reach_search
  - delegate_task
---

# GitHub Delegate-Skills (TGS Skill Guide)

## Overview
GitHub Delegate-Skills empowers Tagisan (TGS) to practice ephemeral micro-agent delegation. Rather than loading large tool collections or bulky context into a monolithic agent, the orchestrator delegates specific tasks to ephemeral specialist subagents with Just-In-Time (JIT) skill hydration and strict resource boundaries.

## Core Capabilities
1. **JIT Skill Hydration**: Reads `.github/skills/*.md` and `assets/skills/*/SKILL.md` declarative manifests with YAML frontmatter, loading tools and instructions on-demand without memory bloat.
2. **Strict Landlock File Boundary**: Subagents are constrained to declared target files, preventing path traversal attacks and accidental workspace mutations.
3. **Automated CI/CD Log Parsing & Self-Healing**: Automatically extracts rustc compiler diagnostics and test failures from GitHub Actions CI logs and formulates surgical repair plans.
4. **Zero 8GB Laptop Freezes**: Employs `libc::malloc_trim(0)` immediately upon task completion, releasing glibc heap arenas back to the Linux kernel.
5. **Anti-Freeze Memory Barrier**: Blocks subagent spawning whenever host available RAM drops below critical thresholds (< 500MB or < 10%).

## REPL Commands
- `/delegate list`: List all available skills discovered in `.github/skills/` and `assets/skills/`
- `/delegate run <skill_name> <target_file> [instruction]`: Execute delegated micro-agent task with memory governor and file boundary
- `/delegate ci <ci_log_snippet>`: Parse CI failure logs and generate surgical auto-healing PR audit
- `/delegate help`: Display command syntax and options
