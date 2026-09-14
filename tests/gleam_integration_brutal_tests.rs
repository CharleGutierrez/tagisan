//! Brutal Automated Test Suite for Gleam Subsystem Integration in Tagisan.
//!
//! Test 1: Gleam Lexer & Parser parsing complex source (ADTs, pattern matching, custom records, actor specs).
//! Test 2: Type Inference & Unification (detects type mismatches, validates custom types).
//! Test 3: Exhaustive Pattern Matching verification (fails when an ADT variant is missing from a `case` expression).
//! Test 4: Gleam to Erlang Codegen and ETF binary term synthesis.
//! Test 5: Dialectical Debate State Machine (Thesis -> Antithesis -> Synthesis -> Verdict), asserting 0 unhandled states.
//! Test 6: Type-Safe MCP Tool Decoder (validates JSON tool outputs into typed Gleam records).
//! Test 7: Runtime Integration with OTP Supervision (spawning a Gleam actor under OTP supervisor, crashing it with an error, verifying clean restart).
//! Test 8: High-Throughput Concurrency Benchmark (spawning 1,000 Gleam actors, dispatching 10,000 typed messages with 100% type safety and sub-millisecond execution).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tagisan::gleam::ast::*;
use tagisan::gleam::codegen::{ErlangCodeGen, EtfSynthesizer};
use tagisan::gleam::parser::parse_gleam_source;
use tagisan::gleam::runtime::{GleamActor, GleamEvaluator, GleamValue};
use tagisan::gleam::types::{TypeEnvironment, TypeError};
use tagisan::otp::actor::{ActorProcess, ActorRef, GenServer};
use tagisan::otp::etf::Term;
use tagisan::otp::supervisor::{RestartStrategy, Supervisor, SupervisorSpec};

// =============================================================================
// Test 1: Gleam Lexer & Parser Parsing Complex Source
// =============================================================================

#[test]
fn test_1_gleam_lexer_and_parser_complex_source() {
    let source = r#"
import gleam/otp/actor
import gleam/erlang/process.{type Subject}

pub type Msg {
  Ping(reply_to: Subject(String))
  Increment(amount: Int)
  UpdateConfig(key: String, value: String)
  Reset
}

pub type State {
  State(name: String, count: Int, active: Bool)
}

pub fn init() -> State {
  State(name: "alpha_worker", count: 0, active: True)
}

pub fn handle_msg(msg: Msg, state: State) -> actor.Next(Msg, State) {
  case msg {
    Ping(reply_to) -> {
      process.send(reply_to, "pong")
      actor.continue(state)
    }
    Increment(amount) -> {
      let new_count = state.count + amount
      actor.continue(State(name: state.name, count: new_count, active: state.active))
    }
    UpdateConfig(key, value) -> {
      actor.continue(state)
    }
    Reset -> {
      actor.continue(State(name: state.name, count: 0, active: True))
    }
  }
}
"#;

    let module = parse_gleam_source(source, "complex_agent")
        .expect("Failed to parse complex Gleam source");

    assert_eq!(module.name, "complex_agent");
    assert_eq!(module.imports.len(), 2);
    assert_eq!(module.imports[0].module, "gleam/otp/actor");
    assert_eq!(module.imports[1].module, "gleam/erlang/process");
    assert_eq!(module.imports[1].unqualified, vec!["type Subject"]);

    // Check types
    assert_eq!(module.types.len(), 2);
    let msg_type = module.find_type("Msg").expect("Msg type definition not found");
    assert_eq!(msg_type.constructors.len(), 4);
    assert_eq!(msg_type.constructors[0].name, "Ping");
    assert_eq!(msg_type.constructors[0].fields.len(), 1);
    assert_eq!(msg_type.constructors[0].fields[0].label, Some("reply_to".to_string()));
    assert_eq!(msg_type.constructors[1].name, "Increment");
    assert_eq!(msg_type.constructors[1].fields.len(), 1);
    assert_eq!(msg_type.constructors[2].name, "UpdateConfig");
    assert_eq!(msg_type.constructors[2].fields.len(), 2);
    assert_eq!(msg_type.constructors[3].name, "Reset");
    assert_eq!(msg_type.constructors[3].fields.len(), 0);

    let state_type = module.find_type("State").expect("State type definition not found");
    assert_eq!(state_type.constructors.len(), 1);
    assert_eq!(state_type.constructors[0].fields.len(), 3);

    // Check functions
    assert_eq!(module.functions.len(), 2);
    let init_fn = module.find_function("init").expect("init function not found");
    assert_eq!(init_fn.parameters.len(), 0);
    assert_eq!(init_fn.return_type, Some(GleamType::custom("State", Vec::new())));

    let handle_fn = module.find_function("handle_msg").expect("handle_msg function not found");
    assert_eq!(handle_fn.parameters.len(), 2);
    assert_eq!(handle_fn.parameters[0].name, "msg");
    assert_eq!(handle_fn.parameters[1].name, "state");
}

