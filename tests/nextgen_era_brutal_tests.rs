//! # Comprehensive Brutal Test Suite for Next-Gen Sovereign Engineering Engines
//!
//! Brutally verifies:
//! 1. Blake3 Merkle tree root calculation, tree construction & inclusion proof verification/tampering.
//! 2. Provenance ledger hash chaining, statutory rule attestations (Rule 141, A.M. No. 03-8-02-SC), and tamper detection.
//! 3. Swarm WAL append-only event sourcing, sequence monotonicity, and payload hashing.
//! 4. Time-travel deterministic checkpointing, state snapshotting, diffs, and sequence rollbacks.
//! 5. Independent swarm execution branch forking.
//! 6. Differential fuzzing across boundary numbers, unicode homoglyphs, injection payloads, and delta shrinking.
//! 7. ZeroConf P2P Swarm Mesh capability discovery, compute tier satisfaction, and heartbeat TTL pruning.
//! 8. Distributed work-stealing queue and cluster load balancing.
//! 9. Speculative hybrid orchestration, Lakandiwa entropy gating, multi-token verification, and speedup metrics.
//! 10. Interactive Cockpit state machine, mid-flight steering actions, and headless Ratatui TUI rendering.

use tagisan::nextgen::*;

// ============================================================================
// Pillar 8: Cryptographic Proof of Autonomous Provenance & Merkle Trees
// ============================================================================

#[test]
fn test_merkle_tree_empty_and_single_leaf() {
    // 0 leaves
    let empty_tree = MerkleTree::new(Vec::new());
    assert_eq!(empty_tree.root(), [0u8; 32]);
    assert!(empty_tree.generate_proof(0).is_none());

    // 1 leaf
    let leaf = *blake3::hash(b"single_leaf_data").as_bytes();
    let single_tree = MerkleTree::new(vec![leaf]);
    assert_eq!(single_tree.root(), leaf);

    let proof = single_tree.generate_proof(0).expect("Proof must exist for leaf 0");
    assert_eq!(proof.leaf_hash, leaf);
    assert!(proof.audit_path.is_empty());
    assert!(proof.verify(), "Single leaf proof must verify against its root");
    assert!(proof.verify_against_root(&leaf));
}

#[test]
fn test_merkle_tree_construction_and_tamper_detection() {
    let num_leaves = 17; // Non-power of two to thoroughly exercise duplication of odd nodes
    let mut leaves = Vec::new();
    for i in 0..num_leaves {
        let h = *blake3::hash(format!("leaf_payload_{}", i).as_bytes()).as_bytes();
        leaves.push(h);
    }

    let tree = MerkleTree::new(leaves.clone());
    let root = tree.root();
    assert_ne!(root, [0u8; 32]);
    assert_eq!(tree.root_hex().len(), 64);

    // Verify inclusion proofs for every single leaf
    for i in 0..num_leaves {
        let proof = tree.generate_proof(i).unwrap_or_else(|| panic!("Proof missing for leaf {}", i));
        assert_eq!(proof.leaf_index, i);
        assert_eq!(proof.leaf_hash, leaves[i]);
        assert_eq!(proof.root_hash, root);
        assert!(proof.verify(), "Proof for leaf {} must verify against tree root", i);
    }

    // Tamper Test 1: Corrupted leaf hash in proof
    let mut tampered_proof = tree.generate_proof(3).unwrap();
    tampered_proof.leaf_hash[0] ^= 0xFF;
    assert!(!tampered_proof.verify(), "Tampered leaf hash must fail verification");

    // Tamper Test 2: Corrupted sibling hash in audit path
    let mut tampered_audit = tree.generate_proof(5).unwrap();
    assert!(!tampered_audit.audit_path.is_empty());
    tampered_audit.audit_path[0].sibling_hash[0] ^= 0xAA;
    assert!(!tampered_audit.verify(), "Tampered sibling hash must fail verification");

    // Tamper Test 3: Verify against wrong root
    let valid_proof = tree.generate_proof(7).unwrap();
    let bogus_root = [0x55u8; 32];
    assert!(!valid_proof.verify_against_root(&bogus_root));
}

