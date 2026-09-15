---
name: agentic-wooldridge-multiagent-systems
description: "BDI (Belief-Desire-Intention) agent architecture, FIPA-ACL communicative acts, contract net protocol, coalition formation, and multi-agent coordination for autonomous software engineering swarms."
triggers: ["wooldridge", "multiagent-systems", "bdi-architecture", "belief-desire-intention", "fipa-acl", "contract-net-protocol", "agent-negotiation", "swarm-coordination"]
---

# agentic-wooldridge-multiagent-systems
> Based on **An Introduction to MultiAgent Systems (2nd ed) - Michael Wooldridge**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **BDI Architecture: Beliefs represent epistemic state, Desires represent motivational goals, and Intentions represent committed computational action plans.**
2. **Speech Act Theory & FIPA-ACL: Every message between agents MUST define performative acts (request, propose, accept-proposal, reject-proposal, inform) with formal pre- and post-conditions.**
3. **Contract Net Protocol (CNP): Task allocation proceeds through Announcement -> Bidding -> Awarding -> Execution -> Result Reporting.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement autonomous multi-agent systems using explicit BDI state loops. Decouple agent communication via strongly typed ACL protocols. Structure task distribution using Contract Net Protocol with timeout guarantees and fallback bids.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Free-form unstructured chatter between agents leading to unbounded conversational divergence.**
- **Conflating desires (potential goals) with intentions (committed executable tasks), causing thrashing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wooldridge"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
