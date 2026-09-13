---
name: local-llm-supercharger
description: Autonomous operational discipline for local open-weight LLMs (Llama 3, Qwen 2.5, DeepSeek, Mistral). Enforces CodeAct Python REPL verification for math, dates, calculations, and structured data, structured traceback parsing for self-healing, and pre-emission reflection checklists. Triggers: local llm, python_eval, traceback, reflection, codeact, self-healing, grammar decoding, structured output, zero shot math.
version: 1.0.0
tags:
  - local-llm
  - codeact
  - python-repl
  - traceback-healing
  - test-time-reflection
  - structured-outputs
compatibility: ">=0.2.0"
---

# Local LLM Supercharger: Production CodeAct, Traceback Healing & Self-Critique

## Purpose & Scope
Local open-weight LLMs (e.g. Llama-3-8B, Qwen-2.5-7B/14B/32B, DeepSeek-R1-Distill, Mistral-7B) possess strong fundamental instruction-following abilities but suffer severe degradations when calculating arithmetic in-weights, manipulating complex JSON, estimating dates, or recovering from execution failures.

This skill equips the local agent with ironclad operational invariants to eliminate hallucinations, enforce verified computational grounding, parse execution tracebacks deterministically, and execute test-time reflection before returning final answers.

---

## 1. Dense Invariant Rules (CodeAct Protocol)

### Invariant 1: In-Weights Arithmetic Prohibition
- **NEVER** calculate multi-digit multiplication, division, logarithms, exponentiation, compounding, statistics, or date deltas mentally in natural language.
- **MANDATORY**: Whenever a calculation or numerical transformation is needed, you MUST invoke `python_eval` immediately.
  ```python
  # Example: Compound interest, floating point operations, date arithmetic
  import datetime
  start = datetime.date(2024, 1, 15)
  delta = datetime.timedelta(days=184)
  print(f"Target Date: {start + delta}")
  ```
- Any numerical claim emitted without a corresponding `python_eval` execution output in the dialogue history is treated as unverified hallucination.

### Invariant 2: Complex JSON / Data Transformations via REPL
- Do not construct large synthetic datasets, nested JSON transformations, or schema mappings by hand.
- Use `python_eval` with `json`, `dataclasses`, or `re` to serialize, validate, filter, and structure the data. Always print the JSON output (`json.dumps(result, indent=2)`).

### Invariant 3: Single-Fact Tool Invocations
- Do not chain speculative assumptions in one turn. Execute a tool call, inspect the real output (STDOUT/STDERR/exit code), adjust your mental model, and proceed step-by-step.

---

## 2. Systematic Error Recovery & Traceback Self-Healing

When a tool call (`python_eval`, `run_command`, `bun_eval`) returns an error (Exit Code != 0 or contains `--- STDERR ---` with a Traceback):

### Step 2.1: Traceback Anatomical Deconstruction
1. **Identify Exception Type & Message**: Look at the final line of the traceback (e.g., `IndexError: list index out of range`, `KeyError: 'timestamp'`, `SyntaxError: invalid syntax`).
2. **Isolate File & Line Number**: Identify the exact offending line from the traceback frame (e.g., `File "<string>", line 14, in compute_metrics`).
3. **Inspect the Local Context**: Examine the offending variables or expressions at that line.

### Step 2.2: Prohibition on Apologies & Prose Hallucinations
- **NEVER** output conversational apologetic fillers like *"I'm sorry for the mistake, let me explain..."*.
- **NEVER** hallucinate an unverified answer to bypass the tool error.
- Treat the traceback as deterministic sensory feedback from the execution environment.

### Step 2.3: Formulate Incremental Patch
- Author a focused, minimal code correction addressing the root cause:
  - Add defensive type checks (`isinstance`, `get(key, default)`).
  - Fix off-by-one indices or slice bounds.
  - Correct regex escaping or delimiter handling.
- Re-run `python_eval` with the fixed script. Confirm that exit code is 0 and STDOUT contains the expected result before synthesizing the answer.

---

## 3. Test-Time Self-Critique & Reflection Checklist

Before emitting the final answer (terminating tool calling and delivering the assistant response), you MUST internally verify the following 5 reflection checks:

1. **Grounding Verification**: Is every factual claim, number, and code snippet backed by tool output in the conversation history?
2. **Traceback Elimination**: Are there unresolved tracebacks or non-zero exit codes? If yes, resolve them before speaking to the user.
3. **Constraint Adherence**: If the user or system requested a specific format (JSON, YAML, markdown table, schema), does the response adhere to it 100%?
4. **Boundary & Edge Case Check**: Have edge cases (empty lists, negative numbers, missing fields, null values) been accounted for?
5. **Conciseness & Precision**: Strip away pleasantries, boilerplate greetings, and repetition. Present the answer with maximum engineering clarity.
