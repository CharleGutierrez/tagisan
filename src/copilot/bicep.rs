//! Azure Bicep & ARM Infrastructure AST Engine
//!
//! Subsystem 3: Microsoft Ecosystem Expansion for Tagisan (`tgs`).
//!
//! Exposes:
//! 1. Bicep Syntax Parser & Resource Dependency Graph Builder:
//!    - Parses `resource` declarations (VNets, Subnets, KeyVault, AppService, CosmosDB, Storage, RoleAssignments, Managed Identities).
//!    - Extracts parameters, properties, and explicit (`dependsOn`) / implicit (`parent`, symbolic member access) dependencies.
//! 2. IaC Blast Radius Analyzer:
//!    - Traces cascading downstream impact when an infrastructure resource is modified, resized, or destroyed.
//!    - Computes blast risk level (`Low`, `Medium`, `High`, `Critical`).
//! 3. RBAC & Security Invariant Checker:
//!    - Verifies least privilege (flags wildcard `*` permissions in role definitions/assignments).
//!    - Verifies compliance invariants (flags `publicNetworkAccess: 'Enabled'`, unencrypted storage, missing managed identities).
//! 4. `CopilotBicepTool`:
//!    - Implements `ToolHandler` exposing these capabilities with full JSON schemas.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

// =========================================================================
// 1. Data Models
// =========================================================================

/// Parsed Azure Bicep resource declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BicepResource {
    pub symbolic_name: String,
    pub resource_type: String,
    pub api_version: String,
    pub name: Option<String>,
    pub location: Option<String>,
    pub parent: Option<String>,
    pub depends_on: Vec<String>,
    pub referenced_symbols: Vec<String>,
    pub properties_raw: String,
    pub line_number: usize,
}

/// Node in the Infrastructure Dependency Graph
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BicepGraphNode {
    pub resource: BicepResource,
    pub downstream_dependents: Vec<String>, // Resources that depend on this one
    pub upstream_dependencies: Vec<String>, // Resources this one depends on
}

/// Infrastructure Dependency Knowledge Graph
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BicepGraph {
    pub nodes: HashMap<String, BicepGraphNode>,
}

/// IaC Blast Radius Report for targeted infrastructure modification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IacBlastRadiusReport {
    pub target_resource: String,
    pub target_type: String,
    pub direct_dependents: Vec<String>,
    pub transitive_dependents: Vec<String>,
    pub total_affected_resources: usize,
    pub risk_level: String,
    pub disruption_impact: String,
    pub recommendations: Vec<String>,
}

/// Security or RBAC Invariant Violation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvariantViolation {
    pub resource_name: String,
    pub resource_type: String,
    pub rule_id: String,
    pub severity: String,
    pub description: String,
    pub remediation: String,
    pub line_number: usize,
}

/// Comprehensive Infrastructure Audit Report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BicepAuditReport {
    pub total_resources: usize,
    pub violations: Vec<InvariantViolation>,
    pub compliance_score: f32,
    pub is_compliant: bool,
    pub summary: String,
}

// =========================================================================
// 2. Bicep Engine
// =========================================================================

/// Azure Bicep & ARM Infrastructure AST Engine
#[derive(Clone, Default)]
pub struct BicepEngine;

impl BicepEngine {
    pub fn new() -> Self {
        Self
    }

