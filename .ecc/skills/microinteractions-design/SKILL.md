---
name: "microinteractions-design"
description: "Tactile feedback, physical spring dynamics, transition states, and microinteraction architecture based on Microinteractions: Designing with Details by Dan Saffer. Use when crafting button states, optimistic UI updates, spring-based animations, drag-and-drop feedback, skeleton loaders, and sub-50ms reactive interactions."
---

# Microinteractions & Tactile Feedback Engineering Skill

Engineering microinteractions that breathe life into digital interfaces, based on Dan Saffer's framework and contemporary tactile UI engineering standards.

---

## 1. 4-Part Microinteraction Anatomy

Every microinteraction comprises four essential components:
1. **Trigger**: Initiates the microinteraction. Can be user-initiated (click, hover, swipe, keypress) or system-initiated (threshold breached, sync completed, battery low).
2. **Rules**: Determines how the interaction functions, what state transitions occur, and constraints.
3. **Feedback**: Sensory notification of what is occurring (visual transform, haptic pulse, audio click). Must trigger in under **50ms**.
4. **Loops & Modes**: Meta-rules governing recurrence, persistence, or modal state changes (e.g. holding a button to engage rapid-fire increment).

---

## 2. Spring Physics vs Mechanical Easings

Linear transitions and rigid cubic-beziers feel robotic. Physical spring simulations introduce natural inertia, momentum, and elasticity.

### Spring Dynamics Standard
- **Stiffness ($k$)**: ~200–300 (controls rapidity of response).
- **Damping Ratio ($\zeta$)**: ~0.7–0.85 (prevents excessive jitter while maintaining smooth organic settling).
- **Mass ($m$)**: ~1.0.

```typescript
// Framer Motion / CSS spring specification for interactive components
export const springPresets = {
  snappy: { type: "spring", stiffness: 350, damping: 25 },
  bouncy: { type: "spring", stiffness: 400, damping: 15 },
  smooth: { type: "spring", stiffness: 200, damping: 30 }
};
```

---

## 3. Sub-50ms Tactile State Transitions

User perception of "lag" begins at 100ms. High-tactility interfaces acknowledge user input on `pointerdown`, not `click`/`pointerup`.

### Tactile Feedback States
- **Active / Pressed**: Subtle scale reduction (`scale-98` or `scale-95`), slight downward translation (`translate-y-0.5`), and background tinting.
- **Hover**: Subtle elevation lift (`-translate-y-0.5`, `shadow-md`), slight brightness increase.
- **Focus-Visible**: High-contrast, non-blurry 2px outline offset by 2px (`ring-2 ring-offset-2 ring-indigo-500`).

---

## 4. Optimistic UI Updates & Skeleton Loaders

### Optimistic Mutation
Never block user interaction while waiting on network roundtrips:
1. Immediately render the completed state locally upon user submit.
2. Fire the asynchronous network mutation in the background.
3. If failure occurs, roll back gracefully and display an informative toast with a retry button.

### Skeleton Loaders & Shimmer
- Match the exact dimensional layout of incoming data to prevent **Cumulative Layout Shift (CLS)**.
- Use subtle gradient shimmers moving left-to-right to signal active processing without causing visual agitation.

```css
@keyframes shimmer {
  100% {
    transform: translateX(100%);
  }
}
.skeleton-shimmer {
  position: relative;
  overflow: hidden;
  background-color: #f1f5f9;
}
.skeleton-shimmer::after {
  position: absolute;
  top: 0; right: 0; bottom: 0; left: 0;
  transform: translateX(-100%);
  background-image: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0,
    rgba(255, 255, 255, 0.4) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  animation: shimmer 1.5s infinite;
  content: '';
}
```

---

## 5. Vibe Coder Directives for AI Prompts

- *"Give all buttons tactile press feedback: apply `active:scale-[0.98] transition-transform duration-75` and instant pointerdown response."*
- *"Replace the loading spinner with an animated skeleton shimmer matching the exact card dimensions to eliminate layout shifts."*
- *"Implement an optimistic UI pattern on this toggle: flip the local switch state instantly, fire the update async, and toast on error."*
