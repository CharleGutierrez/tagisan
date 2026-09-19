use serde_json::json;
use tagisan::engine::oracle::*;
use tagisan::tools::ToolHandler;

#[tokio::test]
async fn test_oracle_suite_full_lifecycle() {
    let suite = OracleSuite::new();

    // 1. Oracle 23ai Hybrid SQL & Vector Search
    let query = OracleVectorSearchQuery {
        table_name: "procurement_contracts".to_string(),
        vector_column: "contract_embedding".to_string(),
        query_vector: vec![0.1, 0.2, 0.3, 0.4],
        relational_filter: Some("status = 'APPROVED' AND total_val > 50000".to_string()),
        metric: OracleVectorMetric::Cosine,
        top_k: 3,
        selected_columns: vec!["contract_id".to_string(), "vendor".to_string()],
    };
    let sql = suite.vector_engine.generate_hybrid_sql(&query);
    assert!(sql.contains("SELECT contract_id, vendor, VECTOR_DISTANCE"));
    assert!(sql.contains("WHERE status = 'APPROVED' AND total_val > 50000"));
    assert!(sql.contains("ORDER BY vector_distance ASC"));
    assert!(sql.contains("FETCH FIRST 3 ROWS ONLY"));

    let results = suite.vector_engine.execute_search(&query);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].row_id, "ORA-ROW-0000");

    // 2. Oracle JSON Relational Duality View Concurrency
    let doc = OracleDualityDocument::new("vendor_duality", "V-100", json!({ "name": "Global Tech", "tier": 1 }));
    assert_eq!(doc.view_name, "vendor_duality");
    assert_eq!(doc.document_id, "V-100");
    assert!(doc.etag.starts_with("etag-"));

    // 3. Oracle GoldenGate CDC Dispatch to Swarm Event Bus
    let cdc_event = OracleCdcEvent::new(
        2097152,
        "ERP_CORE",
        "PURCHASE_ORDERS",
        OracleCdcOpType::Insert,
        Some(json!({ "po_id": "PO-999", "amount": 75000.0 })),
    );
    assert_eq!(cdc_event.to_swarm_topic(), "oracle.cdc.erp_core.purchase_orders.insert");
    let dispatched = suite.cdc_bridge.dispatch_to_swarm(cdc_event);
    assert!(dispatched.is_ok());

    // 4. Oracle Fusion ERP 3-Way Matching Engine
    // Exact match
    let match_exact = suite.erp_engine.evaluate_3way_match("INV-100", 25000.0, "PO-100", 25000.0, Some("GRN-100"), 1.0);
    assert_eq!(match_exact.status, OracleErpMatchStatus::ExactMatch);
    assert!(match_exact.auto_approved);

    // Variance within tolerance (0.5% variance with 1.0% tolerance)
    let match_var = suite.erp_engine.evaluate_3way_match("INV-101", 25125.0, "PO-101", 25000.0, Some("GRN-101"), 1.0);
    assert_eq!(match_var.status, OracleErpMatchStatus::VarianceWithinThreshold);
    assert!(match_var.auto_approved);

    // Discrepancy (5% variance with 1.0% tolerance)
    let match_bad = suite.erp_engine.evaluate_3way_match("INV-102", 26250.0, "PO-102", 25000.0, Some("GRN-102"), 1.0);
    assert_eq!(match_bad.status, OracleErpMatchStatus::PriceDiscrepancy);
    assert!(!match_bad.auto_approved);
    assert!(match_bad.requires_human_audit);

    // Missing GRN
    let match_nogrn = suite.erp_engine.evaluate_3way_match("INV-103", 25000.0, "PO-103", 25000.0, None, 1.0);
    assert_eq!(match_nogrn.status, OracleErpMatchStatus::MissingGoodsReceipt);
    assert!(!match_nogrn.auto_approved);

    // 5. AgentShield Oracle Database Vault Governance
    // Normal query allowed
    assert!(suite.vault.inspect_query("SELECT * FROM po_headers WHERE po_id = 'PO-100'").is_ok());
    // Destructive query blocked
    assert!(suite.vault.inspect_query("DROP TABLE po_headers CASCADE;").is_err());

    // Column-level data masking
    let mut row = std::collections::HashMap::new();
    row.insert("vendor_name".to_string(), json!("Acme Inc"));
    row.insert("bank_account_num".to_string(), json!("98765432101234"));
    row.insert("ssn_owner".to_string(), json!("000-11-2222"));

    let masked = suite.vault.mask_results(row);
    assert_eq!(masked.get("vendor_name").unwrap(), &json!("Acme Inc"));
    assert_eq!(masked.get("bank_account_num").unwrap(), &json!("XXXXXXXX####"));
    assert_eq!(masked.get("ssn_owner").unwrap(), &json!("XXX-XX-XXXX"));

    // 6. GraalVM Native Image Interop
    let graal_rep = suite.graalvm_bridge.invoke_rule("kyc_validation_rule", json!({ "entity": "Acme Inc" }));
    assert_eq!(graal_rep.binary_name, "kyc_validation_rule");
    assert_eq!(graal_rep.exit_code, 0);

    // 7. Built-in Tools Verification
    let search_tool = Oracle23aiSearchTool::new();
    let search_res_str = search_tool.execute(json!({
        "table_name": "invoices",
        "vector_column": "embedding",
        "top_k": 2
    })).await.unwrap();
    let search_res: serde_json::Value = serde_json::from_str(&search_res_str).unwrap();
    assert_eq!(search_res.get("results_count").unwrap(), 2);

    let match_tool = OracleErpMatchTool::new();
    let match_res_str = match_tool.execute(json!({
        "invoice_id": "INV-500",
        "invoice_amount": 1000.0,
        "po_id": "PO-500",
        "po_amount": 1000.0,
        "grn_id": "GRN-500"
    })).await.unwrap();
    let match_res: serde_json::Value = serde_json::from_str(&match_res_str).unwrap();
    assert_eq!(match_res.get("status").unwrap(), "ExactMatch");
    assert_eq!(match_res.get("auto_approved").unwrap(), true);

    // 8. Telemetry Status
    let st = suite.status();
    assert!(st["vector_engine"]["total_queries"].as_u64().unwrap() >= 1);
    assert!(st["erp_engine"]["matches_executed"].as_u64().unwrap() >= 4);
    assert!(st["vault"]["destructive_blocked"].as_u64().unwrap() >= 1);
    assert!(st["vault"]["fields_masked"].as_u64().unwrap() >= 2);
}