// =============================================================================
// Test 2: Type Inference & Unification
// =============================================================================

#[test]
fn test_2_type_inference_and_unification() {
    let mut env = TypeEnvironment::new();

    // 1. Primitive unification
    let int_res = env.unify(&GleamType::Int, &GleamType::Int);
    assert_eq!(int_res, Ok(GleamType::Int));

    let str_res = env.unify(&GleamType::String, &GleamType::String);
    assert_eq!(str_res, Ok(GleamType::String));

    // 2. Type mismatch detection
    let mismatch_res = env.unify(&GleamType::Int, &GleamType::String);
    assert!(mismatch_res.is_err());
    match mismatch_res.unwrap_err() {
        TypeError::Mismatch { expected, got } => {
            assert_eq!(expected, "Int");
            assert_eq!(got, "String");
        }
        other => panic!("Expected Mismatch error, got {:?}", other),
    }

    // 3. Generic variable binding & propagation
    let generic_a = GleamType::Generic("a".to_string());
    let unified_generic = env.unify(&generic_a, &GleamType::Int).expect("Unification failed");
    assert_eq!(unified_generic, GleamType::Int);

    // Apply substitution on another occurrence of 'a'
    let resolved = env.apply_substitutions(&generic_a);
    assert_eq!(resolved, GleamType::Int);

    // 4. Custom Result type unification
    let custom_res1 = GleamType::result(GleamType::Generic("item".to_string()), GleamType::String);
    let custom_res2 = GleamType::result(GleamType::Bool, GleamType::String);
    let unified_result = env.unify(&custom_res1, &custom_res2).expect("Result unification failed");
    assert_eq!(
        unified_result,
        GleamType::result(GleamType::Bool, GleamType::String)
    );

    // 5. Tuple and List unification
    let tuple_1 = GleamType::tuple(vec![GleamType::Int, GleamType::String]);
    let tuple_2 = GleamType::tuple(vec![GleamType::Int, GleamType::String]);
    assert_eq!(env.unify(&tuple_1, &tuple_2), Ok(tuple_1.clone()));

    let tuple_mismatch = env.unify(
        &tuple_1,
        &GleamType::tuple(vec![GleamType::Int, GleamType::Bool]),
    );
    assert!(tuple_mismatch.is_err());
}

// =============================================================================
// Test 3: Exhaustive Pattern Matching Verification
// =============================================================================

