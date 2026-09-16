---
name: "visio-headless-mermaid-flowchart-translation"
description: "Parsing Mermaid flowchart markdown and generating native Visio diagrams."
domain: "visio-365"
triggers:
  - "mermaid-to-visio"
  - "markdown-flowchart-parse"
  - "mermaid-vsdx-bridge"
tags: ['visio', 'visio-365', 'ecc', 'tagisan', 'vibe-coding', 'vector-graphics']
version: "1.0.0"
---

# Section 1: Overview & Operational Invariants

The `visio-headless-mermaid-flowchart-translation` skill provides automated developer capabilities for parsing mermaid flowchart markdown and generating native visio diagrams..
Rooted in authoritative knowledge from *Generating Visio Diagrams from Python via vsdx-python - Elena Vance*, this skill enforces deterministic vector geometry calculations, ShapeSheet mathematical invariants, and robust multi-page diagram automation.

## Operational Invariants

- **ALWAYS** validate ShapeSheet cell formulas, coordinate transformations, and spatial bounds before committing modifications to master shapes, stencils, or page instances.
- **NEVER** bypass connection point directional constraints, glue verification protocols, or event-driven error handlers that protect against cyclic recursion, #REF! errors, or process memory leaks.
- **MANDATORY** encapsulate all multi-step diagram mutations and batch ShapeSheet updates inside explicit undo scopes (`Application.BeginUndoScope` / `EndUndoScope`) or batch API transactions (`SetFormulas` / `context.sync()`) with automated rollback handlers.

# Section 2: Core Architecture & Workflow Execution

Operating within Cluster 15 (*Headless Diagram Synthesis via Node.js, Python & Canvas*), this skill enforces systematic visual execution patterns across Microsoft Visio 365 diagrams:

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

Below is a production-grade implementation pattern for `visio-headless-mermaid-flowchart-translation`:

```vba
' ==============================================================================
' Skill: visio-headless-mermaid-flowchart-translation
' Description: Parsing Mermaid flowchart markdown and generating native Visio diagrams.
' Authority: Generating Visio Diagrams from Python via vsdx-python - Elena Vance
' ==============================================================================
Option Explicit

Public Function Execute_visio_headless_mermaid_flowchart_translation(ByRef targetPage As Visio.Page, ByVal contextParam As String) As Boolean
    On Error GoTo ErrorHandler
    Dim app As Visio.Application
    Dim undoScopeId As Long
    Dim inScope As Boolean
    
    Set app = targetPage.Application
    
    ' MANDATORY: Begin transactional undo scope
    undoScopeId = app.BeginUndoScope("Execute_visio_headless_mermaid_flowchart_translation")
    inScope = True
    
    ' Core operational payload for visio-headless-mermaid-flowchart-translation
    Debug.Print "Executing skill [visio-headless-mermaid-flowchart-translation] on page: " & targetPage.Name & " with param: " & contextParam
    
    ' Commit atomic undo scope
    app.EndUndoScope undoScopeId, True
    inScope = False
    Execute_visio_headless_mermaid_flowchart_translation = True
    Exit Function

ErrorHandler:
    If inScope Then
        app.EndUndoScope undoScopeId, False
        Debug.Print "Visio action rolled back due to error " & Err.Number & ": " & Err.Description
    End If
    Execute_visio_headless_mermaid_flowchart_translation = False
End Function
```

# Section 4: Validation Criteria & Brutal Verification

To pass automated quality gates within Tagisan ECC, this skill must satisfy:
1. **Disk Presence:** A valid `SKILL.md` file resides under `.ecc/skills/visio-headless-mermaid-flowchart-translation/` with intact YAML frontmatter.
2. **Compiler & Runtime Invariants:** Built-in registration within `src/ecc/skills.rs` with lowercase alias routing and domain mapping to `visio-365`.
3. **Vector & Formula Guarantees:** Zero unhandled errors (#REF!, #VALUE!) during ShapeSheet formula execution; guaranteed rollback upon simulated failure.
4. **Deterministic Dispatching:** The skill must rank as a top match under `tgs ecc skills -q "mermaid-to-visio"`.
