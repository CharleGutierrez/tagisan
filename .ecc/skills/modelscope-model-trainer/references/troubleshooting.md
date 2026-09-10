# Troubleshooting

## GPU Quota / `resourceLimit`

- Symptom: job fails at `Creating` with resource threshold message.
- Fix:
  - Set `PAI_DLC_GPU=0`
  - Set `PAI_DLC_CPU=4`
  - Set `PAI_DLC_MEMORY=16Gi`
  - Keep pilot short (`MAX_STEPS` <= 10)

## `bf16` Not Supported

- Symptom: `ValueError: Your setup doesn't support bf16/gpu.`
- Fix:
  - Add `--bf16 false --fp16 false --torch_dtype float32`
  - Use these flags by default on CPU or unknown GPU generations.

## Env Preparing Timeout / Image Pull Failures

- Symptom: long `EnvPreparing`, `enterRunningTimeout`, image pull backoff/not found.
- Fix:
  - Use a verified image tag in target region (for example `pytorch/pytorch:latest`).
  - Avoid custom image tags unless tested in-region.

## Pod Exits After Pip Install Only

- Symptom: logs show dependency installation succeeded, then pod exits without training logs.
- Root cause pattern: startup command was truncated or did not include training entrypoint.
- Fix:
  - Keep startup `user_command` short.
  - Ensure command explicitly runs `swift sft` (or other training command).
  - Avoid embedding large base64 payloads in one shell command.

## OOM During Training

- Reduce sequence length.
- Reduce micro-batch size.
- Increase gradient accumulation.
- Prefer LoRA / QLoRA for pilot.

## Diverging Loss

- Lower learning rate.
- Check dataset formatting and label leakage.
- Validate tokenizer and special tokens.

## Upload Failures

- Confirm token scope and namespace.
- Retry with smaller upload folders.
- If token is missing, mark publish as skipped instead of failing training result.
