---
name: math-nature-of-code
description: Physics simulations, vector kinematics, harmonic oscillations, Hooke's spring-damper dynamics, particle systems, and Craig Reynolds autonomous steering (seek, flee, arrive, wander, flocking boids: separation, alignment, cohesion) based on 'The Nature of Code' by Daniel Shiffman. Triggers: nature-of-code, boids, flocking, autonomous-steering, reynolds-steering, hooke-spring, particle-system, physics-simulation, harmonic-oscillator, vector-kinematics.
triggers:
  - nature-of-code
  - boids
  - boids flocking simulation
  - flocking
  - autonomous-steering
  - autonomous steering
  - reynolds-steering
  - reynolds steering
  - hooke-spring
  - particle-system
  - particle system
  - physics-simulation
  - physics simulation
  - harmonic-oscillator
  - vector-kinematics
---

# The Nature of Code: Physics, Vectors & Autonomous Swarms

## 1. Mathematical Foundations & Formal Principles

### 1.1 Vector Kinematics & Newton's Second Law
A particle state is defined by position $\mathbf{x} \in \mathbb{R}^d$, velocity $\mathbf{v} \in \mathbb{R}^d$, and acceleration $\mathbf{a} \in \mathbb{R}^d$.
Continuous dynamics are governed by Newton's Second Law:
$$\sum \mathbf{F} = m \mathbf{a} \implies \mathbf{a} = \frac{1}{m} \sum \mathbf{F}$$

Discrete integration via semi-implicit Euler integration (symplectic 1st order):
$$\mathbf{v}_{t+\Delta t} = \mathbf{v}_t + \mathbf{a}_t \Delta t$$
$$\mathbf{x}_{t+\Delta t} = \mathbf{x}_t + \mathbf{v}_{t+\Delta t} \Delta t$$
$$\mathbf{a}_{t+\Delta t} = \mathbf{0} \quad (\text{force accumulation reset})$$

### 1.2 Environmental Dissipative Forces
1. **Dynamic Kinetic Friction**:
   $$\mathbf{F}_{\text{friction}} = -\mu N \hat{\mathbf{v}} = -\mu N \frac{\mathbf{v}}{\|\mathbf{v}\| + \epsilon}$$
2. **Fluid Drag (Aerodynamic Resistance)**:
   $$\mathbf{F}_{\text{drag}} = -\frac{1}{2} \rho \|\mathbf{v}\|^2 C_d A \hat{\mathbf{v}} = -k_{\text{drag}} \|\mathbf{v}\| \mathbf{v}$$
