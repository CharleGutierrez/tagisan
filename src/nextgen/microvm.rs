//! # Zero-Trust WASI 0.2 Component & MicroVM Hypervisor (`tgs nextgen microvm`)
//!
//! Provides capability-attenuated sandboxing:
//! - Strict capability tokens for filesystem, network, and environment variables
//! - WASM Component Model (`wasi:cli` 0.2) header and import validation
//! - Capability Attenuation Invariant: Cap(Child) <= Cap(Parent)
//! - Ephemeral jail lifecycle with zero orphaned state

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

/// Granular capability permissions granted to an execution context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    FsRead(PathBuf),
    FsWrite(PathBuf),
    NetConnect { host: String, port: u16 },
    NetListen(u16),
    EnvVar(String),
}

/// Capability security profile for a MicroVM or WASI 0.2 component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MicroVmProfile {
    pub max_memory_bytes: u64,
    pub timeout_ms: u64,
    pub capabilities: HashSet<Capability>,
}

impl Default for MicroVmProfile {
    fn default() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024, // 256 MB
            timeout_ms: 5000,                    // 5 seconds
            capabilities: HashSet::new(),
        }
    }
}

impl MicroVmProfile {
    /// Grant read access to a directory path
    pub fn allow_read_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.capabilities.insert(Capability::FsRead(path.as_ref().to_path_buf()));
        self
    }

    /// Grant write access to a directory path
    pub fn allow_write_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.capabilities.insert(Capability::FsWrite(path.as_ref().to_path_buf()));
        self
    }

    /// Grant network connect access
    pub fn allow_net_connect(&mut self, host: &str, port: u16) -> &mut Self {
        self.capabilities.insert(Capability::NetConnect {
            host: host.to_string(),
            port,
        });
        self
    }

    /// Verify capability attenuation: child permissions MUST be a subset of parent permissions
    pub fn verify_attenuation(&self, child_profile: &MicroVmProfile) -> bool {
        child_profile.capabilities.is_subset(&self.capabilities)
            && child_profile.max_memory_bytes <= self.max_memory_bytes
            && child_profile.timeout_ms <= self.timeout_ms
    }
}

/// WASI 0.2 Component & MicroVM Validator
pub struct WasiMicroVmValidator;

impl WasiMicroVmValidator {
    /// Validate WebAssembly magic header and version bytes (\0asm\1\0\0\0)
    pub fn validate_wasm_binary(bytes: &[u8]) -> Result<()> {
        if bytes.len() < 8 {
            return Err(TagisanError::Execution("Binary too small to be valid WASM".to_string()));
        }

        let magic = &bytes[0..4];
        let version = &bytes[4..8];

        if magic != b"\0asm" {
            return Err(TagisanError::Execution("Invalid WASM magic header".to_string()));
        }

        if version != [0x01, 0x00, 0x00, 0x00] && version != [0x0d, 0x00, 0x01, 0x00] {
            return Err(TagisanError::Execution(format!(
                "Unsupported WASM/WASI version: {:02x?}",
                version
            )));
        }

        Ok(())
    }

    /// Check if a capability is authorized by the current profile
    pub fn is_action_allowed(profile: &MicroVmProfile, action: &Capability) -> bool {
        match action {
            Capability::FsRead(req_path) => profile.capabilities.iter().any(|cap| {
                if let Capability::FsRead(allowed_path) = cap {
                    req_path.starts_with(allowed_path)
                } else {
                    false
                }
            }),
            Capability::FsWrite(req_path) => profile.capabilities.iter().any(|cap| {
                if let Capability::FsWrite(allowed_path) = cap {
                    req_path.starts_with(allowed_path)
                } else {
                    false
                }
            }),
            Capability::NetConnect { host, port } => profile.capabilities.iter().any(|cap| {
                if let Capability::NetConnect { host: h, port: p } = cap {
                    (h == "*" || h == host) && (*p == 0 || *p == *port)
                } else {
                    false
                }
            }),
            Capability::NetListen(port) => profile.capabilities.contains(&Capability::NetListen(*port)),
            Capability::EnvVar(var) => profile.capabilities.contains(&Capability::EnvVar(var.clone())),
        }
    }
}
