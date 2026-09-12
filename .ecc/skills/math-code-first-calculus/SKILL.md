---
name: math-code-first-calculus
description: Code-first calculus and vectors: finite difference derivatives, numerical gradient descent, polygon rasterization, ray casting in-polygon tests, and simulation loops based on 'Math for Programmers' by Paul Orland. Triggers: code-first-calculus, math-for-programmers, paul-orland, finite-differences, numerical-gradient-descent, polygon-rasterization, simulation-loop, vector-geometry-code, ray-casting-polygon.
triggers:
  - code-first-calculus
  - math-for-programmers
  - paul-orland
  - finite-differences
  - numerical-gradient-descent
  - polygon-rasterization
  - simulation-loop
  - vector-geometry-code
  - ray-casting-polygon
---

# Code-First Calculus & Vector Geometry

## 1. Mathematical Foundations & Formal Principles

### 1.1 Finite Difference Approximations
The derivative of $f(x)$ is the limit of the slope of secant lines:
$$f'(x) = \lim_{h \to 0} \frac{f(x+h) - f(x)}{h}$$
In numerical code, $h$ cannot reach 0 due to floating-point catastrophic cancellation.
- **Forward Difference**: $f'(x) \approx \frac{f(x+h) - f(x)}{h} + O(h)$
- **Central Difference (Preferred)**:
  $$f'(x) \approx \frac{f(x+h) - f(x-h)}{2h} + O(h^2)$$
Optimal step size for IEEE-754 double precision: $h \approx \sqrt{\epsilon_{\text{mach}}} \approx 10^{-8}$.

### 1.2 Numerical Gradient Vector
For multivariable scalar field $f(\mathbf{x}): \mathbb{R}^n \to \mathbb{R}$:
$$\nabla f(\mathbf{x}) = \left(\frac{\partial f}{\partial x_1}, \frac{\partial f}{\partial x_2}, \dots, \frac{\partial f}{\partial x_n}\right)^T$$
$$\frac{\partial f}{\partial x_i} \approx \frac{f(\mathbf{x} + h\mathbf{e}_i) - f(\mathbf{x} - h\mathbf{e}_i)}{2h}$$
**Gradient Descent**: Step against the steepest ascent:
$$\mathbf{x}_{k+1} = \mathbf{x}_k - \alpha \nabla f(\mathbf{x}_k)$$

### 1.3 Polygon Geometry & Ray Casting (Jordan Curve Theorem)
A 2D point $\mathbf{p} = (x, y)$ is inside polygon $V = (\mathbf{v}_1, \dots, \mathbf{v}_n)$ if a ray cast from $\mathbf{p}$ along $+x$ intersects the polygon edges an odd number of times.
Edge $(\mathbf{v}_i, \mathbf{v}_{i+1})$ intersects horizontal ray from $\mathbf{p}$ if:
$$(y_i > y \neq y_{i+1} > y) \land \left(x < \frac{(x_{i+1} - x_i)(y - y_i)}{y_{i+1} - y_i} + x_i\right)$$

---

## 2. The Vibe Coding Superpower

1. **Instant Formula-to-Code**: Do not get bogged down in symbolic algebra. Evaluate derivatives and gradients directly via finite differences to solve optimization problems.
2. **Interactive UI Hit-Testing**: Test whether custom irregular polygon boundaries, lasso tools, or SVG canvas nodes contain cursor coordinates.
3. **Adaptive Animation Curves**: Compute velocity and acceleration directly from discrete time samples without analytical curves.

---

## 3. Production Code Implementations

### 3.1 Numerical Gradient Descent in Python
```python
from typing import Callable
import numpy as np

def numerical_gradient(f: Callable[[np.ndarray], float], x: np.ndarray, h: float = 1e-6) -> np.ndarray:
    """
    Central difference gradient approximation for arbitrary multivariable functions.
    """
    grad = np.zeros_like(x, dtype=float)
    x_mut = x.astype(float).copy()

    for i in range(len(x)):
        orig = x_mut[i]
        x_mut[i] = orig + h
        f_pos = f(x_mut)
        x_mut[i] = orig - h
        f_neg = f(x_mut)
        grad[i] = (f_pos - f_neg) / (2.0 * h)
        x_mut[i] = orig

    return grad

def gradient_descent_solver(
    f: Callable[[np.ndarray], float],
    x_init: np.ndarray,
    lr: float = 0.01,
    tol: float = 1e-6,
    max_iter: int = 1000
) -> np.ndarray:
    x = x_init.astype(float).copy()
    for _ in range(max_iter):
        grad = numerical_gradient(f, x)
        if np.linalg.norm(grad) < tol:
            break
        x -= lr * grad
    return x
```

### 3.2 Point-in-Polygon Ray Casting in TypeScript
```typescript
export interface Point2D { x: number; y: number; }

export function isPointInPolygon(p: Point2D, polygon: Point2D[]): boolean {
  let inside = false;
  const n = polygon.length;
  if (n < 3) return false;

  for (let i = 0, j = n - 1; i < n; j = i++) {
    const xi = polygon[i].x, yi = polygon[i].y;
    const xj = polygon[j].x, yj = polygon[j].y;

    const intersect =
      yi > p.y !== yj > p.y &&
      p.x < ((xj - xi) * (p.y - yi)) / (yj - yi + 1e-12) + xi;

    if (intersect) inside = !inside;
  }
  return inside;
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Code-First Math
```markdown
When implementing mathematical optimization or curves:
- Prefer code-first finite difference approximations over complex symbolic solvers unless closed-form is trivial.
- Use central difference (f(x+h) - f(x-h)) / (2*h) with step h = 1e-6.
- Always include termination conditions in optimization loops: max_iterations and gradient norm tolerance.
```
