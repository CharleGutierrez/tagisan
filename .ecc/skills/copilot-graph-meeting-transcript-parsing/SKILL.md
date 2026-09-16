---
name: copilot-graph-meeting-transcript-parsing
description: "Asynchronous meeting transcript retrieval (/onlineMeetings/{id}/transcripts), VTT parsing, and speaker diarization."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Teams Graph API: Channels, Chats and Meetings - Hilton Giesenow"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["meeting-transcript-parsing", "teams-meeting-transcripts", "vtt-parsing", "speaker-diarization"]
---

# copilot-graph-meeting-transcript-parsing
> Based on **Microsoft Teams Graph API: Channels, Chats and Meetings - Hilton Giesenow** (Cluster 4)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Transcript Availability: Transcripts become available asynchronously post-meeting finalization.**
2. **Diarization Model: Map VTT speaker timestamps to resolved Entra ID user identities.**
3. **ALWAYS handle partial or empty transcript streams gracefully.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-meeting-transcript-parsing.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-graph-meeting-transcript-parsing actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Retrieve and parse Teams meeting transcripts into structured speaker-turn JSON blocks for downstream Copilot summarization.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-meeting-transcript-parsing.**
- **Unmonitored runtime execution without telemetry in copilot-graph-meeting-transcript-parsing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "meeting-transcript-parsing"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
