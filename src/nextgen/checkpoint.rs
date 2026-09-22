//! # Time-Travel Debugging & Event-Sourced Swarm Checkpointing (`tgs nextgen checkpoint`)
//!
//! Provides deterministic state persistence, append-only WAL event sourcing,
//! DAG execution state restoration, and time-travel rollback/forking across swarm agents.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

/// Types of discrete events tracked in the Swarm Write-Ahead Log (WAL)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwarmEventType {
    TaskSpawned {
        task_id: String,
        node_name: String,
    },
    NodeStarted {
        node_id: String,
        agent_id: String,
    },
    ToolInvoked {
        node_id: String,
        tool: String,
        input: serde_json::Value,
    },
    StateMutated {
        key: String,
        old_val: Option<serde_json::Value>,
        new_val: serde_json::Value,
    },
    MessageSent {
        from_agent: String,
        to_agent: String,
        message_type: String,
    },
    NodeCompleted {
        node_id: String,
        success: bool,
        output: serde_json::Value,
    },
    BranchForked {
        branch_name: String,
        source_checkpoint_id: String,
    },
    RollbackExecuted {
        target_sequence: u64,
        target_checkpoint_id: String,
    },
}

impl SwarmEventType {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::TaskSpawned { .. } => "TaskSpawned",
            Self::NodeStarted { .. } => "NodeStarted",
            Self::ToolInvoked { .. } => "ToolInvoked",
            Self::StateMutated { .. } => "StateMutated",
            Self::MessageSent { .. } => "MessageSent",
            Self::NodeCompleted { .. } => "NodeCompleted",
            Self::BranchForked { .. } => "BranchForked",
            Self::RollbackExecuted { .. } => "RollbackExecuted",
        }
    }
}

/// Single immutable event recorded in the Swarm WAL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmEvent {
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub agent_id: String,
    pub event_type: SwarmEventType,
    pub payload_blake3: String,
}

impl SwarmEvent {
    pub fn compute_payload_hash(event_type: &SwarmEventType) -> String {
        let serialized = serde_json::to_vec(event_type).unwrap_or_default();
        let hash = blake3::hash(&serialized);
        hash.to_hex().to_string()
    }
}

/// Append-Only Write-Ahead Log (WAL) for Swarm operations
#[derive(Debug, Clone, Default)]
pub struct SwarmWal {
    pub events: Vec<SwarmEvent>,
    pub current_sequence: u64,
}