3. **Harmonic Spring-Mass Oscillator (Hooke's Law with Damping)**:
   $$\mathbf{F}_{\text{spring}} = -k (\mathbf{x} - \mathbf{x}_{\text{rest}}) - c \mathbf{v}$$
   where $k$ is spring stiffness, $c$ is damping coefficient ($c = 2\sqrt{km}$ for critical damping).

### 1.3 Autonomous Steering (Craig Reynolds Model)
Steering force acts to alter velocity towards desired target while obeying acceleration limits:
$$\mathbf{F}_{\text{steer}} = \text{clamp}\left(\mathbf{v}_{\text{desired}} - \mathbf{v}_{\text{current}}, F_{\text{max}}\right)$$

Where $\mathbf{v}_{\text{desired}}$ varies by behavior:
- **Seek**: $\mathbf{v}_{\text{desired}} = v_{\text{max}} \frac{\mathbf{x}_{\text{target}} - \mathbf{x}}{\|\mathbf{x}_{\text{target}} - \mathbf{x}\|}$
- **Arrive**: $\mathbf{v}_{\text{desired}} = v_{\text{max}} \cdot \min\left(1, \frac{r}{r_{\text{slow}}}\right) \frac{\mathbf{x}_{\text{target}} - \mathbf{x}}{r}$ where $r = \|\mathbf{x}_{\text{target}} - \mathbf{x}\|$

### 1.4 Boids Flocking Emergence
For agent $i$ with neighborhood $N_i = \{j \mid \|\mathbf{x}_j - \mathbf{x}_i\| < R\}$:
1. **Separation**: $\mathbf{F}_{\text{sep}} = \sum_{j \in N_i, j \neq i} \frac{\mathbf{x}_i - \mathbf{x}_j}{\|\mathbf{x}_i - \mathbf{x}_j\|^2 + \epsilon}$
2. **Alignment**: $\mathbf{F}_{\text{ali}} = \text{steer}\left(\frac{1}{|N_i|} \sum_{j \in N_i} \mathbf{v}_j\right)$
3. **Cohesion**: $\mathbf{F}_{\text{coh}} = \text{steer}\left(\frac{1}{|N_i|} \sum_{j \in N_i} \mathbf{x}_j - \mathbf{x}_i\right)$
$$\mathbf{F}_{\text{total}} = w_s \mathbf{F}_{\text{sep}} + w_a \mathbf{F}_{\text{ali}} + w_c \mathbf{F}_{\text{coh}}$$

---

## 2. The Vibe Coding Superpower

Vibe coders use these principles to prompt AI to generate organic, living software rather than robotic static interfaces:
1. **Dynamic Interactive UIs**: Apply spring dampers to drag gestures, popovers, and canvas cards rather than rigid CSS cubic-bezier transitions.
2. **Particle Microinteractions**: Implement GPU or Canvas particle feedback on clicks, loading states, and error alerts.
3. **Agent Swarm Topologies**: Position multi-agent swarms using boid separation and cohesion forces on 2D visual canvases or graph layouts.
4. **Zero-Engine Physics**: Implement lightweight, crash-proof simulations in pure TypeScript or Rust without multi-megabyte engine dependencies.

---

## 3. Production Code Implementations

### 3.1 Symplectic Vector Kinematics & Spring Damper in TypeScript
```typescript
export interface Vec2 {
  x: number;
  y: number;
}

export class Particle {
  pos: Vec2;
  vel: Vec2;
  acc: Vec2;
  mass: number;

  constructor(x: number, y: number, mass: number = 1.0) {
    this.pos = { x, y };
    this.vel = { x: 0, y: 0 };
    this.acc = { x: 0, y: 0 };
    this.mass = Math.max(0.0001, mass);
  }

  applyForce(f: Vec2): void {
    if (!Number.isFinite(f.x) || !Number.isFinite(f.y)) return;
    this.acc.x += f.x / this.mass;
    this.acc.y += f.y / this.mass;
  }

  update(dt: number, maxSpeed: number = 20.0): void {
    const clampedDt = Math.min(Math.max(dt, 0.0001), 0.1);
    // Symplectic Euler integration
    this.vel.x += this.acc.x * clampedDt;
    this.vel.y += this.acc.y * clampedDt;

    const speedSq = this.vel.x * this.vel.x + this.vel.y * this.vel.y;
    if (speedSq > maxSpeed * maxSpeed) {
      const speed = Math.sqrt(speedSq);
      this.vel.x = (this.vel.x / speed) * maxSpeed;
      this.vel.y = (this.vel.y / speed) * maxSpeed;
    }

    this.pos.x += this.vel.x * clampedDt;
    this.pos.y += this.vel.y * clampedDt;

    // Reset force accumulator
    this.acc.x = 0;
    this.acc.y = 0;
  }
}

export function springForce(
  pos: Vec2,
  anchor: Vec2,
  vel: Vec2,
  k: number,
  damping: number,
  restLength: number = 0
): Vec2 {
  const dx = pos.x - anchor.x;
  const dy = pos.y - anchor.y;
  const dist = Math.sqrt(dx * dx + dy * dy);
  const eps = 1e-6;
  const displacement = dist - restLength;
  const nx = dist > eps ? dx / dist : 0;
  const ny = dist > eps ? dy / dist : 0;

  const springMag = -k * displacement;
  const dampX = -damping * vel.x;
  const dampY = -damping * vel.y;

  return {
    x: springMag * nx + dampX,
    y: springMag * ny + dampY,
  };
}
```

### 3.2 Reynolds Boid Flocking in Rust
```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn mag_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn mag(&self) -> f32 {
        self.mag_sq().sqrt()
    }
    pub fn normalize_or_zero(&self) -> Self {
        let m = self.mag();
        if m > 1e-6 {
            Self { x: self.x / m, y: self.y / m }
        } else {
            Self::default()
        }
    }
    pub fn limit(&self, max: f32) -> Self {
        let m_sq = self.mag_sq();
        if m_sq > max * max && m_sq > 1e-8 {
            let m = m_sq.sqrt();
            Self { x: (self.x / m) * max, y: (self.y / m) * max }
        } else {
            *self
        }
    }
}

pub struct Boid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
    pub max_speed: f32,
    pub max_force: f32,
}

impl Boid {
    pub fn flock(&mut self, boids: &[Boid], r_sep: f32, r_neigh: f32) {
        let mut sep = Vec2::default();
        let mut ali = Vec2::default();
        let mut coh = Vec2::default();
        let mut count_sep = 0;
        let mut count_neigh = 0;

        for other in boids {
            let dx = self.pos.x - other.pos.x;
            let dy = self.pos.y - other.pos.y;
            let d_sq = dx * dx + dy * dy;

            if d_sq > 1e-6 && d_sq < r_sep * r_sep {
                let d = d_sq.sqrt();
                sep.x += dx / (d * d);
                sep.y += dy / (d * d);
                count_sep += 1;
            }

            if d_sq > 1e-6 && d_sq < r_neigh * r_neigh {
                ali.x += other.vel.x;
                ali.y += other.vel.y;
                coh.x += other.pos.x;
                coh.y += other.pos.y;
                count_neigh += 1;
            }
        }

        if count_sep > 0 {
            sep = sep.normalize_or_zero().limit(self.max_force);
        }
        if count_neigh > 0 {
            ali.x /= count_neigh as f32;
            ali.y /= count_neigh as f32;
            ali = ali.normalize_or_zero().limit(self.max_force);

            coh.x = (coh.x / count_neigh as f32) - self.pos.x;
            coh.y = (coh.y / count_neigh as f32) - self.pos.y;
            coh = coh.normalize_or_zero().limit(self.max_force);
        }

        self.acc.x += sep.x * 1.5 + ali.x * 1.0 + coh.x * 1.0;
        self.acc.y += sep.y * 1.5 + ali.y * 1.0 + coh.y * 1.0;
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Directive for Organic Motion
```markdown
When implementing animations or interactive elements:
- Never use linear transitions for physical entities.
- Model acceleration using Newton's Second Law: a = F / m.
- Accumulate forces per frame and clear them at the end of each tick.
- Apply Hooke's Law for spring oscillations with critical damping c = 2 * sqrt(k * m).
- Protect all vector normalizations with epsilon eps = 1e-6 to prevent division by zero.
```

### 4.2 Few-Shot Prompt for Flocking Agent Layout
```markdown
Prompt:
"Create a React/Canvas component displaying 50 autonomous AI agent nodes that smoothly flock together using Craig Reynolds' separation (1.5x), alignment (1.0x), and cohesion (1.0x). Include mouse repulsion when the user hovers over the canvas."
```
