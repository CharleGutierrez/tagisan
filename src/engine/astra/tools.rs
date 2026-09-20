//! ToolHandler implementations for Astra screen capture and computer use.

use crate::engine::astra::agent::AstraSecurityGuard;
use crate::engine::astra::input::{InputAction, InputEngine, MouseButton};
use crate::engine::astra::screen::ScreenCaptureEngine;
use crate::engine::astra::visual_memory::VisualMemory;
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tool for capturing screenshots with resilient headless virtual fallback
#[derive(Clone)]
pub struct AstraScreenCaptureTool {
    engine: Arc<Mutex<ScreenCaptureEngine>>,
}

impl Default for AstraScreenCaptureTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AstraScreenCaptureTool {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(ScreenCaptureEngine::new())),
        }
    }
}

#[async_trait]
impl ToolHandler for AstraScreenCaptureTool {
    fn name(&self) -> &str {
        "astra_screen_capture"
    }

    fn description(&self) -> &str {
        "Captures a screenshot of the computer screen or a specific subregion using X11/Wayland with resilient HeadlessVirtualFramebuffer fallback. Returns image dimensions, display server, and base64 PNG data."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "region": {
                    "type": "object",
                    "description": "Optional rectangular subregion to crop (x, y, width, height)",
                    "properties": {
                        "x": { "type": "integer" },
                        "y": { "type": "integer" },
                        "width": { "type": "integer" },
                        "height": { "type": "integer" }
                    },
                    "required": ["x", "y", "width", "height"]
                },
                "force_virtual": {
                    "type": "boolean",
                    "description": "Force virtual framebuffer capture even if a physical display server is active"
                },
                "include_base64": {
                    "type": "boolean",
                    "description": "Whether to include full base64 PNG data in the response (default: true)"
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let mut engine = self.engine.lock().await;

        let force_virtual = arguments["force_virtual"].as_bool().unwrap_or(false);
        if force_virtual {
            engine.force_virtual = true;
        }

        let include_base64 = arguments["include_base64"].as_bool().unwrap_or(true);

        let frame = if let Some(region) = arguments.get("region").and_then(|r| r.as_object()) {
            let x = region.get("x").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let y = region.get("y").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let w = region.get("width").and_then(|v| v.as_u64()).unwrap_or(100) as u32;
            let h = region.get("height").and_then(|v| v.as_u64()).unwrap_or(100) as u32;
            engine.capture_region(x, y, w, h)?
        } else {
            engine.capture()?
        };

        let base64_png = if include_base64 {
            Some(frame.to_base64_png())
        } else {
            None
        };

        let response = json!({
            "success": true,
            "frame_id": frame.id,
            "width": frame.width,
            "height": frame.height,
            "display_type": frame.display_type.to_string(),
            "is_virtual": frame.is_virtual,
            "timestamp_ms": frame.timestamp_ms,
            "base64_png": base64_png,
        });

        Ok(serde_json::to_string_pretty(&response)?)
    }
}

/// Tool for synthesizing mouse and keyboard input with AgentShield guardrails and visual verification
#[derive(Clone)]
pub struct AstraComputerUseTool {
    input_engine: Arc<Mutex<InputEngine>>,
    screen_engine: Arc<Mutex<ScreenCaptureEngine>>,
    memory: Arc<Mutex<VisualMemory>>,
    security_guard: AstraSecurityGuard,
}

impl Default for AstraComputerUseTool {
    fn default() -> Self {
        Self::new()
    }
}

impl AstraComputerUseTool {
    pub fn new() -> Self {
        Self {
            input_engine: Arc::new(Mutex::new(InputEngine::new(1920, 1080))),
            screen_engine: Arc::new(Mutex::new(ScreenCaptureEngine::new())),
            memory: Arc::new(Mutex::new(VisualMemory::new(20))),
            security_guard: AstraSecurityGuard::default(),
        }
    }
}

#[async_trait]
impl ToolHandler for AstraComputerUseTool {
    fn name(&self) -> &str {
        "astra_computer_use"
    }

