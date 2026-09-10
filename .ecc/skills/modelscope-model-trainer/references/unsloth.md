# Unsloth Notes

Use `unsloth` when you need memory-efficient LoRA-style fine-tuning.

Guidelines:

- Validate kernel compatibility with your CUDA/PyTorch stack.
- Keep fallback path to standard transformers training.
- Publish merged and adapter-only artifacts separately.
