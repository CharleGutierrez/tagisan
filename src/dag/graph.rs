use crate::dag::node::TaskNode;
use crate::error::{Result, TagisanError};
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::HashMap;

/// Directed Acyclic Graph (DAG) for multi-agent workflows
#[derive(Clone, Debug)]
pub struct WorkflowGraph {
    pub graph: DiGraph<TaskNode, ()>,
    pub(crate) node_map: HashMap<String, NodeIndex>,
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowGraph {
    /// Create a new empty WorkflowGraph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Create a WorkflowGraph with pre-allocated node and edge capacity
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: DiGraph::with_capacity(nodes, edges),
            node_map: HashMap::with_capacity(nodes),
        }
    }

    /// Add a task node into the DAG
    pub fn add_task(&mut self, task: TaskNode) -> Result<NodeIndex> {
        let task_id = task.id.clone();
        if self.node_map.contains_key(&task_id) {
            return Err(TagisanError::Execution(format!(
                "Task with id '{}' already exists in workflow graph",
                task_id
            )));
        }

        let idx = self.graph.add_node(task);
        self.node_map.insert(task_id, idx);
        Ok(idx)
    }

    /// Add a directed dependency edge: `from_id` must execute before `to_id`.
    /// Idempotent: adding an existing edge is a no-op to prevent duplicate execution triggers.
    pub fn add_dependency(&mut self, from_id: &str, to_id: &str) -> Result<()> {
        if from_id == to_id {
            return Err(TagisanError::Execution(format!(
                "Self-referential dependency detected on task '{}'",
                from_id
            )));
        }

        let from_idx = self.node_map.get(from_id).copied().ok_or_else(|| {
            TagisanError::Execution(format!("Dependency source task '{}' not found", from_id))
        })?;

        let to_idx = self.node_map.get(to_id).copied().ok_or_else(|| {
            TagisanError::Execution(format!("Dependency target task '{}' not found", to_id))
        })?;

        // Idempotent edge addition
        if self.graph.contains_edge(from_idx, to_idx) {
            return Ok(());
        }

        self.graph.add_edge(from_idx, to_idx, ());
        Ok(())
    }

    /// Validate the workflow graph: detects cycles and returns task IDs in topological execution order
    pub fn validate(&self) -> Result<Vec<String>> {
        if is_cyclic_directed(&self.graph) {
            return Err(TagisanError::Execution(
                "Cycle detected in workflow DAG. Tasks must form an acyclic graph.".to_string(),
            ));
        }

        match toposort(&self.graph, None) {
            Ok(ordered_indices) => {
                let ordered_ids: Vec<String> = ordered_indices
                    .into_iter()
                    .map(|idx| self.graph[idx].id.clone())
                    .collect();
                Ok(ordered_ids)
            }
            Err(cycle) => Err(TagisanError::Execution(format!(
                "Topological sort failed due to cycle involving node index {:?}",
                cycle.node_id()
            ))),
        }
    }

    /// Get a reference to a task node by ID
    pub fn get_task(&self, id: &str) -> Option<&TaskNode> {
        self.node_map.get(id).and_then(|&idx| self.graph.node_weight(idx))
    }

    /// Get a mutable reference to a task node by ID
    pub fn get_task_mut(&mut self, id: &str) -> Option<&mut TaskNode> {
        if let Some(&idx) = self.node_map.get(id) {
            self.graph.node_weight_mut(idx)
        } else {
            None
        }
    }

    /// Get NodeIndex for a task ID
    pub fn node_index(&self, id: &str) -> Option<NodeIndex> {
        self.node_map.get(id).copied()
    }

    /// List all task IDs registered in the graph
    pub fn task_ids(&self) -> Vec<String> {
        self.node_map.keys().cloned().collect()
    }

    /// Total number of tasks in the graph
    pub fn task_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Total number of dependency edges in the graph
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// True if the graph contains no tasks
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Retrieve all direct upstream dependency task IDs that `id` depends upon (incoming edges)
    pub fn upstream_dependencies(&self, id: &str) -> Result<Vec<String>> {
        let idx = self.node_map.get(id).copied().ok_or_else(|| {
            TagisanError::Execution(format!("Task '{}' not found in workflow graph", id))
        })?;

        let mut upstream: Vec<String> = self
            .graph
            .neighbors_directed(idx, Direction::Incoming)
            .map(|neighbor_idx| self.graph[neighbor_idx].id.clone())
            .collect();
        upstream.sort();
        upstream.dedup();

        Ok(upstream)
    }

    /// Retrieve all direct downstream dependent task IDs that depend upon `id` (outgoing edges)
    pub fn downstream_dependents(&self, id: &str) -> Result<Vec<String>> {
        let idx = self.node_map.get(id).copied().ok_or_else(|| {
            TagisanError::Execution(format!("Task '{}' not found in workflow graph", id))
        })?;

        let mut downstream: Vec<String> = self
            .graph
            .neighbors_directed(idx, Direction::Outgoing)
            .map(|neighbor_idx| self.graph[neighbor_idx].id.clone())
            .collect();
        downstream.sort();
        downstream.dedup();

        Ok(downstream)
    }

    /// Retrieve all root task IDs (tasks with 0 incoming dependencies)
    pub fn root_tasks(&self) -> Vec<String> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .next()
                    .is_none()
            })
            .map(|idx| self.graph[idx].id.clone())
            .collect()
    }

    /// Retrieve all leaf task IDs (tasks with 0 outgoing dependencies)
    pub fn leaf_tasks(&self) -> Vec<String> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .next()
                    .is_none()
            })
            .map(|idx| self.graph[idx].id.clone())
            .collect()
    }
}
