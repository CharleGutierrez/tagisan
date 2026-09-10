# Training Patterns

## Pattern A: Remote SFT Pilot -> Full (Default)

- Generate/reuse minimal train+valid assets.
- Submit a low-cost pilot run (CPU-safe by default).
- Apply one auto-fix if pilot fails.
- Upgrade to full run after pilot success.
- Select last/best checkpoint and publish when token exists.

## Pattern B: Single-stage SFT

- Prepare cleaned dataset
- Train one adapter
- Evaluate and publish best checkpoint

## Pattern C: SFT -> DPO

- Build preference pairs from SFT outputs
- Continue from SFT checkpoint
- Compare alignment metrics before publishing

## Pattern D: Long-running GRPO

- Start from validated SFT/DPO checkpoint
- Use strict checkpoint cadence
- Track rollback point every N steps
