---
name: hallucination-detector
description: Context grounding validation, fact checking against retrieved vector memory, and claim verification
---

# Hallucination Detection & Context Grounding

## Verification Procedure
1. **Claim Extraction**: Decompose LLM output into atomic, verifiable technical assertions.
2. **Grounding Cross-Check**: Verify each claim against source code chunks and episodic memory retrieved via RAG.
3. **Factuality Scoring**: Flag and reject claims that cite non-existent functions, imaginary API endpoints, or phantom types.
4. **Contradiction Analysis**: Detect if proposed logic violates previously established architectural invariants.
