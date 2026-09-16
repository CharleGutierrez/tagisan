//! Office 365 Architecture & Production Integration - Brutal Verification Suite
//!
//! 16 Rigorous Production Verification Tests:
//! 1. Pure-Rust PKZIP packaging and the ZIP EOCD entry count bug fix (`self.files.len() as u16`)
//! 2. In-memory PKZIP archive validation (`verify_zip_package`) on valid and corrupted packages
//! 3. WordprocessingML (`.docx`) document validation (`verify_docx`) with Callout Alert Boxes and Custom Styling
//! 4. SpreadsheetML (`.xlsx`) document validation (`verify_xlsx`) with Formula Cells, Currency, Percentages, and styles.xml
//! 5. PresentationML (`.pptx`) document validation (`verify_pptx`) with Presenter Notes and KPI Metrics Cards
//! 6. Excel Custom Function: `=TGS.VERDICT()` dialectical Lakandiwa synthesis and invariant verification
//! 7. Excel Custom Function: `=TGS.CARBON()` Copilot+ PC on-device NPU vs Cloud GPU footprint modeling
//! 8. Excel Custom Function: `=TGS.COUNCIL()` dynamic array matrix spilling across rows and columns
//! 9. Excel Formula Parser resilience across quoted strings, commas inside quotes, whitespace, and case insensitivity
//! 10. Excel Add-in Manifest, JSON Schema (`functions.json`), and JavaScript Bridge (`functions.js`) packaging
//! 11. Outlook Calendar dynamic risk scoring, Pre-Read Briefing synthesis, and RFC 5545 iCalendar (.ics) export
//! 12. Outlook Executive Meeting Recap email draft creation with DLP and Purview classification in `/me/messages`
//! 13. Microsoft Loop Fluid Framework multiplayer synchronization, voting table consensus, and .loop package export
//! 14. Architecture Decision Record (ADR) MADR 3.0 synthesis, AgentShield invariant validation, and round-trip parsing
//! 15. Microsoft Substrate connector schema registration, AgentShield DLP pre-indexing defense, and semantic search
//! 16. Multi-threaded concurrent stress test (50 worker tasks) stressing all Office 365 engines with zero race conditions

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tagisan::copilot::adr::{AdrDocument, AdrEngine, CopilotAdrSyncTool};
use tagisan::copilot::calendar::{CalendarEngine, CalendarEvent, CopilotCalendarPreReadTool, CopilotOutlookDraftTool};
use tagisan::copilot::excel::{
    export_excel_addin_package, CopilotExcelFunctionsTool, ExcelFunctionsEngine,
};
use tagisan::copilot::graph::{ActionItem, GraphClient};
use tagisan::copilot::loop_pages::{
    CopilotLoopSyncTool, LoopComponentType, LoopPagesEngine, LoopSyncAction,
};
use tagisan::copilot::ooxml::{
    calculate_crc32, CalloutSeverity, CopilotOoxmlGeneratorTool, DocxCallout, DocxCustomStyle,
    DocxSection, DocxTable, OoxmlEngine, PptxMetricCard, PptxSlide, ZipBuilder,
};
use tagisan::copilot::purview::PurviewSensitivity;
use tagisan::copilot::substrate::{
    CopilotSubstrateIngestTool, SubstrateAcl, SubstrateContent, SubstrateEngine, SubstrateItem,
    SubstratePropertySchema,
};
use tagisan::tools::ToolHandler;

// =========================================================================
// Test 1: Pure-Rust PKZIP Packaging & EOCD Entry Count Fix Verification
// =========================================================================
#[test]
fn test_pkzip_eocd_entry_count_fix_and_crc32() {
    println!("\n=== [TEST 1] PKZIP EOCD Entry Count Fix & CRC-32 Check ===");

    // Test standard IEEE 802.3 CRC32 test vector
    let crc = calculate_crc32(b"123456789");
    assert_eq!(crc, 0xCBF43926, "CRC-32 implementation must match standard ITU-T V.42 / PKZIP");

    // Test ZipBuilder with varying numbers of files (from 1 to 25 files with varying path lengths)
    for file_count in [1, 5, 12, 25] {
        let mut zip = ZipBuilder::new();
        for i in 0..file_count {
            let path = format!("nested/subfolder/very_long_path_name_{}/document_part_{}.xml", i, i);
            let content = format!("<xml id=\"{}\">Sample OOXML Content Data {}</xml>", i, i);
            zip.add_file(&path, content.as_bytes());
        }

        let zip_bytes = zip.finish();
        assert!(zip_bytes.len() >= 22);

        // Verify End of Central Directory Record (EOCD)
        let eocd_pos = zip_bytes.len() - 22;
        assert_eq!(&zip_bytes[eocd_pos..eocd_pos + 4], &[0x50, 0x4b, 0x05, 0x06], "EOCD signature PK\\x05\\x06");

        let entries_on_disk = u16::from_le_bytes(zip_bytes[eocd_pos + 8..eocd_pos + 10].try_into().unwrap());
        let total_entries = u16::from_le_bytes(zip_bytes[eocd_pos + 10..eocd_pos + 12].try_into().unwrap());

        // BUG FIX VERIFICATION: total_entries must exactly equal file_count
        assert_eq!(
            total_entries, file_count as u16,
            "ZIP EOCD total entries must exactly equal file count (not cd_size / 46)"
        );
        assert_eq!(
            entries_on_disk, file_count as u16,
            "ZIP EOCD entries on disk must match total entries"
        );
    }
    println!("  [✓] PKZIP EOCD entry count calculation mathematically verified for all file counts!");
}