#[test]
fn test_provenance_ledger_statutory_and_tamper_detection() {
    let mut ledger = ProvenanceLedger::new();

    // Append 5 sovereign agent actions
    ledger.append(
        "agent:ingestion",
        ProvenanceAction::PromptIngestion {
            prompt_blake3: "hash_of_user_prompt_123".to_string(),
            user_urn: "urn:tgs:user:charle".to_string(),
            prompt_tokens: 64,
        },
        vec![StatutoryAttestation::supreme_court_electronic_evidence("agent:ingestion")],
    ).expect("Append prompt ingestion failed");

    ledger.append(
        "agent:reasoner",
        ProvenanceAction::ModelInference {
            model_id: "deepseek-r1:14b".to_string(),
            prompt_tokens: 64,
            completion_tokens: 128,
            temperature: 0.1,
            finish_reason: "stop".to_string(),
        },
        vec![StatutoryAttestation::rule_141_attestation("agent:reasoner")],
    ).expect("Append model inference failed");

    ledger.append(
        "agent:tool_executor",
        ProvenanceAction::ToolExecution {
            tool_name: "smt_solver".to_string(),
            parameters_blake3: "params_hash_xyz".to_string(),
            exit_code: 0,
            duration_ms: 15,
        },
        vec![],
    ).expect("Append tool execution failed");

    ledger.append(
        "agent:synthesizer",
        ProvenanceAction::CodeModification {
            file_path: "src/engine.rs".to_string(),
            diff_blake3: "diff_hash_abc".to_string(),
            lines_added: 12,
            lines_removed: 3,
        },
        vec![],
    ).expect("Append code mod failed");

    ledger.append(
        "agent:supervisor",
        ProvenanceAction::AutonomousDecision {
            rationale: "SMT proof passed, committed patch".to_string(),
            confidence: 0.999,
            policy_urn: "urn:tgs:policy:safety_v1".to_string(),
        },
        vec![],
    ).expect("Append decision failed");

    assert_eq!(ledger.entries.len(), 5);

    // Verify pristine chain integrity
    assert!(ledger.verify_chain_integrity().expect("Integrity check failed"));

    // Verify statutory citations
    let att1 = &ledger.entries[0].attestations[0];
    assert_eq!(att1.standard, StatutoryStandard::AmNo03_8_02_SC);
    assert!(att1.statement.contains("A.M. No. 03-8-02-SC"));
    assert!(att1.certified);

    let att2 = &ledger.entries[1].attestations[0];
    assert_eq!(att2.standard, StatutoryStandard::Rule141);
    assert!(att2.statement.contains("Rule 141"));

    // Generate and verify verification certificates
    for seq in 0..5 {
        let cert = ledger.generate_certificate(seq).expect("Certificate generation failed");
        assert_eq!(cert.entry_sequence, seq);
        assert!(cert.verify(), "Certificate must self-verify cryptographically");
        assert!(ledger.verify_certificate(&cert).expect("Ledger certificate check failed"));
    }

    // Tamper Attack 1: Mutate entry actor
    let mut tampered_ledger1 = ledger.clone();
    tampered_ledger1.entries[2].actor = "impostor_agent".to_string();
    let result1 = tampered_ledger1.verify_chain_integrity();
    assert!(result1.is_err(), "Mutating actor must break hash integrity");

    // Tamper Attack 2: Break prev_hash chain link
    let mut tampered_ledger2 = ledger.clone();
    tampered_ledger2.entries[3].prev_hash = [0x99; 32];
    let result2 = tampered_ledger2.verify_chain_integrity();
    assert!(result2.is_err(), "Breaking prev_hash must be caught by chain verification");
}

// ============================================================================
// Pillar 9: Swarm WAL & Time-Travel Event-Sourced Checkpointing
// ============================================================================

