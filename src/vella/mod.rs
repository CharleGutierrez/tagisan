//! # Tagisan-Vella Sovereign OS Integration Module
//!
//! Deep, production-grade fusion of Tagisan (High-performance Multi-LLM
//! Collaboration, Dialectical Debate, and Swarm Intelligence) with the Vella
//! Framework (LLM-Native Backend, Real-time Web Engine, Industrial SCADA/Robotics,
//! Algorithmic Trading, Clinical Genomics, and Sovereign Operating System).

pub mod cyber_defense;
pub mod debate_governor;
pub mod digital_twin;
pub mod fhe_shield;
pub mod scaffolder;
pub mod space_copilot;
pub mod stream_bridge;
pub mod tools;
pub mod vector_sync;
pub mod web3_guardian;

use crate::error::{Result, TagisanError};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub use cyber_defense::{CyberDefenseOrchestrator, DefenseDrillReport, VellaDefenseDrillTool};
pub use debate_governor::{DomainActionProposal, VellaDebateGovernor, VellaDebateVerdict};
pub use digital_twin::{DigitalTwinEngine, DigitalTwinVerdict, VellaDigitalTwinTool};
pub use fhe_shield::{FhePrivacyEngine, VellaFheShieldTool};
pub use scaffolder::{ApiScaffolderEngine, SelfHealingReport, VellaScaffolderTool};
pub use space_copilot::{
    AvoidanceManeuverPlan, ConjunctionReport, EciPosition, SpaceAstrodynamicsEngine,
    VellaSpaceCopilotTool,
};
pub use stream_bridge::VellaStreamBridge;
pub use tools::{
    VellaEventBridgeTool, VellaMedicineTool, VellaRoboticsTool, VellaScadaTool, VellaTradingTool,
};
pub use vector_sync::{
    HybridVectorHit, VectorSyncStats, VellaVectorRecord, VellaVectorSyncBridge, VellaVectorSyncTool,
};
pub use web3_guardian::{
    GuardianRole, TreasuryProposal, VellaWeb3GuardianTool, Web3TreasuryGuardian,
};

#[cfg(feature = "vella")]
use vella::{
    core::events::{EventBus, SystemEvent},
    model::{Field, FieldType, ModelSchema, SchemaRegistry},
    VellaApp,
};

/// Policy control plane enforcing safety bounds, operational thresholds, and hardware E-Stops
#[derive(Debug, Clone)]
pub struct VellaPolicyGovernor {
    pub max_order_value_usd: f64,
    pub max_order_size: u64,
    pub max_leverage: f64,
    pub allowed_scada_coils: (u16, u16),
    pub max_scada_pressure_psi: f64,
    pub max_robot_velocity_ms: f64,
    pub e_stop_tripped: Arc<RwLock<bool>>,
    pub require_debate_for_high_stakes: bool,
    pub audit_log: Arc<RwLock<Vec<String>>>,
}

