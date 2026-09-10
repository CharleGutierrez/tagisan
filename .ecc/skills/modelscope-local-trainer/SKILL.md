---
name: modelscope-local-trainer
description: Turn plain-language local-training requests into editable configs and launch scripts,
  then run them after the user fills a local config file and replies continue.
...
---
# ModelScope Local Trainer

Use this skill when the user wants local training on a Linux GPU workstation or server and describes the job in natural language.

## Request Style

- Accept requests such as:
  - `Use Qwen/Qwen2.5-0.5B-Instruct on ./datasets/train.jsonl for local SFT, epoch=3, lr=5e-4.`
  - `我想在本地 GPU 机器上做 DPO，对齐数据在 ./data/prefs.jsonl。`
- Infer missing fields instead of forcing a rigid template.
- Default policy:
  - Method: `SFT`
  - Model: `Qwen/Qwen2.5-0.5B-Instruct`
  - Train type: `LoRA`
  - Output: `./outputs/local-<method>`

## What You Must Create

Run `scripts/prepare_local_training_workspace.py` with the user's exact request text. It must create:

1. `configs/local.training.plan.json`
2. `configs/local.required.env`
3. `configs/local.train.env`
4. `scripts/run_local_training.sh`

If the user does not provide a dataset, the script may create a small local example dataset under `data/local/<method>/`.

## Execution Rule

- This skill always pauses once before execution.
- First create `configs/local.required.env` with non-empty default values.
- Tell the user to review or edit that file.
- Tell the user to reply with exactly `continue`.
- Only after the user replies `continue`, run `bash scripts/run_local_training.sh`.
- On macOS or any unknown host, stop after writing the files and report that execution should happen on the intended Linux GPU host.

## Primary Script

- `scripts/prepare_local_training_workspace.py`

## Workflow

1. Read the user request and infer method, model, dataset, and training parameters.
2. Run `scripts/prepare_local_training_workspace.py --request "<user text>"`.
3. Inspect the generated plan, `configs/local.required.env`, and launch script.
4. Ask the user to review or edit `configs/local.required.env`.
5. Tell the user to reply with exactly `continue`.
6. Only after `continue`, run `bash scripts/run_local_training.sh`.

## Command Rules

- `SFT` uses `swift sft`.
- `DPO` and `GRPO` use `swift rlhf --rlhf_type dpo|grpo`.
- Keep the generated launch script readable and editable.
- Prefer one process by default unless the user clearly asks for distributed local training.
- `configs/local.required.env` should start with sensible defaults instead of blank placeholders.

## References

- `references/runtime_requirements.md`

## Guardrails

- Never skip the `continue` gate.
- Never auto-run on macOS or an unknown host.
- Never assume this machine has the right GPU stack without checking.
- Never overwrite an existing launch script without making the new contents explicit in the plan file.
