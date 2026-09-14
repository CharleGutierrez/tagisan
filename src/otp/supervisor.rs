//! True OTP Supervision Trees in Asynchronous Rust.
//!
//! Implements BEAM/OTP supervisor semantics:
//! - Strategies: OneForOne, OneForAll, RestForOne
//! - Restart intensity & period: max_restarts within max_seconds window (crash loop detection)
//! - Child specifications: Permanent, Transient, Temporary
//! - "Let it crash" semantics with dependency-ordered restarts

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::otp::actor::{ActorError, ActorPid, ActorProcess, ActorRef, GenServer, ProcessExit};

/// Supervisor restart strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartStrategy {
    /// If a child process crashes, only that child process is restarted.
    OneForOne,
    /// If a child process crashes, all other child processes are stopped,
    /// and then all child processes (including the crashed one) are restarted.
    OneForAll,
    /// If a child process crashes, any child processes that were started
    /// after it (in child spec list order) are stopped, and then the crashed
    /// child and those following children are restarted.
    RestForOne,
}

/// Child restart type policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartType {
    /// Always restarted, whether termination was normal or abnormal.
    Permanent,
    /// Restarted only if terminated abnormally (crashed or non-zero/error exit).
    Transient,
    /// Never restarted, regardless of the exit reason.
    Temporary,
}

/// Supervisor errors
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SupervisorError {
    #[error("Maximum restart intensity reached ({restarts} restarts in {seconds}s). Supervisor shutting down.")]
    MaxRestartIntensityReached { restarts: usize, seconds: u64 },
    #[error("Child not found with ID: {0}")]
    ChildNotFound(String),
    #[error("Child start failed: {0}")]
    ChildStartFailed(String),
    #[error("Supervisor is shutting down or dead")]
    SupervisorDead,
    #[error("Child already exists with ID: {0}")]
    ChildAlreadyExists(String),
}

/// Factory closure for creating fresh child actor instances
pub type ChildFactory = Arc<dyn Fn() -> Box<dyn GenServer> + Send + Sync>;

/// Child process specification
#[derive(Clone)]
pub struct ChildSpec {
    pub id: String,
    pub start: ChildFactory,
    pub restart_type: RestartType,
    pub shutdown_timeout: Duration,
}

impl ChildSpec {
    pub fn new<F, G>(id: impl Into<String>, factory: F) -> Self
    where
        F: Fn() -> G + Send + Sync + 'static,
        G: GenServer + 'static,
    {
        Self {
            id: id.into(),
            start: Arc::new(move || Box::new(factory())),
            restart_type: RestartType::Permanent,
            shutdown_timeout: Duration::from_millis(500),
        }
    }

    pub fn restart(mut self, restart_type: RestartType) -> Self {
        self.restart_type = restart_type;
        self
    }

    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }
}

/// Complete specification for starting a Supervisor
#[derive(Clone)]
pub struct SupervisorSpec {
    pub name: String,
    pub strategy: RestartStrategy,
    pub max_restarts: usize,
    pub max_seconds: u64,
    pub children: Vec<ChildSpec>,
}

impl SupervisorSpec {
    pub fn new(name: impl Into<String>, strategy: RestartStrategy) -> Self {
        Self {
            name: name.into(),
            strategy,
            max_restarts: 3,
            max_seconds: 5,
            children: Vec::new(),
        }
    }

    pub fn max_restarts(mut self, max_restarts: usize, max_seconds: u64) -> Self {
        self.max_restarts = max_restarts;
        self.max_seconds = max_seconds;
        self
    }

    pub fn add_child(mut self, child: ChildSpec) -> Self {
        self.children.push(child);
        self
    }
}

/// Public child status info
#[derive(Debug, Clone)]
pub struct ChildInfo {
    pub id: String,
    pub pid: Option<ActorPid>,
    pub is_alive: bool,
    pub restart_count: usize,
}

impl fmt::Display for ChildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pid_str = self
            .pid
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "none".to_string());
        write!(
            f,
            "Child(id='{}', pid={}, alive={}, restarts={})",
            self.id, pid_str, self.is_alive, self.restart_count
        )
    }
}

// Internal child tracker
struct SupervisedChild {
    spec: ChildSpec,
    actor_ref: Option<ActorRef>,
    join_handle: Option<JoinHandle<ProcessExit>>,
    restart_count: usize,
}

impl SupervisedChild {
    async fn stop(&mut self) {
        if let Some(actor) = &self.actor_ref {
            let _ = actor.stop("supervisor_shutdown").await;
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = tokio::time::timeout(self.spec.shutdown_timeout, handle).await;
        }
        self.actor_ref = None;
    }

    fn is_alive(&self) -> bool {
        self.actor_ref
            .as_ref()
            .map(|a| a.is_alive())
            .unwrap_or(false)
    }
}

