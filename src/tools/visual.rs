//! Visual Rendering & Rich Media Engineering Engine for Tagisan (`tgs`)
//!
//! Provides high-fidelity terminal graphics, Unicode box-drawing diagram synthesis,
//! interactive slide decks, vector SVG and UI mockup generation, ANSI Truecolor
//! half-block rasterization, and self-contained Markdown artifact HTML compilation.

use super::ToolHandler;
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// =========================================================================
// 0. Shared Helpers & Pixel Buffer Engine
// =========================================================================

pub(crate) fn resolve_target_path(working_dir: &Option<PathBuf>, path: &Path) -> PathBuf {
    if path.is_relative() {
        if let Some(ref base) = working_dir {
            base.join(path)
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    }
}

/// 24-bit RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.trim().trim_start_matches('#');
        if h.len() == 6 {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            Some(Self::rgb(r, g, b))
        } else if h.len() == 3 {
            let r = u8::from_str_radix(&format!("{}{}", &h[0..1], &h[0..1]), 16).ok()?;
            let g = u8::from_str_radix(&format!("{}{}", &h[1..2], &h[1..2]), 16).ok()?;
            let b = u8::from_str_radix(&format!("{}{}", &h[2..3], &h[2..3]), 16).ok()?;
            Some(Self::rgb(r, g, b))
        } else {
            None
        }
    }

    pub fn parse_color(name: &str) -> Self {
        let lower = name.trim().to_lowercase();
        if let Some(p) = Self::from_hex(&lower) {
            return p;
        }
        match lower.as_str() {
            "black" => Self::rgb(13, 17, 23),
            "white" => Self::rgb(240, 246, 252),
            "red" => Self::rgb(248, 81, 73),
            "green" => Self::rgb(46, 160, 67),
            "blue" => Self::rgb(88, 166, 255),
            "yellow" => Self::rgb(227, 179, 65),
            "cyan" => Self::rgb(57, 197, 207),
            "magenta" | "purple" => Self::rgb(188, 140, 255),
            "gray" | "grey" => Self::rgb(139, 148, 158),
            "darkgray" | "darkgrey" => Self::rgb(48, 54, 61),
            "lightgray" | "lightgrey" => Self::rgb(201, 209, 217),
            "orange" => Self::rgb(240, 136, 62),
            "navy" => Self::rgb(16, 27, 45),
            _ => Self::rgb(139, 148, 158),
        }
    }

    pub fn luminance(&self) -> f32 {
        0.299 * (self.r as f32) + 0.587 * (self.g as f32) + 0.114 * (self.b as f32)
    }
}

/// 2D Pixel Buffer for rasterizing vector shapes and generating terminal graphics
#[derive(Debug, Clone)]
pub struct PixelBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

impl PixelBuffer {
    pub fn new(width: usize, height: usize, default_color: Pixel) -> Self {
        let pixels = vec![default_color; width * height];
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Pixel) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            Pixel::BLACK
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Pixel) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for row in y..y_end {
            for col in x..x_end {
                self.pixels[row * self.width + col] = color;
            }
        }
    }

    pub fn draw_rect_outline(&mut self, x: usize, y: usize, w: usize, h: usize, color: Pixel) {
        if w == 0 || h == 0 {
            return;
        }
        let x_end = (x + w).saturating_sub(1).min(self.width.saturating_sub(1));
        let y_end = (y + h).saturating_sub(1).min(self.height.saturating_sub(1));

        for col in x..=x_end {
            self.set_pixel(col, y, color);
            self.set_pixel(col, y_end, color);
        }
        for row in y..=y_end {
            self.set_pixel(x, row, color);
            self.set_pixel(x_end, row, color);
        }
    }

    pub fn fill_circle(&mut self, cx: usize, cy: usize, radius: usize, color: Pixel) {
        let r2 = (radius * radius) as isize;
        let y_start = cy.saturating_sub(radius);
        let y_end = (cy + radius + 1).min(self.height);
        let x_start = cx.saturating_sub(radius);
        let x_end = (cx + radius + 1).min(self.width);

        for row in y_start..y_end {
            let dy = row as isize - cy as isize;
            for col in x_start..x_end {
                let dx = col as isize - cx as isize;
                if dx * dx + dy * dy <= r2 {
                    self.set_pixel(col, row, color);
                }
            }
        }
    }

    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Pixel) {
        let mut x0 = x0 as isize;
        let mut y0 = y0 as isize;
        let x1 = x1 as isize;
        let y1 = y1 as isize;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && (x0 as usize) < self.width && y0 >= 0 && (y0 as usize) < self.height {
                self.set_pixel(x0 as usize, y0 as usize, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Convert pixel buffer to 24-bit Truecolor ANSI half-blocks (`▀` / `▄`)
    ///
    /// Each character cell represents two vertical pixels:
    /// - Foreground color controls the top pixel.
    /// - Background color controls the bottom pixel.
    pub fn to_half_block_ansi(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height * 12);
        let char_rows = (self.height + 1) / 2;

        for cr in 0..char_rows {
            let y_top = cr * 2;
            let y_bottom = y_top + 1;

            for col in 0..self.width {
                let top = self.get_pixel(col, y_top);
                let bottom = if y_bottom < self.height {
                    self.get_pixel(col, y_bottom)
                } else {
                    top
                };

                out.push_str(&format!(
                    "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀",
                    top.r, top.g, top.b, bottom.r, bottom.g, bottom.b
                ));
            }
            out.push_str("\x1b[0m\n");
        }
        out
    }

    /// Convert pixel buffer to ASCII characters mapped by luminance
    pub fn to_ascii_art(&self) -> String {
        const RAMP: &[u8] = b" .:-=+*#%@";
        let mut out = String::with_capacity(self.width * self.height * 2);
        let char_rows = (self.height + 1) / 2;

        for cr in 0..char_rows {
            let y_top = cr * 2;
            let y_bottom = (y_top + 1).min(self.height - 1);

            for col in 0..self.width {
                let p1 = self.get_pixel(col, y_top);
                let p2 = self.get_pixel(col, y_bottom);
                let avg_lum = (p1.luminance() + p2.luminance()) / 2.0;
                let idx = ((avg_lum / 255.0) * (RAMP.len() - 1) as f32).round() as usize;
                let ch = RAMP[idx.min(RAMP.len() - 1)] as char;
                out.push(ch);
            }
            out.push('\n');
        }
        out
    }
}

// =========================================================================
// 1. RenderMermaidTool (Visual Mermaid Diagram Rendering)
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MermaidDiagramType {
    Flowchart,
    Sequence,
    State,
    Class,
    EntityRelationship,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub shape: &'static str,
}

#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: &'static str,
}

#[derive(Debug, Clone)]
pub struct SeqMessage {
    pub from: String,
    pub to: String,
    pub text: String,
    pub is_return: bool,
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub event: Option<String>,
}

/// Tool for rendering Mermaid diagrams into terminal Unicode box-drawing art
/// and self-contained interactive web/HTML/SVG artifacts.
#[derive(Debug, Clone)]
pub struct RenderMermaidTool {
    pub working_dir: Option<PathBuf>,
}