#[tokio::test]
async fn test_oracle_high_concurrency_stress() {
    let suite = std::sync::Arc::new(OracleSuite::new());
    let mut handles = Vec::new();

    // 50 concurrent 3-way matching evaluations
    for i in 0..50 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let inv_amt = 1000.0 + (i as f64);
            let po_amt = 1000.0;
            let rep = s.erp_engine.evaluate_3way_match(
                &format!("INV-{i}"),
                inv_amt,
                &format!("PO-{i}"),
                po_amt,
                Some(&format!("GRN-{i}")),
                2.0,
            );
            assert!(rep.po_id.starts_with("PO-"));
        }));
    }

    // 50 concurrent CDC event dispatches
    for i in 0..50 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let event = OracleCdcEvent::new(
                3000000 + i,
                "FINANCE",
                "PAYMENTS",
                OracleCdcOpType::Insert,
                Some(json!({ "payment_id": format!("PAY-{i}"), "amount": 500.0 * (i as f64) })),
            );
            assert_eq!(event.to_swarm_topic(), "oracle.cdc.finance.payments.insert");
            let _ = s.cdc_bridge.dispatch_to_swarm(event);
        }));
    }

    // 50 concurrent vector queries
    for i in 0..50 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let query = OracleVectorSearchQuery {
                table_name: "contracts".to_string(),
                vector_column: "embedding".to_string(),
                query_vector: vec![0.1 * (i as f32); 4],
                relational_filter: Some(format!("org_id = {i}")),
                metric: OracleVectorMetric::Cosine,
                top_k: 2,
                selected_columns: vec!["contract_id".to_string()],
            };
            let results = s.vector_engine.execute_search(&query);
            assert_eq!(results.len(), 2);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let st = suite.status();
    assert_eq!(st["erp_engine"]["matches_executed"].as_u64().unwrap(), 50);
    assert_eq!(st["cdc_bridge"]["events_ingested"].as_u64().unwrap(), 50);
    assert_eq!(st["vector_engine"]["total_queries"].as_u64().unwrap(), 50);
}

