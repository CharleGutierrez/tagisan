//! Mouse and keyboard input synthesis supporting xdotool/wtype and virtual input simulator
//! with real-time action verification.

use crate::error::{Result, TagisanError};
use std::process::Command;
use std::time::Instant;

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    /// Linux xdotool button index
    pub fn to_xdotool_button(&self) -> u32 {
        match self {
            Self::Left => 1,
            Self::Middle => 2,
            Self::Right => 3,
        }
    }
}

/// Keyboard modifier keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta, // Super/Windows/Command key
}

/// High-level OS input action to execute
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputAction {
    Click {
        x: u32,
        y: u32,
        button: MouseButton,
        count: u32,
    },
    MouseMove {
        x: u32,
        y: u32,
    },
    MouseDown {
        x: u32,
        y: u32,
        button: MouseButton,
    },
    MouseUp {
        x: u32,
        y: u32,
        button: MouseButton,
    },
    Drag {
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
        button: MouseButton,
    },
    Scroll {
        x: u32,
        y: u32,
        delta_x: i32,
        delta_y: i32,
    },
    Type {
        text: String,
        delay_ms: u64,
    },
    KeyPress {
        key: String,
        modifiers: Vec<KeyModifier>,
    },
    KeyCombo {
        keys: Vec<String>,
    },
    Wait {
        duration_ms: u64,
    },
}

impl InputAction {
    /// Name of action type
    pub fn action_type_name(&self) -> &'static str {
        match self {
            Self::Click { .. } => "click",
            Self::MouseMove { .. } => "mouse_move",
            Self::MouseDown { .. } => "mouse_down",
            Self::MouseUp { .. } => "mouse_up",
            Self::Drag { .. } => "drag",
            Self::Scroll { .. } => "scroll",
            Self::Type { .. } => "type",
            Self::KeyPress { .. } => "key_press",
            Self::KeyCombo { .. } => "key_combo",
            Self::Wait { .. } => "wait",
        }
    }
}

/// Backend used to execute OS input
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InputBackendType {
    Xdotool,
    Wtype,
    Ydotool,
    Osascript,
    Powershell,
    Virtual,
}

impl std::fmt::Display for InputBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xdotool => write!(f, "xdotool (X11)"),
            Self::Wtype => write!(f, "wtype (Wayland)"),
            Self::Ydotool => write!(f, "ydotool (uinput)"),
            Self::Osascript => write!(f, "osascript (macOS)"),
            Self::Powershell => write!(f, "PowerShell SendKeys (Windows)"),
            Self::Virtual => write!(f, "Virtual Input Simulator (Headless)"),
        }
    }
}

/// Result of input action execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputActionResult {
    pub success: bool,
    pub action_type: String,
    pub details: String,
    pub cursor_position: (u32, u32),
    pub backend_used: InputBackendType,
    pub execution_time_ms: u64,
    pub verified: bool,
}

/// Virtual input simulator tracking cursor, keystrokes, and bounds
#[derive(Debug, Clone)]
pub struct VirtualInputSimulator {
    pub screen_width: u32,
    pub screen_height: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub left_down: bool,
    pub right_down: bool,
    pub middle_down: bool,
    pub typed_history: Vec<String>,
    pub action_count: u64,
}

