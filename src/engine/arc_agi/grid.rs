use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;

/// 10 Standard ARC-AGI Colors (0-9)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArcColor {
    Black = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Gray = 5,
    Magenta = 6,
    Orange = 7,
    Azure = 8,
    Maroon = 9,
}

impl ArcColor {
    pub const ALL: [ArcColor; 10] = [
        ArcColor::Black,
        ArcColor::Blue,
        ArcColor::Red,
        ArcColor::Green,
        ArcColor::Yellow,
        ArcColor::Gray,
        ArcColor::Magenta,
        ArcColor::Orange,
        ArcColor::Azure,
        ArcColor::Maroon,
    ];

    #[inline]
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(ArcColor::Black),
            1 => Some(ArcColor::Blue),
            2 => Some(ArcColor::Red),
            3 => Some(ArcColor::Green),
            4 => Some(ArcColor::Yellow),
            5 => Some(ArcColor::Gray),
            6 => Some(ArcColor::Magenta),
            7 => Some(ArcColor::Orange),
            8 => Some(ArcColor::Azure),
            9 => Some(ArcColor::Maroon),
            _ => None,
        }
    }

    #[inline]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn name(val: u8) -> &'static str {
        match val {
            0 => "Black",
            1 => "Blue",
            2 => "Red",
            3 => "Green",
            4 => "Yellow",
            5 => "Gray",
            6 => "Magenta",
            7 => "Orange",
            8 => "Azure",
            9 => "Maroon",
            _ => "Unknown",
        }
    }

    pub fn hex(val: u8) -> &'static str {
        match val {
            0 => "#000000",
            1 => "#0074D9",
            2 => "#FF4136",
            3 => "#2ECC40",
            4 => "#FFDC00",
            5 => "#AAAAAA",
            6 => "#F012BE",
            7 => "#FF851B",
            8 => "#7FDBFF",
            9 => "#870C25",
            _ => "#FFFFFF",
        }
    }

    pub fn ansi_char(val: u8) -> char {
        match val {
            0 => '.',
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            _ => '?',
        }
    }

    pub fn ansi_colored_char(val: u8) -> String {
        use colored::*;
        let s = format!("{}", val);
        match val {
            0 => ".".dimmed().to_string(),
            1 => s.blue().bold().to_string(),
            2 => s.red().bold().to_string(),
            3 => s.green().bold().to_string(),
            4 => s.yellow().bold().to_string(),
            5 => s.white().to_string(),
            6 => s.magenta().bold().to_string(),
            7 => s.bright_red().bold().to_string(),
            8 => s.cyan().bold().to_string(),
            9 => s.bright_yellow().bold().to_string(),
            _ => s,
        }
    }
}

/// Bounding box of a region or object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min_r: usize,
    pub min_c: usize,
    pub max_r: usize,
    pub max_c: usize,
}

impl BoundingBox {
    #[inline]
    pub fn new(min_r: usize, min_c: usize, max_r: usize, max_c: usize) -> Self {
        Self {
            min_r,
            min_c,
            max_r,
            max_c,
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        if self.max_r >= self.min_r {
            self.max_r - self.min_r + 1
        } else {
            0
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        if self.max_c >= self.min_c {
            self.max_c - self.min_c + 1
        } else {
            0
        }
    }

    #[inline]
    pub fn area(&self) -> usize {
        self.height() * self.width()
    }

    #[inline]
    pub fn contains(&self, r: usize, c: usize) -> bool {
        r >= self.min_r && r <= self.max_r && c >= self.min_c && c <= self.max_c
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min_r: self.min_r.min(other.min_r),
            min_c: self.min_c.min(other.min_c),
            max_r: self.max_r.max(other.max_r),
            max_c: self.max_c.max(other.max_c),
        }
    }

    pub fn intersect(&self, other: &BoundingBox) -> Option<BoundingBox> {
        let min_r = self.min_r.max(other.min_r);
        let min_c = self.min_c.max(other.min_c);
        let max_r = self.max_r.min(other.max_r);
        let max_c = self.max_c.min(other.max_c);

        if min_r <= max_r && min_c <= max_c {
            Some(BoundingBox {
                min_r,
                min_c,
                max_r,
                max_c,
            })
        } else {
            None
        }
    }
}

/// Discrete 2D grid representation for ARC-AGI
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArcGrid {
    pub cells: Vec<Vec<u8>>,
}

impl Serialize for ArcGrid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.cells.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ArcGrid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cells = Vec::<Vec<u8>>::deserialize(deserializer)?;
        Ok(ArcGrid { cells })
    }
}

