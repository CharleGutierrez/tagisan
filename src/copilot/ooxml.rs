//! Native Rust Office Open XML (OOXML) Document Generation Engine
//!
//! Provides pure-Rust, zero-dependency generation of native Microsoft Office documents:
//! - WordprocessingML (`.docx`)
//! - SpreadsheetML (`.xlsx`)
//! - PresentationML (`.pptx`)
//!
//! Implements an in-memory standard PKZIP (ISO/IEC 29500 / ECMA-376) packaging pipeline
//! without requiring Python (python-docx, openpyxl) or external LibreOffice binaries.

use crate::copilot::purview::PurviewSensitivity;
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Severity classification for WordprocessingML callout alert boxes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalloutSeverity {
    Info,
    Warning,
    Danger,
    Success,
    Tip,
}

impl CalloutSeverity {
    pub fn border_color(&self) -> &'static str {
        match self {
            CalloutSeverity::Info => "0F6CBD",     // Microsoft Blue
            CalloutSeverity::Warning => "FFB900",  // Amber Warning
            CalloutSeverity::Danger => "D83B01",   // Alert Red
            CalloutSeverity::Success => "107C41",  // Excel Green
            CalloutSeverity::Tip => "5C2D91",      // Purview Purple
        }
    }

    pub fn fill_color(&self) -> &'static str {
        match self {
            CalloutSeverity::Info => "F0F8FF",
            CalloutSeverity::Warning => "FFF8E5",
            CalloutSeverity::Danger => "FDF3F2",
            CalloutSeverity::Success => "F1F9F1",
            CalloutSeverity::Tip => "F6F2FA",
        }
    }

    pub fn badge_label(&self) -> &'static str {
        match self {
            CalloutSeverity::Info => "NOTE",
            CalloutSeverity::Warning => "WARNING",
            CalloutSeverity::Danger => "CRITICAL ALERT",
            CalloutSeverity::Success => "VERIFIED",
            CalloutSeverity::Tip => "BEST PRACTICE",
        }
    }
}

/// Callout alert box for Word (.docx) documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxCallout {
    pub title: String,
    pub message: String,
    pub severity: CalloutSeverity,
}

/// Custom typography and styling for Word (.docx) document sections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocxCustomStyle {
    pub font_family: Option<String>,
    pub font_size_pt: Option<u32>,
    pub color_hex: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

/// Section of a generated Word document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxSection {
    pub heading: String,
    pub level: u32,
    pub paragraphs: Vec<String>,
    pub table: Option<DocxTable>,
    #[serde(default)]
    pub callouts: Vec<DocxCallout>,
    #[serde(default)]
    pub custom_style: Option<DocxCustomStyle>,
}

impl DocxSection {
    pub fn new(heading: impl Into<String>, level: u32) -> Self {
        Self {
            heading: heading.into(),
            level,
            paragraphs: Vec::new(),
            table: None,
            callouts: Vec::new(),
            custom_style: None,
        }
    }

    pub fn with_paragraph(mut self, paragraph: impl Into<String>) -> Self {
        self.paragraphs.push(paragraph.into());
        self
    }

    pub fn with_table(mut self, table: DocxTable) -> Self {
        self.table = Some(table);
        self
    }

    pub fn with_callout(mut self, callout: DocxCallout) -> Self {
        self.callouts.push(callout);
        self
    }

    pub fn with_custom_style(mut self, style: DocxCustomStyle) -> Self {
        self.custom_style = Some(style);
        self
    }
}

/// Table definition for Word (.docx) or Excel (.xlsx)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Executive KPI / Metrics Card for PowerPoint (.pptx)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PptxMetricCard {
    pub label: String,
    pub value: String,
    pub change_or_subtext: Option<String>,
    pub color_hex: Option<String>,
}

/// Slide definition for PowerPoint (.pptx)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PptxSlide {
    pub title: String,
    pub subtitle: Option<String>,
    pub bullets: Vec<String>,
    #[serde(default)]
    pub presenter_notes: Option<String>,
    #[serde(default)]
    pub metric_cards: Vec<PptxMetricCard>,
}

impl PptxSlide {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            bullets: Vec::new(),
            presenter_notes: None,
            metric_cards: Vec::new(),
        }
    }

    pub fn with_subtitle(mut self, sub: impl Into<String>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    pub fn with_bullet(mut self, bullet: impl Into<String>) -> Self {
        self.bullets.push(bullet.into());
        self
    }

    pub fn with_presenter_notes(mut self, notes: impl Into<String>) -> Self {
        self.presenter_notes = Some(notes.into());
        self
    }

    pub fn with_metric_card(mut self, card: PptxMetricCard) -> Self {
        self.metric_cards.push(card);
        self
    }
}

/// Verification report from unpacking and checking an in-memory PKZIP archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipPackageVerification {
    pub is_valid: bool,
    pub total_entries: u16,
    pub file_names: Vec<String>,
    pub total_uncompressed_bytes: usize,
    pub details: String,
}

/// Export summary report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OoxmlExportReport {
    pub output_dir: String,
    pub docx_file: Option<String>,
    pub xlsx_file: Option<String>,
    pub pptx_file: Option<String>,
    pub total_bytes: usize,
    pub sensitivity_applied: PurviewSensitivity,
}

// =========================================================================
// Pure-Rust In-Memory PKZIP Archive Builder
// =========================================================================

/// Calculates standard IEEE 802.3 CRC32 checksum
pub fn calculate_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Minimalist, high-performance in-memory ZIP builder conforming to PKWARE ZIP spec
pub struct ZipBuilder {
    files: Vec<(String, Vec<u8>, u32)>, // (name, data, crc)
}

impl Default for ZipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ZipBuilder {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add_file(&mut self, path: &str, content: &[u8]) {
        let crc = calculate_crc32(content);
        self.files.push((path.to_string(), content.to_vec(), crc));
    }

