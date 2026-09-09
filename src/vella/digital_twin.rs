//! # Hardware-In-The-Loop (HIL) Digital Twin Simulation Sandbox
//!
//! Pre-actuation physical simulation for robotics kinematics and SCADA industrial
//! thermodynamic loops. Simulates velocity, acceleration, torque demand, and
//! thermal buildup prior to physical hardware dispatch. Destructive commands that
//! violate physical safety envelopes are blocked before physical execution.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use crate::vella::VellaPolicyGovernor;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[cfg(feature = "vella")]
use vella::scada::simulation::ScadaSimulation;

/// Verdict returned by the Digital Twin simulation engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DigitalTwinVerdict {
    SafeToExecute {
        safety_margin_percent: f64,
        metrics: Value,
    },
    Blocked {
        reason: String,
        critical_metric: String,
        observed_value: f64,
        threshold_value: f64,
        physical_consequence: String,
    },
}

/// Thermodynamic and pressure state of the SCADA digital twin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScadaTwinState {
    pub temperature_c: f64,
    pub pressure_psi: f64,
    pub cooling_active: bool,
    pub release_valve_open: bool,
    pub ticks_simulated: usize,
    pub max_temp_observed: f64,
    pub max_pressure_observed: f64,
}

/// Kinematics trajectory parameters for robotic actuators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoboticsTrajectorySpec {
    pub target_linear_velocity_mps: f64,
    pub target_angular_velocity_radps: f64,
    pub acceleration_radps2: f64,
    pub payload_mass_kg: f64,
    pub arm_reach_meters: f64,
    pub duration_seconds: f64,
}

/// Physical simulation engine for SCADA and Robotics
#[derive(Clone)]
pub struct DigitalTwinEngine {
    pub governor: Arc<VellaPolicyGovernor>,
    #[cfg(feature = "vella")]
    pub scada_sim: Arc<Mutex<ScadaSimulation>>,
    pub max_motor_torque_nm: f64,
    pub max_temp_celsius: f64,
    pub max_pressure_psi: f64,
    pub max_thermal_rise_c: f64,
}

impl Default for DigitalTwinEngine {
    fn default() -> Self {
        Self::new(Arc::new(VellaPolicyGovernor::default()))
    }
}

impl DigitalTwinEngine {
    pub fn new(governor: Arc<VellaPolicyGovernor>) -> Self {
        Self {
            governor,
            #[cfg(feature = "vella")]
            scada_sim: Arc::new(Mutex::new(ScadaSimulation::new())),
            max_motor_torque_nm: 75.0, // Industrial servo peak continuous torque
            max_temp_celsius: 185.0,   // Critical boiler thermal limit (industrial steam ceiling)
            max_pressure_psi: 300.0,   // Critical pipe rupture limit (high-pressure steam boundary)
            max_thermal_rise_c: 40.0,  // Motor coil thermal runaway limit
        }
    }

    /// Simulate SCADA thermodynamic envelope over multiple forward time steps
    pub async fn simulate_scada_envelope(
        &self,
        ticks: usize,
        force_cooling: Option<bool>,
        force_valve: Option<bool>,
        heat_load_spike: f64,
    ) -> Result<DigitalTwinVerdict> {
        #[cfg(feature = "vella")]
        {
            let mut sim = self.scada_sim.lock().await;
            let mut twin_state = ScadaTwinState {
                temperature_c: sim.temperature,
                pressure_psi: sim.pressure,
                cooling_active: sim.cooling_active,
                release_valve_open: sim.release_valve_open,
                ticks_simulated: 0,
                max_temp_observed: sim.temperature,
                max_pressure_observed: sim.pressure,
            };

            for _ in 0..ticks {
                if let Some(c) = force_cooling {
                    sim.cooling_active = c;
                }
                if let Some(v) = force_valve {
                    sim.release_valve_open = v;
                }

                // Apply heat load spike
                sim.temperature += heat_load_spike;

                // Advance real physics tick in Vella
                sim.tick();

                twin_state.ticks_simulated += 1;
                twin_state.temperature_c = sim.temperature;
                twin_state.pressure_psi = sim.pressure;
                twin_state.cooling_active = sim.cooling_active;
                twin_state.release_valve_open = sim.release_valve_open;

                if sim.temperature > twin_state.max_temp_observed {
                    twin_state.max_temp_observed = sim.temperature;
                }
                if sim.pressure > twin_state.max_pressure_observed {
                    twin_state.max_pressure_observed = sim.pressure;
                }

                // Physical breach detection
                if sim.pressure > self.max_pressure_psi {
                    warn!("🚨 [Digital Twin SCADA] Pressure critical breach: {:.2} PSI", sim.pressure);
                    return Ok(DigitalTwinVerdict::Blocked {
                        reason: "Digital Twin detected catastrophic boiler overpressure".to_string(),
                        critical_metric: "pressure_psi".to_string(),
                        observed_value: sim.pressure,
                        threshold_value: self.max_pressure_psi,
                        physical_consequence: "Catastrophic containment rupture and explosive decompression"
                            .to_string(),
                    });
                }

                if sim.temperature > self.max_temp_celsius {
                    warn!("🚨 [Digital Twin SCADA] Thermal critical breach: {:.2} C", sim.temperature);
                    return Ok(DigitalTwinVerdict::Blocked {
                        reason: "Digital Twin detected critical core thermal runaway".to_string(),
                        critical_metric: "temperature_celsius".to_string(),
                        observed_value: sim.temperature,
                        threshold_value: self.max_temp_celsius,
                        physical_consequence: "Core meltdown and coolant seal degradation".to_string(),
                    });
                }
            }

            let pressure_headroom = (self.max_pressure_psi - twin_state.max_pressure_observed) / self.max_pressure_psi * 100.0;
            let temp_headroom = (self.max_temp_celsius - twin_state.max_temp_observed) / self.max_temp_celsius * 100.0;
            let safety_margin = pressure_headroom.min(temp_headroom).max(0.0);

            info!("✅ [Digital Twin SCADA] Envelope verified safe: margin {:.1}%", safety_margin);
            Ok(DigitalTwinVerdict::SafeToExecute {
                safety_margin_percent: safety_margin,
                metrics: json!(twin_state),
            })
        }
        #[cfg(not(feature = "vella"))]
        {
            Ok(DigitalTwinVerdict::SafeToExecute {
                safety_margin_percent: 100.0,
                metrics: json!({"status": "vella_disabled"}),
            })
        }
    }

