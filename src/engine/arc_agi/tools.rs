use super::solver::{ArcAgi3Solver, ArcTask};
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

/// Tool for solving ARC-AGI-3 reasoning tasks using MCTS, DSL synthesis, and neurosymbolic verification
#[derive(Debug, Clone, Default)]
pub struct ArcAgiSolveTool {
    working_dir: Option<PathBuf>,
}

impl ArcAgiSolveTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for ArcAgiSolveTool {
    fn name(&self) -> &'static str {
        "arc_agi_solve"
    }

    fn description(&self) -> &'static str {
        "Solve ARC-AGI-3 (Abstraction and Reasoning Corpus) tasks using Monte Carlo Tree Search (MCTS), functional DSL program synthesis, and neurosymbolic invariant verification. Produces top-2 predictions for test inputs."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "object",
                    "description": "ARC task JSON object with 'train' and 'test' arrays of grids."
                },
                "path": {
                    "type": "string",
                    "description": "File path to ARC task JSON file."
                },
                "max_time_secs": {
                    "type": "integer",
                    "description": "Maximum search compute time in seconds (default: 30)."
                },
                "strategy": {
                    "type": "string",
                    "enum": ["auto", "mcts", "dsl", "cellular", "neurosymbolic"],
                    "description": "Solver strategy to prioritize (default: 'auto')."
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let max_time_secs = arguments
            .get("max_time_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let strategy = arguments
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        // Resolve task from inline JSON or file path
        let task: ArcTask = if let Some(task_val) = arguments.get("task") {
            serde_json::from_value(task_val.clone())
                .map_err(|e| TagisanError::Execution(format!("Invalid inline task JSON: {}", e)))?
        } else if let Some(path_str) = arguments.get("path").and_then(|v| v.as_str()) {
            let path = if let Some(ref wd) = self.working_dir {
                wd.join(path_str)
            } else {
                PathBuf::from(path_str)
            };

            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| TagisanError::Execution(format!("Failed to read task file at {:?}: {}", path, e)))?;

            serde_json::from_str(&content)
                .map_err(|e| TagisanError::Execution(format!("Failed to parse task JSON: {}", e)))?
        } else {
            return Err(TagisanError::Execution(
                "Must provide either 'task' (inline JSON) or 'path' (file path)".to_string(),
            ));
        };

        let solver = ArcAgi3Solver::new()
            .with_max_time(Duration::from_secs(max_time_secs))
            .with_strategy(strategy);

        let result = solver
            .solve(&task)
            .map_err(|e| TagisanError::Execution(format!("ARC solver error: {}", e)))?;

        // Format predictions with ASCII renders
        let predictions_json: Vec<Value> = result
            .predictions
            .iter()
            .enumerate()
            .map(|(idx, pred)| {
                json!({
                    "test_index": idx + 1,
                    "confidence": format!("{}%", pred.confidence),
                    "candidate_1": {
                        "dimensions": pred.candidate_1.dims(),
                        "grid": pred.candidate_1.cells,
                        "ascii": pred.candidate_1.to_ascii(),
                        "program": pred.program_1
                    },
                    "candidate_2": {
                        "dimensions": pred.candidate_2.dims(),
                        "grid": pred.candidate_2.cells,
                        "ascii": pred.candidate_2.to_ascii(),
                        "program": pred.program_2
                    }
                })
            })
            .collect();

        let response = json!({
            "status": "success",
            "solved_training": result.solved_training,
            "strategy_used": result.strategy_used,
            "synthesized_program": result.synthesized_program,
            "duration_ms": result.duration_ms,
            "invariants": {
                "size": format!("{:?}", result.invariants.size),
                "color": format!("{:?}", result.invariants.color),
                "topology": format!("{:?}", result.invariants.topology),
                "preserves_horizontal_symmetry": result.invariants.preserves_horizontal_symmetry,
                "preserves_vertical_symmetry": result.invariants.preserves_vertical_symmetry,
                "preserves_rotational_180": result.invariants.preserves_rotational_180
            },
            "predictions": predictions_json
        });

        Ok(serde_json::to_string_pretty(&response).unwrap_or_default())
    }
}
