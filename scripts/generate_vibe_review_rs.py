import json
import os

OUTPUT_RS = "/home/dyna/TGS Projects/tagisan/src/vibe_review.rs"

# Read the raw books definition
with open("scripts/generate_vibe_review_rs.py", "r") as f:
    orig = f.read()

# Extract raw_books
start_marker = "raw_books = ["
end_marker = "assert len(raw_books) == 100"
books_chunk = orig[orig.find(start_marker):orig.find(end_marker)]
exec(books_chunk)

print(f"Loaded {len(raw_books)} books")

header = r"""//! Tagisan Sovereign Vibe Code Review Engine (`tgs review`)
//!
//! Enforces the Top 100 Code and Program Review Books canon across 10 macro-inspection clusters.
//! Provides automated 5-layer static code auditing, sub-millisecond inverted search index,
//! and CLI inspection workflows tailored for Vibe Coders and autonomous AI swarms.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

// ===========================================================================
// 1. MACRO-INSPECTION CLUSTERS (10 CLUSTERS, 100 TOTAL BOOKS)
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewCluster {
    /// VBR-01: Peer Code Review & Formal Inspection (10 books)
    PeerReviewInspection,
    /// VBR-02: Code Smells, Refactoring & Readability (10 books)
    SmellsRefactoring,
    /// VBR-03: Software Construction & Craftsmanship (10 books)
    CraftsmanshipConstruction,
    /// VBR-04: Security Auditing & Defensive Coding (10 books)
    SecurityAuditingAppSec,
    /// VBR-05: Architecture & System Design Evaluation (10 books)
    ArchitectureSystemDesign,
    /// VBR-06: Concurrency, Multithreading & Invariants (10 books)
    ConcurrencyInvariants,
    /// VBR-07: Testing, Verification & Test Smells (10 books)
    TestingVerification,
    /// VBR-08: Debugging, Profiling & Systems Performance (10 books)
    PerformanceSystemsMemory,
    /// VBR-09: Language-Specific Review Bibles (10 books)
    LanguageSpecificBibles,
    /// VBR-10: Engineering Mindset & AI-Native Review (10 books)
    MindsetAiNativeReview,
}

impl ReviewCluster {
    pub fn all() -> &'static [ReviewCluster] {
        &[
            ReviewCluster::PeerReviewInspection,
            ReviewCluster::SmellsRefactoring,
            ReviewCluster::CraftsmanshipConstruction,
            ReviewCluster::SecurityAuditingAppSec,
            ReviewCluster::ArchitectureSystemDesign,
            ReviewCluster::ConcurrencyInvariants,
            ReviewCluster::TestingVerification,
            ReviewCluster::PerformanceSystemsMemory,
            ReviewCluster::LanguageSpecificBibles,
            ReviewCluster::MindsetAiNativeReview,
        ]
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::PeerReviewInspection => "VBR-01",
            Self::SmellsRefactoring => "VBR-02",
            Self::CraftsmanshipConstruction => "VBR-03",
            Self::SecurityAuditingAppSec => "VBR-04",
            Self::ArchitectureSystemDesign => "VBR-05",
            Self::ConcurrencyInvariants => "VBR-06",
            Self::TestingVerification => "VBR-07",
            Self::PerformanceSystemsMemory => "VBR-08",
            Self::LanguageSpecificBibles => "VBR-09",
            Self::MindsetAiNativeReview => "VBR-10",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::PeerReviewInspection => "Peer Code Review & Formal Inspection",
            Self::SmellsRefactoring => "Code Smells, Refactoring & Readability",
            Self::CraftsmanshipConstruction => "Software Construction & Craftsmanship",
            Self::SecurityAuditingAppSec => "Security Auditing & Defensive Coding",
            Self::ArchitectureSystemDesign => "Architecture & System Design Evaluation",
            Self::ConcurrencyInvariants => "Concurrency, Multithreading & Invariants",
            Self::TestingVerification => "Testing, Verification & Test Smells",
            Self::PerformanceSystemsMemory => "Debugging, Profiling & Systems Performance",
            Self::LanguageSpecificBibles => "Language-Specific Review Bibles",
            Self::MindsetAiNativeReview => "Engineering Mindset & AI-Native Review",
        }
    }

    pub fn canonical_authors(&self) -> &'static str {
        match self {
            Self::PeerReviewInspection => "Wiegers, Winters (Google), Cohen, Gilb, Gee, Radice",
            Self::SmellsRefactoring => "Ousterhout, Fowler, Beck, Feathers, Boswell, Kerievsky",
            Self::CraftsmanshipConstruction => "McConnell, Thomas, Hunt, Kernighan, Pike, Martin",
            Self::SecurityAuditingAppSec => "Dowd, McDonald, Schuh, Howard, LeBlanc, Zalewski, Shostack",
            Self::ArchitectureSystemDesign => "Kleppmann, Richards, Ford, Evans, Newman, Nygard",
            Self::ConcurrencyInvariants => "Goetz, McKenney, Herlihy, Shavit, Cox-Buday, Lamport",
            Self::TestingVerification => "Khorikov, Meszaros, Beck, Freeman, Pryce, Hébert",
            Self::PerformanceSystemsMemory => "Gregg, Bryant, O'Hallaron, Kerrisk, Agans, Zeller",
            Self::LanguageSpecificBibles => "Bloch, Meyers, Blandy, Orendorff, Slatkin, Ramalho, Vanderkam",
            Self::MindsetAiNativeReview => "Brooks, Gamma, Helm, Johnson, Vlissides, Beyer, Geewax",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "vbr-01" | "01" | "1" | "peer" | "inspection" | "critique" => Some(Self::PeerReviewInspection),
            "vbr-02" | "02" | "2" | "smell" | "smells" | "refactor" | "refactoring" | "ousterhout" | "fowler" => Some(Self::SmellsRefactoring),
            "vbr-03" | "03" | "3" | "craft" | "craftsmanship" | "construction" | "pragmatic" | "mcconnell" => Some(Self::CraftsmanshipConstruction),
            "vbr-04" | "04" | "4" | "security" | "appsec" | "audit" | "vulnerability" | "dowd" | "zalewski" => Some(Self::SecurityAuditingAppSec),
            "vbr-05" | "05" | "5" | "arch" | "architecture" | "system" | "systems" | "kleppmann" | "ddia" => Some(Self::ArchitectureSystemDesign),
            "vbr-06" | "06" | "6" | "concurrency" | "threads" | "multithread" | "race" | "goetz" | "tla" => Some(Self::ConcurrencyInvariants),
            "vbr-07" | "07" | "7" | "test" | "testing" | "verification" | "tdd" | "khorikov" | "property" => Some(Self::TestingVerification),
            "vbr-08" | "08" | "8" | "perf" | "performance" | "debug" | "debugging" | "gregg" | "memory" => Some(Self::PerformanceSystemsMemory),
            "vbr-09" | "09" | "9" | "lang" | "language" | "rust" | "python" | "cpp" | "java" | "go" | "ts" => Some(Self::LanguageSpecificBibles),
            "vbr-10" | "10" | "mindset" | "sre" | "api" | "agent" | "agents" | "brooks" | "ai-native" => Some(Self::MindsetAiNativeReview),
            _ => None,
        }
    }
}

// ===========================================================================
// 2. REVIEW BOOK SKILL DEFINITION
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewBookSkill {
    pub id: String,
    pub cluster: ReviewCluster,
    pub title: String,
    pub authors: String,
    pub year: u32,
    pub literature_heuristic: String,
    pub review_invariants: Vec<String>,
    pub ai_hallucination_defense: String,
    pub tools_required: Vec<String>,
}

// ===========================================================================
// 3. SOVEREIGN 100-BOOK CATALOG & INVERTED SEARCH ENGINE
// ===========================================================================

pub struct ReviewCatalog {
    pub books: Vec<ReviewBookSkill>,
    index: HashMap<String, Vec<usize>>,
}

impl Default for ReviewCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewCatalog {
    pub fn new() -> Self {
        let books = Self::init_all_books();
        let mut index: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, book) in books.iter().enumerate() {
            let mut terms: HashSet<String> = HashSet::new();

            let tokenize = |text: &str| -> Vec<String> {
                text.to_lowercase()
                    .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                    .filter(|s| s.len() >= 3)
                    .map(|s| s.to_string())
                    .collect()
            };

            for term in tokenize(&book.id) { terms.insert(term); }
            for term in tokenize(&book.title) { terms.insert(term); }
            for term in tokenize(&book.authors) { terms.insert(term); }
            for term in tokenize(&book.literature_heuristic) { terms.insert(term); }
            for term in tokenize(&book.ai_hallucination_defense) { terms.insert(term); }
            for inv in &book.review_invariants {
                for term in tokenize(inv) { terms.insert(term); }
            }

            for term in terms {
                index.entry(term).or_default().push(idx);
            }
        }

        Self { books, index }
    }

    pub fn search(&self, query: &str, cluster: Option<ReviewCluster>, limit: usize) -> Vec<&ReviewBookSkill> {
        let clean = query.trim().to_lowercase();
        let query_tokens: Vec<&str> = clean
            .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .filter(|s| s.len() >= 2)
            .collect();

        if query_tokens.is_empty() {
            return match cluster {
                Some(c) => self.by_cluster(c).into_iter().take(limit).collect(),
                None => self.books.iter().take(limit).collect(),
            };
        }

        let mut score_map: HashMap<usize, usize> = HashMap::new();

        for token in &query_tokens {
            if let Some(hits) = self.index.get(*token) {
                for &idx in hits {
                    *score_map.entry(idx).or_insert(0) += 10;
                }
            } else {
                for (term, hits) in &self.index {
                    if term.contains(token) {
                        for &idx in hits {
                            *score_map.entry(idx).or_insert(0) += 3;
                        }
                    }
                }
            }
        }

        let mut scored: Vec<(usize, &ReviewBookSkill)> = score_map
            .into_iter()
            .map(|(idx, score)| (score, &self.books[idx]))
            .filter(|(_, book)| match cluster {
                Some(c) => book.cluster == c,
                None => true,
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, b)| b).take(limit).collect()
    }

    pub fn get(&self, id: &str) -> Option<&ReviewBookSkill> {
        let clean = id.trim().to_uppercase();
        self.books.iter().find(|b| b.id.to_uppercase() == clean)
    }

    pub fn by_cluster(&self, cluster: ReviewCluster) -> Vec<&ReviewBookSkill> {
        self.books.iter().filter(|b| b.cluster == cluster).collect()
    }

    pub fn export_markdown(&self, cluster: Option<ReviewCluster>) -> String {
        let mut out = String::new();
        out.push_str("# Tagisan Top 100 Code & Program Review Books\n\n");
        let pool: Vec<&ReviewBookSkill> = match cluster {
            Some(c) => self.by_cluster(c),
            None => self.books.iter().collect(),
        };

        out.push_str(&format!("**Total Books Exported: {}**\n\n", pool.len()));
        out.push_str("| ID | Title | Author(s) | Year | Primary Review Invariant | AI Hallucination Defense |\n");
        out.push_str("|---|---|---|---|---|---|\n");

        for b in pool {
            let inv = b.review_invariants.first().map(|s| s.as_str()).unwrap_or("");
            out.push_str(&format!(
                "| `{}` | **{}** | {} | {} | {} | {} |\n",
                b.id, b.title, b.authors, b.year, inv, b.ai_hallucination_defense
            ));
        }

        out
    }

    pub fn export_json(&self, cluster: Option<ReviewCluster>) -> Result<String> {
        let pool: Vec<&ReviewBookSkill> = match cluster {
            Some(c) => self.by_cluster(c),
            None => self.books.iter().collect(),
        };
        serde_json::to_string_pretty(&pool).map_err(TagisanError::from)
    }

    pub fn playbook(&self, stage: Option<usize>) -> String {
        match stage {
            Some(1) => r#"# Stage 1: The Eyeball Filter (Readability & Smells)
Focus Books:
- VBR-02-001: A Philosophy of Software Design (Ousterhout) - Eliminate shallow wrappers!
- VBR-02-002: Refactoring (Fowler & Beck) - Detect Code Smells.
- VBR-03-001: Code Complete (McConnell) - Variable scoping and defensive coding.
Heuristic: Reject any AI PR where methods exceed 25 lines or where functions merely forward identical arguments."#.to_string(),
            Some(2) => r#"# Stage 2: The Security Shield (AppSec & Boundaries)
Focus Books:
- VBR-04-001: The Art of Software Security Assessment (Dowd, McDonald, Schuh) - Integer wraps & memory safety.
- VBR-04-003: The Tangled Web (Zalewski) - CORS, cookies, and SOP.
- VBR-04-005: Secure by Design (Johnsson, Deogun, Sawano) - Strong domain types.
Heuristic: Never merge code with raw string concatenation in SQL or shell commands. Enforce domain types."#.to_string(),
            Some(3) => r#"# Stage 3: The Verification Cage (Adversarial Testing)
Focus Books:
- VBR-07-001: Unit Testing Principles, Practices, and Patterns (Khorikov) - Verify state invariants.
- VBR-07-002: xUnit Test Patterns (Meszaros) - Spot vanity tests with no assertions.
- VBR-07-005: Property-Based Testing (Hébert) - Randomized invariant testing.
Heuristic: Reject PRs with mock-heavy vanity tests. Demand property-based edge-case tests."#.to_string(),
            Some(4) => r#"# Stage 4: Deep Systems & Concurrency
Focus Books:
- VBR-05-001: Designing Data-Intensive Applications (Kleppmann) - Distributed state and consensus.
- VBR-06-001: Java Concurrency in Practice (Goetz et al.) - Thread safety & memory publication.
- VBR-08-004: The Linux Programming Interface (Kerrisk) - Syscall safety and file descriptor leakage.
Heuristic: Verify all locks, unawaited futures, and network timeouts."#.to_string(),
            Some(5) => r#"# Stage 5: Macro-Architecture & Longevity
Focus Books:
- VBR-01-002: Software Engineering at Google (Winters et al.) - Long-term maintainability.
- VBR-05-002: Fundamentals of Software Architecture (Richards & Ford) - Trade-off analysis.
- VBR-08-001: Systems Performance (Gregg) - USE method and flame graphs.
Heuristic: Ensure the system reflects one unified design philosophy with automated fitness functions."#.to_string(),
            _ => r#"# The Vibe Coder's 5-Stage Reading & Review Path
1. Stage 1: The Eyeball Filter (Ousterhout, Fowler, McConnell) - Eliminate shallow wrapper functions.
2. Stage 2: The Security Shield (Dowd, Zalewski, Johnsson) - Eliminate injection & broken auth.
3. Stage 3: The Verification Cage (Khorikov, Meszaros, Hébert) - Property tests & invariant cages.
4. Stage 4: Deep Systems & Concurrency (Kleppmann, Goetz, Kerrisk) - Race conditions & timeouts.
5. Stage 5: Macro-Architecture & Longevity (Winters, Ford, Gregg) - Conceptual integrity & SLOs.
Run `tgs review playbook --stage <1..5>` for deep dives."#.to_string(),
        }
    }

    fn init_all_books() -> Vec<ReviewBookSkill> {
        vec![
"""

