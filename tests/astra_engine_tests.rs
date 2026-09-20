//! Production-Grade 1000% Reliable Integration Tests for GPT Astra Multimodal Computer-Use Engine

use tagisan::engine::astra::*;
use tagisan::tools::ToolHandler;
use tagisan::swarm::repl::{InteractiveRepl, ReplCommand};
use tagisan::engine::EngineContext;
use tagisan::agent::AutonomousAgent;
use serde_json::json;

#[test]
fn test_display_detection_and_virtual_fallback() {
    let mut engine = ScreenCaptureEngine::new().with_virtual_forced(true);
    let info = engine.display_info();

    assert!(info.is_virtual);
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
    assert!(!info.available_tools.is_empty());

    let frame = engine.capture().expect("Virtual screen capture must succeed");
    assert_eq!(frame.width, 1920);
    assert_eq!(frame.height, 1080);
    assert!(frame.is_virtual);
    assert_eq!(frame.format, FrameFormat::Rgba8);

    let png_bytes = frame.to_png_bytes();
    assert!(!png_bytes.is_empty());
    assert_eq!(&png_bytes[0..8], b"\x89PNG\r\n\x1a\n");

    let dims = parse_png_dimensions(&png_bytes);
    assert_eq!(dims, Some((1920, 1080)));

    let b64 = frame.to_base64_png();
    assert!(!b64.is_empty());
}

#[test]
fn test_headless_virtual_framebuffer_ui_simulation() {
    let mut fb = HeadlessVirtualFramebuffer::new(1280, 720);
    assert_eq!(fb.windows.len(), 2);

    // Verify initial desktop render
    let frame1 = fb.render_frame();
    assert_eq!(frame1.dimensions(), (1280, 720));

    // Focus and click inside Window 1
    fb.click(200, 200, "left");
    assert_eq!(fb.active_window_idx, Some(0));

    // Type command into Window 1
    fb.type_text("echo 'Astra 1000x Reliable'");
    let frame2 = fb.render_frame();
    assert_ne!(frame1.id, frame2.id);

    // Verify visual diff between untouched desktop and typed desktop
    let diff = VisualDiffResult::compute(&frame1, &frame2);
    assert!(diff.diff_percentage > 0.0);
}

#[test]
fn test_input_engine_synthesis_and_bounds_verification() {
    let mut engine = InputEngine::new(1920, 1080).with_virtual_forced(true);

    // Valid Click
    let click_res = engine.click(500, 400, MouseButton::Left).expect("Click should succeed");
    assert!(click_res.success);
    assert_eq!(click_res.cursor_position, (500, 400));
    assert!(click_res.verified);

    // Valid Double Click
    let dclick_res = engine.double_click(600, 450).expect("Double click should succeed");
    assert!(dclick_res.success);
    assert_eq!(dclick_res.cursor_position, (600, 450));

    // Valid Drag
    let drag_res = engine.drag(100, 100, 300, 300).expect("Drag should succeed");
    assert!(drag_res.success);
    assert_eq!(drag_res.cursor_position, (300, 300));

    // Valid Type
    let type_res = engine.type_text("cargo test -j 1", 5).expect("Type should succeed");
    assert!(type_res.success);

    // Valid Key Combo
    let combo_res = engine.key_combo(&["Control", "Alt", "T"]).expect("Combo should succeed");
    assert!(combo_res.success);

    // Invalid Out-of-Bounds Coordinates
    let oob_err = engine.click(3000, 2000, MouseButton::Left);
    assert!(oob_err.is_err(), "Coordinates exceeding screen bounds must be rejected");
}

