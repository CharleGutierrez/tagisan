//! Interactive Swarm & Computer-Use Web Dashboard for Tagisan (TGS).
//!
//! Provides an embedded lightweight HTTP/WebSocket server running on port 7420,
//! real-time Astra computer-use screen capture streaming, swarm task DAG visualization,
//! host memory telemetry, and Reflexion case law exploration.

pub mod state;
pub mod ws;
pub mod server;

pub use state::{
    DashboardState, HostTelemetrySnapshot, SharedDashboardState, SwarmNodeStatus,
};
pub use ws::{
    compute_accept_key, encode_text_frame, sha1_digest, WebSocketBroadcaster,
};
pub use server::{
    SwarmDashboardServer,
};