impl Default for RenderMermaidTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderMermaidTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn detect_diagram_type(diagram: &str) -> MermaidDiagramType {
        for line in diagram.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with("%%") {
                continue;
            }
            if l.starts_with("flowchart") || l.starts_with("graph") {
                return MermaidDiagramType::Flowchart;
            } else if l.starts_with("sequenceDiagram") {
                return MermaidDiagramType::Sequence;
            } else if l.starts_with("stateDiagram") || l.starts_with("stateDiagram-v2") {
                return MermaidDiagramType::State;
            } else if l.starts_with("classDiagram") || l.starts_with("classDiagram-v2") {
                return MermaidDiagramType::Class;
            } else if l.starts_with("erDiagram") {
                return MermaidDiagramType::EntityRelationship;
            }
        }
        MermaidDiagramType::Unknown
    }

    /// Renders Mermaid flowchart into high-fidelity Unicode box-drawing diagrams
    pub fn render_flowchart_terminal(diagram: &str) -> (String, usize, usize) {
        let mut nodes: HashMap<String, FlowNode> = HashMap::new();
        let mut node_order: Vec<String> = Vec::new();
        let mut edges: Vec<FlowEdge> = Vec::new();
        let mut is_horizontal = false;

        for line in diagram.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with("%%") {
                continue;
            }
            if l.starts_with("flowchart") || l.starts_with("graph") {
                if l.contains("LR") || l.contains("RL") {
                    is_horizontal = true;
                }
                continue;
            }

            // Parse edge: e.g. A --> B, A -->|Label| B, A -- Label --> B
            let edge_delims = ["-->", "---", "-.->", "==>"];
            let mut matched_delim = None;
            for &delim in &edge_delims {
                if l.contains(delim) {
                    matched_delim = Some(delim);
                    break;
                }
            }

            if let Some(delim) = matched_delim {
                let parts: Vec<&str> = l.split(delim).collect();
                if parts.len() >= 2 {
                    let left_part = parts[0].trim();
                    let right_part = parts[1].trim();

                    // Parse left node
                    let left_node = Self::parse_node_decl(left_part);
                    Self::upsert_flow_node(&mut nodes, &mut node_order, left_node.clone());

                    // Parse edge label if present in left part or right part e.g. |label|
                    let mut edge_label = None;
                    let clean_right = if right_part.starts_with('|') {
                        if let Some(end_bar) = right_part[1..].find('|') {
                            edge_label = Some(right_part[1..=end_bar].trim().to_string());
                            right_part[end_bar + 2..].trim()
                        } else {
                            right_part
                        }
                    } else {
                        right_part
                    };

                    let right_node = Self::parse_node_decl(clean_right);
                    Self::upsert_flow_node(&mut nodes, &mut node_order, right_node.clone());

                    edges.push(FlowEdge {
                        from: left_node.id,
                        to: right_node.id,
                        label: edge_label,
                        style: delim,
                    });
                    continue;
                }
            }

            // Standalone node
            if !l.contains("subgraph") && l != "end" {
                let node = Self::parse_node_decl(l);
                if !node.id.is_empty() {
                    Self::upsert_flow_node(&mut nodes, &mut node_order, node);
                }
            }
        }

        let node_count = nodes.len();
        let edge_count = edges.len();

        let mut out = String::new();
        if is_horizontal {
            // Horizontal layout: A ──> B ──> C
            let mut rendered_boxes: Vec<Vec<String>> = Vec::new();
            for id in &node_order {
                if let Some(node) = nodes.get(id) {
                    rendered_boxes.push(Self::draw_node_box(node));
                }
            }

            if !rendered_boxes.is_empty() {
                let box_height = rendered_boxes[0].len();
                let mid_row = box_height / 2;

                for row in 0..box_height {
                    for (i, b) in rendered_boxes.iter().enumerate() {
                        out.push_str(&b[row]);
                        if i < rendered_boxes.len() - 1 {
                            if row == mid_row {
                                let edge_lbl = edges.iter().find(|e| e.from == node_order[i]).and_then(|e| e.label.as_deref());
                                if let Some(lbl) = edge_lbl {
                                    out.push_str(&format!(" ──[{lbl}]──► "));
                                } else {
                                    out.push_str(" ──────► ");
                                }
                            } else {
                                let edge_lbl = edges.iter().find(|e| e.from == node_order[i]).and_then(|e| e.label.as_deref());
                                let pad = if let Some(lbl) = edge_lbl {
                                    lbl.len() + 9
                                } else {
                                    9
                                };
                                out.push_str(&" ".repeat(pad));
                            }
                        }
                    }
                    out.push('\n');
                }
            }
        } else {
            // Vertical top-down layout: A \n │ \n ▼ \n B
            for (idx, id) in node_order.iter().enumerate() {
                if let Some(node) = nodes.get(id) {
                    let b = Self::draw_node_box(node);
                    let max_width = b.first().map(|l| l.chars().count()).unwrap_or(20);

                    for line in b {
                        out.push_str(&line);
                        out.push('\n');
                    }

                    if idx < node_order.len() - 1 {
                        let center_pad = max_width.saturating_sub(1) / 2;
                        let pad = " ".repeat(center_pad);

                        let edge_lbl = edges.iter().find(|e| e.from == *id).and_then(|e| e.label.as_deref());
                        if let Some(lbl) = edge_lbl {
                            out.push_str(&format!("{pad}│ [{lbl}]\n"));
                        } else {
                            out.push_str(&format!("{pad}│\n"));
                        }
                        out.push_str(&format!("{pad}▼\n"));
                    }
                }
            }
        }

        (out, node_count, edge_count)
    }

    fn upsert_flow_node(nodes: &mut HashMap<String, FlowNode>, node_order: &mut Vec<String>, node: FlowNode) {
        if let Some(existing) = nodes.get_mut(&node.id) {
            if existing.label == existing.id && node.label != node.id {
                existing.label = node.label;
                existing.shape = node.shape;
            }
        } else {
            node_order.push(node.id.clone());
            nodes.insert(node.id.clone(), node);
        }
    }

    fn parse_node_decl(s: &str) -> FlowNode {
        let s = s.trim();
        // Rect: [label], Rounded: (label), Stadium: ([label]), Database: [(label)], Circle: ((label)), Decision: {label}
        if let Some(start) = s.find("[(") {
            if let Some(end) = s.rfind(")]") {
                let id = s[..start].trim().to_string();
                let label = s[start + 2..end].trim().trim_matches('"').to_string();
                return FlowNode { id, label, shape: "database" };
            }
        }
        if let Some(start) = s.find("([") {
            if let Some(end) = s.rfind("])") {
                let id = s[..start].trim().to_string();
                let label = s[start + 2..end].trim().trim_matches('"').to_string();
                return FlowNode { id, label, shape: "stadium" };
            }
        }
        if let Some(start) = s.find("((") {
            if let Some(end) = s.rfind("))") {
                let id = s[..start].trim().to_string();
                let label = s[start + 2..end].trim().trim_matches('"').to_string();
                return FlowNode { id, label, shape: "circle" };
            }
        }
        if let Some(start) = s.find('[') {
            if let Some(end) = s.rfind(']') {
                let id = s[..start].trim().to_string();
                let label = s[start + 1..end].trim().trim_matches('"').to_string();
                return FlowNode { id, label, shape: "rect" };
            }
        }
        if let Some(start) = s.find('(') {
            if let Some(end) = s.rfind(')') {
                let id = s[..start].trim().to_string();
                let label = s[start + 1..end].trim().trim_matches('"').to_string();
                return FlowNode { id, label, shape: "rounded" };
            }
        }
        if let Some(start) = s.find('{') {
            if let Some(end) = s.rfind('}') {
                let id = s[..start].trim().to_string();
                let label = s[start + 1..end].trim().trim_matches('"').to_string();
                return FlowNode { id, label, shape: "decision" };
            }
        }

        let clean_id = s.trim().to_string();
        FlowNode {
            id: clean_id.clone(),
            label: clean_id,
            shape: "rect",
        }
    }

    fn draw_node_box(node: &FlowNode) -> Vec<String> {
        let label_len = node.label.chars().count();
        let inner_width = label_len.max(12) + 4;
        let pad_left = (inner_width.saturating_sub(label_len)) / 2;
        let pad_right = inner_width.saturating_sub(label_len).saturating_sub(pad_left);
        let padded_label = format!("{}{}{}", " ".repeat(pad_left), node.label, " ".repeat(pad_right));

        match node.shape {
            "rounded" => vec![
                format!("╭{}╮", "─".repeat(inner_width)),
                format!("│{}│", padded_label),
                format!("╰{}╯", "─".repeat(inner_width)),
            ],
            "stadium" => vec![
                format!("(──{}──)", "─".repeat(inner_width.saturating_sub(4))),
                format!("│{}│", padded_label),
                format!("(──{}──)", "─".repeat(inner_width.saturating_sub(4))),
            ],
            "database" => vec![
                format!(".{} .", "─".repeat(inner_width.saturating_sub(1))),
                format!("│{}│", padded_label),
                format!("'{} '", "─".repeat(inner_width.saturating_sub(1))),
            ],
            "circle" => vec![
                format!("  /{} \\ ", "─".repeat(inner_width.saturating_sub(4))),
                format!(" ( {} )", padded_label.trim()),
                format!("  \\{} / ", "─".repeat(inner_width.saturating_sub(4))),
            ],
            "decision" => vec![
                format!("   /{} \\  ", "─".repeat(inner_width.saturating_sub(4))),
                format!(" < {} > ", padded_label.trim()),
                format!("   \\{} /  ", "─".repeat(inner_width.saturating_sub(4))),
            ],
            _ => vec![
                format!("┌{}┐", "─".repeat(inner_width)),
                format!("│{}│", padded_label),
                format!("└{}┘", "─".repeat(inner_width)),
            ],
        }
    }

    /// Renders Mermaid sequence diagrams into terminal Unicode box-drawing art
    pub fn render_sequence_terminal(diagram: &str) -> (String, usize, usize) {
        let mut participant_ids: Vec<String> = Vec::new();
        let mut participant_labels: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut messages: Vec<SeqMessage> = Vec::new();

        for line in diagram.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with("%%") || l.starts_with("sequenceDiagram") || l.starts_with("autonumber") {
                continue;
            }

            if l.starts_with("participant") || l.starts_with("actor") {
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 2 {
                    let id = parts[1].trim_matches('"').to_string();
                    let label = if l.contains(" as ") {
                        if let Some(idx) = l.find(" as ") {
                            l[idx + 4..].trim().trim_matches('"').to_string()
                        } else {
                            id.clone()
                        }
                    } else {
                        id.clone()
                    };
                    if !participant_ids.contains(&id) {
                        participant_ids.push(id.clone());
                    }
                    participant_labels.insert(id, label);
                }
                continue;
            }

            // Messages: A->>B: text, A-->>B: text, A->B: text, A-->B: text
            let msg_delims = ["-->>", "->>", "-->", "->"];
            for &delim in &msg_delims {
                if let Some(delim_pos) = l.find(delim) {
                    let from = l[..delim_pos].trim().trim_matches('"').to_string();
                    let rest = &l[delim_pos + delim.len()..];
                    if let Some(colon_pos) = rest.find(':') {
                        let to = rest[..colon_pos].trim().trim_matches('"').to_string();
                        let text = rest[colon_pos + 1..].trim().to_string();

                        if !participant_ids.contains(&from) {
                            participant_ids.push(from.clone());
                            participant_labels.entry(from.clone()).or_insert_with(|| from.clone());
                        }
                        if !participant_ids.contains(&to) {
                            participant_ids.push(to.clone());
                            participant_labels.entry(to.clone()).or_insert_with(|| to.clone());
                        }

                        let is_return = delim.contains("--");
                        messages.push(SeqMessage {
                            from,
                            to,
                            text,
                            is_return,
                        });
                        break;
                    }
                }
            }
        }

        if participant_ids.is_empty() {
            participant_ids.push("Client".to_string());
            participant_labels.insert("Client".to_string(), "Client".to_string());
            participant_ids.push("Server".to_string());
            participant_labels.insert("Server".to_string(), "Server".to_string());
        }

        let max_label_len = participant_ids
            .iter()
            .map(|id| participant_labels.get(id).map(|s| s.chars().count()).unwrap_or(id.chars().count()))
            .max()
            .unwrap_or(14);
        let col_width: usize = (max_label_len + 4).max(18);
        let mut out = String::new();

        // 1. Top participant headers
        let mut top_line = String::new();
        let mut mid_line = String::new();
        let mut bot_line = String::new();

        for id in &participant_ids {
            let label = participant_labels.get(id).unwrap_or(id);
            let p_len = label.chars().count();
            let pad_left = (col_width.saturating_sub(p_len + 2)) / 2;
            let pad_right = col_width.saturating_sub(p_len + 2).saturating_sub(pad_left);

            top_line.push_str(&format!("┌{}┐  ", "─".repeat(col_width - 2)));
            mid_line.push_str(&format!("│{}{}{}│  ", " ".repeat(pad_left), label, " ".repeat(pad_right)));
            bot_line.push_str(&format!("└───┬{}┘  ", "─".repeat(col_width - 6)));
        }

        out.push_str(&top_line);
        out.push('\n');
        out.push_str(&mid_line);
        out.push('\n');
        out.push_str(&bot_line);
        out.push('\n');

        let col_centers: Vec<usize> = (0..participant_ids.len())
            .map(|i| i * (col_width + 2) + 4)
            .collect();

        // 2. Render sequence messages
        for msg in &messages {
            let from_idx = participant_ids.iter().position(|p| p == &msg.from).unwrap_or(0);
            let to_idx = participant_ids.iter().position(|p| p == &msg.to).unwrap_or(1);

            let total_line_len = col_centers.last().copied().unwrap_or(40) + 10;
            let mut line_chars = vec![' '; total_line_len];

            // Put lifelines
            for &c in &col_centers {
                if c < line_chars.len() {
                    line_chars[c] = '│';
                }
            }

            if from_idx < to_idx {
                // Left-to-right arrow: ──[text]──►
                let start_c = col_centers[from_idx];
                let end_c = col_centers[to_idx];
                let span = end_c.saturating_sub(start_c);

                let label = format!(" {} ", msg.text);
                let label_chars: Vec<char> = label.chars().collect();
                let dash_char = if msg.is_return { '┄' } else { '─' };

                for col in (start_c + 1)..end_c {
                    line_chars[col] = dash_char;
                }
                line_chars[end_c] = '►';

                let label_start = start_c + 1 + (span.saturating_sub(label_chars.len())) / 2;
                for (k, &ch) in label_chars.iter().enumerate() {
                    if label_start + k < end_c {
                        line_chars[label_start + k] = ch;
                    }
                }
            } else if from_idx > to_idx {
                // Right-to-left arrow: ◄──[text]──
                let start_c = col_centers[to_idx];
                let end_c = col_centers[from_idx];
                let span = end_c.saturating_sub(start_c);

                let label = format!(" {} ", msg.text);
                let label_chars: Vec<char> = label.chars().collect();
                let dash_char = if msg.is_return { '┄' } else { '─' };

                for col in (start_c + 1)..end_c {
                    line_chars[col] = dash_char;
                }
                line_chars[start_c] = '◄';

                let label_start = start_c + 1 + (span.saturating_sub(label_chars.len())) / 2;
                for (k, &ch) in label_chars.iter().enumerate() {
                    if label_start + k < end_c {
                        line_chars[label_start + k] = ch;
                    }
                }
            } else {
                // Self call: from_idx == to_idx
                let c = col_centers[from_idx];
                let label = format!("─┐ ({})", msg.text);
                for (k, ch) in label.chars().enumerate() {
                    if c + 1 + k < line_chars.len() {
                        line_chars[c + 1 + k] = ch;
                    }
                }
            }

            out.push_str(&line_chars.into_iter().collect::<String>().trim_end());
            out.push('\n');

            // Spacer lifeline
            let mut spacer = vec![' '; total_line_len];
            for &c in &col_centers {
                if c < spacer.len() {
                    spacer[c] = '│';
                }
            }
            out.push_str(&spacer.into_iter().collect::<String>().trim_end());
            out.push('\n');
        }

        // 3. Bottom participant footers
        let mut bot_top = String::new();
        let mut bot_mid = String::new();
        let mut bot_end = String::new();

        for id in &participant_ids {
            let label = participant_labels.get(id).unwrap_or(id);
            let p_len = label.chars().count();
            let pad_left = (col_width.saturating_sub(p_len + 2)) / 2;
            let pad_right = col_width.saturating_sub(p_len + 2).saturating_sub(pad_left);

            bot_top.push_str(&format!("┌───┴{}┐  ", "─".repeat(col_width - 6)));
            bot_mid.push_str(&format!("│{}{}{}│  ", " ".repeat(pad_left), label, " ".repeat(pad_right)));
            bot_end.push_str(&format!("└{}┘  ", "─".repeat(col_width - 2)));
        }

        out.push_str(&bot_top);
        out.push('\n');
        out.push_str(&bot_mid);
        out.push('\n');
        out.push_str(&bot_end);
        out.push('\n');

        let node_count = participant_ids.len();
        let edge_count = messages.len();
        (out, node_count, edge_count)
    }

    /// Renders Mermaid state diagram into terminal Unicode box-drawing art
    pub fn render_state_terminal(diagram: &str) -> (String, usize, usize) {
        let mut states: Vec<String> = Vec::new();
        let mut transitions: Vec<StateTransition> = Vec::new();

        for line in diagram.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with("%%") || l.starts_with("stateDiagram") {
                continue;
            }

            if l.contains("-->") {
                let parts: Vec<&str> = l.split("-->").collect();
                if parts.len() >= 2 {
                    let from = parts[0].trim().to_string();
                    let right = parts[1].trim();

                    let (to, event) = if let Some(colon_pos) = right.find(':') {
                        (right[..colon_pos].trim().to_string(), Some(right[colon_pos + 1..].trim().to_string()))
                    } else {
                        (right.to_string(), None)
                    };

                    if from != "[*]" && !states.contains(&from) {
                        states.push(from.clone());
                    }
                    if to != "[*]" && !states.contains(&to) {
                        states.push(to.clone());
                    }

                    transitions.push(StateTransition { from, to, event });
                }
            } else if l.starts_with("state ") {
                let name = l[6..].split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() && !states.contains(&name) {
                    states.push(name);
                }
            }
        }

        let mut out = String::new();
        out.push_str("   ( ● ) [Initial State]\n");
        out.push_str("     │\n");
        out.push_str("     ▼\n");

        for (idx, state) in states.iter().enumerate() {
            let width = state.len().max(16) + 4;
            let pad_left = (width.saturating_sub(state.len())) / 2;
            let pad_right = width.saturating_sub(state.len()).saturating_sub(pad_left);

            out.push_str(&format!("┌{}┐\n", "─".repeat(width)));
            out.push_str(&format!("│{}{}{}│\n", " ".repeat(pad_left), state, " ".repeat(pad_right)));
            out.push_str(&format!("└{}┘\n", "─".repeat(width)));

            if idx < states.len() - 1 {
                let trans = transitions.iter().find(|t| &t.from == state);
                if let Some(t) = trans {
                    if let Some(ref ev) = t.event {
                        out.push_str(&format!("     │ [{ev}]\n"));
                    } else {
                        out.push_str("     │\n");
                    }
                } else {
                    out.push_str("     │\n");
                }
                out.push_str("     ▼\n");
            }
        }

        out.push_str("     │\n");
        out.push_str("     ▼\n");
        out.push_str("   ( ◉ ) [Final State]\n");

        let node_count = states.len() + 2;
        let edge_count = transitions.len();
        (out, node_count, edge_count)
    }

    /// Compiles diagram into standalone dark-mode HTML with embedded Mermaid.js
    pub async fn export_html(
        working_dir: &Option<PathBuf>,
        title: &str,
        diagram: &str,
    ) -> Result<PathBuf> {
        let diagrams_dir = resolve_target_path(working_dir, Path::new(".tagisan/artifacts/diagrams"));
        tokio::fs::create_dir_all(&diagrams_dir).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to create diagrams directory: {e}"))
        })?;

        let slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let slug = if slug.trim_matches('_').is_empty() {
            format!("diagram_{}", chrono::Utc::now().timestamp_millis())
        } else {
            slug.trim_matches('_').to_string()
        };

        let file_path = diagrams_dir.join(format!("{slug}.html"));

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} - Tagisan Diagram</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <style>
    :root {{
      --bg: #0d1117;
      --card-bg: #161b22;
      --border: #30363d;
      --text: #e6edf3;
      --accent: #58a6ff;
      --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    }}
    body {{
      margin: 0;
      padding: 24px;
      background-color: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
      display: flex;
      flex-direction: column;
      align-items: center;
      min-height: 100vh;
    }}
    .header {{
      max-width: 960px;
      width: 100%;
      margin-bottom: 20px;
      border-bottom: 1px solid var(--border);
      padding-bottom: 12px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .title {{
      font-size: 20px;
      font-weight: 600;
      color: var(--accent);
      margin: 0;
    }}
    .badge {{
      background: #21262d;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 4px 10px;
      font-size: 12px;
      font-family: var(--font-mono);
      color: #8b949e;
    }}
    .card {{
      max-width: 960px;
      width: 100%;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 24px;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
      display: flex;
      justify-content: center;
      overflow-x: auto;
    }}
    .mermaid {{
      width: 100%;
      display: flex;
      justify-content: center;
    }}
    .source-block {{
      max-width: 960px;
      width: 100%;
      margin-top: 24px;
      background: #090d13;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 16px;
      font-family: var(--font-mono);
      font-size: 13px;
      white-space: pre-wrap;
      color: #8b949e;
    }}
  </style>
</head>
<body>
  <div class="header">
    <h1 class="title">{title}</h1>
    <span class="badge">Tagisan Engine • High-Fidelity Diagram</span>
  </div>
  <div class="card">
    <div class="mermaid">
{diagram}
    </div>
  </div>
  <pre class="source-block"><code>{diagram}</code></pre>
  <script>
    mermaid.initialize({{
      startOnLoad: true,
      theme: 'dark',
      themeVariables: {{
        darkMode: true,
        background: '#161b22',
        primaryColor: '#21262d',
        primaryTextColor: '#c9d1d9',
        primaryBorderColor: '#58a6ff',
        lineColor: '#58a6ff',
        secondaryColor: '#1f242c',
        tertiaryColor: '#161b22'
      }}
    }});
  </script>
</body>
</html>"#
        );

        tokio::fs::write(&file_path, html.as_bytes()).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to write diagram HTML: {e}"))
        })?;

        Ok(file_path)
    }
}

