//! Full BEAM-grade Actor & GenServer implementation in asynchronous Rust.
//!
//! Provides isolated actor processes with private mailboxes, call/cast semantics,
//! panic boundary protection, and a global process registry.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::otp::etf::Term;

/// Unique BEAM-style Process Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorPid {
    pub node: String,
    pub id: u64,
    pub serial: u32,
}

impl ActorPid {
    pub fn new(node: impl Into<String>, id: u64, serial: u32) -> Self {
        Self {
            node: node.into(),
            id,
            serial,
        }
    }

    pub fn generate(node: &str) -> Self {
        static PID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = PID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            node: node.to_string(),
            id,
            serial: 0,
        }
    }
}

impl fmt::Display for ActorPid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}.{}>", self.node, self.id, self.serial)
    }
}

/// Actor and GenServer runtime errors
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ActorError {
    #[error("Process not found or dead: {0}")]
    ProcessDead(String),
    #[error("Call timed out after {0:?}")]
    Timeout(Duration),
    #[error("Process crashed: {0}")]
    ProcessCrashed(String),
    #[error("Actor already registered with name: {0}")]
    AlreadyRegistered(String),
    #[error("Actor registration not found: {0}")]
    NotFound(String),
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Reasons for an actor process exiting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessExit {
    Normal,
    Shutdown(String),
    Crashed(String),
}

impl ProcessExit {
    pub fn is_crash(&self) -> bool {
        matches!(self, ProcessExit::Crashed(_))
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, ProcessExit::Normal)
    }
}

/// Mailbox Envelope
pub enum Envelope {
    Call {
        req: Term,
        reply_tx: oneshot::Sender<Result<Term, ActorError>>,
    },
    Cast {
        msg: Term,
    },
    Info {
        msg: Term,
    },
    Stop {
        reason: String,
    },
}

/// The core GenServer trait defining actor behavior
#[async_trait]
pub trait GenServer: Send + 'static {
    /// Invoked when the server is started
    async fn init(&mut self) -> Result<(), ActorError> {
        Ok(())
    }

    /// Synchronous call (GenServer.call)
    async fn handle_call(&mut self, req: Term) -> Result<Term, ActorError> {
        Ok(req)
    }

    /// Asynchronous cast (GenServer.cast)
    async fn handle_cast(&mut self, _msg: Term) -> Result<(), ActorError> {
        Ok(())
    }

    /// Handling non-call/cast messages (GenServer.handle_info)
    async fn handle_info(&mut self, _msg: Term) -> Result<(), ActorError> {
        Ok(())
    }

    /// Clean shutdown hook
    async fn terminate(&mut self, _reason: &str) {}
}

/// Thread-safe handle for communicating with an actor process
#[derive(Clone, Debug)]
pub struct ActorRef {
    pid: ActorPid,
    sender: mpsc::Sender<Envelope>,
}

impl ActorRef {
    pub fn new(pid: ActorPid, sender: mpsc::Sender<Envelope>) -> Self {
        Self { pid, sender }
    }

    pub fn pid(&self) -> &ActorPid {
        &self.pid
    }

    /// Check if the actor mailbox is still open
    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Synchronous call with timeout (GenServer.call(pid, req, timeout))
    pub async fn call(&self, req: Term, timeout: Duration) -> Result<Term, ActorError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let envelope = Envelope::Call { req, reply_tx };

        if self.sender.send(envelope).await.is_err() {
            return Err(ActorError::ProcessDead(self.pid.to_string()));
        }

        match tokio::time::timeout(timeout, reply_rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(_oneshot_closed)) => Err(ActorError::ProcessDead(self.pid.to_string())),
            Err(_elapsed) => Err(ActorError::Timeout(timeout)),
        }
    }

    /// Asynchronous cast (GenServer.cast(pid, msg))
    pub async fn cast(&self, msg: Term) -> Result<(), ActorError> {
        let envelope = Envelope::Cast { msg };
        self.sender
            .send(envelope)
            .await
            .map_err(|_| ActorError::ProcessDead(self.pid.to_string()))
    }

    /// Asynchronous info message send (send(pid, msg))
    pub async fn send(&self, msg: Term) -> Result<(), ActorError> {
        let envelope = Envelope::Info { msg };
        self.sender
            .send(envelope)
            .await
            .map_err(|_| ActorError::ProcessDead(self.pid.to_string()))
    }

    /// Request process termination
    pub async fn stop(&self, reason: impl Into<String>) -> Result<(), ActorError> {
        let envelope = Envelope::Stop {
            reason: reason.into(),
        };
        self.sender
            .send(envelope)
            .await
            .map_err(|_| ActorError::ProcessDead(self.pid.to_string()))
    }
}

/// Actor process lifecycle manager
pub struct ActorProcess;

impl ActorProcess {
    /// Spawn a GenServer actor on the default node
    pub fn spawn<G: GenServer>(server: G) -> (ActorRef, JoinHandle<ProcessExit>) {
        let pid = ActorPid::generate("tgs@localhost");
        Self::spawn_with_pid(pid, server, 2048)
    }

