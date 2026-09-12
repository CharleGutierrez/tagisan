---
name: math-visual-topology
description: Visual geometry, Gaussian and mean curvature, Euler characteristic, polyhedra, manifold validation, and geodesic paths based on 'Geometry and the Imagination' by David Hilbert and Stefan Cohn-Vossen. Triggers: visual-topology, geometry-and-the-imagination, hilbert-cohn-vossen, gaussian-curvature, euler-characteristic, polyhedra, manifold-mesh, geodesic, subdivision-surfaces, discrete-differential-geometry.
triggers:
  - visual-topology
  - geometry-and-the-imagination
  - hilbert-cohn-vossen
  - gaussian-curvature
  - euler-characteristic
  - polyhedra
  - manifold-mesh
  - geodesic
  - subdivision-surfaces
  - discrete-differential-geometry
---

# Visual Topology: Curvature, Manifolds & Euler Characteristic

## 1. Mathematical Foundations & Formal Principles

### 1.1 Curvature of Surfaces in $\mathbb{R}^3$
At any point on a smooth 2D manifold, principal curvatures $\kappa_1, \kappa_2$ are the minimum and maximum normal curvatures.
- **Gaussian Curvature**: $K = \kappa_1 \kappa_2$
  - $K > 0$: Elliptic point (sphere, dome)
  - $K < 0$: Hyperbolic point (saddle, Pringles chip)
  - $K = 0$: Parabolic point (cylinder, flat plane)
- **Gauss's Theorema Egregium**: Gaussian curvature is an **intrinsic invariant** of the surface, completely determined by local metric measurements without reference to embedding space.

### 1.2 Euler-Poincaré Characteristic
For any polyhedral mesh or closed surface with $V$ vertices, $E$ edges, and $F$ faces:
$$\chi = V - E + F = 2 - 2g$$
Where $g$ is the genus (number of topological handles):
- Sphere ($g=0$): $\chi = 2$
- Torus ($g=1$): $\chi = 0$
- Double Torus ($g=2$): $\chi = -2$

**Gauss-Bonnet Theorem**:
$$\iint_M K \, dA + \int_{\partial M} k_g \, ds = 2\pi \chi(M)$$

### 1.3 2-Manifold Mesh Invariants
A triangle mesh is a valid 2-manifold if:
1. Every edge is shared by at most 2 triangles (no non-manifold edges).
2. The triangles incident to every vertex form an open or closed fan (no pinch points).

---

## 2. The Vibe Coding Superpower

1. **Generative 3D Mesh Integrity**: Automatically repair broken AI-generated 3D meshes by checking the Euler characteristic $\chi = V - E + F$.
2. **Procedural Terrain & Shaders**: Shade surfaces dynamically based on discrete Gaussian curvature (crevices vs peaks).
3. **Latent Manifold Navigation**: Visualize high-dimensional embeddings as lower-dimensional topological manifolds without tearing.

---

## 3. Production Code Implementations

### 3.1 Euler Characteristic & Manifold Validator in Python
```python
def validate_mesh_manifold(vertices: list[list[float]], faces: list[list[int]]) -> dict:
    """
    Validate whether a triangle mesh satisfies 2-manifold topology and compute Euler characteristic.
    """
    V = len(vertices)
    F = len(faces)

    edge_count = {}
    for face in faces:
        assert len(face) == 3, "Only triangular faces supported"
        edges = [
            tuple(sorted([face[0], face[1]])),
            tuple(sorted([face[1], face[2]])),
            tuple(sorted([face[2], face[0]])),
        ]
        for edge in edges:
            edge_count[edge] = edge_count.get(edge, 0) + 1

    E = len(edge_count)
    euler_chi = V - E + F

    # Check for non-manifold edges (> 2 faces sharing an edge)
    non_manifold_edges = [edge for edge, count in edge_count.items() if count > 2]
    boundary_edges = [edge for edge, count in edge_count.items() if count == 1]

    is_closed = len(boundary_edges) == 0
    estimated_genus = (2 - euler_chi) // 2 if is_closed else None

    return {
        "V": V, "E": E, "F": F,
        "euler_characteristic": euler_chi,
        "is_manifold": len(non_manifold_edges) == 0,
        "is_closed": is_closed,
        "estimated_genus": estimated_genus,
        "boundary_edges_count": len(boundary_edges)
    }
```

### 3.2 Discrete Angle Defect (Gaussian Curvature) in Rust
```rust
use std::f32::consts::PI;

pub fn discrete_vertex_curvature(
    vertex_neighbors: &[usize],
    neighbor_angles: &[f32]
) -> f32 {
    let angle_sum: f32 = neighbor_angles.iter().sum();
    // Angle defect: K_v = 2*pi - sum(theta_i)
    2.0 * PI - angle_sum
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for 3D Geometry
```markdown
When generating or manipulating 3D meshes:
- Enforce manifold topology: every edge must be shared by exactly 1 (boundary) or 2 (interior) faces.
- Validate the mesh using Euler-Poincaré invariant: chi = V - E + F.
- Use discrete angle defect to detect saddle points (negative curvature) and convex peaks (positive curvature).
```
