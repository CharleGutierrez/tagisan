# Model Architectures

Transformers.js supports ONNX-friendly transformer architectures for NLP, vision, audio, and multimodal tasks.

When adapting ModelScope repositories:

- Confirm the repository exports compatible ONNX files.
- Keep tokenizer and config files synchronized with weights.
- Validate outputs against Python reference inference.