#[test]
fn test_swarm_wal_monotonicity_and_range_queries() {
    let mut wal = SwarmWal::new();
    assert!(wal.is_empty());

    for i in 0..10 {
        wal.append(
            format!("agent_{}", i),
            SwarmEventType::TaskSpawned {
                task_id: format!("task-{}", i),
                node_name: format!("node-{}", i),
            },
        );
    }

    assert_eq!(wal.len(), 10);
    assert!(wal.verify_integrity());

    // Verify sequence monotonicity
    for (idx, e) in wal.events.iter().enumerate() {
        assert_eq!(e.sequence, idx as u64);
        assert_eq!(e.agent_id, format!("agent_{}", idx));
    }

    // Read range [3, 7]
    let range = wal.read_range(3, 7);
    assert_eq!(range.len(), 5);
    assert_eq!(range[0].sequence, 3);
    assert_eq!(range[4].sequence, 7);

    // Events after 6
    let after = wal.events_after(6);
    assert_eq!(after.len(), 3);
    assert_eq!(after[0].sequence, 7);
    assert_eq!(after[2].sequence, 9);

    // Truncate after sequence 5
    wal.truncate_after(5);
    assert_eq!(wal.len(), 6);
    assert_eq!(wal.events.last().unwrap().sequence, 5);
    assert_eq!(wal.current_sequence, 6);
    assert!(wal.verify_integrity());
}

#[test]
fn test_swarm_checkpoint_time_travel_rollback_and_diff() {
    let mut mgr = SwarmCheckpointManager::new("main");

    // --- Phase 1: Setup Node A ---
    mgr.update_dag_node(
        "node_a",
        "agent_a",
        "Completed",
        "Parsed AST successfully",
        serde_json::json!({"file": "main.rs"}),
        Some(serde_json::json!({"tokens": 120})),
    );
    mgr.set_memory("step", serde_json::json!(1));
    let ckpt1 = mgr.create_checkpoint("Checkpoint Phase 1").unwrap();
    assert_eq!(ckpt1.dag_states.len(), 1);
    assert_eq!(mgr.get_memory("step").unwrap(), &serde_json::json!(1));

    // --- Phase 2: Add Node B and mutate memory ---
    mgr.update_dag_node(
        "node_b",
        "agent_b",
        "Running",
        "Running symbolic verification",
        serde_json::json!({"node_a_out": 120}),
        None,
    );
    mgr.set_memory("step", serde_json::json!(2));
    mgr.set_memory("cache_key", serde_json::json!("hit"));
    let ckpt2 = mgr.create_checkpoint("Checkpoint Phase 2").unwrap();
    assert_eq!(ckpt2.dag_states.len(), 2);

    // --- Phase 3: Add Node C and mutate memory ---
    mgr.update_dag_node(
        "node_c",
        "agent_c",
        "Failed",
        "Division by zero detected",
        serde_json::json!({}),
        Some(serde_json::json!({"err": "div_zero"})),
    );
    mgr.set_memory("step", serde_json::json!(3));
    let ckpt3 = mgr.create_checkpoint("Checkpoint Phase 3").unwrap();
    assert_eq!(ckpt3.dag_states.len(), 3);

    // Checkpoint diff between ckpt1 and ckpt3
    let diff = mgr.diff_checkpoints(&ckpt1.checkpoint_id, &ckpt3.checkpoint_id).unwrap();
    assert_eq!(diff.added_dag_nodes, vec!["node_b".to_string(), "node_c".to_string()]);
    assert!(diff.changed_memory_keys.contains(&"step".to_string()));
    assert!(diff.changed_memory_keys.contains(&"cache_key".to_string()));

    // Time-travel rollback to ckpt1
    let rolled_back = mgr.rollback_to_checkpoint(&ckpt1.checkpoint_id).unwrap();
    assert_eq!(rolled_back.checkpoint_id, ckpt1.checkpoint_id);
    assert_eq!(mgr.current_dag_states.len(), 1);
    assert!(mgr.current_dag_states.contains_key("node_a"));
    assert!(!mgr.current_dag_states.contains_key("node_b"));
    assert!(!mgr.current_dag_states.contains_key("node_c"));
    assert_eq!(mgr.get_memory("step").unwrap(), &serde_json::json!(1));
    assert!(mgr.get_memory("cache_key").is_none());

    // Merkle root consistency after rollback
    let current_root = SwarmCheckpoint::compute_state_root(&mgr.current_dag_states, &mgr.current_swarm_memory);
    assert_eq!(current_root, ckpt1.state_merkle_root);
}

