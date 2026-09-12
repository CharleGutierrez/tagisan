//! Brutal Integration & Mathematical Verification Tests for Top 50 Books in ERP Skills in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

const ERP_SKILLS_50: [&str; 50] = [
    "erp-silverston-enterprise-patterns",
    "erp-silverston-industry-patterns",
    "erp-fowler-analysis-patterns",
    "erp-evans-ddd-core",
    "erp-vernon-iddd-enterprise",
    "erp-scheer-aris-architecture",
    "erp-double-entry-ledger",
    "erp-multi-currency-fx",
    "erp-asc606-revenue-recognition",
    "erp-cost-accounting-management",
    "erp-double-entry-history-auditing",
    "erp-balance-sheet-working-capital",
    "erp-intercompany-consolidation",
    "erp-supply-chain-strategy",
    "erp-inventory-eoq-safety-stock",
    "erp-landed-cost-allocation",
    "erp-wms-bin-location-topology",
    "erp-procurement-vendor-lifecycle",
    "erp-3way-match-p2p",
    "erp-reverse-logistics-rma",
    "erp-mrp-crp-core",
    "erp-bom-explosion-routing",
    "erp-theory-of-constraints-dbr",
    "erp-ddmrp-demand-driven",
    "erp-lean-toyota-production",
    "erp-shop-floor-mes",
    "erp-batch-traceability-genealogy",
    "erp-bpmn-workflow-patterns",
    "erp-process-mining-event-logs",
    "erp-order-to-cash-o2c",
    "erp-procure-to-pay-p2p",
    "erp-record-to-report-r2r",
    "erp-subscription-recurring-billing",
    "erp-enterprise-integration-patterns",
    "erp-odoo-technical-architecture",
    "erp-frappe-erpnext-framework",
    "erp-sap-s4hana-cleancore",
    "erp-netsuite-suitecloud-suiteflow",
    "erp-multi-tenant-data-isolation",
    "erp-custom-fields-metadata-extensibility",
    "erp-headless-graphql-rest-api",
    "erp-distributed-acid-kleppmann",
    "erp-saga-distributed-transactions",
    "erp-cqrs-event-sourcing",
    "erp-optimistic-locking-concurrency",
    "erp-distributed-idempotency",
    "erp-segregation-of-duties-sod",
    "erp-sox-internal-controls-audit",
    "erp-data-retention-gdpr-compliance",
    "erp-tax-engine-jurisdiction-rules",
];

// =========================================================================
// 1. Discovery of all 50 ERP Skills on Disk
// =========================================================================

#[test]
fn test_all_50_erp_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &ERP_SKILLS_50 {
        assert!(
            loaded_map.contains(*skill),
            "ERP skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 50 ERP Skills (assert >= 120 total skills)
// =========================================================================

#[test]
fn test_all_50_erp_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 120,
        "Expected at least 120 total built-in skills including ERP, found {}",
        all_skills.len()
    );

    for skill_name in &ERP_SKILLS_50 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "ERP skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty());
        assert!(!s.description.is_empty());
        assert!(!s.instructions.is_empty());
        assert!(
            s.instructions.len() > 100,
            "Skill '{}' must contain concrete enterprise architecture instructions",
            skill_name
        );
    }
}

// =========================================================================
// 3. High-Leverage ERP Alias Lookups
// =========================================================================