#[test]
fn test_perceptual_grid_hash_and_visual_change_detection() {
    let mut pixels_a = vec![40u8; 256 * 256 * 4];
    let frame_a = ScreenFrame::new_rgba(256, 256, pixels_a.clone(), DisplayServerType::Headless, true);

    let mut pixels_b = pixels_a.clone();
    // Modify lower right quadrant
    for y in 128..256 {
        for x in 128..256 {
            let idx = ((y * 256 + x) * 4) as usize;
            pixels_b[idx] = 220;
            pixels_b[idx + 1] = 220;
            pixels_b[idx + 2] = 220;
        }
    }
    let frame_b = ScreenFrame::new_rgba(256, 256, pixels_b, DisplayServerType::Headless, true);

    let hash_a = PerceptualGridHash::from_frame(&frame_a);
    let hash_b = PerceptualGridHash::from_frame(&frame_b);

    assert!(hash_a.hamming_distance(&hash_b) > 0);
    assert!(hash_a.difference_score(&hash_b) > 0.05);

    let diff = VisualDiffResult::compute(&frame_a, &frame_b);
    assert!(diff.is_significant);
    assert!(diff.bounding_box.is_some());

    let bbox = diff.bounding_box.unwrap();
    assert!(bbox.x >= 120);
    assert!(bbox.y >= 120);
}

#[test]
fn test_visual_memory_lifecycle_and_stuck_detection() {
    let mut mem = VisualMemory::new(15);

    let f1 = ScreenFrame::new_rgba(64, 64, vec![50u8; 64 * 64 * 4], DisplayServerType::Headless, true);
    mem.record_frame(f1, None);

    // Simulate 3 consecutive identical frames (stuck in loop)
    for _ in 0..3 {
        let fn_frame = ScreenFrame::new_rgba(64, 64, vec![50u8; 64 * 64 * 4], DisplayServerType::Headless, true);
        mem.record_frame(fn_frame, Some("click(10, 10)".to_string()));
    }

    assert!(mem.detect_stuck_state(3), "Agent should detect stuck UI state when no change occurs");
}

#[test]
fn test_agentshield_security_guardrails_in_astra() {
    let guard = AstraSecurityGuard::default();

    // Safe actions must pass
    let safe1 = InputAction::Type { text: "git status".to_string(), delay_ms: 0 };
    assert!(guard.audit_action(&safe1).allowed);

    let safe2 = InputAction::Click { x: 500, y: 500, button: MouseButton::Left, count: 1 };
    assert!(guard.audit_action(&safe2).allowed);

    // Destructive shell payloads must be blocked
    let dangerous1 = InputAction::Type { text: "rm -rf / --no-preserve-root".to_string(), delay_ms: 0 };
    let verdict1 = guard.audit_action(&dangerous1);
    assert!(!verdict1.allowed);
    assert!(verdict1.reason.unwrap().contains("Destructive"));

    let dangerous2 = InputAction::Type { text: "DROP DATABASE production;".to_string(), delay_ms: 0 };
    let verdict2 = guard.audit_action(&dangerous2);
    assert!(!verdict2.allowed);

    // Forbidden bounding box zone blocking
    let mut strict_guard = AstraSecurityGuard::default();
    strict_guard.forbidden_regions.push(BoundingBox::new(0, 0, 100, 100));

    let forbidden_click = InputAction::Click { x: 50, y: 50, button: MouseButton::Left, count: 1 };
    let verdict3 = strict_guard.audit_action(&forbidden_click);
    assert!(!verdict3.allowed);
    assert!(verdict3.reason.unwrap().contains("forbidden security zone"));
}

#[tokio::test]
async fn test_astra_visual_agent_autonomous_execution() {
    let config = AstraVisualAgentConfig {
        goal: "Inspect current workspace status".to_string(),
        max_steps: 4,
        provider: None,
        headless: true,
        step_delay_ms: 0,
        ..Default::default()
    };

    let mut agent = AstraVisualAgent::new(config);
    let res = agent.run().await.expect("Autonomous agent execution must succeed");

    assert!(res.success);
    assert_eq!(res.status, AgentExecutionStatus::Completed);
    assert!(!res.step_trace.is_empty());
    assert!(res.final_frame_id > 0);
}

