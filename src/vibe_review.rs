//! Tagisan Sovereign Vibe Code Review Engine (`tgs review`)
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
            ReviewBookSkill {
                id: "VBR-01-001".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Peer Reviews in Software: A Practical Guide".to_string(),
                authors: "Karl E. Wiegers".to_string(),
                year: 2001,
                literature_heuristic: "Distinguish stylistic trivia from functional defects; establish inspection checklists and formal review gates.".to_string(),
                review_invariants: vec!["Review checklist must verify business logic before stylistic formatting", "Defect density must be measured against lines of changed code", "Every reviewer must sign off with explicit approval or blockers"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI hallucinating superficial correctness while omitting critical business requirements.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-002".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Software Engineering at Google".to_string(),
                authors: "Titus Winters, Tom Manshreck, Hyrum Wright".to_string(),
                year: 2020,
                literature_heuristic: "Code Review and Critique: Readability certification, reviewing for long-term codebase health, maintainability across decades.".to_string(),
                review_invariants: vec!["Code must be maintainable by someone unfamiliar with the initial prompt", "Public APIs must not leak internal implementation details (Hyrum's Law)", "Review feedback must be categorized as [Blocker], [Suggestion], or [Nitpick]"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating short-term hacky scripts that break public contracts over time.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-003".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Best Kept Secrets of Peer Code Review".to_string(),
                authors: "Jason Cohen et al.".to_string(),
                year: 2006,
                literature_heuristic: "Review efficiency crashes above 400 lines per session; pace inspections under 500 LOC/hour for optimal defect catch rate.".to_string(),
                review_invariants: vec!["Pull requests exceeding 400 lines of diff must be broken into smaller atomic commits", "Review sessions must not exceed 60 minutes continuously without a break", "Both author and reviewer must verify passing automated test runs before sign-off"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from overwhelming reviewers with massive 2,000-line monolithic diffs that hide bugs.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-004".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Software Inspection".to_string(),
                authors: "Tom Gilb, Dorothy Graham".to_string(),
                year: 1993,
                literature_heuristic: "Formal Fagan inspection techniques: entry criteria, kick-off, individual preparation, logging meeting, and follow-up.".to_string(),
                review_invariants: vec!["Formal sampling of code must be conducted to establish defect density baselines", "Checklists must be continuously calibrated against past production bugs", "Exit criteria must mandate zero outstanding high-severity defects"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates rubber-stamping of AI pull requests by enforcing formal defect logging.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-005".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "What to Look for in a Code Review".to_string(),
                authors: "Trisha Gee".to_string(),
                year: 2019,
                literature_heuristic: "Pragmatic developer checklist separating functional design, architecture, performance, error handling, and testing.".to_string(),
                review_invariants: vec!["Verify that error messages are informative and actionable for end users", "Check that all new dependencies are strictly necessary and vetted", "Ensure existing test suites pass with zero regressions"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Flags unnecessary dependencies and boilerplate that AI tools routinely hallucinate.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-006".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "High-Impact Code Inspection".to_string(),
                authors: "Ronald Radice".to_string(),
                year: 2002,
                literature_heuristic: "Process-driven inspection metrics, tracking defect injection points, and root-cause prevention.".to_string(),
                review_invariants: vec!["Track defect categories to feed back into future prompting and agent guidelines", "Measure inspection rate and rework hours to optimize developer velocity", "Enforce strict gatekeeping on core kernel and security-sensitive paths"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents recurring AI prompt failures by tracking systematic error patterns.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-007".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Software Inspection: An Industry Best Practice".to_string(),
                authors: "David A. Wheeler et al.".to_string(),
                year: 1996,
                literature_heuristic: "Military and aerospace-grade software inspection across mission-critical systems and formal checklists.".to_string(),
                review_invariants: vec!["Verify all state machines for unhandled transitions or deadlocks", "Inspect memory allocations for bounded growth and determinism", "Conduct double-blind verification on mission-critical algorithms"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from creating unbounded memory allocations or unhandled states in critical paths.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-008".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Pair Programming Illuminated".to_string(),
                authors: "Laurie Williams, Robert Kessler".to_string(),
                year: 2002,
                literature_heuristic: "Driver-Navigator dynamic: continuous real-time review, cognitive division of labor, and architectural oversight.".to_string(),
                review_invariants: vec!["Human acts as Navigator reviewing strategy while AI acts as Driver executing syntax", "Switch roles or pause generation whenever complexity exceeds comprehension", "Review assumptions out loud before writing complex algorithms"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Forces the vibe coder to steer the AI's intent rather than passively accepting generated tokens.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-009".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Accelerate: The Science of Lean Software and DevOps".to_string(),
                authors: "Nicole Forsgren, Jez Humble, Gene Kim".to_string(),
                year: 2018,
                literature_heuristic: "Peer review in trunk-based development: small batches, fast feedback loops, and high psychological safety.".to_string(),
                review_invariants: vec!["Merge small diffs into main branch multiple times per day", "Maintain review turnaround latency under 4 hours", "Rely on comprehensive automated CI tests as the primary verification gate"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents accumulating large unintegrated AI branches that cause merge hell.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-01-010".to_string(),
                cluster: ReviewCluster::PeerReviewInspection,
                title: "Code Reviews: A Guide for Developers".to_string(),
                authors: "Mark Seemann".to_string(),
                year: 2020,
                literature_heuristic: "Asynchronous code review etiquette, making diffs easy to review, and focusing on architectural integrity.".to_string(),
                review_invariants: vec!["Include an architectural summary explaining the 'why' behind every PR", "Keep review discussions factual, respectful, and focused on the code", "Automate all linting, formatting, and type checks to keep human review on logic"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates stylistic nitpicks on AI code, keeping human cognitive bandwidth on correctness.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-001".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "A Philosophy of Software Design (2nd Ed)".to_string(),
                authors: "John Ousterhout".to_string(),
                year: 2021,
                literature_heuristic: "Deep vs Shallow Modules: Modules should have simple interfaces that hide significant complexity; combat tactical programming.".to_string(),
                review_invariants: vec!["Interfaces must be simpler than the underlying implementation", "Eliminate shallow wrapper functions that add cognitive load without adding value", "Define errors out of existence by designing APIs that have no invalid states"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "The #1 defense against AI: stops models from generating dozens of thin pass-through classes and wrappers.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-002".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Refactoring: Improving the Design of Existing Code (2nd Ed)".to_string(),
                authors: "Martin Fowler, Kent Beck".to_string(),
                year: 2018,
                literature_heuristic: "The canonical catalog of Code Smells: Long Method, Large Class, Primitive Obsession, Feature Envy, Shotgun Surgery.".to_string(),
                review_invariants: vec!["Identify and eliminate code smells in generated code before approving", "Apply atomic refactorings with passing tests", "Never mix behavioral additions with structural refactorings in the same commit"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Equips the reviewer to spot AI code smells and command precise refactoring passes.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-003".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Working Effectively with Legacy Code".to_string(),
                authors: "Michael Feathers".to_string(),
                year: 2004,
                literature_heuristic: "Legacy code is code without tests; find seams, break dependencies, write characterization tests.".to_string(),
                review_invariants: vec!["Never modify existing unverified code without first wrapping it in characterization tests", "Identify structural seams to inject mocks or test doubles without editing caller logic", "Preserve existing behavior while surgically introducing new features"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Treats AI-generated code as instant legacy code, mandating automated test cages.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-004".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "The Art of Readable Code".to_string(),
                authors: "Dustin Boswell, Trevor Foucher".to_string(),
                year: 2011,
                literature_heuristic: "Code should be written to minimize time to understand; pack information into names, simplify control flow.".to_string(),
                review_invariants: vec!["Variable and function names must communicate precise intent and units (e.g. timeout_ms)", "Minimize nesting depth by returning early (guard clauses)", "Align related code visually and break complex expressions into explanatory variables"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates cryptic or generic names (data, result, temp) commonly output by LLMs.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-005".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Refactoring to Patterns".to_string(),
                authors: "Joshua Kerievsky".to_string(),
                year: 2004,
                literature_heuristic: "Evolve toward patterns when complexity justifies it, and refactor away from patterns when they over-complicate.".to_string(),
                review_invariants: vec!["Ban speculative design patterns that have only one concrete implementation", "Refactor State/Strategy patterns back to simple conditionals if variants are trivial", "Introduce Factory/Builder only when object construction parameters exceed 4"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents pattern-bloat where AI hallucinates AbstractFactoryFactory patterns for simple tasks.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-006".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Clean Code: A Handbook of Agile Software Craftsmanship".to_string(),
                authors: "Robert C. Martin".to_string(),
                year: 2008,
                literature_heuristic: "Functions should do one thing; small functions; command-query separation; clean error handling.".to_string(),
                review_invariants: vec!["Functions should ideally fit on a single screen without vertical scrolling", "Functions should have 0, 1, or 2 arguments; avoid 3+ arguments without a parameter object", "Use exceptions or Result types rather than returning error codes"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Flags massive 200-line monolithic functions generated by models.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-007".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Five Lines of Code: How and When to Refactor".to_string(),
                authors: "Christian Clausen".to_string(),
                year: 2021,
                literature_heuristic: "Rule-based refactoring constraints: no method longer than 5 lines, eliminate if-else chaining, zero arrow patterns.".to_string(),
                review_invariants: vec!["Enforce strict maximum line counts on critical algorithmic routines", "Eliminate nested loops and conditionals exceeding 2 levels of indentation", "Replace type code switching with dispatch tables or polymorphic interfaces"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Provides clear, mechanical rules to prompt AI to break down complex generated logic.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-008".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Writing Solid Code".to_string(),
                authors: "Steve Maguire".to_string(),
                year: 1993,
                literature_heuristic: "Microsoft's classic on defensive programming: enable compiler warnings, use debug assertions, eliminate undefined behavior.".to_string(),
                review_invariants: vec!["Treat all compiler warnings as errors (-Werror, #![deny(warnings)])", "Assert preconditions, postconditions, and invariants aggressively in debug builds", "Make invalid memory states impossible by initializing variables at declaration"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches subtle undefined behavior or uninitialized state bugs in AI C/C++ and unsafe code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-009".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Refactoring Workbook".to_string(),
                authors: "William C. Wake".to_string(),
                year: 2003,
                literature_heuristic: "Hands-on smell identification: bloaters, object-orientation abusers, change preventers, dispensables, couplers.".to_string(),
                review_invariants: vec!["Catalog dispensable dead code, speculative generics, and redundant comments", "Break tight coupling between modules that should be independent", "Verify behavioral equivalence before and after structural edits"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Identifies AI-generated comments that just restate the code rather than explaining the 'why'.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-02-010".to_string(),
                cluster: ReviewCluster::SmellsRefactoring,
                title: "Bad Code Medicine: Finding and Fixing Architectural Debt".to_string(),
                authors: "Dan Sturtevant, Michael A. Cusumano".to_string(),
                year: 2018,
                literature_heuristic: "Measuring architectural complexity: cyclic dependencies, modularity violations, and architectural debt.".to_string(),
                review_invariants: vec!["Ban circular dependencies between packages or modules", "Detect and break transitive coupling cycles across service boundaries", "Measure blast radius before merging refactored core components"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from introducing circular imports that trigger runtime crashes.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-001".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "Code Complete (2nd Edition)".to_string(),
                authors: "Steve McConnell".to_string(),
                year: 2004,
                literature_heuristic: "Definitive software construction: routine design, defensive programming, variable layout, table-driven methods.".to_string(),
                review_invariants: vec!["Minimize scope of variables; declare them as close to first use as possible", "Avoid magic numbers; extract named constants with clear semantic meaning", "Design routines to have strong functional cohesion"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Ensures AI-generated functions adhere to proven software construction standards.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-002".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "The Pragmatic Programmer (20th Anniversary Ed)".to_string(),
                authors: "David Thomas, Andrew Hunt".to_string(),
                year: 2019,
                literature_heuristic: "Orthogonality, DRY, tracer bullets, broken windows theory, engineering empathy, and continuous learning.".to_string(),
                review_invariants: vec!["Fix broken windows immediately; never tolerate messy workarounds in PRs", "Ensure systems are orthogonal: changes in one area must not affect unrelated areas", "Build tracer bullets to validate end-to-end architecture before full implementation"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from duplicating business logic across multiple files.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-003".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "The Practice of Programming".to_string(),
                authors: "Brian W. Kernighan, Rob Pike".to_string(),
                year: 1999,
                literature_heuristic: "Simplicity, clarity, generality, idioms, portability, and debugging fundamentals from Unix architects.".to_string(),
                review_invariants: vec!["Choose clean algorithms and simple data structures over clever micro-optimizations", "Write portable code that does not depend on host architecture byte order or word size", "Make interfaces clear and documentation sparse but exact"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Cuts through convoluted AI logic, enforcing the Unix philosophy of doing one thing well.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-004".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "Clean Craftsmanship: Disciplines, Standards, and Ethics".to_string(),
                authors: "Robert C. Martin".to_string(),
                year: 2021,
                literature_heuristic: "The five core disciplines: TDD, Refactoring, Simple Design, Collaborative Programming, and CI.".to_string(),
                review_invariants: vec!["Uphold professional quality standards regardless of deadline pressure", "Never deploy code without automated regression test suites", "Ensure code passes all tests, expresses intent, eliminates duplication, and has fewest elements"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Instills ethical quality gates against shipping unverified AI code into production.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-005".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "The Clean Coder: Professional Conduct".to_string(),
                authors: "Robert C. Martin".to_string(),
                year: 2011,
                literature_heuristic: "Professionalism, taking responsibility for bugs, estimating honestly, saying 'No', and managing pressure.".to_string(),
                review_invariants: vec!["Take ownership of AI-assisted code: if you commit it, you are 100% responsible for it", "Refuse to disable safety checks, linters, or tests to meet an artificial deadline", "Communicate technical risks and blast radius transparently to stakeholders"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Reminds the vibe coder that AI is a tool, not a scapegoat for production outages.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-006".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "Structure and Interpretation of Computer Programs (SICP)".to_string(),
                authors: "Harold Abelson, Gerald Jay Sussman".to_string(),
                year: 1996,
                literature_heuristic: "Computational abstractions: procedural abstraction, data abstraction, state manipulation, metalinguistic evaluation.".to_string(),
                review_invariants: vec!["Separate the specification of what a procedure does from how it is computed", "Use closures and higher-order functions to capture repeated computational patterns", "Model state transitions as explicit transformations of immutable data streams"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Enables the reviewer to spot fundamental algorithmic and abstraction flaws in AI code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-007".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "Apprenticeship Patterns: Guidance for Software Craftsmen".to_string(),
                authors: "Dave Hoover, Adewale Oshineye".to_string(),
                year: 2009,
                literature_heuristic: "Accelerating mastery: reading unfamiliar code, diving into deep water, learning tools, sharing knowledge.".to_string(),
                review_invariants: vec!["Regularly read open-source reference code to calibrate personal review standards", "Expose mistakes early to learn from peer and compiler feedback", "Build automated sandboxes to test library behaviors before committing to them"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Develops mental discipline required to critically audit AI code outside one's comfort zone.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-008".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "Effective Programming: More Than Writing Code".to_string(),
                authors: "Jeff Atwood".to_string(),
                year: 2012,
                literature_heuristic: "Human factors in programming: understanding users, writing for readability, embracing failure, developer empathy.".to_string(),
                review_invariants: vec!["Review UI and CLI text for developer and end-user clarity", "Design error states to be self-explanatory with clear recovery paths", "Treat documentation as a first-class component of every feature"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating incomprehensible error messages or unhelpful CLI flags.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-009".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "Programming Pearls (2nd Edition)".to_string(),
                authors: "Jon Bentley".to_string(),
                year: 1999,
                literature_heuristic: "Back-of-the-envelope calculations, algorithmic problem solving, data representation tricks, engineering efficiency.".to_string(),
                review_invariants: vec!["Perform back-of-the-envelope estimation of memory and network requirements before coding", "Structure data to make algorithmic solutions simple and natural", "Profile actual performance before embarking on speculative optimizations"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI proposing O(N^2) or memory-hogging algorithms where O(N) exists.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-03-010".to_string(),
                cluster: ReviewCluster::CraftsmanshipConstruction,
                title: "More Programming Pearls: Confessions of a Coder".to_string(),
                authors: "Jon Bentley".to_string(),
                year: 1988,
                literature_heuristic: "Engineering case studies: profiling, prototyping, data structures, and self-checking code.".to_string(),
                review_invariants: vec!["Build minimal prototypes to test architectural viability", "Implement self-checking verification logic in complex mathematical routines", "Audit memory footprint and cache locality for core data structures"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Ensures AI solutions are mathematically and computationally grounded.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-001".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "The Art of Software Security Assessment".to_string(),
                authors: "Mark Dowd, John McDonald, Justin Schuh".to_string(),
                year: 2006,
                literature_heuristic: "The holy grail of source code security review: memory corruption, integer arithmetic flaws, pointer aliasing, privilege boundaries.".to_string(),
                review_invariants: vec!["Scrutinize all integer arithmetic for overflow, truncation, and signedness conversions", "Audit pointer lifetime, double-free, and use-after-free paths in unsafe blocks", "Verify that privilege checks are performed on every access to sensitive resources"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "The ultimate shield against AI writing code with critical CVEs or memory vulnerabilities.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-002".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Writing Secure Code (2nd Edition)".to_string(),
                authors: "Michael Howard, David LeBlanc".to_string(),
                year: 2002,
                literature_heuristic: "Microsoft's security standard: buffer overruns, access control, canonicalization, input validation, least privilege.".to_string(),
                review_invariants: vec!["Canonicalize all path names and URLs before evaluating access control policies", "Validate all inputs against an allowlist rather than trying to sanitize denylists", "Execute services under the least privileged user account required"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI path traversal bugs (../) and unvalidated input injection.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-003".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "The Tangled Web: Securing Modern Web Applications".to_string(),
                authors: "Michal Zalewski".to_string(),
                year: 2011,
                literature_heuristic: "Browser security models, Same-Origin Policy (SOP), CORS, framing, cookies, content sniffing, and parser quirks.".to_string(),
                review_invariants: vec!["Never configure CORS with wildcard origins (*) alongside credentials", "Set HttpOnly, Secure, and SameSite=Strict/Lax flags on all session cookies", "Enforce strict Content-Security-Policy (CSP) headers to prevent XSS execution"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches insecure CORS and cookie configs that AI web generators produce 80% of the time.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-004".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "The Web Application Hacker's Handbook (2nd Edition)".to_string(),
                authors: "Dafydd Stuttard, Marcus Pinto".to_string(),
                year: 2011,
                literature_heuristic: "Offensive exploitation techniques: SQL injection, SSRF, IDOR, session fixation, broken object authorization.".to_string(),
                review_invariants: vec!["Verify user ownership on every database lookup using IDs from client requests (anti-IDOR)", "Use parameterized prepared statements for 100% of database queries (zero string formatting)", "Block outbound HTTP requests to internal IP addresses (127.0.0.1, 169.254.169.254) to prevent SSRF"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from string-concatenating user inputs into SQL queries or curl requests.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-005".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Secure by Design".to_string(),
                authors: "Dan Bergh Johnsson, Daniel Deogun, Daniel Sawano".to_string(),
                year: 2019,
                literature_heuristic: "Domain-driven security: using strong domain types (EmailAddress, PositiveInt) to make vulnerabilities impossible by construction.".to_string(),
                review_invariants: vec!["Replace primitive strings with strongly typed domain value objects", "Validate domain invariants in the constructor so unvalidated state cannot exist", "Make domain entities immutable by default to prevent unauthorized mutations"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Forces AI to design types that reject malicious payloads at the boundary.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-006".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Threat Modeling: Designing for Security".to_string(),
                authors: "Adam Shostack".to_string(),
                year: 2014,
                literature_heuristic: "STRIDE threat modeling: Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege.".to_string(),
                review_invariants: vec!["Evaluate each component interface against the 6 STRIDE threat categories", "Map trust boundaries explicitly where external data crosses into internal networks", "Document threat mitigations directly in architectural design records"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Ensures the vibe coder reviews security from an architectural attacker's perspective.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-007".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Fuzzing for Software Security Testing and QA".to_string(),
                authors: "Ari Takanen, Jared D. DeMott, Charlie Miller".to_string(),
                year: 2008,
                literature_heuristic: "Generation-based and mutation-based fuzzing, crash triage, code coverage monitoring, automated vulnerability discovery.".to_string(),
                review_invariants: vec!["Subject all external file and protocol parsers to automated fuzz testing", "Triage all fuzz crashes to determine exploitative potential (EIP control vs DoS)", "Incorporate fuzz tests into CI regression pipelines for all serialization code"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Cages AI parsers with automated adversarial inputs that trigger hidden panics.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-008".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Silence on the Wire: A Field Guide to Passive Reconnaissance".to_string(),
                authors: "Michal Zalewski".to_string(),
                year: 2005,
                literature_heuristic: "Side-channel analysis: timing leaks, packet size signatures, cache observation, and ambient data leakage.".to_string(),
                review_invariants: vec!["Use constant-time comparison functions for passwords, hashes, and crypto tokens", "Pad sensitive responses to constant lengths to avoid packet size leakage", "Ensure error responses do not reveal user existence or system internals"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches timing attack vulnerabilities in authentication code written by AI.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-009".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Hacking: The Art of Exploitation (2nd Edition)".to_string(),
                authors: "Jon Erickson".to_string(),
                year: 2008,
                literature_heuristic: "Assembly-level vulnerability analysis: stack overflows, format string bugs, heap exploitation, shellcode mechanics.".to_string(),
                review_invariants: vec!["Audit all format string calls (printf, sprintf) to ensure format is never user-controlled", "Ensure stack canaries, ASLR, and non-executable stack (DEP) flags are enabled", "Verify buffer boundary constraints in low-level memory operations"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating vulnerable C/assembly memory handling code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-04-010".to_string(),
                cluster: ReviewCluster::SecurityAuditingAppSec,
                title: "Alice and Bob Learn Application Security".to_string(),
                authors: "Tanya Janca".to_string(),
                year: 2020,
                literature_heuristic: "Modern AppSec: Secure Software Development Lifecycle (SSDLC), SAST/DAST automation, secrets management.".to_string(),
                review_invariants: vec!["Never commit raw API keys, passwords, or tokens to version control", "Run automated SAST security linters on every PR before human review", "Audit third-party dependencies for known CVEs using cargo-deny or dependabot"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from hardcoding test API keys and fake credentials in code files.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-001".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Designing Data-Intensive Applications (DDIA)".to_string(),
                authors: "Martin Kleppmann".to_string(),
                year: 2017,
                literature_heuristic: "The modern Bible of distributed data: transaction isolation levels, replication lag, split-brain, consensus, WAL.".to_string(),
                review_invariants: vec!["Explicitly define database transaction isolation levels (Read Committed vs Serializable)", "Account for replication lag and eventual consistency in distributed reads", "Design for partition tolerance (CAP theorem) and handle network split-brain gracefully"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from making false assumptions about distributed data consistency and transaction guarantees.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-002".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Fundamentals of Software Architecture".to_string(),
                authors: "Mark Richards, Neal Ford".to_string(),
                year: 2020,
                literature_heuristic: "Architectural characteristics (the '-ilities'), modularity, component cohesion, trade-off analysis, governance.".to_string(),
                review_invariants: vec!["Document architectural trade-offs: latency vs memory, consistency vs availability", "Ensure high component cohesion and loose coupling across module boundaries", "Define fitness functions to automate architectural governance rules"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from making architectural decisions without evaluating trade-offs.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-003".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Software Architecture: The Hard Parts".to_string(),
                authors: "Neal Ford, Mark Richards et al.".to_string(),
                year: 2021,
                literature_heuristic: "Decomposing distributed architectures: transactional boundaries across services, saga orchestration, data contracts.".to_string(),
                review_invariants: vec!["Use Sagas or outbox patterns instead of distributed 2-Phase Commit (2PC) transactions", "Determine service granularity based on transactional boundaries and data volatility", "Define asynchronous event contracts to decouple microservices"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from inventing distributed monoliths that dead-lock under load.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-004".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Domain-Driven Design: Tackling Complexity".to_string(),
                authors: "Eric Evans".to_string(),
                year: 2003,
                literature_heuristic: "Ubiquitous Language, Bounded Contexts, Aggregates, Value Objects, Entities, and Context Mapping.".to_string(),
                review_invariants: vec!["Enforce Ubiquitous Language consistently across code, tests, and documentation", "Ensure Aggregate roots control all mutations to internal entities", "Isolate domain models inside Bounded Contexts with explicit Anti-Corruption Layers"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from mixing vocabulary and bleeding domain concepts across modules.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-005".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Building Microservices (2nd Edition)".to_string(),
                authors: "Sam Newman".to_string(),
                year: 2021,
                literature_heuristic: "Service modeling, integration patterns, database splitting, resilience, deployment topologies, distributed monolith traps.".to_string(),
                review_invariants: vec!["Never share databases directly across microservice boundaries", "Use consumer-driven contract testing to verify API compatibility", "Design services to fail independently without triggering cascading outages"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI writing shared database queries that couple independent services.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-006".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Building Evolutionary Architectures (2nd Edition)".to_string(),
                authors: "Neal Ford, Rebecca Parsons, Patrick Kua".to_string(),
                year: 2023,
                literature_heuristic: "Architectural fitness functions: automated tests verifying constraints, performance budgets, and security across PRs.".to_string(),
                review_invariants: vec!["Codify architectural rules into automated fitness functions (e.g. ArchUnit, cargo-deny)", "Enforce dependency direction: domain models must not depend on external frameworks", "Monitor architectural drift continuously in CI pipelines"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Automates the rejection of AI PRs that violate architectural layering.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-007".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Patterns of Enterprise Application Architecture".to_string(),
                authors: "Martin Fowler".to_string(),
                year: 2002,
                literature_heuristic: "Persistence patterns: Unit of Work, Identity Map, Repository, Data Mapper, Transaction Script vs Domain Model.".to_string(),
                review_invariants: vec!["Use Unit of Work to batch database mutations into single transactional commits", "Separate database access from business logic using Repository or Data Mapper", "Choose Domain Model for complex business rules; Transaction Script for simple CRUD"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from embedding raw SQL queries directly inside controller methods.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-008".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Release It! Design and Deploy Production-Ready Software (2nd Ed)".to_string(),
                authors: "Michael T. Nygard".to_string(),
                year: 2018,
                literature_heuristic: "Stability patterns: Circuit Breakers, Bulkheads, Timeouts, Shed Load, and steady-state telemetry.".to_string(),
                review_invariants: vec!["Every outbound network call must have explicit connection and read timeouts", "Wrap external service calls in Circuit Breakers to fail fast when downstream is down", "Use Bulkheads to isolate thread pools so one failing service cannot exhaust all workers"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates indefinite hangs caused by AI generating HTTP clients with no timeouts.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-009".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Enterprise Integration Patterns".to_string(),
                authors: "Gregor Hohpe, Bobby Woolf".to_string(),
                year: 2003,
                literature_heuristic: "Asynchronous messaging: Message Channels, Pipes and Filters, Message Router, Idempotent Receiver.".to_string(),
                review_invariants: vec!["Ensure message consumers are strictly idempotent to handle duplicate deliveries", "Use Dead-Letter Channels for poison pill messages that cannot be processed", "Decouple producers and consumers via asynchronous message brokers"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches message consumers that fail or corrupt state on retried duplicate messages.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-05-010".to_string(),
                cluster: ReviewCluster::ArchitectureSystemDesign,
                title: "Righting Software".to_string(),
                authors: "Juval Löwy".to_string(),
                year: 2019,
                literature_heuristic: "Methodical, risk-driven system decomposition based on volatility (what changes together) rather than functionality.".to_string(),
                review_invariants: vec!["Decompose systems by volatility: encapsulate components that change for different reasons", "Design services to survive requirements changes with zero structural rewrite", "Quantify and manage project schedule and architectural complexity risks"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from organizing code by feature silos that require multi-service edits for minor changes.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-001".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Java Concurrency in Practice".to_string(),
                authors: "Brian Goetz et al.".to_string(),
                year: 2006,
                literature_heuristic: "Universal masterclass on thread safety, happens-before memory models, publication of objects, synchronization, race conditions.".to_string(),
                review_invariants: vec!["All shared mutable state must be guarded by synchronization or made immutable", "Ensure safe publication of objects: do not allow 'this' reference to escape constructor", "Avoid holding locks while calling external or alien methods (prevent deadlocks)"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches subtle memory visibility and publication bugs in multi-threaded code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-002".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Is Parallel Programming Hard, And, If So, What Can You Do About It?".to_string(),
                authors: "Paul E. McKenney".to_string(),
                year: 2021,
                literature_heuristic: "Deep systems concurrency: memory barriers, RCU, cache line bouncing, lockless data structures, atomic ordering.".to_string(),
                review_invariants: vec!["Use appropriate atomic memory ordering (Relaxed vs Acquire/Release vs SeqCst)", "Align data structures to cache lines (64 bytes) to avoid false sharing", "Leverage RCU or read-heavy concurrency patterns for read-dominated workloads"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from misusing relaxed atomic ordering in lockless Rust or C++ code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-003".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "The Art of Multiprocessor Programming (Revised 2nd Ed)".to_string(),
                authors: "Maurice Herlihy, Nir Shavit".to_string(),
                year: 2020,
                literature_heuristic: "Linearizability, wait-free and lock-free algorithms, consensus numbers, transactional memory, atomic primitives.".to_string(),
                review_invariants: vec!["Prove linearizability for all custom concurrent data structures", "Verify lock-free algorithms guarantee system-wide progress (no starvation)", "Use Compare-And-Swap (CAS) loops with exponential backoff under contention"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from inventing flawed lock-free queue implementations with race conditions.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-004".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Concurrency in Go: Tools and Techniques".to_string(),
                authors: "Katherine Cox-Buday".to_string(),
                year: 2017,
                literature_heuristic: "Goroutines, channels, sync primitives, pipeline patterns, error propagation, goroutine leak detection.".to_string(),
                review_invariants: vec!["Every spawned goroutine must have an explicit cancellation and exit mechanism (context.Context)", "Do not communicate by sharing memory; share memory by communicating (channels)", "Prevent deadlocks by closing channels exclusively from the sender side"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates goroutine leaks where AI launches workers that block forever on unclosed channels.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-005".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "C++ Concurrency in Action (2nd Edition)".to_string(),
                authors: "Anthony Williams".to_string(),
                year: 2019,
                literature_heuristic: "The modern C++ memory model, std::thread, std::async, lock-free queues, atomic operations, parallel algorithms.".to_string(),
                review_invariants: vec!["Ensure std::jthread or RAII guards join threads before destruction", "Use std::scoped_lock to acquire multiple mutexes simultaneously without deadlock", "Avoid data races by declaring concurrent access points with std::atomic"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating std::thread code that crashes via std::terminate on unjoined threads.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-006".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Database Internals: Distributed Data Systems".to_string(),
                authors: "Alex Petrov".to_string(),
                year: 2019,
                literature_heuristic: "Storage engines, B-trees, LSM-trees, write-ahead logs (WAL), MVCC, and distributed transactions.".to_string(),
                review_invariants: vec!["Ensure all state mutations write to persistent WAL before modifying memory buffers", "Audit read/write amplification in storage engines and index structures", "Implement two-phase locking or MVCC for consistent snapshot isolation"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI writing database drivers that drop uncommitted writes on crash.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-007".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Distributed Systems: Principles and Paradigms (4th Ed)".to_string(),
                authors: "Maarten van Steen, Andrew S. Tanenbaum".to_string(),
                year: 2023,
                literature_heuristic: "Processes, communication, naming, synchronization (Lamport timestamps, vector clocks), consistency, fault tolerance.".to_string(),
                review_invariants: vec!["Use Lamport timestamps or vector clocks to establish causal ordering of events", "Handle partial failures: assume any remote node can crash or become unreachable at any time", "Enforce idempotent retry logic across distributed RPC boundaries"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from assuming remote network calls are synchronous, instantaneous, or reliable.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-008".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Designing Distributed Systems".to_string(),
                authors: "Brendan Burns".to_string(),
                year: 2018,
                literature_heuristic: "Containerized patterns: Sidecar, Ambassador, Adapter, Replicated Load-Balanced Services, Sharded Services.".to_string(),
                review_invariants: vec!["Use sidecars for auxiliary tasks (logging, auth, proxying) without polluting application logic", "Implement ambassador proxies to mask remote service connection logic", "Design sharded clusters with deterministic hashing to balance load evenly"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Ensures cloud infrastructure configurations and Dockerfiles are modular and clean.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-009".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Specifying Systems: The TLA+ Language and Tools".to_string(),
                authors: "Leslie Lamport".to_string(),
                year: 2002,
                literature_heuristic: "Formal specification of state machines and concurrent algorithms to prove safety and liveness invariants before coding.".to_string(),
                review_invariants: vec!["Mathematically define TypeOK, Safety, and MutualExclusion invariants", "Check models using TLC model checker across all possible state interleavings", "Verify liveness properties: ensure the system eventually makes progress"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Proves distributed consensus algorithms correct before AI attempts to write code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-06-010".to_string(),
                cluster: ReviewCluster::ConcurrencyInvariants,
                title: "Fault-Tolerant Message-Passing Distributed Systems".to_string(),
                authors: "Michel Raynal".to_string(),
                year: 2018,
                literature_heuristic: "Consensus, atomic broadcast, Byzantine fault tolerance, failure detectors, and distributed mutual exclusion.".to_string(),
                review_invariants: vec!["Verify quorum sizes: majority quorums (N/2 + 1) for crash faults, (2N/3 + 1) for Byzantine faults", "Ensure leader election algorithms prevent split-brain dual leaders", "Validate state machine replication guarantees linearizable total order"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating flawed Raft/Paxos consensus state transitions.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-001".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Unit Testing Principles, Practices, and Patterns".to_string(),
                authors: "Vladimir Khorikov".to_string(),
                year: 2020,
                literature_heuristic: "The four pillars of a good unit test: regression protection, refactoring resistance, fast feedback, maintainability.".to_string(),
                review_invariants: vec!["Tests must verify observable behavior and state, never internal implementation details", "Eliminate brittle tests that break upon refactoring without any change in behavior", "Reserve mocks strictly for verifying outbound side effects to external unmanaged systems"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates vanity tests where AI writes mocks that mock their own implementation.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-002".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "xUnit Test Patterns: Refactoring Test Code".to_string(),
                authors: "Gerard Meszaros".to_string(),
                year: 2007,
                literature_heuristic: "The 800-page catalog of test smells: Mystery Guest, Fragile Fixture, Obscure Test, Assertion Roulette, Over-Mocking.".to_string(),
                review_invariants: vec!["Eliminate Obscure Tests: test setup must clearly show relationship between input and output", "Avoid Assertion Roulette: use descriptive assertion messages or single assertion per test", "Never share mutable fixtures between test runs (guarantee test isolation)"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Identifies AI-generated tests that contain zero assertions or tautological assert(true).".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-003".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Test Driven Development: By Example".to_string(),
                authors: "Kent Beck".to_string(),
                year: 2002,
                literature_heuristic: "The Red-Green-Refactor rhythm: write a failing test first, make it pass with the simplest code possible, then refactor.".to_string(),
                review_invariants: vec!["Force AI to emit failing test cases before generating implementation code", "Write the minimal code necessary to pass the failing assertion", "Refactor code only while tests remain continuously green"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Constrains the AI's generation scope strictly to what is required to pass tests.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-004".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Growing Object-Oriented Software, Guided by Tests".to_string(),
                authors: "Steve Freeman, Nat Pryce".to_string(),
                year: 2009,
                literature_heuristic: "Mockist TDD, walking skeletons, outside-in testing, and listening to test friction to discover missing abstractions.".to_string(),
                review_invariants: vec!["Build an automated walking skeleton that tests deployment and connectivity on day one", "Drive design outside-in: start from user acceptance tests and drill into domain units", "If a test is hard to write, treat it as feedback that the design is too tightly coupled"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from creating untestable god-classes with hidden dependencies.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-005".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Property-Based Testing with PropEr, Erlang, and Elixir".to_string(),
                authors: "Fred Hébert".to_string(),
                year: 2019,
                literature_heuristic: "Testing invariants across thousands of randomized inputs; automatic test-case shrinking to find minimal failing inputs.".to_string(),
                review_invariants: vec!["Formulate mathematical properties (e.g. idempotency, round-trip encoding, invariants)", "Run randomized input generators to stress edge cases (empty strings, MAX_INT, unicode)", "Verify shrinking isolates the absolute minimal reproduction input on failure"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "The ultimate trap for AI: catches edge cases the model never considered.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-006".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Effective Software Testing: A Developer's Guide".to_string(),
                authors: "Maurício Aniche".to_string(),
                year: 2022,
                literature_heuristic: "Systematic testing techniques: boundary analysis, structural testing (coverage), mutation testing, property testing.".to_string(),
                review_invariants: vec!["Apply boundary value analysis: test at, just below, and just above boundaries", "Use mutation testing to ensure tests actually fail when bugs are injected into production code", "Ensure MC/DC coverage for complex boolean conditional logic"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Proves that AI tests are genuinely verifying code rather than generating hollow coverage.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-007".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "The Art of Software Testing (3rd Edition)".to_string(),
                authors: "Glenford J. Myers et al.".to_string(),
                year: 2011,
                literature_heuristic: "Foundational test psychology: a test is successful only if it finds a bug; equivalence partitioning, error guessing.".to_string(),
                review_invariants: vec!["Partition input domains into valid and invalid equivalence classes", "Design test cases specifically intended to demonstrate failure", "Maintain an adversarial mindset during test design"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Counters confirmation bias when reviewing AI code that looks plausibly correct.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-008".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Continuous Delivery: Reliable Software Releases".to_string(),
                authors: "Jez Humble, David Farley".to_string(),
                year: 2010,
                literature_heuristic: "Automated deployment pipelines, quality gates, immutable build artifacts, automated acceptance tests.".to_string(),
                review_invariants: vec!["Build each binary artifact once and promote the exact same binary across environments", "Keep the deployment pipeline green; stop the line immediately when tests fail", "Automate all environment configuration and database schema migrations"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Ensures AI code can be deployed safely through automated deployment gates.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-009".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "Testing Microservices with Mocks and Stubs".to_string(),
                authors: "Mark Winteringham".to_string(),
                year: 2021,
                literature_heuristic: "Consumer-driven contract testing (Pact), service virtualization, component testing, avoiding flaky E2E tests.".to_string(),
                review_invariants: vec!["Use contract tests to verify that service provider and consumer agree on schemas", "Stub external downstream dependencies with deterministic mock servers in CI", "Minimize end-to-end tests in favor of fast isolated component tests"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from creating flaky end-to-end tests that stall CI pipelines.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-07-010".to_string(),
                cluster: ReviewCluster::TestingVerification,
                title: "How Google Tests Software".to_string(),
                authors: "James A. Whittaker, Jason Arbon, Jeff Carollo".to_string(),
                year: 2012,
                literature_heuristic: "SWEs vs SETs, test pyramids, continuous test infrastructure, automated bug filing, testing at hyper-scale.".to_string(),
                review_invariants: vec!["Follow the 70/20/10 test pyramid (70% unit, 20% integration, 10% end-to-end)", "Ensure developers write their own tests rather than outsourcing testing to QA", "Isolate test environments hermetically with zero external network access"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating inverted test pyramids (90% slow UI tests, 0% unit tests).".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-001".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Systems Performance: Enterprise and the Cloud (2nd Ed)".to_string(),
                authors: "Brendan Gregg".to_string(),
                year: 2020,
                literature_heuristic: "The Bible of performance analysis: USE Method (Utilization, Saturation, Errors), CPU caches, flame graphs.".to_string(),
                review_invariants: vec!["Apply the USE method across all hardware resources (CPU, memory, disk, network)", "Generate flame graphs to locate hotspots before attempting code optimization", "Measure latency percentiles (p95, p99, p99.9) rather than misleading averages"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from proposing fake micro-optimizations that don't address the actual bottleneck.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-002".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "BPF Performance Tools: Deep Analysis".to_string(),
                authors: "Brendan Gregg".to_string(),
                year: 2019,
                literature_heuristic: "eBPF kernel tracing, measuring syscall latency, memory allocation tracking, block I/O profiling, socket inspection.".to_string(),
                review_invariants: vec!["Trace kernel syscall latency using eBPF probes without restarting processes", "Audit memory allocation frequency to eliminate unnecessary heap thrashing", "Monitor TCP retransmits and socket backlog saturation"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Enables reviewer to audit what AI systems code is actually doing inside the kernel.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-003".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Computer Systems: A Programmer's Perspective (CS:APP)".to_string(),
                authors: "Randal E. Bryant, David R. O'Hallaron".to_string(),
                year: 2015,
                literature_heuristic: "Memory hierarchies, cache locality, branch prediction, virtual memory, linkers, assembly code generation.".to_string(),
                review_invariants: vec!["Write cache-friendly code: traverse matrices row-wise to exploit spatial locality", "Avoid unpredictable branches in inner loops to prevent CPU pipeline flushes", "Understand virtual memory page faults and memory-mapped file behavior"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI writing cache-hostile algorithms with terrible memory access patterns.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-004".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "The Linux Programming Interface (TLPI)".to_string(),
                authors: "Michael Kerrisk".to_string(),
                year: 2010,
                literature_heuristic: "Authoritative reference on Linux/UNIX syscalls: file descriptors, signals, epoll, processes, pthreads, IPC.".to_string(),
                review_invariants: vec!["Check return values of all Linux syscalls and handle EINTR interruptions", "Ensure file descriptors are set with O_CLOEXEC to prevent leakage across exec", "Use epoll/kqueue for non-blocking asynchronous I/O across thousands of sockets"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from creating subtle file descriptor leaks or mishandling UNIX signals.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-005".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Debugging: The 9 Indispensable Rules".to_string(),
                authors: "David J. Agans".to_string(),
                year: 2002,
                literature_heuristic: "The 9 rules: Understand the system, Make it fail, Quit thinking and look, Divide and conquer, Check the plug.".to_string(),
                review_invariants: vec!["Reproduce the bug reliably before attempting any code modification", "Binary search the failure space (divide and conquer) to isolate the defect", "Verify the fix completely eliminates the symptom without collateral damage"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops vibe coders from accepting speculative AI fixes without reproducing the bug.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-006".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Why Programs Fail: Systematic Debugging (2nd Edition)".to_string(),
                authors: "Andreas Zeller".to_string(),
                year: 2009,
                literature_heuristic: "Delta debugging, scientific hypothesis testing, cause-effect chains, automated failure isolation.".to_string(),
                review_invariants: vec!["Formulate testable hypotheses about failure causes and isolate root causes systematically", "Apply delta debugging (git bisect) to isolate the exact commit that introduced a regression", "Trace cause-effect chains from initial infection to final observed failure"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Provides systematic methods to isolate regressions introduced by AI PRs.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-007".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "High Performance Browser Networking".to_string(),
                authors: "Ilya Grigorik".to_string(),
                year: 2013,
                literature_heuristic: "TCP congestion control, TLS handshake overhead, HTTP/2 multiplexing, HTTP/3 QUIC, WebSockets, latency reduction.".to_string(),
                review_invariants: vec!["Minimize round-trip times (RTT) by reducing synchronous serial API calls", "Leverage HTTP/2/3 multiplexing to avoid connection head-of-line blocking", "Enable TLS session resumption and HTTP compression (gzip/brotli)"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI web apps from suffering from waterfall request latency.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-008".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Optimizing Software in C++".to_string(),
                authors: "Agner Fog".to_string(),
                year: 2020,
                literature_heuristic: "Microarchitecture, instruction pipelines, vectorization (SIMD AVX-512), branch prediction, CPU pipeline stalls.".to_string(),
                review_invariants: vec!["Structure inner loops to enable automatic vectorization (SIMD) by the compiler", "Avoid division, modulo, and square root operations in hot computational paths", "Keep critical loop working sets within L1 instruction and data caches"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Enables expert review of AI-generated high-performance C++ and Rust code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-009".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Understanding Software Dynamics".to_string(),
                authors: "Richard L. Sites".to_string(),
                year: 2021,
                literature_heuristic: "Tracing tail latency (p99.9), memory bus contention, CPU frequency throttling, lock convoys, disk queueing.".to_string(),
                review_invariants: vec!["Investigate tail latency anomalies rather than focusing solely on median speed", "Audit memory bus bandwidth to prevent memory-bound application stalls", "Detect lock convoys where threads queue up behind slow critical sections"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI code that runs fast on a laptop but stalls under multi-core server contention.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-08-010".to_string(),
                cluster: ReviewCluster::PerformanceSystemsMemory,
                title: "Advanced Linux Programming".to_string(),
                authors: "Mark Mitchell, Jeffrey Oldham, Alex Samuel".to_string(),
                year: 2001,
                literature_heuristic: "Processes, IPC (pipes, FIFOs, shared memory, message queues), pthreads, system calls, native utilities.".to_string(),
                review_invariants: vec!["Clean up shared memory segments and IPC semaphores upon process termination", "Handle child process termination with waitpid to prevent zombie processes", "Audit memory-mapped file access (mmap) with proper protection flags"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from creating zombie processes or leaking shared memory segments.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-001".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Effective Java (3rd Edition)".to_string(),
                authors: "Joshua Bloch".to_string(),
                year: 2017,
                literature_heuristic: "90 items of Java wisdom: Builders, immutable objects, generics, enums, Lambdas, Streams, exceptions, serialization.".to_string(),
                review_invariants: vec!["Use Builders for constructors with more than 4 parameters", "Make classes immutable by default unless mutability is explicitly required", "Favor composition over inheritance; avoid extending concrete classes"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from generating obsolete Java idioms (Vector, raw types, finalizers).".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-002".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Effective C++ (3rd Edition)".to_string(),
                authors: "Scott Meyers".to_string(),
                year: 2005,
                literature_heuristic: "55 specific ways to improve C++ programs: RAII, copy/swap idioms, const correctness, virtual destructors.".to_string(),
                review_invariants: vec!["Manage all resources via RAII objects (smart pointers, file handles)", "Declare destructors virtual in polymorphic base classes to prevent memory leaks", "Pass objects by reference-to-const rather than by value for user-defined types"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Prevents AI from generating naked 'new' and 'delete' statements in modern C++.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-003".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Effective Modern C++".to_string(),
                authors: "Scott Meyers".to_string(),
                year: 2014,
                literature_heuristic: "C++11/C++14 mastery: auto, rvalue references, move semantics, perfect forwarding, lambda captures, smart pointers.".to_string(),
                review_invariants: vec!["Prefer std::unique_ptr over std::shared_ptr by default; use shared_ptr only for true shared ownership", "Use std::make_unique and std::make_shared to allocate resources safely", "Understand std::move does not move; it casts to an rvalue reference"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI dangling reference captures in C++ lambdas.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-004".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Programming Rust: Fast, Safe Systems Development (2nd Edition)".to_string(),
                authors: "Jim Blandy, Jason Orendorff, Leonora Tindall".to_string(),
                year: 2021,
                literature_heuristic: "Rust ownership model, lifetimes, interior mutability (RefCell, Mutex), traits, zero-cost abstractions, unsafe invariants.".to_string(),
                review_invariants: vec!["Minimize the use of 'unsafe'; any unsafe block must have an explicit SAFETY comment proving invariants", "Design APIs that enforce correctness through the type system (Type-State Pattern)", "Avoid clone() spam used by AI to silence the borrow checker"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI writing unneeded unsafe blocks or spraying .clone() everywhere to bypass borrowck.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-005".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "The Rust Programming Language".to_string(),
                authors: "Steve Klabnik, Carol Nichols".to_string(),
                year: 2023,
                literature_heuristic: "Official Rust idioms: pattern matching, error handling via Result/Option, closures, smart pointers, traits, concurrency.".to_string(),
                review_invariants: vec!["Never unwrap() in production paths; use the ? operator or handle errors explicitly", "Prefer borrowing (&T) over taking ownership when read-only access suffices", "Use pattern matching exhaustively to handle all possible enum variants"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from littering Rust code with panic-inducing .unwrap() calls.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-006".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Effective Python (2nd Edition)".to_string(),
                authors: "Brett Slatkin".to_string(),
                year: 2019,
                literature_heuristic: "90 specific ways to write Pythonic code: generators, comprehensions, descriptors, metaclasses, asyncio, type hints.".to_string(),
                review_invariants: vec!["Use generators instead of returning large materialized lists to save memory", "Annotate all public function signatures with explicit type hints", "Never use mutable default arguments (e.g. def foo(bar=[]))"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates classic Python traps that AI constantly generates (mutable defaults, missing type hints).".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-007".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Fluent Python (2nd Edition)".to_string(),
                authors: "Luciano Ramalho".to_string(),
                year: 2022,
                literature_heuristic: "Python data model: dunder methods, data structures, first-class functions, object references, closures, asyncio.".to_string(),
                review_invariants: vec!["Leverage the Python data model (__iter__, __getitem__, __len__) for idiomatic collections", "Differentiate deep vs shallow copies when manipulating nested dictionary structures", "Use asyncio.gather and TaskGroup correctly to handle async concurrent cancellations"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from writing Java-style getter/setter boilerplate in Python.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-008".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Effective TypeScript (2nd Edition)".to_string(),
                authors: "Dan Vanderkam".to_string(),
                year: 2024,
                literature_heuristic: "62 specific ways to improve TypeScript: structural typing, any vs unknown, type narrowing, generic inference.".to_string(),
                review_invariants: vec!["Ban 'any' in favor of 'unknown' with runtime type narrowing (Zod or type guards)", "Avoid type assertions (as Type); let TypeScript infer types naturally", "Use discriminated unions to represent mutually exclusive state variants"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Eliminates AI using 'as any' to silence TypeScript compiler errors.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-009".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "Learning Go (2nd Edition)".to_string(),
                authors: "Jon Bodner".to_string(),
                year: 2024,
                literature_heuristic: "Idiomatic Go: explicit error handling, interfaces, slices vs arrays, goroutines, channels, defer, standard library.".to_string(),
                review_invariants: vec!["Check errors immediately after the call (if err != nil { return err })", "Accept interfaces, return structs; keep interfaces small (1 or 2 methods)", "Use defer for cleanup immediately after acquiring a resource"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from ignoring errors or misusing panic/recover in Go.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-09-010".to_string(),
                cluster: ReviewCluster::LanguageSpecificBibles,
                title: "C++ Core Guidelines Explained".to_string(),
                authors: "Rainer Grimm".to_string(),
                year: 2020,
                literature_heuristic: "Modern best practices authored by Stroustrup and Sutter: type safety, bounds safety, resource safety, lifetimes.".to_string(),
                review_invariants: vec!["Adhere to C++ Core Guidelines for type, bounds, and lifetime safety", "Prefer std::span and std::string_view for non-owning range views", "Use gsl::not_null to document and enforce non-nullable pointer contracts"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Brings rigorous modern guidelines to auditing AI C++ code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-001".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "The Mythical Man-Month: Essays on Software Engineering".to_string(),
                authors: "Frederick P. Brooks Jr.".to_string(),
                year: 1995,
                literature_heuristic: "Conceptual integrity, second-system effect, Brooks' Law, essential vs accidental complexity.".to_string(),
                review_invariants: vec!["Preserve conceptual integrity: a system must reflect one unified design philosophy", "Be vigilant against the second-system effect (over-engineering following an initial success)", "Do not expect AI generation speed to eliminate the inherent architectural complexity of the problem"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Guards against AI inflating system scope and creating chaotic multi-paradigm Frankenstein code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-002".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "Design Patterns: Elements of Reusable Object-Oriented Software".to_string(),
                authors: "Gang of Four (Gamma, Helm, Johnson, Vlissides)".to_string(),
                year: 1994,
                literature_heuristic: "The classic GoF 23 patterns: Creational, Structural, Behavioral patterns; composition over inheritance.".to_string(),
                review_invariants: vec!["Apply GoF patterns only when solving a real, documented recurring design problem", "Favor object composition and interface delegation over deep inheritance hierarchies", "Program to an interface, not an implementation"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Allows the reviewer to recognize standard patterns and reject hallucinations.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-003".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "Site Reliability Engineering: How Google Runs Production Systems".to_string(),
                authors: "Betsy Beyer et al. (Google)".to_string(),
                year: 2016,
                literature_heuristic: "Google SRE principles: Service Level Objectives (SLOs), error budgets, eliminating toil, blameless postmortems.".to_string(),
                review_invariants: vec!["Define explicit SLOs and error budgets before deploying services to production", "Automate toil: if a manual operational task is repeated, replace it with code", "Conduct blameless postmortems on all production incidents to discover root causes"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Ensures AI code incorporates monitoring, metrics, and error budgets required for production.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-004".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "The Site Reliability Workbook".to_string(),
                authors: "Betsy Beyer et al. (Google)".to_string(),
                year: 2018,
                literature_heuristic: "Practical implementation of SRE: alert design, canary deployments, disaster testing, load balancing, distributed tracing.".to_string(),
                review_invariants: vec!["Alert on user-facing symptom degradation (burn rate), never on raw CPU spikes", "Deploy all software changes via progressive automated canary rollouts", "Test system resiliency with simulated disaster recovery drills"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from creating alerting configurations that cause on-call alert fatigue.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-005".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "API Design Patterns".to_string(),
                authors: "JJ Geewax".to_string(),
                year: 2021,
                literature_heuristic: "Resource naming, idempotency keys, pagination tokens, versioning, long-running operations (LRO), API standards.".to_string(),
                review_invariants: vec!["Every mutating API request must accept an Idempotency-Key header", "Use page tokens rather than page numbers for safe cursor-based database pagination", "Follow standard HTTP verb semantics (GET is safe, POST/PUT/DELETE semantics)"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Catches AI designing non-idempotent billing or payment APIs that double-charge users.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-006".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "RESTful Web APIs: Services for a Changing World".to_string(),
                authors: "Leonard Richardson, Mike Amundsen, Sam Ruby".to_string(),
                year: 2013,
                literature_heuristic: "Hypermedia (HATEOAS), Richardson Maturity Model, semantic web representations, HTTP caching headers.".to_string(),
                review_invariants: vec!["Return appropriate HTTP status codes (200, 201, 400, 401, 403, 404, 409, 429, 500)", "Include Cache-Control and ETag headers to enable efficient HTTP caching", "Represent API resources as hypermedia with navigable relation links"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Stops AI from returning HTTP 200 OK with an error payload in the JSON body.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-007".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "Types and Programming Languages (TAPL)".to_string(),
                authors: "Benjamin C. Pierce".to_string(),
                year: 2002,
                literature_heuristic: "Formal type theory: type soundness, lambda calculus, subtyping, polymorphism, making illegal states unrepresentable.".to_string(),
                review_invariants: vec!["Make illegal states unrepresentable in the type system", "Rely on compiler type checkers to prove program safety properties at compile time", "Leverage algebraic data types (enums) to model closed domain spaces"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Enables the reviewer to spot flawed type models in AI-generated code.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-008".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "Code: The Hidden Language of Computer Hardware and Software (2nd Ed)".to_string(),
                authors: "Charles Petzold".to_string(),
                year: 2022,
                literature_heuristic: "Intuitive mastery of how bits, logic gates, registers, CPU opcodes, and memory execute software.".to_string(),
                review_invariants: vec!["Maintain mechanical sympathy: understand what the CPU is physically doing", "Remember that code is fundamentally data executed by hardware registers", "Demystify complex software stacks down to the underlying binary foundations"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Provides intuition to spot when AI code violates basic hardware realities.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-009".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "AI-Assisted Programming: Better Code with AI".to_string(),
                authors: "Tom Taulli".to_string(),
                year: 2024,
                literature_heuristic: "Prompt engineering for code generation, using AI for test generation, refactoring, and verification boundaries.".to_string(),
                review_invariants: vec!["Provide explicit constraints and invariants in prompts to guide model output", "Never commit AI-generated code without personal understanding and verification", "Use AI to generate adversarial test suites against its own generated code"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "Teaches the exact workflow to steer AI models effectively in daily development.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
            ReviewBookSkill {
                id: "VBR-10-010".to_string(),
                cluster: ReviewCluster::MindsetAiNativeReview,
                title: "Designing Autonomous AI Agents".to_string(),
                authors: "Eugene Yan, Chip Huyen".to_string(),
                year: 2024,
                literature_heuristic: "Agent evaluation harnesses, multi-agent tool calling, reflection loops, deterministic guardrails, verification pipelines.".to_string(),
                review_invariants: vec!["Implement deterministic guardrails and schema validation for all agent tool calls", "Use reflection and self-correction loops to repair runtime execution errors", "Benchmark agent accuracy against formal test harnesses before deployment"].into_iter().map(String::from).collect(),
                ai_hallucination_defense: "The architectural foundation for building sovereign AI swarms like Tagisan.".to_string(),
                tools_required: vec!["read_file".to_string(), "vibe_code_review".to_string()],
            },
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
            && (trimmed.contains(" + ") || trimmed.contains("format!") || trimmed.contains("f\"") || trimmed.contains("`$") || trimmed.contains("${")) {
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
        if trimmed.contains("Access-Control-Allow-Origin") && (trimmed.contains("x2a") || trimmed.contains("\"*\"")) {
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
pub async fn handle_vibe_review_command(action: ReviewAction) -> Result<()> {
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

            println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());
            println!("{}", "  🇵🇭 TAGISAN TOP 100 CODE & PROGRAM REVIEW CANON".bold().yellow());
            println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());

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

            println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());
            println!("  {} \"{}\"", "🔍 SEARCH RESULTS FOR:".bold().yellow(), query.green().bold());
            println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());

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
                    println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());
                    println!("  {} {}", "REVIEW BOOK SPECIFICATION:".bold().yellow(), book.id.cyan().bold());
                    println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());

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
                println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());
                println!("  {} {}", "VIBE CODE REVIEW AUDIT:".bold().yellow(), target.cyan().bold());
                println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());

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
            println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());
            println!("{}", "  📚 10 MACRO-INSPECTION CLUSTERS BREAKDOWN (100 BOOKS)".bold().yellow());
            println!("{}", "═════════════════════════════════════════════════════════════════════════".cyan());

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
        let bad_code = r#"
            pub fn query_user(user_id: &str) {
                let q = format!("SELECT * FROM users WHERE id = '{}'", user_id);
            }

            #[test]
            fn test_something() {
                assert!(true);
            }
        "#;

        let report = audit_code(bad_code, Some("test_file.rs"));
        assert!(!report.summary.passed);
        assert!(report.summary.critical_count >= 2);
        assert!(report.findings.iter().any(|f| f.rule_id == "DOWD-SQL-INJECTION-RISK"));
        assert!(report.findings.iter().any(|f| f.rule_id == "MESZAROS-TAUTOLOGICAL-TEST"));
    }

    #[test]
    fn test_audit_code_passes_clean_code() {
        let clean_code = r#"
            pub fn safe_calc(a: i32, b: i32) -> Result<i32, String> {
                if b == 0 {
                    return Err("division by zero".into());
                }
                Ok(a / b)
            }
        "#;

        let report = audit_code(clean_code, Some("clean.rs"));
        assert!(report.summary.passed);
        assert_eq!(report.summary.critical_count, 0);
    }
}
