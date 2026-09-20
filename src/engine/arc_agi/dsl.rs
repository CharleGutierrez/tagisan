use super::grid::{ArcGrid, Connectivity, GravityDirection};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DslError {
    #[error("Execution exceeded step limit of {0}")]
    StepLimitExceeded(usize),
    #[error("Execution timed out after {0:?}")]
    Timeout(Duration),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Rotation angles supported by ARC DSL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationAngle {
    Deg90,
    Deg180,
    Deg270,
}

impl fmt::Display for RotationAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RotationAngle::Deg90 => write!(f, "90"),
            RotationAngle::Deg180 => write!(f, "180"),
            RotationAngle::Deg270 => write!(f, "270"),
        }
    }
}

/// Reflection axes supported by ARC DSL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
    DiagonalMain,
    DiagonalAnti,
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Axis::Horizontal => write!(f, "Horizontal"),
            Axis::Vertical => write!(f, "Vertical"),
            Axis::DiagonalMain => write!(f, "DiagonalMain"),
            Axis::DiagonalAnti => write!(f, "DiagonalAnti"),
        }
    }
}

/// Predicate for object filtering operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectPredicate {
    KeepLargest,
    KeepSmallest,
    ByColor(u8),
    MinSize(usize),
    MaxSize(usize),
    MostFrequentColor,
    LeastFrequentColor,
}

impl fmt::Display for ObjectPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectPredicate::KeepLargest => write!(f, "KeepLargest"),
            ObjectPredicate::KeepSmallest => write!(f, "KeepSmallest"),
            ObjectPredicate::ByColor(c) => write!(f, "ByColor({})", c),
            ObjectPredicate::MinSize(s) => write!(f, "MinSize({})", s),
            ObjectPredicate::MaxSize(s) => write!(f, "MaxSize({})", s),
            ObjectPredicate::MostFrequentColor => write!(f, "MostFrequentColor"),
            ObjectPredicate::LeastFrequentColor => write!(f, "LeastFrequentColor"),
        }
    }
}

/// Overlay modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OverlayMode {
    Normal,
    Masked(u8), // treat this color as transparent
    Underlay,
}

impl fmt::Display for OverlayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverlayMode::Normal => write!(f, "Normal"),
            OverlayMode::Masked(c) => write!(f, "Masked({})", c),
            OverlayMode::Underlay => write!(f, "Underlay"),
        }
    }
}

/// Functional, composable ARC DSL operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DslOp {
    Rotate(RotationAngle),
    Reflect(Axis),
    Translate(i32, i32),
    CropBoundingBox,
    CropSubgrid { r: usize, c: usize, h: usize, w: usize },
    Gravity(GravityDirection),
    FillColor(u8, u8), // old, new
    FilterObjects(ObjectPredicate),
    Overlay(OverlayMode),
    Tile(u32),
    Tile2D(u32, u32),
    Pad(usize, u8),
    Scale(usize),
    Outline(u8),
    InvertColors,
}

impl fmt::Display for DslOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DslOp::Rotate(ang) => write!(f, "Rotate({})", ang),
            DslOp::Reflect(axis) => write!(f, "Reflect({})", axis),
            DslOp::Translate(dr, dc) => write!(f, "Translate({}, {})", dr, dc),
            DslOp::CropBoundingBox => write!(f, "CropBoundingBox"),
            DslOp::CropSubgrid { r, c, h, w } => {
                write!(f, "CropSubgrid({}, {}, {}, {})", r, c, h, w)
            }
            DslOp::Gravity(dir) => write!(f, "Gravity({:?})", dir),
            DslOp::FillColor(old, new) => write!(f, "FillColor({}, {})", old, new),
            DslOp::FilterObjects(pred) => write!(f, "FilterObjects({})", pred),
            DslOp::Overlay(mode) => write!(f, "Overlay({})", mode),
            DslOp::Tile(factor) => write!(f, "Tile({})", factor),
            DslOp::Tile2D(fr, fc) => write!(f, "Tile2D({}, {})", fr, fc),
            DslOp::Pad(pad, color) => write!(f, "Pad({}, {})", pad, color),
            DslOp::Scale(factor) => write!(f, "Scale({})", factor),
            DslOp::Outline(color) => write!(f, "Outline({})", color),
            DslOp::InvertColors => write!(f, "InvertColors"),
        }
    }
}

