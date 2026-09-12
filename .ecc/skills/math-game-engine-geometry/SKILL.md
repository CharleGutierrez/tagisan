---
name: math-game-engine-geometry
description: Game engine mathematics: 3D affine transformations, quaternions without gimbal lock, SLERP, homogeneous coordinates, ray-plane and ray-AABB slab intersections based on 'Foundations of Game Engine Development, Vol 1: Mathematics' by Eric Lengyel. Triggers: game-engine-geometry, lengyel, quaternions, gimbal-lock, slerp, affine-transformations, ray-aabb-intersection, homogeneous-coordinates, frustum-culling, 3d-rotation.
triggers:
  - game-engine-geometry
  - lengyel
  - quaternions
  - gimbal-lock
  - slerp
  - affine-transformations
  - ray-aabb-intersection
  - homogeneous-coordinates
  - frustum-culling
  - 3d-rotation
---

# Game Engine Geometry: Quaternions, SLERP & Bounding Boxes

## 1. Mathematical Foundations & Formal Principles

### 1.1 Quaternions & 3D Rotations
A quaternion $\mathbf{q} \in \mathbb{H}$ is defined by real scalar $w$ and imaginary vector $\mathbf{v} = (x, y, z)$:
$$\mathbf{q} = w + x\mathbf{i} + y\mathbf{j} + z\mathbf{k} = (w, \mathbf{v})$$
Where Hamilton's rules apply: $\mathbf{i}^2 = \mathbf{j}^2 = \mathbf{k}^2 = \mathbf{i}\mathbf{j}\mathbf{k} = -1$.
Hamilton product:
$$\mathbf{q}_1 \mathbf{q}_2 = (w_1 w_2 - \mathbf{v}_1 \cdot \mathbf{v}_2, \; w_1 \mathbf{v}_2 + w_2 \mathbf{v}_1 + \mathbf{v}_1 \times \mathbf{v}_2)$$

A 3D rotation of vector $\mathbf{p} = (0, \mathbf{x})$ by angle $\theta$ around unit axis $\hat{\mathbf{u}}$ is represented by unit quaternion:
$$\mathbf{q} = \left(\cos\frac{\theta}{2}, \; \hat{\mathbf{u}}\sin\frac{\theta}{2}\right)$$
Rotated vector: $\mathbf{p}' = \mathbf{q} \mathbf{p} \mathbf{q}^*$, where $\mathbf{q}^* = (w, -\mathbf{v})$ is the quaternion conjugate.
**Gimbal Lock Elimination**: Quaternions represent rotations on the 3-sphere $\mathbb{S}^3$, having no coordinate singularities unlike Euler angles ($\phi, \theta, \psi$).

### 1.2 Spherical Linear Interpolation (SLERP)
Given two unit quaternions $\mathbf{q}_1, \mathbf{q}_2$ with $\cos\Omega = \mathbf{q}_1 \cdot \mathbf{q}_2$:
$$\text{SLERP}(\mathbf{q}_1, \mathbf{q}_2; t) = \frac{\sin((1-t)\Omega)}{\sin\Omega}\mathbf{q}_1 + \frac{\sin(t\Omega)}{\sin\Omega}\mathbf{q}_2$$
If $\cos\Omega < 0$, negate $\mathbf{q}_2$ to traverse the shortest path on $\mathbb{S}^3$.
For $\Omega \to 0$, fall back to normalized LERP to avoid division by zero.

### 1.3 Ray-AABB Slab Intersection (Kay-Kajiya Method)
An Axis-Aligned Bounding Box (AABB) is defined by $[\mathbf{p}_{\text{min}}, \mathbf{p}_{\text{max}}]$.
A ray is $\mathbf{r}(t) = \mathbf{o} + t\mathbf{d}, \; t \ge 0$.
For each axis $i \in \{x, y, z\}$:
$$t_{1, i} = \frac{p_{\text{min}, i} - o_i}{d_i}, \quad t_{2, i} = \frac{p_{\text{max}, i} - o_i}{d_i}$$
$$t_{\text{near}} = \max_i(\min(t_{1, i}, t_{2, i})), \quad t_{\text{far}} = \min_i(\max(t_{1, i}, t_{2, i}))$$
Intersection condition: $t_{\text{near}} \le t_{\text{far}}$ and $t_{\text{far}} \ge 0$.

