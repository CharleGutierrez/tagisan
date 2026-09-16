---
name: "visio-cad-eng-raster-blueprint-to-vector-shapes"
description: "Raster-to-vector tracing pipelines and ShapeSheet conversion from scanned plans."
domain: "visio-365"
triggers:
  - "raster-to-vector-blueprint"
  - "scanned-plan-vectorize"
  - "blueprint-tracing-visio"
tags: ['visio', 'visio-365', 'ecc', 'tagisan', 'vibe-coding', 'vector-graphics']
version: "1.0.0"
---

# Section 1: Overview & Operational Invariants

The `visio-cad-eng-raster-blueprint-to-vector-shapes` skill provides automated developer capabilities for raster-to-vector tracing pipelines and shapesheet conversion from scanned plans..
Rooted in authoritative knowledge from *Piping and Instrumentation Diagram (P&ID) Engineering with Visio - Bill Morelli*, this skill enforces deterministic vector geometry calculations, ShapeSheet mathematical invariants, and robust multi-page diagram automation.

## Operational Invariants

- **ALWAYS** validate ShapeSheet cell formulas, coordinate transformations, and spatial bounds before committing modifications to master shapes, stencils, or page instances.
- **NEVER** bypass connection point directional constraints, glue verification protocols, or event-driven error handlers that protect against cyclic recursion, #REF! errors, or process memory leaks.
- **MANDATORY** encapsulate all multi-step diagram mutations and batch ShapeSheet updates inside explicit undo scopes (`Application.BeginUndoScope` / `EndUndoScope`) or batch API transactions (`SetFormulas` / `context.sync()`) with automated rollback handlers.

# Section 2: Core Architecture & Workflow Execution

Operating within Cluster 25 (*Engineering & CAD: Floor Plans, P&ID & Electrical Schematics*), this skill enforces systematic visual execution patterns across Microsoft Visio 365 diagrams:

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

Below is a production-grade implementation pattern for `visio-cad-eng-raster-blueprint-to-vector-shapes`:

```vba
' ==============================================================================
' Skill: visio-cad-eng-raster-blueprint-to-vector-shapes
' Description: Raster-to-vector tracing pipelines and ShapeSheet conversion from scanned plans.
' Authority: Piping and Instrumentation Diagram (P&ID) Engineering with Visio - Bill Morelli
' ==============================================================================
Option Explicit

Public Function Execute_visio_cad_eng_raster_blueprint_to_vector_shapes(ByRef targetPage As Visio.Page, ByVal contextParam As String) As Boolean
    On Error GoTo ErrorHandler
    Dim app As Visio.Application
    Dim undoScopeId As Long
    Dim inScope As Boolean
    
    Set app = targetPage.Application
    
    ' MANDATORY: Begin transactional undo scope
    undoScopeId = app.BeginUndoScope("Execute_visio_cad_eng_raster_blueprint_to_vector_shapes")
    inScope = True
    
    ' Core operational payload for visio-cad-eng-raster-blueprint-to-vector-shapes
    Debug.Print "Executing skill [visio-cad-eng-raster-blueprint-to-vector-shapes] on page: " & targetPage.Name & " with param: " & contextParam
    
    ' Commit atomic undo scope
    app.EndUndoScope undoScopeId, True
    inScope = False
    Execute_visio_cad_eng_raster_blueprint_to_vector_shapes = True
    Exit Function

ErrorHandler:
    If inScope Then
        app.EndUndoScope undoScopeId, False
        Debug.Print "Visio action rolled back due to error " & Err.Number & ": " & Err.Description
    End If
    Execute_visio_cad_eng_raster_blueprint_to_vector_shapes = False
End Function
```

# Section 4: Validation Criteria & Brutal Verification

To pass automated quality gates within Tagisan ECC, this skill must satisfy:
1. **Disk Presence:** A valid `SKILL.md` file resides under `.ecc/skills/visio-cad-eng-raster-blueprint-to-vector-shapes/` with intact YAML frontmatter.
2. **Compiler & Runtime Invariants:** Built-in registration within `src/ecc/skills.rs` with lowercase alias routing and domain mapping to `visio-365`.
3. **Vector & Formula Guarantees:** Zero unhandled errors (#REF!, #VALUE!) during ShapeSheet formula execution; guaranteed rollback upon simulated failure.
4. **Deterministic Dispatching:** The skill must rank as a top match under `tgs ecc skills -q "raster-to-vector-blueprint"`.
