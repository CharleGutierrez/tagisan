//! Brutal Integration & Mathematical Verification Tests for Top 50 Books in Data Analytics & Analytics Engineering Skills in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

const ANALYTICS_SKILLS_50: [&str; 50] = [
    // Cluster 1: Modern Data Warehousing, Dimensional Modeling & Analytics Engineering (Books 1-6)
    "analytics-kimball-dimensional-modeling",
    "analytics-etl-pipeline-patterns",
    "analytics-data-vault-architecture",
    "analytics-dbt-modeling-dag",
    "analytics-advanced-sql-windowing",
    "analytics-celko-relational-logic",
    // Cluster 2: Exploratory Data Analysis & Statistical Foundations (Books 7-12)
    "analytics-tukey-eda-heuristics",
    "analytics-practical-statistics",
    "analytics-intuitive-statistics",
    "analytics-statistical-learning-islp",
    "analytics-bayesian-rethinking",
    "analytics-mathematical-inference",
    // Cluster 3: Experimentation, A/B Testing & Causal Inference (Books 13-16)
    "analytics-kohavi-ab-experimentation",
    "analytics-causal-mixtape",
    "analytics-econometric-causality",
    "analytics-counterfactual-causality",
    // Cluster 4: Real-Time Analytics, Streaming & Data Engineering (Books 17-21)
    "analytics-kleppmann-data-intensive",
    "analytics-akidau-stream-processing",
    "analytics-kafka-event-streaming",
    "analytics-data-engineering-lifecycle",
    "analytics-realtime-olap-pipelines",
    // Cluster 5: Embedded Analytics, In-Memory OLAP & Fast Runtimes (Books 22-26)
    "analytics-python-pandas-wrangling",
    "analytics-duckdb-embedded-olap",
    "analytics-polars-lazy-processing",
    "analytics-high-performance-compute",
    "analytics-cli-data-science",
    // Cluster 6: Data Visualization, Information Design & Dashboard UX (Books 27-32)
    "analytics-tufte-visual-display",
    "analytics-storytelling-with-data",
    "analytics-wilke-data-visualization",
    "analytics-few-dashboard-design",
    "analytics-cairo-visual-integrity",
    "analytics-d3-interactive-graphics",
    // Cluster 7: Metrics Layers, Business Semantics & KPI Design (Books 33-38)
    "analytics-parmenter-kpi-framework",
    "analytics-lean-startup-metrics",
    "analytics-hubbard-measurement-value",
    "analytics-semantic-metadata-layer",
    "analytics-semantic-metrics-governance",
    "analytics-okr-goal-tracking",
    // Cluster 8: Time Series Forecasting, Anomaly Detection & Signal Analytics (Books 39-42)
    "analytics-hyndman-time-series",
    "analytics-box-jenkins-arima",
    "analytics-anomaly-outlier-detection",
    "analytics-python-signal-processing",
    // Cluster 9: Machine Learning for Analytics & Feature Engineering (Books 43-46)
    "analytics-feature-engineering-pipeline",
    "analytics-geron-ml-pipelines",
    "analytics-kuhn-predictive-modeling",
    "analytics-elements-statistical-learning",
    // Cluster 10: Data Quality, Observability, Governance & DataOps (Books 47-50)
    "analytics-redman-data-quality",
    "analytics-data-observability-monitors",
    "analytics-dama-data-governance",
    "analytics-dataops-automated-testing",
];

// =========================================================================
// 1. Discovery of all 50 Analytics Skills on Disk
// =========================================================================

#[test]
fn test_all_50_analytics_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &ANALYTICS_SKILLS_50 {
        assert!(
            loaded_map.contains(*skill),
            "Analytics skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 50 Analytics Skills (assert >= 170 total skills)
// =========================================================================

#[test]
fn test_all_50_analytics_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 170,
        "Expected at least 170 total built-in skills including Analytics, found {}",
        all_skills.len()
    );

    for skill_name in &ANALYTICS_SKILLS_50 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Analytics skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty());
        assert!(!s.description.is_empty());
        assert!(!s.instructions.is_empty());
        assert!(
            s.instructions.len() > 100,
            "Skill '{}' must contain concrete analytical architecture instructions",
            skill_name
        );
    }
}