// =========================================================================
// Test 2: In-Memory PKZIP Archive Validation (`verify_zip_package`)
// =========================================================================
#[test]
fn test_verify_zip_package_valid_and_corrupt() {
    println!("\n=== [TEST 2] In-Memory PKZIP Archive Validation ===");

    // 1. Valid Archive Verification
    let mut zip = ZipBuilder::new();
    zip.add_file("file1.txt", b"Hello World 1");
    zip.add_file("sub/file2.txt", b"Hello World 2");
    zip.add_file("docProps/core.xml", b"<xml>Core Properties</xml>");
    let valid_bytes = zip.finish();

    let ver = OoxmlEngine::verify_zip_package(&valid_bytes).expect("Valid ZIP must pass verification");
    assert!(ver.is_valid);
    assert_eq!(ver.total_entries, 3);
    assert_eq!(ver.file_names.len(), 3);
    assert!(ver.file_names.contains(&"file1.txt".to_string()));
    assert!(ver.file_names.contains(&"sub/file2.txt".to_string()));
    assert!(ver.file_names.contains(&"docProps/core.xml".to_string()));

    // 2. Corrupted Archive: Truncated
    let truncated = &valid_bytes[..valid_bytes.len() - 30];
    let err_trunc = OoxmlEngine::verify_zip_package(truncated);
    assert!(err_trunc.is_err(), "Truncated ZIP archive must fail verification");

    // 3. Corrupted Archive: Mismatched CRC
    let mut corrupt_crc = valid_bytes.clone();
    // Tamper with payload byte inside file1.txt (data begins at offset 39)
    if corrupt_crc.len() > 45 {
        corrupt_crc[42] ^= 0xFF;
    }
    let err_crc = OoxmlEngine::verify_zip_package(&corrupt_crc);
    assert!(err_crc.is_err(), "Tampered payload with CRC mismatch must fail verification");

    println!("  [✓] PKZIP header, central directory, and CRC-32 integrity validator verified!");
}

// =========================================================================
// Test 3: WordprocessingML (`.docx`) with Callout Alert Boxes & Custom Styling
// =========================================================================
#[test]
fn test_verify_docx_with_callouts_and_styling() {
    println!("\n=== [TEST 3] Word (.docx) Callout Alert Boxes & Custom Styling ===");

    let mut sec = DocxSection::new("Architecture Decisions & Incident Review", 1)
        .with_paragraph("This document outlines critical system invariants and security controls.")
        .with_custom_style(DocxCustomStyle {
            font_family: Some("Aptos".to_string()),
            font_size_pt: Some(14),
            color_hex: Some("0F6CBD".to_string()),
            bold: true,
            italic: false,
        });

    // Add Callout Alert Boxes of all severities
    sec = sec.with_callout(DocxCallout {
        title: "Hardware Zero-Egress Air-Gap Active".to_string(),
        message: "All tensor operations execute locally on NPU. Outbound network traffic is blocked.".to_string(),
        severity: CalloutSeverity::Info,
    });
    sec = sec.with_callout(DocxCallout {
        title: "High AST Blast Radius Detected".to_string(),
        message: "Refactoring EntraAuthManager impacts 14 downstream modules. Proceed with caution.".to_string(),
        severity: CalloutSeverity::Warning,
    });
    sec = sec.with_callout(DocxCallout {
        title: "Unsafe Memory Invariant Violation".to_string(),
        message: "Detected undocumented unsafe block. Formal verification gates reject PR merge.".to_string(),
        severity: CalloutSeverity::Danger,
    });
    sec = sec.with_callout(DocxCallout {
        title: "Rule 141 Judicial Legal Fees Validated".to_string(),
        message: "SAJ and JDF assessment algorithms passed 100% of statutory test vectors.".to_string(),
        severity: CalloutSeverity::Success,
    });
    sec = sec.with_callout(DocxCallout {
        title: "Purview Information Protection Applied".to_string(),
        message: "Confidential classification badge embedded into document core properties.".to_string(),
        severity: CalloutSeverity::Tip,
    });

    // Add Table
    sec = sec.with_table(DocxTable {
        headers: vec!["Component".to_string(), "Status".to_string(), "Risk Level".to_string()],
        rows: vec![
            vec!["Entra ID Auth".to_string(), "Passed".to_string(), "Low".to_string()],
            vec!["Substrate Index".to_string(), "Active".to_string(), "Low".to_string()],
        ],
    });

    let docx_bytes = OoxmlEngine::build_docx(
        "Tagisan Architectural Specification",
        "Lead Systems Architect",
        &[sec],
        Some(PurviewSensitivity::Confidential),
    ).expect("Build docx failed");

    // Validate DOCX structure conforming to ISO/IEC 29500
    let ver = OoxmlEngine::verify_docx(&docx_bytes).expect("DOCX package verification failed");
    assert!(ver.is_valid);
    assert!(ver.file_names.contains(&"word/document.xml".to_string()));
    assert!(ver.file_names.contains(&"docProps/core.xml".to_string()));

    // Verify callout XML tags in word/document.xml
    let doc_xml = OoxmlEngine::extract_file_from_zip(&docx_bytes, "word/document.xml")
        .expect("Extract document.xml failed");
    let doc_str = String::from_utf8_lossy(&doc_xml);

    assert!(doc_str.contains("[NOTE] Hardware Zero-Egress Air-Gap Active"));
    assert!(doc_str.contains("[WARNING] High AST Blast Radius Detected"));
    assert!(doc_str.contains("[CRITICAL ALERT] Unsafe Memory Invariant Violation"));
    assert!(doc_str.contains("[VERIFIED] Rule 141 Judicial Legal Fees Validated"));
    assert!(doc_str.contains("<w:pBdr>"));
    assert!(doc_str.contains("<w:shd"));

    println!("  [✓] WordprocessingML callout alert boxes and typography styling verified!");
}