#[test]
fn test_swarm_branch_forking() {
    let mut mgr = SwarmCheckpointManager::new("main");
    mgr.update_dag_node("node_base", "agent_base", "Completed", "Base state", serde_json::json!({}), None);
    mgr.set_memory("base_key", serde_json::json!("v1"));
    let ckpt = mgr.create_checkpoint("Base Checkpoint").unwrap();

    // Fork branch
    let mut forked_mgr = mgr.fork_branch(&ckpt.checkpoint_id, "feature-speculative").unwrap();
    assert_eq!(forked_mgr.active_branch, "feature-speculative");
    assert_eq!(forked_mgr.get_memory("base_key").unwrap(), &serde_json::json!("v1"));

    // Advance forked branch independently
    forked_mgr.set_memory("experimental_flag", serde_json::json!(true));
    let forked_ckpt = forked_mgr.create_checkpoint("Experimental Checkpoint").unwrap();
    assert_eq!(forked_ckpt.branch_name, "feature-speculative");

    // Ensure main branch remains unaffected
    assert!(mgr.get_memory("experimental_flag").is_none());
    assert_eq!(mgr.active_branch, "main");
}

// ============================================================================
// Pillar 10: Synthetic Environment Synthesis & Differential Fuzzing
// ============================================================================

#[test]
fn test_differential_fuzzing_all_strategies_and_shrinker() {
    let engine = AgenticFuzzEngine::new()
        .with_strategy(MutationStrategy::BoundaryNumbers)
        .with_strategy(MutationStrategy::UnicodeHomoglyphs)
        .with_strategy(MutationStrategy::InjectionPayloads)
        .with_strategy(MutationStrategy::ConcurrentCollisions)
        .with_max_iterations(40)
        .with_invariant(PropertyInvariant::NoCrashOrPanic)
        .with_invariant(PropertyInvariant::Utf8Integrity)
        .with_invariant(PropertyInvariant::BoundsSafety { min: -1_000_000, max: 1_000_000 })
        .with_invariant(PropertyInvariant::NoFormatStringVulnerability);

    // Target 1: Resilient sanitized parser -> Expect 0 violations
    let resilient_target = |input: &FuzzInput| -> FuzzTargetOutput {
        // Sanitize all percent signs / format specifiers
        let clean_str = input.raw_string.replace('%', "_");
        let clamped_num = input.numeric_val.map(|n| n.clamp(-1_000_000, 1_000_000));
        FuzzTargetOutput {
            output_str: clean_str,
            numeric_output: clamped_num,
            success: true,
            error_message: None,
            duration_micros: 5,
        }
    };

    let report1 = engine.fuzz(resilient_target);
    assert_eq!(report1.total_iterations, 40);
    assert_eq!(report1.passed_iterations, 40);
    assert_eq!(report1.violations_found, 0);
    assert!(!report1.has_violations());

    // Target 2: Vulnerable parser that reflects format strings and panics on bounds -> Expect violations
    let vulnerable_target = |input: &FuzzInput| -> FuzzTargetOutput {
        if let Some(num) = input.numeric_val {
            if num > 1_000_000 {
                return FuzzTargetOutput::success_with_num("overflow", num);
            }
        }
        if input.raw_string.contains("%n") {
            // Reflect raw format string
            return FuzzTargetOutput::success(format!("Vulnerable reflection: {}", input.raw_string));
        }
        FuzzTargetOutput::success("ok")
    };

    let report2 = engine.fuzz(vulnerable_target);
    assert!(report2.has_violations(), "Vulnerable target must trigger invariant violations");
    assert!(!report2.counterexamples.is_empty());

    // Test Shrinker directly on a long input containing a format string specifier
    let raw_payload = "PREFIX_FILLER_ABCDEF123456_%n_SUFFIX_FILLER_7890XYZ";
    let shrunk = engine.shrink(
        raw_payload,
        &vulnerable_target,
        &PropertyInvariant::NoFormatStringVulnerability,
    );
    assert!(shrunk.len() <= raw_payload.len());
    assert!(shrunk.contains("%n"), "Shrunk payload must retain minimal triggering token");
}

