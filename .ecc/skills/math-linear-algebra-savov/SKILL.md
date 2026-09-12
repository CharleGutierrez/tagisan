---
name: math-linear-algebra-savov
description: Linear algebra fundamentals: vector spaces, basis, dot and cross products, orthogonal projections, Gram-Schmidt orthogonalization, matrix transformations, eigenvalues/eigenvectors, and Singular Value Decomposition (SVD) based on 'No Bullshit Guide to Linear Algebra' by Ivan Savov & 'Linear Algebra Done Right' by Sheldon Axler. Triggers: linear-algebra, linear-algebra-savov, linear-algebra-axler, vector-space, eigenvalues, svd, singular-value-decomposition, orthogonal-projection, gram-schmidt, cosine-similarity.
triggers:
  - linear-algebra
  - linear-algebra-savov
  - linear-algebra-axler
  - vector-space
  - eigenvalues
  - svd
  - singular-value-decomposition
  - orthogonal-projection
  - gram-schmidt
  - cosine-similarity
---

# Linear Algebra & SVD: Latent Geometry & Projections

## 1. Mathematical Foundations & Formal Principles

### 1.1 Vector Spaces & Inner Product
A vector space $V$ over field $\mathbb{R}$ satisfies vector addition and scalar multiplication axioms.
For $\mathbf{u}, \mathbf{v} \in \mathbb{R}^n$, the standard inner product (dot product) is:
$$\langle \mathbf{u}, \mathbf{v} \rangle = \mathbf{u}^T \mathbf{v} = \sum_{i=1}^n u_i v_i = \|\mathbf{u}\| \|\mathbf{v}\| \cos\theta$$

Cosine similarity:
$$\text{sim}(\mathbf{u}, \mathbf{v}) = \frac{\mathbf{u} \cdot \mathbf{v}}{\|\mathbf{u}\|_2 \|\mathbf{v}\|_2 + \epsilon} \in [-1, 1]$$

### 1.2 Orthogonal Projections & Gram-Schmidt
The orthogonal projection of vector $\mathbf{v}$ onto subspace $W = \text{span}(\mathbf{u})$:
$$\text{proj}_{\mathbf{u}}(\mathbf{v}) = \frac{\langle \mathbf{u}, \mathbf{v} \rangle}{\|\mathbf{u}\|^2 + \epsilon} \mathbf{u}$$

The component of $\mathbf{v}$ orthogonal to $\mathbf{u}$ is the rejection:
$$\mathbf{v}_{\perp} = \mathbf{v} - \text{proj}_{\mathbf{u}}(\mathbf{v})$$

Gram-Schmidt process for basis $\{\mathbf{v}_1, \dots, \mathbf{v}_k\}$:
$$\mathbf{u}_1 = \mathbf{v}_1, \quad \mathbf{e}_1 = \frac{\mathbf{u}_1}{\|\mathbf{u}_1\|}$$
$$\mathbf{u}_i = \mathbf{v}_i - \sum_{j=1}^{i-1} \langle \mathbf{v}_i, \mathbf{e}_j \rangle \mathbf{e}_j, \quad \mathbf{e}_i = \frac{\mathbf{u}_i}{\|\mathbf{u}_i\|}$$

### 1.3 Eigenvalues & Eigenvectors
For square matrix $\mathbf{A} \in \mathbb{R}^{n \times n}$:
$$\mathbf{A} \mathbf{v} = \lambda \mathbf{v} \iff (\mathbf{A} - \lambda \mathbf{I})\mathbf{v} = \mathbf{0}$$
Roots of characteristic polynomial $\det(\mathbf{A} - \lambda \mathbf{I}) = 0$ yield eigenvalues $\lambda_i$.
For symmetric matrix $\mathbf{A} = \mathbf{A}^T$, spectral decomposition guarantees orthogonal eigenvectors $\mathbf{Q}$:
$$\mathbf{A} = \mathbf{Q} \mathbf{\Lambda} \mathbf{Q}^T$$

### 1.4 Singular Value Decomposition (SVD)
Every matrix $\mathbf{A} \in \mathbb{R}^{m \times n}$ has an SVD factorization:
$$\mathbf{A} = \mathbf{U} \mathbf{\Sigma} \mathbf{V}^T = \sum_{i=1}^r \sigma_i \mathbf{u}_i \mathbf{v}_i^T$$
Where:
- $\mathbf{U} \in \mathbb{R}^{m \times m}$ is orthogonal (left singular vectors, eigenvectors of $\mathbf{A}\mathbf{A}^T$).
- $\mathbf{\Sigma} \in \mathbb{R}^{m \times n}$ is diagonal with singular values $\sigma_1 \ge \sigma_2 \ge \dots \ge \sigma_r \ge 0$.
- $\mathbf{V} \in \mathbb{R}^{n \times n}$ is orthogonal (right singular vectors, eigenvectors of $\mathbf{A}^T\mathbf{A}$).

