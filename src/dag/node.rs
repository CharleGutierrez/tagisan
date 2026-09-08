use crate::agent::AutonomousAgent;
use crate::types::TokenUsage;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Execution status of a task node in the DAG workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Skipped => write!(f, "Skipped"),
        }
    }
}

/// Retry policy with configurable backoff strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub initial_delay: Duration,
    pub backoff_factor: f64,
    pub max_delay: Option<Duration>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            max_delay: Some(Duration::from_secs(10)),
        }
    }
}

impl RetryPolicy {
    /// Create a no-retry policy
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a custom retry policy
    pub fn new(max_retries: usize, initial_delay: Duration, backoff_factor: f64) -> Self {
        Self {
            max_retries,
            initial_delay,
            backoff_factor,
            max_delay: Some(Duration::from_secs(10)),
        }
    }

    /// Create a linear retry policy with constant delay
    pub fn linear(max_retries: usize, delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay: delay,
            backoff_factor: 1.0,
            max_delay: Some(delay * (max_retries as u32 + 1)),
        }
    }

    /// Create an exponential backoff retry policy
    pub fn exponential(max_retries: usize, initial_delay: Duration, backoff_factor: f64) -> Self {
        Self {
            max_retries,
            initial_delay,
            backoff_factor,
            max_delay: Some(Duration::from_secs(30)),
        }
    }

    /// Set an upper bound cap on the retry delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = Some(max_delay);
        self
    }

    /// Calculate the delay duration before the next retry attempt (1-indexed)
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 || self.max_retries == 0 {
            return Duration::from_millis(0);
        }
        let multiplier = self.backoff_factor.powi((attempt - 1) as i32);
        let delay_millis = (self.initial_delay.as_millis() as f64 * multiplier) as u64;
        let delay = Duration::from_millis(delay_millis);
        if let Some(max) = self.max_delay {
            std::cmp::min(delay, max)
        } else {
            delay
        }
    }
}

/// Output produced by a completed task execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TaskOutput {
    pub text: String,
    pub usage: TokenUsage,
    pub latency: Duration,
}

impl TaskOutput {
    pub fn new(text: impl Into<String>, usage: TokenUsage, latency: Duration) -> Self {
        Self {
            text: text.into(),
            usage,
            latency,
        }
    }
}

/// A discrete task node within a directed acyclic workflow graph
#[derive(Clone)]
pub struct TaskNode {
    pub id: String,
    pub name: String,
    pub prompt_template: String,
    pub system_prompt: Option<String>,
    pub agent: Option<AutonomousAgent>,
    pub status: TaskStatus,
    pub retry_policy: RetryPolicy,
    pub output: Option<TaskOutput>,
}

impl TaskNode {
    /// Create a new TaskNode with ID, human-readable name, and prompt template
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        prompt_template: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            prompt_template: prompt_template.into(),
            system_prompt: None,
            agent: None,
            status: TaskStatus::Pending,
            retry_policy: RetryPolicy::default(),
            output: None,
        }
    }

    /// Attach an autonomous agent to execute this task
    pub fn with_agent(mut self, agent: AutonomousAgent) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Set a custom system instruction for this task
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Configure a retry policy for transient failures
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// Set initial status
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    /// Set pre-existing or mock output
    pub fn with_output(mut self, output: TaskOutput) -> Self {
        self.output = Some(output);
        self
    }
}

impl std::fmt::Debug for TaskNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("prompt_template", &self.prompt_template)
            .field("system_prompt", &self.system_prompt)
            .field("status", &self.status)
            .field("retry_policy", &self.retry_policy)
            .field("output", &self.output)
            .finish()
    }
}