#[test]
fn test_3_exhaustive_pattern_matching_verification() {
    let mut env = TypeEnvironment::new();

    // Register custom ADT: DebatePhase with 4 variants
    let debate_phase = TypeDefinition::new(
        "DebatePhase",
        vec![
            Constructor::unit("ThesisPhase"),
            Constructor::unit("AntithesisPhase"),
            Constructor::unit("SynthesisPhase"),
            Constructor::unit("VerdictPhase"),
        ],
    );
    env.register_type(debate_phase);

    let phase_type = GleamType::custom("DebatePhase", Vec::new());

    // Case 1: Incomplete pattern match (missing 'VerdictPhase')
    let non_exhaustive_clauses = vec![
        CaseClause {
            pattern: Pattern::Constructor {
                name: "ThesisPhase".to_string(),
                module: None,
                args: Vec::new(),
            },
            guard: None,
            body: Expression::int(1),
        },
        CaseClause {
            pattern: Pattern::Constructor {
                name: "AntithesisPhase".to_string(),
                module: None,
                args: Vec::new(),
            },
            guard: None,
            body: Expression::int(2),
        },
        CaseClause {
            pattern: Pattern::Constructor {
                name: "SynthesisPhase".to_string(),
                module: None,
                args: Vec::new(),
            },
            guard: None,
            body: Expression::int(3),
        },
    ];

    let check_res = env.check_pattern_exhaustiveness(&phase_type, &non_exhaustive_clauses);
    assert!(check_res.is_err(), "Expected non-exhaustive pattern match error");
    match check_res.unwrap_err() {
        TypeError::NonExhaustivePatternMatch { missing } => {
            assert_eq!(missing, vec!["VerdictPhase".to_string()]);
        }
        other => panic!("Expected NonExhaustivePatternMatch, got {:?}", other),
    }

    // Case 2: Exhaustive pattern match (all 4 variants present)
    let mut exhaustive_clauses = non_exhaustive_clauses.clone();
    exhaustive_clauses.push(CaseClause {
        pattern: Pattern::Constructor {
            name: "VerdictPhase".to_string(),
            module: None,
            args: Vec::new(),
        },
        guard: None,
        body: Expression::int(4),
    });

    let ok_res = env.check_pattern_exhaustiveness(&phase_type, &exhaustive_clauses);
    assert!(ok_res.is_ok(), "Exhaustive clauses should pass without error");

    // Case 3: Wildcard discard `_` makes partial match exhaustive
    let wildcard_clauses = vec![
        CaseClause {
            pattern: Pattern::Constructor {
                name: "ThesisPhase".to_string(),
                module: None,
                args: Vec::new(),
            },
            guard: None,
            body: Expression::int(1),
        },
        CaseClause {
            pattern: Pattern::Discard(None),
            guard: None,
            body: Expression::int(99),
        },
    ];
    let wildcard_res = env.check_pattern_exhaustiveness(&phase_type, &wildcard_clauses);
    assert!(wildcard_res.is_ok(), "Wildcard clause must satisfy exhaustiveness");
}

// =============================================================================
// Test 4: Gleam to Erlang Codegen and ETF Binary Term Synthesis
// =============================================================================

#[test]
fn test_4_gleam_to_erlang_codegen_and_etf_synthesis() {
    let source = r#"
pub type Msg {
  Ping
  Compute(val: Int)
}

pub fn handle_msg(msg: Msg) -> Int {
  case msg {
    Ping -> 100
    Compute(val) -> val * 2
  }
}
"#;

    let module = parse_gleam_source(source, "calc_service").unwrap();
    let codegen = ErlangCodeGen::new(module);
    let erlang_code = codegen.compile().expect("Failed to compile Gleam to Erlang");

    // Assert Erlang source code properties
    assert!(erlang_code.contains("-module(calc_service)."));
    assert!(erlang_code.contains("-export([handle_msg/1])."));
    assert!(erlang_code.contains("handle_msg(Msg) ->"));
    assert!(erlang_code.contains("case Msg of"));
    assert!(erlang_code.contains("ping ->"));
    assert!(erlang_code.contains("{compute, Val} ->"));

    // Synthesize binary ETF terms
    let ping_term = EtfSynthesizer::synthesize_constructor("Ping", &[]);
    assert_eq!(ping_term, Term::atom("ping"));

    let compute_term = EtfSynthesizer::synthesize_constructor("Compute", &[Term::int(42)]);
    assert_eq!(
        compute_term,
        Term::tuple(vec![Term::atom("compute"), Term::int(42)])
    );

    // ETF Binary serialization roundtrip
    let binary_etf = EtfSynthesizer::encode(&compute_term);
    assert_eq!(binary_etf[0], 131, "First byte must be ETF magic version 131");

    let decoded_term = EtfSynthesizer::decode(&binary_etf).expect("Failed to decode binary ETF");
    assert_eq!(decoded_term, compute_term);
}

// =============================================================================
// Test 5: Dialectical Debate State Machine
// =============================================================================

