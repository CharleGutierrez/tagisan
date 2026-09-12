---
name: math-numerical-methods
description: Numerical recipes: IEEE-754 precision, Runge-Kutta 4th order (RK4), Symplectic Verlet integration, Brent's root-finding, cubic splines, and numerical NaN/Inf protection based on 'Numerical Recipes' by Press, Teukolsky, Vetterling, and Flannery. Triggers: numerical-methods, numerical-recipes, press-teukolsky, rk4-runge-kutta, symplectic-verlet, brent-root-finding, cubic-spline, floating-point-stability, catastrophic-cancellation, condition-number.
triggers:
  - numerical-methods
  - numerical-recipes
  - press-teukolsky
  - rk4-runge-kutta
  - symplectic-verlet
  - brent-root-finding
  - cubic-spline
  - floating-point-stability
  - catastrophic-cancellation
  - condition-number
---

# Numerical Methods: ODE Solvers, Stability & Precision

## 1. Mathematical Foundations & Formal Principles

### 1.1 Floating-Point Realities (IEEE 754)
- **Machine Epsilon**: Smallest $\epsilon$ such that $1.0 + \epsilon \neq 1.0$. ($\approx 1.19 \times 10^{-7}$ for `f32`, $\approx 2.22 \times 10^{-16}$ for `f64`).
- **Catastrophic Cancellation**: Subtracting two nearly identical numbers $a \approx b$ causes loss of significant digits: $\text{relerr}(a - b) \approx \frac{\epsilon}{|a - b|}$.
- **Condition Number**: $\kappa = \frac{\|J(x)\| \|x\|}{\|f(x)\|}$. High condition number ($\kappa \gg 1$) indicates an ill-posed problem.

### 1.2 Runge-Kutta 4th Order (RK4)
For initial value problem $\dot{\mathbf{y}} = \mathbf{f}(t, \mathbf{y})$:
$$\begin{aligned}
\mathbf{k}_1 &= \mathbf{f}(t_n, \mathbf{y}_n) \\
\mathbf{k}_2 &= \mathbf{f}\left(t_n + \frac{h}{2}, \mathbf{y}_n + \frac{h}{2}\mathbf{k}_1\right) \\
\mathbf{k}_3 &= \mathbf{f}\left(t_n + \frac{h}{2}, \mathbf{y}_n + \frac{h}{2}\mathbf{k}_2\right) \\
\mathbf{k}_4 &= \mathbf{f}(t_n + h, \mathbf{y}_n + h\mathbf{k}_3) \\
\mathbf{y}_{n+1} &= \mathbf{y}_n + \frac{h}{6}(\mathbf{k}_1 + 2\mathbf{k}_2 + 2\mathbf{k}_3 + \mathbf{k}_4) + O(h^5)
\end{aligned}$$

### 1.3 Symplectic Störmer-Verlet Integration
For Hamiltonian mechanical systems where $\ddot{\mathbf{x}} = \mathbf{a}(\mathbf{x})$:
$$\mathbf{x}_{n+1} = 2\mathbf{x}_n - \mathbf{x}_{n-1} + \mathbf{a}(\mathbf{x}_n) \Delta t^2$$
Or velocity Verlet:
$$\mathbf{x}_{n+1} = \mathbf{x}_n + \mathbf{v}_n \Delta t + \frac{1}{2}\mathbf{a}_n \Delta t^2$$
$$\mathbf{v}_{n+1} = \mathbf{v}_n + \frac{1}{2}(\mathbf{a}_n + \mathbf{a}_{n+1})\Delta t$$
**Symplectic Invariant**: Preserves phase-space volume and guarantees bounded energy oscillation without artificial damping or explosion.

---

## 2. The Vibe Coding Superpower

1. **Bulletproof Physics Without NaN Crashes**: Explicit Euler blows up and causes objects to fly into outer space. Verlet and RK4 keep simulated engines rock-solid.
2. **Smooth Spline Camera Paths**: Interpolate camera trajectories smoothly through keypoints with $C^2$ continuity using cubic splines.
3. **Safe Mathematical Guards**: Prevent infinite loops and `NaN` propagation by structuring boundary checks into numerical algorithms.

