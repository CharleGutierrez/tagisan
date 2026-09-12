//! Brutal Integration Tests for Top 20 Mathematics Skills for Vibe Code Developers in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

const MATH_SKILLS_20: [&str; 20] = [
    "math-nature-of-code",
    "math-linear-algebra-savov",
    "math-high-dimensional-data",
    "math-information-inference",
    "math-causal-inference",
    "math-category-theory",
    "math-game-engine-geometry",
    "math-code-first-calculus",
    "math-bayesian-reasoning",
    "math-ml-foundations",
    "math-networks-crowds-markets",
    "math-convex-optimization",
    "math-probability-logic",
    "math-programmers-discrete",
    "math-intuitive-calculus",
    "math-concrete-discrete",
    "math-visual-topology",
    "math-chaos-fractals",
    "math-numerical-methods",
    "math-algorithmic-game-theory",
];

// =========================================================================
// 1. Discovery & Catalog Invariants
// =========================================================================

#[test]
fn test_all_20_math_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &MATH_SKILLS_20 {
        assert!(
            loaded_map.contains(*skill),
            "Skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

#[test]
fn test_all_20_math_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 65,
        "Expected at least 65 total built-in skills including math, found {}",
        all_skills.len()
    );

    for skill_name in &MATH_SKILLS_20 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Math skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty());
        assert!(!s.description.is_empty());
        assert!(!s.instructions.is_empty());
        assert!(
            s.instructions.len() > 100,
            "Skill '{}' must contain concrete mathematical instructions",
            skill_name
        );
    }
}

// =========================================================================
// 2. High-Leverage Vibe Coder Alias Lookups
// =========================================================================

#[test]
fn test_vibe_math_skills_alias_lookups() {
    // Nature of Code / Boids
    assert_eq!(find_ecc_skill("boids").unwrap().name, "math-nature-of-code");
    assert_eq!(find_ecc_skill("flocking").unwrap().name, "math-nature-of-code");
    assert_eq!(find_ecc_skill("nature-of-code").unwrap().name, "math-nature-of-code");

    // Game Engine / Quaternions
    assert_eq!(find_ecc_skill("quaternions").unwrap().name, "math-game-engine-geometry");
    assert_eq!(find_ecc_skill("slerp").unwrap().name, "math-game-engine-geometry");

    // Information Theory / Entropy
    assert_eq!(find_ecc_skill("shannon-entropy").unwrap().name, "math-information-inference");
    assert_eq!(find_ecc_skill("token-entropy").unwrap().name, "math-information-inference");

    // Causality / Do-Calculus
    assert_eq!(find_ecc_skill("do-calculus").unwrap().name, "math-causal-inference");
    assert_eq!(find_ecc_skill("causal-inference").unwrap().name, "math-causal-inference");

    // Category Theory / Monads
    assert_eq!(find_ecc_skill("monads").unwrap().name, "math-category-theory");
    assert_eq!(find_ecc_skill("category-theory").unwrap().name, "math-category-theory");

    // Numerical Methods / RK4
    assert_eq!(find_ecc_skill("rk4").unwrap().name, "math-numerical-methods");
    assert_eq!(find_ecc_skill("numerical-recipes").unwrap().name, "math-numerical-methods");

    // Game Theory / Nash
    assert_eq!(find_ecc_skill("nash-equilibrium").unwrap().name, "math-algorithmic-game-theory");
    assert_eq!(find_ecc_skill("shapley-value").unwrap().name, "math-algorithmic-game-theory");

    // Networks / PageRank
    assert_eq!(find_ecc_skill("pagerank").unwrap().name, "math-networks-crowds-markets");

    // FFT
    assert_eq!(find_ecc_skill("fft").unwrap().name, "math-programmers-discrete");

    // Chaos
    assert_eq!(find_ecc_skill("lorenz-attractor").unwrap().name, "math-chaos-fractals");
}

// =========================================================================
// 3. Semantic Intent Top-K Dispatching for Vibe Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_vibe_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        ("Simulate autonomous flocking boids with Craig Reynolds steering forces in WebGL", "math-nature-of-code"),
        ("Interpolate camera orientation smoothly without gimbal lock using quaternions and slerp", "math-game-engine-geometry"),
        ("Calculate Shannon token entropy and mutual information for probabilistic inference", "math-information-inference"),
        ("Diagnose microservice outage root cause using causal DAG and do-calculus", "math-causal-inference"),
        ("Compose tool execution pipelines with Railway-Oriented Programming and Result Monads", "math-category-theory"),
        ("Thompson Sampling multi-armed bandit routing using Beta-Binomial Bayesian priors", "math-bayesian-reasoning"),
        ("Solve spring-damper cloth simulation without numerical explosion using Runge-Kutta RK4", "math-numerical-methods"),
        ("Shapley value credit attribution and Nash equilibrium in algorithmic game theory", "math-algorithmic-game-theory"),
        ("Simulate Lorenz strange attractor with deterministic chaos in WebGL shader", "math-chaos-fractals"),
        ("Solve constrained convex optimization with Lagrangian duality and KKT conditions", "math-convex-optimization"),
    ];

    for (query, expected_skill) in test_cases {
        let results = dispatcher.dispatch(query, 5, None);
        assert!(!results.is_empty(), "Dispatch returned no results for query '{}'", query);

        let found = results.iter().any(|d| d.skill.name == expected_skill);
        assert!(
            found,
            "Semantic dispatch failed for query '{}': expected '{}' in top 5, got {:?}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );

        let matched = results.iter().find(|d| d.skill.name == expected_skill).unwrap();
        assert!(
            matched.score > 0.0,
            "Dispatched skill '{}' score must be > 0.0, got {}",
            expected_skill,
            matched.score
        );
    }
}