impl SwarmWal {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            current_sequence: 0,
        }
    }

    /// Append a new event to the WAL
    pub fn append(&mut self, agent_id: impl Into<String>, event_type: SwarmEventType) -> SwarmEvent {
        let sequence = self.current_sequence;
        self.current_sequence += 1;

        let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
        let payload_blake3 = SwarmEvent::compute_payload_hash(&event_type);

        let event = SwarmEvent {
            sequence,
            timestamp_ns,
            agent_id: agent_id.into(),
            event_type,
            payload_blake3,
        };

        self.events.push(event.clone());
        event
    }

    /// Read events within a sequence range [start_seq, end_seq]
    pub fn read_range(&self, start_seq: u64, end_seq: u64) -> Vec<SwarmEvent> {
        self.events
            .iter()
            .filter(|e| e.sequence >= start_seq && e.sequence <= end_seq)
            .cloned()
            .collect()
    }

    /// Retrieve all events after a specified sequence number
    pub fn events_after(&self, seq: u64) -> Vec<SwarmEvent> {
        self.events
            .iter()
            .filter(|e| e.sequence > seq)
            .cloned()
            .collect()
    }

    /// Truncate WAL after a sequence number (used during hard rollback or branch divergence)
    pub fn truncate_after(&mut self, seq: u64) {
        self.events.retain(|e| e.sequence <= seq);
        self.current_sequence = self.events.last().map(|e| e.sequence + 1).unwrap_or(0);
    }

    /// Verify log sequential ordering and cryptographic payload hashes
    pub fn verify_integrity(&self) -> bool {
        for (idx, event) in self.events.iter().enumerate() {
            if event.sequence != idx as u64 {
                return false;
            }
            let expected_hash = SwarmEvent::compute_payload_hash(&event.event_type);
            if event.payload_blake3 != expected_hash {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Execution state of a single DAG node in a swarm workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagNodeExecutionState {
    pub node_id: String,
    pub agent_id: String,
    pub status: String, // "Pending", "Running", "Completed", "Failed", "Paused"
    pub scratchpad: String,
    pub inputs: serde_json::Value,
    pub outputs: Option<serde_json::Value>,
    pub completed_at_sequence: Option<u64>,
}

/// Deterministic snapshot of the entire swarm state at a given sequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmCheckpoint {
    pub checkpoint_id: String,
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub branch_name: String,
    pub description: String,
    pub dag_states: HashMap<String, DagNodeExecutionState>,
    pub swarm_memory: HashMap<String, serde_json::Value>,
    pub active_agents: Vec<String>,
    pub state_merkle_root: String,
}

impl SwarmCheckpoint {
    /// Compute a deterministic Blake3 root over all DAG node states and shared memory
    pub fn compute_state_root(
        dag_states: &HashMap<String, DagNodeExecutionState>,
        memory: &HashMap<String, serde_json::Value>,
    ) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SWARM_CHECKPOINT_STATE_ROOT_V1:");

        // Deterministically sorted keys for DAG states
        let mut node_keys: Vec<&String> = dag_states.keys().collect();
        node_keys.sort();
        for k in node_keys {
            hasher.update(k.as_bytes());
            let node_bytes = serde_json::to_vec(&dag_states[k]).unwrap_or_default();
            hasher.update(&node_bytes);
        }

        // Deterministically sorted keys for shared memory
        let mut mem_keys: Vec<&String> = memory.keys().collect();
        mem_keys.sort();
        for k in mem_keys {
            hasher.update(k.as_bytes());
            let val_bytes = serde_json::to_vec(&memory[k]).unwrap_or_default();
            hasher.update(&val_bytes);
        }

        hasher.finalize().to_hex().to_string()
    }
}

/// Differential comparison between two checkpoints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointDiff {
    pub checkpoint_a: String,
    pub checkpoint_b: String,
    pub added_dag_nodes: Vec<String>,
    pub removed_dag_nodes: Vec<String>,
    pub modified_dag_nodes: Vec<String>,
    pub changed_memory_keys: Vec<String>,
}

/// High-level Swarm Checkpoint and Time-Travel Manager
#[derive(Debug, Clone)]
pub struct SwarmCheckpointManager {
    pub wal: SwarmWal,
    pub checkpoints: Vec<SwarmCheckpoint>,
    pub active_branch: String,
    pub current_dag_states: HashMap<String, DagNodeExecutionState>,
    pub current_swarm_memory: HashMap<String, serde_json::Value>,
    pub active_agents: Vec<String>,
}

impl Default for SwarmCheckpointManager {
    fn default() -> Self {
        Self::new("main")
    }
}

impl SwarmCheckpointManager {
    pub fn new(branch_name: impl Into<String>) -> Self {
        Self {
            wal: SwarmWal::new(),
            checkpoints: Vec::new(),
            active_branch: branch_name.into(),
            current_dag_states: HashMap::new(),
            current_swarm_memory: HashMap::new(),
            active_agents: Vec::new(),
        }
    }

    /// Record an event in the WAL
    pub fn record_event(&mut self, agent_id: &str, event_type: SwarmEventType) -> SwarmEvent {
        if !self.active_agents.contains(&agent_id.to_string()) {
            self.active_agents.push(agent_id.to_string());
        }
        self.wal.append(agent_id, event_type)
    }

    /// Update execution state of a DAG node
    pub fn update_dag_node(
        &mut self,
        node_id: impl Into<String>,
        agent_id: impl Into<String>,
        status: impl Into<String>,
        scratchpad: impl Into<String>,
        inputs: serde_json::Value,
        outputs: Option<serde_json::Value>,
    ) {
        let node_id_str = node_id.into();
        let agent_id_str = agent_id.into();
        let status_str = status.into();

        let completed_at_seq = if status_str == "Completed" {
            Some(self.wal.current_sequence)
        } else {
            None
        };

        let state = DagNodeExecutionState {
            node_id: node_id_str.clone(),
            agent_id: agent_id_str.clone(),
            status: status_str.clone(),
            scratchpad: scratchpad.into(),
            inputs,
            outputs: outputs.clone(),
            completed_at_sequence: completed_at_seq,
        };

        self.current_dag_states.insert(node_id_str.clone(), state);

        self.record_event(
            &agent_id_str,
            SwarmEventType::NodeCompleted {
                node_id: node_id_str,
                success: status_str == "Completed",
                output: outputs.unwrap_or(serde_json::Value::Null),
            },
        );
    }

