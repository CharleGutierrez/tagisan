---
name: modelscope-local-vision-trainer
description: Turn plain-language local vision-training requests into editable configs and launch
  scripts, then run them after the user fills a local config file and replies continue.
...
---
# ModelScope Local Vision Trainer

Use this skill when the user wants local vision training on a Linux GPU workstation or server and describes the job in natural language.

## Request Style

- Accept requests such as:
  - `Train an image classifier locally with ./data/train and ./data/val, epoch=5.`
  - `我想在本地做目标检测，训练标注是 ./data/annotations/train.json，验证标注是 ./data/annotations/val.json。`
- Infer missing fields where that is safe.
- Default task: `classification`
- Default model:
  - Classification: `google/vit-base-patch16-224`
  - Detection: `facebook/detr-resnet-50`
  - Segmentation: `facebook/sam-vit-base`

## What You Must Create

Run `scripts/prepare_local_vision_training_workspace.py` with the user's exact request text. It must create:

1. `configs/local.vision.training.json`
2. `configs/local.vision.required.env`
3. `configs/local.vision.train.env`
4. `scripts/run_local_vision_training.sh`

## Execution Rule

- This skill always pauses once before execution.
- First create `configs/local.vision.required.env` with non-empty default values.
- Tell the user to review or edit that file.
- Tell the user to reply with exactly `continue`.
- Only after the user replies `continue`, run `bash scripts/run_local_vision_training.sh`.
- On macOS or any unknown host, stop after writing the files and report that execution should happen on the intended Linux GPU host.

## Primary Script

- `scripts/prepare_local_vision_training_workspace.py`

## Workflow

1. Read the user request and infer the task, model, dataset paths, and training parameters.
2. Run `scripts/prepare_local_vision_training_workspace.py --request "<user text>"`.
3. Inspect the generated plan, `configs/local.vision.required.env`, and launch script.
4. Ask the user to review or edit `configs/local.vision.required.env`.
5. Tell the user to reply with exactly `continue`.
6. Only after `continue`, run `bash scripts/run_local_vision_training.sh`.

## Task Rules

- Classification keywords: `classification`, `分类`
- Detection keywords: `detection`, `目标检测`
- Segmentation keywords: `segmentation`, `segment`, `分割`, `sam`
- When task is ambiguous, default to `classification`

## Command Rules

- Keep the generated launch script readable and editable.
- `configs/local.vision.required.env` should start with sensible defaults instead of blank placeholders.
- The launch script must verify that the configured local training entrypoint exists before it tries to run.
- Prefer one process by default unless the user clearly asks for distributed local training.

## References

- `references/runtime_requirements.md`

## Guardrails

- Never skip the `continue` gate.
- Never auto-run on macOS or an unknown host.
- Never assume the local training entrypoint already exists without checking.
- Never claim built-in local execution for a task unless the configured entrypoint is present on the target machine.
