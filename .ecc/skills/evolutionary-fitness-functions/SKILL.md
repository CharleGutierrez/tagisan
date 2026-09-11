---
name: evolutionary-fitness-functions
description: "Building Evolutionary Architectures (Neal Ford, Rebecca Parsons, Patrick Kua): architectural fitness functions, automated structural verification, boundary integrity, and preventing architectural drift across AI iterations."
triggers: ["evolutionary architecture", "fitness functions", "architectural fitness functions", "neal ford", "patrick kua", "architectural drift", "dependency rules", "layering rules", "architectural integrity"]
---

# Evolutionary Architecture and Fitness Functions (Ford, Parsons, Kua)

This skill equips the agent with the disciplines of *Building Evolutionary Architectures* to automate architectural governance and preserve system integrity across continuous AI-driven modifications.

## 1. The Concept of Architectural Fitness Functions
- **Definition**: Any mechanism that provides an objective, automated integrity assessment of an architectural characteristic (maintainability, scalability, modularity, security, performance).
- **Purpose in AI Development**: AI agents naturally create architectural drift over multiple generations of prompts. Fitness functions act as immutable automated fences guarding the design.

## 2. Categories of Fitness Functions
1. **Structural and Layering Fitness Functions**:
   - **Strict Dependency Direction**: Outer layers (CLI, Web, MCP) may depend on Domain models; Domain models must NEVER import outer layer packages (Hexagonal / Clean Architecture).
   - **Acyclic Dependency Enforcement**: Verify that package/crate dependencies form a Directed Acyclic Graph (DAG) with zero circular references.
   - **Package Encapsulation**: Prevent leaking private subsystem structures to unauthorized consumers.
2. **Complexity Fitness Functions**:
   - Automated checks asserting that no single function exceeds cyclomatic complexity > 10 or line length > 60 lines.
   - Assert file size bounds (< 400 lines) to prevent the emergence of God Modules.
3. **Performance and Resource Fitness Functions**:
   - Automated benchmark assertions: fail CI if a critical hot-path operation (e.g. DAG scheduling, skill dispatch) exceeds its p99 latency budget.
4. **Security and Vulnerability Fitness Functions**:
   - Automated dependency audit gates (cargo-deny, npm audit, license compliance gates).

## 3. Operationalizing Fitness Functions in Code
- Encode architectural fitness functions directly into the repository's test suite:
  - Unit/Integration tests that scan codebase AST / imports and assert topological rules.
  - Pre-commit and CI verification gates that run continuously.