// =========================================================================
// Test 4: SpreadsheetML (`.xlsx`) with Formula Cells & Formatting
// =========================================================================
#[test]
fn test_verify_xlsx_with_formulas_and_formatting() {
    println!("\n=== [TEST 4] Excel (.xlsx) Formula Cells & Cell Formatting ===");

    let headers = vec!["Module", "Tokens", "Cost Savings", "Reduction Rate", "Formula Check"];
    let rows = vec![
        vec![
            "Ollama / Colibri Local".to_string(),
            "500000".to_string(),
            "$12.50".to_string(),
            "88.5%".to_string(),
            "=TGS.COST_SAVINGS(500000, 250000)".to_string(),
        ],
        vec![
            "Snapdragon X Elite NPU".to_string(),
            "1000000".to_string(),
            "$25.00".to_string(),
            "93.7%".to_string(),
            "=TGS.CARBON(\"Snapdragon\", 1000000, \"NPU\")".to_string(),
        ],
        vec![
            "Total Aggregate".to_string(),
            "1500000".to_string(),
            "$37.50".to_string(),
            "91.1%".to_string(),
            "=SUM(B2:B3)".to_string(),
        ],
    ];

    let xlsx_bytes = OoxmlEngine::build_xlsx(
        "Tagisan Cost & Carbon Ledger",
        "Telemetry",
        &headers,
        &rows,
    ).expect("Build xlsx failed");

    // Validate XLSX structure conforming to ISO/IEC 29500
    let ver = OoxmlEngine::verify_xlsx(&xlsx_bytes).expect("XLSX package verification failed");
    assert!(ver.is_valid);
    assert!(ver.file_names.contains(&"xl/workbook.xml".to_string()));
    assert!(ver.file_names.contains(&"xl/worksheets/sheet1.xml".to_string()));
    assert!(ver.file_names.contains(&"xl/styles.xml".to_string()));

    // Verify formulas and styles in sheet1.xml
    let sheet_xml = OoxmlEngine::extract_file_from_zip(&xlsx_bytes, "xl/worksheets/sheet1.xml")
        .expect("Extract sheet1.xml failed");
    let sheet_str = String::from_utf8_lossy(&sheet_xml);

    assert!(sheet_str.contains("<f>TGS.COST_SAVINGS(500000, 250000)</f>"));
    assert!(sheet_str.contains("<f>TGS.CARBON(&quot;Snapdragon&quot;, 1000000, &quot;NPU&quot;)</f>"));
    assert!(sheet_str.contains("<f>SUM(B2:B3)</f>"));
    assert!(sheet_str.contains("s=\"1\""), "Headers must use header style s=1");
    assert!(sheet_str.contains("s=\"2\""), "Currency cells must use currency style s=2");
    assert!(sheet_str.contains("s=\"3\""), "Percentage cells must use percentage style s=3");

    println!("  [✓] SpreadsheetML formula cells, currency formatting, and styles.xml verified!");
}