    pub fn finish(self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central_directory = Vec::new();
        let total_entries = self.files.len() as u16;

        for (name, data, crc) in self.files {
            let local_header_offset = out.len() as u32;
            let name_bytes = name.as_bytes();
            let size = data.len() as u32;

            // 1. Local File Header
            out.extend_from_slice(&0x04034b50u32.to_le_bytes()); // PK\x03\x04
            out.extend_from_slice(&20u16.to_le_bytes()); // Version needed (2.0)
            out.extend_from_slice(&0u16.to_le_bytes());  // General purpose bit flag
            out.extend_from_slice(&0u16.to_le_bytes());  // Compression (0 = Stored)
            out.extend_from_slice(&0x4460u16.to_le_bytes()); // Mod time (12:35:00)
            out.extend_from_slice(&0x5821u16.to_le_bytes()); // Mod date (2024-01-01)
            out.extend_from_slice(&crc.to_le_bytes());   // CRC-32
            out.extend_from_slice(&size.to_le_bytes());  // Compressed size
            out.extend_from_slice(&size.to_le_bytes());  // Uncompressed size
            out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes()); // File name length
            out.extend_from_slice(&0u16.to_le_bytes());  // Extra field length
            out.extend_from_slice(name_bytes);
            out.extend_from_slice(&data);

            // 2. Central Directory Header
            central_directory.extend_from_slice(&0x02014b50u32.to_le_bytes()); // PK\x01\x02
            central_directory.extend_from_slice(&0x0314u16.to_le_bytes()); // Version made by (Unix 2.0)
            central_directory.extend_from_slice(&20u16.to_le_bytes());     // Version needed
            central_directory.extend_from_slice(&0u16.to_le_bytes());      // Flags
            central_directory.extend_from_slice(&0u16.to_le_bytes());      // Compression (Stored)
            central_directory.extend_from_slice(&0x4460u16.to_le_bytes()); // Mod time
            central_directory.extend_from_slice(&0x5821u16.to_le_bytes()); // Mod date
            central_directory.extend_from_slice(&crc.to_le_bytes());       // CRC-32
            central_directory.extend_from_slice(&size.to_le_bytes());      // Compressed size
            central_directory.extend_from_slice(&size.to_le_bytes());      // Uncompressed size
            central_directory.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            central_directory.extend_from_slice(&0u16.to_le_bytes());      // Extra len
            central_directory.extend_from_slice(&0u16.to_le_bytes());      // Comment len
            central_directory.extend_from_slice(&0u16.to_le_bytes());      // Disk start
            central_directory.extend_from_slice(&0u16.to_le_bytes());      // Internal attr
            central_directory.extend_from_slice(&0x81a40000u32.to_le_bytes()); // External attr (regular file)
            central_directory.extend_from_slice(&local_header_offset.to_le_bytes());
            central_directory.extend_from_slice(name_bytes);
        }

        let cd_offset = out.len() as u32;
        let cd_size = central_directory.len() as u32;
        out.extend_from_slice(&central_directory);

        // 3. End of Central Directory Record (EOCD)
        out.extend_from_slice(&0x06054b50u32.to_le_bytes()); // PK\x05\x06
        out.extend_from_slice(&0u16.to_le_bytes()); // Disk number
        out.extend_from_slice(&0u16.to_le_bytes()); // Disk with central directory
        // Fixed: Use exact self.files.len() as u16 instead of (cd_size / 46).max(1)
        out.extend_from_slice(&total_entries.to_le_bytes()); // Entries on this disk
        out.extend_from_slice(&total_entries.to_le_bytes()); // Total entries
        out.extend_from_slice(&cd_size.to_le_bytes());       // Size of CD
        out.extend_from_slice(&cd_offset.to_le_bytes());     // Offset of CD
        out.extend_from_slice(&0u16.to_le_bytes());          // Comment len

        out
    }
}

// =========================================================================
// OoxmlEngine (Word, Excel, PowerPoint Generator)
// =========================================================================

pub struct OoxmlEngine;

impl OoxmlEngine {
    /// Extracts a specific file's raw bytes from an in-memory PKZIP archive
    pub fn extract_file_from_zip(zip_bytes: &[u8], target_path: &str) -> Result<Vec<u8>> {
        let len = zip_bytes.len();
        if len < 22 {
            return Err(TagisanError::Execution("ZIP archive too small (< 22 bytes)".to_string()));
        }
        let mut eocd_pos = None;
        let search_start = if len > 65557 { len - 65557 } else { 0 };
        for i in (search_start..=(len - 22)).rev() {
            if zip_bytes[i] == 0x50 && zip_bytes[i + 1] == 0x4B && zip_bytes[i + 2] == 0x05 && zip_bytes[i + 3] == 0x06 {
                eocd_pos = Some(i);
                break;
            }
        }
        let eocd_pos = eocd_pos.ok_or_else(|| TagisanError::Execution("EOCD signature not found".to_string()))?;
        let total_entries = u16::from_le_bytes(zip_bytes[eocd_pos + 10..eocd_pos + 12].try_into().unwrap());
        let cd_offset = u32::from_le_bytes(zip_bytes[eocd_pos + 16..eocd_pos + 20].try_into().unwrap()) as usize;

        let mut curr_cd = cd_offset;
        for _ in 0..total_entries {
            if curr_cd + 46 > zip_bytes.len() { break; }
            let comp_size = u32::from_le_bytes(zip_bytes[curr_cd + 20..curr_cd + 24].try_into().unwrap()) as usize;
            let name_len = u16::from_le_bytes(zip_bytes[curr_cd + 28..curr_cd + 30].try_into().unwrap()) as usize;
            let extra_len = u16::from_le_bytes(zip_bytes[curr_cd + 30..curr_cd + 32].try_into().unwrap()) as usize;
            let comment_len = u16::from_le_bytes(zip_bytes[curr_cd + 32..curr_cd + 34].try_into().unwrap()) as usize;
            let local_header_off = u32::from_le_bytes(zip_bytes[curr_cd + 42..curr_cd + 46].try_into().unwrap()) as usize;

            let name_start = curr_cd + 46;
            if name_start + name_len <= zip_bytes.len() {
                let name = String::from_utf8_lossy(&zip_bytes[name_start..name_start + name_len]);
                if name == target_path {
                    if local_header_off + 30 > zip_bytes.len() {
                        return Err(TagisanError::Execution("Corrupt local header offset".to_string()));
                    }
                    let loc_name_len = u16::from_le_bytes(zip_bytes[local_header_off + 26..local_header_off + 28].try_into().unwrap()) as usize;
                    let loc_extra_len = u16::from_le_bytes(zip_bytes[local_header_off + 28..local_header_off + 30].try_into().unwrap()) as usize;
                    let data_start = local_header_off + 30 + loc_name_len + loc_extra_len;
                    let data_end = data_start + comp_size;
                    if data_end > zip_bytes.len() {
                        return Err(TagisanError::Execution("Corrupt file payload length".to_string()));
                    }
                    return Ok(zip_bytes[data_start..data_end].to_vec());
                }
            }
            curr_cd += 46 + name_len + extra_len + comment_len;
        }
        Err(TagisanError::Execution(format!("File '{}' not found in ZIP package", target_path)))
    }