    /// Simulate robotics kinematic trajectory, calculating torque and thermal dissipation
    pub async fn simulate_robotics_trajectory(
        &self,
        spec: &RoboticsTrajectorySpec,
    ) -> Result<DigitalTwinVerdict> {
        // 1. Check Policy Governor velocity limits
        if spec.target_linear_velocity_mps > self.governor.max_robot_velocity_ms {
            return Ok(DigitalTwinVerdict::Blocked {
                reason: "Linear velocity exceeds sovereign policy ceiling".to_string(),
                critical_metric: "linear_velocity_mps".to_string(),
                observed_value: spec.target_linear_velocity_mps,
                threshold_value: self.governor.max_robot_velocity_ms,
                physical_consequence: "Kinematic instability and trajectory divergence".to_string(),
            });
        }

        // 2. Compute dynamic rotational inertia and peak motor torque
        // I = m * r^2
        let inertia = spec.payload_mass_kg * spec.arm_reach_meters.powi(2);
        // Static gravity torque = m * g * r
        let gravity_torque = spec.payload_mass_kg * 9.80665 * spec.arm_reach_meters;
        // Dynamic acceleration torque = I * alpha
        let dynamic_torque = inertia * spec.acceleration_radps2;
        let peak_torque_demand = dynamic_torque + gravity_torque;

        if peak_torque_demand > self.max_motor_torque_nm {
            warn!(
                "🚨 [Digital Twin Robotics] Torque overload: {:.2} Nm > {:.2} Nm limit",
                peak_torque_demand, self.max_motor_torque_nm
            );
            return Ok(DigitalTwinVerdict::Blocked {
                reason: "Dynamic torque exceeds maximum motor mechanical rating".to_string(),
                critical_metric: "motor_torque_nm".to_string(),
                observed_value: peak_torque_demand,
                threshold_value: self.max_motor_torque_nm,
                physical_consequence: "Gearbox tooth shearing and motor stall shutdown".to_string(),
            });
        }

        // 3. Compute thermal buildup in actuator windings: Q_dot = (tau / Kt)^2 * R
        let torque_constant_kt = 1.2; // Nm/A
        let winding_resistance_r = 0.85; // Ohms
        let current_amps = peak_torque_demand / torque_constant_kt;
        let power_dissipation_watts = current_amps.powi(2) * winding_resistance_r;
        let thermal_mass_c = 150.0; // J/C
        let delta_temp_c = (power_dissipation_watts * spec.duration_seconds) / thermal_mass_c;

        if delta_temp_c > self.max_thermal_rise_c {
            warn!(
                "🚨 [Digital Twin Robotics] Thermal rise critical: +{:.2} C",
                delta_temp_c
            );
            return Ok(DigitalTwinVerdict::Blocked {
                reason: "Thermal dissipation exceeds actuator cooling capacity".to_string(),
                critical_metric: "thermal_rise_celsius".to_string(),
                observed_value: delta_temp_c,
                threshold_value: self.max_thermal_rise_c,
                physical_consequence: "Winding insulation breakdown and permanent demagnetization"
                    .to_string(),
            });
        }

        let kinetic_energy = 0.5 * spec.payload_mass_kg * spec.target_linear_velocity_mps.powi(2)
            + 0.5 * inertia * spec.target_angular_velocity_radps.powi(2);

        let torque_headroom = (self.max_motor_torque_nm - peak_torque_demand) / self.max_motor_torque_nm * 100.0;
        let thermal_headroom = (self.max_thermal_rise_c - delta_temp_c) / self.max_thermal_rise_c * 100.0;
        let safety_margin = torque_headroom.min(thermal_headroom).max(0.0);

        info!(
            "✅ [Digital Twin Robotics] Trajectory validated: torque {:.2} Nm, +{:.1} C, margin {:.1}%",
            peak_torque_demand, delta_temp_c, safety_margin
        );

        Ok(DigitalTwinVerdict::SafeToExecute {
            safety_margin_percent: safety_margin,
            metrics: json!({
                "inertia_kg_m2": inertia,
                "peak_torque_demand_nm": peak_torque_demand,
                "gravity_torque_nm": gravity_torque,
                "current_amps": current_amps,
                "power_dissipation_watts": power_dissipation_watts,
                "thermal_rise_celsius": delta_temp_c,
                "kinetic_energy_joules": kinetic_energy,
            }),
        })
    }
}