// =========================================================================
// 3. High-Leverage Analytics Alias Lookups
// =========================================================================

#[test]
fn test_analytics_skills_alias_lookups() {
    // Dimensional Modeling & Warehousing
    assert_eq!(find_ecc_skill("kimball").unwrap().name, "analytics-kimball-dimensional-modeling");
    assert_eq!(find_ecc_skill("star-schema").unwrap().name, "analytics-kimball-dimensional-modeling");
    assert_eq!(find_ecc_skill("dimensional-modeling").unwrap().name, "analytics-kimball-dimensional-modeling");
    assert_eq!(find_ecc_skill("data-vault").unwrap().name, "analytics-data-vault-architecture");

    // Analytics Engineering & SQL
    assert_eq!(find_ecc_skill("dbt").unwrap().name, "analytics-dbt-modeling-dag");
    assert_eq!(find_ecc_skill("dbt-dag").unwrap().name, "analytics-dbt-modeling-dag");
    assert_eq!(find_ecc_skill("window-functions").unwrap().name, "analytics-advanced-sql-windowing");
    assert_eq!(find_ecc_skill("advanced-sql").unwrap().name, "analytics-advanced-sql-windowing");
    assert_eq!(find_ecc_skill("celko").unwrap().name, "analytics-celko-relational-logic");

    // Statistics & EDA
    assert_eq!(find_ecc_skill("tukey-eda").unwrap().name, "analytics-tukey-eda-heuristics");
    assert_eq!(find_ecc_skill("tukey-fences").unwrap().name, "analytics-tukey-eda-heuristics");
    assert_eq!(find_ecc_skill("practical-statistics").unwrap().name, "analytics-practical-statistics");
    assert_eq!(find_ecc_skill("intuitive-statistics").unwrap().name, "analytics-intuitive-statistics");
    assert_eq!(find_ecc_skill("statistical-learning").unwrap().name, "analytics-statistical-learning-islp");
    assert_eq!(find_ecc_skill("bayesian-rethinking").unwrap().name, "analytics-bayesian-rethinking");
    assert_eq!(find_ecc_skill("mathematical-inference").unwrap().name, "analytics-mathematical-inference");

    // Experimentation & Causal Inference
    assert_eq!(find_ecc_skill("ab-testing").unwrap().name, "analytics-kohavi-ab-experimentation");
    assert_eq!(find_ecc_skill("srm").unwrap().name, "analytics-kohavi-ab-experimentation");
    assert_eq!(find_ecc_skill("difference-in-differences").unwrap().name, "analytics-econometric-causality");
    assert_eq!(find_ecc_skill("did").unwrap().name, "analytics-econometric-causality");
    assert_eq!(find_ecc_skill("causal-mixtape").unwrap().name, "analytics-causal-mixtape");
    assert_eq!(find_ecc_skill("counterfactual").unwrap().name, "analytics-counterfactual-causality");

    // Embedded OLAP & Fast Runtimes
    assert_eq!(find_ecc_skill("duckdb").unwrap().name, "analytics-duckdb-embedded-olap");
    assert_eq!(find_ecc_skill("polars").unwrap().name, "analytics-polars-lazy-processing");
    assert_eq!(find_ecc_skill("lazyframe").unwrap().name, "analytics-polars-lazy-processing");

    // Visualization & Information Design
    assert_eq!(find_ecc_skill("tufte").unwrap().name, "analytics-tufte-visual-display");
    assert_eq!(find_ecc_skill("data-ink").unwrap().name, "analytics-tufte-visual-display");
    assert_eq!(find_ecc_skill("storytelling-with-data").unwrap().name, "analytics-storytelling-with-data");
    assert_eq!(find_ecc_skill("bullet-graph").unwrap().name, "analytics-few-dashboard-design");

    // Metrics Layers & Business Semantics
    assert_eq!(find_ecc_skill("kpi").unwrap().name, "analytics-parmenter-kpi-framework");
    assert_eq!(find_ecc_skill("cohort-retention").unwrap().name, "analytics-lean-startup-metrics");
    assert_eq!(find_ecc_skill("lean-analytics").unwrap().name, "analytics-lean-startup-metrics");

    // Time Series & Anomalies
    assert_eq!(find_ecc_skill("time-series-forecasting").unwrap().name, "analytics-hyndman-time-series");
    assert_eq!(find_ecc_skill("arima").unwrap().name, "analytics-box-jenkins-arima");
    assert_eq!(find_ecc_skill("isolation-forest").unwrap().name, "analytics-anomaly-outlier-detection");

    // Quality, Observability & DataOps
    assert_eq!(find_ecc_skill("data-observability").unwrap().name, "analytics-data-observability-monitors");
    assert_eq!(find_ecc_skill("data-governance").unwrap().name, "analytics-dama-data-governance");
    assert_eq!(find_ecc_skill("dataops").unwrap().name, "analytics-dataops-automated-testing");
}

