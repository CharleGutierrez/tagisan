use super::grid::{ArcGrid, Connectivity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Invariant describing size relationship between input and output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SizeInvariant {
    SameSize,
    FixedSize(usize, usize),
    ProportionalScale { factor_h: usize, factor_w: usize },
    NonIncreasing,
    Variable,
}

/// Invariant describing color conservation between input and output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorInvariant {
    SameColors,
    SubsetOfInput,
    StrictNewColors(BTreeSet<u8>),
    AnyColors,
}

/// Invariant describing topological / object properties
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TopologyInvariant {
    ObjectCountPreserved,
    NonBgAreaPreserved,
    None,
}

/// Invariants deduced from all training demonstrations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcTaskInvariants {
    pub size: SizeInvariant,
    pub color: ColorInvariant,
    pub topology: TopologyInvariant,
    pub preserves_horizontal_symmetry: bool,
    pub preserves_vertical_symmetry: bool,
    pub preserves_rotational_180: bool,
}

/// Result of invariant verification on a candidate grid
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvariantVerdict {
    pub is_valid: bool,
    pub score: f64, // 0.0 to 1.0
    pub violations: Vec<String>,
}

impl InvariantVerdict {
    pub fn pass() -> Self {
        Self {
            is_valid: true,
            score: 1.0,
            violations: Vec::new(),
        }
    }

    pub fn fail(violations: Vec<String>, score: f64) -> Self {
        Self {
            is_valid: false,
            score,
            violations,
        }
    }
}

impl ArcTaskInvariants {
    /// Deduce task invariants from training demonstration pairs
    pub fn deduce(pairs: &[(&ArcGrid, &ArcGrid)]) -> Self {
        if pairs.is_empty() {
            return Self {
                size: SizeInvariant::Variable,
                color: ColorInvariant::AnyColors,
                topology: TopologyInvariant::None,
                preserves_horizontal_symmetry: false,
                preserves_vertical_symmetry: false,
                preserves_rotational_180: false,
            };
        }

        // 1. Deduce Size Invariant
        let mut all_same_size = true;
        let mut all_non_increasing = true;
        let first_out_dims = pairs[0].1.dims();
        let mut all_fixed_size = true;

        let mut all_scale = true;
        let (in_h0, in_w0) = pairs[0].0.dims();
        let (out_h0, out_w0) = pairs[0].1.dims();
        let scale_h = if in_h0 > 0 && out_h0 % in_h0 == 0 {
            out_h0 / in_h0
        } else {
            0
        };
        let scale_w = if in_w0 > 0 && out_w0 % in_w0 == 0 {
            out_w0 / in_w0
        } else {
            0
        };

        for (input, output) in pairs {
            if input.dims() != output.dims() {
                all_same_size = false;
            }
            if output.height() > input.height() || output.width() > input.width() {
                all_non_increasing = false;
            }
            if output.dims() != first_out_dims {
                all_fixed_size = false;
            }
            if scale_h == 0
                || scale_w == 0
                || output.height() != input.height() * scale_h
                || output.width() != input.width() * scale_w
            {
                all_scale = false;
            }
        }

        let size = if all_same_size {
            SizeInvariant::SameSize
        } else if all_fixed_size {
            SizeInvariant::FixedSize(first_out_dims.0, first_out_dims.1)
        } else if all_scale && (scale_h > 1 || scale_w > 1) {
            SizeInvariant::ProportionalScale {
                factor_h: scale_h,
                factor_w: scale_w,
            }
        } else if all_non_increasing {
            SizeInvariant::NonIncreasing
        } else {
            SizeInvariant::Variable
        };

        // 2. Deduce Color Invariant
        let mut all_same_colors = true;
        let mut all_subset_colors = true;
        let mut common_new_colors: Option<BTreeSet<u8>> = None;

        for (input, output) in pairs {
            let in_colors = input.unique_colors();
            let out_colors = output.unique_colors();

            if in_colors != out_colors {
                all_same_colors = false;
            }
            if !out_colors.is_subset(&in_colors) {
                all_subset_colors = false;
                let diff: BTreeSet<u8> = out_colors.difference(&in_colors).copied().collect();
                match &mut common_new_colors {
                    Some(set) => {
                        *set = set.intersection(&diff).copied().collect();
                    }
                    None => {
                        common_new_colors = Some(diff);
                    }
                }
            }
        }

        let color = if all_same_colors {
            ColorInvariant::SameColors
        } else if all_subset_colors {
            ColorInvariant::SubsetOfInput
        } else if let Some(new_cols) = common_new_colors {
            if !new_cols.is_empty() {
                ColorInvariant::StrictNewColors(new_cols)
            } else {
                ColorInvariant::AnyColors
            }
        } else {
            ColorInvariant::AnyColors
        };

        // 3. Deduce Topology Invariant
        let mut all_obj_count_preserved = true;
        let mut all_area_preserved = true;

        for (input, output) in pairs {
            let in_objs = input.find_objects(Connectivity::FourWay, Some(0), true);
            let out_objs = output.find_objects(Connectivity::FourWay, Some(0), true);

            if in_objs.len() != out_objs.len() {
                all_obj_count_preserved = false;
            }

            let in_area: usize = in_objs.iter().map(|o| o.area()).sum();
            let out_area: usize = out_objs.iter().map(|o| o.area()).sum();

            if in_area != out_area {
                all_area_preserved = false;
            }
        }

        let topology = if all_obj_count_preserved {
            TopologyInvariant::ObjectCountPreserved
        } else if all_area_preserved {
            TopologyInvariant::NonBgAreaPreserved
        } else {
            TopologyInvariant::None
        };

        // 4. Deduce Symmetry Preservation
        let mut preserves_h = true;
        let mut preserves_v = true;
        let mut preserves_rot180 = true;

        for (input, output) in pairs {
            let in_sym = input.detect_symmetries();
            let out_sym = output.detect_symmetries();

            if in_sym.horizontal && !out_sym.horizontal {
                preserves_h = false;
            }
            if in_sym.vertical && !out_sym.vertical {
                preserves_v = false;
            }
            if in_sym.rotational_180 && !out_sym.rotational_180 {
                preserves_rot180 = false;
            }
        }

        Self {
            size,
            color,
            topology,
            preserves_horizontal_symmetry: preserves_h,
            preserves_vertical_symmetry: preserves_v,
            preserves_rotational_180: preserves_rot180,
        }
    }