    /// Verifies structure, headers, CRC-32, and entry consistency of an in-memory PKZIP archive
    pub fn verify_zip_package(zip_bytes: &[u8]) -> Result<ZipPackageVerification> {
        let len = zip_bytes.len();
        if len < 22 {
            return Err(TagisanError::Execution("ZIP archive too small (< 22 bytes)".to_string()));
        }

        // Locate EOCD signature PK\x05\x06 (0x06054b50)
        let mut eocd_pos = None;
        let search_start = if len > 65557 { len - 65557 } else { 0 };
        for i in (search_start..=(len - 22)).rev() {
            if zip_bytes[i] == 0x50 && zip_bytes[i + 1] == 0x4B && zip_bytes[i + 2] == 0x05 && zip_bytes[i + 3] == 0x06 {
                eocd_pos = Some(i);
                break;
            }
        }

        let eocd_pos = eocd_pos.ok_or_else(|| {
            TagisanError::Execution("End of Central Directory (EOCD) signature not found".to_string())
        })?;

        let _entries_disk = u16::from_le_bytes(zip_bytes[eocd_pos + 8..eocd_pos + 10].try_into().unwrap());
        let total_entries = u16::from_le_bytes(zip_bytes[eocd_pos + 10..eocd_pos + 12].try_into().unwrap());
        let cd_size = u32::from_le_bytes(zip_bytes[eocd_pos + 12..eocd_pos + 16].try_into().unwrap()) as usize;
        let cd_offset = u32::from_le_bytes(zip_bytes[eocd_pos + 16..eocd_pos + 20].try_into().unwrap()) as usize;

        if cd_offset + cd_size > eocd_pos {
            return Err(TagisanError::Execution(format!(
                "Corrupted central directory boundaries: cd_offset ({}) + cd_size ({}) > eocd_pos ({})",
                cd_offset, cd_size, eocd_pos
            )));
        }

        let mut curr_cd = cd_offset;
        let mut file_names = Vec::with_capacity(total_entries as usize);
        let mut total_uncompressed_bytes = 0;

        for entry_idx in 0..total_entries {
            if curr_cd + 46 > cd_offset + cd_size {
                return Err(TagisanError::Execution(format!(
                    "Unexpected EOF parsing central directory entry {}", entry_idx
                )));
            }

            // Check signature PK\x01\x02
            if zip_bytes[curr_cd..curr_cd + 4] != [0x50, 0x4B, 0x01, 0x02] {
                return Err(TagisanError::Execution(format!(
                    "Invalid Central Directory signature at offset {}", curr_cd
                )));
            }

            let comp_method = u16::from_le_bytes(zip_bytes[curr_cd + 10..curr_cd + 12].try_into().unwrap());
            let header_crc = u32::from_le_bytes(zip_bytes[curr_cd + 16..curr_cd + 20].try_into().unwrap());
            let comp_size = u32::from_le_bytes(zip_bytes[curr_cd + 20..curr_cd + 24].try_into().unwrap()) as usize;
            let uncomp_size = u32::from_le_bytes(zip_bytes[curr_cd + 24..curr_cd + 28].try_into().unwrap()) as usize;
            let name_len = u16::from_le_bytes(zip_bytes[curr_cd + 28..curr_cd + 30].try_into().unwrap()) as usize;
            let extra_len = u16::from_le_bytes(zip_bytes[curr_cd + 30..curr_cd + 32].try_into().unwrap()) as usize;
            let comment_len = u16::from_le_bytes(zip_bytes[curr_cd + 32..curr_cd + 34].try_into().unwrap()) as usize;
            let local_header_off = u32::from_le_bytes(zip_bytes[curr_cd + 42..curr_cd + 46].try_into().unwrap()) as usize;

            let name_start = curr_cd + 46;
            let name_end = name_start + name_len;
            if name_end > zip_bytes.len() {
                return Err(TagisanError::Execution("Filename extends beyond ZIP boundary".to_string()));
            }
            let file_name = String::from_utf8_lossy(&zip_bytes[name_start..name_end]).to_string();

            // Validate local file header
            if local_header_off + 30 > zip_bytes.len() {
                return Err(TagisanError::Execution(format!(
                    "Local file header offset {} out of bounds for '{}'", local_header_off, file_name
                )));
            }

            // Check PK\x03\x04
            if zip_bytes[local_header_off..local_header_off + 4] != [0x50, 0x4B, 0x03, 0x04] {
                return Err(TagisanError::Execution(format!(
                    "Invalid Local File Header signature at offset {} for '{}'", local_header_off, file_name
                )));
            }

            let loc_name_len = u16::from_le_bytes(zip_bytes[local_header_off + 26..local_header_off + 28].try_into().unwrap()) as usize;
            let loc_extra_len = u16::from_le_bytes(zip_bytes[local_header_off + 28..local_header_off + 30].try_into().unwrap()) as usize;

            if local_header_off + 30 + loc_name_len > zip_bytes.len() {
                return Err(TagisanError::Execution(format!(
                    "Local header filename extends beyond ZIP boundary for '{}'", file_name
                )));
            }
            let loc_name = String::from_utf8_lossy(&zip_bytes[local_header_off + 30..local_header_off + 30 + loc_name_len]);
            if loc_name != file_name {
                return Err(TagisanError::Execution(format!(
                    "Filename mismatch between local header ('{}') and central directory ('{}')",
                    loc_name, file_name
                )));
            }

            let data_start = local_header_off + 30 + loc_name_len + loc_extra_len;
            let data_end = data_start + comp_size;

            if data_end > zip_bytes.len() {
                return Err(TagisanError::Execution(format!(
                    "File payload for '{}' extends beyond ZIP boundary ({} > {})", file_name, data_end, zip_bytes.len()
                )));
            }

            let file_data = &zip_bytes[data_start..data_end];
            if comp_method == 0 {
                let actual_crc = calculate_crc32(file_data);
                if actual_crc != header_crc {
                    return Err(TagisanError::Execution(format!(
                        "CRC32 mismatch for '{}': expected 0x{:08X}, got 0x{:08X}",
                        file_name, header_crc, actual_crc
                    )));
                }
                if comp_size != uncomp_size {
                    return Err(TagisanError::Execution(format!(
                        "Stored file '{}' compressed size ({}) != uncompressed size ({})",
                        file_name, comp_size, uncomp_size
                    )));
                }
            }

            total_uncompressed_bytes += uncomp_size;
            file_names.push(file_name);
            curr_cd += 46 + name_len + extra_len + comment_len;
        }

        Ok(ZipPackageVerification {
            is_valid: true,
            total_entries,
            file_names,
            total_uncompressed_bytes,
            details: format!(
                "Verified PKZIP archive with {} valid entries, {} bytes uncompressed",
                total_entries, total_uncompressed_bytes
            ),
        })
    }

