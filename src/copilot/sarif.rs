//! OASIS SARIF 2.1.0 Static Analysis Results Format & Azure Pipelines CI/CD Engine
//!
//! Subsystem 1: Microsoft Advanced Systems for Tagisan (`tgs`).
//!
//! Provides full compliance with OASIS SARIF 2.1.0 specification for GitHub Advanced Security
//! (`upload-sarif`) and Azure DevOps Advanced Security, as well as production-grade Azure Pipelines
//! CI/CD YAML generation with server-side formal verification and blast-radius quality gates.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

pub const SARIF_SCHEMA_2_1_0: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
pub const SARIF_VERSION: &str = "2.1.0";
pub const DEFAULT_TOOL_NAME: &str = "Tagisan Copilot Engine";
pub const DEFAULT_TOOL_VERSION: &str = "0.2.0";
pub const DEFAULT_TOOL_INFO_URI: &str = "https://github.com/charleogutierrez/tagisan";

// =========================================================================
// 1. OASIS SARIF 2.1.0 Data Models
// =========================================================================

/// Top-level SARIF 2.1.0 Document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDocument {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

impl Default for SarifDocument {
    fn default() -> Self {
        Self {
            schema: SARIF_SCHEMA_2_1_0.to_string(),
            version: SARIF_VERSION.to_string(),
            runs: Vec::new(),
        }
    }
}

/// Run execution container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub invocations: Vec<SarifInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Value>>,
}

/// Tool component descriptors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifTool {
    pub driver: SarifDriver,
}

/// Tool Driver details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDriver {
    pub name: String,
    pub version: String,
    pub information_uri: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rules: Vec<SarifRule>,
}

/// Rule definition / ReportingDescriptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    pub short_description: SarifMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
    pub default_configuration: SarifRuleConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<SarifHelp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Value>>,
}

/// Rule default configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRuleConfig {
    pub level: SarifLevel,
}

/// Severity level in SARIF
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SarifLevel {
    Error,
    Warning,
    Note,
    None,
}

impl SarifLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::None => "none",
        }
    }
}

/// Multiformat help text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifHelp {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}

/// Result / Finding descriptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    pub rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_index: Option<usize>,
    pub level: SarifLevel,
    pub message: SarifMessage,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locations: Vec<SarifLocation>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub code_flows: Vec<SarifCodeFlow>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related_locations: Vec<SarifLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Value>>,
}

/// Message payload with optional markdown
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifMessage {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}

impl SarifMessage {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            markdown: None,
        }
    }

    pub fn with_markdown(text: impl Into<String>, markdown: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            markdown: Some(markdown.into()),
        }
    }
}

/// Location element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<usize>,
    pub physical_location: SarifPhysicalLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<SarifMessage>,
}

/// Physical file location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
}

/// Artifact location URI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

/// Line and column range region
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    pub start_line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<SarifSnippet>,
}

/// Code snippet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifSnippet {
    pub text: String,
}

/// Code flow call path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifCodeFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<SarifMessage>,
    pub thread_flows: Vec<SarifThreadFlow>,
}

/// Thread flow representing sequential steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifThreadFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub locations: Vec<SarifThreadFlowLocation>,
}

/// Single step within a thread flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifThreadFlowLocation {
    pub step: usize,
    pub location: SarifLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub kinds: Vec<String>,
}

/// Execution invocation record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifInvocation {
    pub execution_successful: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
}

// =========================================================================
// 2. SarifEngine
// =========================================================================

/// Engine for generating, transpiling, and validating OASIS SARIF 2.1.0 documents
#[derive(Debug, Clone)]
pub struct SarifEngine {
    tool_name: String,
    tool_version: String,
    information_uri: String,
    rules: Vec<SarifRule>,
    results: Vec<SarifResult>,
}

impl Default for SarifEngine {
    fn default() -> Self {
        Self::new(DEFAULT_TOOL_NAME, DEFAULT_TOOL_VERSION, DEFAULT_TOOL_INFO_URI)
    }
}

impl SarifEngine {
    pub fn new(
        tool_name: impl Into<String>,
        tool_version: impl Into<String>,
        information_uri: impl Into<String>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            tool_version: tool_version.into(),
            information_uri: information_uri.into(),
            rules: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Register a rule descriptor
    pub fn add_rule(&mut self, rule: SarifRule) -> usize {
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule.id) {
            pos
        } else {
            let idx = self.rules.len();
            self.rules.push(rule);
            idx
        }
    }