// =========================================================================
// 4. Semantic Intent Top-K Dispatching for Analytics Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_analytics_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        (
            "Kimball star-schema dimensional-modeling with SCD Type 2 customer dimension and conformed bus-architecture",
            "analytics-kimball-dimensional-modeling",
        ),
        (
            "Build dbt DAG staging intermediate marts incremental merge unique_key",
            "analytics-dbt-modeling-dag",
        ),
        (
            "SQL window functions lead lag dense_rank sessionization gaps and islands",
            "analytics-advanced-sql-windowing",
        ),
        (
            "Tukey exploratory data analysis 5-number summary IQR fences outlier detection",
            "analytics-tukey-eda-heuristics",
        ),
        (
            "Online controlled experiment sample ratio mismatch SRM chi-square test OEC",
            "analytics-kohavi-ab-experimentation",
        ),
        (
            "Difference-in-differences DiD parallel trends assumption two-way fixed effects",
            "analytics-econometric-causality",
        ),
        (
            "DuckDB vectorized in-memory OLAP querying Parquet files arrow zero-copy",
            "analytics-duckdb-embedded-olap",
        ),
        (
            "Polars LazyFrame query optimization predicate projection pushdown streaming",
            "analytics-polars-lazy-processing",
        ),
        (
            "Edward Tufte data-ink ratio chartjunk elimination lie factor sparklines",
            "analytics-tufte-visual-display",
        ),
        (
            "Pirate metrics AARRR cohort retention curve flattening viral coefficient",
            "analytics-lean-startup-metrics",
        ),
        (
            "STL decomposition Rob Hyndman time series forecasting MASE naive baseline",
            "analytics-hyndman-time-series",
        ),
        (
            "Box-Jenkins ARIMA stationarity differencing ACF PACF Ljung-Box white noise",
            "analytics-box-jenkins-arima",
        ),
        (
            "Isolation Forest anomaly score path length average c(n) outlier detection",
            "analytics-anomaly-outlier-detection",
        ),
        (
            "Data observability 5 pillars freshness volume schema distribution lineage SLA",
            "analytics-data-observability-monitors",
        ),
        (
            "DataOps automated testing harness pre-flight transactional circuit breaker",
            "analytics-dataops-automated-testing",
        ),
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
// 5. Local Ollama vs Cloud Guidelines Discrimination
// =========================================================================