    fn description(&self) -> &str {
        "Synthesizes mouse and keyboard actions (click, double click, drag, scroll, type, key combos) with visual verification, safety boundaries, and AgentShield guardrails."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action"],
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["click", "double_click", "mouse_move", "mouse_down", "mouse_up", "drag", "scroll", "type", "key_press", "key_combo", "wait", "screenshot", "diff"],
                    "description": "Action to perform on the operating system"
                },
                "x": { "type": "integer", "description": "Target X coordinate (pixels)" },
                "y": { "type": "integer", "description": "Target Y coordinate (pixels)" },
                "button": { "type": "string", "enum": ["left", "right", "middle"], "description": "Mouse button for click/drag (default: left)" },
                "to_x": { "type": "integer", "description": "Destination X coordinate for drag" },
                "to_y": { "type": "integer", "description": "Destination Y coordinate for drag" },
                "delta_x": { "type": "integer", "description": "Horizontal scroll delta" },
                "delta_y": { "type": "integer", "description": "Vertical scroll delta (positive=down, negative=up)" },
                "text": { "type": "string", "description": "Text to type" },
                "key": { "type": "string", "description": "Single key name to press" },
                "keys": { "type": "array", "items": { "type": "string" }, "description": "Key sequence or combination to press" },
                "duration_ms": { "type": "integer", "description": "Wait duration in milliseconds" },
                "force_virtual": { "type": "boolean", "description": "Force execution in virtual simulator without OS dispatch" }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action_str = arguments["action"].as_str().ok_or_else(|| {
            TagisanError::Execution("Missing required parameter: 'action'".to_string())
        })?;

        let button = match arguments["button"].as_str().unwrap_or("left") {
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => MouseButton::Left,
        };

        let x = arguments["x"].as_u64().unwrap_or(0) as u32;
        let y = arguments["y"].as_u64().unwrap_or(0) as u32;
        let force_virtual = arguments["force_virtual"].as_bool().unwrap_or(false);

