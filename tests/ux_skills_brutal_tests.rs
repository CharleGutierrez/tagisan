//! # Brutal Test Suite: Tagisan UI/UX Design System Intelligence & Accessibility Engine
//!
//! Brutally validates:
//! 1. Exact mathematical color science: IEC 61966-2-1 linearization, relative luminance, WCAG 2.2 contrast ratio bounds.
//! 2. Contrast grading thresholds (AAA, AA, Fail) and iterative HSL accessibility solver.
//! 3. Static UI AST / pattern analyzers across TSX, JSX, HTML, Vue, Svelte, and CSS.
//! 4. 8-Point spatial grid enforcement and arbitrary pixel nudge detection.
//! 5. Design Token Synthesizer across 5 brand presets (Linear, Apple, Stripe, Cyberpunk, Nord).
//! 6. Pre-delivery UX Readiness Scoring engine and release gating verdicts.
//! 7. The 500 UX Rules Catalog completeness across all 12 HCI clusters.
//! 8. Built-in ECC Skill registration and frontmatter parsing.

use tagisan::ux::*;
use tagisan::ecc::skills::{all_built_in_skills, find_built_in_skill};

#[test]
fn test_brutal_color_science_and_linearization() {
    // 1. IEC 61966-2-1 linearization invariant checks
    assert_eq!(linearize_srgb(0.0), 0.0);
    assert_eq!(linearize_srgb(1.0), 1.0);

    // Below threshold (0.04045): linear segment
    let c_low = 0.03;
    let lin_low = linearize_srgb(c_low);
    let expected_low = c_low / 12.92;
    assert!((lin_low - expected_low).abs() < 1e-6);

    // Above threshold: exponential curve ((c + 0.055) / 1.055)^2.4
    let c_high = 0.5;
    let lin_high = linearize_srgb(c_high);
    let expected_high = ((c_high + 0.055) / 1.055).powf(2.4);
    assert!((lin_high - expected_high).abs() < 1e-6);

    // Monotonicity: c1 < c2 => linearize(c1) < linearize(c2)
    for i in 0..100 {
        let c1 = i as f64 / 100.0;
        let c2 = (i + 1) as f64 / 100.0;
        assert!(linearize_srgb(c1) <= linearize_srgb(c2), "Linearization must be strictly monotonic");
    }
}

#[test]
fn test_brutal_relative_luminance_and_contrast_ratio_symmetry() {
    let white = Color::rgb(255, 255, 255);
    let black = Color::rgb(0, 0, 0);
    let mid_gray = Color::rgb(128, 128, 128);

    assert_eq!(relative_luminance(&white), 1.0);
    assert_eq!(relative_luminance(&black), 0.0);
    let gray_lum = relative_luminance(&mid_gray);
    assert!(gray_lum > 0.0 && gray_lum < 1.0);

    // Contrast ratio extremes
    let max_ratio = contrast_ratio(&white, &black);
    assert!((max_ratio - 21.0).abs() < 1e-2, "White to black contrast ratio must be exactly 21:1");

    let min_ratio = contrast_ratio(&white, &white);
    assert!((min_ratio - 1.0).abs() < 1e-2, "White to white contrast ratio must be 1:1");

    // Symmetry check: contrast_ratio(A, B) == contrast_ratio(B, A)
    let test_colors = [
        Color::rgb(255, 255, 255),
        Color::rgb(0, 0, 0),
        Color::rgb(94, 106, 210),   // Linear indigo
        Color::rgb(0, 113, 227),    // Apple blue
        Color::rgb(99, 91, 255),    // Stripe blurple
        Color::rgb(252, 238, 10),   // Cyberpunk yellow
        Color::rgb(136, 192, 208),  // Nord frost blue
    ];

    for c1 in &test_colors {
        for c2 in &test_colors {
            let r1 = contrast_ratio(c1, c2);
            let r2 = contrast_ratio(c2, c1);
            assert!((r1 - r2).abs() < 1e-4, "Contrast ratio must be perfectly symmetric");
            assert!(r1 >= 1.0 && r1 <= 21.0, "Contrast ratio must stay within [1.0, 21.0]");
        }
    }
}