#[test]
fn test_5_dialectical_debate_state_machine() {
    let source = std::fs::read_to_string("sdk/gleam/tagisan_gleam/src/tagisan/debate.gleam")
        .expect("Failed to read debate.gleam from SDK");

    let module = parse_gleam_source(&source, "debate").expect("Failed to parse debate.gleam");
    let mut env = TypeEnvironment::new();
    env.check_module(&module).expect("Type check debate.gleam failed");

    // Initialize Thesis
    let thesis = GleamValue::Constructor {
        name: "Thesis".to_string(),
        fields: vec![
            GleamValue::String("alpha_proposer".to_string()),
            GleamValue::String("Actors ensure fault-isolated state".to_string()),
            GleamValue::String("Zero ambient shared mutable memory".to_string()),
        ],
    };

    // Step 1: Audit transition -> Antithesis
    let audit_action = GleamValue::Constructor {
        name: "Audit".to_string(),
        fields: vec![
            GleamValue::String("security_auditor".to_string()),
            GleamValue::String("Mailbox overflow can cause DoS".to_string()),
            GleamValue::Float(0.35),
        ],
    };

    let mut vars = std::collections::HashMap::new();
    vars.insert("state".to_string(), thesis);
    vars.insert("action".to_string(), audit_action);

    let transition_call = Expression::FunctionCall {
        function: Box::new(Expression::Variable("transition".to_string())),
        arguments: vec![
            CallArgument::positional(Expression::Variable("state".to_string())),
            CallArgument::positional(Expression::Variable("action".to_string())),
        ],
    };

    let step1_res = GleamEvaluator::eval_expression(&transition_call, &mut vars, &module)
        .expect("Failed to evaluate step 1");

    // Must be Ok(Antithesis(...))
    match step1_res {
        GleamValue::Constructor { ref name, ref fields } if name == "Ok" => {
            let inner = &fields[0];
            match inner {
                GleamValue::Constructor { ref name, .. } => {
                    assert_eq!(name, "Antithesis");
                }
                _ => panic!("Expected Antithesis inside Ok"),
            }
        }
        other => panic!("Expected Ok(Antithesis), got {:?}", other),
    }

    // Step 2: Synthesis transition
    let antithesis = match step1_res {
        GleamValue::Constructor { ref fields, .. } => fields[0].clone(),
        _ => unreachable!(),
    };

    let synth_action = GleamValue::Constructor {
        name: "Synthesize".to_string(),
        fields: vec![
            GleamValue::String("chief_judge".to_string()),
            GleamValue::String("Bounded mailboxes with backpressure".to_string()),
            GleamValue::Int(95),
        ],
    };

    vars.insert("state".to_string(), antithesis);
    vars.insert("action".to_string(), synth_action);

    let step2_res = GleamEvaluator::eval_expression(&transition_call, &mut vars, &module)
        .expect("Failed to evaluate step 2");

    let synthesis = match step2_res {
        GleamValue::Constructor { ref name, ref fields } if name == "Ok" => {
            assert_eq!(fields[0], GleamValue::Constructor {
                name: "Synthesis".to_string(),
                fields: match &fields[0] {
                    GleamValue::Constructor { fields, .. } => fields.clone(),
                    _ => unreachable!(),
                }
            });
            fields[0].clone()
        }
        other => panic!("Expected Ok(Synthesis), got {:?}", other),
    };

    // Step 3: Adjudicate transition -> Verdict
    let adjudicate_action = GleamValue::Constructor {
        name: "Adjudicate".to_string(),
        fields: vec![
            GleamValue::Bool(true),
            GleamValue::String("Formally approved and verified".to_string()),
            GleamValue::Int(1726300000),
        ],
    };

    vars.insert("state".to_string(), synthesis);
    vars.insert("action".to_string(), adjudicate_action);

    let step3_res = GleamEvaluator::eval_expression(&transition_call, &mut vars, &module)
        .expect("Failed to evaluate step 3");

    match step3_res {
        GleamValue::Constructor { ref name, ref fields } if name == "Ok" => {
            match &fields[0] {
                GleamValue::Constructor { name: v_name, fields: v_fields } => {
                    assert_eq!(v_name, "Verdict");
                    assert_eq!(v_fields[0], GleamValue::Bool(true));
                }
                _ => panic!("Expected Verdict inside Ok"),
            }
        }
        other => panic!("Expected Ok(Verdict), got {:?}", other),
    }
}

// =============================================================================
// Test 6: Type-Safe MCP Tool Decoder
// =============================================================================