impl DslOp {
    /// Apply single operation to an ARC grid
    pub fn apply(&self, grid: &ArcGrid) -> Result<ArcGrid, DslError> {
        let (h, w) = grid.dims();
        let bg_color = 0; // standard ARC background is 0 (Black)

        match self {
            DslOp::Rotate(RotationAngle::Deg90) => Ok(grid.rotate90()),
            DslOp::Rotate(RotationAngle::Deg180) => Ok(grid.rotate180()),
            DslOp::Rotate(RotationAngle::Deg270) => Ok(grid.rotate270()),

            DslOp::Reflect(Axis::Horizontal) => Ok(grid.reflect_horizontal()),
            DslOp::Reflect(Axis::Vertical) => Ok(grid.reflect_vertical()),
            DslOp::Reflect(Axis::DiagonalMain) => Ok(grid.reflect_diagonal_main()),
            DslOp::Reflect(Axis::DiagonalAnti) => Ok(grid.reflect_diagonal_anti()),

            DslOp::Translate(dr, dc) => Ok(grid.translate(*dr, *dc, bg_color)),

            DslOp::CropBoundingBox => Ok(grid.crop_to_content(bg_color)),

            DslOp::CropSubgrid { r, c, h, w } => Ok(grid.crop_subgrid(*r, *c, *h, *w)),

            DslOp::Gravity(dir) => Ok(grid.simulate_gravity(*dir, bg_color)),

            DslOp::FillColor(old, new) => Ok(grid.replace_color(*old, *new)),

            DslOp::FilterObjects(pred) => {
                let objects = grid.find_objects(Connectivity::FourWay, Some(bg_color), true);
                if objects.is_empty() {
                    return Ok(grid.clone());
                }

                let filtered_objects: Vec<_> = match pred {
                    ObjectPredicate::KeepLargest => {
                        let max_area = objects.iter().map(|o| o.area()).max().unwrap_or(0);
                        objects.into_iter().filter(|o| o.area() == max_area).collect()
                    }
                    ObjectPredicate::KeepSmallest => {
                        let min_area = objects.iter().map(|o| o.area()).min().unwrap_or(0);
                        objects.into_iter().filter(|o| o.area() == min_area).collect()
                    }
                    ObjectPredicate::ByColor(col) => {
                        objects.into_iter().filter(|o| o.color == Some(*col)).collect()
                    }
                    ObjectPredicate::MinSize(min_s) => {
                        objects.into_iter().filter(|o| o.area() >= *min_s).collect()
                    }
                    ObjectPredicate::MaxSize(max_s) => {
                        objects.into_iter().filter(|o| o.area() <= *max_s).collect()
                    }
                    ObjectPredicate::MostFrequentColor => {
                        let counts = grid.color_counts();
                        let max_c = counts
                            .into_iter()
                            .filter(|&(c, _)| c != bg_color)
                            .max_by_key(|&(_, count)| count)
                            .map(|(c, _)| c);
                        objects.into_iter().filter(|o| o.color == max_c).collect()
                    }
                    ObjectPredicate::LeastFrequentColor => {
                        let counts = grid.color_counts();
                        let min_c = counts
                            .into_iter()
                            .filter(|&(c, _)| c != bg_color)
                            .min_by_key(|&(_, count)| count)
                            .map(|(c, _)| c);
                        objects.into_iter().filter(|o| o.color == min_c).collect()
                    }
                };

                let mut res = ArcGrid::with_color(h, w, bg_color);
                for obj in filtered_objects {
                    let col = obj.color.unwrap_or(1);
                    for (r, c) in obj.cells {
                        res.set(r, c, col);
                    }
                }
                Ok(res)
            }

            DslOp::Overlay(mode) => {
                // When overlaying, by default we overlay with non-zero pixels
                let transparent = match mode {
                    OverlayMode::Normal => None,
                    OverlayMode::Masked(c) => Some(*c),
                    OverlayMode::Underlay => Some(0),
                };
                Ok(grid.overlay(grid, transparent))
            }

            DslOp::Tile(factor) => Ok(grid.tile(*factor as usize, *factor as usize)),
            DslOp::Tile2D(fr, fc) => Ok(grid.tile(*fr as usize, *fc as usize)),

            DslOp::Pad(pad, color) => Ok(grid.pad(*pad, *pad, *pad, *pad, *color)),

            DslOp::Scale(factor) => Ok(grid.zoom(*factor)),

            DslOp::Outline(color) => {
                let mut res = grid.clone();
                for r in 0..h {
                    for c in 0..w {
                        if grid.cells[r][c] != bg_color {
                            // check if on boundary of non-bg region
                            let mut on_border = false;
                            for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let nr = r as i32 + dr;
                                let nc = c as i32 + dc;
                                if nr < 0 || nr >= h as i32 || nc < 0 || nc >= w as i32 {
                                    on_border = true;
                                    break;
                                }
                                if grid.cells[nr as usize][nc as usize] == bg_color {
                                    on_border = true;
                                    break;
                                }
                            }
                            if on_border {
                                res.set(r, c, *color);
                            }
                        }
                    }
                }
                Ok(res)
            }

            DslOp::InvertColors => {
                let mut res = grid.clone();
                for r in 0..h {
                    for c in 0..w {
                        let cur = grid.cells[r][c];
                        if cur != 0 {
                            res.set(r, c, (10 - cur).min(9));
                        }
                    }
                }
                Ok(res)
            }
        }
    }
}