        // Map into structured InputAction
        let input_action = match action_str {
            "click" => InputAction::Click { x, y, button, count: 1 },
            "double_click" => InputAction::Click { x, y, button, count: 2 },
            "mouse_move" => InputAction::MouseMove { x, y },
            "mouse_down" => InputAction::MouseDown { x, y, button },
            "mouse_up" => InputAction::MouseUp { x, y, button },
            "drag" => {
                let to_x = arguments["to_x"].as_u64().unwrap_or(x as u64) as u32;
                let to_y = arguments["to_y"].as_u64().unwrap_or(y as u64) as u32;
                InputAction::Drag { from_x: x, from_y: y, to_x, to_y, button }
            }
            "scroll" => {
                let delta_x = arguments["delta_x"].as_i64().unwrap_or(0) as i32;
                let delta_y = arguments["delta_y"].as_i64().unwrap_or(5) as i32;
                InputAction::Scroll { x, y, delta_x, delta_y }
            }
            "type" => {
                let text = arguments["text"].as_str().unwrap_or("").to_string();
                InputAction::Type { text, delay_ms: 10 }
            }
            "key_press" => {
                let key = arguments["key"].as_str().unwrap_or("Return").to_string();
                InputAction::KeyPress { key, modifiers: Vec::new() }
            }
            "key_combo" => {
                let keys: Vec<String> = arguments["keys"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                InputAction::KeyCombo { keys }
            }
            "wait" => {
                let duration_ms = arguments["duration_ms"].as_u64().unwrap_or(100);
                InputAction::Wait { duration_ms }
            }
            "screenshot" | "diff" => {
                // Read-only inspection action
                let mut screen_eng = self.screen_engine.lock().await;
                if force_virtual {
                    screen_eng.force_virtual = true;
                }
                let frame = screen_eng.capture()?;
                let mut mem = self.memory.lock().await;
                let diff = mem.record_frame(frame.clone(), Some(action_str.to_string()));

                return Ok(serde_json::to_string_pretty(&json!({
                    "success": true,
                    "action": action_str,
                    "frame_id": frame.id,
                    "dimensions": [frame.width, frame.height],
                    "is_virtual": frame.is_virtual,
                    "diff_percentage": diff.diff_percentage,
                    "changed_cells": diff.changed_cells.len(),
                    "bounding_box": diff.bounding_box,
                }))?);
            }
            other => {
                return Err(TagisanError::Execution(format!("Unknown action: '{other}'")));
            }
        };

        // 1. AgentShield Security Audit
        let audit = self.security_guard.audit_action(&input_action);
        if !audit.allowed {
            let reason = audit.reason.unwrap_or_else(|| "Security violation".to_string());
            return Ok(serde_json::to_string_pretty(&json!({
                "success": false,
                "action": action_str,
                "blocked": true,
                "reason": format!("AgentShield Security Gate Blocked: {reason}"),
                "threat_level": audit.threat_level,
            }))?);
        }

        // 2. Dispatch to input engine
        let mut input_eng = self.input_engine.lock().await;
        if force_virtual {
            input_eng.force_virtual = true;
        }

        let exec_res = input_eng.execute(input_action)?;

        // 3. Update virtual framebuffer if applicable
        let mut screen_eng = self.screen_engine.lock().await;
        if force_virtual || screen_eng.force_virtual {
            match action_str {
                "click" | "double_click" => {
                    screen_eng.virtual_framebuffer.click(x, y, "left");
                }
                "type" => {
                    if let Some(txt) = arguments["text"].as_str() {
                        screen_eng.virtual_framebuffer.type_text(txt);
                    }
                }
                "mouse_move" => {
                    screen_eng.virtual_framebuffer.set_cursor(x, y);
                }
                _ => {}
            }
        }

        // 4. Capture post-action frame and record visual diff
        let post_frame = screen_eng.capture()?;
        let mut mem = self.memory.lock().await;
        let diff = mem.record_frame(post_frame.clone(), Some(action_str.to_string()));

        let output = json!({
            "success": exec_res.success,
            "action": action_str,
            "details": exec_res.details,
            "cursor_position": exec_res.cursor_position,
            "backend_used": exec_res.backend_used.to_string(),
            "execution_time_ms": exec_res.execution_time_ms,
            "visual_diff": {
                "diff_percentage": diff.diff_percentage,
                "changed_cells": diff.changed_cells.len(),
                "bounding_box": diff.bounding_box,
                "is_significant": diff.is_significant,
            }
        });

        Ok(serde_json::to_string_pretty(&output)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_astra_screen_capture_tool() {
        let tool = AstraScreenCaptureTool::new();
        let res = tool.execute(json!({
            "force_virtual": true,
            "include_base64": false
        })).await.expect("Tool execution should succeed");

        let val: Value = serde_json::from_str(&res).unwrap();
        assert_eq!(val["success"], true);
        assert_eq!(val["is_virtual"], true);
    }

    #[tokio::test]
    async fn test_astra_computer_use_tool_click_and_type() {
        let tool = AstraComputerUseTool::new();

        let click_res = tool.execute(json!({
            "action": "click",
            "x": 150,
            "y": 150,
            "force_virtual": true
        })).await.expect("Click should succeed");
        let click_val: Value = serde_json::from_str(&click_res).unwrap();
        assert_eq!(click_val["success"], true);

        let type_res = tool.execute(json!({
            "action": "type",
            "text": "ls -la",
            "force_virtual": true
        })).await.expect("Type should succeed");
        let type_val: Value = serde_json::from_str(&type_res).unwrap();
        assert_eq!(type_val["success"], true);
    }

    #[tokio::test]
    async fn test_astra_computer_use_tool_security_block() {
        let tool = AstraComputerUseTool::new();
        let res = tool.execute(json!({
            "action": "type",
            "text": "rm -rf / --no-preserve-root",
            "force_virtual": true
        })).await.expect("Tool should return security block result");

        let val: Value = serde_json::from_str(&res).unwrap();
        assert_eq!(val["success"], false);
        assert_eq!(val["blocked"], true);
    }
}
