//! Microsoft 365 Copilot Plugin, Declarative Agent Manifest & OpenAPI 3.0 Specification Generator
//!
//! Generates:
//! - `ai-plugin.json` for Copilot Studio / OpenAI Plugin compatibility
//! - `declarativeAgent.json` conforming to Microsoft 365 Copilot Declarative Agent schema
//! - `openapi.json` exposing Tagisan's dialectical debate, agentic execution, grounding, and blast radius
//! - Teams App Package `manifest.json`
//! - `export_copilot_package()` bundling manifests, specs, and valid app icon binaries.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Package export summary report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotPackageInfo {
    pub output_dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub total_bytes: usize,
}

/// Generates a valid ai-plugin.json for Microsoft Copilot Studio and ChatGPT Plugins
pub fn generate_ai_plugin_json(base_url: &str) -> Value {
    let clean_base = base_url.trim_end_matches('/');
    json!({
        "schema_version": "v1",
        "name_for_human": "Tagisan Multi-LLM Copilot",
        "name_for_model": "tagisan",
        "description_for_human": "Dialectical AI debate, autonomous coding agents, invariant verification, and enterprise intelligence.",
        "description_for_model": "Tagisan engine for multi-agent adversarial debate, formal invariant grounding, codebase graph blast radius, and autonomous engineering.",
        "auth": {
            "type": "none"
        },
        "api": {
            "type": "openapi",
            "url": format!("{clean_base}/openapi.json")
        },
        "logo_url": format!("{clean_base}/color.png"),
        "contact_email": "copilot-support@tagisan.ai",
        "legal_info_url": "https://tagisan.ai/legal"
    })
}

/// Generates Microsoft 365 Copilot Declarative Agent manifest (declarativeAgent.json)
pub fn generate_declarative_agent_manifest(_base_url: &str) -> Value {
    json!({
        "$schema": "https://developer.microsoft.com/json-schemas/copilot/declarative-agent/v1.0/schema.json",
        "version": "v1.0",
        "name": "Tagisan Enterprise Copilot",
        "description": "Multi-LLM dialectical debate, autonomous coding agent, formal invariant verification, and Microsoft Graph knowledge grounding.",
        "instructions": "You are the Tagisan Enterprise Copilot assistant. You orchestrate multi-model dialectical debates, verify mathematical invariants with tgs ground, compute symbol blast radius across codebases, synthesize actionable engineering patches from Teams meetings, and generate executive telemetry Adaptive Cards.",
        "conversation_starters": [
            { "text": "Debate Rust vs Go for high-throughput distributed consensus" },
            { "text": "Extract engineering action items from our latest Teams meeting" },
            { "text": "Verify formal invariants for our authentication service" },
            { "text": "Calculate the blast radius of refactoring EntraAuthManager" },
            { "text": "Execute meeting-to-code pipeline on our latest sprint sync transcript" },
            { "text": "Generate Teams Adaptive Card for blast radius of refactoring EntraAuthManager" },
            { "text": "Dispatch 3-round dialectical debate on microservices RFC to Teams" },
            { "text": "Enforce Microsoft Purview Zero-Egress Air-Gapping on confidential payload" },
            { "text": "Synthesize Architectural Decision Record (ADR) and sync to OneNote & SharePoint" },
            { "text": "Create ephemeral Git branch and Azure DevOps PR with blast telemetry" },
            { "text": "Compile responsive executive briefing slide deck for leadership" }
        ],
        "actions": [
            {
                "id": "tagisanActions",
                "file": "openapi.json"
            }
        ],
        "capabilities": [
            { "name": "WebSearch" },
            { "name": "OneDriveAndSharePoint" }
        ]
    })
}

/// Generates Microsoft Teams App Package Manifest (manifest.json)
pub fn generate_teams_app_manifest(_base_url: &str) -> Value {
    json!({
        "$schema": "https://developer.microsoft.com/en-us/json-schemas/teams/v1.16/MicrosoftTeams.schema.json",
        "manifestVersion": "1.16",
        "version": "1.0.0",
        "id": "a981c20e-6f8d-4a11-8a4b-tagisan36501",
        "packageName": "com.tagisan.copilot",
        "developer": {
            "name": "Tagisan AI",
            "websiteUrl": "https://tagisan.ai",
            "privacyUrl": "https://tagisan.ai/privacy",
            "termsOfUseUrl": "https://tagisan.ai/terms"
        },
        "icons": {
            "color": "color.png",
            "outline": "outline.png"
        },
        "name": {
            "short": "Tagisan Copilot",
            "full": "Tagisan Multi-LLM Engineering Copilot"
        },
        "description": {
            "short": "Dialectical AI debate and code verification",
            "full": "Production-grade multi-model dialectical debate, autonomous coding agents, formal invariant verification, and Copilot tools."
        },
        "accentColor": "#0078D4",
        "copilotAgents": {
            "declarativeAgents": [
                {
                    "id": "tagisanDeclarativeAgent",
                    "file": "declarativeAgent.json"
                }
            ]
        }
    })
}

