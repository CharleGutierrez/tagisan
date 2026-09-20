use super::dsl::{DslError, DslProgram};
use super::grid::{ArcColor, ArcGrid, Connectivity};
use super::verifier::ArcTaskInvariants;
use serde::{Deserialize, Serialize};

/// Detailed object summary for LLM prompt formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSummary {
    pub index: usize,
    pub color_name: String,
    pub color_id: u8,
    pub area: usize,
    pub bbox_str: String,
    pub centroid: (f64, f64),
}

/// Detailed ASCII grid summary for neurosymbolic induction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSummary {
    pub dims: (usize, usize),
    pub ascii: String,
    pub unique_colors: Vec<(u8, String)>,
    pub objects: Vec<ObjectSummary>,
    pub symmetries: Vec<String>,
}

impl GridSummary {
    pub fn from_grid(grid: &ArcGrid) -> Self {
        let (h, w) = grid.dims();
        let ascii = grid.to_ascii();

        let unique_colors = grid
            .unique_colors()
            .into_iter()
            .map(|c| (c, ArcColor::name(c).to_string()))
            .collect();

        let raw_objects = grid.find_objects(Connectivity::FourWay, Some(0), true);
        let objects = raw_objects
            .into_iter()
            .enumerate()
            .map(|(idx, obj)| {
                let color_id = obj.color.unwrap_or(0);
                let color_name = ArcColor::name(color_id).to_string();
                let bbox_str = format!(
                    "({}, {}) to ({}, {}) [{}x{}]",
                    obj.bbox.min_r,
                    obj.bbox.min_c,
                    obj.bbox.max_r,
                    obj.bbox.max_c,
                    obj.bbox.height(),
                    obj.bbox.width()
                );
                ObjectSummary {
                    index: idx + 1,
                    color_name,
                    color_id,
                    area: obj.area(),
                    bbox_str,
                    centroid: obj.centroid(),
                }
            })
            .collect();

        let sym = grid.detect_symmetries();
        let mut symmetries = Vec::new();
        if sym.horizontal {
            symmetries.push("Horizontal".to_string());
        }
        if sym.vertical {
            symmetries.push("Vertical".to_string());
        }
        if sym.diagonal_main {
            symmetries.push("DiagonalMain".to_string());
        }
        if sym.diagonal_anti {
            symmetries.push("DiagonalAnti".to_string());
        }
        if sym.rotational_90 {
            symmetries.push("Rotational90".to_string());
        }
        if sym.rotational_180 {
            symmetries.push("Rotational180".to_string());
        }

        Self {
            dims: (h, w),
            ascii,
            unique_colors,
            objects,
            symmetries,
        }
    }
}

/// Pixel-level error mismatch
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelError {
    pub row: usize,
    pub col: usize,
    pub expected: u8,
    pub actual: u8,
}

/// Verification result for a single demonstration pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairExecutionResult {
    pub pair_index: usize,
    pub passed: bool,
    pub dim_mismatch: Option<((usize, usize), (usize, usize))>,
    pub pixel_errors: Vec<PixelError>,
    pub total_pixels: usize,
    pub error_count: usize,
    pub diff_ascii: Option<String>,
}

/// Comprehensive sandbox execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecutionReport {
    pub program_code: String,
    pub all_passed: bool,
    pub pass_count: usize,
    pub total_pairs: usize,
    pub pair_results: Vec<PairExecutionResult>,
    pub execution_error: Option<String>,
}

/// Formatter and Neurosymbolic LLM induction engine with self-correcting feedback
pub struct NeurosymbolicEngine;

