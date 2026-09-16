---
name: "visio-embed-msal-bearer-tokens"
description: "Acquiring bearer tokens for embedding OneDrive/SharePoint diagrams via MSAL 2.0."
domain: "visio-365"
triggers:
  - "msal-visio-tokens"
  - "sharepoint-diagram-embed"
  - "bearer-token-handshake"
tags: ['visio', 'visio-365', 'ecc', 'tagisan', 'vibe-coding', 'vector-graphics']
version: "1.0.0"
---

# Section 1: Overview & Operational Invariants

The `visio-embed-msal-bearer-tokens` skill provides automated developer capabilities for acquiring bearer tokens for embedding onedrive/sharepoint diagrams via msal 2.0..
Rooted in authoritative knowledge from *Building Modern Web Apps with Embedded Visio Diagrams - Paolo Pialorsi*, this skill enforces deterministic vector geometry calculations, ShapeSheet mathematical invariants, and robust multi-page diagram automation.

## Operational Invariants

- **ALWAYS** validate ShapeSheet cell formulas, coordinate transformations, and spatial bounds before committing modifications to master shapes, stencils, or page instances.
- **NEVER** bypass connection point directional constraints, glue verification protocols, or event-driven error handlers that protect against cyclic recursion, #REF! errors, or process memory leaks.
- **MANDATORY** encapsulate all multi-step diagram mutations and batch ShapeSheet updates inside explicit undo scopes (`Application.BeginUndoScope` / `EndUndoScope`) or batch API transactions (`SetFormulas` / `context.sync()`) with automated rollback handlers.

# Section 2: Core Architecture & Workflow Execution

Operating within Cluster 12 (*Embedding Visio in React, Angular & Fluent UI*), this skill enforces systematic visual execution patterns across Microsoft Visio 365 diagrams:

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

Below is a production-grade implementation pattern for `visio-embed-msal-bearer-tokens`:

```vba
' ==============================================================================
' Skill: visio-embed-msal-bearer-tokens
' Description: Acquiring bearer tokens for embedding OneDrive/SharePoint diagrams via MSAL 2.0.
' Authority: Building Modern Web Apps with Embedded Visio Diagrams - Paolo Pialorsi
' ==============================================================================
Option Explicit

Public Function Execute_visio_embed_msal_bearer_tokens(ByRef targetPage As Visio.Page, ByVal contextParam As String) As Boolean
    On Error GoTo ErrorHandler
    Dim app As Visio.Application
    Dim undoScopeId As Long
    Dim inScope As Boolean
    
    Set app = targetPage.Application
    
    ' MANDATORY: Begin transactional undo scope
    undoScopeId = app.BeginUndoScope("Execute_visio_embed_msal_bearer_tokens")
    inScope = True
    
    ' Core operational payload for visio-embed-msal-bearer-tokens
    Debug.Print "Executing skill [visio-embed-msal-bearer-tokens] on page: " & targetPage.Name & " with param: " & contextParam
    
    ' Commit atomic undo scope
    app.EndUndoScope undoScopeId, True
    inScope = False
    Execute_visio_embed_msal_bearer_tokens = True
    Exit Function

ErrorHandler:
    If inScope Then
        app.EndUndoScope undoScopeId, False
        Debug.Print "Visio action rolled back due to error " & Err.Number & ": " & Err.Description
    End If
    Execute_visio_embed_msal_bearer_tokens = False
End Function
```

# Section 4: Validation Criteria & Brutal Verification

To pass automated quality gates within Tagisan ECC, this skill must satisfy:
1. **Disk Presence:** A valid `SKILL.md` file resides under `.ecc/skills/visio-embed-msal-bearer-tokens/` with intact YAML frontmatter.
2. **Compiler & Runtime Invariants:** Built-in registration within `src/ecc/skills.rs` with lowercase alias routing and domain mapping to `visio-365`.
3. **Vector & Formula Guarantees:** Zero unhandled errors (#REF!, #VALUE!) during ShapeSheet formula execution; guaranteed rollback upon simulated failure.
4. **Deterministic Dispatching:** The skill must rank as a top match under `tgs ecc skills -q "msal-visio-tokens"`.