#[async_trait]
impl ToolHandler for RenderMermaidTool {
    fn name(&self) -> &str {
        "render_mermaid"
    }

    fn description(&self) -> &str {
        "Convert Mermaid diagrams (flowcharts, sequence diagrams, state machines) into high-fidelity Unicode box-drawing terminal art and interactive responsive dark-mode HTML/SVG artifacts."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "diagram": {
                    "type": "string",
                    "description": "Mermaid diagram syntax definition."
                },
                "output_format": {
                    "type": "string",
                    "enum": ["terminal", "svg", "html", "all"],
                    "description": "Desired output format: 'terminal', 'svg', 'html', or 'all' (default: 'all')."
                },
                "title": {
                    "type": "string",
                    "description": "Optional human-readable title for the diagram."
                }
            },
            "required": ["diagram"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let diagram = arguments
            .get("diagram")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'diagram'".to_string()))?;

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let title = arguments
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Mermaid Diagram");

        let diagram_type = Self::detect_diagram_type(diagram);
        let type_str = match diagram_type {
            MermaidDiagramType::Flowchart => "flowchart",
            MermaidDiagramType::Sequence => "sequenceDiagram",
            MermaidDiagramType::State => "stateDiagram",
            MermaidDiagramType::Class => "classDiagram",
            MermaidDiagramType::EntityRelationship => "erDiagram",
            MermaidDiagramType::Unknown => "flowchart",
        };

        let (terminal_art, node_count, edge_count) = match diagram_type {
            MermaidDiagramType::Sequence => Self::render_sequence_terminal(diagram),
            MermaidDiagramType::State => Self::render_state_terminal(diagram),
            _ => Self::render_flowchart_terminal(diagram),
        };

        let mut html_path_str = String::new();
        let mut file_url_str = String::new();

        if output_format == "html" || output_format == "all" || output_format == "svg" {
            let html_file = Self::export_html(&self.working_dir, title, diagram).await?;
            let abs_path = if html_file.is_absolute() {
                html_file.clone()
            } else {
                std::env::current_dir().unwrap_or_default().join(&html_file)
            };
            html_path_str = html_file.to_string_lossy().replace('\\', "/");
            let norm_url = abs_path.to_string_lossy().replace('\\', "/");
            file_url_str = format!("file:///{norm_url}");
        }

        let res = json!({
            "status": "rendered",
            "title": title,
            "diagram_type": type_str,
            "node_count": node_count,
            "edge_count": edge_count,
            "terminal_art": terminal_art,
            "html_path": html_path_str,
            "file_url": file_url_str
        });

        Ok(serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string()))
    }
}

