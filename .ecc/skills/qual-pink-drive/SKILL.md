---
name: qual-pink-drive
description: "Intrinsic motivation (Motivation 3.0): The fatal flaws of carrot-and-stick extrinsic rewards; the three pillars of Autonomy, Mastery, and Purpose."
triggers: ["daniel-pink", "drive", "intrinsic-motivation", "autonomy-mastery-purpose", "motivation-3-0", "competence-feedback"]
---

# qual-pink-drive
> Based on **Drive: The Surprising Truth About What Motivates Us - Daniel H. Pink**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Provide users with sovereign autonomy over their workspace (keyboard shortcuts, custom layouts, export formats).**
2. **ALWAYS: Design clear, transparent feedback loops that signal progressive mastery and competence over the tool.**
3. **NEVER: Replace genuine product utility or mastery with superficial extrinsic gamification badges, points, or leaderboards.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Foster intrinsic motivation by granting user autonomy, clear feedback on functional mastery, and transparency of purpose across all system operations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Adding meaningless digital confetti and badges for routine data entry.**
- **Locking workspace layout customization behind arbitrary paywalls.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-pink-drive"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
