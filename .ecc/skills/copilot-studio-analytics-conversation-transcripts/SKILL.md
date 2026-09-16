---
name: copilot-studio-analytics-conversation-transcripts
description: "Analyzing CSAT, session transcripts, unhandled queries, and conversation KPIs."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Conversational Analytics and Session Telemetry - Microsoft Learn"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-analytics", "conversation-transcripts", "csat-monitoring", "session-telemetry"]
---

# copilot-studio-analytics-conversation-transcripts
> Based on **Conversational Analytics and Session Telemetry - Microsoft Learn** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Session Metrics: Engagement rate, resolution rate, escalation rate, and CSAT scores.**
2. **Transcript Auditing: Conversation transcripts stored securely in Dataverse (`ConversationTranscript` table).**
3. **MANDATORY retention policies governing conversation history in compliance with GDPR/HIPAA.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-studio-analytics-conversation-transcripts.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-analytics-conversation-transcripts.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build automated Power BI dashboards connected to Dataverse transcript tables to surface top unhandled triggers and user friction points.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Failing to review unhandled query reports, letting bot accuracy degrade over time.**
- **Storing unredacted PII in conversation transcript telemetry.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-analytics"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