#[tokio::test]
async fn test_oracle_edge_cases_and_security() {
    let suite = OracleSuite::new();

    // 1. Zero tolerance matching (exact match passes, even $0.01 variance fails)
    let rep_exact = suite.erp_engine.evaluate_3way_match("INV-Z1", 100.0, "PO-Z1", 100.0, Some("GRN-Z1"), 0.0);
    assert_eq!(rep_exact.status, OracleErpMatchStatus::ExactMatch);
    assert!(rep_exact.auto_approved);

    let rep_diff = suite.erp_engine.evaluate_3way_match("INV-Z2", 100.01, "PO-Z2", 100.0, Some("GRN-Z2"), 0.0);
    assert_eq!(rep_diff.status, OracleErpMatchStatus::PriceDiscrepancy);
    assert!(!rep_diff.auto_approved);

    // 2. Zero amount edge cases
    let rep_zero = suite.erp_engine.evaluate_3way_match("INV-Z3", 0.0, "PO-Z3", 0.0, Some("GRN-Z3"), 1.0);
    assert_eq!(rep_zero.status, OracleErpMatchStatus::ExactMatch);
    assert!(rep_zero.auto_approved);

    // 3. Negative amount / reversal
    let rep_neg = suite.erp_engine.evaluate_3way_match("INV-Z4", -500.0, "PO-Z4", -500.0, Some("GRN-Z4"), 1.0);
    assert_eq!(rep_neg.status, OracleErpMatchStatus::ExactMatch);

    // 4. Database Vault SQL injection / evasion attempts
    assert!(suite.vault.inspect_query("drop table users;").is_err());
    assert!(suite.vault.inspect_query("DROP TABLE users CASCADE CONSTRAINTS;").is_err());
    assert!(suite.vault.inspect_query("truncate table audit_log;").is_err());
    assert!(suite.vault.inspect_query("alter table employees drop column salary;").is_err());
    assert!(suite.vault.inspect_query("grant all privileges to hacker;").is_err());

    // Safe SELECT queries
    assert!(suite.vault.inspect_query("SELECT id, name FROM users WHERE id = 42").is_ok());
    assert!(suite.vault.inspect_query("INSERT INTO ledger (id, amount) VALUES (1, 100.0)").is_ok());

    // 5. Column Masking with multiple rules
    let mut row = std::collections::HashMap::new();
    row.insert("customer_ssn".to_string(), json!("999-88-7777"));
    row.insert("vendor_bank_account_num".to_string(), json!("1234567890"));
    row.insert("credit_card_num".to_string(), json!("4111222233334444"));
    row.insert("tax_identification_number".to_string(), json!("12-3456789"));
    row.insert("company_name".to_string(), json!("Supreme Court of the Philippines"));

    let masked = suite.vault.mask_results(row);
    assert_eq!(masked.get("customer_ssn").unwrap(), &json!("XXX-XX-XXXX"));
    assert_eq!(masked.get("vendor_bank_account_num").unwrap(), &json!("XXXXXXXX####"));
    assert_eq!(masked.get("credit_card_num").unwrap(), &json!("XXXX-XXXX-XXXX-####"));
    assert_eq!(masked.get("tax_identification_number").unwrap(), &json!("REDACTED_TIN"));
    assert_eq!(masked.get("company_name").unwrap(), &json!("Supreme Court of the Philippines"));
}

