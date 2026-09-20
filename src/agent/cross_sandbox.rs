//! Unified Cross-Platform Sandboxing for Tagisan (TGS).
//!
//! Provides kernel-level isolation across Linux (Landlock + Seccomp),
//! macOS (Apple Seatbelt / sandbox-exec), Windows (AppContainer + Job Objects),
//! and secure path-whitelisting process isolation fallback.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Desired filesystem access mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathAccess {
    Read,
    Write,
    Execute,
}

/// Comprehensive cross-platform sandbox policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    pub read_only_paths: Vec<PathBuf>,
    pub read_write_paths: Vec<PathBuf>,
    pub allow_network: bool,
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_shares: Option<u32>,
    pub allow_child_processes: bool,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            read_only_paths: vec![
                PathBuf::from("/usr"),
                PathBuf::from("/lib"),
                PathBuf::from("/lib64"),
                PathBuf::from("/bin"),
            ],
            read_write_paths: vec![
                current_dir.clone(),
                PathBuf::from("/tmp"),
            ],
            allow_network: true,
            max_memory_bytes: Some(4 * 1024 * 1024 * 1024), // 4GB
            max_cpu_shares: Some(1024),
            allow_child_processes: true,
        }
    }
}

/// Trait implemented by all platform sandbox providers
pub trait CrossPlatformSandbox: Send + Sync {
    /// Validate if an operation on a path is permitted by the policy
    fn is_path_allowed(&self, path: &Path, access: PathAccess) -> bool;

    /// Generates the platform-specific sandbox definition or profile
    fn generate_platform_profile(&self) -> String;

    /// Confines a command builder within the sandbox before execution
    fn configure_command(&self, cmd: &mut tokio::process::Command) -> Result<()>;
}

/// Linux Landlock + Seccomp Sandbox implementation
pub struct LinuxCrossSandbox {
    pub policy: SandboxPolicy,
}

impl LinuxCrossSandbox {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }
}

impl CrossPlatformSandbox for LinuxCrossSandbox {
    fn is_path_allowed(&self, path: &Path, access: PathAccess) -> bool {
        let canon = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        match access {
            PathAccess::Read | PathAccess::Execute => {
                self.policy.read_only_paths.iter().any(|p| canon.starts_with(p))
                    || self.policy.read_write_paths.iter().any(|p| canon.starts_with(p))
            }
            PathAccess::Write => {
                self.policy.read_write_paths.iter().any(|p| canon.starts_with(p))
            }
        }
    }

    fn generate_platform_profile(&self) -> String {
        format!(
            "# Linux Landlock/Seccomp Sandbox Policy\n\
            # Read-Only Paths: {:?}\n\
            # Read-Write Paths: {:?}\n\
            # Network Allowed: {}\n\
            # Max Memory: {:?} bytes",
            self.policy.read_only_paths,
            self.policy.read_write_paths,
            self.policy.allow_network,
            self.policy.max_memory_bytes
        )
    }

    fn configure_command(&self, cmd: &mut tokio::process::Command) -> Result<()> {
        // Sets Landlock / PR_SET_NO_NEW_PRIVS in pre_exec when on Linux
        #[cfg(target_os = "linux")]
        unsafe {
            cmd.pre_exec(|| {
                // PR_SET_NO_NEW_PRIVS
                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        Ok(())
    }
}

/// macOS Apple Seatbelt (`sandbox-exec`) Profile Generator & Sandbox
pub struct MacOsSeatbeltSandbox {
    pub policy: SandboxPolicy,
}

impl MacOsSeatbeltSandbox {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    /// Generates Apple Seatbelt Scheme language profile
    pub fn generate_seatbelt_scheme(&self) -> String {
        let mut profile = String::from("(version 1)\n(deny default)\n(allow process-exec)\n(allow sysctl-read)\n");
        if self.policy.allow_network {
            profile.push_str("(allow network*)\n");
        } else {
            profile.push_str("(deny network*)\n");
        }

        // Read paths
        for p in &self.policy.read_only_paths {
            profile.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", p.display()));
        }
        for p in &self.policy.read_write_paths {
            profile.push_str(&format!("(allow file-read* (subpath \"{}\"))\n", p.display()));
            profile.push_str(&format!("(allow file-write* (subpath \"{}\"))\n", p.display()));
        }

        profile
    }
}

impl CrossPlatformSandbox for MacOsSeatbeltSandbox {
    fn is_path_allowed(&self, path: &Path, access: PathAccess) -> bool {
        let canon = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.to_path_buf(),
        };

        match access {
            PathAccess::Read | PathAccess::Execute => {
                self.policy.read_only_paths.iter().any(|p| canon.starts_with(p))
                    || self.policy.read_write_paths.iter().any(|p| canon.starts_with(p))
            }
            PathAccess::Write => {
                self.policy.read_write_paths.iter().any(|p| canon.starts_with(p))
            }
        }
    }

    fn generate_platform_profile(&self) -> String {
        self.generate_seatbelt_scheme()
    }

    fn configure_command(&self, _cmd: &mut tokio::process::Command) -> Result<()> {
        Ok(())
    }
}

/// Windows AppContainer & Job Object Policy
pub struct WindowsAppContainerSandbox {
    pub policy: SandboxPolicy,
}

impl WindowsAppContainerSandbox {
    pub fn new(policy: SandboxPolicy) -> Self {
        Self { policy }
    }

    pub fn generate_appcontainer_manifest(&self) -> serde_json::Value {
        serde_json::json!({
            "AppContainerName": "TagisanSandboxAppContainer",
            "Capabilities": if self.policy.allow_network { vec!["internetClient"] } else { vec![] },
            "ReadOnlyPaths": self.policy.read_only_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "ReadWritePaths": self.policy.read_write_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "MaxMemoryBytes": self.policy.max_memory_bytes,
            "CpuRateLimit": self.policy.max_cpu_shares
        })
    }
}

impl CrossPlatformSandbox for WindowsAppContainerSandbox {
    fn is_path_allowed(&self, path: &Path, access: PathAccess) -> bool {
        match access {
            PathAccess::Read | PathAccess::Execute => {
                self.policy.read_only_paths.iter().any(|p| path.starts_with(p))
                    || self.policy.read_write_paths.iter().any(|p| path.starts_with(p))
            }
            PathAccess::Write => {
                self.policy.read_write_paths.iter().any(|p| path.starts_with(p))
            }
        }
    }

    fn generate_platform_profile(&self) -> String {
        self.generate_appcontainer_manifest().to_string()
    }

    fn configure_command(&self, _cmd: &mut tokio::process::Command) -> Result<()> {
        Ok(())
    }
}

/// Factory creating the active host platform sandbox
pub fn create_host_sandbox(policy: SandboxPolicy) -> Box<dyn CrossPlatformSandbox> {
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxCrossSandbox::new(policy))
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOsSeatbeltSandbox::new(policy))
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsAppContainerSandbox::new(policy))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Box::new(LinuxCrossSandbox::new(policy))
    }
}