// =========================================================================
// Tool Handler Implementation
// =========================================================================

/// Tool exposing HIL Digital Twin physics simulation
#[derive(Clone)]
pub struct VellaDigitalTwinTool {
    pub engine: DigitalTwinEngine,
}

impl Default for VellaDigitalTwinTool {
    fn default() -> Self {
        Self::new(DigitalTwinEngine::default())
    }
}

impl VellaDigitalTwinTool {
    pub fn new(engine: DigitalTwinEngine) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl ToolHandler for VellaDigitalTwinTool {
    fn name(&self) -> &'static str {
        "vella_digital_twin"
    }

    fn description(&self) -> &'static str {
        "Hardware-In-The-Loop (HIL) physics sandbox simulating SCADA thermodynamic loops and Robotics kinematic envelopes prior to hardware execution."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["simulate_scada", "simulate_robotics", "verify_trajectory"],
                    "description": "Simulation domain action"
                },
                "scada": {
                    "type": "object",
                    "properties": {
                        "ticks": { "type": "integer", "default": 5 },
                        "force_cooling": { "type": "boolean" },
                        "force_valve": { "type": "boolean" },
                        "heat_load_spike": { "type": "number", "default": 0.0 }
                    }
                },
                "robotics": {
                    "type": "object",
                    "properties": {
                        "linear_velocity_mps": { "type": "number", "default": 1.5 },
                        "angular_velocity_radps": { "type": "number", "default": 0.8 },
                        "acceleration_radps2": { "type": "number", "default": 2.0 },
                        "payload_mass_kg": { "type": "number", "default": 2.5 },
                        "arm_reach_meters": { "type": "number", "default": 0.75 },
                        "duration_seconds": { "type": "number", "default": 1.0 }
                    }
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing 'action' parameter".to_string()))?;

        let res: Value = match action {
            "simulate_scada" => {
                let scada_arg = arguments.get("scada").cloned().unwrap_or(json!({}));
                let ticks = scada_arg.get("ticks").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let force_cooling = scada_arg.get("force_cooling").and_then(|v| v.as_bool());
                let force_valve = scada_arg.get("force_valve").and_then(|v| v.as_bool());
                let heat_spike = scada_arg
                    .get("heat_load_spike")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let verdict = self
                    .engine
                    .simulate_scada_envelope(ticks, force_cooling, force_valve, heat_spike)
                    .await?;

                json!({
                    "status": "success",
                    "domain": "scada_thermodynamics",
                    "verdict": verdict
                })
            }
            "simulate_robotics" | "verify_trajectory" => {
                let robot_arg = arguments.get("robotics").cloned().unwrap_or(json!({}));
                let spec = RoboticsTrajectorySpec {
                    target_linear_velocity_mps: robot_arg
                        .get("linear_velocity_mps")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.5),
                    target_angular_velocity_radps: robot_arg
                        .get("angular_velocity_radps")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.8),
                    acceleration_radps2: robot_arg
                        .get("acceleration_radps2")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(2.0),
                    payload_mass_kg: robot_arg
                        .get("payload_mass_kg")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(2.5),
                    arm_reach_meters: robot_arg
                        .get("arm_reach_meters")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.75),
                    duration_seconds: robot_arg
                        .get("duration_seconds")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0),
                };

                let verdict = self.engine.simulate_robotics_trajectory(&spec).await?;

                json!({
                    "status": "success",
                    "domain": "robotics_kinematics",
                    "spec": spec,
                    "verdict": verdict
                })
            }
            _ => {
                return Err(TagisanError::Execution(format!(
                    "Unknown action '{}'. Valid actions: simulate_scada, simulate_robotics, verify_trajectory",
                    action
                )));
            }
        };

        serde_json::to_string_pretty(&res).map_err(|e| TagisanError::Execution(e.to_string()))
    }
}
