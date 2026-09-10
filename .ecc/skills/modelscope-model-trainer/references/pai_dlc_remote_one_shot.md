# PAI DLC Remote One-Shot Playbook

## Goal

Submit and complete ms-swift training remotely with minimal retries.

## Preflight

- Required:
  - `ALIBABA_CLOUD_ACCESS_KEY_ID`
  - `ALIBABA_CLOUD_ACCESS_KEY_SECRET`
  - `PAI_REGION`
- Optional:
  - `PAI_WORKSPACE_ID`
  - `MODELSCOPE_API_TOKEN`
  - `MS_REPO_OWNER`
  - `MS_REPO_BASE`

## Recommended Defaults (Budget Safe)

- `PAI_DLC_GPU=0`
- `PAI_DLC_CPU=4`
- `PAI_DLC_MEMORY=16Gi`
- `PAI_DLC_IMAGE=pytorch/pytorch:latest`
- `EXTRA_SWIFT_ARGS=--bf16 false --fp16 false --torch_dtype float32`

## Run Sequence

1. Prepare assets/config:
   - dataset train/valid
   - pilot/full env files
2. Submit pilot and wait.
3. If pilot passes, submit full.
4. Read logs and pick the last saved checkpoint.
5. Publish if ModelScope token is available.

## Must-Have Log Evidence

- Training command executed (`swift sft` or equivalent).
- Step progress lines (`global_step/max_steps`).
- Checkpoint save lines (`Saving model checkpoint to ...`).

## Common Auto-Fixes

- `resourceLimit`: fall back to CPU.
- `bf16` unsupported: force `float32`, disable `bf16/fp16`.
- `enterRunningTimeout`: switch to verified image.
- install-only exits: shorten startup command and ensure training entrypoint is present.