    /// Spawn a GenServer actor with custom PID and mailbox capacity
    pub fn spawn_with_pid<G: GenServer>(
        pid: ActorPid,
        mut server: G,
        mailbox_capacity: usize,
    ) -> (ActorRef, JoinHandle<ProcessExit>) {
        let (tx, mut rx) = mpsc::channel(mailbox_capacity);
        let actor_ref = ActorRef::new(pid.clone(), tx);

        let handle = tokio::spawn(async move {
            // Run init hook with panic protection
            let init_res = std::panic::AssertUnwindSafe(server.init())
                .catch_unwind()
                .await;

            match init_res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let reason = format!("init_failed: {}", e);
                    server.terminate(&reason).await;
                    return ProcessExit::Crashed(reason);
                }
                Err(panic_err) => {
                    let panic_msg = extract_panic_message(panic_err);
                    server.terminate(&panic_msg).await;
                    return ProcessExit::Crashed(panic_msg);
                }
            }

            let mut exit_reason = ProcessExit::Normal;

            while let Some(envelope) = rx.recv().await {
                match envelope {
                    Envelope::Call { req, reply_tx } => {
                        let call_res = std::panic::AssertUnwindSafe(server.handle_call(req))
                            .catch_unwind()
                            .await;

                        match call_res {
                            Ok(Ok(reply)) => {
                                let _ = reply_tx.send(Ok(reply));
                            }
                            Ok(Err(err)) => {
                                let _ = reply_tx.send(Err(err.clone()));
                                exit_reason = ProcessExit::Crashed(err.to_string());
                                break;
                            }
                            Err(panic_err) => {
                                let panic_msg = extract_panic_message(panic_err);
                                let _ = reply_tx.send(Err(ActorError::ProcessCrashed(panic_msg.clone())));
                                exit_reason = ProcessExit::Crashed(panic_msg);
                                break;
                            }
                        }
                    }
                    Envelope::Cast { msg } => {
                        let cast_res = std::panic::AssertUnwindSafe(server.handle_cast(msg))
                            .catch_unwind()
                            .await;

                        match cast_res {
                            Ok(Ok(())) => {}
                            Ok(Err(err)) => {
                                exit_reason = ProcessExit::Crashed(err.to_string());
                                break;
                            }
                            Err(panic_err) => {
                                let panic_msg = extract_panic_message(panic_err);
                                exit_reason = ProcessExit::Crashed(panic_msg);
                                break;
                            }
                        }
                    }
                    Envelope::Info { msg } => {
                        let info_res = std::panic::AssertUnwindSafe(server.handle_info(msg))
                            .catch_unwind()
                            .await;

                        match info_res {
                            Ok(Ok(())) => {}
                            Ok(Err(err)) => {
                                exit_reason = ProcessExit::Crashed(err.to_string());
                                break;
                            }
                            Err(panic_err) => {
                                let panic_msg = extract_panic_message(panic_err);
                                exit_reason = ProcessExit::Crashed(panic_msg);
                                break;
                            }
                        }
                    }
                    Envelope::Stop { reason } => {
                        exit_reason = ProcessExit::Shutdown(reason);
                        break;
                    }
                }
            }

            let term_reason = match &exit_reason {
                ProcessExit::Normal => "normal",
                ProcessExit::Shutdown(s) => s.as_str(),
                ProcessExit::Crashed(s) => s.as_str(),
            };
            server.terminate(term_reason).await;
            exit_reason
        });

        (actor_ref, handle)
    }
}

fn extract_panic_message(panic_payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "actor panicked".to_string()
    }
}

// -----------------------------------------------------------------------------
// Global Actor Process Registry
// -----------------------------------------------------------------------------

static GLOBAL_REGISTRY: LazyLock<RwLock<HashMap<String, ActorRef>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct ProcessRegistry;

impl ProcessRegistry {
    /// Look up an actor by registered name (like BEAM Process.whereis/1)
    pub fn whereis(name: &str) -> Option<ActorRef> {
        let read = GLOBAL_REGISTRY.read().ok()?;
        if let Some(actor) = read.get(name) {
            if actor.is_alive() {
                return Some(actor.clone());
            }
        }
        drop(read);

        // Lazily clean up dead reference
        if let Ok(mut write) = GLOBAL_REGISTRY.write() {
            if let Some(actor) = write.get(name) {
                if !actor.is_alive() {
                    write.remove(name);
                }
            }
        }
        None
    }

    /// Register an actor under a unique name (like BEAM Process.register/2)
    pub fn register(name: impl Into<String>, actor: ActorRef) -> Result<(), ActorError> {
        let name = name.into();
        let mut write = GLOBAL_REGISTRY
            .write()
            .map_err(|e| ActorError::Custom(e.to_string()))?;

        if let Some(existing) = write.get(&name) {
            if existing.is_alive() {
                return Err(ActorError::AlreadyRegistered(name));
            }
        }

        write.insert(name, actor);
        Ok(())
    }

    /// Unregister a named actor
    pub fn unregister(name: &str) -> Option<ActorRef> {
        let mut write = GLOBAL_REGISTRY.write().ok()?;
        write.remove(name)
    }

    /// List all registered alive actors
    pub fn all_registered() -> Vec<(String, ActorPid)> {
        let mut result = Vec::new();
        if let Ok(read) = GLOBAL_REGISTRY.read() {
            for (name, actor) in read.iter() {
                if actor.is_alive() {
                    result.push((name.clone(), actor.pid().clone()));
                }
            }
        }
        result
    }

    /// Clear all registrations (useful for test isolation)
    pub fn clear() {
        if let Ok(mut write) = GLOBAL_REGISTRY.write() {
            write.clear();
        }
    }
}
