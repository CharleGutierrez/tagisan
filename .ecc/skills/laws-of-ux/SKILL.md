---
name: "laws-of-ux"
description: "Cognitive psychology, ergonomics standards, and behavioral UX principles based on Laws of UX by Jon Yablonski. Use when architecting user flows, optimizing cognitive load, enforcing Doherty threshold responsiveness (<400ms), applying Hick's law to decision menus, and sizing touch targets via Fitts's law."
---

# Laws of UX Engineering Skill

Cognitive psychology and human-computer interaction heuristics codified into actionable UI engineering practices, derived from Jon Yablonski's *Laws of UX*.

---

## 1. Key Laws & Engineering Implementations

### 1. Doherty Threshold (<400ms)
- **Law**: Productivity soars when a computer and its users interact at a pace (<400ms) that ensures neither has to wait on the other.
- **Implementation**: Provide instant visual feedback (<100ms) for any action. For operations lasting >400ms, display deterministic progress bars; for operations <400ms, skip spinners entirely to prevent flicker.

### 2. Hick's Law (Choice Minimization)
- **Law**: The time it takes to make a decision increases logarithmically with the number and complexity of choices.
- **Implementation**: Break complex multi-step forms into sequential chunks (wizards). Limit primary actions in any view to **1 primary**, at most **2 secondary**, and group overflow actions in a dropdown menu.

### 3. Fitts's Law (Target Accessibility)
- **Law**: The time to acquire a target is a function of the distance to and width of the target.
- **Implementation**: Ensure touch targets are at least **44x44px** (iOS HIG) or **48x48px** (Material). Place critical actions at the viewport perimeter or bottom on mobile devices where thumb reach is fastest.

### 4. Miller's Law (Chunking: 7 ± 2)
- **Law**: The average person can only keep 7 (plus or minus 2) items in their working memory.
- **Implementation**: Structure data displays into visually delineated clusters of 5 to 7 elements. Format long strings (phone numbers, API keys, credit cards, hashes) into hyphenated chunks.

### 5. Jakob's Law (Mental Models)
- **Law**: Users spend most of their time on other sites, meaning they prefer your site to work the same way as all the others they already know.
- **Implementation**: Do not reinvent common design patterns. Place search in the header, profile top-right, navigation top or left, shopping cart top-right.

### 6. Aesthetic-Usability Effect
- **Law**: Users often perceive aesthetically pleasing design as design that's more usable and error-tolerant.
- **Implementation**: Invest in typography consistency, harmonious spacing, smooth transitions, and high-fidelity iconography.

### 7. Peak-End Rule
- **Law**: People judge an experience largely based on how they felt at its peak and at its end.
- **Implementation**: Ensure the completion state of critical workflows (checkout, deployment, test run) delivers a memorable, positive celebration (clear summary, copyable outputs, confetti).

---

## 2. Quantitative UI Audit Checklist

| Metric | Target | Verification |
| :--- | :--- | :--- |
| First Input Delay (FID) | < 100ms | Performance profiler |
| Interactive Feedback | < 50ms | CSS active state audit |
| Touch Target Minimum | 44x44px | CSS box-model inspection |
| Max Choices per Context | <= 5 | Component action review |
| Line Reading Length | 45–75 chars | Column width constraints |

---

## 3. Vibe Coder Directives for AI Prompts

- *"Refactor this dashboard view according to Hick's Law: condense the 12 floating action buttons into 1 primary 'Deploy' button and an overflow 'Actions' menu."*
- *"Audit touch target accessibility according to Fitts's Law: wrap all mobile nav icons in minimum 44x44px click boundaries."*
- *"Apply Miller's Law chunking to this metrics table: group related statistics into 3 distinct visual cards with clear headers."*