impl NeurosymbolicEngine {
    /// Format an ARC task into rich prompt context with grid summaries, ASCII art, and invariants
    pub fn format_task_prompt(
        pairs: &[(&ArcGrid, &ArcGrid)],
        test_inputs: &[&ArcGrid],
        invariants: Option<&ArcTaskInvariants>,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str("=== ARC-AGI-3 NEUROSYMBOLIC TASK REASONING ===\n\n");
        prompt.push_str("You are an ARC-AGI-3 Neurosymbolic Reasoner. Your task is to discover the underlying transformation rule and express it as a composable DSL program.\n\n");

        if let Some(inv) = invariants {
            prompt.push_str("--- DEDUCED TASK INVARIANTS ---\n");
            prompt.push_str(&format!("Size Invariant: {:?}\n", inv.size));
            prompt.push_str(&format!("Color Invariant: {:?}\n", inv.color));
            prompt.push_str(&format!("Topology Invariant: {:?}\n", inv.topology));
            prompt.push_str(&format!(
                "Symmetries Preserved: H={}, V={}, Rot180={}\n\n",
                inv.preserves_horizontal_symmetry,
                inv.preserves_vertical_symmetry,
                inv.preserves_rotational_180
            ));
        }

        prompt.push_str("--- TRAINING DEMONSTRATIONS ---\n");
        for (idx, (input, output)) in pairs.iter().enumerate() {
            prompt.push_str(&format!("\n[DEMONSTRATION {}]\n", idx + 1));

            let in_sum = GridSummary::from_grid(input);
            prompt.push_str(&format!(
                "INPUT ({}x{}):\n{}\nUnique Colors: {:?}\n",
                in_sum.dims.0, in_sum.dims.1, in_sum.ascii, in_sum.unique_colors
            ));
            if !in_sum.objects.is_empty() {
                prompt.push_str("Objects Detected:\n");
                for obj in &in_sum.objects {
                    prompt.push_str(&format!(
                        "  - Obj {}: color={} (id={}), area={}, bbox={}\n",
                        obj.index, obj.color_name, obj.color_id, obj.area, obj.bbox_str
                    ));
                }
            }

            let out_sum = GridSummary::from_grid(output);
            prompt.push_str(&format!(
                "\nOUTPUT ({}x{}):\n{}\nUnique Colors: {:?}\n",
                out_sum.dims.0, out_sum.dims.1, out_sum.ascii, out_sum.unique_colors
            ));
        }

        prompt.push_str("\n--- TEST INPUTS ---\n");
        for (idx, test_in) in test_inputs.iter().enumerate() {
            prompt.push_str(&format!("\n[TEST INPUT {}]\n", idx + 1));
            let test_sum = GridSummary::from_grid(test_in);
            prompt.push_str(&format!(
                "INPUT ({}x{}):\n{}\nUnique Colors: {:?}\n",
                test_sum.dims.0, test_sum.dims.1, test_sum.ascii, test_sum.unique_colors
            ));
        }

        prompt.push_str("\n--- DSL GRAMMAR PRIMITIVES ---\n");
        prompt.push_str("Rotate(90|180|270) | Reflect(Horizontal|Vertical|DiagonalMain|DiagonalAnti)\n");
        prompt.push_str("Translate(dr, dc) | CropBoundingBox | Gravity(Up|Down|Left|Right)\n");
        prompt.push_str("FillColor(old, new) | FilterObjects(KeepLargest|KeepSmallest|ByColor(c)|MinSize(s)|MaxSize(s)|MostFrequentColor|LeastFrequentColor)\n");
        prompt.push_str("Tile(factor) | Scale(factor) | Pad(margin, color) | Outline(color) | InvertColors\n\n");
        prompt.push_str("Respond with the exact DSL program string that solves all demonstrations, e.g.:\n");
        prompt.push_str("PROGRAM: Gravity(Down) | FillColor(1, 2)\n");

        prompt
    }

    /// Execute candidate program against all demonstrations in a sandbox and compute pixel-level diffs
    pub fn verify_in_sandbox(
        program: &DslProgram,
        pairs: &[(&ArcGrid, &ArcGrid)],
    ) -> SandboxExecutionReport {
        let mut pair_results = Vec::new();
        let mut pass_count = 0;
        let mut execution_error = None;

        for (idx, &(input, output)) in pairs.iter().enumerate() {
            match program.execute(input) {
                Ok(pred) => {
                    let passed = pred == *output;
                    if passed {
                        pass_count += 1;
                        pair_results.push(PairExecutionResult {
                            pair_index: idx + 1,
                            passed: true,
                            dim_mismatch: None,
                            pixel_errors: Vec::new(),
                            total_pixels: output.height() * output.width(),
                            error_count: 0,
                            diff_ascii: None,
                        });
                    } else if pred.dims() != output.dims() {
                        pair_results.push(PairExecutionResult {
                            pair_index: idx + 1,
                            passed: false,
                            dim_mismatch: Some((pred.dims(), output.dims())),
                            pixel_errors: Vec::new(),
                            total_pixels: output.height() * output.width(),
                            error_count: output.height() * output.width(),
                            diff_ascii: None,
                        });
                    } else {
                        // Compute pixel-level mismatches
                        let mut pixel_errors = Vec::new();
                        let mut diff_str = String::new();

                        for r in 0..output.height() {
                            for c in 0..output.width() {
                                let exp = output.cells[r][c];
                                let act = pred.cells[r][c];
                                if exp != act {
                                    pixel_errors.push(PixelError {
                                        row: r,
                                        col: c,
                                        expected: exp,
                                        actual: act,
                                    });
                                    diff_str.push('X'); // error marker
                                } else {
                                    diff_str.push(ArcColor::ansi_char(act));
                                }
                                diff_str.push(' ');
                            }
                            diff_str.push('\n');
                        }

                        let total = output.height() * output.width();
                        pair_results.push(PairExecutionResult {
                            pair_index: idx + 1,
                            passed: false,
                            dim_mismatch: None,
                            pixel_errors: pixel_errors.clone(),
                            total_pixels: total,
                            error_count: pixel_errors.len(),
                            diff_ascii: Some(diff_str),
                        });
                    }
                }
                Err(e) => {
                    execution_error = Some(e.to_string());
                    pair_results.push(PairExecutionResult {
                        pair_index: idx + 1,
                        passed: false,
                        dim_mismatch: None,
                        pixel_errors: Vec::new(),
                        total_pixels: output.height() * output.width(),
                        error_count: output.height() * output.width(),
                        diff_ascii: None,
                    });
                }
            }
        }

        let all_passed = pass_count == pairs.len() && !pairs.is_empty();

        SandboxExecutionReport {
            program_code: program.to_code(),
            all_passed,
            pass_count,
            total_pairs: pairs.len(),
            pair_results,
            execution_error,
        }
    }