#[tokio::test]
async fn test_oracle_extended_ecosystem() {
    let suite = OracleSuite::global();

    // 1. Blockchain Tool (append, proof, verify)
    let b_tool = OracleBlockchainTool::new();
    let app_res_str = b_tool.execute(json!({
        "action_type": "append",
        "audit_action": "SUPREME_COURT_EN_BANC_VOTE",
        "actor": "Chief_Justice_Agent",
        "payload": { "decision": "AFFIRMED", "docket": "G.R. No. 2026-001" }
    })).await.unwrap();
    let app_res: serde_json::Value = serde_json::from_str(&app_res_str).unwrap();
    assert_eq!(app_res["action"], "SUPREME_COURT_EN_BANC_VOTE");
    let block_num = app_res["block_number"].as_u64().unwrap();

    let proof_res_str = b_tool.execute(json!({
        "action_type": "proof",
        "block_number": block_num
    })).await.unwrap();
    let proof_res: serde_json::Value = serde_json::from_str(&proof_res_str).unwrap();
    assert_eq!(proof_res["actor_agent"], "Chief_Justice_Agent");

    let verify_res_str = b_tool.execute(json!({ "action_type": "verify" })).await.unwrap();
    let verify_res: serde_json::Value = serde_json::from_str(&verify_res_str).unwrap();
    assert_eq!(verify_res["chain_valid"], true);
    assert_eq!(verify_res["tamper_detected"], false);

    // 2. APEX Low-Code Generator Tool
    let apex_tool = OracleApexGenTool::new();
    let apex_res_str = apex_tool.execute(json!({
        "app_id": 300,
        "app_name": "RTC Court Docket Dashboard",
        "table_name": "RTC_DOCKETS"
    })).await.unwrap();
    let apex_res: serde_json::Value = serde_json::from_str(&apex_res_str).unwrap();
    assert_eq!(apex_res["metadata"]["app_id"], 300);
    assert!(apex_res["full_ddl_bytes"].as_u64().unwrap() > 0);

    // 3. HeatWave Lakehouse & AutoML Tool
    let hw_tool = OracleHeatWaveTool::new();
    // Query acceleration
    let hw_q_str = hw_tool.execute(json!({
        "operation": "query",
        "table_name": "procurement_lakehouse"
    })).await.unwrap();
    let hw_q: serde_json::Value = serde_json::from_str(&hw_q_str).unwrap();
    assert!(hw_q["lakehouse_sql"].as_str().unwrap().contains("SECONDARY_ENGINE(RAPID)"));

    // In-DB AutoML training
    let hw_ml_str = hw_tool.execute(json!({
        "operation": "automl",
        "table_name": "supplier_records",
        "target_column": "default_risk",
        "task_type": "CLASSIFICATION"
    })).await.unwrap();
    let hw_ml: serde_json::Value = serde_json::from_str(&hw_ml_str).unwrap();
    assert_eq!(hw_ml["task_type"], "CLASSIFICATION");
    assert!(hw_ml["accuracy_or_r2"].as_f64().unwrap() > 0.90);

    // 4. OCI Sovereign Cloud Residency
    let oci = &suite.oci_sovereign;
    assert!(oci.verify_residency_compliance(&OciSovereignRegion::ApManila1Sovereign, &OciSovereignRegion::ApManila1Sovereign).is_ok());
    assert!(oci.verify_residency_compliance(&OciSovereignRegion::UsGovAshburn1, &OciSovereignRegion::ApManila1Sovereign).is_err());
    let inf = oci.route_sovereign_inference(4096, &OciSovereignRegion::ApManila1Sovereign).unwrap();
    assert!(inf.contains("0 foreign egress"));

    // 5. Data Guard Failover
    let dg = &suite.data_guard;
    assert!(dg.status().rpo_zero_guaranteed);
    let fo = dg.trigger_fast_start_failover().unwrap();
    assert!(fo.contains("Fast-Start Failover succeeded"));

    // 6. OIC Enterprise Mesh
    let oic = &suite.oic_mesh;
    let tx = oic.dispatch_enterprise_sync(OicConnectorType::Workday, json!({"employee_id": "EMP-2026"}));
    assert_eq!(tx.response_code, 200);
    assert!(tx.two_phase_commit_synced);

    // 7. Global Status Telemetry
    let st = suite.status();
    assert!(st["blockchain"]["total_blocks"].as_u64().unwrap() >= 1);
    assert!(st["apex"]["apps_generated"].as_u64().unwrap() >= 1);
    assert!(st["oci_sovereign"]["tokens_routed"].as_u64().unwrap() >= 4096);
    assert!(st["data_guard"]["failovers_executed"].as_u64().unwrap() >= 1);
    assert!(st["heatwave"]["queries_accelerated"].as_u64().unwrap() >= 1);
    assert!(st["heatwave"]["models_trained"].as_u64().unwrap() >= 1);
    assert!(st["oic_mesh"]["dispatched_count"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_oracle_advanced_subsystems_lifecycle() {
    let suite = OracleSuite::global();

    // 1. Oracle Property Graph Tool (oracle_graph_query)
    let graph_tool = OraclePropertyGraphTool::new();
    let q_res_str = graph_tool.execute(json!({
        "action": "query",
        "graph_name": "FINANCIAL_FRAUD_NETWORK",
        "vertex_pattern": "(acc1:Account)-[tx:TRANSFERS]->(acc2:Account)",
        "where_clause": "tx.amount >= 1000000",
        "columns": ["acc1.holder_name", "acc2.holder_name", "tx.amount"]
    })).await.unwrap();
    let q_res: serde_json::Value = serde_json::from_str(&q_res_str).unwrap();
    assert!(q_res["graph_table_sql"].as_str().unwrap().contains("GRAPH_TABLE (FINANCIAL_FRAUD_NETWORK"));
    assert!(q_res["graph_table_sql"].as_str().unwrap().contains("WHERE tx.amount >= 1000000"));

    let algo_res_str = graph_tool.execute(json!({
        "action": "algorithm",
        "graph_name": "FINANCIAL_FRAUD_NETWORK",
        "algorithm": "PAGERANK"
    })).await.unwrap();
    let algo_res: serde_json::Value = serde_json::from_str(&algo_res_str).unwrap();
    assert_eq!(algo_res["algorithm"], "PAGERANK");
    assert_eq!(algo_res["status"], "COMPLETED");

    // 2. Oracle Exadata Smart Scan Tool (oracle_exadata_smart_scan)
    let exa_tool = OracleExadataSmartScanTool::new();
    let exa_res_str = exa_tool.execute(json!({
        "table": "supreme_court_dockets",
        "predicate": "filing_year = 2026 AND status = 'EN_BANC'",
        "projection": ["docket_id", "title", "ponente"],
        "simulate": true
    })).await.unwrap();
    let exa_res: serde_json::Value = serde_json::from_str(&exa_res_str).unwrap();
    assert!(exa_res["smart_scan_sql"].as_str().unwrap().contains("CELL_OFFLOAD"));
    assert_eq!(exa_res["simulation_report"]["io_reduction_pct"].as_f64().unwrap(), 94.2);

    // 3. Oracle Text & RRF Hybrid Search Tool (oracle_text_rrf_search)
    let text_tool = OracleTextTool::new();
    let text_res_str = text_tool.execute(json!({
        "index_column": "full_text_judgment",
        "search_expression": "mandamus WITHIN sentence",
        "top_k": 3,
        "hybrid_rrf": true
    })).await.unwrap();
    let text_res: serde_json::Value = serde_json::from_str(&text_res_str).unwrap();
    assert!(text_res["contains_sql"].as_str().unwrap().contains("CONTAINS(full_text_judgment"));
    assert_eq!(text_res["rrf_results"].as_array().unwrap().len(), 3);

    // 4. Oracle Spatial Jurisdiction Tool (oracle_spatial_jurisdiction)
    let spatial_tool = OracleSpatialTool::new();
    // Court jurisdiction check in Manila
    let court_res_str = spatial_tool.execute(json!({
        "action": "court_jurisdiction",
        "lat": 14.5995,
        "lon": 120.9842,
        "venue_type": "RTC"
    })).await.unwrap();
    let court_res: serde_json::Value = serde_json::from_str(&court_res_str).unwrap();
    assert_eq!(court_res["has_jurisdiction"], true);
    assert!(court_res["court_name"].as_str().unwrap().contains("Manila"));

    // GRN Geofencing check
    let geofence_res_str = spatial_tool.execute(json!({
        "action": "grn_geofence",
        "lat": 14.5002,
        "lon": 121.0002,
        "warehouse_lat": 14.5000,
        "warehouse_lon": 121.0000,
        "max_radius_meters": 300.0
    })).await.unwrap();
    let geofence_res: serde_json::Value = serde_json::from_str(&geofence_res_str).unwrap();
    assert_eq!(geofence_res["geofence_verified"], true);
    assert_eq!(geofence_res["status"], "DELIVERY_LOCATION_AUTHENTICATED");

    // 5. Oracle Key Vault HSM Tool (oracle_key_vault_sign)
    let okv_tool = OracleKeyVaultTool::new();
    let hash = "ab530a13e45914982b79f9b7e3fba994cfd1f3fb22f71cea1afbf02b460c6d1d";
    let sign_res_str = okv_tool.execute(json!({
        "action": "sign",
        "key_id": "OKV-PRODUCTION-HSM-KEY",
        "document_hash": hash
    })).await.unwrap();
    let sign_res: serde_json::Value = serde_json::from_str(&sign_res_str).unwrap();
    let sig = sign_res["signature"].as_str().unwrap();
    assert!(sig.starts_with("HSM-SIG-"));

    // Verify signature
    let ver_res_str = okv_tool.execute(json!({
        "action": "verify",
        "key_id": "OKV-PRODUCTION-HSM-KEY",
        "document_hash": hash,
        "signature": sig
    })).await.unwrap();
    let ver_res: serde_json::Value = serde_json::from_str(&ver_res_str).unwrap();
    assert_eq!(ver_res["valid"], true);

    // Rotate TDE master key
    let rot_res_str = okv_tool.execute(json!({ "action": "rotate_tde" })).await.unwrap();
    let rot_res: serde_json::Value = serde_json::from_str(&rot_res_str).unwrap();
    assert_eq!(rot_res["status"], "ROTATED");
    assert!(rot_res["tde_master_key_id"].as_str().unwrap().starts_with("TDE-MASTER-KEY-ROTATED-"));

    // 6. Oracle Coherence Distributed Swarm Grid
    let coherence = &suite.coherence;
    coherence.put_swarm_memory("agent-sentinel", "consensus_state", json!({ "epoch": 99, "leader": "agent-sentinel" })).unwrap();
    let mem = coherence.get_swarm_memory("agent-sentinel", "consensus_state").unwrap();
    assert_eq!(mem["epoch"], 99);
    assert_eq!(mem["leader"], "agent-sentinel");

    assert!(coherence.acquire_distributed_lock("lock:settlement_ledger", 5000));
    assert!(!coherence.acquire_distributed_lock("lock:settlement_ledger", 5000));
    assert!(coherence.release_distributed_lock("lock:settlement_ledger"));

    // 7. Oracle Autonomous Health Framework (AHF)
    let ahf = &suite.ahf;
    let anoms = ahf.diagnose_workload(120, 88.0);
    assert_eq!(anoms.len(), 2);
    let rem1 = ahf.remediate_anomaly(&anoms[0].issue_type);
    assert!(rem1.contains("successfully applied remediation"));

    // 8. Global Status Telemetry Assertion
    let st = suite.status();
    assert!(st["property_graph"]["graph_queries"].as_u64().unwrap() >= 1);
    assert!(st["property_graph"]["traversals_executed"].as_u64().unwrap() >= 1);
    assert!(st["exadata"]["scans_offloaded"].as_u64().unwrap() >= 1);
    assert!(st["exadata"]["bytes_saved_gb"].as_u64().unwrap() >= 94);
    assert!(st["text_engine"]["text_searches"].as_u64().unwrap() >= 1);
    assert!(st["text_engine"]["rrf_fusions"].as_u64().unwrap() >= 1);
    assert!(st["spatial"]["spatial_queries"].as_u64().unwrap() >= 1);
    assert!(st["spatial"]["geofence_checks"].as_u64().unwrap() >= 1);
    assert!(st["key_vault"]["hsm_signings"].as_u64().unwrap() >= 1);
    assert!(st["key_vault"]["rotations_executed"].as_u64().unwrap() >= 1);
    assert!(st["coherence"]["cache_hits"].as_u64().unwrap() >= 1);
    assert!(st["ahf"]["anomalies_detected"].as_u64().unwrap() >= 2);
    assert!(st["ahf"]["remediations_applied"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_oracle_advanced_concurrency_stress() {
    let suite = OracleSuite::global();
    let mut handles = Vec::new();

    // 30 concurrent Property Graph queries & algorithms
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let q = OracleGraphTableQuery {
                graph_name: format!("GRAPH_{i}"),
                vertex_pattern: "(a)-[r]->(b)".to_string(),
                where_clause: Some(format!("r.weight > {i}")),
                columns: vec!["a.id".to_string(), "b.id".to_string()],
            };
            let sql = s.property_graph.generate_graph_table_sql(&q);
            assert!(sql.contains("GRAPH_TABLE"));
            let algo = s.property_graph.run_graph_algorithm(&format!("GRAPH_{i}"), "SHORTEST_PATH");
            assert_eq!(algo["status"], "COMPLETED");
        }));
    }

    // 30 concurrent Exadata Smart Scan offloads
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let q = ExadataScanQuery {
                table: format!("TABLE_{i}"),
                predicate: format!("id > {i}"),
                projection: vec!["id".to_string(), "name".to_string()],
            };
            let _sql = s.exadata.generate_smart_scan_sql(&q);
            let rep = s.exadata.simulate_smart_scan(&q);
            assert_eq!(rep.io_reduction_pct, 94.2);
        }));
    }

    // 30 concurrent Spatial Geofence evaluations
    for _i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let wh = GeoPoint { lat: 14.5000, lon: 121.0000, srid: 4326 };
            let trk = GeoPoint { lat: 14.5001, lon: 121.0001, srid: 4326 };
            assert!(s.spatial.verify_grn_geofence(&wh, &trk, 500.0));
        }));
    }

    // 30 concurrent OKV HSM signings & verifications
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let hash = format!("HASH-{i}-DEADBEEF");
            let sig = s.key_vault.sign_document_in_hsm("OKV-CONCURRENCY-KEY", &hash).unwrap();
            assert!(s.key_vault.verify_hsm_signature("OKV-CONCURRENCY-KEY", &hash, &sig));
        }));
    }

    // 30 concurrent Coherence Swarm Grid puts & gets
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            s.coherence.put_swarm_memory(&format!("agent-{i}"), "tick", json!({ "tick": i })).unwrap();
            let val = s.coherence.get_swarm_memory(&format!("agent-{i}"), "tick").unwrap();
            assert_eq!(val["tick"], i);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let st = suite.status();
    assert!(st["property_graph"]["graph_queries"].as_u64().unwrap() >= 30);
    assert!(st["exadata"]["scans_offloaded"].as_u64().unwrap() >= 30);
    assert!(st["spatial"]["geofence_checks"].as_u64().unwrap() >= 30);
    assert!(st["key_vault"]["hsm_signings"].as_u64().unwrap() >= 30);
    assert!(st["coherence"]["cache_hits"].as_u64().unwrap() >= 30);
}