// Internal messages to supervisor event loop
enum SupervisorCmd {
    WhichChildren(oneshot::Sender<Vec<ChildInfo>>),
    GetChild(String, oneshot::Sender<Option<ActorRef>>),
    RestartChild(String, oneshot::Sender<Result<(), SupervisorError>>),
    StopChild(String, oneshot::Sender<Result<(), SupervisorError>>),
    Terminate(oneshot::Sender<()>),
}

struct ChildExitEvent {
    id: String,
    pid: ActorPid,
    exit: ProcessExit,
}

/// Supervisor controller handle
pub struct SupervisorHandle {
    pub name: String,
    cmd_tx: mpsc::Sender<SupervisorCmd>,
    is_alive: Arc<AtomicBool>,
}

impl SupervisorHandle {
    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst) && !self.cmd_tx.is_closed()
    }

    pub async fn which_children(&self) -> Result<Vec<ChildInfo>, SupervisorError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(SupervisorCmd::WhichChildren(tx))
            .await
            .map_err(|_| SupervisorError::SupervisorDead)?;
        rx.await.map_err(|_| SupervisorError::SupervisorDead)
    }

    pub async fn get_child(&self, id: &str) -> Option<ActorRef> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(SupervisorCmd::GetChild(id.to_string(), tx))
            .await
            .ok()?;
        rx.await.ok().flatten()
    }

    pub async fn restart_child(&self, id: &str) -> Result<(), SupervisorError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(SupervisorCmd::RestartChild(id.to_string(), tx))
            .await
            .map_err(|_| SupervisorError::SupervisorDead)?;
        rx.await.map_err(|_| SupervisorError::SupervisorDead)?
    }

    pub async fn stop_child(&self, id: &str) -> Result<(), SupervisorError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(SupervisorCmd::StopChild(id.to_string(), tx))
            .await
            .map_err(|_| SupervisorError::SupervisorDead)?;
        rx.await.map_err(|_| SupervisorError::SupervisorDead)?
    }

    pub async fn terminate(&self) -> Result<(), SupervisorError> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(SupervisorCmd::Terminate(tx))
            .await
            .map_err(|_| SupervisorError::SupervisorDead)?;
        let _ = rx.await;
        self.is_alive.store(false, Ordering::SeqCst);
        Ok(())
    }
}

/// Supervisor implementation
pub struct Supervisor;

impl Supervisor {
    /// Start a supervised actor hierarchy
    pub async fn start(spec: SupervisorSpec) -> Result<Arc<SupervisorHandle>, SupervisorError> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<SupervisorCmd>(64);
        let (exit_tx, mut exit_rx) = mpsc::channel::<ChildExitEvent>(128);
        let is_alive = Arc::new(AtomicBool::new(true));

        let mut children: Vec<SupervisedChild> = Vec::with_capacity(spec.children.len());

        // Initial forward-order startup
        for child_spec in spec.children.clone() {
            let mut child = SupervisedChild {
                spec: child_spec,
                actor_ref: None,
                join_handle: None,
                restart_count: 0,
            };
            Self::spawn_child_actor(&mut child, &exit_tx);
            children.push(child);
        }

        let is_alive_clone = is_alive.clone();
        let spec_clone = spec.clone();

