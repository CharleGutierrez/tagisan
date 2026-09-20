//! Spatial and visual memory buffer, perceptual difference hashing (grid hash),
//! visual change detection, and UI state transition tracking.

use crate::engine::astra::screen::ScreenFrame;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

const GRID_COLS: usize = 8;
const GRID_ROWS: usize = 8;
const TOTAL_CELLS: usize = GRID_COLS * GRID_ROWS;

/// Bounding box in pixel space
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BoundingBox {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn contains(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// 8x8 Perceptual Grid Hash capturing spatial luminance and structure
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PerceptualGridHash {
    pub cell_luminance: Vec<u8>,
    pub binary_hash: u64,
}

impl PerceptualGridHash {
    /// Compute perceptual grid hash from ScreenFrame
    pub fn from_frame(frame: &ScreenFrame) -> Self {
        let (w, h) = frame.dimensions();
        if w == 0 || h == 0 {
            return Self {
                cell_luminance: vec![0; TOTAL_CELLS],
                binary_hash: 0,
            };
        }

        let cell_w = (w / GRID_COLS as u32).max(1);
        let cell_h = (h / GRID_ROWS as u32).max(1);
        let mut cell_luminance = [0u8; TOTAL_CELLS];

        let mut sum_luminance: u64 = 0;

        for gy in 0..GRID_ROWS {
            for gx in 0..GRID_COLS {
                let cell_idx = gy * GRID_COLS + gx;
                let start_x = (gx as u32) * cell_w;
                let start_y = (gy as u32) * cell_h;

                let mut sample_count = 0u64;
                let mut lum_total = 0u64;

                // Sample points inside cell
                for sy in (start_y..(start_y + cell_h)).step_by(4) {
                    for sx in (start_x..(start_x + cell_w)).step_by(4) {
                        if let Some((r, g, b, _)) = frame.sample_pixel(sx, sy) {
                            // ITU-R BT.601 luminance
                            let lum = (r as u64 * 299 + g as u64 * 587 + b as u64 * 114) / 1000;
                            lum_total += lum;
                            sample_count += 1;
                        }
                    }
                }

                let avg_lum = if sample_count > 0 {
                    (lum_total / sample_count).min(255) as u8
                } else {
                    128
                };

                cell_luminance[cell_idx] = avg_lum;
                sum_luminance += avg_lum as u64;
            }
        }

        let mean_lum = (sum_luminance / TOTAL_CELLS as u64) as u8;
        let mut binary_hash = 0u64;
        for (i, &lum) in cell_luminance.iter().enumerate() {
            if lum >= mean_lum {
                binary_hash |= 1u64 << i;
            }
        }

        Self {
            cell_luminance: cell_luminance.to_vec(),
            binary_hash,
        }
    }

    /// Hamming distance between binary hashes (0 to 64)
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        (self.binary_hash ^ other.binary_hash).count_ones()
    }

    /// Normalized difference score between two hashes [0.0 = identical, 1.0 = completely different]
    pub fn difference_score(&self, other: &Self) -> f64 {
        let mut total_diff: u64 = 0;
        for i in 0..TOTAL_CELLS {
            let diff = (self.cell_luminance[i] as i32 - other.cell_luminance[i] as i32).abs();
            total_diff += diff as u64;
        }
        total_diff as f64 / (TOTAL_CELLS as f64 * 255.0)
    }
}

/// Visual difference analysis between two frames
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisualDiffResult {
    pub diff_percentage: f64,
    pub changed_cells: Vec<(u32, u32)>,
    pub bounding_box: Option<BoundingBox>,
    pub is_significant: bool,
    pub summary: String,
}

impl VisualDiffResult {
    /// Compute visual diff between two frames
    pub fn compute(prev: &ScreenFrame, curr: &ScreenFrame) -> Self {
        let hash_prev = PerceptualGridHash::from_frame(prev);
        let hash_curr = PerceptualGridHash::from_frame(curr);

        let diff_score = hash_prev.difference_score(&hash_curr);
        let diff_percentage = (diff_score * 100.0).min(100.0);

        let mut changed_cells = Vec::new();
        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        let (w, h) = curr.dimensions();
        let cell_w = (w / GRID_COLS as u32).max(1);
        let cell_h = (h / GRID_ROWS as u32).max(1);

        // Threshold for individual cell change
        const CELL_CHANGE_THRESHOLD: i32 = 12;

        for gy in 0..GRID_ROWS {
            for gx in 0..GRID_COLS {
                let idx = gy * GRID_COLS + gx;
                let delta = (hash_prev.cell_luminance[idx] as i32 - hash_curr.cell_luminance[idx] as i32).abs();
                if delta >= CELL_CHANGE_THRESHOLD {
                    changed_cells.push((gx as u32, gy as u32));
                    let px0 = (gx as u32) * cell_w;
                    let py0 = (gy as u32) * cell_h;
                    let px1 = px0 + cell_w;
                    let py1 = py0 + cell_h;

                    min_x = min_x.min(px0);
                    min_y = min_y.min(py0);
                    max_x = max_x.max(px1);
                    max_y = max_y.max(py1);
                }
            }
        }

        let bounding_box = if !changed_cells.is_empty() && min_x < max_x && min_y < max_y {
            Some(BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        };

        let is_significant = diff_percentage >= 0.5 || !changed_cells.is_empty();

        let summary = format!(
            "Diff: {:.2}% change across {}/{} cells. BoundingBox: {:?}",
            diff_percentage,
            changed_cells.len(),
            TOTAL_CELLS,
            bounding_box
        );

        Self {
            diff_percentage,
            changed_cells,
            bounding_box,
            is_significant,
            summary,
        }
    }
}

/// Recorded UI State snapshot
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiState {
    pub state_id: String,
    pub timestamp_ms: u64,
    pub frame_id: u64,
    pub description: String,
    pub grid_hash: PerceptualGridHash,
}

/// State transition triggered by an action
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiTransition {
    pub from_state_id: String,
    pub to_state_id: String,
    pub action_taken: String,
    pub diff_percentage: f64,
    pub success: bool,
    pub timestamp_ms: u64,
}

/// Statistics on the visual memory buffer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisualMemoryStats {
    pub total_frames_recorded: usize,
    pub unique_states_discovered: usize,
    pub total_transitions: usize,
    pub last_diff_percentage: f64,
    pub is_stuck: bool,
}

/// Spatial and visual memory buffer
pub struct VisualMemory {
    frames: VecDeque<ScreenFrame>,
    states: Vec<UiState>,
    transitions: Vec<UiTransition>,
    max_history: usize,
    last_diff: f64,
}

impl Default for VisualMemory {
    fn default() -> Self {
        Self::new(30)
    }
}

impl VisualMemory {
    pub fn new(max_history: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(max_history),
            states: Vec::new(),
            transitions: Vec::new(),
            max_history: max_history.max(5),
            last_diff: 0.0,
        }
    }

    /// Record a new screen frame and compute difference against the prior frame
    pub fn record_frame(&mut self, frame: ScreenFrame, action_taken: Option<String>) -> VisualDiffResult {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let grid_hash = PerceptualGridHash::from_frame(&frame);
        let state_id = format!("{:016x}", grid_hash.binary_hash);

        let diff = if let Some(prev) = self.frames.back() {
            VisualDiffResult::compute(prev, &frame)
        } else {
            VisualDiffResult {
                diff_percentage: 100.0,
                changed_cells: Vec::new(),
                bounding_box: None,
                is_significant: true,
                summary: "Initial frame recorded".to_string(),
            }
        };

        self.last_diff = diff.diff_percentage;

        // Record transition if previous state exists and action was taken
        if let Some(prev_state) = self.states.last() {
            if let Some(action) = action_taken {
                let transition = UiTransition {
                    from_state_id: prev_state.state_id.clone(),
                    to_state_id: state_id.clone(),
                    action_taken: action,
                    diff_percentage: diff.diff_percentage,
                    success: diff.is_significant,
                    timestamp_ms,
                };
                self.transitions.push(transition);
            }
        }

        let ui_state = UiState {
            state_id,
            timestamp_ms,
            frame_id: frame.id,
            description: format!("UI State #{} (diff: {:.1}%)", self.states.len() + 1, diff.diff_percentage),
            grid_hash,
        };
        self.states.push(ui_state);

        if self.frames.len() >= self.max_history {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);

        diff
    }

    /// Retrieve the most recently recorded frame
    pub fn last_frame(&self) -> Option<&ScreenFrame> {
        self.frames.back()
    }

    /// Check if the agent is stuck in an invariant UI loop
    pub fn detect_stuck_state(&self, threshold_steps: usize) -> bool {
        if self.transitions.len() < threshold_steps {
            return false;
        }

        let recent = &self.transitions[self.transitions.len() - threshold_steps..];
        // If all recent transitions had < 0.1% change, agent is stuck
        recent.iter().all(|t| t.diff_percentage < 0.1)
    }

    /// Memory statistics
    pub fn stats(&self) -> VisualMemoryStats {
        VisualMemoryStats {
            total_frames_recorded: self.frames.len(),
            unique_states_discovered: self.states.len(),
            total_transitions: self.transitions.len(),
            last_diff_percentage: self.last_diff,
            is_stuck: self.detect_stuck_state(3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::astra::screen::{DisplayServerType, ScreenFrame};

    #[test]
    fn test_perceptual_hash_and_diff() {
        let mut pixels1 = vec![50u8; 128 * 128 * 4];
        let frame1 = ScreenFrame::new_rgba(128, 128, pixels1.clone(), DisplayServerType::Headless, true);

        // Modify half of pixels2
        let mut pixels2 = pixels1.clone();
        for chunk in pixels2[0..(64 * 128 * 4)].chunks_mut(4) {
            chunk[0] = 240;
            chunk[1] = 240;
            chunk[2] = 240;
        }
        let frame2 = ScreenFrame::new_rgba(128, 128, pixels2, DisplayServerType::Headless, true);

        let hash1 = PerceptualGridHash::from_frame(&frame1);
        let hash2 = PerceptualGridHash::from_frame(&frame2);
        assert_ne!(hash1.binary_hash, hash2.binary_hash);

        let diff = VisualDiffResult::compute(&frame1, &frame2);
        assert!(diff.is_significant);
        assert!(diff.diff_percentage > 5.0);
        assert!(diff.bounding_box.is_some());
    }

    #[test]
    fn test_visual_memory_transition_tracking() {
        let mut mem = VisualMemory::new(10);
        let frame1 = ScreenFrame::new_rgba(64, 64, vec![30u8; 64 * 64 * 4], DisplayServerType::Headless, true);
        let diff1 = mem.record_frame(frame1, None);
        assert_eq!(diff1.diff_percentage, 100.0);

        let frame2 = ScreenFrame::new_rgba(64, 64, vec![200u8; 64 * 64 * 4], DisplayServerType::Headless, true);
        let diff2 = mem.record_frame(frame2, Some("click(32, 32)".to_string()));
        assert!(diff2.is_significant);

        let stats = mem.stats();
        assert_eq!(stats.total_transitions, 1);
        assert!(!stats.is_stuck);
    }
}