/// Generates OpenAPI 3.0.3 specification exposing Tagisan's core capabilities
pub fn generate_openapi_spec(base_url: &str) -> Value {
    let clean_base = base_url.trim_end_matches('/');
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Tagisan Copilot API",
            "version": "0.2.0",
            "description": "REST API exposing Tagisan Dialectical Debate, Autonomous Agent, Invariant Grounding, and Blast Radius Calculation"
        },
        "servers": [
            {
                "url": clean_base,
                "description": "Tagisan Engine Gateway"
            }
        ],
        "paths": {
            "/api/debate": {
                "post": {
                    "operationId": "runDialecticalDebate",
                    "summary": "Execute 3-round dialectical debate between models (Thesis, Adversarial Critique, Lakandiwa Synthesis)",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "prompt": {
                                            "type": "string",
                                            "description": "The technical decision, architecture question, or bug hypothesis"
                                        },
                                        "proponent": { "type": "string", "description": "Proponent model name" },
                                        "adversary": { "type": "string", "description": "Adversary model name" },
                                        "lakandiwa": { "type": "string", "description": "Lakandiwa judge model name" }
                                    },
                                    "required": ["prompt"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Debate synthesis and verdict",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "final_answer": { "type": "string" },
                                            "total_cost_usd": { "type": "number" },
                                            "total_latency_seconds": { "type": "number" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/agent": {
                "post": {
                    "operationId": "runAutonomousAgent",
                    "summary": "Execute autonomous multi-turn ReAct agent with sandboxed tools and vector memory",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "prompt": { "type": "string", "description": "The objective or task to complete" },
                                        "memory": { "type": "boolean", "description": "Whether to auto-inject codebase context" }
                                    },
                                    "required": ["prompt"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Agent execution result",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "result": { "type": "string" },
                                            "steps_executed": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/ground": {
                "post": {
                    "operationId": "verifyFormalGrounding",
                    "summary": "Run invariant verification and formal grounding check on code or architecture (tgs ground)",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "target": { "type": "string", "description": "Target module or subsystem" },
                                        "code": { "type": "string", "description": "Code snippet or specification text" },
                                        "invariants": {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "description": "List of formal invariants to check"
                                        }
                                    },
                                    "required": ["target"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Invariant verification report",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "passed": { "type": "boolean" },
                                            "stage": { "type": "string" },
                                            "findings": { "type": "array", "items": { "type": "string" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/graph/blast-radius": {
                "post": {
                    "operationId": "calculateBlastRadius",
                    "summary": "Calculate blast radius and transitive dependencies for modifying a code symbol",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "symbol": { "type": "string", "description": "Symbol name (function, struct, method)" },
                                        "max_depth": { "type": "integer", "description": "Max graph traversal depth (default: 3)" }
                                    },
                                    "required": ["symbol"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Blast radius report",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "symbol": { "type": "string" },
                                            "risk_level": { "type": "string" },
                                            "impacted_symbols_count": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/action-items": {
                "post": {
                    "operationId": "extractMeetingActionItems",
                    "summary": "Extract engineering action items and priority assignments from meeting transcripts",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "meeting_id": { "type": "string" },
                                        "transcript": {
                                            "type": "array",
                                            "items": {
                                                "type": "object",
                                                "properties": {
                                                    "speaker": { "type": "string" },
                                                    "text": { "type": "string" }
                                                }
                                            }
                                        }
                                    },
                                    "required": ["transcript"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Extracted action items",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "action_items": { "type": "array", "items": { "type": "object" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/meeting-to-code": {
                "post": {
                    "operationId": "runMeetingToCodePipeline",
                    "summary": "Execute end-to-end meeting-to-code pipeline: extract action items, compute AST blast radius, synthesize code patches with AgentShield verification",
                    "requestBody": {
                        "required": false,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "meeting_id": { "type": "string" },
                                        "transcript_text": { "type": "string" },
                                        "codebase_path": { "type": "string" },
                                        "auto_patch": { "type": "boolean" },
                                        "channel": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Meeting-to-code execution report and patch proposals",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "pipeline_report": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/blast-radius-report": {
                "post": {
                    "operationId": "generateBlastRadiusReportCard",
                    "summary": "Generate executive codebase telemetry and Adaptive Card / HTML report for blast radius refactoring risk",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "symbol": { "type": "string" },
                                        "max_depth": { "type": "integer" },
                                        "format": { "type": "string" },
                                        "post_to_teams": { "type": "string" },
                                        "export_email": { "type": "string" }
                                    },
                                    "required": ["symbol"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Blast radius telemetry report and Adaptive Card",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "symbol": { "type": "string" },
                                            "risk_level": { "type": "string" },
                                            "adaptive_card": { "type": "object" },
                                            "html_report": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/debate-dispatch": {
                "post": {
                    "operationId": "dispatchDialecticalDebate",
                    "summary": "Run 3-round dialectical debate (Thesis, Antithesis, Lakandiwa Synthesis) on an RFC proposal and dispatch verdict to Teams or Outlook",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "proposal": { "type": "string" },
                                        "title": { "type": "string" },
                                        "proponent": { "type": "string" },
                                        "adversary": { "type": "string" },
                                        "lakandiwa": { "type": "string" },
                                        "post_to_teams": { "type": "string" },
                                        "send_to_email": { "type": "string" }
                                    },
                                    "required": ["proposal"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Dialectical debate synthesis and dispatch status",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "verdict": { "type": "string" },
                                            "full_transcript": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/purview-guard": {
                "post": {
                    "operationId": "enforcePurviewZeroEgress",
                    "summary": "Classify Microsoft Purview sensitivity labels, enforce Zero-Egress Air-Gapping, and issue SHA-256 audit receipts",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "content": { "type": "string" },
                                        "label": { "type": "string" },
                                        "destination": { "type": "string" }
                                    },
                                    "required": ["content"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Purview evaluation result and cryptographic receipt",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "sensitivity": { "type": "string" },
                                            "air_gapped": { "type": "boolean" },
                                            "routing_engine": { "type": "string" },
                                            "receipt": { "type": "object" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/adr-sync": {
                "post": {
                    "operationId": "syncArchitectureDecisionRecord",
                    "summary": "Synthesize dialectical debate verdicts into MADR Architecture Decision Records and sync to OneNote & SharePoint",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "proposal": { "type": "string" },
                                        "verdict": { "type": "string" },
                                        "title": { "type": "string" },
                                        "invariants": { "type": "array", "items": { "type": "string" } },
                                        "onenote_section": { "type": "string" },
                                        "sharepoint_folder": { "type": "string" }
                                    },
                                    "required": ["proposal"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "ADR synthesis report and sync identifiers",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "adr_id": { "type": "string" },
                                            "onenote_page_id": { "type": "string" },
                                            "sharepoint_item_id": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/create-pr": {
                "post": {
                    "operationId": "createAutonomousPullRequest",
                    "summary": "Create ephemeral Git branch, generate conventional commit, format PR with blast telemetry, and notify Teams",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "patch": { "type": "string" },
                                        "title": { "type": "string" },
                                        "branch_name": { "type": "string" },
                                        "target_platform": { "type": "string" },
                                        "symbol": { "type": "string" },
                                        "post_to_teams": { "type": "string" }
                                    },
                                    "required": ["patch"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Created Pull Request report and Adaptive Card telemetry",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "pr_url": { "type": "string" },
                                            "branch_name": { "type": "string" },
                                            "commit_message": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/copilot/export-deck": {
                "post": {
                    "operationId": "generateExecutiveBriefingDeck",
                    "summary": "Compile 5-slide executive presentation briefing deck (HTML & Markdown) with KPI counters and blast hotspots",
                    "requestBody": {
                        "required": false,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "title": { "type": "string" },
                                        "format": { "type": "string" },
                                        "output_path": { "type": "string" },
                                        "export_email": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Executive briefing slide deck",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "title": { "type": "string" },
                                            "slides_count": { "type": "integer" },
                                            "html_deck": { "type": "string" },
                                            "markdown_deck": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}


/// Create valid minimal PNG binary bytes with proper PNG signature, IHDR, IDAT (zlib), and IEND chunks.
/// Generates a valid uncompressed single-color RGBA image for Microsoft 365 icon compliance.
pub fn generate_valid_png(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
    let mut png = Vec::new();

    // 1. PNG Signature (8 bytes)
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // 2. IHDR Chunk (13 bytes)
    let mut ihdr_data = Vec::with_capacity(13);
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8); // Bit depth: 8
    ihdr_data.push(6); // Color type: 6 (RGBA)
    ihdr_data.push(0); // Compression method: 0 (deflate)
    ihdr_data.push(0); // Filter method: 0 (standard)
    ihdr_data.push(0); // Interlace method: 0 (none)
    write_png_chunk(&mut png, b"IHDR", &ihdr_data);

    // 3. IDAT Chunk: raw uncompressed scanlines wrapped in zlib container
    // Each row has 1 filter byte (0x00) followed by `width * 4` RGBA bytes
    let row_len = 1 + (width as usize * 4);
    let raw_data_len = row_len * height as usize;
    let mut raw_scanlines = Vec::with_capacity(raw_data_len);
    for _ in 0..height {
        raw_scanlines.push(0); // Filter byte 0 (None)
        for _ in 0..width {
            raw_scanlines.extend_from_slice(&[r, g, b, a]);
        }
    }

    // Wrap raw scanlines in zlib stream (RFC 1950)
    let mut zlib_stream = Vec::new();
    zlib_stream.push(0x78); // CMF: Deflate, 32K window
    zlib_stream.push(0x01); // FLG: No preset dictionary, check bits

    // Deflate uncompressed blocks (BTYPE = 00) in chunks of up to 65535 bytes
    let chunks = raw_scanlines.chunks(65535);
    let chunk_count = (raw_scanlines.len() + 65534) / 65535;
    for (i, chunk) in chunks.enumerate() {
        let is_last = (i + 1) == chunk_count;
        let bfinal = if is_last { 0x01 } else { 0x00 };
        zlib_stream.push(bfinal);
        let len = chunk.len() as u16;
        let nlen = !len;
        zlib_stream.extend_from_slice(&len.to_le_bytes());
        zlib_stream.extend_from_slice(&nlen.to_le_bytes());
        zlib_stream.extend_from_slice(chunk);
    }

    // Adler-32 checksum
    let adler = compute_adler32(&raw_scanlines);
    zlib_stream.extend_from_slice(&adler.to_be_bytes());

    write_png_chunk(&mut png, b"IDAT", &zlib_stream);

    // 4. IEND Chunk
    write_png_chunk(&mut png, b"IEND", &[]);

    png
}

fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);

    // CRC-32 over chunk_type and data
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    let crc = compute_crc32(&crc_input);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn compute_adler32(data: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &byte in data {
        s1 = (s1 + byte as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = -( (crc & 1) as i32 ) as u32;
            crc = (crc >> 1) ^ (0xEDB88320 & mask);
        }
    }
    !crc
}

/// Export complete Copilot package bundle to target directory
pub fn export_copilot_package(output_dir: &Path, base_url: &str) -> Result<CopilotPackageInfo> {
    std::fs::create_dir_all(output_dir)?;

    let mut files = Vec::new();
    let mut total_bytes = 0;

    // 1. ai-plugin.json
    let ai_plugin = generate_ai_plugin_json(base_url);
    let ai_plugin_path = output_dir.join("ai-plugin.json");
    let ai_plugin_bytes = serde_json::to_vec_pretty(&ai_plugin)?;
    std::fs::write(&ai_plugin_path, &ai_plugin_bytes)?;
    total_bytes += ai_plugin_bytes.len();
    files.push(ai_plugin_path);

    // 2. declarativeAgent.json
    let decl_agent = generate_declarative_agent_manifest(base_url);
    let decl_agent_path = output_dir.join("declarativeAgent.json");
    let decl_agent_bytes = serde_json::to_vec_pretty(&decl_agent)?;
    std::fs::write(&decl_agent_path, &decl_agent_bytes)?;
    total_bytes += decl_agent_bytes.len();
    files.push(decl_agent_path);

    // 3. manifest.json
    let teams_manifest = generate_teams_app_manifest(base_url);
    let teams_manifest_path = output_dir.join("manifest.json");
    let teams_manifest_bytes = serde_json::to_vec_pretty(&teams_manifest)?;
    std::fs::write(&teams_manifest_path, &teams_manifest_bytes)?;
    total_bytes += teams_manifest_bytes.len();
    files.push(teams_manifest_path);

    // 4. openapi.json
    let openapi = generate_openapi_spec(base_url);
    let openapi_path = output_dir.join("openapi.json");
    let openapi_bytes = serde_json::to_vec_pretty(&openapi)?;
    std::fs::write(&openapi_path, &openapi_bytes)?;
    total_bytes += openapi_bytes.len();
    files.push(openapi_path);

    // 5. color.png (192x192 RGBA - Tagisan Deep Cobalt/Cyan)
    let color_png_bytes = generate_valid_png(192, 192, 0, 120, 212, 255);
    let color_png_path = output_dir.join("color.png");
    std::fs::write(&color_png_path, &color_png_bytes)?;
    total_bytes += color_png_bytes.len();
    files.push(color_png_path);

    // 6. outline.png (32x32 RGBA - Transparent Monochrome)
    let outline_png_bytes = generate_valid_png(32, 32, 255, 255, 255, 255);
    let outline_png_path = output_dir.join("outline.png");
    std::fs::write(&outline_png_path, &outline_png_bytes)?;
    total_bytes += outline_png_bytes.len();
    files.push(outline_png_path);

    Ok(CopilotPackageInfo {
        output_dir: output_dir.to_path_buf(),
        files,
        total_bytes,
    })
}