        tokio::spawn(async move {
            let mut restart_window: VecDeque<Instant> = VecDeque::new();
            let mut manual_stopped_ids: Vec<String> = Vec::new();

            loop {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            SupervisorCmd::WhichChildren(reply) => {
                                let info = children
                                    .iter()
                                    .map(|c| ChildInfo {
                                        id: c.spec.id.clone(),
                                        pid: c.actor_ref.as_ref().map(|a| a.pid().clone()),
                                        is_alive: c.is_alive(),
                                        restart_count: c.restart_count,
                                    })
                                    .collect();
                                let _ = reply.send(info);
                            }
                            SupervisorCmd::GetChild(id, reply) => {
                                let actor = children
                                    .iter()
                                    .find(|c| c.spec.id == id && c.is_alive())
                                    .and_then(|c| c.actor_ref.clone());
                                let _ = reply.send(actor);
                            }
                            SupervisorCmd::RestartChild(id, reply) => {
                                if let Some(pos) = children.iter().position(|c| c.spec.id == id) {
                                    manual_stopped_ids.retain(|x| x != &id);
                                    let child = &mut children[pos];
                                    child.stop().await;
                                    Self::spawn_child_actor(child, &exit_tx);
                                    child.restart_count += 1;
                                    let _ = reply.send(Ok(()));
                                } else {
                                    let _ = reply.send(Err(SupervisorError::ChildNotFound(id)));
                                }
                            }
                            SupervisorCmd::StopChild(id, reply) => {
                                if let Some(child) = children.iter_mut().find(|c| c.spec.id == id) {
                                    if !manual_stopped_ids.contains(&id) {
                                        manual_stopped_ids.push(id.clone());
                                    }
                                    child.stop().await;
                                    let _ = reply.send(Ok(()));
                                } else {
                                    let _ = reply.send(Err(SupervisorError::ChildNotFound(id)));
                                }
                            }
                            SupervisorCmd::Terminate(reply) => {
                                for child in children.iter_mut().rev() {
                                    child.stop().await;
                                }
                                let _ = reply.send(());
                                break;
                            }
                        }
                    }

                    Some(ChildExitEvent { id, pid, exit }) = exit_rx.recv() => {
                        // If manually stopped via StopChild, ignore exit
                        if manual_stopped_ids.contains(&id) {
                            continue;
                        }

                        let child_idx = match children.iter().position(|c| c.spec.id == id) {
                            Some(idx) => idx,
                            None => continue,
                        };

                        // Check if this exit matches the child's current PID.
                        // Stale exits from previously stopped generations are discarded!
                        let is_current = children[child_idx]
                            .actor_ref
                            .as_ref()
                            .map(|a| a.pid() == &pid)
                            .unwrap_or(false);
                        if !is_current {
                            continue;
                        }

                        let should_restart = match children[child_idx].spec.restart_type {
                            RestartType::Permanent => true,
                            RestartType::Transient => exit.is_crash(),
                            RestartType::Temporary => false,
                        };

                        if !should_restart {
                            children[child_idx].actor_ref = None;
                            children[child_idx].join_handle = None;
                            continue;
                        }

                        // Check restart intensity & crash loop window
                        let now = Instant::now();
                        let max_duration = Duration::from_secs(spec_clone.max_seconds);
                        while let Some(&t) = restart_window.front() {
                            if now.duration_since(t) > max_duration {
                                restart_window.pop_front();
                            } else {
                                break;
                            }
                        }
                        restart_window.push_back(now);

                        if restart_window.len() > spec_clone.max_restarts {
                            // Crash loop detected! Stop all children and terminate supervisor
                            for c in children.iter_mut().rev() {
                                c.stop().await;
                            }
                            is_alive_clone.store(false, Ordering::SeqCst);
                            break;
                        }

                        // Apply supervision strategy
                        match spec_clone.strategy {
                            RestartStrategy::OneForOne => {
                                let child = &mut children[child_idx];
                                child.stop().await;
                                Self::spawn_child_actor(child, &exit_tx);
                                child.restart_count += 1;
                            }
                            RestartStrategy::OneForAll => {
                                // Stop all children in reverse order
                                for c in children.iter_mut().rev() {
                                    c.stop().await;
                                }
                                // Restart all children in forward order
                                for c in children.iter_mut() {
                                    Self::spawn_child_actor(c, &exit_tx);
                                    c.restart_count += 1;
                                }
                            }
                            RestartStrategy::RestForOne => {
                                let total = children.len();
                                // Stop subsequent children in reverse order down to child_idx
                                for i in (child_idx..total).rev() {
                                    children[i].stop().await;
                                }
                                // Restart crashed child and subsequent children in forward order
                                for i in child_idx..total {
                                    let c = &mut children[i];
                                    Self::spawn_child_actor(c, &exit_tx);
                                    c.restart_count += 1;
                                }
                            }
                        }
                    }

                    else => break,
                }
            }

            is_alive_clone.store(false, Ordering::SeqCst);
        });

        Ok(Arc::new(SupervisorHandle {
            name: spec.name,
            cmd_tx,
            is_alive,
        }))
    }

    fn spawn_child_actor(child: &mut SupervisedChild, exit_tx: &mpsc::Sender<ChildExitEvent>) {
        let server = (child.spec.start)();
        let (actor_ref, join_handle) = ActorProcess::spawn(BoxedGenServer(server));

        let tx = exit_tx.clone();
        let child_id = child.spec.id.clone();
        let pid = actor_ref.pid().clone();

        tokio::spawn(async move {
            let exit_res = join_handle.await;
            let exit = match exit_res {
                Ok(exit) => exit,
                Err(join_err) => {
                    if join_err.is_panic() {
                        ProcessExit::Crashed("task panicked".to_string())
                    } else {
                        ProcessExit::Crashed("task cancelled".to_string())
                    }
                }
            };
            let _ = tx.send(ChildExitEvent { id: child_id, pid, exit }).await;
        });

        child.actor_ref = Some(actor_ref);
    }
}

// Wrapper to allow trait objects Box<dyn GenServer> inside ActorProcess
struct BoxedGenServer(Box<dyn GenServer>);

#[async_trait::async_trait]
impl GenServer for BoxedGenServer {
    async fn init(&mut self) -> Result<(), ActorError> {
        self.0.init().await
    }

    async fn handle_call(&mut self, req: crate::otp::etf::Term) -> Result<crate::otp::etf::Term, ActorError> {
        self.0.handle_call(req).await
    }

    async fn handle_cast(&mut self, msg: crate::otp::etf::Term) -> Result<(), ActorError> {
        self.0.handle_cast(msg).await
    }

    async fn handle_info(&mut self, msg: crate::otp::etf::Term) -> Result<(), ActorError> {
        self.0.handle_info(msg).await
    }

    async fn terminate(&mut self, reason: &str) {
        self.0.terminate(reason).await;
    }
}
