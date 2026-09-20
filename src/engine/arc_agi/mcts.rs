use super::dsl::{Axis, DslOp, DslProgram, ObjectPredicate, RotationAngle};
use super::grid::{ArcGrid, GravityDirection};
use super::verifier::ArcTaskInvariants;
use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

/// Configuration for MCTS / Best-First Program Search
#[derive(Debug, Clone)]
pub struct MctsConfig {
    pub max_iterations: usize,
    pub max_depth: usize,
    pub exploration_c: f64,
    pub timeout: Duration,
    pub max_program_length: usize,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            max_iterations: 2000,
            max_depth: 4,
            exploration_c: 1.414,
            timeout: Duration::from_secs(5),
            max_program_length: 4,
        }
    }
}

/// Evaluation score for a candidate program on demonstration pairs
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EvaluationResult {
    pub score: f64,
    pub exact_match: bool,
    pub avg_pixel_match: f64,
}

/// Monte Carlo Tree Search Node
#[derive(Debug, Clone)]
pub struct MctsNode {
    pub program: DslProgram,
    pub visits: u32,
    pub total_value: f64,
    pub best_score: f64,
    pub children: Vec<MctsNode>,
    pub is_expanded: bool,
}

impl MctsNode {
    pub fn new(program: DslProgram) -> Self {
        Self {
            program,
            visits: 0,
            total_value: 0.0,
            best_score: 0.0,
            children: Vec::new(),
            is_expanded: false,
        }
    }

    #[inline]
    pub fn mean_value(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_value / self.visits as f64
        }
    }

    /// Compute UCB1 score for child selection
    pub fn ucb1(&self, parent_visits: u32, c: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        let exploitation = self.mean_value();
        let exploration = c * ((parent_visits as f64).ln() / (self.visits as f64)).sqrt();
        exploitation + exploration
    }
}

/// Monte Carlo Tree Search engine for ARC-AGI program synthesis
pub struct MctsSearcher {
    pub config: MctsConfig,
    pub candidates_pool: Vec<DslOp>,
}

impl MctsSearcher {
    pub fn new(config: MctsConfig) -> Self {
        Self {
            config,
            candidates_pool: Vec::new(),
        }
    }

    /// Generate domain-specific candidate operations tailored to the task pairs
    pub fn populate_action_space(&mut self, pairs: &[(&ArcGrid, &ArcGrid)]) {
        let mut ops = Vec::new();

        // 1. Basic geometric transforms
        ops.push(DslOp::Rotate(RotationAngle::Deg90));
        ops.push(DslOp::Rotate(RotationAngle::Deg180));
        ops.push(DslOp::Rotate(RotationAngle::Deg270));
        ops.push(DslOp::Reflect(Axis::Horizontal));
        ops.push(DslOp::Reflect(Axis::Vertical));
        ops.push(DslOp::Reflect(Axis::DiagonalMain));
        ops.push(DslOp::Reflect(Axis::DiagonalAnti));

        // 2. Cellular gravity
        ops.push(DslOp::Gravity(GravityDirection::Down));
        ops.push(DslOp::Gravity(GravityDirection::Up));
        ops.push(DslOp::Gravity(GravityDirection::Left));
        ops.push(DslOp::Gravity(GravityDirection::Right));

        // 3. Translations (small deltas)
        for dr in [-1, 1, 0] {
            for dc in [-1, 1, 0] {
                if dr != 0 || dc != 0 {
                    ops.push(DslOp::Translate(dr, dc));
                }
            }
        }

        // 4. Crop
        ops.push(DslOp::CropBoundingBox);

        // 5. Inversion
        ops.push(DslOp::InvertColors);

        // 6. Object filters
        ops.push(DslOp::FilterObjects(ObjectPredicate::KeepLargest));
        ops.push(DslOp::FilterObjects(ObjectPredicate::KeepSmallest));
        ops.push(DslOp::FilterObjects(ObjectPredicate::MostFrequentColor));
        ops.push(DslOp::FilterObjects(ObjectPredicate::LeastFrequentColor));

        // 7. Extract colors from pairs for FillColor and Object filters
        let mut input_colors = BTreeSet::new();
        let mut output_colors = BTreeSet::new();

        for (in_grid, out_grid) in pairs {
            input_colors.extend(in_grid.unique_colors());
            output_colors.extend(out_grid.unique_colors());
        }

        for &c in &output_colors {
            ops.push(DslOp::Outline(c));
            ops.push(DslOp::FilterObjects(ObjectPredicate::ByColor(c)));
        }

        for &old_c in &input_colors {
            for &new_c in &output_colors {
                if old_c != new_c {
                    ops.push(DslOp::FillColor(old_c, new_c));
                }
            }
        }

        // 8. Tiling / scaling if applicable
        let mut max_scale_r = 1;
        let mut max_scale_c = 1;
        for (in_grid, out_grid) in pairs {
            if in_grid.height() > 0 && out_grid.height() % in_grid.height() == 0 {
                max_scale_r = max_scale_r.max(out_grid.height() / in_grid.height());
            }
            if in_grid.width() > 0 && out_grid.width() % in_grid.width() == 0 {
                max_scale_c = max_scale_c.max(out_grid.width() / in_grid.width());
            }
        }

        if max_scale_r > 1 && max_scale_r == max_scale_c && max_scale_r <= 5 {
            ops.push(DslOp::Tile(max_scale_r as u32));
            ops.push(DslOp::Scale(max_scale_r));
        } else if (max_scale_r > 1 || max_scale_c > 1) && max_scale_r <= 5 && max_scale_c <= 5 {
            ops.push(DslOp::Tile2D(max_scale_r as u32, max_scale_c as u32));
        }

        self.candidates_pool = ops;
    }

