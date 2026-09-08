use crate::dag::graph::WorkflowGraph;
use crate::dag::node::{TaskNode, TaskOutput, TaskStatus};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::types::TokenUsage;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tracing::{debug, error, info, warn};

/// Lifecycle events emitted during DAG workflow execution
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    WorkflowStarted {
        workflow_id: String,
        total_tasks: usize,
    },
    TaskStarted {
        task_id: String,
        task_name: String,
        attempt: usize,
    },
    TaskProgress {
        task_id: String,
        message: String,
    },
    TaskRetry {
        task_id: String,
        attempt: usize,
        max_retries: usize,
        delay: Duration,
        error: String,
    },
    TaskCompleted {
        task_id: String,
        output: TaskOutput,
    },
    TaskFailed {
        task_id: String,
        error: String,
        attempts: usize,
    },
    TaskSkipped {
        task_id: String,
        reason: String,
    },
    WorkflowCompleted {
        workflow_id: String,
        total_tasks: usize,
        completed_tasks: usize,
        failed_tasks: usize,
        total_usage: TokenUsage,
        total_cost_usd: f64,
        total_latency: Duration,
    },
    WorkflowFailed {
        workflow_id: String,
        error: String,
    },
}

/// Comprehensive outcome of a completed DAG workflow run
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub task_outputs: HashMap<String, TaskOutput>,
    pub leaf_outputs: HashMap<String, TaskOutput>,
    pub final_task_id: Option<String>,
    pub final_output: Option<String>,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub total_usage: TokenUsage,
    pub total_cost_usd: f64,
    pub total_latency: Duration,
}

/// Helper to interpolate upstream task outputs into downstream prompt templates
pub fn interpolate_prompt(
    template: &str,
    upstream_outputs: &HashMap<String, TaskOutput>,
) -> String {
    let mut result = template.to_string();
    let mut any_substituted = false;

    for (upstream_id, output) in upstream_outputs {
        let var_output = format!("{{{}.output}}", upstream_id);
        let var_id = format!("{{{}}}", upstream_id);

        if result.contains(&var_output) {
            result = result.replace(&var_output, &output.text);
            any_substituted = true;
        }
        if result.contains(&var_id) {
            result = result.replace(&var_id, &output.text);
            any_substituted = true;
        }
    }

    // If there are upstream outputs and no explicit placeholder was matched, append context block
    if !any_substituted && !upstream_outputs.is_empty() {
        let mut context_block = String::from("\n\n--- Upstream Context ---");
        let mut sorted_keys: Vec<_> = upstream_outputs.keys().collect();
        sorted_keys.sort();
        for upstream_id in sorted_keys {
            if let Some(output) = upstream_outputs.get(upstream_id) {
                context_block.push_str(&format!("\n[{upstream_id}]:\n{}", output.text));
            }
        }
        result.push_str(&context_block);
    }

    result
}

/// Helper to synchronize in-flight execution state into the WorkflowGraph
fn sync_graph_state(
    graph: &mut WorkflowGraph,
    completed: &HashMap<String, TaskOutput>,
    failed: &HashMap<String, String>,
) {
    for (id, output) in completed {
        if let Some(node) = graph.get_task_mut(id) {
            node.status = TaskStatus::Completed;
            node.output = Some(output.clone());
        }
    }
    for (id, _err) in failed {
        if let Some(node) = graph.get_task_mut(id) {
            node.status = TaskStatus::Failed;
        }
    }
}

/// Asynchronous parallel DAG scheduler for Tagisan multi-agent workflows
pub struct DagScheduler {
    pub workflow_id: String,
    pub event_tx: Option<mpsc::UnboundedSender<WorkflowEvent>>,
    pub concurrency_limit: Option<usize>,
}

pub type WorkflowRunner = DagScheduler;

impl Default for DagScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl DagScheduler {
    /// Create a new DAG scheduler
    pub fn new() -> Self {
        Self {
            workflow_id: format!("wf_{}", uuid_simple()),
            event_tx: None,
            concurrency_limit: None,
        }
    }

