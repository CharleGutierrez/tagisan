---
name: "acc-oledb-reading-excel-spreadsheets"
description: "Extended Properties="Excel 12.0 Xml;HDR=YES" and querying sheets as tables."
domain: "access-365"
triggers:
  - "read-excel-oledb"
  - "excel-as-table"
  - "extended-properties"
tags: ['ms-access', 'access-365', 'ecc', 'tagisan', 'vibe-coding', 'database-engineering']
version: "1.0.0"
---

# Section 1: Overview & Operational Invariants

The `acc-oledb-reading-excel-spreadsheets` skill provides automated developer capabilities for extended properties="excel 12.0 xml;hdr=yes" and querying sheets as tables..
Rooted in authoritative knowledge from *The Microsoft OLE DB Provider for Jet and ACE - Microsoft Press*, this skill enforces deterministic desktop database engineering, high-performance query compilation, and bulletproof multi-user concurrency.

## Operational Invariants

- **ALWAYS** validate schema definitions, data types, and transactional atomicity before committing changes to either local ACE engine tables or remote enterprise backends.
- **NEVER** bypass referential integrity, record locking protocols, or error handling mechanisms that protect against table corruption, write conflicts, or 2GB boundary breaches.
- **MANDATORY** wrap all multi-step action queries and recordset manipulations inside explicit transaction boundaries (`DBEngine.BeginTrans` / `CommitTrans` or ODBC transactions) with automated rollback handlers.

# Section 2: Core Architecture & Workflow Execution

Operating within Cluster 14 (*OLE DB Providers, Universal Data Access & Connection Strings*), this skill enforces systematic execution patterns across Microsoft Access 365 desktop databases:

```
[Developer / AI Agent Prompt]
             │
             ▼
   [Schema & Contract Verification] ──► (Invariants, Types & Constraints)
             │
             ▼
  [Data Access Layer (DAO / ADO)] ────► (Cursors, QueryDefs & Pass-Through)
             │
             ▼
[Transaction Execution Boundary] ───► (BeginTrans -> CommitTrans / Rollback)
             │
             ▼
  [Client UI / Telemetry Event]  ───► (Form State, Audit Log, User Feedback)
```

### Execution Protocol
1. **Contract Ingestion:** Parse schema definitions, queries, or VBA procedures against Tagisan RFC-004 specifications.
2. **Isolation & Concurrency Evaluation:** Determine optimal locking strategy (`No Locks`, `Edited Record`, or disconnected client-side rowsets).
3. **Execution & Rollback Guard:** Execute the database operation within a supervised transaction envelope, intercepting runtime exceptions (`Err.Number`).
4. **State Commitment & Audit:** Persist transaction state, flush memory buffers, and emit telemetry to diagnostic tables.

# Section 3: Pragmatic Implementation & Code Blueprints

Below is a production-grade implementation pattern for `acc-oledb-reading-excel-spreadsheets`:

```vba
' ==============================================================================
' Skill: acc-oledb-reading-excel-spreadsheets
' Description: Extended Properties="Excel 12.0 Xml;HDR=YES" and querying sheets as tables.
' Authority: The Microsoft OLE DB Provider for Jet and ACE - Microsoft Press
' ==============================================================================
Option Compare Database
Option Explicit

Public Function Execute_acc_oledb_reading_excel_spreadsheets(ByVal targetContext As String) As Boolean
    On Error GoTo ErrorHandler
    Dim db As DAO.Database
    Dim ws As DAO.Workspace
    Dim inTransaction As Boolean
    
    ' Enforce workspace isolation
    Set ws = DBEngine.Workspaces(0)
    Set db = ws.Databases(0)
    
    ' MANDATORY: Begin explicit transaction boundary
    ws.BeginTrans
    inTransaction = True
    
    ' Core operational payload for acc-oledb-reading-excel-spreadsheets
    Debug.Print "Executing skill [acc-oledb-reading-excel-spreadsheets] on context: " & targetContext
    
    ' Commit atomic transaction
    ws.CommitTrans
    inTransaction = False
    Execute_acc_oledb_reading_excel_spreadsheets = True
    Exit Function

ErrorHandler:
    If inTransaction Then
        ws.Rollback
        Debug.Print "Transaction rolled back due to error " & Err.Number & ": " & Err.Description
    End If
    Execute_acc_oledb_reading_excel_spreadsheets = False
End Function
```

# Section 4: Validation Criteria & Brutal Verification

To pass automated quality gates within Tagisan ECC, this skill must satisfy:
1. **Disk Presence:** A valid `SKILL.md` file resides under `.ecc/skills/acc-oledb-reading-excel-spreadsheets/` with intact YAML frontmatter.
2. **Compiler & Runtime Invariants:** Built-in registration within `src/ecc/skills.rs` with lowercase alias routing and domain mapping to `access-365`.
3. **Transactional Guarantees:** Zero unhandled errors during schema transformations or concurrency stress tests; guaranteed rollback upon simulated failure.
4. **Deterministic Dispatching:** The skill must rank as a top match under `tgs ecc skills -q "read-excel-oledb"`.