    /// Parses Bicep template code into structured BicepResource nodes
    pub fn parse_bicep(&self, bicep_code: &str) -> Result<Vec<BicepResource>> {
        let mut resources = Vec::new();
        let lines: Vec<&str> = bicep_code.lines().collect();

        // Regex for resource header: resource <symbolicName> '<type>@<apiVersion>' = {
        let resource_header_re = Regex::new(
            r"^resource\s+(?P<symbol>[a-zA-Z0-9_]+)\s+'(?P<type>[^@']+)(?:@(?P<version>[^']+))?'\s*=\s*\{",
        )
        .map_err(|e| TagisanError::Execution(format!("Regex compile error: {e}")))?;

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if let Some(caps) = resource_header_re.captures(line) {
                let symbolic_name = caps["symbol"].to_string();
                let resource_type = caps["type"].to_string();
                let api_version = caps.name("version").map(|m| m.as_str()).unwrap_or("latest").to_string();
                let line_number = i + 1;

                // Collect resource body by tracking braces
                let mut body_lines = Vec::new();
                let mut brace_count = 1;
                i += 1;

                while i < lines.len() && brace_count > 0 {
                    let current = lines[i];
                    for ch in current.chars() {
                        if ch == '{' {
                            brace_count += 1;
                        } else if ch == '}' {
                            brace_count -= 1;
                        }
                    }
                    if brace_count > 0 {
                        body_lines.push(current);
                    }
                    i += 1;
                }

                let body = body_lines.join("\n");

                // Extract properties from body
                let name = Self::extract_string_property(&body, "name");
                let location = Self::extract_string_property(&body, "location");
                let parent = Self::extract_identifier_property(&body, "parent");

                // Extract dependsOn array
                let depends_on = Self::extract_depends_on(&body);

                // Extract symbolic references across the body (e.g. `vnet.id`, `appServicePlan.id`)
                let referenced_symbols = Self::extract_symbolic_references(&body, &symbolic_name);

                resources.push(BicepResource {
                    symbolic_name,
                    resource_type,
                    api_version,
                    name,
                    location,
                    parent,
                    depends_on,
                    referenced_symbols,
                    properties_raw: body,
                    line_number,
                });
                continue;
            }
            i += 1;
        }