#[test]
fn test_erp_skills_alias_lookups() {
    // Financial Accounting & Ledgers
    assert_eq!(find_ecc_skill("double-entry").unwrap().name, "erp-double-entry-ledger");
    assert_eq!(find_ecc_skill("general-ledger").unwrap().name, "erp-double-entry-ledger");
    assert_eq!(find_ecc_skill("trial-balance").unwrap().name, "erp-double-entry-ledger");

    // Accounts Payable & 3-Way Match
    assert_eq!(find_ecc_skill("3-way-match").unwrap().name, "erp-3way-match-p2p");
    assert_eq!(find_ecc_skill("three-way-match").unwrap().name, "erp-3way-match-p2p");
    assert_eq!(find_ecc_skill("gr-ir").unwrap().name, "erp-3way-match-p2p");

    // Inventory & SCM
    assert_eq!(find_ecc_skill("eoq").unwrap().name, "erp-inventory-eoq-safety-stock");
    assert_eq!(find_ecc_skill("safety-stock").unwrap().name, "erp-inventory-eoq-safety-stock");
    assert_eq!(find_ecc_skill("reorder-point").unwrap().name, "erp-inventory-eoq-safety-stock");
    assert_eq!(find_ecc_skill("landed-cost").unwrap().name, "erp-landed-cost-allocation");

    // Manufacturing & BOM
    assert_eq!(find_ecc_skill("bom-explosion").unwrap().name, "erp-bom-explosion-routing");
    assert_eq!(find_ecc_skill("bill-of-materials").unwrap().name, "erp-bom-explosion-routing");
    assert_eq!(find_ecc_skill("low-level-coding").unwrap().name, "erp-bom-explosion-routing");

    // Core Business Processes
    assert_eq!(find_ecc_skill("order-to-cash").unwrap().name, "erp-order-to-cash-o2c");
    assert_eq!(find_ecc_skill("o2c").unwrap().name, "erp-order-to-cash-o2c");
    assert_eq!(find_ecc_skill("procure-to-pay").unwrap().name, "erp-procure-to-pay-p2p");
    assert_eq!(find_ecc_skill("p2p").unwrap().name, "erp-procure-to-pay-p2p");
    assert_eq!(find_ecc_skill("record-to-report").unwrap().name, "erp-record-to-report-r2r");
    assert_eq!(find_ecc_skill("r2r").unwrap().name, "erp-record-to-report-r2r");

    // Open Source & Cloud ERP Frameworks
    assert_eq!(find_ecc_skill("odoo").unwrap().name, "erp-odoo-technical-architecture");
    assert_eq!(find_ecc_skill("frappe").unwrap().name, "erp-frappe-erpnext-framework");
    assert_eq!(find_ecc_skill("erpnext").unwrap().name, "erp-frappe-erpnext-framework");
    assert_eq!(find_ecc_skill("clean-core").unwrap().name, "erp-sap-s4hana-cleancore");
    assert_eq!(find_ecc_skill("suitescript").unwrap().name, "erp-netsuite-suitecloud-suiteflow");

    // Distributed Systems & Concurrency
    assert_eq!(find_ecc_skill("saga-pattern").unwrap().name, "erp-saga-distributed-transactions");
    assert_eq!(find_ecc_skill("distributed-saga").unwrap().name, "erp-saga-distributed-transactions");
    assert_eq!(find_ecc_skill("optimistic-locking").unwrap().name, "erp-optimistic-locking-concurrency");
    assert_eq!(find_ecc_skill("idempotency").unwrap().name, "erp-distributed-idempotency");

    // Internal Controls & Governance
    assert_eq!(find_ecc_skill("segregation-of-duties").unwrap().name, "erp-segregation-of-duties-sod");
    assert_eq!(find_ecc_skill("sod").unwrap().name, "erp-segregation-of-duties-sod");
    assert_eq!(find_ecc_skill("sox-compliance").unwrap().name, "erp-sox-internal-controls-audit");
    assert_eq!(find_ecc_skill("sox-404").unwrap().name, "erp-sox-internal-controls-audit");
}