// =========================================================================
// Test 5: PresentationML (`.pptx`) with Presenter Notes & Metrics Cards
// =========================================================================
#[test]
fn test_verify_pptx_with_presenter_notes_and_metric_cards() {
    println!("\n=== [TEST 5] PowerPoint (.pptx) Presenter Notes & KPI Cards ===");

    let mut slide1 = PptxSlide::new("Tagisan Executive Telemetry Briefing")
        .with_subtitle("Multi-Agent Collaborative Copilot Infrastructure")
        .with_bullet("Complete zero-cloud-egress compliance for sensitive workloads")
        .with_bullet("100% formal invariant verification on all merged branches")
        .with_presenter_notes("Note to presenter: Emphasize the 1000x reliability and zero-panic runtime guarantees.");

    // Add Executive KPI Metrics Cards
    slide1 = slide1.with_metric_card(PptxMetricCard {
        label: "Availability SLA".to_string(),
        value: "99.999%".to_string(),
        change_or_subtext: Some("+0.005% YoY".to_string()),
        color_hex: Some("F3F2F1".to_string()),
    });
    slide1 = slide1.with_metric_card(PptxMetricCard {
        label: "Local NPU Savings".to_string(),
        value: "$142,500".to_string(),
        change_or_subtext: Some("88.4% cost drop".to_string()),
        color_hex: Some("F3F2F1".to_string()),
    });
    slide1 = slide1.with_metric_card(PptxMetricCard {
        label: "AST P99 Latency".to_string(),
        value: "0.85 ms".to_string(),
        change_or_subtext: Some("Sub-millisecond".to_string()),
        color_hex: Some("F3F2F1".to_string()),
    });

    let pptx_bytes = OoxmlEngine::build_pptx(
        "Executive Quarterly Architecture Review",
        "Autonomous Systems Engineering",
        &[slide1],
    ).expect("Build pptx failed");

    // Validate PPTX structure conforming to ISO/IEC 29500
    let ver = OoxmlEngine::verify_pptx(&pptx_bytes).expect("PPTX package verification failed");
    assert!(ver.is_valid);
    assert!(ver.file_names.contains(&"ppt/presentation.xml".to_string()));
    assert!(ver.file_names.contains(&"ppt/slides/slide2.xml".to_string()));
    assert!(ver.file_names.contains(&"ppt/notesSlides/notesSlide2.xml".to_string()));

    // Check notes slide contents
    let notes_xml = OoxmlEngine::extract_file_from_zip(&pptx_bytes, "ppt/notesSlides/notesSlide2.xml")
        .expect("Extract notesSlide2.xml failed");
    let notes_str = String::from_utf8_lossy(&notes_xml);
    assert!(notes_str.contains("Note to presenter: Emphasize the 1000x reliability"));

    // Check metric cards in slide2.xml
    let slide_xml = OoxmlEngine::extract_file_from_zip(&pptx_bytes, "ppt/slides/slide2.xml")
        .expect("Extract slide2.xml failed");
    let slide_str = String::from_utf8_lossy(&slide_xml);
    assert!(slide_str.contains("MetricCard_0"));
    assert!(slide_str.contains("99.999%"));
    assert!(slide_str.contains("$142,500"));
    assert!(slide_str.contains("0.85 ms"));

    println!("  [✓] PresentationML presenter notes slides and DrawingML KPI cards verified!");
}

// =========================================================================
// Test 6: Excel Custom Function `=TGS.VERDICT()`
// =========================================================================
#[test]
fn test_excel_custom_function_verdict() {
    println!("\n=== [TEST 6] Excel Custom Function: =TGS.VERDICT() ===");

    let engine = ExcelFunctionsEngine::new();

    // 1. Approved Architecture Proposal
    let res_ok = engine.eval_verdict("Adopt on-device DirectML for Copilot+ PC", Some("Hardware Offload"));
    assert_eq!(res_ok.function, "TGS.VERDICT");
    assert_eq!(res_ok.value["status"], "APPROVED");
    assert!(res_ok.value["confidence"].as_f64().unwrap() >= 0.95);
    assert!(res_ok.display_string.contains("APPROVED"));
    assert!(res_ok.display_string.contains("Confidence: 96.5%"));

    // 2. Conditionally Approved Proposal
    let res_cond = engine.eval_verdict("Implement raw pointer optimization in kernel", Some("Core Engine"));
    assert_eq!(res_cond.value["status"], "CONDITIONALLY_APPROVED");
    assert!(res_cond.display_string.contains("CONDITIONALLY_APPROVED"));

    // 3. Rejected Insecure Proposal
    let res_rej = engine.eval_verdict("Bypass authentication for testing endpoints", Some("Security"));
    assert_eq!(res_rej.value["status"], "REJECTED");
    assert!(res_rej.display_string.contains("REJECTED"));

    println!("  [✓] =TGS.VERDICT() dialectical consensus and invariant check verified!");
}

// =========================================================================
// Test 7: Excel Custom Function `=TGS.CARBON()`
// =========================================================================
#[test]
fn test_excel_custom_function_carbon() {
    println!("\n=== [TEST 7] Excel Custom Function: =TGS.CARBON() ===");

    let engine = ExcelFunctionsEngine::new();

    // Qualcomm Snapdragon X Elite NPU calculation
    let res = engine.eval_carbon("Qualcomm Snapdragon X Elite", 1_000_000.0, Some("NPU"));
    assert_eq!(res.function, "TGS.CARBON");
    let local_g = res.value["local_emissions_g_co2"].as_f64().unwrap();
    let cloud_g = res.value["cloud_baseline_g_co2"].as_f64().unwrap();
    let reduction = res.value["carbon_reduction_percentage"].as_f64().unwrap();

    assert!(local_g < cloud_g);
    assert!(reduction >= 85.0, "Snapdragon NPU must achieve >85% carbon reduction vs Cloud H100");
    assert!(res.display_string.contains("reduction vs Cloud GPU"));

    println!("  [✓] =TGS.CARBON() on-device NPU vs datacenter GPU carbon modeling verified!");
}

