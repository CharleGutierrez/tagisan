---
name: transformers-js
description: Build browser or Node.js inference workflows from plain-language requests using Transformers.js
  and locally downloaded model assets.
...
---
# Transformers.js For ModelScope Workflows

Use this skill when the user wants JS or TS inference and describes the behavior in natural language.

## Request Style

- Accept requests such as:
  - `Make a small Node.js text-classification demo.`
  - `做一个浏览器端摘要页面，模型文件先下载到本地。`

## Workflow

1. Decide whether the runtime is browser, Node.js, or both.
2. Download model assets explicitly to a local folder.
3. Load them with `@huggingface/transformers`.
4. Leave the project runnable with clear install and run commands.

## Rules

- Keep local model paths explicit.
- Prefer the smallest runnable example over framework-heavy scaffolding.
- Pin versions when editing production code.

## References

- `references/EXAMPLES.md`
- `references/TEXT_GENERATION.md`
- `references/PIPELINE_OPTIONS.md`
- `references/CONFIGURATION.md`
- `references/CACHE.md`

Load only what the current task needs.

## Guardrails

- Never hide where model artifacts come from.
- Never assume browser-only code will run in Node.js or the reverse.
- Keep asset caching deterministic.
