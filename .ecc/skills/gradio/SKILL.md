---
name: gradio
description: Build or edit Gradio apps from plain-language UI requests, choosing the simplest valid
  Gradio pattern for the job.
...
---
# Gradio

Use this skill when the user wants a Gradio demo, interface, or app and describes the UI in natural language.

## Request Style

- Accept requests such as:
  - `Build a simple image classification demo.`
  - `做一个聊天界面，支持流式回复和参数调节。`
- Infer the lightest valid Gradio abstraction:
  - `Interface` for one function in and out
  - `Blocks` for custom layouts and multiple events
  - `ChatInterface` for chatbot-style flows

## Workflow

1. Map the request to `Interface`, `Blocks`, or `ChatInterface`.
2. Reuse the existing app structure when present.
3. Keep the code runnable with clear imports and launch logic.
4. Add only the components and event wiring the task actually needs.

## References

- `examples.md`
- Official Gradio guides linked there when a deeper pattern is needed

## Guardrails

- Never overbuild a tiny demo.
- Never choose `Blocks` if `Interface` is enough.
- Keep custom JS and CSS purposeful and minimal.