    /// Generate a self-correcting refinement prompt based on pixel diffs and failure reports
    pub fn generate_refinement_prompt(report: &SandboxExecutionReport) -> String {
        let mut prompt = String::new();
        prompt.push_str("=== ARC-AGI-3 SELF-CORRECTING REFINEMENT FEEDBACK ===\n\n");
        prompt.push_str(&format!(
            "Your candidate program `{}` FAILED {} of {} demonstrations.\n\n",
            report.program_code,
            report.total_pairs - report.pass_count,
            report.total_pairs
        ));

        if let Some(ref err) = report.execution_error {
            prompt.push_str(&format!("Runtime Error: {}\n\n", err));
        }

        for res in &report.pair_results {
            if !res.passed {
                prompt.push_str(&format!("--- Demonstration {} Failure ---\n", res.pair_index));
                if let Some(((pred_h, pred_w), (exp_h, exp_w))) = res.dim_mismatch {
                    prompt.push_str(&format!(
                        "Dimension Mismatch: Produced grid of {}x{}, but expected {}x{}.\n",
                        pred_h, pred_w, exp_h, exp_w
                    ));
                    if pred_h > exp_h || pred_w > exp_w {
                        prompt.push_str("Hint: The output is smaller than the candidate. Consider using `CropBoundingBox` or subgrid cropping.\n");
                    } else {
                        prompt.push_str("Hint: The output is larger than the candidate. Consider `Tile` or `Scale` or `Pad`.\n");
                    }
                } else {
                    prompt.push_str(&format!(
                        "Pixel Mismatches: {} out of {} pixels incorrect.\n",
                        res.error_count, res.total_pixels
                    ));
                    prompt.push_str("Sample Mismatches (First 5):\n");
                    for pe in res.pixel_errors.iter().take(5) {
                        prompt.push_str(&format!(
                            "  - At ({}, {}): Expected {} ({}), but got {} ({})\n",
                            pe.row,
                            pe.col,
                            pe.expected,
                            ArcColor::name(pe.expected),
                            pe.actual,
                            ArcColor::name(pe.actual)
                        ));
                    }
                    if let Some(ref diff) = res.diff_ascii {
                        prompt.push_str("\nDiff Visual Map ('X' marks mismatch):\n");
                        prompt.push_str(diff);
                    }
                }
                prompt.push('\n');
            }
        }

        prompt.push_str("Analyze the failure points above. Refine the program to correct all pixel mismatches.\n");
        prompt.push_str("Respond with: PROGRAM: <new_dsl_program>\n");

        prompt
    }

    /// Extract candidate DSL program from LLM text response
    pub fn parse_llm_response(response: &str) -> Result<DslProgram, DslError> {
        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("PROGRAM:") {
                let code = trimmed["PROGRAM:".len()..].trim();
                return DslProgram::from_code(code);
            }
            if trimmed.starts_with("```") {
                continue;
            }
            if trimmed.contains("Rotate(")
                || trimmed.contains("Reflect(")
                || trimmed.contains("Gravity(")
                || trimmed.contains("FillColor(")
                || trimmed.contains("CropBoundingBox")
                || trimmed.contains("Tile(")
            {
                // Try parsing directly
                if let Ok(prog) = DslProgram::from_code(trimmed) {
                    if !prog.is_empty() {
                        return Ok(prog);
                    }
                }
            }
        }
        DslProgram::from_code(response.trim())
    }
}
