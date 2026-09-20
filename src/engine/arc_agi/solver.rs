use super::dsl::{Axis, DslOp, DslProgram, RotationAngle};
use super::grid::{ArcGrid, GravityDirection};
use super::mcts::{MctsConfig, MctsSearcher};
use super::neurosymbolic::NeurosymbolicEngine;
use super::verifier::ArcTaskInvariants;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// A single ARC demonstration or test example
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcExample {
    pub input: ArcGrid,
    #[serde(default)]
    pub output: Option<ArcGrid>,
}

impl ArcExample {
    pub fn new(input: ArcGrid, output: Option<ArcGrid>) -> Self {
        Self { input, output }
    }
}

/// A complete ARC task consisting of training demonstrations and test inputs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcTask {
    #[serde(default)]
    pub id: Option<String>,
    pub train: Vec<ArcExample>,
    pub test: Vec<ArcExample>,
}

impl ArcTask {
    pub fn new(train: Vec<ArcExample>, test: Vec<ArcExample>) -> Self {
        Self {
            id: None,
            train,
            test,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Standard top-2 predictions per ARC test input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcPrediction {
    pub candidate_1: ArcGrid,
    pub candidate_2: ArcGrid,
    pub program_1: Option<String>,
    pub program_2: Option<String>,
    pub confidence: u32, // 0 to 100 percentage
}

/// Comprehensive ARC solve result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveResult {
    pub task_id: Option<String>,
    pub predictions: Vec<ArcPrediction>,
    pub solved_training: bool,
    pub strategy_used: String,
    pub synthesized_program: Option<String>,
    pub invariants: ArcTaskInvariants,
    pub duration_ms: u64,
}

pub type ArcSolverResult = SolveResult;

/// Master ARC-AGI-3 Solver
pub struct ArcAgi3Solver {
    pub max_time: Duration,
    pub strategy: String,
}

impl Default for ArcAgi3Solver {
    fn default() -> Self {
        Self {
            max_time: Duration::from_secs(30),
            strategy: "auto".to_string(),
        }
    }
}

impl ArcAgi3Solver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_time(mut self, duration: Duration) -> Self {
        self.max_time = duration;
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = strategy.into();
        self
    }

    /// Solve the ARC task
    pub fn solve(&self, task: &ArcTask) -> Result<SolveResult, String> {
        let start = Instant::now();

        if task.train.is_empty() {
            return Err("Task has no training demonstrations".to_string());
        }
        if task.test.is_empty() {
            return Err("Task has no test inputs".to_string());
        }

        // Collect valid training pairs
        let mut pairs: Vec<(&ArcGrid, &ArcGrid)> = Vec::new();
        for ex in &task.train {
            if let Some(ref out) = ex.output {
                pairs.push((&ex.input, out));
            } else {
                return Err("Training example missing output grid".to_string());
            }
        }

        // Deduce task invariants
        let invariants = ArcTaskInvariants::deduce(&pairs);

        // 1. Fast Path: Check Identity
        let mut is_identity = true;
        for (in_grid, out_grid) in &pairs {
            if *in_grid != *out_grid {
                is_identity = false;
                break;
            }
        }

        if is_identity {
            let mut predictions = Vec::new();
            for test_ex in &task.test {
                predictions.push(ArcPrediction {
                    candidate_1: test_ex.input.clone(),
                    candidate_2: test_ex.input.clone(),
                    program_1: Some("Identity".to_string()),
                    program_2: Some("Identity".to_string()),
                    confidence: 100,
                });
            }

            return Ok(SolveResult {
                task_id: task.id.clone(),
                predictions,
                solved_training: true,
                strategy_used: "fast_identity".to_string(),
                synthesized_program: Some("Identity".to_string()),
                invariants,
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }

        // 2. Fast Path: Single-operation checks (Cellular Gravity, Rotations, Reflections, Crop, FillColor)
        let single_ops = [
            DslOp::Gravity(GravityDirection::Down),
            DslOp::Gravity(GravityDirection::Up),
            DslOp::Gravity(GravityDirection::Left),
            DslOp::Gravity(GravityDirection::Right),
            DslOp::Rotate(RotationAngle::Deg90),
            DslOp::Rotate(RotationAngle::Deg180),
            DslOp::Rotate(RotationAngle::Deg270),
            DslOp::Reflect(Axis::Horizontal),
            DslOp::Reflect(Axis::Vertical),
            DslOp::Reflect(Axis::DiagonalMain),
            DslOp::Reflect(Axis::DiagonalAnti),
            DslOp::CropBoundingBox,
            DslOp::InvertColors,
        ];

        for op in &single_ops {
            let prog = DslProgram::from_ops(vec![op.clone()]);
            let eval = MctsSearcher::evaluate_program(&prog, &pairs, Some(&invariants));
            if eval.exact_match {
                let mut predictions = Vec::new();
                for test_ex in &task.test {
                    let c1 = prog.execute(&test_ex.input).unwrap_or_else(|_| test_ex.input.clone());
                    predictions.push(ArcPrediction {
                        candidate_1: c1.clone(),
                        candidate_2: c1,
                        program_1: Some(prog.to_code()),
                        program_2: Some(prog.to_code()),
                        confidence: 100,
                    });
                }

                return Ok(SolveResult {
                    task_id: task.id.clone(),
                    predictions,
                    solved_training: true,
                    strategy_used: "fast_single_op".to_string(),
                    synthesized_program: Some(prog.to_code()),
                    invariants,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }

        // Fast Path: Single FillColor checks
        for (old_c, new_c) in generate_color_pairs(&pairs) {
            let prog = DslProgram::from_ops(vec![DslOp::FillColor(old_c, new_c)]);
            let eval = MctsSearcher::evaluate_program(&prog, &pairs, Some(&invariants));
            if eval.exact_match {
                let mut predictions = Vec::new();
                for test_ex in &task.test {
                    let c1 = prog.execute(&test_ex.input).unwrap_or_else(|_| test_ex.input.clone());
                    predictions.push(ArcPrediction {
                        candidate_1: c1.clone(),
                        candidate_2: c1,
                        program_1: Some(prog.to_code()),
                        program_2: Some(prog.to_code()),
                        confidence: 100,
                    });
                }

                return Ok(SolveResult {
                    task_id: task.id.clone(),
                    predictions,
                    solved_training: true,
                    strategy_used: "fast_color_swap".to_string(),
                    synthesized_program: Some(prog.to_code()),
                    invariants,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }

        // 3. MCTS Program Synthesis
        let mcts_timeout = self.max_time.saturating_sub(start.elapsed());
        let mcts_config = MctsConfig {
            max_iterations: 3000,
            max_depth: 4,
            exploration_c: 1.414,
            timeout: mcts_timeout,
            max_program_length: 4,
        };

        let mut searcher = MctsSearcher::new(mcts_config);
        let (exact_prog, top_candidates) = searcher.search(&pairs, Some(&invariants));

        if let Some(prog) = exact_prog {
            let mut predictions = Vec::new();
            for test_ex in &task.test {
                let c1 = prog.execute(&test_ex.input).unwrap_or_else(|_| test_ex.input.clone());
                let c2 = if top_candidates.len() > 1 {
                    top_candidates[1].0.execute(&test_ex.input).unwrap_or_else(|_| c1.clone())
                } else {
                    c1.clone()
                };

                predictions.push(ArcPrediction {
                    candidate_1: c1,
                    candidate_2: c2,
                    program_1: Some(prog.to_code()),
                    program_2: top_candidates.get(1).map(|(p, _)| p.to_code()),
                    confidence: 95,
                });
            }

            return Ok(SolveResult {
                task_id: task.id.clone(),
                predictions,
                solved_training: true,
                strategy_used: "mcts_dsl_synthesis".to_string(),
                synthesized_program: Some(prog.to_code()),
                invariants,
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }

        // 4. Neurosymbolic Refinement & Fallback
        // Check if top candidate had decent score (> 0.6)
        let (best_prog, best_score) = top_candidates
            .first()
            .cloned()
            .unwrap_or_else(|| (DslProgram::new(), 0.0));

        let report = NeurosymbolicEngine::verify_in_sandbox(&best_prog, &pairs);
        let mut predictions = Vec::new();

        for test_ex in &task.test {
            let c1 = best_prog
                .execute(&test_ex.input)
                .unwrap_or_else(|_| test_ex.input.clone());

            let c2 = if top_candidates.len() > 1 {
                top_candidates[1]
                    .0
                    .execute(&test_ex.input)
                    .unwrap_or_else(|_| test_ex.input.clone())
            } else {
                test_ex.input.clone()
            };

            predictions.push(ArcPrediction {
                candidate_1: c1,
                candidate_2: c2,
                program_1: Some(best_prog.to_code()),
                program_2: top_candidates.get(1).map(|(p, _)| p.to_code()),
                confidence: (best_score * 100.0) as u32,
            });
        }

        Ok(SolveResult {
            task_id: task.id.clone(),
            predictions,
            solved_training: report.all_passed,
            strategy_used: "neurosymbolic_heuristic".to_string(),
            synthesized_program: Some(best_prog.to_code()),
            invariants,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

fn generate_color_pairs(pairs: &[(&ArcGrid, &ArcGrid)]) -> Vec<(u8, u8)> {
    let mut old_colors = std::collections::BTreeSet::new();
    let mut new_colors = std::collections::BTreeSet::new();

    for (in_grid, out_grid) in pairs {
        old_colors.extend(in_grid.unique_colors());
        new_colors.extend(out_grid.unique_colors());
    }

    let mut result = Vec::new();
    for &old_c in &old_colors {
        for &new_c in &new_colors {
            if old_c != new_c {
                result.push((old_c, new_c));
            }
        }
    }
    result
}