    /// Evaluate a candidate program across all training demonstrations
    pub fn evaluate_program(
        program: &DslProgram,
        pairs: &[(&ArcGrid, &ArcGrid)],
        invariants: Option<&ArcTaskInvariants>,
    ) -> EvaluationResult {
        if pairs.is_empty() {
            return EvaluationResult {
                score: 0.0,
                exact_match: false,
                avg_pixel_match: 0.0,
            };
        }

        let mut all_exact = true;
        let mut total_pair_score = 0.0;
        let mut total_pixel_match = 0.0;

        for &(input, target) in pairs {
            match program.execute(input) {
                Ok(pred) => {
                    let exact = pred == *target;
                    if !exact {
                        all_exact = false;
                    }

                    if pred.dims() == target.dims() {
                        let match_ratio = pred.pixel_match_ratio(target);
                        total_pixel_match += match_ratio;

                        let color_overlap = color_histogram_overlap(&pred, target);
                        let pair_score = 0.75 * match_ratio + 0.25 * color_overlap;

                        // Invariant penalty check
                        let inv_factor = if let Some(inv) = invariants {
                            inv.verify_candidate(input, &pred).score
                        } else {
                            1.0
                        };

                        total_pair_score += pair_score * inv_factor;
                    } else {
                        all_exact = false;
                        let (ph, pw) = pred.dims();
                        let (th, tw) = target.dims();
                        let min_area = (ph.min(th) * pw.min(tw)) as f64;
                        let max_area = (ph.max(th) * pw.max(tw)) as f64;
                        let dim_overlap = if max_area > 0.0 {
                            min_area / max_area
                        } else {
                            0.0
                        };
                        total_pair_score += 0.15 * dim_overlap;
                    }
                }
                Err(_) => {
                    all_exact = false;
                    // Error gets 0 score
                }
            }
        }

        let n = pairs.len() as f64;
        let avg_score = total_pair_score / n;
        let avg_pixel = total_pixel_match / n;

        EvaluationResult {
            score: if all_exact { 1.0 } else { avg_score },
            exact_match: all_exact,
            avg_pixel_match: avg_pixel,
        }
    }

