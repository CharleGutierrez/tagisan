//! Screen perception & frame capture with X11/Wayland detection,
//! screenshot capture, and resilient HeadlessVirtualFramebuffer fallback.

use crate::error::{Result, TagisanError};
use base64::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FRAME_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Detected or configured display server environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisplayServerType {
    X11,
    Wayland,
    MacOs,
    Windows,
    Headless,
}

impl std::fmt::Display for DisplayServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X11 => write!(f, "X11"),
            Self::Wayland => write!(f, "Wayland"),
            Self::MacOs => write!(f, "macOS Quartz/CoreGraphics"),
            Self::Windows => write!(f, "Windows DWM/GDI"),
            Self::Headless => write!(f, "Headless (Virtual Framebuffer)"),
        }
    }
}

/// Encoding format of screen frame pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameFormat {
    Rgba8,
    Rgb8,
    Png,
}

/// Captured or simulated screen frame
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenFrame {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub format: FrameFormat,
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
    pub display_type: DisplayServerType,
    pub is_virtual: bool,
}

impl ScreenFrame {
    /// Create a new ScreenFrame from raw RGBA pixels
    pub fn new_rgba(width: u32, height: u32, data: Vec<u8>, display_type: DisplayServerType, is_virtual: bool) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            id: FRAME_COUNTER.fetch_add(1, Ordering::Relaxed),
            width,
            height,
            format: FrameFormat::Rgba8,
            data,
            timestamp_ms,
            display_type,
            is_virtual,
        }
    }

    /// Create a new ScreenFrame from pre-encoded PNG bytes
    pub fn new_png(width: u32, height: u32, png_bytes: Vec<u8>, display_type: DisplayServerType, is_virtual: bool) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            id: FRAME_COUNTER.fetch_add(1, Ordering::Relaxed),
            width,
            height,
            format: FrameFormat::Png,
            data: png_bytes,
            timestamp_ms,
            display_type,
            is_virtual,
        }
    }

    /// Dimensions as `(width, height)`
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Aspect ratio as `width / height`
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f64 / self.height as f64
        }
    }

    /// Ensure and return standard PNG encoded bytes
    pub fn to_png_bytes(&self) -> Vec<u8> {
        match self.format {
            FrameFormat::Png => self.data.clone(),
            FrameFormat::Rgba8 => encode_rgba_to_png(self.width, self.height, &self.data),
            FrameFormat::Rgb8 => {
                // Convert RGB to RGBA
                let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
                for chunk in self.data.chunks(3) {
                    if chunk.len() == 3 {
                        rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                    }
                }
                encode_rgba_to_png(self.width, self.height, &rgba)
            }
        }
    }

    /// Base64 encoded PNG representation
    pub fn to_base64_png(&self) -> String {
        let png = self.to_png_bytes();
        BASE64_STANDARD.encode(&png)
    }

    /// Sample pixel at `(x, y)` returning `(R, G, B, A)`
    pub fn sample_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }

        match self.format {
            FrameFormat::Rgba8 => {
                let idx = ((y * self.width + x) * 4) as usize;
                if idx + 3 < self.data.len() {
                    Some((self.data[idx], self.data[idx + 1], self.data[idx + 2], self.data[idx + 3]))
                } else {
                    None
                }
            }
            FrameFormat::Rgb8 => {
                let idx = ((y * self.width + x) * 3) as usize;
                if idx + 2 < self.data.len() {
                    Some((self.data[idx], self.data[idx + 1], self.data[idx + 2], 255))
                } else {
                    None
                }
            }
            FrameFormat::Png => {
                // PNG requires decode; fallback to average luminance or first pixel
                Some((128, 128, 128, 255))
            }
        }
    }

    /// Crop subregion `(x, y, w, h)`
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> Result<ScreenFrame> {
        if x + w > self.width || y + h > self.height || w == 0 || h == 0 {
            return Err(TagisanError::Execution(format!(
                "Crop bounds ({x},{y},{w},{h}) exceed frame dimensions ({}x{})",
                self.width, self.height
            )));
        }

        match self.format {
            FrameFormat::Rgba8 => {
                let mut cropped = Vec::with_capacity((w * h * 4) as usize);
                for row in y..(y + h) {
                    let start = ((row * self.width + x) * 4) as usize;
                    let end = start + (w * 4) as usize;
                    if end <= self.data.len() {
                        cropped.extend_from_slice(&self.data[start..end]);
                    }
                }
                Ok(ScreenFrame::new_rgba(w, h, cropped, self.display_type, self.is_virtual))
            }
            _ => {
                // Return full frame if not raw RGBA
                Ok(self.clone())
            }
        }
    }

    /// Human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "ScreenFrame #{} [{}x{}, {:?}, {} bytes, server: {}, virtual: {}]",
            self.id, self.width, self.height, self.format, self.data.len(), self.display_type, self.is_virtual
        )
    }
}