#[tokio::test]
async fn test_astra_streaming_session_and_events() {
    let config = AstraVisualAgentConfig {
        goal: "Inspect screen in stream".to_string(),
        headless: true,
        max_steps: 3,
        step_delay_ms: 0,
        ..Default::default()
    };

    let mut session = AstraSession::new("test-stream-1", config);
    let mut rx = session.subscribe();

    // Verify initial state
    assert_eq!(session.state, SessionState::Idle);

    // Run streaming session
    let res = session.run().await.expect("Streaming session must succeed");
    assert!(res.success);
    assert_eq!(session.state, SessionState::Completed);

    // Check events received
    let mut event_count = 0;
    while let Ok(_event) = rx.try_recv() {
        event_count += 1;
    }
    assert!(event_count > 0, "Session event bus must publish execution events");
}

#[tokio::test]
async fn test_astra_tools_execution_via_tool_handler() {
    // 1. AstraScreenCaptureTool
    let capture_tool = AstraScreenCaptureTool::new();
    assert_eq!(capture_tool.name(), "astra_screen_capture");

    let cap_res = capture_tool.execute(json!({
        "force_virtual": true,
        "include_base64": false
    })).await.expect("Screen capture tool should execute");

    let cap_val: serde_json::Value = serde_json::from_str(&cap_res).unwrap();
    assert_eq!(cap_val["success"], true);
    assert_eq!(cap_val["is_virtual"], true);
    assert_eq!(cap_val["width"], 1920);
    assert_eq!(cap_val["height"], 1080);

    // 2. AstraComputerUseTool
    let cu_tool = AstraComputerUseTool::new();
    assert_eq!(cu_tool.name(), "astra_computer_use");

    let click_res = cu_tool.execute(json!({
        "action": "click",
        "x": 250,
        "y": 300,
        "force_virtual": true
    })).await.expect("Computer use click should execute");

    let click_val: serde_json::Value = serde_json::from_str(&click_res).unwrap();
    assert_eq!(click_val["success"], true);
    assert_eq!(click_val["action"], "click");

    let type_res = cu_tool.execute(json!({
        "action": "type",
        "text": "tgs astra status",
        "force_virtual": true
    })).await.expect("Computer use type should execute");

    let type_val: serde_json::Value = serde_json::from_str(&type_res).unwrap();
    assert_eq!(type_val["success"], true);
    assert_eq!(type_val["action"], "type");

    // Test AgentShield block via tool
    let block_res = cu_tool.execute(json!({
        "action": "type",
        "text": "rm -rf /",
        "force_virtual": true
    })).await.expect("Tool should return security block result");

    let block_val: serde_json::Value = serde_json::from_str(&block_res).unwrap();
    assert_eq!(block_val["success"], false);
    assert_eq!(block_val["blocked"], true);
}

#[tokio::test]
async fn test_astra_repl_slash_command() {
    let cmd = InteractiveRepl::parse_command("/astra status");
    assert_eq!(cmd, ReplCommand::Astra("status".to_string()));

    let cmd_click = InteractiveRepl::parse_command("/astra click 100 200");
    assert_eq!(cmd_click, ReplCommand::Astra("click 100 200".to_string()));

    let cmd_cu = InteractiveRepl::parse_command("/cu run inspect desktop");
    assert_eq!(cmd_cu, ReplCommand::Astra("run inspect desktop".to_string()));

    // Verify REPL execution of /astra status
    let provider = std::sync::Arc::new(tagisan::providers::ollama::OllamaProvider::default_local());
    let tools = tagisan::tools::ToolRegistry::new();
    let agent = AutonomousAgent::new(provider, "test-model", tools);
    let ctx = EngineContext::new(100.0);
    let mut repl = InteractiveRepl::new(agent, "test-session", "test-model", ctx);

    let status_out = repl.execute_command(ReplCommand::Astra("status".to_string())).await
        .expect("REPL /astra status should succeed");
    assert!(status_out.is_some());
    let text = status_out.unwrap();
    assert!(text.contains("GPT ASTRA MULTIMODAL COMPUTER-USE ENGINE STATUS"));
}