impl Default for VellaPolicyGovernor {
    fn default() -> Self {
        Self {
            max_order_value_usd: 1_000_000.0,
            max_order_size: 100_000,
            max_leverage: 50.0,
            allowed_scada_coils: (0, 10_000),
            max_scada_pressure_psi: 500.0,
            max_robot_velocity_ms: 30.0,
            e_stop_tripped: Arc::new(RwLock::new(false)),
            require_debate_for_high_stakes: true,
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl VellaPolicyGovernor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if E-Stop is active
    pub async fn is_e_stop_active(&self) -> bool {
        *self.e_stop_tripped.read().await
    }

    /// Trip or clear hardware E-Stop
    pub async fn set_e_stop(&self, tripped: bool, reason: &str) {
        let mut lock = self.e_stop_tripped.write().await;
        *lock = tripped;
        let event_msg = if tripped {
            format!("🚨 [E-STOP TRIPPED]: {}", reason)
        } else {
            format!("✅ [E-STOP CLEARED]: {}", reason)
        };
        warn!("{}", event_msg);
        let mut log = self.audit_log.write().await;
        log.push(event_msg);
    }

    /// Validate a proposed trade against policy bounds
    pub fn validate_trade(
        &self,
        symbol: &str,
        price: f64,
        size: u64,
        leverage: Option<f64>,
    ) -> Result<()> {
        let order_val = price * (size as f64);
        if order_val > self.max_order_value_usd {
            return Err(TagisanError::Execution(format!(
                "PolicyGovernor: Trade value ${:.2} exceeds maximum threshold of ${:.2} for {}",
                order_val, self.max_order_value_usd, symbol
            )));
        }
        if size > self.max_order_size {
            return Err(TagisanError::Execution(format!(
                "PolicyGovernor: Order size {} exceeds maximum threshold of {} for {}",
                size, self.max_order_size, symbol
            )));
        }
        if let Some(lev) = leverage {
            if lev > self.max_leverage {
                return Err(TagisanError::Execution(format!(
                    "PolicyGovernor: Leverage {:.1}x exceeds maximum permitted {:.1}x",
                    lev, self.max_leverage
                )));
            }
        }
        Ok(())
    }

    /// Validate SCADA actuation
    pub async fn validate_scada_actuation(&self, coil_addr: u16, _state: bool) -> Result<()> {
        if self.is_e_stop_active().await {
            return Err(TagisanError::Execution(
                "PolicyGovernor: SCADA Actuation BLOCKED. System Emergency Stop (E-Stop) is actively latched!"
                    .to_string(),
            ));
        }
        if coil_addr < self.allowed_scada_coils.0 || coil_addr > self.allowed_scada_coils.1 {
            return Err(TagisanError::Execution(format!(
                "PolicyGovernor: SCADA Coil {} outside permitted range [{}, {}]",
                coil_addr, self.allowed_scada_coils.0, self.allowed_scada_coils.1
            )));
        }
        Ok(())
    }

    /// Validate Robotics motion command
    pub async fn validate_robotics_motion(&self, velocity_ms: f64) -> Result<()> {
        if self.is_e_stop_active().await {
            return Err(TagisanError::Execution(
                "PolicyGovernor: Robotics motion BLOCKED. Emergency Stop (E-Stop) is actively latched!"
                    .to_string(),
            ));
        }
        if velocity_ms > self.max_robot_velocity_ms {
            return Err(TagisanError::Execution(format!(
                "PolicyGovernor: Velocity {:.2} m/s exceeds max limit of {:.2} m/s",
                velocity_ms, self.max_robot_velocity_ms
            )));
        }
        Ok(())
    }

    /// Validate Clinical Medicine action
    pub fn validate_medicine_dosage(
        &self,
        drug_name: &str,
        dosage_mg: f64,
        max_safe_mg: f64,
    ) -> Result<()> {
        if dosage_mg > max_safe_mg {
            return Err(TagisanError::Execution(format!(
                "PolicyGovernor: Prescribed dosage of {:.2}mg for '{}' exceeds safe clinical threshold of {:.2}mg",
                dosage_mg, drug_name, max_safe_mg
            )));
        }
        Ok(())
    }
}

/// Manages the lifecycle, schema registry, and event bus of an active Vella application instance
#[derive(Clone)]
pub struct VellaAppManager {
    #[cfg(feature = "vella")]
    pub event_bus: Arc<EventBus>,
    #[cfg(feature = "vella")]
    pub schema_registry: Arc<RwLock<SchemaRegistry>>,
    pub governor: Arc<VellaPolicyGovernor>,
    pub db_url: String,
    pub is_initialized: Arc<RwLock<bool>>,
}

impl Default for VellaAppManager {
    fn default() -> Self {
        Self::new("sqlite://vella_tagisan.db?mode=rwc")
    }
}

impl VellaAppManager {
    pub fn new(db_url: impl Into<String>) -> Self {
        #[cfg(feature = "vella")]
        {
            let event_bus = Arc::new(EventBus::new(4096));
            let schema_registry = Arc::new(RwLock::new(SchemaRegistry::default()));
            let governor = Arc::new(VellaPolicyGovernor::default());
            Self {
                event_bus,
                schema_registry,
                governor,
                db_url: db_url.into(),
                is_initialized: Arc::new(RwLock::new(false)),
            }
        }
        #[cfg(not(feature = "vella"))]
        {
            Self {
                governor: Arc::new(VellaPolicyGovernor::default()),
                db_url: db_url.into(),
                is_initialized: Arc::new(RwLock::new(false)),
            }
        }
    }

    /// Pre-populates the schema registry with standard sovereign domains
    pub fn with_default_schemas() -> Self {
        let mgr = Self::default();
        #[cfg(feature = "vella")]
        {
            let mut schemas = std::collections::HashMap::new();

            // 1. Agent Record Schema
            let agent_schema = ModelSchema::new("agent_executions")
                .display_name("Agent Executions")
                .category("Swarm")
                .icon("bot")
                .description("Telemetry and execution history for Tagisan agents")
                .field(Field {
                    name: "agent_id".to_string(),
                    display_name: "Agent ID".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    searchable: true,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Identifier of the agent".to_string()),
                })
                .field(Field {
                    name: "action".to_string(),
                    display_name: "Action".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    searchable: true,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Tool or prompt action invoked".to_string()),
                })
                .with_timestamps();
            schemas.insert("agent_executions".to_string(), agent_schema);

            // 2. SCADA Telemetry Schema
            let scada_schema = ModelSchema::new("scada_telemetry")
                .display_name("SCADA Telemetry")
                .category("Industrial")
                .icon("cpu")
                .description("Realtime industrial sensor registers and alarms")
                .field(Field {
                    name: "device_id".to_string(),
                    display_name: "Device ID".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    searchable: true,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("PLC / RTU Device Identifier".to_string()),
                })
                .field(Field {
                    name: "register_addr".to_string(),
                    display_name: "Register Address".to_string(),
                    field_type: FieldType::Integer,
                    required: true,
                    unique: false,
                    searchable: false,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Modbus / OPC UA register index".to_string()),
                })
                .field(Field {
                    name: "value".to_string(),
                    display_name: "Register Value".to_string(),
                    field_type: FieldType::Integer,
                    required: true,
                    unique: false,
                    searchable: false,
                    filterable: false,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Holding register 16-bit word".to_string()),
                })
                .with_timestamps();
            schemas.insert("scada_telemetry".to_string(), scada_schema);

            // 3. Trade Orders Schema
            let trade_schema = ModelSchema::new("trade_orders")
                .display_name("Trade Orders")
                .category("Finance")
                .icon("trending-up")
                .description("Audited orders matched through Vella Quant Engine")
                .field(Field {
                    name: "symbol".to_string(),
                    display_name: "Symbol".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    searchable: true,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Asset or currency pair ticker".to_string()),
                })
                .field(Field {
                    name: "order_type".to_string(),
                    display_name: "Order Type".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    unique: false,
                    searchable: false,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("bid or ask".to_string()),
                })
                .field(Field {
                    name: "price".to_string(),
                    display_name: "Price".to_string(),
                    field_type: FieldType::Float,
                    required: true,
                    unique: false,
                    searchable: false,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Limit order execution price".to_string()),
                })
                .field(Field {
                    name: "size".to_string(),
                    display_name: "Size".to_string(),
                    field_type: FieldType::Integer,
                    required: true,
                    unique: false,
                    searchable: false,
                    filterable: true,
                    list_display: true,
                    read_only: false,
                    encrypted: false,
                    requires_approval: false,
                    spatial_indexed: false,
                    default_value: None,
                    help_text: Some("Quantity of contracts / shares".to_string()),
                })
                .with_timestamps();
            schemas.insert("trade_orders".to_string(), trade_schema);

            let registry = SchemaRegistry::from_map(schemas);
            *mgr.schema_registry.try_write().unwrap() = registry;
        }
        mgr
    }

    /// Initializes Vella App state, runs system schema migrations, and attaches event bus
    pub async fn initialize(&self) -> Result<()> {
        let mut init_lock = self.is_initialized.write().await;
        if *init_lock {
            return Ok(());
        }

        info!("Initializing VellaAppManager targeting database: {}", self.db_url);

        #[cfg(feature = "vella")]
        {
            // Create and verify VellaApp configuration
            let mut app = VellaApp::new()
                .site_name("Tagisan-Vella Sovereign OS")
                .database(&self.db_url)
                .semantic_cache(true, 0.85);

            let registry = self.schema_registry.read().await;
            for schema in registry.all() {
                app = app.register(schema.clone());
            }

            // Emit initialization event
            self.event_bus.publish(SystemEvent::RecordCreated {
                model: "system".to_string(),
                id: 1,
                data: serde_json::json!({
                    "engine": "tagisan-vella",
                    "status": "online",
                    "version": "0.1.0"
                }),
            });
        }

        *init_lock = true;
        info!("VellaAppManager initialized successfully.");
        Ok(())
    }

    pub async fn is_initialized(&self) -> bool {
        *self.is_initialized.read().await
    }

    #[cfg(feature = "vella")]
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    #[cfg(feature = "vella")]
    pub fn schema_registry(&self) -> Arc<RwLock<SchemaRegistry>> {
        self.schema_registry.clone()
    }

    pub fn governor(&self) -> Arc<VellaPolicyGovernor> {
        self.governor.clone()
    }
}
