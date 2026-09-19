---
name: agent-reach-integration
description: Real-time multi-platform intelligence gathering across Twitter/X, Reddit, GitHub, YouTube, and Jina WebReader without API fees.
author: Tagisan Core Team & Panniantong
version: 1.0.0
tags:
  - agent-reach
  - web-intelligence
  - twitter
  - reddit
  - github
  - youtube
  - scraping
  - zero-cost-api
tools:
  - reach_search
  - reach_fetch
---

# Agent-Reach Multi-Platform Intelligence Gathering

## Overview
The `agent-reach-integration` skill provides Tagisan (TGS) autonomous agents with real-time, zero-cost internet perception across 6 major technical and social platforms:
1. **Twitter / X**: Developer discussions, breaking news, AI benchmark releases.
2. **Reddit**: In-depth community engineering discussions (`r/rust`, `r/LocalLLaMA`, `r/netsec`).
3. **GitHub**: Active issue threads, unmerged PR reviews, commit diffs, and trending repositories.
4. **YouTube**: Technical keynote and lecture video transcripts.
5. **Web Reader (Jina)**: Clean markdown extraction from any arbitrary URL via `r.jina.ai`.
6. **RSS / Technical Feeds**: Changelog and release feed aggregation.

---

## Architectural Principles

### 1. Zero API Fees via Local Session Cookies
Agent-Reach utilizes lightweight browser sessions and authenticated cookie profiles rather than enterprise paywalled APIs ($100-$5,000/mo). Configuration can be optionally provided in `~/.agent-reach/config.yaml`:

```yaml
# ~/.agent-reach/config.yaml
twitter:
  auth_token: "your_twitter_auth_token_here"
reddit:
  session: "your_reddit_session_cookie"
github:
  token: "ghp_optional_personal_access_token"
timeout_seconds: 15
user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"
```

If the configuration file is absent, Agent-Reach automatically uses public unauthenticated endpoints and deterministic fallback parsers.

### 2. Mandatory Prompt Injection Defense
Because live web content is inherently untrusted, all retrieved content is automatically passed through `ReachPromptInjectionSanitizer`:
- Strips zero-width stealth characters (`U+200B`, `U+FEFF`, `U+00AD`).
- Neutralizes prompt injection patterns (`Ignore previous instructions`, `system prompt override`).
- Automatically redacts leaked API keys (`sk-...`, `ghp_...`, `Bearer ...`).

### 3. Mixture-of-Agents (MoA) Workhorse Routing
Always combine Agent-Reach with the **Hermes Workhorse** (`hermes-3-llama-3.1-8b` or local Ollama) for scraping and formatting. Escalate to the **Cloud Orchestrator** (DeepSeek R1 / Claude 3.5 Sonnet) only when synthesizing architectural decisions.

---

## REPL & Slash Command Usage

Inside the TGS Interactive REPL:
- `/reach doctor`: Run diagnostic check on scrapers and connectivity.
- `/reach x <query>`: Search Twitter / X developer posts.
- `/reach reddit <query>`: Search Reddit community discussions.
- `/reach github <repo_or_query>`: Search GitHub issues and code.
- `/reach youtube <url_or_topic>`: Search and extract YouTube transcripts.
- `/reach web <url>`: Ingest clean markdown from any URL via Jina Reader.

---

## Tool Definitions for Autonomous Agents

### `reach_search`
```json
{
  "platform": "reddit",
  "query": "tokio 1.43 deprecations",
  "sub_context": "rust",
  "limit": 5
}
```

### `reach_fetch`
```json
{
  "url": "https://github.com/NousResearch/Hermes-3/issues/42"
}
```
