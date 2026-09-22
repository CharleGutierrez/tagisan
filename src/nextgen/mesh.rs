//! # ZeroConf P2P Swarm Mesh (`tgs nextgen mesh`)
//!
//! Provides autonomous peer node beaconing, hardware capability discovery
//! (VRAM, compute tiers, loaded models), distributed work-stealing, and cluster load balancing.

use std::collections::{HashMap, VecDeque};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

/// Compute capabilities tiers across heterogeneous hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComputeTier {
    /// Local edge device / NPU / mobile (8-16GB VRAM, low latency, e.g. smollm2, phi-3)
    Tier3EdgeLocal = 1,
    /// LAN workstation / NPU server (24-128GB unified RAM, e.g. Dual 4090, M3 Ultra, 70B models)
    Tier2NpuServer = 2,
    /// Frontier Cloud Reasoning API (Infinite capacity, highest reasoning, e.g. Gemini 3 Pro, Claude 3.7)
    Tier1FrontierCloud = 3,
}

impl ComputeTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tier3EdgeLocal => "Tier 3: Edge Local (NPU/Laptop)",
            Self::Tier2NpuServer => "Tier 2: Workstation NPU Server",
            Self::Tier1FrontierCloud => "Tier 1: Frontier Cloud Reasoning",
        }
    }

    /// Whether this tier can satisfy a task requiring `required_tier`
    pub fn can_satisfy(&self, required: &ComputeTier) -> bool {
        *self as u8 >= *required as u8
    }
}

/// Operational status of a peer node in the mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerStatus {
    Online,
    Busy,
    Draining,
    Offline,
}

/// Hardware capabilities and telemetry published by a peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerCapability {
    pub node_id: String,
    pub node_name: String,
    pub endpoint: String,
    pub tier: ComputeTier,
    pub total_vram_mb: u64,
    pub available_vram_mb: u64,
    pub loaded_models: Vec<String>,
    pub quantization_levels: Vec<String>,
    pub max_concurrency: usize,
    pub active_tasks: usize,
    pub last_heartbeat_epoch_ms: u64,
    pub status: PeerStatus,
}

impl PeerCapability {
    pub fn new(
        node_id: impl Into<String>,
        node_name: impl Into<String>,
        endpoint: impl Into<String>,
        tier: ComputeTier,
        total_vram_mb: u64,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            node_name: node_name.into(),
            endpoint: endpoint.into(),
            tier,
            total_vram_mb,
            available_vram_mb: total_vram_mb,
            loaded_models: Vec::new(),
            quantization_levels: vec!["q4_k_m".to_string(), "q8_0".to_string()],
            max_concurrency: 4,
            active_tasks: 0,
            last_heartbeat_epoch_ms: Utc::now().timestamp_millis() as u64,
            status: PeerStatus::Online,
        }
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.loaded_models = models;
        self
    }

    pub fn utilization_rate(&self) -> f64 {
        if self.max_concurrency == 0 {
            1.0
        } else {
            (self.active_tasks as f64) / (self.max_concurrency as f64)
        }
    }
}

/// ZeroConf discovery beacon broadcast across local network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshBeacon {
    pub cluster_id: String,
    pub sender: PeerCapability,
    pub timestamp_epoch_ms: u64,
    pub nonce: u64,
    pub signature_token: String,
}

/// Distributed compute task submitted to the mesh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeshTask {
    pub task_id: String,
    pub task_name: String,
    pub required_tier: ComputeTier,
    pub required_vram_mb: u64,
    pub target_model: Option<String>,
    pub priority: u8, // 0 = Lowest, 255 = Urgent / Highest
    pub payload: serde_json::Value,
    pub origin_node_id: String,
    pub assigned_node_id: Option<String>,
    pub created_at_epoch_ms: u64,
    pub steals_count: u32,
}

impl MeshTask {
    pub fn new(
        task_id: impl Into<String>,
        task_name: impl Into<String>,
        required_tier: ComputeTier,
        required_vram_mb: u64,
        payload: serde_json::Value,
        origin_node_id: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            task_name: task_name.into(),
            required_tier,
            required_vram_mb,
            target_model: None,
            priority: 128,
            payload,
            origin_node_id: origin_node_id.into(),
            assigned_node_id: None,
            created_at_epoch_ms: Utc::now().timestamp_millis() as u64,
            steals_count: 0,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_target_model(mut self, model: impl Into<String>) -> Self {
        self.target_model = Some(model.into());
        self
    }
}

/// Thread-safe Work-Stealing Queue
#[derive(Debug, Clone, Default)]
pub struct WorkStealingQueue {
    pub tasks: VecDeque<MeshTask>,
}

impl WorkStealingQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    /// Push a task locally
    pub fn push_local(&mut self, task: MeshTask) {
        // Higher priority tasks go to the front
        if task.priority > 128 {
            self.tasks.push_front(task);
        } else {
            self.tasks.push_back(task);
        }
    }

