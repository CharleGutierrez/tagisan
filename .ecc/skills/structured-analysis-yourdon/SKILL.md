---
name: structured-analysis-yourdon
description: "Modern Structured Analysis (Edward Yourdon): environmental modeling, context diagrams, event-response lists, leveled DFDs, and state transition diagrams for AI-driven software architecture."
triggers: ["structured analysis", "yourdon", "context diagram", "event list", "data flow diagram", "dfd", "environmental model", "behavioral model", "process specification"]
---

# Modern Structured Analysis & Design (Edward Yourdon)

This skill instructs the agent to apply the rigorous methodologies of Edward Yourdon's *Modern Structured Analysis* to decompose complex software requirements into deterministic, testable specifications.

## 1. The Environmental Model (System Boundary Definition)
Before writing any implementation code or detailed schemas:
1. **Statement of Purpose**: Define a concise 1-2 sentence statement defining the exact objective and boundaries of the system.
2. **Context Diagram (DFD Level-0)**:
   - Represent the entire system as a single central process (Circle 0).
   - Identify all external entities/terminators (users, external databases, payment gateways, regulatory systems).
   - Draw all incoming stimulus flows (inputs) and outgoing response flows (outputs).
3. **Event-Response List**: Categorize all stimuli into an exhaustive table:
   - **External Events**: Initiated by external actors (e.g., *Litigant files petition*, *Clerk enters assessment*).
   - **Temporal Events**: Initiated by the passage of time (e.g., *Daily raffle cutoff at 14:00*, *30-day summons expiration*).
   - **State Events**: Initiated by internal threshold transitions (e.g., *Escrow balance drops below zero*).

## 2. The Behavioral Model (Leveled Data Flow Diagrams)
1. **Event Partitioning (DFD Level-1)**:
   - For every event in the Event List, define exactly one process bubble to handle the stimulus.
   - Define the inputs required by the process and the outputs emitted.
   - Define the shared Data Stores (databases, memory tables, file stores) accessed by multiple processes.
2. **Level-2 Decomposition**:
   - Decompose any complex Level-1 process bubble that contains multiple logical sub-tasks until each leaf bubble represents a single cohesive transformation.

## 3. State Transition Diagrams (STDs)
For time-dependent, concurrent, or asynchronous workflows:
1. Define discrete, mutually exclusive system states.
2. For every arrow between states, explicitly annotate:
   `Event [Condition] / Action -> NextState`
3. Verify that every possible event has an explicit handling path (transition, ignore, or error); never leave unhandled state transitions.

## 4. AI Vibe Coding Verification Rules
- **Rule 1**: Reject vague prompts. If an objective is given without clear boundaries, formulate the Context Diagram and Event List first.
- **Rule 2**: Never generate code spanning multiple DFD bubbles in a single file without clear modular boundaries.
- **Rule 3**: Ensure all data stores have explicit read/write schemas before writing processing logic.