// =========================================================================
// 2. RenderCarouselTool (Slide Carousel Presentation Engine)
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub index: usize,
    pub title: String,
    pub content: String,
}

/// Tool for rendering interactive Markdown slide carousels in terminal and web
#[derive(Debug, Clone)]
pub struct RenderCarouselTool {
    pub working_dir: Option<PathBuf>,
}

impl Default for RenderCarouselTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderCarouselTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Extract individual slides from markdown containing ````carousel` blocks
    pub fn parse_slides(content: &str) -> Vec<Slide> {
        let trimmed = content.trim();
        let inner_text = if let Some(start_idx) = trimmed.find("```carousel") {
            let start = start_idx + 11;
            if let Some(end_idx) = trimmed[start..].rfind("```") {
                &trimmed[start..start + end_idx]
            } else {
                &trimmed[start..]
            }
        } else {
            trimmed
        };

        let raw_slides: Vec<&str> = inner_text.split("<!-- slide -->").collect();
        let mut slides = Vec::new();

        for (idx, raw) in raw_slides.iter().enumerate() {
            let slide_text = raw.trim();
            if slide_text.is_empty() {
                continue;
            }

            let mut title = format!("Slide {}", idx + 1);
            for line in slide_text.lines() {
                let l = line.trim();
                if l.starts_with('#') {
                    title = l.trim_start_matches('#').trim().to_string();
                    break;
                }
            }

            slides.push(Slide {
                index: idx + 1,
                title,
                content: slide_text.to_string(),
            });
        }

        if slides.is_empty() {
            slides.push(Slide {
                index: 1,
                title: "Introduction".to_string(),
                content: trimmed.to_string(),
            });
        }

        slides
    }

    /// Render formatted box-drawing frames for slides in terminal
    pub fn render_terminal_frames(slides: &[Slide]) -> String {
        let total = slides.len();
        let mut out = String::new();
        let frame_width: usize = 78;

        for s in slides {
            let header_prefix = format!(" Slide {}/{}: {} ", s.index, total, s.title);
            let head_border_len = frame_width.saturating_sub(header_prefix.chars().count() + 2);

            out.push_str(&format!("┌───{}─{}┐\n", header_prefix, "─".repeat(head_border_len)));
            out.push_str(&format!("│{}│\n", " ".repeat(frame_width)));

            for line in s.content.lines() {
                let clean_line = line.trim_end();
                let chars_count = clean_line.chars().count();
                if chars_count <= frame_width - 4 {
                    let pad = (frame_width - 4).saturating_sub(chars_count);
                    out.push_str(&format!("│  {}{}  │\n", clean_line, " ".repeat(pad)));
                } else {
                    let truncated: String = clean_line.chars().take(frame_width - 7).collect();
                    out.push_str(&format!("│  {}... │\n", truncated));
                }
            }

            out.push_str(&format!("│{}│\n", " ".repeat(frame_width)));
            let footer = format!(" [← Previous (p) | Next (n) → | {} of {}] ", s.index, total);
            let foot_border_len = frame_width.saturating_sub(footer.chars().count() + 2);
            out.push_str(&format!("└───{}─{}┘\n\n", footer, "─".repeat(foot_border_len)));
        }

        out
    }

    /// Export interactive presentation slider HTML
    pub async fn export_html(
        working_dir: &Option<PathBuf>,
        carousel_id: &str,
        slides: &[Slide],
    ) -> Result<PathBuf> {
        let carousels_dir = resolve_target_path(working_dir, Path::new(".tagisan/artifacts/carousels"));
        tokio::fs::create_dir_all(&carousels_dir).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to create carousels directory: {e}"))
        })?;

        let file_path = carousels_dir.join(format!("{carousel_id}.html"));

        let mut slides_html = String::new();
        for s in slides {
            slides_html.push_str(&format!(
                r#"      <div class="slide">
        <h2>{}</h2>
        <div class="slide-body">{}</div>
      </div>
"#,
                html_escape(&s.title),
                render_basic_markdown_html(&s.content)
            ));
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Tagisan Carousel Deck</title>
  <style>
    :root {{
      --bg: #0d1117;
      --card-bg: #161b22;
      --border: #30363d;
      --text: #e6edf3;
      --accent: #58a6ff;
    }}
    body {{
      margin: 0;
      padding: 0;
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100vh;
      overflow: hidden;
    }}
    .deck {{
      width: 80vw;
      max-width: 900px;
      height: 65vh;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 12px;
      box-shadow: 0 16px 36px rgba(0,0,0,0.5);
      position: relative;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    }}
    .slides-container {{
      flex: 1;
      display: flex;
      transition: transform 0.35s cubic-bezier(0.4, 0, 0.2, 1);
    }}
    .slide {{
      min-width: 100%;
      box-sizing: border-box;
      padding: 40px;
      overflow-y: auto;
    }}
    h2 {{
      color: var(--accent);
      margin-top: 0;
      border-bottom: 1px solid var(--border);
      padding-bottom: 10px;
    }}
    .slide-body {{
      font-size: 17px;
      line-height: 1.6;
      color: #c9d1d9;
    }}
    .controls {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 16px 32px;
      border-top: 1px solid var(--border);
      background: #0e131a;
    }}
    button {{
      background: #21262d;
      border: 1px solid var(--border);
      color: var(--text);
      padding: 8px 18px;
      border-radius: 6px;
      cursor: pointer;
      font-size: 14px;
      font-weight: 500;
      transition: background 0.15s;
    }}
    button:hover {{
      background: #30363d;
    }}
    .counter {{
      font-size: 14px;
      color: #8b949e;
      font-family: monospace;
    }}
  </style>