#[test]
fn test_brutal_wcag_contrast_report_and_accessible_suggestions() {
    let bg_dark = Color::rgb(8, 9, 10); // Very dark
    let fg_low_contrast = Color::rgb(30, 32, 40); // Too low contrast

    let report = evaluate_contrast(&fg_low_contrast, &bg_dark);
    assert_eq!(report.grade, WcagGrade::Fail);
    assert!(!report.normal_text_aa);
    assert!(!report.normal_text_aaa);
    assert!(report.suggested_foreground.is_some(), "Engine must generate accessible suggestion");

    let suggested = report.suggested_foreground.unwrap();
    let suggested_ratio = contrast_ratio(&suggested, &bg_dark);
    assert!(
        suggested_ratio >= 4.5,
        "Suggested color must achieve at least 4.5:1 ratio (got {:.2}:1)",
        suggested_ratio
    );

    // Light background scenario
    let bg_light = Color::rgb(255, 255, 255);
    let fg_light_yellow = Color::rgb(240, 230, 140); // Low contrast yellow on white
    let report_light = evaluate_contrast(&fg_light_yellow, &bg_light);
    assert_eq!(report_light.grade, WcagGrade::Fail);
    assert!(!report_light.normal_text_aa);

    if let Some(sugg) = report_light.suggested_foreground {
        let new_ratio = contrast_ratio(&sugg, &bg_light);
        assert!(new_ratio >= 4.5, "Suggested darker color must meet 4.5:1 ratio against white");
    }
}

#[test]
fn test_brutal_static_linter_polyglot_scenarios() {
    let linter = UxLinter::new();

    // 1. Catches multi-line JSX icon buttons without aria-label
    let bad_jsx = r#"
        export function Nav() {
            return (
                <nav className="flex items-center">
                    <button className="p-2 rounded hover:bg-gray-100">
                        <svg className="w-5 h-5 text-gray-500" fill="none" viewBox="0 0 24 24">
                            <path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                        </svg>
                    </button>
                    <IconButton className="ml-2">
                        <SearchIcon size={20} />
                    </IconButton>
                </nav>
            );
        }
    "#;
    let v_jsx = linter.lint_source("Nav.tsx", bad_jsx);
    let a11y_icons: Vec<_> = v_jsx.iter().filter(|v| v.rule_id == "UX-A11Y-001").collect();
    assert_eq!(a11y_icons.len(), 2, "Must catch both icon buttons missing accessible names");

    // 2. Catches div and span click handlers without keyboard accessibility
    let bad_clicks = r#"
        <div onClick={() => submit()} className="btn">Submit</div>
        <span @click="toggleMenu" class="dropdown-trigger">Menu</span>
        <div on:click={handleClick}>Svelte Div</div>
    "#;
    let v_clicks = linter.lint_source("Dropdown.vue", bad_clicks);
    let click_violations: Vec<_> = v_clicks.iter().filter(|v| v.rule_id == "UX-A11Y-002").collect();
    assert_eq!(click_violations.len(), 3, "Must catch all inaccessible click handlers");

    // 3. Catches missing alt on images
    let bad_imgs = r#"
        <img src="/logo.svg" className="h-8" />
        <Image src={user.avatar} className="avatar" />
    "#;
    let v_imgs = linter.lint_source("Header.tsx", bad_imgs);
    let img_violations: Vec<_> = v_imgs.iter().filter(|v| v.rule_id == "UX-A11Y-003").collect();
    assert_eq!(img_violations.len(), 2, "Must catch both images without alt attribute");

    // 4. Catches buttons missing explicit type="button" in forms
    let bad_buttons = r#"
        <form onSubmit={handleSubmit}>
            <button className="bg-blue-500">Save</button>
            <button className="bg-gray-200">Cancel</button>
        </form>
    "#;
    let v_buttons = linter.lint_source("Form.tsx", bad_buttons);
    let btn_violations: Vec<_> = v_buttons.iter().filter(|v| v.rule_id == "UX-FORMS-001").collect();
    assert_eq!(btn_violations.len(), 2, "Must catch buttons lacking explicit type");

    // 5. Catches 8-point spatial grid violations in Tailwind and CSS
    let bad_grid = r#"
        <div className="p-[7px] m-[13px] gap-[9px] top-[23px]">
            <section style="margin: 15px; padding: 21px; gap: 7px;"></section>
        </div>
    "#;
    let v_grid = linter.lint_source("Layout.html", bad_grid);
    let grid_violations: Vec<_> = v_grid.iter().filter(|v| v.rule_id == "UX-GRID-001").collect();
    assert!(grid_violations.len() >= 4, "Must catch arbitrary non-8pt pixel values");

    // 6. Catches hardcoded hex colors
    let bad_tokens = r#"
        <div className="bg-[#123456] text-[#abcdef] border-[#ff0033]">
            <p style="color: #ffaa00; background-color: #000000;"></p>
        </div>
    "#;
    let v_tokens = linter.lint_source("Theme.tsx", bad_tokens);
    let token_violations: Vec<_> = v_tokens.iter().filter(|v| v.rule_id == "UX-TOKEN-001").collect();
    assert!(token_violations.len() >= 4, "Must catch hardcoded hex colors");

    // 7. Catches unsized media elements (CLS vulnerability)
    let bad_cls = r#"
        <img src="/hero.jpg" alt="Hero banner" />
        <video src="/demo.mp4" controls></video>
        <iframe src="https://example.com/embed"></iframe>
    "#;
    let v_cls = linter.lint_source("Page.html", bad_cls);
    let cls_violations: Vec<_> = v_cls.iter().filter(|v| v.rule_id == "UX-CLS-001").collect();
    assert_eq!(cls_violations.len(), 3, "Must catch unsized media elements");
}