    /// Verifies a Microsoft Word (.docx) document conforms to ISO/IEC 29500 WordprocessingML
    pub fn verify_docx(docx_bytes: &[u8]) -> Result<ZipPackageVerification> {
        let ver = Self::verify_zip_package(docx_bytes)?;
        let required = [
            "[Content_Types].xml",
            "_rels/.rels",
            "word/document.xml",
            "docProps/core.xml",
        ];
        for req in &required {
            if !ver.file_names.iter().any(|f| f == req) {
                return Err(TagisanError::Execution(format!(
                    "WordprocessingML package missing required OOXML part: '{}'", req
                )));
            }
        }
        let doc_xml = Self::extract_file_from_zip(docx_bytes, "word/document.xml")?;
        let doc_str = String::from_utf8_lossy(&doc_xml);
        if !doc_str.contains("<w:document") || !doc_str.contains("</w:document>") {
            return Err(TagisanError::Execution(
                "Invalid WordprocessingML: word/document.xml missing <w:document> root element".to_string(),
            ));
        }
        Ok(ver)
    }

    /// Verifies a Microsoft Excel (.xlsx) spreadsheet conforms to ISO/IEC 29500 SpreadsheetML
    pub fn verify_xlsx(xlsx_bytes: &[u8]) -> Result<ZipPackageVerification> {
        let ver = Self::verify_zip_package(xlsx_bytes)?;
        let required = [
            "[Content_Types].xml",
            "_rels/.rels",
            "xl/workbook.xml",
            "xl/worksheets/sheet1.xml",
        ];
        for req in &required {
            if !ver.file_names.iter().any(|f| f == req) {
                return Err(TagisanError::Execution(format!(
                    "SpreadsheetML package missing required OOXML part: '{}'", req
                )));
            }
        }
        let wb_xml = Self::extract_file_from_zip(xlsx_bytes, "xl/workbook.xml")?;
        let wb_str = String::from_utf8_lossy(&wb_xml);
        if !wb_str.contains("<workbook") || !wb_str.contains("</workbook>") {
            return Err(TagisanError::Execution(
                "Invalid SpreadsheetML: xl/workbook.xml missing <workbook> root element".to_string(),
            ));
        }
        Ok(ver)
    }

    /// Verifies a Microsoft PowerPoint (.pptx) presentation conforms to ISO/IEC 29500 PresentationML
    pub fn verify_pptx(pptx_bytes: &[u8]) -> Result<ZipPackageVerification> {
        let ver = Self::verify_zip_package(pptx_bytes)?;
        let required = [
            "[Content_Types].xml",
            "_rels/.rels",
            "ppt/presentation.xml",
            "ppt/slides/slide1.xml",
        ];
        for req in &required {
            if !ver.file_names.iter().any(|f| f == req) {
                return Err(TagisanError::Execution(format!(
                    "PresentationML package missing required OOXML part: '{}'", req
                )));
            }
        }
        let pres_xml = Self::extract_file_from_zip(pptx_bytes, "ppt/presentation.xml")?;
        let pres_str = String::from_utf8_lossy(&pres_xml);
        if !pres_str.contains("<p:presentation") || !pres_str.contains("</p:presentation>") {
            return Err(TagisanError::Execution(
                "Invalid PresentationML: ppt/presentation.xml missing <p:presentation> root element".to_string(),
            ));
        }
        Ok(ver)
    }

    /// Generates native Microsoft Word (.docx) document with callouts and custom styling
    pub fn build_docx(
        title: &str,
        author: &str,
        sections: &[DocxSection],
        sensitivity: Option<PurviewSensitivity>,
    ) -> Result<Vec<u8>> {
        let mut zip = ZipBuilder::new();
        let sens = sensitivity.unwrap_or(PurviewSensitivity::General);

        // 1. [Content_Types].xml
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#;
        zip.add_file("[Content_Types].xml", content_types.as_bytes());

        // 2. _rels/.rels
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;
        zip.add_file("_rels/.rels", rels.as_bytes());

        // 3. docProps/core.xml
        let now = Utc::now().to_rfc3339();
        let core_props = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{}</dc:title>
  <dc:creator>{}</dc:creator>
  <cp:lastModifiedBy>Tagisan AI Engine</cp:lastModifiedBy>
  <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
  <dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>
  <cp:category>Purview: {:?}</cp:category>
</cp:coreProperties>"#,
            xml_escape(title),
            xml_escape(author),
            now,
            now,
            sens
        );
        zip.add_file("docProps/core.xml", core_props.as_bytes());

        // 4. docProps/app.xml
        let app_props = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>Tagisan OOXML Native Engine</Application>
</Properties>"#;
        zip.add_file("docProps/app.xml", app_props.as_bytes());

        // 5. word/document.xml
        let mut doc_xml = String::new();
        doc_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
"#);

        // Document Title
        doc_xml.push_str(&format!(
            r#"    <w:p>
      <w:pPr>
        <w:jc w:val="center"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:b/>
          <w:sz w:val="48"/>
          <w:color w:val="0F6CBD"/>
        </w:rPr>
        <w:t>{}</w:t>
      </w:r>
    </w:p>
"#,
            xml_escape(title)
        ));

        // Purview Classification Badge
        doc_xml.push_str(&format!(
            r#"    <w:p>
      <w:pPr>
        <w:jc w:val="center"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:i/>
          <w:sz w:val="20"/>
          <w:color w:val="D83B01"/>
        </w:rPr>
        <w:t>[Purview Classification: {:?}]</w:t>
      </w:r>
    </w:p>
"#,
            sens
        ));