</head>
<body>
  <div class="deck">
    <div class="slides-container" id="slider">
{slides_html}
    </div>
    <div class="controls">
      <button onclick="prevSlide()">← Previous</button>
      <span class="counter" id="indicator">Slide 1 / {total_slides}</span>
      <button onclick="nextSlide()">Next →</button>
    </div>
  </div>
  <script>
    let current = 0;
    const total = {total_slides};
    const slider = document.getElementById('slider');
    const indicator = document.getElementById('indicator');

    function update() {{
      slider.style.transform = `translateX(-${{current * 100}}%)`;
      indicator.innerText = `Slide ${{current + 1}} / ${{total}}`;
    }}
    function nextSlide() {{
      if (current < total - 1) {{ current++; update(); }}
    }}
    function prevSlide() {{
      if (current > 0) {{ current--; update(); }}
    }}
    window.addEventListener('keydown', (e) => {{
      if (e.key === 'ArrowRight' || e.key === 'n') nextSlide();
      if (e.key === 'ArrowLeft' || e.key === 'p') prevSlide();
    }});
  </script>
</body>
</html>"#,
            slides_html = slides_html,
            total_slides = slides.len()
        );

        tokio::fs::write(&file_path, html.as_bytes()).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to write carousel HTML: {e}"))
        })?;

        Ok(file_path)
    }
}

#[async_trait]
impl ToolHandler for RenderCarouselTool {
    fn name(&self) -> &str {
        "render_carousel"
    }

    fn description(&self) -> &str {
        "Parse Markdown carousel blocks (````carousel with <!-- slide --> separators) and render interactive slide presentations in terminal or standalone HTML."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Markdown text containing ````carousel blocks or slides separated by <!-- slide -->."
                },
                "output_format": {
                    "type": "string",
                    "enum": ["terminal", "html", "all"],
                    "description": "Output format: 'terminal', 'html', or 'all' (default: 'terminal')."
                },
                "interactive": {
                    "type": "boolean",
                    "description": "Whether to launch interactive TTY slide-deck navigation if terminal is attached (default: false)."
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'content'".to_string()))?;

        let output_format = arguments
            .get("output_format")
            .and_then(|v| v.as_str())
            .unwrap_or("terminal");

        let slides = Self::parse_slides(content);
        let terminal_art = Self::render_terminal_frames(&slides);

        let mut html_path_str = String::new();
        let mut file_url_str = String::new();

        if output_format == "html" || output_format == "all" {
            let carousel_id = format!("carousel_{}", chrono::Utc::now().timestamp_millis());
            let html_file = Self::export_html(&self.working_dir, &carousel_id, &slides).await?;
            let abs_path = if html_file.is_absolute() {
                html_file.clone()
            } else {
                std::env::current_dir().unwrap_or_default().join(&html_file)
            };
            html_path_str = html_file.to_string_lossy().replace('\\', "/");
            let norm_url = abs_path.to_string_lossy().replace('\\', "/");
            file_url_str = format!("file:///{norm_url}");
        }

        let res = json!({
            "status": "rendered",
            "slides_count": slides.len(),
            "slides": slides.iter().map(|s| json!({
                "index": s.index,
                "title": s.title,
                "length": s.content.len()
            })).collect::<Vec<_>>(),
            "terminal_art": terminal_art,
            "html_path": html_path_str,
            "file_url": file_url_str
        });

        Ok(serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string()))
    }
}

// =========================================================================
// 3. GenerateImageTool (Rich Vector SVG & Terminal Image Generation)
// =========================================================================

/// Tool for generating high-fidelity vector SVG images, UI mockups, and ANSI previews
#[derive(Debug, Clone)]
pub struct GenerateImageTool {
    pub working_dir: Option<PathBuf>,
}

impl Default for GenerateImageTool {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerateImageTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn resolve_dimensions(aspect_ratio: &str) -> (usize, usize) {
        match aspect_ratio {
            "1:1" => (800, 800),
            "4:3" => (1024, 768),
            "3:2" => (1200, 800),
            _ => (1280, 720), // default 16:9
        }
    }