    /// Pop a task for local execution (pops highest priority first)
    pub fn pop_local(&mut self) -> Option<MeshTask> {
        self.tasks.pop_front()
    }

    /// Steal a task from this queue (steals from the back to minimize contention)
    pub fn steal(&mut self) -> Option<MeshTask> {
        if let Some(mut task) = self.tasks.pop_back() {
            task.steals_count += 1;
            Some(task)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Aggregate cluster telemetry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterMetrics {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub total_vram_mb: u64,
    pub available_vram_mb: u64,
    pub total_queued_tasks: usize,
    pub average_utilization: f64,
    pub available_models: Vec<String>,
}

/// ZeroConf Peer-to-Peer Swarm Mesh Engine
#[derive(Debug, Clone)]
pub struct P2pSwarmMesh {
    pub local_node: PeerCapability,
    pub cluster_id: String,
    pub peers: HashMap<String, PeerCapability>,
    pub local_queue: WorkStealingQueue,
    pub peer_queues: HashMap<String, WorkStealingQueue>,
    pub completed_tasks: HashMap<String, serde_json::Value>,
    pub heartbeat_ttl_ms: u64,
}

impl P2pSwarmMesh {
    pub fn new(local_node: PeerCapability, cluster_id: impl Into<String>) -> Self {
        Self {
            local_node,
            cluster_id: cluster_id.into(),
            peers: HashMap::new(),
            local_queue: WorkStealingQueue::new(),
            peer_queues: HashMap::new(),
            completed_tasks: HashMap::new(),
            heartbeat_ttl_ms: 10_000, // 10 seconds TTL
        }
    }

    /// Broadcast a ZeroConf discovery beacon
    pub fn create_beacon(&self) -> MeshBeacon {
        let now = Utc::now().timestamp_millis() as u64;
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.cluster_id.as_bytes());
        hasher.update(self.local_node.node_id.as_bytes());
        hasher.update(&now.to_le_bytes());
        let token = format!("sig:mesh:{}", &hasher.finalize().to_hex()[..16]);

        MeshBeacon {
            cluster_id: self.cluster_id.clone(),
            sender: self.local_node.clone(),
            timestamp_epoch_ms: now,
            nonce: now,
            signature_token: token,
        }
    }

    /// Receive and process an incoming peer beacon
    pub fn receive_beacon(&mut self, beacon: MeshBeacon) -> Result<bool> {
        if beacon.cluster_id != self.cluster_id {
            return Ok(false); // Ignore beacons from other swarms
        }

        if beacon.sender.node_id == self.local_node.node_id {
            return Ok(false); // Ignore own beacon loopback
        }

        let peer_id = beacon.sender.node_id.clone();
        self.peers.insert(peer_id.clone(), beacon.sender);
        self.peer_queues.entry(peer_id).or_insert_with(WorkStealingQueue::new);

        Ok(true)
    }

    /// Prune stale peer nodes that have missed heartbeats past TTL
    pub fn prune_stale_peers(&mut self, now_ms: u64) -> Vec<String> {
        let mut stale_node_ids = Vec::new();
        for (id, peer) in &mut self.peers {
            if now_ms.saturating_sub(peer.last_heartbeat_epoch_ms) > self.heartbeat_ttl_ms {
                peer.status = PeerStatus::Offline;
                stale_node_ids.push(id.clone());
            }
        }
        stale_node_ids
    }

    /// Check if a node satisfies requirements to execute a given task
    /// Check if a node satisfies requirements to execute a given task
    pub fn can_node_execute(&self, node: &PeerCapability, task: &MeshTask) -> bool {
        Self::can_execute(node, task)
    }

    /// Static validator for capability satisfaction
    pub fn can_execute(node: &PeerCapability, task: &MeshTask) -> bool {
        if node.status != PeerStatus::Online && node.status != PeerStatus::Busy {
            return false;
        }

        // 1. Check compute tier satisfaction
        if !node.tier.can_satisfy(&task.required_tier) {
            return false;
        }

        // 2. Check VRAM availability
        if node.available_vram_mb < task.required_vram_mb {
            return false;
        }

        // 3. Check loaded model constraint if specified
        if let Some(target_model) = &task.target_model {
            if !node.loaded_models.iter().any(|m| m == target_model) {
                return false;
            }
        }

        true
    }

    /// Submit a task to the local work queue
    pub fn submit_task(&mut self, task: MeshTask) -> Result<()> {
        self.local_queue.push_local(task);
        self.local_node.active_tasks = self.local_queue.len();
        Ok(())
    }

    /// Pop a task for immediate execution on the local node
    pub fn pop_local_task(&mut self) -> Option<MeshTask> {
        let task = self.local_queue.pop_local();
        self.local_node.active_tasks = self.local_queue.len();
        task
    }

    /// Steal a task from a specific remote peer queue
    pub fn steal_from_peer(&mut self, peer_id: &str) -> Option<MeshTask> {
        if let Some(queue) = self.peer_queues.get_mut(peer_id) {
            if let Some(stolen) = queue.steal() {
                if let Some(peer) = self.peers.get_mut(peer_id) {
                    peer.active_tasks = queue.len();
                }
                return Some(stolen);
            }
        }
        None
    }

    /// Autonomous work stealing: steals work from the most overloaded compatible peer
    pub fn auto_work_steal(&mut self) -> Option<MeshTask> {
        // If local node is already saturated, don't steal
        if self.local_node.active_tasks >= self.local_node.max_concurrency {
            return None;
        }

        // Find the peer with the highest queued task count
        let mut sorted_peers: Vec<String> = self.peers.keys().cloned().collect();
        sorted_peers.sort_by_key(|id| {
            std::cmp::Reverse(self.peer_queues.get(id).map(|q| q.len()).unwrap_or(0))
        });

        let local_node = self.local_node.clone();

        for peer_id in sorted_peers {
            let task_opt = if let Some(queue) = self.peer_queues.get_mut(&peer_id) {
                queue.steal()
            } else {
                None
            };

            if let Some(candidate) = task_opt {
                if Self::can_execute(&local_node, &candidate) {
                    if let Some(queue) = self.peer_queues.get_mut(&peer_id) {
                        if let Some(peer) = self.peers.get_mut(&peer_id) {
                            peer.active_tasks = queue.len();
                        }
                    }
                    return Some(candidate);
                } else {
                    // Put back if local node cannot execute
                    if let Some(queue) = self.peer_queues.get_mut(&peer_id) {
                        queue.push_local(candidate);
                    }
                }
            }
        }

        None
    }

    /// Autonomous cluster rebalancer: migrates tasks from overloaded nodes to underloaded nodes
    pub fn auto_balance_cluster(&mut self) -> Vec<(String, String)> {
        let mut migrations = Vec::new();

        let underloaded_peers: Vec<String> = self
            .peers
            .iter()
            .filter(|(_, p)| p.status == PeerStatus::Online && p.utilization_rate() < 0.5)
            .map(|(id, _)| id.clone())
            .collect();

        for target_id in underloaded_peers {
            let target_node = match self.peers.get(&target_id) {
                Some(n) => n.clone(),
                None => continue,
            };

            // Check if local queue has excess tasks
            if self.local_queue.len() > self.local_node.max_concurrency {
                if let Some(task) = self.local_queue.steal() {
                    if Self::can_execute(&target_node, &task) {
                        migrations.push((self.local_node.node_id.clone(), target_id.clone()));
                        if let Some(t_queue) = self.peer_queues.get_mut(&target_id) {
                            t_queue.push_local(task);
                        }
                    } else {
                        self.local_queue.push_local(task);
                    }
                }
            }
        }

        migrations
    }

    /// Find the optimal node in the cluster to execute a task based on tier, VRAM, and load
    pub fn find_best_node_for_task(&self, task: &MeshTask) -> Option<String> {
        let mut candidates = Vec::new();

        if self.can_node_execute(&self.local_node, task) {
            candidates.push((self.local_node.node_id.clone(), self.local_node.utilization_rate()));
        }

        for (id, peer) in &self.peers {
            if self.can_node_execute(peer, task) {
                candidates.push((id.clone(), peer.utilization_rate()));
            }
        }

        // Pick candidate with lowest utilization rate
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.first().map(|c| c.0.clone())
    }

    /// Compute real-time cluster telemetry metrics
    pub fn cluster_metrics(&self) -> ClusterMetrics {
        let total_nodes = self.peers.len() + 1;
        let mut online_nodes = if self.local_node.status == PeerStatus::Online { 1 } else { 0 };
        let mut total_vram = self.local_node.total_vram_mb;
        let mut avail_vram = self.local_node.available_vram_mb;
        let mut total_tasks = self.local_queue.len();
        let mut total_util = self.local_node.utilization_rate();

        let mut model_set = std::collections::HashSet::new();
        for m in &self.local_node.loaded_models {
            model_set.insert(m.clone());
        }

        for peer in self.peers.values() {
            if peer.status == PeerStatus::Online || peer.status == PeerStatus::Busy {
                online_nodes += 1;
            }
            total_vram += peer.total_vram_mb;
            avail_vram += peer.available_vram_mb;
            total_util += peer.utilization_rate();
            for m in &peer.loaded_models {
                model_set.insert(m.clone());
            }
        }

        for queue in self.peer_queues.values() {
            total_tasks += queue.len();
        }

        let mut available_models: Vec<String> = model_set.into_iter().collect();
        available_models.sort();

        ClusterMetrics {
            total_nodes,
            online_nodes,
            total_vram_mb: total_vram,
            available_vram_mb: avail_vram,
            total_queued_tasks: total_tasks,
            average_utilization: if total_nodes > 0 { total_util / (total_nodes as f64) } else { 0.0 },
            available_models,
        }
    }
}