    /// Verify candidate prediction against deduced invariants
    pub fn verify_candidate(&self, input: &ArcGrid, candidate: &ArcGrid) -> InvariantVerdict {
        let mut violations = Vec::new();
        let mut penalty = 0.0;

        // Size check
        match &self.size {
            SizeInvariant::SameSize => {
                if candidate.dims() != input.dims() {
                    violations.push(format!(
                        "Size mismatch: expected {:?} (same as input), got {:?}",
                        input.dims(),
                        candidate.dims()
                    ));
                    penalty += 0.4;
                }
            }
            SizeInvariant::FixedSize(fh, fw) => {
                if candidate.dims() != (*fh, *fw) {
                    violations.push(format!(
                        "Size mismatch: expected fixed size ({}, {}), got {:?}",
                        fh,
                        fw,
                        candidate.dims()
                    ));
                    penalty += 0.4;
                }
            }
            SizeInvariant::ProportionalScale { factor_h, factor_w } => {
                let exp_dims = (input.height() * factor_h, input.width() * factor_w);
                if candidate.dims() != exp_dims {
                    violations.push(format!(
                        "Size scale mismatch: expected {:?}, got {:?}",
                        exp_dims,
                        candidate.dims()
                    ));
                    penalty += 0.4;
                }
            }
            SizeInvariant::NonIncreasing => {
                if candidate.height() > input.height() || candidate.width() > input.width() {
                    violations.push(format!(
                        "Size violated NonIncreasing invariant: input {:?}, got {:?}",
                        input.dims(),
                        candidate.dims()
                    ));
                    penalty += 0.2;
                }
            }
            SizeInvariant::Variable => {}
        }

        // Color check
        let in_colors = input.unique_colors();
        let cand_colors = candidate.unique_colors();

        match &self.color {
            ColorInvariant::SameColors => {
                if cand_colors != in_colors {
                    violations.push(format!(
                        "Color mismatch: expected exactly {:?}, got {:?}",
                        in_colors, cand_colors
                    ));
                    penalty += 0.3;
                }
            }
            ColorInvariant::SubsetOfInput => {
                if !cand_colors.is_subset(&in_colors) {
                    violations.push(format!(
                        "Color mismatch: candidate contains colors {:?} not present in input {:?}",
                        cand_colors.difference(&in_colors).collect::<Vec<_>>(),
                        in_colors
                    ));
                    penalty += 0.3;
                }
            }
            ColorInvariant::StrictNewColors(expected_new) => {
                let diff: BTreeSet<u8> = cand_colors.difference(&in_colors).copied().collect();
                if !diff.is_subset(expected_new) {
                    violations.push(format!(
                        "Color mismatch: unexpected new colors {:?} (expected subset of {:?})",
                        diff, expected_new
                    ));
                    penalty += 0.2;
                }
            }
            ColorInvariant::AnyColors => {}
        }

        // Topology check
        match self.topology {
            TopologyInvariant::ObjectCountPreserved => {
                let in_objs = input.find_objects(Connectivity::FourWay, Some(0), true);
                let cand_objs = candidate.find_objects(Connectivity::FourWay, Some(0), true);
                if in_objs.len() != cand_objs.len() {
                    violations.push(format!(
                        "Topology mismatch: expected {} objects, got {}",
                        in_objs.len(),
                        cand_objs.len()
                    ));
                    penalty += 0.2;
                }
            }
            TopologyInvariant::NonBgAreaPreserved => {
                let in_objs = input.find_objects(Connectivity::FourWay, Some(0), true);
                let cand_objs = candidate.find_objects(Connectivity::FourWay, Some(0), true);
                let in_area: usize = in_objs.iter().map(|o| o.area()).sum();
                let cand_area: usize = cand_objs.iter().map(|o| o.area()).sum();
                if in_area != cand_area {
                    violations.push(format!(
                        "Area mismatch: expected non-bg area {}, got {}",
                        in_area, cand_area
                    ));
                    penalty += 0.2;
                }
            }
            TopologyInvariant::None => {}
        }

        // Symmetry preservation check
        let in_sym = input.detect_symmetries();
        let cand_sym = candidate.detect_symmetries();

        if self.preserves_horizontal_symmetry && in_sym.horizontal && !cand_sym.horizontal {
            violations.push("Candidate broke horizontal symmetry".to_string());
            penalty += 0.1;
        }
        if self.preserves_vertical_symmetry && in_sym.vertical && !cand_sym.vertical {
            violations.push("Candidate broke vertical symmetry".to_string());
            penalty += 0.1;
        }
        if self.preserves_rotational_180 && in_sym.rotational_180 && !cand_sym.rotational_180 {
            violations.push("Candidate broke rotational 180 symmetry".to_string());
            penalty += 0.1;
        }

        let is_valid = violations.is_empty();
        let score: f64 = (1.0f64 - penalty).max(0.0f64);

        if is_valid {
            InvariantVerdict::pass()
        } else {
            InvariantVerdict::fail(violations, score)
        }
    }
}
