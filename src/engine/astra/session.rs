//! Real-time bi-directional streaming session with event bus and multimodal frame queues.

use crate::engine::astra::agent::{AstraAgentResult, AstraVisualAgent, AstraVisualAgentConfig};
use crate::engine::astra::screen::ScreenFrame;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Lifecycle state of the streaming computer-use session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Idle,
    Running,
    Paused,
    Completed,
    Aborted,
    Error,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Aborted => write!(f, "Aborted"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Event broadcast across the Astra real-time event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstraEvent {
    ScreenCaptured {
        frame_id: u64,
        width: u32,
        height: u32,
        timestamp_ms: u64,
        is_virtual: bool,
    },
    InputSynthesized {
        action: String,
        details: String,
        success: bool,
        cursor: (u32, u32),
    },
    VisualDiffDetected {
        diff_percentage: f64,
        is_significant: bool,
        changed_cells_count: usize,
    },
    AgentThought {
        step: usize,
        thought: String,
    },
    ActionProposed {
        step: usize,
        action: String,
        target: Option<(u32, u32)>,
    },
    SecurityAudit {
        step: usize,
        action: String,
        allowed: bool,
        reason: Option<String>,
    },
    SessionStateChanged {
        old_state: SessionState,
        new_state: SessionState,
    },
    GoalUpdated {
        new_goal: String,
    },
    AgentStepComplete {
        step: usize,
        duration_ms: u64,
        goal_accomplished: bool,
    },
    Error {
        message: String,
    },
}

/// Real-time broadcast event bus for Astra sessions
#[derive(Clone)]
pub struct AstraEventBus {
    sender: Arc<broadcast::Sender<AstraEvent>>,
}

impl Default for AstraEventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl AstraEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Publish an event to all active subscribers
    pub fn publish(&self, event: AstraEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to real-time events
    pub fn subscribe(&self) -> broadcast::Receiver<AstraEvent> {
        self.sender.subscribe()
    }
}

/// Bounded multimodal frame queue for real-time streaming
#[derive(Debug, Clone)]
pub struct AstraFrameQueue {
    frames: VecDeque<ScreenFrame>,
    max_capacity: usize,
}

impl Default for AstraFrameQueue {
    fn default() -> Self {
        Self::new(10)
    }
}

impl AstraFrameQueue {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_capacity),
            max_capacity: max_capacity.max(2),
        }
    }

    /// Push frame into queue, evicting oldest if capacity reached
    pub fn push(&mut self, frame: ScreenFrame) {
        if self.frames.len() >= self.max_capacity {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    /// Most recently captured frame
    pub fn latest(&self) -> Option<&ScreenFrame> {
        self.frames.back()
    }

    /// Find frame by ID
    pub fn get_by_id(&self, frame_id: u64) -> Option<&ScreenFrame> {
        self.frames.iter().find(|f| f.id == frame_id)
    }

    /// Current count of queued frames
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

/// Real-time streaming computer-use session
pub struct AstraSession {
    pub session_id: String,
    pub state: SessionState,
    pub agent: AstraVisualAgent,
    pub event_bus: AstraEventBus,
    pub frame_queue: AstraFrameQueue,
}

impl AstraSession {
    /// Create new AstraSession with specified configuration
    pub fn new(session_id: impl Into<String>, config: AstraVisualAgentConfig) -> Self {
        let sid = session_id.into();
        let agent = AstraVisualAgent::new(config);
        let event_bus = AstraEventBus::default();
        let frame_queue = AstraFrameQueue::default();

        Self {
            session_id: sid,
            state: SessionState::Idle,
            agent,
            event_bus,
            frame_queue,
        }
    }

    /// Subscribe to session events
    pub fn subscribe(&self) -> broadcast::Receiver<AstraEvent> {
        self.event_bus.subscribe()
    }

    /// Update session state and emit event
    pub fn set_state(&mut self, new_state: SessionState) {
        let old_state = self.state;
        self.state = new_state;
        self.event_bus.publish(AstraEvent::SessionStateChanged {
            old_state,
            new_state,
        });
    }

    /// Pause the session execution
    pub fn pause(&mut self) {
        if self.state == SessionState::Running {
            self.set_state(SessionState::Paused);
        }
    }

    /// Resume a paused session
    pub fn resume(&mut self) {
        if self.state == SessionState::Paused {
            self.set_state(SessionState::Running);
        }
    }

    /// Abort execution
    pub fn abort(&mut self) {
        self.set_state(SessionState::Aborted);
    }

    /// Inject guidance or new user directive into ongoing session
    pub fn inject_guidance(&mut self, guidance: &str) {
        self.agent.config.goal = format!("{} (Guidance: {})", self.agent.config.goal, guidance);
        self.event_bus.publish(AstraEvent::GoalUpdated {
            new_goal: self.agent.config.goal.clone(),
        });
    }

    /// Run full streaming loop with event publishing
    pub async fn run(&mut self) -> Result<AstraAgentResult> {
        self.set_state(SessionState::Running);

        // Subscribe to internal frame updates
        let res = self.agent.run().await;

        match res {
            Ok(agent_result) => {
                // Post-process trace and publish events
                for step in &agent_result.step_trace {
                    self.event_bus.publish(AstraEvent::AgentThought {
                        step: step.step_number,
                        thought: step.thought.clone(),
                    });
                    self.event_bus.publish(AstraEvent::ActionProposed {
                        step: step.step_number,
                        action: step.proposed_action.clone(),
                        target: step.execution_result.as_ref().map(|r| r.cursor_position),
                    });
                    self.event_bus.publish(AstraEvent::AgentStepComplete {
                        step: step.step_number,
                        duration_ms: step.duration_ms,
                        goal_accomplished: agent_result.success,
                    });
                }

                if agent_result.success {
                    self.set_state(SessionState::Completed);
                } else {
                    self.set_state(SessionState::Aborted);
                }

                Ok(agent_result)
            }
            Err(e) => {
                self.set_state(SessionState::Error);
                self.event_bus.publish(AstraEvent::Error {
                    message: e.to_string(),
                });
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_astra_session_event_bus() {
        let session_id = "test-session-1";
        let config = AstraVisualAgentConfig {
            goal: "Test session event bus".to_string(),
            headless: true,
            max_steps: 2,
            step_delay_ms: 0,
            ..Default::default()
        };

        let mut session = AstraSession::new(session_id, config);
        let mut rx = session.subscribe();

        session.set_state(SessionState::Running);
        let event = rx.recv().await.expect("Should receive state change event");
        if let AstraEvent::SessionStateChanged { old_state, new_state } = event {
            assert_eq!(old_state, SessionState::Idle);
            assert_eq!(new_state, SessionState::Running);
        } else {
            panic!("Expected SessionStateChanged event");
        }
    }
}
