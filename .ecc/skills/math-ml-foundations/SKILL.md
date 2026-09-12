---
name: math-ml-foundations
description: Mathematics for Machine Learning: vector calculus, Jacobians, Hessians, multivariate Gaussian distributions, PCA via eigendecomposition, and Lagrange multipliers based on 'Mathematics for Machine Learning' by Deisenroth, Faisal, and Ong. Triggers: ml-foundations, mathematics-for-machine-learning, deisenroth, vector-calculus-ml, jacobian-hessian, multivariate-gaussian, pca-eigendecomposition, lagrange-multipliers, custom-loss-functions, loss-landscape.
triggers:
  - ml-foundations
  - mathematics-for-machine-learning
  - deisenroth
  - vector-calculus-ml
  - jacobian-hessian
  - multivariate-gaussian
  - pca-eigendecomposition
  - lagrange-multipliers
  - custom-loss-functions
  - loss-landscape
---

# ML Foundations: Calculus, Gaussians & Loss Landscapes

## 1. Mathematical Foundations & Formal Principles

### 1.1 Vector & Matrix Calculus
For vector function $\mathbf{f}: \mathbb{R}^n \to \mathbb{R}^m$, the **Jacobian Matrix** $\mathbf{J} \in \mathbb{R}^{m \times n}$:
$$\mathbf{J}_{ij} = \frac{\partial f_i}{\partial x_j}$$
For scalar loss $L: \mathbb{R}^n \to \mathbb{R}$, the **Hessian Matrix** $\mathbf{H} \in \mathbb{R}^{n \times n}$ contains second partial derivatives:
$$\mathbf{H}_{ij} = \frac{\partial^2 L}{\partial x_i \partial x_j}$$
- If $\mathbf{H} \succ 0$ (positive definite): $L$ is strictly convex at $\mathbf{x}$ (local minimum).
- If $\mathbf{H}$ has mixed eigenvalue signs: $\mathbf{x}$ is a saddle point.

### 1.2 Multivariate Gaussian Distribution
For $\mathbf{x} \in \mathbb{R}^D$ with mean $\boldsymbol{\mu}$ and positive-definite covariance $\boldsymbol{\Sigma}$:
$$\mathcal{N}(\mathbf{x} \mid \boldsymbol{\mu}, \boldsymbol{\Sigma}) = \frac{1}{(2\pi)^{D/2} |\boldsymbol{\Sigma}|^{1/2}} \exp\left(-\frac{1}{2}(\mathbf{x} - \boldsymbol{\mu})^T \boldsymbol{\Sigma}^{-1}(\mathbf{x} - \boldsymbol{\mu})\right)$$
Log-likelihood:
$$\ln \mathcal{N} = -\frac{D}{2}\ln(2\pi) - \frac{1}{2}\ln|\boldsymbol{\Sigma}| - \frac{1}{2}(\mathbf{x} - \boldsymbol{\mu})^T \boldsymbol{\Sigma}^{-1}(\mathbf{x} - \boldsymbol{\mu})$$

### 1.3 Principal Component Analysis (PCA)
Given zero-centered data matrix $\mathbf{X} \in \mathbb{R}^{N \times D}$, sample covariance matrix:
$$\mathbf{S} = \frac{1}{N} \mathbf{X}^T \mathbf{X}$$
Eigen-decomposition of $\mathbf{S}$:
$$\mathbf{S} \mathbf{u}_i = \lambda_i \mathbf{u}_i, \quad \lambda_1 \ge \lambda_2 \ge \dots \ge \lambda_D \ge 0$$
First principal component $\mathbf{u}_1$ maximizes projected variance $\mathbf{u}_1^T \mathbf{S} \mathbf{u}_1$ subject to $\|\mathbf{u}_1\|_2 = 1$.

### 1.4 Constrained Optimization via Lagrange Multipliers
To minimize $f(\mathbf{x})$ subject to equality constraints $g_i(\mathbf{x}) = 0$:
$$\mathcal{L}(\mathbf{x}, \boldsymbol{\lambda}) = f(\mathbf{x}) + \sum_{i=1}^m \lambda_i g_i(\mathbf{x})$$
Stationary points satisfy $\nabla_{\mathbf{x}} \mathcal{L} = \mathbf{0}$ and $\nabla_{\boldsymbol{\lambda}} \mathcal{L} = \mathbf{0}$.

---

## 2. The Vibe Coding Superpower

1. **Debugging PyTorch Shapes & Gradients**: Understanding Jacobians and tensor contractions prevents broadcasting bugs and silent gradient zeroing.
2. **Designing Custom Loss Functions**: Authoring penalty terms (e.g. gradient norm clipping, entropy regularization) that maintain positive semi-definite loss curvature.
3. **Dimensionality Reduction in Visual Dashboards**: Compress embedding vectors to 2D/3D using PCA without library bloat.

---

## 3. Production Code Implementations

### 3.1 PCA from Covariance Eigendecomposition in Python
```python
import numpy as np

def compute_pca(data: np.ndarray, n_components: int = 2) -> tuple[np.ndarray, np.ndarray]:
    """
    Principal Component Analysis via covariance matrix eigendecomposition.
    """
    # 1. Center the data
    mean = np.mean(data, axis=0)
    centered = data - mean

    # 2. Covariance matrix S = (1/N) * X^T * X
    n_samples = len(data)
    cov = np.dot(centered.T, centered) / max(n_samples - 1, 1)

    # 3. Eigendecomposition (eigh for symmetric matrix)
    eigenvalues, eigenvectors = np.linalg.eigh(cov)

    # 4. Sort descending
    idx = np.argsort(eigenvalues)[::-1]
    top_eigenvectors = eigenvectors[:, idx[:n_components]]
    top_eigenvalues = eigenvalues[idx[:n_components]]

    # 5. Project data
    projected = np.dot(centered, top_eigenvectors)
    return projected, top_eigenvectors
```

### 3.2 Multivariate Gaussian Density with Cholesky Guard in Rust
```rust
pub struct MultivariateGaussian {
    pub mean: Vec<f64>,
    pub cov_diag: Vec<f64>, // Diagonal covariance for O(D) stability
}

impl MultivariateGaussian {
    pub fn log_prob(&self, x: &[f64]) -> f64 {
        assert_eq!(x.len(), self.mean.len());
        let d = x.len() as f64;
        let mut mahalanobis = 0.0;
        let mut log_det = 0.0;

        for i in 0..x.len() {
            let var = self.cov_diag[i].max(1e-8);
            let diff = x[i] - self.mean[i];
            mahalanobis += (diff * diff) / var;
            log_det += var.ln();
        }

        -0.5 * (d * std::f64::consts::PI.ln() + log_det + mahalanobis)
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Loss Engineering
```markdown
When designing neural loss functions or ML objective formulations:
- Verify that the loss function is bounded below (e.g. L >= 0).
- Check Hessian curvature: avoid saturating gradients (vanishing gradients) in saturated activation regimes.
- Use Cholesky decomposition or diagonal jitter (eps * I) when inverting covariance matrices to guarantee positive-definiteness.
```