// =========================================================================
// 4. Local (Ollama) vs Cloud Guidelines Discrimination
// =========================================================================

#[test]
fn test_math_skills_local_vs_cloud_formatting() {
    let dispatcher = global_ecc_dispatcher();
    let query = "Simulate autonomous flocking boids with Craig Reynolds steering forces";

    // 1. Local Ollama formatting: condensed cheat sheet
    let (local_prompt, local_skills) = dispatcher.equip_prompt_for_provider(
        "Base system contract",
        query,
        "ollama",
        None,
    );
    assert!(!local_skills.is_empty());
    assert!(
        local_prompt.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local Ollama prompt must contain Cheat Sheet header"
    );
    assert!(
        local_skills.len() <= 2,
        "Local Ollama context must be bounded to <= 2 skills"
    );

    // 2. Cloud Anthropic / OpenAI formatting: comprehensive architectural guidelines
    let (cloud_prompt, cloud_skills) = dispatcher.equip_prompt_for_provider(
        "Base system contract",
        query,
        "anthropic",
        None,
    );
    assert!(!cloud_skills.is_empty());
    assert!(
        cloud_prompt.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"),
        "Cloud prompt must contain Comprehensive Specifications header"
    );
    assert!(
        cloud_prompt.contains("Full Specification & Directives:"),
        "Cloud prompt must contain full directive body"
    );
}

// =========================================================================
// 5. 50-Thread Concurrent Dispatch Stress Test
// =========================================================================

#[test]
fn test_multithreaded_concurrent_math_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "boids flocking simulation with reynolds steering",
        "quaternion camera slerp rotation",
        "shannon entropy hallucination detector",
        "do-calculus causal structural DAG",
        "railway oriented result monad pipeline",
        "thompson sampling multi armed bandit",
        "runge kutta rk4 numerical integration",
        "shapley value attribution multi-agent",
        "lorenz strange attractor chaotic simulation",
        "kkt convex optimization resource allocation",
    ]);

    let thread_count = 50;
    let iterations_per_thread = 20;
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(thread_count);

    let start_time = Instant::now();

    for t_idx in 0..thread_count {
        let d_clone = Arc::clone(&dispatcher);
        let q_clone = Arc::clone(&queries);
        let s_clone = Arc::clone(&success_count);

        handles.push(thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let query = &q_clone[(t_idx + i) % q_clone.len()];
                let results = d_clone.dispatch(query, 3, None);
                if !results.is_empty() && results[0].score > 0.0 {
                    s_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked during concurrent dispatch");
    }

    let elapsed = start_time.elapsed();
    let total_dispatches = thread_count * iterations_per_thread;
    let successful = success_count.load(Ordering::SeqCst);

    assert_eq!(
        successful, total_dispatches,
        "All 50 concurrent dispatch threads must succeed across {} iterations (elapsed: {:?})",
        total_dispatches, elapsed
    );
}

// =========================================================================
// 6. Mathematical & Numerical Safety Invariants
// =========================================================================

#[test]
fn test_mathematical_safety_invariants() {
    // 1. Quaternion Unit Norm Invariant: ||q|| = 1
    let q = [0.70710678f32, 0.0f32, 0.70710678f32, 0.0f32]; // 90 deg around Y
    let norm = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "Quaternion must maintain unit norm");

    // 2. Division-by-Zero Epsilon Guard
    let eps = 1e-8f64;
    let zero_vel = 0.0f64;
    let normalized = zero_vel / (zero_vel.abs() + eps);
    assert!(normalized.is_finite(), "Epsilon guard must prevent division by zero NaN");
    assert_eq!(normalized, 0.0);

    // 3. Shannon Entropy Non-Negativity: H(P) >= 0
    let probs = [0.5f64, 0.25f64, 0.125f64, 0.125f64];
    let entropy: f64 = -probs.iter().map(|p| p * p.log2()).sum::<f64>();
    assert!(entropy >= 0.0, "Entropy must be non-negative, got {}", entropy);
    assert!((entropy - 1.75).abs() < 1e-5, "Expected entropy 1.75 bits, got {}", entropy);

    // 4. Runge-Kutta 4th Order Convergence Check (dy/dt = -y, exact solution y(t) = y0 * e^{-t})
    let y0 = 1.0f64;
    let dt = 0.1f64;
    let f = |y: f64| -y;
    let k1 = f(y0);
    let k2 = f(y0 + 0.5 * dt * k1);
    let k3 = f(y0 + 0.5 * dt * k2);
    let k4 = f(y0 + dt * k3);
    let y1 = y0 + (dt / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
    let y_exact = y0 * (-dt).exp();
    let error = (y1 - y_exact).abs();
    assert!(error < 1e-6, "RK4 error must be O(dt^4), got error {}", error);
}