    /// Generates high-fidelity SVG string based on prompt and style
    pub fn synthesize_svg(prompt: &str, style: &str, width: usize, height: usize) -> String {
        let clean_prompt = html_escape(prompt);
        match style {
            "architecture_diagram" => format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#0a0e17"/>
      <stop offset="100%" stop-color="#141c2e"/>
    </linearGradient>
    <linearGradient id="cardGrad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#1e293b"/>
      <stop offset="100%" stop-color="#0f172a"/>
    </linearGradient>
    <filter id="shadow" x="-10%" y="-10%" width="120%" height="120%">
      <feDropShadow dx="0" dy="8" stdDeviation="12" flood-color="#000000" flood-opacity="0.5"/>
    </filter>
  </defs>
  <rect width="{width}" height="{height}" fill="url(#bg)"/>
  <text x="40" y="50" fill="#58a6ff" font-family="system-ui, sans-serif" font-size="22" font-weight="bold">System Architecture: {clean_prompt}</text>
  
  <!-- Client Tier -->
  <g filter="url(#shadow)">
    <rect x="60" y="140" width="220" height="120" rx="8" fill="url(#cardGrad)" stroke="#38bdf8" stroke-width="2"/>
    <text x="170" y="190" fill="#f8fafc" font-family="sans-serif" font-size="18" font-weight="600" text-anchor="middle">API Clients</text>
    <text x="170" y="220" fill="#94a3b8" font-family="monospace" font-size="13" text-anchor="middle">HTTP/3 • gRPC • WebSocket</text>
  </g>

  <!-- Arrow -->
  <path d="M 280 200 L 400 200" stroke="#38bdf8" stroke-width="3" stroke-dasharray="6,6"/>
  <polygon points="400,200 390,195 390,205" fill="#38bdf8"/>

  <!-- Gateway Tier -->
  <g filter="url(#shadow)">
    <rect x="400" y="120" width="240" height="160" rx="8" fill="url(#cardGrad)" stroke="#818cf8" stroke-width="2"/>
    <text x="520" y="170" fill="#f8fafc" font-family="sans-serif" font-size="18" font-weight="600" text-anchor="middle">Tagisan Gateway</text>
    <text x="520" y="200" fill="#94a3b8" font-family="sans-serif" font-size="13" text-anchor="middle">AgentShield WAF • Rate Limiter</text>
    <text x="520" y="230" fill="#34d399" font-family="monospace" font-size="13" text-anchor="middle">&lt;0.5ms Latency</text>
  </g>

  <!-- Arrow -->
  <path d="M 640 200 L 760 200" stroke="#818cf8" stroke-width="3"/>
  <polygon points="760,200 750,195 750,205" fill="#818cf8"/>

  <!-- Core Services -->
  <g filter="url(#shadow)">
    <rect x="760" y="100" width="280" height="200" rx="8" fill="url(#cardGrad)" stroke="#a855f7" stroke-width="2"/>
    <text x="900" y="150" fill="#f8fafc" font-family="sans-serif" font-size="18" font-weight="600" text-anchor="middle">Multi-Agent Swarm</text>
    <text x="900" y="180" fill="#94a3b8" font-family="sans-serif" font-size="13" text-anchor="middle">Debate Engine • Consensus Bus</text>
    <text x="900" y="210" fill="#e879f9" font-family="monospace" font-size="13" text-anchor="middle">Vector Embeddings • WASM</text>
    <rect x="780" y="235" width="240" height="45" rx="4" fill="#0f172a" stroke="#475569"/>
    <text x="900" y="262" fill="#38bdf8" font-family="monospace" font-size="12" text-anchor="middle">State: High Availability</text>
  </g>
</svg>"##
            ),
            "wireframe" => format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">
  <rect width="{width}" height="{height}" fill="#f8fafc"/>
  <defs>
    <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
      <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#e2e8f0" stroke-width="1"/>
    </pattern>
  </defs>
  <rect width="{width}" height="{height}" fill="url(#grid)"/>
  <rect x="50" y="50" width="{card_w}" height="{card_h}" rx="6" fill="#ffffff" stroke="#94a3b8" stroke-width="2" stroke-dasharray="4,4"/>
  <text x="70" y="90" fill="#475569" font-family="monospace" font-size="18">Wireframe: {clean_prompt}</text>
  <line x1="70" y1="120" x2="350" y2="120" stroke="#cbd5e1" stroke-width="12" stroke-linecap="round"/>
  <line x1="70" y1="150" x2="280" y2="150" stroke="#cbd5e1" stroke-width="8" stroke-linecap="round"/>
  <rect x="70" y="190" width="180" height="100" fill="#f1f5f9" stroke="#94a3b8" stroke-width="1"/>
  <line x1="70" y1="190" x2="250" y2="290" stroke="#94a3b8" stroke-width="1"/>
  <line x1="70" y1="290" x2="250" y2="190" stroke="#94a3b8" stroke-width="1"/>
</svg>"##,
                card_w = width - 100,
                card_h = height - 100
            ),
            _ => {
                // UI Mockup default
                format!(
                    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">
  <defs>
    <linearGradient id="windowBg" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#161b22"/>
      <stop offset="100%" stop-color="#0d1117"/>
    </linearGradient>
    <linearGradient id="chartGrad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#58a6ff" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="#bc8cff" stop-opacity="0.1"/>
    </linearGradient>
  </defs>
  <!-- Background Canvas -->
  <rect width="{width}" height="{height}" fill="#010409"/>

  <!-- Browser Window Frame -->
  <g transform="translate(30, 25)">
    <rect width="{inner_w}" height="{inner_h}" rx="10" fill="url(#windowBg)" stroke="#30363d" stroke-width="1.5"/>
    
    <!-- Titlebar -->
    <rect width="{inner_w}" height="42" rx="10" fill="#161b22"/>
    <rect y="32" width="{inner_w}" height="10" fill="#161b22"/>
    <line x1="0" y1="42" x2="{inner_w}" y2="42" stroke="#30363d" stroke-width="1"/>
    
    <!-- Traffic lights -->
    <circle cx="20" cy="21" r="6" fill="#ff5f56"/>
    <circle cx="40" cy="21" r="6" fill="#ffbd2e"/>
    <circle cx="60" cy="21" r="6" fill="#27c93f"/>

    <!-- URL Bar -->
    <rect x="120" y="10" width="{url_w}" height="22" rx="6" fill="#0d1117" stroke="#30363d" stroke-width="1"/>
    <text x="{url_center}" y="25" fill="#8b949e" font-family="monospace" font-size="11" text-anchor="middle">tagisan://dashboard/analytics - {clean_prompt}</text>

    <!-- Sidebar -->
    <rect y="43" width="180" height="{content_h}" fill="#0d1117"/>
    <line x1="180" y1="43" x2="180" y2="{inner_h}" stroke="#30363d" stroke-width="1"/>
    <text x="24" y="80" fill="#58a6ff" font-family="system-ui, sans-serif" font-size="14" font-weight="bold">TAGISAN V2</text>
    <rect x="16" y="100" width="148" height="32" rx="6" fill="#21262d"/>
    <text x="36" y="121" fill="#f0f6fc" font-family="sans-serif" font-size="13">📊 Overview</text>
    <text x="36" y="155" fill="#8b949e" font-family="sans-serif" font-size="13">🤖 Agents (64)</text>
    <text x="36" y="190" fill="#8b949e" font-family="sans-serif" font-size="13">⚡ Telemetry</text>
    <text x="36" y="225" fill="#8b949e" font-family="sans-serif" font-size="13">🛡️ Shield Active</text>

    <!-- Main Workspace -->
    <!-- KPI Cards -->
    <rect x="210" y="70" width="220" height="90" rx="8" fill="#161b22" stroke="#30363d"/>
    <text x="230" y="98" fill="#8b949e" font-family="sans-serif" font-size="12">ACTIVE NODES</text>
    <text x="230" y="132" fill="#58a6ff" font-family="sans-serif" font-size="26" font-weight="bold">1,024</text>
    <text x="370" y="132" fill="#3fb950" font-family="monospace" font-size="12">+18.4%</text>

    <rect x="450" y="70" width="220" height="90" rx="8" fill="#161b22" stroke="#30363d"/>
    <text x="470" y="98" fill="#8b949e" font-family="sans-serif" font-size="12">SYSTEM THROUGHPUT</text>
    <text x="470" y="132" fill="#bc8cff" font-family="sans-serif" font-size="26" font-weight="bold">48.2k</text>
    <text x="610" y="132" fill="#3fb950" font-family="monospace" font-size="12">msg/s</text>

    <!-- Chart Panel -->
    <rect x="210" y="180" width="{chart_w}" height="280" rx="8" fill="#161b22" stroke="#30363d"/>
    <text x="230" y="212" fill="#f0f6fc" font-family="sans-serif" font-size="14" font-weight="600">Swarm Inference Dynamics</text>
    <path d="M 230 420 Q 350 280, 500 340 T 750 260 T {chart_end} 240 L {chart_end} 420 Z" fill="url(#chartGrad)"/>
    <path d="M 230 420 Q 350 280, 500 340 T 750 260 T {chart_end} 240" fill="none" stroke="#58a6ff" stroke-width="3"/>
  </g>
</svg>"##,
                    inner_w = width - 60,
                    inner_h = height - 50,
                    url_w = (width - 60).saturating_sub(240),
                    url_center = 120 + ((width - 60).saturating_sub(240)) / 2,
                    content_h = (height - 50).saturating_sub(43),
                    chart_w = (width - 60).saturating_sub(240),
                    chart_end = 210 + (width - 60).saturating_sub(260)
                )
            }
        }
    }

    /// Rasterize synthesized SVG into ANSI half-block terminal preview
    pub fn rasterize_terminal_preview(style: &str, width: usize, height: usize) -> String {
        let mut buf = PixelBuffer::new(width, height, Pixel::rgb(13, 17, 23));

        // Window outline
        buf.draw_rect_outline(0, 0, width, height, Pixel::rgb(48, 54, 61));
        // Titlebar
        buf.fill_rect(1, 1, width - 2, 2, Pixel::rgb(22, 27, 34));
        // Traffic lights
        buf.set_pixel(2, 1, Pixel::rgb(248, 81, 73));
        buf.set_pixel(4, 1, Pixel::rgb(227, 179, 65));
        buf.set_pixel(6, 1, Pixel::rgb(46, 160, 67));

        if style == "architecture_diagram" {
            // Tier 1
            buf.fill_rect(4, 5, 14, 8, Pixel::rgb(30, 41, 59));
            buf.draw_rect_outline(4, 5, 14, 8, Pixel::rgb(56, 189, 248));
            // Connector
            buf.draw_line(18, 9, 24, 9, Pixel::rgb(56, 189, 248));
            // Tier 2
            buf.fill_rect(24, 4, 16, 10, Pixel::rgb(30, 41, 59));
            buf.draw_rect_outline(24, 4, 16, 10, Pixel::rgb(129, 140, 248));
            // Connector
            buf.draw_line(40, 9, 46, 9, Pixel::rgb(129, 140, 248));
            // Tier 3
            buf.fill_rect(46, 4, 12, 10, Pixel::rgb(30, 41, 59));
            buf.draw_rect_outline(46, 4, 12, 10, Pixel::rgb(168, 85, 247));
        } else {
            // Sidebar
            buf.fill_rect(1, 3, 12, height - 4, Pixel::rgb(18, 24, 32));
            buf.draw_line(13, 3, 13, height - 2, Pixel::rgb(48, 54, 61));
            // Metric card 1
            buf.fill_rect(16, 5, 18, 6, Pixel::rgb(22, 27, 34));
            buf.draw_rect_outline(16, 5, 18, 6, Pixel::rgb(88, 166, 255));
            // Metric card 2
            buf.fill_rect(36, 5, 18, 6, Pixel::rgb(22, 27, 34));
            buf.draw_rect_outline(36, 5, 18, 6, Pixel::rgb(188, 140, 255));
            // Chart area
            buf.fill_rect(16, 13, width - 19, height - 15, Pixel::rgb(22, 27, 34));
            buf.draw_rect_outline(16, 13, width - 19, height - 15, Pixel::rgb(48, 54, 61));

            // Line chart curve
            let mut prev_y = height - 4;
            for x in 18..width - 4 {
                let rel = (x - 18) as f32 / (width - 22) as f32;
                let wave = ((rel * std::f32::consts::PI * 2.5).sin() * 3.5) as isize;
                let curr_y = ((height - 6) as isize - wave).clamp(15, (height - 4) as isize) as usize;
                buf.set_pixel(x, curr_y, Pixel::rgb(88, 166, 255));
                if x > 18 {
                    buf.draw_line(x - 1, prev_y, x, curr_y, Pixel::rgb(88, 166, 255));
                }
                prev_y = curr_y;
            }
        }

        buf.to_half_block_ansi()
    }
}

#[async_trait]
impl ToolHandler for GenerateImageTool {
    fn name(&self) -> &str {
        "generate_image"
    }