#[test]
fn test_6_type_safe_mcp_tool_decoder() {
    let source = std::fs::read_to_string("sdk/gleam/tagisan_gleam/src/tagisan/mcp.gleam")
        .expect("Failed to read mcp.gleam from SDK");

    let module = parse_gleam_source(&source, "mcp").expect("Failed to parse mcp.gleam");
    let mut env = TypeEnvironment::new();
    env.check_module(&module).expect("Type check mcp.gleam failed");

    // 1. Test decode_tool_call with valid tool
    let mut vars = std::collections::HashMap::new();
    vars.insert("tool_name".to_string(), GleamValue::String("run_command".to_string()));
    vars.insert("args_json".to_string(), GleamValue::String("{\"command\": \"cargo build\"}".to_string()));

    let decode_call = Expression::FunctionCall {
        function: Box::new(Expression::Variable("decode_tool_call".to_string())),
        arguments: vec![
            CallArgument::positional(Expression::Variable("tool_name".to_string())),
            CallArgument::positional(Expression::Variable("args_json".to_string())),
        ],
    };

    let result = GleamEvaluator::eval_expression(&decode_call, &mut vars, &module)
        .expect("Failed to evaluate decode_tool_call");

    match result {
        GleamValue::Constructor { name, fields } => {
            assert_eq!(name, "Ok");
            let tool_call = &fields[0];
            match tool_call {
                GleamValue::Constructor { name, fields } => {
                    assert_eq!(name, "ToolCall");
                    assert_eq!(fields[0], GleamValue::String("run_command".to_string()));
                    assert_eq!(fields[1], GleamValue::String("{\"command\": \"cargo build\"}".to_string()));
                }
                _ => panic!("Expected ToolCall inside Ok"),
            }
        }
        _ => panic!("Expected Ok(ToolCall)"),
    }

    // 2. Test decode_tool_call with empty tool name (fails with Error)
    vars.insert("tool_name".to_string(), GleamValue::String("".to_string()));
    let err_result = GleamEvaluator::eval_expression(&decode_call, &mut vars, &module)
        .expect("Failed to evaluate decode_tool_call with empty name");

    match err_result {
        GleamValue::Constructor { name, .. } => {
            assert_eq!(name, "Error");
        }
        _ => panic!("Expected Error constructor for empty tool name"),
    }
}

// =============================================================================
// Test 7: Runtime Integration with OTP Supervision
// =============================================================================

#[tokio::test]
async fn test_7_runtime_integration_with_otp_supervision() {
    let source = r#"
pub type State {
  State(count: Int)
}

pub type Msg {
  Ping
  Add(n: Int)
  Crash
}

pub fn init() -> State {
  State(count: 0)
}

pub fn handle_msg(msg: Msg, state: State) -> State {
  case msg {
    Ping -> state
    Add(n) -> State(count: state.count + n)
    Crash -> panic
  }
}
"#;

    let initial_state = GleamValue::Constructor {
        name: "State".to_string(),
        fields: vec![GleamValue::Int(0)],
    };

    let actor = GleamActor::from_source("supervised_actor", source, initial_state.clone())
        .expect("Failed to build GleamActor from source");

    assert_eq!(actor.msg_type_name, "Msg");

    let source_clone = source.to_string();
    let initial_clone = initial_state.clone();

    // Wrap in OTP ChildSpec
    let spec = GleamActor::child_spec("supervised_actor", move || {
        GleamActor::from_source("supervised_actor", &source_clone, initial_clone.clone()).unwrap()
    });

    // Start under OTP Supervisor with OneForOne strategy
    let sup_spec = SupervisorSpec::new("gleam_supervisor", RestartStrategy::OneForOne)
        .max_restarts(5, 10)
        .add_child(spec);

    let supervisor = Supervisor::start(sup_spec)
        .await
        .expect("Failed to start supervisor");

    // 1. Verify child is alive and responds to calls
    let child_ref = supervisor.get_child("supervised_actor").await.expect("Child not found");
    assert!(child_ref.is_alive());

    // Send Add(10)
    let add_msg = Term::tuple(vec![Term::atom("add"), Term::int(10)]);
    let reply = child_ref
        .call(add_msg, Duration::from_secs(1))
        .await
        .expect("Call Add(10) failed");

    // State updated to 10
    match reply {
        Term::Tuple(elems) => {
            assert_eq!(elems[0], Term::atom("state"));
            assert_eq!(elems[1], Term::int(10));
        }
        other => panic!("Expected state tuple, got {:?}", other),
    }

    // 2. Trigger intentional crash by sending 'Crash' variant
    let crash_msg = Term::atom("crash");
    let _ = child_ref.call(crash_msg, Duration::from_millis(200)).await;

    // Allow supervisor to intercept crash and restart child
    tokio::time::sleep(Duration::from_millis(150)).await;

    // 3. Verify clean restart by OTP supervisor
    let children_info = supervisor.which_children().await.expect("which_children failed");
    assert_eq!(children_info.len(), 1);
    let child_info = &children_info[0];

    assert_eq!(child_info.id, "supervised_actor");
    assert!(child_info.is_alive, "Crashed actor must be restarted and alive");
    assert_eq!(child_info.restart_count, 1, "Restart count must be exactly 1");

    // 4. Verify resurrected actor is fully operational with reset initial state
    let resurrected_actor = supervisor.get_child("supervised_actor").await.expect("Child not found");
    let ping_msg = Term::atom("ping");
    let ping_reply = resurrected_actor
        .call(ping_msg, Duration::from_secs(1))
        .await
        .expect("Ping call failed on restarted actor");

    match ping_reply {
        Term::Tuple(elems) => {
            assert_eq!(elems[0], Term::atom("state"));
            assert_eq!(elems[1], Term::int(0), "Restarted actor state was cleanly re-initialized to 0");
        }
        other => panic!("Expected reset state tuple, got {:?}", other),
    }

    supervisor.terminate().await.expect("Failed to terminate supervisor");
}