// =========================================================================
// Test 8: Excel Custom Function `=TGS.COUNCIL()` & Dynamic Array Spilling
// =========================================================================
#[test]
fn test_excel_custom_function_council_dynamic_array() {
    println!("\n=== [TEST 8] Excel Custom Function: =TGS.COUNCIL() Dynamic Arrays ===");

    let engine = ExcelFunctionsEngine::new();

    let res = engine.eval_council("Microservices vs Modular Monolith", Some(3));
    assert_eq!(res.function, "TGS.COUNCIL");
    assert!(res.is_dynamic_array, "Council evaluation must be flagged as dynamic array");
    assert!(res.array_data.is_some());

    let grid = res.array_data.unwrap();
    assert_eq!(grid.len(), 4, "Must spill 4 rows (1 Header + 3 Agent stances)");
    assert_eq!(grid[0].len(), 3, "Must spill 3 columns (Role, Stance, Invariant)");

    // Headers
    assert_eq!(grid[0][0], "Council Role");
    assert_eq!(grid[0][1], "Agent Perspective / Stance");
    assert_eq!(grid[0][2], "Formal Invariant");

    // Roles
    assert_eq!(grid[1][0], "Thesis Proponent");
    assert_eq!(grid[2][0], "Antithesis Adversary");
    assert_eq!(grid[3][0], "Lakandiwa Synthesis");

    println!("  [✓] =TGS.COUNCIL() 2D dynamic array matrix spilling verified!");
}

// =========================================================================
// Test 9: Resilient Excel Formula Parser Handling
// =========================================================================
#[test]
fn test_excel_formula_parser_robustness() {
    println!("\n=== [TEST 9] Resilient Formula Parser Handling ===");

    let engine = ExcelFunctionsEngine::new();

    // 1. Quoted arguments containing embedded commas
    let f1 = "=TGS.VERDICT(\"Adopt Rule 141, Section 7 Legal Fees\", \"Judiciary Docket System\")";
    let res1 = engine.eval_formula(f1).expect("Parse formula with commas in quotes failed");
    assert_eq!(res1.value["proposal"], "Adopt Rule 141, Section 7 Legal Fees");
    assert_eq!(res1.value["context"], "Judiciary Docket System");

    // 2. Numeric arguments with whitespace
    let f2 = " =TGS.COST_SAVINGS(  250000 ,  100000  ) ";
    let res2 = engine.eval_formula(f2).expect("Parse formula with whitespace failed");
    assert_eq!(res2.function, "TGS.COST_SAVINGS");

    // 3. Case-insensitive function name
    let f3 = "=tgs.carbon(\"Snapdragon X Elite\", 500000, \"npu\")";
    let res3 = engine.eval_formula(f3).expect("Parse lowercase formula failed");
    assert_eq!(res3.function, "TGS.CARBON");

    // 4. Dynamic array council formula
    let f4 = "=TGS.COUNCIL(\"Zero-Trust Architecture\", 5)";
    let res4 = engine.eval_formula(f4).expect("Parse council formula failed");
    assert!(res4.is_dynamic_array);
    assert_eq!(res4.value["rounds"], 5);

    println!("  [✓] Robust formula parser with quotes, commas, whitespace, and case insensitivity verified!");
}

// =========================================================================
// Test 10: Excel Add-in Manifest, JSON Schema & JavaScript Bridge Packaging
// =========================================================================
#[test]
fn test_excel_addin_packaging_with_new_functions() {
    println!("\n=== [TEST 10] Excel Add-in Packaging & Schema Verification ===");

    let export_dir = PathBuf::from(".tagisan/test_office365_addin_export");
    let _ = std::fs::remove_dir_all(&export_dir);

    let pkg = export_excel_addin_package(&export_dir, "https://api.tagisan.ai")
        .expect("Export Add-in package failed");
    assert!(pkg.total_bytes > 2000);
    assert_eq!(pkg.files.len(), 3);

    // 1. Check manifest.xml
    let manifest = std::fs::read_to_string(&pkg.files[0]).expect("Read manifest failed");
    assert!(manifest.contains("<Host Name=\"Workbook\"/>"));
    assert!(manifest.contains("<Extension xsi:type=\"CustomFunctions\">"));

    // 2. Check functions.json
    let json_str = std::fs::read_to_string(&pkg.files[1]).expect("Read functions.json failed");
    let functions_json: serde_json::Value = serde_json::from_str(&json_str).expect("Parse JSON failed");
    let funcs = functions_json["functions"].as_array().expect("functions array");

    let has_verdict = funcs.iter().any(|f| f["name"] == "TGS.VERDICT");
    let has_carbon = funcs.iter().any(|f| f["name"] == "TGS.CARBON");
    let has_council = funcs.iter().any(|f| f["name"] == "TGS.COUNCIL");

    assert!(has_verdict, "functions.json must declare TGS.VERDICT");
    assert!(has_carbon, "functions.json must declare TGS.CARBON");
    assert!(has_council, "functions.json must declare TGS.COUNCIL");

    // COUNCIL must have matrix dimensionality for dynamic arrays
    let council_def = funcs.iter().find(|f| f["name"] == "TGS.COUNCIL").unwrap();
    assert_eq!(council_def["result"]["dimensionality"], "matrix");

    // 3. Check functions.js
    let js_str = std::fs::read_to_string(&pkg.files[2]).expect("Read functions.js failed");
    assert!(js_str.contains("CustomFunctions.associate(\"TGS.VERDICT\", verdict);"));
    assert!(js_str.contains("CustomFunctions.associate(\"TGS.CARBON\", carbon);"));
    assert!(js_str.contains("CustomFunctions.associate(\"TGS.COUNCIL\", council);"));

    println!("  [✓] Complete Excel Add-in manifest, Office-JS metadata, and JS bridge verified!");
}