impl fmt::Debug for ArcGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ArcGrid({}x{})\n{}",
            self.height(),
            self.width(),
            self.to_ascii()
        )
    }
}

impl fmt::Display for ArcGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ascii())
    }
}

impl ArcGrid {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            cells: vec![vec![0; width]; height],
        }
    }

    pub fn with_color(height: usize, width: usize, color: u8) -> Self {
        Self {
            cells: vec![vec![color; width]; height],
        }
    }

    pub fn from_vec2d(cells: Vec<Vec<u8>>) -> Self {
        Self { cells }
    }

    pub fn from_slice(slice: &[&[u8]]) -> Self {
        let cells = slice.iter().map(|row| row.to_vec()).collect();
        Self { cells }
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    pub fn width(&self) -> usize {
        if self.cells.is_empty() {
            0
        } else {
            self.cells[0].len()
        }
    }

    #[inline]
    pub fn dims(&self) -> (usize, usize) {
        (self.height(), self.width())
    }

    #[inline]
    pub fn get(&self, r: usize, c: usize) -> Option<u8> {
        self.cells.get(r).and_then(|row| row.get(c)).copied()
    }

    #[inline]
    pub fn set(&mut self, r: usize, c: usize, val: u8) -> bool {
        if r < self.height() && c < self.width() {
            self.cells[r][c] = val;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn is_valid_coord(&self, r: usize, c: usize) -> bool {
        r < self.height() && c < self.width()
    }

    pub fn count_color(&self, color: u8) -> usize {
        let mut count = 0;
        for row in &self.cells {
            for &c in row {
                if c == color {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn unique_colors(&self) -> BTreeSet<u8> {
        let mut colors = BTreeSet::new();
        for row in &self.cells {
            for &c in row {
                colors.insert(c);
            }
        }
        colors
    }

    pub fn color_counts(&self) -> HashMap<u8, usize> {
        let mut counts = HashMap::new();
        for row in &self.cells {
            for &c in row {
                *counts.entry(c).or_insert(0) += 1;
            }
        }
        counts
    }

    pub fn dominant_color(&self) -> u8 {
        let counts = self.color_counts();
        counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(color, _)| color)
            .unwrap_or(0)
    }

    pub fn hamming_distance(&self, other: &ArcGrid) -> usize {
        if self.dims() != other.dims() {
            return usize::MAX;
        }
        let mut diff = 0;
        for r in 0..self.height() {
            for c in 0..self.width() {
                if self.cells[r][c] != other.cells[r][c] {
                    diff += 1;
                }
            }
        }
        diff
    }

    pub fn pixel_match_ratio(&self, other: &ArcGrid) -> f64 {
        if self.dims() != other.dims() {
            return 0.0;
        }
        let total = self.height() * self.width();
        if total == 0 {
            return 1.0;
        }
        let distance = self.hamming_distance(other);
        (total.saturating_sub(distance)) as f64 / total as f64
    }

    pub fn to_ascii(&self) -> String {
        let mut out = String::new();
        for row in &self.cells {
            for &c in row {
                out.push(ArcColor::ansi_char(c));
                out.push(' ');
            }
            if !row.is_empty() {
                out.pop();
            }
            out.push('\n');
        }
        out
    }

    pub fn to_colored_ascii(&self) -> String {
        let mut out = String::new();
        for row in &self.cells {
            for &c in row {
                out.push_str(&ArcColor::ansi_colored_char(c));
                out.push(' ');
            }
            if !row.is_empty() {
                out.pop();
            }
            out.push('\n');
        }
        out
    }

    pub fn from_ascii(s: &str) -> Result<Self, String> {
        let mut rows = Vec::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut row = Vec::new();
            for token in line.split_whitespace() {
                if token.len() == 1 {
                    let ch = token.chars().next().unwrap();
                    if ch == '.' {
                        row.push(0);
                    } else if let Some(d) = ch.to_digit(10) {
                        row.push(d as u8);
                    } else {
                        return Err(format!("Invalid token in ARC ASCII: {}", token));
                    }
                } else {
                    let val = token
                        .parse::<u8>()
                        .map_err(|e| format!("Parse error: {}", e))?;
                    row.push(val);
                }
            }
            if !row.is_empty() {
                rows.push(row);
            }
        }
        if rows.is_empty() {
            return Ok(ArcGrid::new(0, 0));
        }
        let w = rows[0].len();
        for r in &rows {
            if r.len() != w {
                return Err(format!("Inconsistent row width: expected {}, got {}", w, r.len()));
            }
        }
        Ok(ArcGrid { cells: rows })
    }

    // =========================================================================
    // Geometric Transformations
    // =========================================================================

    /// Rotate 90 degrees clockwise
    pub fn rotate90(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; h]; w];
        for r in 0..h {
            for c in 0..w {
                new_cells[c][h - 1 - r] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Rotate 180 degrees
    pub fn rotate180(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; w]; h];
        for r in 0..h {
            for c in 0..w {
                new_cells[h - 1 - r][w - 1 - c] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Rotate 270 degrees clockwise (90 counter-clockwise)
    pub fn rotate270(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; h]; w];
        for r in 0..h {
            for c in 0..w {
                new_cells[w - 1 - c][r] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Reflect across horizontal centerline (flip vertically / swap rows)
    pub fn reflect_horizontal(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; w]; h];
        for r in 0..h {
            for c in 0..w {
                new_cells[h - 1 - r][c] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Reflect across vertical centerline (flip horizontally / swap columns)
    pub fn reflect_vertical(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; w]; h];
        for r in 0..h {
            for c in 0..w {
                new_cells[r][w - 1 - c] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Reflect across main diagonal (transpose): (r, c) -> (c, r)
    pub fn reflect_diagonal_main(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; h]; w];
        for r in 0..h {
            for c in 0..w {
                new_cells[c][r] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Reflect across anti-diagonal: (r, c) -> (w - 1 - c, h - 1 - r)
    pub fn reflect_diagonal_anti(&self) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![0; h]; w];
        for r in 0..h {
            for c in 0..w {
                new_cells[w - 1 - c][h - 1 - r] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Translate by (dr, dc), filling empty spaces with `fill`
    pub fn translate(&self, dr: i32, dc: i32, fill: u8) -> ArcGrid {
        let (h, w) = self.dims();
        let mut new_cells = vec![vec![fill; w]; h];
        for r in 0..h {
            for c in 0..w {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                    new_cells[nr as usize][nc as usize] = self.cells[r][c];
                }
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Crop to specified bounding box
    pub fn crop(&self, bbox: &BoundingBox) -> ArcGrid {
        let h = bbox.height();
        let w = bbox.width();
        if h == 0 || w == 0 {
            return ArcGrid::new(0, 0);
        }
        let mut new_cells = vec![vec![0; w]; h];
        for r in 0..h {
            for c in 0..w {
                let src_r = bbox.min_r + r;
                let src_c = bbox.min_c + c;
                if src_r < self.height() && src_c < self.width() {
                    new_cells[r][c] = self.cells[src_r][src_c];
                }
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Crop a subgrid starting at (r, c) with dimensions (h, w)
    pub fn crop_subgrid(&self, r: usize, c: usize, h: usize, w: usize) -> ArcGrid {
        let bbox = BoundingBox::new(r, c, r.saturating_add(h).saturating_sub(1), c.saturating_add(w).saturating_sub(1));
        self.crop(&bbox)
    }

    /// Find bounding box of all non-background pixels
    pub fn content_bounding_box(&self, bg_color: u8) -> Option<BoundingBox> {
        let (h, w) = self.dims();
        let mut min_r = usize::MAX;
        let mut min_c = usize::MAX;
        let mut max_r = 0;
        let mut max_c = 0;
        let mut found = false;

        for r in 0..h {
            for c in 0..w {
                if self.cells[r][c] != bg_color {
                    found = true;
                    min_r = min_r.min(r);
                    min_c = min_c.min(c);
                    max_r = max_r.max(r);
                    max_c = max_c.max(c);
                }
            }
        }

        if found {
            Some(BoundingBox::new(min_r, min_c, max_r, max_c))
        } else {
            None
        }
    }

    /// Crop to the bounding box of non-background pixels (defaults to 0 if all background)
    pub fn crop_to_content(&self, bg_color: u8) -> ArcGrid {
        match self.content_bounding_box(bg_color) {
            Some(bbox) => self.crop(&bbox),
            None => ArcGrid::new(1, 1),
        }
    }

    /// Pad grid with margins (top, bottom, left, right) filled with `fill`
    pub fn pad(&self, top: usize, bottom: usize, left: usize, right: usize, fill: u8) -> ArcGrid {
        let (h, w) = self.dims();
        let new_h = h + top + bottom;
        let new_w = w + left + right;
        let mut new_cells = vec![vec![fill; new_w]; new_h];

        for r in 0..h {
            for c in 0..w {
                new_cells[top + r][left + c] = self.cells[r][c];
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Zoom / upscale grid by an integer factor
    pub fn zoom(&self, factor: usize) -> ArcGrid {
        if factor == 0 {
            return ArcGrid::new(0, 0);
        }
        if factor == 1 {
            return self.clone();
        }
        let (h, w) = self.dims();
        let new_h = h * factor;
        let new_w = w * factor;
        let mut new_cells = vec![vec![0; new_w]; new_h];

        for r in 0..h {
            for c in 0..w {
                let color = self.cells[r][c];
                for zr in 0..factor {
                    for zc in 0..factor {
                        new_cells[r * factor + zr][c * factor + zc] = color;
                    }
                }
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Tile / repeat grid factor_r times vertically and factor_c times horizontally
    pub fn tile(&self, factor_r: usize, factor_c: usize) -> ArcGrid {
        if factor_r == 0 || factor_c == 0 {
            return ArcGrid::new(0, 0);
        }
        let (h, w) = self.dims();
        let new_h = h * factor_r;
        let new_w = w * factor_c;
        let mut new_cells = vec![vec![0; new_w]; new_h];

        for tr in 0..factor_r {
            for tc in 0..factor_c {
                for r in 0..h {
                    for c in 0..w {
                        new_cells[tr * h + r][tc * w + c] = self.cells[r][c];
                    }
                }
            }
        }
        ArcGrid { cells: new_cells }
    }

    /// Replace all occurrences of `old` color with `new_color`
    pub fn replace_color(&self, old: u8, new_color: u8) -> ArcGrid {
        let mut next = self.clone();
        for row in &mut next.cells {
            for cell in row {
                if *cell == old {
                    *cell = new_color;
                }
            }
        }
        next
    }

    /// Overlay another grid on top of this grid. If `transparent` is Some(c), pixels of color `c` in `other` are ignored.
    pub fn overlay(&self, other: &ArcGrid, transparent: Option<u8>) -> ArcGrid {
        let (h, w) = self.dims();
        let mut next = self.clone();
        let oh = other.height().min(h);
        let ow = other.width().min(w);

        for r in 0..oh {
            for c in 0..ow {
                let oc = other.cells[r][c];
                if Some(oc) != transparent {
                    next.cells[r][c] = oc;
                }
            }
        }
        next
    }

    // =========================================================================
    // Symmetry Detection
    // =========================================================================

    pub fn detect_symmetries(&self) -> ArcSymmetryReport {
        let (h, w) = self.dims();
        let is_square = h == w;

        let horizontal = *self == self.reflect_horizontal();
        let vertical = *self == self.reflect_vertical();
        let diagonal_main = is_square && *self == self.reflect_diagonal_main();
        let diagonal_anti = is_square && *self == self.reflect_diagonal_anti();

        let rotational_90 = is_square && *self == self.rotate90();
        let rotational_180 = *self == self.rotate180();
        let rotational_270 = is_square && *self == self.rotate270();

        ArcSymmetryReport {
            horizontal,
            vertical,
            diagonal_main,
            diagonal_anti,
            rotational_90,
            rotational_180,
            rotational_270,
        }
    }

    // =========================================================================
    // Gravity Simulation
    // =========================================================================

    /// Simulate cellular gravity in the given direction.
    /// Non-background cells fall until blocked by boundary or obstacle/non-bg cell.
    pub fn simulate_gravity(&self, dir: GravityDirection, bg_color: u8) -> ArcGrid {
        let (h, w) = self.dims();
        let mut res = ArcGrid::with_color(h, w, bg_color);

        match dir {
            GravityDirection::Down => {
                for c in 0..w {
                    let mut write_r = h as i32 - 1;
                    for r in (0..h).rev() {
                        let color = self.cells[r][c];
                        if color != bg_color {
                            if write_r >= 0 {
                                res.cells[write_r as usize][c] = color;
                                write_r -= 1;
                            }
                        }
                    }
                }
            }
            GravityDirection::Up => {
                for c in 0..w {
                    let mut write_r = 0;
                    for r in 0..h {
                        let color = self.cells[r][c];
                        if color != bg_color {
                            if write_r < h {
                                res.cells[write_r][c] = color;
                                write_r += 1;
                            }
                        }
                    }
                }
            }
            GravityDirection::Right => {
                for r in 0..h {
                    let mut write_c = w as i32 - 1;
                    for c in (0..w).rev() {
                        let color = self.cells[r][c];
                        if color != bg_color {
                            if write_c >= 0 {
                                res.cells[r][write_c as usize] = color;
                                write_c -= 1;
                            }
                        }
                    }
                }
            }
            GravityDirection::Left => {
                for r in 0..h {
                    let mut write_c = 0;
                    for c in 0..w {
                        let color = self.cells[r][c];
                        if color != bg_color {
                            if write_c < w {
                                res.cells[r][write_c] = color;
                                write_c += 1;
                            }
                        }
                    }
                }
            }
        }
        res
    }
}

/// Gravity directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GravityDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Symmetry report across standard axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArcSymmetryReport {
    pub horizontal: bool,
    pub vertical: bool,
    pub diagonal_main: bool,
    pub diagonal_anti: bool,
    pub rotational_90: bool,
    pub rotational_180: bool,
    pub rotational_270: bool,
}

impl ArcSymmetryReport {
    pub fn has_any_symmetry(&self) -> bool {
        self.horizontal
            || self.vertical
            || self.diagonal_main
            || self.diagonal_anti
            || self.rotational_90
            || self.rotational_180
            || self.rotational_270
    }
}

// =========================================================================
// Connected Component Object Segmentation
// =========================================================================

/// Connectivity type for object extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Connectivity {
    FourWay,
    EightWay,
}

/// Segmented object within an ARC grid
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcObject {
    pub cells: Vec<(usize, usize)>,
    pub color: Option<u8>,
    pub bbox: BoundingBox,
}

impl ArcObject {
    pub fn new(cells: Vec<(usize, usize)>, color: Option<u8>) -> Self {
        if cells.is_empty() {
            return Self {
                cells,
                color,
                bbox: BoundingBox::new(0, 0, 0, 0),
            };
        }

        let mut min_r = usize::MAX;
        let mut min_c = usize::MAX;
        let mut max_r = 0;
        let mut max_c = 0;

        for &(r, c) in &cells {
            min_r = min_r.min(r);
            min_c = min_c.min(c);
            max_r = max_r.max(r);
            max_c = max_c.max(c);
        }

        Self {
            cells,
            color,
            bbox: BoundingBox::new(min_r, min_c, max_r, max_c),
        }
    }

    #[inline]
    pub fn area(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    pub fn bbox_area(&self) -> usize {
        self.bbox.area()
    }

    pub fn centroid(&self) -> (f64, f64) {
        if self.cells.is_empty() {
            return (0.0, 0.0);
        }
        let (sum_r, sum_c) = self
            .cells
            .iter()
            .fold((0, 0), |(sr, sc), &(r, c)| (sr + r, sc + c));
        (
            sum_r as f64 / self.cells.len() as f64,
            sum_c as f64 / self.cells.len() as f64,
        )
    }

    /// Render object cropped to its bounding box
    pub fn to_grid(&self, default_bg: u8) -> ArcGrid {
        let h = self.bbox.height();
        let w = self.bbox.width();
        let mut grid = ArcGrid::with_color(h, w, default_bg);
        let col = self.color.unwrap_or(1);

        for &(r, c) in &self.cells {
            let lr = r - self.bbox.min_r;
            let lc = c - self.bbox.min_c;
            grid.set(lr, lc, col);
        }
        grid
    }

    /// Render object onto a canvas of given size at its original position
    pub fn to_canvas(&self, height: usize, width: usize, bg: u8) -> ArcGrid {
        let mut grid = ArcGrid::with_color(height, width, bg);
        let col = self.color.unwrap_or(1);
        for &(r, c) in &self.cells {
            grid.set(r, c, col);
        }
        grid
    }

    /// Translate object cells by (dr, dc)
    pub fn translate(&self, dr: i32, dc: i32) -> ArcObject {
        let mut new_cells = Vec::with_capacity(self.cells.len());
        for &(r, c) in &self.cells {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr >= 0 && nc >= 0 {
                new_cells.push((nr as usize, nc as usize));
            }
        }
        ArcObject::new(new_cells, self.color)
    }
}

impl ArcGrid {
    /// Segment grid into connected components.
    /// - `connectivity`: FourWay or EightWay.
    /// - `background`: If Some(bg), cells of this color are ignored as background.
    /// - `same_color_only`: If true, components must have the same color. If false, non-bg cells connected together form an object.
    pub fn find_objects(
        &self,
        connectivity: Connectivity,
        background: Option<u8>,
        same_color_only: bool,
    ) -> Vec<ArcObject> {
        let (h, w) = self.dims();
        if h == 0 || w == 0 {
            return Vec::new();
        }

        let mut visited = vec![vec![false; w]; h];
        let mut objects = Vec::new();

        let deltas: &[(i32, i32)] = match connectivity {
            Connectivity::FourWay => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
            Connectivity::EightWay => &[
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
            ],
        };

        for r in 0..h {
            for c in 0..w {
                if visited[r][c] {
                    continue;
                }

                let color = self.cells[r][c];
                if Some(color) == background {
                    visited[r][c] = true;
                    continue;
                }

                // BFS Flood Fill
                let mut comp_cells = Vec::new();
                let mut queue = VecDeque::new();

                visited[r][c] = true;
                queue.push_back((r, c));

                while let Some((cr, cc)) = queue.pop_front() {
                    comp_cells.push((cr, cc));

                    for &(dr, dc) in deltas {
                        let nr = cr as i32 + dr;
                        let nc = cc as i32 + dc;

                        if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                            let (ur, uc) = (nr as usize, nc as usize);
                            if !visited[ur][uc] {
                                let ncolor = self.cells[ur][uc];
                                if Some(ncolor) == background {
                                    continue;
                                }

                                if !same_color_only || ncolor == color {
                                    visited[ur][uc] = true;
                                    queue.push_back((ur, uc));
                                }
                            }
                        }
                    }
                }

                if !comp_cells.is_empty() {
                    let obj_color = if same_color_only { Some(color) } else { None };
                    objects.push(ArcObject::new(comp_cells, obj_color));
                }
            }
        }

        objects
    }
}