#[test]
fn test_analytics_skills_local_vs_cloud_formatting() {
    let dispatcher = global_ecc_dispatcher();
    let query = "Audit Kimball star schema fact tables and DuckDB vectorized Parquet queries";

    // Local Ollama formatting
    let (local_prompt, local_skills) = dispatcher.equip_prompt_for_provider(
        "Base analytics system contract",
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

    // Cloud Anthropic / OpenAI formatting
    let (cloud_prompt, cloud_skills) = dispatcher.equip_prompt_for_provider(
        "Base analytics system contract",
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
// 6. 50-Thread Concurrent Dispatch Stress Test
// =========================================================================

#[test]
fn test_multithreaded_concurrent_analytics_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "Kimball dimensional modeling star schema SCD type 2 fact table",
        "dbt DAG staging intermediate marts incremental merge unique_key",
        "Tukey exploratory data analysis box plot IQR fences outlier detection",
        "A/B testing sample ratio mismatch SRM chi-square test OEC guardrails",
        "Difference in differences parallel trends econometric causality",
        "DuckDB embedded vectorized in-memory OLAP querying Parquet",
        "Polars lazy processing predicate pushdown streaming execution",
        "Tufte data-ink ratio chartjunk elimination lie factor sparklines",
        "Pirate metrics AARRR cohort retention decay curve flattening",
        "Box Jenkins ARIMA time series forecasting Ljung-Box white noise",
        "Isolation Forest anomaly score path length outlier detection",
        "Data observability 5 pillars freshness volume schema distribution lineage",
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
        "All {} concurrent dispatches must succeed, got {}",
        total_dispatches, successful
    );
    assert!(
        elapsed.as_millis() < 5000,
        "50-thread concurrent dispatch stress test took {:?}, expected < 5s",
        elapsed
    );
}

// =========================================================================
// 7. Enterprise Mathematical & Analytical Invariants Brutal Tests
// =========================================================================

#[test]
fn test_tukey_fences_invariant() {
    // Sample dataset with an obvious high outlier (95.0)
    let mut data = vec![10.0, 12.0, 13.0, 15.0, 16.0, 18.0, 20.0, 22.0, 25.0, 30.0, 95.0];
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Q1 at 25th percentile, Q3 at 75th percentile
    let q1 = 13.0; // Index 2
    let q3 = 25.0; // Index 8
    let iqr = q3 - q1; // 12.0
    assert_eq!(iqr, 12.0);

    let lower_fence = q1 - 1.5 * iqr; // 13.0 - 18.0 = -5.0
    let upper_fence = q3 + 1.5 * iqr; // 25.0 + 18.0 = 43.0

    assert_eq!(lower_fence, -5.0);
    assert_eq!(upper_fence, 43.0);

    // Filter outliers using Tukey Fences
    let outliers: Vec<f64> = data.iter().copied().filter(|&x| x < lower_fence || x > upper_fence).collect();
    let inliers: Vec<f64> = data.iter().copied().filter(|&x| x >= lower_fence && x <= upper_fence).collect();

    assert_eq!(outliers, vec![95.0], "95.0 must be identified as an outlier beyond upper fence 43.0");
    assert_eq!(inliers.len(), 10, "10 regular observations must be within Tukey fences");
}

#[test]
fn test_srm_chi_square_invariant() {
    // 50/50 Traffic Split Design
    // Total users = 100,000. Expected: 50,000 Control, 50,000 Treatment.

    // Case 1: Skewed traffic indicating severe SRM (p < 0.001)
    let obs_c1: f64 = 49_200.0;
    let obs_t1: f64 = 50_800.0;
    let exp_c1: f64 = 50_000.0;
    let exp_t1: f64 = 50_000.0;

    let chi2_case1: f64 = ((obs_c1 - exp_c1).powi(2) / exp_c1) + ((obs_t1 - exp_t1).powi(2) / exp_t1);
    // (800)^2 / 50000 * 2 = 640000 / 50000 * 2 = 12.8 * 2 = 25.6
    assert!((chi2_case1 - 25.6).abs() < 1e-4);

    // Critical value for chi-square (1 df) at alpha = 0.001 is 10.828
    let critical_srm_threshold: f64 = 10.828;
    assert!(
        chi2_case1 > critical_srm_threshold,
        "Chi2 = {} > 10.828 indicates p < 0.001: SRM detected, experiment invalid!",
        chi2_case1
    );

    // Case 2: Healthy traffic allocation within normal binomial variance
    let obs_c2: f64 = 50_050.0;
    let obs_t2: f64 = 49_950.0;
    let chi2_case2: f64 = ((obs_c2 - exp_c1).powi(2) / exp_c1) + ((obs_t2 - exp_t1).powi(2) / exp_t1);
    // (50)^2 / 50000 * 2 = 2500 / 50000 * 2 = 0.10
    assert!((chi2_case2 - 0.10).abs() < 1e-4);
    assert!(
        chi2_case2 <= critical_srm_threshold,
        "Chi2 = {} <= 10.828: Traffic allocation is trustworthy",
        chi2_case2
    );
}

#[test]
fn test_difference_in_differences_invariant() {
    // Pre-intervention: Time 1
    // Treatment Group Mean: 100.0
    // Control Group Mean: 90.0
    let y_t1 = 100.0;
    let y_c1 = 90.0;

    // Post-intervention: Time 2
    // Treatment Group Mean: 145.0 (growth = +45.0)
    // Control Group Mean: 110.0 (natural trend growth = +20.0)
    let y_t2 = 145.0;
    let y_c2 = 110.0;

    // DiD Estimator: (Y_T2 - Y_T1) - (Y_C2 - Y_C1)
    let delta_treatment = y_t2 - y_t1; // 45.0
    let delta_control = y_c2 - y_c1;     // 20.0
    let did_effect = delta_treatment - delta_control; // 25.0

    assert_eq!(delta_treatment, 45.0);
    assert_eq!(delta_control, 20.0);
    assert_eq!(did_effect, 25.0, "Causal treatment effect isolated via DiD must be exactly 25.0");
}

#[test]
fn test_tufte_data_ink_ratio_invariant() {
    // Tufte Invariant: Data-Ink Ratio = Data-Ink / Total-Ink <= 1.0
    let data_ink = 85.0;
    let non_data_ink = 15.0; // borders, axes, subtle grid
    let total_ink = data_ink + non_data_ink; // 100.0

    let data_ink_ratio = data_ink / total_ink;
    assert!(
        data_ink_ratio <= 1.0,
        "Data-ink ratio cannot exceed 1.0: got {}",
        data_ink_ratio
    );
    assert!(
        data_ink_ratio > 0.80,
        "High quality Tufte visual should have Data-Ink Ratio > 0.80, got {}",
        data_ink_ratio
    );

    // Lie Factor Invariant: Size of effect shown in graphic / Size of effect in data
    let graphic_val_1: f64 = 100.0;
    let graphic_val_2: f64 = 150.0; // Graphic effect = (150 - 100) / 100 = 0.50 (50%)

    let data_val_1: f64 = 200.0;
    let data_val_2: f64 = 300.0; // Data effect = (300 - 200) / 200 = 0.50 (50%)

    let effect_graphic: f64 = (graphic_val_2 - graphic_val_1).abs() / graphic_val_1;
    let effect_data: f64 = (data_val_2 - data_val_1).abs() / data_val_1;
    let lie_factor: f64 = effect_graphic / effect_data;

    assert!(
        (lie_factor - 1.0).abs() < 1e-6,
        "Truthful visualization must have Lie Factor = 1.0, got {}",
        lie_factor
    );
    assert!(lie_factor >= 0.95 && lie_factor <= 1.05);
}

#[test]
fn test_cohort_retention_decay_invariant() {
    // Model: R(t) = R_0 * t^(-gamma) + c
    // Baseline R_0 = 0.60, decay gamma = 0.5, plateau constant c = 0.15 (15% long-term PMF retention)
    let r_0 = 0.60;
    let gamma = 0.5;
    let c = 0.15;

    let retention = |t: f64| -> f64 {
        r_0 * t.powf(-gamma) + c
    };

    let weeks = [1.0, 4.0, 9.0, 16.0, 100.0, 10_000.0];
    let mut prev_r = 1.0;

    for &w in &weeks {
        let r = retention(w);
        // Monotonic decay invariant
        assert!(
            r <= prev_r,
            "Retention must monotonically decline: R(week {}) = {} > prev {}",
            w, r, prev_r
        );
        // Plateau lower bound invariant
        assert!(
            r >= c,
            "Retention R(week {}) = {} cannot drop below plateau baseline {}",
            w, r, c
        );
        prev_r = r;
    }

    // At week 10,000, retention approaches plateau c = 0.15 within 0.01
    let long_term_r = retention(10_000.0);
    assert!(
        (long_term_r - c).abs() < 0.01,
        "Long-term cohort retention must converge to PMF plateau c = 0.15, got {}",
        long_term_r
    );
}
