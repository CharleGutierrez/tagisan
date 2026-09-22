//! # Living Codebase Hypergraph Digital Twin (`tgs nextgen hypergraph`)
//!
//! Multi-dimensional dependency and execution hypergraph unifying:
//! - Tree-sitter CST / LSP symbol definitions and call-graphs
//! - Git commit lineages and author blast-radii
//! - Runtime execution traces and test coverage mappings
//! - Tarjan SCC cycle detection and blast radius impact analysis

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

/// Kinds of hypergraph code nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HypergraphNodeKind {
    File { path: String, loc: usize },
    Function { name: String, file: String, line: usize },
    Struct { name: String, file: String },
    Trait { name: String, file: String },
    Module { name: String, path: String },
    Test { name: String, target_symbol: String },
    Commit { sha: String, message: String },
}

/// Kinds of directed hypergraph edges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HypergraphEdgeKind {
    Calls,
    Imports,
    Defines,
    Implements,
    ModifiedBy,
    CoveredBy,
}

/// Node representation in the Codebase Hypergraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypergraphNode {
    pub id: String,
    pub kind: HypergraphNodeKind,
    pub metadata: HashMap<String, String>,
}

/// Directed Codebase Hypergraph
pub struct CodebaseHypergraph {
    pub graph: DiGraph<HypergraphNode, HypergraphEdgeKind>,
    pub node_map: HashMap<String, NodeIndex>,
}

impl Default for CodebaseHypergraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CodebaseHypergraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node to the hypergraph
    pub fn add_node(&mut self, id: &str, kind: HypergraphNodeKind) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(id) {
            return idx;
        }

        let node = HypergraphNode {
            id: id.to_string(),
            kind,
            metadata: HashMap::new(),
        };

        let idx = self.graph.add_node(node);
        self.node_map.insert(id.to_string(), idx);
        idx
    }

    /// Add a directed dependency edge
    pub fn add_edge(&mut self, from_id: &str, to_id: &str, kind: HypergraphEdgeKind) {
        if let (Some(&from_idx), Some(&to_idx)) = (self.node_map.get(from_id), self.node_map.get(to_id)) {
            self.graph.add_edge(from_idx, to_idx, kind);
        }
    }

    /// Calculate the forward blast radius (all downstream components affected by a change)
    pub fn calculate_blast_radius(&self, target_id: &str, max_depth: usize) -> Vec<String> {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();

        let start_idx = match self.node_map.get(target_id) {
            Some(&idx) => idx,
            None => return affected,
        };

        let mut queue = VecDeque::new();
        queue.push_back((start_idx, 0));
        visited.insert(start_idx);

        while let Some((curr_idx, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }

            // Incoming edges to curr_idx represent callers/dependents that rely on this node
            let mut neighbors = self.graph.neighbors_directed(curr_idx, Direction::Incoming).detach();
            while let Some((_, neighbor_idx)) = neighbors.next(&self.graph) {
                if !visited.contains(&neighbor_idx) {
                    visited.insert(neighbor_idx);
                    let dep_node = &self.graph[neighbor_idx];
                    affected.push(dep_node.id.clone());
                    queue.push_back((neighbor_idx, depth + 1));
                }
            }
        }

        affected
    }

    /// Detect circular dependency cycles using Tarjan's Strongly Connected Components
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            if scc.len() > 1 {
                let cycle_names = scc.iter().map(|&idx| self.graph[idx].id.clone()).collect();
                cycles.push(cycle_names);
            }
        }

        cycles
    }

    /// Persist graph nodes and edges to SQLite
    pub fn save_to_sqlite(&self, db_path: &Path) -> Result<()> {
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| TagisanError::Execution(format!("SQLite open failed: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS hypergraph_nodes (
                id TEXT PRIMARY KEY,
                kind_json TEXT NOT NULL
            )",
            [],
        ).map_err(|e| TagisanError::Execution(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS hypergraph_edges (
                from_id TEXT NOT NULL,
                to_id TEXT NOT NULL,
                kind_text TEXT NOT NULL
            )",
            [],
        ).map_err(|e| TagisanError::Execution(e.to_string()))?;

        let tx = conn.unchecked_transaction()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;

        for node_idx in self.graph.node_indices() {
            let node = &self.graph[node_idx];
            let kind_json = serde_json::to_string(&node.kind)
                .map_err(|e| TagisanError::Execution(e.to_string()))?;
            tx.execute(
                "INSERT OR REPLACE INTO hypergraph_nodes (id, kind_json) VALUES (?1, ?2)",
                [&node.id, &kind_json],
            ).map_err(|e| TagisanError::Execution(e.to_string()))?;
        }

        for edge_idx in self.graph.edge_indices() {
            if let Some((from_idx, to_idx)) = self.graph.edge_endpoints(edge_idx) {
                let from_id = &self.graph[from_idx].id;
                let to_id = &self.graph[to_idx].id;
                let kind_str = format!("{:?}", self.graph[edge_idx]);
                tx.execute(
                    "INSERT INTO hypergraph_edges (from_id, to_id, kind_text) VALUES (?1, ?2, ?3)",
                    [from_id, to_id, &kind_str],
                ).map_err(|e| TagisanError::Execution(e.to_string()))?;
            }
        }

        tx.commit().map_err(|e| TagisanError::Execution(e.to_string()))?;
        Ok(())
    }
}