books_entries = []
for b in raw_books:
    id_str = b[0]
    cluster_str = b[1]
    title_str = b[2].replace('"', '\\"')
    authors_str = b[3].replace('"', '\\"')
    year_int = b[4]
    heuristic_str = b[5].replace('"', '\\"')
    invs_formatted = ", ".join(f'"{inv}"'.replace('\\"', '"') for inv in b[6])
    invs_str = f"vec![{invs_formatted}].into_iter().map(String::from).collect()"
    defense_str = b[7].replace('"', '\\"')

    entry = f"""            ReviewBookSkill {{
                id: "{id_str}".to_string(),
                cluster: ReviewCluster::{cluster_str},
                title: "{title_str}".to_string(),
                authors: "{authors_str}".to_string(),
                year: {year_int},
                literature_heuristic: "{heuristic_str}".to_string(),
                review_invariants: {invs_str},
                ai_hallucination_defense: "{defense_str}".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            }},"""
    books_entries.append(entry)

books_block = "\n".join(books_entries)

footer = r"""
        ]
    }
}

// ===========================================================================
// 4. AUTOMATED 5-LAYER STATIC CODE REVIEW AUDITOR
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionLayer {
    Layer1CorrectnessInvariants,
    Layer2BoundaryEdgeCases,
    Layer3SecurityTrustBoundaries,
    Layer4ArchitectureDeepModules,
    Layer5VerificationCage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Critical,
    Warning,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeFinding {
    pub layer: InspectionLayer,
    pub severity: FindingSeverity,
    pub rule_id: String,
    pub canonical_book: String,
    pub message: String,
    pub line_number: Option<usize>,
    pub snippet: Option<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_lines: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub advisory_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeAuditReport {
    pub target: String,
    pub summary: AuditSummary,
    pub findings: Vec<VibeFinding>,
}

/// Automated 5-Layer Code Review Analyzer inspecting code against the 100-book literature canon
pub fn audit_code(code: &str, target_hint: Option<&str>) -> VibeAuditReport {
    let mut findings = Vec::new();
    let lines: Vec<&str> = code.lines().collect();
    let total_lines = lines.len();

    let target = target_hint.unwrap_or("snippet").to_string();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // -------------------------------------------------------------
        // Layer 3: Security & Trust Boundaries (Dowd, Zalewski, Howard)
        // -------------------------------------------------------------
        // SQL Injection String Concatenation
        if (trimmed.contains("SELECT ") || trimmed.contains("INSERT ") || trimmed.contains("UPDATE ") || trimmed.contains("DELETE "))
            && (trimmed.contains(" + ") || trimmed.contains("format!") || trimmed.contains("f\"") || trimmed.contains("`$")) {
            findings.push(VibeFinding {
                layer: InspectionLayer::Layer3SecurityTrustBoundaries,
                severity: FindingSeverity::Critical,
                rule_id: "DOWD-SQL-INJECTION-RISK".to_string(),
                canonical_book: "VBR-04-001 (The Art of Software Security Assessment - Dowd et al.)".to_string(),
                message: "Potential SQL injection via raw string formatting/concatenation.".to_string(),
                line_number: Some(line_num),
                snippet: Some(trimmed.to_string()),
                remediation: "Use parameterized prepared statements with bind variables rather than formatting raw strings.".to_string(),
            });
        }

        // Command Injection
        if (trimmed.contains("Command::new(\"sh\")") || trimmed.contains("Command::new(\"bash\")") || trimmed.contains("os.system(") || trimmed.contains("exec("))
            && (trimmed.contains("-c") || trimmed.contains(".arg(")) {
            findings.push(VibeFinding {
                layer: InspectionLayer::Layer3SecurityTrustBoundaries,
                severity: FindingSeverity::Critical,
                rule_id: "DOWD-COMMAND-INJECTION-RISK".to_string(),
                canonical_book: "VBR-04-002 (Writing Secure Code - Howard & LeBlanc)".to_string(),
                message: "Subprocess shell command execution with dynamic arguments detected.".to_string(),
                line_number: Some(line_num),
                snippet: Some(trimmed.to_string()),
                remediation: "Invoke binaries directly with explicit argument arrays without spawning a shell (sh -c).".to_string(),
            });
        }

        // Wildcard CORS
        if trimmed.contains("Access-Control-Allow-Origin") && (trimmed.contains("'*'") || trimmed.contains("\"*\"")) {
            findings.push(VibeFinding {
                layer: InspectionLayer::Layer3SecurityTrustBoundaries,
                severity: FindingSeverity::Warning,
                rule_id: "ZALEWSKI-WILDCARD-CORS".to_string(),
                canonical_book: "VBR-04-003 (The Tangled Web - Zalewski)".to_string(),
                message: "Wildcard CORS origin ('*') detected in header configuration.".to_string(),
                line_number: Some(line_num),
                snippet: Some(trimmed.to_string()),
                remediation: "Specify an explicit, validated origin allowlist rather than allowing all external domains.".to_string(),
            });
        }

        // Hardcoded Credentials / Secrets
        if (trimmed.to_lowercase().contains("api_key") || trimmed.to_lowercase().contains("secret_key") || trimmed.to_lowercase().contains("password"))
            && (trimmed.contains(" = \"") || trimmed.contains(" = '") || trimmed.contains(": \""))
            && !trimmed.contains("env::") && !trimmed.contains("os.environ") && !trimmed.contains("process.env") {
            let val = trimmed.split(['=', ':']).nth(1).unwrap_or("");
            if val.len() >= 12 && !val.contains("TODO") && !val.contains("example") {
                findings.push(VibeFinding {
                    layer: InspectionLayer::Layer3SecurityTrustBoundaries,
                    severity: FindingSeverity::Critical,
                    rule_id: "JANCA-HARDCODED-SECRET".to_string(),
                    canonical_book: "VBR-04-010 (Alice and Bob Learn Application Security - Janca)".to_string(),
                    message: "Apparent hardcoded secret or API credential token.".to_string(),
                    line_number: Some(line_num),
                    snippet: Some(trimmed.to_string()),
                    remediation: "Load credentials securely from environment variables, secret managers, or key vaults.".to_string(),
                });
            }
        }

        // -------------------------------------------------------------
        // Layer 1: Correctness & Invariants (Ousterhout, Maguire, Klabnik)
        // -------------------------------------------------------------
        // Dangerous unwrap in production path
        if trimmed.contains(".unwrap()") && !target.contains("test") && !target.contains("spec") {
            findings.push(VibeFinding {
                layer: InspectionLayer::Layer1CorrectnessInvariants,
                severity: FindingSeverity::Warning,
                rule_id: "KLABNIK-UNCHECKED-UNWRAP".to_string(),
                canonical_book: "VBR-09-005 (The Rust Programming Language - Klabnik & Nichols)".to_string(),
                message: "Unchecked .unwrap() call in production code path can cause runtime panics.".to_string(),
                line_number: Some(line_num),
                snippet: Some(trimmed.to_string()),
                remediation: "Use pattern matching, if let, or propagate errors cleanly using the ? operator.".to_string(),
            });
        }

        // -------------------------------------------------------------
        // Layer 4: Architecture & Deep Modules (Ousterhout, Fowler)
        // -------------------------------------------------------------
        // Deep nesting check (> 4 indent levels of logic)
        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
        if leading_spaces >= 16 && (trimmed.starts_with("if ") || trimmed.starts_with("for ") || trimmed.starts_with("while ")) {
            findings.push(VibeFinding {
                layer: InspectionLayer::Layer4ArchitectureDeepModules,
                severity: FindingSeverity::Warning,
                rule_id: "OUSTERHOUT-COGNITIVE-NESTING".to_string(),
                canonical_book: "VBR-02-001 (A Philosophy of Software Design - Ousterhout)".to_string(),
                message: "Deeply nested control flow (cognitive nesting >= 4 levels).".to_string(),
                line_number: Some(line_num),
                snippet: Some(trimmed.to_string()),
                remediation: "Flatten control flow using early guard return clauses or extract helper methods.".to_string(),
            });
        }

        // Long parameter list (> 5 parameters)
        if (trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") || trimmed.starts_with("def ") || trimmed.starts_with("function "))
            && trimmed.contains('(') {
            if let Some(param_str) = trimmed.split('(').nth(1).and_then(|s| s.split(')').next()) {
                let count = param_str.split(',').filter(|p| !p.trim().is_empty()).count();
                if count >= 6 {
                    findings.push(VibeFinding {
                        layer: InspectionLayer::Layer4ArchitectureDeepModules,
                        severity: FindingSeverity::Advisory,
                        rule_id: "FOWLER-LONG-PARAMETER-LIST".to_string(),
                        canonical_book: "VBR-02-002 (Refactoring - Fowler & Beck)".to_string(),
                        message: format!("Function takes {} parameters (threshold: <= 5).", count),
                        line_number: Some(line_num),
                        snippet: Some(trimmed.to_string()),
                        remediation: "Introduce a parameter object or builder struct to bundle related configuration.".to_string(),
                    });
                }
            }
        }

        // -------------------------------------------------------------
        // Layer 5: Verification Cage & Test Smells (Khorikov, Meszaros)
        // -------------------------------------------------------------
        if trimmed.contains("assert!(true)") || trimmed.contains("assertTrue(true)") || trimmed.contains("assert True") {
            findings.push(VibeFinding {
                layer: InspectionLayer::Layer5VerificationCage,
                severity: FindingSeverity::Critical,
                rule_id: "MESZAROS-TAUTOLOGICAL-TEST".to_string(),
                canonical_book: "VBR-07-002 (xUnit Test Patterns - Meszaros)".to_string(),
                message: "Tautological assertion detected: assert!(true) tests nothing.".to_string(),
                line_number: Some(line_num),
                snippet: Some(trimmed.to_string()),
                remediation: "Replace with an assertion that verifies real state or output invariants of the system.".to_string(),
            });
        }

        // -------------------------------------------------------------
        // Layer 2: Concurrency & Timeouts (Goetz, Nygard)
        // -------------------------------------------------------------
        if trimmed.contains("reqwest::Client::new()") || trimmed.contains("HttpClient.newHttpClient()") {
            if !code.contains("timeout") {
                findings.push(VibeFinding {
                    layer: InspectionLayer::Layer2BoundaryEdgeCases,
                    severity: FindingSeverity::Warning,
                    rule_id: "NYGARD-MISSING-TIMEOUT".to_string(),
                    canonical_book: "VBR-05-008 (Release It! - Nygard)".to_string(),
                    message: "HTTP Client instantiated without explicit connection/read timeouts.".to_string(),
                    line_number: Some(line_num),
                    snippet: Some(trimmed.to_string()),
                    remediation: "Configure explicit connect_timeout and timeout durations on the client builder.".to_string(),
                });
            }
        }
    }

    let critical_count = findings.iter().filter(|f| f.severity == FindingSeverity::Critical).count();
    let warning_count = findings.iter().filter(|f| f.severity == FindingSeverity::Warning).count();
    let advisory_count = findings.iter().filter(|f| f.severity == FindingSeverity::Advisory).count();
    let passed = critical_count == 0;

    VibeAuditReport {
        target,
        summary: AuditSummary {
            total_lines,
            critical_count,
            warning_count,
            advisory_count,
            passed,
        },
        findings,
    }
}

// ===========================================================================
// 5. CLI ACTIONS & DISPATCH (`tgs review`)
// ===========================================================================

#[derive(clap::Subcommand, Debug, Clone)]
pub enum ReviewAction {
    /// List the 100-book review canon with optional cluster filtering and pagination
    List {
        /// Filter by cluster code (e.g. vbr-01, vbr-02, ..., vbr-10, ousterhout, fowler, security)
        #[arg(short, long)]
        cluster: Option<String>,
        /// Maximum number of books to display (default: 20)
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        /// Starting offset index for pagination (default: 0)
        #[arg(short, long, default_value_t = 0)]
        offset: usize,
    },
    /// Sub-millisecond inverted search across the 100-book review canon
    Search {
        /// Keywords, authors, or concepts (e.g. "shallow modules", "race conditions", "sql injection")
        query: String,
        /// Maximum results to display (default: 15)
        #[arg(short, long, default_value_t = 15)]
        limit: usize,
        /// Optional cluster filter (e.g. vbr-02, vbr-04)
        #[arg(short, long)]
        cluster: Option<String>,
    },
    /// Retrieve full specification and invariants for a specific book by ID
    Get {
        /// Book ID (e.g. VBR-02-001, VBR-04-001, VBR-06-001)
        id: String,
        /// Output format: terminal, json, markdown (default: terminal)
        #[arg(short, long, default_value = "terminal")]
        format: String,
    },
    /// Run the automated 5-layer static code auditor on a file or directory
    Audit {
        /// Path to target source file or directory
        target: String,
        /// Output report as JSON
        #[arg(long)]
        json: bool,
    },
    /// Display the curated 5-stage Vibe Coder reading and review playbook
    Playbook {
        /// Stage number (1 to 5)
        #[arg(short, long)]
        stage: Option<usize>,
    },
    /// Display full 10-cluster breakdown and catalog health statistics
    Breakdown,
    /// Export the 100-book review catalog to Markdown or JSON
    Export {
        /// Optional cluster filter
        #[arg(short, long)]
        cluster: Option<String>,
        /// Format: markdown, json (default: markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        /// Output file path (prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
}

/// Main execution handler for `tgs review` subcommands
pub async fn handle_review_command(action: ReviewAction) -> Result<()> {
    let catalog = ReviewCatalog::new();

    match action {
        ReviewAction::List { cluster, limit, offset } => {
            let target_cluster = cluster.as_deref().and_then(ReviewCluster::from_str);
            let pool: Vec<&ReviewBookSkill> = match target_cluster {
                Some(c) => catalog.by_cluster(c),
                None => catalog.books.iter().collect(),
            };

            let total_matching = pool.len();
            let slice: Vec<&ReviewBookSkill> = pool.into_iter().skip(offset).take(limit).collect();

            println!("{}", "=========================================================================".cyan());
            println!("{}", "  TAGISAN TOP 100 CODE & PROGRAM REVIEW CANON".bold().yellow());
            println!("{}\n", "=========================================================================".cyan());

            if let Some(c) = target_cluster {
                println!("  Active Cluster Filter: {} ({})\n", c.title().bold().green(), c.code().cyan());
            } else {
                println!("  Listing across: All 10 Macro-Inspection Clusters\n");
            }

            println!("  Displaying: {} to {} of {} matching review books\n", offset + 1, (offset + slice.len()).min(total_matching), total_matching);

            for book in slice {
                println!(
                    "  {} {} ({})",
                    format!("[{}]", book.id).bold().cyan(),
                    book.title.bold(),
                    book.year.to_string().dimmed()
                );
                println!("    Author(s):   {}", book.authors.yellow());
                println!("    Cluster:     {} ({})", book.cluster.title().dimmed(), book.cluster.code().dimmed());
                println!("    Heuristic:   {}", book.literature_heuristic.white());
                println!("    AI Defense:  {}\n", book.ai_hallucination_defense.green());
            }
        }

        ReviewAction::Search { query, limit, cluster } => {
            let target_cluster = cluster.as_deref().and_then(ReviewCluster::from_str);
            let results = catalog.search(&query, target_cluster, limit);

            println!("{}", "=========================================================================".cyan());
            println!("  {} \"{}\"", "SEARCH RESULTS FOR:".bold().yellow(), query.green().bold());
            println!("{}\n", "=========================================================================".cyan());

            if results.is_empty() {
                println!("  No matching review books found. Try broader keywords (e.g. 'modules', 'concurrency', 'security').");
            } else {
                for book in results {
                    println!(
                        "  {} {} ({})",
                        format!("[{}]", book.id).bold().cyan(),
                        book.title.bold(),
                        book.year.to_string().dimmed()
                    );
                    println!("    Author(s):  {}", book.authors.yellow());
                    println!("    Heuristic:  {}", book.literature_heuristic);
                    println!("    AI Defense: {}\n", book.ai_hallucination_defense.green());
                }
            }
        }

        ReviewAction::Get { id, format } => {
            let book = catalog.get(&id).ok_or_else(|| {
                TagisanError::Execution(format!("Review book with ID '{}' not found in catalog.", id))
            })?;

            match format.to_lowercase().as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(book).map_err(TagisanError::from)?;
                    println!("{}", json);
                }
                "markdown" | "md" => {
                    println!("# {} - {}", book.id, book.title);
                    println!("**Author(s):** {} ({})\n", book.authors, book.year);
                    println!("**Cluster:** {} ({})\n", book.cluster.title(), book.cluster.code());
                    println!("## Literature Heuristic\n{}\n", book.literature_heuristic);
                    println!("## AI Hallucination Defense\n{}\n", book.ai_hallucination_defense);
                    println!("## Review Invariants");
                    for inv in &book.review_invariants {
                        println!("- {}", inv);
                    }
                }
                _ => {
                    println!("{}", "=========================================================================".cyan());
                    println!("  {} {}", "REVIEW BOOK SPECIFICATION:".bold().yellow(), book.id.cyan().bold());
                    println!("{}\n", "=========================================================================".cyan());

                    println!("  Title:       {}", book.title.bold());
                    println!("  Author(s):   {} ({})", book.authors.yellow(), book.year);
                    println!("  Cluster:     {} ({})", book.cluster.title().bold().green(), book.cluster.code().cyan());
                    println!("  Heuristic:   {}", book.literature_heuristic.white());
                    println!("  AI Defense:  {}", book.ai_hallucination_defense.green().bold());
                    println!("\n  {}", "Review Invariants:".bold().yellow());
                    for inv in &book.review_invariants {
                        println!("    • {}", inv);
                    }
                    println!();
                }
            }
        }

        ReviewAction::Audit { target, json } => {
            let path = Path::new(&target);
            if !path.exists() {
                return Err(TagisanError::Execution(format!("Target file '{}' does not exist.", target)));
            }

            let code = fs::read_to_string(path).map_err(|e| {
                TagisanError::Execution(format!("Failed to read target file '{}': {}", target, e))
            })?;

            let report = audit_code(&code, Some(&target));

            if json {
                let j = serde_json::to_string_pretty(&report).map_err(TagisanError::from)?;
                println!("{}", j);
            } else {
                println!("{}", "=========================================================================".cyan());
                println!("  {} {}", "VIBE CODE REVIEW AUDIT:".bold().yellow(), target.cyan().bold());
                println!("{}\n", "=========================================================================".cyan());

                println!("  Target File:     {}", target.bold());
                println!("  Analyzed Lines:  {}", report.summary.total_lines);
                println!(
                    "  Verdict:         {}",
                    if report.summary.passed { "PASS (Zero Critical Blockers)".green().bold() } else { "FAIL (Critical Blockers Found)".red().bold() }
                );
                println!(
                    "  Findings:        {} Critical, {} Warning, {} Advisory\n",
                    report.summary.critical_count.to_string().red().bold(),
                    report.summary.warning_count.to_string().yellow().bold(),
                    report.summary.advisory_count.to_string().cyan()
                );

                if report.findings.is_empty() {
                    println!("  {} No violations detected against 100-book review canon.", "✔".green().bold());
                } else {
                    for f in &report.findings {
                        let sev_badge = match f.severity {
                            FindingSeverity::Critical => "[CRITICAL]".red().bold(),
                            FindingSeverity::Warning => "[WARNING]".yellow().bold(),
                            FindingSeverity::Advisory => "[ADVISORY]".cyan(),
                        };
                        println!("  {} {}", sev_badge, f.rule_id.bold());
                        println!("    Source:       {}", f.canonical_book.dimmed());
                        println!("    Message:      {}", f.message);
                        if let Some(ln) = f.line_number {
                            println!("    Line:         {}", ln);
                        }
                        if let Some(snip) = &f.snippet {
                            println!("    Snippet:      {}", snip.dimmed());
                        }
                        println!("    Remediation:  {}\n", f.remediation.green());
                    }
                }
            }
        }

        ReviewAction::Playbook { stage } => {
            println!("{}", catalog.playbook(stage));
        }

        ReviewAction::Breakdown => {
            println!("{}", "=========================================================================".cyan());
            println!("{}", "  10 MACRO-INSPECTION CLUSTERS BREAKDOWN (100 BOOKS)".bold().yellow());
            println!("{}\n", "=========================================================================".cyan());

            for c in ReviewCluster::all() {
                let books = catalog.by_cluster(*c);
                println!("  {} {} ({} books)", c.code().cyan().bold(), c.title().bold(), books.len().to_string().yellow());
                println!("    Canonical Authors: {}", c.canonical_authors().dimmed());
                for b in &books {
                    println!("      • {} {}", b.id.cyan(), b.title);
                }
                println!();
            }
        }

        ReviewAction::Export { cluster, format, output } => {
            let target_cluster = cluster.as_deref().and_then(ReviewCluster::from_str);
            let content = match format.to_lowercase().as_str() {
                "json" => catalog.export_json(target_cluster)?,
                _ => catalog.export_markdown(target_cluster),
            };

            if let Some(dest) = output {
                fs::write(&dest, &content).map_err(|e| {
                    TagisanError::Execution(format!("Failed to write export to '{}': {}", dest, e))
                })?;
                println!("{} Successfully exported review catalog to '{}'.", "✔".green().bold(), dest);
            } else {
                println!("{}", content);
            }
        }
    }

    Ok(())
}

// ===========================================================================
// 6. UNIT TESTS
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_exactly_100_books() {
        let catalog = ReviewCatalog::new();
        assert_eq!(catalog.books.len(), 100, "Catalog must have exactly 100 books");
    }

    #[test]
    fn test_every_cluster_has_10_books() {
        let catalog = ReviewCatalog::new();
        for c in ReviewCluster::all() {
            let count = catalog.by_cluster(*c).len();
            assert_eq!(count, 10, "Cluster {:?} must have exactly 10 books, found {}", c, count);
        }
    }

    #[test]
    fn test_search_ousterhout_and_deep_modules() {
        let catalog = ReviewCatalog::new();
        let hits = catalog.search("ousterhout", None, 5);
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|b| b.id == "VBR-02-001"));
    }

    #[test]
    fn test_search_concurrency_and_goetz() {
        let catalog = ReviewCatalog::new();
        let hits = catalog.search("goetz", None, 5);
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|b| b.id == "VBR-06-001"));
    }

    #[test]
    fn test_audit_code_catches_sql_injection_and_tautological_test() {
        let bad_code = "pub fn query_user(user_id: &str) {\n    let q = format!(\"SELECT * FROM users WHERE id = '{}'\", user_id);\n}\n#[test]\nfn test_something() {\n    assert!(true);\n}\n";

        let report = audit_code(bad_code, Some("test_file.rs"));
        assert!(!report.summary.passed);
        assert!(report.summary.critical_count >= 2);
        assert!(report.findings.iter().any(|f| f.rule_id == "DOWD-SQL-INJECTION-RISK"));
        assert!(report.findings.iter().any(|f| f.rule_id == "MESZAROS-TAUTOLOGICAL-TEST"));
    }

    #[test]
    fn test_audit_code_passes_clean_code() {
        let clean_code = "pub fn safe_calc(a: i32, b: i32) -> Result<i32, String> {\n    if b == 0 {\n        return Err(\"division by zero\".into());\n    }\n    Ok(a / b)\n}\n";

        let report = audit_code(clean_code, Some("clean.rs"));
        assert!(report.summary.passed);
        assert_eq!(report.summary.critical_count, 0);
    }
}
"""

with open(OUTPUT_RS, "w") as f:
    f.write(header + books_block + footer)

print(f"Generated {OUTPUT_RS} successfully with {len(raw_books)} books.")