    /// Run MCTS search to synthesize a DSL program solving all training pairs
    pub fn search(
        &mut self,
        pairs: &[(&ArcGrid, &ArcGrid)],
        invariants: Option<&ArcTaskInvariants>,
    ) -> (Option<DslProgram>, Vec<(DslProgram, f64)>) {
        let start_time = Instant::now();
        self.populate_action_space(pairs);

        let mut root = MctsNode::new(DslProgram::new());
        let initial_eval = Self::evaluate_program(&root.program, pairs, invariants);
        if initial_eval.exact_match {
            return (Some(root.program.clone()), vec![(root.program, 1.0)]);
        }

        let mut best_program: Option<DslProgram> = None;
        let mut best_score = initial_eval.score;
        let mut top_candidates: Vec<(DslProgram, f64)> = vec![(root.program.clone(), initial_eval.score)];

        // Run MCTS iterations
        for _iteration in 0..self.config.max_iterations {
            if start_time.elapsed() >= self.config.timeout {
                break;
            }

            // 1. Selection & Expansion
            let mut path = Vec::new();
            path.push(0); // root index in path

            // Expand root if not expanded
            if !root.is_expanded {
                root.children = self
                    .candidates_pool
                    .iter()
                    .map(|op| {
                        let mut p = root.program.clone();
                        p.push(op.clone());
                        MctsNode::new(p)
                    })
                    .collect();
                root.is_expanded = true;
            }

            // Select child with highest UCB1
            if root.children.is_empty() {
                break;
            }

            let mut best_child_idx = 0;
            let mut best_ucb1 = f64::NEG_INFINITY;

            for (idx, child) in root.children.iter().enumerate() {
                let u = child.ucb1(root.visits, self.config.exploration_c);
                if u > best_ucb1 {
                    best_ucb1 = u;
                    best_child_idx = idx;
                }
            }

            // Explore / evaluate selected child
            let child = &mut root.children[best_child_idx];
            let eval = Self::evaluate_program(&child.program, pairs, invariants);

            // Update child stats
            child.visits += 1;
            child.total_value += eval.score;
            child.best_score = child.best_score.max(eval.score);

            // Update root stats
            root.visits += 1;
            root.total_value += eval.score;
            root.best_score = root.best_score.max(eval.score);

            // Check if better program found
            if eval.score > best_score {
                best_score = eval.score;
                best_program = Some(child.program.clone());
                top_candidates.push((child.program.clone(), eval.score));
            }

            // Check exact match termination
            if eval.exact_match {
                return (Some(child.program.clone()), vec![(child.program.clone(), 1.0)]);
            }

            // Secondary expansion if depth permits
            if child.visits >= 2
                && !child.is_expanded
                && child.program.len() < self.config.max_program_length
            {
                child.children = self
                    .candidates_pool
                    .iter()
                    .map(|op| {
                        let mut p = child.program.clone();
                        p.push(op.clone());
                        MctsNode::new(p)
                    })
                    .collect();
                child.is_expanded = true;
            } else if child.is_expanded && !child.children.is_empty() {
                // Explore sub-child
                let mut sub_best_idx = 0;
                let mut sub_best_ucb1 = f64::NEG_INFINITY;
                for (sidx, sub_child) in child.children.iter().enumerate() {
                    let su = sub_child.ucb1(child.visits, self.config.exploration_c);
                    if su > sub_best_ucb1 {
                        sub_best_ucb1 = su;
                        sub_best_idx = sidx;
                    }
                }

                let sub_child = &mut child.children[sub_best_idx];
                let sub_eval = Self::evaluate_program(&sub_child.program, pairs, invariants);

                sub_child.visits += 1;
                sub_child.total_value += sub_eval.score;
                sub_child.best_score = sub_child.best_score.max(sub_eval.score);

                child.visits += 1;
                child.total_value += sub_eval.score;

                root.visits += 1;
                root.total_value += sub_eval.score;

                if sub_eval.score > best_score {
                    best_score = sub_eval.score;
                    best_program = Some(sub_child.program.clone());
                    top_candidates.push((sub_child.program.clone(), sub_eval.score));
                }

                if sub_eval.exact_match {
                    return (
                        Some(sub_child.program.clone()),
                        vec![(sub_child.program.clone(), 1.0)],
                    );
                }
            }
        }

        // Sort top candidates descending by score
        top_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_candidates.dedup_by(|a, b| a.0 == b.0);
        top_candidates.truncate(5);

        (best_program, top_candidates)
    }
}

/// Compute color histogram overlap between two grids (0.0 to 1.0)
fn color_histogram_overlap(a: &ArcGrid, b: &ArcGrid) -> f64 {
    let counts_a = a.color_counts();
    let counts_b = b.color_counts();
    let total_a: usize = counts_a.values().sum();
    let total_b: usize = counts_b.values().sum();

    if total_a == 0 && total_b == 0 {
        return 1.0;
    }
    if total_a == 0 || total_b == 0 {
        return 0.0;
    }

    let mut overlap = 0.0;
    for (&color, &count_a) in &counts_a {
        let count_b = counts_b.get(&color).copied().unwrap_or(0);
        let freq_a = count_a as f64 / total_a as f64;
        let freq_b = count_b as f64 / total_b as f64;
        overlap += freq_a.min(freq_b);
    }
    overlap
}