// =============================================================================
// Test 8: High-Throughput Concurrency Benchmark
// =============================================================================

#[tokio::test]
async fn test_8_high_throughput_concurrency_benchmark() {
    const NUM_ACTORS: usize = 1_000;
    const TOTAL_MESSAGES: usize = 10_000;

    let source = r#"
pub type CounterState {
  CounterState(val: Int)
}

pub type Msg {
  Tick(amount: Int)
  Query
}

pub fn handle_msg(msg: Msg, state: CounterState) -> CounterState {
  case msg {
    Tick(n) -> CounterState(val: state.val + n)
    Query -> state
  }
}
"#;

    let total_received = Arc::new(AtomicUsize::new(0));

    // Actor worker wrapping message counting
    struct BenchActor {
        inner: GleamActor,
        counter: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl GenServer for BenchActor {
        async fn handle_cast(&mut self, msg: Term) -> Result<(), tagisan::otp::actor::ActorError> {
            let _ = self.inner.handle_cast(msg).await?;
            self.counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    println!("\nSpawning {} concurrent Gleam actors...", NUM_ACTORS);
    let start_spawn = Instant::now();
    let mut actor_handles: Vec<ActorRef> = Vec::with_capacity(NUM_ACTORS);

    for i in 0..NUM_ACTORS {
        let initial_state = GleamValue::Constructor {
            name: "CounterState".to_string(),
            fields: vec![GleamValue::Int(0)],
        };
        let actor = GleamActor::from_source(format!("actor_{}", i), source, initial_state).unwrap();
        let bench_actor = BenchActor {
            inner: actor,
            counter: total_received.clone(),
        };
        let (actor_ref, _) = ActorProcess::spawn(bench_actor);
        actor_handles.push(actor_ref);
    }

    let spawn_latency = start_spawn.elapsed();
    println!("Spawned {} actors in {:.2?} ({:.0} actors/sec)",
        NUM_ACTORS,
        spawn_latency,
        NUM_ACTORS as f64 / spawn_latency.as_secs_f64()
    );

    // Dispatch 10,000 typed messages across actors
    println!("Dispatching {} typed ETF messages...", TOTAL_MESSAGES);
    let start_dispatch = Instant::now();

    for i in 0..TOTAL_MESSAGES {
        let target = &actor_handles[i % NUM_ACTORS];
        let msg = Term::tuple(vec![Term::atom("tick"), Term::int(1)]);
        target.cast(msg).await.expect("Cast failed");
    }

    let dispatch_duration = start_dispatch.elapsed();
    let avg_dispatch_micros = dispatch_duration.as_micros() as f64 / TOTAL_MESSAGES as f64;
    println!("Dispatched {} messages in {:.2?} (avg {:.2} µs/msg, throughput: {:.0} msgs/sec)",
        TOTAL_MESSAGES,
        dispatch_duration,
        avg_dispatch_micros,
        TOTAL_MESSAGES as f64 / dispatch_duration.as_secs_f64()
    );

    // Wait for all messages to be processed
    for _ in 0..100 {
        if total_received.load(Ordering::Relaxed) >= TOTAL_MESSAGES {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    let final_processed = total_received.load(Ordering::Relaxed);
    println!("Total processed: {}/{} messages", final_processed, TOTAL_MESSAGES);
    assert_eq!(
        final_processed, TOTAL_MESSAGES,
        "100% of messages must be processed with zero drops"
    );

    // Assert sub-millisecond dispatch
    assert!(
        avg_dispatch_micros < 1000.0,
        "Average dispatch latency must be strictly sub-millisecond (< 1,000 µs), was {:.2} µs",
        avg_dispatch_micros
    );
}
