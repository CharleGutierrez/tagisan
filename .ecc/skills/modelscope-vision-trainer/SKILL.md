---
name: modelscope-vision-trainer
description: Turn plain-language vision fine-tuning requests into task plans, config files, and
  PAI-ready submission assets.
...
---
# ModelScope Vision Trainer

Use this skill for image classification, object detection, and segmentation training when the user explains the goal in natural language.

## Request Style

- Accept requests such as:
  - `Fine-tune a classifier on ./data/vision/train and ./data/vision/val.`
  - `我想做目标检测，标注在 ./data/annotations/train.json 和 ./data/annotations/val.json。`
- Infer missing fields where that is safe.
- Default task: `classification`
- Default behavior: inspect first, write configs next, submit remotely only when a runnable training entrypoint exists

## Files You Must Create

Run `scripts/prepare_training_workspace.py` with the user's request text. It writes:

1. `configs/vision.training.json`
2. `configs/remote.auto.env`
3. `configs/pai.required.env` when the required PAI fields are missing

These files are the source of truth for the rest of the workflow.

## Credential Gate

Before the first real PAI DLC submission, require:

- `ALIBABA_CLOUD_ACCESS_KEY_ID`
- `ALIBABA_CLOUD_ACCESS_KEY_SECRET`
- `PAI_REGION`
- `PAI_WORKSPACE_ID`

If any are missing:

1. Create `configs/pai.required.env`.
2. Tell the user to fill it.
3. Wait for `continue`.

## Primary Scripts

- `scripts/prepare_training_workspace.py`
- `scripts/dataset_inspector.py`
- `scripts/image_classification_training.py`
- `scripts/object_detection_training.py`
- `scripts/sam_segmentation_training.py`
- `scripts/estimate_cost.py`
- `scripts/submit_pai_dlc.py`

The three task scripts output structured plans. They are not fake training commands anymore.

## Workflow

1. Infer the task from the request or dataset layout.
2. Run `scripts/prepare_training_workspace.py --request "<user text>"`.
3. Inspect local data with `scripts/dataset_inspector.py` when paths exist.
4. If `configs/pai.required.env` exists, stop and wait for user input.
5. If the workspace already contains a real training entrypoint, package it and submit to PAI DLC.
6. If it does not, scaffold or request the entrypoint explicitly. Do not pretend the run is executable.

## Task Selection

- Classification keywords: `classification`, `分类`
- Detection keywords: `detection`, `目标检测`
- Segmentation keywords: `segmentation`, `segment`, `分割`, `sam`
- When task is ambiguous, default to `classification`

## Remote Submission Rules

- The local machine is the control plane only.
- Use `REMOTE_ASSET_PATHS` when local scripts, configs, or data must be shipped to PAI DLC.
- Local dataset directories, annotation files, and image roots should be appended to `REMOTE_ASSET_PATHS` automatically when the planner can resolve them.
- Keep the remote job honest: if a real training script is missing, the result is a remote-ready package, not a completed training run.

## References

- `references/reliability_principles.md`
- `references/timm_trainer.md`
- `references/object_detection_training_notebook.md`
- `references/image_classification_training_notebook.md`
- `references/finetune_sam2_trainer.md`

Load only the task-specific reference you need.

## Guardrails

- Never skip dataset sanity checks.
- Never assume annotations are valid without inspection.
- Never claim built-in end-to-end support for a task unless the runnable code exists.
- Never bypass the PAI credential gate.
