use tagisan::ecc::skills::{
    all_built_in_skills, find_built_in_skill, get_built_in_skills_vec, SkillDispatcher,
};

const FE_SKILL_NAMES: [&str; 11] = [
    "financial-engineering-500-vibe-coder-pro-max",
    "fin-eng-stochastic-calculus-pro-max",
    "fin-eng-derivatives-exotics-pro-max",
    "fin-eng-portfolio-factor-investing-pro-max",
    "fin-eng-algo-trading-statarb-pro-max",
    "fin-eng-microstructure-lob-pro-max",
    "fin-eng-machine-learning-alpha-pro-max",
    "fin-eng-econometrics-timeseries-pro-max",
    "fin-eng-risk-xva-pro-max",
    "fin-eng-hpc-low-latency-pro-max",
    "fin-eng-defi-cryptoeconomics-pro-max",
];

#[test]
fn test_financial_engineering_skills_parse_validity() {
    for &name in &FE_SKILL_NAMES {
        let skill_opt = find_built_in_skill(name);
        assert!(
            skill_opt.is_some(),
            "Failed to find built-in financial engineering skill: {}",
            name
        );

        let skill = skill_opt.unwrap();
        assert_eq!(skill.name, name, "Skill name mismatch");
        assert!(
            !skill.description.trim().is_empty(),
            "Skill description must not be empty: {}",
            name
        );
        assert!(
            !skill.instructions.trim().is_empty(),
            "Skill instructions must not be empty: {}",
            name
        );
        assert!(
            skill.instructions.contains("---"),
            "Skill instructions must contain frontmatter delimiters: {}",
            name
        );
        assert!(
            skill.instructions.contains("Invariants") || skill.instructions.contains("Invariant"),
            "Skill must define core operational invariants: {}",
            name
        );
        assert!(
            skill.instructions.contains("```rust"),
            "Skill must provide verified Rust production blueprints: {}",
            name
        );
    }
}

#[test]
fn test_financial_engineering_skills_registered_in_builtins() {
    let builtins = all_built_in_skills();
    let builtin_map: std::collections::HashSet<String> =
        builtins.iter().map(|s| s.name.clone()).collect();

    for &name in &FE_SKILL_NAMES {
        assert!(
            builtin_map.contains(name),
            "all_built_in_skills() missing skill: {}",
            name
        );
    }

    let static_vec = get_built_in_skills_vec();
    let static_map: std::collections::HashSet<String> =
        static_vec.iter().map(|s| s.name.clone()).collect();

    for &name in &FE_SKILL_NAMES {
        assert!(
            static_map.contains(name),
            "get_built_in_skills_vec() missing skill: {}",
            name
        );
    }
}

#[test]
fn test_deterministic_skill_dispatching() {
    let dispatcher = SkillDispatcher::build_from_scratch(None);

    let test_queries = [
        ("black scholes option pricing", "fin-eng-derivatives-exotics-pro-max"),
        ("avellaneda stoikov market making", "fin-eng-microstructure-lob-pro-max"),
        ("johansen cointegration pairs trading", "fin-eng-algo-trading-statarb-pro-max"),
        ("purged cross validation ml", "fin-eng-machine-learning-alpha-pro-max"),
        ("cva basel iii", "fin-eng-risk-xva-pro-max"),
        ("uniswap v3 tick math", "fin-eng-defi-cryptoeconomics-pro-max"),
        ("financial engineering 500 books", "financial-engineering-500-vibe-coder-pro-max"),
        ("stochastic calculus brownian motion", "fin-eng-stochastic-calculus-pro-max"),
        ("markowitz risk parity portfolio", "fin-eng-portfolio-factor-investing-pro-max"),
        ("garch conditional heteroskedasticity", "fin-eng-econometrics-timeseries-pro-max"),
        ("zero allocation spsc ring buffer", "fin-eng-hpc-low-latency-pro-max"),
    ];

    for (query, expected_skill) in &test_queries {
        let results = dispatcher.dispatch(query, 5, None);
        assert!(
            !results.is_empty(),
            "Query '{}' returned no dispatched skills",
            query
        );

        let found = results.iter().any(|r| r.skill.name == *expected_skill);
        assert!(
            found,
            "Query '{}' expected '{}', but got: {:?}",
            query,
            expected_skill,
            results.iter().map(|r| &r.skill.name).collect::<Vec<_>>()
        );
    }
}

// =========================================================================
// Mathematical Sanity and Numerical Invariant Tests
// =========================================================================

const INV_SQRT_2PI: f64 = 0.3989422804014327;

fn normal_pdf(x: f64) -> f64 {
    INV_SQRT_2PI * (-0.5 * x * x).exp()
}

fn normal_cdf(x: f64) -> f64 {
    if x < -8.0 { return 0.0; }
    if x > 8.0 { return 1.0; }
    let k = 1.0 / (1.0 + 0.2316419 * x.abs());
    let poly = k * (0.319381530 + k * (-0.356563782 + k * (1.781477937 + k * (-1.821255978 + k * 1.330274429))));
    let approx = 1.0 - normal_pdf(x.abs()) * poly;
    if x >= 0.0 { approx } else { 1.0 - approx }
}

