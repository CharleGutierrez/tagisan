use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use tracing::warn;

/// A reusable engineering skill defined according to the ECC specification
#[derive(Debug, Clone, PartialEq)]
pub struct EccSkill {
    pub name: String,
    pub description: String,
    pub instructions: String,
}

impl EccSkill {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            instructions: instructions.into(),
        }
    }

    /// Parse a SKILL.md file with YAML frontmatter
    pub fn parse(content: &str) -> Result<Self> {
        Self::parse_with_default_name(content, None)
    }

    /// Parse a SKILL.md file with YAML frontmatter and an optional fallback name
    pub fn parse_with_default_name(content: &str, default_name: Option<&str>) -> Result<Self> {
        let trimmed = content.trim_start_matches('\u{feff}').trim_start();
        if !trimmed.starts_with("---") {
            if let Some(def) = default_name {
                let first_line = trimmed.lines().find(|l| !l.trim().is_empty()).unwrap_or(def);
                let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                return Ok(Self {
                    name: def.to_string(),
                    description: if desc.is_empty() { def.to_string() } else { desc },
                    instructions: trimmed.to_string(),
                });
            }
            return Err(TagisanError::Execution(
                "Invalid ECC skill format: missing leading '---' frontmatter delimiter".to_string(),
            ));
        }

        let rest = &trimmed[3..];
        let end_idx = match rest.find("\n---").or_else(|| rest.find("\r\n---")) {
            Some(idx) => idx,
            None => {
                if let Some(def) = default_name {
                    let first_line = rest.lines().find(|l| !l.trim().is_empty()).unwrap_or(def);
                    let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                    return Ok(Self {
                        name: def.to_string(),
                        description: if desc.is_empty() { def.to_string() } else { desc },
                        instructions: rest.to_string(),
                    });
                }
                return Err(TagisanError::Execution(
                    "Invalid ECC skill format: missing closing '---' frontmatter delimiter".to_string(),
                ));
            }
        };

        let frontmatter_str = &rest[..end_idx];
        let after_close = &rest[end_idx..];
        let delim_pos = after_close.find("---").unwrap_or(0);
        let mut body_start_offset = delim_pos + 3;
        if let Some(nl_pos) = after_close[body_start_offset..].find('\n') {
            body_start_offset += nl_pos + 1;
        } else {
            body_start_offset = after_close.len();
        }
        let body = after_close.get(body_start_offset..).unwrap_or("").trim().to_string();

        let mut name = String::new();
        let mut description = String::new();

        let lines: Vec<&str> = frontmatter_str.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let raw_line = lines[i];
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim().trim_matches('"').trim_matches('\'').trim();

                match key.as_str() {
                    "name" => {
                        if name.is_empty() {
                            name = val.to_string();
                        }
                    }
                    "description" => {
                        if description.is_empty() {
                            if val == "|" || val == "|-" || val == ">" || val == ">-" || val.is_empty() {
                                let mut desc_lines = Vec::new();
                                i += 1;
                                while i < lines.len() {
                                    let next_raw = lines[i];
                                    if next_raw.starts_with(' ') || next_raw.starts_with('\t') {
                                        desc_lines.push(next_raw.trim());
                                        i += 1;
                                    } else if next_raw.trim().is_empty() {
                                        i += 1;
                                    } else {
                                        break;
                                    }
                                }
                                description = desc_lines.join(" ");
                                continue;
                            } else {
                                description = val.to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
        }

        if name.trim().is_empty() {
            if let Some(def) = default_name {
                name = def.to_string();
            } else {
                return Err(TagisanError::Execution(
                    "Invalid ECC skill format: missing 'name' field in frontmatter".to_string(),
                ));
            }
        }

        if description.trim().is_empty() {
            let first_line = body.lines().find(|l| !l.trim().is_empty()).unwrap_or(&name);
            description = first_line.trim().trim_start_matches('#').trim().to_string();
            if description.is_empty() {
                description = name.clone();
            }
        }

        let instructions = if body.trim().is_empty() {
            description.clone()
        } else {
            body
        };

        Ok(Self {
            name,
            description,
            instructions,
        })
    }

    /// Load an ECC skill from a file on disk
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read ECC skill file '{}': {}",
                path_ref.display(),
                e
            ))
        })?;
        let default_name = path_ref
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str());
        Self::parse_with_default_name(&content, default_name)
    }
}

/// Return all built-in ECC engineering skills
pub fn all_built_in_skills() -> Vec<EccSkill> {
    vec![
        tdd_workflow(),
        security_review(),
        api_design(),
        verification_loop(),
        tokio_async_tuning(),
        rust_idiomatic_hygiene(),
        rust_proptest_fuzzing(),
        criterion_benchmarking(),
        cargo_deny_security(),
        llm_as_a_judge_rubrics(),
        hallucination_detector(),
        swe_bench_decomposition(),
        prompt_injection_defense(),
        secret_entropy_scanner(),
        posix_shell_sanitizer(),
        bun_native_apis(),
        seccomp_sandbox_rules(),
        hexagonal_architecture(),
        backend_patterns(),
        coding_standards(),
        structured_analysis_yourdon(),
        modular_coupling_cohesion(),
        data_dictionary_minispecs(),
        domain_driven_design(),
        design_by_contract(),
        statechart_fsm_modeling(),
        data_intensive_architecture(),
        deep_modules_complexity(),
        legacy_seams_characterization(),
        catalog_refactoring_smells(),
        production_resilience_release_it(),
        evolutionary_fitness_functions(),
        temporal_invariants_tla(),
        conceptual_integrity_systems(),
        grokking_algorithms(),
        ostep_mechanical_sympathy(),
        system_design_building_blocks(),
        pragmatic_programmer_craft(),
        site_reliability_engineering(),
        // Advanced UX & UI Vibe Engineering Skills
        refactoring_ui(),
        microinteractions_design(),
        laws_of_ux(),
        design_systems_tokens(),
        about_face_interaction_design(),
        designing_for_emotion(),
        // Quantitative & Financial AI Foundation Model Skills
        kronos_kline_modeling(),
        qlib_alpha_engineering(),
        chronos_moirai_forecasting(),
        fingpt_multimodal_nlp(),
        rdagent_factor_mining(),
        // Mathematics for Vibe Code Developers (Top 20 Foundational Books)
        math_nature_of_code(),
        math_linear_algebra_savov(),
        math_high_dimensional_data(),
        math_information_inference(),
        math_causal_inference(),
        math_category_theory(),
        math_game_engine_geometry(),
        math_code_first_calculus(),
        math_bayesian_reasoning(),
        math_ml_foundations(),
        math_networks_crowds_markets(),
        math_convex_optimization(),
        math_probability_logic(),
        math_programmers_discrete(),
        math_intuitive_calculus(),
        math_concrete_discrete(),
        math_visual_topology(),
        math_chaos_fractals(),
        math_numerical_methods(),
        math_algorithmic_game_theory(),
        // Enterprise Resource Planning & Enterprise Systems Architect (Top 50 Books)
        erp_silverston_enterprise_patterns(),
        erp_silverston_industry_patterns(),
        erp_fowler_analysis_patterns(),
        erp_evans_ddd_core(),
        erp_vernon_iddd_enterprise(),
        erp_scheer_aris_architecture(),
        erp_double_entry_ledger(),
        erp_multi_currency_fx(),
        erp_asc606_revenue_recognition(),
        erp_cost_accounting_management(),
        erp_double_entry_history_auditing(),
        erp_balance_sheet_working_capital(),
        erp_intercompany_consolidation(),
        erp_supply_chain_strategy(),
        erp_inventory_eoq_safety_stock(),
        erp_landed_cost_allocation(),
        erp_wms_bin_location_topology(),
        erp_procurement_vendor_lifecycle(),
        erp_3way_match_p2p(),
        erp_reverse_logistics_rma(),
        erp_mrp_crp_core(),
        erp_bom_explosion_routing(),
        erp_theory_of_constraints_dbr(),
        erp_ddmrp_demand_driven(),
        erp_lean_toyota_production(),
        erp_shop_floor_mes(),
        erp_batch_traceability_genealogy(),
        erp_bpmn_workflow_patterns(),
        erp_process_mining_event_logs(),
        erp_order_to_cash_o2c(),
        erp_procure_to_pay_p2p(),
        erp_record_to_report_r2r(),
        erp_subscription_recurring_billing(),
        erp_enterprise_integration_patterns(),
        erp_odoo_technical_architecture(),
        erp_frappe_erpnext_framework(),
        erp_sap_s4hana_cleancore(),
        erp_netsuite_suitecloud_suiteflow(),
        erp_multi_tenant_data_isolation(),
        erp_custom_fields_metadata_extensibility(),
        erp_headless_graphql_rest_api(),
        erp_distributed_acid_kleppmann(),
        erp_saga_distributed_transactions(),
        erp_cqrs_event_sourcing(),
        erp_optimistic_locking_concurrency(),
        erp_distributed_idempotency(),
        erp_segregation_of_duties_sod(),
        erp_sox_internal_controls_audit(),
        erp_data_retention_gdpr_compliance(),
        erp_tax_engine_jurisdiction_rules(),
        // Data Analytics & Analytics Engineering Architect Skills (Top 50 Books)
        analytics_kimball_dimensional_modeling(),
        analytics_etl_pipeline_patterns(),
        analytics_data_vault_architecture(),
        analytics_dbt_modeling_dag(),
        analytics_advanced_sql_windowing(),
        analytics_celko_relational_logic(),
        analytics_tukey_eda_heuristics(),
        analytics_practical_statistics(),
        analytics_intuitive_statistics(),
        analytics_statistical_learning_islp(),
        analytics_bayesian_rethinking(),
        analytics_mathematical_inference(),
        analytics_kohavi_ab_experimentation(),
        analytics_causal_mixtape(),
        analytics_econometric_causality(),
        analytics_counterfactual_causality(),
        analytics_kleppmann_data_intensive(),
        analytics_akidau_stream_processing(),
        analytics_kafka_event_streaming(),
        analytics_data_engineering_lifecycle(),
        analytics_realtime_olap_pipelines(),
        analytics_python_pandas_wrangling(),
        analytics_duckdb_embedded_olap(),
        analytics_polars_lazy_processing(),
        analytics_high_performance_compute(),
        analytics_cli_data_science(),
        analytics_tufte_visual_display(),
        analytics_storytelling_with_data(),
        analytics_wilke_data_visualization(),
        analytics_few_dashboard_design(),
        analytics_cairo_visual_integrity(),
        analytics_d3_interactive_graphics(),
        analytics_parmenter_kpi_framework(),
        analytics_lean_startup_metrics(),
        analytics_hubbard_measurement_value(),
        analytics_semantic_metadata_layer(),
        analytics_semantic_metrics_governance(),
        analytics_okr_goal_tracking(),
        analytics_hyndman_time_series(),
        analytics_box_jenkins_arima(),
        analytics_anomaly_outlier_detection(),
        analytics_python_signal_processing(),
        analytics_feature_engineering_pipeline(),
        analytics_geron_ml_pipelines(),
        analytics_kuhn_predictive_modeling(),
        analytics_elements_statistical_learning(),
        analytics_redman_data_quality(),
        analytics_data_observability_monitors(),
        analytics_dama_data_governance(),
        analytics_dataops_automated_testing(),
        // Business & Functional Analysis Architect Skills (Top 50 Canonical Books)
        ba_wiegers_requirements_engineering(),
        ba_volere_requirements_specification(),
        ba_cockburn_use_case_modeling(),
        ba_nfr_quality_attributes(),
        ba_requirements_traceability_matrix(),
        ba_adzic_specification_by_example(),
        ba_cucumber_gherkin_syntax(),
        ba_bdd_three_amigos_workshop(),
        ba_atdd_acceptance_criteria(),
        ba_living_documentation_tooling(),
        ba_evans_ubiquitous_language(),
        ba_brandolini_event_storming(),
        ba_bounded_context_mapping(),
        ba_domain_storytelling(),
        ba_subdomain_core_domain_triage(),
        ba_patton_user_story_mapping(),
        ba_invest_user_stories(),
        ba_story_splitting_patterns(),
        ba_wsjf_backlog_prioritization(),
        ba_kanban_value_stream_metrics(),
        ba_bpmn_level2_process_modeling(),
        ba_bpmn_gateway_soundness(),
        ba_bpmn_timer_boundary_events(),
        ba_state_machine_lifecycle_modeling(),
        ba_process_waste_elimination_vsm(),
        ba_torres_opportunity_solution_tree(),
        ba_cagan_four_product_risks(),
        ba_customer_journey_mapping(),
        ba_jobs_to_be_done_jtbd(),
        ba_assumption_mapping_experimentation(),
        ba_hoberman_data_modeling_resource(),
        ba_relational_normalization_3nf(),
        ba_cardinality_erd_relationship_rules(),
        ba_temporal_bitemporal_data_patterns(),
        ba_data_dictionary_master_metadata(),
        ba_ross_business_rule_manifesto(),
        ba_dmn_decision_table_completeness(),
        ba_drd_decision_requirements_diagrams(),
        ba_feel_expression_language(),
        ba_decision_table_verification_solver(),
        ba_meadows_systems_thinking_leverage(),
        ba_causal_loop_diagrams_archetypes(),
        ba_galls_law_system_evolution(),
        ba_wardley_mapping_strategic_landscape(),
        ba_cynefin_framework_decision_making(),
        ba_cooper_goal_directed_design(),
        ba_norman_affordance_signifiers(),
        ba_johnson_gui_bloopers_heuristics(),
        ba_crud_form_functional_specifications(),
        ba_information_architecture_wireflow(),
    ]
}

/// Retrieve a built-in skill by name
pub fn find_built_in_skill(name: &str) -> Option<EccSkill> {
    let lower = name.to_lowercase().replace('_', "-");
    // Math Vibe Skills Aliases
    if lower == "nature-of-code" || lower == "boids" || lower == "flocking" || lower == "autonomous-steering" {
        return find_built_in_skill("math-nature-of-code");
    }
    if lower == "linear-algebra" || lower == "savov" || lower == "axler" || lower == "matrix-projections" {
        return find_built_in_skill("math-linear-algebra-savov");
    }
    if lower == "high-dimensional-data" || lower == "hopcroft" || lower == "curse-of-dimensionality" {
        return find_built_in_skill("math-high-dimensional-data");
    }
    if lower == "information-theory" || lower == "shannon-entropy" || lower == "mackay" || lower == "token-entropy" {
        return find_built_in_skill("math-information-inference");
    }
    if lower == "causal-inference" || lower == "book-of-why" || lower == "do-calculus" || lower == "pearl-causality" {
        return find_built_in_skill("math-causal-inference");
    }
    if lower == "category-theory" || lower == "milewski" || lower == "monads" || lower == "functors" {
        return find_built_in_skill("math-category-theory");
    }
    if lower == "game-engine-geometry" || lower == "quaternions" || lower == "slerp" || lower == "lengyel" {
        return find_built_in_skill("math-game-engine-geometry");
    }
    if lower == "code-first-calculus" || lower == "paul-orland" || lower == "gradient-descent-math" {
        return find_built_in_skill("math-code-first-calculus");
    }
    if lower == "bayesian-reasoning" || lower == "will-kurt" || lower == "thompson-sampling" || lower == "bayes" {
        return find_built_in_skill("math-bayesian-reasoning");
    }
    if lower == "ml-foundations" || lower == "deisenroth" || lower == "matrix-calculus" {
        return find_built_in_skill("math-ml-foundations");
    }
    if lower == "networks-crowds-markets" || lower == "pagerank" || lower == "kleinberg" || lower == "easley" {
        return find_built_in_skill("math-networks-crowds-markets");
    }
    if lower == "convex-optimization" || lower == "boyd" || lower == "kkt" || lower == "kkt-conditions" {
        return find_built_in_skill("math-convex-optimization");
    }
    if lower == "probability-logic" || lower == "jaynes" || lower == "maximum-entropy" || lower == "maxent" {
        return find_built_in_skill("math-probability-logic");
    }
    if lower == "programmers-discrete" || lower == "jeremy-kun" || lower == "fft" || lower == "graph-laplacian" {
        return find_built_in_skill("math-programmers-discrete");
    }
    if lower == "intuitive-calculus" || lower == "calculus-made-easy" || lower == "pid-controller" {
        return find_built_in_skill("math-intuitive-calculus");
    }
    if lower == "concrete-discrete" || lower == "concrete-math" || lower == "knuth-math" || lower == "recurrences" {
        return find_built_in_skill("math-concrete-discrete");
    }
    if lower == "visual-topology" || lower == "hilbert-geometry" || lower == "gaussian-curvature" {
        return find_built_in_skill("math-visual-topology");
    }
    if lower == "chaos-fractals" || lower == "lorenz-attractor" || lower == "fractals" || lower == "mandelbrot" {
        return find_built_in_skill("math-chaos-fractals");
    }
    if lower == "numerical-methods" || lower == "numerical-recipes" || lower == "rk4" || lower == "runge-kutta" {
        return find_built_in_skill("math-numerical-methods");
    }
    if lower == "algorithmic-game-theory" || lower == "nash-equilibrium" || lower == "shapley-value" || lower == "vcg-auction" {
        return find_built_in_skill("math-algorithmic-game-theory");
    }
    if lower == "kronos" || lower == "kronos-bsq" || lower == "kline-modeling" || lower == "k-line-modeling" {
        return find_built_in_skill("kronos-kline-modeling");
    }
    if lower == "qlib" || lower == "qlib-alpha" || lower == "qlib-factor-neutralization" {
        return find_built_in_skill("qlib-alpha-engineering");
    }
    if lower == "chronos" || lower == "moirai" || lower == "chronos-moirai" || lower == "moirai-forecasting" {
        return find_built_in_skill("chronos-moirai-forecasting");
    }
    if lower == "fingpt" || lower == "finrl" || lower == "fingpt-finrl" || lower == "financial-nlp" {
        return find_built_in_skill("fingpt-multimodal-nlp");
    }
    if lower == "rdagent" || lower == "rd-agent" || lower == "rdagent-mining" || lower == "rdagent-alpha" {
        return find_built_in_skill("rdagent-factor-mining");
    }
    if lower == "rust-tokio-concurrency" || lower == "rust-concurrency" || lower == "tokio-concurrency" {
        if let Some(mut s) = find_built_in_skill("tokio-async-tuning") {
            s.name = "rust-tokio-concurrency".to_string();
            return Some(s);
        }
    }
    // ERP & Enterprise Systems Skills Aliases
    if lower == "double-entry" || lower == "general-ledger" || lower == "debit-credit" || lower == "trial-balance" {
        return find_built_in_skill("erp-double-entry-ledger");
    }
    if lower == "3-way-match" || lower == "three-way-match" || lower == "po-grn-match" || lower == "gr-ir" {
        return find_built_in_skill("erp-3way-match-p2p");
    }
    if lower == "eoq" || lower == "economic-order-quantity" {
        return find_built_in_skill("erp-inventory-eoq-safety-stock");
    }
    if lower == "safety-stock" || lower == "reorder-point" || lower == "rop" {
        return find_built_in_skill("erp-inventory-eoq-safety-stock");
    }
    if lower == "landed-cost" || lower == "landed-cost-allocation" || lower == "freight-absorption" {
        return find_built_in_skill("erp-landed-cost-allocation");
    }
    if lower == "bom-explosion" || lower == "bill-of-materials" || lower == "low-level-coding" {
        return find_built_in_skill("erp-bom-explosion-routing");
    }
    if lower == "order-to-cash" || lower == "o2c" {
        return find_built_in_skill("erp-order-to-cash-o2c");
    }
    if lower == "procure-to-pay" || lower == "p2p" {
        return find_built_in_skill("erp-procure-to-pay-p2p");
    }
    if lower == "record-to-report" || lower == "r2r" || lower == "fast-close" {
        return find_built_in_skill("erp-record-to-report-r2r");
    }
    if lower == "odoo" || lower == "odoo-orm" {
        return find_built_in_skill("erp-odoo-technical-architecture");
    }
    if lower == "frappe" || lower == "erpnext" {
        return find_built_in_skill("erp-frappe-erpnext-framework");
    }
    if lower == "saga-pattern" || lower == "saga-orchestration" || lower == "distributed-saga" {
        return find_built_in_skill("erp-saga-distributed-transactions");
    }
    if lower == "segregation-of-duties" || lower == "sod" {
        return find_built_in_skill("erp-segregation-of-duties-sod");
    }
    if lower == "sox-compliance" || lower == "sox-audit" || lower == "sox-404" || lower == "itgc" {
        return find_built_in_skill("erp-sox-internal-controls-audit");
    }
    if lower == "silverston" || lower == "party-model" {
        return find_built_in_skill("erp-silverston-enterprise-patterns");
    }
    if lower == "fowler-analysis" || lower == "analysis-patterns" {
        return find_built_in_skill("erp-fowler-analysis-patterns");
    }
    if lower == "evans-ddd" || lower == "domain-driven-design" {
        return find_built_in_skill("erp-evans-ddd-core");
    }
    if lower == "vernon-iddd" || lower == "iddd" {
        return find_built_in_skill("erp-vernon-iddd-enterprise");
    }
    if lower == "aris" || lower == "scheer-aris" || lower == "epc" {
        return find_built_in_skill("erp-scheer-aris-architecture");
    }
    if lower == "multi-currency" || lower == "fx-revaluation" {
        return find_built_in_skill("erp-multi-currency-fx");
    }
    if lower == "asc606" || lower == "revenue-recognition" || lower == "ifrs15" {
        return find_built_in_skill("erp-asc606-revenue-recognition");
    }
    if lower == "cost-accounting" || lower == "standard-costing" || lower == "variance-analysis" {
        return find_built_in_skill("erp-cost-accounting-management");
    }
    if lower == "working-capital" || lower == "cash-conversion-cycle" || lower == "ccc" {
        return find_built_in_skill("erp-balance-sheet-working-capital");
    }
    if lower == "intercompany" || lower == "consolidation" || lower == "financial-consolidation" {
        return find_built_in_skill("erp-intercompany-consolidation");
    }
    if lower == "supply-chain" || lower == "bullwhip-effect" {
        return find_built_in_skill("erp-supply-chain-strategy");
    }
    if lower == "wms" || lower == "warehouse-topology" || lower == "bin-location" {
        return find_built_in_skill("erp-wms-bin-location-topology");
    }
    if lower == "procurement" || lower == "strategic-sourcing" || lower == "otif" {
        return find_built_in_skill("erp-procurement-vendor-lifecycle");
    }
    if lower == "reverse-logistics" || lower == "rma" {
        return find_built_in_skill("erp-reverse-logistics-rma");
    }
    if lower == "mrp" || lower == "crp" || lower == "master-production-schedule" {
        return find_built_in_skill("erp-mrp-crp-core");
    }
    if lower == "theory-of-constraints" || lower == "toc" || lower == "drum-buffer-rope" || lower == "dbr" {
        return find_built_in_skill("erp-theory-of-constraints-dbr");
    }
    if lower == "ddmrp" || lower == "demand-driven-mrp" {
        return find_built_in_skill("erp-ddmrp-demand-driven");
    }
    if lower == "toyota-production-system" || lower == "tps" || lower == "lean-manufacturing" || lower == "kanban" {
        return find_built_in_skill("erp-lean-toyota-production");
    }
    if lower == "mes" || lower == "shop-floor" || lower == "oee" {
        return find_built_in_skill("erp-shop-floor-mes");
    }
    if lower == "traceability" || lower == "lot-genealogy" || lower == "fefo" {
        return find_built_in_skill("erp-batch-traceability-genealogy");
    }
    if lower == "bpmn" || lower == "workflow-patterns" {
        return find_built_in_skill("erp-bpmn-workflow-patterns");
    }
    if lower == "process-mining" || lower == "alpha-miner" || lower == "xes" {
        return find_built_in_skill("erp-process-mining-event-logs");
    }
    if lower == "subscription-billing" || lower == "recurring-billing" || lower == "mrr" {
        return find_built_in_skill("erp-subscription-recurring-billing");
    }
    if lower == "enterprise-integration-patterns" || lower == "eip" || lower == "transactional-outbox" {
        return find_built_in_skill("erp-enterprise-integration-patterns");
    }
    if lower == "sap-cleancore" || lower == "clean-core" || lower == "s4hana" {
        return find_built_in_skill("erp-sap-s4hana-cleancore");
    }
    if lower == "netsuite" || lower == "suitecloud" || lower == "suitescript" {
        return find_built_in_skill("erp-netsuite-suitecloud-suiteflow");
    }
    if lower == "multi-tenant" || lower == "rls" {
        return find_built_in_skill("erp-multi-tenant-data-isolation");
    }
    if lower == "custom-fields" || lower == "eav" {
        return find_built_in_skill("erp-custom-fields-metadata-extensibility");
    }
    if lower == "headless-erp" || lower == "dataloader" {
        return find_built_in_skill("erp-headless-graphql-rest-api");
    }
    if lower == "distributed-acid" || lower == "kleppmann" || lower == "write-skew" {
        return find_built_in_skill("erp-distributed-acid-kleppmann");
    }
    if lower == "cqrs" || lower == "event-sourcing" {
        return find_built_in_skill("erp-cqrs-event-sourcing");
    }
    if lower == "optimistic-locking" || lower == "optimistic-concurrency" {
        return find_built_in_skill("erp-optimistic-locking-concurrency");
    }
    if lower == "idempotency" || lower == "idempotency-key" {
        return find_built_in_skill("erp-distributed-idempotency");
    }
    if lower == "gdpr" || lower == "data-retention" {
        return find_built_in_skill("erp-data-retention-gdpr-compliance");
    }
    if lower == "tax-engine" || lower == "sales-tax" || lower == "vat" || lower == "tax-nexus" {
        return find_built_in_skill("erp-tax-engine-jurisdiction-rules");
    }
    // Data Analytics & Analytics Engineering Skills Aliases
    if lower == "kimball" || lower == "star-schema" || lower == "dimensional-modeling" || lower == "snowflake-schema" {
        return find_built_in_skill("analytics-kimball-dimensional-modeling");
    }
    if lower == "etl-patterns" || lower == "kimball-etl" || lower == "data-warehouse-etl" {
        return find_built_in_skill("analytics-etl-pipeline-patterns");
    }
    if lower == "data-vault" || lower == "raw-vault" || lower == "business-vault" {
        return find_built_in_skill("analytics-data-vault-architecture");
    }
    if lower == "dbt" || lower == "dbt-dag" || lower == "analytics-engineering" {
        return find_built_in_skill("analytics-dbt-modeling-dag");
    }
    if lower == "window-functions" || lower == "advanced-sql" || lower == "sql-windowing" {
        return find_built_in_skill("analytics-advanced-sql-windowing");
    }
    if lower == "celko" || lower == "nested-sets" || lower == "relational-division" {
        return find_built_in_skill("analytics-celko-relational-logic");
    }
    if lower == "tukey-eda" || lower == "tukey" || lower == "eda" || lower == "tukey-fences" {
        return find_built_in_skill("analytics-tukey-eda-heuristics");
    }
    if lower == "practical-statistics" || lower == "bootstrap-resampling" || lower == "permutation-test" {
        return find_built_in_skill("analytics-practical-statistics");
    }
    if lower == "intuitive-statistics" || lower == "naked-statistics" || lower == "simpsons-paradox" {
        return find_built_in_skill("analytics-intuitive-statistics");
    }
    if lower == "statistical-learning" || lower == "islp" || lower == "bias-variance" {
        return find_built_in_skill("analytics-statistical-learning-islp");
    }
    if lower == "bayesian-rethinking" || lower == "mcelreath" || lower == "collider-bias" {
        return find_built_in_skill("analytics-bayesian-rethinking");
    }
    if lower == "mathematical-inference" || lower == "all-of-statistics" || lower == "wasserman" {
        return find_built_in_skill("analytics-mathematical-inference");
    }
    if lower == "ab-testing" || lower == "srm" || lower == "sample-ratio-mismatch" || lower == "kohavi" {
        return find_built_in_skill("analytics-kohavi-ab-experimentation");
    }
    if lower == "causal-mixtape" || lower == "rubin-causal-model" || lower == "potential-outcomes" || lower == "analytics-causal-inference" {
        return find_built_in_skill("analytics-causal-mixtape");
    }
    if lower == "difference-in-differences" || lower == "did" || lower == "econometric-causality" || lower == "mostly-harmless-econometrics" {
        return find_built_in_skill("analytics-econometric-causality");
    }
    if lower == "counterfactual" || lower == "counterfactual-causality" || lower == "ipw" {
        return find_built_in_skill("analytics-counterfactual-causality");
    }
    if lower == "duckdb" || lower == "embedded-olap" {
        return find_built_in_skill("analytics-duckdb-embedded-olap");
    }
    if lower == "polars" || lower == "lazyframe" {
        return find_built_in_skill("analytics-polars-lazy-processing");
    }
    if lower == "tufte" || lower == "data-ink" || lower == "chartjunk" {
        return find_built_in_skill("analytics-tufte-visual-display");
    }
    if lower == "storytelling-with-data" || lower == "knaflic" {
        return find_built_in_skill("analytics-storytelling-with-data");
    }
    if lower == "bullet-graph" || lower == "stephen-few" {
        return find_built_in_skill("analytics-few-dashboard-design");
    }
    if lower == "kpi" || lower == "parmenter" || lower == "kri" {
        return find_built_in_skill("analytics-parmenter-kpi-framework");
    }
    if lower == "cohort-retention" || lower == "lean-analytics" || lower == "omtm" || lower == "aarrr" {
        return find_built_in_skill("analytics-lean-startup-metrics");
    }
    if lower == "time-series-forecasting" || lower == "stl-decomposition" || lower == "hyndman" {
        return find_built_in_skill("analytics-hyndman-time-series");
    }
    if lower == "arima" || lower == "box-jenkins" || lower == "sarima" {
        return find_built_in_skill("analytics-box-jenkins-arima");
    }
    if lower == "isolation-forest" || lower == "outlier-detection" || lower == "anomaly-detection" {
        return find_built_in_skill("analytics-anomaly-outlier-detection");
    }
    if lower == "data-observability" || lower == "5-pillars-observability" || lower == "freshness-monitoring" {
        return find_built_in_skill("analytics-data-observability-monitors");
    }
    if lower == "data-governance" || lower == "dama" || lower == "dmbok" || lower == "master-data-management" {
        return find_built_in_skill("analytics-dama-data-governance");
    }
    if lower == "dataops" || lower == "dataops-testing" || lower == "data-pipeline-testing" {
        return find_built_in_skill("analytics-dataops-automated-testing");
    }

    // Business & Functional Analysis Skills Aliases
    if lower == "wiegers" || lower == "software-requirements" || lower == "requirements-engineering" {
        return find_built_in_skill("ba-wiegers-requirements-engineering");
    }
    if lower == "volere" || lower == "fit-criteria" || lower == "fit-criterion" {
        return find_built_in_skill("ba-volere-requirements-specification");
    }
    if lower == "use-case" || lower == "use-cases" || lower == "cockburn" {
        return find_built_in_skill("ba-cockburn-use-case-modeling");
    }
    if lower == "nfr" || lower == "quality-attributes" || lower == "iso-25010" {
        return find_built_in_skill("ba-nfr-quality-attributes");
    }
    if lower == "rtm" || lower == "traceability-matrix" {
        return find_built_in_skill("ba-requirements-traceability-matrix");
    }
    if lower == "specification-by-example" || lower == "adzic" || lower == "living-docs" {
        return find_built_in_skill("ba-adzic-specification-by-example");
    }
    if lower == "gherkin" || lower == "cucumber" || lower == "given-when-then" || lower == "bdd-syntax" {
        return find_built_in_skill("ba-cucumber-gherkin-syntax");
    }
    if lower == "three-amigos" || lower == "example-mapping" || lower == "dan-north" {
        return find_built_in_skill("ba-bdd-three-amigos-workshop");
    }
    if lower == "atdd" || lower == "acceptance-test-driven" || lower == "acceptance-criteria" {
        return find_built_in_skill("ba-atdd-acceptance-criteria");
    }
    if lower == "living-documentation" || lower == "martraire" {
        return find_built_in_skill("ba-living-documentation-tooling");
    }
    if lower == "ubiquitous-language" || lower == "domain-lexicon" || lower == "evans-ddd" {
        return find_built_in_skill("ba-evans-ubiquitous-language");
    }
    if lower == "event-storming" || lower == "brandolini" || lower == "eventstorming" {
        return find_built_in_skill("ba-brandolini-event-storming");
    }
    if lower == "context-mapping" || lower == "anti-corruption-layer" || lower == "acl" {
        return find_built_in_skill("ba-bounded-context-mapping");
    }
    if lower == "domain-storytelling" || lower == "hofer" {
        return find_built_in_skill("ba-domain-storytelling");
    }
    if lower == "core-domain" || lower == "subdomain-triage" || lower == "nick-tune" {
        return find_built_in_skill("ba-subdomain-core-domain-triage");
    }
    if lower == "story-mapping" || lower == "user-story-mapping" || lower == "jeff-patton" || lower == "walking-skeleton" {
        return find_built_in_skill("ba-patton-user-story-mapping");
    }
    if lower == "invest" || lower == "user-stories" || lower == "cohn-stories" {
        return find_built_in_skill("ba-invest-user-stories");
    }
    if lower == "story-splitting" || lower == "vertical-slicing" {
        return find_built_in_skill("ba-story-splitting-patterns");
    }
    if lower == "wsjf" || lower == "cost-of-delay" || lower == "reinertsen" {
        return find_built_in_skill("ba-wsjf-backlog-prioritization");
    }
    if lower == "littles-law" || lower == "wip-limits" || lower == "flow-metrics" || lower == "kanban-metrics" {
        return find_built_in_skill("ba-kanban-value-stream-metrics");
    }
    if lower == "bpmn-2" || lower == "process-modeling" || lower == "bruce-silver" || lower == "bpmn-method-and-style" {
        return find_built_in_skill("ba-bpmn-level2-process-modeling");
    }
    if lower == "gateway-soundness" || lower == "deadlock-freedom" || lower == "dumas-bpm" {
        return find_built_in_skill("ba-bpmn-gateway-soundness");
    }
    if lower == "boundary-events" || lower == "timer-boundary" {
        return find_built_in_skill("ba-bpmn-timer-boundary-events");
    }
    if lower == "state-machine" || lower == "fsm" || lower == "statecharts" || lower == "harel" {
        return find_built_in_skill("ba-state-machine-lifecycle-modeling");
    }
    if lower == "vsm" || lower == "value-stream-mapping" || lower == "7-wastes" || lower == "muda" {
        return find_built_in_skill("ba-process-waste-elimination-vsm");
    }
    if lower == "ost" || lower == "opportunity-solution-tree" || lower == "torres" {
        return find_built_in_skill("ba-torres-opportunity-solution-tree");
    }
    if lower == "four-product-risks" || lower == "cagan" || lower == "product-risks" {
        return find_built_in_skill("ba-cagan-four-product-risks");
    }
    if lower == "customer-journey" || lower == "journey-mapping" || lower == "kalbach" {
        return find_built_in_skill("ba-customer-journey-mapping");
    }
    if lower == "jtbd" || lower == "jobs-to-be-done" || lower == "job-story" {
        return find_built_in_skill("ba-jobs-to-be-done-jtbd");
    }
    if lower == "assumption-mapping" || lower == "bland-osterwalder" || lower == "rat" {
        return find_built_in_skill("ba-assumption-mapping-experimentation");
    }
    if lower == "conceptual-data-model" || lower == "hoberman" || lower == "logical-data-model" {
        return find_built_in_skill("ba-hoberman-data-modeling-resource");
    }
    if lower == "normalization" || lower == "3nf" || lower == "bcnf" || lower == "relational-normalization" {
        return find_built_in_skill("ba-relational-normalization-3nf");
    }
    if lower == "crows-foot" || lower == "cardinality" || lower == "barker" || lower == "peter-chen" {
        return find_built_in_skill("ba-cardinality-erd-relationship-rules");
    }
    if lower == "bitemporal" || lower == "snodgrass" || lower == "valid-time" || lower == "transaction-time" {
        return find_built_in_skill("ba-temporal-bitemporal-data-patterns");
    }
    if lower == "data-dictionary" || lower == "iso-11179" || lower == "dmbok" {
        return find_built_in_skill("ba-data-dictionary-master-metadata");
    }
    if lower == "rulespeak" || lower == "business-rule-manifesto" || lower == "ronald-ross" {
        return find_built_in_skill("ba-ross-business-rule-manifesto");
    }
    if lower == "dmn" || lower == "decision-table" || lower == "hit-policy" {
        return find_built_in_skill("ba-dmn-decision-table-completeness");
    }
    if lower == "drd" || lower == "decision-requirements-diagram" || lower == "bkm" {
        return find_built_in_skill("ba-drd-decision-requirements-diagrams");
    }
    if lower == "feel" || lower == "dmn-feel" || lower == "feel-expression" {
        return find_built_in_skill("ba-feel-expression-language");
    }
    if lower == "decision-solver" || lower == "rule-overlap" || lower == "shadowed-rules" {
        return find_built_in_skill("ba-decision-table-verification-solver");
    }
    if lower == "systems-thinking" || lower == "meadows" || lower == "stocks-and-flows" || lower == "leverage-points" {
        return find_built_in_skill("ba-meadows-systems-thinking-leverage");
    }
    if lower == "causal-loop" || lower == "cld" || lower == "senge" || lower == "systems-archetypes" {
        return find_built_in_skill("ba-causal-loop-diagrams-archetypes");
    }
    if lower == "galls-law" || lower == "systemantics" || lower == "john-gall" {
        return find_built_in_skill("ba-galls-law-system-evolution");
    }
    if lower == "wardley-maps" || lower == "wardley" || lower == "value-chain-mapping" {
        return find_built_in_skill("ba-wardley-mapping-strategic-landscape");
    }
    if lower == "cynefin" || lower == "snowden" || lower == "sense-making" {
        return find_built_in_skill("ba-cynefin-framework-decision-making");
    }
    if lower == "goal-directed-design" || lower == "alan-cooper" || lower == "mental-model" {
        return find_built_in_skill("ba-cooper-goal-directed-design");
    }
    if lower == "affordances" || lower == "signifiers" || lower == "don-norman" || lower == "gulf-of-execution" {
        return find_built_in_skill("ba-norman-affordance-signifiers");
    }
    if lower == "gui-bloopers" || lower == "jeff-johnson" || lower == "usability-heuristics" {
        return find_built_in_skill("ba-johnson-gui-bloopers-heuristics");
    }
    if lower == "form-design" || lower == "luke-wroblewski" || lower == "double-submit" {
        return find_built_in_skill("ba-crud-form-functional-specifications");
    }
    if lower == "wireflow" || lower == "jesse-james-garrett" || lower == "5-planes" {
        return find_built_in_skill("ba-information-architecture-wireflow");
    }
    all_built_in_skills().into_iter().find(|s| s.name == lower)
}

/// 1. TDD Workflow Skill
pub fn tdd_workflow() -> EccSkill {
    EccSkill::new(
        "tdd-workflow",
        "Test-Driven Development discipline: write failing assertions and reproduction cases before code. Triggers: test driven development, unit test, assertions, regression, tdd workflow, failing tests.",
        r#"# ECC TDD Workflow

## Phase 1: Test Formulation
- Always formulate concrete, failing unit tests before authoring implementation logic.
- Identify the public interface, inputs, expected outputs, and error variants.
- Write tests that capture boundary limits (0, 1, MAX, empty, null/None, unicode).

## Phase 2: Minimal Implementation
- Write the minimum amount of code required to make the tests pass.
- Resist premature optimization during greening phase.

## Phase 3: Refactor
- Eliminate duplicate code (DRY).
- Improve naming and readability without altering observable external behavior.
- Ensure all tests continue to pass.
"#,
    )
}

/// 2. Security Review Skill
pub fn security_review() -> EccSkill {
    EccSkill::new(
        "security-review",
        "Rigorous offensive and defensive threat modeling and vulnerability scanning. Triggers: security threat model, injection, memory safety, vulnerability audit, security review.",
        r#"# ECC Security Review

## Threat Checklist
1. Input Sanitization: Validate and sanitize all external inputs (CLI flags, file inputs, network bytes).
2. Injection Prevention: Avoid shell concatenation or string interpolation in command execution.
3. Concurrency Safety: Check for race conditions, deadlock cycles, and time-of-check to time-of-use (TOCTOU).
4. Secret Protection: Ensure API keys, tokens, and credentials are never logged or echoed to stdout.
5. Memory & Resource Safety: Validate array bounds, recursion depths, and allocation limits to prevent denial-of-service.
"#,
    )
}

/// 3. API Design Skill
pub fn api_design() -> EccSkill {
    EccSkill::new(
        "api-design",
        "Robust API design patterns emphasizing backward compatibility and intuitive ergonomic interfaces",
        r#"# ECC API Design

## Principles
1. Explicit over Implicit: Design function signatures where failure modes are represented in types (`Result`, `Option`).
2. Least Astonishment: Follow canonical idioms of the host programming language.
3. Extensibility: Use the builder pattern or options structs for functions with many parameters.
4. Documentation: Document every public struct, enum, and function with doc comments and usage examples.
"#,
    )
}

/// 4. Verification Loop Skill
pub fn verification_loop() -> EccSkill {
    EccSkill::new(
        "verification-loop",
        "Continuous verification, automated test runs, and regression monitoring",
        r#"# ECC Verification Loop

## Protocol
1. Baseline: Run the existing test suite to ensure an unpolluted baseline.
2. Reproduction: If fixing a bug, write a test that fails reliably without the fix.
3. Application: Apply the targeted, minimal fix.
4. Confirmation: Rerun tests to confirm the reproduction test passes AND zero regressions occur in existing tests.
"#,
    )
}

/// 5. Tokio Async Tuning Skill
pub fn tokio_async_tuning() -> EccSkill {
    EccSkill::new(
        "tokio-async-tuning",
        "Tokio async concurrency, lock contention avoidance, bounded channels, cancellation tokens, and JoinSet task scheduling. Triggers: tokio, concurrency, async, rust-tokio-concurrency, joinset, mpsc, mutex, await, blocking, cancellation.",
        r#"# Tokio Async & Concurrency Optimization

## Core Concurrency Protocols
1. Channel Bounding: Always use bounded channels (`tokio::sync::mpsc::channel(N)`); never unbounded in production hot paths.
2. Lock Contention Minimization: Never hold a `MutexGuard` across an `.await` boundary. Use message passing or atomic primitives (`AtomicBool`, `AtomicUsize`) for scalars.
3. Task Orchestration: Prefer `tokio::task::JoinSet` over loose `tokio::spawn` calls to ensure structured concurrency, cancellation propagation, and clean resource cleanup.
4. Blocking Operations: Offload heavy synchronous computations, CPU-bound parsing, or blocking filesystem I/O to `tokio::task::spawn_blocking`.
5. Clean Cancellation: Use tokio_util::sync::CancellationToken for cooperative shutdown and timeout propagation.
"#,
    )
}

/// 6. Rust Idiomatic Hygiene Skill
pub fn rust_idiomatic_hygiene() -> EccSkill {
    EccSkill::new(
        "rust-idiomatic-hygiene",
        "Idiomatic Rust patterns, RAII memory safety, thiserror error trees, and zero-copy Cow abstractions",
        r#"# Rust Idiomatic Hygiene & Memory Architecture

## Core Guidelines
1. Error Architecture: Represent domain failure modes with explicit enum variants derived via `thiserror::Error`. Never discard errors with `.unwrap()` in library code.
2. Zero-Copy Ergonomics: Use `std::borrow::Cow<'a, str>` or `&str` when data transformation is optional.
3. RAII & Clean Teardown: Leverage the `Drop` trait to guarantee deterministic cleanup of file handles, temporary directories, worktrees, and sockets.
4. Type-Driven Design: Make illegal states unrepresentable using Rust's algebraic data types (`enum`, `struct`, newtype pattern).
"#,
    )
}

/// 7. Rust Property-Based Testing & Fuzzing Skill
pub fn rust_proptest_fuzzing() -> EccSkill {
    EccSkill::new(
        "rust-proptest-fuzzing",
        "Property-based invariant testing, fuzzing with proptest/quickcheck, and minimal failure shrink analysis",
        r#"# Rust Property-Based Testing & Invariant Fuzzing

## Fuzzing Protocols
1. Invariant Identification: Define mathematical or structural invariants (e.g., `decode(encode(x)) == x`, `sort(x).is_sorted()`, `a + b == b + a`).
2. Generator Strategies: Author custom `proptest::strategy::Strategy` implementations that span entire domains including edge cases (empty collections, MAX/MIN bounds, NaN, UTF-8 surrogate code points).
3. Minimal Failing Vector Analysis: Utilize proptest's automated shrinking algorithm to isolate the exact minimal reproduction input on failure.
"#,
    )
}

/// 8. Criterion Benchmarking Skill
pub fn criterion_benchmarking() -> EccSkill {
    EccSkill::new(
        "criterion-benchmarking",
        "Statistical micro-benchmarking with Criterion, throughput measurement, and allocation profiling",
        r#"# Criterion Statistical Micro-Benchmarking

## Benchmarking Protocol
1. Statistical Isolation: Use `criterion::black_box` to prevent LLVM dead-code elimination and constant folding.
2. Throughput Metrics: Configure benchmarks with `Throughput::Bytes` or `Throughput::Elements` for realistic MB/s evaluations.
3. Warmup & Outlier Detection: Enforce minimum 3-second warmup and 5-second measurement periods across at least 100 samples.
4. Flamegraph Integration: Pair benchmarks with `pprof` or `cargo-flamegraph` to visualize CPU bottlenecks in hot loops.
"#,
    )
}

/// 9. Cargo Deny & Supply Chain Security Skill
pub fn cargo_deny_security() -> EccSkill {
    EccSkill::new(
        "cargo-deny-security",
        "Supply chain crate security, license compliance, duplicate dependencies, and cargo-audit scanning",
        r#"# Cargo Supply Chain Security & Dependency Hygiene

## Audit Guidelines
1. Vulnerability Scanning: Audit all transitive dependencies with `cargo audit` and RustSec Advisory Database.
2. License Enforcement: Reject non-permissive or ambiguous licenses (enforce MIT, Apache-2.0, BSD-3-Clause).
3. Duplicate Elimination: Detect and consolidate duplicate versions of common crates (`serde`, `tokio`, `syn`).
4. Ban Lists: Block unmaintained or unsound crates with known CVEs or memory safety violations.
"#,
    )
}

/// 10. LLM as a Judge Scoring Rubrics Skill
pub fn llm_as_a_judge_rubrics() -> EccSkill {
    EccSkill::new(
        "llm-as-a-judge-rubrics",
        "Structured evaluation scoring rubrics, Borda count ranking, and adversarial debate adjudication",
        r#"# LLM-as-a-Judge Evaluation & Adjudication Rubrics

## Evaluation Rubrics
1. Correctness (Weight 40%): Mathematical soundness, constraint satisfaction, compile-time validity, edge-case coverage.
2. Security & Guardrails (Weight 30%): Resistance to injection, privilege escalation, credential exposure, buffer exhaustion.
3. Maintainability (Weight 15%): Modularity, idiomatic naming, documentation density, clear separation of concerns.
4. Performance (Weight 15%): Algorithmic time/space complexity, memory footprint, cache-friendliness.
5. Consensus Synthesis: Aggregate individual agent assessments using Positional Borda Count or Supermajority voting.
"#,
    )
}

/// 11. Hallucination Detector Skill
pub fn hallucination_detector() -> EccSkill {
    EccSkill::new(
        "hallucination-detector",
        "Context grounding validation, fact checking against retrieved vector memory, and claim verification",
        r#"# Hallucination Detection & Context Grounding

## Verification Procedure
1. Claim Extraction: Decompose LLM output into atomic, verifiable technical assertions.
2. Grounding Cross-Check: Verify each claim against source code chunks and episodic memory retrieved via RAG.
3. Factuality Scoring: Flag and reject claims that cite non-existent functions, imaginary API endpoints, or phantom types.
4. Contradiction Analysis: Detect if proposed logic violates previously established architectural invariants.
"#,
    )
}

/// 12. SWE-Bench Issue Decomposition Skill
pub fn swe_bench_decomposition() -> EccSkill {
    EccSkill::new(
        "swe-bench-decomposition",
        "Decomposition of complex real-world software engineering issues into test-first DAG execution plans",
        r#"# SWE-Bench Problem Decomposition & Resolution

## Structured Workflow
1. Issue Ingestion: Parse bug reports, stack traces, and reproduction scripts.
2. Localization: Use vector search and codebase indexing to identify the minimal set of modified files.
3. Failing Regression Test: Author a dedicated unit/integration test reproducing the exact failure.
4. Surgical Remediation: Apply targeted modifications with zero side-effects to unrelated subsystems.
5. Verification Pass: Run the full test suite and confirm zero regressions.
"#,
    )
}

/// 13. Prompt Injection Defense Skill
pub fn prompt_injection_defense() -> EccSkill {
    EccSkill::new(
        "prompt-injection-defense",
        "Indirect prompt injection defense, token smuggling mitigations, and untrusted payload containment",
        r#"# Prompt Injection Defense & Input Armor

## Defense Guidelines
1. Boundary Enclosure: Wrap untrusted user inputs, external MCP payloads, and file contents in distinct XML-style delimiters (`<user_data>...</user_data>`).
2. Instruction Isolation: Explicitly instruct models to treat enclosed blocks as passive data, never as system instructions.
3. Token Smuggling Scanning: Strip or escape homoglyphs, zero-width characters, and base64-encoded jailbreak preambles.
4. AgentShield Interception: Verify all outbound tool calls generated by models against policy rules before OS execution.
"#,
    )
}

/// 14. Secret Entropy Scanner Skill
pub fn secret_entropy_scanner() -> EccSkill {
    EccSkill::new(
        "secret-entropy-scanner",
        "Shannon entropy secret detection, token format matching, and automated credential redaction",
        r#"# Secret & Credential Entropy Scanner

## Scanning Rules
1. Pattern Matching: Scan for known secret prefixes (`sk-ant-`, `sk-proj-`, `AIzaSy`, `xai-`, `ghp_`, `gho_`, `AWS_SECRET`).
2. Shannon Entropy Analysis: Flag high-entropy base64/hex strings (>4.5 bits/char) appearing in unexpected string literals.
3. Redaction Protocol: Automatically replace discovered tokens with `[REDACTED_SECRET_KEY]` before logging or stdout exposure.
4. Workspace Isolation: Block tools from reading `.env`, `.pem`, `id_rsa`, `id_ed25519`, and `.gnupg` directory files.
"#,
    )
}

/// 15. POSIX Shell Sanitizer Skill
pub fn posix_shell_sanitizer() -> EccSkill {
    EccSkill::new(
        "posix-shell-sanitizer",
        "POSIX shell AST validation, command injection prevention, and destructive execution interceptors",
        r#"# POSIX Shell Sanitizer & Command Interception

## Safety Rules
1. Prohibited Commands: Block destructive operations (`rm -rf /`, `mkfs`, `dd if=/dev/zero`, `:(){ :|:& };:`).
2. Argument Whitelisting: Enforce strict parameter validation on shell commands.
3. Path Confinement: Reject commands with traversal escapes (`../../../`) targeting root filesystems.
4. Subshell Isolation: Execute commands in dedicated subprocesses with strict timeouts and resource limits.
"#,
    )
}

/// 16. Bun Native APIs Skill
pub fn bun_native_apis() -> EccSkill {
    EccSkill::new(
        "bun-native-apis",
        "High-performance Bun native APIs, bun:sqlite vector storage, Bun.serve, and zero-transpile TypeScript",
        r#"# Bun Native Performance & APIs

## Best Practices
1. Fast I/O: Prefer `Bun.file(path)` and `Bun.write(path, content)` over legacy `node:fs` streams.
2. Built-in SQLite: Leverage `import { Database } from "bun:sqlite"` with prepared statements for zero-overhead persistence.
3. High-Throughput HTTP: Use `Bun.serve({ fetch(req) { ... } })` with native WebSocket pub/sub for sub-millisecond latency.
4. TypeScript Runtime: Rely on Bun's built-in transpiler without separate `tsc` build steps during development.
"#,
    )
}

/// 17. Seccomp Sandbox Rules Skill
pub fn seccomp_sandbox_rules() -> EccSkill {
    EccSkill::new(
        "seccomp-sandbox-rules",
        "Linux kernel system call filtering, namespace/cgroup isolation, and process jail safety policies",
        r#"# Linux Seccomp & Kernel Sandbox Rules

## Sandboxing Policy
1. System Call Whitelisting: Allow essential POSIX calls (`read`, `write`, `futex`, `epoll_wait`, `nanosleep`); block dangerous primitives (`ptrace`, `kexec_load`, `mount`, `reboot`).
2. Resource Quotas: Enforce memory quotas (`RLIMIT_AS`), CPU time caps (`RLIMIT_CPU`), and open descriptor limits (`RLIMIT_NOFILE`).
3. Filesystem Jails: Restrict write access strictly to designated temporary working directories.
"#,
    )
}

/// 18. Hexagonal Architecture Skill
pub fn hexagonal_architecture() -> EccSkill {
    EccSkill::new(
        "hexagonal-architecture",
        "Ports & Adapters clean architecture, decoupling business domain logic from databases and external APIs",
        r#"# Hexagonal (Ports & Adapters) Architecture

## Architectural Invariants
1. Core Domain Independence: Pure domain logic must have zero dependencies on frameworks, databases, or network protocols.
2. Ports as Traits/Interfaces: Define primary (driving) and secondary (driven) ports as language-native abstractions (`trait` / `interface`).
3. Adapters in Isolation: Database clients, HTTP endpoints, and CLI handlers live in distinct adapter modules wrapping ports.
4. Testability: The domain layer must be 100% unit-testable in memory without databases or external servers.
"#,
    )
}

/// 19. Backend Patterns Skill
pub fn backend_patterns() -> EccSkill {
    EccSkill::new(
        "backend-patterns",
        "Enterprise backend resilience: connection pooling, idempotent operations, circuit breakers, and graceful shutdown",
        r#"# Enterprise Backend Resilience Patterns

## Patterns
1. Idempotency: Enforce idempotency keys on mutating endpoints to handle network retries safely.
2. Connection Pooling: Use robust pools with connection health checks, max lifespans, and acquisition timeouts.
3. Circuit Breakers: Guard upstream microservices with circuit breakers to prevent cascading system failures.
4. Graceful Shutdown: Handle `SIGINT`/`SIGTERM` by draining active in-flight requests before terminating worker pools.
"#,
    )
}

/// 20. Coding Standards Skill
pub fn coding_standards() -> EccSkill {
    EccSkill::new(
        "coding-standards",
        "Strict code hygiene, cyclomatic complexity limits, zero dead code, and maintainability enforcement",
        r#"# Engineering Coding Standards

## Standards Checklist
1. Cyclomatic Complexity: Keep function complexity under 10; decompose complex branched logic into small, testable helpers.
2. Zero Dead Code: Eliminate unused variables, dead imports, and obsolete comments.
3. Consistent Formatting: Follow standard linters and formatters (`cargo fmt`, `cargo clippy`, `prettier`).
4. Self-Documenting Code: Name identifiers for intent rather than implementation mechanics.
"#,
    )
}

/// 21. Structured Analysis Yourdon Skill
pub fn structured_analysis_yourdon() -> EccSkill {
    EccSkill::new(
        "structured-analysis-yourdon",
        "Modern Structured Analysis (Edward Yourdon): environmental modeling, context diagrams, event-response lists, leveled DFDs, and state transition diagrams",
        r#"# Modern Structured Analysis & Design (Edward Yourdon)

## 1. The Environmental Model (System Boundary Definition)
- Statement of Purpose: Define a concise 1-2 sentence statement defining exact objective and boundaries.
- Context Diagram (DFD Level-0): Represent the system as a single process 0 surrounded by external terminators.
- Event-Response List: Categorize stimuli into External Events (actors), Temporal Events (time), and State Events (internal thresholds).

## 2. The Behavioral Model (Leveled DFDs)
- Event Partitioning (DFD Level-1): Draw exactly one process bubble per event in the event list.
- Data Stores: Shared memory/databases between processes.
- Level-2 Decomposition: Decompose complex process bubbles until leaf nodes represent single cohesive transformations.

## 3. State Transition Diagrams (STDs)
- Model time-dependent behavior: States, Transitions, Guard Conditions, and Actions.
- Ensure exhaustive event handling with zero unhandled state deadlocks.
"#,
    )
}

/// 22. Modular Coupling & Cohesion Skill
pub fn modular_coupling_cohesion() -> EccSkill {
    EccSkill::new(
        "modular-coupling-cohesion",
        "Structured Systems Design (Meilir Page-Jones): module cohesion hierarchy, coupling reduction, fan-in/fan-out bounds, and transform/transaction factoring",
        r#"# Modular Systems Design: Cohesion & Coupling (Meilir Page-Jones)

## 1. The 7 Levels of Module Cohesion (Target: Functional Cohesion)
1. Functional (Highest): Performs exactly one problem-related task. Every line contributes to that task.
2. Sequential: Output of one step is direct input to the next step.
3. Communicational: Operates on the same shared input data set.
4. Procedural (Avoid): Grouped solely by execution order.
5. Temporal (Avoid): Grouped solely because they execute at the same time (e.g. init_everything).
6. Logical (Dangerous): Multi-branch switch doing unrelated tasks based on a flag.
7. Coincidental (Forbidden): Arbitrary groupings (e.g. utils.ts, misc.rs).

## 2. The 5 Levels of Module Coupling (Target: Data Coupling)
1. Data (Best): Modules communicate solely by passing discrete, typed arguments.
2. Stamp: Passing composite structs when only a few fields are needed; prune to pass only what is needed.
3. Control (Avoid): Passing flags (is_admin, mode) that alter internal control flow.
4. Common (Dangerous): Communicating via global shared mutable memory.
5. Content (Forbidden): Directly accessing or mutating another module's internal state.

## 3. Structural Factoring Heuristics
- Fan-out <= 7 (a module coordinates at most 7 subordinates).
- High fan-in is encouraged (maximize reuse of pure logic).
- File limit <= 300 lines, function limit <= 50 lines.
"#,
    )
}

/// 23. Data Dictionary & Mini-Specs Skill
pub fn data_dictionary_minispecs() -> EccSkill {
    EccSkill::new(
        "data-dictionary-minispecs",
        "Structured System Specification (Tom DeMarco): formal data dictionary definitions, structured English mini-specifications, and DFD conservation balancing",
        r#"# Structured Specification & Mini-Specs (Tom DeMarco)

## 1. Formal Data Dictionary Notation
- `=` is composed of (definition)
- `+` AND (concatenation)
- `[ | ]` OR (exclusive choice / discriminated union)
- `{}` Iteration / Array (0 or more occurrences)
- `()` Optional field (0 or 1 occurrence)
- `*...*` Semantic comment / unit invariant

## 2. Structured English Mini-Specifications
- Imperative Action Verbs: COMPUTE, VALIDATE, LOOKUP, DISPATCH, PERSIST, EMIT.
- Deterministic Control Structures: IF/THEN/ELSE, CASE/OF, FOR EACH, WHILE.
- Zero Ambiguity: Eliminate vague adjectives; all thresholds must be concrete constants.

## 3. Conservation & Balancing Rules
- Rule of Data Conservation: A process cannot create data from nothing or discard necessary data.
- Rule of Leveled Balancing: Child diagram inputs and outputs must exactly equal parent process bubble inputs and outputs.
"#,
    )
}

/// 24. Domain-Driven Design Skill
pub fn domain_driven_design() -> EccSkill {
    EccSkill::new(
        "domain-driven-design",
        "Domain-Driven Design (Eric Evans & Vlad Khononov): Ubiquitous Language, Bounded Contexts, Aggregate Roots, Value Objects, Domain Events, and Anti-Corruption Layers",
        r#"# Domain-Driven Design (Eric Evans & Vlad Khononov)

## 1. Strategic Design
- Ubiquitous Language: Shared, strictly defined domain vocabulary in code and speech.
- Bounded Contexts: Explicit architectural and linguistic boundaries; never merge contexts into god-models.
- Anti-Corruption Layer (ACL): Translate external/legacy models into pure internal domain types.

## 2. Tactical Design
- Value Objects: Immutable, self-validating, structural equality, no identity (e.g. Money, DocketNumber).
- Entities: Enduring identity across lifecycle; mutate only via domain methods.
- Aggregates & Aggregate Roots: Transactional consistency boundary; external callers reference ONLY the root.
- Domain Events: Past-tense immutable records (CaseRaffled, FeePaid) decoupling side effects.
"#,
    )
}

/// 25. Design by Contract Skill
pub fn design_by_contract() -> EccSkill {
    EccSkill::new(
        "design-by-contract",
        "Design by Contract (Bertrand Meyer): preconditions, postconditions, class invariants, defensive boundary validation, and fail-fast invariant enforcement",
        r#"# Design by Contract (Bertrand Meyer)

## 1. The Contract Triad
- Preconditions (`require`): Obligations on the caller. Violation indicates a caller bug; fail-fast.
- Postconditions (`ensure`): Guarantees made by the callee. Violation indicates a callee bug.
- Class/Aggregate Invariants (`invariant`): Truths that must hold before and after every public method.

## 2. Contract Principles
- Distinguish contract violations (programmer bugs -> panic/assert) from expected domain errors (user inputs -> Result::Err).
- Never catch and swallow contract violations.
- Derive property tests directly from contract preconditions and postconditions.
"#,
    )
}

/// 26. Statechart & FSM Modeling Skill
pub fn statechart_fsm_modeling() -> EccSkill {
    EccSkill::new(
        "statechart-fsm-modeling",
        "Hierarchical Statecharts & FSM Modeling (David Harel & Ian Horrocks): finite state machines, orthogonal regions, guarded transitions, entry/exit actions, and deadlock-free event lifecycles",
        r#"# Hierarchical Statecharts & FSM Modeling (David Harel & Ian Horrocks)

## 1. Statechart Formalisms
- Superstates & Substates: Hierarchical state clustering to eliminate transition explosion.
- Orthogonal Regions: Concurrent independent state machines in a single entity (e.g. Judicial Track || Financial Track).
- Guarded Transitions: Event [GuardCondition] / Action -> TargetState.
- Entry/Exit Actions: Guaranteed execution on entering and leaving states.

## 2. Determinism & Safety
- Run-to-Completion: Events are fully processed before the next event begins.
- Exhaustive Coverage: In Rust/TypeScript, model states as closed enums; handle every event explicitly.
- Make invalid transitions unrepresentable in the type system.
"#,
    )
}

/// 27. Data-Intensive Architecture Skill
pub fn data_intensive_architecture() -> EccSkill {
    EccSkill::new(
        "data-intensive-architecture",
        "Data-Intensive Applications Architecture (Martin Kleppmann): transactional isolation, ACID guarantees, idempotency keys, write-ahead logs, CQRS, eventual consistency, and distributed consensus",
        r#"# Designing Data-Intensive Architecture (Martin Kleppmann)

## 1. The Core Trinity
- Reliability: Fault tolerance; absorb individual component faults without system failures.
- Scalability: Characterize load (throughput, fan-out, p99 latencies) and scale bottlenecks.
- Maintainability: Operability, simplicity, and schema evolvability.

## 2. Transactions & Concurrency
- Isolation levels: Understand dirty reads, non-repeatable reads, phantom reads, and write skew.
- Use Serializable or explicit pessimistic locking (SELECT FOR UPDATE) for financial and judicial allocations.

## 3. Distributed Patterns
- Idempotency Keys: Enforce on every mutating endpoint and tool call.
- Write-Ahead Log (WAL): Canonical append-only log as source of truth; derived views updated asynchronously.
- CQRS: Separate write command validation from high-performance read query projections.
"#,
    )
}

/// 28. Deep Modules & Complexity Skill (John Ousterhout)
pub fn deep_modules_complexity() -> EccSkill {
    EccSkill::new(
        "deep-modules-complexity",
        "A Philosophy of Software Design (John Ousterhout): deep vs shallow modules, narrow interfaces hiding complex implementations, information hiding, defining errors out of existence, and eliminating pass-through abstractions",
        r#"# Deep Modules & Complexity Control (John Ousterhout)

## 1. Deep vs. Shallow Modules
- Deep Module: Simple, narrow interface concealing deep, sophisticated internal logic.
- Shallow Module: Wide or complex interface with trivial implementation; eliminate shallow wrappers.
- Information Hiding: Internal data formats, synchronization, and caching must never leak into API signatures.
- Information Leakage: Callers should never have to coordinate multi-step lifecycle sequences when a single call suffices.

## 2. Defining Errors Out of Existence
- Subsumption: Redefine semantics so edge cases are valid normal behavior (e.g. deleting non-existent item is a no-op).
- Masking: Recover or retry internally rather than bubbling transient errors.
- Aggregation: Handle errors at overarching subsystem boundaries.

## 3. Eliminating Pass-Through Anti-Patterns
- Flatten pass-through methods that merely forward parameters without transformation.
- Avoid pass-through arguments across layers using context or constructor bindings.
"#,
    )
}

/// 29. Legacy Code Seams & Characterization Skill (Michael Feathers)
pub fn legacy_seams_characterization() -> EccSkill {
    EccSkill::new(
        "legacy-seams-characterization",
        "Working Effectively with Legacy Code (Michael Feathers): test-harness establishment, identifying seams, sensing and separation, characterization testing, and non-destructive sprout/wrap methods",
        r#"# Working Effectively with Legacy Code & Seams (Michael Feathers)

## 1. The Legacy Dilemma
- Legacy code is code without automated tests (including newly AI-generated unverified code).
- Establish seams without altering production behavior to bring code under test.

## 2. Seams: Object, Link, and Compile Seams
- Object Seams: Inject traits, interfaces, or subclass overrides.
- Sensing: Use seams to observe side-effects and values computed inside opaque functions.
- Separation: Use seams to decouple external dependencies (databases, network) during tests.

## 3. Characterization Testing & Safe Interventions
- Characterization Tests: Pin existing black-box behavior before refactoring.
- Sprout Method/Class: Write new features as pure, independently tested sprouts.
- Wrap Method/Class: Decorate legacy calls without mutating internal mechanics.
"#,
    )
}

/// 30. Code Smells & Atomic Refactoring Catalog Skill (Martin Fowler)
pub fn catalog_refactoring_smells() -> EccSkill {
    EccSkill::new(
        "catalog-refactoring-smells",
        "Refactoring & Code Smells Catalog (Martin Fowler): deterministic detection of architectural code smells, behavioral preservation, and atomic AST transformations",
        r#"# Code Smells & Atomic Refactoring Catalog (Martin Fowler)

## 1. Diagnostic Code Smells
- Primitive Obsession: Replace raw strings/numbers with validated Value Objects.
- Feature Envy: Move methods to the data structures they envy.
- Data Clumps: Bundle recurring parameter groups into Parameter Objects/Structs.
- Divergent Change vs. Shotgun Surgery: Separate divergent responsibilities; consolidate shotgun edits.

## 2. Atomic Behavior-Preserving Transformations
- Extract Function: Decompose high cognitive load blocks into expressive helpers.
- Replace Temp with Query: Eliminate mutable temp variables with pure deterministic queries.
- Replace Conditional with Polymorphism/Match: Replace sprawling if-else ladders with exhaustive pattern matching or traits.
- The Refactoring Rhythm: Make one atomic transformation, run test suite, ensure green, commit.
"#,
    )
}

/// 31. Production Resiliency & Stability Patterns Skill (Michael Nygard)
pub fn production_resilience_release_it() -> EccSkill {
    EccSkill::new(
        "production-resilience-release-it",
        "Production Resiliency Engineering (Michael Nygard - Release It!): circuit breakers, bulkheads, timeouts, steady-state stability, anti-fragility, and defense against cascading failures and retry storms",
        r#"# Production Resiliency & Stability Patterns (Michael Nygard - Release It!)

## 1. Stability Anti-Patterns
- Cascading Failures: Unbounded thread or pool blocking bringing down upstream services.
- Retry Storms: Blind retries without exponential backoff and randomized jitter.
- Unbounded Queues: OOM crashes under backpressure; enforce hard bounds.
- Missing Timeouts: Network calls must always define explicit connect and read timeouts.

## 2. Core Stability Patterns
- Circuit Breaker: Closed, Open, and Half-Open states guarding fragile downstreams.
- Bulkheads: Partition thread pools, connections, and compute into isolated failure domains.
- Fail Fast: Validate preconditions early before locking or allocating resources.
- Steady State: Prevent memory/disk leaks with automatic purging and resource recycling.
- Load Shedding: Drop excess load with backpressure rather than entering latency death spirals.
"#,
    )
}

/// 32. Evolutionary Architecture & Fitness Functions Skill (Ford, Parsons, Kua)
pub fn evolutionary_fitness_functions() -> EccSkill {
    EccSkill::new(
        "evolutionary-fitness-functions",
        "Building Evolutionary Architectures (Neal Ford, Rebecca Parsons, Patrick Kua): architectural fitness functions, automated structural verification, boundary integrity, and preventing architectural drift across AI iterations",
        r#"# Evolutionary Architecture & Fitness Functions (Ford, Parsons, Kua)

## 1. Architectural Fitness Functions
- Automated, objective verification tests asserting architectural characteristics.
- Prevent architectural drift and erosion across multi-prompt AI development sessions.

## 2. Categories of Fitness Functions
- Layering & Direction: Strict inward dependency rules (Hexagonal/Clean); Domain never imports CLI/Web.
- Acyclic Dependencies: Enforce DAG structure across modules with zero circular references.
- Complexity Budgets: Automated gates on cyclomatic complexity and max file/function lengths.
- Performance & Compliance: Automated benchmark regression gates and vulnerability scanners.
"#,
    )
}

/// 33. Temporal Invariants & Formal Specification Skill (Lamport & Wayne)
pub fn temporal_invariants_tla() -> EccSkill {
    EccSkill::new(
        "temporal-invariants-tla",
        "Temporal Invariants & Formal Specification (Leslie Lamport & Hillel Wayne): safety invariants, liveness guarantees, state space exhaustion, and race-free concurrent and distributed state modeling",
        r#"# Temporal Invariants & State Space Verification (Lamport / Wayne)

## 1. Safety vs. Liveness
- Safety Properties ('Nothing bad happens'): Invariants that must hold across every reachable state.
- Liveness Properties ('Something good eventually happens'): Guarantees of forward progress without deadlock or livelock.

## 2. State Space Modeling Before Coding
- Minimal State Tuple: Define discrete state variables explicitly before writing async logic.
- Guarded Transitions: Explicitly define enabled conditions for every state mutation.
- Concurrency Interleaving: Model concurrent interleavings to eliminate race conditions.
- Type-Level Guarantees: Make invalid states unrepresentable in the type system.
"#,
    )
}

/// 34. Conceptual Integrity & Engineering Over Time Skill (Brooks & Winters)
pub fn conceptual_integrity_systems() -> EccSkill {
    EccSkill::new(
        "conceptual-integrity-systems",
        "Conceptual Integrity & Software Engineering at Scale (Fred Brooks & Titus Winters): unified architectural vision, second-system syndrome avoidance, Hyrum's law, and sustainability over time",
        r#"# Conceptual Integrity & Engineering Over Time (Brooks & Winters)

## 1. Conceptual Integrity (Fred Brooks)
- The Central Virtue: A unified architectural vision outweighs an accumulation of uncoordinated features.
- Singular Design Dialect: Enforce consistent naming, error handling, and concurrency patterns across all modules.
- Second-System Syndrome: Resist over-complicating extensions with speculative features.

## 2. Software Engineering Over Time (Titus Winters)
- Programming Integrated Over Time: Design code to be maintainable, upgradeable, and decay-resistant for years.
- Hyrum's Law: All observable behaviors become contractual dependencies; explicitly encapsulate internals.
- Shift-Left Verification: Catch regressions as early as possible in the development lifecycle.
"#,
    )
}

/// 35. Refactoring UI Skill (Adam Wathan & Steve Schoger)
pub fn refactoring_ui() -> EccSkill {
    EccSkill::new(
        "refactoring-ui",
        "Tactical visual design & layout refactoring (Wathan & Schoger): grayscale-first design, optical alignment, 8pt spatial grid, layered elevation shadows, typography contrast scales. Triggers: grayscale first, 8pt grid, optical balance, typography scale, layered shadows, visual hierarchy.",
        r#"# Refactoring UI Engineering Skill

## 1. Core Principles
- Grayscale First: Formulate hierarchy, whitespace, and visual balance in monochrome before introducing color.
- Hierarchy Over Pure Sizing: Use font weight, contrast (text-slate-900 vs text-slate-500), and spatial isolation.
- Systematic Spacing: Enforce an 8pt/4pt scale (4, 8, 12, 16, 24, 32, 48, 64px).
- Optical Balancing: Shift asymmetric shapes manually by 1–2px for visual centering and optical balance.
- Layered Elevation: Ambient soft drop shadows combined with directional key-light shadows.
"#,
    )
}

/// 36. Microinteractions & Tactile Feedback Skill (Dan Saffer)
pub fn microinteractions_design() -> EccSkill {
    EccSkill::new(
        "microinteractions-design",
        "Microinteractions & Tactile Feedback (Dan Saffer): trigger-rule-feedback-loops, spring physics, sub-50ms button press, optimistic UI, skeleton shimmers, tactile states. Triggers: spring physics, sub-50ms, button press, optimistic ui, tactile feedback, skeleton shimmer, microinteractions.",
        r#"# Microinteractions & Tactile Feedback Skill

## 1. 4-Part Interaction Anatomy
- Trigger: User-initiated or system-initiated event.
- Rules: State machine and programmatic constraints.
- Feedback: Real-time sensory response (<= 50ms).
- Loops & Modes: Recurrence parameters and transient UI states.

## 2. Spring Physics
- Damped Harmonic Oscillator: Replace mechanical bezier curves with physical springs (k=260, damping=20).
- Immediate Response: Instant feedback on pointerdown (scale-98, tint) rather than awaiting mouseup.
- Skeleton Shimmers: Preserve layout stability and eliminate Cumulative Layout Shift (CLS).
"#,
    )
}

/// 37. Laws of UX Skill (Jon Yablonski)
pub fn laws_of_ux() -> EccSkill {
    EccSkill::new(
        "laws-of-ux",
        "Laws of UX & Cognitive Ergonomics (Jon Yablonski): Doherty threshold (<400ms), Hick's law, progressive disclosure, Fitts's law, Miller's law (7±2), Jakob's law, Aesthetic-Usability effect. Triggers: doherty threshold, hicks law, progressive disclosure, fitts law, millers law, cognitive ergonomics, laws of ux.",
        r#"# Laws of UX Engineering Skill

## 1. Cognitive Ergonomics Standards
- Doherty Threshold: Provide visual acknowledgement in < 100ms; full render in < 400ms.
- Hick's Law: Employ progressive disclosure; present no more than 3-5 primary actions per context.
- Fitts's Law: Enlarge touch targets to >= 44x44px; anchor high-frequency actions to viewport edges/bottom.
- Miller's Law: Chunk complex data into discrete visual modules (cards, grouped rows).
- Jakob's Law: Adhere to familiar mental models for navigation and search.
- Aesthetic-Usability Effect: Visual polish and harmony increase user tolerance and perceived performance.
"#,
    )
}

/// 38. Design Systems & Token Architecture Skill (Alla Kholmatova & Brad Frost)
pub fn design_systems_tokens() -> EccSkill {
    EccSkill::new(
        "design-systems-tokens",
        "Design Systems & Atomic Token Architecture (Kholmatova & Frost): 3-tier token hierarchy, semantic tokens, atomic design composition (atoms-to-pages), slot patterns. Triggers: 3-tier token hierarchy, semantic tokens, atomic design, design systems, design tokens, slot composition.",
        r#"# Design Systems & Token Architecture Skill

## 1. 3-Tier Token Hierarchy
- Tier 1 (Global Primitives): Raw palette and scales (--slate-900, --space-4).
- Tier 2 (Semantic Intent): Contextual variables (--surface-canvas, --text-primary).
- Tier 3 (Component Binding): Bound scoped properties (--button-primary-bg).

## 2. Atomic Composition
- Atoms -> Molecules -> Organisms -> Templates -> Pages.
- Slot Composition: Favor compound slots over 30+ prop explosion anti-patterns.
"#,
    )
}

/// 39. About Face & Power-User Ergonomics Skill (Alan Cooper)
pub fn about_face_interaction_design() -> EccSkill {
    EccSkill::new(
        "about-face-interaction-design",
        "About Face & Power-User Ergonomics (Alan Cooper): software posture theory (sovereign posture, transient, daemonic), eliminate excise, reversible undo stacks, cmd+k command palette, undo banner. Triggers: sovereign posture, cmd+k, command palette, eliminate excise, undo banner, interaction design, about face.",
        r#"# About Face & Power-User Ergonomics Skill

## 1. Software Posture Theory
- Sovereign Posture: High-density, dark/subdued palettes, deep keyboard shortcuts, multi-pane workspaces.
- Transient Posture: Single-function utilities with immediate dismissibility.
- Daemonic Posture: Background processes and status indicators.

## 2. Elimination of Excise
- Reversible Actions: Replace intrusive modal confirmation alerts with optimistic deletes and 1-click Undo.
- Universal Command Palette: Index all actions and navigation under Cmd+K / Ctrl+K.
- State Persistence: Never discard window bounds, scroll offsets, or drafts.
"#,
    )
}

/// 40. Designing for Emotion & Delight Skill (Aarron Walter)
pub fn designing_for_emotion() -> EccSkill {
    EccSkill::new(
        "designing-for-emotion",
        "Designing for Emotion & Delight (Aarron Walter): Maslow emotional hierarchy, delight hierarchy, creative empty state, empathetic error state handling, brand personality, celebratory confetti milestones. Triggers: delight hierarchy, empathetic error state, celebratory confetti, empty state, emotional design, designing for emotion.",
        r#"# Designing for Emotion Skill

## 1. Emotional Hierarchy
- Functional -> Reliable -> Usable -> Pleasurable & Delightful.

## 2. Humanized Interactions
- Creative Empty States: Turn zero-data screens into narrative invitations with clear primary CTAs.
- Empathetic Errors: Explain clearly without jargon, confirm data safety, and offer 1-click recovery.
- Celebratory Milestones: Reward user completions with tasteful micro-delight (confetti, achievement badges).
"#,
    )
}

/// 41. Kronos Candlestick K-Line Discrete Tokenization Skill
pub fn kronos_kline_modeling() -> EccSkill {
    EccSkill::new(
        "kronos-kline-modeling",
        "Financial Candlestick (K-line) discrete tokenization using Binary Spherical Quantization (BSQ), autoregressive Transformer architectures, hierarchical dual decoding (s1 price tokens, s2 volume/residual tokens), and synthetic market generation. Triggers: kronos, k-line, candlestick, bsq, binary spherical quantization, dual decoding, market simulation, synthetic market generation, ohlcv sliding window.",
        r#"# Kronos Financial Candlestick Modeling Skill

## 1. Architectural Foundations
- Sliding-Window Invariant Normalization: Anchor OHLC values relative to reference price P_base = C_{t-W}; standardize volume against trailing rolling EMA.
- Binary Spherical Quantization (BSQ): Project continuous latent embeddings onto unit hypersphere S^{D-1}, followed by sign binarization into discrete codebook tokens: q_b = sign(z_b / ||z||_2).
- Hierarchical Dual Decoding: Autoregressively decompose interval generation into s1 (macro price direction & trend) and s2 (microstructure volume & residual spread conditioned on s1).
- Market Physics Invariants: Strictly preserve candlestick boundaries: Low <= min(Open, Close) <= max(Open, Close) <= High, Volume >= 0.

## 2. Quantitative Formulation & Loss
- Hypersphere projection: z_tilde = z / (||z||_2 + eps).
- Token ID: k_tau = sum_{b=0}^{B-1} 2^b * I(q_b > 0).
- Straight-Through Estimator (STE): z_q = z_tilde + stop_gradient(q(z_tilde) - z_tilde).
- Joint Likelihood: P(S_{1:T}) = prod_{t=1}^T P(s_1^t | s_1^{<t}, s_2^{<t}) * P(s_2^t | s_1^{<=t}, s_2^{<t}).
"#,
    )
}

/// 42. Microsoft Qlib Alpha Engineering Skill
pub fn qlib_alpha_engineering() -> EccSkill {
    EccSkill::new(
        "qlib-alpha-engineering",
        "Microsoft Qlib AI-driven quantitative investment pipeline, alpha factor engineering, Barra risk factor neutralization (market beta, size, industry, volatility), turnover cost/slippage modeling, and Top-K backtesting with RankIC evaluation. Triggers: qlib, alpha factors, barra risk neutralization, top-k backtesting, rankic, information coefficient, icir, portfolio turnover, slippage modeling.",
        r#"# Microsoft Qlib Alpha Engineering Skill

## 1. Core Principles
- Point-in-Time Data Integrity: Strict elimination of look-ahead bias and survivorship bias across cross-sectional universes.
- Barra Multi-Factor Neutralization: Orthogonalize raw alpha factor predictions against systematic risk exposures (Market Beta, Log Market Cap, Industry Dummies, Volatility).
- Turnover & Slippage Modeling: Model realistic turnover drag and execution frictions: Cost_t = Turnover_t * (c_fee + c_tax + Slippage(V_t)).
- Non-Parametric RankIC & ICIR: Evaluate factor predictive performance via cross-sectional Spearman rank correlation.

## 2. Mathematical Rigor
- OLS Risk Neutralization: f_t = X_t * beta_t + epsilon_t.
- Regularized Annihilator: M_t = I - X_t * (X_t^T * X_t + lambda * I)^{-1} * X_t^T; neutralized alpha f_tilde = M_t * f_t.
- RankIC = Corr(rank(f_tilde_t), rank(r_{t+1})).
- Annualized ICIR = (Mean(RankIC) / Std(RankIC)) * sqrt(252). Target Mean(RankIC) > 0.05, ICIR > 0.8.
"#,
    )
}

/// 43. Amazon Chronos & Salesforce MOIRAI Forecasting Skill
pub fn chronos_moirai_forecasting() -> EccSkill {
    EccSkill::new(
        "chronos-moirai-forecasting",
        "Universal time series foundation models (Amazon Chronos, Salesforce MOIRAI), zero-shot probabilistic forecasting, patch-based multi-resolution architectures, and quantile Value-at-Risk (VaR / CVaR) estimation. Triggers: chronos, moirai, zero-shot time series, probabilistic forecasting, quantile prediction, value-at-risk, var, cvar, patch forecasting, universal time series.",
        r#"# Chronos & MOIRAI Time Series Foundation Models Skill

## 1. Architectural Foundations
- Universal Time Series Modeling: Pre-trained foundation models replacing single-asset statistical fits across diverse financial instruments.
- Patch-Based Multi-Resolution Tokenization: Decompose real-valued series into multi-scale patches (8, 16, 32, 64) with Any-Variate attention.
- Zero-Shot Probabilistic Forecasting: Generate full quantile distributions y_hat^{(q)} for q in {0.01, 0.05, 0.10, 0.50, 0.90, 0.95, 0.99}.
- Quantile Value-at-Risk (VaR) & Expected Shortfall (CVaR): VaR_alpha = -y_hat^{(alpha)}, CVaR_alpha = -(1/alpha) * integral_0^alpha y_hat^{(u)} du.

## 2. Loss & Calibration
- Pinball Loss (Quantile Loss): L_q(y, y_hat) = max(q * (y - y_hat), (1 - q) * (y_hat - y)).
- Reversible Instance Normalization (RevIN): Affine de-trending and instance normalization preserving relative percentage volatility.
- Quantile Monotonicity: Enforce non-crossing quantile invariants: y_hat^{(q1)} <= y_hat^{(q2)} for all q1 < q2.
"#,
    )
}

/// 44. FinGPT & FinRL Multimodal Financial NLP Skill
pub fn fingpt_multimodal_nlp() -> EccSkill {
    EccSkill::new(
        "fingpt-multimodal-nlp",
        "Multimodal financial NLP (FinGPT) and deep reinforcement learning (FinRL) for SEC 10-K/10-Q filing analysis, sentiment alpha factor extraction, and optimal trade execution (PPO/DDPG). Triggers: fingpt, finrl, financial nlp, sec filing sentiment, 10-k analysis, 10-q filings, sentiment factors, reinforcement learning execution, almgren-chriss, ppo trade execution.",
        r#"# FinGPT & FinRL Multimodal Financial NLP Skill

## 1. Architectural Foundations
- Domain-Adapted Financial LLMs: Parameter-efficient fine-tuning (LoRA) on EDGAR SEC filings, financial news, and conference calls.
- Longitudinal Item Parsing: Differential semantic comparison of Item 1A (Risk Factors), Item 7 (MD&A), and Item 7A (Market Risk).
- Sentiment Factor Construction: Exponential half-life decay aggregation of sentiment polarity, neutralized against market sentiment beta.
- Deep RL Execution Optimization (FinRL): MDP formulation of TWAP/VWAP liquidation with Almgren-Chriss market impact penalties.

## 2. Execution MDP & Reward
- State: s_k = (r_k, Spread_k, Imbalance_k, q_k / Q, k / N).
- Action: a_k in [0, 1] (fraction of remaining inventory executed).
- Almgren-Chriss Reward: R_k = (P_k - P_benchmark) * Delta_q_k - eta * (Delta_q_k)^2 - lambda * sigma^2 * q_k^2 * tau.
- Timestamp Grounding: Always index filings by EDGAR acceptance_datetime, never fiscal period end.
"#,
    )
}

/// 45. Microsoft RD-Agent Autonomous Alpha Mining Skill
pub fn rdagent_factor_mining() -> EccSkill {
    EccSkill::new(
        "rdagent-factor-mining",
        "Microsoft RD-Agent autonomous quantitative R&D, automated alpha factor mining, market anomaly hypothesis formulation, symbolic factor expression trees, collinearity filtering, and factor half-life decay management. Triggers: rdagent, rd-agent, autonomous alpha mining, rd-agent alpha mining, factor mining, hypothesis generation, factor expression tree, automated quant r&d, factor decay.",
        r#"# Microsoft RD-Agent Autonomous Alpha Mining Skill

## 1. Autonomous R&D Loop
- Economic Hypothesis Formulation: Ground factor generation in market microstructure anomalies (liquidity premium, PEAD, idiosyncratic volatility).
- Symbolic Factor Expression Trees: Represent factors as DAG expression trees over unary time-series operators (Ts_Mean, Ts_Std, Ts_Rank, Ts_DecayLinear) and cross-sectional operators (Cs_Rank, Cs_ZScore, Cs_Neutralize).
- Factor Collinearity Pruning: Reject candidate factors with cross-sectional correlation rho > 0.60 against existing factor library.
- Factor Decay & Half-Life Monitoring: Fit RankIC(t) = RankIC_0 * 2^{-t / tau}; auto-retire factors when ICIR < 0.30 or turnover > 40%.

## 2. Evaluation & Overfitting Guardrails
- Deflated Sharpe Ratio (DSR) & Holm-Bonferroni correction for multiple hypothesis testing.
- Tree Complexity Penalties: Cap tree depth <= 6 and node count <= 15 to prevent curve fitting.
- Net-of-Fee Requirement: Candidate alpha must sustain Sharpe > 1.5 after 15 bps two-way friction.
"#,
    )
}

/// 51. math-nature-of-code Skill
pub fn math_nature_of_code() -> EccSkill {
    EccSkill::new(
        "math-nature-of-code",
        "Physics simulations, vector kinematics, harmonic oscillations, Hooke's spring-damper dynamics, particle systems, and Craig Reynolds autonomous steering (seek, arrive, flocking boids: separation, alignment, cohesion) based on 'The Nature of Code' by Daniel Shiffman. Triggers: nature-of-code, boids, flocking, autonomous-steering, reynolds-steering, hooke-spring, particle-system, physics-simulation, harmonic-oscillator, vector-kinematics.",
        r#"# The Nature of Code: Physics, Vectors & Autonomous Swarms
- Semi-implicit Euler integration: v_{t+dt} = v_t + a_t * dt; x_{t+dt} = x_t + v_{t+dt} * dt.
- Damped harmonic oscillator: F_spring = -k * (x - x_rest) - c * v.
- Craig Reynolds steering: F_steer = clamp(v_desired - v_current, F_max).
- Flocking emergence: F_total = w_s * F_sep + w_a * F_ali + w_c * F_coh.
"#,
    )
}

/// 52. math-linear-algebra-savov Skill
pub fn math_linear_algebra_savov() -> EccSkill {
    EccSkill::new(
        "math-linear-algebra-savov",
        "Intuitive linear algebra, vector spaces, dot products, orthogonal projections, Gram-Schmidt orthogonalization, eigenvalues/eigenvectors, and Singular Value Decomposition (SVD) for low-rank embeddings based on 'No Bullshit Guide to Linear Algebra' by Ivan Savov and Sheldon Axler. Triggers: linear-algebra, vector-spaces, dot-product, orthogonal-projection, svd, singular-value-decomposition, eigenvalues, eigenvectors, gram-schmidt, low-rank-matrix.",
        r#"# Linear Algebra Done Right: Vector Spaces & Projections
- Orthogonal projection: P_W = A * (A^T * A)^{-1} * A^T.
- Cosine similarity: cos(theta) = (u . v) / (||u|| * ||v|| + eps).
- SVD factorization: A = U * Sigma * V^T.
- Low-rank approximation: A_k = sum_{i=1}^k sigma_i * u_i * v_i^T.
"#,
    )
}

/// 53. math-high-dimensional-data Skill
pub fn math_high_dimensional_data() -> EccSkill {
    EccSkill::new(
        "math-high-dimensional-data",
        "High-dimensional geometry, unit ball surface crust concentration, random vector near-orthogonality, Johnson-Lindenstrauss dimension reduction, and Markov chain random walks based on 'Foundations of Data Science' by Blum, Hopcroft, and Kannan. Triggers: high-dimensional-geometry, unit-ball-crust, johnson-lindenstrauss, random-projection, curse-of-dimensionality, random-walks, markov-chains, embedding-geometry.",
        r#"# Foundations of Data Science: High-Dimensional Geometry
- Volume concentration: Vol(B_d(1 - eps)) / Vol(B_d(1)) = (1 - eps)^d <= e^{-eps * d}.
- High-d orthogonality: For random unit vectors u, v, |u . v| <= sqrt((2 * ln(2/delta)) / d).
- Johnson-Lindenstrauss lemma: Distances preserved within (1 +/- eps) with target dim k = O(eps^{-2} * ln n).
"#,
    )
}

/// 54. math-information-inference Skill
pub fn math_information_inference() -> EccSkill {
    EccSkill::new(
        "math-information-inference",
        "Information theory, Shannon entropy, mutual information, Kullback-Leibler (KL) divergence, cross-entropy, Bayesian inference, and token entropy for hallucination detection based on David J.C. MacKay. Triggers: information-theory, shannon-entropy, mutual-information, kl-divergence, token-entropy, perplexity, hallucination-detection, bayesian-inference, mcmc, cross-entropy.",
        r#"# Information Theory, Inference & Learning
- Shannon entropy: H(X) = -sum p(x) * log2(p(x)).
- KL divergence: D_{KL}(P || Q) = sum P(x) * log(P(x) / Q(x)).
- Mutual information: I(X; Y) = H(X) - H(X|Y).
- Token uncertainty: Hallucination spike when H_token > tau_entropy.
"#,
    )
}

/// 55. math-causal-inference Skill
pub fn math_causal_inference() -> EccSkill {
    EccSkill::new(
        "math-causal-inference",
        "Causal inference, Structural Causal Models (SCM), Directed Acyclic Graphs (DAGs), d-separation, backdoor criterion, do-calculus, and counterfactual analysis based on 'The Book of Why' by Judea Pearl. Triggers: causal-inference, causal-dag, do-calculus, backdoor-criterion, confounding-bias, counterfactuals, structural-causal-model, d-separation, root-cause-analysis.",
        r#"# Causal Inference & Do-Calculus
- Observational vs Interventional: P(Y | X=x) != P(Y | do(X=x)).
- Backdoor adjustment: P(Y | do(X=x)) = sum_z P(Y | X=x, Z=z) * P(Z=z).
- Counterfactual: P(Y_{X=x} | X=x', Y=y').
"#,
    )
}

/// 56. math-category-theory Skill
pub fn math_category_theory() -> EccSkill {
    EccSkill::new(
        "math-category-theory",
        "Category theory for software architecture, functors, monads, natural transformations, Kleisli categories, and Railway-Oriented Programming (Result/Option monads) based on 'Category Theory for Programmers' by Bartosz Milewski. Triggers: category-theory, functors, monads, kleisli-category, natural-transformations, compositionality, railway-oriented-programming, morphisms, monoid.",
        r#"# Category Theory for Programmers
- Morphism composition: (g . f)(x) = g(f(x)), associative with identity id_A.
- Functor F: maps objects A -> F(A) and morphisms f -> F(f).
- Monad (T, eta, mu): unit eta: I -> T, multiplication mu: T^2 -> T.
- Kleisli composition for error pipelines: f >=> g = mu . T(g) . f.
"#,
    )
}

/// 57. math-game-engine-geometry Skill
pub fn math_game_engine_geometry() -> EccSkill {
    EccSkill::new(
        "math-game-engine-geometry",
        "Game engine 3D mathematics, unit quaternions without gimbal lock, Spherical Linear Interpolation (Slerp), 4x4 affine transforms, and ray-AABB intersection tests based on Eric Lengyel. Triggers: game-engine-math, quaternions, slerp, gimbal-lock, affine-transform, ray-intersection, aabb-box, 3d-geometry, perspective-projection.",
        r#"# Foundations of Game Engine Development: Quaternions & Transforms
- Unit quaternion: q = [s, v] = cos(theta/2) + sin(theta/2) * (u_x i + u_y j + u_z k).
- Slerp interpolation: Slerp(q1, q2, t) = (sin((1-t)*theta)*q1 + sin(t*theta)*q2) / sin(theta).
- Vector rotation: v' = q * [0, v] * q^{-1}.
"#,
    )
}

/// 58. math-code-first-calculus Skill
pub fn math_code_first_calculus() -> EccSkill {
    EccSkill::new(
        "math-code-first-calculus",
        "Code-first calculus, numerical finite-difference gradients, gradient descent optimization, 2D/3D polygon rasterization, and coordinate transforms in Python based on Paul Orland. Triggers: code-first-calculus, numerical-gradient, gradient-descent, polygon-rasterization, coordinate-transform, finite-difference, math-for-programmers.",
        r#"# Math for Programmers: Code-First Calculus & Optimization
- Finite difference gradient: df/dx = (f(x + h) - f(x - h)) / (2 * h).
- Gradient descent step: theta_{t+1} = theta_t - alpha * grad(J(theta_t)).
- 2D cross product for polygon orientation: (B_x - A_x)*(C_y - A_y) - (B_y - A_y)*(C_x - A_x).
"#,
    )
}

/// 59. math-bayesian-reasoning Skill
pub fn math_bayesian_reasoning() -> EccSkill {
    EccSkill::new(
        "math-bayesian-reasoning",
        "Bayesian statistics, prior/posterior distributions, Beta-Binomial conjugate updating, Bayes factors, and Thompson Sampling multi-armed bandit routing based on 'Bayesian Statistics the Fun Way' by Will Kurt. Triggers: bayesian-statistics, bayes-theorem, beta-binomial, posterior-probability, thompson-sampling, multi-armed-bandit, prior-distribution, epistemic-uncertainty.",
        r#"# Bayesian Statistics: Belief Updating & Multi-Armed Bandits
- Bayes' rule: P(theta | D) = (P(D | theta) * P(theta)) / P(D).
- Beta-Binomial update: Beta(alpha + successes, beta + failures).
- Thompson Sampling: sample theta_i ~ Beta(alpha_i, beta_i), pick arm with max sample.
"#,
    )
}

/// 60. math-ml-foundations Skill
pub fn math_ml_foundations() -> EccSkill {
    EccSkill::new(
        "math-ml-foundations",
        "Mathematical foundations of machine learning, matrix calculus, Jacobians, Hessians, multivariate Gaussian distributions, PCA eigendecomposition, and continuous optimization based on Deisenroth, Faisal, and Ong. Triggers: machine-learning-math, matrix-calculus, jacobian, hessian, multivariate-gaussian, pca, continuous-optimization, lagrange-multipliers.",
        r#"# Mathematics for Machine Learning: Optimization & Distributions
- Multivariate Gaussian: N(x; mu, Sigma) = (2*pi)^{-d/2} * |Sigma|^{-1/2} * exp(-1/2 * (x-mu)^T * Sigma^{-1} * (x-mu)).
- Principal Component Analysis: Sigma * v_i = lambda_i * v_i.
- Lagrangian: L(x, lambda) = f(x) + sum lambda_i * g_i(x).
"#,
    )
}

/// 61. math-networks-crowds-markets Skill
pub fn math_networks_crowds_markets() -> EccSkill {
    EccSkill::new(
        "math-networks-crowds-markets",
        "Graph theory, network centrality, PageRank power iteration, information cascades, small-world networks, and multi-agent swarm topologies based on Easley and Kleinberg. Triggers: network-graph-theory, pagerank, network-centrality, betweenness-centrality, information-cascades, small-world, agent-swarm-topology, graph-laplacian.",
        r#"# Networks, Crowds & Markets: Graph Topology & Cascades
- PageRank iteration: r_{t+1} = d * M * r_t + (1 - d) / N * 1.
- Degree & betweenness centrality: C_B(v) = sum_{s != v != t} (sigma_{st}(v) / sigma_{st}).
- Small-world Watts-Strogatz: high clustering coefficient C with small characteristic path length L.
"#,
    )
}

/// 62. math-convex-optimization Skill
pub fn math_convex_optimization() -> EccSkill {
    EccSkill::new(
        "math-convex-optimization",
        "Convex optimization, convex sets and functions, Karush-Kuhn-Tucker (KKT) optimality conditions, Lagrangian duality, and model latency/cost allocation based on Stephen Boyd and Lieven Vandenberghe. Triggers: convex-optimization, kkt-conditions, lagrangian-duality, convex-sets, slaters-condition, quadratic-programming, dual-problem.",
        r#"# Convex Optimization: Duality & KKT Conditions
- Convex problem: min f_0(x) s.t. f_i(x) <= 0, h_j(x) = 0.
- KKT conditions: Stationarity (grad f_0 + sum lambda_i grad f_i + sum nu_j grad h_j = 0), Primal feasibility, Dual feasibility (lambda_i >= 0), Complementary slackness (lambda_i * f_i(x) = 0).
"#,
    )
}

/// 63. math-probability-logic Skill
pub fn math_probability_logic() -> EccSkill {
    EccSkill::new(
        "math-probability-logic",
        "Probability theory as extended formal logic under incomplete information, Cox's consistency theorems, and the Principle of Maximum Entropy (MaxEnt) based on E.T. Jaynes. Triggers: probability-as-logic, cox-theorems, maximum-entropy, maxent, epistemic-probability, bayesian-logic, inductive-reasoning, uninformative-priors.",
        r#"# Probability Theory: The Logic of Science
- Cox's consistency axioms: transitiveness of plausibility, complementary negations, structural uniqueness.
- Maximum Entropy Principle: max H(p) = -sum p_i ln p_i subject to expectation constraints sum p_i f_k(x_i) = <f_k>.
- Grounding uncertainty: reject false certainty without sufficient observational evidence.
"#,
    )
}

/// 64. math-programmers-discrete Skill
pub fn math_programmers_discrete() -> EccSkill {
    EccSkill::new(
        "math-programmers-discrete",
        "Programmer-focused discrete mathematics, Fast Fourier Transform (FFT) for motion/audio signals, Graph Laplacian, and spectral bisection based on Jeremy Kun. Triggers: programmers-discrete-math, fast-fourier-transform, fft, graph-laplacian, spectral-bisection, polynomial-rings, fourier-analysis, signal-processing.",
        r#"# A Programmer's Introduction to Mathematics: FFT & Graph Laplacians
- Discrete Fourier Transform: X_k = sum_{n=0}^{N-1} x_n * exp(-i * 2*pi * k * n / N).
- Cooley-Tukey FFT: divide-and-conquer O(N log N) frequency transformation.
- Graph Laplacian: L = D - A; Fiedler vector (2nd smallest eigenvalue) partitions graph minimally.
"#,
    )
}

/// 65. math-intuitive-calculus Skill
pub fn math_intuitive_calculus() -> EccSkill {
    EccSkill::new(
        "math-intuitive-calculus",
        "Intuitive differential and integral calculus, instantaneous velocity, exponential decay, accumulation beneath curves, and PID controllers with anti-windup clamping based on 'Calculus Made Easy' by Thompson and Gardner. Triggers: intuitive-calculus, rates-of-change, velocity-accumulation, exponential-decay, pid-controller, anti-windup, numerical-integration.",
        r#"# Calculus Made Easy: Intuitive Differentials & PID Dynamics
- Infinitesimals: dy/dx represents instantaneous rate of change.
- Chain rule: dy/dx = (dy/du) * (du/dx).
- PID controller with anti-windup: u(t) = K_p * e(t) + K_i * int_0^t e(tau) dtau + K_d * de/dt clamped to [u_min, u_max].
"#,
    )
}

/// 66. math-concrete-discrete Skill
pub fn math_concrete_discrete() -> EccSkill {
    EccSkill::new(
        "math-concrete-discrete",
        "Concrete discrete mathematics, recurrence relations, generating functions, discrete sums, binomial coefficients, and asymptotic bounds for agent token recursion based on Graham, Knuth, and Patashnik. Triggers: concrete-mathematics, recurrence-relations, generating-functions, discrete-sums, binomial-coefficients, knuth-math, asymptotic-analysis, token-budget-bound.",
        r#"# Concrete Mathematics: Discrete Recurrences & Asymptotics
- Recurrence solution via master theorem: T(n) = a * T(n/b) + f(n).
- Ordinary generating functions: G(z) = sum_{n=0}^infty a_n * z^n.
- Bounding recursive agent swarms: ensure branching factor b and depth d satisfy sum b^k <= Token_Budget.
"#,
    )
}

/// 67. math-visual-topology Skill
pub fn math_visual_topology() -> EccSkill {
    EccSkill::new(
        "math-visual-topology",
        "Visual geometry, Gaussian curvature, Euler characteristic, minimal surfaces, manifold topology, and 2-manifold mesh integrity validation based on David Hilbert and Stephan Cohn-Vossen. Triggers: visual-geometry, gaussian-curvature, euler-characteristic, manifold-topology, minimal-surfaces, mesh-validation, hilbert-geometry.",
        r#"# Geometry and the Imagination: Curvature & Manifolds
- Gaussian curvature: K = kappa_1 * kappa_2; Theorema Egregium (intrinsic invariance).
- Euler-Poincare characteristic: chi = V - E + F = 2 - 2g for closed surface of genus g.
- 2-manifold invariant: each edge shared by exactly two faces; vertex star homeomorphic to disk.
"#,
    )
}

/// 68. math-chaos-fractals Skill
pub fn math_chaos_fractals() -> EccSkill {
    EccSkill::new(
        "math-chaos-fractals",
        "Fractal geometry, scale invariance, power laws, Hausdorff dimension, Lorenz strange attractors, and deterministic chaos simulations based on Manfred Schroeder. Triggers: chaos-theory, fractals, strange-attractor, lorenz-attractor, power-laws, lyapunov-exponent, mandelbrot, scale-invariance.",
        r#"# Fractals, Chaos & Power Laws
- Hausdorff dimension: D = lim_{eps -> 0} (log N(eps) / log(1/eps)).
- Lorenz chaotic system: dx/dt = sigma*(y - x), dy/dt = x*(rho - z) - y, dz/dt = x*y - beta*z.
- Lyapunov sensitivity: delta x(t) approx delta x_0 * exp(lambda * t).
"#,
    )
}

/// 69. math-numerical-methods Skill
pub fn math_numerical_methods() -> EccSkill {
    EccSkill::new(
        "math-numerical-methods",
        "Scientific numerical computing, IEEE-754 floating-point stability, Runge-Kutta 4th Order (RK4) ODE solvers, Symplectic Velocity Verlet, and spline smoothing based on 'Numerical Recipes'. Triggers: numerical-methods, rk4, runge-kutta, floating-point-stability, velocity-verlet, cubic-spline, numerical-recipes, nan-prevention.",
        r#"# Numerical Recipes: RK4 & Floating-Point Stability
- Runge-Kutta 4th order: y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4).
- Symplectic Velocity Verlet: x(t+dt) = x(t) + v(t)*dt + 1/2*a(t)*dt^2; v(t+dt) = v(t) + 1/2*(a(t) + a(t+dt))*dt.
- Numerical safety: guard against catastrophic cancellation: sqrt(x + eps) - sqrt(x) = eps / (sqrt(x+eps) + sqrt(x)).
"#,
    )
}

/// 70. math-algorithmic-game-theory Skill
pub fn math_algorithmic_game_theory() -> EccSkill {
    EccSkill::new(
        "math-algorithmic-game-theory",
        "Algorithmic game theory, Nash equilibria, mechanism design, Vickrey-Clarke-Groves (VCG) truthful auctions, Price of Anarchy (PoA), and Shapley value attribution based on Nisan, Roughgarden, Tardos, and Vazirani. Triggers: algorithmic-game-theory, nash-equilibrium, mechanism-design, vcg-auction, price-of-anarchy, shapley-value, multi-agent-incentives, cooperative-games.",
        r#"# Algorithmic Game Theory: Multi-Agent Equilibrium & Incentives
- Nash equilibrium: no agent has incentive to unilaterally deviate u_i(s_i^*, s_{-i}^*) >= u_i(s_i, s_{-i}^*).
- Price of Anarchy: PoA = (Worst Nash Social Welfare) / (Optimal Social Welfare).
- Shapley value credit attribution: phi_i(v) = sum_{S subseteq N \ {i}} (|S|! * (|N| - |S| - 1)! / |N|!) * (v(S union {i}) - v(S)).
"#,
    )
}

/// 71. erp-silverston-enterprise-patterns Skill
pub fn erp_silverston_enterprise_patterns() -> EccSkill {
    EccSkill::new(
        "erp-silverston-enterprise-patterns",
        "Universal enterprise data patterns for Party, Role, Relationship, Product, Order, Shipment, Work Effort, and Financial Account hierarchies based on Len Silverston's canonical models. Triggers: silverston-patterns, enterprise-data-model, party-role-relationship, universal-data-model, product-hierarchy, order-shipment-pattern, enterprise-party-model, canonical-erp-schema.",
        r#"# Canonical Enterprise Data Modeling: Universal Party, Product, Order & Account Patterns
> Based on **The Data Model Resource Book, Vol 1 & 2 - Len Silverston**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Order Line & Aggregation Invariants
For any order $O$ with line items $i \in \{1, \dots, n\}$:
$$\text{LineTotal}_i = Q_i \times P_i - D_i$$
$$\text{Subtotal}(O) = \sum_{i=1}^n \text{LineTotal}_i$$
$$\text{TotalAmount}(O) = \text{Subtotal}(O) + \text{TaxAmount}(O) + \text{ShippingFee}(O)$$

### 2.2 Shipment Quantity Conservation
Let $Q_i^{\text{ordered}}$ be the ordered quantity of line item $i$. The cumulative shipped quantity across all shipments $S_k$ must satisfy:
$$\sum_{k} Q_{i, k}^{\text{shipped}} \le Q_i^{\text{ordered}}$$
If $\sum_k Q_{i, k}^{\text{shipped}} = Q_i^{\text{ordered}}$ for all $i$, the order transitions to `COMPLETED`.

### 2.3 Party Hierarchy Directed Acyclic Graph (DAG) Invariant
Let $G = (V, E)$ be the graph formed by party relationships where $V$ are `parties` and $E$ are `SUBSIDIARY_OF` relationships.
$$\forall v \in V, \quad v \notin \text{Ancestors}(v) \iff \text{Cycles}(G) = \emptyset$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Order Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> DRAFT
    DRAFT --> PLACED: place_order()
    DRAFT --> CANCELLED: cancel()
    PLACED --> APPROVED: approve_credit()
    PLACED --> CANCELLED: reject()
    APPROVED --> PARTIALLY_SHIPPED: dispatch_first_shipment()
    APPROVED --> COMPLETED: dispatch_full_shipment()
    PARTIALLY_SHIPPED --> COMPLETED: dispatch_final_shipment()
    PARTIALLY_SHIPPED --> CANCELLED: cancel_remaining()
    COMPLETED --> [*]
    CANCELLED --> [*]
```

### 3.2 Invariant Enforcement Rules
- **Rule 1**: A `CANCELLED` order cannot receive shipments or accept further modifications.
- **Rule 2**: Transition from `DRAFT` to `PLACED` requires at least 1 line item with positive quantity.
- **Rule 3**: `COMPLETED` is an immutable terminal state.

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce Party-Role-Relationship decoupling: Never put 'is_customer' boolean on party table; use party_roles with date ranges.
- Validate Order Line Total invariant: line_total = quantity * unit_price - discount.
- Enforce cumulative shipment check: sum(shipped_quantity) <= ordered_quantity.
- Prevent cyclical party hierarchies with recursive CTE validation before insert.
- Ensure terminal states (COMPLETED, CANCELLED) reject any subsequent update mutations.
```
"#,
    )
}

/// 72. erp-silverston-industry-patterns Skill
pub fn erp_silverston_industry_patterns() -> EccSkill {
    EccSkill::new(
        "erp-silverston-industry-patterns",
        "Specialized enterprise data models across Manufacturing, Telecommunications, Healthcare, Financial Services, and Professional Services based on Len Silverston and Paul Agnew's industry blueprints. Triggers: industry-data-models, silverston-industry, telecom-cdr-model, healthcare-clinical-encounter, financial-services-deposit-loan, professional-services-timesheet, specialized-erp-patterns, vertical-erp-schemas.",
        r#"# Industry-Specific Enterprise Data Models: Manufacturing, Telecom, Healthcare & Financial Services
> Based on **The Data Model Resource Book, Vol 3 - Len Silverston & Paul Agnew**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Loan Principal Amortization
For a loan with principal $P$, monthly interest rate $r = \frac{R_{\text{annual}}}{12}$, and term $n$ months:
$$\text{Monthly Payment } M = P \cdot \frac{r(1+r)^n}{(1+r)^n - 1}$$
At month $k$:
$$\text{Interest Payment } I_k = B_{k-1} \cdot r$$
$$\text{Principal Payment } P_k = M - I_k$$
$$\text{Remaining Balance } B_k = B_{k-1} - P_k$$

### 2.2 Telecom Usage Rating Invariant
For a CDR with duration $t$ seconds and rating increment $\Delta t = 60\text{s}$ with rate $R_{\text{minute}}$:
$$\text{Billable Minutes} = \left\lceil \frac{t}{60} \right\rceil$$
$$\text{Rated Amount} = \text{Billable Minutes} \times R_{\text{minute}}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Loan Account Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> APPLICATION
    APPLICATION --> UNDERWRITING: submit_documents()
    APPLICATION --> CANCELLED: withdraw()
    UNDERWRITING --> APPROVED: credit_score_pass()
    UNDERWRITING --> DEFAULTED: reject()
    APPROVED --> DISBURSED: wire_funds()
    DISBURSED --> ACTIVE: first_payment_due()
    ACTIVE --> PAID_OFF: balance_zero()
    ACTIVE --> DEFAULTED: dunning_exceeded()
    PAID_OFF --> [*]
    DEFAULTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never calculate billing amounts client-side; use STORED generated columns or database triggers.
- Enforce positive CDR duration and data bytes (duration >= 0, data_bytes >= 0).
- Loan balance invariant: balance_k = balance_{k-1} - principal_repayment.
- Guard against zero-rate division in amortization equations.
- Healthcare encounter admit_time must precede or equal discharge_time.
```
"#,
    )
}

/// 73. erp-fowler-analysis-patterns Skill
pub fn erp_fowler_analysis_patterns() -> EccSkill {
    EccSkill::new(
        "erp-fowler-analysis-patterns",
        "Enterprise structural patterns for Accountability graphs, Observation and Measurement protocols, Tiered Pricing, and Accounting Execution Posting Rules based on Martin Fowler's Analysis Patterns. Triggers: fowler-analysis-patterns, accountability-pattern, observation-measurement, tiered-pricing-pattern, posting-rules, enterprise-object-models, operational-vs-knowledge-level, fowler-accounting.",
        r#"# Fowler Analysis Patterns: Accountability, Observation, Tiered Pricing & Accounting Rules
> Based on **Analysis Patterns: Reusable Object Models - Martin Fowler**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Unit of Measure Conversion Invariant
Let $Q_A$ be a quantity in unit $A$ and $Q_B$ in unit $B$ with conversion factors $C_A, C_B$ to the canonical dimension base unit:
$$Q_{\text{base}} = Q_A \times C_A$$
$$Q_B = \frac{Q_{\text{base}}}{C_B} = Q_A \times \frac{C_A}{C_B}$$

### 2.2 Tiered Pricing Function
For an order quantity $q$, pricing under tiered bands $[L_k, U_k)$ with rates $P_k$:
$$\text{TotalPrice}(q) = \sum_{k=1}^m \max(0, \min(q, U_k) - L_k) \times P_k$$
Ensuring monotonicity: $q_1 < q_2 \implies \text{TotalPrice}(q_1) \le \text{TotalPrice}(q_2)$.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Posting Rule Execution FSM
```mermaid
stateDiagram-v2
    [*] --> PENDING_EVALUATION
    PENDING_EVALUATION --> MATCHED: evaluate_event()
    PENDING_EVALUATION --> UNMATCHED: no_applicable_rule()
    MATCHED --> POSTED: generate_balanced_journal_entry()
    MATCHED --> REJECTED: balance_mismatch()
    POSTED --> [*]
    UNMATCHED --> ESCALATED: notify_accountant()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Differentiate Knowledge Level (AccountabilityType, PhenomenonType) from Operational Level (Accountability, Observation).
- Never mix dimension types in conversions (e.g. converting MASS to TIME must fail).
- Verify tiered pricing monotonicity: higher volume must not produce lower total invoice amount.
- Ensure all posting rules generate debits strictly equal to credits.
```
"#,
    )
}

/// 74. erp-evans-ddd-core Skill
pub fn erp_evans_ddd_core() -> EccSkill {
    EccSkill::new(
        "erp-evans-ddd-core",
        "Tactical and strategic Domain-Driven Design for enterprise systems, establishing Ubiquitous Language, Bounded Contexts, Aggregate Roots, Invariant Boundaries, and Anti-Corruption Layers (ACL) based on Eric Evans. Triggers: evans-ddd, domain-driven-design, bounded-context, aggregate-root, ubiquitous-language, anti-corruption-layer, context-mapping, domain-events-core, ddd-invariants.",
        r#"# Domain-Driven Design in ERP: Ubiquitous Language, Aggregates & Anti-Corruption Layers
> Based on **Domain-Driven Design: Tackling Complexity in the Heart of Software - Eric Evans**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Aggregate Invariant Preservation
Let an Aggregate Root state be $S$, satisfying domain invariant predicate $\Phi(S) = \text{true}$.
A command $C$ produces state $S'$ and domain events $E$:
$$(S', E) = f(S, C)$$
$$\Phi(S') = \text{true} \quad \forall C \in \text{ValidCommands}$$
If $\Phi(S') = \text{false}$, command $C$ is rejected and transaction rolls back.

### 2.2 Optimistic Versioning Invariant
Let $V_t$ be the aggregate version at time of read:
$$\text{UPDATE} \iff V_{\text{current}} = V_t \implies V_{\text{new}} = V_t + 1$$
If $V_{\text{current}} \ne V_t$, throw `ConcurrencyException` to guarantee serializable consistency.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Aggregate Lifecycle State Machine
```mermaid
stateDiagram-v2
    [*] --> INITIALIZING
    INITIALIZING --> ACTIVE: apply(CreatedEvent)
    ACTIVE --> MODIFIED: apply(UpdatedEvent)
    MODIFIED --> ACTIVE: commit_transaction()
    ACTIVE --> CLOSED: apply(ArchivedEvent)
    CLOSED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never reference entities across Aggregate Root boundaries by object reference; use IDs only.
- Ensure transactions update only a single Aggregate Root per request.
- Use Outbox Pattern table within the same database transaction to publish Domain Events.
- Prevent domain logic contamination from external legacy schemas via an Anti-Corruption Layer (ACL).
- Invariants must be enforced inside the Aggregate Root boundary, never in external UI controllers.
```
"#,
    )
}

/// 75. erp-vernon-iddd-enterprise Skill
pub fn erp_vernon_iddd_enterprise() -> EccSkill {
    EccSkill::new(
        "erp-vernon-iddd-enterprise",
        "Production implementation of Domain-Driven Design, covering Event Sourcing append-only streams, CQRS read/write projections, Saga orchestration, and idempotent messaging based on Vaughn Vernon. Triggers: vernon-iddd, event-sourcing-core, cqrs-architecture, saga-orchestrator, aggregate-invariants, event-store-schema, idempotent-domain-events, enterprise-ddd-implementation.",
        r#"# Implementing DDD: Event Sourcing, Sagas & CQRS Enterprise Patterns
> Based on **Implementing Domain-Driven Design - Vaughn Vernon**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Event Fold Invariant
An aggregate state at version $n$ is a deterministic left-fold over its historic event stream:
$$S_n = \text{foldl}(\text{apply}, S_0, [e_1, e_2, \dots, e_n])$$
Given snapshot $S_k$ at version $k < n$:
$$S_n = \text{foldl}(\text{apply}, S_k, [e_{k+1}, \dots, e_n])$$

### 2.2 Strict Monotonicity of Event Stream
$$\text{Version}(e_{i}) = \text{Version}(e_{i-1}) + 1$$
Any gap or duplicate aborts append with `ConcurrencyConflictException`.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Saga Orchestration State Machine
```mermaid
stateDiagram-v2
    [*] --> STARTED
    STARTED --> PAYMENT_RESERVED: process_payment()
    PAYMENT_RESERVED --> INVENTORY_ALLOCATED: allocate_stock()
    PAYMENT_RESERVED --> PAYMENT_FAILED: insufficient_funds()
    PAYMENT_FAILED --> COMPENSATING_PAYMENT: refund()
    INVENTORY_ALLOCATED --> ORDER_COMPLETED: dispatch()
    INVENTORY_ALLOCATED --> COMPENSATING_INVENTORY: stockout()
    COMPENSATING_INVENTORY --> COMPENSATING_PAYMENT: rollback_stock()
    COMPENSATING_PAYMENT --> SAGA_ABORTED: refund_finished()
    ORDER_COMPLETED --> [*]
    SAGA_ABORTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never execute UPDATE or DELETE statements against the event_store; it is strictly append-only.
- Verify unique constraint on (stream_id, stream_version) to prevent concurrent write collisions.
- Reconstruct aggregate state using left-fold over ordered events.
- Compensating transactions in Sagas must be strictly idempotent.
```
"#,
    )
}

/// 76. erp-scheer-aris-architecture Skill
pub fn erp_scheer_aris_architecture() -> EccSkill {
    EccSkill::new(
        "erp-scheer-aris-architecture",
        "Enterprise business process engineering using Scheer's ARIS framework, Event-Driven Process Chains (EPC), Function-Data-Organization alignment, and process-to-data integration. Triggers: scheer-aris, aris-architecture, event-driven-process-chains, epc-workflows, business-process-engineering, aris-house, process-to-data, sap-reference-model.",
        r#"# ARIS Architecture of Integrated Information Systems: EPC Workflows & Process-to-Data Alignment
> Based on **ARIS - Business Process Modeling - August-Wilhelm Scheer**
## 2. Mathematical Foundations & Business Invariants

### 2.1 EPC Grammar & Syntax Invariants
Let an EPC graph be $G = (V, E)$ where $V = V_{\text{Event}} \cup V_{\text{Function}} \cup V_{\text{Connector}}$:
1. **Alternation Principle**: An Event must not be immediately followed by another Event:
   $$(u, v) \in E \land u \in V_{\text{Event}} \implies v \notin V_{\text{Event}}$$
2. **Decision Authority**: An Event cannot make decisions; XOR/OR connectors cannot immediately follow an Event:
   $$(u, v) \in E \land u \in V_{\text{Event}} \implies v \notin \{ \text{XOR\_SPLIT}, \text{OR\_SPLIT} \}$$
   Only a `FUNCTION` possesses organizational agency to route decisions.

### 2.2 Token Game Net Soundness
A workflow net is sound iff:
- **Option to Complete**: $\forall M \in [M_0\rangle, \exists M' \in [M\rangle \text{ s.t. } M' \ge M_{\text{final}}$.
- **Proper Completion**: $M \ge M_{\text{final}} \implies M = M_{\text{final}}$.
- **No Dead Transitions**: $\forall t \in T, \exists M \in [M_0\rangle \text{ s.t. } M \xrightarrow{t}$.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 EPC Process Execution Flow
```mermaid
graph TD
    E1([Event: Customer Order Received]) --> F1[Function: Check Credit Limit]
    F1 --> C1{XOR Split}
    C1 -->|Credit OK| E2([Event: Credit Approved])
    C1 -->|Credit Bad| E3([Event: Credit Denied])
    E2 --> F2[Function: Release Order to Warehouse]
    E3 --> F3[Function: Notify Account Manager]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce Scheer's EPC grammar: Never allow an Event to directly connect to an Event.
- Prevent XOR splits immediately after an Event; decisions require a Function.
- Map every EPC Function to responsible Organizational Unit and CRUD Data Objects.
- Enforce Petri net soundness (no deadlocks, no dangling execution tokens).
```
"#,
    )
}

/// 77. erp-double-entry-ledger Skill
pub fn erp_double_entry_ledger() -> EccSkill {
    EccSkill::new(
        "erp-double-entry-ledger",
        "Fundamental double-entry accounting mechanics, Chart of Accounts, General Ledger posting, Trial Balance generation, and the fundamental invariant sum(debits) == sum(credits) based on Mike Piper. Triggers: double-entry-ledger, general-ledger, accounting-equation, debit-credit-invariant, chart-of-accounts, trial-balance, journal-entry, accounting-made-simple, debits-equal-credits.",
        r#"# Double-Entry Bookkeeping & General Ledger: Mathematical Accounting Invariants
> Based on **Accounting Made Simple - Mike Piper**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Fundamental Accounting Invariant
The foundational equation of double-entry bookkeeping:
$$\text{Assets} = \text{Liabilities} + \text{Equity}$$
Expanded with nominal accounts (income statement):
$$\text{Assets} = \text{Liabilities} + \text{Equity} + (\text{Revenue} - \text{Expenses}) - \text{Dividends}$$
Rearranging into pure debit/credit parity:
$$\underbrace{\text{Assets} + \text{Expenses} + \text{Dividends}}_{\text{Normal Debit Balance}} = \underbrace{\text{Liabilities} + \text{Equity} + \text{Revenue}}_{\text{Normal Credit Balance}}$$

### 2.2 Strict Balance Invariant
For every posted journal entry $J$:
$$\sum_{l \in \text{Lines}(J)} \text{debit}_l - \sum_{l \in \text{Lines}(J)} \text{credit}_l = 0.0000$$
Any journal entry where $|\sum \text{debit} - \sum \text{credit}| > 10^{-4}$ MUST be rejected.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Journal Entry State Lifecycle
```mermaid
stateDiagram-v2
    [*] --> DRAFT
    DRAFT --> POSTED: post_entry() [assert debits == credits]
    DRAFT --> VOIDED: void()
    POSTED --> REVERSED: reverse_entry() [generates inverse entry]
    POSTED --> [*]
    REVERSED --> [*]
    VOIDED --> [*]
```
- **Invariant**: Once `POSTED`, a journal entry CANNOT be edited or deleted. Correction requires generating a compensating `REVERSED` entry.

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce strict debit == credit invariant before saving any journal entry.
- Disallow single-line entries: minimum 2 lines required per journal voucher.
- Never update or delete a POSTED entry; issue a reversing debit/credit voucher instead.
- Require positive amounts: debit >= 0 and credit >= 0, mutually exclusive per line.
```
"#,
    )
}

/// 78. erp-multi-currency-fx Skill
pub fn erp_multi_currency_fx() -> EccSkill {
    EccSkill::new(
        "erp-multi-currency-fx",
        "Multi-currency ERP mechanics, base vs functional vs transaction currencies, Realized FX gains/losses upon settlement, Unrealized FX balance sheet revaluations under IAS 21 / ASC 830. Triggers: multi-currency-fx, foreign-exchange-accounting, realized-gain-loss, unrealized-fx-revaluation, ias21-fx, asc830-currency, fx-triangulation, functional-currency.",
        r#"# Multi-Currency Accounting: Functional Currency, Triangulation & FX Revaluation
> Based on **Financial Accounting: An Integrated Approach - Kenneth Trotman**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Currency Triangulation Invariant
If a direct exchange rate between currency $A$ and currency $B$ is not published, it must be triangulated via base currency $C$:
$$R_{A \to B} = \frac{R_{A \to C}}{R_{B \to C}}$$

### 2.2 Realized FX Gain/Loss Calculation
When an invoice booked at rate $R_{\text{orig}}$ is settled at payment rate $R_{\text{settle}}$:
$$\text{Base Amount}_{\text{orig}} = \text{Amount}_{\text{txn}} \times R_{\text{orig}}$$
$$\text{Base Amount}_{\text{settle}} = \text{Amount}_{\text{txn}} \times R_{\text{settle}}$$
$$\text{Realized FX Gain/Loss} = \text{Base Amount}_{\text{settle}} - \text{Base Amount}_{\text{orig}}$$
For AR: positive is Gain, negative is Loss. For AP: positive is Loss, negative is Gain.

### 2.3 Unrealized FX Balance Sheet Revaluation (IAS 21 / ASC 830)
At accounting period close date $T$:
$$\text{Unrealized Gain/Loss} = B_{\text{foreign}} \times R_{\text{closing}} - B_{\text{book, base}}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 FX Month-End Revaluation FSM
```mermaid
stateDiagram-v2
    [*] --> DRAFT_PERIOD
    DRAFT_PERIOD --> RATES_FROZEN: lock_closing_rates()
    RATES_FROZEN --> COMPUTING: calculate_unrealized_diffs()
    COMPUTING --> VOUCHER_POSTED: post_revaluation_journal()
    VOUCHER_POSTED --> REVERSED_NEXT_PERIOD: auto_reverse_day1()
    REVERSED_NEXT_PERIOD --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never mix transaction currency amounts with base currency amounts in the same column.
- Always record exchange rate alongside both foreign amount and converted base amount.
- Auto-reverse month-end unrealized FX revaluation entries on day 1 of the subsequent period.
- Handle triangular cross-rate calculations cleanly via base currency anchor.
```
"#,
    )
}

/// 79. erp-asc606-revenue-recognition Skill
pub fn erp_asc606_revenue_recognition() -> EccSkill {
    EccSkill::new(
        "erp-asc606-revenue-recognition",
        "5-Step Revenue Recognition model under ASC 606 / IFRS 15, Standalone Selling Price (SSP) allocation, performance obligations (POBs), deferred revenue waterfalls, and contract asset/liability management. Triggers: asc606-revenue-recognition, ifrs15-revenue, 5-step-revenue-model, performance-obligation, ssp-allocation, deferred-revenue-schedule, contract-asset-liability, unearned-revenue-waterfall.",
        r#"# ASC 606 / IFRS 15 Revenue Recognition: 5-Step Model, SSP Allocation & Deferred Schedules
> Based on **Revenue Recognition: ASC 606 / IFRS 15 - AICPA / Frank Sellitti**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Relative Standalone Selling Price (SSP) Allocation Invariant
For contract $C$ with total transaction price $T$ and $m$ distinct performance obligations:
$$\text{Allocation Factor}_k = \frac{\text{SSP}_k}{\sum_{j=1}^m \text{SSP}_j}$$
$$\text{Allocated Price}_k = T \times \text{Allocation Factor}_k$$
$$\text{Conservation Invariant: } \sum_{k=1}^m \text{Allocated Price}_k = T$$

### 2.2 Balance Sheet Revenue Identity
At all times across all periods $t$:
$$\text{Billed To Date} = \text{Cumulative Recognized Revenue} + \text{Deferred Revenue Balance}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Performance Obligation Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> UNSATISFIED
    UNSATISFIED --> PARTIALLY_SATISFIED: milestone_completed()
    UNSATISFIED --> FULLY_SATISFIED: goods_delivered() [Point-in-Time]
    PARTIALLY_SATISFIED --> FULLY_SATISFIED: final_period_amortized()
    FULLY_SATISFIED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Implement ASC 606 5-Step process: Contract -> Obligations -> Price -> Allocation -> Recognition.
- Always plug rounding discrepancies on the final obligation to ensure sum(allocations) == total_price.
- Deferred Revenue must be treated as a Balance Sheet Liability until performance obligations are satisfied.
- Point-in-time revenue recognized upon delivery; Over-time revenue recognized via straight-line or milestone completion.
```
"#,
    )
}

/// 80. erp-cost-accounting-management Skill
pub fn erp_cost_accounting_management() -> EccSkill {
    EccSkill::new(
        "erp-cost-accounting-management",
        "Standard costing systems, Manufacturing Overhead (MOH) absorption, Activity-Based Costing (ABC) pools/drivers, and comprehensive variance analysis (Price, Efficiency, Volume, Spending) based on Horngren. Triggers: cost-accounting-management, standard-costing, variance-analysis, activity-based-costing, direct-labor-variance, material-price-variance, overhead-absorption, cost-pools-drivers.",
        r#"# Cost Accounting & Managerial Control: Standard Costing, ABC & Variance Analysis
> Based on **Cost Accounting: A Managerial Emphasis - Horngren, Datar, Rajan**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Standard Cost Variance Formulas
1. **Direct Material Price Variance (MPV)**:
   $$\text{MPV} = (\text{Actual Price} - \text{Standard Price}) \times \text{Actual Quantity Purchased}$$
   $(\text{AP} > \text{SP} \implies \text{Unfavorable}; \text{AP} < \text{SP} \implies \text{Favorable})$

2. **Direct Material Efficiency/Quantity Variance (MQV)**:
   $$\text{MQV} = (\text{Actual Quantity Used} - \text{Standard Quantity Allowed}) \times \text{Standard Price}$$

3. **Direct Labor Rate Variance (LRV)**:
   $$\text{LRV} = (\text{Actual Wage Rate} - \text{Standard Wage Rate}) \times \text{Actual Hours Worked}$$

4. **Direct Labor Efficiency Variance (LEV)**:
   $$\text{LEV} = (\text{Actual Hours Worked} - \text{Standard Hours Allowed}) \times \text{Standard Wage Rate}$$

### 2.2 Total Cost Reconciliation
$$\text{Total Actual Cost} = \text{Standard Cost of Output} + \sum \text{Unfavorable Variances} - \sum \text{Favorable Variances}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Cost Accounting Period-End FSM
```mermaid
stateDiagram-v2
    [*] --> OPEN
    OPEN --> ACCUMULATING_ACTUALS: end_of_month()
    ACCUMULATING_ACTUALS --> VARIANCE_COMPUTED: run_variance_engine()
    VARIANCE_COMPUTED --> VARIANCE_PRORATED: write_to_cogs_and_inventory()
    VARIANCE_PRORATED --> CLOSED: lock_period()
    CLOSED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Differentiate favorable (credit balance / negative cost) from unfavorable (debit balance / positive cost) variances.
- Use predetermined overhead rates: PredeterminedRate = BudgetedCost / BudgetedDriverUnits.
- Material price variance is recognized upon purchase; material quantity variance is recognized upon consumption.
- Variance accounts must be cleared at period-end by disposition to COGS and Ending WIP/Inventory.
```
"#,
    )
}

/// 81. erp-double-entry-history-auditing Skill
pub fn erp_double_entry_history_auditing() -> EccSkill {
    EccSkill::new(
        "erp-double-entry-history-auditing",
        "Historical principles of Venetian double-entry bookkeeping, modern continuous auditing trails, tamper-evident cryptographic hash chains (Merkle/Blake3), and strict zero-edit ledger immutability based on Jane Gleeson-White. Triggers: double-entry-history-auditing, venetian-bookkeeping, immutable-ledger-audit, tamper-evident-chain, pacioli-principles, hash-chained-journals, continuous-audit-trail, zero-edit-ledger.",
        r#"# Continuous Ledger Auditing: Tamper-Evident Hash Chains & Pacioli Audit Discipline
> Based on **Double Entry: How the Merchants of Venice Created Modern Capitalism - Jane Gleeson-White**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Cryptographic Hash Chain Invariant
For entry sequence $n$:
$$H_0 = \text{GenesisHash} = \text{SHA256}(\text{"GENESIS\_LEDGER\_START"})$$
$$H_n = \text{SHA256}(H_{n-1} \parallel \text{Seq}_n \parallel \text{Timestamp}_n \parallel \text{CanonicalJSON}_n)$$

### 2.2 Immutability Proof
If any historical record $k < n$ is altered ($P_k \to P'_k$):
$$H_k' \ne H_k \implies H_{k+1}' \ne H_{k+1} \implies \dots \implies H_n' \ne H_n$$
Any single-bit perturbation invalidates the entire subsequent chain.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Audit Verification State Machine
```mermaid
stateDiagram-v2
    [*] --> CONTINUOUS_VERIFICATION
    CONTINUOUS_VERIFICATION --> VERIFIED: all_hashes_match()
    CONTINUOUS_VERIFICATION --> TAMPER_DETECTED: hash_mismatch()
    TAMPER_DETECTED --> SYSTEM_QUARANTINE: alert_cfo_and_auditor()
    SYSTEM_QUARANTINE --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never allow SQL UPDATE or DELETE on ledger tables; enforce append-only policies.
- Cryptographically chain each journal entry to its immediate predecessor via SHA-256 / Blake3.
- Discard floating-point representations when hashing: use canonical integer or decimal strings.
- Voided entries must be appended as explicitly signed reversing entries, never deleted.
```
"#,
    )
}

/// 82. erp-balance-sheet-working-capital Skill
pub fn erp_balance_sheet_working_capital() -> EccSkill {
    EccSkill::new(
        "erp-balance-sheet-working-capital",
        "Financial statement analysis, Working Capital management, Cash Conversion Cycle (CCC = DSO + DIO - DPO), liquidity ratios (Current, Quick, Cash), and Free Cash Flow modeling based on Stephen Penman. Triggers: balance-sheet-working-capital, cash-conversion-cycle, working-capital-optimization, days-sales-outstanding, dso-dio-dpo, liquidity-ratios, free-cash-flow-model, penman-financial-analysis.",
        r#"# Working Capital Optimization: Cash Conversion Cycle, Current Ratio & Liquidity Analysis
> Based on **Financial Statement Analysis and Security Valuation - Stephen Penman**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Cash Conversion Cycle (CCC) Formulation
The Cash Conversion Cycle measures the time (in days) required to convert resource inputs into cash flows:
$$\text{CCC} = \text{DSO} + \text{DIO} - \text{DPO}$$
Where:
1. **Days Sales Outstanding (DSO)**:
   $$\text{DSO} = \frac{\text{Accounts Receivable}}{\text{Total Credit Sales}} \times 365$$
2. **Days Inventory Outstanding (DIO)**:
   $$\text{DIO} = \frac{\text{Average Inventory}}{\text{Cost of Goods Sold (COGS)}} \times 365$$
3. **Days Payable Outstanding (DPO)**:
   $$\text{DPO} = \frac{\text{Accounts Payable}}{\text{Cost of Goods Sold / Purchases}} \times 365$$

### 2.2 Liquidity Ratios
$$\text{Current Ratio} = \frac{\text{Current Assets}}{\text{Current Liabilities}}$$
$$\text{Quick Ratio} = \frac{\text{Cash} + \text{Marketable Securities} + \text{Accounts Receivable}}{\text{Current Liabilities}}$$
$$\text{Cash Ratio} = \frac{\text{Cash and Equivalents}}{\text{Current Liabilities}}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Financial Statement Analysis Workflow
```mermaid
graph TD
    A[Subledger Balances Locked] --> B[Generate Trial Balance]
    B --> C[Compute Balance Sheet & Income Statement]
    C --> D[Calculate Working Capital & Liquidity Ratios]
    D --> E{Current Ratio < 1.0 or CCC Spike?}
    E -->|Yes| F[Trigger Working Capital Early Warning]
    E -->|No| G[Approve Period Financial Report]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce Cash Conversion Cycle identity: CCC = DSO + DIO - DPO.
- Net Working Capital = Current Assets - Current Liabilities.
- Alert when Current Ratio drops below 1.2 or CCC expands by > 15 days in consecutive quarters.
- Ensure denominator checks guard against zero-revenue or zero-cogs division.
```
"#,
    )
}

/// 83. erp-intercompany-consolidation Skill
pub fn erp_intercompany_consolidation() -> EccSkill {
    EccSkill::new(
        "erp-intercompany-consolidation",
        "Multi-entity enterprise consolidation, Intercompany (IC) transactions, elimination journal entries (IC AR/AP, IC Sales/COGS, Unrealized Inventory Profit), Non-Controlling Interest (NCI), and Cumulative Translation Adjustments (CTA) based on Hoyle. Triggers: intercompany-consolidation, financial-consolidation, elimination-journal-entries, intercompany-elimination, non-controlling-interest, cumulative-translation-adjustment, multi-entity-erp, advanced-accounting.",
        r#"# Intercompany Accounting & Financial Consolidation: Eliminations, NCI & CTA
> Based on **Advanced Accounting - Hoyle, Schaefer, Doupnik**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Intercompany Balance Elimination Invariant
For legal entities $A$ and $B$:
$$\text{Receivable}_{A \to B} - \text{Payable}_{B \to A} = 0$$
Elimination Entry:
$$\text{Debit: Accounts Payable } (\text{Entity } B) \quad \text{Credit: Accounts Receivable } (\text{Entity } A)$$

### 2.2 Unrealized Intercompany Inventory Profit Elimination
If entity $A$ sells goods to entity $B$ at markup $M = \frac{\text{Profit}}{\text{Price}}$, and fraction $F$ remains unsold in $B$'s inventory at period-end:
$$\text{Unrealized Profit} = F \times \text{IC Sales Amount} \times M$$
Elimination Entry:
$$\text{Debit: Consolidated COGS} \quad \text{Credit: Consolidated Inventory (Asset)}$$

### 2.3 Non-Controlling Interest (NCI)
For subsidiary $S$ with ownership fraction $\alpha \in (0, 1)$:
$$\text{NCI Share of Net Income} = (1 - \alpha) \times \text{Net Income}_S$$
$$\text{NCI Share of Equity} = (1 - \alpha) \times \text{Ending Equity}_S$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Financial Consolidation Run FSM
```mermaid
stateDiagram-v2
    [*] --> ENTITY_CLOSE
    ENTITY_CLOSE --> FX_TRANSLATION: all_subsidiaries_submitted()
    FX_TRANSLATION --> IC_MATCHING: translate_to_group_currency()
    IC_MATCHING --> DISCREPANCY: mismatch_found()
    IC_MATCHING --> ELIMINATIONS_POSTED: all_ic_matched()
    ELIMINATIONS_POSTED --> CONSOLIDATED_STATEMENT: post_elimination_journals()
    CONSOLIDATED_STATEMENT --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never report consolidated financials without netting intercompany receivables against payables.
- Eliminate intercompany sales and cost of goods sold in full.
- Remove unrealized markup from ending inventory balances for goods remaining within the group.
- Allocate minority share of subsidiary net income to Non-Controlling Interest (NCI).
```
"#,
    )
}

/// 84. erp-supply-chain-strategy Skill
pub fn erp_supply_chain_strategy() -> EccSkill {
    EccSkill::new(
        "erp-supply-chain-strategy",
        "Supply chain network design, SCOR framework metrics, Bullwhip Effect quantification, Push-Pull boundaries, and aggregate planning optimization based on Chopra and Meindl. Triggers: supply-chain-strategy, bullwhip-effect, scor-framework, push-pull-boundary, network-design, aggregate-planning, safety-inventory-cycle, supply-chain-optimization.",
        r#"# Supply Chain Strategy & Network Design: SCOR Framework, Bullwhip & Safety Buffers
> Based on **Supply Chain Management: Strategy, Planning, and Operation - Sunil Chopra & Peter Meindl**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Bullwhip Effect Quantification Invariant
The Bullwhip Measure $B$ measures demand variance amplification across tier $k$ to tier $k+1$:
$$B = \frac{\sigma_{\text{orders}}^2 / \mu_{\text{orders}}}{\sigma_{\text{demand}}^2 / \mu_{\text{demand}}}$$
If $B > 1.0$, information distortion and phantom demand amplification are present.

### 2.2 Centralized Inventory Pooling Benefit (Square Root Law)
Consolidating inventory from $N$ decentralized distribution centers into 1 central warehouse reduces aggregate safety stock:
$$\text{Safety Stock}_{\text{centralized}} = \frac{1}{\sqrt{N}} \sum_{i=1}^N \text{Safety Stock}_i$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 S&OP (Sales and Operations Planning) State Machine
```mermaid
stateDiagram-v2
    [*] --> DEMAND_FORECASTING
    DEMAND_FORECASTING --> CAPACITY_ANALYSIS: finalize_unconstrained_demand()
    CAPACITY_ANALYSIS --> S_AND_OP_MEETING: identify_bottlenecks()
    S_AND_OP_MEETING --> MASTER_SCHEDULE_COMMITTED: resolve_tradeoffs()
    MASTER_SCHEDULE_COMMITTED --> PRODUCTION_EXECUTION: release_orders()
    PRODUCTION_EXECUTION --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Measure Bullwhip Effect: Ratio of variance of orders to variance of end-customer demand.
- Apply the Square Root Law when evaluating warehouse consolidation.
- Set Push-Pull Decoupling points based on lead times versus customer order tolerance.
- Track SCOR metrics: Perfect Order Fulfillment, Order Cycle Time, Cash-to-Cash.
```
"#,
    )
}

/// 85. erp-inventory-eoq-safety-stock Skill
pub fn erp_inventory_eoq_safety_stock() -> EccSkill {
    EccSkill::new(
        "erp-inventory-eoq-safety-stock",
        "Deterministic and stochastic inventory optimization, Economic Order Quantity (EOQ), Safety Stock under lead-time and demand variance, Reorder Point (ROP), (s, S) policies, and ABC/XYZ classification based on Silver, Pyke, and Peterson. Triggers: inventory-eoq-safety-stock, economic-order-quantity, safety-stock-formula, reorder-point-rop, abc-xyz-inventory, cycle-service-level, holding-cost-optimization, inventory-math.",
        r#"# Inventory Math & Replenishment: EOQ, Safety Stock, Reorder Point & ABC/XYZ Classification
> Based on **Inventory Management and Production Planning and Scheduling - Silver, Pyke, Peterson**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Economic Order Quantity (EOQ) Formula
For annual demand $D$, fixed order setup cost $S$, and annual holding cost per unit $H$:
$$\text{Total Annual Cost } C(Q) = \frac{D}{Q} S + \frac{Q}{2} H$$
Taking $\frac{dC}{dQ} = 0$:
$$\text{EOQ} = Q^* = \sqrt{\frac{2 D S}{H}}$$
At EOQ: $\text{Annual Ordering Cost} = \text{Annual Holding Cost}$.

### 2.2 Safety Stock with Variable Demand and Variable Lead Time
For daily demand mean $d$ and standard deviation $\sigma_d$, lead time mean $L$ and standard deviation $\sigma_L$, and normal inverse service level $Z$:
$$\sigma_{\text{lead time demand}} = \sqrt{L \cdot \sigma_d^2 + d^2 \cdot \sigma_L^2}$$
$$\text{Safety Stock (SS)} = Z \times \sigma_{\text{lead time demand}} = Z \sqrt{L \sigma_d^2 + d^2 \sigma_L^2}$$
$$\text{Reorder Point (ROP)} = (d \times L) + \text{SS}$$

### 2.3 ABC/XYZ Classification
- **ABC (Revenue/Value Volume)**: A = Top 80% value (~20% items), B = Next 15% value (~30% items), C = Bottom 5% value (~50% items).
- **XYZ (Demand Predictability)**: Coefficient of Variation $CV = \frac{\sigma_d}{\mu_d}$:
  - X: $CV \le 0.5$ (constant, highly predictable).
  - Y: $0.5 < CV \le 1.0$ (variable demand, trend/seasonality).
  - Z: $CV > 1.0$ (sporadic, erratic demand).

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Continuous Inventory Replenishment (s, Q) FSM
```mermaid
stateDiagram-v2
    [*] --> SUFFICIENT_STOCK
    SUFFICIENT_STOCK --> REORDER_TRIGGERED: on_hand - allocated + on_order <= ROP
    REORDER_TRIGGERED --> PO_GENERATED: emit_purchase_order(qty = EOQ)
    PO_GENERATED --> IN_TRANSIT: vendor_confirmed()
    IN_TRANSIT --> SUFFICIENT_STOCK: goods_received_and_shelved()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- EOQ formula: sqrt(2 * D * S / H).
- Safety Stock accounts for both demand variance and lead time variance: Z * sqrt(L * sigma_D^2 + D^2 * sigma_L^2).
- Reorder Point formula: ROP = (daily_demand * lead_time) + safety_stock.
- Net Available Stock = On-Hand - Allocated + On-Order.
- Trigger purchase order generation when Net Available Stock falls to or below ROP.
```
"#,
    )
}

/// 86. erp-landed-cost-allocation Skill
pub fn erp_landed_cost_allocation() -> EccSkill {
    EccSkill::new(
        "erp-landed-cost-allocation",
        "Landed cost voucher processing, absorption of freight, customs tariffs, marine insurance, and port handling into perpetual inventory cost layers based on Gwynne Richards. Triggers: landed-cost-allocation, landed-cost-voucher, freight-absorption, customs-tariffs-allocation, perpetual-inventory-costing, inventory-valuation-fifo, landed-cost-conservation, landed-cost.",
        r#"# Landed Cost Allocation: Absorption of Freight, Tariffs & Demurrage into Inventory Valuation
> Based on **Warehouse Management: A Complete Guide to Improving Efficiency and Minimizing Costs - Gwynne Richards**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Landed Cost Allocation Invariant (Conservation of Cost)
Let $C_{\text{total}}$ be the additional landed expense (e.g. shipping invoice total) and $n$ receipt lines:
$$\sum_{i=1}^n \text{AllocatedCost}_i = C_{\text{total}}$$
Where for allocation basis metric $M_i \in \{\text{Value}_i, \text{Weight}_i, \text{Volume}_i, \text{Qty}_i\}$:
$$\text{AllocatedCost}_i = C_{\text{total}} \times \frac{M_i}{\sum_{j=1}^n M_j}$$

### 2.2 Inventory Cost Layer Absorption
The new unit inventory valuation layer $U_i$ absorbed into perpetual inventory (FIFO / Moving Average):
$$U_i = U_i^{\text{original}} + \frac{\text{AllocatedCost}_i}{Q_i}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Landed Cost Voucher Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> DRAFT
    DRAFT --> ALLOCATED: execute_allocation(basis)
    ALLOCATED --> DRAFT: recompute()
    ALLOCATED --> POSTED: post_to_gl_and_inventory()
    POSTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never discard landed cost fractions; allocate exactly 100% of freight and customs expenses.
- Support allocation bases: By Value, By Net Weight, By Volume, By Quantity.
- Use the final line item as a plug to ensure sum(allocated_costs) == total_landed_invoice.
- Absorb allocated costs into inventory balance if stock is unsold; expense to COGS if already sold.
```
"#,
    )
}

/// 87. erp-wms-bin-location-topology Skill
pub fn erp_wms_bin_location_topology() -> EccSkill {
    EccSkill::new(
        "erp-wms-bin-location-topology",
        "Warehouse physical topology modeling (Zone, Aisle, Bay, Level, Bin), directed putaway, wave/batch picking routes, Cube-Per-Order Index (COI) slotting, and License Plate Numbers (LPN) based on Edward Frazelle. Triggers: wms-bin-location-topology, warehouse-topology, directed-putaway, wave-picking, coi-slotting-optimization, license-plate-numbers, warehouse-management-system, wms-bin-routing.",
        r#"# Warehouse Management Topology: Directed Putaway, Wave Picking & Slotting Optimization
> Based on **World Class Warehousing and Material Handling - Edward Frazelle**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Cube-Per-Order Index (COI) Slotting Invariant
To minimize travel distance, items are slotted into forward pick locations based on the Cube-Per-Order Index:
$$\text{COI}_i = \frac{\text{Required Storage Space (Cube)}_i}{\text{Order Frequency (Picks)}_i}$$
**Slotting Invariant**: Items with the lowest COI MUST be assigned to bins closest to the packing/shipping dock.

### 2.2 Bin Volumetric & Weight Capacity Invariant
For any bin $B$ containing LPNs $k$:
$$\sum_k \text{Weight}(\text{LPN}_k) \le B_{\text{max\_weight}}$$
$$\sum_k \text{Volume}(\text{LPN}_k) \le B_{\text{max\_volume}}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Putaway Task Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> ASSIGNED
    ASSIGNED --> EN_ROUTE: scan_forklift()
    EN_ROUTE --> ARRIVED_AT_BIN: scan_bin_barcode()
    ARRIVED_AT_BIN --> CONFIRMED_STORED: scan_lpn_and_confirm_qty()
    CONFIRMED_STORED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Structure bin hierarchy: Warehouse -> Zone -> Aisle -> Bay -> Level -> Position.
- Enforce strict bin capacity checks on both weight (kg) and volume (cbm) before putaway.
- Track warehouse inventory via unique License Plate Numbers (LPN).
- Order pick lists using S-shape or optimal traveling salesman routing through aisles.
```
"#,
    )
}

/// 88. erp-procurement-vendor-lifecycle Skill
pub fn erp_procurement_vendor_lifecycle() -> EccSkill {
    EccSkill::new(
        "erp-procurement-vendor-lifecycle",
        "Strategic purchasing, vendor onboarding and compliance qualification, Request for Quotation (RFQ), Purchase Requisition (PR) to Purchase Order (PO) approval matrices, and OTIF vendor scorecards based on Monczka. Triggers: procurement-vendor-lifecycle, strategic-sourcing, pr-to-po-workflow, vendor-scorecard-otif, rfq-management, supplier-qualification, purchasing-approval-matrix, vendor-management.",
        r#"# Procurement & Vendor Lifecycle: Strategic Sourcing, RFQ, PR-to-PO & Scorecards
> Based on **Purchasing and Supply Chain Management - Robert Monczka**
## 2. Mathematical Foundations & Business Invariants

### 2.1 On-Time In-Full (OTIF) Quality Metric
A shipment is successful under OTIF iff it meets both delivery window and quantity criteria:
$$\text{OTIF} = \frac{\sum_{i=1}^N \mathbf{1}_{\{\text{OnTime}_i \land \text{InFull}_i\}}}{N} \times 100\%$$
Where:
- $\text{OnTime}_i \iff \text{ActualDate}_i \le \text{PromisedDate}_i$
- $\text{InFull}_i \iff \text{ReceivedQuantity}_i \ge \text{OrderedQuantity}_i$

### 2.2 Composite Vendor Rating Invariant
$$\text{Score} = w_1 \cdot \text{OTIF} + w_2 \cdot \left(100 - \frac{\text{PPM}}{100}\right) + w_3 \cdot \text{PriceScore}$$
With weights $\sum w_i = 1.0$.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Purchase Requisition to PO State Machine
```mermaid
stateDiagram-v2
    [*] --> PR_DRAFT
    PR_DRAFT --> PR_SUBMITTED: submit_for_approval()
    PR_SUBMITTED --> PR_APPROVED: manager_signoff()
    PR_SUBMITTED --> PR_REJECTED: over_budget()
    PR_APPROVED --> PO_ISSUED: convert_to_po()
    PO_ISSUED --> PO_ACKNOWLEDGED: vendor_confirms()
    PO_ACKNOWLEDGED --> PARTIALLY_RECEIVED: dock_receives_first_batch()
    PARTIALLY_RECEIVED --> PO_CLOSED: all_lines_received()
    PO_CLOSED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never issue a Purchase Order without an approved Purchase Requisition.
- Calculate OTIF: Only deliveries that are BOTH On-Time AND In-Full count toward the numerator.
- Suspend vendors whose quarterly OTIF score drops below threshold (e.g. 90%).
- Enforce segregation of duties: Requester cannot approve their own Purchase Requisition.
```
"#,
    )
}

/// 89. erp-3way-match-p2p Skill
pub fn erp_3way_match_p2p() -> EccSkill {
    EccSkill::new(
        "erp-3way-match-p2p",
        "3-Way Match controls in Procure-to-Pay, reconciling Purchase Order (PO) vs Goods Receipt Note (GRN) vs Vendor Invoice, price and quantity tolerance thresholds, and GR/IR clearing mechanics based on Mary Schaeffer. Triggers: 3way-match-p2p, three-way-match, po-grn-invoice-matching, gr-ir-clearing, ap-invoice-tolerances, accounts-payable-controls, invoice-exception-management, p2p-matching.",
        r#"# 3-Way Matching & Accounts Payable: PO, Goods Receipt & Vendor Invoice Reconciliation
> Based on **Accounts Payable Best Practices - Mary Schaeffer**
## 2. Mathematical Foundations & Business Invariants

### 2.1 3-Way Match Verification Invariant
Let $Q_{\text{inv}}$ be the invoice quantity, $Q_{\text{received}}$ be the cumulative GRN received quantity, $P_{\text{inv}}$ be the invoice unit price, and $P_{\text{po}}$ be the authorized PO unit price.
With allowable tolerance thresholds $\tau_{\text{qty}}$ (e.g. 1%) and $\tau_{\text{price}}$ (e.g. 0.5%):
$$\frac{|Q_{\text{inv}} - Q_{\text{received}}|}{Q_{\text{received}}} \le \tau_{\text{qty}}$$
$$\frac{|P_{\text{inv}} - P_{\text{po}}|}{P_{\text{po}}} \le \tau_{\text{price}}$$
If both conditions hold, the match passes and the invoice is released for payment.

### 2.2 GR/IR Clearing Account Mechanics
Upon Goods Receipt:
$$\text{Debit: Raw Materials Inventory} \quad \text{Credit: GR/IR Clearing Account}$$
Upon Invoice Receipt (3-Way Match Pass):
$$\text{Debit: GR/IR Clearing Account} \quad \text{Credit: Accounts Payable Liability}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 AP Invoice 3-Way Match FSM
```mermaid
stateDiagram-v2
    [*] --> ENTERED
    ENTERED --> MATCH_PASS: run_match() [discrepancy <= tolerance]
    ENTERED --> EXCEPTION_HOLD: run_match() [discrepancy > tolerance]
    EXCEPTION_HOLD --> MATCH_PASS: buyer_tolerance_override()
    EXCEPTION_HOLD --> DISPUTED: vendor_credit_memo_requested()
    MATCH_PASS --> APPROVED: schedule_payment()
    APPROVED --> PAID: execute_payment_run()
    PAID --> [*]
    DISPUTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Reconcile Purchase Order (PO), Goods Receipt (GRN), and Vendor Invoice before paying.
- Match Invoice Quantity against Received Quantity (NOT PO quantity).
- Match Invoice Price against Authorized PO Price.
- Never clear GR/IR accounts manually; clear them via verified 3-way match transactions.
```
"#,
    )
}

/// 90. erp-reverse-logistics-rma Skill
pub fn erp_reverse_logistics_rma() -> EccSkill {
    EccSkill::new(
        "erp-reverse-logistics-rma",
        "Return Merchandise Authorization (RMA) workflows, reverse logistics disposition routing (Restock, Rework, Scrap, Return-to-Vendor), customer credit memos, and restocking fee accounting based on Rogers and Lembke. Triggers: reverse-logistics-rma, rma-workflows, return-merchandise-authorization, disposition-routing, restocking-fee, customer-credit-memos, salvage-accounting, returns-management.",
        r#"# Reverse Logistics & Returns: RMA Workflows, Disposition Routing & Salvage Accounting
> Based on **Going Backwards: Reverse Logistics Trends and Practices - Dale Rogers & Ronald Lembke**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Customer Credit Memo Calculation Invariant
For RMA with accepted items $i$, unit price $P_i$, received quantity $Q_i$, and restocking fee percentage $R$:
$$\text{Gross Refund} = \sum_{i} (Q_i \times P_i)$$
$$\text{Restocking Fee} = \text{Gross Refund} \times \frac{R}{100}$$
$$\text{Net Credit Amount} = \text{Gross Refund} - \text{Restocking Fee}$$

### 2.2 Quantity Return Limit Invariant
$$\sum \text{AuthorizedQuantity}_{\text{RMA}} \le Q_{\text{Shipped}}(\text{Original Order})$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 RMA Lifecycle State Machine
```mermaid
stateDiagram-v2
    [*] --> REQUESTED
    REQUESTED --> AUTHORIZED: validate_within_return_window()
    REQUESTED --> REJECTED: outside_policy()
    AUTHORIZED --> RECEIVED: dock_receives_parcel()
    RECEIVED --> INSPECTED: quality_grade()
    INSPECTED --> COMPLETED: issue_credit_memo_and_route()
    COMPLETED --> [*]
    REJECTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never issue a refund without an approved RMA and inspection receipt.
- Restocking fee must be deducted from gross refund: Net Credit = Gross - Fee.
- Verify return quantity never exceeds originally shipped order quantity.
- Route inspected returns strictly by disposition: Restock, Refurbish, Scrap, or Return-to-Vendor.
```
"#,
    )
}

/// 91. erp-mrp-crp-core Skill
pub fn erp_mrp_crp_core() -> EccSkill {
    EccSkill::new(
        "erp-mrp-crp-core",
        "Master Production Scheduling (MPS), Material Requirements Planning (MRP I), Capacity Requirements Planning (CRP), work center loading, and finite vs infinite scheduling based on Jacobs, Berry, Whybark, and Vollmann. Triggers: mrp-crp-core, manufacturing-planning-control, master-production-schedule, mps-mrp, capacity-requirements-planning, work-center-loading, finite-capacity-scheduling, rough-cut-capacity.",
        r#"# Manufacturing Planning & Control: MPS, MRP I & Capacity Requirements Planning (CRP)
> Based on **Manufacturing Planning and Control for Supply Chain Management - Jacobs, Berry, Whybark, Vollmann**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Net Requirements Formulation
For product $i$ in planning period $t$:
$$\text{Net Requirements}_t = \max\left(0, \text{Gross Requirements}_t - \text{Projected Available}_{t-1} - \text{Scheduled Receipts}_t + \text{Safety Stock}\right)$$
$$\text{Projected Available}_t = \text{Projected Available}_{t-1} + \text{Scheduled Receipts}_t + \text{Planned Order Receipts}_t - \text{Gross Requirements}_t$$

### 2.2 Capacity Requirements Planning (CRP) Loading Invariant
For work center $W$ on day $t$ with operations $j \in \text{ScheduledOps}(W, t)$:
$$\text{Required Load Hours}(W, t) = \sum_{j} \left(\text{SetupTime}_j + \text{RunTimePerUnit}_j \times \text{BatchSize}_j\right)$$
**Finite Capacity Constraint**:
$$\text{Required Load Hours}(W, t) \le \text{Rated Capacity Hours}(W, t)$$
If load exceeds rated capacity, work must be shifted forward or backward.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 MRP Order Status Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> PLANNED
    PLANNED --> FIRM: planner_locks_horizon()
    FIRM --> RELEASED: material_availability_verified()
    RELEASED --> WORK_ORDER_ACTIVE: shop_floor_dispatch()
    WORK_ORDER_ACTIVE --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Calculate net requirements: Net = max(0, Gross - OnHand - SchedReceipts + SafetyStock).
- Work center capacity: RatedCapacity = AvailableHours * Efficiency * Utilization.
- Reject infinite capacity overloads when operating under finite scheduling constraints.
- Offset planned order release date from due date using manufacturing lead time.
```
"#,
    )
}

/// 92. erp-bom-explosion-routing Skill
pub fn erp_bom_explosion_routing() -> EccSkill {
    EccSkill::new(
        "erp-bom-explosion-routing",
        "Multi-level Bill of Materials (BOM), Directed Acyclic Graph (DAG) acyclicity verification, Low-Level Coding (LLC) algorithms, recursive gross-to-net BOM explosion, scrap factors, and routing operations based on Orlicky. Triggers: bom-explosion-routing, bill-of-materials, multi-level-bom, low-level-coding, orlicky-mrp, bom-scrap-factor, routing-operations, bom-explosion.",
        r#"# Bill of Materials & Routing: Low-Level Coding, Multi-Level Explosion & Scrap Factors
> Based on **Orlicky's Material Requirements Planning - Joseph Orlicky / Carol Ptak & Chad Smith**
## 2. Mathematical Foundations & Business Invariants

### 2.1 BOM Directed Acyclic Graph (DAG) Invariant
Let $G = (V, E)$ be the BOM graph where vertices $V$ are products and directed edges $(u, v) \in E$ represent component $v$ contained in parent assembly $u$:
$$\forall v \in V, \quad v \notin \text{Descendants}(v) \iff \text{Cycles}(G) = \emptyset$$
Any circular component reference (e.g. $A \to B \to C \to A$) is structurally illegal.

### 2.2 Low-Level Code (LLC) Definition
The Low-Level Code of an item $i$ is the maximum depth at which it appears in any BOM tree:
$$\text{LLC}(i) = \begin{cases} 0 & \text{if } i \text{ is an end-item (never a component)} \\ 1 + \max_{(p, i) \in E} \text{LLC}(p) & \text{otherwise} \end{cases}$$
**MRP Explosion Invariant**: Requirements for an item with LLC $k$ MUST NOT be processed until all items with LLC $< k$ have completed processing.

### 2.3 Scrap Factor Requirement Inflation
For parent requirement $Q_{\text{parent}}$, quantity per assembly $q$, and scrap rate $s \in [0, 1)$:
$$\text{Gross Component Requirement} = \frac{Q_{\text{parent}} \times q}{1 - s}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Engineering Change Order (ECO) BOM Lifecycle
```mermaid
stateDiagram-v2
    [*] --> DRAFT
    DRAFT --> UNDER_REVIEW: submit_eco()
    UNDER_REVIEW --> ACTIVE: approve_engineering_change()
    UNDER_REVIEW --> REJECTED: reject_revision()
    ACTIVE --> OBSOLETE: superseded_by_new_revision()
    OBSOLETE --> [*]
    REJECTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never allow circular parent-child relationships in BOMs: verify acyclic DAG property.
- Assign Low-Level Codes (LLC) to every part; process MRP level by level (level 0 first).
- Inflate gross requirements by scrap rate: Gross = (ParentQty * QtyPer) / (1 - ScrapRate).
- Routing operations must have unique, strictly increasing sequence numbers (10, 20, 30).
```
"#,
    )
}

/// 93. erp-theory-of-constraints-dbr Skill
pub fn erp_theory_of_constraints_dbr() -> EccSkill {
    EccSkill::new(
        "erp-theory-of-constraints-dbr",
        "Theory of Constraints (TOC), the 5 Focusing Steps, Drum-Buffer-Rope (DBR) shop floor scheduling, and Throughput Accounting (T, I, OE) based on Eliyahu Goldratt. Triggers: theory-of-constraints-dbr, goldratt-toc, drum-buffer-rope, throughput-accounting, bottleneck-scheduling, five-focusing-steps, dbr-buffer-management, inventory-operating-expense.",
        r#"# Theory of Constraints & Drum-Buffer-Rope: Bottleneck Scheduling & Throughput Accounting
> Based on **The Goal: A Process of Ongoing Improvement - Eliyahu Goldratt & Jeff Cox**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Goldratt's Throughput Accounting Definitions
1. **Throughput ($T$)**: The rate at which the system generates money through sales:
   $$T = \text{Revenue} - \text{Truly Variable Costs (TVC)}$$
   *(Direct labor is generally considered part of Operating Expense, NOT TVC)*.
2. **Investment/Inventory ($I$)**: All the money tied up in the system (raw materials, WIP, plant assets).
3. **Operating Expense ($\text{OE}$)**: All the money spent to turn Inventory into Throughput (labor, rent, electricity).
$$\text{Net Profit (NP)} = T - \text{OE}$$
$$\text{Return on Investment (ROI)} = \frac{T - \text{OE}}{I}$$

### 2.2 Drum-Buffer-Rope (DBR) Synchronization Invariant
Let $C_{\text{drum}}$ be the capacity of the bottleneck resource. The rate of material release at the gateway operation (the Rope) must be synchronized strictly to the Drum:
$$\text{ReleaseRate}_{\text{rope}} \le C_{\text{drum}}$$
Releasing material faster than $C_{\text{drum}}$ does not increase throughput; it merely swells WIP inventory and elongates lead time.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 DBR Buffer Management Zone FSM
```mermaid
stateDiagram-v2
    [*] --> ZONE_1_GREEN
    ZONE_1_GREEN --> ZONE_2_YELLOW: job_delayed_into_middle_third()
    ZONE_2_YELLOW --> ZONE_3_RED: job_delayed_into_final_third()
    ZONE_3_RED --> EXPEDITE_TRIGGERED: alert_shop_supervisor()
    ZONE_3_RED --> ZONE_1_GREEN: job_arrives_at_drum()
    EXPEDITE_TRIGGERED --> ZONE_1_GREEN: expedited_batch_fed_to_drum()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Throughput = Revenue - Truly Variable Costs (TVC). Direct labor is part of OE.
- Never run non-bottlenecks at 100% capacity; subordinate non-bottlenecks to the Drum.
- Choke release of materials at the Rope to match bottleneck consumption rate.
- Color code DBR buffers: Green (OK), Yellow (Plan), Red (Expedite immediately).
```
"#,
    )
}

/// 94. erp-ddmrp-demand-driven Skill
pub fn erp_ddmrp_demand_driven() -> EccSkill {
    EccSkill::new(
        "erp-ddmrp-demand-driven",
        "DDMRP 5-component framework, Strategic Inventory Positioning, Decoupled Lead Time (DLT), Red/Yellow/Green dynamic buffer sizing, and the Net Flow Equation based on Ptak and Smith. Triggers: ddmrp-demand-driven, demand-driven-mrp, decoupled-lead-time, ddmrp-buffer-zones, net-flow-equation, average-daily-usage, demand-driven-planning, ddmrp.",
        r#"# Demand Driven MRP (DDMRP): Decoupled Lead Time, Dynamic Buffers & Net Flow Equation
> Based on **Demand Driven Material Requirements Planning (DDMRP) - Carol Ptak & Chad Smith**
## 2. Mathematical Foundations & Business Invariants

### 2.1 DDMRP 3-Color Buffer Zone Sizing Formulas
For item with Average Daily Usage (ADU), Decoupled Lead Time (DLT), Lead Time Factor (LTF), and Variability Factor (VF):
1. **Yellow Zone**:
   $$\text{Yellow Zone} = \text{ADU} \times \text{DLT}$$
2. **Red Zone**:
   $$\text{Red Base} = \text{ADU} \times \text{DLT} \times \text{LTF}$$
   $$\text{Red Safety} = \text{Red Base} \times \text{VF}$$
   $$\text{Red Zone} = \text{Red Base} + \text{Red Safety}$$
3. **Green Zone**:
   $$\text{Green Zone} = \max\left(\text{MinOrderQty}, \text{ADU} \times \text{DLT} \times \text{LTF}\right)$$

### 2.2 The Net Flow Equation & Order Recommendation
$$\text{Net Flow Position} = \text{On-Hand} + \text{On-Order (Open Supply)} - \text{Qualified Demand Spikes}$$
*(A Qualified Demand Spike is any sales order due within the spike horizon that exceeds threshold, typically $50\% \text{ of Red Base}$)*.
**Replenishment Invariant**:
$$\text{If } \text{Net Flow Position} \le \text{Top of Yellow} \implies \text{Order Recommended}$$
$$\text{Recommended Order Quantity} = \text{Top of Green} - \text{Net Flow Position}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 DDMRP Execution Priority State Machine
```mermaid
stateDiagram-v2
    [*] --> GREEN_HEALTHY
    GREEN_HEALTHY --> YELLOW_REORDER: net_flow <= top_of_yellow
    YELLOW_REORDER --> RED_ALERT: on_hand_drops_into_red
    RED_ALERT --> DARK_RED_CRITICAL: on_hand <= 50_pct_of_red
    DARK_RED_CRITICAL --> GREEN_HEALTHY: emergency_supply_received()
    RED_ALERT --> GREEN_HEALTHY: replenishment_received()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Net Flow Equation = On-Hand + Open Supply - Qualified Demand Spikes.
- Replenish whenever Net Flow <= Top of Yellow. Order Quantity = Top of Green - Net Flow.
- Size Yellow = ADU * DLT; Red = RedBase * (1 + VF); Green = max(MOQ, RedBase).
- Color-code execution priority by On-Hand percentage of Red Zone.
```
"#,
    )
}

/// 95. erp-lean-toyota-production Skill
pub fn erp_lean_toyota_production() -> EccSkill {
    EccSkill::new(
        "erp-lean-toyota-production",
        "Toyota Production System (TPS), Kanban loop mathematics, Heijunka production leveling, Takt Time calculation, Jidoka (autonomation), and elimination of the 7 Mudas based on Taiichi Ohno. Triggers: lean-toyota-production, toyota-production-system, kanban-math, heijunka-leveling, takt-time, jidoka-andon, seven-mudas, pull-production.",
        r#"# Toyota Production System (TPS): Lean Manufacturing, Kanban Math & Jidoka
> Based on **Toyota Production System: Beyond Large-Scale Production - Taiichi Ohno**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Kanban Card Calculation Invariant
To support pull production without overproducing, the total number of circulating kanban containers $K$ is strictly fixed:
$$K = \left\lceil \frac{D \times L \times (1 + \alpha)}{C} \right\rceil$$
Where:
- $D$: Demand rate (units per time unit).
- $L$: Replenishment lead time (production + conveyance + wait).
- $\alpha$: Safety factor (typically $0.05 \le \alpha \le 0.20$).
- $C$: Container standard batch capacity.

### 2.2 Takt Time Invariant
$$\text{Takt Time} = \frac{\text{Net Available Working Time per Day}}{\text{Customer Daily Demand Quantity}}$$
If line cycle time $> \text{Takt Time}$, overtime or bottlenecks occur. If cycle time $< \text{Takt Time}$, waste of overproduction occurs.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Kanban Card State Cycle
```mermaid
stateDiagram-v2
    [*] --> IN_BIN_ATTACHED
    IN_BIN_ATTACHED --> POSTED_BOARD: container_emptied_by_consumer()
    POSTED_BOARD --> IN_PRODUCTION: producer_pulls_card()
    IN_PRODUCTION --> IN_TRANSIT: container_filled_and_card_reattached()
    IN_TRANSIT --> IN_BIN_ATTACHED: delivered_to_consumer_supermarket()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Kanban formula: K = ceil((Demand * LeadTime * (1 + SafetyFactor)) / ContainerCapacity).
- Number of Kanban cards in circulation must remain strictly constant.
- Takt Time = Net Available Operating Time / Daily Customer Demand.
- Stop-the-line on defect (Jidoka): An Andon event must immediately pause downstream feed.
```
"#,
    )
}

/// 96. erp-shop-floor-mes Skill
pub fn erp_shop_floor_mes() -> EccSkill {
    EccSkill::new(
        "erp-shop-floor-mes",
        "Manufacturing Execution Systems (MES), ISA-95 standard, machine telemetry integration, work order dispatching, and Overall Equipment Effectiveness (OEE = Availability * Performance * Quality) based on Jürgen Kletti. Triggers: shop-floor-mes, isa-95-mes, overall-equipment-effectiveness, oee-tracking, shop-floor-dispatching, machine-downtime-logging, manufacturing-execution-systems, oee-calculation.",
        r#"# Manufacturing Execution Systems (MES): ISA-95 Shop Floor Control & OEE Tracking
> Based on **MES: Manufacturing Execution Systems - Jürgen Kletti**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Overall Equipment Effectiveness (OEE) Decomposition
$$\text{OEE} = \text{Availability} \times \text{Performance} \times \text{Quality}$$
Where:
1. **Availability ($A$)**:
   $$A = \frac{\text{Operating Time}}{\text{Planned Production Time}} = \frac{\text{Planned Time} - \text{Downtime}}{\text{Planned Time}}$$
2. **Performance ($P$)**:
   $$P = \frac{\text{Ideal Cycle Time} \times \text{Total Parts Produced}}{\text{Operating Time (seconds)}}$$
3. **Quality ($Q$)**:
   $$Q = \frac{\text{Good Parts Produced}}{\text{Total Parts Produced}}$$
**OEE Invariant**: $0.0 \le \text{OEE} \le 1.0$. World class benchmark is $\text{OEE} \ge 85\%$ ($A \ge 90\%, P \ge 95\%, Q \ge 99.9\%$).

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Work Center Machine Operational FSM
```mermaid
stateDiagram-v2
    [*] --> OFFLINE
    OFFLINE --> SETUP: start_changeover()
    SETUP --> RUNNING: changeover_complete()
    RUNNING --> UNPLANNED_STOP: sensor_fault()
    UNPLANNED_STOP --> RUNNING: technician_clears_jam()
    RUNNING --> PLANNED_MAINTENANCE: schedule_pms()
    PLANNED_MAINTENANCE --> OFFLINE: shift_ends()
    RUNNING --> OFFLINE: shift_ends()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- OEE formula: Availability * Performance * Quality.
- Availability = (Planned Time - Downtime) / Planned Time.
- Performance = (Ideal Cycle Time * Total Count) / Operating Time.
- Quality = Good Count / Total Count.
- Target world-class OEE benchmark of 85%.
```
"#,
    )
}

/// 97. erp-batch-traceability-genealogy Skill
pub fn erp_batch_traceability_genealogy() -> EccSkill {
    EccSkill::new(
        "erp-batch-traceability-genealogy",
        "Forward and backward lot traceability, bidirectional genealogy DAGs, FEFO shelf-life management, electronic batch records (EBR), and mock recall execution based on GS1 Standards and FDA 21 CFR Part 11. Triggers: batch-traceability-genealogy, lot-traceability, backward-forward-tracing, fefo-expiry-management, fda-21-cfr-part-11, electronic-batch-record, product-recall-mock, gs1-traceability.",
        r#"# Lot Traceability & Genealogies: Forward/Backward Tracing & FDA 21 CFR Part 11
> Based on **Traceability in Food and Pharma - GS1 Standard & FDA 21 CFR Part 11**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Bidirectional Genealogy DAG Invariant
Let the lot genealogy graph be $G = (V, E)$.
1. **Backward Trace (Root Cause Analysis)**:
   $$\text{TraceBackward}(L) = \{u \in V \mid \text{path } u \rightsquigarrow L \text{ exists in } G\}$$
2. **Forward Trace (Blast Radius for Recall)**:
   $$\text{TraceForward}(L) = \{v \in V \mid \text{path } L \rightsquigarrow v \text{ exists in } G\}$$
**Acyclicity Constraint**: $\forall v \in V, v \notin \text{TraceForward}(v) \land v \notin \text{TraceBackward}(v)$.

### 2.2 First-Expired, First-Out (FEFO) Dispatch Invariant
When picking lot $L$ for delivery at time $t$:
$$\text{ExpiryDate}(L) = \min_{L' \in \text{AvailableLots}(P)} \text{ExpiryDate}(L')$$
Any issue of lot $L'$ with $\text{ExpiryDate}(L') > \min(\text{ExpiryDate})$ is an unauthorized FEFO breach.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Pharma Lot Quality Quarantine FSM
```mermaid
stateDiagram-v2
    [*] --> QUARANTINE
    QUARANTINE --> RELEASED: lab_qc_passed() [e-signature required]
    QUARANTINE --> ON_HOLD: qc_deviation_investigation()
    ON_HOLD --> RELEASED: deviation_cleared_by_qa()
    ON_HOLD --> RECALLED: contamination_confirmed()
    RELEASED --> EXPIRED: current_date > expiry_date
    RELEASED --> RECALLED: market_incident_reported()
    RECALLED --> [*]
    EXPIRED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce FEFO (First-Expired, First-Out): always allocate the lot with earliest expiration date.
- Bidirectional traceability: Forward (blast radius recall) and Backward (root cause analysis).
- Never allow consumption of lots in QUARANTINE or ON_HOLD status.
- Electronic Batch Records must store cryptographic signatures complying with 21 CFR Part 11.
```
"#,
    )
}

/// 98. erp-bpmn-workflow-patterns Skill
pub fn erp_bpmn_workflow_patterns() -> EccSkill {
    EccSkill::new(
        "erp-bpmn-workflow-patterns",
        "BPMN 2.0 executable workflow patterns, XOR/AND/OR gateways, boundary interrupting/non-interrupting timer and error events, sub-processes, and workflow net soundness based on Dumas et al. Triggers: bpmn-workflow-patterns, bpmn-orchestration, workflow-soundness, xor-and-or-gateways, boundary-events, van-der-aalst-patterns, bpmn20-engine, process-orchestration.",
        r#"# BPMN 2.0 Process Orchestration: Gateways, Boundary Events & Workflow Patterns
> Based on **Fundamentals of Business Process Management - Marlon Dumas, Marcello La Rosa, Jan Mendling, Hajo Reijers**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Workflow Net Soundness Invariant
A workflow net $W$ is sound iff:
1. **Safeness (No Token Multiplication)**: No place in the Petri net ever contains more than one token during normal execution.
2. **Proper Completion**: When the end place is marked with a token, all other places in the net must be empty:
   $$\forall M \in [M_0\rangle, \quad M \ge M_{\text{end}} \implies M = M_{\text{end}}$$
3. **Dead Transition Freedom**: No transition in the workflow net can become unreachable from the start place.

### 2.2 Parallel Split (AND-Split) & Synchronization (AND-Join)
When an AND-Split fires with 1 incoming token, it generates $k$ concurrent tokens on all outgoing branches.
An AND-Join cannot fire until ALL $k$ incoming branch tokens have arrived at its input ports.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 BPMN Process Instance State Machine
```mermaid
stateDiagram-v2
    [*] --> ACTIVE
    ACTIVE --> SUSPENDED: suspend_process()
    SUSPENDED --> ACTIVE: resume_process()
    ACTIVE --> COMPLETED: reach_none_end_event()
    ACTIVE --> TERMINATED: reach_terminate_event()
    COMPLETED --> [*]
    TERMINATED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce workflow net soundness: no deadlocks, no dangling unconsumed tokens.
- AND-Join must wait for tokens from all incoming parallel branches before firing.
- XOR-Split evaluates conditions in order and routes token down exactly one branch.
- Boundary error events cancel active tasks within the scope unless configured as non-interrupting.
```
"#,
    )
}

/// 99. erp-process-mining-event-logs Skill
pub fn erp_process_mining_event_logs() -> EccSkill {
    EccSkill::new(
        "erp-process-mining-event-logs",
        "Process mining algorithms, discovery of process models from event logs (Alpha miner), conformance checking (fitness and precision), and discovery of bottlenecks using IEEE XES event logs based on Wil van der Aalst. Triggers: process-mining-event-logs, van-der-aalst-process-mining, xes-event-logs, alpha-miner, conformance-checking, process-discovery, process-bottleneck-analysis, petri-net-mining.",
        r#"# Process Mining & Conformance: XES Event Logs, Alpha Miner & Bottleneck Discovery
> Based on **Process Mining: Data Science in Action - Wil van der Aalst**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Directly-Follows Relation ($\succ$) Invariant
Let $L$ be an event log. Activity $a$ directly follows $b$ ($b \succ_L a$) iff there exists a trace $\sigma = \langle t_1, t_2, \dots, t_n \rangle \in L$ and index $i$ such that $t_i = b$ and $t_{i+1} = a$.
1. **Causality ($a \to_L b$)**: $a \succ_L b \land b \not\succ_L a$.
2. **Parallelism ($a \parallel_L b$)**: $a \succ_L b \land b \succ_L a$.
3. **Choice/Unrelated ($a \ \#_L\ b$)**: $a \not\succ_L b \land b \not\succ_L a$.

### 2.2 Conformance Checking Fitness Metric
Fitness ($f$) measures the fraction of event log behavior that can be replayed by the Petri net model without error:
$$f = \frac{1}{2} \left(1 - \frac{m}{c}\right) + \frac{1}{2} \left(1 - \frac{r}{p}\right)$$
Where $m$ is missing tokens, $c$ is consumed tokens, $r$ is remaining tokens, and $p$ is produced tokens. Perfect conformance $\implies f = 1.0$.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Process Mining Analytics Pipeline
```mermaid
graph TD
    A[Raw ERP Event Log] --> B[Directly-Follows Graph DFG Extraction]
    B --> C[Alpha / Inductive Miner Algorithm]
    C --> D[Discover Petri Net Process Model]
    D --> E[Token Replay Conformance Checking]
    E --> F[Highlight Deviations & SoD Violations]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Parse event logs by mandatory fields: Case ID, Activity, and Timestamp.
- Extract Directly-Follows relations: a -> b iff a occurs immediately before b in trace.
- Conformance fitness: penalize models for missing tokens during alignment replay.
- Identify process deviations: highlight traces bypassing mandatory credit or approval steps.
```
"#,
    )
}

/// 100. erp-order-to-cash-o2c Skill
pub fn erp_order_to_cash_o2c() -> EccSkill {
    EccSkill::new(
        "erp-order-to-cash-o2c",
        "Order-to-Cash (O2C) comprehensive workflow, customer credit limit validation, warehouse picking/packing/shipping, billing document creation, and cash payment matching based on Magal and Word. Triggers: order-to-cash-o2c, o2c-process, customer-credit-limit, goods-issue-shipment, o2c-invoicing, cash-application-matching, order-fulfillment-workflow, o2c.",
        r#"# Order-to-Cash (O2C) End-to-End: Quotation, Credit Limits, Shipping & Reconciliation
> Based on **Essentials of Business Processes and Information Systems - Simha Magal & Jeffrey Word**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Customer Credit Limit Check Invariant
Before order $O$ can transition from `PLACED` to `APPROVED`:
$$\text{Total Exposure} = \text{Current AR Balance} + \text{Open Orders Value} + \text{Total}(O)$$
$$\text{Invariant: } \text{Total Exposure} \le \text{Credit Limit}$$
If total exposure exceeds credit limit, the order is automatically placed on `CREDIT_HOLD`.

### 2.2 Cash Application Conservation
$$\text{Payment Amount} = \sum_{k} \text{AppliedToInvoice}_k + \text{Unallocated Cash}$$
$$\text{Invoice Open Balance} = \text{Total Amount} - \sum \text{Applied Payments} \ge 0.0000$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Order-to-Cash (O2C) State Machine
```mermaid
stateDiagram-v2
    [*] --> QUOTE_DRAFT
    QUOTE_DRAFT --> ORDER_CREATED: accept_quote()
    ORDER_CREATED --> CREDIT_HOLD: exposure > limit
    ORDER_CREATED --> ALLOCATED: credit_check_passed()
    CREDIT_HOLD --> ALLOCATED: credit_manager_release()
    ALLOCATED --> PICKED_AND_PACKED: warehouse_process()
    PICKED_AND_PACKED --> GOODS_ISSUED: carrier_scans_bol()
    GOODS_ISSUED --> INVOICED: generate_billing_doc()
    INVOICED --> PAID: cash_payment_reconciled()
    PAID --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never release an order without automated credit limit verification.
- Credit check formula: Current AR + Open Orders + New Order <= Credit Limit.
- Post Goods Issue (PGI) triggers inventory reduction and COGS recognition in General Ledger.
- Invoice Open Balance = Invoice Total - Payments Applied (must never be negative).
```
"#,
    )
}

/// 101. erp-procure-to-pay-p2p Skill
pub fn erp_procure_to_pay_p2p() -> EccSkill {
    EccSkill::new(
        "erp-procure-to-pay-p2p",
        "Procure-to-Pay (P2P) full lifecycle integration, requisition approval workflows, purchase order transmission, goods receipt posting, AP voucher entry, and disbursement runs based on Magal and Word. Triggers: procure-to-pay-p2p, p2p-lifecycle, purchase-requisition-to-po, goods-receipt-posting, ap-voucher-entry, p2p-disbursement, payment-run, p2p.",
        r#"# Procure-to-Pay (P2P) End-to-End: Requisitions, Purchase Orders, Goods Receipt & AP Vouchers
> Based on **Integrated Business Processes with ERP Systems - Simha Magal & Jeffrey Word**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Early Payment Cash Discount Invariant (e.g., 2/10 Net 30)
If payment is disbursed on date $T_{\text{pay}} \le T_{\text{discount\_date}}$:
$$\text{Discount Amount} = \text{Invoice Amount} \times \frac{\text{DiscountPct}}{100}$$
$$\text{Net Disbursed Amount} = \text{Invoice Amount} - \text{Discount Amount}$$
Otherwise:
$$\text{Net Disbursed Amount} = \text{Invoice Amount}$$

### 2.2 Cost of Forgoing Cash Discount
$$\text{Effective Annual Rate} = \frac{\text{DiscountPct}}{100 - \text{DiscountPct}} \times \frac{365}{\text{Total Term Days} - \text{Discount Days}}$$
For 2/10 Net 30: $\frac{2}{98} \times \frac{365}{20} = 37.24\%$ annual cost of capital.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 P2P Lifecycle State Machine
```mermaid
stateDiagram-v2
    [*] --> REQUISITION_APPROVED
    REQUISITION_APPROVED --> PO_TRANSMITTED: issue_po_to_vendor()
    PO_TRANSMITTED --> GOODS_RECEIVED: post_grn()
    GOODS_RECEIVED --> AP_VOUCHER_ENTERED: match_supplier_invoice()
    AP_VOUCHER_ENTERED --> APPROVED_FOR_PAYMENT: verify_3way_match()
    APPROVED_FOR_PAYMENT --> PAID: execute_payment_run()
    PAID --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never pay an AP voucher without 3-Way Match confirmation.
- Calculate early settlement discounts (e.g. 2/10 net 30) dynamically based on payment execution date.
- Goods receipt increases inventory asset and credits GR/IR clearing liability.
- Payment run debits Accounts Payable and credits Cash/Bank account.
```
"#,
    )
}

/// 102. erp-record-to-report-r2r Skill
pub fn erp_record_to_report_r2r() -> EccSkill {
    EccSkill::new(
        "erp-record-to-report-r2r",
        "Record-to-Report (R2R) financial closing lifecycle, Fast Close disciplines, subledger posting cutoffs, automated accruals/deferrals, depreciation runs, and management reporting based on Steven Bragg. Triggers: record-to-report-r2r, fast-close-methodology, financial-close-checklist, period-end-cutoff, subledger-closing-lock, accrual-deferral-engine, r2r-workflow, r2r.",
        r#"# Record-to-Report (R2R) & Fast Close: Closing Cutoffs, Accruals & Consolidation
> Based on **Fast Close: A Guide to Closing the Books Quickly - Steven Bragg**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Fast Close Critical Path Invariant
Let the financial close consist of tasks $T_i$ with durations $d_i$ and dependency graph $G$.
$$\text{Total Close Days} = \max_{\text{paths } P} \sum_{i \in P} d_i$$
To achieve a "Fast Close" ($T_{\text{close}} \le 3 \text{ days}$), subledger locks must execute concurrently:
$$\text{Lock}(\text{AP}) \parallel \text{Lock}(\text{AR}) \parallel \text{Lock}(\text{Inventory})$$

### 2.2 Balance Sheet Roll-Forward Invariant
For Retained Earnings (Equity):
$$\text{Ending RE}_t = \text{Beginning RE}_t + \text{Net Income}_t - \text{Dividends Declared}_t$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Financial Close Period Lifecycle FSM
```mermaid
stateDiagram-v2
    [*] --> OPEN
    OPEN --> SUBLEDGER_LOCKED: day_0_cutoff()
    SUBLEDGER_LOCKED --> ADJUSTMENTS_ONLY: post_accruals_and_depreciation()
    ADJUSTMENTS_ONLY --> HARD_CLOSED: cfo_signs_financial_statements()
    HARD_CLOSED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce strict cutoff: lock subledgers (AP, AR, Inventory) before posting GL adjustments.
- Auto-reverse month-end accruals on Day 1 of the following period.
- Depreciation and amortization schedules must run prior to financial statement generation.
- Hard Closed periods reject any subsequent posting transactions without exception.
```
"#,
    )
}

/// 103. erp-subscription-recurring-billing Skill
pub fn erp_subscription_recurring_billing() -> EccSkill {
    EccSkill::new(
        "erp-subscription-recurring-billing",
        "Recurring billing engine, subscription state machines, MRR/ARR waterfall metrics, usage-based consumption rating, mid-cycle proration, automated dunning, and churn modeling based on Tien Tzuo. Triggers: subscription-recurring-billing, recurring-billing-engine, mrr-arr-waterfall, usage-metering, subscription-proration, automated-dunning, churn-reduction, tien-tzuo-subscribed.",
        r#"# Subscription Economy & Recurring Billing: MRR/ARR, Usage Metering & Dunning
> Based on **Subscribed: Why the Subscription Model Will Be Your Company's Future - Tien Tzuo**
## 2. Mathematical Foundations & Business Invariants

### 2.1 MRR Waterfall Equation
For month $t$:
$$\text{Ending MRR}_t = \text{Beginning MRR}_t + \text{New MRR}_t + \text{Expansion MRR}_t - \text{Contraction MRR}_t - \text{Churned MRR}_t$$
$$\text{Annual Recurring Revenue (ARR)} = \text{Ending MRR} \times 12$$

### 2.2 Mid-Cycle Plan Upgrade Proration Invariant
If a customer switches from plan with rate $P_1$ to plan with rate $P_2$ ($P_2 > P_1$) on day $d$ of an $N$-day period:
$$\text{Unused Credit} = P_1 \times \frac{N - d}{N}$$
$$\text{New Charge} = P_2 \times \frac{N - d}{N}$$
$$\text{Prorated Immediate Invoice} = \text{New Charge} - \text{Unused Credit} = (P_2 - P_1) \times \frac{N - d}{N}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Subscription State Machine & Dunning
```mermaid
stateDiagram-v2
    [*] --> TRIAL
    TRIAL --> ACTIVE: payment_method_entered()
    ACTIVE --> PAST_DUE: payment_failed()
    PAST_DUE --> ACTIVE: retry_payment_success()
    PAST_DUE --> CANCELLED: dunning_max_retries_exhausted()
    ACTIVE --> CANCELLED: user_cancels()
    ACTIVE --> PAUSED: user_pauses()
    PAUSED --> ACTIVE: user_resumes()
    CANCELLED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Calculate MRR strictly on normalized monthly basis: ARR = MRR * 12.
- Compute mid-cycle proration using exact active day fractions: (P2 - P1) * (RemainingDays / TotalDays).
- Smart Dunning: Retry failed recurring card charges on days 1, 3, 5, and 7 before cancelling.
- Track usage-based consumption with idempotent deduplicated meter events.
```
"#,
    )
}

/// 104. erp-enterprise-integration-patterns Skill
pub fn erp_enterprise_integration_patterns() -> EccSkill {
    EccSkill::new(
        "erp-enterprise-integration-patterns",
        "Enterprise Integration Patterns (EIP), Content-Based Routers, Splitter/Aggregator, Claim Check, Transactional Outbox pattern, and Dead Letter Queues (DLQ) based on Hohpe and Woolf. Triggers: enterprise-integration-patterns, eip-messaging, transactional-outbox, content-based-router, splitter-aggregator, claim-check-pattern, dead-letter-queue, enterprise-service-bus.",
        r#"# Enterprise Integration Patterns: Messaging, Content-Based Routers & Transactional Outbox
> Based on **Enterprise Integration Patterns - Gregor Hohpe & Bobby Woolf**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Transactional Outbox Atomic Guarantee
Let state transition be $T_{\text{state}}$ and event publication be $T_{\text{event}}$.
In distributed databases, writing to the message broker directly risks inconsistency (Dual-Write Problem).
**Outbox Invariant**:
$$\text{Transaction} = \{ \text{Mutate Aggregate Table}, \text{Insert into } \text{transactional\_outbox} \}$$
Atomicity is guaranteed by local relational ACID:
$$\text{State Mutated} \iff \text{Outbox Row Created}$$

### 2.2 Splitter-Aggregator Cardinality Invariant
When a composite order with $N$ lines is decomposed by a Splitter:
$$\text{Tokens Generated} = N$$
An Aggregator awaiting correlation key $K$ will not emit the consolidated batch until:
$$\text{Received Tokens}(K) = N$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Outbox Message Publisher FSM
```mermaid
stateDiagram-v2
    [*] --> PENDING
    PENDING --> SENT: broker_acknowledges_receipt()
    PENDING --> RETRYING: broker_nack_or_timeout()
    RETRYING --> SENT: retry_succeeds()
    RETRYING --> DEAD_LETTER: max_retries_exceeded()
    SENT --> [*]
    DEAD_LETTER --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never write to database and message broker independently: use Transactional Outbox.
- Insert outbox event in the same ACID transaction as the business aggregate update.
- Use Content-Based Routers to isolate routing rules from payload producers.
- Move unprocessable messages to Dead Letter Queue (DLQ) after exponential retries.
```
"#,
    )
}

/// 105. erp-odoo-technical-architecture Skill
pub fn erp_odoo_technical_architecture() -> EccSkill {
    EccSkill::new(
        "erp-odoo-technical-architecture",
        "Odoo ORM technical architecture, models.Model, classical and prototype inheritance (_inherit, _inherits), relational fields (One2many, Many2many), Record Rules (ir.rule), and automated XML/QWeb views based on Greg Moss and Daniel Reis. Triggers: odoo-technical-architecture, odoo-orm, odoo-inheritance, ir-rule-security, odoo-computed-fields, odoo-module-design, working-with-odoo, odoo-framework.",
        r#"# Odoo Technical Architecture: ORM Models, Inheritance, Domain Rules & Automated Views
> Based on **Working with Odoo - Greg Moss / Daniel Reis**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Odoo Domain Polish Notation Invariant
Odoo evaluates record filtering domains in prefix notation (Polish Notation):
$$\text{Domain} = [\&, ('\text{stage\_id}', '=', 1), ('\text{user\_id}', '=', \text{uid})]$$
Unary operator: $!$ (NOT). Binary operators: $\&$ (AND - default), $|$ (OR).
**Evaluation Invariant**: Every operator of arity $k$ must be followed by exactly $k$ valid operands or sub-expressions.

### 2.2 Computed Fields & Depends Invalidation
A field $F$ marked with `@api.depends('line_ids.price_subtotal')` must recompute whenever:
$$\Delta(\text{line\_ids}) \ne \emptyset \lor \Delta(\text{price\_subtotal}) \ne \emptyset$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Odoo Standard Document Workflow State
```mermaid
stateDiagram-v2
    [*] --> DRAFT: create()
    DRAFT --> CONFIRMED: action_confirm()
    CONFIRMED --> DONE: action_done()
    CONFIRMED --> CANCEL: action_cancel()
    DONE --> CANCEL: action_cancel() [if permitted by module]
    CANCEL --> DRAFT: action_draft()
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Models inherit via _inherit for in-place extension and _inherits for delegation inheritance.
- Use @api.depends for computed fields; declare all source dependencies to prevent stale cache.
- Filter multi-company data using Record Rules: [('company_id', 'in', company_ids)].
- Always call super() in overridden model methods (create, write, unlink).
```
"#,
    )
}

/// 106. erp-frappe-erpnext-framework Skill
pub fn erp_frappe_erpnext_framework() -> EccSkill {
    EccSkill::new(
        "erp-frappe-erpnext-framework",
        "Frappe Framework architecture, DocType metadata engine, Submittable Documents (docstatus: Draft -> Submitted -> Cancelled), doc_events hooks, and Server/Client Scripts based on Rushabh Mehta. Triggers: frappe-erpnext-framework, frappe-doctype-engine, submittable-documents, erpnext-architecture, frappe-hooks, frappe-server-scripts, open-source-erpnext, rushabh-mehta.",
        r#"# Frappe Framework & ERPNext: DocType Engine, Submittable Docs & Hooks
> Based on **ERPNext: Open Source ERP - Rushabh Mehta**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Submittable Document State (DocStatus) Monotonicity Invariant
Frappe enforces strict monotonic state progression for accounting and inventory ledger documents:
$$\text{DocStatus} \in \{0, 1, 2\}$$
Where:
- $0$: Draft (Editable, non-posting).
- $1$: Submitted (Immutable, active General Ledger and Stock Ledger entries).
- $2$: Cancelled (Voided, creates inverse GL/SL balancing records).
**State Transition Invariant**:
$$0 \xrightarrow{\text{submit}} 1 \xrightarrow{\text{cancel}} 2$$
Transitions $1 \to 0$ or $2 \to 1$ are strictly forbidden. Direct deletion of a document with $\text{docstatus} = 1$ is an illegal operation.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Frappe DocStatus State Machine
```mermaid
stateDiagram-v2
    [*] --> DRAFT: docstatus = 0
    DRAFT --> SUBMITTED: on_submit() [docstatus = 1]
    SUBMITTED --> CANCELLED: on_cancel() [docstatus = 2]
    CANCELLED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- DocStatus values: 0 = Draft, 1 = Submitted, 2 = Cancelled.
- Submitted documents (docstatus = 1) are strictly immutable; reverse them by cancelling.
- Wire business logic via hooks.py doc_events rather than editing core DocTypes.
- Child tables use fieldtype='Table' linked to sub-DocTypes.
```
"#,
    )
}

/// 107. erp-sap-s4hana-cleancore Skill
pub fn erp_sap_s4hana_cleancore() -> EccSkill {
    EccSkill::new(
        "erp-sap-s4hana-cleancore",
        "SAP Clean Core architectural strategy, side-by-side extensibility on SAP BTP, ABAP RESTful Application Programming Model (RAP), Core Data Services (CDS) Views, and zero-modification ERP upgrades based on Thomas Saueressig. Triggers: sap-s4hana-cleancore, clean-core-strategy, sap-btp-extensibility, abap-cloud-rap, core-data-services-cds, zero-modification-erp, s4hana-architecture, sap-clean-core.",
        r#"# SAP S/4HANA Clean Core: Side-by-Side Extensibility, BTP & ABAP Cloud (RAP)
> Based on **SAP S/4HANA Architecture - Thomas Saueressig**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Clean Core Compliance Ratio Invariant
For an enterprise SAP landscape with extensions $E$:
$$\text{Clean Core Index} = \frac{\sum_{e \in E} \mathbf{1}_{\{e \text{ uses released public APIs}\}}}{|E|} \times 100\%$$
**Clean Core Rule**: Upgradeability invariant requires $\text{Clean Core Index} = 100\%$. Any modification to SAP standard core objects (SSCR key hacks) violates the Clean Core contract.

### 2.2 CDS View Association Join Minimization
CDS Views with associations execute deferred (lazy) on-demand SQL joins:
$$Q(V) = \text{Base Projection} \cup (\text{Association} \iff \text{Field Accessed})$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 ABAP RAP Draft-Enabled Business Object Lifecycle
```mermaid
stateDiagram-v2
    [*] --> DRAFT_ACTIVE
    DRAFT_ACTIVE --> DRAFT_SAVED: user_edits_field()
    DRAFT_SAVED --> VALIDATED: activate()
    VALIDATED --> ACTIVE_PERSISTENCE: save() [writes to active DB table]
    VALIDATED --> DRAFT_SAVED: validation_failed()
    ACTIVE_PERSISTENCE --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never modify standard core tables or programs: keep core clean.
- Build extensions side-by-side on SAP BTP or using on-stack ABAP Cloud (RAP).
- Consume only released SAP APIs with C1 release contract.
- Use draft-enabled Core Data Services (CDS) views for stateful Fiori UX.
```
"#,
    )
}

/// 108. erp-netsuite-suitecloud-suiteflow Skill
pub fn erp_netsuite_suitecloud_suiteflow() -> EccSkill {
    EccSkill::new(
        "erp-netsuite-suitecloud-suiteflow",
        "NetSuite SuiteCloud technical platform, SuiteScript 2.1 triggers (UserEvent, ClientScript, MapReduce, RESTlet), SuiteFlow visual state machines, and SuiteScript governance limits based on David Geilhufe. Triggers: netsuite-suitecloud-suiteflow, suitescript-21, suiteflow-state-machine, netsuite-custom-records, suitescript-governance-units, user-event-script, netsuite-erp-architecture, mapreduce-script.",
        r#"# NetSuite SuiteCloud & SuiteScript 2.1: Custom Records, SuiteFlow & Governance
> Based on **NetSuite ERP Architecture - David Geilhufe**
## 2. Mathematical Foundations & Business Invariants

### 2.1 NetSuite Governance Budget Invariant
Every SuiteScript execution context has a strict governance usage budget $B$:
$$\sum_{k=1}^m U(\text{API\_Operation}_k) \le B$$
Where operations consume units (e.g., `record.load`: 5 units, `record.save`: 20 units, `search.run`: 10 units).
**Governance Invariant**: If cumulative units exceed $B$, the engine immediately aborts with `SSS_USAGE_LIMIT_EXCEEDED`.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 SuiteScript UserEvent Pipeline
```mermaid
stateDiagram-v2
    [*] --> BEFORE_LOAD
    BEFORE_LOAD --> USER_EDITS: render_ui()
    USER_EDITS --> BEFORE_SUBMIT: client_clicks_save()
    BEFORE_SUBMIT --> AFTER_SUBMIT: commit_to_database()
    AFTER_SUBMIT --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Monitor governance usage: UserEvent scripts have strict 1,000 unit limits; MapReduce has 10,000 units.
- UserEvent lifecycle: beforeLoad (UI customization) -> beforeSubmit (validation) -> afterSubmit (cascade updates).
- Offload long-running mass updates from UserEvent to MapReduce scripts.
- Structure custom records to maintain parent-child relationships using custom record links.
```
"#,
    )
}

/// 109. erp-multi-tenant-data-isolation Skill
pub fn erp_multi_tenant_data_isolation() -> EccSkill {
    EccSkill::new(
        "erp-multi-tenant-data-isolation",
        "Multi-tenant data isolation patterns, shared-database shared-schema with PostgreSQL Row-Level Security (RLS), schema-per-tenant isolation, and cross-tenant leakage prevention based on Guy Harrison. Triggers: multi-tenant-data-isolation, row-level-security-rls, schema-per-tenant, tenant-isolation-patterns, cross-tenant-leakage-prevention, saas-erp-multitenancy, tenant-routing.",
        r#"# Multi-Tenant Data Architecture: Row-Level Security, Schema-per-Tenant & Cross-Tenant Isolation
> Based on **Multi-Tenant Architecture - Guy Harrison**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Multi-Tenant Separation Invariant
Let $T_A$ and $T_B$ be distinct tenants ($T_A \ne T_B$).
For any query $Q$ executed in context of $T_A$, the result set $R(Q, T_A)$ must satisfy:
$$\forall r \in R(Q, T_A), \quad \text{tenant\_id}(r) = T_A$$
$$\text{Probability of cross-tenant data leak } P(r \in R \mid \text{tenant\_id}(r) = T_B) = 0$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Tenant Lifecycle State Machine
```mermaid
stateDiagram-v2
    [*] --> PROVISIONING
    PROVISIONING --> ACTIVE: complete_setup()
    ACTIVE --> SUSPENDED: payment_failure()
    SUSPENDED --> ACTIVE: invoice_paid()
    SUSPENDED --> DEPROVISIONED: data_retention_period_expired()
    ACTIVE --> DEPROVISIONED: customer_churn_requested()
    DEPROVISIONED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Every table in a shared-schema architecture MUST contain a tenant_id column.
- Enable PostgreSQL Row-Level Security (RLS) on all tenant-specific tables.
- Set session variable before executing queries: SET LOCAL app.current_tenant_id = '...'.
- Prevent SQL injection into tenant routing logic via parameterized prepared statements.
```
"#,
    )
}

/// 110. erp-custom-fields-metadata-extensibility Skill
pub fn erp_custom_fields_metadata_extensibility() -> EccSkill {
    EccSkill::new(
        "erp-custom-fields-metadata-extensibility",
        "Metadata-driven dynamic architecture, Entity-Attribute-Value (EAV) vs PostgreSQL JSONB document extensions, schema validation, GIN index acceleration, and zero-downtime field extensions. Triggers: custom-fields-metadata-extensibility, metadata-driven-architecture, eav-pattern, postgres-jsonb-custom-fields, dynamic-virtual-fields, schema-extensibility, zero-downtime-schema, enterprise-extensibility.",
        r#"# Metadata-Driven Extensibility: EAV, JSONB Schemas & Dynamic Virtual Fields
> Based on **Enterprise Software Architecture: Extensibility & Custom Fields**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Metadata Schema Validation Invariant
For entity $E$ with custom payload $D = \{k_1: v_1, \dots, k_n: v_n\}$:
$$\forall (k, v) \in D, \quad \exists \text{Def} \in \text{Fields}(E) \text{ s.t. } \text{Type}(v) = \text{Def.Type} \land (\text{Regex}(v) = \text{True})$$
If any field violates its defined metadata specification, the insert/update transaction must abort.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Custom Field Definition Lifecycle
```mermaid
stateDiagram-v2
    [*] --> DRAFT_FIELD
    DRAFT_FIELD --> ACTIVE_FIELD: publish_field()
    ACTIVE_FIELD --> DEPRECATED_FIELD: deprecate()
    DEPRECATED_FIELD --> ARCHIVED_FIELD: purge_field_values()
    ARCHIVED_FIELD --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Avoid traditional EAV join complexity: prefer PostgreSQL JSONB with GIN indexing.
- Validate custom field types against metadata definitions before writing to JSONB.
- Create functional B-tree indexes for high-frequency custom query filters: (custom_fields->>'tax_code').
- Manage custom field deprecation without dropping historic data.
```
"#,
    )
}

/// 111. erp-headless-graphql-rest-api Skill
pub fn erp_headless_graphql_rest_api() -> EccSkill {
    EccSkill::new(
        "erp-headless-graphql-rest-api",
        "Headless ERP API design, RESTful resource endpoints, GraphQL schemas, DataLoader batching patterns to prevent N+1 query exhaustion, and HMAC-signed webhook delivery based on Jin, Sahni, and Shevat. Triggers: headless-graphql-rest-api, headless-erp, enterprise-graphql-schema, dataloader-n-plus-1, webhook-hmac-signatures, restful-erp-endpoints, api-rate-limiting, api-design.",
        r#"# Headless ERP Architecture: GraphQL, REST APIs, Webhooks & N+1 DataLoader Defense
> Based on **Designing Web APIs - Brenda Jin, Saurabh Sahni, Amir Shevat**
## 2. Mathematical Foundations & Business Invariants

### 2.1 DataLoader Batching Complexity Reduction
Without DataLoader, resolving child elements for $N$ parent objects results in:
$$\text{Queries} = 1 + N \implies O(N)$$
With DataLoader key batching:
$$\text{Queries} = 1 + 1 = 2 \implies O(1)$$

### 2.2 Webhook HMAC-SHA256 Signature Verification
To prevent spoofing and replay attacks:
$$\text{Signature} = \text{HMAC-SHA256}(\text{SecretToken}, \text{Timestamp} \parallel \text{"."} \parallel \text{PayloadBody})$$
The receiver must reject any webhook where computed signature $\ne$ header signature or where $|t_{\text{current}} - t_{\text{header}}| > 300\text{s}$.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Webhook Dispatcher State Machine
```mermaid
stateDiagram-v2
    [*] --> PENDING
    PENDING --> DELIVERED: http_200_ok()
    PENDING --> RETRYING: http_5xx_or_timeout()
    RETRYING --> DELIVERED: retry_success()
    RETRYING --> FAILED: max_retries_exceeded()
    DELIVERED --> [*]
    FAILED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never write GraphQL resolvers that fetch children in a loop; always use DataLoader.
- Sign all outbound webhook payloads using HMAC-SHA256 with timestamp protection.
- Support Idempotency-Key headers on all POST/PUT endpoints.
- Enforce token bucket rate limiting on API keys.
```
"#,
    )
}

/// 112. erp-distributed-acid-kleppmann Skill
pub fn erp_distributed_acid_kleppmann() -> EccSkill {
    EccSkill::new(
        "erp-distributed-acid-kleppmann",
        "Distributed data systems, ACID vs BASE, transaction isolation levels, Snapshot Isolation (SSI), write skew anomaly prevention, Two-Phase Commit (2PC), and consensus mechanisms based on Martin Kleppmann. Triggers: distributed-acid-kleppmann, data-intensive-applications, serializable-snapshot-isolation, write-skew-prevention, two-phase-commit-2pc, distributed-transactions, acid-guarantees, consensus-raft.",
        r#"# Distributed ACID & Consensus: Serializability, 2PC & Kleppmann Enterprise Invariants
> Based on **Designing Data-Intensive Applications - Martin Kleppmann**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Write Skew Invariant & Anti-Dependency Cycle
Write skew occurs under Snapshot Isolation when two concurrent transactions read overlapping datasets, verify invariant $P$, and update disjoint records such that $P$ is violated.
**Prevention Rule**:
$$\text{To prevent write skew: Use } \text{SELECT ... FOR UPDATE} \lor \text{SERIALIZABLE isolation}$$
Example (On-Call Shift Invariant): At least 1 doctor must be on call:
$$\sum_{d \in \text{Doctors}} \mathbf{1}_{\{\text{on\_call}(d)\}} \ge 1$$
If both active doctors concurrently check the sum ($= 2$) and each sets their own `on_call = false`, both commit under Snapshot Isolation, leaving 0 doctors on call (Write Skew!).

### 2.2 Two-Phase Commit (2PC) Unanimity Invariant
Let $P_1, \dots, P_k$ be the resource managers:
$$\text{Commit Decision} = \begin{cases} \text{COMMIT} & \text{iff } \bigwedge_{i=1}^k \text{Vote}(P_i) = \text{"YES"} \\ \text{ABORT} & \text{if } \exists i \text{ s.t. } \text{Vote}(P_i) = \text{"NO"} \lor \text{Timeout} \end{cases}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Two-Phase Commit (2PC) Coordinator FSM
```mermaid
stateDiagram-v2
    [*] --> PREPARING
    PREPARING --> COMMITTED: all_participants_vote_yes()
    PREPARING --> ABORTED: any_vote_no_or_timeout()
    COMMITTED --> [*]
    ABORTED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Prevent Write Skew: Use SELECT ... FOR UPDATE or SERIALIZABLE isolation when checking aggregate invariants.
- 2PC rule: All participants must vote YES to commit; a single NO or timeout aborts.
- Differentiate Read Committed, Repeatable Read, and Serializable levels.
- Beware of clock drift: do not rely on local wall-clock timestamps for total order.
```
"#,
    )
}

/// 113. erp-saga-distributed-transactions Skill
pub fn erp_saga_distributed_transactions() -> EccSkill {
    EccSkill::new(
        "erp-saga-distributed-transactions",
        "Saga pattern for distributed enterprise microservices, Orchestration vs Choreography, Compensating Transactions (semantic rollbacks), pivot transactions, and idempotent event listeners based on Chris Richardson. Triggers: saga-distributed-transactions, saga-orchestrator, compensating-transactions, semantic-rollback, choreography-vs-orchestration, microservices-patterns, outbox-cdc-saga, saga-pattern.",
        r#"# Saga Distributed Transactions: Orchestration, Compensation & Semantic Rollbacks
> Based on **Microservices Patterns: With Examples in Java - Chris Richardson**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Saga Forward & Backward Recovery Invariant
A Saga consists of $n$ steps: $T_1, T_2, \dots, T_n$ with corresponding compensations $C_1, C_2, \dots, C_{n-1}$.
If step $T_k$ fails ($k \le n$):
$$\text{Execution Sequence} = [T_1, T_2, \dots, T_{k-1}, T_k (\text{Fail}), C_{k-1}, C_{k-2}, \dots, C_1]$$
**Compensation Invariant**:
Compensating transactions MUST be idempotent and guaranteed to succeed (or alert human operations for manual intervention).

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Saga Orchestration State Machine
```mermaid
stateDiagram-v2
    [*] --> IN_PROGRESS
    IN_PROGRESS --> COMPLETED: all_forward_steps_ok()
    IN_PROGRESS --> COMPENSATING: step_fails()
    COMPENSATING --> COMPENSATED: all_compensations_succeed()
    COMPENSATING --> FAILED_CRITICAL: compensation_fails()
    COMPLETED --> [*]
    COMPENSATED --> [*]
    FAILED_CRITICAL --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Compensating transactions must be idempotent: C(C(x)) == C(x).
- Execute compensations in reverse order of forward operations (LIFO).
- Identify the Pivot Transaction: Once the pivot commits, subsequent steps must be retriable and cannot fail.
- Store saga state in persistent storage to survive coordinator crashes.
```
"#,
    )
}

/// 114. erp-cqrs-event-sourcing Skill
pub fn erp_cqrs_event_sourcing() -> EccSkill {
    EccSkill::new(
        "erp-cqrs-event-sourcing",
        "Command Query Responsibility Segregation (CQRS) and Event Sourcing (ES), append-only event stores, deterministic aggregate reconstruction, asynchronous read-model projections, and eventual consistency management based on Adam Bellemare. Triggers: cqrs-event-sourcing, event-driven-microservices, command-query-segregation, read-model-projections, aggregate-hydration, eventual-consistency, event-store-append-only, cqrs-es.",
        r#"# CQRS & Event Sourcing: Write-Model Event Stores & Read-Model Projections
> Based on **Building Event-Driven Microservices - Adam Bellemare**
## 2. Mathematical Foundations & Business Invariants

### 2.1 State as a Pure Function of Events (Fold Invariant)
The current state $S$ of an aggregate is computed by folding historic events:
$$S_t = \text{foldl}(\text{apply}, S_0, E_{1..t})$$
**Deterministic Replay Invariant**:
Replaying identical events on an uninitialized state MUST always yield identical aggregate state:
$$\text{Replay}(E) = \text{Replay}(E)$$

### 2.2 Projection Checkpoint Monotonicity
For any asynchronous projection reader $P$:
$$\text{Offset}_t(P) \ge \text{Offset}_{t-1}(P)$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 CQRS Command-to-Projection Pipeline
```mermaid
graph LR
    Command[Client Command] --> Aggregate[Aggregate Root Domain]
    Aggregate -->|Emit Event| EventStore[(Append-Only Event Store)]
    EventStore -->|Tail Log| Projector[Async Projector Worker]
    Projector -->|Update| ReadDB[(Read-Model DB)]
    Query[Client Query] --> ReadDB
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Separate write models (Commands) from read models (Queries).
- Event Store is append-only: never UPDATE or DELETE event records.
- Hydrate aggregate by left-folding events starting from snapshot or initial state.
- Keep read projections idempotent so they can be rebuilt from scratch at any time.
```
"#,
    )
}

/// 115. erp-optimistic-locking-concurrency Skill
pub fn erp_optimistic_locking_concurrency() -> EccSkill {
    EccSkill::new(
        "erp-optimistic-locking-concurrency",
        "Enterprise concurrency control patterns, Optimistic Offline Lock using version numbers, Pessimistic Offline Lock, lost update anomaly elimination, and high-contention inventory balance decrements based on Martin Fowler. Triggers: optimistic-locking-concurrency, optimistic-offline-lock, pessimistic-locking, lost-update-anomaly, inventory-concurrency-control, row-versioning, fowler-enterprise-patterns, optimistic-concurrency.",
        r#"# Concurrency Control in ERP: Optimistic Offline Lock, Pessimistic Locking & Lost Updates
> Based on **Patterns of Enterprise Application Architecture - Martin Fowler**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Optimistic Offline Lock SQL Invariant
To prevent the Lost Update anomaly without holding long-lived database locks:
$$\text{UPDATE table SET col} = v, \text{version} = \text{version} + 1 \quad \text{WHERE id} = \text{target\_id} \land \text{version} = v_{\text{expected}}$$
**Rows Affected Invariant**:
$$\text{RowsAffected} = \begin{cases} 1 & \text{Update Succeeded} \\ 0 & \text{OptimisticLockConflictException (Abort / Retry)} \end{cases}$$

### 2.2 High-Contention Atomic Decrement
For high-frequency inventory reservations, avoid select-then-update:
$$\text{UPDATE inventory SET qty} = \text{qty} - \Delta \quad \text{WHERE product\_id} = P \land \text{qty} \ge \Delta$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Optimistic Concurrency Retry Loop
```mermaid
stateDiagram-v2
    [*] --> READ_ENTITY: fetch_state_and_version()
    READ_ENTITY --> MUTATE_MEMORY: user_or_agent_edits()
    MUTATE_MEMORY --> ATTEMPT_COMMIT: execute_versioned_update()
    ATTEMPT_COMMIT --> SUCCESS: rows_affected == 1
    ATTEMPT_COMMIT --> RETRY_EXPONENTIAL: rows_affected == 0
    RETRY_EXPONENTIAL --> READ_ENTITY: retry_count < max
    RETRY_EXPONENTIAL --> ABORT_CONFLICT: retry_count >= max
    SUCCESS --> [*]
    ABORT_CONFLICT --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Add a version BIGINT NOT NULL DEFAULT 1 column to all mutable enterprise entities.
- Execute updates checking WHERE id = :id AND version = :expected_version.
- If rows affected == 0, throw OptimisticLockConflictException and retry with jittered exponential backoff.
- Use atomic conditional updates (WHERE qty >= :requested) for inventory deductions.
```
"#,
    )
}

/// 116. erp-distributed-idempotency Skill
pub fn erp_distributed_idempotency() -> EccSkill {
    EccSkill::new(
        "erp-distributed-idempotency",
        "Distributed idempotency keys, IETF Idempotency-Key specification, request payload fingerprinting, atomic state transitions (PROCESSING -> COMPLETED), and cached response replaying. Triggers: distributed-idempotency, idempotency-keys, request-deduplication, ietf-idempotency-key, atomic-idempotency, payment-deduplication, safe-api-retries, idempotency-pattern.",
        r#"# Distributed Idempotency: Idempotency Keys, Deduplication & Atomic Execution
> Based on **Enterprise Integration Patterns / Distributed Systems Standards**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Mathematical Idempotency Invariant
A function or operation $f$ is idempotent iff applying it multiple times yields the same result as a single invocation:
$$f(f(x)) = f(x) \quad \forall x$$
In API terms:
$$\text{Exec}(K, P) = \text{Exec}(K, P) \implies \text{SideEffects}(K, P) \text{ execute exactly once}$$

### 2.2 Payload Consistency Invariant
If a client sends the same idempotency key $K$ with conflicting payload $P' \ne P$:
$$\text{Hash}(P') \ne \text{Hash}(P) \implies \text{Throw 422 Unprocessable Entity ("Idempotency key payload mismatch")}$$

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Idempotency Key Processing Lifecycle
```mermaid
stateDiagram-v2
    [*] --> INSERT_PROCESSING: insert_key_if_not_exists()
    INSERT_PROCESSING --> ALREADY_COMPLETED: key_exists_and_status_completed()
    INSERT_PROCESSING --> CONCURRENT_COLLISION: key_exists_and_locked()
    INSERT_PROCESSING --> EXECUTE_BUSINESS_LOGIC: newly_inserted()
    EXECUTE_BUSINESS_LOGIC --> COMPLETED: save_response_payload()
    EXECUTE_BUSINESS_LOGIC --> FAILED: business_logic_throws()
    ALREADY_COMPLETED --> RETURN_CACHED_RESPONSE
    RETURN_CACHED_RESPONSE --> [*]
    COMPLETED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Accept Idempotency-Key header on all state-mutating HTTP requests (POST, PUT).
- Atomically insert key with status 'PROCESSING' before executing business logic.
- Return cached response immediately on duplicate requests with matching payload hash.
- Reject requests using the same idempotency key with modified payloads (HTTP 422).
```
"#,
    )
}

/// 117. erp-segregation-of-duties-sod Skill
pub fn erp_segregation_of_duties_sod() -> EccSkill {
    EccSkill::new(
        "erp-segregation-of-duties-sod",
        "Segregation of Duties (SoD), separation of Authorization, Custody, Recording, and Reconciliation (ACRR), conflicting role matrices, toxic combinations, and compensating controls based on Romney and Steinbart. Triggers: segregation-of-duties-sod, sod-matrix, incompatible-roles, acrr-framework, fraud-prevention-controls, toxic-role-combinations, internal-accounting-controls, sod-conflict.",
        r#"# Segregation of Duties (SoD): Incompatible Role Matrices & Fraud Prevention
> Based on **Accounting Information Systems - Marshall Romney & Paul Steinbart**
## 2. Mathematical Foundations & Business Invariants

### 2.1 The ACRR Segregation Invariant
In any internal control system, the four primary responsibilities must be segregated:
$$\text{Func}(A) \cap \text{Func}(C) \cap \text{Func}(R) \cap \text{Func}(\text{Rec}) = \emptyset$$
Where:
- $A$: Authorization (e.g. approving a purchase order or disbursement).
- $C$: Custody (e.g. physical handling of cash, checks, or warehouse inventory).
- $R$: Recording (e.g. posting general ledger entries or creating invoices).
- $\text{Rec}$: Reconciliation (e.g. performing bank or inventory reconciliations).

### 2.2 Toxic Role Combination Rule
Let user $u$ have active function set $F(u)$.
$$\forall (f_1, f_2) \in \text{ConflictingRules}, \quad \{f_1, f_2\} \subseteq F(u) \implies \text{SoD Violation!}$$
Example: Cannot both `CREATE_VENDOR` and `DISBURSE_PAYMENT`.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 SoD Exception Waiver Lifecycle
```mermaid
stateDiagram-v2
    [*] --> VIOLATION_DETECTED
    VIOLATION_DETECTED --> WAIVER_REQUESTED: submit_business_justification()
    WAIVER_REQUESTED --> APPROVED_WITH_COMPENSATING_CONTROL: internal_audit_signoff()
    WAIVER_REQUESTED --> REVOKED: role_removed_from_user()
    APPROVED_WITH_COMPENSATING_CONTROL --> EXPIRED: 90_day_waiver_ends()
    EXPIRED --> REVOKED
    REVOKED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce ACRR: Authorization, Custody, Recording, and Reconciliation must be segregated.
- Prevent toxic combinations: User who creates vendor cannot authorize vendor payments.
- User who counts inventory cannot authorize inventory write-off adjustments.
- Require dual-authorization (four-eyes principle) for all transactions exceeding authority limits.
```
"#,
    )
}

/// 118. erp-sox-internal-controls-audit Skill
pub fn erp_sox_internal_controls_audit() -> EccSkill {
    EccSkill::new(
        "erp-sox-internal-controls-audit",
        "Sarbanes-Oxley (SOX) Section 404 compliance, COSO internal control framework, IT General Controls (ITGC), Delegation of Authority (DoA) approval tiers, and tamper-resistant audit logs based on Robert Moeller. Triggers: sox-internal-controls-audit, sox-404-compliance, itgc-controls, coso-framework, delegation-of-authority-doa, approval-hierarchy-matrix, internal-controls-audit, sox-compliance.",
        r#"# SOX 404 Internal Controls & Audit Trails: ITGC, COSO Framework & Delegation of Authority
> Based on **Executive's Guide to IT Governance: Improving Systems Processes with COSO, COBIT, and Sarbanes-Oxley - Robert Moeller**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Delegation of Authority (DoA) Approval Threshold Invariant
For expenditure request $R$ with amount $A$:
$$\text{Required Approval Role} = \min \{ \text{Role} \mid \text{MaxLimit}(\text{Role}) \ge A \}$$
**Compliance Invariant**:
Any transaction where:
$$\text{ApproverLimit} < A$$
MUST be rejected as a SOX deficiency.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 DoA Tiered Approval Escalation Workflow
```mermaid
stateDiagram-v2
    [*] --> SUBMITTED: amount_requested
    SUBMITTED --> MANAGER_APPROVED: amount <= 10k
    SUBMITTED --> ESCALATED_DIRECTOR: amount > 10k
    ESCALATED_DIRECTOR --> DIRECTOR_APPROVED: amount <= 50k
    ESCALATED_DIRECTOR --> ESCALATED_CFO: amount > 50k
    ESCALATED_CFO --> CFO_APPROVED: amount <= 1M
    ESCALATED_CFO --> BOARD_APPROVED: amount > 1M [board vote]
    MANAGER_APPROVED --> [*]
    DIRECTOR_APPROVED --> [*]
    CFO_APPROVED --> [*]
    BOARD_APPROVED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never allow approval of transactions exceeding the user's Delegation of Authority limit.
- Audit trail entries must capture: Who, What, When, Why, Old State, and New State.
- Disallow hard deletions of financial documents; require auditable soft-delete / status updates.
- ITGC rule: Separate production deployment access from development privileges.
```
"#,
    )
}

/// 119. erp-data-retention-gdpr-compliance Skill
pub fn erp_data_retention_gdpr_compliance() -> EccSkill {
    EccSkill::new(
        "erp-data-retention-gdpr-compliance",
        "Balancing GDPR Right to Erasure (Article 17) against statutory tax/accounting retention mandates (e.g. 7-10 years), legal holds, cryptographic PII anonymization, and audit log preservation. Triggers: data-retention-gdpr-compliance, gdpr-vs-statutory-retention, right-to-be-forgotten-erp, legal-hold-management, pii-anonymization-accounting, tax-retention-period, gdpr-compliance, data-privacy.",
        r#"# Data Privacy vs Statutory Retention: GDPR Article 17, Legal Holds & PII Anonymization
> Based on **Data Privacy and GDPR: A Practical Guide for Enterprise Architects**
## 2. Mathematical Foundations & Business Invariants

### 2.1 GDPR Art. 17(3)(b) vs Statutory Retention Invariant
Under GDPR Article 17, a data subject's Right to Erasure does NOT apply when processing is necessary for compliance with a legal obligation (e.g. tax/accounting retention mandates).
$$\text{Can Purge}(D) \iff \text{Age}(D) > \text{StatutoryPeriod}(D) \land \neg \text{LegalHold}(D)$$

### 2.2 Pseudonymization Invariant (Preserving Financial Integrity)
When an erasure request is executed on an active ledger participant:
$$\text{Anonymize}(\text{Name, Email, Address, Phone}) \to \text{HMAC}(\text{PII}, K_{\text{salt}})$$
**Invariant**: Debit and credit balances, transaction timestamps, and financial account numbers MUST remain strictly untouched and mathematically balanced.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 GDPR Erasure Request Lifecycle
```mermaid
stateDiagram-v2
    [*] --> PENDING_REVIEW
    PENDING_REVIEW --> BLOCKED_BY_LEGAL_HOLD: active_litigation_exists()
    PENDING_REVIEW --> BLOCKED_BY_STATUTE: transaction_age < 7_years()
    BLOCKED_BY_STATUTE --> ANONYMIZED_PII_ONLY: redact_pii_preserve_balances()
    PENDING_REVIEW --> FULLY_PURGED: no_financial_records_and_no_holds()
    ANONYMIZED_PII_ONLY --> [*]
    FULLY_PURGED --> [*]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Never delete posted financial ledger rows to comply with GDPR: invoke Art 17(3)(b) exemption.
- Anonymize PII (replace with cryptographic hash or '[REDACTED]') while preserving monetary balances.
- Legal holds immediately override all automated deletion or archiving schedules.
- Maintain a data retention schedule table defining minimum statutory retention per document type.
```
"#,
    )
}

/// 120. erp-tax-engine-jurisdiction-rules Skill
pub fn erp_tax_engine_jurisdiction_rules() -> EccSkill {
    EccSkill::new(
        "erp-tax-engine-jurisdiction-rules",
        "Multi-jurisdictional enterprise tax engine, VAT/GST input vs output credits, US State & Local Sales Tax economic nexus (Wayfair), tax-inclusive vs tax-exclusive arithmetic, and B2B reverse charge rules based on Richard Doernberg. Triggers: tax-engine-jurisdiction-rules, international-tax-engine, vat-gst-calculation, us-sales-tax-nexus, economic-nexus-wayfair, reverse-charge-mechanism, tax-inclusive-vs-exclusive, tax-rules.",
        r#"# International Tax Engine: VAT, GST, US Sales Tax Nexus & Reverse Charge
> Based on **International Taxation in a Nutshell - Richard Doernberg**
## 2. Mathematical Foundations & Business Invariants

### 2.1 Tax Inclusive vs Tax Exclusive Arithmetic
Let $P_{\text{net}}$ be the net unit price, $P_{\text{gross}}$ be the gross price, and $r$ be the tax rate:
1. **Tax Exclusive (Standard US B2B)**:
   $$\text{Tax} = P_{\text{net}} \times r$$
   $$P_{\text{gross}} = P_{\text{net}} + \text{Tax} = P_{\text{net}} \times (1 + r)$$
2. **Tax Inclusive (EU B2C VAT)**:
   $$P_{\text{net}} = \frac{P_{\text{gross}}}{1 + r}$$
   $$\text{Tax} = P_{\text{gross}} - P_{\text{net}} = P_{\text{gross}} \times \left(1 - \frac{1}{1 + r}\right) = P_{\text{gross}} \times \frac{r}{1 + r}$$

### 2.2 VAT Net Payable / Refundable Invariant
$$\text{Net VAT Payable to Government} = \sum \text{Output VAT (Collected on Sales)} - \sum \text{Input VAT (Paid on Purchases)}$$
If $\text{Net VAT} < 0$, the enterprise is entitled to a tax refund.

## 3. Finite State Machine (FSM) & Lifecycle Invariants

### 3.1 Tax Determination Workflow
```mermaid
graph TD
    A[Line Item Entered] --> B{Ship-to Jurisdiction Nexus?}
    B -->|No Nexus| C[Zero Tax Exempt]
    B -->|Has Nexus| D{Cross-Border B2B with Valid VAT ID?}
    D -->|Yes| E[Apply Reverse Charge 0%]
    D -->|No| F[Determine Product Taxability Category]
    F --> G[Compute Jurisdiction State + County + City Rates]
    G --> H[Record Tax Breakdown Line]
```

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Tax Exclusive: Tax = Net * rate; Gross = Net + Tax.
- Tax Inclusive: Net = Gross / (1 + rate); Tax = Gross - Net.
- Check Economic Nexus thresholds (e.g. $100k sales or 200 txns in US state) before charging sales tax.
- Intra-EU B2B with verified VAT ID triggers Reverse Charge mechanism (0% output tax, buyer accounts for tax).
```
"#,
    )
}


/// 121. analytics-kimball-dimensional-modeling Skill
pub fn analytics_kimball_dimensional_modeling() -> EccSkill {
    EccSkill::new(
        "analytics-kimball-dimensional-modeling",
        "Enterprise dimensional modeling: Star/Snowflake schemas, Fact table types (transaction, periodic snapshot, accumulating snapshot, factless), SCD Types 1-6, conformed dimensions, and Bus Architecture. Triggers: kimball-dimensional-modeling, dimensional-modeling, star-schema, snowflake-schema, fact-table, dimension-table, scd-type-2, conformed-dimensions, bus-architecture, surrogate-keys.",
        r#"# Analytics Kimball Dimensional Modeling
> Based on **The Data Warehouse Toolkit - Ralph Kimball & Margy Ross**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Star Schema DDL: DuckDB & PostgreSQL
CREATE TABLE dim_date (
    date_key INT PRIMARY KEY, -- YYYYMMDD
    full_date DATE NOT NULL,
    day_of_week VARCHAR(10) NOT NULL,
    month INT NOT NULL,
    quarter INT NOT NULL,
    year INT NOT NULL,
    is_weekend BOOLEAN NOT NULL
);

CREATE TABLE dim_customer (
    customer_sk BIGINT PRIMARY KEY, -- Surrogate Key
    customer_id VARCHAR(50) NOT NULL, -- Natural/Business Key
    full_name VARCHAR(100) NOT NULL,
    segment VARCHAR(50) NOT NULL,
    state VARCHAR(50) NOT NULL,
    valid_from TIMESTAMP NOT NULL,
    valid_to TIMESTAMP NOT NULL DEFAULT '9999-12-31 23:59:59',
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE fct_sales (
    sales_key BIGINT PRIMARY KEY,
    date_key INT NOT NULL REFERENCES dim_date(date_key),
    customer_sk BIGINT NOT NULL REFERENCES dim_customer(customer_sk),
    order_number VARCHAR(50) NOT NULL, -- Degenerate Dimension
    quantity INT NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(12, 4) NOT NULL,
    discount_amount NUMERIC(12, 4) NOT NULL DEFAULT 0.0000,
    net_sales_amount NUMERIC(12, 4) NOT NULL
);

CREATE INDEX idx_fct_sales_date ON fct_sales(date_key);
CREATE INDEX idx_fct_sales_cust ON fct_sales(customer_sk);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 SCD Type 2 Temporal Validity Invariant
For any customer record $i$ with natural key $K$:
$$\text{valid\_from}_{i} < \text{valid\_to}_{i}$$
For consecutive revisions $i$ and $i+1$ of the same natural key $K$:
$$\text{valid\_to}_{i} = \text{valid\_from}_{i+1}$$
$$\sum_{r \in \text{Revisions}(K)} [\text{is\_current} = \text{TRUE}] \equiv 1$$

### 2.2 Additive vs Non-Additive Metric Invariant
Net sales is fully additive across all dimensions:
$$\text{Net Sales} = \sum_{j \in \text{Lines}} (\text{quantity}_j \times \text{unit\_price}_j - \text{discount}_j)$$
Unit prices and ratios are non-additive and must be calculated post-aggregation:
$$\text{Average Unit Price} = \frac{\sum \text{Net Sales}}{\sum \text{Quantity}} \neq \text{AVG}(\text{unit\_price})$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
stateDiagram-v2
    [*] --> NewRecord
    NewRecord --> InsertCurrent: Natural key not found
    InsertCurrent --> ActiveRecord: Set valid_from=NOW(), is_current=TRUE
    ActiveRecord --> ChangeDetected: Incoming payload has mutated attribute
    ChangeDetected --> ExpireOldRecord: UPDATE old SET valid_to=NOW(), is_current=FALSE
    ExpireOldRecord --> InsertNewSCD2: INSERT new with valid_from=NOW(), valid_to=9999-12-31
    InsertNewSCD2 --> ActiveRecord
```
- **Invariant**: Once expired (`is_current = FALSE`), a historical dimension row is immutable.

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- SCD Type 2 Merge in DuckDB / PostgreSQL
WITH incoming AS (
    SELECT customer_id, full_name, segment, state FROM staging_customer
),
to_expire AS (
    SELECT d.customer_sk
    FROM dim_customer d
    JOIN incoming i ON d.customer_id = i.customer_id
    WHERE d.is_current = TRUE
      AND (d.full_name != i.full_name OR d.segment != i.segment OR d.state != i.state)
)
UPDATE dim_customer
SET valid_to = CURRENT_TIMESTAMP, is_current = FALSE
WHERE customer_sk IN (SELECT customer_sk FROM to_expire);
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Enforce grain strictly: one row per physical transaction event.
- Use integer surrogate keys for dimensions; never expose natural keys as primary keys.
- Preserve temporal continuity in SCD2: valid_from < valid_to, exactly one is_current=TRUE per natural key.
- Never aggregate non-additive metrics (averages/ratios) in ETL; store raw numerators and denominators.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design and implement an enterprise dimensional star schema following Kimball best practices:
1. Formulate a conformed bus matrix identifying shared dimensions (Date, Customer, Organization, Product) across business processes.
2. Implement automated SCD Type 2 dimension loaders in SQL/dbt with zero temporal overlap.
3. Validate grain integrity and fact additive behavior using automated assertions.
```
"#,
    )
}

/// 122. analytics-etl-pipeline-patterns Skill
pub fn analytics_etl_pipeline_patterns() -> EccSkill {
    EccSkill::new(
        "analytics-etl-pipeline-patterns",
        "Enterprise ETL/ELT architecture: 34 subsystems of ETL, surrogate key generation pipelines, late-arriving dimensions and facts, audit logging, and change data capture. Triggers: etl-pipeline-patterns, kimball-etl, data-warehouse-etl, surrogate-key-pipeline, late-arriving-facts, late-arriving-dimensions, data-profiling, cdc-pipeline.",
        r#"# Analytics Etl Pipeline Patterns
> Based on **The Data Warehouse ETL Toolkit - Ralph Kimball & Joe Caserta**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Pipeline Audit & Control Metadata Schema
CREATE TABLE etl_batch_control (
    batch_id UUID PRIMARY KEY,
    pipeline_name VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('RUNNING', 'SUCCESS', 'FAILED')),
    rows_extracted BIGINT NOT NULL DEFAULT 0,
    rows_inserted BIGINT NOT NULL DEFAULT 0,
    rows_updated BIGINT NOT NULL DEFAULT 0,
    rows_rejected BIGINT NOT NULL DEFAULT 0,
    start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP
);

CREATE TABLE etl_late_arriving_facts (
    fact_id BIGINT PRIMARY KEY,
    natural_key VARCHAR(100) NOT NULL,
    unresolved_dimension VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    received_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Late-Arriving Dimension Invariant
When a fact arrives with natural key $K_{dim}$ not yet present in dimension $D$:
$$\text{Assign surrogate key } SK = -1 \quad (\text{Default 'Unknown' Member})$$
$$\text{Enqueue to } \text{LateArrivingQueue}(K_{dim}, \text{fact\_id})$$
Upon arrival of dimension record $K_{dim}$ at $T_{load}$:
$$\text{UPDATE } F \text{ SET } SK = SK_{new} \text{ WHERE } SK = -1 \land K_{dim} = \text{match}$$

### 2.2 Pipeline Row Conservation Invariant
For every batch execution $B$:
$$\text{Rows}_{\text{Extracted}} = \text{Rows}_{\text{Inserted}} + \text{Rows}_{\text{Updated}} + \text{Rows}_{\text{Rejected}}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
stateDiagram-v2
    [*] --> Extract
    Extract --> CleanseAndProfile: Extract source CDC
    CleanseAndProfile --> RejectRow: Validation failed
    CleanseAndProfile --> SurrogateKeyLookup: Validation passed
    SurrogateKeyLookup --> DefaultUnknown: Dimension member missing
    SurrogateKeyLookup --> LoadFact: Dimension member exists
    DefaultUnknown --> EnqueueLateArriving
    EnqueueLateArriving --> LoadFact
    LoadFact --> CommitBatch
    CommitBatch --> [*]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- Late Arriving Surrogate Key Resolution
INSERT INTO dim_customer (customer_sk, customer_id, full_name, segment, state, valid_from, is_current)
VALUES (-1, 'UNKNOWN', 'Unknown Customer', 'Unknown', 'NA', '1970-01-01', TRUE)
ON CONFLICT (customer_sk) DO NOTHING;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Maintain an audit log for every pipeline run recording extracted, inserted, updated, and rejected row counts.
- Allocate surrogate key -1 for missing dimension members; never allow NULL foreign keys in fact tables.
- Isolate bad source data into error quarantine tables without terminating the pipeline.
- Implement strictly idempotent load steps using staging deduplication and upserts.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect a fault-tolerant Kimball ETL delivery subsystem:
1. Implement surrogate key generation and late-arriving dimension handling with placeholder keys.
2. Build an audit logging mechanism tracking row conservation: Extracted = Inserted + Updated + Rejected.
3. Design change data capture (CDC) deduplication and backfill replay strategies.
```
"#,
    )
}

/// 123. analytics-data-vault-architecture Skill
pub fn analytics_data_vault_architecture() -> EccSkill {
    EccSkill::new(
        "analytics-data-vault-architecture",
        "Data Vault 2.0 enterprise modeling: Hubs (business keys), Links (units of work relationships), and Satellites (descriptive context with hash diffs). Scalable, insert-only architecture. Triggers: data-vault-architecture, data-vault, raw-vault, business-vault, hubs-links-satellites, hash-keys, hash-diff, dv2, insert-only-warehouse.",
        r#"# Analytics Data Vault Architecture
> Based on **Building a Scalable Data Warehouse with Data Vault 2.0 - Dan Linstedt & Michael Olschimke**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Data Vault 2.0 Core DDL
CREATE TABLE hub_customer (
    customer_hk CHAR(32) PRIMARY KEY, -- MD5/SHA256 Hash Key
    customer_bk VARCHAR(50) NOT NULL, -- Business Key
    load_dts TIMESTAMP NOT NULL,
    rec_src VARCHAR(50) NOT NULL
);

CREATE TABLE lnk_customer_order (
    order_customer_hk CHAR(32) PRIMARY KEY,
    customer_hk CHAR(32) NOT NULL REFERENCES hub_customer(customer_hk),
    order_hk CHAR(32) NOT NULL,
    load_dts TIMESTAMP NOT NULL,
    rec_src VARCHAR(50) NOT NULL
);

CREATE TABLE sat_customer (
    customer_hk CHAR(32) NOT NULL REFERENCES hub_customer(customer_hk),
    load_dts TIMESTAMP NOT NULL,
    hash_diff CHAR(32) NOT NULL, -- Hash of all descriptive columns
    customer_name VARCHAR(100) NOT NULL,
    email VARCHAR(100),
    tier VARCHAR(20),
    rec_src VARCHAR(50) NOT NULL,
    PRIMARY KEY (customer_hk, load_dts)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Hash Key & Hash Diff Computation
Let $BK$ be the business key and $C_1, C_2, \dots, C_n$ be descriptive attributes:
$$HK = \text{MD5}(\text{UPPER}(\text{TRIM}(BK)))$$
$$\text{HashDiff} = \text{MD5}(\text{COALESCE}(\text{TRIM}(C_1), '') \parallel ';' \parallel \dots \parallel ';' \parallel \text{COALESCE}(\text{TRIM}(C_n), ''))$$

### 2.2 Insert-Only Satellite Invariant
A new satellite record is inserted if and only if:
$$\text{HashDiff}_{\text{incoming}} \neq \text{HashDiff}_{\text{current}}$$
No rows in Hubs, Links, or Satellites are ever updated or deleted in Raw Vault.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Source[Staging Stream] --> Hash[Compute Hash Keys & HashDiff]
    Hash --> Hub[Insert Hub if BK not seen]
    Hash --> Link[Insert Link if HK pair not seen]
    Hash --> SatCheck{HashDiff != Latest Sat HashDiff?}
    SatCheck -->|Yes| InsertSat[Insert New Satellite Row]
    SatCheck -->|No| Discard[No-op Skip]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- DuckDB Data Vault Hash Staging
SELECT 
    md5(upper(trim(customer_id))) AS customer_hk,
    customer_id AS customer_bk,
    md5(coalesce(trim(name),'') || ';' || coalesce(trim(email),'') || ';' || coalesce(trim(tier),'')) AS hash_diff,
    current_timestamp AS load_dts,
    'CRM_SOURCE' AS rec_src
FROM raw_customers;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Hash keys must be deterministic: UPPER, TRIM, and standard concatenation separator.
- Hubs store business keys and hash keys only; never put descriptive attributes in Hubs or Links.
- Satellites are strictly append-only; insert a new row only when HashDiff changes.
- Links model unit-of-work relationships across multiple hubs.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build an enterprise Data Vault 2.0 architecture:
1. Generate Raw Vault DDL (Hubs, Links, Satellites) with cryptographic hash keys (SHA-256/MD5).
2. Author scalable ingestion pipelines applying deterministic HashDiff comparison.
3. Design the Business Vault and Information Mart layer (Point-in-Time PIT and Bridge tables).
```
"#,
    )
}

/// 124. analytics-dbt-modeling-dag Skill
pub fn analytics_dbt_modeling_dag() -> EccSkill {
    EccSkill::new(
        "analytics-dbt-modeling-dag",
        "dbt analytics engineering DAG design: Medallion layering (staging, intermediate, marts), incremental materializations, custom generic tests, and snapshot SCD2 automation. Triggers: dbt-modeling-dag, dbt, analytics-engineering, dbt-dag, staging-marts, dbt-incremental, dbt-tests, dbt-snapshots, jinja-sql.",
        r#"# Analytics Dbt Modeling Dag
> Based on **Analytics Engineering with SQL and dbt - Rui Machado & Helder Silva**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Mart Incremental Table DDL (DuckDB / Snowflake)
CREATE TABLE fct_orders (
    order_id VARCHAR(50) PRIMARY KEY,
    customer_id VARCHAR(50) NOT NULL,
    order_date DATE NOT NULL,
    status VARCHAR(20) NOT NULL,
    total_amount NUMERIC(14, 4) NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Incremental Convergence Invariant
An incremental model $M$ evaluated on window $[t_0, t]$ must yield identical state to full refresh:
$$M_{\text{incremental}}(\Delta D_t) \equiv M_{\text{full}}(D_{0..t})$$

### 2.2 DAG Acyclicity Invariant
Let $G = (V, E)$ be the dbt dependency graph where $u \to v$ indicates model $v$ depends on $u$:
$$\forall v \in V, \quad v \notin \text{Descendants}(v) \quad (\text{No cycles permitted})$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph LR
    Sources --> Staging[stg_*.sql: 1-to-1 Renaming & Casting]
    Staging --> Intermediate[int_*.sql: Business Logic & Joins]
    Intermediate --> Marts[fct_* and dim_*: Reporting Entities]
    Marts --> Exposures[Dashboards & ML Models]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- dbt Incremental Pattern
{{ config(
    materialized='incremental',
    unique_key='order_id',
    incremental_strategy='merge'
) }}

WITH source_data AS (
    SELECT * FROM {{ ref('stg_orders') }}
    {% if is_incremental() %}
    WHERE updated_at > (SELECT MAX(updated_at) FROM {{ this }})
    {% endif %}
)
SELECT * FROM source_data;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Structure dbt models into 3 distinct layers: staging (cleaning), intermediate (joins), and marts (business facts/dims).
- Always specify unique_key in incremental models to avoid duplicate records on merge.
- Add primary key uniqueness and not_null schema tests to every staging and mart model.
- Refactor repeated CTEs into intermediate models or generic dbt macros.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Construct a production dbt analytics engineering DAG:
1. Establish modular staging, intermediate, and marts layers following medallion architecture.
2. Implement high-performance incremental models using merge strategies and lookback windows.
3. Formulate comprehensive generic tests, singular tests, and dbt snapshots for SCD tracking.
```
"#,
    )
}

/// 125. analytics-advanced-sql-windowing Skill
pub fn analytics_advanced_sql_windowing() -> EccSkill {
    EccSkill::new(
        "analytics-advanced-sql-windowing",
        "Advanced analytical SQL: window frames, lead/lag offsets, cumulative distributions, dense ranking, sessionization, and gaps-and-islands problem solving. Triggers: advanced-sql-windowing, sql-window-functions, windowing, lead-lag, dense-rank, sessionization, gaps-and-islands, rolling-aggregations.",
        r#"# Analytics Advanced Sql Windowing
> Based on **SQL for Data Analysis - Cathy Tanimura**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Event Stream Table for Window Analytics
CREATE TABLE user_events (
    event_id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    event_timestamp TIMESTAMP NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    page_url VARCHAR(255)
);

CREATE INDEX idx_user_events_stream ON user_events(user_id, event_timestamp);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Dense Rank Continuity Invariant
For sorted series $X = [x_1, x_2, \dots, x_n]$:
$$\text{DENSE\_RANK}(x_i) - \text{DENSE\_RANK}(x_{i-1}) \in \{0, 1\}$$

### 2.2 Gaps-and-Islands Grouping Invariant
Let $t_i$ be event time and $t_{i-1}$ be preceding event time for a user:
$$\text{IsNewSession} = \begin{cases} 1 & \text{if } t_i - t_{i-1} > 30 \text{ minutes} \lor t_{i-1} \text{ is NULL} \\ 0 & \text{otherwise} \end{cases}$$
$$\text{SessionID} = \sum_{k=1}^i \text{IsNewSession}_k$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawEvents[Ordered User Events] --> PrevTime[LAG event_timestamp]
    PrevTime --> GapCheck{Delta > 30 mins?}
    GapCheck -->|Yes| NewIsland[Mark New Island Flag = 1]
    GapCheck -->|No| SameIsland[Mark Flag = 0]
    NewIsland --> CumulativeSum[SUM Flag OVER Window]
    SameIsland --> CumulativeSum
    CumulativeSum --> SessionGroups[Distinct Session IDs]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- 30-Minute Inactivity Sessionization in SQL
WITH flagged_events AS (
    SELECT 
        event_id,
        user_id,
        event_timestamp,
        CASE 
            WHEN event_timestamp - LAG(event_timestamp) OVER (PARTITION BY user_id ORDER BY event_timestamp) > INTERVAL '30 minutes'
                 OR LAG(event_timestamp) OVER (PARTITION BY user_id ORDER BY event_timestamp) IS NULL 
            THEN 1 ELSE 0 
        END AS is_new_session
    FROM user_events
)
SELECT 
    event_id,
    user_id,
    event_timestamp,
    SUM(is_new_session) OVER (PARTITION BY user_id ORDER BY event_timestamp ROWS UNBOUNDED PRECEDING) AS session_id
FROM flagged_events;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Use ROWS BETWEEN for deterministic physical window frames; avoid unbounded RANGE unless ordering is unique.
- Solve sessionization via LAG() timestamp delta followed by running SUM() over partition.
- DENSE_RANK guarantees consecutive integer rank without gaps; RANK skips numbers on ties.
- Compute rolling 7-day or 30-day moving averages using PRECEDING frame bounds.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Author advanced analytical SQL solutions:
1. Formulate sessionization and gaps-and-islands algorithms using window offsets and running sums.
2. Build cohort retention matrices and rolling trailing aggregations with exact window frames.
3. Optimize window execution plans via composite sorting indexes on PARTITION BY + ORDER BY keys.
```
"#,
    )
}

/// 126. analytics-celko-relational-logic Skill
pub fn analytics_celko_relational_logic() -> EccSkill {
    EccSkill::new(
        "analytics-celko-relational-logic",
        "Advanced relational logic, Nested Sets tree models, relational division, temporal intervals, and ANSI Three-Valued Logic (3VL). Triggers: celko-relational-logic, relational-logic, nested-sets, tree-traversal-sql, relational-division, three-valued-logic, temporal-intervals.",
        r#"# Analytics Celko Relational Logic
> Based on **Joe Celko's SQL for Smarties - Joe Celko**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Celko Nested Sets Hierarchy
CREATE TABLE org_chart (
    emp_id INT PRIMARY KEY,
    emp_name VARCHAR(100) NOT NULL,
    lft INT NOT NULL UNIQUE CHECK (lft > 0),
    rgt INT NOT NULL UNIQUE CHECK (rgt > lft),
    CONSTRAINT chk_nested_range CHECK (lft < rgt)
);

CREATE INDEX idx_nested_sets ON org_chart(lft, rgt);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Nested Sets Invariant
For every node $N$ in a nested sets hierarchy:
$$\text{Count of Subtree Descendants} = \frac{\text{rgt}_N - \text{lft}_N - 1}{2}$$
For any child node $C$ under parent $P$:
$$\text{lft}_P < \text{lft}_C < \text{rgt}_C < \text{rgt}_P$$

### 2.2 Relational Exact Division
A entity $E$ matches all requirements $R$ if and only if:
$$\text{Count}(E \cap R) = |R| \land \text{Count}(E \setminus R) = 0$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Root[Root: lft=1, rgt=14] --> SubA[Dept A: lft=2, rgt=7]
    Root --> SubB[Dept B: lft=8, rgt=13]
    SubA --> Emp1[Emp 1: lft=3, rgt=4]
    SubA --> Emp2[Emp 2: lft=5, rgt=6]
    SubB --> Emp3[Emp 3: lft=9, rgt=10]
    SubB --> Emp4[Emp 4: lft=11, rgt=12]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- Relational Division: Candidates possessing ALL required skills
SELECT candidate_id
FROM candidate_skills
WHERE skill_name IN ('SQL', 'Rust', 'DuckDB')
GROUP BY candidate_id
HAVING COUNT(DISTINCT skill_name) = 3;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- In nested sets: node descendants count is strictly (rgt - lft - 1) / 2.
- Querying entire subtrees in nested sets requires zero recursion: WHERE lft BETWEEN p.lft AND p.rgt.
- Remember 3-valued logic: NULL = NULL yields UNKNOWN; NOT IN with NULL returns zero rows.
- Implement relational division using GROUP BY and HAVING COUNT(DISTINCT requirement) = total.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement complex relational structures using Celko's engineering principles:
1. Design Nested Sets tree schemas supporting instant sub-tree traversal without recursive CTEs.
2. Implement exact relational division queries for multi-attribute matching and skill matrices.
3. Enforce strict ANSI 3-valued logic safety when handling NULLs in outer joins and NOT IN clauses.
```
"#,
    )
}

/// 127. analytics-tukey-eda-heuristics Skill
pub fn analytics_tukey_eda_heuristics() -> EccSkill {
    EccSkill::new(
        "analytics-tukey-eda-heuristics",
        "Tukey exploratory heuristics: 5-number summary, box-and-whisker diagnostics, IQR Tukey Fences, stem-and-leaf, and median polish for two-way additive layouts. Triggers: tukey-eda-heuristics, exploratory-data-analysis, tukey-eda, box-plot, interquartile-range, iqr, tukey-fences, stem-and-leaf, median-polish.",
        r#"# Analytics Tukey Eda Heuristics
> Based on **Exploratory Data Analysis - John W. Tukey**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Summary Metrics Table for Tukey EDA Profiling
CREATE TABLE eda_metric_profile (
    metric_name VARCHAR(100) PRIMARY KEY,
    sample_size BIGINT NOT NULL,
    min_val DOUBLE PRECISION NOT NULL,
    q1_val DOUBLE PRECISION NOT NULL,
    median_val DOUBLE PRECISION NOT NULL,
    q3_val DOUBLE PRECISION NOT NULL,
    max_val DOUBLE PRECISION NOT NULL,
    iqr DOUBLE PRECISION NOT NULL,
    lower_fence DOUBLE PRECISION NOT NULL,
    upper_fence DOUBLE PRECISION NOT NULL,
    outlier_count BIGINT NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Tukey Fences Invariant
Let $Q_1$ and $Q_3$ be the 25th and 75th percentiles. Interquartile Range:
$$IQR = Q_3 - Q_1$$
$$\text{Lower Inner Fence } (LF) = Q_1 - 1.5 \times IQR$$
$$\text{Upper Inner Fence } (UF) = Q_3 + 1.5 \times IQR$$
$$\text{Lower Outer Fence} = Q_1 - 3.0 \times IQR, \quad \text{Upper Outer Fence} = Q_3 + 3.0 \times IQR$$
A data point $x$ is an outlier if $x < LF \lor x > UF$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[Raw Sample] --> Sort[Order Data Points]
    Sort --> FiveNum[Extract Min, Q1, Median, Q3, Max]
    FiveNum --> CalcIQR[IQR = Q3 - Q1]
    CalcIQR --> Fences[Compute LF = Q1 - 1.5*IQR, UF = Q3 + 1.5*IQR]
    Fences --> FilterOutliers[Identify Outliers < LF or > UF]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Tukey EDA Fences in Python / Polars
import polars as pl

def compute_tukey_fences(series: pl.Series) -> dict:
    q1 = series.quantile(0.25)
    q3 = series.quantile(0.75)
    iqr = q3 - q1
    lf = q1 - 1.5 * iqr
    uf = q3 + 1.5 * iqr
    outliers = series.filter((series < lf) | (series > uf))
    return {"q1": q1, "q3": q3, "iqr": iqr, "lf": lf, "uf": uf, "outliers_count": len(outliers)}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Compute Tukey Fences: LF = Q1 - 1.5 * IQR and UF = Q3 + 1.5 * IQR.
- Data points outside inner fences are flagged as potential outliers for manual review.
- Always report 5-number summary (Min, Q1, Median, Q3, Max) rather than solely Mean and StdDev.
- Use median polish for additive two-way contingency tables to resist outlier skew.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build automated Tukey EDA heuristic pipelines:
1. Implement vectorized 5-number summary and IQR fence calculations in DuckDB/Polars.
2. Formulate median polish algorithms decomposing row and column effects in matrix data.
3. Automatically generate box-and-whisker distributions and outlier inspection queues.
```
"#,
    )
}

/// 128. analytics-practical-statistics Skill
pub fn analytics_practical_statistics() -> EccSkill {
    EccSkill::new(
        "analytics-practical-statistics",
        "Practical statistical methods: bootstrap resampling, permutation testing, robust statistics, median absolute deviation (MAD), and sampling distribution validation. Triggers: practical-statistics, bootstrap-resampling, permutation-test, robust-statistics, median-absolute-deviation, mad, trimmed-mean, sampling-variability.",
        r#"# Analytics Practical Statistics
> Based on **Practical Statistics for Data Scientists - Peter Bruce, Andrew Bruce, Peter Gedeck**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Statistical Test Results Tracking
CREATE TABLE experiment_hypothesis_tests (
    test_id UUID PRIMARY KEY,
    metric_name VARCHAR(100) NOT NULL,
    test_type VARCHAR(50) NOT NULL, -- BOOTSTRAP, PERMUTATION
    observed_diff DOUBLE PRECISION NOT NULL,
    p_value DOUBLE PRECISION NOT NULL,
    ci_lower DOUBLE PRECISION NOT NULL,
    ci_upper DOUBLE PRECISION NOT NULL,
    iterations INT NOT NULL DEFAULT 10000
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Median Absolute Deviation (MAD)
For sample $X$:
$$MAD = \text{median}(|X_i - \text{median}(X)|)$$
$$\hat{\sigma}_{\text{robust}} = 1.4826 \times MAD$$

### 2.2 Bootstrap Standard Error
Given $B$ bootstrap samples $\hat{\theta}^*_1, \dots, \hat{\theta}^*_B$:
$$SE_{\text{boot}} = \sqrt{\frac{1}{B-1}\sum_{b=1}^B (\hat{\theta}^*_b - \bar{\theta}^*)^2}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[Sample Groups A & B] --> CalcObs[Calculate Observed Delta]
    CalcObs --> Pool[Pool Data & Strip Group Labels]
    Pool --> Shuffle[Permute / Shuffle Pooled Data]
    Shuffle --> Reassign[Reassign to Synthetic A & B]
    Reassign --> Recalc[Compute Permuted Delta]
    Recalc --> Iterate{Repeat 10,000 times}
    Iterate --> CalcPVal[p = Count(|Delta_perm| >= |Delta_obs|) / 10000]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def permutation_test(group_a: np.ndarray, group_b: np.ndarray, n_iter: int = 10000) -> float:
    obs_diff = np.abs(np.mean(group_a) - np.mean(group_b))
    combined = np.concatenate([group_a, group_b])
    n_a = len(group_a)
    count = 0
    for _ in range(n_iter):
        np.random.shuffle(combined)
        perm_diff = np.abs(np.mean(combined[:n_a]) - np.mean(combined[n_a:]))
        if perm_diff >= obs_diff:
            count += 1
    return count / n_iter
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Use bootstrap resampling to estimate standard errors and confidence intervals without normality assumptions.
- Calculate robust scale via MAD: sigma_est = 1.4826 * median(|x - median(x)|).
- Apply permutation tests to determine exact p-values for difference in means or medians.
- Rely on trimmed mean (e.g. 10% trim) to protect location metrics from heavy-tailed outliers.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect a production statistical inference library:
1. Build non-parametric bootstrap and permutation testing harnesses executing 10,000+ iterations.
2. Implement robust estimators (Huber loss, trimmed mean, MAD) resistant to non-Gaussian noise.
3. Validate sample size adequacy and empirical sampling distributions across continuous KPIs.
```
"#,
    )
}

/// 129. analytics-intuitive-statistics Skill
pub fn analytics_intuitive_statistics() -> EccSkill {
    EccSkill::new(
        "analytics-intuitive-statistics",
        "Foundational intuitive statistics: Central Limit Theorem (CLT), Law of Large Numbers, standard errors, Type I/II errors, statistical power, and Simpson's Paradox. Triggers: intuitive-statistics, central-limit-theorem, clt, law-of-large-numbers, standard-error, simpsons-paradox, type-1-type-2-errors, statistical-power.",
        r#"# Analytics Intuitive Statistics
> Based on **Naked Statistics - Charles Wheelan**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Metric Sampling Distribution Summary
CREATE TABLE sampling_distribution_runs (
    run_id UUID PRIMARY KEY,
    population_size BIGINT NOT NULL,
    sample_size INT NOT NULL,
    mean_of_means DOUBLE PRECISION NOT NULL,
    empirical_se DOUBLE PRECISION NOT NULL,
    theoretical_se DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Central Limit Theorem Invariant
For independent, identically distributed random variables with mean $\mu$ and variance $\sigma^2$:
$$\bar{X}_n = \frac{1}{n}\sum_{i=1}^n X_i \xrightarrow{d} \mathcal{N}\left(\mu, \frac{\sigma^2}{n}\right) \quad \text{as } n \to \infty$$
$$\text{Standard Error } (SE) = \frac{s}{\sqrt{n}}$$

### 2.2 Simpson's Paradox Invariant
An observed correlation in aggregate can reverse when conditioned on confounding variable $Z$:
$$\text{sgn}\left(\frac{\partial E[Y|X]}{\partial X}\right) \neq \text{sgn}\left(\frac{\partial E[Y|X, Z]}{\partial X}\right)$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Population[Non-Normal Population] --> Draw[Draw K Samples of size N]
    Draw --> SampleMeans[Compute Mean for each Sample]
    SampleMeans --> Distribution[Plot Means Distribution]
    Distribution --> BellCurve[Distribution converges to Gaussian N(mu, sigma^2/N)]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# CLT Verification in Python
import numpy as np

def verify_clt(population: np.ndarray, sample_size: int = 100, num_samples: int = 1000):
    sample_means = [np.mean(np.random.choice(population, size=sample_size)) for _ in range(num_samples)]
    theoretical_se = np.std(population) / np.sqrt(sample_size)
    empirical_se = np.std(sample_means)
    return {"mean": np.mean(sample_means), "theoretical_se": theoretical_se, "empirical_se": empirical_se}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Standard Error of the Mean is s / sqrt(n); increasing sample size 4x cuts SE by half.
- Always segment data to detect Simpson's Paradox where aggregate trends contradict sub-cohort trends.
- Balance Type I error alpha (false positive) and Type II error beta (false negative).
- Statistical significance is not practical significance: tiny effects become significant at massive n.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design foundational statistical safeguards for analytics reporting:
1. Implement automated checks detecting Simpson's paradox across high-cardinality dimensions.
2. Calculate power curves and minimum sample sizes to prevent underpowered experiments.
3. Formulate intuitive standard error and confidence interval displays for executive stakeholders.
```
"#,
    )
}

/// 130. analytics-statistical-learning-islp Skill
pub fn analytics_statistical_learning_islp() -> EccSkill {
    EccSkill::new(
        "analytics-statistical-learning-islp",
        "Core statistical learning: Bias-Variance tradeoff, K-Fold Cross-Validation, Ridge (L2) and Lasso (L1) regularized regression, and logistic classification. Triggers: statistical-learning-islp, islp, bias-variance-tradeoff, cross-validation, ridge-regression, lasso-regression, regularized-loss, logistic-regression.",
        r#"# Analytics Statistical Learning Islp
> Based on **An Introduction to Statistical Learning - Gareth James, Daniela Witten, Trevor Hastie, Robert Tibshirani**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Cross-Validation Performance Log
CREATE TABLE model_cv_runs (
    model_id UUID PRIMARY KEY,
    algorithm VARCHAR(50) NOT NULL,
    hyperparameters JSONB NOT NULL,
    k_folds INT NOT NULL DEFAULT 5,
    mean_cv_rmse DOUBLE PRECISION NOT NULL,
    std_cv_rmse DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Bias-Variance Decomposition
For target $y = f(x) + \epsilon$ with $\epsilon \sim \mathcal{N}(0, \sigma^2_\epsilon)$:
$$E[(y - \hat{f}(x))^2] = \text{Bias}(\hat{f}(x))^2 + \text{Var}(\hat{f}(x)) + \sigma^2_\epsilon$$

### 2.2 Elastic Net & Regularized Loss Invariant
$$\min_\beta \left( \frac{1}{2n} \|y - X\beta\|_2^2 + \lambda_1 \|\beta\|_1 + \frac{\lambda_2}{2} \|\beta\|_2^2 \right)$$
- When $\lambda_2 = 0$, pure Lasso ($L_1$ penalty) forces non-informative coefficients to exact zero.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Dataset --> Split[Split into K Equal Folds]
    Split --> Fold1[Train on K-1 folds, Test on Fold 1]
    Split --> Fold2[Train on K-1 folds, Test on Fold 2]
    Split --> FoldK[Train on K-1 folds, Test on Fold K]
    Fold1 --> Aggregate[Compute Mean CV Error & Standard Error]
    Fold2 --> Aggregate
    FoldK --> Aggregate
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from sklearn.linear_model import LassoCV
from sklearn.preprocessing import StandardScaler
from sklearn.pipeline import make_pipeline

pipeline = make_pipeline(StandardScaler(), LassoCV(cv=5, random_state=42))
# pipeline.fit(X_train, y_train)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- High bias causes underfitting; high variance causes overfitting on training data.
- Use K-Fold Cross-Validation (K=5 or 10) to select hyperparameters minimizing test error.
- Lasso (L1) yields sparse models via feature selection; Ridge (L2) shrinks collinear weights.
- Always standardize features (zero mean, unit variance) prior to fitting penalized regression.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build statistical learning regression and classification engines:
1. Implement cross-validated Ridge and Lasso pipelines with automated lambda grid search.
2. Evaluate bias-variance profiles using empirical learning curves across training set sizes.
3. Validate classification thresholds optimizing precision-recall tradeoffs for imbalanced business KPIs.
```
"#,
    )
}

/// 131. analytics-bayesian-rethinking Skill
pub fn analytics_bayesian_rethinking() -> EccSkill {
    EccSkill::new(
        "analytics-bayesian-rethinking",
        "Bayesian modeling: DAG causal graphs, prior predictive simulation, MCMC sampling, Highest Posterior Density Intervals (HPDI), and collider conditioning avoidance. Triggers: bayesian-rethinking, bayesian-modeling, prior-posterior, directed-acyclic-graphs, collider-bias, mcmc-sampling, hpdi, posterior-predictive.",
        r#"# Analytics Bayesian Rethinking
> Based on **Statistical Rethinking - Richard McElreath**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Bayesian Model Parameter Posterior Summary
CREATE TABLE bayesian_posteriors (
    model_name VARCHAR(100) NOT NULL,
    parameter_name VARCHAR(100) NOT NULL,
    mean DOUBLE PRECISION NOT NULL,
    std_dev DOUBLE PRECISION NOT NULL,
    hpdi_lower_95 DOUBLE PRECISION NOT NULL,
    hpdi_upper_95 DOUBLE PRECISION NOT NULL,
    r_hat DOUBLE PRECISION NOT NULL CHECK (r_hat < 1.05),
    PRIMARY KEY (model_name, parameter_name)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Bayes' Theorem Invariant
$$P(\theta | D) = \frac{P(D | \theta) P(\theta)}{P(D)} = \frac{P(D | \theta) P(\theta)}{\int P(D | \theta) P(\theta) d\theta}$$

### 2.2 Collider Bias Invariant
In DAG $X \to C \leftarrow Y$, conditioning on collider $C$ creates spurious association:
$$X \perp Y \quad \text{but} \quad X \not\perp Y \mid C$$
Rule: NEVER condition on a collider when estimating the causal effect of $X$ on $Y$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Prior[Define Generative Prior P(theta)] --> PriorPred[Prior Predictive Simulation]
    PriorPred --> Likelihood[Formulate Data Likelihood P(D|theta)]
    Likelihood --> HMC[Run Hamiltonian Monte Carlo Sampling]
    HMC --> CheckConv[Check R-hat < 1.01 and ESS > 400]
    CheckConv --> Posterior[Analyze HPDI & Posterior Predictive Checks]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# PyMC Generative Model
import pymc as pm

with pm.Model() as model:
    alpha = pm.Normal("alpha", mu=0, sigma=10)
    beta = pm.Normal("beta", mu=0, sigma=5)
    sigma = pm.Exponential("sigma", lam=1)
    mu = alpha + beta * x_obs
    y = pm.Normal("y", mu=mu, sigma=sigma, observed=y_obs)
    # trace = pm.sample(draws=2000, tune=1000, target_accept=0.95)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Perform prior predictive checks before observing data to verify realistic parameter ranges.
- Never condition on a collider (X -> C <- Y); it creates spurious correlations.
- Convergence requires Gelman-Rubin R-hat < 1.05 and high Effective Sample Size (ESS).
- Report 89% or 95% HPDI (Highest Posterior Density Interval) rather than point estimates.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement Bayesian statistical modeling workflows:
1. Formulate causal DAGs identifying backdoor adjustment paths and excluding colliders.
2. Execute Hamiltonian Monte Carlo (HMC) sampling with convergence verification (R-hat, divergences).
3. Generate posterior predictive distributions to validate model calibration against empirical data.
```
"#,
    )
}

/// 132. analytics-mathematical-inference Skill
pub fn analytics_mathematical_inference() -> EccSkill {
    EccSkill::new(
        "analytics-mathematical-inference",
        "Rigorous mathematical inference: Maximum Likelihood Estimation (MLE), Fisher Information, Cramér-Rao Lower Bound, asymptotic normality, and empirical CDFs. Triggers: mathematical-inference, all-of-statistics, maximum-likelihood, mle, fisher-information, cramer-rao, asymptotic-normality, empirical-cdf.",
        r#"# Analytics Mathematical Inference
> Based on **All of Statistics - Larry Wasserman**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- MLE Estimation Results Store
CREATE TABLE mle_parameter_estimates (
    estimation_id UUID PRIMARY KEY,
    model_type VARCHAR(50) NOT NULL,
    param_name VARCHAR(50) NOT NULL,
    mle_value DOUBLE PRECISION NOT NULL,
    fisher_info DOUBLE PRECISION NOT NULL,
    cramer_rao_bound DOUBLE PRECISION NOT NULL,
    asymptotic_se DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Fisher Information & Cramér-Rao Invariant
$$I_n(\theta) = -E\left[ \frac{\partial^2}{\partial \theta^2} \ell_n(\theta) \right] = n I_1(\theta)$$
For any unbiased estimator $\hat{\theta}$:
$$\text{Var}(\hat{\theta}) \ge \frac{1}{I_n(\theta)}$$

### 2.2 Asymptotic Normality of MLE
$$\sqrt{n}(\hat{\theta}_{\text{MLE}} - \theta_0) \xrightarrow{d} \mathcal{N}\left(0, I_1(\theta_0)^{-1}\right)$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Likelihood[Formulate Log-Likelihood Function] --> Score[Compute Score Function = First Derivative]
    Score --> Hessian[Compute Negative Hessian = Fisher Information]
    Score --> Solve[Solve Score = 0 for MLE]
    Solve --> VarBound[Asymptotic Variance = Inverse Fisher Info]
    VarBound --> ConfInterval[Construct Wald 95% Confidence Interval]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def mle_exponential(data: np.ndarray):
    n = len(data)
    lambda_hat = n / np.sum(data)
    fisher_info = n / (lambda_hat ** 2)
    se = 1.0 / np.sqrt(fisher_info)
    return {"lambda_mle": lambda_hat, "se": se, "ci_95": (lambda_hat - 1.96*se, lambda_hat + 1.96*se)}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Maximum Likelihood Estimator (MLE) is asymptotically efficient, attaining Cramér-Rao lower bound.
- Asymptotic variance of MLE equals the inverse of the Fisher Information matrix.
- Construct Wald confidence intervals as theta_hat +/- z * SE where SE = 1 / sqrt(I(theta)).
- Glivenko-Cantelli theorem guarantees uniform convergence of empirical CDF to true CDF.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Construct mathematically rigorous statistical inference modules:
1. Derive exact log-likelihoods, score equations, and Fisher Information matrices.
2. Prove asymptotic normality and construct Wald and Likelihood-Ratio test statistics.
3. Validate coverage probabilities of parametric and non-parametric confidence intervals.
```
"#,
    )
}

/// 133. analytics-kohavi-ab-experimentation Skill
pub fn analytics_kohavi_ab_experimentation() -> EccSkill {
    EccSkill::new(
        "analytics-kohavi-ab-experimentation",
        "Enterprise online controlled experimentation: Sample Ratio Mismatch (SRM) chi-square test, Overall Evaluation Criterion (OEC), sample size sizing, Twyman's law, and guardrail metrics. Triggers: kohavi-ab-experimentation, ab-testing, online-controlled-experiments, srm, sample-ratio-mismatch, chi-square-srm, oec, minimum-detectable-effect, twymans-law.",
        r#"# Analytics Kohavi Ab Experimentation
> Based on **Trustworthy Online Controlled Experiments - Ronny Kohavi, Diane Tang, Ya Xu**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- A/B Experiment Registry & SRM Guardrails
CREATE TABLE experiment_runs (
    experiment_id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    control_ratio DOUBLE PRECISION NOT NULL DEFAULT 0.5,
    treatment_ratio DOUBLE PRECISION NOT NULL DEFAULT 0.5,
    control_count BIGINT NOT NULL DEFAULT 0,
    treatment_count BIGINT NOT NULL DEFAULT 0,
    srm_p_value DOUBLE PRECISION,
    srm_flagged BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('ACTIVE', 'CONCLUDED', 'INVALID_SRM'))
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Sample Ratio Mismatch (SRM) Chi-Square Invariant
Let $O_C, O_T$ be observed traffic counts and $E_C, E_T$ be expected counts based on design ratios $r_C, r_T$:
$$E_C = (O_C + O_T) \times r_C, \quad E_T = (O_C + O_T) \times r_T$$
$$\chi^2 = \frac{(O_C - E_C)^2}{E_C} + \frac{(O_T - E_T)^2}{E_T} \sim \chi^2(1)$$
If $p = P(\chi^2(1) \ge \chi^2_{\text{obs}}) < 0.001$, experiment is invalid due to SRM. All downstream metrics are VOID.

### 2.2 Sample Size Determination per Variant
For two-sided test with significance $\alpha = 0.05$ ($z_{1-\alpha/2} = 1.96$) and power $1 - \beta = 0.80$ ($z_{1-\beta} = 0.84$):
$$n \approx \frac{16 \sigma^2}{\Delta^2}$$
where $\Delta = \mu_T - \mu_C$ is the Minimum Detectable Effect (MDE).

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Traffic[User Visits Site] --> Hash[MD5 User ID + Salt]
    Hash --> Modulo{Bucket <= Ratio?}
    Modulo -->|Control| VariantA[Assign Control]
    Modulo -->|Treatment| VariantB[Assign Treatment]
    VariantA --> LogEvent[Log Allocation Event]
    VariantB --> LogEvent
    LogEvent --> DailySRM[Daily SRM Chi-Square Check]
    DailySRM -->|p < 0.001| Abort[INVALIDATE: Abort Experiment]
    DailySRM -->|p >= 0.001| ComputeOEC[Compute OEC & Guardrail Metrics]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from scipy.stats import chisquare

def verify_srm(obs_control: int, obs_treatment: int, p_control: float = 0.5) -> dict:
    total = obs_control + obs_treatment
    exp_control = total * p_control
    exp_treatment = total * (1.0 - p_control)
    chi2_stat, p_val = chisquare([obs_control, obs_treatment], [exp_control, exp_treatment])
    is_srm = p_val < 0.001
    return {"chi2": chi2_stat, "p_value": p_val, "srm_detected": is_srm}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Always run the SRM Chi-Square test before analyzing A/B experiment results.
- If SRM p-value < 0.001, halt the experiment; never attempt to interpret metric deltas.
- Twyman's Law: Any statistic that appears extraordinarily positive or negative is almost certainly an error.
- Enforce sample size requirements prior to launch: n >= 16 * sigma^2 / MDE^2.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect an enterprise experimentation platform:
1. Build automated SRM detection pipelines terminating skewed experiments (p < 0.001).
2. Formulate composite OEC (Overall Evaluation Criterion) functions balancing conversion vs latency.
3. Enforce sequential testing and variance reduction (CUPED) to accelerate decision cycles.
```
"#,
    )
}

/// 134. analytics-causal-mixtape Skill
pub fn analytics_causal_mixtape() -> EccSkill {
    EccSkill::new(
        "analytics-causal-mixtape",
        "Causal identification: potential outcomes framework, Average Treatment Effect (ATE), selection bias decomposition, instrumental variables, and regression discontinuity. Triggers: causal-mixtape, causal-inference, rubin-causal-model, potential-outcomes, ate, att, selection-bias, instrumental-variables, regression-discontinuity.",
        r#"# Analytics Causal Mixtape
> Based on **Causal Inference: The Mixtape - Scott Cunningham**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Causal Analysis Cohort Data Table
CREATE TABLE causal_cohort_observations (
    entity_id BIGINT PRIMARY KEY,
    treatment_assigned INT NOT NULL CHECK (treatment_assigned IN (0, 1)),
    observed_outcome DOUBLE PRECISION NOT NULL,
    running_variable DOUBLE PRECISION, -- For Regression Discontinuity
    instrument DOUBLE PRECISION -- For Instrumental Variables
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Potential Outcomes & Selection Bias Decomposition
Let $Y_i(1)$ and $Y_i(0)$ be potential outcomes under treatment and control:
$$E[Y | D = 1] - E[Y | D = 0] = \underbrace{E[Y(1) - Y(0)]}_{\text{ATE}} + \underbrace{\{E[Y(0) | D=1] - E[Y(0) | D=0]\}}_{\text{Selection Bias}}$$
In randomized experiments, assignment $D \perp (Y(1), Y(0)) \implies \text{Selection Bias} = 0$.

### 2.2 Wald Instrumental Variable Estimator
For binary instrument $Z$, treatment $D$, and outcome $Y$:
$$\hat{\beta}_{\text{IV}} = \frac{E[Y | Z = 1] - E[Y | Z = 0]}{E[D | Z = 1] - E[D | Z = 0]} = \frac{\text{Cov}(Y, Z)}{\text{Cov}(D, Z)}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    ObservedDiff[Observed Group Difference] --> CheckRandom{Randomized Assignment?}
    CheckRandom -->|Yes| ValidATE[Selection Bias = 0: Observed Diff = ATE]
    CheckRandom -->|No| Identify[Causal Identification Strategy]
    Identify --> IV[Instrumental Variables: Z -> D -> Y]
    Identify --> RDD[Regression Discontinuity: Cutoff c]
    Identify --> Match[Propensity Score Matching]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Instrumental Variable Wald Estimator in Python
import numpy as np

def wald_iv_estimator(z: np.ndarray, d: np.ndarray, y: np.ndarray) -> float:
    delta_y = np.mean(y[z == 1]) - np.mean(y[z == 0])
    delta_d = np.mean(d[z == 1]) - np.mean(d[z == 0])
    if abs(delta_d) < 1e-6:
        raise ValueError("Weak instrument: zero compliance effect")
    return delta_y / delta_d
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Observed difference in means equals ATE plus Selection Bias; observational studies contain selection bias.
- An instrument Z must satisfy Relevance (Cov(Z, D) != 0) and Exclusion Restriction (Cov(Z, epsilon) = 0).
- Regression Discontinuity compares entities immediately above and below an arbitrary threshold cutoff.
- Never interpret raw correlation as causation without an explicit identification strategy.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build production causal inference engines:
1. Implement potential outcomes estimation with explicit selection bias diagnostics.
2. Build Two-Stage Least Squares (2SLS) and Wald instrumental variable estimators with weak instrument F-tests.
3. Design Sharp and Fuzzy Regression Discontinuity (RDD) pipelines with optimal bandwidth selection.
```
"#,
    )
}

/// 135. analytics-econometric-causality Skill
pub fn analytics_econometric_causality() -> EccSkill {
    EccSkill::new(
        "analytics-econometric-causality",
        "Applied econometrics: Difference-in-Differences (DiD), parallel trends testing, Two-Way Fixed Effects (TWFE), omitted variable bias formula, and cluster-robust standard errors. Triggers: econometric-causality, mostly-harmless-econometrics, difference-in-differences, did-estimator, omitted-variable-bias, 2sls, parallel-trends.",
        r#"# Analytics Econometric Causality
> Based on **Mostly Harmless Econometrics - Joshua Angrist & Jörn-Steffen Pischke**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Panel Data Table for Difference-in-Differences
CREATE TABLE panel_observations (
    entity_id BIGINT NOT NULL,
    time_period INT NOT NULL,
    is_treated_group INT NOT NULL CHECK (is_treated_group IN (0, 1)),
    is_post_period INT NOT NULL CHECK (is_post_period IN (0, 1)),
    outcome DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (entity_id, time_period)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Difference-in-Differences (DiD) Estimator
Let $\bar{Y}_{G, T}$ be the sample mean of group $G \in \{T, C\}$ at time $T \in \{1, 2\}$:
$$\hat{\delta}_{\text{DiD}} = (\bar{Y}_{T, 2} - \bar{Y}_{T, 1}) - (\bar{Y}_{C, 2} - \bar{Y}_{C, 1})$$
Regression specification:
$$Y_{it} = \alpha + \beta \cdot \text{Treated}_i + \gamma \cdot \text{Post}_t + \delta_{\text{DiD}} (\text{Treated}_i \times \text{Post}_t) + \epsilon_{it}$$

### 2.2 Omitted Variable Bias (OVB) Invariant
$$\hat{\beta}_{\text{short}} = \beta_{\text{long}} + \gamma \frac{\text{Cov}(X, Z)}{\text{Var}(X)}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    PrePeriod[Pre-Treatment Periods 1..k] --> TestParallel{Parallel Pre-Trends Test}
    TestParallel -->|Reject: Non-parallel| Invalidate[DiD Invalid: Trends diverging before treatment]
    TestParallel -->|Accept: Parallel trends| Intervention[Intervention Occurs at Period k+1]
    Intervention --> PostPeriod[Post-Treatment Periods k+1..T]
    PostPeriod --> EstimateDiD[Compute (Delta Y_treat) - (Delta Y_control)]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def calculate_did(y_t1: float, y_t2: float, y_c1: float, y_c2: float) -> float:
    # DiD = (Y_T2 - Y_T1) - (Y_C2 - Y_C1)
    treat_diff = y_t2 - y_t1
    control_diff = y_c2 - y_c1
    return treat_diff - control_diff
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- DiD formula: (Y_treatment_post - Y_treatment_pre) - (Y_control_post - Y_control_pre).
- The fundamental identifying assumption is Parallel Trends: treatment and control must track identically in pre-period.
- Always cluster standard errors at the state/entity level to prevent deflated p-values from autocorrelation.
- Omitted Variable Bias = (Relationship of omitted with Y) * (Regression of omitted on X).
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement an econometric causal analysis suite:
1. Build automated Difference-in-Differences estimators with event-study pre-trend diagnostic tests.
2. Formulate Two-Way Fixed Effects regressions with cluster-robust sandwich covariance estimators.
3. Validate parallel trends robustness against staggered rollouts using Callaway-Sant'Anna estimators.
```
"#,
    )
}

/// 136. analytics-counterfactual-causality Skill
pub fn analytics_counterfactual_causality() -> EccSkill {
    EccSkill::new(
        "analytics-counterfactual-causality",
        "Counterfactual causal theory: identifiability conditions (Exchangeability, Positivity, Consistency), Inverse Probability Weighting (IPW), marginal structural models, and g-computation. Triggers: counterfactual-causality, causal-inference-what-if, exchangeability, positivity, consistency, inverse-probability-weighting, ipw, g-methods.",
        r#"# Analytics Counterfactual Causality
> Based on **Causal Inference: What If - Miguel Hernán & James Robins**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Propensity Scores & IPW Weights Table
CREATE TABLE propensity_weighted_cohort (
    subject_id BIGINT PRIMARY KEY,
    treatment INT NOT NULL CHECK (treatment IN (0, 1)),
    propensity_score DOUBLE PRECISION NOT NULL CHECK (propensity_score > 0.0 AND propensity_score < 1.0),
    ipw_weight DOUBLE PRECISION NOT NULL,
    outcome DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Identifiability Conditions
1. **Conditional Exchangeability**: $Y(a) \perp A \mid L$
2. **Positivity**: $P(A = a \mid L = l) > 0 \quad \forall a, l$
3. **Consistency**: If $A = a$, then $Y = Y(a)$

### 2.2 Inverse Probability Weighting (IPW) Invariant
Let $e(L) = P(A = 1 \mid L)$ be the propensity score:
$$W = \frac{A}{e(L)} + \frac{1 - A}{1 - e(L)}$$
In the pseudo-population weighted by $W$, treatment assignment is unconfounded by $L$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Confounders[Measure Confounders L] --> FitPropensity[Fit Logistic Regression: P(A=1|L)]
    FitPropensity --> CheckPositivity{Verify Positivity: 0.01 < e(L) < 0.99}
    CheckPositivity -->|Fail| Trim[Trim Non-Overlapping Support]
    CheckPositivity -->|Pass| CalcWeights[Calculate IPW Weights W]
    CalcWeights --> FitMSM[Fit Marginal Structural Model on Weighted Cohort]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def compute_ipw_weights(treatment: np.ndarray, prop_score: np.ndarray) -> np.ndarray:
    # Truncate to prevent extreme weights
    ps = np.clip(prop_score, 0.01, 0.99)
    weights = treatment / ps + (1 - treatment) / (1 - ps)
    return weights
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Identifiability requires 3 pillars: Exchangeability (no unmeasured confounding), Positivity, Consistency.
- Inverse Probability Weighting: W = A / e(L) + (1 - A) / (1 - e(L)).
- Clip extreme propensity scores (e.g. [0.01, 0.99]) to avoid high-variance weight explosion.
- Check covariate balance post-weighting: standardized mean difference should be < 0.1 for all features.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build counterfactual estimation pipelines:
1. Implement Inverse Probability Weighting (IPW) with stabilized weights and covariate balance checks.
2. Detect positivity violations and support overlap failures across high-dimensional confounders.
3. Formulate G-computation algorithms simulating multi-stage dynamic treatment regimes.
```
"#,
    )
}

/// 137. analytics-kleppmann-data-intensive Skill
pub fn analytics_kleppmann_data_intensive() -> EccSkill {
    EccSkill::new(
        "analytics-kleppmann-data-intensive",
        "Data systems architecture: LSM-trees vs B-trees, replication quorums, write-skew anomalies, partitioning schemes, and event sourcing. Triggers: kleppmann-data-intensive, data-intensive-applications, lsm-trees, sstables, b-trees, quorum-consensus, write-skew, replication-partitioning.",
        r#"# Analytics Kleppmann Data Intensive
> Based on **Designing Data-Intensive Applications - Martin Kleppmann**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Replication Quorum Node Metadata
CREATE TABLE quorum_cluster_nodes (
    node_id INT PRIMARY KEY,
    datacenter VARCHAR(50) NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('LEADER', 'FOLLOWER')),
    last_applied_index BIGINT NOT NULL,
    is_alive BOOLEAN NOT NULL DEFAULT TRUE
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Quorum Consensus Invariant
In a cluster of $N$ replicas with read quorum $R$ and write quorum $W$:
$$R + W > N$$
Guarantees that at least one node in the read set $R$ contains the latest write from $W$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Write[Incoming Write] --> WAL[Append to Write-Ahead Log]
    WAL --> MemTable[Write to In-Memory MemTable]
    MemTable --> Full{MemTable Full?}
    Full -->|Yes| Flush[Flush to Immutable SSTable on Disk]
    Full -->|No| Ack[Return Ack to Client]
    Flush --> Compaction[Background Leveled / Size-Tiered Compaction]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust Quorum Overlap Validator
pub fn validate_quorum(n: usize, r: usize, w: usize) -> bool {
    r + w > n && w > n / 2
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Quorum consensus condition: R + W > N ensures read quorum contains at least one up-to-date replica.
- LSM-Trees (Log-Structured Merge-trees) optimize sequential writes; B-Trees optimize random reads.
- Prevent write-skew race conditions under Snapshot Isolation using explicit row locking or serializable isolation.
- Change Data Capture (CDC) turns operational database commit logs into deterministic event streams.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect reliable, high-throughput distributed data backends:
1. Configure leaderless and leader-based replication topologies with strict quorum validation (R + W > N).
2. Tune LSM-tree SSTable compaction algorithms (leveled vs size-tiered) to balance write amplification.
3. Design CDC-driven event-sourced pipelines ensuring zero loss and deterministic ordering.
```
"#,
    )
}

/// 138. analytics-akidau-stream-processing Skill
pub fn analytics_akidau_stream_processing() -> EccSkill {
    EccSkill::new(
        "analytics-akidau-stream-processing",
        "Stream processing primitives: What, Where, When, How questions; event-time watermarking, tumbling/sliding/session windows, triggers, and late data accumulation. Triggers: akidau-stream-processing, stream-processing, streaming-systems, event-time-watermarks, tumbling-windows, sliding-windows, session-windows, triggers-accumulation.",
        r#"# Analytics Akidau Stream Processing
> Based on **Streaming Systems - Tyler Akidau, Slava Chernyak, Reuven Lax**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Stream Watermark & Window State Tracking
CREATE TABLE stream_watermark_state (
    partition_id INT PRIMARY KEY,
    current_watermark TIMESTAMP NOT NULL,
    max_event_time_seen TIMESTAMP NOT NULL,
    allowed_lateness_sec INT NOT NULL DEFAULT 300
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Watermark Monotonicity Invariant
Let $W(t)$ be the event-time watermark at processing time $t$:
$$W(t) = \max_{i} (E_i) - \Delta_{\text{skew}}$$
$$W(t_2) \ge W(t_1) \quad \forall t_2 > t_1 \quad (\text{Watermark never moves backwards})$$
Late data condition: Event $e$ is late iff $\text{event\_time}(e) < W(t)$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Event[Incoming Stream Event] --> ExtractTime[Extract Event Timestamp]
    ExtractTime --> WatermarkCheck{Event Time < Current Watermark?}
    WatermarkCheck -->|Yes| LateData[Route to Late Data / Retract Trigger]
    WatermarkCheck -->|No| WindowAssign[Assign to Tumbling / Sliding Window]
    WindowAssign --> Emit{Watermark passes Window End?}
    Emit -->|Yes| Flush[Emit Window Aggregation Result]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust Monotonic Watermark Tracker
pub struct WatermarkTracker {
    current_watermark: u64,
    allowed_lateness_ms: u64,
}

impl WatermarkTracker {
    pub fn update(&mut self, event_time_ms: u64) -> u64 {
        if event_time_ms > self.allowed_lateness_ms {
            let proposed = event_time_ms - self.allowed_lateness_ms;
            if proposed > self.current_watermark {
                self.current_watermark = proposed;
            }
        }
        self.current_watermark
    }
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- The 4 questions: What is computed (transforms)? Where in event time (windowing)? When in processing time (watermarks)? How results relate (accumulating/retracting)?
- Watermarks must be monotonically increasing; they establish completeness guarantees in event time.
- Tumbling windows partition time discretely; session windows dynamically merge on user inactivity gaps.
- Handle late arrivals explicitly using allowed lateness windows and retraction streams.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design real-time stream processing architectures:
1. Implement heuristic and punctuated event-time watermarking engines in Rust/Flink.
2. Build dynamic sessionization windows merging overlapping state upon out-of-order event arrival.
3. Configure accumulating and retracting triggers emitting low-latency speculative updates.
```
"#,
    )
}

/// 139. analytics-kafka-event-streaming Skill
pub fn analytics_kafka_event_streaming() -> EccSkill {
    EccSkill::new(
        "analytics-kafka-event-streaming",
        "Apache Kafka event streaming: partition topologies, consumer group rebalancing, commit offsets, log compaction, and Exactly-Once Semantics (EOS). Triggers: kafka-event-streaming, apache-kafka, event-streaming, kafka-partitions, consumer-groups, exactly-once-semantics, kafka-eos, transactional-producer.",
        r#"# Analytics Kafka Event Streaming
> Based on **Kafka: The Definitive Guide - Gwen Shapira et al.**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Kafka Consumer Offset Checkpoint Table
CREATE TABLE kafka_consumer_offsets (
    consumer_group VARCHAR(100) NOT NULL,
    topic VARCHAR(100) NOT NULL,
    partition_id INT NOT NULL,
    committed_offset BIGINT NOT NULL,
    last_heartbeat TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (consumer_group, topic, partition_id)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Exactly-Once Semantics (EOS) Sequence Invariant
For producer with Producer ID ($PID$) sending message with sequence number $seq$:
$$\text{Broker Acceptance Condition}: \quad seq = seq_{\text{last}} + 1$$
If $seq \le seq_{\text{last}}$, message is rejected as duplicate. If $seq > seq_{\text{last}} + 1$, broker raises out-of-order error.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Prod[Transactional Producer] --> Begin[beginTransaction]
    Begin --> Send[Send messages to Partition P1, P2]
    Send --> Offsets[sendOffsetsToTransaction]
    Offsets --> Commit[commitTransaction: 2-Phase Commit]
    Commit --> WriteMarker[Write COMMIT marker to log]
    WriteMarker --> Consumers[Read_Committed Consumers read data]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Python Kafka Idempotent Producer Configuration
producer_config = {
    'bootstrap.servers': 'localhost:9092',
    'enable.idempotence': True,
    'acks': 'all',
    'retries': 10000000,
    'max.in.flight.requests.per.connection': 5
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Set enable.idempotence=True and acks=all to guarantee zero message duplication and zero data loss.
- Partition key determines ordering: Kafka guarantees strict ordering within a single partition only.
- Consumer lag (LogEndOffset - CurrentOffset) is the primary operational health metric.
- Use transactional producer (read-process-write) for end-to-end Exactly-Once Semantics.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Construct production Apache Kafka event architectures:
1. Size and distribute partition counts matching analytical consumer parallelism requirements.
2. Implement transactional producers achieving Exactly-Once Semantics (EOS) across topic hops.
3. Build consumer lag monitoring systems automatically remediating rebalance storms.
```
"#,
    )
}

/// 140. analytics-data-engineering-lifecycle Skill
pub fn analytics_data_engineering_lifecycle() -> EccSkill {
    EccSkill::new(
        "analytics-data-engineering-lifecycle",
        "Data engineering lifecycle: generation, storage, ingestion, transformation, and serving; architecture tradeoffs across batch vs streaming. Triggers: data-engineering-lifecycle, fundamentals-of-data-engineering, data-ingestion-storage, serving-transformation, batch-vs-streaming, data-architecture-tradeoffs.",
        r#"# Analytics Data Engineering Lifecycle
> Based on **Fundamentals of Data Engineering - Joe Reis & Matt Housley**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Pipeline SLA Performance Metadata
CREATE TABLE pipeline_sla_metrics (
    pipeline_id VARCHAR(100) PRIMARY KEY,
    cadence VARCHAR(20) NOT NULL CHECK (cadence IN ('STREAMING', 'MICROBATCH', 'HOURLY', 'DAILY')),
    target_sla_sec INT NOT NULL,
    actual_latency_sec INT NOT NULL,
    cost_per_run_usd NUMERIC(10, 4) NOT NULL,
    last_success_timestamp TIMESTAMP NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Latency vs Throughput Tradeoff
Let $B$ be batch size and $T_{\text{overhead}}$ be per-batch fixed overhead:
$$\text{Throughput}(B) = \frac{B}{B \cdot t_{\text{item}} + T_{\text{overhead}}}$$
As $B \to \infty$, throughput reaches maximum $\frac{1}{t_{\text{item}}}$, but latency increases linearly with $B$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph LR
    Source[Generation: Apps, DBs] --> Ingest[Ingestion: Batch / Streaming]
    Ingest --> Storage[Storage: Object Store / Lakehouse]
    Storage --> Transform[Transformation: SQL, dbt, Spark]
    Transform --> Serving[Serving: Analytics, ML, Reverse ETL]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Batch vs Streaming Tradeoff Decision Matrix
def choose_pipeline_architecture(max_tolerable_latency_sec: int, budget_tier: str) -> str:
    if max_tolerable_latency_sec < 60:
        return "STREAMING_EVENT_DRIVEN"
    elif max_tolerable_latency_sec < 3600 and budget_tier != "LOW":
        return "MICROBATCH_5MIN"
    else:
        return "SCHEDULED_BATCH_HOURLY" 
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- The 5 lifecycle stages: Generation -> Ingestion -> Storage -> Transformation -> Serving.
- Address undercurrents throughout: Security, Data Management, DataOps, Architecture, Orchestration.
- Choose batch by default unless business value demonstrably requires sub-minute streaming latency.
- Design every transformation step to be idempotent: re-running never produces duplicate data.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Formulate end-to-end data engineering lifecycle blueprints:
1. Conduct objective latency vs cost tradeoff evaluations between streaming and micro-batch architectures.
2. Design secure, cost-optimized object storage lakehouse tiering (Bronze/Silver/Gold).
3. Implement automated orchestration DAGs embedding DataOps and governance undercurrents.
```
"#,
    )
}

/// 141. analytics-realtime-olap-pipelines Skill
pub fn analytics_realtime_olap_pipelines() -> EccSkill {
    EccSkill::new(
        "analytics-realtime-olap-pipelines",
        "Real-time analytical OLAP pipelines: ClickHouse, Apache Pinot, StarRocks, vectorized columnar execution, MergeTree engines, and materialized views. Triggers: realtime-olap-pipelines, clickhouse, pinot, starrocks, realtime-olap, columnar-mergetree, materialized-views, streaming-ingestion.",
        r#"# Analytics Realtime Olap Pipelines
> Based on **Building Real-Time Data Pipelines - Gerard Maas & François Garillot**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- ClickHouse Real-Time OLAP DDL
CREATE TABLE default.events_stream (
    event_time DateTime64(3),
    user_id UInt64,
    event_type LowCardinality(String),
    cost Float64
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(event_time)
ORDER BY (event_type, user_id, event_time)
SETTINGS index_granularity = 8192;

CREATE MATERIALIZED VIEW default.events_hourly_mv
ENGINE = SummingMergeTree()
PRIMARY KEY (toStartOfHour(event_time), event_type)
AS SELECT 
    toStartOfHour(event_time) AS hour,
    event_type,
    count() AS total_events,
    sum(cost) AS total_cost
FROM default.events_stream
GROUP BY hour, event_type;
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Sparse Index Granularity Invariant
Let $N$ be total row count and $G = 8192$ be index granularity:
$$\text{Number of Index Marks} = \left\lceil \frac{N}{8192} \right\rceil$$
Binary search over primary index marks runs in $O(\log_2(N/G))$ memory buffer operations.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Kafka[Kafka Topic Stream] --> Buffer[ClickHouse In-Memory Buffer Part]
    Buffer --> WritePart[Write Compressed Columnar Part to Disk]
    WritePart --> MergeParts[Background MergeTree Engine Compaction]
    MergeParts --> ReadQuery[Sub-second Analytical Vectorized Aggregations]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- ClickHouse Query with SummingMergeTree final state
SELECT hour, event_type, sum(total_events), sum(total_cost)
FROM default.events_hourly_mv
GROUP BY hour, event_type;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Use MergeTree engines in ClickHouse; choose primary key ordering to maximize compression and filter skipping.
- SummingMergeTree and AggregatingMergeTree perform pre-aggregations during background part merges.
- LowCardinality(String) dictionary-encodes repetitive string columns, slashing memory by 80%.
- Stream into buffer tables or ingest in micro-batches (>= 1,000 rows) to avoid creating tiny disk parts.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build sub-second real-time OLAP systems with ClickHouse/Pinot:
1. Design optimized sparse primary keys matching the dominant query filter and group-by predicates.
2. Deploy materialized views with AggregatingMergeTree engines for pre-computed metric rollups.
3. Architect Kafka streaming ingestion pipelines buffering micro-batches to prevent part explosion.
```
"#,
    )
}

/// 142. analytics-python-pandas-wrangling Skill
pub fn analytics_python_pandas_wrangling() -> EccSkill {
    EccSkill::new(
        "analytics-python-pandas-wrangling",
        "High-performance data wrangling in pandas: split-apply-combine, memory downcasting, categorical types, MultiIndex operations, and vectorized transformations. Triggers: python-pandas-wrangling, pandas-data-wrangling, split-apply-combine, groupby-aggregations, memory-downcasting, categorical-types, multiindex.",
        r#"# Analytics Python Pandas Wrangling
> Based on **Python for Data Analysis - Wes McKinney**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Analytical Aggregation Table Result
CREATE TABLE pandas_agg_results (
    cohort_month VARCHAR(7) NOT NULL,
    channel VARCHAR(50) NOT NULL,
    active_users BIGINT NOT NULL,
    total_spend NUMERIC(14, 2) NOT NULL,
    retention_rate_d30 NUMERIC(6, 4) NOT NULL,
    PRIMARY KEY (cohort_month, channel)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Split-Apply-Combine Partition Invariant
Let dataframe $D$ be partitioned by key set $K = \{k_1, \dots, k_m\}$:
$$\bigcup_{i=1}^m D_{k_i} = D \quad \text{and} \quad D_{k_i} \cap D_{k_j} = \emptyset \quad \forall i \neq j$$
$$\sum_{i=1}^m |D_{k_i}| = |D|$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawDF[Raw Large DataFrame] --> Downcast[Downcast int64->int32, float64->float32]
    Downcast --> Categorize[Convert Low-Cardinality object->category]
    Categorize --> GroupBy[Split: GroupBy Cohort & Channel]
    GroupBy --> Apply[Apply Vectorized Aggregations: sum, mean]
    Apply --> Combine[Combine into Compact Analytical Summary]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import pandas as pd

def optimize_pandas_memory(df: pd.DataFrame) -> pd.DataFrame:
    for col in df.columns:
        if df[col].dtype == 'object' and df[col].nunique() / len(df) < 0.5:
            df[col] = df[col].astype('category')
        elif df[col].dtype == 'int64':
            df[col] = pd.to_numeric(df[col], downcast='integer')
        elif df[col].dtype == 'float64':
            df[col] = pd.to_numeric(df[col], downcast='float')
    return df
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Convert low-cardinality string columns (nunique / len < 0.5) to category to slash RAM usage.
- Downcast int64 -> int32/int16 and float64 -> float32; cuts memory consumption up to 75%.
- Avoid row-by-row iteration (iterrows); always use vectorized expressions or groupby agg.
- Chain operations using pipe() for readable, functional data transformation pipelines.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build enterprise Python pandas data manipulation pipelines:
1. Implement automated memory profiling and type downcasting routines for large-scale datasets.
2. Build multi-level index pivot tables and windowed aggregations using vectorized groupby idioms.
3. Benchmark and refactor bottlenecks utilizing PyArrow-backed pandas 2.0 backend engines.
```
"#,
    )
}

/// 143. analytics-duckdb-embedded-olap Skill
pub fn analytics_duckdb_embedded_olap() -> EccSkill {
    EccSkill::new(
        "analytics-duckdb-embedded-olap",
        "Embedded in-memory OLAP with DuckDB: vectorized execution, direct Parquet/CSV querying without loading, zero-copy Arrow integration, and out-of-core streaming. Triggers: duckdb-embedded-olap, duckdb, embedded-olap, parquet-querying, columnar-vectorized, arrow-zero-copy, out-of-core-processing.",
        r#"# Analytics Duckdb Embedded Olap
> Based on **DuckDB in Action - Mark Needham & Michael Hunger**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- DuckDB Analytical Persistent Catalog
CREATE TABLE parquet_metadata_cache (
    file_path VARCHAR PRIMARY KEY,
    row_count BIGINT NOT NULL,
    min_timestamp TIMESTAMP,
    max_timestamp TIMESTAMP,
    file_size_bytes BIGINT NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Parquet Statistics Min/Max Pushdown
For query with filter condition $X > c$ over Parquet row group $RG_j$:
$$\text{If } \max_{x \in RG_j}(X) \le c, \quad \text{Skip reading entire Row Group } RG_j$$
Eliminates I/O for non-matching data partitions entirely.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    ParquetFiles[Parquet Files on S3/Disk] --> Pushdown[Pushdown Projection & Predicates]
    Pushdown --> VectorBatch[Stream in 2048-row Vector Chunks]
    VectorBatch --> SIMD[SIMD Vectorized Query Execution]
    SIMD --> ArrowOut[Zero-Copy Export to Apache Arrow]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- DuckDB In-Place Parquet Aggregation
SELECT 
    vendor_id,
    date_trunc('month', pickup_datetime) AS month,
    count(*) AS trip_count,
    avg(trip_distance) AS avg_dist,
    sum(total_amount) AS revenue
FROM read_parquet('s3://taxi-data/parquet/*.parquet')
WHERE passenger_count > 0
GROUP BY ALL
ORDER BY month, revenue DESC;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Query Parquet files directly using read_parquet('*.parquet') without loading data into tables.
- DuckDB executes in vectorized 2048-row chunks with automatic SIMD hardware acceleration.
- Use GROUP BY ALL to automatically infer grouping keys from the SELECT list.
- Interoperate with Polars, Pandas, and Arrow using zero-copy in-process pointers.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect embedded OLAP analytical runtimes using DuckDB:
1. Build direct-query engines over Parquet data lakes utilizing min/max column statistics pushdown.
2. Integrate DuckDB with Apache Arrow for zero-copy memory transfers into local ML pipelines.
3. Configure out-of-core spilling thresholds to process datasets larger than physical RAM.
```
"#,
    )
}

/// 144. analytics-polars-lazy-processing Skill
pub fn analytics_polars_lazy_processing() -> EccSkill {
    EccSkill::new(
        "analytics-polars-lazy-processing",
        "Rust-native Polars analytics: LazyFrame query planning, predicate/projection/slice pushdown, parallel expression execution, and streaming engines. Triggers: polars-lazy-processing, polars, polars-rust, lazyframe-optimizer, predicate-pushdown, projection-pushdown, arrow-native.",
        r#"# Analytics Polars Lazy Processing
> Based on **Data Analysis with Polars - Jeroen Janssens**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Polars Pipeline Execution Trace Log
CREATE TABLE polars_job_metrics (
    job_id UUID PRIMARY KEY,
    rows_scanned BIGINT NOT NULL,
    rows_output BIGINT NOT NULL,
    execution_time_ms BIGINT NOT NULL,
    peak_memory_mb DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Relational Algebra Pushdown Optimization
Let $\sigma_p$ be predicate filter and $\pi_C$ be column projection:
$$\pi_C(\sigma_p(R)) \equiv \pi_C(\sigma_p(\pi_{C \cup \text{Vars}(p)}(R)))$$
Polars reads only columns $C \cup \text{Vars}(p)$ from disk, discarding non-matching rows before materialization.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    LazyCode[LazyFrame: .filter().select().groupby()] --> Plan[Construct Logical Plan]
    Plan --> Optimize[Query Optimizer: Pushdown Predicates & Projections]
    Optimize --> Physical[Construct Physical Plan with Rayon Threads]
    Physical --> Collect[Execute .collect() across CPU Cores]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import polars as pl

q = (
    pl.scan_parquet("data/*.parquet")
    .filter(pl.col("status") == "COMPLETED")
    .group_by(["cohort", "region"])
    .agg([
        pl.col("revenue").sum().alias("total_rev"),
        pl.col("user_id").n_unique().alias("unique_users")
    ])
    .sort("total_rev", descending=True)
)
# df = q.collect(streaming=True)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Always use scan_parquet() and LazyFrame; call .collect() only at the final step.
- Polars optimizer automatically re-orders operations: filters and column selections are pushed to source.
- Never use Python loops or apply(); use Polars native expression contexts (select, with_columns).
- Enable streaming=True in .collect() to process datasets that exceed available RAM.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design high-performance analytical engines using Polars and Apache Arrow:
1. Formulate lazy query plans utilizing projection and predicate pushdown for optimal I/O throughput.
2. Build multi-threaded aggregation pipelines scaling linearly across CPU cores with Rayon.
3. Deploy Polars streaming mode to execute out-of-core transforms on multi-gigabyte datasets.
```
"#,
    )
}

/// 145. analytics-high-performance-compute Skill
pub fn analytics_high_performance_compute() -> EccSkill {
    EccSkill::new(
        "analytics-high-performance-compute",
        "High-performance numerical analytics: profiling bottlenecks, NumPy SIMD vectorization, Numba JIT compilation, multiprocessing, and Amdahl's Law optimization. Triggers: high-performance-compute, high-performance-python, numba-jit, numpy-vectorization, amdahls-law, cython, memory-profiling, gil-bypass.",
        r#"# Analytics High Performance Compute
> Based on **High Performance Python - Micha Gorelick & Ian Ozsvald**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Compute Profiling Benchmark Results
CREATE TABLE compute_benchmarks (
    function_name VARCHAR(100) PRIMARY KEY,
    implementation VARCHAR(50) NOT NULL, -- PURE_PYTHON, NUMPY, NUMBA_JIT
    execution_time_ms DOUBLE PRECISION NOT NULL,
    speedup_factor DOUBLE PRECISION NOT NULL,
    memory_peak_mb DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Amdahl's Law Speedup Limit
For program with parallel fraction $p$ accelerated by factor $s$ across $n$ cores:
$$S(n) = \frac{1}{(1 - p) + \frac{p}{n}}$$
As $n \to \infty$, maximum theoretical speedup is bounded strictly by $\frac{1}{1 - p}$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Identify[Profile Code: cProfile, line_profiler] --> Bottleneck{Found CPU Hotspot?}
    Bottleneck -->|Yes| Vectorize[Vectorize with NumPy SIMD]
    Vectorize --> JIT[Compile Hot Loops with @numba.njit]
    JIT --> Parallel[Scale over Cores via Multiprocessing / Rayon]
    Bottleneck -->|No| IOBound[Optimize I/O: Async or Polars Streaming]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from numba import njit, prange
import numpy as np

@njit(parallel=True, fastmath=True)
def compute_distance_matrix(x: np.ndarray, y: np.ndarray) -> np.ndarray:
    n = len(x)
    dist = np.zeros((n, n), dtype=np.float64)
    for i in prange(n):
        for j in range(n):
            dist[i, j] = np.sqrt((x[i] - x[j])**2 + (y[i] - y[j])**2)
    return dist
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Measure first with cProfile and line_profiler before attempting any code optimization.
- Replace Python loops over numerical arrays with @numba.njit(parallel=True, fastmath=True).
- Amdahl's Law dictates that optimizing a step taking 10% of runtime can never exceed 1.11x total speedup.
- Use zero-copy memory views and contiguous NumPy arrays (C-contiguous) for optimal CPU cache utilization.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Engineer high-throughput quantitative compute runtimes:
1. Conduct granular profiling (CPU instructions, cache misses, memory footprint) across analytical bottlenecks.
2. Implement JIT-compiled kernels (Numba / Cython / Rust FFI) achieving native C-speed execution.
3. Structure parallel tasks bypassing the GIL via multiprocessing and shared-memory Arrow buffers.
```
"#,
    )
}

/// 146. analytics-cli-data-science Skill
pub fn analytics_cli_data_science() -> EccSkill {
    EccSkill::new(
        "analytics-cli-data-science",
        "UNIX command-line analytics: streaming pipes, stream filtering, jq, xsv, csvkit, GNU parallel, and composable command-line pipelines. Triggers: cli-data-science, command-line-analytics, jq-json, csvkit, xsv, gnu-parallel, unix-pipes, reproducible-cli.",
        r#"# Analytics Cli Data Science
> Based on **Data Science on the Command Line - Jeroen Janssens**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- CLI Pipeline Run Tracking
CREATE TABLE cli_job_executions (
    command_hash CHAR(32) PRIMARY KEY,
    raw_command TEXT NOT NULL,
    exit_code INT NOT NULL,
    input_records BIGINT NOT NULL,
    output_records BIGINT NOT NULL,
    execution_time_sec DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Pipe Stream Invariance
Let $f_1, f_2, \dots, f_k$ be stateless stream filter programs:
$$(f_k \circ \dots \circ f_1)(S) = f_k(\dots(f_1(S)))$$
Memory overhead remains strictly $O(1)$ constant buffer size regardless of stream size $|S|$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph LR
    Source[Raw JSON / CSV Stream] --> JQ[jq: Parse & Filter Objects]
    JQ --> XSV[xsv: Select & Slice Columns]
    XSV --> Parallel[parallel: Distribute across Cores]
    Parallel --> Output[Sink: Parquet / Database]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# High-Performance CLI One-Liner Pipeline
# Extract 99th percentile response time per endpoint from 10GB logs
cat access.log \
  | jq -r '[.endpoint, .response_time_ms] | @csv' \
  | xsv select 1,2 \
  | xsv stats --nulls \
  | xsv table
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Compose UNIX command-line tools via standard pipes (stdin -> filter -> stdout).
- Use xsv for blazing-fast CSV slicing, indexing, frequency counts, and joins.
- Use jq for zero-dependency JSON extraction and reshaping.
- Scale multi-core batch processing using GNU parallel: parallel --jobs 8 < jobs.txt.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build reproducible command-line data processing architectures:
1. Construct streaming UNIX pipelines utilizing jq, xsv, and awk operating in constant memory.
2. Parallelize batch processing across multi-core systems using GNU parallel.
3. Package CLI tools into deterministic, containerized pipelines for automated CI/CD validation.
```
"#,
    )
}

/// 147. analytics-tufte-visual-display Skill
pub fn analytics_tufte_visual_display() -> EccSkill {
    EccSkill::new(
        "analytics-tufte-visual-display",
        "Tufte information design: Data-Ink Ratio maximization, chartjunk elimination, Lie Factor calculation, sparklines, small multiples, and graphical integrity. Triggers: tufte-visual-display, data-ink-ratio, tufte, chartjunk-elimination, lie-factor, sparklines, small-multiples, graphical-integrity.",
        r#"# Analytics Tufte Visual Display
> Based on **The Visual Display of Quantitative Information - Edward R. Tufte**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Visualization Audit Metrics
CREATE TABLE chart_integrity_audits (
    chart_id VARCHAR(50) PRIMARY KEY,
    chart_title VARCHAR(100) NOT NULL,
    data_ink_ratio DOUBLE PRECISION NOT NULL CHECK (data_ink_ratio > 0.0 AND data_ink_ratio <= 1.0),
    lie_factor DOUBLE PRECISION NOT NULL,
    has_3d_effects BOOLEAN NOT NULL DEFAULT FALSE,
    has_heavy_gridlines BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('APPROVED', 'REJECTED_CHARTJUNK'))
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Data-Ink Ratio Invariant
$$\text{Data-Ink Ratio} = \frac{\text{Data-Ink}}{\text{Total Ink Used to Print Graphic}} \le 1.0$$
Objective: Maximize data-ink ratio; erase non-data-ink and redundant data-ink.

### 2.2 Lie Factor Invariant
$$\text{Lie Factor} = \frac{\text{Size of Effect Shown in Graphic}}{\text{Size of Effect in Data}} = \frac{\frac{|G_2 - G_1|}{G_1}}{\frac{|D_2 - D_1|}{D_1}}$$
A truthful graphic must have:
$$0.95 \le \text{Lie Factor} \le 1.05$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawChart[Draft Chart] --> AuditInk[Measure Non-Data Ink: 3D, Heavy Grids, Moiré]
    AuditInk --> StripJunk[Strip Chartjunk: Remove borders, soften grid to light gray]
    StripJunk --> CalcLie[Compute Lie Factor: Size Effect Graphic / Size Effect Data]
    CalcLie --> ValidLie{0.95 <= Lie Factor <= 1.05?}
    ValidLie -->|No| RedesignScale[Fix Truncated / Non-Linear Scales]
    ValidLie -->|Yes| Approve[Publish High Data-Ink Graphic]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Tufte Lie Factor Calculator
def calculate_lie_factor(graphic_val_1: float, graphic_val_2: float, data_val_1: float, data_val_2: float) -> float:
    size_effect_graphic = abs(graphic_val_2 - graphic_val_1) / graphic_val_1
    size_effect_data = abs(data_val_2 - data_val_1) / data_val_1
    if size_effect_data == 0:
        return 1.0
    return size_effect_graphic / size_effect_data
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Maximize the Data-Ink Ratio: every drop of ink must represent meaningful quantitative variation.
- Eliminate chartjunk: ban 3D pseudo-perspective, dark grid lines, decorative textures, and useless icons.
- Maintain Lie Factor between 0.95 and 1.05; graphic variations must match data variations.
- Use sparklines (word-sized data graphics) to show dense historical context directly within text.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Deploy Tufte quantitative visual integrity standards across business intelligence:
1. Audit dashboard assets enforcing Data-Ink Ratio maximization and complete chartjunk elimination.
2. Calculate and alert on Lie Factor violations caused by non-zero baselines or non-linear scaling.
3. Implement small multiples and embedded sparklines displaying high-density temporal context.
```
"#,
    )
}

/// 148. analytics-storytelling-with-data Skill
pub fn analytics_storytelling_with_data() -> EccSkill {
    EccSkill::new(
        "analytics-storytelling-with-data",
        "Visual storytelling: preattentive visual attributes, decluttering charts, Gestalt principles of perception, visual hierarchy, and focus-directing design. Triggers: storytelling-with-data, preattentive-attributes, decluttering-charts, gestalt-principles, visual-hierarchy, action-oriented-analytics.",
        r#"# Analytics Storytelling With Data
> Based on **Storytelling with Data - Cole Nussbaumer Knaflic**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Presentation Chart Style Specification
CREATE TABLE viz_style_specs (
    spec_id VARCHAR(50) PRIMARY KEY,
    primary_color CHAR(7) NOT NULL DEFAULT '#1E40AF', -- Intentional focal accent
    neutral_color CHAR(7) NOT NULL DEFAULT '#94A3B8', -- Muted gray background
    alert_color CHAR(7) NOT NULL DEFAULT '#DC2626',
    font_family VARCHAR(50) NOT NULL DEFAULT 'Inter',
    max_accent_elements INT NOT NULL DEFAULT 3
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Sensory Memory Preattentive Threshold
Preattentive visual features (Color Hue, Position, Size) are processed in sensory memory:
$$T_{\text{perception}} < 200 \text{ ms}$$
Invariant: Use at most 1 primary preattentive accent color per visual to prevent cognitive dissonance:
$$\sum \text{Accent Hues} \le 1 \quad (\text{Rest must be muted grays})$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawData[Raw Visualization] --> Step1[Understand the Context & Target Audience]
    Step1 --> Step2[Choose Appropriate Display: Bar, Line, Table]
    Step2 --> Step3[Eliminate Clutter: Remove borders, legends, ticks]
    Step3 --> Step4[Apply Gestalt: Proximity, Similarity, Enclosure]
    Step4 --> Step5[Direct Attention: Apply Preattentive Color Accent]
    Step5 --> Step6[Tell a Story: Action-Oriented Title & Annotations]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Decluttering Matplotlib Style Example
import matplotlib.pyplot as plt

def apply_storytelling_style(ax):
    # Remove top and right spines
    ax.spines['top'].set_visible(False)
    ax.spines['right'].set_visible(False)
    ax.spines['left'].set_color('#CBD5E1')
    ax.spines['bottom'].set_color('#CBD5E1')
    ax.tick_params(colors='#64748B')
    ax.yaxis.grid(True, linestyle='--', alpha=0.5, color='#E2E8F0')
    ax.xaxis.grid(False)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Declutter: remove borders, 3D effects, dark background fills, and redundant axis labels.
- Color must be used intentionally: mute 90% of data in light gray; highlight the focal point in bold blue/coral.
- Replace generic chart titles with action headlines summarizing the key takeaway.
- Leverage Gestalt principles (proximity, similarity, enclosure) to group related data points naturally.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Standardize executive visual communications using Storytelling with Data:
1. Formulate automated chart decluttering templates removing extraneous visual noise and chart borders.
2. Establish strict visual hierarchies utilizing preattentive color encoding (< 200ms processing threshold).
3. Create annotated narrative charts communicating actionable insights directly to decision makers.
```
"#,
    )
}

/// 149. analytics-wilke-data-visualization Skill
pub fn analytics_wilke_data_visualization() -> EccSkill {
    EccSkill::new(
        "analytics-wilke-data-visualization",
        "Visual encoding fundamentals: aesthetic mappings, sequential vs diverging vs qualitative color scales, colorblind safety (Viridis), coordinate projections, and avoiding dual y-axes. Triggers: wilke-data-visualization, color-scale-design, visual-encodings, viridis-colormap, proportions-visuals, avoid-dual-axes, wilke-viz.",
        r#"# Analytics Wilke Data Visualization
> Based on **Fundamentals of Data Visualization - Claus O. Wilke**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Color Scale Registry
CREATE TABLE color_scale_palettes (
    palette_name VARCHAR(50) PRIMARY KEY,
    palette_type VARCHAR(20) NOT NULL CHECK (palette_type IN ('SEQUENTIAL', 'DIVERGING', 'QUALITATIVE')),
    is_colorblind_safe BOOLEAN NOT NULL DEFAULT TRUE,
    hex_values JSONB NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Perceptually Uniform Color Invariant
Let $\Delta E$ be perceptual color difference in CIELAB space and $\Delta y$ be data variation:
$$\frac{\Delta E(c_1, c_2)}{|y_1 - y_2|} \approx \text{constant}$$
Viridis, Inferno, and Cividis maintain strict perceptual uniformity across all color vision proficiencies.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    MetricType{Metric Data Type?}
    MetricType -->|Ordered Continuous| Sequential[Sequential Palette: Light to Dark single hue]
    MetricType -->|Zero-Centered Deviation| Diverging[Diverging Palette: Neutral Midpoint]
    MetricType -->|Unordered Categories| Qualitative[Qualitative Palette: Distinct Hues, Equal Luminance]
    Sequential --> ColorblindCheck{Passes Deuteranopia Simulation?}
    Diverging --> ColorblindCheck
    Qualitative --> ColorblindCheck
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Safe Diverging Colormap Midpoint in Python
import matplotlib.colors as mcolors

def get_diverging_norm(vmin: float, vmax: float, vcenter: float = 0.0):
    return mcolors.TwoSlopeNorm(vmin=vmin, vcenter=vcenter, vmax=vmax)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Use sequential palettes for ordered magnitude; diverging for values with a natural zero midpoint.
- Never use rainbow/jet colormaps; use perceptually uniform Viridis or ColorBrewer palettes.
- Never use dual y-axes with different scales; plot two separate aligned panels instead.
- Verify colorblind accessibility: 8% of men have red-green color vision deficiency.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement rigorous visual encoding standards across enterprise analytics:
1. Deploy perceptually uniform color maps (Viridis, Okabe-Ito) certified for colorblind accessibility.
2. Eliminate misleading dual y-axes by automatically decomposing multi-scale metrics into vertically aligned panels.
3. Validate visual encodings (position, size, color, shape) matching data scale properties.
```
"#,
    )
}

/// 150. analytics-few-dashboard-design Skill
pub fn analytics_few_dashboard_design() -> EccSkill {
    EccSkill::new(
        "analytics-few-dashboard-design",
        "Dashboard UX & visual monitoring: 13 common design mistakes, Bullet Graphs, single-screen display constraint, high data density, and operational alerts. Triggers: few-dashboard-design, bullet-graph, dashboard-ux, stephen-few, high-data-density, visual-monitoring, operational-dashboards.",
        r#"# Analytics Few Dashboard Design
> Based on **Information Dashboard Design - Stephen Few**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Bullet Graph KPI Target Configuration
CREATE TABLE bullet_graph_configs (
    kpi_id VARCHAR(50) PRIMARY KEY,
    title VARCHAR(100) NOT NULL,
    current_value DOUBLE PRECISION NOT NULL,
    target_value DOUBLE PRECISION NOT NULL,
    poor_threshold DOUBLE PRECISION NOT NULL,
    satisfactory_threshold DOUBLE PRECISION NOT NULL,
    max_range DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Bullet Graph Spatial Efficiency Invariant
A bullet graph displays 5 distinct quantitative dimensions within a 1D linear profile:
$$\text{Dimensions} = \{ \text{Actual Value}, \text{Target Marker}, \text{Poor Range}, \text{Satisfactory Range}, \text{Good Range} \}$$
$$\text{Area}_{\text{Bullet}} \le 0.25 \times \text{Area}_{\text{Radial Gauge}}$$
Yields $> 75\%$ screen space reduction while presenting richer operational context.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph LR
    Sub1[Poor Range: 0..60] --> Sub2[Satisfactory: 60..85]
    Sub2 --> Sub3[Good Range: 85..100]
    Sub1 -.-> ActualBar[Actual Performance Bar: 78]
    Sub2 -.-> ActualBar
    Sub2 -.-> TargetLine[Target Marker: 80]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust Bullet Graph Validator
pub struct BulletGraph {
    pub actual: f64,
    pub target: f64,
    pub poor: f64,
    pub satisfactory: f64,
    pub max: f64,
}

impl BulletGraph {
    pub fn is_on_target(&self) -> bool {
        self.actual >= self.target
    }
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Single-screen constraint: all vital operational metrics must fit on one screen without scrolling.
- Replace circular dial gauges with linear Bullet Graphs to save 75% screen space.
- Avoid 13 classic mistakes: excessive detail, inadequate context, useless decoration, pie charts.
- Design for glanceability: alert status (normal, warning, critical) must be instantly perceptible.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Engineer operational dashboards following Stephen Few's principles:
1. Enforce strict single-screen non-scrolling layouts displaying high data density without clutter.
2. Implement custom Bullet Graph components encoding actuals, comparative targets, and qualitative ranges.
3. Establish glanceable operational hierarchies prioritizing immediate exception detection.
```
"#,
    )
}

/// 151. analytics-cairo-visual-integrity Skill
pub fn analytics_cairo_visual_integrity() -> EccSkill {
    EccSkill::new(
        "analytics-cairo-visual-integrity",
        "Visual integrity and deception detection: truncated bar axes, dual-scale manipulation, cherry-picked time windows, and visual uncertainty representation. Triggers: cairo-visual-integrity, how-charts-lie, visual-deception, truncated-axis, dual-scales, zero-baseline, uncertainty-visualization.",
        r#"# Analytics Cairo Visual Integrity
> Based on **How Charts Lie - Alberto Cairo**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Chart Validation Rule Results
CREATE TABLE chart_deception_audits (
    chart_id VARCHAR(50) PRIMARY KEY,
    chart_type VARCHAR(30) NOT NULL,
    y_axis_starts_at_zero BOOLEAN NOT NULL,
    is_scale_truncated BOOLEAN NOT NULL,
    visualizes_uncertainty BOOLEAN NOT NULL,
    audit_verdict VARCHAR(20) NOT NULL CHECK (audit_verdict IN ('PASS', 'FAIL_MISLEADING_AXIS'))
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Bar Chart Zero-Baseline Invariant
For bar charts where magnitude is encoded as bar length/height:
$$y_{\text{baseline}} = 0.0000$$
Truncating the baseline ($y_{\text{min}} > 0$) distorts the visual ratio of lengths:
$$\frac{\text{Length}(A)}{\text{Length}(B)} = \frac{A - y_{\text{min}}}{B - y_{\text{min}}} \neq \frac{A}{B}$$
This produces visual exaggeration and violates graphical truthfulness.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Chart[Inspect Visual Artifact] --> TypeCheck{Is it a Bar Chart?}
    TypeCheck -->|Yes| ZeroCheck{Does Y-axis start at 0?}
    ZeroCheck -->|No| Reject[REJECT: Truncated Bar Chart creates Lie Factor]
    ZeroCheck -->|Yes| CheckDual{Uses Dual Y-Axes?}
    TypeCheck -->|No: Line Chart| CheckDual
    CheckDual -->|Yes| RejectDual[REJECT: Dual axes can arbitrarily scale trends]
    CheckDual -->|No| CheckUncertainty[Verify Confidence Bands / Margins of Error]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Automated Bar Chart Axis Validator
def audit_bar_chart_axes(ymin: float, chart_type: str) -> bool:
    if chart_type.lower() == 'bar':
        if abs(ymin) > 1e-6:
            return False # Fails zero-baseline invariant
    return True
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Bar charts MUST always start at zero; truncating the baseline exaggerates minor differences.
- Line charts do not require zero baseline, but the non-zero origin must be clearly labeled.
- Never use dual axes with different scales; they allow author to manipulate visual intersection points.
- Always display uncertainty: show confidence intervals, margin of error, or hypothetical outcome plots.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Deploy automated chart integrity linters:
1. Build continuous CI tests scanning dashboard Vega/D3 specs for truncated bar chart baselines.
2. Flag deceptive visualizations exhibiting cherry-picked time intervals or uncalibrated dual axes.
3. Enforce visual uncertainty representation (gradient bands, error bars) on all predictive forecasts.
```
"#,
    )
}

/// 152. analytics-d3-interactive-graphics Skill
pub fn analytics_d3_interactive_graphics() -> EccSkill {
    EccSkill::new(
        "analytics-d3-interactive-graphics",
        "Interactive web visualization: D3.js data binding pattern (enter, update, exit), mathematical scales, SVG/Canvas rendering, transitions, and force layouts. Triggers: d3-interactive-graphics, d3js, enter-update-exit, d3-scales, svg-visualization, interactive-dashboards, data-joins.",
        r#"# Analytics D3 Interactive Graphics
> Based on **Interactive Data Visualization for the Web - Scott Murray**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Dynamic Dashboard Widget Layout Registry
CREATE TABLE d3_widget_configs (
    widget_id VARCHAR(50) PRIMARY KEY,
    chart_type VARCHAR(50) NOT NULL,
    width INT NOT NULL,
    height INT NOT NULL,
    margin_json JSONB NOT NULL,
    animation_duration_ms INT NOT NULL DEFAULT 750
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 D3 Linear Scale Mapping Invariant
For domain $[x_{\text{min}}, x_{\text{max}}]$ and screen range $[y_{\text{min}}, y_{\text{max}}]$:
$$f(x) = y_{\text{min}} + \frac{x - x_{\text{min}}}{x_{\text{max}} - x_{\text{min}}} (y_{\text{max}} - y_{\text{min}})$$
Scale function preserves order and linearity:
$$f(x_1) < f(x_2) \iff x_1 < x_2 \quad (\text{for positive range gradient})$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[New Data Array] --> Join[selection.data(data, d => d.id)]
    Join --> Enter[enter(): Create new DOM nodes for new items]
    Join --> Update[update: Transition existing DOM elements to new positions]
    Join --> Exit[exit(): Remove DOM elements with no matching data]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// D3.js General Update Pattern
function updateBars(svg, data, xScale, yScale, height) {
    const bars = svg.selectAll("rect.bar")
        .data(data, d => d.id);

    // Enter
    bars.enter()
        .append("rect")
        .attr("class", "bar")
        .attr("x", d => xScale(d.key))
        .attr("y", height)
        .attr("width", xScale.bandwidth())
        .attr("height", 0)
        .transition().duration(750)
        .attr("y", d => yScale(d.value))
        .attr("height", d => height - yScale(d.value));

    // Update
    bars.transition().duration(750)
        .attr("x", d => xScale(d.key))
        .attr("y", d => yScale(d.value))
        .attr("height", d => height - yScale(d.value));

    // Exit
    bars.exit()
        .transition().duration(750)
        .attr("height", 0)
        .attr("y", height)
        .remove();
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- The D3 data join matches DOM elements to data items using key functions (d => d.id).
- Master the Enter-Update-Exit pattern: enter() appends, update transitions, exit() cleans up DOM.
- Use d3.scaleLinear and d3.scaleBand to map mathematical domains to SVG pixel coordinates.
- Separate SVG margins pattern: wrapper <g> offset by margin.left and margin.top.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build bespoke interactive data visualization applications:
1. Implement responsive SVG/Canvas visual graphics leveraging D3.js general update patterns.
2. Build interactive brush, pan, zoom, and force-directed graph physics simulations.
3. Optimize DOM rendering performance using virtual canvases for datasets exceeding 50,000 points.
```
"#,
    )
}

/// 153. analytics-parmenter-kpi-framework Skill
pub fn analytics_parmenter_kpi_framework() -> EccSkill {
    EccSkill::new(
        "analytics-parmenter-kpi-framework",
        "Executive KPI design: the 10/80/10 rule, distinguishing KRIs (Key Result Indicators) from true KPIs, leading vs lagging indicators, and Critical Success Factors (CSFs). Triggers: parmenter-kpi-framework, key-performance-indicators, kri-vs-kpi, leading-indicators, critical-success-factors, 10-80-10-rule.",
        r#"# Analytics Parmenter Kpi Framework
> Based on **Key Performance Indicators - David Parmenter**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- KPI Classification & Governance Schema
CREATE TABLE metric_governance_registry (
    metric_id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    measure_type VARCHAR(10) NOT NULL CHECK (measure_type IN ('KRI', 'RI', 'PI', 'KPI')),
    is_financial BOOLEAN NOT NULL,
    frequency VARCHAR(20) NOT NULL CHECK (frequency IN ('REALTIME', 'DAILY', 'WEEKLY', 'MONTHLY')),
    executive_owner VARCHAR(100) NOT NULL,
    critical_success_factor VARCHAR(255) NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 The 10 / 80 / 10 Metric Distribution Invariant
In a well-governed enterprise performance management framework:
$$\text{Count}(\text{KRIs}) \approx 10, \quad \text{Count}(\text{PIs}) \approx 80, \quad \text{Count}(\text{KPIs}) \le 10$$
True KPIs must satisfy:
$$\text{is\_financial} = \text{FALSE} \land \text{frequency} \in \{\text{'REALTIME'}, \text{'DAILY'}\}$$
Financial metrics (e.g. Net Profit, EBITDA) are KRIs (outcomes), never actionable leading KPIs.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Metric[Candidate Performance Metric] --> FinancialCheck{Is it Financial?}
    FinancialCheck -->|Yes: Dollars, Margin| KRI[Classify as KRI: Key Result Indicator]
    FinancialCheck -->|No| FrequencyCheck{Measured 24/7 or Daily?}
    FrequencyCheck -->|No: Monthly| PI[Classify as PI: Performance Indicator]
    FrequencyCheck -->|Yes: Real-time| ActionCheck{Does it directly drive CEO action?}
    ActionCheck -->|No| PI
    ActionCheck -->|Yes| KPI[Classify as True KPI: <= 10 per Organization]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust KPI Classification Rule
pub fn classify_metric(is_financial: bool, is_daily: bool, is_ceo_actionable: bool) -> &'static str {
    if is_financial {
        "KRI" // Key Result Indicator (lagging outcome)
    } else if is_daily && is_ceo_actionable {
        "KPI" // True Key Performance Indicator (leading, non-financial)
    } else {
        "PI"  // Operational Performance Indicator
    }
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- The 10/80/10 Rule: Max 10 KRIs (lagging results), 80 PIs (operational), 10 KPIs (critical leading).
- True KPIs are non-financial, measured daily or 24/7, and immediately actionable by the CEO.
- Profit, Revenue, and EBITDA are Key Result Indicators (KRIs), NOT KPIs.
- Tie every single KPI directly to one of the organization's Critical Success Factors (CSFs).
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect an enterprise KPI governance and semantics layer:
1. Audit corporate scorecard metrics against Parmenter's 10/80/10 rule, separating KRIs from true KPIs.
2. Link every leading operational KPI directly to Critical Success Factors (CSFs).
3. Build real-time alerting engines notifying executive leadership upon KPI boundary deviations.
```
"#,
    )
}

/// 154. analytics-lean-startup-metrics Skill
pub fn analytics_lean_startup_metrics() -> EccSkill {
    EccSkill::new(
        "analytics-lean-startup-metrics",
        "Growth & product analytics: One Metric That Matters (OMTM), Pirate Metrics (AARRR: Acquisition, Activation, Retention, Referral, Revenue), cohort retention curves, and viral loops. Triggers: lean-startup-metrics, lean-analytics, omtm, aarrr-pirate-metrics, cohort-retention, viral-coefficient, actionable-vs-vanity.",
        r#"# Analytics Lean Startup Metrics
> Based on **Lean Analytics - Alistair Croll & Benjamin Yoskovitz**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Cohort Retention Activity Ledger
CREATE TABLE cohort_retention_records (
    cohort_week DATE NOT NULL,
    user_id BIGINT NOT NULL,
    week_number INT NOT NULL,
    is_active INT NOT NULL CHECK (is_active IN (0, 1)),
    PRIMARY KEY (cohort_week, user_id, week_number)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Viral Coefficient Invariant
Let $i$ be invitations sent per customer and $c$ be conversion rate per invite:
$$K = i \times c$$
If $K > 1.0$, the product experiences exponential viral growth.

### 2.2 Cohort Retention Decay Invariant
Retention of a cohort over time $t$:
$$R(t) = R_0 \cdot t^{-\gamma} + c$$
- If $c = 0$, retention asymptotically decays to zero (broken product).
- If $c > 0$, the retention curve flattens, indicating Product-Market Fit (PMF).

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Acquisition[Acquisition: User lands on site] --> Activation[Activation: Happy first experience]
    Activation --> Retention[Retention: User returns repeatedly]
    Retention --> Referral[Referral: User invites others K = i * c]
    Retention --> Revenue[Revenue: User makes purchase / monetizes]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Retention Flattening Calculator in Python
import numpy as np

def check_pmf_retention_flattening(retention_weeks: list[float]) -> bool:
    # If delta between week 8 and week 12 is less than 1%, retention has flattened
    if len(retention_weeks) >= 12:
        return abs(retention_weeks[-1] - retention_weeks[-4]) < 0.01 and retention_weeks[-1] > 0.05
    return False
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Focus on the One Metric That Matters (OMTM) for your current startup stage.
- Pirate Metrics (AARRR): Acquisition -> Activation -> Retention -> Referral -> Revenue.
- Retention is the most critical metric: if the retention curve doesn't flatten, growth will fail.
- Viral coefficient K = invites_per_user * conversion_rate; K > 1.0 indicates viral loop.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build product analytics and growth experimentation frameworks:
1. Construct automated cohort retention analysis matrices tracking curve stabilization and PMF.
2. Build AARRR pirate metrics tracking pipelines differentiating actionable metrics from vanity metrics.
3. Model viral loop coefficients (K-factor) and user referral attribution graphs.
```
"#,
    )
}

/// 155. analytics-hubbard-measurement-value Skill
pub fn analytics_hubbard_measurement_value() -> EccSkill {
    EccSkill::new(
        "analytics-hubbard-measurement-value",
        "Applied Information Economics (AIE): Expected Value of Information (EVI), Expected Value of Perfect Information (EVPI), calibrated probability estimates, and Monte Carlo decision modeling. Triggers: hubbard-measurement-value, how-to-measure-anything, applied-information-economics, evpi, value-of-information, monte-carlo-decision, calibrated-estimates.",
        r#"# Analytics Hubbard Measurement Value
> Based on **How to Measure Anything - Douglas W. Hubbard**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Information Economics Decision Model Table
CREATE TABLE decision_information_values (
    decision_id VARCHAR(50) PRIMARY KEY,
    description TEXT NOT NULL,
    cost_of_measurement DOUBLE PRECISION NOT NULL,
    expected_value_perfect_info DOUBLE PRECISION NOT NULL,
    should_measure BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Expected Value of Perfect Information (EVPI)
Let $D$ be decision choices and $\theta$ be states of nature with prior distribution $P(\theta)$:
$$\text{EVPI} = \sum_{\theta} P(\theta) \max_{d \in D} V(d, \theta) - \max_{d \in D} \sum_{\theta} P(\theta) V(d, \theta)$$
Rule: Never spend more on data measurement than the EVPI of the decision:
$$\text{Cost of Measurement} \le \text{EVPI}$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Problem[Define Decision & Threshold] --> Calibrate[Elicit 90% Calibrated CIs from Experts]
    Calibrate --> MonteCarlo[Run 100,000 Monte Carlo Simulations]
    MonteCarlo --> ComputeEVPI[Calculate EVPI: Expected Value of Perfect Info]
    ComputeEVPI --> CheckCost{Measurement Cost < EVPI?}
    CheckCost -->|Yes| Measure[Proceed with Targeted Empirical Measurement]
    CheckCost -->|No| ActNow[Make Decision Immediately without more Data]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Monte Carlo EVPI Calculation in Python
import numpy as np

def calculate_evpi(net_benefits_scenario: np.ndarray, prob_success: float) -> float:
    # Vector of payoffs under (Decision 1 vs Decision 2) across simulations
    val_with_info = np.mean(np.maximum(net_benefits_scenario, 0.0))
    val_without_info = max(np.mean(net_benefits_scenario), 0.0)
    return float(val_with_info - val_without_info)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Anything can be measured: if it matters to business, it is observable in the real world.
- Calculate EVPI before funding analytics: never spend $50k on research if EVPI is only $10k.
- Calibrate estimators to give true 90% confidence intervals (hits target 9 times out of 10).
- Run Monte Carlo simulations over range distributions rather than single-point estimates.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Implement Applied Information Economics (AIE) decision systems:
1. Build Monte Carlo decision simulation models incorporating calibrated probabilistic ranges.
2. Compute Expected Value of Information (EVI/EVPI) to mathematically prioritize data investments.
3. Establish training and verification scoring loops to calibrate human expert probabilistic judgments.
```
"#,
    )
}

/// 156. analytics-semantic-metadata-layer Skill
pub fn analytics_semantic_metadata_layer() -> EccSkill {
    EccSkill::new(
        "analytics-semantic-metadata-layer",
        "Semantic modeling & metadata ontologies: RDF triples, RDFS/OWL formal semantics, SPARQL querying, knowledge graphs, and unified business glossaries. Triggers: semantic-metadata-layer, enterprise-knowledge-graph, rdf-triples, owl-ontologies, sparql, metadata-catalog, business-glossary.",
        r#"# Analytics Semantic Metadata Layer
> Based on **The Semantic Web for the Working Ontologist - Dean Allemang & James Hendler**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Relational Triple Store Schema for Semantic Graphs
CREATE TABLE enterprise_rdf_triples (
    subject_uri VARCHAR(255) NOT NULL,
    predicate_uri VARCHAR(255) NOT NULL,
    object_uri_or_literal TEXT NOT NULL,
    is_literal BOOLEAN NOT NULL DEFAULT FALSE,
    graph_context VARCHAR(100) NOT NULL DEFAULT 'default',
    PRIMARY KEY (subject_uri, predicate_uri, object_uri_or_literal)
);

CREATE INDEX idx_triples_spo ON enterprise_rdf_triples(subject_uri, predicate_uri);
CREATE INDEX idx_triples_po ON enterprise_rdf_triples(predicate_uri, object_uri_or_literal);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Transitive SubClass Inference Invariant
Under RDFS / OWL semantics:
$$\forall A, B, C: \quad (A \sqsubseteq B) \land (B \sqsubseteq C) \implies (A \sqsubseteq C)$$
$$\forall x, A, B: \quad x \in A \land (A \sqsubseteq B) \implies x \in B$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph LR
    Sub[Subject: Customer] -->|Predicate: placesOrder| Obj[Object: Order_1234]
    Obj -->|Predicate: hasItem| Item[Object: SKU_5567]
    Item -->|Predicate: partOfCategory| Cat[Object: Electronics]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# SPARQL Semantic Query
# SELECT ?customer ?order ?sku WHERE {
#   ?customer <http://schema.org/placesOrder> ?order .
#   ?order <http://schema.org/orderedItem> ?sku .
# }
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Model metadata as RDF triples: Subject -> Predicate -> Object.
- OWL enables formal semantic reasoning and automatic class inference across models.
- Enterprise knowledge graphs connect disparate database silos into a single queryable graph.
- Establish an enterprise business glossary mapping business terms to physical database columns.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect enterprise semantic metadata graphs:
1. Build RDF/OWL ontology repositories defining corporate business entities and tax-compliant taxonomies.
2. Deploy SPARQL query endpoints and graph traversal indexes over distributed metadata stores.
3. Automatically link technical data dictionary columns to unified business ontology concepts.
```
"#,
    )
}

/// 157. analytics-semantic-metrics-governance Skill
pub fn analytics_semantic_metrics_governance() -> EccSkill {
    EccSkill::new(
        "analytics-semantic-metrics-governance",
        "Centralized metric layers and Headless BI: Cube, dbt Semantic Layer, MetricFlow, Single Source of Truth, and preventing metric drift across tools. Triggers: semantic-metrics-governance, winning-with-data, metric-layer, headless-bi, single-source-of-truth, cube-semantic-layer, metricflow.",
        r#"# Analytics Semantic Metrics Governance
> Based on **Winning with Data - Tomasz Tunguz & Frank Bien**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Centralized Metric Definition Repository
CREATE TABLE semantic_metric_definitions (
    metric_name VARCHAR(100) PRIMARY KEY,
    metric_type VARCHAR(30) NOT NULL CHECK (metric_type IN ('SIMPLE', 'RATIO', 'CUMULATIVE', 'DERIVED')),
    sql_formula TEXT NOT NULL,
    underlying_table VARCHAR(100) NOT NULL,
    owner_team VARCHAR(50) NOT NULL,
    version INT NOT NULL DEFAULT 1,
    is_certified BOOLEAN NOT NULL DEFAULT TRUE
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Metric Expression Uniqueness Invariant
For any metric identifier $M$:
$$|\text{Definitions}(M)| \equiv 1$$
Every BI tool, API endpoint, and dashboard must resolve metric $M$ by querying the semantic layer API:
$$\text{Value}(M) \equiv \text{Eval}(\text{Formula}(M), \text{Dataset})$$
Eliminates contradictory revenue figures between Finance and Sales.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    DB[Data Warehouse Tables] --> Semantic[Semantic Layer: Cube / MetricFlow]
    Semantic --> Formula[Define Revenue = SUM(net_amount)]
    Formula --> BI[Tableau / Looker: Queries Semantic API]
    Formula --> App[Internal Portal: Queries Semantic API]
    Formula --> AI[LLM Agents: Queries Semantic API]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# MetricFlow / dbt Semantic Layer Spec
# metric:
#   name: monthly_recurring_revenue
#   type: simple
#   type_params:
#     measure: subscription_amount
#   filter: |
#     subscription_status = 'ACTIVE' 
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Define metrics once in the semantic layer; never hardcode aggregation SQL in BI dashboards.
- A semantic layer prevents 'metric drift' where different departments report conflicting numbers.
- Differentiate metric types: simple sums, ratios, cumulative lifetime metrics, derived formulas.
- Treat metrics as code: store definitions in Git, test changes, and automate deployment.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design and deploy enterprise headless BI semantic layers:
1. Implement centralized metric stores (Cube / dbt MetricFlow) exposing governed GraphQL/SQL APIs.
2. Eliminate redundant ad-hoc logic by defining business dimensions and measures as version-controlled code.
3. Integrate AI agent querying directly with semantic layer APIs for hallucination-free analytics.
```
"#,
    )
}

/// 158. analytics-okr-goal-tracking Skill
pub fn analytics_okr_goal_tracking() -> EccSkill {
    EccSkill::new(
        "analytics-okr-goal-tracking",
        "Enterprise goal alignment: Objectives and Key Results (OKRs), committed vs aspirational goals, CFRs (Conversations, Feedback, Recognition), and mathematical scoring. Triggers: okr-goal-tracking, measure-what-matters, okrs, key-results, aspirational-vs-committed, cfr-alignment, goal-scoring.",
        r#"# Analytics Okr Goal Tracking
> Based on **Measure What Matters - John Doerr**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Enterprise OKR Tracking Table
CREATE TABLE okr_key_results (
    kr_id VARCHAR(50) PRIMARY KEY,
    objective_id VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    baseline_value DOUBLE PRECISION NOT NULL,
    target_value DOUBLE PRECISION NOT NULL,
    current_value DOUBLE PRECISION NOT NULL,
    is_aspirational BOOLEAN NOT NULL DEFAULT FALSE,
    score DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (score >= 0.0 AND score <= 1.0)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Key Result Progress & Scoring Invariant
For Key Result with baseline $B$, target $T$, and current measurement $C$:
$$\text{Score} = \text{clamp}\left( \frac{C - B}{T - B}, 0.0, 1.0 \right)$$
For aspirational (moonshot) OKRs:
$$\text{Optimal Performance Sweet Spot} \in [0.6, 0.7]$$
A consistent score of 1.0 indicates under-ambitious goal setting.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Obj[Objective: Qualitative Inspiring Goal] --> KR1[Key Result 1: Quantitative Metric]
    Obj --> KR2[Key Result 2: Quantitative Metric]
    Obj --> KR3[Key Result 3: Quantitative Metric]
    KR1 --> ScoreKR[Score = (Current - Base) / (Target - Base)]
    ScoreKR --> AvgScore[Aggregate Objective Score: Sweet spot 0.7]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust OKR Scorer
pub fn score_key_result(baseline: f64, target: f64, current: f64) -> f64 {
    if (target - baseline).abs() < 1e-6 {
        return 1.0;
    }
    let raw = (current - baseline) / (target - baseline);
    raw.clamp(0.0, 1.0)
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Objectives are qualitative, inspiring, and time-bound; Key Results are strictly quantitative.
- Every Key Result must have a number, baseline, and target date: 'measure what matters'.
- Score KRs from 0.0 to 1.0: a score of 0.6 - 0.7 is the sweet spot for aspirational goals.
- Separate OKR performance evaluation from salary/compensation reviews to encourage risk-taking.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build automated OKR metric tracking architectures:
1. Connect enterprise OKR platforms directly to data warehouse analytical tables for real-time progress.
2. Implement mathematical scoring algorithms distinguishing committed (1.0 target) vs aspirational goals.
3. Build cascading alignment graphs tracing team key results directly to corporate objectives.
```
"#,
    )
}

/// 159. analytics-hyndman-time-series Skill
pub fn analytics_hyndman_time_series() -> EccSkill {
    EccSkill::new(
        "analytics-hyndman-time-series",
        "Modern time series forecasting: STL decomposition (LOESS), exponential smoothing (Holt-Winters), forecasting benchmarks, and accuracy metrics (MASE, MAPE, RMSE). Triggers: hyndman-time-series, fpp3, time-series-forecasting, stl-decomposition, holt-winters, exponential-smoothing, mase-metric.",
        r#"# Analytics Hyndman Time Series
> Based on **Forecasting: Principles and Practice - Rob J. Hyndman & George Athanasopoulos**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Time Series Forecast Evaluations
CREATE TABLE time_series_forecast_evals (
    series_id VARCHAR(50) PRIMARY KEY,
    model_name VARCHAR(50) NOT NULL,
    horizon_days INT NOT NULL,
    mae DOUBLE PRECISION NOT NULL,
    rmse DOUBLE PRECISION NOT NULL,
    mase DOUBLE PRECISION NOT NULL,
    is_better_than_naive BOOLEAN NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 STL Additive Decomposition Invariant
$$Y_t = T_t + S_t + R_t$$
where $T_t$ is trend, $S_t$ is seasonal component, and $R_t$ is remainder.

### 2.2 Mean Absolute Scaled Error (MASE)
$$\text{MASE} = \frac{\frac{1}{h} \sum_{t=T+1}^{T+h} |Y_t - \hat{Y}_t|}{\frac{1}{T-1} \sum_{t=2}^T |Y_t - Y_{t-1}|}$$
- $\text{MASE} < 1.0 \implies$ Model outperforms in-sample naive persistence baseline.
- $\text{MASE} > 1.0 \implies$ Model performs worse than a simple naive forecast.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawSeries[Time Series Data] --> CheckSeasonality{Seasonal Periodicity?}
    CheckSeasonality --> STL[STL Decomposition: Extract Trend & Seasonal]
    STL --> Decompose[Model Deseasonalized Series]
    Decompose --> HoltWinters[Fit Holt-Winters Exponential Smoothing]
    HoltWinters --> EvalMASE[Compute MASE vs Naive Baseline]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# MASE Calculation in Python
import numpy as np

def calculate_mase(y_train: np.ndarray, y_test: np.ndarray, y_pred: np.ndarray) -> float:
    naive_mae = np.mean(np.abs(np.diff(y_train)))
    if naive_mae == 0:
        return 1.0
    model_mae = np.mean(np.abs(y_test - y_pred))
    return float(model_mae / naive_mae)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Decompose time series using STL (LOESS) into Trend, Seasonality, and Remainder.
- Use MASE (Mean Absolute Scaled Error) to evaluate forecasts: MASE < 1 beats the naive forecast.
- Always benchmark complex forecasting models against Naive and Seasonal Naive baselines.
- Holt-Winters supports additive (constant variance) and multiplicative (proportional variance) seasonality.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Engineer scalable automated time-series forecasting pipelines:
1. Implement STL decomposition pipelines isolating seasonal patterns and baseline trends.
2. Build multi-model tournament runners (ETS, ARIMA, Prophet) evaluating out-of-sample MASE.
3. Generate calibrated prediction intervals (80% and 95%) modeling demand uncertainty for operations.
```
"#,
    )
}

/// 160. analytics-box-jenkins-arima Skill
pub fn analytics_box_jenkins_arima() -> EccSkill {
    EccSkill::new(
        "analytics-box-jenkins-arima",
        "Box-Jenkins time series methodology: ARIMA/SARIMA modeling, stationary differencing, ACF/PACF diagnostics, and Ljung-Box residual white-noise testing. Triggers: box-jenkins-arima, arima-modeling, stationarity-differencing, acf-pacf, ljung-box-test, white-noise-residuals, sarima.",
        r#"# Analytics Box Jenkins Arima
> Based on **Time Series Analysis: Forecasting and Control - George Box et al.**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- ARIMA Model Diagnostic Registry
CREATE TABLE arima_model_diagnostics (
    series_id VARCHAR(50) PRIMARY KEY,
    p INT NOT NULL, -- AR order
    d INT NOT NULL, -- Differencing order
    q INT NOT NULL, -- MA order
    aic DOUBLE PRECISION NOT NULL,
    bic DOUBLE PRECISION NOT NULL,
    ljung_box_stat DOUBLE PRECISION NOT NULL,
    ljung_box_p_value DOUBLE PRECISION NOT NULL,
    residuals_are_white_noise BOOLEAN NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 ARIMA(p, d, q) Mathematical Formulation
$$(1 - \sum_{i=1}^p \phi_i B^i) (1 - B)^d Y_t = c + (1 + \sum_{j=1}^q \theta_j B^j) \epsilon_t$$
where $B$ is the backshift operator ($B^k Y_t = Y_{t-k}$) and $\epsilon_t \sim \mathcal{N}(0, \sigma^2)$.

### 2.2 Ljung-Box White Noise Diagnostic Invariant
$$Q = n(n + 2) \sum_{k=1}^h \frac{\hat{\rho}_k^2}{n - k} \sim \chi^2(h - p - q)$$
Residuals are pure white noise if and only if $p = P(\chi^2 \ge Q) > 0.05$.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Raw[Raw Time Series] --> StationarityCheck{ADF Test: Is Series Stationary?}
    StationarityCheck -->|No| Difference[Difference Series d times: (1-B)^d Y_t]
    Difference --> StationarityCheck
    StationarityCheck -->|Yes| InspectACF[Inspect ACF & PACF to identify p, q]
    InspectACF --> Estimate[Estimate Parameters via MLE]
    Estimate --> Diagnostic{Ljung-Box p-value > 0.05?}
    Diagnostic -->|No: Autocorrelated| Refit[Adjust p, q Orders]
    Diagnostic -->|Yes: White Noise| Forecast[Generate Production Forecast]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from statsmodels.tsa.stattools import acf
import numpy as np

def ljung_box_test(residuals: np.ndarray, lags: int = 10) -> tuple[float, float]:
    from statsmodels.stats.diagnostic import acorr_ljungbox
    res = acorr_ljungbox(residuals, lags=[lags], return_df=True)
    stat = res['lb_stat'].iloc[0]
    p_val = res['lb_pvalue'].iloc[0]
    return float(stat), float(p_val)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Box-Jenkins 3-stage iterative cycle: 1. Identification -> 2. Estimation -> 3. Diagnostics.
- Difference the series d times until stationary (verify with Augmented Dickey-Fuller test).
- PACF cuts off at lag p for AR(p); ACF cuts off at lag q for MA(q).
- Diagnostic rule: residuals must be uncorrelated white noise (Ljung-Box test p-value > 0.05).
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build enterprise ARIMA/SARIMA econometric forecasting modules:
1. Implement automated Box-Jenkins pipelines running ADF unit-root tests and optimal differencing.
2. Search hyperparameter space (p, d, q, P, D, Q) minimizing AIC/BIC information criteria.
3. Validate residual white-noise compliance with automated Ljung-Box and normality tests.
```
"#,
    )
}

/// 161. analytics-anomaly-outlier-detection Skill
pub fn analytics_anomaly_outlier_detection() -> EccSkill {
    EccSkill::new(
        "analytics-anomaly-outlier-detection",
        "Multi-dimensional anomaly detection: Isolation Forests, Local Outlier Factor (LOF), Mahalanobis distance, extreme value metrics, and anomaly scoring. Triggers: anomaly-outlier-detection, isolation-forest, local-outlier-factor, outlier-analysis, extreme-value-detection, lof-score, anomaly-scores.",
        r#"# Analytics Anomaly Outlier Detection
> Based on **Outlier Analysis - Charu C. Aggarwal**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Anomaly Detection Alert Incidents
CREATE TABLE anomaly_incidents (
    incident_id UUID PRIMARY KEY,
    entity_id VARCHAR(100) NOT NULL,
    detector_algorithm VARCHAR(50) NOT NULL,
    anomaly_score DOUBLE PRECISION NOT NULL CHECK (anomaly_score >= 0.0 AND anomaly_score <= 1.0),
    is_anomaly BOOLEAN NOT NULL DEFAULT FALSE,
    feature_contributions JSONB NOT NULL,
    detected_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Isolation Forest Anomaly Score Invariant
For sample size $n$ and average path length $E(h(x))$ across isolation trees:
$$c(n) = 2 \ln(n - 1) + 0.5772156649 - \frac{2(n - 1)}{n}$$
$$s(x, n) = 2^{-\frac{E(h(x))}{c(n)}}$$
- If $s \to 1.0$ (path length $E(h(x)) \to 0$): Definite anomaly (isolated rapidly).
- If $s < 0.5$: Normal observation.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    DataPoint[Incoming Multi-Dimensional Record] --> Forest[Pass through N Isolation Trees]
    Forest --> PathLength[Measure Tree Depth to Isolate Point]
    PathLength --> AvgDepth[Compute Mean Path Length E(h(x))]
    AvgDepth --> Score[Compute Anomaly Score s = 2^(-E(h)/c(n))]
    Score --> ThresholdCheck{Score > 0.60?}
    ThresholdCheck -->|Yes| Alert[Raise High-Priority Anomaly Incident]
    ThresholdCheck -->|No| Normal[Pass Record as Nominal]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from sklearn.ensemble import IsolationForest
import numpy as np

def detect_outliers_isolation_forest(X: np.ndarray, contamination: float = 0.01):
    clf = IsolationForest(contamination=contamination, random_state=42)
    preds = clf.fit_predict(X) # -1 for outlier, 1 for inlier
    scores = -clf.score_samples(X) # Higher score = more anomalous
    return {"outlier_mask": preds == -1, "scores": scores}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Isolation Forest isolates anomalies using random partitioning; outliers have short path lengths.
- Anomaly score s(x) > 0.6 indicates strong anomaly; s(x) < 0.5 indicates normal data.
- Local Outlier Factor (LOF) compares local density of an entity to its k-nearest neighbors.
- Use Mahalanobis distance for multivariate Gaussian data to account for inter-feature covariance.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Deploy real-time multi-variate anomaly detection microservices:
1. Train Isolation Forest and Local Outlier Factor models across high-dimensional feature spaces.
2. Decompose anomaly contributions to provide explainable root-cause attribution JSON payloads.
3. Build continuous streaming anomaly monitors alerting on operational metric drifts.
```
"#,
    )
}

/// 162. analytics-python-signal-processing Skill
pub fn analytics_python_signal_processing() -> EccSkill {
    EccSkill::new(
        "analytics-python-signal-processing",
        "Signal analytics & temporal algorithms: Fast Fourier Transform (FFT) spectral analysis, Dynamic Time Warping (DTW), CUSUM drift detection, and rolling digital filters. Triggers: python-signal-processing, practical-time-series, fft-spectral-analysis, dtw-dynamic-time-warping, change-point-detection, cusum-drift.",
        r#"# Analytics Python Signal Processing
> Based on **Practical Time Series Analysis - Aileen Nielsen**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Signal Spectral Profile Cache
CREATE TABLE signal_spectral_peaks (
    sensor_id VARCHAR(50) NOT NULL,
    dominant_frequency_hz DOUBLE PRECISION NOT NULL,
    peak_amplitude DOUBLE PRECISION NOT NULL,
    detected_at TIMESTAMP NOT NULL,
    PRIMARY KEY (sensor_id, dominant_frequency_hz)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Discrete Fourier Transform (FFT)
For discrete signal $x_0, \dots, x_{N-1}$:
$$X_k = \sum_{n=0}^{N-1} x_n \cdot e^{-i 2\pi k n / N}$$
Identifies hidden periodic frequencies and cyclical components in time series.

### 2.2 CUSUM Change-Point Detection Invariant
$$S_t^+ = \max(0, S_{t-1}^+ + (X_t - \mu_0) - k)$$
Alarm triggers when $S_t^+ > h$ where $h$ is decision threshold and $k$ is allowance.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Signal[Raw Noisy Temporal Signal] --> RollingFilter[Apply Savitzky-Golay / Butterworth Filter]
    RollingFilter --> FFT[Compute Fast Fourier Transform]
    FFT --> FindPeaks[Extract Top Cyclical Frequencies]
    RollingFilter --> CUSUM[Run CUSUM Cumulative Sum Drift Detector]
    CUSUM --> DriftCheck{S_t > Threshold h?}
    DriftCheck -->|Yes| Alert[Trigger Structural Change-Point Alarm]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def cusum_detector(series: np.ndarray, target_mean: float, allowance: float = 0.5, threshold: float = 5.0) -> list[int]:
    s_pos = 0.0
    change_points = []
    for t, val in enumerate(series):
        s_pos = max(0.0, s_pos + (val - target_mean) - allowance)
        if s_pos > threshold:
            change_points.append(t)
            s_pos = 0.0 # reset
    return change_points
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Use FFT (Fast Fourier Transform) to discover hidden cyclical periodicities in time-series data.
- CUSUM triggers alerts on mean drift: S_t = max(0, S_{t-1} + (x_t - target) - k) > h.
- Dynamic Time Warping (DTW) measures similarity between time series of differing lengths and speeds.
- Apply rolling window filters (Butterworth or Savitzky-Golay) to smooth high-frequency noise.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build signal processing and change-point analytics pipelines:
1. Implement spectral decomposition using FFT and Wavelet transforms to isolate cyclical signals.
2. Deploy real-time CUSUM change-point detectors identifying structural macroeconomic shifts.
3. Build Dynamic Time Warping (DTW) distance matrix engines clustering customer trajectory patterns.
```
"#,
    )
}

/// 163. analytics-feature-engineering-pipeline Skill
pub fn analytics_feature_engineering_pipeline() -> EccSkill {
    EccSkill::new(
        "analytics-feature-engineering-pipeline",
        "Production feature engineering: Box-Cox power transforms, target/mean encoding with empirical Bayes smoothing, quantile binning, and interaction terms. Triggers: feature-engineering-pipeline, feature-engineering, box-cox-transform, target-encoding, empirical-bayes-smoothing, quantile-binning, tfidf-encoding.",
        r#"# Analytics Feature Engineering Pipeline
> Based on **Feature Engineering for Machine Learning - Alice Zheng & Amanda Casari**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Target Encoding Metadata Store
CREATE TABLE target_encoding_priors (
    feature_name VARCHAR(100) NOT NULL,
    category_value VARCHAR(100) NOT NULL,
    global_mean DOUBLE PRECISION NOT NULL,
    category_mean DOUBLE PRECISION NOT NULL,
    category_count BIGINT NOT NULL,
    smoothed_value DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (feature_name, category_value)
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Empirical Bayes Smoothed Target Encoding
For category $k$ with $n_k$ observations and sample mean $\bar{y}_k$:
$$S_k = \lambda(n_k) \bar{y}_k + (1 - \lambda(n_k)) \bar{y}_{\text{global}}$$
$$\text{where } \lambda(n_k) = \frac{1}{1 + e^{-(n_k - m) / s}}$$
Prevents target leakage and overfitting on rare categories.

### 2.2 Box-Cox Power Transformation
$$y^{(\lambda)} = \begin{cases} \frac{y^\lambda - 1}{\lambda} & \text{if } \lambda \neq 0 \\ \ln(y) & \text{if } \lambda = 0 \end{cases} \quad (y > 0)$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawCat[High-Cardinality Categorical Feature] --> OutOfFold[Split Data into K-Folds for Encoding]
    OutOfFold --> ComputePriors[Calculate Global Mean & Category Mean]
    ComputePriors --> Smooth[Apply Sigmoid Shrinkage Weight lambda]
    Smooth --> Assign[Assign Smoothed Value to Out-of-Fold Partition]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import numpy as np

def empirical_bayes_target_encode(n_k: np.ndarray, y_k: np.ndarray, global_mean: float, m: float = 10.0, s: float = 2.0) -> np.ndarray:
    # Sigmoid smoothing weight
    weight = 1.0 / (1.0 + np.exp(-(n_k - m) / s))
    return weight * y_k + (1.0 - weight) * global_mean
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Prevent target encoding leakage: always compute target encoding out-of-fold using cross-validation.
- Smooth small categories toward the global prior using empirical Bayes shrinkage.
- Apply Box-Cox or Log1p transformations to heavy-tailed continuous variables to normalize distributions.
- Quantile binning transforms non-linear variables into uniform categorical intervals.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect enterprise ML feature engineering platforms:
1. Build leak-free out-of-fold target encoding pipelines with empirical Bayes smoothing.
2. Implement automated power transforms (Box-Cox, Yeo-Johnson) normalizing skewed feature inputs.
3. Deploy centralized feature stores ensuring training-serving feature parity across online and offline engines.
```
"#,
    )
}

/// 164. analytics-geron-ml-pipelines Skill
pub fn analytics_geron_ml_pipelines() -> EccSkill {
    EccSkill::new(
        "analytics-geron-ml-pipelines",
        "Scikit-Learn & ML production pipelines: ColumnTransformer, cross-validation tuning, ensemble algorithms (Random Forest, XGBoost), and ROC-AUC metrics. Triggers: geron-ml-pipelines, hands-on-ml, scikit-learn-pipelines, columntransformer, cross-validation-tuning, gradient-boosting, roc-auc.",
        r#"# Analytics Geron Ml Pipelines
> Based on **Hands-On Machine Learning - Aurélien Géron**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Trained ML Model Artifact Registry
CREATE TABLE ml_model_artifacts (
    model_id UUID PRIMARY KEY,
    model_name VARCHAR(100) NOT NULL,
    version VARCHAR(20) NOT NULL,
    train_roc_auc DOUBLE PRECISION NOT NULL,
    test_roc_auc DOUBLE PRECISION NOT NULL,
    f1_score DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Data Leakage Prevention Invariant
Let $T$ be a transformer with parameters $\theta$. For dataset split $D_{\text{train}}, D_{\text{test}}$:
$$\theta \text{ must be estimated strictly from } D_{\text{train}}: \quad \theta = \text{Fit}(D_{\text{train}})$$
$$D_{\text{test}}^{\prime} = \text{Transform}(D_{\text{test}}, \theta)$$
Fitting on the combined dataset $D_{\text{train}} \cup D_{\text{test}}$ produces severe optimistic bias.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    RawData[Raw Features Dataset] --> Split[Train / Test Split 80/20]
    Split --> ColTrans[ColumnTransformer: Numerical & Categorical pipelines]
    ColTrans --> FitTrain[fit_transform strictly on Train set]
    FitTrain --> Estimator[Fit Gradient Boosting Classifier]
    Estimator --> EvalTest[transform and evaluate on Test set]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
from sklearn.compose import ColumnTransformer
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler, OneHotEncoder
from sklearn.ensemble import HistGradientBoostingClassifier

num_pipe = Pipeline([('scaler', StandardScaler())])
cat_pipe = Pipeline([('encoder', OneHotEncoder(handle_unknown='ignore'))])

preprocessor = ColumnTransformer([
    ('num', num_pipe, ['age', 'income', 'credit_score']),
    ('cat', cat_pipe, ['channel', 'state'])
])

full_model = Pipeline([
    ('prep', preprocessor),
    ('clf', HistGradientBoostingClassifier(random_state=42))
])
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Wrap preprocessing and estimation inside a single sklearn Pipeline to eliminate data leakage.
- Use ColumnTransformer to apply different transformations to numerical vs categorical features.
- Evaluate imbalanced classifiers using ROC-AUC and Precision-Recall AUC, never accuracy alone.
- Tune hyperparameters with RandomizedSearchCV to explore high-dimensional spaces efficiently.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build enterprise production machine learning pipelines:
1. Deploy end-to-end Scikit-Learn pipelines utilizing ColumnTransformer preventing data leakage.
2. Train gradient boosted ensembles with early stopping on validation metric plateaus.
3. Export pipeline artifacts to ONNX runtimes for low-latency sub-millisecond scoring.
```
"#,
    )
}

/// 165. analytics-kuhn-predictive-modeling Skill
pub fn analytics_kuhn_predictive_modeling() -> EccSkill {
    EccSkill::new(
        "analytics-kuhn-predictive-modeling",
        "Predictive modeling diagnostics: near-zero variance predictors, Variance Inflation Factor (VIF), multicollinearity handling, SMOTE class balancing, and PCA preprocessing. Triggers: kuhn-predictive-modeling, applied-predictive-modeling, near-zero-variance, variance-inflation-factor, vif, smote-class-imbalance, pca-preprocessing.",
        r#"# Analytics Kuhn Predictive Modeling
> Based on **Applied Predictive Modeling - Max Kuhn & Kjell Johnson**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Predictor Quality Diagnostics Cache
CREATE TABLE predictor_quality_audits (
    feature_name VARCHAR(100) PRIMARY KEY,
    frequency_ratio DOUBLE PRECISION NOT NULL,
    percent_unique DOUBLE PRECISION NOT NULL,
    is_near_zero_variance BOOLEAN NOT NULL,
    vif_score DOUBLE PRECISION NOT NULL,
    action_taken VARCHAR(50) NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Variance Inflation Factor (VIF)
For predictor $X_j$ regressed on all other $p-1$ predictors:
$$VIF_j = \frac{1}{1 - R_j^2}$$
- $VIF_j > 5.0$: Moderate multicollinearity.
- $VIF_j > 10.0$: Severe collinearity; predictor coefficients become numerically unstable and must be pruned.

### 2.2 Near-Zero Variance Condition
A feature has near-zero variance if:
$$\frac{\text{Freq}(\text{Most Common})}{\text{Freq}(\text{2nd Most Common})} > 19.0 \quad (95/5 \text{ split}) \quad \land \quad \frac{\text{Unique Count}}{N} < 0.10$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Features[Candidate Feature Matrix] --> CheckNZV{Near-Zero Variance Check}
    CheckNZV -->|Flagged| DropNZV[Drop Predictor: Lacks Information]
    CheckNZV -->|Pass| CheckCorr[Compute Correlation Matrix]
    CheckCorr --> CalcVIF[Compute Variance Inflation Factor VIF]
    CalcVIF --> VIFCheck{VIF > 10.0?}
    VIFCheck -->|Yes| DropCollinear[Prune or Apply PCA Reduction]
    VIFCheck -->|No| ModelReady[Feature Matrix Validated for Modeling]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Near-Zero Variance Filter in Python
import pandas as pd

def filter_near_zero_variance(df: pd.DataFrame, freq_cut: float = 95/5, unique_cut: float = 10.0) -> list[str]:
    dropped = []
    n = len(df)
    for col in df.columns:
        counts = df[col].value_counts()
        if len(counts) <= 1:
            dropped.append(col); continue
        freq_ratio = counts.iloc[0] / counts.iloc[1]
        pct_unique = (df[col].nunique() / n) * 100
        if freq_ratio > freq_cut and pct_unique < unique_cut:
            dropped.append(col)
    return dropped
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Drop near-zero variance features: frequency ratio > 95/5 and unique percentage < 10%.
- Compute Variance Inflation Factor (VIF): remove or combine features with VIF > 10.
- Multicollinearity inflates standard errors of regression coefficients without improving predictive power.
- Apply SMOTE (Synthetic Minority Over-sampling) strictly on training folds to address class imbalance.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Engineer robust feature preprocessing architectures:
1. Implement automated near-zero variance and multi-collinearity filtering pipelines.
2. Build iterative VIF reduction algorithms eliminating correlated predictors prior to linear modeling.
3. Deploy cross-validated SMOTE and downsampling routines balancing severe class skew.
```
"#,
    )
}

/// 166. analytics-elements-statistical-learning Skill
pub fn analytics_elements_statistical_learning() -> EccSkill {
    EccSkill::new(
        "analytics-elements-statistical-learning",
        "Advanced statistical learning theory: Support Vector Machines (maximal margin hyperplanes), KKT optimality conditions, kernel methods, boosting, and cost-complexity pruning. Triggers: elements-statistical-learning, esl, statistical-learning-theory, svm-margin, kernel-tricks, boosting-trees, cost-complexity-pruning.",
        r#"# Analytics Elements Statistical Learning
> Based on **The Elements of Statistical Learning - Trevor Hastie, Robert Tibshirani, Jerome Friedman**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Statistical Theory Algorithm Benchmarks
CREATE TABLE learning_theory_benchmarks (
    algorithm VARCHAR(50) PRIMARY KEY,
    loss_function VARCHAR(50) NOT NULL,
    generalization_bound DOUBLE PRECISION NOT NULL,
    empirical_risk DOUBLE PRECISION NOT NULL,
    structural_risk DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Support Vector Machine Maximal Margin Formulation
$$\min_{w, b, \xi} \frac{1}{2} \|w\|^2 + C \sum_{i=1}^n \xi_i$$
subject to:
$$y_i (w^T \phi(x_i) + b) \ge 1 - \xi_i, \quad \xi_i \ge 0 \quad \forall i$$
Karush-Kuhn-Tucker (KKT) complementary slackness condition:
$$\alpha_i [y_i(w^T \phi(x_i) + b) - 1 + \xi_i] = 0$$

### 2.2 Cost-Complexity Tree Pruning
$$C_\alpha(T) = R(T) + \alpha |T|$$
where $R(T)$ is misclassification risk, $|T|$ is terminal leaf count, and $\alpha$ is complexity penalty.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Data[Input Space X] --> Kernel[Kernel Mapping: Inner Product in Hilbert Space K(x, x')]
    Kernel --> SolveDual[Solve Dual Quadratic Optimization for alpha_i]
    SolveDual --> SupportVectors[Identify Support Vectors: alpha_i > 0]
    SupportVectors --> DecisionBoundary[Construct Optimal Separating Hyperplane]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Radial Basis Function (RBF) Kernel in NumPy
import numpy as np

def rbf_kernel(X1: np.ndarray, X2: np.ndarray, gamma: float = 0.1) -> np.ndarray:
    # K(x, y) = exp(-gamma * ||x - y||^2)
    dist_sq = np.sum(X1**2, 1).reshape(-1, 1) + np.sum(X2**2, 1) - 2 * np.dot(X1, X2.T)
    return np.exp(-gamma * dist_sq)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Support Vector Machines maximize the geometric margin 2 / ||w|| between classes.
- Only data points lying on the margin (support vectors, alpha > 0) influence the decision boundary.
- The Kernel Trick computes inner products in high-dimensional Hilbert spaces without explicit mapping.
- Tree cost-complexity pruning balances in-sample error R(T) against tree size alpha * |T|.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Deploy advanced statistical learning algorithms:
1. Implement maximal-margin quadratic programming solvers with soft-margin slack penalties.
2. Build Mercer-compliant kernel functions (RBF, Polynomial) enabling non-linear boundary separation.
3. Formulate cost-complexity pruning routines optimizing decision tree generalization limits.
```
"#,
    )
}

/// 167. analytics-redman-data-quality Skill
pub fn analytics_redman_data_quality() -> EccSkill {
    EccSkill::new(
        "analytics-redman-data-quality",
        "Data quality engineering: Friday Afternoon Measurement (FAM), the 1-10-100 Rule of Ten, root-cause error prevention at point of data creation, and core DQ dimensions. Triggers: redman-data-quality, data-quality-framework, rule-of-ten, 1-10-100-rule, friday-afternoon-measurement, dq-dimensions, root-cause-prevention.",
        r#"# Analytics Redman Data Quality
> Based on **Data Quality: The Field Guide - Thomas C. Redman**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Friday Afternoon Measurement (FAM) Log
CREATE TABLE fam_data_quality_audits (
    audit_date DATE PRIMARY KEY,
    dataset_name VARCHAR(100) NOT NULL,
    sample_size INT NOT NULL DEFAULT 100,
    perfect_records INT NOT NULL,
    defective_records INT NOT NULL,
    data_quality_percent NUMERIC(5, 2) NOT NULL,
    primary_defect_root_cause TEXT
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 The 1-10-100 Rule of Ten Invariant
$$\text{Cost of Data Error} = \begin{cases} \$1 & \text{Prevention at point of creation} \\ \$10 & \text{Correction in ETL / warehouse} \\ \$100+ & \text{Failure remediation in business operations} \end{cases}$$
Economic imperative: Move validation rules upstream to data entry sources.

### 2.2 FAM Quality Fraction
For a sample of $N = 100$ records across $K$ critical attributes:
$$DQ\% = \frac{\text{Count}(\text{Records with ZERO defects})}{100} \times 100\%$$

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    Audit[Weekly Friday Afternoon Measurement: Sample 100 Records] --> Inspect[Inspect 10-15 Critical Attributes]
    Inspect --> CountDefects[Count Records with ANY defect]
    CountDefects --> Score[Score DQ%: Target >= 95%]
    Score --> TraceOrigin[Trace Defect to Root Cause Creation Point]
    TraceOrigin --> FixSystem[Fix Source Software to prevent recurrence]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust Friday Afternoon Measurement Calculator
pub struct FamAudit {
    pub total_records: usize,
    pub defective_records: usize,
}

impl FamAudit {
    pub fn quality_percentage(&self) -> f64 {
        if self.total_records == 0 { return 100.0; }
        let perfect = self.total_records.saturating_sub(self.defective_records);
        (perfect as f64 / self.total_records as f64) * 100.0
    }
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Rule of Ten (1-10-100): $1 to prevent at source, $10 to fix in ETL, $100+ to remediate downstream.
- Run Friday Afternoon Measurement (FAM): audit 100 random records weekly across core business attributes.
- A record with a single defective field is defective; measure percentage of completely error-free records.
- Stop cleaning data repeatedly in ETL; trace defects to the source application and prevent them there.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build enterprise data quality management systems:
1. Implement weekly automated Friday Afternoon Measurement (FAM) sampling protocols across operational databases.
2. Formulate upstream validation gates enforcing the 1-10-100 cost prevention rule.
3. Build root-cause tracking incident dashboards linking warehouse errors directly to source application defects.
```
"#,
    )
}

/// 168. analytics-data-observability-monitors Skill
pub fn analytics_data_observability_monitors() -> EccSkill {
    EccSkill::new(
        "analytics-data-observability-monitors",
        "Data observability architecture: 5 pillars (Freshness, Volume, Schema, Distribution, Lineage), automated anomaly alerts, and SLA breach detection. Triggers: data-observability-monitors, data-observability, 5-pillars-observability, freshness-monitoring, volume-anomalies, schema-drift, distribution-tracking, data-lineage.",
        r#"# Analytics Data Observability Monitors
> Based on **Data Observability - Barr Moses, Lior Gavrish, Andy Petrella**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Data Observability 5 Pillars Monitoring Store
CREATE TABLE table_observability_monitors (
    table_name VARCHAR(100) PRIMARY KEY,
    last_updated_at TIMESTAMP NOT NULL,
    freshness_sla_sec INT NOT NULL,
    current_row_count BIGINT NOT NULL,
    expected_row_count_mean DOUBLE PRECISION NOT NULL,
    expected_row_count_std DOUBLE PRECISION NOT NULL,
    schema_hash CHAR(32) NOT NULL,
    null_rate_percent DOUBLE PRECISION NOT NULL,
    is_freshness_breached BOOLEAN NOT NULL DEFAULT FALSE,
    is_volume_anomalous BOOLEAN NOT NULL DEFAULT FALSE,
    is_schema_drifted BOOLEAN NOT NULL DEFAULT FALSE
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 The 5 Pillars of Data Observability Invariants
1. **Freshness Invariant**:
   $$\Delta t = T_{\text{now}} - \max(T_{\text{updated}}) \le \text{SLA}_{\text{freshness}}$$
2. **Volume Anomaly Invariant**:
   $$Z_{\text{volume}} = \frac{|N_{\text{rows}} - \mu_{\text{volume}}|}{\sigma_{\text{volume}}} \le 3.0$$
3. **Schema Integrity Invariant**:
   $$\text{Hash}(\text{Schema}_{t}) \equiv \text{Hash}(\text{Schema}_{t-1})$$
4. **Distribution Invariant**: Null rates and quantile metrics within historical control bounds.
5. **Lineage Invariant**: Directed acyclic graph tracking upstream dependencies and downstream consumers.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    TableUpdate[Table Ingestion Completes] --> CheckFreshness[1. Freshness: Has data updated on schedule?]
    TableUpdate --> CheckVolume[2. Volume: Is row count within 3 sigma?]
    TableUpdate --> CheckSchema[3. Schema: Have columns changed or types altered?]
    TableUpdate --> CheckDistribution[4. Distribution: Have null rates or values shifted?]
    TableUpdate --> CheckLineage[5. Lineage: Update dependency graph & downstream impact]
    CheckFreshness --> AlertEngine{Any Pillar Breached?}
    CheckVolume --> AlertEngine
    CheckSchema --> AlertEngine
    CheckDistribution --> AlertEngine
    AlertEngine -->|Yes| PagerDuty[Dispatch Immediate DataOps Alert]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
-- Freshness and Volume Automated Observability Check
SELECT 
    'fct_orders' AS table_name,
    EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - MAX(updated_at))) AS freshness_delay_sec,
    COUNT(*) AS current_row_count,
    COUNT(*) FILTER (WHERE customer_id IS NULL) * 100.0 / COUNT(*) AS null_rate_customer
FROM fct_orders;
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Monitor the 5 Pillars of Data Observability: Freshness, Volume, Schema, Distribution, Lineage.
- Freshness breach alert triggers when CURRENT_TIMESTAMP - MAX(updated_at) > SLA.
- Volume anomaly triggers when current row count deviates > 3 standard deviations from rolling mean.
- Schema drift must block downstream pipeline jobs to prevent silent analytics corruption.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Construct an enterprise Data Observability platform:
1. Deploy continuous monitoring daemons evaluating the 5 pillars across all production analytical tables.
2. Build statistical volume and freshness anomaly detectors with rolling seasonal baselines.
3. Automatically maintain and visualize end-to-end column-level lineage dependency graphs.
```
"#,
    )
}

/// 169. analytics-dama-data-governance Skill
pub fn analytics_dama_data_governance() -> EccSkill {
    EccSkill::new(
        "analytics-dama-data-governance",
        "Enterprise data governance: DAMA-DMBOK wheel, Master Data Management (MDM), golden record survivorship rules, data stewardship, and metadata management. Triggers: dama-data-governance, dama-dmbok, master-data-management, mdm-golden-record, data-governance-council, data-stewardship, data-architecture-wheel.",
        r#"# Analytics Dama Data Governance
> Based on **DAMA-DMBOK - DAMA International**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Master Data Management (MDM) Golden Record Store
CREATE TABLE mdm_golden_customer (
    golden_customer_id UUID PRIMARY KEY,
    primary_source_system VARCHAR(50) NOT NULL,
    source_record_id VARCHAR(100) NOT NULL,
    consolidated_name VARCHAR(100) NOT NULL,
    consolidated_email VARCHAR(100) NOT NULL,
    confidence_score DOUBLE PRECISION NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    survivorship_rule_applied VARCHAR(50) NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 MDM Golden Record Deterministic Survivorship Invariant
Let entity records $R_1, \dots, R_m$ represent the same real-world identity from source systems $S_1, \dots, S_m$:
$$\text{Value}(A_{\text{golden}}) = \text{Select}\left( \{R_i.A\}, \text{Precedence}(S_1 \succ S_2 \succ \dots \succ S_m) \lor \max(R_i.\text{timestamp}) \right)$$
The survivorship function must be deterministic and fully auditable.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    SourceA[CRM Customer Record] --> MatchEngine[Match Engine: Fuzzy Jaro-Winkler & Exact Keys]
    SourceB[ERP Customer Record] --> MatchEngine
    SourceC[Billing Customer Record] --> MatchEngine
    MatchEngine --> MatchCheck{Confidence >= 0.85?}
    MatchCheck -->|Yes| Survivorship[Apply Survivorship Rules: Source Precedence / Recency]
    Survivorship --> GoldenRecord[Publish Golden Record to Master Catalog]
    MatchCheck -->|No| StewardshipQueue[Route to Human Data Steward Queue]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
// Rust MDM Survivorship Rule Engine
pub struct SourceAttribute {
    pub system_priority: usize, // Lower number = higher priority
    pub timestamp: u64,
    pub value: String,
}

pub fn resolve_golden_value(candidates: &[SourceAttribute]) -> Option<String> {
    candidates.iter()
        .min_by_key(|c| (c.system_priority, u64::MAX - c.timestamp))
        .map(|c| c.value.clone())
}
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- The DAMA-DMBOK wheel centers on Data Governance, surrounded by 10 knowledge areas.
- Master Data Management (MDM) establishes a Single Source of Truth ('Golden Record') for core entities.
- Survivorship rules must be deterministic: define source-of-record hierarchy or most-recent-valid-timestamp.
- Route ambiguous identity matches below confidence threshold (e.g. < 0.85) to Data Stewards.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Architect an enterprise data governance program adhering to DAMA-DMBOK:
1. Formulate master data golden record survivorship rules (source-of-record priority and recency).
2. Establish a Data Governance Council charter with clear stewardship roles and RACI matrices.
3. Deploy automated metadata catalogs tracking classification, data lineage, and retention policies.
```
"#,
    )
}

/// 170. analytics-dataops-automated-testing Skill
pub fn analytics_dataops_automated_testing() -> EccSkill {
    EccSkill::new(
        "analytics-dataops-automated-testing",
        "DataOps principles: agile data analytics, CI/CD automated testing harnesses, pre-flight circuit breakers, and ephemeral development sandboxes. Triggers: dataops-automated-testing, dataops-cookbook, dataops-pipeline, data-testing-ci-cd, circuit-breaker-testing, analytical-sandboxes, automated-data-verification.",
        r#"# Analytics Dataops Automated Testing
> Based on **The DataOps Cookbook - Christopher Bergh et al.**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- DataOps Pipeline Test Execution Log
CREATE TABLE dataops_test_executions (
    test_run_id UUID PRIMARY KEY,
    pipeline_id VARCHAR(100) NOT NULL,
    git_commit_sha CHAR(40) NOT NULL,
    environment VARCHAR(20) NOT NULL CHECK (environment IN ('DEV', 'STAGING', 'PROD')),
    tests_executed INT NOT NULL,
    tests_passed INT NOT NULL,
    tests_failed INT NOT NULL,
    circuit_breaker_triggered BOOLEAN NOT NULL,
    executed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Production Circuit Breaker Invariant
Let $T = \{t_1, t_2, \dots, t_k\}$ be the pre-flight assertion suite executed on staging buffer $S$:
$$\text{If } \sum_{i=1}^k [t_i(S) = \text{FAIL}] > 0 \implies \text{TRIGGER CIRCUIT BREAKER}$$
$$\text{Action}: \quad \text{ROLLBACK TRANSACTION} \land \text{PREVENT PROD MERGE} \land \text{ALERT}$$
Dirty data is strictly quarantined before reaching consumer-facing tables.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    CodeCommit[Git Commit: New Data Model] --> CI[CI Runner: Spin Up Ephemeral DuckDB / Postgres Sandbox]
    CI --> RunTests[Execute Unit & Schema Tests]
    RunTests --> TestResult{All Tests Pass?}
    TestResult -->|No| BlockPR[Block Pull Request]
    TestResult -->|Yes| Deploy[Merge to Main & Deploy to Staging]
    Deploy --> StagingIngest[Ingest Data into Staging Buffer]
    StagingIngest --> CircuitBreaker{Pre-flight Integrity Assertions Pass?}
    CircuitBreaker -->|Fail| Abort[Rollback Transaction & Alert On-Call]
    CircuitBreaker -->|Pass| Promote[Atomic Swap / Upsert into Production Tables]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
# Python DataOps Circuit Breaker Runner
def run_preflight_circuit_breaker(connection, checks: list[str]) -> bool:
    cursor = connection.cursor()
    for query in checks:
        cursor.execute(query)
        result = cursor.fetchone()[0]
        if result > 0: # Check query returns count of invalid rows
            connection.rollback()
            return False # Circuit breaker triggered
    connection.commit()
    return True
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Apply DataOps principles: version control all code, automate CI/CD testing, spin up isolated sandboxes.
- Implement automated circuit breakers: abort the pipeline and rollback if pre-flight assertions fail.
- Test both data code (unit tests in CI) and data content (integrity tests on staging data).
- Ensure environment parity: development sandboxes must mirror production schema and constraints.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Build enterprise DataOps continuous integration and automated testing systems:
1. Establish automated CI/CD workflows provisioning ephemeral DuckDB/PostgreSQL sandboxes on every PR.
2. Deploy pre-flight transactional circuit breakers aborting data loads on integrity violations.
3. Implement automated regression testing harnesses comparing production vs pull-request data diffs.
```
"#,
    )
}

/// Discover and load all ECC skills from a directory (scanning both `*.md` and `<dir>/SKILL.md`)
pub fn load_skills_from_dir(dir: impl AsRef<Path>) -> Vec<EccSkill> {
    let dir_ref = dir.as_ref();
    if !dir_ref.is_dir() {
        return Vec::new();
    }

    let mut candidate_targets = Vec::new();
    if let Ok(entries) = fs::read_dir(dir_ref) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                candidate_targets.push(path);
            } else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                let skill_md_alt = path.join("skill.md");
                if skill_md.is_file() {
                    candidate_targets.push(skill_md);
                } else if skill_md_alt.is_file() {
                    candidate_targets.push(skill_md_alt);
                }
            }
        }
    }

    use rayon::prelude::*;
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        .min(4);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .thread_name(|i| format!("tgs-skill-loader-{}", i))
        .build();

    let mut skills: Vec<EccSkill> = match pool {
        Ok(p) => p.install(|| {
            candidate_targets
                .par_iter()
                .filter_map(|target| match EccSkill::from_file(target) {
                    Ok(skill) => Some(skill),
                    Err(e) => {
                        warn!("Failed to load skill from '{}': {}", target.display(), e);
                        None
                    }
                })
                .collect()
        }),
        Err(_) => candidate_targets
            .into_iter()
            .filter_map(|target| match EccSkill::from_file(&target) {
                Ok(skill) => Some(skill),
                Err(e) => {
                    warn!("Failed to load skill from '{}': {}", target.display(), e);
                    None
                }
            })
            .collect(),
    };

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Retrieve an ECC skill by name, checking built-in skills first, then an optional disk directory
pub fn resolve_skill(name: &str, custom_dir: Option<&Path>) -> Option<EccSkill> {
    let default_skills_dir = Path::new(".ecc/skills");
    if custom_dir.is_none() || custom_dir == Some(default_skills_dir) {
        if let Some(skill) = global_dispatcher().get_skill(name) {
            return Some(skill);
        }
    }

    if let Some(skill) = find_built_in_skill(name) {
        return Some(skill);
    }

    if let Some(dir) = custom_dir {
        let lower = name.to_lowercase().replace('_', "-");

        // 1. Fast O(1) direct path lookup by directory name
        let candidate_dir = dir.join(&lower);
        if candidate_dir.is_dir() {
            let skill_md = candidate_dir.join("SKILL.md");
            let skill_md_alt = candidate_dir.join("skill.md");
            let target = if skill_md.is_file() {
                Some(skill_md)
            } else if skill_md_alt.is_file() {
                Some(skill_md_alt)
            } else {
                None
            };
            if let Some(file) = target {
                if let Ok(skill) = EccSkill::from_file(&file) {
                    return Some(skill);
                }
            }
        }

        // 2. Scan all loaded skills
        let loaded = load_skills_from_dir(dir);
        if let Some(skill) = loaded.iter().find(|s| {
            s.name.eq_ignore_ascii_case(&lower)
                || s.name.eq_ignore_ascii_case(lower.trim_start_matches("oracle-"))
                || s.name.eq_ignore_ascii_case(lower.trim_start_matches("oci-"))
                || format!("oracle-{}", s.name).eq_ignore_ascii_case(&lower)
                || format!("oci-{}", s.name).eq_ignore_ascii_case(&lower)
        }) {
            return Some(skill.clone());
        }
    }

    None
}

// =========================================================================
// Tagisan Automated Skill Dispatcher ("Right Tools/Skills for Right Job")
// =========================================================================

/// Pre-indexed metadata for deterministic sub-millisecond ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub id: usize,
    pub name: String,
    pub domain: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub is_builtin: bool,
    /// Pre-computed normalized term frequency sparse vector: (term_id, normalized_weight) sorted by term_id
    pub tfidf_vector: Vec<(u32, f32)>,
    /// Pre-computed L2 norm
    pub norm: f32,
    /// Tokenized name terms for Jaccard overlap
    pub name_tokens: Vec<String>,
}

/// Result of dispatching a skill query
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchedSkill {
    pub skill: EccSkill,
    pub score: f32,
    pub matched_triggers: Vec<String>,
    pub domain: String,
}

/// Injection representation mode for LLM prompt assembly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionMode {
    /// Token-compressed AST-extracted invariant DSL (ALWAYS, NEVER, STRICT_REJECT, AUDIT)
    DenseInvariants,
    /// Full architectural specification with workflows, code samples, templates
    Comprehensive,
    /// 3-Tier progressive disclosure: Tier 1 (Capability manifest) + Tier 2 (Top invariants) + Tier 3 (JIT tool hint)
    Hierarchical,
    /// Local cheat sheet (top 12 extracted bullet rules)
    CheatSheet,
}

/// Token budget and injection constraints for LLM contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Estimated total context window of the target LLM
    pub context_window: usize,
    /// Maximum tokens reserved for skill prompt injection
    pub max_tokens: usize,
    /// Formatting / representation mode
    pub mode: InjectionMode,
    /// Maximum skills per domain to enforce orthogonal multi-domain coverage
    pub max_per_domain: usize,
}

impl TokenBudget {
    pub fn new(context_window: usize, max_tokens: usize, mode: InjectionMode, max_per_domain: usize) -> Self {
        Self {
            context_window,
            max_tokens,
            mode,
            max_per_domain,
        }
    }

    /// Auto-calibrate optimal TokenBudget based on provider ID and model name
    pub fn for_provider_and_model(provider: &str, model: Option<&str>) -> Self {
        let is_local = SkillDispatcher::is_local_provider(provider);
        let model_lower = model.unwrap_or("").to_lowercase();

        if is_local {
            let (ctx, budget, mode) = if model_lower.contains("32k")
                || model_lower.contains("qwen2.5")
                || model_lower.contains("mistral")
                || model_lower.contains("deepseek-r1:14b")
                || model_lower.contains("deepseek-r1:32b")
            {
                (32_768, 3_500, InjectionMode::Hierarchical)
            } else if model_lower.contains("16k") {
                (16_384, 2_000, InjectionMode::DenseInvariants)
            } else {
                (8_192, 1_200, InjectionMode::DenseInvariants)
            };
            Self {
                context_window: ctx,
                max_tokens: budget,
                mode,
                max_per_domain: 1, // Enforce diversity: max 1 skill per domain on local
            }
        } else {
            let (ctx, budget) = if provider.eq_ignore_ascii_case("gemini") || model_lower.contains("gemini") {
                (1_000_000, 16_000)
            } else if provider.eq_ignore_ascii_case("anthropic") || model_lower.contains("claude") {
                (200_000, 12_000)
            } else {
                (128_000, 8_000)
            };
            Self {
                context_window: ctx,
                max_tokens: budget,
                mode: InjectionMode::Hierarchical,
                max_per_domain: 2, // Allow up to 2 skills per domain for cloud
            }
        }
    }

    pub fn local_default() -> Self {
        Self::for_provider_and_model("ollama", None)
    }

    pub fn cloud_default() -> Self {
        Self::for_provider_and_model("anthropic", None)
    }

    pub fn with_mode(mut self, mode: InjectionMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_max_per_domain(mut self, max_per_domain: usize) -> Self {
        self.max_per_domain = max_per_domain;
        self
    }
}

/// Detailed result of diversified skill dispatching
#[derive(Debug, Clone, PartialEq)]
pub struct DiversifiedDispatchResult {
    /// Primary skills selected to receive deep invariant/specification injection within budget
    pub primary: Vec<DispatchedSkill>,
    /// Secondary radar/manifest skills included in the Tier 1 capability registry table
    pub manifest: Vec<DispatchedSkill>,
    /// Total estimated tokens consumed by primary skills
    pub total_estimated_tokens: usize,
    /// The token budget configuration used for this dispatch
    pub budget: TokenBudget,
}

/// Sub-microsecond deterministic token estimator calibrated for code, markdown, and prose
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let bytes = text.as_bytes();
    let char_len = bytes.len();
    let mut words = 0;
    let mut punct = 0;
    let mut in_word = false;

    for &b in bytes {
        if b.is_ascii_whitespace() {
            in_word = false;
        } else if b.is_ascii_punctuation() {
            punct += 1;
            in_word = false;
        } else if !in_word {
            words += 1;
            in_word = true;
        }
    }

    let estimated = words + (punct / 2);
    let min_bound = char_len / 5;
    let max_bound = (char_len / 3).max(1);

    estimated.clamp(min_bound, max_bound).max(1)
}

pub const SKILLS_CACHE_MAGIC: u32 = 0x54475331; // "TGS1"
pub const SKILLS_CACHE_VERSION: u32 = 2;

/// Binary serialized cache container for fast cold startup (< 2ms)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCatalog {
    pub magic: u32,
    pub version: u32,
    pub builtin_count: usize,
    pub dir_fingerprint: u64,
    pub skills: Vec<SkillMetadata>,
    pub inverted_index: HashMap<u32, Vec<usize>>,
    pub trigger_index: HashMap<String, Vec<usize>>,
    pub name_index: HashMap<String, usize>,
    pub vocab: HashMap<String, u32>,
    pub idf: Vec<f32>,
}

/// Compute a fast BLAKE3-based fingerprint of a skill directory
pub fn compute_dir_fingerprint(dir: Option<&Path>) -> u64 {
    let Some(dir) = dir else { return 0 };
    if !dir.is_dir() {
        return 0;
    }

    let mut hasher = blake3::Hasher::new();
    if let Ok(meta) = dir.metadata() {
        if let Ok(mtime) = meta.modified() {
            if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                hasher.update(&dur.as_secs().to_le_bytes());
                hasher.update(&dur.subsec_nanos().to_le_bytes());
            }
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut count = 0u64;
        let mut max_secs = 0u64;
        let mut max_nanos = 0u32;
        for entry in entries.flatten() {
            count += 1;
            let p = entry.path();
            let target_mtime = if p.is_dir() {
                let s1 = p.join("SKILL.md");
                let s2 = p.join("skill.md");
                if let Ok(m) = s1.metadata() {
                    m.modified().ok()
                } else if let Ok(m) = s2.metadata() {
                    m.modified().ok()
                } else {
                    entry.metadata().ok().and_then(|m| m.modified().ok())
                }
            } else {
                entry.metadata().ok().and_then(|m| m.modified().ok())
            };

            if let Some(t) = target_mtime {
                if let Ok(dur) = t.duration_since(std::time::UNIX_EPOCH) {
                    let s = dur.as_secs();
                    let n = dur.subsec_nanos();
                    if s > max_secs || (s == max_secs && n > max_nanos) {
                        max_secs = s;
                        max_nanos = n;
                    }
                }
            }
        }
        hasher.update(&count.to_le_bytes());
        hasher.update(&max_secs.to_le_bytes());
        hasher.update(&max_nanos.to_le_bytes());
    }

    let hash = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[0..8]);
    u64::from_le_bytes(bytes)
}

/// In-memory hybrid skill dispatcher capable of ranking 3,840+ skills in < 0.5ms
pub struct SkillDispatcher {
    skills: Vec<SkillMetadata>,
    /// Inverted index: term_id -> list of skill IDs
    inverted_index: HashMap<u32, Vec<usize>>,
    /// Fast exact trigger map: lowercased_trigger -> list of skill IDs
    trigger_index: HashMap<String, Vec<usize>>,
    /// Fast exact name map: lowercased_name -> skill ID
    name_index: HashMap<String, usize>,
    /// Lexicon mapping term -> term_id
    vocab: HashMap<String, u32>,
    /// Inverse document frequencies
    idf: Vec<f32>,
    /// JIT cache of loaded full EccSkill bodies
    skill_cache: RwLock<HashMap<usize, EccSkill>>,
}

static GLOBAL_DISPATCHER: OnceLock<SkillDispatcher> = OnceLock::new();

/// Return a static reference to the lazily initialized global SkillDispatcher singleton
pub fn global_dispatcher() -> &'static SkillDispatcher {
    GLOBAL_DISPATCHER.get_or_init(SkillDispatcher::default_catalog)
}

impl SkillDispatcher {
    /// Default cache path for skills: .tagisan/skills.cache
    pub fn default_cache_path() -> PathBuf {
        PathBuf::from(".tagisan/skills.cache")
    }

    /// Try loading the compiled skill catalog from binary cache using memory-mapped I/O (< 2ms)
    pub fn try_load_from_cache(
        cache_path: &Path,
        expected_builtin_count: usize,
        expected_fingerprint: u64,
    ) -> Option<Self> {
        let file = File::open(cache_path).ok()?;
        let mmap = unsafe { memmap2::Mmap::map(&file).ok()? };
        let catalog: CachedCatalog = bincode::deserialize(&mmap).ok()?;

        if catalog.magic != SKILLS_CACHE_MAGIC
            || catalog.version != SKILLS_CACHE_VERSION
            || catalog.builtin_count != expected_builtin_count
            || catalog.dir_fingerprint != expected_fingerprint
        {
            return None;
        }

        Some(Self {
            skills: catalog.skills,
            inverted_index: catalog.inverted_index,
            trigger_index: catalog.trigger_index,
            name_index: catalog.name_index,
            vocab: catalog.vocab,
            idf: catalog.idf,
            skill_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Serialize and write the skill catalog to binary cache atomically
    pub fn save_to_cache(&self, cache_path: &Path, dir_fingerprint: u64) -> Result<()> {
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let catalog = CachedCatalog {
            magic: SKILLS_CACHE_MAGIC,
            version: SKILLS_CACHE_VERSION,
            builtin_count: all_built_in_skills().len(),
            dir_fingerprint,
            skills: self.skills.clone(),
            inverted_index: self.inverted_index.clone(),
            trigger_index: self.trigger_index.clone(),
            name_index: self.name_index.clone(),
            vocab: self.vocab.clone(),
            idf: self.idf.clone(),
        };

        let encoded = bincode::serialize(&catalog)
            .map_err(|e| TagisanError::Execution(format!("Failed to serialize skills cache: {}", e)))?;

        let temp_path = cache_path.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&temp_path, encoded)
            .map_err(|e| TagisanError::Execution(format!("Failed to write skills cache temp: {}", e)))?;
        fs::rename(&temp_path, cache_path)
            .map_err(|e| TagisanError::Execution(format!("Failed to commit skills cache: {}", e)))?;
        Ok(())
    }

    /// Load or build skill dispatcher with optional explicit cache path
    pub fn load_or_build_with_cache(custom_dir: Option<&Path>, cache_path: Option<&Path>) -> Self {
        let builtin_count = all_built_in_skills().len();
        let fingerprint = compute_dir_fingerprint(custom_dir);

        if let Some(cp) = cache_path {
            if let Some(dispatcher) = Self::try_load_from_cache(cp, builtin_count, fingerprint) {
                return dispatcher;
            }
        }

        let dispatcher = Self::build_from_scratch(custom_dir);
        if let Some(cp) = cache_path {
            if let Err(e) = dispatcher.save_to_cache(cp, fingerprint) {
                eprintln!("\n[TAGISAN CACHE SAVE ERROR]: {}\n", e);
            }
        }
        dispatcher
    }

    /// Load or build skill dispatcher using default `.tagisan/skills.cache`
    pub fn load_or_build(custom_dir: Option<&Path>) -> Self {
        let default_cache = Self::default_cache_path();
        Self::load_or_build_with_cache(custom_dir, Some(&default_cache))
    }

    /// Invalidate existing cache file
    pub fn invalidate_cache(cache_path: &Path) {
        let _ = fs::remove_file(cache_path);
    }

    /// Initialize dispatcher from the standard locations (.ecc/skills + built-ins)
    pub fn default_catalog() -> Self {
        let skills_dir = Path::new(".ecc/skills");
        let custom_dir = if skills_dir.exists() {
            Some(skills_dir)
        } else {
            None
        };
        Self::load_or_build(custom_dir)
    }

    /// Initialize dispatcher scanning built-ins and an optional custom skills directory
    pub fn new(custom_dir: Option<&Path>) -> Self {
        Self::load_or_build(custom_dir)
    }

    /// Build dispatcher index from scratch using bounded Rayon and O(1) built-in bypass
    pub fn build_from_scratch(custom_dir: Option<&Path>) -> Self {
        struct RawEntry {
            name: String,
            description: String,
            triggers: Vec<String>,
            file_path: Option<PathBuf>,
            is_builtin: bool,
        }

        let mut raw_entries: Vec<RawEntry> = Vec::new();

        // Collect built-in skill names for O(1) bypass
        let mut built_in_names = HashSet::new();

        // 1. Ingest built-in skills (lightweight metadata only; JIT body loading!)
        for s in all_built_in_skills() {
            let raw_trigs = if let Some((_, _, trigs)) = parse_frontmatter_metadata(&s.instructions, Some(&s.name)) {
                trigs
            } else {
                Vec::new()
            };
            let trigs = extract_triggers_from_text(&s.name, &s.description, &raw_trigs);
            built_in_names.insert(s.name.to_lowercase());
            raw_entries.push(RawEntry {
                name: s.name,
                description: s.description,
                triggers: trigs,
                file_path: None,
                is_builtin: true,
            });
        }

        // 2. Scan disk directory with O(1) fast built-in bypass and Bounded Rayon Parallel Scanner
        if let Some(dir) = custom_dir {
            if dir.is_dir() {
                let mut candidate_targets = Vec::new();
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let folder_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                            // O(1) fast bypass if already built in!
                            if built_in_names.contains(&folder_name.to_lowercase()) {
                                continue;
                            }
                            let skill_md = path.join("SKILL.md");
                            let skill_md_alt = path.join("skill.md");
                            if skill_md.is_file() {
                                candidate_targets.push((skill_md, folder_name.to_string()));
                            } else if skill_md_alt.is_file() {
                                candidate_targets.push((skill_md_alt, folder_name.to_string()));
                            }
                        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                            if built_in_names.contains(&file_stem.to_lowercase()) {
                                continue;
                            }
                            candidate_targets.push((path, file_stem));
                        }
                    }
                }

                // Bounded Rayon pool: strictly max 4 threads to protect dual-core Core i3 laptop
                let num_threads = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(2)
                    .min(4);

                use rayon::prelude::*;
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(num_threads)
                    .thread_name(|i| format!("tgs-skill-scanner-{}", i))
                    .build();

                let disk_entries: Vec<RawEntry> = match pool {
                    Ok(p) => p.install(|| {
                        candidate_targets
                            .par_iter()
                            .filter_map(|(target_file, fallback_name)| {
                                if let Ok(mut file) = File::open(target_file) {
                                    let mut buf = [0u8; 8192];
                                    let read_bytes = file.read(&mut buf).unwrap_or(0);
                                    let header = String::from_utf8_lossy(&buf[..read_bytes]);
                                    if let Some((name, desc, raw_trigs)) = parse_frontmatter_metadata(&header, Some(fallback_name)) {
                                        let trigs = extract_triggers_from_text(&name, &desc, &raw_trigs);
                                        Some(RawEntry {
                                            name,
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file.clone()),
                                            is_builtin: false,
                                        })
                                    } else {
                                        let first_line = header.lines().find(|l| !l.trim().is_empty()).unwrap_or(fallback_name);
                                        let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                                        let trigs = extract_triggers_from_text(fallback_name, &desc, &[]);
                                        Some(RawEntry {
                                            name: fallback_name.clone(),
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file.clone()),
                                            is_builtin: false,
                                        })
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }),
                    Err(_) => {
                        candidate_targets
                            .into_iter()
                            .filter_map(|(target_file, fallback_name)| {
                                if let Ok(mut file) = File::open(&target_file) {
                                    let mut buf = [0u8; 8192];
                                    let read_bytes = file.read(&mut buf).unwrap_or(0);
                                    let header = String::from_utf8_lossy(&buf[..read_bytes]);
                                    if let Some((name, desc, raw_trigs)) = parse_frontmatter_metadata(&header, Some(&fallback_name)) {
                                        let trigs = extract_triggers_from_text(&name, &desc, &raw_trigs);
                                        Some(RawEntry {
                                            name,
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file),
                                            is_builtin: false,
                                        })
                                    } else {
                                        let first_line = header.lines().find(|l| !l.trim().is_empty()).unwrap_or(&fallback_name);
                                        let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                                        let trigs = extract_triggers_from_text(&fallback_name, &desc, &[]);
                                        Some(RawEntry {
                                            name: fallback_name,
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file),
                                            is_builtin: false,
                                        })
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                };

                raw_entries.extend(disk_entries);
            }
        }

        let num_docs = raw_entries.len().max(1) as f32;

        // 3. Build Vocabulary & Document Frequencies
        let mut doc_term_counts: Vec<HashMap<String, (usize, usize)>> = Vec::with_capacity(raw_entries.len());
        let mut df: HashMap<String, usize> = HashMap::new();

        for entry in &raw_entries {
            let mut counts: HashMap<String, (usize, usize)> = HashMap::new();
            for token in tokenize(&entry.name) {
                counts.entry(token).or_insert((0, 0)).0 += 1;
            }
            for token in tokenize(&entry.description) {
                counts.entry(token).or_insert((0, 0)).1 += 1;
            }
            for term in counts.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
            doc_term_counts.push(counts);
        }

        // Sort terms for deterministic IDs
        let mut sorted_vocab: Vec<String> = df.keys().cloned().collect();
        sorted_vocab.sort();

        let mut vocab = HashMap::new();
        let mut idf = Vec::with_capacity(sorted_vocab.len());

        for (idx, term) in sorted_vocab.into_iter().enumerate() {
            let doc_freq = df.get(&term).copied().unwrap_or(1) as f32;
            // Smoothed BM25-style IDF: ln(1.0 + (N - df + 0.5) / (df + 0.5)) + 1.0
            let term_idf = ((num_docs - doc_freq + 0.5) / (doc_freq + 0.5)).ln_1p() + 1.0;
            vocab.insert(term, idx as u32);
            idf.push(term_idf);
        }

        // 4. Build SkillMetadata, Inverted Index & Name/Trigger Indices
        let mut skills = Vec::with_capacity(raw_entries.len());
        let mut inverted_index: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut trigger_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut name_index: HashMap<String, usize> = HashMap::new();

        for (id, (entry, counts)) in raw_entries.into_iter().zip(doc_term_counts).enumerate() {
            let domain = infer_domain(&entry.name);
            let name_tokens = tokenize(&entry.name);

            // Calculate TF-IDF vector with field boosting (Name: 3.0x, Desc: 1.0x)
            let mut sparse_vec: Vec<(u32, f32)> = Vec::new();
            let mut norm_sq = 0.0f32;

            for (term, (name_count, desc_count)) in counts {
                if let Some(&tid) = vocab.get(&term) {
                    let weighted_count = (name_count as f32 * 3.0) + (desc_count as f32 * 1.0);
                    if weighted_count > 0.0 {
                        let tf = 1.0 + weighted_count.ln();
                        let weight = tf * idf[tid as usize];
                        sparse_vec.push((tid, weight));
                        norm_sq += weight * weight;
                    }
                }
            }

            // Sort sparse vector by term_id for fast two-pointer dot product
            sparse_vec.sort_by_key(|&(tid, _)| tid);

            let norm = norm_sq.sqrt();
            let normalized_vec: Vec<(u32, f32)> = if norm > 1e-6 {
                sparse_vec.into_iter().map(|(tid, w)| (tid, w / norm)).collect()
            } else {
                sparse_vec
            };

            for &(tid, _) in &normalized_vec {
                inverted_index.entry(tid).or_default().push(id);
            }

            // Index triggers
            for tr in &entry.triggers {
                let lower_tr = tr.to_lowercase();
                trigger_index.entry(lower_tr).or_default().push(id);
            }

            // Index name
            let lower_name = entry.name.to_lowercase();
            name_index.insert(lower_name.clone(), id);
            let spaced = lower_name.replace('-', " ");
            if spaced != lower_name {
                name_index.insert(spaced, id);
            }

            skills.push(SkillMetadata {
                id,
                name: entry.name,
                domain,
                description: entry.description,
                triggers: entry.triggers,
                file_path: entry.file_path,
                is_builtin: entry.is_builtin,
                tfidf_vector: normalized_vec,
                norm,
                name_tokens,
            });
        }

        Self {
            skills,
            inverted_index,
            trigger_index,
            name_index,
            vocab,
            idf,
            skill_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Retrieve total number of indexed skills
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Retrieve read-only slice of all indexed skills
    pub fn skills(&self) -> &[SkillMetadata] {
        &self.skills
    }

    /// Retrieve single skill by exact or normalized name
    pub fn get_skill(&self, name: &str) -> Option<EccSkill> {
        let lower = name.to_lowercase().replace('_', "-");
        if let Some(&doc_id) = self.name_index.get(&lower) {
            return self.load_skill_by_id(doc_id);
        }

        if lower == "rust-tokio-concurrency" || lower == "rust-concurrency" || lower == "tokio-concurrency" {
            if let Some(&doc_id) = self.name_index.get("tokio-async-tuning") {
                if let Some(s) = self.load_skill_by_id(doc_id) {
                    return Some(s);
                }
            }
        }

        // Try prefixes
        for prefix in ["oracle-", "oci-", "azure-", "aws-", "ibm-", "sap-", "alibaba-", "sf-"] {
            let stripped = lower.trim_start_matches(prefix);
            if let Some(&doc_id) = self.name_index.get(stripped) {
                return self.load_skill_by_id(doc_id);
            }
        }

        find_built_in_skill(name)
    }

    /// Load full EccSkill body lazily from cache or disk
    pub fn load_skill_by_id(&self, doc_id: usize) -> Option<EccSkill> {
        if doc_id >= self.skills.len() {
            return None;
        }

        // 1. Check read lock
        {
            if let Ok(cache) = self.skill_cache.read() {
                if let Some(skill) = cache.get(&doc_id) {
                    return Some(skill.clone());
                }
            }
        }

        // 2. Read from disk
        let meta = &self.skills[doc_id];
        let skill = if let Some(ref path) = meta.file_path {
            EccSkill::from_file(path).unwrap_or_else(|_| {
                EccSkill::new(&meta.name, &meta.description, &meta.description)
            })
        } else {
            find_built_in_skill(&meta.name).unwrap_or_else(|| {
                EccSkill::new(&meta.name, &meta.description, &meta.description)
            })
        };

        // 3. Populate write lock
        if let Ok(mut cache) = self.skill_cache.write() {
            cache.insert(doc_id, skill.clone());
        }

        Some(skill)
    }

    /// Dynamically register an external or plugin skill into the in-memory index
    pub fn register_skill(&mut self, skill: EccSkill) {
        let id = self.skills.len();
        let domain = infer_domain(&skill.name);
        let trigs = extract_triggers_from_text(&skill.name, &skill.description, &[]);
        let name_tokens = tokenize(&skill.name);

        let lower_name = skill.name.to_lowercase();
        self.name_index.insert(lower_name.clone(), id);
        let spaced = lower_name.replace('-', " ");
        if spaced != lower_name {
            self.name_index.insert(spaced, id);
        }

        for tr in &trigs {
            self.trigger_index.entry(tr.to_lowercase()).or_default().push(id);
        }

        if let Ok(mut cache) = self.skill_cache.write() {
            cache.insert(id, skill.clone());
        }

        self.skills.push(SkillMetadata {
            id,
            name: skill.name,
            domain,
            description: skill.description,
            triggers: trigs,
            file_path: None,
            is_builtin: false,
            tfidf_vector: Vec::new(),
            norm: 0.0,
            name_tokens,
        });
    }

    /// Rank and dispatch top-K skills matching query in < 0.5ms
    pub fn dispatch(
        &self,
        query: &str,
        top_k: usize,
        domain_bias: Option<&str>,
    ) -> Vec<DispatchedSkill> {
        if self.skills.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let lower_query = query.to_lowercase();
        let query_tokens = tokenize(query);

        // 1. Gather candidate skills via Inverted Index and Trigger Index
        let mut candidates = HashSet::new();

        for token in &query_tokens {
            if let Some(&tid) = self.vocab.get(token) {
                if let Some(doc_ids) = self.inverted_index.get(&tid) {
                    candidates.extend(doc_ids.iter().copied());
                }
            }
        }

        // 2. Fast query n-grams lookup in trigger_index and name_index (1-gram to 5-gram)
        let words: Vec<&str> = lower_query.split_whitespace().collect();
        let max_n = 5.min(words.len());
        for n in 1..=max_n {
            for window in words.windows(n) {
                let phrase = window.join(" ");
                if let Some(doc_ids) = self.trigger_index.get(&phrase) {
                    candidates.extend(doc_ids.iter().copied());
                }
                if let Some(&doc_id) = self.name_index.get(&phrase) {
                    candidates.insert(doc_id);
                }
                let hyphenated = window.join("-");
                if let Some(&doc_id) = self.name_index.get(&hyphenated) {
                    candidates.insert(doc_id);
                }
                if let Some(doc_ids) = self.trigger_index.get(&hyphenated) {
                    candidates.extend(doc_ids.iter().copied());
                }
            }
        }

        // Fallback: if candidates empty, score all documents
        let candidate_list: Vec<usize> = if candidates.is_empty() {
            (0..self.skills.len()).collect()
        } else {
            candidates.into_iter().collect()
        };

        // 2. Compute query sparse TF-IDF vector
        let mut q_counts: HashMap<u32, usize> = HashMap::new();
        for token in &query_tokens {
            if let Some(&tid) = self.vocab.get(token) {
                *q_counts.entry(tid).or_insert(0) += 1;
            }
        }

        let mut query_vec: Vec<(u32, f32)> = Vec::new();
        let mut q_norm_sq = 0.0f32;

        for (tid, count) in q_counts {
            let tf = 1.0 + (count as f32).ln();
            let weight = tf * self.idf[tid as usize];
            query_vec.push((tid, weight));
            q_norm_sq += weight * weight;
        }

        query_vec.sort_by_key(|&(tid, _)| tid);

        let q_norm = q_norm_sq.sqrt();
        let normalized_q_vec: Vec<(u32, f32)> = if q_norm > 1e-6 {
            query_vec.into_iter().map(|(tid, w)| (tid, w / q_norm)).collect()
        } else {
            query_vec
        };

        // 3. Multi-factor scoring loop
        let mut ranked: Vec<(usize, f32, Vec<String>)> = Vec::with_capacity(candidate_list.len());

        for doc_id in candidate_list {
            let meta = &self.skills[doc_id];
            let mut score = 0.0f32;
            let mut matched_triggers = Vec::new();

            // 3a. Exact Name Match (+200.0)
            let lower_name = meta.name.to_lowercase();
            let spaced_name = lower_name.replace('-', " ");
            if lower_query.contains(&lower_name) || lower_query.contains(&spaced_name) {
                score += 200.0;
            }

            // 3b. Trigger Matches (+100 for first, +25 subsequent, max +150)
            let mut trigger_score = 0.0f32;
            for tr in &meta.triggers {
                let matches = if tr.len() <= 3 {
                    words.contains(&tr.as_str())
                } else {
                    lower_query.contains(tr.as_str()) || (tr.contains('-') && lower_query.contains(&tr.replace('-', " ")))
                };
                if tr.len() >= 3 && matches {
                    if trigger_score == 0.0 {
                        trigger_score += 100.0;
                    } else if trigger_score < 150.0 {
                        trigger_score = (trigger_score + 25.0).min(150.0);
                    }
                    matched_triggers.push(tr.clone());
                }
            }
            score += trigger_score;

            // 3c. Domain Prefix & Stage Bias Boost
            let mut domain_score = 0.0f32;
            if let Some(bias) = domain_bias {
                if bias.eq_ignore_ascii_case(&meta.domain) {
                    domain_score += 35.0;
                }
            }
            if lower_query.contains(&meta.domain) || query_tokens.iter().any(|t| t == &meta.domain) {
                domain_score += 25.0;
            }
            score += domain_score.min(60.0);

            // 3d. Name Token Overlap (+30.0)
            if !meta.name_tokens.is_empty() {
                let overlap = meta.name_tokens.iter().filter(|t| query_tokens.contains(t)).count();
                score += (overlap as f32 / meta.name_tokens.len() as f32) * 30.0;
            }

            // 3e. Sublinear TF-IDF Cosine Similarity (+40.0)
            if !normalized_q_vec.is_empty() && !meta.tfidf_vector.is_empty() {
                let mut dot = 0.0f32;
                let mut p_q = 0;
                let mut p_d = 0;
                while p_q < normalized_q_vec.len() && p_d < meta.tfidf_vector.len() {
                    let (q_tid, q_val) = normalized_q_vec[p_q];
                    let (d_tid, d_val) = meta.tfidf_vector[p_d];
                    if q_tid == d_tid {
                        dot += q_val * d_val;
                        p_q += 1;
                        p_d += 1;
                    } else if q_tid < d_tid {
                        p_q += 1;
                    } else {
                        p_d += 1;
                    }
                }
                score += dot * 40.0;
            }

            if score >= 10.0 {
                ranked.push((doc_id, score, matched_triggers));
            }
        }

        // Sort descending by score, tie-break by name
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| self.skills[a.0].name.cmp(&self.skills[b.0].name))
        });

        // Take Top-K
        let mut results = Vec::with_capacity(top_k.min(ranked.len()));
        for (doc_id, score, matched_trigs) in ranked.into_iter().take(top_k) {
            if let Some(skill) = self.load_skill_by_id(doc_id) {
                results.push(DispatchedSkill {
                    domain: self.skills[doc_id].domain.clone(),
                    skill,
                    score,
                    matched_triggers: matched_trigs,
                });
            }
        }

        results
    }

    /// Search and format results as Markdown for LLM prompt injection or tool returns
    pub fn search_and_format(
        &self,
        query: &str,
        limit: usize,
        domain: Option<&str>,
        include_instructions: bool,
    ) -> String {
        let results = self.dispatch(query, limit, domain);
        if results.is_empty() {
            return format!("No engineering skills found matching query: \"{query}\"");
        }

        let mut output = format!("Found {} relevant engineering skills for \"{}\":\n\n", results.len(), query);
        for (idx, d) in results.iter().enumerate() {
            output.push_str(&format!(
                "### {}. {} [Score: {:.1} | Domain: {}]\n{}\n",
                idx + 1,
                d.skill.name,
                d.score,
                d.domain,
                d.skill.description
            ));

            if !d.matched_triggers.is_empty() {
                output.push_str(&format!("Matched Triggers: {:?}\n", d.matched_triggers));
            }

            if include_instructions && !d.skill.instructions.is_empty() {
                output.push_str("\n#### Instructions:\n");
                output.push_str(d.skill.instructions.trim());
                output.push_str("\n");
            }
            output.push_str("\n---\n");
        }

        output.trim_end().to_string()
    }

    /// Check if a provider identifier corresponds to a local LLM runtime (e.g. Ollama, local)
    pub fn is_local_provider(provider: &str) -> bool {
        let p = provider.trim().to_lowercase();
        p == "ollama"
            || p == "local"
            || p.starts_with("ollama")
            || p.starts_with("local")
            || p.contains("ollama")
            || p.contains("local")
            || p.contains("llama")
            || p.contains("vllm")
    }

    /// Format dispatched skills as a condensed Cheat Sheet for local LLMs (<1,000 tokens footprint for 8k context)
    pub fn format_cheat_sheet(skills: &[DispatchedSkill]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = String::from("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]\n");
        for (idx, ds) in skills.iter().enumerate() {
            out.push_str(&format!("\n#### Skill {}: {} ({})\n", idx + 1, ds.skill.name, ds.domain));

            let mut rules: Vec<String> = Vec::new();
            for line in ds.skill.instructions.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with('#') {
                    let h_clean = trimmed.trim_start_matches('#').trim();
                    let h_lower = h_clean.to_lowercase();
                    if !h_clean.is_empty() && !h_lower.contains("overview") && !h_lower.contains("ecc") {
                        rules.push(format!("* [Context: {}]", h_clean));
                    }
                    continue;
                }
                if trimmed.starts_with('-') || trimmed.starts_with('*') {
                    let rule_text = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c.is_whitespace());
                    if !rule_text.is_empty() {
                        rules.push(format!("- {}", rule_text));
                    }
                } else if trimmed.as_bytes().first().map_or(false, |b| b.is_ascii_digit()) && trimmed.contains('.') {
                    if let Some((_, rest)) = trimmed.split_once('.') {
                        let rule_text = rest.trim();
                        if !rule_text.is_empty() {
                            rules.push(format!("- {}", rule_text));
                        }
                    }
                } else {
                    let lower = trimmed.to_lowercase();
                    if lower.starts_with("rule:")
                        || lower.starts_with("invariant:")
                        || lower.starts_with("constraint:")
                        || lower.starts_with("always")
                        || lower.starts_with("never")
                        || lower.starts_with("must")
                    {
                        rules.push(format!("- {}", trimmed));
                    }
                }
            }

            if rules.is_empty() {
                for line in ds.skill.instructions.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        rules.push(format!("- {}", trimmed));
                    }
                }
            }

            let capped_rules = rules.into_iter().take(12);
            for r in capped_rules {
                out.push_str(&r);
                out.push('\n');
            }
        }
        out.trim().to_string()
    }

    /// Format dispatched skills as a Comprehensive Architectural Specification mode with full instructions, checklists, invariants
    pub fn format_cloud_guidelines(skills: &[DispatchedSkill]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = String::from("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]\n");
        for (idx, ds) in skills.iter().enumerate() {
            out.push_str(&format!(
                "\n---\n#### Skill {}: {} [Domain: {} | Match Score: {:.1}]\n",
                idx + 1, ds.skill.name, ds.domain, ds.score
            ));
            out.push_str(&format!("**Description:** {}\n\n", ds.skill.description.trim()));
            if !ds.matched_triggers.is_empty() {
                out.push_str(&format!("**Triggers:** {}\n\n", ds.matched_triggers.join(", ")));
            }
            out.push_str("#### Full Specification & Directives:\n");
            out.push_str(ds.skill.instructions.trim());
            out.push_str("\n");
        }
        out.trim().to_string()
    }

    /// Format dispatched skills as a dense invariant DSL (ALWAYS, NEVER, STRICT_REJECT, AUDIT)
    /// Yields 60-75% token reduction compared to conversational markdown instructions.
    pub fn format_dense_invariants(skills: &[DispatchedSkill]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = String::from("### [TAGISAN ECC INVARIANT DIRECTIVES: HIGH-DENSITY ENFORCEMENT]\n");
        for (idx, ds) in skills.iter().enumerate() {
            out.push_str(&format!(
                "\n#### Skill {}: {} [{}]\n",
                idx + 1, ds.skill.name, ds.domain
            ));
            out.push_str("INVARIANTS:\n");

            let mut extracted: Vec<String> = Vec::new();
            let mut in_frontmatter = false;
            let mut in_code_block = false;

            for line in ds.skill.instructions.lines() {
                let trimmed = line.trim();
                if trimmed == "---" {
                    in_frontmatter = !in_frontmatter;
                    continue;
                }
                if in_frontmatter {
                    continue;
                }
                if trimmed.starts_with("```") {
                    in_code_block = !in_code_block;
                    continue;
                }
                if in_code_block {
                    continue;
                }
                if trimmed.starts_with('#') || trimmed.starts_with('>') || trimmed.is_empty() {
                    continue;
                }

                let clean = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c.is_ascii_digit() || c == '.' || c.is_whitespace());
                if clean.is_empty() {
                    continue;
                }

                let lower = clean.to_lowercase();
                let is_rule = trimmed.starts_with('-')
                    || trimmed.starts_with('*')
                    || (trimmed.as_bytes().first().map_or(false, |b| b.is_ascii_digit()) && trimmed.contains('.'))
                    || lower.contains("always")
                    || lower.contains("never")
                    || lower.contains("must")
                    || lower.contains("reject")
                    || lower.contains("audit")
                    || lower.starts_with("rule:")
                    || lower.starts_with("invariant:")
                    || lower.starts_with("constraint:");

                if !is_rule {
                    continue;
                }

                let tag = if lower.contains("never") || lower.contains("must not") || lower.contains("do not") || lower.contains("disallow") {
                    "NEVER"
                } else if lower.contains("always") || lower.contains("must ensure") || lower.contains("mandatory") || lower.contains("guarantee") {
                    "ALWAYS"
                } else if lower.contains("reject") || lower.contains("abort") || lower.contains("invalid") || lower.contains("conflict") || lower.contains("409") || lower.contains("400") {
                    "STRICT_REJECT"
                } else if lower.contains("audit") || lower.contains("log") || lower.contains("trace") || lower.contains("telemetry") || lower.contains("metric") {
                    "AUDIT"
                } else if lower.starts_with("rule:") || lower.starts_with("invariant:") || lower.starts_with("constraint:") {
                    "INVARIANT"
                } else {
                    "DIRECTIVE"
                };

                let formatted = format!("- {}: {}", tag, clean);
                if !extracted.contains(&formatted) {
                    extracted.push(formatted);
                }
            }

            if extracted.is_empty() {
                for line in ds.skill.instructions.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("```") && !trimmed.starts_with("---") {
                        extracted.push(format!("- DIRECTIVE: {}", trimmed));
                    }
                }
            }

            for item in extracted.into_iter().take(4) {
                out.push_str(&item);
                out.push('\n');
            }
        }
        out.trim().to_string()
    }

    /// Format multi-tier progressive disclosure:
    /// Tier 1: Capability Radar (Table of all relevant indexed skills)
    /// Tier 2: Operational Directives (Dense Invariants or Full Specs for top primary skills)
    /// Tier 3: JIT On-Demand Tool Invocation Hint
    pub fn format_hierarchical(
        manifest_skills: &[DispatchedSkill],
        primary_skills: &[DispatchedSkill],
        primary_mode: InjectionMode,
    ) -> String {
        let mut out = String::from("### [TAGISAN ECC MULTI-TIER COGNITIVE ARCHITECTURE]\n\n");

        // TIER 1: Capability Radar Manifest
        out.push_str("#### TIER 1: ACTIVE CAPABILITY RADAR\n");
        out.push_str("The following specialized engineering capabilities are activated in this session:\n\n");
        out.push_str("| # | Skill ID | Domain | Core Focus & Triggers |\n");
        out.push_str("|---|---|---|---|\n");

        for (idx, ds) in manifest_skills.iter().enumerate() {
            let triggers_str = if ds.matched_triggers.is_empty() {
                ds.skill.description.chars().take(80).collect::<String>()
            } else {
                ds.matched_triggers.join(", ")
            };
            let clean_summary = triggers_str.replace('|', "/").replace('\n', " ");
            out.push_str(&format!(
                "| {} | `{}` | {} | {} |\n",
                idx + 1,
                ds.skill.name,
                ds.domain,
                clean_summary
            ));
        }
        out.push('\n');

        // TIER 2: Primary Directives
        out.push_str("#### TIER 2: PRIMARY OPERATIONAL INVARIANTS\n");
        if primary_skills.is_empty() {
            out.push_str("(No primary invariants required for this prompt)\n");
        } else {
            match primary_mode {
                InjectionMode::Comprehensive => {
                    out.push_str(&Self::format_cloud_guidelines(primary_skills));
                }
                InjectionMode::CheatSheet => {
                    out.push_str(&Self::format_cheat_sheet(primary_skills));
                }
                _ => {
                    out.push_str(&Self::format_dense_invariants(primary_skills));
                }
            }
        }
        out.push_str("\n\n");

        // TIER 3: JIT Tool Hint
        out.push_str("#### TIER 3: ON-DEMAND JIT KNOWLEDGE RETRIEVAL\n");
        out.push_str("To inspect the complete specification, checklist, or template for any Tier 1 skill above, ");
        out.push_str("call the native tool `fetch_skill(name: \"<skill_id>\")` or search with `search_skills(query: \"<topic>\")`.\n");

        out.trim().to_string()
    }

    /// Retrieve full formatted specification of any skill by exact or fuzzy name
    pub fn get_skill_spec(&self, name: &str) -> Option<String> {
        self.get_skill(name).map(|s| {
            let domain = infer_domain(&s.name);
            format!(
                "# Skill: {} [Domain: {}]\n\n{}\n\n## Instructions\n{}",
                s.name, domain, s.description.trim(), s.instructions.trim()
            )
        })
    }

    /// Diversified dispatching using Maximal Marginal Relevance (MMR) and domain quota constraints.
    /// Sorts output deterministically to ensure optimal prefix stability for LLM prompt caching.
    pub fn dispatch_diversified(
        &self,
        query: &str,
        budget: TokenBudget,
        domain_bias: Option<&str>,
    ) -> DiversifiedDispatchResult {
        if self.skills.is_empty() || budget.max_tokens == 0 {
            return DiversifiedDispatchResult {
                primary: Vec::new(),
                manifest: Vec::new(),
                total_estimated_tokens: 0,
                budget,
            };
        }

        let lower_query = query.to_lowercase();
        let query_tokens = tokenize(query);

        // 1. Gather candidate skills via Inverted Index and Trigger Index
        let mut candidates = HashSet::new();

        for token in &query_tokens {
            if let Some(&tid) = self.vocab.get(token) {
                if let Some(doc_ids) = self.inverted_index.get(&tid) {
                    candidates.extend(doc_ids.iter().copied());
                }
            }
        }

        let words: Vec<&str> = lower_query.split_whitespace().collect();
        let max_n = 5.min(words.len());
        for n in 1..=max_n {
            for window in words.windows(n) {
                let phrase = window.join(" ");
                if let Some(doc_ids) = self.trigger_index.get(&phrase) {
                    candidates.extend(doc_ids.iter().copied());
                }
                if let Some(&doc_id) = self.name_index.get(&phrase) {
                    candidates.insert(doc_id);
                }
                let hyphenated = window.join("-");
                if let Some(&doc_id) = self.name_index.get(&hyphenated) {
                    candidates.insert(doc_id);
                }
                if let Some(doc_ids) = self.trigger_index.get(&hyphenated) {
                    candidates.extend(doc_ids.iter().copied());
                }
            }
        }

        if let Some(bias) = domain_bias {
            for (id, meta) in self.skills.iter().enumerate() {
                if bias.eq_ignore_ascii_case(&meta.domain) {
                    candidates.insert(id);
                }
            }
        }

        let candidate_list: Vec<usize> = if candidates.is_empty() {
            (0..self.skills.len()).collect()
        } else {
            candidates.into_iter().collect()
        };

        // 2. Compute query sparse TF-IDF vector
        let mut q_counts: HashMap<u32, usize> = HashMap::new();
        for token in &query_tokens {
            if let Some(&tid) = self.vocab.get(token) {
                *q_counts.entry(tid).or_insert(0) += 1;
            }
        }

        let mut query_vec: Vec<(u32, f32)> = Vec::new();
        let mut q_norm_sq = 0.0f32;

        for (tid, count) in q_counts {
            let tf = 1.0 + (count as f32).ln();
            let weight = tf * self.idf[tid as usize];
            query_vec.push((tid, weight));
            q_norm_sq += weight * weight;
        }

        query_vec.sort_by_key(|&(tid, _)| tid);
        let q_norm = q_norm_sq.sqrt();
        let normalized_q_vec: Vec<(u32, f32)> = if q_norm > 1e-6 {
            query_vec.into_iter().map(|(tid, w)| (tid, w / q_norm)).collect()
        } else {
            query_vec
        };

        // 3. Multi-factor scoring loop
        let mut ranked: Vec<(usize, f32, Vec<String>)> = Vec::with_capacity(candidate_list.len());

        for doc_id in candidate_list {
            let meta = &self.skills[doc_id];
            let mut score = 0.0f32;
            let mut matched_triggers = Vec::new();

            // 3a. Exact Name Match (+200.0)
            let lower_name = meta.name.to_lowercase();
            let spaced_name = lower_name.replace('-', " ");
            if lower_query.contains(&lower_name) || lower_query.contains(&spaced_name) {
                score += 200.0;
            }

            // 3b. Trigger Matches (+100 for first, +25 subsequent, max +150)
            let mut trigger_score = 0.0f32;
            for tr in &meta.triggers {
                let matches = if tr.len() <= 3 {
                    words.contains(&tr.as_str())
                } else {
                    lower_query.contains(tr.as_str()) || (tr.contains('-') && lower_query.contains(&tr.replace('-', " ")))
                };
                if tr.len() >= 3 && matches {
                    if trigger_score == 0.0 {
                        trigger_score += 100.0;
                    } else if trigger_score < 150.0 {
                        trigger_score = (trigger_score + 25.0).min(150.0);
                    }
                    matched_triggers.push(tr.clone());
                }
            }
            score += trigger_score;

            // 3c. Domain Prefix & Stage Bias Boost
            if let Some(bias) = domain_bias {
                if bias.eq_ignore_ascii_case(&meta.domain) {
                    score += 150.0;
                }
            }
            if lower_query.contains(&meta.domain) || query_tokens.iter().any(|t| t == &meta.domain) {
                score += 35.0;
            }

            // 3d. Name Token Overlap (+30.0)
            if !meta.name_tokens.is_empty() {
                let overlap = meta.name_tokens.iter().filter(|t| query_tokens.contains(t)).count();
                score += (overlap as f32 / meta.name_tokens.len() as f32) * 30.0;
            }

            // 3e. Sublinear TF-IDF Cosine Similarity (+40.0)
            if !normalized_q_vec.is_empty() && !meta.tfidf_vector.is_empty() {
                let mut dot = 0.0f32;
                let mut p_q = 0;
                let mut p_d = 0;
                while p_q < normalized_q_vec.len() && p_d < meta.tfidf_vector.len() {
                    let (q_tid, q_val) = normalized_q_vec[p_q];
                    let (d_tid, d_val) = meta.tfidf_vector[p_d];
                    if q_tid == d_tid {
                        dot += q_val * d_val;
                        p_q += 1;
                        p_d += 1;
                    } else if q_tid < d_tid {
                        p_q += 1;
                    } else {
                        p_d += 1;
                    }
                }
                score += dot * 40.0;
            }

            if score >= 10.0 {
                ranked.push((doc_id, score, matched_triggers));
            }
        }

        // Sort descending by initial score
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| self.skills[a.0].name.cmp(&self.skills[b.0].name))
        });

        // 4. Greedy MMR & Domain Diversity Selection Loop
        let mut primary: Vec<DispatchedSkill> = Vec::new();
        let mut manifest: Vec<DispatchedSkill> = Vec::new();
        let mut domain_counts: HashMap<String, usize> = HashMap::new();
        let mut selected_tokens: HashSet<String> = HashSet::new();
        let mut total_tokens = 0usize;

        for (doc_id, base_score, matched_triggers) in &ranked {
            let meta = &self.skills[*doc_id];
            let domain = &meta.domain;
            let count = *domain_counts.get(domain).unwrap_or(&0);

            let is_domain_allowed = count < budget.max_per_domain;

            // MMR token overlap penalty against already selected skills
            let token_overlap_count = meta.name_tokens.iter().filter(|t| selected_tokens.contains(*t)).count();
            let mmr_penalty = if meta.name_tokens.is_empty() {
                0.0
            } else {
                (token_overlap_count as f32 / meta.name_tokens.len() as f32) * 0.4
            };
            let adjusted_score = base_score * (1.0 - mmr_penalty);

            if is_domain_allowed {
                if let Some(skill) = self.load_skill_by_id(*doc_id) {
                    let est = match budget.mode {
                        InjectionMode::DenseInvariants => 150,
                        InjectionMode::CheatSheet => 250,
                        InjectionMode::Comprehensive => estimate_tokens(&skill.instructions) + 50,
                        InjectionMode::Hierarchical => 150,
                    };

                    if primary.is_empty() || total_tokens + est <= budget.max_tokens {
                        *domain_counts.entry(domain.clone()).or_insert(0) += 1;
                        total_tokens += est;
                        selected_tokens.extend(meta.name_tokens.iter().cloned());

                        primary.push(DispatchedSkill {
                            domain: domain.clone(),
                            skill: skill.clone(),
                            score: adjusted_score,
                            matched_triggers: matched_triggers.clone(),
                        });
                    }
                }
            }

            // Populate Tier 1 Manifest (up to 20 distinct relevant skills)
            if manifest.len() < 20 && !manifest.iter().any(|m| m.skill.name == meta.name) {
                if let Some(skill) = self.load_skill_by_id(*doc_id) {
                    manifest.push(DispatchedSkill {
                        domain: domain.clone(),
                        skill,
                        score: *base_score,
                        matched_triggers: matched_triggers.clone(),
                    });
                }
            }
        }

        // 5. Deterministic Canonical Ordering for LLM Prompt Caching
        // Sorts both primary and manifest by domain alphabetically, then by skill name.
        primary.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.skill.name.cmp(&b.skill.name)));
        manifest.sort_by(|a, b| a.domain.cmp(&b.domain).then_with(|| a.skill.name.cmp(&b.skill.name)));

        DiversifiedDispatchResult {
            primary,
            manifest,
            total_estimated_tokens: total_tokens,
            budget,
        }
    }

    /// Master method for maximized provider-aware skill injection
    pub fn equip_prompt_maximized(
        &self,
        base_prompt: &str,
        query: &str,
        provider: &str,
        model: Option<&str>,
        explicit_skill: Option<&str>,
        custom_budget: Option<TokenBudget>,
        domain_bias: Option<&str>,
    ) -> (String, Vec<DispatchedSkill>, TokenBudget) {
        let budget = custom_budget.unwrap_or_else(|| TokenBudget::for_provider_and_model(provider, model));

        if let Some(skill_name) = explicit_skill {
            let clean = skill_name.trim();
            if !clean.is_empty() {
                if let Some(skill) = self.get_skill(clean) {
                    let domain = infer_domain(&skill.name);
                    let ds = DispatchedSkill {
                        domain: domain.clone(),
                        skill,
                        score: 100.0,
                        matched_triggers: vec![clean.to_string()],
                    };
                    let formatted = match budget.mode {
                        InjectionMode::DenseInvariants => Self::format_dense_invariants(&[ds.clone()]),
                        InjectionMode::CheatSheet => Self::format_cheat_sheet(&[ds.clone()]),
                        InjectionMode::Comprehensive => Self::format_cloud_guidelines(&[ds.clone()]),
                        InjectionMode::Hierarchical => {
                            Self::format_hierarchical(&[ds.clone()], &[ds.clone()], InjectionMode::DenseInvariants)
                        }
                    };
                    let equipped = if base_prompt.trim().is_empty() {
                        formatted
                    } else {
                        format!("{}\n\n{}", base_prompt.trim_end(), formatted)
                    };
                    return (equipped, vec![ds], budget);
                }
            }
        }

        let result = self.dispatch_diversified(query, budget, domain_bias);
        if result.primary.is_empty() && result.manifest.is_empty() {
            return (base_prompt.to_string(), Vec::new(), budget);
        }

        let section = match budget.mode {
            InjectionMode::DenseInvariants => Self::format_dense_invariants(&result.primary),
            InjectionMode::Comprehensive => Self::format_cloud_guidelines(&result.primary),
            InjectionMode::CheatSheet => Self::format_cheat_sheet(&result.primary),
            InjectionMode::Hierarchical => {
                Self::format_hierarchical(&result.manifest, &result.primary, InjectionMode::DenseInvariants)
            }
        };

        let equipped = if base_prompt.trim().is_empty() {
            section
        } else {
            format!("{}\n\n{}", base_prompt.trim_end(), section)
        };

        (equipped, result.primary, budget)
    }

    /// Synthesizes provider-aware prompts with domain bias
    pub fn equip_prompt_for_provider_with_bias(
        &self,
        base_prompt: &str,
        query: &str,
        provider: &str,
        explicit_skill: Option<&str>,
        domain_bias: Option<&str>,
    ) -> (String, Vec<DispatchedSkill>) {
        let (equipped, skills, _) = self.equip_prompt_maximized(
            base_prompt,
            query,
            provider,
            None,
            explicit_skill,
            None,
            domain_bias,
        );
        (equipped, skills)
    }

    /// Synthesizes provider-aware prompts injecting either condensed Cheat Sheet (for Local) or Comprehensive Guidelines (for Cloud)
    pub fn equip_prompt_for_provider(
        &self,
        base_prompt: &str,
        query: &str,
        provider: &str,
        explicit_skill: Option<&str>,
    ) -> (String, Vec<DispatchedSkill>) {
        let is_local = Self::is_local_provider(provider);
        let limit = if is_local { 2 } else { 4 };
        let mode = if is_local {
            InjectionMode::CheatSheet
        } else {
            InjectionMode::Comprehensive
        };
        let budget = TokenBudget::new(
            if is_local { 8_192 } else { 128_000 },
            if is_local { 500 } else { 8_000 },
            mode,
            if is_local { 1 } else { 2 },
        );
        let (mut prompt, mut skills, _) = self.equip_prompt_maximized(
            base_prompt,
            query,
            provider,
            None,
            explicit_skill,
            Some(budget),
            None,
        );
        if skills.len() > limit {
            skills.truncate(limit);
            let section = match mode {
                InjectionMode::CheatSheet => Self::format_cheat_sheet(&skills),
                _ => Self::format_cloud_guidelines(&skills),
            };
            prompt = if base_prompt.trim().is_empty() {
                section
            } else {
                format!("{}\n\n{}", base_prompt.trim_end(), section)
            };
        }
        (prompt, skills)
    }

    /// Auto-equips skills into a system prompt string
    pub fn equip_prompt(&self, base_prompt: &str, query: &str, limit: usize) -> String {
        let dispatched = self.dispatch(query, limit, None);
        if dispatched.is_empty() {
            return base_prompt.to_string();
        }
        let section = Self::format_cheat_sheet(&dispatched);
        if base_prompt.trim().is_empty() {
            section
        } else {
            format!("{}\n\n{}", base_prompt.trim_end(), section)
        }
    }
}

/// 31. Grokking Algorithms Skill (Aditya Bhargava)
pub fn grokking_algorithms() -> EccSkill {
    EccSkill::new(
        "grokking-algorithms",
        "Grokking Algorithms (Aditya Bhargava): Big-O complexity radar, spatial vs temporal trade-offs, graph search, greedy heuristics, dynamic programming memoization, hash table collision avoidance, eliminating O(N^2) loops. Triggers: grokking algorithms, big-o, complexity radar, spatial vs temporal, dynamic programming, memoization, hash table, nested loops, bfs, dfs, algorithmic optimization.",
        r#"# Grokking Algorithms & Complexity Control (Aditya Bhargava)

## 1. Big-O Complexity Radar
- Eliminate O(N^2) Anti-patterns: Never use nested loops over unindexed arrays if a hash map/set can reduce lookup to O(1).
- Spatial vs Temporal Trade-offs: Trade memory (caching, hash tables) for speed (CPU cycles) where appropriate.

## 2. Graph Search & Heuristics
- BFS vs DFS: Use BFS for shortest path/minimum edges. Use DFS for exhaustive search and backtracking.
- Greedy Heuristics: Use for optimization problems where a local optimum leads to a global optimum (e.g., fractional knapsack, set cover).

## 3. Dynamic Programming & Hashing
- DP Memoization: Cache subproblem results to avoid exponential recursion trees.
- Hash Table Collision Avoidance: Understand load factors and choose prime modulus sizes or good hash functions (e.g. SipHash) to prevent clustering.
"#,
    )
}

/// 32. OSTEP Mechanical Sympathy Skill (Arpaci-Dusseau & Petzold)
pub fn ostep_mechanical_sympathy() -> EccSkill {
    EccSkill::new(
        "ostep-mechanical-sympathy",
        "Operating Systems & Mechanical Sympathy (Remzi Arpaci-Dusseau & Charles Petzold): Virtualization, Concurrency, Persistence, epoll/kqueue async loops, cache locality. Triggers: ostep, mechanical sympathy, operating systems, cache locality, stack vs heap, concurrency, atomic operations, write-ahead logging, fsync, persistence, epoll, kqueue.",
        r#"# Mechanical Sympathy & Operating Systems (Arpaci-Dusseau & Petzold)

## 1. CPU & Memory Virtualization
- Cache Locality: Prefer contiguous memory structures (e.g. `Vec`, flat arrays) over pointer-chasing data structures (Linked Lists) to maximize L1/L2 cache hits.
- Stack vs Heap: Keep allocations on the stack where possible. Minimize heap fragmentation.

## 2. Concurrency Primitives
- Data Races & Deadlocks: Acquire multiple locks in a strict global order. Prefer message passing over shared memory if possible.
- Atomic Operations: Use lock-free atomics (Compare-and-Swap) for shared counters instead of heavy Mutexes.
- Async Event Loops: Understand epoll/kqueue fundamentals underlying Tokio/Node.js; never block the reactor thread.

## 3. Persistence & Storage
- fsync & Write-Ahead Logging: Use explicit fsync for durability guarantees. Keep append-only logs to minimize SSD page amplification and random writes.
"#,
    )
}

/// 33. System Design Building Blocks Skill (Alex Xu & Sahn Lam)
pub fn system_design_building_blocks() -> EccSkill {
    EccSkill::new(
        "system-design-building-blocks",
        "System Design Interview (Alex Xu): Caching topologies, Rate limiting, Distributed message queues, Sharding, Idempotency keys. Triggers: system design, caching topologies, cache aside, write through, thundering herd, rate limiting, token bucket, sliding window, backpressure, sharding, idempotency keys, message queues.",
        r#"# System Design Building Blocks (Alex Xu & Sahn Lam)

## 1. Caching Topologies
- Cache-Aside vs Write-Through: Choose appropriately. Use Cache-Aside for read-heavy workloads.
- Thundering Herd Prevention: Use probabilistic early expiration or single-flight request coalescing to prevent database stampedes on cache misses.

## 2. Rate Limiting & Flow Control
- Token Bucket & Sliding Window: Implement rate limiters at the API gateway to prevent resource exhaustion and abuse.
- Backpressure: Message queues must exert backpressure on publishers when consumer groups fall behind.

## 3. Distributed Data
- Sharding & Read Replicas: Distribute read pressure via replicas and partition data horizontally via consistent hashing.
- Idempotency Keys: Required for all mutating requests (e.g. POST, PUT) to survive network retries and duplicate deliveries safely.
"#,
    )
}

/// 34. The Pragmatic Programmer Skill (David Thomas & Andrew Hunt)
pub fn pragmatic_programmer_craft() -> EccSkill {
    EccSkill::new(
        "pragmatic-programmer-craft",
        "The Pragmatic Programmer (Thomas & Hunt): Tracer bullets vs prototypes, Orthogonality, Broken windows theory, DRY, Design by Contract, Crash early. Triggers: pragmatic programmer, tracer bullets, prototypes, orthogonality, broken windows, dry, design by contract, crash early, dead programs tell no lies.",
        r#"# Pragmatic Programmer Craft (David Thomas & Andrew Hunt)

## 1. Prototyping & Orthogonality
- Tracer Bullets vs Prototypes: Tracer bullets are structural skeletons intended for production. Prototypes are disposable learning exercises.
- Orthogonality: Changes in one module should not ripple into others. Design independent, decoupled components.

## 2. Code Quality & Broken Windows
- Broken Windows Theory: Fix small issues, warnings, and code smells immediately before they normalize entropy in the codebase.
- DRY across Models: Don't Repeat Yourself applies to data models, documentation, and configuration, not just code.

## 3. Resilience & Defense
- Crash Early / Dead Programs Tell No Lies: Panic/Abort immediately on impossible states or contract violations instead of limping along with corrupted state.
"#,
    )
}

/// 35. Site Reliability Engineering Skill (Betsy Beyer / Google SRE)
pub fn site_reliability_engineering() -> EccSkill {
    EccSkill::new(
        "site-reliability-engineering",
        "Google SRE (Betsy Beyer et al.): SLIs/SLOs/SLAs, Error budgets, Circuit breakers, Exponential backoff with full jitter, Graceful degradation, Distributed tracing. Triggers: site reliability engineering, sre, sli, slo, error budgets, circuit breakers, exponential backoff, jitter, graceful degradation, distributed tracing, structured logging.",
        r#"# Site Reliability Engineering (Google SRE)

## 1. Service Level Indicators & Error Budgets
- SLIs/SLOs: Define strict, measurable service level objectives (e.g., 99.9% of requests < 200ms).
- Error Budgets: Freeze feature deployments when the error budget is exhausted; redirect engineering to reliability.

## 2. Resilience Mechanisms
- Circuit Breakers: Fast-fail downstream calls when a dependency degrades to prevent cascading thread pool exhaustion.
- Exponential Backoff with Jitter: Always add randomization (jitter) to retry loops to avoid thundering herd synchronized retries.
- Graceful Degradation: Serve cached, stale, or partial data when non-critical backends fail.

## 3. Observability
- Distributed Tracing & Correlation IDs: Inject and propagate trace IDs (e.g. W3C Trace Context) across all network boundaries and asynchronous queues.
- Structured Logging: Log payloads as JSON (not flat text) with strict schemas for automated indexing and alerting.
"#,
    )
}

// -------------------------------------------------------------------------
// Helper Parsing & Lexical Functions
// -------------------------------------------------------------------------

/// Fast frontmatter metadata parser extracting (name, description, triggers) without reading entire file
fn parse_frontmatter_metadata(
    content: &str,
    default_name: Option<&str>,
) -> Option<(String, String, Vec<String>)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let rest = &trimmed[3..];
    let end_idx = rest.find("\n---").or_else(|| rest.find("\r\n---"))?;
    let frontmatter_str = &rest[..end_idx];

    let mut name = String::new();
    let mut description = String::new();
    let mut triggers = Vec::new();

    let lines: Vec<&str> = frontmatter_str.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let raw_line = lines[i];
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let val = val.trim().trim_matches('"').trim_matches('\'').trim();

            match key.as_str() {
                "name" => {
                    if name.is_empty() {
                        name = val.to_string();
                    }
                }
                "description" => {
                    if description.is_empty() {
                        if val == "|" || val == "|-" || val == ">" || val == ">-" || val.is_empty() {
                            let mut desc_lines = Vec::new();
                            i += 1;
                            while i < lines.len() {
                                let next_raw = lines[i];
                                if next_raw.starts_with(' ') || next_raw.starts_with('\t') {
                                    desc_lines.push(next_raw.trim());
                                    i += 1;
                                } else if next_raw.trim().is_empty() {
                                    i += 1;
                                } else {
                                    break;
                                }
                            }
                            description = desc_lines.join(" ");
                            continue;
                        } else {
                            description = val.to_string();
                        }
                    }
                }
                "triggers" => {
                    if val.starts_with('[') && val.ends_with(']') {
                        let inner = &val[1..val.len() - 1];
                        for item in inner.split(',') {
                            let cleaned = item.trim().trim_matches('"').trim_matches('\'').trim();
                            if !cleaned.is_empty() {
                                triggers.push(cleaned.to_lowercase());
                            }
                        }
                    } else if val.is_empty() {
                        i += 1;
                        while i < lines.len() {
                            let next_raw = lines[i];
                            let next_trim = next_raw.trim();
                            if next_trim.starts_with('-') {
                                let item = next_trim.trim_start_matches('-').trim().trim_matches('"').trim_matches('\'').trim();
                                if !item.is_empty() {
                                    triggers.push(item.to_lowercase());
                                }
                                i += 1;
                            } else if next_trim.is_empty() {
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        continue;
                    } else {
                        triggers.push(val.to_lowercase());
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    if name.is_empty() {
        if let Some(def) = default_name {
            name = def.to_string();
        } else {
            return None;
        }
    }

    Some((name, description, triggers))
}

/// Extract candidate triggers from skill name, description, and declared triggers
pub fn extract_triggers_from_text(name: &str, description: &str, explicit_triggers: &[String]) -> Vec<String> {
    let mut triggers = Vec::new();
    let lower_name = name.to_lowercase();
    triggers.push(lower_name.clone());

    let spaced_name = lower_name.replace('-', " ");
    if spaced_name != lower_name {
        triggers.push(spaced_name);
    }
    let underscored_name = lower_name.replace('_', " ");
    if underscored_name != lower_name && !triggers.contains(&underscored_name) {
        triggers.push(underscored_name);
    }

    for t in explicit_triggers {
        let clean = t.trim().to_lowercase();
        if !clean.is_empty() && !triggers.contains(&clean) {
            triggers.push(clean.clone());
            let spaced = clean.replace('-', " ");
            if spaced != clean && !triggers.contains(&spaced) {
                triggers.push(spaced);
            }
        }
    }

    let lower_desc = description.to_lowercase();
    if let Some(pos) = lower_desc.find("triggers:") {
        let trigger_part = &description[pos + 9..];
        for token in trigger_part.split(&['"', '\'', ','][..]) {
            let clean = token.trim().trim_matches('.').trim().to_lowercase();
            if clean.len() >= 3 && clean.len() <= 60 && !clean.contains('\n') && !triggers.contains(&clean) {
                triggers.push(clean.clone());
                let spaced = clean.replace('-', " ");
                if spaced != clean && !triggers.contains(&spaced) {
                    triggers.push(spaced);
                }
            }
        }
    }
    if let Some(pos) = lower_desc.find("keywords:") {
        let kw_part = &description[pos + 9..];
        for token in kw_part.split(&[',', ';', '.'][..]) {
            let clean = token.trim().trim_matches('"').trim_matches('\'').trim().to_lowercase();
            if clean.len() >= 3 && clean.len() <= 40 && !clean.contains('\n') && !triggers.contains(&clean) {
                triggers.push(clean.clone());
                let spaced = clean.replace('-', " ");
                if spaced != clean && !triggers.contains(&spaced) {
                    triggers.push(spaced);
                }
            }
        }
    }

    triggers
}

/// Tokenize alphanumeric text into normalized, filtered tokens
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else {
            if current.len() >= 2 && !is_stop_word(&current) {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if current.len() >= 2 && !is_stop_word(&current) {
        tokens.push(current);
    }
    tokens
}

/// Infer high-level engineering domain from skill identifier
fn infer_domain(name: &str) -> String {
    let lower = name.to_lowercase();
    let prefixes = [
        ("ba-", "ba"),
        ("ba", "ba"),
        ("azure", "azure"),
        ("aws", "aws"),
        ("amazon", "aws"),
        ("gcp", "gcp"),
        ("google", "gcp"),
        ("oci", "oracle"),
        ("oracle", "oracle"),
        ("ibm", "ibm"),
        ("openshift", "ibm"),
        ("sap", "sap"),
        ("abap", "sap"),
        ("rap", "sap"),
        ("alibaba", "alibaba"),
        ("aliyun", "alibaba"),
        ("salesforce", "salesforce"),
        ("sf", "salesforce"),
        ("agentforce", "salesforce"),
        ("rust", "rust"),
        ("tokio", "rust"),
        ("axum", "rust"),
        ("ratatui", "rust"),
        ("bun", "bun"),
        ("flutter", "flutter"),
        ("swift", "swift"),
        ("swiftui", "swift"),
        ("ios", "swift"),
        ("react", "react"),
        ("nextjs", "react"),
        ("vue", "vue"),
        ("svelte", "svelte"),
        ("angular", "angular"),
        ("typescript", "typescript"),
        ("ts", "typescript"),
        ("tailwind", "tailwind"),
        ("tdd", "test"),
        ("test", "test"),
        ("testing", "test"),
        ("vitest", "test"),
        ("playwright", "test"),
        ("security", "security"),
        ("sec", "security"),
        ("threat", "security"),
        ("audit", "security"),
        ("sandbox", "security"),
        ("math", "math"),
        ("vibe-math", "math"),
        ("boids", "math"),
        ("quaternion", "math"),
        ("calculus", "math"),
        ("geometry", "math"),
        ("fractal", "math"),
        ("kronos", "quant"),
        ("qlib", "quant"),
        ("chronos", "quant"),
        ("moirai", "quant"),
        ("fingpt", "quant"),
        ("finrl", "quant"),
        ("rdagent", "quant"),
        ("quant", "quant"),
        ("refactoring-ui", "ux"),
        ("microinteractions", "ux"),
        ("laws-of-ux", "ux"),
        ("design-systems", "ux"),
        ("about-face", "ux"),
        ("designing-for-emotion", "ux"),
        ("architect", "architecture"),
        ("architecture", "architecture"),
        ("design", "architecture"),
        ("review", "review"),
        ("clean", "review"),
        ("standards", "review"),
        ("sre", "sre"),
        ("devops", "sre"),
        ("devsecops", "security"),
        ("scrum", "agile"),
        ("erp", "erp"),
        ("odoo", "erp"),
        ("frappe", "erp"),
        ("erpnext", "erp"),
        ("mrp", "erp"),
        ("crp", "erp"),
        ("ddmrp", "erp"),
        ("wms", "erp"),
        ("mes", "erp"),
        ("bom", "erp"),
        ("o2c", "erp"),
        ("p2p", "erp"),
        ("r2r", "erp"),
        ("ledger", "erp"),
        ("accounting", "erp"),
        ("tax", "erp"),
        ("supply-chain", "erp"),
        ("inventory", "erp"),
        ("procurement", "erp"),
        ("sox", "erp"),
        ("sod", "erp"),
        ("kanban", "erp"),
        ("analytics", "analytics"),
        ("warehouse", "warehouse"),
        ("olap", "olap"),
        ("dbt", "dbt"),
        ("statistics", "statistics"),
        ("experimentation", "experimentation"),
        ("streaming", "streaming"),
        ("duckdb", "duckdb"),
        ("polars", "polars"),
        ("visualization", "visualization"),
        ("metrics", "metrics"),
        ("forecasting", "forecasting"),
        ("observability", "observability"),
        ("dataops", "dataops"),
    ];
    for (prefix, dom) in prefixes {
        if lower.starts_with(prefix) {
            return dom.to_string();
        }
    }
    if let Some((first, _)) = lower.split_once('-') {
        if first == "ba" || first.len() >= 3 {
            return first.to_string();
        }
    }
    "general".to_string()
}

/// Standard lexical stop-words to eliminate uninformative terms from TF-IDF indexing
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an" | "the" | "and" | "or" | "but" | "if" | "then" | "else" | "when" | "at" | "by"
            | "for" | "with" | "about" | "against" | "between" | "into" | "through" | "during"
            | "before" | "after" | "above" | "below" | "to" | "from" | "up" | "down" | "in"
            | "out" | "on" | "off" | "over" | "under" | "again" | "further" | "once" | "here"
            | "there" | "all" | "any" | "both" | "each" | "few" | "more" | "most" | "other"
            | "some" | "such" | "no" | "nor" | "not" | "only" | "own" | "same" | "so" | "than"
            | "too" | "very" | "can" | "will" | "just" | "don" | "should" | "now" | "use"
            | "using" | "uses" | "used" | "this" | "that" | "these" | "those" | "is" | "are"
            | "was" | "were" | "be" | "been" | "being" | "have" | "has" | "had" | "having"
            | "do" | "does" | "did" | "doing"
    )
}

/// Standalone check if a provider identifier corresponds to a local LLM runtime (e.g. Ollama, local)
pub fn is_local_provider(provider: &str) -> bool {
    SkillDispatcher::is_local_provider(provider)
}

/// Standalone formatter for condensed Cheat Sheet format for local LLMs
pub fn format_cheat_sheet(skills: &[DispatchedSkill]) -> String {
    SkillDispatcher::format_cheat_sheet(skills)
}

/// Standalone formatter for Comprehensive Architectural Specification format for cloud LLMs
pub fn format_cloud_guidelines(skills: &[DispatchedSkill]) -> String {
    SkillDispatcher::format_cloud_guidelines(skills)
}

/// Standalone formatter for dense invariant DSL
pub fn format_dense_invariants(skills: &[DispatchedSkill]) -> String {
    SkillDispatcher::format_dense_invariants(skills)
}

/// Standalone formatter for multi-tier progressive disclosure
pub fn format_hierarchical(
    manifest_skills: &[DispatchedSkill],
    primary_skills: &[DispatchedSkill],
    primary_mode: InjectionMode,
) -> String {
    SkillDispatcher::format_hierarchical(manifest_skills, primary_skills, primary_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_local_provider() {
        assert!(is_local_provider("ollama"));
        assert!(is_local_provider("local"));
        assert!(is_local_provider("localhost"));
        assert!(is_local_provider("ollama-remote"));
        assert!(is_local_provider("llama.cpp"));

        assert!(!is_local_provider("anthropic"));
        assert!(!is_local_provider("openai"));
        assert!(!is_local_provider("gemini"));
        assert!(!is_local_provider("deepseek"));
        assert!(!is_local_provider("grok"));
        assert!(!is_local_provider("xai"));
    }

    #[test]
    fn test_equip_prompt_for_provider_formats() {
        let dispatcher = global_dispatcher();
        let query = "tokio async concurrency channel deadlock";

        // Local Ollama
        let (local_text, local_skills) = dispatcher.equip_prompt_for_provider(
            "Base prompt",
            query,
            "ollama",
            None,
        );
        assert!(!local_skills.is_empty());
        assert!(local_skills.len() <= 2);
        assert!(local_text.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
        assert!(!local_text.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"));

        // Cloud Anthropic
        let (cloud_text, cloud_skills) = dispatcher.equip_prompt_for_provider(
            "Base prompt",
            query,
            "anthropic",
            None,
        );
        assert!(!cloud_skills.is_empty());
        assert!(cloud_skills.len() <= 4);
        assert!(cloud_text.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"));
        assert!(cloud_text.contains("Full Specification & Directives:"));
        assert!(!cloud_text.contains("[LOCAL LLM CHEAT SHEET"));
    }

    #[test]
    fn test_explicit_skill_alias_override() {
        let dispatcher = global_dispatcher();

        let (text, skills) = dispatcher.equip_prompt_for_provider(
            "Base",
            "irrelevant",
            "ollama",
            Some("rust-tokio-concurrency"),
        );
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill.name, "tokio-async-tuning");
        assert!(text.contains("tokio-async-tuning"));
        assert!(text.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
    }

    #[test]
    fn test_vibe_cs_books_skills_registered_and_dispatchable() {
        let skills = [
            "grokking-algorithms",
            "ostep-mechanical-sympathy",
            "system-design-building-blocks",
            "pragmatic-programmer-craft",
            "site-reliability-engineering",
        ];

        for skill_name in skills {
            let found = find_built_in_skill(skill_name);
            assert!(found.is_some(), "Skill '{}' should be registered in built-in skills", skill_name);
            let skill = found.unwrap();
            assert!(!skill.description.is_empty());
            assert!(!skill.instructions.is_empty());
        }

        let dispatcher = global_dispatcher();
        let dispatched = dispatcher.dispatch("eliminate O(N^2) nested loop with hash map and memoization", 3, None);
        assert!(!dispatched.is_empty());
        assert!(dispatched.iter().any(|d| d.skill.name == "grokking-algorithms"));

        let dispatched_sys = dispatcher.dispatch("caching topologies thundering herd rate limiting token bucket", 3, None);
        assert!(!dispatched_sys.is_empty());
        assert!(dispatched_sys.iter().any(|d| d.skill.name == "system-design-building-blocks"));
    }
}



// =========================================================================
// Business & Functional Analysis Skills Built-in Implementations (Top 50)
// =========================================================================

/// 171. ba-wiegers-requirements-engineering Skill
pub fn ba_wiegers_requirements_engineering() -> EccSkill {
    EccSkill::new(
        "ba-wiegers-requirements-engineering",
        "Three-tier requirements engineering: Business Requirements (Vision & Scope), User Requirements (Tasks & Use Cases), and Functional Requirements (Invariants, RTM, Planguage quality attributes).",
        r#"---
name: ba-wiegers-requirements-engineering
description: "Three-tier requirements engineering: Business Requirements (Vision & Scope), User Requirements (Tasks & Use Cases), and Functional Requirements (Invariants, RTM, Planguage quality attributes)."
triggers: ["wiegers-requirements-engineering", "wiegers", "software-requirements", "requirements-traceability", "planguage", "functional-requirements", "requirements-engineering", "prd-specification"]
---

# ba-wiegers-requirements-engineering
> Based on **Software Requirements (3rd Edition) - Karl Wiegers & Joy Beatty**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Every functional requirement must trace back to exactly one business requirement and have at least one test case (1:N:M traceability).**
2. **Prohibit ambiguous linguistic quantifiers in specifications ('user-friendly', 'fast', 'scalable', 'secure', 'appropriate') without quantifiable metrics.**
3. **Enforce strict RFC 2119 / IEEE 830 modal verbs: SHALL (mandatory), SHOULD (strongly recommended), MAY (optional).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define three-tier requirements (Business, User, Functional). Assign Planguage benchmarks (Scale, Meter, Target) to all non-functional attributes. Enforce full traceability in the RTM.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Specifying implementation details in business requirements.**
- **Orphaned functional requirements with no test verification criteria.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wiegers-requirements-engineering"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 172. ba-volere-requirements-specification Skill
pub fn ba_volere_requirements_specification() -> EccSkill {
    EccSkill::new(
        "ba-volere-requirements-specification",
        "Volere Requirements Process: Volere Snow Card, quantifiable Fit Criteria, customer satisfaction/dissatisfaction gradients, and event-driven elicitation.",
        r#"---
name: ba-volere-requirements-specification
description: "Volere Requirements Process: Volere Snow Card, quantifiable Fit Criteria, customer satisfaction/dissatisfaction gradients, and event-driven elicitation."
triggers: ["volere-requirements-specification", "volere", "fit-criteria", "robertson-requirements", "volere-snow-card", "quantifiable-requirements"]
---

# ba-volere-requirements-specification
> Based on **Mastering the Requirements Process (3rd Edition) - Suzanne Robertson & James Robertson**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **A requirement is undefined until its Fit Criterion is formulated: Fit Criterion = a concrete, unambiguous measurement test that determines whether a solution satisfies the requirement.**
2. **Customer Satisfaction (1 to 5) and Customer Dissatisfaction (1 to 5) gradients must be assigned to distinguish delighters from table-stakes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Every requirement must use the Volere Snow Card: Requirement #, Type, Event, Description, Rationale, Source, Fit Criterion, Customer Satisfaction/Dissatisfaction.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Accepting subjective requirements that cannot be verified by an automated test.**
- **Conflating product desires with statutory constraints.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "volere-requirements-specification"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 173. ba-cockburn-use-case-modeling Skill
pub fn ba_cockburn_use_case_modeling() -> EccSkill {
    EccSkill::new(
        "ba-cockburn-use-case-modeling",
        "Goal-oriented use case modeling: Goal levels (Cloud, Sea-Level, Fish), Main Success Scenario, Extension branches, Minimal and Success Guarantees.",
        r#"---
name: ba-cockburn-use-case-modeling
description: "Goal-oriented use case modeling: Goal levels (Cloud, Sea-Level, Fish), Main Success Scenario, Extension branches, Minimal and Success Guarantees."
triggers: ["cockburn-use-case-modeling", "cockburn", "use-case-modeling", "sea-level-goal", "main-success-scenario", "preconditions-guarantees"]
---

# ba-cockburn-use-case-modeling
> Based on **Writing Effective Use Cases - Alistair Cockburn**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Sea-Level Goal Invariant: User task use cases must represent an atomic business interaction delivering immediate value to the primary actor in a single session.**
2. **Complete Branch Coverage: Every failure or alternate branch must specify an extension step (e.g., 3a, 3b) and define whether it resumes or aborts the Main Success Scenario.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format use cases with Cockburn standard: Primary Actor, Scope, Level (Sea-level), Preconditions, Minimal Guarantee, Success Guarantee, Main Success Scenario (1-N), Extensions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing use cases at the fish/clam level for single button clicks.**
- **Omitting minimal guarantees for failure scenarios.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cockburn-use-case-modeling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 174. ba-nfr-quality-attributes Skill
pub fn ba_nfr_quality_attributes() -> EccSkill {
    EccSkill::new(
        "ba-nfr-quality-attributes",
        "Non-Functional Requirements & Quality Attribute Scenarios: 6-part scenarios (Source, Stimulus, Artifact, Environment, Response, Response Measure), SLA/SLO metrics.",
        r#"---
name: ba-nfr-quality-attributes
description: "Non-Functional Requirements & Quality Attribute Scenarios: 6-part scenarios (Source, Stimulus, Artifact, Environment, Response, Response Measure), SLA/SLO metrics."
triggers: ["nfr-quality-attributes", "quality-attribute-scenarios", "iso-25010", "non-functional-requirements", "bass-clements-kazman", "sla-slo"]
---

# ba-nfr-quality-attributes
> Based on **Software Architecture in Practice (4th Ed) & ISO 25010 - Bass, Clements, Kazman**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **6-Part Quality Attribute Scenario: Every NFR must define Source of Stimulus, Stimulus, Artifact, Environment, Response, and quantifiable Response Measure.**
2. **SLO Quantifiability: Performance, Availability, Security, and Scalability must be formulated with objective numerical thresholds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Specify all NFRs as 6-part scenarios: Source of Stimulus, Stimulus, Artifact, Environment, Response, and quantifiable Response Measure.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing vague NFRs like 'System must be high-performance'.**
- **Ignoring degraded environment modes in NFR specifications.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "nfr-quality-attributes"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 175. ba-requirements-traceability-matrix Skill
pub fn ba_requirements_traceability_matrix() -> EccSkill {
    EccSkill::new(
        "ba-requirements-traceability-matrix",
        "Bidirectional Requirements Traceability Matrix (RTM): Forward and backward traceability, gap analysis, orphan detection, and verification coverage.",
        r#"---
name: ba-requirements-traceability-matrix
description: "Bidirectional Requirements Traceability Matrix (RTM): Forward and backward traceability, gap analysis, orphan detection, and verification coverage."
triggers: ["requirements-traceability-matrix", "rtm", "bidirectional-traceability", "orphan-detection", "ieee-29148", "compliance-matrix"]
---

# ba-requirements-traceability-matrix
> Based on **IEEE Std 830 / ISO/IEC/IEEE 29148 - Requirements Engineering Standards**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Bidirectional Linkage: Business Need <-> System Requirement <-> Architecture Component <-> Source Code <-> Automated Test Case.**
2. **Zero Orphan Rule: Every line of application code and every test must trace back to an authorized requirement.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a bidirectional RTM. Flag any requirement with 0 test cases as an unverified defect. Flag any code feature with 0 requirements as unauthorized scope creep.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Building features that have no upstream business justification.**
- **Writing unit tests that test implementation details instead of requirement criteria.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "requirements-traceability-matrix"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 176. ba-adzic-specification-by-example Skill
pub fn ba_adzic_specification_by_example() -> EccSkill {
    EccSkill::new(
        "ba-adzic-specification-by-example",
        "Executable specifications and living documentation: Deriving scope from goals, illustrating requirements using concrete examples, and single-source-of-truth test suites.",
        r#"---
name: ba-adzic-specification-by-example
description: "Executable specifications and living documentation: Deriving scope from goals, illustrating requirements using concrete examples, and single-source-of-truth test suites."
triggers: ["adzic-specification-by-example", "specification-by-example", "living-documentation", "gojko-adzic", "executable-specifications"]
---

# ba-adzic-specification-by-example
> Based on **Specification by Example - Gojko Adzic**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Illustrate rules using concrete data examples rather than abstract formulas.**
2. **Living Documentation Invariant: The specification and the regression test suite must be the exact same artifact.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Before implementing business logic, construct tabular concrete examples showing exact input vectors and expected output values.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing abstract requirements without concrete input/output test vectors.**
- **Letting documentation drift from test suites.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "adzic-specification-by-example"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 177. ba-cucumber-gherkin-syntax Skill
pub fn ba_cucumber_gherkin_syntax() -> EccSkill {
    EccSkill::new(
        "ba-cucumber-gherkin-syntax",
        "Behavior-Driven Development & Gherkin AST Grammar: Given/When/Then, Scenario Outlines, declarative steps, and business-focused acceptance criteria.",
        r#"---
name: ba-cucumber-gherkin-syntax
description: "Behavior-Driven Development & Gherkin AST Grammar: Given/When/Then, Scenario Outlines, declarative steps, and business-focused acceptance criteria."
triggers: ["cucumber-gherkin-syntax", "gherkin", "bdd", "cucumber", "given-when-then", "scenario-outline", "declarative-testing"]
---

# ba-cucumber-gherkin-syntax
> Based on **The Cucumber Book - Matt Wynne & Aslak Hellesøy**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Strict Gherkin Grammar: Given (context/setup) -> When (action/event) -> Then (observable outcome).**
2. **Declarative over Imperative: Never mention UI widgets in Gherkin; express business intent.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format scenarios in valid Gherkin syntax: Feature, Scenario Outline, Given, When, Then, Examples table. Never mention UI selectors.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mixing setup and assertions inside When steps.**
- **Writing UI-driven imperative steps that break on simple CSS refactoring.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cucumber-gherkin-syntax"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 178. ba-bdd-three-amigos-workshop Skill
pub fn ba_bdd_three_amigos_workshop() -> EccSkill {
    EccSkill::new(
        "ba-bdd-three-amigos-workshop",
        "Collaborative requirement discovery & Example Mapping: Product, Developer, and Tester alignment using Story, Rule, Example, and Question matrices.",
        r#"---
name: ba-bdd-three-amigos-workshop
description: "Collaborative requirement discovery & Example Mapping: Product, Developer, and Tester alignment using Story, Rule, Example, and Question matrices."
triggers: ["bdd-three-amigos-workshop", "three-amigos", "example-mapping", "dan-north", "liz-keogh", "discovery-cards"]
---

# ba-bdd-three-amigos-workshop
> Based on **Example Mapping - Dan North, Liz Keogh, George Dinwiddie**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Three Amigos Alignment: A story cannot start development without explicit consensus across Business (Why/What), Engineering (How), and QA (What could go wrong).**
2. **Example Mapping Heuristic: If a story has > 3 unresolved Red Questions or > 5 Blue Rules, split the story before coding.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Decompose stories into 4 elements: Yellow Story, Blue Rules (business logic), Green Examples (concrete truth vectors), Red Questions (unresolved ambiguities).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Starting implementation while Red Questions remain unresolved.**
- **Writing stories without engineering feasibility or QA boundary input.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bdd-three-amigos-workshop"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 179. ba-atdd-acceptance-criteria Skill
pub fn ba_atdd_acceptance_criteria() -> EccSkill {
    EccSkill::new(
        "ba-atdd-acceptance-criteria",
        "Acceptance Test-Driven Development (ATDD): Test-first specification, boundary value analysis, pass/fail gating, and customer acceptance criteria.",
        r#"---
name: ba-atdd-acceptance-criteria
description: "Acceptance Test-Driven Development (ATDD): Test-first specification, boundary value analysis, pass/fail gating, and customer acceptance criteria."
triggers: ["atdd-acceptance-criteria", "atdd", "acceptance-test-driven-development", "ken-pugh", "acceptance-criteria", "pass-fail-gates"]
---

# ba-atdd-acceptance-criteria
> Based on **ATDD by Example - Ken Pugh & Lisa Crispin**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Test-First Verification Gate: Automated acceptance tests must be written, run, and proven failing *before* production code is written.**
2. **Boundary Value Analysis: Every numeric or range constraint must be tested at minimum, maximum, and outside boundaries (e.g., n-1, n, n+1).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Write automated acceptance tests before writing feature code. Verify tests fail for the right reason, then write minimal code to pass.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing tests after the code is finished (confirmation bias).**
- **Testing only happy-path scenarios and ignoring boundary limits.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "atdd-acceptance-criteria"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 180. ba-living-documentation-tooling Skill
pub fn ba_living_documentation_tooling() -> EccSkill {
    EccSkill::new(
        "ba-living-documentation-tooling",
        "Living documentation architecture: AST analysis, domain-driven annotations, automated diagram extraction, and synchronizing code with domain knowledge.",
        r#"---
name: ba-living-documentation-tooling
description: "Living documentation architecture: AST analysis, domain-driven annotations, automated diagram extraction, and synchronizing code with domain knowledge."
triggers: ["living-documentation-tooling", "living-documentation", "cyrille-martraire", "code-as-documentation", "ast-analysis", "domain-annotations"]
---

# ba-living-documentation-tooling
> Based on **Living Documentation: Continuous Knowledge Sharing - Cyrille Martraire**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Single Source of Truth: Documentation must be derived directly from verified source code and executable tests, never maintained in disconnected wikis.**
2. **Executable Invariants: Domain rules documented in comments or markdown must be backed by unit tests or compiler type assertions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Annotate domain entities with living doc annotations. Extract architecture diagrams and markdown glossaries directly from codebase AST.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Maintaining stale Word documents or wiki pages that drift from codebase reality.**
- **Writing code comments that contradict executable behavior.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "living-documentation-tooling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 181. ba-evans-ubiquitous-language Skill
pub fn ba_evans_ubiquitous_language() -> EccSkill {
    EccSkill::new(
        "ba-evans-ubiquitous-language",
        "Ubiquitous Language & Strategic Modeling: Shared domain lexicon, eliminating translation layers, contextual isomorphism, and bounded linguistic contexts.",
        r#"---
name: ba-evans-ubiquitous-language
description: "Ubiquitous Language & Strategic Modeling: Shared domain lexicon, eliminating translation layers, contextual isomorphism, and bounded linguistic contexts."
triggers: ["evans-ubiquitous-language", "ubiquitous-language", "eric-evans", "domain-driven-design", "ddd", "domain-lexicon"]
---

# ba-evans-ubiquitous-language
> Based on **Domain-Driven Design - Eric Evans**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Ubiquitous Language Invariant: If a term is not used by business domain experts in conversation, it must never appear as a class or table name.**
2. **Zero Synonym Drift: Prohibit using different terms for the same domain entity across files within the same bounded context.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a strict GLOSSARY.md for the project. Use domain terms verbatim in code, class names, database tables, and API endpoints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using technical jargon ('Record', 'DTO', 'Entity') in business conversation.**
- **Letting developers rename business concepts to suit technical habits.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "evans-ubiquitous-language"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 182. ba-brandolini-event-storming Skill
pub fn ba_brandolini_event_storming() -> EccSkill {
    EccSkill::new(
        "ba-brandolini-event-storming",
        "EventStorming domain discovery: Domain Events (orange), Commands (blue), Aggregates (yellow), Read Models (green), Policies (pink), and chronological event flows.",
        r#"---
name: ba-brandolini-event-storming
description: "EventStorming domain discovery: Domain Events (orange), Commands (blue), Aggregates (yellow), Read Models (green), Policies (pink), and chronological event flows."
triggers: ["brandolini-event-storming", "event-storming", "alberto-brandolini", "domain-events", "commands-aggregates", "event-driven-analysis"]
---

# ba-brandolini-event-storming
> Based on **Introducing EventStorming - Alberto Brandolini**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Event Past-Tense Invariant: Domain Events must always be written in the past tense (e.g., `OrderPlaced`, `CaseRaffled`, `PaymentDeclined`).**
2. **Command-Event Causality: Every Domain Event is triggered by a Command, an External Event, or a Business Policy.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map processes using EventStorming sequence: [COMMAND] -> [AGGREGATE] -> [EVENT] -> [POLICY] -> [NEXT COMMAND].

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using present-tense or imperative verbs for events (e.g., `PlaceOrder`).**
- **Missing the aggregate consistency boundary where commands are validated.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "brandolini-event-storming"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 183. ba-bounded-context-mapping Skill
pub fn ba_bounded_context_mapping() -> EccSkill {
    EccSkill::new(
        "ba-bounded-context-mapping",
        "Strategic Context Mapping: Bounded Context boundaries, Anti-Corruption Layers (ACL), Open Host Service (OHS), Shared Kernel, Customer-Supplier, and Conformist patterns.",
        r#"---
name: ba-bounded-context-mapping
description: "Strategic Context Mapping: Bounded Context boundaries, Anti-Corruption Layers (ACL), Open Host Service (OHS), Shared Kernel, Customer-Supplier, and Conformist patterns."
triggers: ["bounded-context-mapping", "bounded-context", "context-mapping", "anti-corruption-layer", "acl", "vlad-khononov", "strategic-ddd"]
---

# ba-bounded-context-mapping
> Based on **Learning Domain-Driven Design - Vlad Khononov & Eric Evans**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Bounded Context Autonomy: A bounded context must own its data schema and be deployable independently of other contexts.**
2. **Anti-Corruption Layer (ACL): When integrating with legacy systems or third-party APIs, always insert an ACL to translate foreign data models into pure internal domain types.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Explicitly define Bounded Context boundaries and relationship patterns (ACL, Customer-Supplier, Conformist, Open Host Service) in CONTEXT_MAP.md.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing multiple contexts to share read/write access to the same database tables.**
- **Letting upstream vendor schemas leak directly into internal domain models.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bounded-context-mapping"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 184. ba-domain-storytelling Skill
pub fn ba_domain_storytelling() -> EccSkill {
    EccSkill::new(
        "ba-domain-storytelling",
        "Domain Storytelling methodology: Visual, actor-centric storytelling using pictographic notations, work objects, activities, and sequence numbers.",
        r#"---
name: ba-domain-storytelling
description: "Domain Storytelling methodology: Visual, actor-centric storytelling using pictographic notations, work objects, activities, and sequence numbers."
triggers: ["domain-storytelling", "stefan-hofer", "henning-schwentner", "domain-stories", "work-objects", "actor-activities"]
---

# ba-domain-storytelling
> Based on **Domain Storytelling - Stefan Hofer & Henning Schwentner**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Grammar of Domain Stories: Actor -> Activity -> Work Object -> Destination Actor. Every step must have a strict integer sequence number (1, 2, 3...).**
2. **Concrete People & Real Objects: Use specific real-world examples and work objects, avoiding abstract data structures.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model business conversations as Domain Stories: (1) Actor A sends Work Object to Actor B; (2) Actor B evaluates Work Object; (3) Actor B creates Output.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Abstracting away the actors and turning stories into generic data flow diagrams.**
- **Skipping sequence numbers and creating ambiguous execution paths.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "domain-storytelling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 185. ba-subdomain-core-domain-triage Skill
pub fn ba_subdomain_core_domain_triage() -> EccSkill {
    EccSkill::new(
        "ba-subdomain-core-domain-triage",
        "Strategic Subdomain Triage: Categorizing domains into Core (differentiator), Supporting (custom auxiliary), and Generic (commodity), guiding engineering investment.",
        r#"---
name: ba-subdomain-core-domain-triage
description: "Strategic Subdomain Triage: Categorizing domains into Core (differentiator), Supporting (custom auxiliary), and Generic (commodity), guiding engineering investment."
triggers: ["subdomain-core-domain-triage", "core-domain", "supporting-subdomain", "generic-subdomain", "nick-tune", "strategic-spend", "buy-vs-build"]
---

# ba-subdomain-core-domain-triage
> Based on **Architecture Modernization & Strategic DDD - Eric Evans & Nick Tune**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Strategic Investment Invariant: 80% of custom engineering effort must be directed to the Core Domain. Supporting subdomains should be minimal; Generic subdomains must use off-the-shelf software.**
2. **Core Domain Isolation: The core domain must be strictly isolated from infrastructure frameworks and third-party libraries.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit every feature against Core vs Supporting vs Generic categorization. Prohibit custom vibe-coded implementations for generic subdomains (e.g. auth, payments).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Wasting engineering budget writing custom implementations for generic commodities.**
- **Treating supporting administrative tools with the same priority as the core engine.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "subdomain-core-domain-triage"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 186. ba-patton-user-story-mapping Skill
pub fn ba_patton_user_story_mapping() -> EccSkill {
    EccSkill::new(
        "ba-patton-user-story-mapping",
        "User Story Mapping: Dual-backbone 2D grid, user activities and tasks, horizontal slicing, walking skeletons, and incremental release framing.",
        r#"---
name: ba-patton-user-story-mapping
description: "User Story Mapping: Dual-backbone 2D grid, user activities and tasks, horizontal slicing, walking skeletons, and incremental release framing."
triggers: ["patton-user-story-mapping", "user-story-mapping", "jeff-patton", "story-mapping", "walking-skeleton", "narrative-backbone"]
---

# ba-patton-user-story-mapping
> Based on **User Story Mapping - Jeff Patton**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Story Map Topology: Horizontal axis represents narrative time (User Activities -> User Tasks); Vertical axis represents release priority slices (MVP -> Release 2).**
2. **Walking Skeleton Invariant: The MVP slice must constitute an end-to-end functional path through the entire user journey, even if implemented with bare-bones technology.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure project roadmaps as a 2D Story Map: Activities across the top, tasks below, sliced horizontally into Walking Skeleton, MVP, and Future Releases.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Building 100% of Module 1 before building any of Module 2 or 3.**
- **Writing isolated user stories that have no clear place in the overall narrative journey.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "patton-user-story-mapping"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 187. ba-invest-user-stories Skill
pub fn ba_invest_user_stories() -> EccSkill {
    EccSkill::new(
        "ba-invest-user-stories",
        "Agile User Stories & INVEST Rubric: Independent, Negotiable, Valuable, Estimable, Small, Testable stories, Card-Conversation-Confirmation (3Cs).",
        r#"---
name: ba-invest-user-stories
description: "Agile User Stories & INVEST Rubric: Independent, Negotiable, Valuable, Estimable, Small, Testable stories, Card-Conversation-Confirmation (3Cs)."
triggers: ["invest-user-stories", "invest-rubric", "mike-cohn", "bill-wake", "user-stories", "3cs-card-conversation-confirmation"]
---

# ba-invest-user-stories
> Based on **User Stories Applied - Bill Wake & Mike Cohn**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **INVEST Verification: Every story must be Independent, Negotiable, Valuable, Estimable, Small (fits in one prompt/sprint), and Testable.**
2. **Story Syntax: 'As a [specific persona], I want [capability/action] so that [business value/benefit]'.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Validate all backlog items against INVEST checklist before asking AI to code. Ensure every story has concrete Given/When/Then acceptance criteria.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing technical tasks disguised as user stories.**
- **Writing huge epic stories that exceed the AI agent's single-turn context.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "invest-user-stories"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 188. ba-story-splitting-patterns Skill
pub fn ba_story_splitting_patterns() -> EccSkill {
    EccSkill::new(
        "ba-story-splitting-patterns",
        "User Story Splitting Heuristics: 10 vertical splitting patterns (workflow steps, business rules, happy/unhappy, interface variations, simple/complex).",
        r#"---
name: ba-story-splitting-patterns
description: "User Story Splitting Heuristics: 10 vertical splitting patterns (workflow steps, business rules, happy/unhappy, interface variations, simple/complex)."
triggers: ["story-splitting-patterns", "story-splitting", "richard-lawrence", "peter-green", "vertical-slicing", "story-decomposition"]
---

# ba-story-splitting-patterns
> Based on **Patterns for Splitting User Stories - Richard Lawrence & Peter Green**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Vertical Slicing: When splitting a user story, every split sub-story must cut through all architectural layers (UI, Logic, Storage) and produce usable value.**
2. **Split Heuristic 1 (Operations): Split CRUD into individual user-driven capabilities (Create vs Search vs Update vs Archive).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Apply vertical story splitting patterns: Workflow steps, Business rule variations, Happy vs Exception paths, Simple vs Complex data variations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Horizontal slicing (e.g. 'Build backend DB' as Story 1, 'Build React UI' as Story 2).**
- **Splitting stories so small that individual stories deliver no customer value.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "story-splitting-patterns"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 189. ba-wsjf-backlog-prioritization Skill
pub fn ba_wsjf_backlog_prioritization() -> EccSkill {
    EccSkill::new(
        "ba-wsjf-backlog-prioritization",
        "Weighted Shortest Job First (WSJF) & Cost of Delay (CoD): User-business value, time criticality, risk reduction/opportunity enablement, and economic job sizing.",
        r#"---
name: ba-wsjf-backlog-prioritization
description: "Weighted Shortest Job First (WSJF) & Cost of Delay (CoD): User-business value, time criticality, risk reduction/opportunity enablement, and economic job sizing."
triggers: ["wsjf-backlog-prioritization", "wsjf", "reinertsen", "cost-of-delay", "economic-prioritization", "job-duration"]
---

# ba-wsjf-backlog-prioritization
> Based on **The Principles of Product Development Flow - Donald G. Reinertsen**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **WSJF Formula: WSJF = Cost of Delay / Job Duration = (User Value + Time Criticality + RR/OE) / Size.**
2. **Economic Priority: Always schedule items with the highest WSJF score first to maximize economic throughput.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Calculate WSJF for all backlog items: CoD = (User-Business Value + Time Criticality + Risk Reduction) / Job Duration. Sort backlog descending by WSJF.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Prioritizing items based on executive loudness (HiPPO) rather than Cost of Delay.**
- **Ignoring job size/duration and scheduling large low-value tasks first.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wsjf-backlog-prioritization"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 190. ba-kanban-value-stream-metrics Skill
pub fn ba_kanban_value_stream_metrics() -> EccSkill {
    EccSkill::new(
        "ba-kanban-value-stream-metrics",
        "Kanban Value Stream Metrics & Flow Control: Little's Law, Work in Progress (WIP) limits, Cycle Time, Lead Time, and Cumulative Flow Diagrams (CFD).",
        r#"---
name: ba-kanban-value-stream-metrics
description: "Kanban Value Stream Metrics & Flow Control: Little's Law, Work in Progress (WIP) limits, Cycle Time, Lead Time, and Cumulative Flow Diagrams (CFD)."
triggers: ["kanban-value-stream-metrics", "kanban", "littles-law", "wip-limits", "cycle-time", "lead-time", "cumulative-flow-diagram"]
---

# ba-kanban-value-stream-metrics
> Based on **Kanban: Successful Evolutionary Change - David J. Anderson**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Little's Law Invariant: Average Lead Time = Work In Progress (WIP) / Throughput.**
2. **WIP Restriction: To reduce cycle time and improve delivery predictability, ruthlessly limit active WIP at each stage of the engineering pipeline.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Monitor delivery using Little's Law. Set WIP limits on each state in the development pipeline. Measure and minimize Lead Time and Cycle Time.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Starting 20 tasks in parallel, causing context switching and blowing out lead times.**
- **Ignoring bottlenecks where tasks pile up indefinitely.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kanban-value-stream-metrics"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 191. ba-bpmn-level2-process-modeling Skill
pub fn ba_bpmn_level2_process_modeling() -> EccSkill {
    EccSkill::new(
        "ba-bpmn-level2-process-modeling",
        "Descriptive & Analytic BPMN 2.0: Pools, lanes, event markers, exclusive (XOR), parallel (AND), inclusive (OR) gateways, and token flow semantics.",
        r#"---
name: ba-bpmn-level2-process-modeling
description: "Descriptive & Analytic BPMN 2.0: Pools, lanes, event markers, exclusive (XOR), parallel (AND), inclusive (OR) gateways, and token flow semantics."
triggers: ["bpmn-level2-process-modeling", "bpmn", "bruce-silver", "bpmn-method-and-style", "workflow-modeling", "token-flow", "gateways"]
---

# ba-bpmn-level2-process-modeling
> Based on **BPMN Method and Style (2nd Edition) - Bruce Silver**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Gateway Soundness: An Exclusive Gateway (XOR) must split flow into exactly one branch; a Parallel Gateway (AND) split must be paired with an AND join to avoid deadlocks.**
2. **Token Conservation: Every token generated at a Start Event must eventually be consumed at an End Event without leaking or being indefinitely trapped.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model business workflows using BPMN 2.0 rules: Clear Pools/Lanes, Start/End events, matched split/join gateways, and message flows across pool boundaries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using an AND join for paths originating from an XOR split, causing deadlocks.**
- **Drawing sequence flows across pool boundaries (violating BPMN standard).**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bpmn-level2-process-modeling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 192. ba-bpmn-gateway-soundness Skill
pub fn ba_bpmn_gateway_soundness() -> EccSkill {
    EccSkill::new(
        "ba-bpmn-gateway-soundness",
        "Workflow Soundness & Deadlock Freedom: Van der Aalst soundness criteria, liveness, bounded Petri net validation, and gateway matching.",
        r#"---
name: ba-bpmn-gateway-soundness
description: "Workflow Soundness & Deadlock Freedom: Van der Aalst soundness criteria, liveness, bounded Petri net validation, and gateway matching."
triggers: ["bpmn-gateway-soundness", "workflow-soundness", "marlon-dumas", "deadlock-freedom", "petri-net", "gateway-matching"]
---

# ba-bpmn-gateway-soundness
> Based on **Fundamentals of Business Process Management (2nd Edition) - Dumas, La Rosa, Mendling, Reijers**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Option to Complete: For every reachable state, it is always possible to reach the terminal end state.**
2. **Proper Completion: When the terminal end state is reached, no tokens remain active in any other branch of the process.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit all process flowcharts for Van der Aalst Soundness: (1) Liveness, (2) Proper Completion, (3) Option to complete. Eliminate potential deadlocks or token leaks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Creating cyclic loops without terminating exit guards.**
- **Mismatched gateways that cause token accumulation or thread starvation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bpmn-gateway-soundness"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 193. ba-bpmn-timer-boundary-events Skill
pub fn ba_bpmn_timer_boundary_events() -> EccSkill {
    EccSkill::new(
        "ba-bpmn-timer-boundary-events",
        "Boundary Events & Exception Flow: Interrupting vs non-interrupting boundary events, timer escalations, error boundaries, compensation, and cancel events.",
        r#"---
name: ba-bpmn-timer-boundary-events
description: "Boundary Events & Exception Flow: Interrupting vs non-interrupting boundary events, timer escalations, error boundaries, compensation, and cancel events."
triggers: ["bpmn-timer-boundary-events", "boundary-events", "timer-event", "error-event", "compensation-event", "bpmn-exceptions"]
---

# ba-bpmn-timer-boundary-events
> Based on **OMG BPMN 2.0 Executable Specification - Object Management Group**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Boundary Event Semantics: An Interrupting Boundary Event cancels the attached activity and diverts token flow; a Non-Interrupting Event spawns a parallel execution branch.**
2. **Compensation Invariant: Compensation events can only be triggered after the associated activity has completed successfully.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Specify SLA timeouts using boundary timer events. Route technical errors and business exceptions via explicit boundary error events.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Letting tasks hang indefinitely without a timer boundary event.**
- **Using interrupting events when a background notification was intended.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bpmn-timer-boundary-events"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 194. ba-state-machine-lifecycle-modeling Skill
pub fn ba_state_machine_lifecycle_modeling() -> EccSkill {
    EccSkill::new(
        "ba-state-machine-lifecycle-modeling",
        "Hierarchical State Machines (FSM & Statecharts): Orthogonal regions, guarded transitions, entry/exit actions, deterministic state lifecycles, and transition matrices.",
        r#"---
name: ba-state-machine-lifecycle-modeling
description: "Hierarchical State Machines (FSM & Statecharts): Orthogonal regions, guarded transitions, entry/exit actions, deterministic state lifecycles, and transition matrices."
triggers: ["state-machine-lifecycle-modeling", "state-machine", "statecharts", "david-harel", "fsm", "transition-matrix", "finite-state-machine"]
---

# ba-state-machine-lifecycle-modeling
> Based on **Statecharts: A Visual Formalism - David Harel & Martin Fowler**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **State Machine Determinism: For any state S and event E, at most one transition guard evaluates to true (deterministic next state).**
2. **Transition Completeness: The state transition matrix must explicitly define behavior for all (State x Event) combinations.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define entity lifecycles as a formal Finite State Machine (FSM): States, Events, Guards, Actions. Enforce strict transition checks in code.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mutating entity status without validating whether the current state permits the transition.**
- **Omitting error/cancelled states in entity lifecycles.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "state-machine-lifecycle-modeling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 195. ba-process-waste-elimination-vsm Skill
pub fn ba_process_waste_elimination_vsm() -> EccSkill {
    EccSkill::new(
        "ba-process-waste-elimination-vsm",
        "Lean Value Stream Mapping (VSM): Eliminating the 7 Wastes (Mudas: Waiting, Transit, Overprocessing, Inventory, Defects, Motion, Overproduction), Takt Time, PCE ratio.",
        r#"---
name: ba-process-waste-elimination-vsm
description: "Lean Value Stream Mapping (VSM): Eliminating the 7 Wastes (Mudas: Waiting, Transit, Overprocessing, Inventory, Defects, Motion, Overproduction), Takt Time, PCE ratio."
triggers: ["process-waste-elimination-vsm", "vsm", "value-stream-mapping", "womack-jones", "7-wastes", "muda", "process-cycle-efficiency"]
---

# ba-process-waste-elimination-vsm
> Based on **Lean Thinking - James P. Womack & Daniel T. Jones**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Process Cycle Efficiency (PCE): PCE = (Value-Add Time / Total Lead Time) * 100. Target: Increase PCE by eliminating waiting and transit mudas.**
2. **Takt Time Pacing: Takt Time = Available Production Time / Customer Demand Rate.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map process steps as Value-Added (VA), Business-Value-Added (BVA), or Non-Value-Added (NVA/Waste). Formulate redesigns to eliminate 100% of NVA waste.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Automating unnecessary non-value-added steps instead of removing them.**
- **Measuring local sub-task speed while ignoring massive wait queues.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "process-waste-elimination-vsm"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 196. ba-torres-opportunity-solution-tree Skill
pub fn ba_torres_opportunity_solution_tree() -> EccSkill {
    EccSkill::new(
        "ba-torres-opportunity-solution-tree",
        "Opportunity Solution Trees (OST): Desired outcomes, opportunity space exploration, multiple solution candidates, and continuous assumption testing.",
        r#"---
name: ba-torres-opportunity-solution-tree
description: "Opportunity Solution Trees (OST): Desired outcomes, opportunity space exploration, multiple solution candidates, and continuous assumption testing."
triggers: ["torres-opportunity-solution-tree", "opportunity-solution-tree", "ost", "teresa-torres", "continuous-discovery", "assumption-testing"]
---

# ba-torres-opportunity-solution-tree
> Based on **Continuous Discovery Habits - Teresa Torres**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **OST Tree Topology: Desired Business Outcome -> Customer Opportunities (Pain Points/Needs) -> Potential Solutions -> Assumption Tests.**
2. **Never implement a Solution without testing at least 3 distinct Opportunity candidates.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Before writing code, map the Opportunity Solution Tree: Top Outcome -> 2-3 Opportunities -> 2-3 Solutions per Opportunity -> Assumption test cards.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Jumping straight from a metric to code without mapping the customer opportunity space.**
- **Testing solutions as monolithic wholes instead of testing underlying assumptions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "torres-opportunity-solution-tree"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 197. ba-cagan-four-product-risks Skill
pub fn ba_cagan_four_product_risks() -> EccSkill {
    EccSkill::new(
        "ba-cagan-four-product-risks",
        "The Four Big Product Risks: Value Risk (will they choose it?), Usability Risk (can they use it?), Feasibility Risk (can we build it?), and Business Viability Risk (compliance/finance/legal).",
        r#"---
name: ba-cagan-four-product-risks
description: "The Four Big Product Risks: Value Risk (will they choose it?), Usability Risk (can they use it?), Feasibility Risk (can we build it?), and Business Viability Risk (compliance/finance/legal)."
triggers: ["cagan-four-product-risks", "four-product-risks", "marty-cagan", "inspired", "value-risk", "viability-risk", "feasibility-risk"]
---

# ba-cagan-four-product-risks
> Based on **Inspired: How to Create Tech Products Customers Love - Marty Cagan**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Four Big Risks Assessment: Before writing production code, explicitly validate: Value, Usability, Feasibility, and Business Viability.**
2. **Feasibility vs Viability: AI solves feasibility quickly; the human analyst must rigorously police viability (statutory laws, privacy acts, security).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit every major feature against the 4 Risks: Value, Usability, Feasibility, and Business Viability (legal, compliance, financial, privacy).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Starting coding when only feasibility is understood while ignoring statutory viability risks.**
- **Allowing AI to guess legal compliance rules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cagan-four-product-risks"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 198. ba-customer-journey-mapping Skill
pub fn ba_customer_journey_mapping() -> EccSkill {
    EccSkill::new(
        "ba-customer-journey-mapping",
        "Customer Journey Mapping: User personas, journey phases, touchpoints, emotional curves, pain point heatmaps, and moment-of-truth interventions.",
        r#"---
name: ba-customer-journey-mapping
description: "Customer Journey Mapping: User personas, journey phases, touchpoints, emotional curves, pain point heatmaps, and moment-of-truth interventions."
triggers: ["customer-journey-mapping", "journey-mapping", "jim-kalbach", "touchpoints", "customer-experience", "emotional-arc"]
---

# ba-customer-journey-mapping
> Based on **Mapping Experiences (2nd Edition) - Jim Kalbach**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Touchpoint Completeness: Map all chronological touchpoints across Pre-service, In-service, and Post-service phases.**
2. **Friction Score Quantifiability: Rate customer friction and emotional sentiment at each touchpoint to highlight critical drop-off cliffs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Document the end-to-end customer journey: Persona, Phases, User Actions, Touchpoints, Emotional Sentiment (-5 to +5), Pain Points, Opportunities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mapping internal organization department handoffs instead of the user's actual external experience.**
- **Ignoring the post-service follow-up phase.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "customer-journey-mapping"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 199. ba-jobs-to-be-done-jtbd Skill
pub fn ba_jobs_to_be_done_jtbd() -> EccSkill {
    EccSkill::new(
        "ba-jobs-to-be-done-jtbd",
        "Jobs to Be Done (JTBD) Theory: Customer progress models, Job Statements, Outcome-Driven Innovation, and the Four Forces of Progress (Push, Pull, Habit, Anxiety).",
        r#"---
name: ba-jobs-to-be-done-jtbd
description: "Jobs to Be Done (JTBD) Theory: Customer progress models, Job Statements, Outcome-Driven Innovation, and the Four Forces of Progress (Push, Pull, Habit, Anxiety)."
triggers: ["jobs-to-be-done-jtbd", "jtbd", "clayton-christensen", "anthony-ulwick", "job-story", "forces-of-progress", "outcome-driven-innovation"]
---

# ba-jobs-to-be-done-jtbd
> Based on **Competing Against Chance - Clayton M. Christensen & Anthony Ulwick**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Job Statement Grammar: 'When [struggling situation/context], I want to [motivation/progress], so I can [desired outcome/transformation].'**
2. **Forces of Progress Balance: A new solution is adopted only when (Push of current situation + Pull of new solution) > (Habit of current solution + Anxiety of new solution).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format feature specifications as Job Stories: When [context], I want to [action], so I can [expected outcome]. Address Push, Pull, Habit, and Anxiety.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Defining products around user demographics rather than the functional/emotional Job to be Done.**
- **Ignoring the friction of user Habits when introducing new software.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "jobs-to-be-done-jtbd"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 200. ba-assumption-mapping-experimentation Skill
pub fn ba_assumption_mapping_experimentation() -> EccSkill {
    EccSkill::new(
        "ba-assumption-mapping-experimentation",
        "Assumption Mapping & Experiment Design: The 2x2 Importance vs Evidence grid, riskiest assumption tests (RAT), prototype fidelity ladders, and experiment loops.",
        r#"---
name: ba-assumption-mapping-experimentation
description: "Assumption Mapping & Experiment Design: The 2x2 Importance vs Evidence grid, riskiest assumption tests (RAT), prototype fidelity ladders, and experiment loops."
triggers: ["assumption-mapping-experimentation", "assumption-mapping", "david-bland", "alex-osterwalder", "testing-business-ideas", "rat-riskiest-assumption"]
---

# ba-assumption-mapping-experimentation
> Based on **Testing Business Ideas - David J. Bland & Alex Osterwalder**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Riskiest Assumption First (RAT): Only invest engineering resources in testing assumptions categorized in the top-right quadrant (High Importance, Low Evidence).**
2. **Falsifiable Hypotheses: Experiments must define a pass/fail threshold before data collection starts.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Plot assumptions on a 2x2 grid (Importance vs Evidence). Design quick spikes or prototypes for High Importance / Low Evidence assumptions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Testing low-risk assumptions because they are easy to measure.**
- **Moving forward with development after an experiment fails its pre-set benchmark.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "assumption-mapping-experimentation"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 201. ba-hoberman-data-modeling-resource Skill
pub fn ba_hoberman_data_modeling_resource() -> EccSkill {
    EccSkill::new(
        "ba-hoberman-data-modeling-resource",
        "Conceptual, Logical & Physical Data Modeling: Entity definitions, cardinalities, relational boundaries, data dictionaries, and ERD verification.",
        r#"---
name: ba-hoberman-data-modeling-resource
description: "Conceptual, Logical & Physical Data Modeling: Entity definitions, cardinalities, relational boundaries, data dictionaries, and ERD verification."
triggers: ["hoberman-data-modeling-resource", "steve-hoberman", "data-modeling", "conceptual-model", "logical-data-model", "erd-modeling"]
---

# ba-hoberman-data-modeling-resource
> Based on **Data Modeling Made Simple (2nd Edition) - Steve Hoberman**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Three-Level Schema Architecture: Conceptual (Business entities) -> Logical (Attributes, normalized, keys) -> Physical (Data types, indexes, partitions).**
2. **Cardinality Invariant: Every relationship between Entity A and Entity B must specify minimum and maximum cardinality (0..1, 1..1, 0..N, 1..N) on both ends.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model data at Conceptual, Logical, and Physical levels. Document all cardinalities (1:1, 1:N, M:N) and foreign key constraints in ERD diagrams.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Generating physical tables without first modeling conceptual entities and cardinalities.**
- **Using many-to-many relationships without an explicit junction table.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "hoberman-data-modeling-resource"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 202. ba-relational-normalization-3nf Skill
pub fn ba_relational_normalization_3nf() -> EccSkill {
    EccSkill::new(
        "ba-relational-normalization-3nf",
        "Relational Normalization & Normal Forms: 1NF atomicity, 2NF partial key dependency elimination, 3NF transitive dependency elimination, and Boyce-Codd Normal Form (BCNF).",
        r#"---
name: ba-relational-normalization-3nf
description: "Relational Normalization & Normal Forms: 1NF atomicity, 2NF partial key dependency elimination, 3NF transitive dependency elimination, and Boyce-Codd Normal Form (BCNF)."
triggers: ["relational-normalization-3nf", "normalization", "3nf", "bcnf", "codd-date", "functional-dependencies", "database-normalization"]
---

# ba-relational-normalization-3nf
> Based on **An Introduction to Database Systems - E.F. Codd & C.J. Date**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **First Normal Form (1NF): All attributes must be atomic; no repeating groups or serialized arrays in single columns.**
2. **Second Normal Form (2NF): In 1NF and every non-key attribute is fully functionally dependent on the entire primary key.**
3. **Third Normal Form (3NF): In 2NF and no non-key attribute is transitively dependent on the primary key.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Verify all relational tables comply with 3NF/BCNF. Decompose partial and transitive dependencies into normalized child tables.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Storing comma-separated lists or unstructured JSON in relational columns requiring query filtering.**
- **Premature denormalization before establishing baseline 3NF.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "relational-normalization-3nf"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 203. ba-cardinality-erd-relationship-rules Skill
pub fn ba_cardinality_erd_relationship_rules() -> EccSkill {
    EccSkill::new(
        "ba-cardinality-erd-relationship-rules",
        "Entity Relationship Modeling & Crow's Foot Notation: Relationship mandatory vs optional participation, foreign key constraints, and cascading integrity rules.",
        r#"---
name: ba-cardinality-erd-relationship-rules
description: "Entity Relationship Modeling & Crow's Foot Notation: Relationship mandatory vs optional participation, foreign key constraints, and cascading integrity rules."
triggers: ["cardinality-erd-relationship-rules", "crows-foot", "richard-barker", "peter-chen", "cardinality-rules", "referential-integrity"]
---

# ba-cardinality-erd-relationship-rules
> Based on **CASE*Method: Entity Relationship Modelling - Peter Chen & Richard Barker**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Participation Invariant: Explicitly determine whether foreign keys are nullable (optional, 0..1) or NOT NULL (mandatory, 1..1).**
2. **Referential Integrity Cascades: Every foreign key must specify explicit `ON DELETE` behavior (`RESTRICT`, `CASCADE`, `SET NULL`).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Draw ER diagrams with strict Crow's Foot notation. Explicitly annotate nullability, unique constraints, and foreign key cascade behaviors.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Defaulting foreign keys to nullable without business justification.**
- **Omitting foreign key indexes, causing slow table-scan joins.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cardinality-erd-relationship-rules"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 204. ba-temporal-bitemporal-data-patterns Skill
pub fn ba_temporal_bitemporal_data_patterns() -> EccSkill {
    EccSkill::new(
        "ba-temporal-bitemporal-data-patterns",
        "Bitemporal Data Modeling: Valid Time (business reality) vs Transaction Time (system audit record), immutable append-only ledgers, and point-in-time state reconstruction.",
        r#"---
name: ba-temporal-bitemporal-data-patterns
description: "Bitemporal Data Modeling: Valid Time (business reality) vs Transaction Time (system audit record), immutable append-only ledgers, and point-in-time state reconstruction."
triggers: ["temporal-bitemporal-data-patterns", "bitemporal", "snodgrass", "valid-time", "transaction-time", "temporal-database", "audit-immutability"]
---

# ba-temporal-bitemporal-data-patterns
> Based on **Developing Time-Oriented Database Applications in SQL - Richard T. Snodgrass**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Bitemporal Invariant: Distinguish Valid Time [vt_start, vt_end] (when fact was true in real world) from Transaction Time [tt_start, tt_end] (when recorded in database).**
2. **Immutability of History: Never execute destructive SQL `UPDATE` or `DELETE` on financial or statutory tables; append new temporal records.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model temporal records with `valid_from`, `valid_to`, `recorded_at`, and `recorded_by`. Use temporal ranges for point-in-time reconstruction.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overwriting previous historical data, destroying legal audit trails.**
- **Conflating database system insertion timestamp with the real-world business event date.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "temporal-bitemporal-data-patterns"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 205. ba-data-dictionary-master-metadata Skill
pub fn ba_data_dictionary_master_metadata() -> EccSkill {
    EccSkill::new(
        "ba-data-dictionary-master-metadata",
        "Data Dictionary & Enterprise Metadata Standards: Data element definitions, ISO 11179 naming conventions, permitted domain values, nullability, and single sources of truth.",
        r#"---
name: ba-data-dictionary-master-metadata
description: "Data Dictionary & Enterprise Metadata Standards: Data element definitions, ISO 11179 naming conventions, permitted domain values, nullability, and single sources of truth."
triggers: ["data-dictionary-master-metadata", "data-dictionary", "dmbok", "iso-11179", "metadata-standards", "canonical-data-dictionary"]
---

# ba-data-dictionary-master-metadata
> Based on **DAMA-DMBOK2 & ISO/IEC 11179 - Data Management Association**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Data Element Standard: Name, Definition, Data Type, Precision, Valid Values Domain, Mandatory/Optional, Source of Record.**
2. **Canonical Single Definition: Each enterprise business element must have exactly one authoritative data definition shared across all systems.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a project DATA_DICTIONARY.md defining every column, data type, allowed enum values, nullability, and business rule constraints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Leaving column names cryptic or undocumented (e.g. `c_stat_cd`).**
- **Allowing conflicting data types for the same concept across different tables.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "data-dictionary-master-metadata"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 206. ba-ross-business-rule-manifesto Skill
pub fn ba_ross_business_rule_manifesto() -> EccSkill {
    EccSkill::new(
        "ba-ross-business-rule-manifesto",
        "Declarative Business Rules & RuleSpeak Grammar: Rules as first-class citizens, separating logic from procedural code, structural vs behavioral rules, and atomic invariants.",
        r#"---
name: ba-ross-business-rule-manifesto
description: "Declarative Business Rules & RuleSpeak Grammar: Rules as first-class citizens, separating logic from procedural code, structural vs behavioral rules, and atomic invariants."
triggers: ["ross-business-rule-manifesto", "rulespeak", "business-rule-manifesto", "ronald-ross", "declarative-rules", "policy-rules"]
---

# ba-ross-business-rule-manifesto
> Based on **The Business Rules Manifesto & RuleSpeak - Ronald G. Ross**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Rule Independence: Business rules must be stated declaratively and exist independently of the procedures, screens, or workflows that enforce them.**
2. **RuleSpeak Grammar: 'It is mandatory that [condition]' or 'It is prohibited that [condition]' or 'A [concept] must [constraint]'.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
State all business rules using RuleSpeak declarative syntax. Separate business policy logic from user interface workflows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Burying business rules in UI click handlers or database triggers.**
- **Writing procedural step-by-step rules instead of declarative invariants.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ross-business-rule-manifesto"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 207. ba-dmn-decision-table-completeness Skill
pub fn ba_dmn_decision_table_completeness() -> EccSkill {
    EccSkill::new(
        "ba-dmn-decision-table-completeness",
        "Decision Model and Notation (DMN 1.3/1.4): Decision table Hit Policies (Unique, First, Priority, Any, Collect), completeness checking, and non-overlapping input domains.",
        r#"---
name: ba-dmn-decision-table-completeness
description: "Decision Model and Notation (DMN 1.3/1.4): Decision table Hit Policies (Unique, First, Priority, Any, Collect), completeness checking, and non-overlapping input domains."
triggers: ["dmn-decision-table-completeness", "dmn", "decision-table", "hit-policy", "bruce-silver-dmn", "completeness-checking"]
---

# ba-dmn-decision-table-completeness
> Based on **DMN Method and Style (2nd Edition) - Bruce Silver**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Completeness & Non-Overlap (Hit Policy U): For every possible combination of inputs, exactly one rule evaluates to true.**
2. **Catch-all Fallback: Every decision table must have an explicit default rule to handle edge-case inputs safely.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure complex multi-condition logic into DMN Decision Tables: Hit Policy (U/F/C), Input Clauses, Output Clauses, Rule Rows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overlapping rule conditions in Hit Policy Unique tables.**
- **Leaving numerical boundary gaps where inputs match zero rules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "dmn-decision-table-completeness"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 208. ba-drd-decision-requirements-diagrams Skill
pub fn ba_drd_decision_requirements_diagrams() -> EccSkill {
    EccSkill::new(
        "ba-drd-decision-requirements-diagrams",
        "Decision Requirements Diagrams (DRD): Decision nodes, Input Data nodes, Business Knowledge Models (BKMs), Knowledge Sources, and decision decomposition.",
        r#"---
name: ba-drd-decision-requirements-diagrams
description: "Decision Requirements Diagrams (DRD): Decision nodes, Input Data nodes, Business Knowledge Models (BKMs), Knowledge Sources, and decision decomposition."
triggers: ["drd-decision-requirements-diagrams", "drd", "decision-requirements-diagram", "bkm", "business-knowledge-models", "dmn-decomposition"]
---

# ba-drd-decision-requirements-diagrams
> Based on **Decision Model and Notation (DMN 1.5) - OMG Standard**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **DRD Hierarchy: High-level decisions decompose into sub-decisions, input data, and reusable Business Knowledge Models (BKMs).**
2. **Process-Decision Separation: BPMN process tasks invoke DMN decision services via clean input/output interfaces.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model complex decision systems as a DRD: Link Input Data -> Sub-decisions -> Final Decision, externalizing calculations into BKMs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing monolithic 500-line decision scripts without decomposing into sub-decisions.**
- **Mixing procedural workflow routing with business calculation logic.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "drd-decision-requirements-diagrams"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 209. ba-feel-expression-language Skill
pub fn ba_feel_expression_language() -> EccSkill {
    EccSkill::new(
        "ba-feel-expression-language",
        "FEEL Expression Language: Strongly-typed expressions, numerical intervals ([a..b], (a..b)), disjunctions, temporal dates/durations, and null-safe navigations.",
        r#"---
name: ba-feel-expression-language
description: "FEEL Expression Language: Strongly-typed expressions, numerical intervals ([a..b], (a..b)), disjunctions, temporal dates/durations, and null-safe navigations."
triggers: ["feel-expression-language", "feel", "dmn-feel", "friendly-enough-expression-language", "dmn-expressions", "range-syntax"]
---

# ba-feel-expression-language
> Based on **Friendly Enough Expression Language (FEEL) - OMG DMN Standard**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **FEEL Type Safety: Strongly typed expressions supporting numbers, strings, booleans, dates, times, durations, and lists.**
2. **Interval Semantics: `[a..b]` (inclusive), `(a..b)` (exclusive), `[a..b)` (inclusive-exclusive), `> x`, `<= y`.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use standard FEEL expressions for all rule conditions: Numerical intervals ([100..500]), list memberships, date comparisons.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using language-specific scripting expressions (e.g. JavaScript eval) inside business rules.**
- **Failing to account for NULL or undefined inputs in FEEL logic.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "feel-expression-language"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 210. ba-decision-table-verification-solver Skill
pub fn ba_decision_table_verification_solver() -> EccSkill {
    EccSkill::new(
        "ba-decision-table-verification-solver",
        "Automated Decision Table Verification: SAT/SMT solver principles, detecting rule overlap, finding completeness gaps, and shadowed rule elimination.",
        r#"---
name: ba-decision-table-verification-solver
description: "Automated Decision Table Verification: SAT/SMT solver principles, detecting rule overlap, finding completeness gaps, and shadowed rule elimination."
triggers: ["decision-table-verification-solver", "decision-table-solver", "rule-overlap-detection", "shadowed-rules", "formal-verification", "sat-solver"]
---

# ba-decision-table-verification-solver
> Based on **Formal Logic & Automated Decision Verification - Silver, Taylor, Ross**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Overlap Detection: Prohibit any two rules in a Unique hit policy table from both evaluating to true for the same input vector.**
2. **Shadowed Rule Elimination: Flag and eliminate any rule whose conditions are a strict subset of an earlier rule that takes precedence.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Run automated SAT/solver checks on decision tables: Verify zero rule overlaps, zero completeness gaps, and zero shadowed rules.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deploying decision tables that contain dead rules shadowed by earlier catch-all rows.**
- **Manual eyeball review of tables with > 10 input variables.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "decision-table-verification-solver"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 211. ba-meadows-systems-thinking-leverage Skill
pub fn ba_meadows_systems_thinking_leverage() -> EccSkill {
    EccSkill::new(
        "ba-meadows-systems-thinking-leverage",
        "Systems Thinking & Dynamic Stocks and Flows: Stocks, inflows, outflows, balancing/reinforcing feedback loops, system delays, and the 12 leverage points.",
        r#"---
name: ba-meadows-systems-thinking-leverage
description: "Systems Thinking & Dynamic Stocks and Flows: Stocks, inflows, outflows, balancing/reinforcing feedback loops, system delays, and the 12 leverage points."
triggers: ["meadows-systems-thinking-leverage", "systems-thinking", "donella-meadows", "stocks-and-flows", "feedback-loops", "leverage-points", "system-delays"]
---

# ba-meadows-systems-thinking-leverage
> Based on **Thinking in Systems: A Primer - Donella H. Meadows**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Stock and Flow Conservation: dS/dt = Inflow(t) - Outflow(t). A stock can only change through its inflows and outflows.**
2. **Leverage Hierarchy: Parameter changes have the lowest leverage; shifting system goals, rules, and mindsets has the highest systemic leverage.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Analyze problem domains as dynamic systems: Define Stocks (accumulations), Inflows, Outflows, Delays, and Feedback Loops.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating symptoms by modifying surface parameters while ignoring broken system feedback loops.**
- **Ignoring delays and over-correcting system controls.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "meadows-systems-thinking-leverage"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 212. ba-causal-loop-diagrams-archetypes Skill
pub fn ba_causal_loop_diagrams_archetypes() -> EccSkill {
    EccSkill::new(
        "ba-causal-loop-diagrams-archetypes",
        "Causal Loop Diagrams (CLD) & Systems Archetypes: Reinforcing loops (R), Balancing loops (B), delays, and canonical archetypes (Fixes that Fail, Shifting the Burden, Limits to Growth).",
        r#"---
name: ba-causal-loop-diagrams-archetypes
description: "Causal Loop Diagrams (CLD) & Systems Archetypes: Reinforcing loops (R), Balancing loops (B), delays, and canonical archetypes (Fixes that Fail, Shifting the Burden, Limits to Growth)."
triggers: ["causal-loop-diagrams-archetypes", "causal-loop-diagram", "peter-senge", "systems-archetypes", "fixes-that-fail", "shifting-the-burden"]
---

# ba-causal-loop-diagrams-archetypes
> Based on **The Fifth Discipline: The Art & Practice of the Learning Organization - Peter Senge**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Loop Polarity Multiplication: An even number of negative (-) links creates a Reinforcing loop; an odd number of negative (-) links creates a Balancing loop.**
2. **Archetype Recognition: Identify systemic traps before writing software (e.g. Fixes that Fail: short-term fix worsens underlying problem).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Draw Causal Loop Diagrams (CLD) showing variables, causal links (+/-), delays, and system archetypes (Fixes that Fail, Shifting the Burden).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Implementing quick patches that trigger delayed negative unintended side-effects.**
- **Confusing correlation with causal feedback polarity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "causal-loop-diagrams-archetypes"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 213. ba-galls-law-system-evolution Skill
pub fn ba_galls_law_system_evolution() -> EccSkill {
    EccSkill::new(
        "ba-galls-law-system-evolution",
        "Gall's Law & Complex System Evolution: How complex systems evolve from simple working systems, failure modes of premature complexity, and the functional core.",
        r#"---
name: ba-galls-law-system-evolution
description: "Gall's Law & Complex System Evolution: How complex systems evolve from simple working systems, failure modes of premature complexity, and the functional core."
triggers: ["galls-law-system-evolution", "galls-law", "john-gall", "systemantics", "evolutionary-architecture", "functional-core"]
---

# ba-galls-law-system-evolution
> Based on **The Systems Bible (Systemantics) - John Gall**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Gall's Law: 'A complex system that works is invariably found to have evolved from a simple system that worked. A complex system designed from scratch never works and cannot be made to work.'**
2. **Incremental Complexity Invariant: Never prompt an AI agent to build a multi-tier distributed system in one step; evolve from a verified simple working core.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce Gall's Law: Phase 1: Build minimal working core in-memory; Phase 2: Add persistence; Phase 3: Add rules/auth; Phase 4: Add distributed scaling.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Scaffolding microservices, queues, and distributed consensus before verifying the core domain logic.**
- **Trying to fix an unworking complex system with more complexity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "galls-law-system-evolution"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 214. ba-wardley-mapping-strategic-landscape Skill
pub fn ba_wardley_mapping_strategic_landscape() -> EccSkill {
    EccSkill::new(
        "ba-wardley-mapping-strategic-landscape",
        "Wardley Mapping & Strategic Value Chains: Anchor customer need, Value Chain positioning, Evolution axis (Genesis -> Custom-Built -> Product/Rental -> Commodity/Utility).",
        r#"---
name: ba-wardley-mapping-strategic-landscape
description: "Wardley Mapping & Strategic Value Chains: Anchor customer need, Value Chain positioning, Evolution axis (Genesis -> Custom-Built -> Product/Rental -> Commodity/Utility)."
triggers: ["wardley-mapping-strategic-landscape", "wardley-maps", "simon-wardley", "value-chain", "evolution-axis", "strategic-mapping"]
---

# ba-wardley-mapping-strategic-landscape
> Based on **Wardley Maps: Topographical Intelligence in Business - Simon Wardley**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Evolution Axis Monotonicity: Components inevitably evolve from Genesis -> Custom -> Product -> Commodity over time under competitive pressure.**
2. **Value Chain Positioning: Components higher on the Y-axis are visible to the user; components lower down are invisible infrastructure dependencies.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map system components on a Wardley Map: Y-axis (User Visibility), X-axis (Evolution: Genesis, Custom, Product, Commodity). Outsource commodity layers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Custom-building components that already exist as commodities.**
- **Treating custom-built proprietary software as if it were a stable commodity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wardley-mapping-strategic-landscape"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 215. ba-cynefin-framework-decision-making Skill
pub fn ba_cynefin_framework_decision_making() -> EccSkill {
    EccSkill::new(
        "ba-cynefin-framework-decision-making",
        "The Cynefin Sense-Making Framework: Classifying contexts into Clear (Best practice), Complicated (Good practice), Complex (Emergent practice), and Chaotic (Novel practice).",
        r#"---
name: ba-cynefin-framework-decision-making
description: "The Cynefin Sense-Making Framework: Classifying contexts into Clear (Best practice), Complicated (Good practice), Complex (Emergent practice), and Chaotic (Novel practice)."
triggers: ["cynefin-framework-decision-making", "cynefin", "dave-snowden", "sense-making", "complex-adaptive-systems", "decision-framework"]
---

# ba-cynefin-framework-decision-making
> Based on **The Cynefin Framework - Dave Snowden**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Domain Categorization: Clear (Sense-Categorize-Respond), Complicated (Sense-Analyze-Respond), Complex (Probe-Sense-Respond), Chaotic (Act-Sense-Respond).**
2. **Probe-Sense-Respond in Complex Domains: In complex user spaces, conduct small safe-to-fail experiments rather than over-analyzing.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Classify project requirements into Cynefin domains. Use standard templates for Clear; analysis for Complicated; safe-to-fail probes for Complex.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Applying rigid 'best practices' to complex emergent problems.**
- **Over-analyzing chaotic situations instead of acting to stabilize flow.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cynefin-framework-decision-making"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 216. ba-cooper-goal-directed-design Skill
pub fn ba_cooper_goal_directed_design() -> EccSkill {
    EccSkill::new(
        "ba-cooper-goal-directed-design",
        "Goal-Directed Interaction Design: User mental models vs implementation models, personas, software posture (sovereign, transient, daemonic), flow, and state preservation.",
        r#"---
name: ba-cooper-goal-directed-design
description: "Goal-Directed Interaction Design: User mental models vs implementation models, personas, software posture (sovereign, transient, daemonic), flow, and state preservation."
triggers: ["cooper-goal-directed-design", "about-face", "alan-cooper", "goal-directed-design", "mental-models", "software-posture", "interaction-design"]
---

# ba-cooper-goal-directed-design
> Based on **About Face: The Essentials of Interaction Design (4th Edition) - Alan Cooper et al.**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Mental Model Invariant: The user interface must reflect the user's mental model of their work, NEVER the underlying database implementation model.**
2. **Posture Alignment: Sovereign applications must maximize screen density, keyboard shortcuts, and flow; Transient applications must prioritize instant comprehension.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design UI flows aligned with user mental models. Never expose database IDs or stack traces. Tailor screen layout to application posture.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Exposing internal database schemas directly as form fields.**
- **Interrupting user flow with unnecessary modal dialogs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cooper-goal-directed-design"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 217. ba-norman-affordance-signifiers Skill
pub fn ba_norman_affordance_signifiers() -> EccSkill {
    EccSkill::new(
        "ba-norman-affordance-signifiers",
        "Design Psychology & Norman Principles: Affordances, signifiers, constraints (physical, cultural, semantic, logical), mappings, feedback, and bridging Gulf of Execution/Evaluation.",
        r#"---
name: ba-norman-affordance-signifiers
description: "Design Psychology & Norman Principles: Affordances, signifiers, constraints (physical, cultural, semantic, logical), mappings, feedback, and bridging Gulf of Execution/Evaluation."
triggers: ["norman-affordance-signifiers", "don-norman", "design-of-everyday-things", "affordances", "signifiers", "gulf-of-execution", "immediate-feedback"]
---

# ba-norman-affordance-signifiers
> Based on **The Design of Everyday Things - Don Norman**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Signifier Invariant: Interactive elements must have clear visual signifiers indicating where and how to interact.**
2. **100ms Feedback Rule: The system must provide perceptible feedback for every user action within 100 milliseconds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Provide visual signifiers for all clickable elements. Deliver state feedback within 100ms. Enforce natural mappings between controls and effects.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Designing clickable buttons that look like static text.**
- **Performing async operations without giving the user loading indicators.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "norman-affordance-signifiers"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 218. ba-johnson-gui-bloopers-heuristics Skill
pub fn ba_johnson_gui_bloopers_heuristics() -> EccSkill {
    EccSkill::new(
        "ba-johnson-gui-bloopers-heuristics",
        "Cognitive Usability & GUI Blooper Prevention: Reducing cognitive load, Hick's Law, Fitts's Law, Nielsen's 10 heuristics, and error prevention.",
        r#"---
name: ba-johnson-gui-bloopers-heuristics
description: "Cognitive Usability & GUI Blooper Prevention: Reducing cognitive load, Hick's Law, Fitts's Law, Nielsen's 10 heuristics, and error prevention."
triggers: ["johnson-gui-bloopers-heuristics", "gui-bloopers", "jeff-johnson", "usability-heuristics", "nielsen-heuristics", "cognitive-load", "hicks-law"]
---

# ba-johnson-gui-bloopers-heuristics
> Based on **GUI Bloopers 2.0 & Usability Heuristics - Jeff Johnson & Jakob Nielsen**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Hick's Law: Decision time increases logarithmically with the number and complexity of choices. Group and minimize choices.**
2. **Error Prevention: Design interfaces to make errors impossible (e.g. disabling invalid options, date pickers) rather than relying on error dialogs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit UI against Nielsen heuristics and Johnson bloopers: Eliminate cognitive clutter, minimize choice counts, and prevent errors through constraints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Presenting 30 unsorted options in a dropdown.**
- **Blaming the user with accusatory error messages when validation fails.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "johnson-gui-bloopers-heuristics"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 219. ba-crud-form-functional-specifications Skill
pub fn ba_crud_form_functional_specifications() -> EccSkill {
    EccSkill::new(
        "ba-crud-form-functional-specifications",
        "Functional Form Specifications & State Machines: Form FSM (Pristine, Dirty, Validating, Submitting, Submitted, Error), optimistic UI, inline validation, dirty tracking.",
        r#"---
name: ba-crud-form-functional-specifications
description: "Functional Form Specifications & State Machines: Form FSM (Pristine, Dirty, Validating, Submitting, Submitted, Error), optimistic UI, inline validation, dirty tracking."
triggers: ["crud-form-functional-specifications", "form-design", "luke-wroblewski", "form-state-machine", "dirty-tracking", "double-submit-protection"]
---

# ba-crud-form-functional-specifications
> Based on **Web Form Design: Filling in the Blanks - Luke Wroblewski**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Double-Submit Protection: While form state is `Submitting`, disable submit buttons and reject duplicate requests.**
2. **Dirty Navigation Guard: If form state is `Dirty` and user navigates away, prompt confirmation to prevent data loss.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement complete Form FSM: Pristine -> Dirty -> Validating -> Submitting -> Submitted / Error. Provide inline validation and dirty navigation guards.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing double-clicks on submit buttons that fire duplicate API requests.**
- **Silently discarding user form inputs when navigating away.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "crud-form-functional-specifications"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}

/// 220. ba-information-architecture-wireflow Skill
pub fn ba_information_architecture_wireflow() -> EccSkill {
    EccSkill::new(
        "ba-information-architecture-wireflow",
        "Information Architecture & Wireflow Modeling: The 5 Planes (Strategy, Scope, Structure, Skeleton, Surface), Wireflows (wireframe + state machine transition diagram).",
        r#"---
name: ba-information-architecture-wireflow
description: "Information Architecture & Wireflow Modeling: The 5 Planes (Strategy, Scope, Structure, Skeleton, Surface), Wireflows (wireframe + state machine transition diagram)."
triggers: ["information-architecture-wireflow", "wireflow", "information-architecture", "jesse-james-garrett", "5-planes", "screen-state-flow"]
---

# ba-information-architecture-wireflow
> Based on **The Elements of User Experience - Jesse James Garrett**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Wireflow Graph Completeness: Every screen wireframe must define entry transitions, exit transitions, empty states, loading skeletons, and error toasts.**
2. **Navigational Hierarchy: Users must always know where they are, where they can go, and how to get back to home in <= 3 clicks.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model UI as a Wireflow Graph: Connect screen wireframes with state transition arrows. Specify Empty, Loading, and Error states for every screen.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Designing static wireframes without documenting screen transition triggers.**
- **Forgetting empty states for screens before data is populated.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "information-architecture-wireflow"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
"#,
    )
}
