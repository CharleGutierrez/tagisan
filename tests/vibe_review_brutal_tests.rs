use serde_json::{json, Value};
use std::collections::HashSet;
use tagisan::ecc::{all_presets, find_preset, EccSkill};
use tagisan::tools::{ToolHandler, VibeCodeReviewTool};
use tagisan::vibe_review::{audit_code, FindingSeverity, ReviewCatalog, ReviewCluster};

#[test]
fn test_brutal_catalog_integrity_100_books() {
    let catalog = ReviewCatalog::new();
    assert_eq!(catalog.books.len(), 100, "Catalog must contain exactly 100 books");

    let mut ids = HashSet::new();
    let mut titles = HashSet::new();

    for book in &catalog.books {
        assert!(book.id.starts_with("VBR-"), "ID '{}' must start with 'VBR-'", book.id);
        assert!(!book.title.trim().is_empty(), "Book '{}' must have non-empty title", book.id);
        assert!(!book.authors.trim().is_empty(), "Book '{}' must have authors", book.id);
        assert!(book.year >= 1970 && book.year <= 2030, "Book '{}' year invalid: {}", book.id, book.year);
        assert!(!book.literature_heuristic.trim().is_empty(), "Book '{}' heuristic missing", book.id);
        assert!(!book.review_invariants.is_empty(), "Book '{}' must have review invariants", book.id);
        assert!(!book.ai_hallucination_defense.trim().is_empty(), "Book '{}' defense missing", book.id);

        assert!(ids.insert(book.id.clone()), "Duplicate book ID: {}", book.id);
        assert!(titles.insert(book.title.clone()), "Duplicate book title: {}", book.title);
    }
}

#[test]
fn test_brutal_cluster_distribution() {
    let catalog = ReviewCatalog::new();
    assert_eq!(ReviewCluster::all().len(), 10, "Must have exactly 10 clusters");

    for cluster in ReviewCluster::all() {
        let books = catalog.by_cluster(*cluster);
        assert_eq!(books.len(), 10, "Cluster {:?} must have exactly 10 books", cluster);
        assert!(!cluster.code().is_empty());
        assert!(!cluster.title().is_empty());
        assert!(!cluster.canonical_authors().is_empty());

        let parsed = ReviewCluster::from_str(cluster.code());
        assert_eq!(parsed, Some(*cluster), "Cluster code must parse back: {}", cluster.code());
    }
}

#[test]
fn test_brutal_inverted_search() {
    let catalog = ReviewCatalog::new();

    // 1. Search Ousterhout
    let ousterhout_hits = catalog.search("ousterhout", None, 5);
    assert!(!ousterhout_hits.is_empty(), "Search for 'ousterhout' must yield results");
    assert!(ousterhout_hits.iter().any(|b| b.id == "VBR-02-001"));

    // 2. Search Dowd (Security)
    let dowd_hits = catalog.search("dowd", None, 5);
    assert!(!dowd_hits.is_empty(), "Search for 'dowd' must yield results");
    assert!(dowd_hits.iter().any(|b| b.id == "VBR-04-001"));

    // 3. Search Concurrency / Goetz
    let goetz_hits = catalog.search("goetz", None, 5);
    assert!(!goetz_hits.is_empty(), "Search for 'goetz' must yield results");
    assert!(goetz_hits.iter().any(|b| b.id == "VBR-06-001"));

    // 4. Search Kleppmann / DDIA
    let kleppmann_hits = catalog.search("kleppmann", None, 5);
    assert!(!kleppmann_hits.is_empty(), "Search for 'kleppmann' must yield results");
    assert!(kleppmann_hits.iter().any(|b| b.id == "VBR-05-001"));

    // 5. Search Khorikov (Unit Testing)
    let khorikov_hits = catalog.search("khorikov", None, 5);
    assert!(!khorikov_hits.is_empty(), "Search for 'khorikov' must yield results");
    assert!(khorikov_hits.iter().any(|b| b.id == "VBR-07-001"));

    // 6. Search with Cluster Filter
    let filtered_hits = catalog.search("refactoring", Some(ReviewCluster::SmellsRefactoring), 5);
    assert!(!filtered_hits.is_empty());
    assert!(filtered_hits.iter().all(|b| b.cluster == ReviewCluster::SmellsRefactoring));
}