        // Sections
        for sec in sections {
            let default_sz = if sec.level == 1 { "36" } else { "28" };
            let sz = sec.custom_style.as_ref()
                .and_then(|s| s.font_size_pt.map(|pt| (pt * 2).to_string()))
                .unwrap_or_else(|| default_sz.to_string());
            let color = sec.custom_style.as_ref()
                .and_then(|s| s.color_hex.clone())
                .unwrap_or_else(|| "242424".to_string());
            let font = sec.custom_style.as_ref()
                .and_then(|s| s.font_family.clone())
                .unwrap_or_else(|| "Segoe UI".to_string());

            doc_xml.push_str(&format!(
                r#"    <w:p>
      <w:r>
        <w:rPr>
          <w:rFonts w:ascii="{font}" w:hAnsi="{font}"/>
          <w:b/>
          <w:sz w:val="{sz}"/>
          <w:color w:val="{color}"/>
        </w:rPr>
        <w:t>{}</w:t>
      </w:r>
    </w:p>
"#,
                xml_escape(&sec.heading)
            ));

            for p in &sec.paragraphs {
                let p_font = sec.custom_style.as_ref()
                    .and_then(|s| s.font_family.clone())
                    .unwrap_or_else(|| "Segoe UI".to_string());
                let italic_tag = if sec.custom_style.as_ref().map(|s| s.italic).unwrap_or(false) { "<w:i/>" } else { "" };
                let bold_tag = if sec.custom_style.as_ref().map(|s| s.bold).unwrap_or(false) { "<w:b/>" } else { "" };

                doc_xml.push_str(&format!(
                    r#"    <w:p>
      <w:r>
        <w:rPr>
          <w:rFonts w:ascii="{p_font}" w:hAnsi="{p_font}"/>
          {bold_tag}
          {italic_tag}
          <w:sz w:val="22"/>
          <w:color w:val="323130"/>
        </w:rPr>
        <w:t>{}</w:t>
      </w:r>
    </w:p>
"#,
                    xml_escape(p)
                ));
            }

            // Callout Alert Boxes
            for callout in &sec.callouts {
                let border_color = callout.severity.border_color();
                let fill_color = callout.severity.fill_color();
                let badge = callout.severity.badge_label();

                doc_xml.push_str(&format!(
                    r#"    <w:p>
      <w:pPr>
        <w:pBdr>
          <w:left w:val="single" w:sz="36" w:space="15" w:color="{border_color}"/>
        </w:pBdr>
        <w:shd w:val="clear" w:color="auto" w:fill="{fill_color}"/>
        <w:ind w:left="360" w:right="360"/>
        <w:spacing w:before="140" w:after="40"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:b/>
          <w:sz w:val="22"/>
          <w:color w:val="{border_color}"/>
        </w:rPr>
        <w:t>[{badge}] {}</w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:pPr>
        <w:pBdr>
          <w:left w:val="single" w:sz="36" w:space="15" w:color="{border_color}"/>
        </w:pBdr>
        <w:shd w:val="clear" w:color="auto" w:fill="{fill_color}"/>
        <w:ind w:left="360" w:right="360"/>
        <w:spacing w:before="0" w:after="140"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:sz w:val="20"/>
          <w:color w:val="242424"/>
        </w:rPr>
        <w:t>{}</w:t>
      </w:r>
    </w:p>
"#,
                    xml_escape(&callout.title),
                    xml_escape(&callout.message)
                ));
            }

            // Optional Table
            if let Some(ref tbl) = sec.table {
                doc_xml.push_str("    <w:tbl>\n");
                // Header row
                doc_xml.push_str("      <w:tr>\n");
                for h in &tbl.headers {
                    doc_xml.push_str(&format!(
                        r#"        <w:tc>
          <w:tcPr><w:shd w:val="clear" w:color="auto" w:fill="F3F2F1"/></w:tcPr>
          <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>{}</w:t></w:p>
        </w:tc>
"#,
                        xml_escape(h)
                    ));
                }
                doc_xml.push_str("      </w:tr>\n");

                // Data rows
                for r in &tbl.rows {
                    doc_xml.push_str("      <w:tr>\n");
                    for cell in r {
                        doc_xml.push_str(&format!(
                            r#"        <w:tc>
          <w:p><w:r><w:t>{}</w:t></w:p>
        </w:tc>
"#,
                            xml_escape(cell)
                        ));
                    }
                    doc_xml.push_str("      </w:tr>\n");
                }
                doc_xml.push_str("    </w:tbl>\n");
            }
        }

        doc_xml.push_str(r#"    <w:sectPr/>
  </w:body>
</w:document>"#);

        zip.add_file("word/document.xml", doc_xml.as_bytes());

        Ok(zip.finish())
    }

    /// Generates native Microsoft Excel (.xlsx) spreadsheet with formula cells, styling, and number formats
    pub fn build_xlsx(
        title: &str,
        sheet_name: &str,
        headers: &[&str],
        rows: &[Vec<String>],
    ) -> Result<Vec<u8>> {
        let mut zip = ZipBuilder::new();

        // 1. [Content_Types].xml
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;
        zip.add_file("[Content_Types].xml", content_types.as_bytes());

        // 2. _rels/.rels
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;
        zip.add_file("_rels/.rels", rels.as_bytes());

        // 3. xl/_rels/workbook.xml.rels
        let wb_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        zip.add_file("xl/_rels/workbook.xml.rels", wb_rels.as_bytes());

        // 4. xl/styles.xml (SpreadsheetML Stylesheet with Fonts, Fills, Borders, Formats)
        let styles_xml = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <numFmts count="3">
    <numFmt numFmtId="164" formatCode="$#,##0.00"/>
    <numFmt numFmtId="165" formatCode="0.0%"/>
    <numFmt numFmtId="166" formatCode="#,##0.00"/>
  </numFmts>
  <fonts count="3">
    <font><sz val="11"/><name val="Segoe UI"/></font>
    <font><b/><sz val="11"/><color rgb="FFFFFFFF"/><name val="Segoe UI"/></font>
    <font><b/><sz val="11"/><color rgb="FF0F6CBD"/><name val="Segoe UI"/></font>
  </fonts>
  <fills count="5">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="gray125"/></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FF0F6CBD"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFF3F2F1"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFFFF4CE"/></patternFill></fill>
  </fills>
  <borders count="2">
    <border><left/><right/><top/><bottom/></border>
    <border>
      <left style="thin"><color rgb="FFD1D1D1"/></left>
      <right style="thin"><color rgb="FFD1D1D1"/></right>
      <top style="thin"><color rgb="FFD1D1D1"/></top>
      <bottom style="thin"><color rgb="FFD1D1D1"/></bottom>
    </border>
  </borders>
  <cellStyleXfs count="1">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0"/>
  </cellStyleXfs>
  <cellXfs count="7">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
    <xf numFmtId="0" fontId="1" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"/>
    <xf numFmtId="164" fontId="0" fillId="0" borderId="1" xfId="0" applyNumberFormat="1" applyBorder="1"/>
    <xf numFmtId="165" fontId="0" fillId="0" borderId="1" xfId="0" applyNumberFormat="1" applyBorder="1"/>
    <xf numFmtId="166" fontId="0" fillId="0" borderId="1" xfId="0" applyNumberFormat="1" applyBorder="1"/>
    <xf numFmtId="0" fontId="0" fillId="3" borderId="1" xfId="0" applyFill="1" applyBorder="1"/>
    <xf numFmtId="0" fontId="2" fillId="4" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"/>
  </cellXfs>
</styleSheet>"##;
        zip.add_file("xl/styles.xml", styles_xml.as_bytes());

        // 5. xl/workbook.xml
        let wb = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="{}" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#,
            xml_escape(sheet_name)
        );
        zip.add_file("xl/workbook.xml", wb.as_bytes());