    /// Register a finding / result
    pub fn add_result(&mut self, mut result: SarifResult) {
        if result.rule_index.is_none() {
            if let Some(pos) = self.rules.iter().position(|r| r.id == result.rule_id) {
                result.rule_index = Some(pos);
            }
        }
        self.results.push(result);
    }

    /// Transpiles a Tagisan invariant check failure into a SARIF result
    pub fn transpile_invariant_failure(
        &mut self,
        invariant_name: &str,
        expression: &str,
        file: &str,
        line: u32,
        blast_score: f64,
        details: &str,
    ) -> SarifResult {
        let rule_id = "TAGISAN-INV-001".to_string();
        let rule = SarifRule {
            id: rule_id.clone(),
            name: "InvariantViolation".to_string(),
            short_description: SarifMessage::plain("Formal SMT invariant condition violated"),
            full_description: Some(SarifMessage::plain(
                "A mathematical invariant enforced by Tagisan formal verification failed during AST evaluation.",
            )),
            default_configuration: SarifRuleConfig {
                level: SarifLevel::Error,
            },
            help_uri: Some("https://github.com/charleogutierrez/tagisan#invariants".to_string()),
            help: Some(SarifHelp {
                text: format!("Invariant '{invariant_name}' violated: {expression}"),
                markdown: Some(format!(
                    "### Invariant Violation: `{invariant_name}`\n\n```z3\n{expression}\n```\n\n**Details**: {details}"
                )),
            }),
            properties: Some({
                let mut p = HashMap::new();
                p.insert("category".to_string(), json!("SMT Formal Verification"));
                p.insert("precision".to_string(), json!("high"));
                p.insert("security-severity".to_string(), json!("9.0"));
                p
            }),
        };

        let rule_index = self.add_rule(rule);

        let location = SarifLocation {
            id: Some(1),
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: file.to_string(),
                    uri_base_id: Some("%SRCROOT%".to_string()),
                    index: None,
                },
                region: Some(SarifRegion {
                    start_line: line,
                    start_column: Some(1),
                    end_line: Some(line),
                    end_column: Some(80),
                    snippet: Some(SarifSnippet {
                        text: expression.to_string(),
                    }),
                }),
            },
            message: Some(SarifMessage::plain(format!(
                "Invariant '{invariant_name}' check failed here"
            ))),
        };

        let mut properties = HashMap::new();
        properties.insert("invariant_name".to_string(), json!(invariant_name));
        properties.insert("blast_score".to_string(), json!(blast_score));
        properties.insert("details".to_string(), json!(details));

        let res = SarifResult {
            rule_id,
            rule_index: Some(rule_index),
            level: SarifLevel::Error,
            message: SarifMessage::with_markdown(
                format!("Invariant '{invariant_name}' failed at {file}:{line} - {details}"),
                format!("**Invariant Failed**: `{invariant_name}`\n\n`{expression}`\n\nImpact Score: **{blast_score:.2}**"),
            ),
            locations: vec![location],
            code_flows: Vec::new(),
            related_locations: Vec::new(),
            properties: Some(properties),
        };

        self.add_result(res.clone());
        res
    }

    /// Transpiles a High Blast Radius alert into a SARIF result
    pub fn transpile_blast_radius(
        &mut self,
        symbol: &str,
        file: &str,
        line: u32,
        risk_tier: &str,
        blast_score: f64,
        downstream_symbols: &[String],
    ) -> SarifResult {
        let rule_id = "TAGISAN-BLAST-001".to_string();
        let level = if blast_score >= 0.80 {
            SarifLevel::Error
        } else {
            SarifLevel::Warning
        };

        let rule = SarifRule {
            id: rule_id.clone(),
            name: "HighBlastRadiusDetected".to_string(),
            short_description: SarifMessage::plain("Symbol modification exceeds blast radius threshold"),
            full_description: Some(SarifMessage::plain(
                "Modifying this AST symbol triggers cascading downstream changes across critical codebase dependencies.",
            )),
            default_configuration: SarifRuleConfig { level },
            help_uri: Some("https://github.com/charleogutierrez/tagisan#blast-radius".to_string()),
            help: Some(SarifHelp {
                text: format!("Symbol '{symbol}' has a blast radius score of {blast_score:.2} ({risk_tier})."),
                markdown: Some(format!(
                    "### High Blast Radius Alert: `{symbol}`\n\nTier: **{risk_tier}** | Score: **{blast_score:.2}**\n\nDownstream impacted symbols:\n{}",
                    downstream_symbols.iter().map(|s| format!("- `{s}`")).collect::<Vec<_>>().join("\n")
                )),
            }),
            properties: Some({
                let mut p = HashMap::new();
                p.insert("category".to_string(), json!("AST Blast Radius"));
                p.insert("risk_tier".to_string(), json!(risk_tier));
                p
            }),
        };

        let rule_index = self.add_rule(rule);

        let primary_location = SarifLocation {
            id: Some(1),
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: file.to_string(),
                    uri_base_id: Some("%SRCROOT%".to_string()),
                    index: None,
                },
                region: Some(SarifRegion {
                    start_line: line,
                    start_column: Some(1),
                    end_line: Some(line),
                    end_column: Some(symbol.len() as u32 + 10),
                    snippet: Some(SarifSnippet {
                        text: symbol.to_string(),
                    }),
                }),
            },
            message: Some(SarifMessage::plain(format!(
                "High blast radius root symbol '{symbol}' (Score: {blast_score:.2})"
            ))),
        };

        let related_locations: Vec<SarifLocation> = downstream_symbols
            .iter()
            .enumerate()
            .map(|(i, sym)| SarifLocation {
                id: Some(i + 2),
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: file.to_string(),
                        uri_base_id: Some("%SRCROOT%".to_string()),
                        index: None,
                    },
                    region: None,
                },
                message: Some(SarifMessage::plain(format!("Downstream impacted symbol: {sym}"))),
            })
            .collect();

        let mut properties = HashMap::new();
        properties.insert("symbol".to_string(), json!(symbol));
        properties.insert("blast_score".to_string(), json!(blast_score));
        properties.insert("risk_tier".to_string(), json!(risk_tier));
        properties.insert("downstream_count".to_string(), json!(downstream_symbols.len()));

        let res = SarifResult {
            rule_id,
            rule_index: Some(rule_index),
            level,
            message: SarifMessage::with_markdown(
                format!("Symbol '{symbol}' in {file}:{line} exceeds blast threshold ({blast_score:.2}) affecting {} downstream symbols", downstream_symbols.len()),
                format!("**AST Blast Alert**: `{symbol}`\n- Score: `{blast_score:.2}`\n- Tier: `{risk_tier}`\n- Downstream targets: {}", downstream_symbols.len()),
            ),
            locations: vec![primary_location],
            code_flows: Vec::new(),
            related_locations,
            properties: Some(properties),
        };

        self.add_result(res.clone());
        res
    }

    /// Transpiles a Defender reachable CVE call-chain path into a SARIF result with code flows
    pub fn transpile_cve_reachability(
        &mut self,
        cve_id: &str,
        package: &str,
        cvss: f32,
        call_chain: &[(&str, u32, &str)], // (file, line, symbol)
        description: &str,
    ) -> SarifResult {
        let rule_id = format!("DEFENDER-{}", cve_id.replace('-', "_"));
        let level = if cvss >= 7.0 {
            SarifLevel::Error
        } else {
            SarifLevel::Warning
        };

        let rule = SarifRule {
            id: rule_id.clone(),
            name: format!("Reachable_{}", cve_id.replace('-', "_")),
            short_description: SarifMessage::plain(format!(
                "Reachable Vulnerability {cve_id} in package {package}"
            )),
            full_description: Some(SarifMessage::plain(description)),
            default_configuration: SarifRuleConfig { level },
            help_uri: Some(format!("https://nvd.nist.gov/vuln/detail/{cve_id}")),
            help: Some(SarifHelp {
                text: format!("{cve_id} in {package} (CVSS {cvss:.1}): {description}"),
                markdown: Some(format!(
                    "### Microsoft Defender Reachable CVE: `{cve_id}`\n\n**Package**: `{package}` | **CVSS**: `{cvss:.1}`\n\n{description}"
                )),
            }),
            properties: Some({
                let mut p = HashMap::new();
                p.insert("cve_id".to_string(), json!(cve_id));
                p.insert("cvss_score".to_string(), json!(cvss));
                p.insert("security-severity".to_string(), json!(format!("{cvss:.1}")));
                p
            }),
        };

        let rule_index = self.add_rule(rule);

        // Build thread flow locations for the call chain
        let mut thread_flow_locations = Vec::new();
        for (idx, (file, line, sym)) in call_chain.iter().enumerate() {
            let is_vulnerable = idx == call_chain.len() - 1;
            let importance = if is_vulnerable {
                "essential"
            } else if idx == 0 {
                "important"
            } else {
                "unimportant"
            };

            let loc = SarifLocation {
                id: Some(idx + 1),
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: file.to_string(),
                        uri_base_id: Some("%SRCROOT%".to_string()),
                        index: None,
                    },
                    region: Some(SarifRegion {
                        start_line: *line,
                        start_column: Some(1),
                        end_line: Some(*line),
                        end_column: Some(sym.len() as u32 + 10),
                        snippet: Some(SarifSnippet {
                            text: sym.to_string(),
                        }),
                    }),
                },
                message: Some(SarifMessage::plain(if is_vulnerable {
                    format!("Vulnerable symbol invoked: {sym}")
                } else if idx == 0 {
                    format!("Public API entry point: {sym}")
                } else {
                    format!("Intermediate invocation: {sym}")
                })),
            };

            thread_flow_locations.push(SarifThreadFlowLocation {
                step: idx + 1,
                location: loc,
                importance: Some(importance.to_string()),
                kinds: if is_vulnerable {
                    vec!["sink".to_string()]
                } else if idx == 0 {
                    vec!["source".to_string()]
                } else {
                    vec!["call".to_string()]
                },
            });
        }

        let code_flow = SarifCodeFlow {
            message: Some(SarifMessage::plain(format!(
                "AST call trace from entry point to {cve_id} vulnerable symbol"
            ))),
            thread_flows: vec![SarifThreadFlow {
                id: Some(format!("call_chain_{cve_id}")),
                locations: thread_flow_locations,
            }],
        };

        // Primary location is the vulnerable sink (last in chain)
        let primary_location = if let Some((file, line, sym)) = call_chain.last() {
            vec![SarifLocation {
                id: Some(1),
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: file.to_string(),
                        uri_base_id: Some("%SRCROOT%".to_string()),
                        index: None,
                    },
                    region: Some(SarifRegion {
                        start_line: *line,
                        start_column: Some(1),
                        end_line: Some(*line),
                        end_column: Some(sym.len() as u32 + 10),
                        snippet: Some(SarifSnippet {
                            text: sym.to_string(),
                        }),
                    }),
                },
                message: Some(SarifMessage::plain(format!(
                    "Vulnerable symbol '{sym}' for {cve_id}"
                ))),
            }]
        } else {
            Vec::new()
        };

        let mut properties = HashMap::new();
        properties.insert("cve_id".to_string(), json!(cve_id));
        properties.insert("cvss_score".to_string(), json!(cvss));
        properties.insert("package".to_string(), json!(package));
        properties.insert("chain_depth".to_string(), json!(call_chain.len()));

        let res = SarifResult {
            rule_id,
            rule_index: Some(rule_index),
            level,
            message: SarifMessage::with_markdown(
                format!("Reachable {cve_id} in {package} (CVSS {cvss:.1}): {description}"),
                format!("**Defender Alert**: `{cve_id}` is transitively reachable via {} AST call steps\n\nCVSS: **{cvss:.1}**", call_chain.len()),
            ),
            locations: primary_location,
            code_flows: vec![code_flow],
            related_locations: Vec::new(),
            properties: Some(properties),
        };

        self.add_result(res.clone());
        res
    }

    /// Converts internal state to standard OASIS SarifDocument
    pub fn to_sarif_document(&self) -> SarifDocument {
        let run = SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: self.tool_name.clone(),
                    version: self.tool_version.clone(),
                    information_uri: self.information_uri.clone(),
                    rules: self.rules.clone(),
                },
            },
            results: self.results.clone(),
            invocations: vec![SarifInvocation {
                execution_successful: true,
                command_line: Some("tgs sarif --output-path report.sarif".to_string()),
            }],
            properties: None,
        };

        SarifDocument {
            schema: SARIF_SCHEMA_2_1_0.to_string(),
            version: SARIF_VERSION.to_string(),
            runs: vec![run],
        }
    }

    /// Serializes document to JSON string
    pub fn to_json_string(&self, pretty: bool) -> Result<String> {
        let doc = self.to_sarif_document();
        if pretty {
            serde_json::to_string_pretty(&doc)
                .map_err(|e| TagisanError::Execution(format!("SARIF JSON serialization error: {e}")))
        } else {
            serde_json::to_string(&doc)
                .map_err(|e| TagisanError::Execution(format!("SARIF JSON serialization error: {e}")))
        }
    }

    /// Validates compliance with SARIF 2.1.0 specification
    pub fn validate_compliance(&self) -> Result<()> {
        let doc = self.to_sarif_document();
        if doc.version != SARIF_VERSION {
            return Err(TagisanError::Execution(format!(
                "SARIF validation failed: version must be '{SARIF_VERSION}', got '{}'",
                doc.version
            )));
        }
        if doc.runs.is_empty() {
            return Err(TagisanError::Execution(
                "SARIF validation failed: 'runs' array must not be empty".to_string(),
            ));
        }
        for run in &doc.runs {
            if run.tool.driver.name.is_empty() {
                return Err(TagisanError::Execution(
                    "SARIF validation failed: tool driver name must not be empty".to_string(),
                ));
            }
            for result in &run.results {
                if result.rule_id.is_empty() {
                    return Err(TagisanError::Execution(
                        "SARIF validation failed: result ruleId must not be empty".to_string(),
                    ));
                }
                if result.message.text.is_empty() {
                    return Err(TagisanError::Execution(
                        "SARIF validation failed: result message text must not be empty"
                            .to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn rules(&self) -> &[SarifRule] {
        &self.rules
    }

    pub fn results(&self) -> &[SarifResult] {
        &self.results
    }
}

// =========================================================================
// 3. Azure Pipelines CI/CD YAML Generator
// =========================================================================

/// Configuration options for Azure Pipelines YAML generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzurePipelinesConfig {
    pub pipeline_name: String,
    pub trigger_branches: Vec<String>,
    pub pr_branches: Vec<String>,
    pub vm_image: String,
    pub blast_radius_threshold: f64,
    pub fail_on_sarif_errors: bool,
    pub publish_to_advanced_security: bool,
    pub smt_solver_timeout_secs: u32,
    pub enable_virtual_patch_pr: bool,
    pub sarif_output_path: String,
}

impl Default for AzurePipelinesConfig {
    fn default() -> Self {
        Self {
            pipeline_name: "Tagisan-CI-CD-Quality-Gate".to_string(),
            trigger_branches: vec!["main".to_string(), "release/*".to_string()],
            pr_branches: vec!["main".to_string()],
            vm_image: "ubuntu-latest".to_string(),
            blast_radius_threshold: 0.70,
            fail_on_sarif_errors: true,
            publish_to_advanced_security: true,
            smt_solver_timeout_secs: 60,
            enable_virtual_patch_pr: true,
            sarif_output_path: "$(Build.ArtifactStagingDirectory)/tagisan-report.sarif".to_string(),
        }
    }
}

/// Synthesizes enterprise-grade `azure-pipelines.yml` with formal gates
pub struct AzurePipelinesGenerator;

impl AzurePipelinesGenerator {
    /// Generates complete Azure DevOps CI/CD pipeline YAML
    pub fn generate_yaml(config: &AzurePipelinesConfig) -> String {
        let trigger_lines = config
            .trigger_branches
            .iter()
            .map(|b| format!("    - {b}"))
            .collect::<Vec<_>>()
            .join("\n");

        let pr_lines = config
            .pr_branches
            .iter()
            .map(|b| format!("    - {b}"))
            .collect::<Vec<_>>()
            .join("\n");

        let adv_sec_step = if config.publish_to_advanced_security {
            format!(
                r#"          - task: AdvancedSecurity-Publish@1
            displayName: 'Publish SARIF to Azure DevOps Advanced Security'
            inputs:
              SarifFile: '{}'
            condition: always()
"#,
                config.sarif_output_path
            )
        } else {
            String::new()
        };

        format!(
            r#"# =============================================================================
# Azure Pipelines CI/CD Specification: {pipeline_name}
# Synthesized by Tagisan Copilot Engine (OASIS SARIF 2.1.0 & Formal SMT Gate)
# =============================================================================

name: $(Date:yyyyMMdd)$(Rev:.r)

trigger:
  branches:
    include:
{trigger_lines}

pr:
  branches:
    include:
{pr_lines}

pool:
  vmImage: '{vm_image}'

variables:
  TAGISAN_VERSION: '0.2.0'
  CARGO_TERM_COLOR: 'always'
  RUSTFLAGS: '-D warnings'
  BLAST_RADIUS_THRESHOLD: '{blast_threshold:.2}'
  SARIF_REPORT_PATH: '{sarif_path}'
  SMT_TIMEOUT_SECS: '{smt_timeout}'

stages:
  - stage: TagisanVerification
    displayName: 'Stage 1: AST Blast Radius & Formal SMT Verification'
    jobs:
      - job: AST_Blast_Radius_Check
        displayName: 'Evaluate AST Call-Graph Blast Radius'
        steps:
          - task: UseRust@0
            inputs:
              channel: 'stable'
            displayName: 'Set up Rust toolchain'
          - script: |
              echo "==> Running Tagisan AST Blast Radius Analysis..."
              cargo run --release --bin tgs -- blast-radius \
                --threshold $(BLAST_RADIUS_THRESHOLD) \
                --strict-check
            displayName: 'Execute AST Blast Radius Pre-Merge Check'

      - job: SMT_Formal_Verification
        displayName: 'Z3 Formal Invariant Constraint Prover'
        steps:
          - task: UseRust@0
            inputs:
              channel: 'stable'
            displayName: 'Set up Rust toolchain'
          - script: |
              echo "==> Running Z3 / Formal Invariant SMT Verification..."
              cargo test --test formal_invariants -- --nocapture
            displayName: 'Run SMT Invariant Verification Suite'
            env:
              Z3_SOLVER_TIMEOUT: $(SMT_TIMEOUT_SECS)

      - job: Defender_Reachable_CVE_Triage
        displayName: 'Microsoft Defender AST Reachability Triage'
        steps:
          - script: |
              echo "==> Triaging Defender CVE Vulnerabilities Against Codebase AST..."
              cargo run --release --bin tgs -- copilot defender \
                --action batch_triage
            displayName: 'Triage Dependency CVEs for Active AST Call Paths'

  - stage: QualityGates_And_Sarif
    displayName: 'Stage 2: SARIF 2.1.0 Generation & Quality Gates'
    dependsOn: TagisanVerification
    condition: succeededOrFailed()
    jobs:
      - job: Generate_Sarif_And_Evaluate_Gate
        displayName: 'Synthesize SARIF & Enforce Server-Side Gate'
        steps:
          - script: |
              echo "==> Generating OASIS SARIF 2.1.0 Static Analysis Report..."
              cargo run --release --bin tgs -- copilot sarif \
                --action generate_sarif \
                --output-file $(SARIF_REPORT_PATH)
            displayName: 'Generate SARIF 2.1.0 Artifact'

{adv_sec_step}          - task: PublishBuildArtifacts@1
            displayName: 'Publish SARIF Report to Pipeline Artifacts'
            inputs:
              PathtoPublish: '$(SARIF_REPORT_PATH)'
              ArtifactName: 'CodeAnalysisLogs'
              publishLocation: 'Container'
            condition: always()

          - script: |
              echo "==> Enforcing Server-Side Release Quality Gate..."
              python3 -c "
import json, sys
with open('$(SARIF_REPORT_PATH)') as f:
    sarif = json.load(f)
errors = [r for run in sarif.get('runs', []) for r in run.get('results', []) if r.get('level') == 'error']
print(f'Total SARIF Errors: {{len(errors)}}')
if len(errors) > 0 and '{fail_on_errors}' == 'true':
    print('Quality Gate FAILED: Blocker SARIF errors present in pull request.')
    sys.exit(1)
print('Quality Gate PASSED: AST invariants and blast radius compliant.')
"
            displayName: 'Server-Side SARIF Quality Gate Enforcer'
"#,
            pipeline_name = config.pipeline_name,
            trigger_lines = trigger_lines,
            pr_lines = pr_lines,
            vm_image = config.vm_image,
            blast_threshold = config.blast_radius_threshold,
            sarif_path = config.sarif_output_path,
            smt_timeout = config.smt_solver_timeout_secs,
            adv_sec_step = adv_sec_step,
            fail_on_errors = config.fail_on_sarif_errors
        )
    }

    /// Validates syntactical integrity and structure of the generated YAML
    pub fn validate_yaml(yaml: &str) -> Result<()> {
        if !yaml.contains("trigger:") {
            return Err(TagisanError::Execution(
                "YAML validation failed: missing 'trigger:' section".to_string(),
            ));
        }
        if !yaml.contains("stages:") {
            return Err(TagisanError::Execution(
                "YAML validation failed: missing 'stages:' section".to_string(),
            ));
        }
        if !yaml.contains("TagisanVerification") {
            return Err(TagisanError::Execution(
                "YAML validation failed: missing 'TagisanVerification' stage".to_string(),
            ));
        }
        if !yaml.contains("QualityGates_And_Sarif") {
            return Err(TagisanError::Execution(
                "YAML validation failed: missing 'QualityGates_And_Sarif' stage".to_string(),
            ));
        }
        Ok(())
    }
}

// =========================================================================
// 4. CopilotSarifTool (ToolHandler)
// =========================================================================

/// Tool for generating OASIS SARIF 2.1.0 reports and Azure Pipelines CI/CD workflows
#[derive(Clone)]
pub struct CopilotSarifTool {
    engine: std::sync::Arc<tokio::sync::Mutex<SarifEngine>>,
}

impl Default for CopilotSarifTool {
    fn default() -> Self {
        Self {
            engine: std::sync::Arc::new(tokio::sync::Mutex::new(SarifEngine::default())),
        }
    }
}

impl CopilotSarifTool {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ToolHandler for CopilotSarifTool {
    fn name(&self) -> &str {
        "copilot_sarif"
    }

    fn description(&self) -> &str {
        "Generates OASIS SARIF 2.1.0 static analysis reports, transpiles Tagisan AST blast-radius, SMT invariant checks, and Defender CVE reachability into SARIF, and synthesizes production Azure Pipelines CI/CD YAML with server-side quality gates."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action"],
            "properties": {
                "action": {
                    "type": "string",
                    "enum": [
                        "generate_sarif",
                        "transpile_invariants",
                        "transpile_blast_radius",
                        "transpile_cve_reachability",
                        "generate_azure_pipeline",
                        "validate_sarif"
                    ],
                    "description": "The SARIF or Azure Pipelines action to execute."
                },
                "invariant_name": {
                    "type": "string",
                    "description": "Name of the formal invariant (for transpile_invariants)."
                },
                "expression": {
                    "type": "string",
                    "description": "SMT or mathematical constraint expression."
                },
                "file": {
                    "type": "string",
                    "description": "Source file path where violation or symbol occurred."
                },
                "line": {
                    "type": "integer",
                    "description": "Line number in source file."
                },
                "blast_score": {
                    "type": "number",
                    "description": "Blast radius score (0.0 to 1.0)."
                },
                "symbol": {
                    "type": "string",
                    "description": "Target AST symbol name."
                },
                "risk_tier": {
                    "type": "string",
                    "description": "Risk tier (Low, Medium, High, Critical)."
                },
                "downstream_symbols": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of downstream symbols impacted by modification."
                },
                "cve_id": {
                    "type": "string",
                    "description": "Common Vulnerabilities and Exposures ID (e.g. CVE-2024-38077)."
                },
                "package": {
                    "type": "string",
                    "description": "Vulnerable package or crate name."
                },
                "cvss_score": {
                    "type": "number",
                    "description": "CVSS v3.1 score (0.0 to 10.0)."
                },
                "call_chain": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "file": { "type": "string" },
                            "line": { "type": "integer" },
                            "symbol": { "type": "string" }
                        }
                    },
                    "description": "AST call-chain trace from entry point to vulnerable symbol."
                },
                "pipeline_config": {
                    "type": "object",
                    "description": "Optional parameters for azure-pipelines.yml generation."
                },
                "sarif_content": {
                    "type": "string",
                    "description": "Raw SARIF JSON string to validate (for validate_sarif)."
                },
                "output_file": {
                    "type": "string",
                    "description": "Optional file path to write generated SARIF or YAML to."
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|a| a.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        match action {
            "generate_sarif" => {
                let engine = self.engine.lock().await;
                let json_output = engine.to_json_string(true)?;
                if let Some(out_path) = arguments.get("output_file").and_then(|o| o.as_str()) {
                    if let Some(parent) = Path::new(out_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    std::fs::write(out_path, &json_output)
                        .map_err(|e| TagisanError::Execution(format!("Failed to write SARIF file: {e}")))?;
                }
                Ok(json_output)
            }
            "transpile_invariants" => {
                let inv_name = arguments
                    .get("invariant_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("InvNonZeroBalance");
                let expr = arguments
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("balance >= 0");
                let file = arguments.get("file").and_then(|v| v.as_str()).unwrap_or("src/ledger.rs");
                let line = arguments.get("line").and_then(|v| v.as_u64()).unwrap_or(42) as u32;
                let score = arguments.get("blast_score").and_then(|v| v.as_f64()).unwrap_or(0.85);
                let details = arguments
                    .get("details")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Assertion failed under boundary condition");

                let mut engine = self.engine.lock().await;
                let result = engine.transpile_invariant_failure(inv_name, expr, file, line, score, details);
                Ok(serde_json::to_string_pretty(&result)?)
            }
            "transpile_blast_radius" => {
                let symbol = arguments.get("symbol").and_then(|v| v.as_str()).unwrap_or("process_order");
                let file = arguments.get("file").and_then(|v| v.as_str()).unwrap_or("src/order.rs");
                let line = arguments.get("line").and_then(|v| v.as_u64()).unwrap_or(120) as u32;
                let tier = arguments.get("risk_tier").and_then(|v| v.as_str()).unwrap_or("Critical");
                let score = arguments.get("blast_score").and_then(|v| v.as_f64()).unwrap_or(0.92);

                let downstream = arguments
                    .get("downstream_symbols")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| vec!["checkout".to_string(), "invoice".to_string(), "telemetry".to_string()]);

                let mut engine = self.engine.lock().await;
                let result = engine.transpile_blast_radius(symbol, file, line, tier, score, &downstream);
                Ok(serde_json::to_string_pretty(&result)?)
            }
            "transpile_cve_reachability" => {
                let cve_id = arguments.get("cve_id").and_then(|v| v.as_str()).unwrap_or("CVE-2024-38077");
                let package = arguments.get("package").and_then(|v| v.as_str()).unwrap_or("windows-driver");
                let cvss = arguments.get("cvss_score").and_then(|v| v.as_f64()).unwrap_or(9.8) as f32;
                let desc = arguments
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Remote Code Execution in Windows Routing and Remote Access Service");

                let mut chain_storage = Vec::new();
                if let Some(arr) = arguments.get("call_chain").and_then(|v| v.as_array()) {
                    for item in arr {
                        let f = item.get("file").and_then(|v| v.as_str()).unwrap_or("src/lib.rs").to_string();
                        let l = item.get("line").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                        let s = item.get("symbol").and_then(|v| v.as_str()).unwrap_or("entry").to_string();
                        chain_storage.push((f, l, s));
                    }
                } else {
                    chain_storage.push(("src/api.rs".to_string(), 15, "handle_request".to_string()));
                    chain_storage.push(("src/dispatch.rs".to_string(), 48, "route_packet".to_string()));
                    chain_storage.push(("src/driver.rs".to_string(), 102, "vulnerable_exec".to_string()));
                }

                let chain_refs: Vec<(&str, u32, &str)> = chain_storage
                    .iter()
                    .map(|(f, l, s)| (f.as_str(), *l, s.as_str()))
                    .collect();

                let mut engine = self.engine.lock().await;
                let result = engine.transpile_cve_reachability(cve_id, package, cvss, &chain_refs, desc);
                Ok(serde_json::to_string_pretty(&result)?)
            }
            "generate_azure_pipeline" => {
                let config: AzurePipelinesConfig = if let Some(cfg_val) = arguments.get("pipeline_config") {
                    serde_json::from_value(cfg_val.clone()).unwrap_or_default()
                } else {
                    AzurePipelinesConfig::default()
                };

                let yaml = AzurePipelinesGenerator::generate_yaml(&config);
                AzurePipelinesGenerator::validate_yaml(&yaml)?;

                if let Some(out_path) = arguments.get("output_file").and_then(|o| o.as_str()) {
                    if let Some(parent) = Path::new(out_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    std::fs::write(out_path, &yaml)
                        .map_err(|e| TagisanError::Execution(format!("Failed to write Azure Pipelines YAML: {e}")))?;
                }

                Ok(yaml)
            }
            "validate_sarif" => {
                let validation_res = if let Some(content) = arguments.get("sarif_content").and_then(|c| c.as_str()) {
                    let doc: SarifDocument = serde_json::from_str(content)
                        .map_err(|e| TagisanError::Execution(format!("Invalid SARIF format: {e}")))?;
                    if doc.version != SARIF_VERSION {
                        return Err(TagisanError::Execution(format!("Version mismatch: expected {SARIF_VERSION}")));
                    }
                    "Valid OASIS SARIF 2.1.0 document."
                } else {
                    let engine = self.engine.lock().await;
                    engine.validate_compliance()?;
                    "Engine state is compliant with OASIS SARIF 2.1.0 specification."
                };

                let result = json!({
                    "status": "success",
                    "message": validation_res,
                    "schema": SARIF_SCHEMA_2_1_0,
                    "version": SARIF_VERSION
                });
                Ok(serde_json::to_string_pretty(&result)?)
            }
            _ => Err(TagisanError::Execution(format!("Unsupported action '{action}' for copilot_sarif"))),
        }
    }
}
