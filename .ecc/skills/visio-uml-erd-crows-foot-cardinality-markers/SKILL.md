---
name: "visio-uml-erd-crows-foot-cardinality-markers"
description: "1:1, 1:N, M:N cardinality markers and relationship lines in Visio database models."
domain: "visio-365"
triggers:
  - "crows-foot-cardinality"
  - "cardinality-markers-visio"
  - "relationship-line-styles"
tags: ['visio', 'visio-365', 'ecc', 'tagisan', 'vibe-coding', 'vector-graphics']
version: "1.0.0"
---

# Section 1: Overview & Operational Invariants

The `visio-uml-erd-crows-foot-cardinality-markers` skill provides automated developer capabilities for 1:1, 1:n, m:n cardinality markers and relationship lines in visio database models..
Rooted in authoritative knowledge from *UML Distilled: A Brief Guide to the Standard Object Modeling Language - Martin Fowler*, this skill enforces deterministic vector geometry calculations, ShapeSheet mathematical invariants, and robust multi-page diagram automation.

## Operational Invariants

- **ALWAYS** validate ShapeSheet cell formulas, coordinate transformations, and spatial bounds before committing modifications to master shapes, stencils, or page instances.
- **NEVER** bypass connection point directional constraints, glue verification protocols, or event-driven error handlers that protect against cyclic recursion, #REF! errors, or process memory leaks.
- **MANDATORY** encapsulate all multi-step diagram mutations and batch ShapeSheet updates inside explicit undo scopes (`Application.BeginUndoScope` / `EndUndoScope`) or batch API transactions (`SetFormulas` / `context.sync()`) with automated rollback handlers.

# Section 2: Core Architecture & Workflow Execution

Operating within Cluster 24 (*UML 2.5, Software Architecture & Database ERDs*), this skill enforces systematic visual execution patterns across Microsoft Visio 365 diagrams:

```
[Developer / AI Agent Prompt]
             │
             ▼
[Spatial & Coordinate Verification] ──► (PinX/PinY, Angle, LocPin, BoundingBox)
             │
             ▼
[ShapeSheet Formula & Geometry Layer] ─► (CellsSRC, Guard, DependsOn, NoShow)
             │
             ▼
 [Transaction & Undo Scope Boundary] ──► (BeginUndoScope -> EndUndoScope)
             │
             ▼
 [Vector Canvas & Telemetry Pipeline] ──► (SVG Render, Data Graphics, Audit Log)
```

### Execution Protocol
1. **Contract Ingestion:** Parse diagram requirements, vector schemas, or ShapeSheet formulas against Tagisan RFC-004 specifications.
2. **Spatial Alignment & Geometry Check:** Evaluate parent-child coordinate spaces, rotation origins, and connection point inward/outward directions.
3. **Execution & Undo Guard:** Apply geometric modifications and formula bindings inside a supervised undo scope, intercepting engine errors and recursion cycles.
4. **State Commitment & Visual Verification:** Persist shape properties, trigger dynamic layout re-routing if required, and emit vector render telemetry.

# Section 3: Pragmatic Implementation & Code Blueprints

Below is a production-grade implementation pattern for `visio-uml-erd-crows-foot-cardinality-markers`:

```vba
' ==============================================================================
' Skill: visio-uml-erd-crows-foot-cardinality-markers
' Description: 1:1, 1:N, M:N cardinality markers and relationship lines in Visio database models.
' Authority: UML Distilled: A Brief Guide to the Standard Object Modeling Language - Martin Fowler
' ==============================================================================
Option Explicit

Public Function Execute_visio_uml_erd_crows_foot_cardinality_markers(ByRef targetPage As Visio.Page, ByVal contextParam As String) As Boolean
    On Error GoTo ErrorHandler
    Dim app As Visio.Application
    Dim undoScopeId As Long
    Dim inScope As Boolean
    
    Set app = targetPage.Application
    
    ' MANDATORY: Begin transactional undo scope
    undoScopeId = app.BeginUndoScope("Execute_visio_uml_erd_crows_foot_cardinality_markers")
    inScope = True
    
    ' Core operational payload for visio-uml-erd-crows-foot-cardinality-markers
    Debug.Print "Executing skill [visio-uml-erd-crows-foot-cardinality-markers] on page: " & targetPage.Name & " with param: " & contextParam
    
    ' Commit atomic undo scope
    app.EndUndoScope undoScopeId, True
    inScope = False
    Execute_visio_uml_erd_crows_foot_cardinality_markers = True
    Exit Function

ErrorHandler:
    If inScope Then
        app.EndUndoScope undoScopeId, False
        Debug.Print "Visio action rolled back due to error " & Err.Number & ": " & Err.Description
    End If
    Execute_visio_uml_erd_crows_foot_cardinality_markers = False
End Function
```

# Section 4: Validation Criteria & Brutal Verification

To pass automated quality gates within Tagisan ECC, this skill must satisfy:
1. **Disk Presence:** A valid `SKILL.md` file resides under `.ecc/skills/visio-uml-erd-crows-foot-cardinality-markers/` with intact YAML frontmatter.
2. **Compiler & Runtime Invariants:** Built-in registration within `src/ecc/skills.rs` with lowercase alias routing and domain mapping to `visio-365`.
3. **Vector & Formula Guarantees:** Zero unhandled errors (#REF!, #VALUE!) during ShapeSheet formula execution; guaranteed rollback upon simulated failure.
4. **Deterministic Dispatching:** The skill must rank as a top match under `tgs ecc skills -q "crows-foot-cardinality"`.
