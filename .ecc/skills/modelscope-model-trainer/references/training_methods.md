# Training Methods

## SFT

Use when you have instruction-response pairs and need baseline adaptation.

## DPO

Use when you have preference pairs (`chosen`/`rejected`) and want alignment without reward model training.

## GRPO

Use when reinforcement-style optimization is needed with group-relative rewards.

## Selection Rule

- Default to SFT.
- Use DPO only when preference-pair data exists (for example `chosen`/`rejected` fields).
- If data schema is ambiguous, run SFT pilot first and postpone DPO.
- Use GRPO only when reward signals justify extra complexity.
