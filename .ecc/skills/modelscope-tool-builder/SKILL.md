---
name: modelscope-tool-builder
description: Convert plain-language ModelScope automation needs into reusable scripts with stable
  flags, structured output, and official API usage.
...
---
# ModelScope Tool Builder

Use this skill when the user keeps repeating a ModelScope workflow and it should become a reusable script.

## Request Style

- Accept requests like:
  - `Make me a tool that syncs model cards in batch.`
  - `把这个 Hub 数据收集流程做成可复用脚本。`

## Workflow

1. Identify the repeatable unit of work.
2. Prefer official CLI or SDK behavior.
3. Build one script with explicit flags and actionable errors.
4. Emit structured output when downstream automation benefits from it.
5. Validate the lightest meaningful path locally.

## References

- `references/baseline_modelscope_api.py`
- `references/baseline_modelscope_api.sh`
- `references/baseline_modelscope_api.tsx`
- `references/modelscope_enrich_models.sh`
- `references/modelscope_model_card_frontmatter.sh`
- `references/modelscope_model_papers_auth.sh`

## Guardrails

- Never bake secrets into the script.
- Never scrape undocumented endpoints when official interfaces exist.
- Prefer one small reliable utility over a vague framework.