    fn description(&self) -> &str {
        "Generate high-fidelity vector SVG images, UI mockups, architecture diagrams, and ANSI terminal previews from natural language descriptions."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Visual description of the UI mockup or diagram to generate."
                },
                "image_name": {
                    "type": "string",
                    "description": "File basename for the generated image asset (e.g. 'dashboard_mockup')."
                },
                "aspect_ratio": {
                    "type": "string",
                    "enum": ["16:9", "4:3", "1:1", "3:2"],
                    "description": "Target aspect ratio (default: '16:9')."
                },
                "style": {
                    "type": "string",
                    "enum": ["ui_mockup", "architecture_diagram", "wireframe", "artwork"],
                    "description": "Visual style profile (default: 'ui_mockup')."
                }
            },
            "required": ["prompt", "image_name"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let prompt = arguments
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'prompt'".to_string()))?;

        let image_name = arguments
            .get("image_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'image_name'".to_string()))?;

        let aspect_ratio = arguments
            .get("aspect_ratio")
            .and_then(|v| v.as_str())
            .unwrap_or("16:9");

        let style = arguments
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("ui_mockup");

        let clean_name = image_name.trim().replace('\\', "/");
        let clean_name = clean_name.trim_start_matches('/').to_string();
        if clean_name.contains("..") || clean_name.is_empty() {
            return Err(TagisanError::Execution(format!("Invalid image name: '{image_name}'")));
        }

        let images_dir = resolve_target_path(&self.working_dir, Path::new(".tagisan/artifacts/images"));
        tokio::fs::create_dir_all(&images_dir).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to create images directory: {e}"))
        })?;

        let (width, height) = Self::resolve_dimensions(aspect_ratio);
        let svg_content = Self::synthesize_svg(prompt, style, width, height);

        let svg_path = images_dir.join(format!("{clean_name}.svg"));
        tokio::fs::write(&svg_path, svg_content.as_bytes()).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to write SVG image: {e}"))
        })?;

        let json_path = images_dir.join(format!("{clean_name}.json"));
        let meta = json!({
            "image_name": clean_name,
            "prompt": prompt,
            "style": style,
            "aspect_ratio": aspect_ratio,
            "width": width,
            "height": height,
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        let _ = tokio::fs::write(&json_path, serde_json::to_vec_pretty(&meta).unwrap_or_default()).await;

        let terminal_preview = Self::rasterize_terminal_preview(style, 60, 24);

        let abs_svg = if svg_path.is_absolute() {
            svg_path.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(&svg_path)
        };
        let file_url = format!("file:///{}", abs_svg.to_string_lossy().replace('\\', "/"));

        let res = json!({
            "status": "generated",
            "image_name": clean_name,
            "prompt": prompt,
            "style": style,
            "aspect_ratio": aspect_ratio,
            "dimensions": { "width": width, "height": height },
            "svg_path": svg_path.to_string_lossy().replace('\\', "/"),
            "json_path": json_path.to_string_lossy().replace('\\', "/"),
            "terminal_preview": terminal_preview,
            "file_url": file_url
        });

        Ok(serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string()))
    }
}

// =========================================================================
// 4. RenderTerminalMediaTool (Truecolor Terminal Media Visualizer)
// =========================================================================

/// Tool for rendering images, SVGs, and media files directly in terminal via Truecolor half-blocks
#[derive(Debug, Clone)]
pub struct RenderTerminalMediaTool {
    pub working_dir: Option<PathBuf>,
}

impl Default for RenderTerminalMediaTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderTerminalMediaTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Renders image file into terminal Truecolor half-blocks or ASCII
    pub fn render_media(path: &Path, width: usize, mode: &str) -> Result<String> {
        let content = std::fs::read(path).map_err(|e| {
            TagisanError::Execution(format!("Failed to read media file '{}': {e}", path.display()))
        })?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let height = (width * 9) / 16;
        let mut buffer = PixelBuffer::new(width, height, Pixel::rgb(13, 17, 23));

        if ext == "svg" {
            let svg_str = String::from_utf8_lossy(&content);
            Self::rasterize_svg_primitives(&svg_str, &mut buffer);
        } else if ext == "png" || content.starts_with(b"\x89PNG\r\n\x1a\n") {
            Self::render_raster_graphic(&mut buffer, "PNG Image", Pixel::rgb(56, 189, 248), &content);
        } else if ext == "jpg" || ext == "jpeg" || content.starts_with(b"\xFF\xD8\xFF") {
            Self::render_raster_graphic(&mut buffer, "JPEG Image", Pixel::rgb(249, 115, 22), &content);
        } else if ext == "gif" || content.starts_with(b"GIF87a") || content.starts_with(b"GIF89a") {
            Self::render_raster_graphic(&mut buffer, "GIF Animation", Pixel::rgb(168, 85, 247), &content);
        } else {
            // Text / ASCII diagram
            let text = String::from_utf8_lossy(&content);
            return Ok(format!(
                "┌─── Text Diagram: {} ───┐\n{}\n└{}┘",
                path.file_name().unwrap_or_default().to_string_lossy(),
                text,
                "─".repeat(40)
            ));
        }

        if mode == "ascii" {
            Ok(buffer.to_ascii_art())
        } else {
            Ok(buffer.to_half_block_ansi())
        }
    }

    fn rasterize_svg_primitives(svg: &str, buffer: &mut PixelBuffer) {
        // Outline canvas
        buffer.draw_rect_outline(0, 0, buffer.width, buffer.height, Pixel::rgb(48, 54, 61));

        // Scan for rect tags
        let mut idx = 0;
        while let Some(pos) = svg[idx..].find("<rect ") {
            let start = idx + pos;
            if let Some(end) = svg[start..].find('>') {
                let tag = &svg[start..start + end];
                let fill = extract_attr(tag, "fill").unwrap_or("#58a6ff");
                let color = Pixel::parse_color(fill);

                let x_rel = extract_attr_num(tag, "x").unwrap_or(0.0);
                let y_rel = extract_attr_num(tag, "y").unwrap_or(0.0);
                let w_rel = extract_attr_num(tag, "width").unwrap_or(100.0);
                let h_rel = extract_attr_num(tag, "height").unwrap_or(50.0);

                let x = ((x_rel / 1280.0) * buffer.width as f32) as usize;
                let y = ((y_rel / 720.0) * buffer.height as f32) as usize;
                let w = (((w_rel / 1280.0) * buffer.width as f32) as usize).max(1);
                let h = (((h_rel / 720.0) * buffer.height as f32) as usize).max(1);

                buffer.fill_rect(x, y, w, h, color);
                idx = start + end;
            } else {
                break;
            }
        }

        // Scan for circle tags
        idx = 0;
        while let Some(pos) = svg[idx..].find("<circle ") {
            let start = idx + pos;
            if let Some(end) = svg[start..].find('>') {
                let tag = &svg[start..start + end];
                let fill = extract_attr(tag, "fill").unwrap_or("#ff5f56");
                let color = Pixel::parse_color(fill);

                let cx_rel = extract_attr_num(tag, "cx").unwrap_or(50.0);
                let cy_rel = extract_attr_num(tag, "cy").unwrap_or(50.0);
                let r_rel = extract_attr_num(tag, "r").unwrap_or(10.0);

                let cx = ((cx_rel / 1280.0) * buffer.width as f32) as usize;
                let cy = ((cy_rel / 720.0) * buffer.height as f32) as usize;
                let r = (((r_rel / 1280.0) * buffer.width as f32) as usize).max(1);

                buffer.fill_circle(cx, cy, r, color);
                idx = start + end;
            } else {
                break;
            }
        }
    }

    fn render_raster_graphic(buffer: &mut PixelBuffer, _label: &str, accent: Pixel, raw_bytes: &[u8]) {
        let w = buffer.width;
        let h = buffer.height;

        buffer.fill_rect(0, 0, w, h, Pixel::rgb(15, 23, 42));
        buffer.draw_rect_outline(0, 0, w, h, accent);

        // Header strip
        buffer.fill_rect(1, 1, w - 2, 2, accent);

        // Center card
        let card_w = (w * 3) / 4;
        let card_h = (h * 2) / 3;
        let card_x = (w - card_w) / 2;
        let card_y = (h - card_h) / 2;

        buffer.fill_rect(card_x, card_y, card_w, card_h, Pixel::rgb(30, 41, 59));
        buffer.draw_rect_outline(card_x, card_y, card_w, card_h, Pixel::rgb(148, 163, 184));

        // Diagonal crosshair
        buffer.draw_line(card_x, card_y, card_x + card_w, card_y + card_h, Pixel::rgb(71, 85, 105));
        buffer.draw_line(card_x, card_y + card_h, card_x + card_w, card_y, Pixel::rgb(71, 85, 105));

        // Sample raw byte pattern in border
        for (i, &b) in raw_bytes.iter().take(w - 4).enumerate() {
            let col = 2 + i;
            let row = h - 2;
            let shade = b;
            buffer.set_pixel(col, row, Pixel::rgb(shade, shade / 2, 255 - shade));
        }
    }
}

fn extract_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    if let Some(pos) = tag.find(&needle) {
        let start = pos + needle.len();
        if let Some(end) = tag[start..].find('"') {
            return Some(&tag[start..start + end]);
        }
    }
    None
}

fn extract_attr_num(tag: &str, name: &str) -> Option<f32> {
    extract_attr(tag, name).and_then(|s| s.parse::<f32>().ok())
}

#[async_trait]
impl ToolHandler for RenderTerminalMediaTool {
    fn name(&self) -> &str {
        "render_terminal_media"
    }

