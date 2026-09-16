---
name: copilot-semantic-index-architecture
description: "Semantic Index for Copilot: tenant-level and user-level vector graph substrate, freshness crawling, and cross-workload ranking."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Deep Dive into Microsoft 365 Semantic Index - Satya Nadella et al."
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["semantic-index", "semantic-index-architecture", "tenant-vector-substrate", "user-level-embeddings"]
---

# copilot-semantic-index-architecture
> Based on **Deep Dive into Microsoft 365 Semantic Index - Satya Nadella et al.** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Dual-Layer Architecture: User-level index (emails, chats, personal docs) and Tenant-level index (public SharePoint, shared repositories).**
2. **ACL Preservation Invariant: The Semantic Index NEVER surfaces an entity unless the querying user possesses explicit read access at indexing and query time.**
3. **Freshness SLA: Changes in source documents must trigger Graph change notifications and incremental vector updates.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-semantic-index-architecture.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-semantic-index-architecture actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize enterprise data for Semantic Index ingestion by providing rich metadata, explicit entity relationships, and hierarchical Markdown structures.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying on broad 'Everyone except external users' permissions to mask broken ACL hierarchies.**
- **Treating Semantic Index as a static snapshot rather than an incremental live graph.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "semantic-index"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