// ============================================================================
// Pillar 11: ZeroConf P2P Swarm Mesh & Work Stealing
// ============================================================================

#[test]
fn test_p2p_mesh_discovery_capabilities_and_work_stealing() {
    let local_node = PeerCapability::new(
        "node_laptop",
        "DevLaptop-M3",
        "127.0.0.1:7420",
        ComputeTier::Tier3EdgeLocal,
        16_384, // 16GB VRAM
    ).with_models(vec!["smollm2:1.7b".to_string(), "phi-3".to_string()]);

    let mut mesh = P2pSwarmMesh::new(local_node, "tgs-cluster-phoenix");

    // Peer 1: High-end GPU Workstation
    let gpu_peer = PeerCapability::new(
        "node_rig",
        "Dual-4090-Workstation",
        "192.168.1.100:7420",
        ComputeTier::Tier2NpuServer,
        49_152, // 48GB VRAM
    ).with_models(vec!["llama3.3:70b".to_string(), "deepseek-r1:14b".to_string()]);

    let beacon = MeshBeacon {
        cluster_id: "tgs-cluster-phoenix".to_string(),
        sender: gpu_peer,
        timestamp_epoch_ms: 5000,
        nonce: 42,
        signature_token: "sig:rig".to_string(),
    };

    let accepted = mesh.receive_beacon(beacon).expect("Beacon failed");
    assert!(accepted);
    assert_eq!(mesh.peers.len(), 1);

    // Check capability satisfaction
    let task_light = MeshTask::new(
        "task_light",
        "Local Draft",
        ComputeTier::Tier3EdgeLocal,
        4_096,
        serde_json::json!({}),
        "node_laptop",
    );
    assert!(mesh.can_node_execute(&mesh.local_node, &task_light));

    let task_heavy = MeshTask::new(
        "task_heavy",
        "70B Deep Reasoning",
        ComputeTier::Tier2NpuServer,
        32_768,
        serde_json::json!({}),
        "node_laptop",
    ).with_target_model("llama3.3:70b");

    // Local edge node cannot run heavy 70B task
    assert!(!mesh.can_node_execute(&mesh.local_node, &task_heavy));
    // Remote GPU rig can run heavy task
    assert!(mesh.can_node_execute(mesh.peers.get("node_rig").unwrap(), &task_heavy));

    // Best node locator should select "node_rig" for heavy task
    let best = mesh.find_best_node_for_task(&task_heavy);
    assert_eq!(best, Some("node_rig".to_string()));

    // Test Work Stealing:
    // Push light tasks into remote peer queue
    if let Some(queue) = mesh.peer_queues.get_mut("node_rig") {
        for i in 0..4 {
            queue.push_local(MeshTask::new(
                format!("task_peer_{}", i),
                "Drafting",
                ComputeTier::Tier3EdgeLocal,
                2048,
                serde_json::json!({}),
                "node_rig",
            ));
        }
    }

    assert_eq!(mesh.peer_queues.get("node_rig").unwrap().len(), 4);

    // Local idle node steals work from "node_rig"
    let stolen = mesh.auto_work_steal();
    assert!(stolen.is_some());
    let stolen_task = stolen.unwrap();
    assert_eq!(stolen_task.steals_count, 1);
    assert_eq!(mesh.peer_queues.get("node_rig").unwrap().len(), 3);

    // Verify cluster metrics
    let metrics = mesh.cluster_metrics();
    assert_eq!(metrics.total_nodes, 2);
    assert_eq!(metrics.online_nodes, 2);
    assert!(metrics.total_vram_mb >= 65_536);
    assert!(metrics.available_models.contains(&"llama3.3:70b".to_string()));
}

