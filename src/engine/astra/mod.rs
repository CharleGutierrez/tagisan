//! Astra Multimodal Computer-Use Engine for Tagisan (TGS)
//!
//! Production-grade autonomous computer-use agent, screen perception & frame capture
//! with X11/Wayland detection, resilient HeadlessVirtualFramebuffer fallback,
//! OS input synthesis (click, double click, drag, scroll, type, key combos) with action verification,
//! visual memory buffer & perceptual grid hashing, goal-driven visual reasoning loop with Vision LLM integration,
//! strict AgentShield security guardrails, and real-time bi-directional streaming sessions.

pub mod screen;
pub mod input;
pub mod visual_memory;
pub mod agent;
pub mod session;
pub mod tools;

pub use screen::{
    detect_display_server, encode_rgba_to_png, get_available_capture_tools, parse_png_dimensions,
    DisplayInfo, DisplayServerType, FrameFormat, HeadlessVirtualFramebuffer, ScreenCaptureEngine,
    ScreenFrame, VirtualWindow,
};

pub use input::{
    detect_input_backend, InputActionResult, InputAction, InputBackendType, InputEngine,
    KeyModifier, MouseButton, VirtualInputSimulator,
};

pub use visual_memory::{
    BoundingBox, PerceptualGridHash, UiState, UiTransition, VisualDiffResult, VisualMemory,
    VisualMemoryStats,
};

pub use agent::{
    AgentActionProposal, AgentExecutionStatus, AstraAgentResult, AstraAgentStep,
    AstraSecurityGuard, AstraVisualAgent, AstraVisualAgentConfig, SecurityAuditVerdict,
};

pub use session::{
    AstraEvent, AstraEventBus, AstraFrameQueue, AstraSession, SessionState,
};

pub use tools::{
    AstraComputerUseTool, AstraScreenCaptureTool,
};
