//! # Vella EventBus to BunStreamBus uWebSockets Telemetry Bridge
//!
//! Bridges Vella's internal broadcast EventBus to Tagisan's Bun uWebSockets
//! stream bus for ultra-high throughput telemetry and multi-topic pub/sub.

use crate::error::Result;
use crate::tools::bun_serve::BunStreamBusTool;
use crate::tools::ToolHandler;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[cfg(feature = "vella")]
use vella::core::events::EventBus;

/// Bridges Vella System Events to a high-speed uWebSockets stream bus
pub struct VellaStreamBridge {
    #[cfg(feature = "vella")]
    pub event_bus: Arc<EventBus>,
    pub stream_bus: Arc<BunStreamBusTool>,
    pub is_running: Arc<AtomicBool>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub topic: String,
    pub bus_port: Option<u16>,
}

impl VellaStreamBridge {
    #[cfg(feature = "vella")]
    pub fn new(event_bus: Arc<EventBus>, stream_bus: Arc<BunStreamBusTool>) -> Self {
        Self {
            event_bus,
            stream_bus,
            is_running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
            topic: "vella_events".to_string(),
            bus_port: None,
        }
    }

    #[cfg(not(feature = "vella"))]
    pub fn new(stream_bus: Arc<BunStreamBusTool>) -> Self {
        Self {
            stream_bus,
            is_running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
            topic: "vella_events".to_string(),
            bus_port: None,
        }
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = topic.into();
        self
    }

    pub fn with_bus_port(mut self, port: u16) -> Self {
        self.bus_port = Some(port);
        self
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Start bridging events from Vella EventBus to BunStreamBus
    pub async fn start(&self) -> Result<()> {
        if self.is_running.swap(true, Ordering::SeqCst) {
            warn!("VellaStreamBridge is already running.");
            return Ok(());
        }

        info!("Starting VellaStreamBridge forwarding to topic '{}'", self.topic);

        #[cfg(feature = "vella")]
        {
            let mut rx = self.event_bus.subscribe();
            let stream_bus = self.stream_bus.clone();
            let is_running = self.is_running.clone();
            let topic = self.topic.clone();
            let bus_port = self.bus_port;

            let handle = tokio::spawn(async move {
                while is_running.load(Ordering::SeqCst) {
                    match rx.recv().await {
                        Ok(event) => {
                            let payload = match serde_json::to_value(&event) {
                                Ok(val) => val,
                                Err(e) => {
                                    error!("Failed to serialize Vella SystemEvent: {}", e);
                                    continue;
                                }
                            };

                            let mut publish_args = json!({
                                "action": "publish",
                                "topic": topic,
                                "message": payload
                            });

                            if let Some(port) = bus_port {
                                publish_args["port"] = json!(port);
                            }

                            if let Err(e) = stream_bus.execute(publish_args).await {
                                // Bus may not be active or ready yet, log at debug/trace
                                tracing::debug!("StreamBridge publish warning: {}", e);
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("VellaStreamBridge lagged, skipped {} events", skipped);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!("Vella EventBus channel closed, stopping bridge task.");
                            break;
                        }
                    }
                }
                is_running.store(false, Ordering::SeqCst);
            });

            let mut guard = self.task_handle.lock().await;
            *guard = Some(handle);
        }

        Ok(())
    }

    /// Stop the stream bridge
    pub async fn stop(&self) -> Result<()> {
        self.is_running.store(false, Ordering::SeqCst);
        let mut guard = self.task_handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
        info!("VellaStreamBridge stopped.");
        Ok(())
    }
}