// ============================================================================
// Pillar 2: Speculative Swarm Orchestration & Lakandiwa Entropy Gate
// ============================================================================

#[test]
fn test_speculative_hybrid_entropy_gating_and_metrics() {
    let config = EntropyGateConfig {
        max_entropy_bits: 1.5,
        min_margin: 0.20,
        thermal_throttle_celsius: 80.0,
        min_battery_pct: 15,
    };
    let engine = SpeculativeHybridEngine::new(config);

    // High confidence distribution -> Low entropy -> Accept
    let confident_dist = DraftDistribution::new(vec![
        CandidateToken { token: "fn".to_string(), probability: 0.90, logprob: -0.15 },
        CandidateToken { token: "let".to_string(), probability: 0.08, logprob: -2.52 },
        CandidateToken { token: "pub".to_string(), probability: 0.02, logprob: -3.91 },
    ]);
    assert!(confident_dist.shannon_entropy() < 0.6);
    assert!(confident_dist.top_margin() > 0.8);

    let verdict1 = engine.evaluate_draft(&confident_dist);
    match &verdict1 {
        LakandiwaVerdict::AcceptLocalDraft { token, confidence, .. } => {
            assert_eq!(token, "fn");
            assert!(*confidence > 0.85);
        }
        _ => panic!("Expected AcceptLocalDraft, got {:?}", verdict1),
    }

    // High uncertainty distribution -> High entropy -> Escalate
    let uncertain_dist = DraftDistribution::new(vec![
        CandidateToken { token: "match".to_string(), probability: 0.35, logprob: -1.04 },
        CandidateToken { token: "if".to_string(), probability: 0.33, logprob: -1.10 },
        CandidateToken { token: "while".to_string(), probability: 0.32, logprob: -1.13 },
    ]);
    assert!(uncertain_dist.shannon_entropy() > 1.5);
    assert!(uncertain_dist.top_margin() < 0.05);

    let verdict2 = engine.evaluate_draft(&uncertain_dist);
    match &verdict2 {
        LakandiwaVerdict::EscalateToFrontier { reason, .. } => {
            assert!(reason.contains("High Shannon entropy") || reason.contains("margin too narrow"));
        }
        _ => panic!("Expected EscalateToFrontier, got {:?}", verdict2),
    }

    // Multi-token speculative batch sequence
    let batch = vec![
        confident_dist.clone(),
        confident_dist.clone(),
        uncertain_dist.clone(),
        confident_dist.clone(), // Should be invalidated due to prior escalation
    ];
    let batch_result = engine.evaluate_draft_sequence(&batch);
    assert_eq!(batch_result.total_evaluated, 4);
    assert_eq!(batch_result.accepted_tokens.len(), 2);
    assert_eq!(batch_result.first_escalation_index, Some(2));
    assert_eq!(batch_result.acceptance_rate, 0.5);

    // Test SpeculativeSwarmAcceptanceMetrics
    let mut metrics = SpeculativeSwarmAcceptanceMetrics::new();
    for _ in 0..8 {
        metrics.record_verdict(&verdict1);
    }
    for _ in 0..2 {
        metrics.record_verdict(&verdict2);
    }
    assert_eq!(metrics.total_draft_tokens, 10);
    assert_eq!(metrics.accepted_draft_tokens, 8);
    assert_eq!(metrics.rejected_draft_tokens, 2);
    assert!((metrics.acceptance_rate() - 0.8).abs() < 1e-6);

    // Speedup calculation for 5x draft model
    let speedup = metrics.effective_speedup(5.0);
    assert!(speedup > 2.0 && speedup < 5.0, "Speedup should be in expected speculative range");
}

