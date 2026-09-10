# Reliability Principles

- Reproducibility first: seed, config, and data revision must be pinned.
- Fail fast: run smoke tests before full epochs.
- Observable runs: capture training + validation metrics each checkpoint.
- Safe publishing: push only validated checkpoints.
- Remote-first execution on macOS: local host is control plane only.
- Budget-safe defaults: CPU pilot first when quota or budget is unclear.
- Command safety: keep remote startup commands short to avoid truncation.
- Recovery-first behavior: auto-apply one common fix and retry before stopping.