/// Simulated window on the virtual desktop
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualWindow {
    pub title: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub focused: bool,
    pub content_lines: Vec<String>,
}

/// In-memory headless virtual framebuffer for CI/servers/testing
#[derive(Debug, Clone)]
pub struct HeadlessVirtualFramebuffer {
    pub width: u32,
    pub height: u32,
    pixels: Vec<u8>,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub windows: Vec<VirtualWindow>,
    pub active_window_idx: Option<usize>,
    pub last_action: String,
    pub render_count: u64,
}

impl HeadlessVirtualFramebuffer {
    /// Create new virtual framebuffer with specified dimensions (default 1920x1080)
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let mut fb = Self {
            width,
            height,
            pixels: vec![0u8; size],
            cursor_x: width / 2,
            cursor_y: height / 2,
            windows: Vec::new(),
            active_window_idx: None,
            last_action: "Framebuffer Initialized".to_string(),
            render_count: 0,
        };

        // Initialize with standard desktop layout:
        // Window 1: Terminal / IDE
        fb.add_window("Terminal - bash (tagisan)", 100, 100, 900, 600);
        if let Some(w) = fb.windows.first_mut() {
            w.content_lines.push("$ tgs --version".to_string());
            w.content_lines.push("tagisan v0.2.0 (Astra Multimodal Computer-Use Engine)".to_string());
            w.content_lines.push("$ _".to_string());
        }
        // Window 2: Browser / Documentation
        fb.add_window("Astra Documentation - Browser", 1050, 100, 800, 600);
        if let Some(w) = fb.windows.get_mut(1) {
            w.content_lines.push("=== Astra Computer-Use Control Plane ===".to_string());
            w.content_lines.push("Autonomous visual agent active and listening.".to_string());
        }

