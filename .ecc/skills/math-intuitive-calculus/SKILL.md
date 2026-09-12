---
name: math-intuitive-calculus
description: Intuitive differential and integral calculus: rates of change, velocity/acceleration accumulators, PID control loops, and momentum scrolling based on 'Calculus Made Easy' by Silvanus P. Thompson and Martin Gardner. Triggers: intuitive-calculus, calculus-made-easy, thompson-gardner, infinitesimals, instantaneous-rate, numerical-integration, pid-controller, ui-velocity-accumulator, momentum-scrolling, exponential-decay.
triggers:
  - intuitive-calculus
  - calculus-made-easy
  - thompson-gardner
  - infinitesimals
  - instantaneous-rate
  - numerical-integration
  - pid-controller
  - ui-velocity-accumulator
  - momentum-scrolling
  - exponential-decay
---

# Intuitive Calculus: Velocity, Accumulation & PID Loops

## 1. Mathematical Foundations & Formal Principles

### 1.1 The Essence of Infinitesimals
*"What one fool can do, another can."*
Differential calculus is simply the study of **speed of change**: $\frac{dy}{dx}$.
Integral calculus is the study of **accumulation of small parts**: $\int y \, dx$.

Core operational rules:
- **Power Rule**: $\frac{d}{dx}[x^n] = n x^{n-1}$
- **Product Rule**: $\frac{d}{dx}[uv] = u \frac{dv}{dx} + v \frac{du}{dx}$
- **Chain Rule**: $\frac{dy}{dx} = \frac{dy}{du} \frac{du}{dx}$
- **Exponential Decay**: $\frac{dy}{dt} = -k y \implies y(t) = y_0 e^{-kt}$

### 1.2 Discrete Velocity Accumulation
In UI gesture physics, position $x(t)$ is sampled discretely at timestamps $t_i$:
Instantaneous velocity:
$$v_i = \frac{x_i - x_{i-1}}{t_i - t_{i-1} + \epsilon}$$
Exponential moving decay for smooth momentum scrolling:
$$v_{\text{smooth}} \leftarrow v_{\text{smooth}} \cdot e^{-\lambda \Delta t}$$

### 1.3 PID Controller (Proportional-Integral-Derivative)
Control error $e(t) = r(t) - y(t)$:
$$u(t) = K_p e(t) + K_i \int_0^t e(\tau) d\tau + K_d \frac{de(t)}{dt}$$
- **Proportional ($K_p$)**: Corrects current error.
- **Integral ($K_i$)**: Eliminates steady-state error accumulation (requires anti-windup clamping).
- **Derivative ($K_d$)**: Dampens oscillations by anticipating future error trends.

---

## 2. The Vibe Coding Superpower

1. **Fluid UI Drag & Fling Gestures**: Accumulate gesture velocity and apply exponential decay for native-feeling mobile momentum scrolling.
2. **Robotic Camera Smoothing**: Implement PID controllers for 3D camera tracking that never overshoot or stutter.
3. **Leaky Bucket Token Rate-Limiting**: Model token bucket consumption through continuous integration with constant leak rate.

---

## 3. Production Code Implementations

### 3.1 PID Controller with Anti-Windup in TypeScript
```typescript
export class PIDController {
  kp: number;
  ki: number;
  kd: number;
  integral: number = 0;
  prevError: number = 0;
  maxIntegral: number;

  constructor(kp: number, ki: number, kd: number, maxIntegral: number = 100.0) {
    this.kp = kp;
    this.ki = ki;
    this.kd = kd;
    this.maxIntegral = maxIntegral;
  }

  update(setpoint: number, current: number, dt: number): number {
    const clampedDt = Math.min(Math.max(dt, 0.001), 0.1);
    const error = setpoint - current;

    // Integral accumulation with anti-windup guard
    this.integral += error * clampedDt;
    this.integral = Math.max(-this.maxIntegral, Math.min(this.integral, this.maxIntegral));

    // Derivative: rate of error change
    const derivative = (error - this.prevError) / clampedDt;
    this.prevError = error;

    return this.kp * error + this.ki * this.integral + this.kd * derivative;
  }
}
```

### 3.2 UI Gesture Velocity Tracker in Rust
```rust
pub struct VelocityTracker {
    last_pos: f32,
    last_time: f64,
    velocity: f32,
    friction: f32,
}

impl VelocityTracker {
    pub fn new(friction: f32) -> Self {
        Self {
            last_pos: 0.0,
            last_time: 0.0,
            velocity: 0.0,
            friction,
        }
    }

    pub fn push_sample(&mut self, pos: f32, time_sec: f64) {
        let dt = (time_sec - self.last_time).max(0.001) as f32;
        let instant_v = (pos - self.last_pos) / dt;
        // Low-pass filter smoothing
        self.velocity = 0.7 * self.velocity + 0.3 * instant_v;
        self.last_pos = pos;
        self.last_time = time_sec;
    }

    pub fn decay(&mut self, dt: f32) -> f32 {
        self.velocity *= (-self.friction * dt).exp();
        self.velocity
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Dynamic Controls
```markdown
When generating animation curves, gestures, or continuous rate limiters:
- Use PID controllers with anti-windup clamping on the integral term to prevent runaway oscillations.
- Compute velocities as finite differences: (x_new - x_old) / dt with delta-time clamping.
- Apply exponential decay exp(-decay_rate * dt) for frame-rate-independent friction.
```