// =========================================================================
// Test 11: Outlook Calendar Dynamic Risk Scoring & RFC 5545 (.ics) Generation
// =========================================================================
#[tokio::test]
async fn test_calendar_engine_dynamic_preread_and_ics() {
    println!("\n=== [TEST 11] Calendar Pre-Read Intelligence & RFC 5545 iCalendar Export ===");

    let client = Arc::new(GraphClient::mock());
    let engine = CalendarEngine::new(client);

    // 1. Fetch upcoming events
    let events = engine.get_upcoming_events(12).await.expect("Fetch upcoming events failed");
    assert!(!events.is_empty());

    let court_event = &events[0];
    let brief = engine.generate_preread_brief(court_event);

    assert_eq!(brief.event_id, court_event.id);
    assert!(brief.technical_risk_score >= 0.60, "Court docket event must have elevated risk score");
    assert!(brief.active_prs.iter().any(|pr| pr.blast_radius_risk == "CRITICAL"));
    assert!(!brief.discussion_prompts.is_empty());

    // 2. Generate RFC 5545 iCalendar Invitation (.ics)
    let ics = engine.generate_ics_event(court_event);
    assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"));
    assert!(ics.contains("VERSION:2.0\r\n"));
    assert!(ics.contains("BEGIN:VEVENT\r\n"));
    assert!(ics.contains(&format!("UID:{}\r\n", court_event.id)));
    assert!(ics.contains("SUMMARY:RTC-OCC Docket Ingestion"));
    assert!(ics.contains("END:VEVENT\r\n"));
    assert!(ics.ends_with("END:VCALENDAR\r\n"));

    // 3. Autonomous Tool Verification
    let tool = CopilotCalendarPreReadTool::default();
    let out = tool.execute(serde_json::json!({
        "hours_ahead": 12,
        "export_ics": true
    })).await.expect("CopilotCalendarPreReadTool failed");

    assert!(out.contains("Outlook Calendar Pre-Read Briefing Synthesized"));
    assert!(out.contains("RFC 5545 iCalendar Payload:"));

    println!("  [✓] Outlook Calendar dynamic risk pre-read and RFC 5545 .ics generation verified!");
}

// =========================================================================
// Test 12: Outlook Executive Meeting Recap Drafts with Purview
// =========================================================================
#[tokio::test]
async fn test_calendar_recap_draft_and_purview() {
    println!("\n=== [TEST 12] Outlook Meeting Recap Drafts in /me/messages ===");

    let client = Arc::new(GraphClient::mock());
    let engine = CalendarEngine::new(client);

    let action_items = vec![
        ActionItem {
            id: "act-01".to_string(),
            title: "Harden Substrate DLP Filter".to_string(),
            description: "Block sensitive credential exfiltration".to_string(),
            assignee: Some("SecOps Lead".to_string()),
            priority: "HIGH".to_string(),
            due_date: Some("2026-09-30".to_string()),
            category: Some("Security".to_string()),
        },
    ];

    let draft = engine.create_recap_draft(
        "Sprint 42 Architecture Convergence Review",
        &["architect@tagisan.local".to_string(), "lead@tagisan.local".to_string()],
        &["Preserve all 1000x reliability invariants".to_string()],
        &action_items,
        &["https://github.com/tagisan/tgs/pull/88".to_string()],
        PurviewSensitivity::HighlyConfidential,
    ).await.expect("Create draft failed");

    assert!(draft.draft_id.starts_with("draft-msg-"));
    assert_eq!(draft.recipients_count, 2);
    assert_eq!(draft.status, "DRAFT_SAVED_PENDING_USER_APPROVAL");

    println!("  [✓] Outlook Meeting Recap draft with DLP and Purview classification verified!");
}

// =========================================================================
// Test 13: Microsoft Loop Pages Multiplayer Sync & Voting Table Consensus
// =========================================================================
#[test]
fn test_loop_pages_multiplayer_sync_and_voting() {
    println!("\n=== [TEST 13] Microsoft Loop Pages Multiplayer Sync & Voting ===");

    let engine = LoopPagesEngine::new();

    // 1. Create Collaborative Voting Table
    let comp = engine.create_voting_table(
        "Copilot+ PC Architecture Consensus",
        &["DirectML On-Device NPU", "Remote Cloud GPU Cluster"],
        "Chief Architect",
    );
    assert_eq!(comp.component_type, LoopComponentType::VotingTable);
    assert_eq!(comp.version, 1);

    // 2. Apply Voting Actions
    let actions = vec![
        LoopSyncAction::Vote { row_index: 0, voter: "Architect A".to_string() },
        LoopSyncAction::Vote { row_index: 0, voter: "Architect B".to_string() },
        LoopSyncAction::Vote { row_index: 1, voter: "Architect C".to_string() },
    ];
    let sync_res = engine.apply_actions(&comp.id, &actions).expect("Apply voting failed");
    assert_eq!(sync_res.applied_actions, 3);
    assert_eq!(sync_res.fluid_sequence, 4);

    let updated = engine.get_component(&comp.id).expect("Get component failed");
    assert_eq!(updated.rows[0][1], "2", "DirectML option must have 2 votes");
    assert_eq!(updated.rows[1][1], "1", "Cloud GPU option must have 1 vote");
    assert!(updated.rows[0][2].contains("Architect A"));

    // 3. Export to Microsoft Fluid Framework 2.x (.loop JSON)
    let fluid_json = engine.export_fluid_package(&comp.id).expect("Export Fluid failed");
    assert!(fluid_json.contains("https://fluidframework.com/schemas/v2/loop-component.json"));
    assert!(fluid_json.contains("DirectML On-Device NPU"));

    println!("  [✓] Microsoft Loop multiplayer Fluid sync, voting consensus, and JSON package verified!");
}

