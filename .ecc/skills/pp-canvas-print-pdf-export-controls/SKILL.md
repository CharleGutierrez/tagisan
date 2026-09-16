---
name: pp-canvas-print-pdf-export-controls
description: "PDF() function configuration, printable screen layout stylesheets, page breaks, and tabular receipt/invoice generation."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["print-pdf-export", "pdf-function-power-fx", "printable-canvas-screens", "invoice-receipt-generation"]
---

# pp-canvas-print-pdf-export-controls

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- PDF Generation Engine: The `PDF()` function captures target screen or container DOM trees into portable PDF binary blobs.
- Layout Invariant: ALWAYS design dedicated printable screens formatted to standard dimensions (A4: 794x1123px or US Letter: 816x1056px).
- NEVER attempt to export responsive fluid auto-layout screens with dynamic scrollbars directly to PDF without print formatting.
- MANDATORY elimination of interactive controls (buttons, inputs) from the printable container via `DPI` and visibility flags.
- Source Reference: *Report Generation and PDF Export in Power Apps - Eickhel Mendoza*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Generate customer invoice PDF: `Set(varInvoiceBlob, PDF(PrintableInvoiceContainer, { ExpandContainers: true, Margin: '10px' }))`. Pass `varInvoiceBlob` to Power Automate or Office 365 Outlook connector for email attachment.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Invoking browser window.print() on unconstrained canvas app screens, producing clipped multi-page outputs.
- Attempting to paginate 1,000 records inside a single canvas PDF container instead of offloading to Power BI or SSRS.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("print-pdf-export", "pdf-function-power-fx", "printable-canvas-screens", "invoice-receipt-generation") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
