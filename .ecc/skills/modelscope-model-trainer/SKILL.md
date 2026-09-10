---
name: modelscope-model-trainer
description: Turn plain-language language-model training requests into ms-swift configs, PAI DLC
  jobs, and ModelScope publications with sensible defaults.
...
---
# ModelScope Model Trainer

Use this skill for language-model SFT, DPO, or GRPO when the user describes the job in natural language instead of handing over finished config files.

## Request Style

- Accept requests such as:
  - `Use Qwen/Qwen2.5-7B-Instruct on my dataset for SFT.`
  - `我想用 Qwen2.5 做 DPO，对齐数据在 ./data/prefs.jsonl，学习率 2e-5。`
- Infer missing fields instead of asking for a template.
- Default policy:
  - Method: `SFT`
  - Model: `Qwen/Qwen2.5-0.5B-Instruct`
  - Train type: `LoRA`
  - Dataset: generate a small local dataset under `data/<method>/` when the user does not provide one
  - Remote path: PAI DLC pilot first, then full run

## Files You Must Create

Run `scripts/training_workspace.py` with the user's exact request text. That script must create:

1. `configs/training.plan.json`
2. `configs/remote.auto.env`
3. `configs/pilot.env`
4. `configs/full.env`
5. `configs/pai.required.env` when the four required PAI fields are still missing

You may also call `scripts/bootstrap_remote_assets.py` for the same preparation flow.

## Credential Gate

Before the first real PAI DLC submission, check whether these four values are already available:

- `ALIBABA_CLOUD_ACCESS_KEY_ID`
- `ALIBABA_CLOUD_ACCESS_KEY_SECRET`
- `PAI_REGION`
- `PAI_WORKSPACE_ID`

If any are missing:

1. Create `configs/pai.required.env` with blank placeholders.
2. Tell the user to fill that file.
3. Tell the user to reply with exactly `continue`.
4. Stop before real submission.

Do not skip this pause.

`MODELSCOPE_API_TOKEN` is still optional for the gate itself, but it is required later for remote asset upload or automatic publication.

## Primary Scripts

- `scripts/training_workspace.py`
  Natural-language request to config files.
- `scripts/submit_pai_dlc.py`
  Remote submission and polling.
- `scripts/train_sft_example.py`
- `scripts/train_dpo_example.py`
- `scripts/train_grpo_example.py`
- `scripts/dataset_inspector.py`
- `scripts/estimate_cost.py`
- `scripts/diagnose_dlc_report.py`
- `scripts/convert_to_gguf.py`

## Workflow

1. Read the user request and infer method, model, dataset, and training parameters.
2. Run `scripts/training_workspace.py --request "<user text>"`.
3. Inspect the generated plan and env files.
4. If `configs/pai.required.env` was created, stop and wait for `continue`.
5. Run a pilot job first:
   - `uv run scripts/submit_pai_dlc.py --env-file configs/remote.auto.env --mode pilot --wait --retry-once`
6. Only after a successful pilot, run the full job:
   - `uv run scripts/submit_pai_dlc.py --env-file configs/remote.auto.env --mode full --wait`
7. Publish checkpoints to ModelScope when token and target repo are available.

## Method Rules

- Use `SFT` by default.
- Use `DPO` only when the user explicitly asks for it or the data clearly represents preference pairs.
- Use `GRPO` only when the user explicitly asks for reinforcement-style optimization or reward-based training.
- When method choice is ambiguous, prefer `SFT`.

The generated default datasets follow ms-swift-friendly schemas:

- `SFT`: chat `messages`
- `DPO`: `messages` plus `rejected_response`
- `GRPO`: `messages` plus `solution`

## Command Rules

- `SFT` uses `swift sft`.
- `DPO` and `GRPO` use `swift rlhf --rlhf_type dpo|grpo`.
- Keep remote `user_command` short and deterministic.
- Use `REMOTE_ASSET_PATHS` when local files must be visible inside PAI DLC.
- If the user provides a local dataset path anywhere in the workspace or as an absolute path, `scripts/training_workspace.py` should automatically append it to `REMOTE_ASSET_PATHS` and rewrite the training dataset path to `./remote_assets/...`.
- Never pretend a local path is available remotely without packaging or upload.

## Defaults And References

- Low-risk defaults live in `scripts/training_workspace.py`.
- Read `references/training_methods.md` when method choice is uncertain.
- Read `references/troubleshooting.md` when a remote job fails.
- Read `references/pai_dlc_remote_one_shot.md` for remote submission details.
- Read `references/gguf_conversion.md` only when export is requested.

## Guardrails

- Never force the user into a rigid prompt template.
- Never skip the credential gate.
- Never claim training has started if you only wrote config files.
- Never overwrite an existing publish target without explicit approval.
- Always keep a pilot stage before a costly full run unless the user explicitly asks to skip it.
