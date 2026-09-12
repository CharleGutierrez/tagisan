use crate::error::{Result, TagisanError};
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

/// Discover and load all ECC skills from a directory (scanning both `*.md` and `<dir>/SKILL.md`)
pub fn load_skills_from_dir(dir: impl AsRef<Path>) -> Vec<EccSkill> {
    let mut skills = Vec::new();
    let dir_ref = dir.as_ref();

    if !dir_ref.is_dir() {
        return skills;
    }

    if let Ok(entries) = fs::read_dir(dir_ref) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match EccSkill::from_file(&path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => warn!("Failed to load skill from '{}': {}", path.display(), e),
                }
            } else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                let skill_md_alt = path.join("skill.md");
                let target = if skill_md.is_file() {
                    Some(skill_md)
                } else if skill_md_alt.is_file() {
                    Some(skill_md_alt)
                } else {
                    None
                };

                if let Some(target_file) = target {
                    match EccSkill::from_file(&target_file) {
                        Ok(skill) => skills.push(skill),
                        Err(e) => warn!("Failed to load skill from '{}': {}", target_file.display(), e),
                    }
                }
            }
        }
    }

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
#[derive(Debug, Clone)]
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
    /// Initialize dispatcher from the standard locations (.ecc/skills + 20 built-ins)
    pub fn default_catalog() -> Self {
        let skills_dir = Path::new(".ecc/skills");
        let custom_dir = if skills_dir.exists() {
            Some(skills_dir)
        } else {
            None
        };
        Self::new(custom_dir)
    }

    /// Initialize dispatcher scanning built-ins and an optional custom skills directory
    pub fn new(custom_dir: Option<&Path>) -> Self {
        struct RawEntry {
            name: String,
            description: String,
            triggers: Vec<String>,
            file_path: Option<PathBuf>,
            is_builtin: bool,
        }

        let mut raw_entries: Vec<RawEntry> = Vec::new();
        let mut initial_cache = HashMap::new();

        // 1. Ingest built-in skills
        for s in all_built_in_skills() {
            let trigs = extract_triggers_from_text(&s.name, &s.description, &[]);
            let id = raw_entries.len();
            initial_cache.insert(id, s.clone());
            raw_entries.push(RawEntry {
                name: s.name.clone(),
                description: s.description.clone(),
                triggers: trigs,
                file_path: None,
                is_builtin: true,
            });
        }

        // 2. Fast scan disk directory frontmatters (reading first 3KB per file)
        if let Some(dir) = custom_dir {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let skill_md = path.join("SKILL.md");
                            let skill_md_alt = path.join("skill.md");
                            let target = if skill_md.is_file() {
                                Some(skill_md)
                            } else if skill_md_alt.is_file() {
                                Some(skill_md_alt)
                            } else {
                                None
                            };

                            if let Some(target_file) = target {
                                if let Ok(mut file) = File::open(&target_file) {
                                    let mut buf = [0u8; 8192];
                                    let read_bytes = file.read(&mut buf).unwrap_or(0);
                                    let header = String::from_utf8_lossy(&buf[..read_bytes]);
                                    let folder_name = path.file_name().and_then(|f| f.to_str());
                                    if let Some((name, desc, raw_trigs)) = parse_frontmatter_metadata(&header, folder_name) {
                                        let trigs = extract_triggers_from_text(&name, &desc, &raw_trigs);
                                        raw_entries.push(RawEntry {
                                            name,
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file),
                                            is_builtin: false,
                                        });
                                    } else if let Some(fname) = folder_name {
                                        // Robust fallback for markdown files without standard --- delimiters
                                        let first_line = header.lines().find(|l| !l.trim().is_empty()).unwrap_or(fname);
                                        let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                                        let trigs = extract_triggers_from_text(fname, &desc, &[]);
                                        raw_entries.push(RawEntry {
                                            name: fname.to_string(),
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file),
                                            is_builtin: false,
                                        });
                                    }
                                }
                            }
                        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                            if let Ok(mut file) = File::open(&path) {
                                let mut buf = [0u8; 8192];
                                let read_bytes = file.read(&mut buf).unwrap_or(0);
                                let header = String::from_utf8_lossy(&buf[..read_bytes]);
                                let file_stem = path.file_stem().and_then(|s| s.to_str());
                                if let Some((name, desc, raw_trigs)) = parse_frontmatter_metadata(&header, file_stem) {
                                    let trigs = extract_triggers_from_text(&name, &desc, &raw_trigs);
                                    raw_entries.push(RawEntry {
                                        name,
                                        description: desc,
                                        triggers: trigs,
                                        file_path: Some(path),
                                        is_builtin: false,
                                    });
                                } else if let Some(sname) = file_stem {
                                    let first_line = header.lines().find(|l| !l.trim().is_empty()).unwrap_or(sname);
                                    let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                                    let trigs = extract_triggers_from_text(sname, &desc, &[]);
                                    raw_entries.push(RawEntry {
                                        name: sname.to_string(),
                                        description: desc,
                                        triggers: trigs,
                                        file_path: Some(path),
                                        is_builtin: false,
                                    });
                                }
                            }
                        }
                    }
                }
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
            skill_cache: RwLock::new(initial_cache),
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
                if tr.len() >= 3 && lower_query.contains(tr.as_str()) {
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

    /// Synthesizes provider-aware prompts injecting either condensed Cheat Sheet (for Local) or Comprehensive Guidelines (for Cloud)
    pub fn equip_prompt_for_provider(
        &self,
        base_prompt: &str,
        query: &str,
        provider: &str,
        explicit_skill: Option<&str>,
    ) -> (String, Vec<DispatchedSkill>) {
        let is_local = Self::is_local_provider(provider);
        let max_skills = if is_local { 2 } else { 4 };

        let skills: Vec<DispatchedSkill> = if let Some(skill_name) = explicit_skill {
            let clean = skill_name.trim();
            if !clean.is_empty() {
                if let Some(skill) = self.get_skill(clean) {
                    vec![DispatchedSkill {
                        domain: infer_domain(&skill.name),
                        skill,
                        score: 100.0,
                        matched_triggers: vec![clean.to_string()],
                    }]
                } else {
                    self.dispatch(clean, max_skills, None)
                }
            } else {
                self.dispatch(query, max_skills, None)
            }
        } else {
            self.dispatch(query, max_skills, None)
        };

        if skills.is_empty() {
            return (base_prompt.to_string(), Vec::new());
        }

        let section = if is_local {
            Self::format_cheat_sheet(&skills)
        } else {
            Self::format_cloud_guidelines(&skills)
        };

        let equipped_prompt = if base_prompt.trim().is_empty() {
            section
        } else {
            format!("{}\n\n{}", base_prompt.trim_end(), section)
        };

        (equipped_prompt, skills)
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
            triggers.push(clean);
        }
    }

    let lower_desc = description.to_lowercase();
    if let Some(pos) = lower_desc.find("triggers:") {
        let trigger_part = &description[pos + 9..];
        for token in trigger_part.split(&['"', '\'', ','][..]) {
            let clean = token.trim().trim_matches('.').trim().to_lowercase();
            if clean.len() >= 3 && clean.len() <= 60 && !clean.contains('\n') && !triggers.contains(&clean) {
                triggers.push(clean);
            }
        }
    }
    if let Some(pos) = lower_desc.find("keywords:") {
        let kw_part = &description[pos + 9..];
        for token in kw_part.split(&[',', ';', '.'][..]) {
            let clean = token.trim().trim_matches('"').trim_matches('\'').trim().to_lowercase();
            if clean.len() >= 3 && clean.len() <= 40 && !clean.contains('\n') && !triggers.contains(&clean) {
                triggers.push(clean);
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
    ];
    for (prefix, dom) in prefixes {
        if lower.starts_with(prefix) {
            return dom.to_string();
        }
    }
    if let Some((first, _)) = lower.split_once('-') {
        if first.len() >= 3 {
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