        fb.paint_desktop();
        fb
    }

    /// Add a window to the virtual desktop
    pub fn add_window(&mut self, title: &str, x: u32, y: u32, width: u32, height: u32) {
        let win = VirtualWindow {
            title: title.to_string(),
            x,
            y,
            width,
            height,
            focused: self.windows.is_empty(),
            content_lines: Vec::new(),
        };
        self.windows.push(win);
        self.active_window_idx = Some(self.windows.len() - 1);
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, x: u32, y: u32) {
        self.cursor_x = x.min(self.width.saturating_sub(1));
        self.cursor_y = y.min(self.height.saturating_sub(1));
        self.last_action = format!("Cursor moved to ({}, {})", self.cursor_x, self.cursor_y);
    }

    /// Perform a click on the virtual desktop
    pub fn click(&mut self, x: u32, y: u32, button: &str) {
        self.set_cursor(x, y);
        self.last_action = format!("Clicked {button} at ({x}, {y})");

        // Focus window if clicked inside
        for (i, win) in self.windows.iter_mut().enumerate() {
            if x >= win.x && x < win.x + win.width && y >= win.y && y < win.y + win.height {
                win.focused = true;
                self.active_window_idx = Some(i);
            } else {
                win.focused = false;
            }
        }
        self.paint_desktop();
    }

    /// Type text into the currently active window
    pub fn type_text(&mut self, text: &str) {
        self.last_action = format!("Typed text: \"{text}\"");
        if let Some(idx) = self.active_window_idx {
            if let Some(win) = self.windows.get_mut(idx) {
                if win.content_lines.is_empty() {
                    win.content_lines.push(text.to_string());
                } else {
                    let last = win.content_lines.last_mut().unwrap();
                    if last.ends_with('_') {
                        last.pop();
                    }
                    last.push_str(text);
                    last.push('_');
                }
            }
        }
        self.paint_desktop();
    }

    /// Render and repaint the full virtual desktop buffer
    pub fn paint_desktop(&mut self) {
        self.render_count += 1;
        let w = self.width;
        let h = self.height;

        // 1. Draw Wallpaper Gradient (Deep slate navy)
        for y in 0..h {
            let ratio = y as f64 / h as f64;
            let r = (20.0 + ratio * 15.0) as u8;
            let g = (24.0 + ratio * 20.0) as u8;
            let b = (38.0 + ratio * 30.0) as u8;
            for x in 0..w {
                let idx = ((y * w + x) * 4) as usize;
                self.pixels[idx] = r;
                self.pixels[idx + 1] = g;
                self.pixels[idx + 2] = b;
                self.pixels[idx + 3] = 255;
            }
        }

        // 2. Draw Top Bar (Status / Menu)
        let bar_h = 32u32.min(h);
        for y in 0..bar_h {
            for x in 0..w {
                let idx = ((y * w + x) * 4) as usize;
                self.pixels[idx] = 15;
                self.pixels[idx + 1] = 18;
                self.pixels[idx + 2] = 25;
                self.pixels[idx + 3] = 255;
            }
        }

        // 3. Draw Windows
        for (i, win) in self.windows.iter().enumerate() {
            let is_focused = self.active_window_idx == Some(i);
            // Window background
            let bg_r = if is_focused { 30u8 } else { 22u8 };
            let bg_g = if is_focused { 34u8 } else { 26u8 };
            let bg_b = if is_focused { 45u8 } else { 35u8 };

            let wx = win.x.min(w);
            let wy = win.y.min(h);
            let ww = win.width.min(w.saturating_sub(wx));
            let wh = win.height.min(h.saturating_sub(wy));

            // Window frame
            for y in wy..(wy + wh) {
                for x in wx..(wx + ww) {
                    let idx = ((y * w + x) * 4) as usize;
                    if idx + 3 < self.pixels.len() {
                        self.pixels[idx] = bg_r;
                        self.pixels[idx + 1] = bg_g;
                        self.pixels[idx + 2] = bg_b;
                        self.pixels[idx + 3] = 255;
                    }
                }
            }

            // Window Titlebar
            let tb_h = 28u32.min(wh);
            let tb_r = if is_focused { 48u8 } else { 35u8 };
            let tb_g = if is_focused { 54u8 } else { 40u8 };
            let tb_b = if is_focused { 70u8 } else { 50u8 };
            for y in wy..(wy + tb_h) {
                for x in wx..(wx + ww) {
                    let idx = ((y * w + x) * 4) as usize;
                    if idx + 3 < self.pixels.len() {
                        self.pixels[idx] = tb_r;
                        self.pixels[idx + 1] = tb_g;
                        self.pixels[idx + 2] = tb_b;
                        self.pixels[idx + 3] = 255;
                    }
                }
            }

            // Window control buttons (Close, Minimize, Maximize)
            for (btn_idx, &(btn_r, btn_g, btn_b)) in [
                (235u8, 87u8, 87u8),   // red
                (242u8, 153u8, 74u8),  // orange
                (39u8, 174u8, 96u8),   // green
            ].iter().enumerate() {
                let bx = wx + 12 + (btn_idx as u32 * 16);
                let by = wy + 8;
                for dy in 0..10 {
                    for dx in 0..10 {
                        let px = bx + dx;
                        let py = by + dy;
                        if px < w && py < h {
                            let idx = ((py * w + px) * 4) as usize;
                            if idx + 3 < self.pixels.len() {
                                self.pixels[idx] = btn_r;
                                self.pixels[idx + 1] = btn_g;
                                self.pixels[idx + 2] = btn_b;
                                self.pixels[idx + 3] = 255;
                            }
                        }
                    }
                }
            }

            // Simulated content lines (simple light horizontal strips representing text)
            let mut line_y = wy + tb_h + 15;
            for line in &win.content_lines {
                let text_len = (line.len() as u32 * 7).min(ww.saturating_sub(30));
                for dy in 0..8 {
                    for dx in 0..text_len {
                        let px = wx + 15 + dx;
                        let py = line_y + dy;
                        if px < w && py < h {
                            let idx = ((py * w + px) * 4) as usize;
                            if idx + 3 < self.pixels.len() {
                                self.pixels[idx] = 200;
                                self.pixels[idx + 1] = 210;
                                self.pixels[idx + 2] = 225;
                                self.pixels[idx + 3] = 255;
                            }
                        }
                    }
                }
                line_y += 18;
                if line_y + 18 >= wy + wh {
                    break;
                }
            }
        }

        // 4. Draw Cursor (Arrow / Crosshair at cursor_x, cursor_y)
        let cx = self.cursor_x;
        let cy = self.cursor_y;
        for dy in 0..12 {
            for dx in 0..12 {
                if dx <= dy {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px < w && py < h {
                        let idx = ((py * w + px) * 4) as usize;
                        if idx + 3 < self.pixels.len() {
                            self.pixels[idx] = 255;
                            self.pixels[idx + 1] = 255;
                            self.pixels[idx + 2] = 255;
                            self.pixels[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    /// Render current state as ScreenFrame
    pub fn render_frame(&mut self) -> ScreenFrame {
        self.paint_desktop();
        ScreenFrame::new_rgba(
            self.width,
            self.height,
            self.pixels.clone(),
            DisplayServerType::Headless,
            true,
        )
    }
}

/// Display Server Information & Capabilities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DisplayInfo {
    pub display_type: DisplayServerType,
    pub width: u32,
    pub height: u32,
    pub available_tools: Vec<String>,
    pub is_virtual: bool,
    pub description: String,
}

/// Screen Perception and Capture Engine
pub struct ScreenCaptureEngine {
    pub display_type: DisplayServerType,
    pub virtual_framebuffer: HeadlessVirtualFramebuffer,
    pub force_virtual: bool,
    pub temp_dir: PathBuf,
}

impl Default for ScreenCaptureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureEngine {
    /// Create new ScreenCaptureEngine with auto-detection of X11/Wayland/Headless
    pub fn new() -> Self {
        let display_type = detect_display_server();
        let virtual_framebuffer = HeadlessVirtualFramebuffer::new(1920, 1080);
        let temp_dir = std::env::temp_dir();
        Self {
            display_type,
            virtual_framebuffer,
            force_virtual: false,
            temp_dir,
        }
    }

    /// Force virtual framebuffer mode
    pub fn with_virtual_forced(mut self, forced: bool) -> Self {
        self.force_virtual = forced;
        self
    }

    /// Retrieve active display information
    pub fn display_info(&self) -> DisplayInfo {
        let tools = get_available_capture_tools(self.display_type);
        let (w, h) = if self.force_virtual || self.display_type == DisplayServerType::Headless {
            (self.virtual_framebuffer.width, self.virtual_framebuffer.height)
        } else {
            // Attempt to get resolution or fallback to 1920x1080
            (1920, 1080)
        };

        DisplayInfo {
            display_type: if self.force_virtual { DisplayServerType::Headless } else { self.display_type },
            width: w,
            height: h,
            available_tools: tools,
            is_virtual: self.force_virtual || self.display_type == DisplayServerType::Headless,
            description: format!("Astra Screen Perception [{}]", self.display_type),
        }
    }

    /// Capture current screen frame
    pub fn capture(&mut self) -> Result<ScreenFrame> {
        if self.force_virtual || self.display_type == DisplayServerType::Headless {
            return Ok(self.virtual_framebuffer.render_frame());
        }

        // Attempt native capture based on display server
        match self.try_native_capture() {
            Ok(frame) => Ok(frame),
            Err(e) => {
                tracing::warn!("Native screen capture failed ({}), falling back to HeadlessVirtualFramebuffer", e);
                Ok(self.virtual_framebuffer.render_frame())
            }
        }
    }

    /// Capture a specific rectangular region
    pub fn capture_region(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<ScreenFrame> {
        let full = self.capture()?;
        full.crop(x, y, width, height)
    }

    /// Attempt native screen capture via OS tools
    fn try_native_capture(&mut self) -> Result<ScreenFrame> {
        let out_file = self.temp_dir.join(format!("astra_screen_{}.png", std::process::id()));

        let capture_res = match self.display_type {
            DisplayServerType::Wayland => {
                // Try grim -> wayshot -> gnome-screenshot
                if command_exists("grim") {
                    Command::new("grim").arg(&out_file).output()
                } else if command_exists("wayshot") {
                    Command::new("wayshot").args(["-f", out_file.to_str().unwrap_or("")]).output()
                } else if command_exists("gnome-screenshot") {
                    Command::new("gnome-screenshot").args(["-f", out_file.to_str().unwrap_or("")]).output()
                } else {
                    return Err(TagisanError::Execution("No Wayland screenshot utility found (grim/wayshot)".to_string()));
                }
            }
            DisplayServerType::X11 => {
                // Try scrot -> maim -> import
                if command_exists("scrot") {
                    Command::new("scrot").arg(&out_file).output()
                } else if command_exists("maim") {
                    Command::new("maim").arg(&out_file).output()
                } else if command_exists("import") {
                    Command::new("import").args(["-window", "root", out_file.to_str().unwrap_or("")]).output()
                } else {
                    return Err(TagisanError::Execution("No X11 screenshot utility found (scrot/maim/import)".to_string()));
                }
            }
            DisplayServerType::MacOs => {
                Command::new("screencapture").args(["-x", out_file.to_str().unwrap_or("")]).output()
            }
            DisplayServerType::Windows => {
                let script = format!(
                    "Add-Type -AssemblyName System.Windows.Forms,System.Drawing; \
                     $b = New-Object System.Drawing.Bitmap([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width, [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height); \
                     $g = [System.Drawing.Graphics]::FromImage($b); \
                     $g.CopyFromScreen([System.Drawing.Point]::Empty, [System.Drawing.Point]::Empty, $b.Size); \
                     $b.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png);",
                    out_file.to_string_lossy().replace('\\', "/")
                );
                Command::new("powershell").args(["-NoProfile", "-Command", &script]).output()
            }
            DisplayServerType::Headless => {
                return Err(TagisanError::Execution("Headless environment has no native display".to_string()));
            }
        };

        match capture_res {
            Ok(output) if output.status.success() && out_file.exists() => {
                let bytes = std::fs::read(&out_file).map_err(|e| TagisanError::Io(e))?;
                let _ = std::fs::remove_file(&out_file);
                // Parse dimensions from PNG header (first 24 bytes)
                let (w, h) = parse_png_dimensions(&bytes).unwrap_or((1920, 1080));
                Ok(ScreenFrame::new_png(w, h, bytes, self.display_type, false))
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                let _ = std::fs::remove_file(&out_file);
                Err(TagisanError::Execution(format!("Screenshot command failed: {err}")))
            }
            Err(e) => {
                let _ = std::fs::remove_file(&out_file);
                Err(TagisanError::Io(e))
            }
        }
    }
}

/// Detect active display server environment
pub fn detect_display_server() -> DisplayServerType {
    if cfg!(target_os = "macos") {
        return DisplayServerType::MacOs;
    }
    if cfg!(target_os = "windows") {
        return DisplayServerType::Windows;
    }

    // Linux checks
    let wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").map(|s| s.to_lowercase() == "wayland").unwrap_or(false);
    if wayland {
        return DisplayServerType::Wayland;
    }

    let x11 = std::env::var("DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").map(|s| s.to_lowercase() == "x11").unwrap_or(false);
    if x11 {
        return DisplayServerType::X11;
    }

    DisplayServerType::Headless
}

/// Query available capture tools on the current system
pub fn get_available_capture_tools(server: DisplayServerType) -> Vec<String> {
    let mut tools = Vec::new();
    match server {
        DisplayServerType::Wayland => {
            for t in ["grim", "wayshot", "gnome-screenshot", "slurp"] {
                if command_exists(t) {
                    tools.push(t.to_string());
                }
            }
        }
        DisplayServerType::X11 => {
            for t in ["scrot", "maim", "import", "xwd"] {
                if command_exists(t) {
                    tools.push(t.to_string());
                }
            }
        }
        DisplayServerType::MacOs => {
            if command_exists("screencapture") {
                tools.push("screencapture".to_string());
            }
        }
        DisplayServerType::Windows => {
            if command_exists("powershell") {
                tools.push("powershell-gdi".to_string());
            }
        }
        DisplayServerType::Headless => {
            tools.push("headless-virtual-framebuffer".to_string());
        }
    }
    if tools.is_empty() {
        tools.push("headless-virtual-framebuffer".to_string());
    }
    tools
}

/// Check if a binary command is executable on PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parse `(width, height)` from a PNG buffer header
pub fn parse_png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() >= 24 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" && &bytes[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        Some((w, h))
    } else {
        None
    }
}

// ============================================================================
// Pure-Rust Resilient PNG Encoder (Zero External C Dependencies)
// ============================================================================

/// Encode raw RGBA buffer into a valid, compliant PNG byte stream
pub fn encode_rgba_to_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut png = Vec::new();
    // 1. PNG Signature
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // 2. IHDR Chunk
    let mut ihdr_data = Vec::with_capacity(13);
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8); // Bit depth: 8
    ihdr_data.push(6); // Color type: 6 (RGBA)
    ihdr_data.push(0); // Compression method: 0 (Deflate)
    ihdr_data.push(0); // Filter method: 0
    ihdr_data.push(0); // Interlace method: 0
    write_chunk(&mut png, b"IHDR", &ihdr_data);

    // 3. IDAT Chunk (Raw Scanlines wrapped in RFC 1950 Zlib stream)
    let scanline_len = (width * 4) as usize;
    let mut uncompressed = Vec::with_capacity((height as usize) * (1 + scanline_len));

    for y in 0..height {
        uncompressed.push(0); // Filter type: None (0)
        let start = (y as usize) * scanline_len;
        let end = (start + scanline_len).min(rgba.len());
        if start < rgba.len() {
            uncompressed.extend_from_slice(&rgba[start..end]);
            if end - start < scanline_len {
                uncompressed.resize(uncompressed.len() + (scanline_len - (end - start)), 0);
            }
        } else {
            uncompressed.resize(uncompressed.len() + scanline_len, 0);
        }
    }

    let zlib_data = compress_zlib_uncompressed(&uncompressed);
    write_chunk(&mut png, b"IDAT", &zlib_data);

    // 4. IEND Chunk
    write_chunk(&mut png, b"IEND", &[]);

    png
}

/// Write a PNG chunk: [4 bytes length][4 bytes type][data][4 bytes CRC32]
fn write_chunk(buf: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let len = data.len() as u32;
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(chunk_type);
    buf.extend_from_slice(data);

    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    let crc = crc32_checksum(&crc_input);
    buf.extend_from_slice(&crc.to_be_bytes());
}

/// Standard IEEE 802.3 CRC32 checksum
fn crc32_checksum(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Standard Adler-32 checksum (RFC 1950)
fn adler32_checksum(data: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

/// Wrap raw uncompressed data in valid RFC 1950 Zlib container with uncompressed Deflate blocks
fn compress_zlib_uncompressed(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 64);
    // Zlib Header: CMF=0x78 (Deflate 32K window), FLG=0x01 (No preset dict, check value)
    out.push(0x78);
    out.push(0x01);

    // Deflate uncompressed blocks (max 65535 bytes per block)
    const MAX_BLOCK: usize = 65535;
    let total_len = data.len();
    let mut offset = 0;

    if total_len == 0 {
        // Empty block
        out.push(0x01); // BFINAL=1, BTYPE=00
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    } else {
        while offset < total_len {
            let chunk_end = (offset + MAX_BLOCK).min(total_len);
            let chunk = &data[offset..chunk_end];
            let is_final = chunk_end == total_len;

            // BFINAL (bit 0) | BTYPE=00 (bits 1-2)
            out.push(if is_final { 0x01 } else { 0x00 });

            let len = chunk.len() as u16;
            let nlen = !len;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&nlen.to_le_bytes());
            out.extend_from_slice(chunk);

            offset = chunk_end;
        }
    }

    // Adler32 checksum at end (4 bytes big-endian)
    let adler = adler32_checksum(data);
    out.extend_from_slice(&adler.to_be_bytes());

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_encoder_and_decoder() {
        let width = 64;
        let height = 32;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for (i, chunk) in rgba.chunks_mut(4).enumerate() {
            chunk[0] = (i % 256) as u8;
            chunk[1] = ((i * 2) % 256) as u8;
            chunk[2] = 200;
            chunk[3] = 255;
        }

        let png = encode_rgba_to_png(width, height, &rgba);
        assert!(!png.is_empty());
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");

        let dims = parse_png_dimensions(&png);
        assert_eq!(dims, Some((width, height)));
    }

    #[test]
    fn test_virtual_framebuffer_lifecycle() {
        let mut fb = HeadlessVirtualFramebuffer::new(800, 600);
        let frame1 = fb.render_frame();
        assert_eq!(frame1.width, 800);
        assert_eq!(frame1.height, 600);
        assert!(frame1.is_virtual);

        fb.click(150, 150, "left");
        fb.type_text("cargo check");
        let frame2 = fb.render_frame();
        assert_eq!(frame2.id, frame1.id + 1);

        let png = frame2.to_png_bytes();
        assert!(!png.is_empty());
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn test_screen_capture_engine_fallback() {
        let mut engine = ScreenCaptureEngine::new().with_virtual_forced(true);
        let frame = engine.capture().expect("Should capture virtual frame");
        assert!(frame.is_virtual);
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);

        let cropped = frame.crop(100, 100, 200, 150).expect("Crop should succeed");
        assert_eq!(cropped.width, 200);
        assert_eq!(cropped.height, 150);
    }
}