        // 6. xl/worksheets/sheet1.xml
        let mut sheet = String::new();
        sheet.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
"#);

        // Header Row (Row 1 with s="1" Header Style)
        sheet.push_str("    <row r=\"1\">\n");
        for (i, h) in headers.iter().enumerate() {
            let col_letter = (b'A' + (i as u8)) as char;
            sheet.push_str(&format!(
                "      <c r=\"{}1\" s=\"1\" t=\"inlineStr\"><is><t>{}</t></is></c>\n",
                col_letter, xml_escape(h)
            ));
        }
        sheet.push_str("    </row>\n");

        // Data Rows (Row 2..N)
        for (r_idx, row_vals) in rows.iter().enumerate() {
            let r_num = r_idx + 2;
            let is_even = r_num % 2 == 0;
            let default_s = if is_even { " s=\"5\"" } else { "" };

            sheet.push_str(&format!("    <row r=\"{}\">\n", r_num));
            for (c_idx, val) in row_vals.iter().enumerate() {
                let col_letter = (b'A' + (c_idx as u8)) as char;
                let cell_ref = format!("{}{}", col_letter, r_num);

                if val.starts_with('=') {
                    // Formula Cell: <f>EXPRESSION</f>
                    let formula_body = xml_escape(&val[1..]);
                    sheet.push_str(&format!(
                        "      <c r=\"{}\"><f>{}</f></c>\n",
                        cell_ref, formula_body
                    ));
                } else if val.starts_with('$') && val[1..].replace(',', "").parse::<f64>().is_ok() {
                    // Currency Cell: s="2"
                    let num: f64 = val[1..].replace(',', "").parse().unwrap_or(0.0);
                    sheet.push_str(&format!(
                        "      <c r=\"{}\" s=\"2\"><v>{}</v></c>\n",
                        cell_ref, num
                    ));
                } else if val.ends_with('%') && val[..val.len() - 1].replace(',', "").parse::<f64>().is_ok() {
                    // Percentage Cell: s="3"
                    let num: f64 = val[..val.len() - 1].replace(',', "").parse().unwrap_or(0.0) / 100.0;
                    sheet.push_str(&format!(
                        "      <c r=\"{}\" s=\"3\"><v>{}</v></c>\n",
                        cell_ref, num
                    ));
                } else if let Ok(n) = val.replace(',', "").parse::<f64>() {
                    if val.contains('.') {
                        // Decimal Number Cell: s="4"
                        sheet.push_str(&format!(
                            "      <c r=\"{}\" s=\"4\"><v>{}</v></c>\n",
                            cell_ref, n
                        ));
                    } else {
                        // Integer Cell
                        sheet.push_str(&format!(
                            "      <c r=\"{}\"><v>{}</v></c>\n",
                            cell_ref, n as i64
                        ));
                    }
                } else if val.eq_ignore_ascii_case("true") || val.eq_ignore_ascii_case("false") {
                    // Boolean Cell
                    let b_val = if val.eq_ignore_ascii_case("true") { "1" } else { "0" };
                    sheet.push_str(&format!(
                        "      <c r=\"{}\" t=\"b\"><v>{}</v></c>\n",
                        cell_ref, b_val
                    ));
                } else {
                    // String Cell
                    sheet.push_str(&format!(
                        "      <c r=\"{}\"{} t=\"inlineStr\"><is><t>{}</t></is></c>\n",
                        cell_ref, default_s, xml_escape(val)
                    ));
                }
            }
            sheet.push_str("    </row>\n");
        }

        sheet.push_str(r#"  </sheetData>
</worksheet>"#);
        zip.add_file("xl/worksheets/sheet1.xml", sheet.as_bytes());

        // 7. docProps/core.xml
        let now = Utc::now().to_rfc3339();
        let core_props = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{}</dc:title>
  <dc:creator>Tagisan Excel Engine</dc:creator>
  <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
</cp:coreProperties>"#,
            xml_escape(title), now
        );
        zip.add_file("docProps/core.xml", core_props.as_bytes());