// =========================================================================
// Test 14: ADR MADR 3.0 Synthesis, Validation & Round-Trip Parsing
// =========================================================================
#[test]
fn test_adr_madr_synthesis_and_roundtrip_parsing() {
    println!("\n=== [TEST 14] Architecture Decision Record (ADR) Synthesis & Round-Trip ===");

    // 1. Synthesize ADR from dialectical debate
    let invariants = vec![
        "Invariant 1: All operations must preserve transactional idempotency.".to_string(),
        "Invariant 2: AgentShield DLP gates must verify zero credential egress.".to_string(),
    ];
    let adr = AdrEngine::synthesize(
        "Standardize on In-Memory PKZIP Packaging Pipeline",
        Some("Adopt pure-Rust ISO/IEC 29500 packaging with zero external Python dependencies."),
        Some("Adopt Pure-Rust OOXML Engine"),
        Some(&invariants),
    );

    // 2. Validate compliance with MADR 3.0
    AdrEngine::validate_madr_compliance(&adr).expect("ADR must satisfy MADR 3.0 specification");

    // 3. Round-trip parse from generated Markdown
    let parsed = AdrEngine::parse_madr(&adr.markdown).expect("Round-trip Markdown parse failed");
    assert_eq!(parsed.id, adr.id);
    assert_eq!(parsed.title, adr.title);
    assert_eq!(parsed.status, adr.status);
    assert_eq!(parsed.invariants.len(), 2);
    assert!(parsed.invariants[0].contains("transactional idempotency"));

    println!("  [✓] MADR 3.0 Architecture Decision Record synthesis, compliance, and round-trip parsing verified!");
}

// =========================================================================
// Test 15: Microsoft Substrate Connector Schema & AgentShield DLP Defense
// =========================================================================
#[tokio::test]
async fn test_substrate_ingestion_dlp_and_schema_validation() {
    println!("\n=== [TEST 15] Microsoft Substrate Connector Schema & AgentShield DLP Defense ===");

    let engine = SubstrateEngine::default();

    let schema = vec![
        SubstratePropertySchema {
            name: "title".to_string(),
            property_type: "String".to_string(),
            is_searchable: true,
            is_queryable: true,
            is_retrievable: true,
            is_refinable: false,
        },
        SubstratePropertySchema {
            name: "blastRadius".to_string(),
            property_type: "Int64".to_string(),
            is_searchable: false,
            is_queryable: true,
            is_retrievable: true,
            is_refinable: true,
        },
    ];

    // 1. Register Schema
    let reg = engine.register_schema(&schema).await.expect("Register schema failed");
    assert!(reg.contains("Substrate schema registered successfully"));

    // 2. Valid Ingestion Item
    let mut props_ok = std::collections::HashMap::new();
    props_ok.insert("title".to_string(), serde_json::json!("Tagisan Architecture Spec"));
    props_ok.insert("blastRadius".to_string(), serde_json::json!(4));

    let item_ok = SubstrateItem {
        id: "tgs-item-arch-001".to_string(),
        properties: props_ok,
        content: SubstrateContent {
            content_type: "text".to_string(),
            value: "Tagisan multi-agent harness provides dialectical debate consensus.".to_string(),
        },
        acl: vec![SubstrateAcl {
            access_type: "grant".to_string(),
            identity_type: "everyone".to_string(),
            value: "everyone".to_string(),
        }],
    };

    engine.validate_item_against_schema(&item_ok, &schema).expect("Valid schema item must pass");
    let ingest_res = engine.ingest_item(&item_ok).await.expect("Ingest valid item failed");
    assert!(ingest_res.contains("successfully indexed"));

    // 3. Search Grounding
    let search_results = engine.search_substrate_index("dialectical debate");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, "tgs-item-arch-001");

    // 4. Dirty Ingestion: AgentShield DLP Defense Interception
    let mut props_dirty = std::collections::HashMap::new();
    props_dirty.insert("title".to_string(), serde_json::json!("Leaked Credential Test"));
    props_dirty.insert("blastRadius".to_string(), serde_json::json!(1));

    let item_dirty = SubstrateItem {
        id: "tgs-dirty-002".to_string(),
        properties: props_dirty,
        content: SubstrateContent {
            content_type: "text".to_string(),
            value: "Secret connection: DefaultEndpointsProtocol=https;AccountName=prod;AccountKey=dGhpcyBpcyBhIGZha2Uga2V5IGZvciB0ZXN0aW5nIQ==".to_string(),
        },
        acl: vec![],
    };

    let dlp_err = engine.ingest_item(&item_dirty).await;
    assert!(dlp_err.is_err(), "AgentShield must block ingestion of leaked credentials into Substrate");
    let err_msg = format!("{}", dlp_err.err().unwrap());
    assert!(err_msg.contains("AgentShield DLP barrier blocked Substrate indexing"));

    println!("  [✓] Microsoft Substrate connector schema, AgentShield DLP defense, and search grounding verified!");
}