    /// Set a custom identifier for this workflow execution
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = id.into();
        self
    }

    /// Attach a channel sender to stream real-time workflow lifecycle events
    pub fn with_event_sender(mut self, tx: mpsc::UnboundedSender<WorkflowEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }

    /// Set maximum concurrent tasks allowed to run simultaneously
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = Some(limit);
        self
    }

    fn emit_event(&self, event: WorkflowEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Execute the DAG workflow to completion
    pub async fn run(
        &self,
        graph: &mut WorkflowGraph,
        ctx: &EngineContext,
    ) -> Result<WorkflowResult> {
        let start_time = Instant::now();
        let total_tasks = graph.task_count();

        if total_tasks == 0 {
            return Ok(WorkflowResult {
                workflow_id: self.workflow_id.clone(),
                task_outputs: HashMap::new(),
                leaf_outputs: HashMap::new(),
                final_task_id: None,
                final_output: None,
                completed_tasks: 0,
                failed_tasks: 0,
                total_usage: TokenUsage::default(),
                total_cost_usd: 0.0,
                total_latency: Duration::from_millis(0),
            });
        }

        // 1. Validate DAG and detect cycles
        let topo_order = graph.validate()?;
        let last_topo_id = topo_order.last().cloned();

        self.emit_event(WorkflowEvent::WorkflowStarted {
            workflow_id: self.workflow_id.clone(),
            total_tasks,
        });

        // 2. Build dependency metadata
        let mut downstream_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut upstream_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut remaining_deps: HashMap<String, HashSet<String>> = HashMap::new();
        let mut task_nodes: HashMap<String, TaskNode> = HashMap::new();

        for id in graph.task_ids() {
            let task = graph.get_task(&id).cloned().unwrap();
            let upstreams = graph.upstream_dependencies(&id)?;
            let downstreams = graph.downstream_dependents(&id)?;

            remaining_deps.insert(id.clone(), upstreams.clone().into_iter().collect());
            upstream_map.insert(id.clone(), upstreams);
            downstream_map.insert(id.clone(), downstreams);
            task_nodes.insert(id.clone(), task);
        }

        let completed_outputs = Arc::new(Mutex::new(HashMap::<String, TaskOutput>::new()));
        let failed_tasks = Arc::new(Mutex::new(HashMap::<String, String>::new()));
        let semaphore = self.concurrency_limit.map(|c| Arc::new(Semaphore::new(c)));

        // Channels for scheduling coordination
        let (ready_tx, mut ready_rx) = mpsc::unbounded_channel::<String>();
        let (done_tx, mut done_rx) = mpsc::unbounded_channel::<(String, Result<TaskOutput>)>();

        // Enqueue root tasks (tasks with 0 dependencies)
        for (id, deps) in &remaining_deps {
            if deps.is_empty() {
                let _ = ready_tx.send(id.clone());
            }
        }

        let mut in_flight_count = 0;
        let mut total_completed = 0;

        // Loop until all tasks finish or an unrecoverable failure / cancellation occurs
        while total_completed < total_tasks {
            if ctx.cancellation_token.is_cancelled() {
                let comp = completed_outputs.lock().await.clone();
                let fail = failed_tasks.lock().await.clone();
                sync_graph_state(graph, &comp, &fail);

                self.emit_event(WorkflowEvent::WorkflowFailed {
                    workflow_id: self.workflow_id.clone(),
                    error: "Workflow cancelled".to_string(),
                });
                return Err(TagisanError::Cancelled);
            }

            // Spawn all currently ready tasks
            while let Ok(ready_id) = ready_rx.try_recv() {
                debug!("Dispatching ready task '{}'", ready_id);
                let task_node = task_nodes.get(&ready_id).cloned().unwrap();
                let upstream_ids = upstream_map.get(&ready_id).cloned().unwrap_or_default();
                let event_tx_clone = self.event_tx.clone();
                let done_tx_clone = done_tx.clone();
                let sem_clone = semaphore.clone();
                let completed_outputs_clone = completed_outputs.clone();
                let cancel_token = ctx.cancellation_token.clone();

                let provider_opt = if let Some(ref agent) = task_node.agent {
                    Some(agent.provider.clone())
                } else {
                    ctx.default_provider()
                };

                let budget_tracker_clone = ctx.budget_tracker.clone();

                in_flight_count += 1;

                tokio::spawn(async move {
                    let _permit = match sem_clone {
                        Some(ref sem) => match sem.acquire().await {
                            Ok(p) => Some(p),
                            Err(_) => None,
                        },
                        None => None,
                    };

                    // Guard worker execution against unexpected panics
                    let execution_res = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(
                        execute_task_with_retries(
                            task_node,
                            upstream_ids,
                            provider_opt,
                            budget_tracker_clone,
                            cancel_token,
                            completed_outputs_clone,
                            event_tx_clone,
                        ),
                    ))
                    .await;

                    let final_res = match execution_res {
                        Ok(res) => res,
                        Err(panic_payload) => {
                            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                                s.to_string()
                            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                                s.clone()
                            } else {
                                "Task worker panicked unexpectedly".to_string()
                            };
                            Err(TagisanError::Execution(format!("Panic in task '{ready_id}': {msg}")))
                        }
                    };

                    let _ = done_tx_clone.send((ready_id, final_res));
                });
            }

            // If tasks are running, await the next task completion
            if in_flight_count > 0 {
                tokio::select! {
                    _ = ctx.cancellation_token.cancelled() => {
                        let comp = completed_outputs.lock().await.clone();
                        let fail = failed_tasks.lock().await.clone();
                        sync_graph_state(graph, &comp, &fail);

                        self.emit_event(WorkflowEvent::WorkflowFailed {
                            workflow_id: self.workflow_id.clone(),
                            error: "Workflow cancelled".to_string(),
                        });
                        return Err(TagisanError::Cancelled);
                    }
                    Some((done_id, task_result)) = done_rx.recv() => {
                        in_flight_count -= 1;
                        match task_result {
                            Ok(output) => {
                                total_completed += 1;
                                completed_outputs.lock().await.insert(done_id.clone(), output);

                                // Check downstream tasks that were waiting on this task
                                if let Some(downstreams) = downstream_map.get(&done_id) {
                                    for succ_id in downstreams {
                                        if let Some(deps) = remaining_deps.get_mut(succ_id) {
                                            deps.remove(&done_id);
                                            if deps.is_empty() {
                                                let _ = ready_tx.send(succ_id.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                let err_str = err.to_string();
                                failed_tasks.lock().await.insert(done_id.clone(), err_str.clone());

                                let comp = completed_outputs.lock().await.clone();
                                let fail = failed_tasks.lock().await.clone();
                                sync_graph_state(graph, &comp, &fail);

                                // If a task permanently failed, fail the workflow immediately
                                self.emit_event(WorkflowEvent::WorkflowFailed {
                                    workflow_id: self.workflow_id.clone(),
                                    error: format!("Task '{}' failed: {}", done_id, err_str),
                                });
                                return Err(err);
                            }
                        }
                    }
                }
            } else if ready_rx.is_empty() {
                // No in-flight tasks and no ready tasks, but not all tasks completed
                if total_completed < total_tasks {
                    let comp = completed_outputs.lock().await.clone();
                    let fail = failed_tasks.lock().await.clone();
                    sync_graph_state(graph, &comp, &fail);

                    let err_msg = "Workflow deadlock or unresolvable dependencies detected".to_string();
                    self.emit_event(WorkflowEvent::WorkflowFailed {
                        workflow_id: self.workflow_id.clone(),
                        error: err_msg.clone(),
                    });
                    return Err(TagisanError::Execution(err_msg));
                }
                break;
            }
        }

        // 3. Collect final outputs and update graph node status
        let final_task_outputs = completed_outputs.lock().await.clone();
        let failed_map = failed_tasks.lock().await.clone();
        sync_graph_state(graph, &final_task_outputs, &failed_map);

        let mut total_usage = TokenUsage::default();
        for (_id, output) in &final_task_outputs {
            total_usage.prompt_tokens += output.usage.prompt_tokens;
            total_usage.completion_tokens += output.usage.completion_tokens;
            if let Some(rt) = output.usage.reasoning_tokens {
                total_usage.reasoning_tokens =
                    Some(total_usage.reasoning_tokens.unwrap_or(0) + rt);
            }
        }

        let mut leaf_outputs = HashMap::new();
        for leaf_id in graph.leaf_tasks() {
            if let Some(out) = final_task_outputs.get(&leaf_id) {
                leaf_outputs.insert(leaf_id, out.clone());
            }
        }

        let total_latency = start_time.elapsed();
        let total_cost = ctx.budget_tracker.current_spent_usd();

        let final_task_id = last_topo_id;
        let final_output = final_task_id
            .as_ref()
            .and_then(|id| final_task_outputs.get(id))
            .map(|o| o.text.clone());

        self.emit_event(WorkflowEvent::WorkflowCompleted {
            workflow_id: self.workflow_id.clone(),
            total_tasks,
            completed_tasks: total_completed,
            failed_tasks: failed_map.len(),
            total_usage: total_usage.clone(),
            total_cost_usd: total_cost,
            total_latency,
        });

        info!(
            "Workflow '{}' completed in {:.2}s ({} tasks, ${:.4} USD)",
            self.workflow_id,
            total_latency.as_secs_f32(),
            total_completed,
            total_cost
        );

        Ok(WorkflowResult {
            workflow_id: self.workflow_id.clone(),
            task_outputs: final_task_outputs,
            leaf_outputs,
            final_task_id,
            final_output,
            completed_tasks: total_completed,
            failed_tasks: failed_map.len(),
            total_usage,
            total_cost_usd: total_cost,
            total_latency,
        })
    }
}

/// Internal worker to execute a single task with retries and prompt interpolation
async fn execute_task_with_retries(
    task: TaskNode,
    upstream_ids: Vec<String>,
    default_provider: Option<Arc<dyn crate::providers::LlmProvider>>,
    budget_tracker: Arc<crate::engine::budget::TokenBudgetTracker>,
    cancel_token: tokio_util::sync::CancellationToken,
    completed_outputs: Arc<Mutex<HashMap<String, TaskOutput>>>,
    event_tx: Option<mpsc::UnboundedSender<WorkflowEvent>>,
) -> Result<TaskOutput> {
    let emit = |evt: WorkflowEvent| {
        if let Some(ref tx) = event_tx {
            let _ = tx.send(evt);
        }
    };

    let mut attempt = 1;
    let max_attempts = task.retry_policy.max_retries + 1;

    let outputs_snapshot = completed_outputs.lock().await.clone();
    let mut upstream_outputs = HashMap::new();
    for up_id in &upstream_ids {
        if let Some(out) = outputs_snapshot.get(up_id) {
            upstream_outputs.insert(up_id.clone(), out.clone());
        }
    }
    let interpolated_prompt = interpolate_prompt(&task.prompt_template, &upstream_outputs);

    let mut accumulated_usage = TokenUsage::default();
    let task_start_overall = Instant::now();

    while attempt <= max_attempts {
        if cancel_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        emit(WorkflowEvent::TaskStarted {
            task_id: task.id.clone(),
            task_name: task.name.clone(),
            attempt,
        });

        let attempt_start = Instant::now();

        let mut task_ctx = EngineContext::new(100.0);
        task_ctx.cancellation_token = cancel_token.clone();
        task_ctx.budget_tracker = budget_tracker.clone();

        let run_result = if let Some(ref agent) = task.agent {
            let mut agent_cloned = agent.clone();
            if let Some(ref sys) = task.system_prompt {
                agent_cloned = agent_cloned.with_system_prompt(sys);
            }
            agent_cloned.run(&interpolated_prompt, &task_ctx).await
        } else if let Some(provider) = default_provider.clone() {
            let agent = crate::agent::AutonomousAgent::new(
                provider,
                "default",
                crate::tools::ToolRegistry::new(),
            );
            let mut agent_cloned = agent;
            if let Some(ref sys) = task.system_prompt {
                agent_cloned = agent_cloned.with_system_prompt(sys);
            }
            agent_cloned.run(&interpolated_prompt, &task_ctx).await
        } else {
            Ok(crate::agent::AgentResult {
                final_answer: interpolated_prompt.clone(),
                history: vec![],
                iterations: 1,
                steps: vec![],
                total_usage: TokenUsage::default(),
                total_cost_usd: 0.0,
                total_latency: attempt_start.elapsed(),
            })
        };

        match run_result {
            Ok(agent_res) => {
                accumulated_usage.prompt_tokens += agent_res.total_usage.prompt_tokens;
                accumulated_usage.completion_tokens += agent_res.total_usage.completion_tokens;
                if let Some(rt) = agent_res.total_usage.reasoning_tokens {
                    accumulated_usage.reasoning_tokens =
                        Some(accumulated_usage.reasoning_tokens.unwrap_or(0) + rt);
                }

                let output = TaskOutput {
                    text: agent_res.final_answer,
                    usage: accumulated_usage,
                    latency: task_start_overall.elapsed(),
                };
                emit(WorkflowEvent::TaskCompleted {
                    task_id: task.id.clone(),
                    output: output.clone(),
                });
                return Ok(output);
            }
            Err(err) => {
                if cancel_token.is_cancelled() {
                    return Err(TagisanError::Cancelled);
                }

                if !err.is_retryable() {
                    error!(
                        "Task '{}' encountered non-retryable fatal error: {:?}",
                        task.id, err
                    );
                    emit(WorkflowEvent::TaskFailed {
                        task_id: task.id.clone(),
                        error: err.to_string(),
                        attempts: attempt,
                    });
                    return Err(err);
                }

                if attempt < max_attempts {
                    let delay = task.retry_policy.calculate_delay(attempt);
                    warn!(
                        "Task '{}' attempt {}/{} failed with error: {:?}. Retrying in {:?}",
                        task.id, attempt, max_attempts, err, delay
                    );
                    emit(WorkflowEvent::TaskRetry {
                        task_id: task.id.clone(),
                        attempt,
                        max_retries: task.retry_policy.max_retries,
                        delay,
                        error: err.to_string(),
                    });

                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {},
                        _ = cancel_token.cancelled() => return Err(TagisanError::Cancelled),
                    }
                    attempt += 1;
                } else {
                    error!(
                        "Task '{}' exhausted all {} attempt(s). Error: {:?}",
                        task.id, max_attempts, err
                    );
                    emit(WorkflowEvent::TaskFailed {
                        task_id: task.id.clone(),
                        error: err.to_string(),
                        attempts: attempt,
                    });
                    return Err(err);
                }
            }
        }
    }

    Err(TagisanError::Execution(format!(
        "Task '{}' failed after {} attempts",
        task.id, max_attempts
    )))
}

fn uuid_simple() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1000);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}", id)
}