// ============================================================================
// Pillar 12: Interactive Cockpit TUI Engine & Mid-Flight Steering
// ============================================================================

#[test]
fn test_cockpit_steering_and_headless_rendering() {
    let mut state = CockpitState::new();

    let mut node1 = CockpitDagNode::new("n_parse", "AST Code Parser", "smollm2:1.7b");
    node1.status = CockpitNodeStatus::Running { progress_pct: 45 };
    node1.tokens_used = 210;
    node1.scratchpad = "Tokenizing Rust source tree...".to_string();

    let mut node2 = CockpitDagNode::new("n_prover", "Z3 SMT Invariant Prover", "deepseek-r1:14b");
    node2.status = CockpitNodeStatus::Pending;
    node2.dependencies = vec!["n_parse".to_string()];

    state.add_node(node1);
    state.add_node(node2);
    assert_eq!(state.nodes.len(), 2);

    // Mid-Flight Steering Action 1: Pause Node
    state.apply_steering(SteeringAction::Pause { node_id: "n_parse".to_string() }).unwrap();
    assert_eq!(state.nodes[0].status, CockpitNodeStatus::Paused);

    // Mid-Flight Steering Action 2: Edit Scratchpad
    state.apply_steering(SteeringAction::EditScratchpad {
        node_id: "n_parse".to_string(),
        new_scratchpad: "Injecting custom grammar rule: allow unsafe blocks.".to_string(),
    }).unwrap();
    assert_eq!(state.nodes[0].scratchpad, "Injecting custom grammar rule: allow unsafe blocks.");
    assert!(matches!(state.nodes[0].status, CockpitNodeStatus::Steered { .. }));

    // Mid-Flight Steering Action 3: Redirect Tool
    state.apply_steering(SteeringAction::RedirectTool {
        node_id: "n_prover".to_string(),
        new_tool_name: "kani_bounded_model_checker".to_string(),
        parameters: serde_json::json!({"unwind": 10}),
    }).unwrap();
    assert_eq!(state.nodes[1].tool_calls.len(), 1);
    assert!(state.nodes[1].tool_calls[0].contains("kani_bounded_model_checker"));

    // Mid-Flight Steering Action 4: Resume
    state.apply_steering(SteeringAction::Resume { node_id: "n_parse".to_string() }).unwrap();
    assert!(matches!(state.nodes[0].status, CockpitNodeStatus::Running { .. }));

    // Verify Steering History
    assert_eq!(state.steering_history.len(), 4);

    // Test Navigation
    assert_eq!(state.selected_index, 0);
    state.select_next();
    assert_eq!(state.selected_index, 1);
    state.select_next();
    assert_eq!(state.selected_index, 0); // Wraps around
    state.select_prev();
    assert_eq!(state.selected_index, 1);

    // Update Telemetry
    state.update_telemetry(CockpitTelemetry {
        total_tokens: 15_420,
        tokens_per_sec: 142.5,
        peak_temperature_celsius: 58.2,
        battery_pct: Some(92),
        lakandiwa_entropy_bits: 0.32,
        speculative_acceptance_rate: 0.91,
        active_agents: 4,
    });

    // Headless Ratatui Render Test (140 columns x 35 rows)
    let buffer = state.render_headless(140, 35);
    let buffer_str: String = buffer.content.iter().map(|c| c.symbol()).collect();

    assert!(buffer_str.contains("Peak Thermal"), "Buffer must contain Thermal header");
    assert!(buffer_str.contains("Throughput"), "Buffer must contain Throughput header");
    assert!(buffer_str.contains("Lakandiwa Entropy"), "Buffer must contain Entropy header");
    assert!(buffer_str.contains("Draft Acceptance"), "Buffer must contain Acceptance header");
    assert!(buffer_str.contains("AST Code"), "Buffer must render Node 1 name");
    assert!(buffer_str.contains("Z3 SMT"), "Buffer must render Node 2 name");
    assert!(buffer_str.contains("Mid-Flight Steering Controls"), "Buffer must contain controls");
}