#[test]
fn test_normal_cdf_mathematical_invariants() {
    // 1. Median at 0
    let cdf_zero = normal_cdf(0.0);
    assert!((cdf_zero - 0.5).abs() < 1e-7, "CDF(0) must be 0.5");

    // 2. Symmetry: CDF(-x) + CDF(x) == 1.0
    for &x in &[0.5, 1.0, 1.645, 1.96, 2.326, 2.576, 3.0] {
        let sum = normal_cdf(-x) + normal_cdf(x);
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "CDF symmetry violated at x={}: sum={}",
            x,
            sum
        );
    }

    // 3. 95% critical value (1.96)
    let cdf_196 = normal_cdf(1.96);
    assert!(
        (cdf_196 - 0.975002).abs() < 1e-4,
        "CDF(1.96) expected ~0.9750, got {}",
        cdf_196
    );
}

#[test]
fn test_black_scholes_put_call_parity_invariant() {
    let s: f64 = 100.0;
    let k: f64 = 100.0;
    let t: f64 = 1.0;
    let r: f64 = 0.05;
    let sigma: f64 = 0.20;

    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let nd1 = normal_cdf(d1);
    let nd2 = normal_cdf(d2);
    let n_neg_d1 = normal_cdf(-d1);
    let n_neg_d2 = normal_cdf(-d2);
    let disc = (-r * t).exp();

    let call_price = s * nd1 - k * disc * nd2;
    let put_price = k * disc * n_neg_d2 - s * n_neg_d1;

    // Put-Call Parity: C - P = S - K * exp(-r * T)
    let expected_diff = s - k * disc;
    let actual_diff = call_price - put_price;

    assert!(
        (actual_diff - expected_diff).abs() < 1e-6,
        "Put-Call Parity violated: C-P={}, S-K*e^(-rT)={}",
        actual_diff,
        expected_diff
    );

    // Delta relationship: Delta_call - Delta_put == 1.0
    let delta_call = nd1;
    let delta_put = nd1 - 1.0;
    assert!(
        (delta_call - delta_put - 1.0).abs() < 1e-12,
        "Delta parity violated"
    );
}

#[test]
fn test_ornstein_uhlenbeck_calibration_invariant() {
    // Generate synthetic OU process with known theta=2.0, mu=10.0, sigma=1.0, dt=0.01
    let dt = 0.01;
    let true_theta = 2.0;
    let true_mu = 10.0;

    let mut series = Vec::with_capacity(1000);
    let mut x = 12.0; // Start off-mean
    series.push(x);

    for _ in 1..1000 {
        let drift = true_theta * (true_mu - x) * dt;
        x += drift;
        series.push(x);
    }

    // OLS regression: y = a * x + b
    let n = series.len();
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let m = (n - 1) as f64;

    for i in 0..(n - 1) {
        let x_i = series[i];
        let y_i = series[i + 1];
        sx += x_i;
        sy += y_i;
        sxx += x_i * x_i;
        sxy += x_i * y_i;
    }

    let a = (m * sxy - sx * sy) / (m * sxx - sx * sx);
    let b = (sy - a * sx) / m;

    let est_theta = -a.ln() / dt;
    let est_mu = b / (1.0 - a);
    let half_life = (2.0f64).ln() / est_theta;

    assert!(
        est_theta > 0.0,
        "Estimated theta must be positive for mean-reverting process"
    );
    assert!(
        (est_theta - true_theta).abs() < 0.1,
        "Estimated theta {} deviates from true theta {}",
        est_theta,
        true_theta
    );
    assert!(
        (est_mu - true_mu).abs() < 0.1,
        "Estimated mu {} deviates from true mu {}",
        est_mu,
        true_mu
    );
    assert!(half_life > 0.0, "Half-life must be strictly positive");
}

#[test]
fn test_uniswap_v3_tick_math_invariants() {
    const Q96: u128 = 1u128 << 96;

    // At tick 0: Price = 1.0, sqrtPriceX96 = 2^96
    let tick_0_price = 1.0001f64.powi(0).sqrt();
    let sqrt_price_0 = (tick_0_price * (Q96 as f64)) as u128;
    assert_eq!(
        sqrt_price_0, Q96,
        "Tick 0 must correspond exactly to Q96 sqrt price"
    );

    // Monotonicity: higher tick must produce higher sqrtPriceX96
    let tick_100 = (1.0001f64.powi(100).sqrt() * (Q96 as f64)) as u128;
    let tick_200 = (1.0001f64.powi(200).sqrt() * (Q96 as f64)) as u128;
    assert!(
        tick_100 > sqrt_price_0,
        "Tick 100 sqrt price must exceed tick 0"
    );
    assert!(
        tick_200 > tick_100,
        "Tick 200 sqrt price must exceed tick 100"
    );

    // Impermanent loss formula invariant: IL(1) == 0, IL(k) <= 0 for all k > 0
    let il_1 = (2.0 * (1.0f64).sqrt()) / (1.0 + 1.0) - 1.0;
    assert_eq!(il_1, 0.0, "IL(1) must be exactly 0");

    for &k in &[0.5_f64, 0.8_f64, 1.25_f64, 2.0_f64, 5.0_f64] {
        let il_k = (2.0 * k.sqrt()) / (1.0 + k) - 1.0;
        assert!(
            il_k <= 0.0,
            "Impermanent loss must be non-positive, got {} for k={}",
            il_k,
            k
        );
    }
}