/// Ordered sequence of ARC DSL operations with execution engine and safety guards
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DslProgram {
    pub ops: Vec<DslOp>,
    #[serde(default = "default_step_limit")]
    pub step_limit: usize,
}

fn default_step_limit() -> usize {
    50
}

impl DslProgram {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            step_limit: default_step_limit(),
        }
    }

    pub fn from_ops(ops: Vec<DslOp>) -> Self {
        Self {
            ops,
            step_limit: default_step_limit(),
        }
    }

    pub fn with_step_limit(mut self, limit: usize) -> Self {
        self.step_limit = limit;
        self
    }

    pub fn push(&mut self, op: DslOp) {
        self.ops.push(op);
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Execute the DSL program sequentially on the input grid
    pub fn execute(&self, input: &ArcGrid) -> Result<ArcGrid, DslError> {
        self.execute_with_timeout(input, 10_000)
    }

    /// Execute with a strict timeout in milliseconds
    pub fn execute_with_timeout(&self, input: &ArcGrid, timeout_ms: u64) -> Result<ArcGrid, DslError> {
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        if self.ops.len() > self.step_limit {
            return Err(DslError::StepLimitExceeded(self.step_limit));
        }

        let mut current = input.clone();

        for (idx, op) in self.ops.iter().enumerate() {
            if idx >= self.step_limit {
                return Err(DslError::StepLimitExceeded(self.step_limit));
            }

            if start.elapsed() > timeout {
                return Err(DslError::Timeout(timeout));
            }

            current = op.apply(&current)?;
        }

        Ok(current)
    }

    /// Format program as readable code string
    pub fn to_code(&self) -> String {
        if self.ops.is_empty() {
            return "Identity".to_string();
        }
        self.ops
            .iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// Parse a DSL program from code string
    pub fn from_code(code: &str) -> Result<Self, DslError> {
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed == "Identity" {
            return Ok(DslProgram::new());
        }

        let mut ops = Vec::new();
        for part in trimmed.split('|') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part == "CropBoundingBox" {
                ops.push(DslOp::CropBoundingBox);
            } else if part == "InvertColors" {
                ops.push(DslOp::InvertColors);
            } else if part.starts_with("Rotate(") && part.ends_with(')') {
                let inner = &part[7..part.len() - 1].trim();
                match *inner {
                    "90" => ops.push(DslOp::Rotate(RotationAngle::Deg90)),
                    "180" => ops.push(DslOp::Rotate(RotationAngle::Deg180)),
                    "270" => ops.push(DslOp::Rotate(RotationAngle::Deg270)),
                    _ => return Err(DslError::ParseError(format!("Unknown rotation: {}", inner))),
                }
            } else if part.starts_with("Reflect(") && part.ends_with(')') {
                let inner = &part[8..part.len() - 1].trim();
                match *inner {
                    "Horizontal" => ops.push(DslOp::Reflect(Axis::Horizontal)),
                    "Vertical" => ops.push(DslOp::Reflect(Axis::Vertical)),
                    "DiagonalMain" => ops.push(DslOp::Reflect(Axis::DiagonalMain)),
                    "DiagonalAnti" => ops.push(DslOp::Reflect(Axis::DiagonalAnti)),
                    _ => return Err(DslError::ParseError(format!("Unknown reflection: {}", inner))),
                }
            } else if part.starts_with("Translate(") && part.ends_with(')') {
                let inner = &part[10..part.len() - 1];
                let coords: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if coords.len() == 2 {
                    let dr = coords[0].parse::<i32>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    let dc = coords[1].parse::<i32>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    ops.push(DslOp::Translate(dr, dc));
                }
            } else if part.starts_with("Gravity(") && part.ends_with(')') {
                let inner = &part[8..part.len() - 1].trim();
                match *inner {
                    "Down" => ops.push(DslOp::Gravity(GravityDirection::Down)),
                    "Up" => ops.push(DslOp::Gravity(GravityDirection::Up)),
                    "Left" => ops.push(DslOp::Gravity(GravityDirection::Left)),
                    "Right" => ops.push(DslOp::Gravity(GravityDirection::Right)),
                    _ => return Err(DslError::ParseError(format!("Unknown gravity direction: {}", inner))),
                }
            } else if part.starts_with("FillColor(") && part.ends_with(')') {
                let inner = &part[10..part.len() - 1];
                let colors: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if colors.len() == 2 {
                    let old = colors[0].parse::<u8>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    let new = colors[1].parse::<u8>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    ops.push(DslOp::FillColor(old, new));
                }
            } else if part.starts_with("Tile(") && part.ends_with(')') {
                let factor = part[5..part.len() - 1].trim().parse::<u32>().map_err(|e| DslError::ParseError(e.to_string()))?;
                ops.push(DslOp::Tile(factor));
            } else if part.starts_with("Scale(") && part.ends_with(')') {
                let factor = part[6..part.len() - 1].trim().parse::<usize>().map_err(|e| DslError::ParseError(e.to_string()))?;
                ops.push(DslOp::Scale(factor));
            } else if part.starts_with("Pad(") && part.ends_with(')') {
                let inner = &part[4..part.len() - 1];
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let p = parts[0].parse::<usize>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    let c = parts[1].parse::<u8>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    ops.push(DslOp::Pad(p, c));
                }
            } else if part.starts_with("Outline(") && part.ends_with(')') {
                let color = part[8..part.len() - 1].trim().parse::<u8>().map_err(|e| DslError::ParseError(e.to_string()))?;
                ops.push(DslOp::Outline(color));
            } else if part.starts_with("FilterObjects(") && part.ends_with(')') {
                let inner = &part[14..part.len() - 1].trim();
                if *inner == "KeepLargest" {
                    ops.push(DslOp::FilterObjects(ObjectPredicate::KeepLargest));
                } else if *inner == "KeepSmallest" {
                    ops.push(DslOp::FilterObjects(ObjectPredicate::KeepSmallest));
                } else if inner.starts_with("ByColor(") && inner.ends_with(')') {
                    let c = inner[8..inner.len() - 1].trim().parse::<u8>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    ops.push(DslOp::FilterObjects(ObjectPredicate::ByColor(c)));
                } else if inner.starts_with("MinSize(") && inner.ends_with(')') {
                    let s = inner[8..inner.len() - 1].trim().parse::<usize>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    ops.push(DslOp::FilterObjects(ObjectPredicate::MinSize(s)));
                } else if inner.starts_with("MaxSize(") && inner.ends_with(')') {
                    let s = inner[8..inner.len() - 1].trim().parse::<usize>().map_err(|e| DslError::ParseError(e.to_string()))?;
                    ops.push(DslOp::FilterObjects(ObjectPredicate::MaxSize(s)));
                } else if *inner == "MostFrequentColor" {
                    ops.push(DslOp::FilterObjects(ObjectPredicate::MostFrequentColor));
                } else if *inner == "LeastFrequentColor" {
                    ops.push(DslOp::FilterObjects(ObjectPredicate::LeastFrequentColor));
                }
            } else {
                return Err(DslError::ParseError(format!("Unrecognized DSL operation: {}", part)));
            }
        }

        Ok(DslProgram::from_ops(ops))
    }
}