#[test]
fn test_brutal_clean_code_zero_violations() {
    let linter = UxLinter::new();

    let clean_code = r#"
        export function AccessibleHeader() {
            return (
                <header className="flex items-center p-4 gap-4">
                    <img src="/logo.svg" alt="Tagisan Logo" width={120} height={40} />
                    <button type="button" aria-label="Search documents" className="p-2 rounded bg-primary text-primary-foreground">
                        <SearchIcon size={16} />
                    </button>
                    <button type="button" className="px-4 py-2 bg-secondary text-secondary-foreground rounded">
                        Settings
                    </button>
                </header>
            );
        }
    "#;

    let violations = linter.lint_source("CleanHeader.tsx", clean_code);
    assert!(violations.is_empty(), "Clean, accessible code must generate 0 violations, found {:?}", violations);
}

#[test]
fn test_brutal_design_tokens_all_presets_export_formats() {
    let presets = ["linear", "apple", "stripe", "cyberpunk", "nord"];

    for name in &presets {
        let preset = DesignTokenPreset::get_by_name(name);
        assert!(preset.is_some(), "Preset '{}' must exist in registry", name);
        let p = preset.unwrap();

        // 1. Tailwind config validation
        let tailwind = p.export_tailwind();
        assert!(tailwind.contains("module.exports = {"), "Must export valid CommonJS module");
        assert!(tailwind.contains("colors: {"), "Must configure theme colors");
        assert!(tailwind.contains("spacing: {"), "Must configure 8pt spacing scale");

        // 2. CSS variables validation
        let css = p.export_css_variables();
        assert!(css.contains(":root {"), "Must define :root variables");
        assert!(css.contains(".dark {"), "Must define .dark variables");
        assert!(css.contains("--primary:"), "Must declare primary color token");
        assert!(css.contains("--background:"), "Must declare background color token");

        // 3. W3C DTCG standard JSON tokens validation
        let json = p.export_json_tokens();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Exported JSON must be valid JSON");
        assert!(parsed.get("color").is_some(), "DTCG JSON must have top-level 'color' group");
        assert!(parsed["color"].get("primary").is_some(), "DTCG JSON must define 'primary' token");
        assert_eq!(parsed["color"]["primary"]["$type"], "color", "Must comply with W3C DTCG format");
    }
}

