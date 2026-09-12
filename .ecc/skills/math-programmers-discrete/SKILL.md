---
name: math-programmers-discrete
description: Discrete mathematics for programmers: modular arithmetic, RSA cryptography, spectral graph theory, Graph Laplacian, Fast Fourier Transform (FFT), and discrete convolution based on 'A Programmer's Introduction to Mathematics' by Jeremy Kun. Triggers: programmers-discrete, jeremy-kun, fft, fast-fourier-transform, spectral-graph-theory, graph-laplacian, modular-arithmetic, discrete-convolution, roots-of-unity, fiedler-vector.
triggers:
  - programmers-discrete
  - jeremy-kun
  - fft
  - fast-fourier-transform
  - spectral-graph-theory
  - graph-laplacian
  - modular-arithmetic
  - discrete-convolution
  - roots-of-unity
  - fiedler-vector
---

# Discrete Math: FFT, Spectral Graphs & Modular Arithmetic

## 1. Mathematical Foundations & Formal Principles

### 1.1 Fast Fourier Transform (FFT)
The Discrete Fourier Transform (DFT) maps vector $\mathbf{x} \in \mathbb{C}^N$ to frequency domain $\mathbf{X} \in \mathbb{C}^N$:
$$X_k = \sum_{n=0}^{N-1} x_n \omega_N^{nk}, \quad \omega_N = e^{-i \frac{2\pi}{N}}$$
Direct computation requires $O(N^2)$ operations.
**Cooley-Tukey Radix-2 FFT**: Decompose into even and odd indices:
$$X_k = E_k + \omega_N^k O_k, \quad X_{k + N/2} = E_k - \omega_N^k O_k$$
Reduces complexity to $O(N \log N)$.
**Convolution Theorem**: $f * g = \text{IFFT}(\text{FFT}(f) \odot \text{FFT}(g))$ in $O(N \log N)$ vs $O(N^2)$ direct sum.

### 1.2 Spectral Graph Theory & The Graph Laplacian
For graph $G = (V, E)$ with degree matrix $\mathbf{D}$ and adjacency matrix $\mathbf{A}$:
$$\text{Unnormalized Laplacian: } \mathbf{L} = \mathbf{D} - \mathbf{A}$$
$$\text{Normalized Laplacian: } \mathbf{L}_{\text{norm}} = \mathbf{D}^{-1/2}\mathbf{L}\mathbf{D}^{-1/2} = \mathbf{I} - \mathbf{D}^{-1/2}\mathbf{A}\mathbf{D}^{-1/2}$$
Properties:
1. $\mathbf{L}$ is symmetric and positive semi-definite ($\mathbf{x}^T \mathbf{L} \mathbf{x} = \sum_{(u, v) \in E} (x_u - x_v)^2 \ge 0$).
2. Smallest eigenvalue $\lambda_0 = 0$ with eigenvector $\mathbf{1}$.
3. Multiplicity of eigenvalue 0 equals the number of connected components.
4. **Fiedler Vector**: Eigenvector corresponding to second-smallest eigenvalue $\lambda_1$ provides optimal graph spectral bisection.

---

## 2. The Vibe Coding Superpower

1. **Real-Time Audio & Signal Analysis**: Compute frequency spectrums in Web Audio or canvas visualizers with 60 FPS performance.
2. **Spectral Community Clustering**: Cluster complex microservice dependency graphs or agent conversation networks via Fiedler vector partitioning.
3. **High-Speed Cryptographic Operations**: Perform modular arithmetic and Shamir secret sharing for agent consensus credentials.

---

## 3. Production Code Implementations

### 3.1 Radix-2 Cooley-Tukey FFT in Python
```python
import numpy as np

def cooley_tukey_fft(x: np.ndarray) -> np.ndarray:
    """
    Cooley-Tukey Radix-2 FFT algorithm in O(N log N).
    Input length must be a power of 2.
    """
    n = len(x)
    if n <= 1:
        return x.astype(complex)

    even = cooley_tukey_fft(x[0::2])
    odd = cooley_tukey_fft(x[1::2])

    factor = np.exp(-2j * np.pi * np.arange(n // 2) / n)
    return np.concatenate([
        even + factor * odd,
        even - factor * odd
    ])

def spectral_graph_bisection(adj_matrix: np.ndarray) -> np.ndarray:
    """
    Partition graph nodes into two balanced clusters using the Fiedler vector.
    """
    d = np.diag(np.sum(adj_matrix, axis=1))
    laplacian = d - adj_matrix

    eigenvalues, eigenvectors = np.linalg.eigh(laplacian)
    fiedler_vector = eigenvectors[:, 1]  # eigenvector for second smallest eigenvalue
    return (fiedler_vector >= 0).astype(int)
```

### 3.2 Modular Inverse in Rust (Extended Euclidean Algorithm)
```rust
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        (b, 0, 1)
    } else {
        let (g, x, y) = extended_gcd(b % a, a);
        (g, y - (b / a) * x, x)
    }
}

pub fn mod_inverse(a: i64, m: i64) -> Option<i64> {
    let (g, x, _) = extended_gcd(a, m);
    if g == 1 {
        Some((x % m + m) % m)
    } else {
        None // Inverse does not exist
    }
}
```

---

## 4. Vibe Coder Prompt Engineering Recipes

### 4.1 System Prompt Directive for Discrete Spectral Analysis
```markdown
When implementing signal processing or graph partitioning:
- Compute convolutions via FFT multiplication: IFFT(FFT(A) * FFT(B)).
- For spectral graph clustering, construct the Laplacian L = D - A and partition using the sign of the Fiedler eigenvector.
- For modular exponentiation, always use binary exponentiation to prevent 64-bit integer overflow.
```