**Eckart-Young-Mirsky Theorem**: The rank-$k$ truncated SVD $\mathbf{A}_k = \sum_{i=1}^k \sigma_i \mathbf{u}_i \mathbf{v}_i^T$ is the optimal rank-$k$ approximation minimizing Frobenius norm $\|\mathbf{A} - \mathbf{A}_k\|_F$.

---

## 2. The Vibe Coding Superpower

Linear algebra is the native language of AI models and embeddings:
1. **Semantic Search & RAG**: Cosine similarity is inner product of normalized vectors. Pre-normalizing embedding vectors turns search into high-speed matrix multiplication $\mathbf{S} = \mathbf{Q} \mathbf{K}^T$.
2. **Concept Steering & Latent Interventions**: Extract direction vector $\mathbf{v}_{\text{concept}} = \boldsymbol{\mu}_{\text{positive}} - \boldsymbol{\mu}_{\text{negative}}$. Steer latent embeddings: $\mathbf{z}' = \mathbf{z} + \alpha \mathbf{v}_{\text{concept}}$.
3. **Weight & KV-Cache Compression**: Low-rank SVD truncation shrinks embedding matrices and context caches without noticeable loss of reasoning accuracy.
4. **Dimension Orthogonalization**: Remove unwanted style/bias features using orthogonal projection rejection: $\mathbf{x}_{\text{debiased}} = \mathbf{x} - \text{proj}_{\mathbf{b}}(\mathbf{x})$.

---

## 3. Production Code Implementations

### 3.1 Cosine Similarity & Concept Projection in Python
```python
import numpy as np

def stable_cosine_similarity(a: np.ndarray, b: np.ndarray, eps: float = 1e-8) -> np.ndarray:
    """
    Compute pairwise or batch cosine similarity with numerical zero guards.
    """
    a_norm = a / np.maximum(np.linalg.norm(a, axis=-1, keepdims=True), eps)
    b_norm = b / np.maximum(np.linalg.norm(b, axis=-1, keepdims=True), eps)
    return np.dot(a_norm, b_norm.T)

def project_and_reject(v: np.ndarray, direction: np.ndarray, eps: float = 1e-8) -> tuple[np.ndarray, np.ndarray]:
    """
    Decompose vector v into parallel projection along direction and orthogonal rejection.
    v = v_parallel + v_orthogonal
    """
    u = direction / np.maximum(np.linalg.norm(direction), eps)
    v_parallel = np.dot(v, u) * u
    v_orthogonal = v - v_parallel
    return v_parallel, v_orthogonal

def truncated_svd_compress(matrix: np.ndarray, rank: int) -> tuple[np.ndarray, np.ndarray]:
    """
    Low-rank factorization matrix ≈ U_k @ V_k_T using SVD for memory optimization.
    """
    u, s, vt = np.linalg.svd(matrix, full_matrices=False)
    k = min(rank, len(s))
    u_k = u[:, :k] * np.sqrt(s[:k])
    vt_k = np.sqrt(s[:k, np.newaxis]) * vt[:k, :]
    return u_k, vt_k
```

### 3.2 High-Performance Dot Product & Projection in Rust
```rust
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have equal length");
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-8);
    (dot / denom).clamp(-1.0, 1.0)
}

pub fn gram_schmidt_orthogonalize(vectors: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let mut basis: Vec<Vec<f32>> = Vec::with_capacity(vectors.len());

    for v in vectors {
        let mut u = v.clone();
        for b in &basis {
            let dot_uv: f32 = v.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            for (u_i, b_i) in u.iter_mut().zip(b.iter()) {
                *u_i -= dot_uv * b_i;
            }
        }
        let norm: f32 = u.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        for x in u.iter_mut() {
            *x /= norm;
        }
        basis.push(u);
    }
    basis
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Vector Projections
```markdown
When generating embeddings, search algorithms, or vector similarity routines:
- Always enforce L2 normalization before distance comparisons.
- Always add epsilon (1e-8) to vector norms to prevent NaN from zero vectors.
- For concept neutralization or debiasing, project the embedding onto the bias vector and subtract it.
- Never use Euclidean distance directly on high-dimensional raw embeddings without normalization.
```