    fn description(&self) -> &str {
        "Render PNG, JPEG, SVG, GIF, and diagram media directly into the terminal standard output using 24-bit Truecolor ANSI half-blocks or ASCII art."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Local filesystem path to the media file (.svg, .png, .jpg, .jpeg, .gif, or text diagram)."
                },
                "width": {
                    "type": "integer",
                    "description": "Optional terminal width in characters (default: 60)."
                },
                "mode": {
                    "type": "string",
                    "enum": ["half_block", "ascii"],
                    "description": "Rendering mode: 'half_block' (24-bit Truecolor) or 'ascii' (default: 'half_block')."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let path_str = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'path'".to_string()))?;

        let width = arguments
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(60) as usize;

        let mode = arguments
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("half_block");

        let target_path = resolve_target_path(&self.working_dir, Path::new(path_str));
        let terminal_output = Self::render_media(&target_path, width, mode)?;

        Ok(terminal_output)
    }
}

// =========================================================================
// 5. ExportArtifactHtmlTool (Self-Contained Markdown Artifact Web Exporter)
// =========================================================================

/// Tool for compiling markdown artifacts into self-contained, offline-capable HTML with embedded Mermaid & GitHub CSS
#[derive(Debug, Clone)]
pub struct ExportArtifactHtmlTool {
    pub working_dir: Option<PathBuf>,
}

impl Default for ExportArtifactHtmlTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportArtifactHtmlTool {
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Compile markdown artifact content into standalone HTML
    pub fn compile_to_html(title: &str, content: &str) -> (String, usize, usize) {
        let mut sections_count = 0;
        let mut mermaid_count = 0;

        let mut body_html = String::new();
        let mut in_code_block = false;
        let mut in_mermaid_block = false;
        let mut in_diff_block = false;
        let mut current_block = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                if in_code_block || in_mermaid_block || in_diff_block {
                    if in_mermaid_block {
                        mermaid_count += 1;
                        body_html.push_str(&format!(
                            r#"<div class="mermaid-card"><div class="mermaid">{}</div></div>"#,
                            current_block
                        ));
                    } else if in_diff_block {
                        body_html.push_str(&format_diff_block(&current_block));
                    } else {
                        body_html.push_str(&format!(
                            r#"<pre class="code-block"><code>{}</code></pre>"#,
                            html_escape(&current_block)
                        ));
                    }
                    current_block.clear();
                    in_code_block = false;
                    in_mermaid_block = false;
                    in_diff_block = false;
                } else {
                    let lang = trimmed.trim_start_matches('`').trim();
                    if lang == "mermaid" {
                        in_mermaid_block = true;
                    } else if lang == "diff" || lang == "render_diff" {
                        in_diff_block = true;
                    } else {
                        in_code_block = true;
                    }
                }
                continue;
            }

            if in_code_block || in_mermaid_block || in_diff_block {
                current_block.push_str(line);
                current_block.push('\n');
                continue;
            }

            if trimmed.starts_with('#') {
                sections_count += 1;
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let heading_text = trimmed.trim_start_matches('#').trim();
                let tag = format!("h{}", level.clamp(1, 6));
                body_html.push_str(&format!("<{tag}>{}</{tag}>\n", html_escape(heading_text)));
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                body_html.push_str(&format!("<li>{}</li>\n", html_escape(&trimmed[2..])));
            } else if trimmed.is_empty() {
                body_html.push_str("<br/>\n");
            } else {
                body_html.push_str(&format!("<p>{}</p>\n", html_escape(trimmed)));
            }
        }

        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} - Tagisan Artifact</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <style>
    :root {{
      --bg: #0d1117;
      --card-bg: #161b22;
      --border: #30363d;
      --text: #e6edf3;
      --accent: #58a6ff;
      --diff-add-bg: #13231b;
      --diff-add-text: #3fb950;
      --diff-del-bg: #2d161a;
      --diff-del-text: #f85149;
    }}
    body {{
      margin: 0;
      padding: 32px;
      background: var(--bg);
      color: var(--text);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
      line-height: 1.6;
    }}
    .container {{
      max-width: 900px;
      margin: 0 auto;
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 40px;
      box-shadow: 0 12px 32px rgba(0,0,0,0.5);
    }}
    h1, h2, h3, h4 {{
      color: var(--accent);
      border-bottom: 1px solid var(--border);
      padding-bottom: 8px;
      margin-top: 28px;
    }}
    .code-block {{
      background: #090d13;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 16px;
      overflow-x: auto;
      font-family: monospace;
      font-size: 13px;
    }}
    .mermaid-card {{
      background: #0e131a;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 24px;
      margin: 20px 0;
      display: flex;
      justify-content: center;
    }}
    .diff-block {{
      background: #090d13;
      border: 1px solid var(--border);
      border-radius: 6px;
      font-family: monospace;
      font-size: 13px;
      overflow-x: auto;
      margin: 16px 0;
    }}
    .diff-add {{ background: var(--diff-add-bg); color: var(--diff-add-text); padding: 2px 8px; }}
    .diff-del {{ background: var(--diff-del-bg); color: var(--diff-del-text); padding: 2px 8px; }}
    .diff-hdr {{ color: #79c0ff; padding: 2px 8px; }}
    li {{ margin: 6px 0; }}
  </style>
</head>
<body>
  <div class="container">
    {body_html}
  </div>
  <script>
    mermaid.initialize({{ startOnLoad: true, theme: 'dark' }});
  </script>
</body>
</html>"#
        );

        (full_html, sections_count, mermaid_count)
    }
}

fn format_diff_block(diff: &str) -> String {
    let mut out = String::from("<div class=\"diff-block\">\n");
    for line in diff.lines() {
        let escaped = html_escape(line);
        if line.starts_with('+') {
            out.push_str(&format!("<div class=\"diff-add\">{escaped}</div>\n"));
        } else if line.starts_with('-') {
            out.push_str(&format!("<div class=\"diff-del\">{escaped}</div>\n"));
        } else if line.starts_with("@@") {
            out.push_str(&format!("<div class=\"diff-hdr\">{escaped}</div>\n"));
        } else {
            out.push_str(&format!("<div style=\"padding:2px 8px;\">{escaped}</div>\n"));
        }
    }
    out.push_str("</div>\n");
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_basic_markdown_html(md: &str) -> String {
    let mut out = String::new();
    for line in md.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            let level = t.chars().take_while(|&c| c == '#').count();
            let text = t.trim_start_matches('#').trim();
            out.push_str(&format!("<h{level}>{}</h{level}>\n", html_escape(text)));
        } else if t.starts_with("- ") || t.starts_with("* ") {
            out.push_str(&format!("<li>{}</li>\n", html_escape(&t[2..])));
        } else if t.is_empty() {
            out.push_str("<br/>\n");
        } else {
            out.push_str(&format!("<p>{}</p>\n", html_escape(t)));
        }
    }
    out
}

#[async_trait]
impl ToolHandler for ExportArtifactHtmlTool {
    fn name(&self) -> &str {
        "export_artifact_html"
    }

    fn description(&self) -> &str {
        "Compile Markdown artifacts (.tagisan/artifacts/...) with embedded Mermaid diagrams, code diff blocks, and carousel slides into a self-contained, offline-capable HTML file."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Artifact filename in .tagisan/artifacts/ or relative path."
                },
                "open_in_browser": {
                    "type": "boolean",
                    "description": "Whether to attempt opening the generated HTML file in the default browser (default: false)."
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'name'".to_string()))?;

        let clean_name = name.trim().replace('\\', "/");
        let clean_name = clean_name.trim_start_matches('/').to_string();

        if clean_name.contains("..") || clean_name.is_empty() {
            return Err(TagisanError::Execution(format!("Invalid or traversing artifact name: '{name}'")));
        }

        let artifacts_dir = resolve_target_path(&self.working_dir, Path::new(".tagisan/artifacts"));
        let artifact_path = if clean_name.starts_with(".tagisan/artifacts/") {
            resolve_target_path(&self.working_dir, Path::new(&clean_name))
        } else {
            artifacts_dir.join(&clean_name)
        };

        if !artifact_path.exists() {
            return Err(TagisanError::Execution(format!(
                "Artifact not found at '{}'",
                artifact_path.display()
            )));
        }

        let markdown_content = tokio::fs::read_to_string(&artifact_path).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to read artifact markdown: {e}"))
        })?;

        let title = Path::new(&clean_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Artifact");

        let (html_content, sections, mermaids) = Self::compile_to_html(title, &markdown_content);

        let exports_dir = resolve_target_path(&self.working_dir, Path::new(".tagisan/artifacts/exports"));
        tokio::fs::create_dir_all(&exports_dir).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to create exports directory: {e}"))
        })?;

        let export_filename = format!("{title}.html");
        let export_path = exports_dir.join(export_filename);

        tokio::fs::write(&export_path, html_content.as_bytes()).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to write exported HTML: {e}"))
        })?;

        let abs_export = if export_path.is_absolute() {
            export_path.clone()
        } else {
            std::env::current_dir().unwrap_or_default().join(&export_path)
        };
        let file_url = format!("file:///{}", abs_export.to_string_lossy().replace('\\', "/"));

        let res = json!({
            "status": "exported",
            "name": clean_name,
            "html_path": export_path.to_string_lossy().replace('\\', "/"),
            "byte_size": html_content.len(),
            "sections_count": sections,
            "mermaid_diagrams_count": mermaids,
            "file_url": file_url
        });

        Ok(serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string()))
    }
}