---

## 2. The Vibe Coding Superpower

1. **Gimbal-Lock-Free Three.js / WebGPU**: Smooth camera controllers and object rotations in WebGL/WebGPU shaders using quaternions.
2. **Raycasting in 3D UI**: Test clicks and laser pointers against spatial menus in VR/AR with ultra-fast slab tests.
3. **Camera Interpolation**: Use SLERP to smoothly glide cameras between points of interest without erratic flips.

---

## 3. Production Code Implementations

### 3.1 Quaternion & SLERP in TypeScript / Three.js style
```typescript
export class Quaternion {
  w: number; x: number; y: number; z: number;

  constructor(w = 1, x = 0, y = 0, z = 0) {
    this.w = w; this.x = x; this.y = y; this.z = z;
  }

  static fromAxisAngle(axis: { x: number; y: number; z: number }, rad: number): Quaternion {
    const half = rad * 0.5;
    const s = Math.sin(half);
    const len = Math.sqrt(axis.x * axis.x + axis.y * axis.y + axis.z * axis.z) || 1;
    return new Quaternion(Math.cos(half), (axis.x / len) * s, (axis.y / len) * s, (axis.z / len) * s);
  }

  dot(other: Quaternion): number {
    return this.w * other.w + this.x * other.x + this.y * other.y + this.z * other.z;
  }

  slerp(target: Quaternion, t: number): Quaternion {
    let cosOmega = this.dot(target);
    let targetCopy = { ...target };

    // Shortest path traversal
    if (cosOmega < 0) {
      cosOmega = -cosOmega;
      targetCopy.w = -targetCopy.w;
      targetCopy.x = -targetCopy.x;
      targetCopy.y = -targetCopy.y;
      targetCopy.z = -targetCopy.z;
    }

    if (cosOmega > 0.9995) {
      // Linear fallback to avoid division by zero
      const w = this.w + t * (targetCopy.w - this.w);
      const x = this.x + t * (targetCopy.x - this.x);
      const y = this.y + t * (targetCopy.y - this.y);
      const z = this.z + t * (targetCopy.z - this.z);
      const norm = Math.sqrt(w*w + x*x + y*y + z*z) || 1;
      return new Quaternion(w/norm, x/norm, y/norm, z/norm);
    }

    const omega = Math.acos(cosOmega);
    const sinOmega = Math.sin(omega);
    const s1 = Math.sin((1 - t) * omega) / sinOmega;
    const s2 = Math.sin(t * omega) / sinOmega;

    return new Quaternion(
      s1 * this.w + s2 * targetCopy.w,
      s1 * this.x + s2 * targetCopy.x,
      s1 * this.y + s2 * targetCopy.y,
      s1 * this.z + s2 * targetCopy.z
    );
  }
}
```

### 3.2 Ray-AABB Slab Test in GLSL / Rust
```rust
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: [f32; 3],
    pub dir: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

pub fn ray_intersects_aabb(ray: &Ray, box_: &AABB) -> Option<f32> {
    let mut t_near = f32::NEG_INFINITY;
    let mut t_far = f32::INFINITY;

    for i in 0..3 {
        let d = ray.dir[i];
        let o = ray.origin[i];

        if d.abs() < 1e-8 {
            if o < box_.min[i] || o > box_.max[i] {
                return None; // Ray parallel and outside slab
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (box_.min[i] - o) * inv_d;
            let mut t2 = (box_.max[i] - o) * inv_d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            t_near = t_near.max(t1);
            t_far = t_far.min(t2);

            if t_near > t_far || t_far < 0.0 {
                return None;
            }
        }
    }

    Some(if t_near >= 0.0 { t_near } else { t_far })
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for 3D Math
```markdown
In all 3D rotational calculations:
- Never represent orientations via Euler angles (pitch, yaw, roll) due to gimbal lock.
- Always use unit quaternions.
- For smooth camera and joint rotations, use SLERP with dot-product sign flipping to take the shortest geodesic on S^3.
- When computing Ray-AABB collisions, guard against 0 division in reciprocal ray directions.
```