        Ok(resources)
    }

    fn extract_string_property(body: &str, prop: &str) -> Option<String> {
        let pattern = format!(r#"{prop}\s*:\s*(?:'([^']+)'|"([^"]+)")"#);
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(caps) = re.captures(body) {
                return caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_identifier_property(body: &str, prop: &str) -> Option<String> {
        let pattern = format!(r#"{prop}\s*:\s*([a-zA-Z0-9_]+)"#);
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(caps) = re.captures(body) {
                return caps.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_depends_on(body: &str) -> Vec<String> {
        let mut deps = Vec::new();
        if let Some(pos) = body.find("dependsOn:") {
            let after = &body[pos..];
            if let Some(start_bracket) = after.find('[') {
                if let Some(end_bracket) = after[start_bracket..].find(']') {
                    let array_slice = &after[start_bracket + 1..start_bracket + end_bracket];
                    for item in array_slice.split_whitespace() {
                        let clean = item.trim_matches(|c| c == ',' || c == '\'' || c == '"');
                        if !clean.is_empty() && clean != "[" && clean != "]" {
                            deps.push(clean.to_string());
                        }
                    }
                }
            }
        }
        deps
    }

    fn extract_symbolic_references(body: &str, current_sym: &str) -> Vec<String> {
        let mut refs = HashSet::new();
        // Look for expressions like `<symbol>.id`, `<symbol>.name`, `<symbol>.properties`
        if let Ok(re) = Regex::new(r"([a-zA-Z0-9_]+)\.(id|name|properties|outputs)") {
            for caps in re.captures_iter(body) {
                let sym = caps.get(1).unwrap().as_str();
                if sym != current_sym && sym != "resourceGroup" && sym != "subscription" && sym != "deployment" {
                    refs.insert(sym.to_string());
                }
            }
        }
        refs.into_iter().collect()
    }

    /// Builds the Directed Infrastructure Dependency Graph
    pub fn build_graph(&self, resources: &[BicepResource]) -> BicepGraph {
        let mut graph = BicepGraph::default();

        // 1. Initialize nodes
        for res in resources {
            graph.nodes.insert(
                res.symbolic_name.clone(),
                BicepGraphNode {
                    resource: res.clone(),
                    downstream_dependents: Vec::new(),
                    upstream_dependencies: Vec::new(),
                },
            );
        }

        // 2. Connect edges
        for res in resources {
            let mut all_deps = HashSet::new();

            // Explicit dependsOn
            for d in &res.depends_on {
                all_deps.insert(d.clone());
            }

            // Parent reference
            if let Some(ref p) = res.parent {
                all_deps.insert(p.clone());
            }

            // Member access references (e.g. `vnet.id`)
            for r in &res.referenced_symbols {
                all_deps.insert(r.clone());
            }

            // Register upstream dependencies for current resource
            for dep in &all_deps {
                if graph.nodes.contains_key(dep) {
                    if let Some(node) = graph.nodes.get_mut(&res.symbolic_name) {
                        node.upstream_dependencies.push(dep.clone());
                    }
                    // Register current resource as downstream dependent of dep
                    if let Some(dep_node) = graph.nodes.get_mut(dep) {
                        dep_node.downstream_dependents.push(res.symbolic_name.clone());
                    }
                }
            }
        }

        graph
    }

    /// Computes IaC Blast Radius for a targeted resource modification or destruction
    pub fn calculate_blast_radius(
        &self,
        graph: &BicepGraph,
        target_symbol: &str,
    ) -> Result<IacBlastRadiusReport> {
        let target_node = graph.nodes.get(target_symbol).ok_or_else(|| {
            TagisanError::Execution(format!("Target resource '{target_symbol}' not found in Bicep graph"))
        })?;

        let mut direct_dependents = Vec::new();
        let mut transitive_dependents = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(target_symbol.to_string());

        let mut queue = VecDeque::new();
        for direct in &target_node.downstream_dependents {
            direct_dependents.push(direct.clone());
            visited.insert(direct.clone());
            queue.push_back((direct.clone(), 1));
        }

        while let Some((curr, depth)) = queue.pop_front() {
            if let Some(node) = graph.nodes.get(&curr) {
                for next in &node.downstream_dependents {
                    if !visited.contains(next) {
                        visited.insert(next.clone());
                        transitive_dependents.push(next.clone());
                        queue.push_back((next.clone(), depth + 1));
                    }
                }
            }
        }

        let total_affected = direct_dependents.len() + transitive_dependents.len();
        let is_core_infra = target_node.resource.resource_type.contains("virtualNetworks")
            || target_node.resource.resource_type.contains("vaults")
            || target_node.resource.resource_type.contains("serverfarms");

        let (risk_level, disruption_impact) = if total_affected >= 5 || (is_core_infra && total_affected >= 2) {
            (
                "Critical".to_string(),
                format!(
                    "SEVERE DISRUPTION: Modifying '{}' affects {} infrastructure components including mission-critical downstream services.",
                    target_symbol, total_affected
                ),
            )
        } else if total_affected >= 3 || is_core_infra {
            (
                "High".to_string(),
                format!(
                    "HIGH IMPACT: Modifying '{}' causes configuration churn across {} connected resources.",
                    target_symbol, total_affected
                ),
            )
        } else if total_affected >= 1 {
            (
                "Medium".to_string(),
                format!(
                    "MODERATE IMPACT: Localized downstream updates required for {} dependent resource(s).",
                    total_affected
                ),
            )
        } else {
            (
                "Low".to_string(),
                format!(
                    "MINIMAL IMPACT: Resource '{}' has zero downstream dependents. Safe to modify or delete.",
                    target_symbol
                ),
            )
        };

        let mut recommendations = Vec::new();
        if risk_level == "Critical" || risk_level == "High" {
            recommendations.push("Execute pre-flight 'az deployment group what-if' before applying changes.".to_string());
            recommendations.push("Implement staged deployment slots to eliminate downtime for downstream App Services.".to_string());
            recommendations.push("Verify KeyVault secret rotation policies prior to modifying vault configurations.".to_string());
        } else {
            recommendations.push("Standard IaC deployment pipeline is safe to proceed.".to_string());
        }

        Ok(IacBlastRadiusReport {
            target_resource: target_symbol.to_string(),
            target_type: target_node.resource.resource_type.clone(),
            direct_dependents,
            transitive_dependents,
            total_affected_resources: total_affected,
            risk_level,
            disruption_impact,
            recommendations,
        })
    }

    /// Verifies RBAC Least-Privilege & Security Compliance Invariants across Bicep resources
    pub fn check_invariants(&self, resources: &[BicepResource]) -> BicepAuditReport {
        let mut violations = Vec::new();

        for res in resources {
            let body = &res.properties_raw;
            let type_str = &res.resource_type;

            // 1. RBAC Least-Privilege Wildcard Detection (Rule RBAC-W001)
            if type_str.contains("roleAssignments") || type_str.contains("roleDefinitions") {
                let has_wildcard = body.contains("'*'")
                    || body.contains("\"*\"")
                    || body.contains("roleDefinitions/*")
                    || body.contains("Contributor")
                    || body.contains("Owner");

                if has_wildcard {
                    violations.push(InvariantViolation {
                        resource_name: res.symbolic_name.clone(),
                        resource_type: type_str.clone(),
                        rule_id: "RBAC-W001".to_string(),
                        severity: "Critical".to_string(),
                        description: format!(
                            "Resource '{}' grants wildcard or overly permissive RBAC roles ('*'/Contributor/Owner).",
                            res.symbolic_name
                        ),
                        remediation: "Scope role assignments to custom least-privilege actions (e.g. 'Microsoft.Storage/storageAccounts/read').".to_string(),
                        line_number: res.line_number,
                    });
                }
            }

            // 2. Public Network Access Invariant (Rule SEC-NET-001)
            let is_data_service = type_str.contains("vaults")
                || type_str.contains("storageAccounts")
                || type_str.contains("databaseAccounts")
                || type_str.contains("sql/servers");

            if is_data_service {
                let public_access_enabled = body.contains("publicNetworkAccess: 'Enabled'")
                    || body.contains("publicNetworkAccess: \"Enabled\"")
                    || body.contains("publicNetworkAccess: 'true'")
                    || body.contains("publicNetworkAccess: true");

                if public_access_enabled {
                    violations.push(InvariantViolation {
                        resource_name: res.symbolic_name.clone(),
                        resource_type: type_str.clone(),
                        rule_id: "SEC-NET-001".to_string(),
                        severity: "High".to_string(),
                        description: format!(
                            "Data service '{}' enables public network access (`publicNetworkAccess: 'Enabled'`).",
                            res.symbolic_name
                        ),
                        remediation: "Set `publicNetworkAccess: 'Disabled'` and configure Azure Private Endpoints within the Virtual Network.".to_string(),
                        line_number: res.line_number,
                    });
                }
            }

            // 3. Storage Encryption & HTTPS Invariant (Rule SEC-ENC-001)
            if type_str.contains("storageAccounts") {
                let insecure_traffic = body.contains("supportsHttpsTrafficOnly: false")
                    || body.contains("supportsHttpsTrafficOnly: 'false'")
                    || body.contains("allowBlobPublicAccess: true");

                if insecure_traffic {
                    violations.push(InvariantViolation {
                        resource_name: res.symbolic_name.clone(),
                        resource_type: type_str.clone(),
                        rule_id: "SEC-ENC-001".to_string(),
                        severity: "Critical".to_string(),
                        description: format!(
                            "Storage Account '{}' allows unencrypted HTTP traffic or public blob access.",
                            res.symbolic_name
                        ),
                        remediation: "Ensure `supportsHttpsTrafficOnly: true` and `allowBlobPublicAccess: false`.".to_string(),
                        line_number: res.line_number,
                    });
                }
            }

            // 4. Managed Identity Invariant (Rule SEC-ID-001)
            let requires_identity = type_str.contains("sites") || type_str.contains("databaseAccounts");
            if requires_identity {
                let has_identity = body.contains("identity:")
                    && (body.contains("SystemAssigned") || body.contains("UserAssigned"));

                let identity_none = body.contains("type: 'None'") || body.contains("type: \"None\"");

                if !has_identity || identity_none {
                    violations.push(InvariantViolation {
                        resource_name: res.symbolic_name.clone(),
                        resource_type: type_str.clone(),
                        rule_id: "SEC-ID-001".to_string(),
                        severity: "High".to_string(),
                        description: format!(
                            "Service '{}' does not configure a Managed Identity for zero-trust token authentication.",
                            res.symbolic_name
                        ),
                        remediation: "Add `identity: { type: 'SystemAssigned' }` or UserAssigned identity to avoid storing credentials in code.".to_string(),
                        line_number: res.line_number,
                    });
                }
            }

            // 5. Minimum TLS Version Invariant (Rule SEC-TLS-001)
            if body.contains("minTlsVersion") {
                let outdated_tls = body.contains("minTlsVersion: '1.0'")
                    || body.contains("minTlsVersion: '1.1'")
                    || body.contains("minTlsVersion: \"1.0\"")
                    || body.contains("minTlsVersion: \"1.1\"");

                if outdated_tls {
                    violations.push(InvariantViolation {
                        resource_name: res.symbolic_name.clone(),
                        resource_type: type_str.clone(),
                        rule_id: "SEC-TLS-001".to_string(),
                        severity: "Medium".to_string(),
                        description: format!(
                            "Resource '{}' specifies legacy TLS version (< 1.2).",
                            res.symbolic_name
                        ),
                        remediation: "Update `minTlsVersion: '1.2'` across all endpoints.".to_string(),
                        line_number: res.line_number,
                    });
                }
            }
        }

        let total_res = resources.len();
        let violation_count = violations.len();
        let compliance_score = if total_res == 0 {
            100.0
        } else {
            ((total_res as f32 - (violation_count as f32 * 0.5)).max(0.0) / total_res as f32) * 100.0
        };

        let is_compliant = violations.is_empty();
        let summary = format!(
            "Audited {} Bicep resources: {} violation(s) detected. Compliance Score: {:.1}% ({})",
            total_res,
            violation_count,
            compliance_score,
            if is_compliant { "PASSED" } else { "FAILED" }
        );

        BicepAuditReport {
            total_resources: total_res,
            violations,
            compliance_score,
            is_compliant,
            summary,
        }
    }
}

// =========================================================================
// 3. CopilotBicepTool (ToolHandler Implementation)
// =========================================================================

/// Autonomous Tool exposing Azure Bicep IaC Parsing, Dependency Graphing, Blast Radius Analysis,
/// and Security Invariant Verification to Agent Workflows and MCP
#[derive(Clone, Default)]
pub struct CopilotBicepTool {
    engine: Arc<BicepEngine>,
}

impl CopilotBicepTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_engine(engine: BicepEngine) -> Self {
        Self {
            engine: Arc::new(engine),
        }
    }
}

#[async_trait]
impl ToolHandler for CopilotBicepTool {
    fn name(&self) -> &str {
        "copilot_bicep"
    }

    fn description(&self) -> &str {
        "Azure Bicep & ARM Infrastructure AST Engine: parses resources, builds dependency graphs, analyzes IaC blast radius, and verifies RBAC least-privilege invariants."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "parse_bicep",
                        "build_graph",
                        "analyze_blast_radius",
                        "check_invariants",
                        "audit_rbac"
                    ],
                    "description": "The specific Bicep IaC capability to execute."
                },
                "bicep_code": {
                    "type": "string",
                    "description": "Azure Bicep template code string."
                },
                "target_resource": {
                    "type": "string",
                    "description": "Symbolic name of the resource to evaluate blast radius for (e.g. 'vnet', 'keyVault')."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|a| a.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        let bicep_code = arguments
            .get("bicep_code")
            .and_then(|c| c.as_str())
            .unwrap_or("");

        match action {
            "parse_bicep" => {
                let resources = self.engine.parse_bicep(bicep_code)?;
                Ok(serde_json::to_string_pretty(&resources)?)
            }
            "build_graph" => {
                let resources = self.engine.parse_bicep(bicep_code)?;
                let graph = self.engine.build_graph(&resources);
                Ok(serde_json::to_string_pretty(&graph)?)
            }
            "analyze_blast_radius" => {
                let target = arguments
                    .get("target_resource")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing required 'target_resource'".to_string()))?;

                let resources = self.engine.parse_bicep(bicep_code)?;
                let graph = self.engine.build_graph(&resources);
                let report = self.engine.calculate_blast_radius(&graph, target)?;
                Ok(serde_json::to_string_pretty(&report)?)
            }
            "check_invariants" | "audit_rbac" => {
                let resources = self.engine.parse_bicep(bicep_code)?;
                let audit = self.engine.check_invariants(&resources);
                Ok(serde_json::to_string_pretty(&audit)?)
            }
            _ => Err(TagisanError::Execution(format!(
                "Unsupported action '{action}' for copilot_bicep"
            ))),
        }
    }
}