#[tokio::test]
async fn test_oracle_frontier_subsystems_lifecycle() {
    let suite = OracleSuite::global();

    // 1. Oracle TxEQ Tool (oracle_txeq_queue)
    let txeq_tool = OracleTxEqTool::new();
    let enq_res_str = txeq_tool.execute(json!({
        "action": "enqueue",
        "queue_name": "AGENT_DISPATCH_QUEUE",
        "payload": { "action": "FORENSIC_AUDIT", "target": "VENDOR_001" },
        "priority": "HIGH"
    })).await.unwrap();
    let enq_res: serde_json::Value = serde_json::from_str(&enq_res_str).unwrap();
    assert_eq!(enq_res["status"], "ENQUEUED");
    assert_eq!(enq_res["priority"], "HIGH");

    let deq_res_str = txeq_tool.execute(json!({
        "action": "dequeue",
        "queue_name": "AGENT_DISPATCH_QUEUE"
    })).await.unwrap();
    let deq_res: serde_json::Value = serde_json::from_str(&deq_res_str).unwrap();
    assert_eq!(deq_res["found"], true);

    let batch_res_str = txeq_tool.execute(json!({
        "action": "batch_commit",
        "queue_name": "OUTBOX_TX_QUEUE",
        "messages": [{ "tx_id": 1 }, { "tx_id": 2 }]
    })).await.unwrap();
    let batch_res: serde_json::Value = serde_json::from_str(&batch_res_str).unwrap();
    assert_eq!(batch_res["messages_committed"], 2);

    // 2. Oracle Database In-Memory Tool (oracle_inmemory_query)
    let inmem_tool = OracleInMemoryTool::new();
    // Pre-populate inmemory table
    suite.inmemory.load_table("COURT_FEES", vec![
        [("amount".to_string(), 1500.0)].into_iter().collect(),
        [("amount".to_string(), 3500.0)].into_iter().collect(),
        [("amount".to_string(), 5000.0)].into_iter().collect(),
    ]);

    let agg_res_str = inmem_tool.execute(json!({
        "action": "aggregate",
        "table_name": "COURT_FEES",
        "column_name": "amount",
        "operation": "SUM"
    })).await.unwrap();
    let agg_res: serde_json::Value = serde_json::from_str(&agg_res_str).unwrap();
    assert_eq!(agg_res["result_value"], 10000.0);
    assert_eq!(agg_res["rows_scanned"], 3);

    let sql_res_str = inmem_tool.execute(json!({
        "action": "generate_sql",
        "table_name": "COURT_FEES",
        "projection": "fee_id, amount",
        "filter": "amount > 2000"
    })).await.unwrap();
    let sql_res: serde_json::Value = serde_json::from_str(&sql_res_str).unwrap();
    assert!(sql_res["inmemory_sql"].as_str().unwrap().contains("INMEMORY MEMCOMPRESS FOR QUERY HIGH"));

    // 3. Oracle SQL Firewall Tool (oracle_sql_firewall)
    let fw_tool = OracleSqlFirewallTool::new();
    let ok_res_str = fw_tool.execute(json!({
        "sql": "SELECT * FROM supreme_court_rulings WHERE ruling_year = 2026",
        "agent_role": "RESEARCH_AGENT"
    })).await.unwrap();
    let ok_res: serde_json::Value = serde_json::from_str(&ok_res_str).unwrap();
    assert_eq!(ok_res["allowed"], true);

    let blocked_res_str = fw_tool.execute(json!({
        "sql": "SELECT * FROM users WHERE id = 1 OR '1'='1' --",
        "agent_role": "MALICIOUS_PROMPT_INJECTOR"
    })).await.unwrap();
    let blocked_res: serde_json::Value = serde_json::from_str(&blocked_res_str).unwrap();
    assert_eq!(blocked_res["allowed"], false);
    assert_eq!(blocked_res["detected_signature"], "SQL_INJECTION");

    // 4. Oracle RAS / VPD Security Tool (oracle_ras_security)
    let ras_tool = OracleRasTool::new();
    let rewrite_res_str = ras_tool.execute(json!({
        "action": "rewrite_query",
        "table_name": "COURT_DOCKETS",
        "base_sql": "SELECT * FROM COURT_DOCKETS",
        "user_id": "judge_ncr_01",
        "branch_id": 5,
        "clearance_level": 2
    })).await.unwrap();
    let rewrite_res: serde_json::Value = serde_json::from_str(&rewrite_res_str).unwrap();
    assert!(rewrite_res["rewritten_sql"].as_str().unwrap().contains("branch_id = 5"));
    assert!(rewrite_res["rewritten_sql"].as_str().unwrap().contains("classification_level <= 2"));

    let filter_res_str = ras_tool.execute(json!({
        "action": "filter_record",
        "table_name": "COURT_DOCKETS",
        "user_id": "judge_ncr_01",
        "branch_id": 5,
        "clearance_level": 2,
        "record": { "branch_id": 5, "classification_level": 2 }
    })).await.unwrap();
    let filter_res: serde_json::Value = serde_json::from_str(&filter_res_str).unwrap();
    assert_eq!(filter_res["access_granted"], true);

    // 5. Oracle True Cache Tool (oracle_true_cache)
    let tc_tool = OracleTrueCacheTool::new();
    let put_res_str = tc_tool.execute(json!({
        "action": "put",
        "table_name": "LEGAL_TERMS",
        "key": "CERTIORARI",
        "value": { "definition": "A writ issuing from a superior court to call up records of an inferior court" }
    })).await.unwrap();
    let put_res: serde_json::Value = serde_json::from_str(&put_res_str).unwrap();
    assert_eq!(put_res["status"], "CACHED");

    let get_res_str = tc_tool.execute(json!({
        "action": "get",
        "table_name": "LEGAL_TERMS",
        "key": "CERTIORARI"
    })).await.unwrap();
    let get_res: serde_json::Value = serde_json::from_str(&get_res_str).unwrap();
    assert_eq!(get_res["hit"], true);

    let inv_res_str = tc_tool.execute(json!({
        "action": "invalidate",
        "table_name": "LEGAL_TERMS"
    })).await.unwrap();
    let inv_res: serde_json::Value = serde_json::from_str(&inv_res_str).unwrap();
    assert_eq!(inv_res["status"], "INVALIDATED");

    // 6. GDD Sharding & GGSA CEP Analytics Engines
    let shard_ph = suite.gdd.route_query_to_shard("PH_METRO_MANILA_CASE_01").unwrap();
    assert_eq!(shard_ph.region, "ap-manila-1");
    let tx_2pc = suite.gdd.execute_2pc_distributed_tx("TX-CROSS-SHARD-001", vec![1, 2]);
    assert!(tx_2pc.two_phase_commit_success);

    // Ingest events into GGSA CEP
    suite.ggsa.ingest_event(100_000.0, "CEBU");
    suite.ggsa.ingest_event(150_000.0, "MANILA");
    suite.ggsa.ingest_event(200_000.0, "DAVAO");
    let alert = suite.ggsa.ingest_event(250_000.0, "ILOILO");
    assert!(alert.is_some());

    // 7. Telemetry Status
    let st = suite.status();
    assert!(st["txeq"]["enqueued_count"].as_u64().unwrap() >= 1);
    assert!(st["txeq"]["dequeued_count"].as_u64().unwrap() >= 1);
    assert!(st["inmemory"]["queries_accelerated"].as_u64().unwrap() >= 2);
    assert!(st["sql_firewall"]["queries_inspected"].as_u64().unwrap() >= 2);
    assert!(st["sql_firewall"]["blocked_attempts"].as_u64().unwrap() >= 1);
    assert!(st["ras"]["policies_enforced"].as_u64().unwrap() >= 1);
    assert!(st["true_cache"]["cache_hits"].as_u64().unwrap() >= 1);
    assert!(st["true_cache"]["invalidations_received"].as_u64().unwrap() >= 1);
    assert!(st["gdd"]["routed_queries"].as_u64().unwrap() >= 1);
    assert!(st["ggsa"]["patterns_detected"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_oracle_frontier_concurrency_stress() {
    let suite = OracleSuite::global();
    let mut handles = Vec::new();

    // 30 concurrent TxEQ enqueue and dequeue operations
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let msg_id = s.txeq.enqueue("CONCURRENCY_Q", json!({"idx": i}), TxEqPriority::Normal, None).unwrap();
            assert!(msg_id.starts_with("TXEQ-MSG-"));
            let deq = s.txeq.dequeue("CONCURRENCY_Q");
            assert!(deq.is_some());
        }));
    }

    // 30 concurrent SQL Firewall inspections
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let insp = s.sql_firewall.inspect_sql(&format!("SELECT * FROM records WHERE id = {i}"), "AGENT");
            assert!(insp.allowed);
        }));
    }

    // 30 concurrent RAS VPD rewrites
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let ctx = RasSessionContext {
                user_id: format!("agent_{i}"),
                role: "WORKER".to_string(),
                branch_id: (i % 5) + 1,
                clearance_level: 2,
            };
            let rew = s.ras.rewrite_query_with_vpd("COURT_DOCKETS", "SELECT * FROM COURT_DOCKETS", &ctx);
            assert!(rew.contains("branch_id"));
        }));
    }

    // 30 concurrent True Cache puts & gets
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            s.true_cache.put("BENCHMARKS", &format!("key_{i}"), json!({"score": i * 10}));
            let hit = s.true_cache.get("BENCHMARKS", &format!("key_{i}")).unwrap();
            assert_eq!(hit["score"], i * 10);
        }));
    }

    // 30 concurrent GDD shard routing calls
    for i in 0..30 {
        let s = suite.clone();
        handles.push(tokio::spawn(async move {
            let route = s.gdd.route_query_to_shard(&format!("PH_KEY_{i}")).unwrap();
            assert_eq!(route.region, "ap-manila-1");
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let st = suite.status();
    assert!(st["txeq"]["enqueued_count"].as_u64().unwrap() >= 30);
    assert!(st["sql_firewall"]["queries_inspected"].as_u64().unwrap() >= 30);
    assert!(st["ras"]["policies_enforced"].as_u64().unwrap() >= 30);
    assert!(st["true_cache"]["cache_hits"].as_u64().unwrap() >= 30);
    assert!(st["gdd"]["routed_queries"].as_u64().unwrap() >= 30);
}