#[test]
fn test_brutal_code_auditor_findings() {
    let malicious_vulnerable_code = r#"
        use std::process::Command;

        pub fn handle_request(user_input: &str, file_name: &str) {
            // 1. SQL injection
            let query = format!("SELECT * FROM users WHERE username = '{}'", user_input);

            // 2. Command injection
            let output = Command::new("sh").arg("-c").arg(user_input).output();

            // 3. Hardcoded secret
            let api_key = "sk_live_1234567890abcdef123456";

            // 4. Wildcard CORS
            let cors = "Access-Control-Allow-Origin: '*'";

            // 5. Unchecked unwrap in production
            let parsed: u32 = user_input.parse().unwrap();
        }

        #[test]
        fn test_vanity() {
            assert!(true);
        }
    "#;

    let report = audit_code(malicious_vulnerable_code, Some("src/handler.rs"));
    assert!(!report.summary.passed, "Vulnerable code must fail audit");
    assert!(report.summary.critical_count >= 3, "Must flag multiple critical findings");

    let rule_ids: Vec<&str> = report.findings.iter().map(|f| f.rule_id.as_str()).collect();
    assert!(rule_ids.contains(&"DOWD-SQL-INJECTION-RISK"));
    assert!(rule_ids.contains(&"DOWD-COMMAND-INJECTION-RISK"));
    assert!(rule_ids.contains(&"JANCA-HARDCODED-SECRET"));
    assert!(rule_ids.contains(&"ZALEWSKI-WILDCARD-CORS"));
    assert!(rule_ids.contains(&"KLABNIK-UNCHECKED-UNWRAP"));
    assert!(rule_ids.contains(&"MESZAROS-TAUTOLOGICAL-TEST"));
}

#[test]
fn test_brutal_code_auditor_clean_code() {
    let clean_code = r#"
        use std::time::Duration;

        pub fn safe_division(numerator: i64, denominator: i64) -> Result<i64, String> {
            if denominator == 0 {
                return Err("denominator cannot be zero".into());
            }
            Ok(numerator / denominator)
        }

        #[test]
        fn test_division_valid() {
            let res = safe_division(10, 2);
            assert_eq!(res, Ok(5));
        }
    "#;

    let report = audit_code(clean_code, Some("src/safe.rs"));
    assert!(report.summary.passed, "Clean code must pass audit");
    assert_eq!(report.summary.critical_count, 0);
}

#[tokio::test]
async fn test_brutal_vibe_code_review_tool_execution() {
    let tool = VibeCodeReviewTool::new();
    assert_eq!(tool.name(), "vibe_code_review");

    // 1. Search Action
    let res = tool.execute(json!({
        "action": "search",
        "query": "shallow modules"
    })).await;
    assert!(res.is_ok(), "Search execution must succeed: {:?}", res);
    let parsed: Value = serde_json::from_str(&res.unwrap()).unwrap();
    assert!(parsed.is_array());
    assert!(!parsed.as_array().unwrap().is_empty());

    // 2. Get Book Action
    let res = tool.execute(json!({
        "action": "get_book",
        "book_id": "VBR-02-001"
    })).await;
    assert!(res.is_ok());
    let parsed: Value = serde_json::from_str(&res.unwrap()).unwrap();
    assert_eq!(parsed["id"], "VBR-02-001");
    assert_eq!(parsed["title"], "A Philosophy of Software Design (2nd Ed)");

    // 3. Playbook Action
    let res = tool.execute(json!({
        "action": "playbook",
        "stage": 1
    })).await;
    assert!(res.is_ok());
    let playbook_text = res.unwrap();
    assert!(playbook_text.contains("Stage 1: The Eyeball Filter"));
    assert!(playbook_text.contains("Ousterhout"));

    // 4. List Clusters Action
    let res = tool.execute(json!({
        "action": "list_clusters"
    })).await;
    assert!(res.is_ok());
    let clusters: Value = serde_json::from_str(&res.unwrap()).unwrap();
    assert_eq!(clusters.as_array().unwrap().len(), 10);

    // 5. Audit Code Action
    let bad_snippet = "let q = format!(\"SELECT * FROM users WHERE id = '{}'\", uid);";
    let res = tool.execute(json!({
        "action": "audit",
        "code": bad_snippet
    })).await;
    assert!(res.is_ok());
    let audit_res: Value = serde_json::from_str(&res.unwrap()).unwrap();
    assert_eq!(audit_res["summary"]["passed"], false);
    assert!(audit_res["summary"]["critical_count"].as_u64().unwrap() >= 1);
}

#[test]
fn test_brutal_ecc_vibe_reviewer_preset_and_skill() {
    // 1. Check ECC Preset
    let presets = all_presets();
    assert!(presets.iter().any(|p| p.name == "vibe-code-reviewer"));

    let reviewer = find_preset("vibe-code-reviewer").expect("vibe-code-reviewer preset must exist");
    assert!(reviewer.tools.contains(&"vibe_code_review".to_string()));
    assert!(reviewer.system_prompt.contains("100-Book Code Review Canon"));

    // 2. Check SKILL.md file parses cleanly
    let skill = EccSkill::from_file("assets/skills/vibe-code-review-pro-max/SKILL.md")
        .expect("assets/skills/vibe-code-review-pro-max/SKILL.md must be a valid ECC skill");
    assert_eq!(skill.name, "vibe-code-review-pro-max");
    assert!(skill.description.contains("Top 100 Code and Program Review Books"));
    assert!(skill.instructions.contains("VBR-01"));
    assert!(skill.instructions.contains("5-Layer Inspection Checklist"));
}
