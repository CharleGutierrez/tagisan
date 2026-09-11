---
name: "about-face-interaction-design"
description: "Advanced interaction design, software posture theory, ergonomic workspace architecture, and state resilience based on About Face: The Essentials of Interaction Design by Alan Cooper. Use when designing complex desktop/web software, sovereign power-user applications, multi-pane workspaces, undo/redo state stacks, command palettes, and non-modal feedback systems."
---

# About Face & Power-User Ergonomics Engineering Skill

Interaction design engineering for professional, high-density applications, rooted in Alan Cooper's Goal-Directed Design and Software Posture Theory.

---

## 1. Software Posture Theory

Different software requires distinct interaction ergonomics based on how users engage with it:

1. **Sovereign Posture**:
   - **Context**: Applications that monopolize user attention for hours at a time (e.g. IDEs, trading terminals, creative suites, CAD, Tagisan TUI/dashboard).
   - **Ergonomics**: High information density, subdued/dark background palettes to prevent eye fatigue, deep keyboard shortcut accessibility, customizable multi-pane layouts.
2. **Transient Posture**:
   - **Context**: Single-function utilities invoked briefly to accomplish a quick task, then dismissed (e.g. calculator, file uploader, settings dialog).
   - **Ergonomics**: Clear, large visual signifiers, minimal controls, immediate obvious exit routes.
3. **Daemonic Posture**:
   - **Context**: Background processes without continuous UI presence (e.g. agent schedulers, sync workers).
   - **Ergonomics**: Status-bar indicators, non-intrusive toast notifications, comprehensive error logs.

---

## 2. Elimination of Excise

**Excise** is the extra effort users must expend in order to use software that does not directly contribute to achieving their goals.
- **Eliminate Confirmation Dialogs**: Instead of asking *"Are you sure you want to delete this?"*, execute the deletion immediately and provide a persistent, non-modal **Undo** banner (`toast.undo("Task deleted", onUndo)`).
- **Remember User State**: Always persist window sizes, split-pane ratios, scroll positions, and unsubmitted draft text to local storage. Never wipe user state on refresh or navigation.

---

## 3. Command Palettes & Keyboard-First Navigation

For sovereign power-user applications, every action accessible via mouse must be accessible via keyboard:
- Universal Command Palette bound to `Cmd+K` / `Ctrl+K`.
- Sequential spatial tab navigation (`tabIndex` management).
- Clear mnemonic keyboard hints displayed on hover (`tooltip: "Run Test (Ctrl+Enter)"`).

---

## 4. Multi-Pane Inspector Ergonomics

- **Hierarchy**: Left-to-right visual flow (Navigation Sidebar -> Master List / Graph -> Detail / Inspector Panel).
- **Independent Scroll Regions**: Never scroll the entire page; ensure each pane maintains its own bounded scroll context (`overflow-y-auto`).

---

## 5. Vibe Coder Directives for AI Prompts

- *"Eliminate modal delete confirmations: perform optimistic deletion and display an undo banner with a 5-second countdown."*
- *"Implement a global `Cmd+K` command palette indexing all workspace actions, agent dispatchers, and project navigation."*
- *"Persist split-pane widths, active tabs, and filter selections in local storage so the workspace restores exactly across sessions."*
