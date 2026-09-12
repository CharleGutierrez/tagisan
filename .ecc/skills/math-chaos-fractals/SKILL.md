---
name: math-chaos-fractals
description: Fractals, deterministic chaos, strange attractors, Lyapunov exponents, power law scaling, and Mandelbrot procedural generation based on 'Fractals, Chaos, Power Laws' by Manfred Schroeder. Triggers: chaos-fractals, fractals-chaos-power-laws, manfred-schroeder, lorenz-attractor, mandelbrot-set, lyapunov-exponent, power-law, scale-invariance, bifurcation, procedural-generation-noise.
triggers:
  - chaos-fractals
  - fractals-chaos-power-laws
  - manfred-schroeder
  - lorenz-attractor
  - mandelbrot-set
  - lyapunov-exponent
  - power-law
  - scale-invariance
  - bifurcation
  - procedural-generation-noise
---

# Chaos, Fractals & Power Laws: Strange Attractors & Scale Invariance

## 1. Mathematical Foundations & Formal Principles

### 1.1 Scale Invariance & Fractal Dimension
A set is self-similar if it can be decomposed into $N$ copies scaled by factor $s$.
The **Hausdorff / Similarity Dimension**:
$$D = \frac{\log N}{\log(1/s)}$$
- Cantor Dust: $N = 2, s = 1/3 \implies D = \frac{\log 2}{\log 3} \approx 0.6309$
- Koch Snowflake: $N = 4, s = 1/3 \implies D = \frac{\log 4}{\log 3} \approx 1.2619$
- Sierpinski Gasket: $N = 3, s = 1/2 \implies D = \frac{\log 3}{\log 2} \approx 1.5850$

### 1.2 Deterministic Chaos & The Lorenz Attractor
A system is chaotic if it exhibits sensitive dependence on initial conditions (Lyapunov exponent $\lambda > 0$).
Edward Lorenz's 3D convective fluid atmospheric model:
$$\begin{aligned}
\dot{x} &= \sigma(y - x) \\
\dot{y} &= x(\rho - z) - y \\
\dot{z} &= x y - \beta z
\end{aligned}$$
Standard chaotic parameters: $\sigma = 10, \rho = 28, \beta = 8/3$.
The trajectory never intersects itself, remaining confined to a strange attractor with fractal dimension $D \approx 2.06$.

### 1.3 Power Law Distributions
A random variable $X$ obeys a power law if:
$$P(X > x) \sim x^{-\alpha}, \quad \alpha > 0$$
Scale invariance property: $P(c X > x) = c^{-\alpha} P(X > x)$.
Governs web link topologies, token frequency (Zipf's Law $\alpha \approx 1$), and software bug distribution.

---

## 2. The Vibe Coding Superpower

1. **Hypnotic Generative Art & Particle Systems**: Drive particle visualizers using the Lorenz strange attractor for organic, non-repeating fluid trajectories.
2. **Procedural World Generation**: Generate mountains and coastlines using fractional Brownian motion ($1/f^\alpha$ noise).
3. **Realistic Workload Modeling**: Simulate realistic multi-user API and token consumption following Pareto power laws ($80/20$ rule) rather than unrealistic Gaussian assumptions.

---

## 3. Production Code Implementations

### 3.1 Lorenz Attractor RK4 Integrator in Python
```python
import numpy as np

def lorenz_derivatives(state: np.ndarray, sigma=10.0, rho=28.0, beta=8.0/3.0) -> np.ndarray:
    x, y, z = state
    dx = sigma * (y - x)
    dy = x * (rho - z) - y
    dz = x * y - beta * z
    return np.array([dx, dy, dz])

def rk4_lorenz_step(state: np.ndarray, dt: float = 0.01) -> np.ndarray:
    """
    Runge-Kutta 4th Order step for smooth, stable chaotic integration.
    """
    k1 = lorenz_derivatives(state)
    k2 = lorenz_derivatives(state + 0.5 * dt * k1)
    k3 = lorenz_derivatives(state + 0.5 * dt * k2)
    k4 = lorenz_derivatives(state + dt * k3)
    return state + (dt / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4)
```

### 3.2 Mandelbrot Set GLSL Shader
```glsl
// Fragment shader for real-time Mandelbrot zoom
precision highp float;
uniform vec2 u_resolution;
uniform vec2 u_offset;
uniform float u_zoom;

void main() {
    vec2 c = (gl_FragCoord.xy - u_resolution * 0.5) / u_zoom + u_offset;
    vec2 z = vec2(0.0);
    float n = 0.0;
    const float max_iter = 128.0;

    for (float i = 0.0; i < max_iter; i++) {
        if (dot(z, z) > 4.0) { n = i; break; }
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
    }

    if (n == 0.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        float color = n / max_iter;
        gl_FragColor = vec4(sin(color * 3.0), sin(color * 5.0), sin(color * 7.0), 1.0);
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Procedural Generation
```markdown
When building procedural generators, particle animations, or network simulators:
- Avoid uniform or Gaussian distributions for heavy-tailed real-world quantities (token usage, file sizes); use Pareto power-law distributions.
- Integrate chaotic dynamical systems (Lorenz, Rössler) using RK4 to prevent numerical instability.
- Fractal self-similarity: compose noise across multiple octaves with persistence factor 0.5.
```