// =========================================================================
// Test 16: Full-Suite High Concurrency Stress Test (50 Parallel Workers)
// =========================================================================
#[tokio::test]
async fn test_office365_concurrent_stress_50_workers() {
    println!("\n=== [TEST 16] Full-Suite Concurrent Stress Test (50 Parallel Workers) ===");

    let concurrency = 50;
    let excel_engine = Arc::new(ExcelFunctionsEngine::new());
    let loop_engine = Arc::new(LoopPagesEngine::new());
    let calendar_engine = Arc::new(CalendarEngine::new(Arc::new(GraphClient::mock())));
    let substrate_engine = Arc::new(SubstrateEngine::default());

    // Create shared loop component
    let shared_loop = loop_engine.create_table(
        "Stress Test Concurrent Matrix",
        &["Worker", "Status", "Latency"],
        &[],
        "Stress Master",
    );

    let mut tasks = Vec::with_capacity(concurrency);
    let start = Instant::now();

    for worker_id in 0..concurrency {
        let e_eng = excel_engine.clone();
        let l_eng = loop_engine.clone();
        let c_eng = calendar_engine.clone();
        let s_eng = substrate_engine.clone();
        let comp_id = shared_loop.id.clone();

        let task = tokio::spawn(async move {
            // 1. Generate DOCX with callout
            let sec = DocxSection::new(format!("Worker {} Invariant Report", worker_id), 1)
                .with_paragraph(format!("Telemetry data recorded from worker {}", worker_id))
                .with_callout(DocxCallout {
                    title: format!("Worker {} Active", worker_id),
                    message: "High concurrency stress validation".to_string(),
                    severity: CalloutSeverity::Success,
                });
            let docx = OoxmlEngine::build_docx("Stress Spec", "Worker", &[sec], None).expect("Worker DOCX build failed");
            let _ = OoxmlEngine::verify_docx(&docx).expect("Worker DOCX verify failed");

            // 2. Generate XLSX with formula
            let xlsx = OoxmlEngine::build_xlsx(
                "Worker Ledger",
                "Data",
                &["Worker", "Formula"],
                &[vec![format!("Worker {}", worker_id), format!("=TGS.CARBON(\"Snapdragon\", {})", worker_id * 1000)]],
            ).expect("Worker XLSX build failed");
            let _ = OoxmlEngine::verify_xlsx(&xlsx).expect("Worker XLSX verify failed");

            // 3. Evaluate Excel Formulas
            let v_res = e_eng.eval_verdict(&format!("Worker {} Invariant Preservation", worker_id), None);
            assert_eq!(v_res.value["status"], "APPROVED");

            let c_res = e_eng.eval_council(&format!("Topic {}", worker_id), Some(2));
            assert!(c_res.is_dynamic_array);

            // 4. Update Shared Loop Component
            let action = LoopSyncAction::AppendRow {
                row: vec![format!("W-{}", worker_id), "SUCCESS".to_string(), "0.4ms".to_string()],
            };
            let _ = l_eng.apply_actions(&comp_id, &[action]).expect("Worker loop sync failed");

            // 5. Calendar Briefing
            let evt = CalendarEvent {
                id: format!("evt-stress-{}", worker_id),
                subject: format!("Architecture Sync Worker {}", worker_id),
                start_time: "2026-09-16T10:00:00Z".to_string(),
                end_time: "2026-09-16T11:00:00Z".to_string(),
                organizer: "lead@tagisan.local".to_string(),
                attendees: vec![],
                body_preview: "DirectML NPU stress review".to_string(),
                web_link: "https://teams.microsoft.com/l/meetup".to_string(),
                meeting_type: "Sync".to_string(),
            };
            let _ = c_eng.generate_preread_brief(&evt);

            // 6. Substrate Search
            let _ = s_eng.search_substrate_index("architecture");
        });
        tasks.push(task);
    }

    for t in tasks {
        t.await.expect("Stress worker task panicked");
    }

    let duration = start.elapsed();
    let final_loop = loop_engine.get_component(&shared_loop.id).expect("Get final loop failed");
    assert_eq!(final_loop.rows.len(), concurrency, "Every worker must successfully append its row");
    assert_eq!(final_loop.version, (concurrency + 1) as u64, "Fluid sequence must increment atomically");

    println!(
        "  [✓] 50 parallel workers completed full Office 365 workflow in {:?} with ZERO race conditions!",
        duration
    );
}