    /// Set a key-value pair in shared swarm memory
    pub fn set_memory(&mut self, key: impl Into<String>, value: serde_json::Value) {
        let key_str = key.into();
        let old_val = self.current_swarm_memory.get(&key_str).cloned();
        self.current_swarm_memory.insert(key_str.clone(), value.clone());

        self.record_event(
            "swarm:orchestrator",
            SwarmEventType::StateMutated {
                key: key_str,
                old_val,
                new_val: value,
            },
        );
    }

    /// Retrieve a key from shared memory
    pub fn get_memory(&self, key: &str) -> Option<&serde_json::Value> {
        self.current_swarm_memory.get(key)
    }

    /// Snapshot the current state and append a checkpoint
    pub fn create_checkpoint(&mut self, description: impl Into<String>) -> Result<SwarmCheckpoint> {
        let sequence = self.wal.current_sequence;
        let timestamp_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
        let desc = description.into();

        let state_merkle_root = SwarmCheckpoint::compute_state_root(
            &self.current_dag_states,
            &self.current_swarm_memory,
        );

        let checkpoint_id = format!("ckpt-{}-{}-{}", self.active_branch, sequence, &state_merkle_root[..8]);

        let checkpoint = SwarmCheckpoint {
            checkpoint_id: checkpoint_id.clone(),
            sequence,
            timestamp_ns,
            branch_name: self.active_branch.clone(),
            description: desc,
            dag_states: self.current_dag_states.clone(),
            swarm_memory: self.current_swarm_memory.clone(),
            active_agents: self.active_agents.clone(),
            state_merkle_root,
        };

        self.checkpoints.push(checkpoint.clone());
        Ok(checkpoint)
    }

    /// Time-travel rollback: restore state to a specific checkpoint
    pub fn rollback_to_checkpoint(&mut self, checkpoint_id: &str) -> Result<SwarmCheckpoint> {
        let target = self
            .checkpoints
            .iter()
            .find(|c| c.checkpoint_id == checkpoint_id)
            .cloned()
            .ok_or_else(|| {
                TagisanError::Execution(format!("Checkpoint '{}' not found", checkpoint_id))
            })?;

        // Restore state
        self.current_dag_states = target.dag_states.clone();
        self.current_swarm_memory = target.swarm_memory.clone();
        self.active_agents = target.active_agents.clone();

        // Record the rollback event in the WAL
        self.record_event(
            "swarm:orchestrator",
            SwarmEventType::RollbackExecuted {
                target_sequence: target.sequence,
                target_checkpoint_id: checkpoint_id.to_string(),
            },
        );

        Ok(target)
    }

    /// Rollback to a specific WAL sequence by finding the nearest checkpoint and replaying
    pub fn rollback_to_sequence(&mut self, target_sequence: u64) -> Result<SwarmCheckpoint> {
        let best_checkpoint = self
            .checkpoints
            .iter()
            .filter(|c| c.sequence <= target_sequence)
            .max_by_key(|c| c.sequence)
            .cloned();

        let mut restored = match best_checkpoint {
            Some(ckpt) => {
                self.current_dag_states = ckpt.dag_states.clone();
                self.current_swarm_memory = ckpt.swarm_memory.clone();
                self.active_agents = ckpt.active_agents.clone();
                ckpt
            }
            None => {
                // Reset to empty state
                self.current_dag_states.clear();
                self.current_swarm_memory.clear();
                self.active_agents.clear();
                self.create_checkpoint("initial_restoration")?
            }
        };

        // Replay events between restored checkpoint and target_sequence
        let replay_events = self.wal.read_range(restored.sequence + 1, target_sequence);
        for event in replay_events {
            match event.event_type {
                SwarmEventType::StateMutated { key, new_val, .. } => {
                    self.current_swarm_memory.insert(key, new_val);
                }
                SwarmEventType::NodeCompleted { node_id, success, output } => {
                    if let Some(node) = self.current_dag_states.get_mut(&node_id) {
                        node.status = if success { "Completed".to_string() } else { "Failed".to_string() };
                        node.outputs = Some(output);
                    }
                }
                _ => {}
            }
        }

        // Create a synthetic checkpoint representing the restored state
        restored = self.create_checkpoint(format!("time_travel_to_seq_{}", target_sequence))?;
        Ok(restored)
    }

