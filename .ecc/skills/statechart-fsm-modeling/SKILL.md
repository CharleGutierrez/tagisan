---
name: statechart-fsm-modeling
description: "Hierarchical Statecharts & FSM Modeling (David Harel & Ian Horrocks): finite state machines, orthogonal regions, guarded transitions, entry/exit actions, and deadlock-free event lifecycles."
triggers: ["statechart", "fsm", "finite state machine", "harel statechart", "horrocks", "state machine", "workflow state", "orthogonal regions", "guarded transitions"]
---

# Hierarchical Statecharts & FSM Modeling (David Harel & Ian Horrocks)

This skill instructs the agent to design complex stateful systems, asynchronous workflows, and business lifecycles using Harel Statecharts and Ian Horrocks' UI/System Statechart architecture.

## 1. Statechart Formalisms (Beyond Flat FSMs)
Flat state machines suffer from exponential state explosion. Use Harel's hierarchical extensions:
1. **Hierarchical States (Superstates & Substates)**:
   - Group related states into a parent superstate.
   - Transitions from a superstate apply to all its substates (eliminating duplicate transition lines).
   - Example: Superstate `InTrial` contains substates `PreTrialConference`, `PresentationOfEvidence`, and `SubmittedForDecision`.
2. **Orthogonal (Concurrent) Regions**:
   - Model parallel independent aspects of an entity within the same lifecycle.
   - Example: A court case has two orthogonal regions:
     - *Judicial Track*: `[PendingRaffle -> InBranch -> Trial -> Promulgated -> Closed]`
     - *Financial Track*: `[AssessmentPending -> PartiallyPaid -> FullyPaid -> Audited]`
3. **Guarded Transitions**:
   - Transitions trigger only when: `Event [GuardCondition] / Action -> TargetState`.
   - If the condition is false, the event is ignored or rejected; state remains untouched.
4. **Entry and Exit Actions**:
   - Execute actions automatically on entering or exiting a state, guaranteeing cleanup and initialization invariants.

## 2. Determinism and Exhaustiveness
1. **Run-to-Completion (RTC) Semantics**:
   - An event must be fully processed and all entry/exit actions executed before the next event can be accepted.
2. **Exhaustive Event Coverage**:
   - In type systems (Rust `match`, TypeScript `switch`), make states and events explicit `enum` variants.
   - For every state, define explicit handling for all possible events. Never allow an unexpected event to crash or hang the system.
3. **History States**:
   - Use shallow or deep history (`H`) to resume a substate where it left off when returning from an interruption.

## 3. Implementation in Code
- Model states as closed discriminated unions:
  ```rust
  pub enum CaseStatus {
      Draft(DraftMetadata),
      Filed { docket_no: DocketNo, filed_at: Instant },
      Assessed { fees: FeeBreakdown },
      Raffled { branch_id: BranchId, date: NaiveDate },
      Archived { reason: String },
  }
  ```
- Make invalid transitions unrepresentable in the type system.