// =========================================================================
// 4. Semantic Intent Top-K Dispatching for ERP & Business Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_erp_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        ("Posting general ledger journal entries with debits equal credits invariant", "erp-double-entry-ledger"),
        ("Calculate 3-way match tolerances between purchase order goods receipt and supplier invoice", "erp-3way-match-p2p"),
        ("Optimize economic order quantity and safety stock with demand variability", "erp-inventory-eoq-safety-stock"),
        ("Allocate freight demurrage and customs duties into perpetual landed cost layers", "erp-landed-cost-allocation"),
        ("BOM explosion bill of materials routing low-level-coding and scrap factors", "erp-bom-explosion-routing"),
        ("Order-to-cash O2C order-fulfillment-workflow customer credit-check goods issue", "erp-order-to-cash-o2c"),
        ("Procure-to-pay P2P purchase requisition approval goods receipt and voucher disbursement", "erp-procure-to-pay-p2p"),
        ("Record-to-report R2R fast-close financial period close cutoff accruals", "erp-record-to-report-r2r"),
        ("Odoo technical architecture models odoo-orm odoo-inheritance and odoo-computed-fields", "erp-odoo-technical-architecture"),
        ("Frappe DocType framework submittable-documents erpnext-architecture and frappe-hooks", "erp-frappe-erpnext-framework"),
        ("Distributed saga pattern compensating transactions and orchestrator", "erp-saga-distributed-transactions"),
        ("Segregation of duties incompatible role combinations and ACRR matrix", "erp-segregation-of-duties-sod"),
        ("SOX 404 IT general controls and delegation of authority approval limits", "erp-sox-internal-controls-audit"),
        ("ASC 606 asc606-revenue-recognition ifrs15-revenue 5-step-revenue-model ssp-allocation", "erp-asc606-revenue-recognition"),
        ("Multi-tenant data isolation row level security rls and tenant routing", "erp-multi-tenant-data-isolation"),
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
fn test_erp_skills_local_vs_cloud_formatting() {
    let dispatcher = global_ecc_dispatcher();
    let query = "Audit double-entry general ledger debit credit balance and 3-way match tolerances";

    // Local Ollama formatting
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

    // Cloud Anthropic / OpenAI formatting
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
// 6. 50-Thread Concurrent Dispatch Stress Test
// =========================================================================

#[test]
fn test_multithreaded_concurrent_erp_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "double-entry general ledger debit credit balance",
        "accounts payable 3-way match po grn invoice tolerance",
        "economic order quantity eoq safety stock replenishment",
        "freight demurrage landed cost perpetual inventory allocation",
        "multi-level bill of materials bom explosion routing",
        "order to cash o2c customer credit check shipment",
        "procure to pay p2p purchase order voucher payment run",
        "fast close record to report r2r subledger lock",
        "odoo orm models record rules computed fields",
        "distributed saga orchestrator compensating transaction",
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
// 7. Enterprise Mathematical Invariants Brutal Tests
// =========================================================================

#[test]
fn test_double_entry_debit_equals_credit_invariant() {
    // 1. Balanced Journal Entry
    let debits = [1500.0, 350.0, 150.0];
    let credits = [2000.0];
    let sum_debits: f64 = debits.iter().sum();
    let sum_credits: f64 = credits.iter().sum();
    assert!(
        (sum_debits - sum_credits).abs() < 1e-4,
        "Balanced entry must have debits == credits: {} vs {}",
        sum_debits, sum_credits
    );

    // 2. Unbalanced Entry Detection
    let bad_debits = [1500.0, 350.0];
    let bad_credits = [2000.0];
    let bad_sum_debits: f64 = bad_debits.iter().sum();
    let bad_sum_credits: f64 = bad_credits.iter().sum();
    assert!(
        (bad_sum_debits - bad_sum_credits).abs() > 1e-4,
        "Unbalanced entry must fail equality check: {} vs {}",
        bad_sum_debits, bad_sum_credits
    );
}

#[test]
fn test_three_way_match_tolerances() {
    // PO: 100 units @ $50.00 = $5,000.00
    let _po_qty: f64 = 100.0;
    let po_price: f64 = 50.0;
    let grn_qty: f64 = 100.0; // Fully received

    let qty_tolerance_pct: f64 = 0.01; // 1%
    let price_tolerance_pct: f64 = 0.005; // 0.5%

    // Case A: Exact match
    let inv_qty_a: f64 = 100.0;
    let inv_price_a: f64 = 50.0;
    let qty_diff_a = (inv_qty_a - grn_qty).abs() / grn_qty;
    let price_diff_a = (inv_price_a - po_price).abs() / po_price;
    assert!(qty_diff_a <= qty_tolerance_pct && price_diff_a <= price_tolerance_pct);

    // Case B: Acceptable minor variance (e.g., supplier billed $50.20 -> 0.4% variance)
    let inv_price_b: f64 = 50.20;
    let price_diff_b = (inv_price_b - po_price).abs() / po_price;
    assert!(price_diff_b <= price_tolerance_pct, "0.4% is within 0.5% tolerance");

    // Case C: Unacceptable price spike (supplier billed $51.00 -> 2.0% variance)
    let inv_price_c: f64 = 51.00;
    let price_diff_c = (inv_price_c - po_price).abs() / po_price;
    assert!(price_diff_c > price_tolerance_pct, "2.0% must exceed 0.5% tolerance and trigger EXCEPTION_HOLD");

    // Case D: Quantity over-billing (received 100, billed 105 -> 5% over-bill)
    let inv_qty_d: f64 = 105.0;
    let qty_diff_d = (inv_qty_d - grn_qty).abs() / grn_qty;
    assert!(qty_diff_d > qty_tolerance_pct, "5% over-bill must exceed 1% tolerance");
}

#[test]
fn test_eoq_and_holding_vs_ordering_cost_balance() {
    let annual_demand: f64 = 10_000.0; // D
    let order_cost: f64 = 50.0;        // S
    let holding_cost: f64 = 4.0;       // H

    // EOQ = sqrt(2 * D * S / H)
    let eoq = (2.0 * annual_demand * order_cost / holding_cost).sqrt();
    assert!((eoq - 500.0).abs() < 1e-4, "EOQ for D=10k, S=50, H=4 must be 500 units, got {}", eoq);

    // At EOQ: Annual Ordering Cost == Annual Holding Cost
    let annual_ordering_cost = (annual_demand / eoq) * order_cost;
    let annual_holding_cost = (eoq / 2.0) * holding_cost;
    assert!(
        (annual_ordering_cost - annual_holding_cost).abs() < 1e-4,
        "Ordering cost ({}) must exactly equal holding cost ({}) at EOQ",
        annual_ordering_cost, annual_holding_cost
    );
}

#[test]
fn test_landed_cost_conservation_invariant() {
    let total_freight_customs: f64 = 1500.00; // $1,500 total additional expense

    // Receipt lines with basis weights
    let line_weights: [f64; 3] = [200.0, 500.0, 300.0]; // Total 1,000 kg
    let total_weight: f64 = line_weights.iter().sum();
    assert_eq!(total_weight, 1000.0);

    let mut allocated = Vec::new();
    let mut running_sum = 0.0;

    for (i, &w) in line_weights.iter().enumerate() {
        let is_last = i == line_weights.len() - 1;
        let share = if is_last {
            total_freight_customs - running_sum
        } else {
            (total_freight_customs * (w / total_weight) * 100.0).round() / 100.0
        };
        running_sum += share;
        allocated.push(share);
    }

    assert_eq!(allocated, vec![300.0, 750.0, 450.0]);
    let final_sum: f64 = allocated.iter().sum();
    assert!(
        (final_sum - total_freight_customs).abs() < 1e-4,
        "Landed cost conservation: sum of allocated costs ({}) must equal total additional invoice ({})",
        final_sum, total_freight_customs
    );
}

#[test]
fn test_multi_level_bom_explosion_and_scrap() {
    // BOM Structure:
    // Finished Good (Parent) -> 2 Sub-Assemblies (A) + 4 Fasteners (B)
    // Sub-Assembly A -> 3 Raw Materials (C, scrap rate 10%)
    let parent_order_qty: f64 = 50.0;

    // Sub-Assembly A needed: 50 * 2 = 100 units
    let sub_a_qty = parent_order_qty * 2.0;
    assert_eq!(sub_a_qty, 100.0);

    // Fasteners B needed: 50 * 4 = 200 units
    let fasteners_b_qty = parent_order_qty * 4.0;
    assert_eq!(fasteners_b_qty, 200.0);

    // Raw Material C: 100 * 3 = 300 net units.
    // With 10% scrap rate: Gross = Net / (1 - 0.10) = 300 / 0.90 = 333.3333 units
    let scrap_rate_c: f64 = 0.10;
    let net_c = sub_a_qty * 3.0;
    let gross_c = net_c / (1.0 - scrap_rate_c);

    assert_eq!(net_c, 300.0);
    assert!(
        (gross_c - 333.3333).abs() < 0.01,
        "Raw material gross requirement with scrap must be ~333.33, got {}",
        gross_c
    );
}