    /// Fork an independent branch from an existing checkpoint
    pub fn fork_branch(&self, checkpoint_id: &str, new_branch_name: &str) -> Result<SwarmCheckpointManager> {
        let source_ckpt = self
            .checkpoints
            .iter()
            .find(|c| c.checkpoint_id == checkpoint_id)
            .ok_or_else(|| {
                TagisanError::Execution(format!("Source checkpoint '{}' not found", checkpoint_id))
            })?;

        let mut forked = SwarmCheckpointManager::new(new_branch_name);
        // Copy WAL events up to source sequence
        forked.wal.events = self.wal.read_range(0, source_ckpt.sequence);
        forked.wal.current_sequence = source_ckpt.sequence + 1;
        forked.current_dag_states = source_ckpt.dag_states.clone();
        forked.current_swarm_memory = source_ckpt.swarm_memory.clone();
        forked.active_agents = source_ckpt.active_agents.clone();

        // Record branch forked event
        forked.record_event(
            "swarm:orchestrator",
            SwarmEventType::BranchForked {
                branch_name: new_branch_name.to_string(),
                source_checkpoint_id: checkpoint_id.to_string(),
            },
        );

        forked.create_checkpoint(format!("fork_from_{}", checkpoint_id))?;
        Ok(forked)
    }

    /// List all checkpoints
    pub fn list_checkpoints(&self) -> &[SwarmCheckpoint] {
        &self.checkpoints
    }

    /// Get a checkpoint by ID
    pub fn get_checkpoint(&self, checkpoint_id: &str) -> Option<&SwarmCheckpoint> {
        self.checkpoints.iter().find(|c| c.checkpoint_id == checkpoint_id)
    }

    /// Compute diff between two checkpoints
    pub fn diff_checkpoints(&self, id_a: &str, id_b: &str) -> Result<CheckpointDiff> {
        let ckpt_a = self.get_checkpoint(id_a).ok_or_else(|| {
            TagisanError::Execution(format!("Checkpoint '{}' not found", id_a))
        })?;
        let ckpt_b = self.get_checkpoint(id_b).ok_or_else(|| {
            TagisanError::Execution(format!("Checkpoint '{}' not found", id_b))
        })?;

        let mut added_dag_nodes = Vec::new();
        let mut removed_dag_nodes = Vec::new();
        let mut modified_dag_nodes = Vec::new();

        for key in ckpt_b.dag_states.keys() {
            if !ckpt_a.dag_states.contains_key(key) {
                added_dag_nodes.push(key.clone());
            } else if ckpt_a.dag_states[key] != ckpt_b.dag_states[key] {
                modified_dag_nodes.push(key.clone());
            }
        }

        for key in ckpt_a.dag_states.keys() {
            if !ckpt_b.dag_states.contains_key(key) {
                removed_dag_nodes.push(key.clone());
            }
        }

        let mut changed_memory_keys = Vec::new();
        for (k, v) in &ckpt_b.swarm_memory {
            if ckpt_a.swarm_memory.get(k) != Some(v) {
                changed_memory_keys.push(k.clone());
            }
        }
        for k in ckpt_a.swarm_memory.keys() {
            if !ckpt_b.swarm_memory.contains_key(k) && !changed_memory_keys.contains(k) {
                changed_memory_keys.push(k.clone());
            }
        }

        added_dag_nodes.sort();
        removed_dag_nodes.sort();
        modified_dag_nodes.sort();
        changed_memory_keys.sort();

        Ok(CheckpointDiff {
            checkpoint_a: id_a.to_string(),
            checkpoint_b: id_b.to_string(),
            added_dag_nodes,
            removed_dag_nodes,
            modified_dag_nodes,
            changed_memory_keys,
        })
    }
}