#[test]
fn test_brutal_readiness_scoring_engine_and_verdicts() {
    // Perfect state -> 100% Ship Ready
    let empty_violations: Vec<UxLintViolation> = vec![];
    let score_clean = ReadinessScoringEngine::evaluate(&empty_violations);
    assert_eq!(score_clean.total_score, 100);
    assert_eq!(score_clean.status, ReadinessStatus::ShipReady);
    assert!(score_clean.status.badge().contains("SHIP READY"));

    // Critical accessibility error -> Blocked for release
    let critical_violations = vec![
        UxLintViolation {
            rule_id: "UX-A11Y-001".to_string(),
            rule_name: "Icon button missing accessible label".to_string(),
            severity: ViolationSeverity::Error,
            category: UxCategory::Accessibility,
            file_path: "App.tsx".to_string(),
            line_number: 14,
            line_content: "<button><svg /></button>".to_string(),
            message: "Missing aria-label".to_string(),
            suggestion: "Add aria-label".to_string(),
        }
    ];
    let score_crit = ReadinessScoringEngine::evaluate(&critical_violations);
    assert_eq!(score_crit.status, ReadinessStatus::BlockedForRelease, "Any critical error must block release");

    // Warnings only -> Minor polish
    let minor_violations = vec![
        UxLintViolation {
            rule_id: "UX-GRID-001".to_string(),
            rule_name: "Arbitrary spacing".to_string(),
            severity: ViolationSeverity::Warning,
            category: UxCategory::LayoutGrid,
            file_path: "Card.tsx".to_string(),
            line_number: 5,
            line_content: "<div className=\"p-[7px]\">".to_string(),
            message: "Arbitrary 7px".to_string(),
            suggestion: "Use 8px".to_string(),
        }
    ];
    let score_minor = ReadinessScoringEngine::evaluate(&minor_violations);
    assert!(score_minor.total_score >= 90);
    assert_eq!(score_minor.status, ReadinessStatus::ShipReady);
}

#[test]
fn test_brutal_500_ux_rules_catalog_completeness() {
    let catalog = UxCatalog::new();
    assert_eq!(catalog.rules.len(), 500, "Catalog must contain exactly 500 rules");

    let clusters = [
        ("a11y", 45),
        ("typo", 40),
        ("color", 40),
        ("grid", 40),
        ("nav", 40),
        ("form", 45),
        ("ctrl", 40),
        ("feed", 40),
        ("moto", 40),
        ("perf", 45),
        ("copy", 40),
        ("genai", 45),
    ];

    let mut total_counted = 0;
    for (cluster_name, expected_count) in &clusters {
        let cluster_enum = UxCluster::from_str(cluster_name).unwrap();
        let rules_in_cluster = catalog.query_by_cluster(cluster_enum);
        assert_eq!(
            rules_in_cluster.len(),
            *expected_count,
            "Cluster '{}' must have exactly {} rules (got {})",
            cluster_name,
            expected_count,
            rules_in_cluster.len()
        );
        total_counted += rules_in_cluster.len();
    }
    assert_eq!(total_counted, 500, "Sum of all clusters must equal 500");

    // Search query capability
    let contrast_rules = catalog.search("contrast");
    assert!(!contrast_rules.is_empty(), "Search for 'contrast' must yield relevant rules");

    let touch_target_rules = catalog.search("touch target");
    assert!(!touch_target_rules.is_empty(), "Search for 'touch target' must yield relevant rules");

    // Lookup by ID
    let r1 = catalog.get_rule("UX-A11Y-001");
    assert!(r1.is_some(), "Must retrieve UX-A11Y-001 by ID");
    assert_eq!(r1.unwrap().cluster, UxCluster::Accessibility);

    let r_genai = catalog.get_rule("UX-GENAI-001");
    assert!(r_genai.is_some(), "Must retrieve UX-GENAI-001 by ID");
    assert_eq!(r_genai.unwrap().cluster, UxCluster::GenAIInterfaces);
}

#[test]
fn test_brutal_ecc_skill_registration() {
    let all_skills = all_built_in_skills();
    assert!(
        all_skills.iter().any(|s| s.name == "ui-ux-pro-max"),
        "ui-ux-pro-max must be present in all_built_in_skills()"
    );

    let skill_opt = find_built_in_skill("ui-ux-pro-max");
    assert!(skill_opt.is_some(), "find_built_in_skill('ui-ux-pro-max') must return Some");

    let skill = skill_opt.unwrap();
    assert_eq!(skill.name, "ui-ux-pro-max");
    assert!(!skill.description.is_empty());
    assert!(skill.instructions.contains("WCAG 2.2"));
    assert!(skill.instructions.contains("500 UX Rules"));
    assert!(skill.instructions.contains("Linear"));
}