---

## 3. Production Code Implementations

### 3.1 Symplectic Velocity Verlet Integrator in Rust
```rust
#[derive(Clone, Copy, Debug)]
pub struct PhysicalState {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
}

pub fn verlet_step<F>(state: &mut PhysicalState, dt: f32, compute_acc: F)
where
    F: Fn(&[f32; 3]) -> [f32; 3],
{
    // Clamp delta-time to prevent stiff explosion
    let clamped_dt = dt.clamp(0.0001, 0.05);

    let a_curr = compute_acc(&state.pos);

    // Position update: x(t + dt) = x(t) + v(t)*dt + 0.5*a(t)*dt^2
    let mut next_pos = [0.0f32; 3];
    for i in 0..3 {
        next_pos[i] = state.pos[i] + state.vel[i] * clamped_dt + 0.5 * a_curr[i] * clamped_dt * clamped_dt;
        if !next_pos[i].is_finite() {
            next_pos[i] = state.pos[i]; // Guard against NaN
        }
    }

    let a_next = compute_acc(&next_pos);

    // Velocity update: v(t + dt) = v(t) + 0.5*(a(t) + a(t + dt))*dt
    for i in 0..3 {
        state.vel[i] += 0.5 * (a_curr[i] + a_next[i]) * clamped_dt;
        if !state.vel[i].is_finite() {
            state.vel[i] = 0.0;
        }
    }
    state.pos = next_pos;
}
```

### 3.2 Brent's Root-Finding Method in Python
```python
from typing import Callable

def brent_find_root(f: Callable[[float], float], a: float, b: float, tol: float = 1e-8, max_iter: int = 100) -> float:
    """
    Brent's root-finding method combining bisection, secant, and inverse quadratic interpolation.
    Guaranteed convergence if f(a) * f(b) < 0.
    """
    fa, fb = f(a), f(b)
    assert fa * fb <= 0.0, "Root must be bracketed: f(a) and f(b) must have opposite signs"

    if abs(fa) < abs(fb):
        a, b = b, a
        fa, fb = fb, fa

    c, fc = a, fa
    mflag = True
    d = 0.0

    for _ in range(max_iter):
        if abs(fb) < tol:
            return b

        if fa != fc and fb != fc:
            # Inverse quadratic interpolation
            s = (a * fb * fc) / ((fa - fb) * (fa - fc)) + \
                (b * fa * fc) / ((fb - fa) * (fb - fc)) + \
                (c * fa * fb) / ((fc - fa) * (fc - fb))
        else:
            # Secant method
            s = b - fb * (b - a) / (fb - fa)

        # Conditions to force bisection fallback
        cond1 = not ((3 * a + b) / 4 <= s <= b or b <= s <= (3 * a + b) / 4)
        cond2 = mflag and abs(s - b) >= abs(b - c) / 2.0
        cond3 = not mflag and abs(s - b) >= abs(c - d) / 2.0
        cond4 = mflag and abs(b - c) < tol
        cond5 = not mflag and abs(c - d) < tol

        if cond1 or cond2 or cond3 or cond4 or cond5:
            s = (a + b) / 2.0
            mflag = True
        else:
            mflag = False

        fs = f(s)
        d, c, fc = c, b, fb

        if fa * fs < 0:
            b, fb = s, fs
        else:
            a, fa = s, fs

        if abs(fa) < abs(fb):
            a, b = b, a
            fa, fb = fb, fa

    return b
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Numerical Robustness
```markdown
In all scientific computing, ODE simulations, and physics loops:
- Never use forward Euler for physical simulations; use Symplectic Velocity Verlet or RK4.
- Guard against NaN and Inf: check `is_finite()` at every step.
- Clamp delta-time dt <= 0.05 to prevent explosions during frame drops.
- Always bracket roots before running root-finding iterations.
```