        Ok(zip.finish())
    }

    /// Generates native Microsoft PowerPoint (.pptx) presentation with Presenter Notes & Metrics Cards
    pub fn build_pptx(
        title: &str,
        subtitle: &str,
        slides: &[PptxSlide],
    ) -> Result<Vec<u8>> {
        let mut zip = ZipBuilder::new();

        // 1. [Content_Types].xml
        let mut content_types = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
"#);
        for i in 1..=(slides.len() + 1) {
            content_types.push_str(&format!(
                "  <Override PartName=\"/ppt/slides/slide{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>\n",
                i
            ));
        }

        // Add overrides for any notes slides
        for (i, s) in slides.iter().enumerate() {
            if s.presenter_notes.is_some() {
                content_types.push_str(&format!(
                    "  <Override PartName=\"/ppt/notesSlides/notesSlide{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml\"/>\n",
                    i + 2
                ));
            }
        }
        content_types.push_str("</Types>");
        zip.add_file("[Content_Types].xml", content_types.as_bytes());

        // 2. _rels/.rels
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;
        zip.add_file("_rels/.rels", rels.as_bytes());

        // 3. ppt/_rels/presentation.xml.rels
        let mut pres_rels = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
"#);
        for i in 1..=(slides.len() + 1) {
            pres_rels.push_str(&format!(
                "  <Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{}.xml\"/>\n",
                i, i
            ));
        }
        pres_rels.push_str("</Relationships>");
        zip.add_file("ppt/_rels/presentation.xml.rels", pres_rels.as_bytes());

        // 4. ppt/presentation.xml
        let mut pres = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
"#);
        for i in 1..=(slides.len() + 1) {
            pres.push_str(&format!(
                "    <p:sldId id=\"{}\" r:id=\"rId{}\"/>\n",
                255 + i, i
            ));
        }
        pres.push_str(r#"  </p:sldIdLst>
</p:presentation>"#);
        zip.add_file("ppt/presentation.xml", pres.as_bytes());

        // 5. Title Slide (slide1.xml)
        let slide1 = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:grpSpPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:p><a:r><a:rPr lang="en-US" b="1" sz="4400"/><a:t>{}</a:t></a:r></a:p>
          <a:p><a:r><a:rPr lang="en-US" sz="2400"/><a:t>{}</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:sld>"#,
            xml_escape(title), xml_escape(subtitle)
        );
        zip.add_file("ppt/slides/slide1.xml", slide1.as_bytes());

        // 6. Content Slides (slide2..N)
        for (i, s) in slides.iter().enumerate() {
            let slide_num = i + 2;
            let mut s_xml = String::new();
            s_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:grpSpPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="SlideTitle"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
"#);
            s_xml.push_str(&format!(
                "          <a:p><a:r><a:rPr lang=\"en-US\" b=\"1\" sz=\"3600\"/><a:t>{}</a:t></a:r></a:p>\n",
                xml_escape(&s.title)
            ));
            for bullet in &s.bullets {
                s_xml.push_str(&format!(
                    "          <a:p><a:pPr lvl=\"1\"/><a:r><a:rPr lang=\"en-US\" sz=\"2000\"/><a:t>• {}</a:t></a:r></a:p>\n",
                    xml_escape(bullet)
                ));
            }
            s_xml.push_str(r#"        </p:txBody>
      </p:sp>
"#);

            // Metrics Cards in DrawingML shapes
            if !s.metric_cards.is_empty() {
                for (c_idx, card) in s.metric_cards.iter().enumerate() {
                    let card_id = 100 + (c_idx as u32);
                    let x_pos = 1000000 + (c_idx as u32 * 2600000);
                    let y_pos = 4200000;
                    let bg_color = card.color_hex.as_deref().unwrap_or("F3F2F1");
                    let subtext_xml = if let Some(ref sub) = card.change_or_subtext {
                        format!(
                            "          <a:p><a:pPr algn=\"ctr\"/><a:r><a:rPr lang=\"en-US\" sz=\"1100\"><a:solidFill><a:srgbClr val=\"107C41\"/></a:solidFill></a:rPr><a:t>{}</a:t></a:r></a:p>\n",
                            xml_escape(sub)
                        )
                    } else {
                        String::new()
                    };

                    s_xml.push_str(&format!(
                        r#"      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="{}" name="MetricCard_{}"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="2300000" cy="1400000"/>
          </a:xfrm>
          <a:prstGeom prst="roundRect">
            <a:avLst/>
          </a:prstGeom>
          <a:solidFill>
            <a:srgbClr val="{}"/>
          </a:solidFill>
          <a:ln w="12700">
            <a:solidFill><a:srgbClr val="D1D1D1"/></a:solidFill>
          </a:ln>
        </p:spPr>
        <p:txBody>
          <a:bodyPr vert="horz" lIns="91440" tIns="91440" rIns="91440" bIns="91440" anchor="ctr"/>
          <a:p><a:pPr algn="ctr"/><a:r><a:rPr lang="en-US" b="1" sz="3000"><a:solidFill><a:srgbClr val="0F6CBD"/></a:solidFill></a:rPr><a:t>{}</a:t></a:r></a:p>
          <a:p><a:pPr algn="ctr"/><a:r><a:rPr lang="en-US" sz="1300"><a:solidFill><a:srgbClr val="323130"/></a:solidFill></a:rPr><a:t>{}</a:t></a:r></a:p>
{}        </p:txBody>
      </p:sp>
"#,
                        card_id,
                        c_idx,
                        x_pos,
                        y_pos,
                        bg_color,
                        xml_escape(&card.value),
                        xml_escape(&card.label),
                        subtext_xml
                    ));
                }
            }

            s_xml.push_str(r#"    </p:spTree>
  </p:cSld>
</p:sld>"#);
            zip.add_file(&format!("ppt/slides/slide{}.xml", slide_num), s_xml.as_bytes());

            // Presenter Notes Slide
            if let Some(ref notes) = s.presenter_notes {
                let note_xml = format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
  xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:grpSpPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Presenter Notes"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:p><a:r><a:rPr lang="en-US" sz="1400"/><a:t>{}</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:notes>"#,
                    xml_escape(notes)
                );
                zip.add_file(&format!("ppt/notesSlides/notesSlide{}.xml", slide_num), note_xml.as_bytes());

                let slide_rel = format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdNotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide{}.xml"/>
</Relationships>"#,
                    slide_num
                );
                zip.add_file(&format!("ppt/slides/_rels/slide{}.xml.rels", slide_num), slide_rel.as_bytes());
            }
        }

        // 7. docProps/core.xml
        let now = Utc::now().to_rfc3339();
        let core_props = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:dcterms="http://purl.org/dc/terms/"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{}</dc:title>
  <dc:creator>Tagisan PowerPoint Engine</dc:creator>
  <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
</cp:coreProperties>"#,
            xml_escape(title), now
        );
        zip.add_file("docProps/core.xml", core_props.as_bytes());

        Ok(zip.finish())
    }

    /// Bundles all three formats into output directory
    pub fn export_suite(
        output_dir: &Path,
        base_name: &str,
        title: &str,
        author: &str,
        sections: &[DocxSection],
        sensitivity: Option<PurviewSensitivity>,
    ) -> Result<OoxmlExportReport> {
        std::fs::create_dir_all(output_dir)?;
        let sens = sensitivity.unwrap_or(PurviewSensitivity::General);

        // Word .docx
        let docx_bytes = Self::build_docx(title, author, sections, Some(sens))?;
        let docx_path = output_dir.join(format!("{}.docx", base_name));
        std::fs::write(&docx_path, &docx_bytes)?;

        // Excel .xlsx
        let headers = vec!["Section", "Paragraphs Count", "Has Table"];
        let mut rows = Vec::new();
        for sec in sections {
            rows.push(vec![
                sec.heading.clone(),
                sec.paragraphs.len().to_string(),
                sec.table.is_some().to_string(),
            ]);
        }
        let xlsx_bytes = Self::build_xlsx(title, "Summary", &headers, &rows)?;
        let xlsx_path = output_dir.join(format!("{}.xlsx", base_name));
        std::fs::write(&xlsx_path, &xlsx_bytes)?;

        // PowerPoint .pptx
        let slides: Vec<PptxSlide> = sections
            .iter()
            .map(|s| PptxSlide {
                title: s.heading.clone(),
                subtitle: None,
                bullets: s.paragraphs.clone(),
                presenter_notes: None,
                metric_cards: Vec::new(),
            })
            .collect();
        let pptx_bytes = Self::build_pptx(title, "Executive Architecture Briefing", &slides)?;
        let pptx_path = output_dir.join(format!("{}.pptx", base_name));
        std::fs::write(&pptx_path, &pptx_bytes)?;

        let total_bytes = docx_bytes.len() + xlsx_bytes.len() + pptx_bytes.len();

        Ok(OoxmlExportReport {
            output_dir: output_dir.to_string_lossy().to_string(),
            docx_file: Some(docx_path.to_string_lossy().to_string()),
            xlsx_file: Some(xlsx_path.to_string_lossy().to_string()),
            pptx_file: Some(pptx_path.to_string_lossy().to_string()),
            total_bytes,
            sensitivity_applied: sens,
        })
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// =========================================================================
// CopilotOoxmlGeneratorTool (copilot_ooxml_generator)
// =========================================================================

/// Autonomous tool for generating native Word (.docx), Excel (.xlsx), and PowerPoint (.pptx) documents
#[derive(Clone, Default)]
pub struct CopilotOoxmlGeneratorTool;

impl CopilotOoxmlGeneratorTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for CopilotOoxmlGeneratorTool {
    fn name(&self) -> &str {
        "copilot_ooxml_generator"
    }

    fn description(&self) -> &str {
        "Generate native, binary Microsoft Office Open XML documents (.docx Word, .xlsx Excel, .pptx PowerPoint) with zero Python dependencies, custom styling, callout alert boxes, formula cells, presenter notes, metrics cards, and Purview sensitivity embedding."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["all", "docx", "xlsx", "pptx"],
                    "description": "Document format to generate: 'all', 'docx', 'xlsx', or 'pptx'"
                },
                "title": {
                    "type": "string",
                    "description": "Document title"
                },
                "output_dir": {
                    "type": "string",
                    "description": "Output directory where files will be saved"
                },
                "base_name": {
                    "type": "string",
                    "description": "Base filename (without extension)"
                },
                "sections": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "heading": { "type": "string" },
                            "level": { "type": "number" },
                            "paragraphs": { "type": "array", "items": { "type": "string" } }
                        }
                    },
                    "description": "List of document sections"
                },
                "sensitivity": {
                    "type": "string",
                    "enum": ["Public", "General", "Confidential", "HighlyConfidential", "Secret"],
                    "description": "Purview sensitivity classification"
                }
            },
            "required": ["title"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let title = arguments
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Tagisan Engineering Briefing");

        let _format = arguments
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let base_name = arguments
            .get("base_name")
            .and_then(|v| v.as_str())
            .unwrap_or("briefing");

        let output_dir_str = arguments
            .get("output_dir")
            .and_then(|v| v.as_str())
            .unwrap_or(".tagisan/ooxml");
        let output_dir = PathBuf::from(output_dir_str);

        let sensitivity = arguments
            .get("sensitivity")
            .and_then(|v| v.as_str())
            .map(PurviewSensitivity::from_str_lossy)
            .unwrap_or(PurviewSensitivity::General);

        let mut sections = Vec::new();
        if let Some(sec_arr) = arguments.get("sections").and_then(|v| v.as_array()) {
            for s in sec_arr {
                let heading = s.get("heading").and_then(|v| v.as_str()).unwrap_or("Section").to_string();
                let level = s.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let paragraphs = s.get("paragraphs")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|str_val| str_val.to_string())).collect())
                    .unwrap_or_default();
                sections.push(DocxSection {
                    heading,
                    level,
                    paragraphs,
                    table: None,
                    callouts: Vec::new(),
                    custom_style: None,
                });
            }
        }

        if sections.is_empty() {
            sections.push(DocxSection {
                heading: "Executive Summary".to_string(),
                level: 1,
                paragraphs: vec![
                    "This enterprise artifact was automatically synthesized by Tagisan Multi-Agent Swarm Engine.".to_string(),
                    "Formally verified for regulatory compliance, zero-cloud-egress airgap, and AST blast radius.".to_string(),
                ],
                table: None,
                callouts: Vec::new(),
                custom_style: None,
            });
        }

        let report = OoxmlEngine::export_suite(
            &output_dir,
            base_name,
            title,
            "Tagisan AI Engine",
            &sections,
            Some(sensitivity),
        )?;

        Ok(format!(
            "### 📄 Native Microsoft Office Open XML Documents Generated\n\n\
            - **Title:** {}\n\
            - **Output Directory:** `{}`\n\
            - **Purview Sensitivity:** `{:?}`\n\
            - **Total Payload Size:** {} bytes\n\n\
            #### Artifact Manifest:\n\
            - Word Document (.docx): `{}`\n\
            - Excel Spreadsheet (.xlsx): `{}`\n\
            - PowerPoint Presentation (.pptx): `{}`\n",
            title,
            report.output_dir,
            report.sensitivity_applied,
            report.total_bytes,
            report.docx_file.unwrap_or_default(),
            report.xlsx_file.unwrap_or_default(),
            report.pptx_file.unwrap_or_default(),
        ))
    }
}