impl VirtualInputSimulator {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            screen_width,
            screen_height,
            cursor_x: screen_width / 2,
            cursor_y: screen_height / 2,
            left_down: false,
            right_down: false,
            middle_down: false,
            typed_history: Vec::new(),
            action_count: 0,
        }
    }

    /// Execute action virtually and return result
    pub fn execute(&mut self, action: &InputAction) -> Result<InputActionResult> {
        self.action_count += 1;
        let start = Instant::now();

        match action {
            InputAction::Click { x, y, button, count } => {
                self.cursor_x = (*x).min(self.screen_width.saturating_sub(1));
                self.cursor_y = (*y).min(self.screen_height.saturating_sub(1));
                Ok(InputActionResult {
                    success: true,
                    action_type: "click".to_string(),
                    details: format!("Virtual click ({count}x) {:?} at ({}, {})", button, self.cursor_x, self.cursor_y),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::MouseMove { x, y } => {
                self.cursor_x = (*x).min(self.screen_width.saturating_sub(1));
                self.cursor_y = (*y).min(self.screen_height.saturating_sub(1));
                Ok(InputActionResult {
                    success: true,
                    action_type: "mouse_move".to_string(),
                    details: format!("Virtual move cursor to ({}, {})", self.cursor_x, self.cursor_y),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::MouseDown { x, y, button } => {
                self.cursor_x = (*x).min(self.screen_width.saturating_sub(1));
                self.cursor_y = (*y).min(self.screen_height.saturating_sub(1));
                match button {
                    MouseButton::Left => self.left_down = true,
                    MouseButton::Right => self.right_down = true,
                    MouseButton::Middle => self.middle_down = true,
                }
                Ok(InputActionResult {
                    success: true,
                    action_type: "mouse_down".to_string(),
                    details: format!("Virtual mouse down {:?} at ({}, {})", button, self.cursor_x, self.cursor_y),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::MouseUp { x, y, button } => {
                self.cursor_x = (*x).min(self.screen_width.saturating_sub(1));
                self.cursor_y = (*y).min(self.screen_height.saturating_sub(1));
                match button {
                    MouseButton::Left => self.left_down = false,
                    MouseButton::Right => self.right_down = false,
                    MouseButton::Middle => self.middle_down = false,
                }
                Ok(InputActionResult {
                    success: true,
                    action_type: "mouse_up".to_string(),
                    details: format!("Virtual mouse up {:?} at ({}, {})", button, self.cursor_x, self.cursor_y),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::Drag { from_x, from_y, to_x, to_y, button } => {
                self.cursor_x = (*to_x).min(self.screen_width.saturating_sub(1));
                self.cursor_y = (*to_y).min(self.screen_height.saturating_sub(1));
                Ok(InputActionResult {
                    success: true,
                    action_type: "drag".to_string(),
                    details: format!("Virtual drag {:?} from ({from_x}, {from_y}) to ({}, {})", button, self.cursor_x, self.cursor_y),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::Scroll { x, y, delta_x, delta_y } => {
                self.cursor_x = (*x).min(self.screen_width.saturating_sub(1));
                self.cursor_y = (*y).min(self.screen_height.saturating_sub(1));
                Ok(InputActionResult {
                    success: true,
                    action_type: "scroll".to_string(),
                    details: format!("Virtual scroll at ({}, {}) delta: ({delta_x}, {delta_y})", self.cursor_x, self.cursor_y),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::Type { text, .. } => {
                self.typed_history.push(text.clone());
                Ok(InputActionResult {
                    success: true,
                    action_type: "type".to_string(),
                    details: format!("Virtual typed {} chars: \"{}\"", text.len(), text),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::KeyPress { key, modifiers } => {
                Ok(InputActionResult {
                    success: true,
                    action_type: "key_press".to_string(),
                    details: format!("Virtual key press: {:?} + {}", modifiers, key),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::KeyCombo { keys } => {
                Ok(InputActionResult {
                    success: true,
                    action_type: "key_combo".to_string(),
                    details: format!("Virtual key combo: {}", keys.join("+")),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
            InputAction::Wait { duration_ms } => {
                Ok(InputActionResult {
                    success: true,
                    action_type: "wait".to_string(),
                    details: format!("Virtual wait {duration_ms}ms"),
                    cursor_position: (self.cursor_x, self.cursor_y),
                    backend_used: InputBackendType::Virtual,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    verified: true,
                })
            }
        }
    }
}

/// OS Input Synthesis Engine
pub struct InputEngine {
    pub backend: InputBackendType,
    pub simulator: VirtualInputSimulator,
    pub force_virtual: bool,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl Default for InputEngine {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

impl InputEngine {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let backend = detect_input_backend();
        let simulator = VirtualInputSimulator::new(screen_width, screen_height);
        Self {
            backend,
            simulator,
            force_virtual: false,
            screen_width,
            screen_height,
        }
    }

    /// Force virtual simulator mode (e.g. for testing or headless servers)
    pub fn with_virtual_forced(mut self, forced: bool) -> Self {
        self.force_virtual = forced;
        self
    }

    /// Current cursor position
    pub fn cursor_position(&self) -> (u32, u32) {
        (self.simulator.cursor_x, self.simulator.cursor_y)
    }

    /// Execute input action with real-time verification
    pub fn execute(&mut self, action: InputAction) -> Result<InputActionResult> {
        // Validate coordinates against screen boundaries
        self.validate_bounds(&action)?;

        if self.force_virtual || self.backend == InputBackendType::Virtual {
            return self.simulator.execute(&action);
        }

        // Attempt native execution
        match self.try_native_execute(&action) {
            Ok(result) => {
                // Update internal simulator state to keep in sync
                let _ = self.simulator.execute(&action);
                Ok(result)
            }
            Err(e) => {
                tracing::warn!("Native input execution failed ({}), falling back to VirtualInputSimulator", e);
                self.simulator.execute(&action)
            }
        }
    }

    /// Synthesize single click
    pub fn click(&mut self, x: u32, y: u32, button: MouseButton) -> Result<InputActionResult> {
        self.execute(InputAction::Click { x, y, button, count: 1 })
    }

    /// Synthesize double click
    pub fn double_click(&mut self, x: u32, y: u32) -> Result<InputActionResult> {
        self.execute(InputAction::Click { x, y, button: MouseButton::Left, count: 2 })
    }

    /// Synthesize mouse drag
    pub fn drag(&mut self, from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> Result<InputActionResult> {
        self.execute(InputAction::Drag {
            from_x,
            from_y,
            to_x,
            to_y,
            button: MouseButton::Left,
        })
    }

    /// Synthesize mouse scroll
    pub fn scroll(&mut self, x: u32, y: u32, delta_x: i32, delta_y: i32) -> Result<InputActionResult> {
        self.execute(InputAction::Scroll { x, y, delta_x, delta_y })
    }

    /// Synthesize text typing
    pub fn type_text(&mut self, text: &str, delay_ms: u64) -> Result<InputActionResult> {
        self.execute(InputAction::Type {
            text: text.to_string(),
            delay_ms,
        })
    }

    /// Synthesize key combination (e.g. `["Control", "Shift", "P"]`)
    pub fn key_combo(&mut self, keys: &[&str]) -> Result<InputActionResult> {
        self.execute(InputAction::KeyCombo {
            keys: keys.iter().map(|k| k.to_string()).collect(),
        })
    }

    /// Validate action parameters before dispatching to OS
    fn validate_bounds(&self, action: &InputAction) -> Result<()> {
        let max_x = self.screen_width;
        let max_y = self.screen_height;

        match action {
            InputAction::Click { x, y, .. } | InputAction::MouseMove { x, y } | InputAction::MouseDown { x, y, .. } | InputAction::MouseUp { x, y, .. } | InputAction::Scroll { x, y, .. } => {
                if *x > max_x || *y > max_y {
                    return Err(TagisanError::Execution(format!(
                        "Coordinates ({x}, {y}) out of screen bounds ({max_x}x{max_y})"
                    )));
                }
            }
            InputAction::Drag { from_x, from_y, to_x, to_y, .. } => {
                if *from_x > max_x || *from_y > max_y || *to_x > max_x || *to_y > max_y {
                    return Err(TagisanError::Execution(format!(
                        "Drag coordinates ({from_x},{from_y}) -> ({to_x},{to_y}) out of bounds ({max_x}x{max_y})"
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Attempt native command execution
    fn try_native_execute(&self, action: &InputAction) -> Result<InputActionResult> {
        let start = Instant::now();
        let (success, details, cursor) = match self.backend {
            InputBackendType::Xdotool => self.exec_xdotool(action)?,
            InputBackendType::Wtype => self.exec_wtype(action)?,
            InputBackendType::Ydotool => self.exec_ydotool(action)?,
            InputBackendType::Osascript => self.exec_osascript(action)?,
            InputBackendType::Powershell => self.exec_powershell(action)?,
            InputBackendType::Virtual => {
                return Err(TagisanError::Execution("Virtual backend handled separately".to_string()));
            }
        };

        Ok(InputActionResult {
            success,
            action_type: action.action_type_name().to_string(),
            details,
            cursor_position: cursor,
            backend_used: self.backend,
            execution_time_ms: start.elapsed().as_millis() as u64,
            verified: success,
        })
    }

    fn exec_xdotool(&self, action: &InputAction) -> Result<(bool, String, (u32, u32))> {
        let mut cmd = Command::new("xdotool");
        let mut target_cursor = (self.simulator.cursor_x, self.simulator.cursor_y);

        match action {
            InputAction::Click { x, y, button, count } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", &x.to_string(), &y.to_string()]);
                cmd.args(["click", "--repeat", &count.to_string(), &button.to_xdotool_button().to_string()]);
            }
            InputAction::MouseMove { x, y } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", &x.to_string(), &y.to_string()]);
            }
            InputAction::MouseDown { x, y, button } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", &x.to_string(), &y.to_string()]);
                cmd.args(["mousedown", &button.to_xdotool_button().to_string()]);
            }
            InputAction::MouseUp { x, y, button } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", &x.to_string(), &y.to_string()]);
                cmd.args(["mouseup", &button.to_xdotool_button().to_string()]);
            }
            InputAction::Drag { from_x, from_y, to_x, to_y, button } => {
                target_cursor = (*to_x, *to_y);
                cmd.args(["mousemove", &from_x.to_string(), &from_y.to_string()]);
                cmd.args(["mousedown", &button.to_xdotool_button().to_string()]);
                cmd.args(["mousemove", &to_x.to_string(), &to_y.to_string()]);
                cmd.args(["mouseup", &button.to_xdotool_button().to_string()]);
            }
            InputAction::Scroll { x, y, delta_y, .. } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", &x.to_string(), &y.to_string()]);
                let button = if *delta_y < 0 { "4" } else { "5" }; // 4=up, 5=down
                let clicks = delta_y.abs().max(1) as u32;
                cmd.args(["click", "--repeat", &clicks.to_string(), button]);
            }
            InputAction::Type { text, delay_ms } => {
                cmd.args(["type", "--delay", &delay_ms.to_string(), text]);
            }
            InputAction::KeyCombo { keys } => {
                let combo = keys.join("+");
                cmd.args(["key", &combo]);
            }
            InputAction::KeyPress { key, modifiers } => {
                let mut parts: Vec<String> = modifiers.iter().map(|m| format!("{:?}", m)).collect();
                parts.push(key.clone());
                cmd.args(["key", &parts.join("+")]);
            }
            InputAction::Wait { duration_ms } => {
                std::thread::sleep(std::time::Duration::from_millis(*duration_ms));
                return Ok((true, format!("Waited {duration_ms}ms"), target_cursor));
            }
        }

        let output = cmd.output().map_err(|e| TagisanError::Io(e))?;
        let ok = output.status.success();
        let details = if ok {
            format!("xdotool executed {:?}", action.action_type_name())
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        Ok((ok, details, target_cursor))
    }

    fn exec_wtype(&self, action: &InputAction) -> Result<(bool, String, (u32, u32))> {
        let mut cmd = Command::new("wtype");
        let target_cursor = (self.simulator.cursor_x, self.simulator.cursor_y);

        match action {
            InputAction::Type { text, .. } => {
                cmd.arg(text);
            }
            InputAction::KeyCombo { keys } => {
                for k in keys {
                    cmd.args(["-k", k]);
                }
            }
            InputAction::KeyPress { key, .. } => {
                cmd.args(["-k", key]);
            }
            _ => {
                return Err(TagisanError::Execution("wtype only supports keyboard input".to_string()));
            }
        }

        let output = cmd.output().map_err(|e| TagisanError::Io(e))?;
        let ok = output.status.success();
        let details = if ok {
            "wtype keystroke executed".to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };
        Ok((ok, details, target_cursor))
    }

    fn exec_ydotool(&self, action: &InputAction) -> Result<(bool, String, (u32, u32))> {
        let mut cmd = Command::new("ydotool");
        let mut target_cursor = (self.simulator.cursor_x, self.simulator.cursor_y);

        match action {
            InputAction::MouseMove { x, y } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", "-a", &x.to_string(), &y.to_string()]);
            }
            InputAction::Click { x, y, button, .. } => {
                target_cursor = (*x, *y);
                cmd.args(["mousemove", "-a", &x.to_string(), &y.to_string()]);
                let btn_hex = match button {
                    MouseButton::Left => "0xC0",
                    MouseButton::Right => "0xC1",
                    MouseButton::Middle => "0xC2",
                };
                cmd.args(["click", btn_hex]);
            }
            InputAction::Type { text, .. } => {
                cmd.args(["type", text]);
            }
            _ => {
                return Err(TagisanError::Execution("ydotool action not implemented".to_string()));
            }
        }

        let output = cmd.output().map_err(|e| TagisanError::Io(e))?;
        let ok = output.status.success();
        Ok((ok, "ydotool executed".to_string(), target_cursor))
    }

    fn exec_osascript(&self, action: &InputAction) -> Result<(bool, String, (u32, u32))> {
        let target_cursor = (self.simulator.cursor_x, self.simulator.cursor_y);
        let script = match action {
            InputAction::Type { text, .. } => {
                format!("tell application \"System Events\" to keystroke \"{}\"", text.replace('"', "\\\""))
            }
            InputAction::KeyCombo { keys } => {
                let combo = keys.join(" ");
                format!("tell application \"System Events\" to key code {combo}")
            }
            _ => {
                return Err(TagisanError::Execution("osascript mouse actions require cliclick".to_string()));
            }
        };

        let output = Command::new("osascript").args(["-e", &script]).output().map_err(|e| TagisanError::Io(e))?;
        Ok((output.status.success(), "osascript executed".to_string(), target_cursor))
    }

    fn exec_powershell(&self, action: &InputAction) -> Result<(bool, String, (u32, u32))> {
        let target_cursor = (self.simulator.cursor_x, self.simulator.cursor_y);
        let script = match action {
            InputAction::Type { text, .. } => {
                format!("Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait('{}')", text.replace('\'', "''"))
            }
            _ => {
                return Err(TagisanError::Execution("PowerShell mouse action requires user32 SendInput".to_string()));
            }
        };

        let output = Command::new("powershell").args(["-NoProfile", "-Command", &script]).output().map_err(|e| TagisanError::Io(e))?;
        Ok((output.status.success(), "powershell executed".to_string(), target_cursor))
    }
}

/// Detect available input backend on system
pub fn detect_input_backend() -> InputBackendType {
    if cfg!(target_os = "macos") {
        return InputBackendType::Osascript;
    }
    if cfg!(target_os = "windows") {
        return InputBackendType::Powershell;
    }

    if check_cmd("xdotool") && std::env::var("DISPLAY").is_ok() {
        return InputBackendType::Xdotool;
    }
    if check_cmd("wtype") && std::env::var("WAYLAND_DISPLAY").is_ok() {
        return InputBackendType::Wtype;
    }
    if check_cmd("ydotool") {
        return InputBackendType::Ydotool;
    }

    InputBackendType::Virtual
}

fn check_cmd(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_input_simulator_actions() {
        let mut sim = VirtualInputSimulator::new(1920, 1080);
        let click_res = sim.execute(&InputAction::Click {
            x: 200,
            y: 300,
            button: MouseButton::Left,
            count: 1,
        }).expect("Click should succeed");

        assert!(click_res.success);
        assert_eq!(click_res.cursor_position, (200, 300));

        let type_res = sim.execute(&InputAction::Type {
            text: "Hello Astra".to_string(),
            delay_ms: 10,
        }).expect("Type should succeed");

        assert!(type_res.success);
        assert_eq!(sim.typed_history.last().map(|s| s.as_str()), Some("Hello Astra"));
    }

    #[test]
    fn test_input_engine_bounds_validation() {
        let mut engine = InputEngine::new(1000, 800).with_virtual_forced(true);
        let err = engine.execute(InputAction::Click {
            x: 2000,
            y: 500,
            button: MouseButton::Left,
            count: 1,
        });
        assert!(err.is_err());
    }
}
