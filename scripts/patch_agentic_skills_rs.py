#!/usr/bin/env python3
"""
scripts/patch_agentic_skills_rs.py

Patches src/ecc/skills.rs to:
1. Append all 100 `pub fn agentic_*() -> EccSkill` constructors.
2. Register all 100 in `all_built_in_skills()`.
3. Add high-leverage alias lookups to `find_built_in_skill()`.
4. Add domain prefixes to `infer_domain()`.
"""

import sys
from pathlib import Path

# Base directories
BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_RS = BASE_DIR / "src" / "ecc" / "skills.rs"
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

# Import skill definitions from generator
sys.path.insert(0, str(BASE_DIR))
from scripts.generate_agentic_skills import SKILLS

def main():
    print(f"Reading {SKILLS_RS}...")
    content = SKILLS_RS.read_text(encoding="utf-8")

    # 1. Generate calls for all_built_in_skills()
    calls = []
    for s in SKILLS:
        fn_name = s["id"].replace("-", "_")
        calls.append(f"        {fn_name}(),")
    
    calls_block = "\n        // Top 100 Books for Agentic Engineering & Vibe Code Developers\n" + "\n".join(calls) + "\n"
    
    target_all_skills = "        tla_consensus_formal_model_checker(),\n    ]"
    if target_all_skills not in content:
        raise ValueError("Could not find insertion point 'tla_consensus_formal_model_checker(),\\n    ]' in all_built_in_skills()")
    
    replacement_all_skills = f"        tla_consensus_formal_model_checker(),{calls_block}    ]"
    content = content.replace(target_all_skills, replacement_all_skills, 1)
    print("  [+] Inserted 100 skill calls into all_built_in_skills()")

    # 2. Generate high-leverage alias lookups for find_built_in_skill()
    aliases = [
        # Cluster 1
        ("wooldridge", ["wooldridge", "multiagent-systems", "bdi", "bdi-architecture", "fipa-acl", "contract-net-protocol"], "agentic-wooldridge-multiagent-systems"),
        ("shoham", ["shoham", "leyton-brown", "discsp", "mechanism-design"], "agentic-shoham-multiagent-foundations"),
        ("weiss", ["weiss", "distributed-ai", "blackboard-architecture"], "agentic-weiss-distributed-ai"),
        ("ferber", ["ferber", "reactive-agents", "subsumption"], "agentic-ferber-reactive-agents"),
        ("kennedy", ["kennedy", "eberhart", "pso", "particle-swarm"], "agentic-kennedy-swarm-intelligence"),
        ("bonabeau", ["bonabeau", "stigmergy", "digital-pheromones"], "agentic-bonabeau-swarm-stigmergy"),
        ("camazine", ["camazine", "self-organization", "quorum-sensing"], "agentic-camazine-self-organization"),
        ("reynolds", ["reynolds", "reynolds-steering", "autonomous-steering-agents"], "agentic-reynolds-autonomous-agents"),
        ("dorigo", ["dorigo", "aco", "ant-colony"], "agentic-dorigo-ant-colony"),
        ("murray", ["murray", "olfati-saber", "graph-laplacian", "consensus-cooperation"], "agentic-murray-consensus-cooperation"),
        # Cluster 2
        ("alammar", ["alammar", "illustrated-transformer", "transformer-mechanics", "kv-cache"], "agentic-alammar-transformer-mechanics"),
        ("chip-huyen", ["chip-huyen", "huyen", "ai-engineering"], "agentic-huyen-ai-engineering"),
        ("rothman", ["rothman", "transformers-nlp"], "agentic-rothman-transformers-nlp"),
        ("tunstall", ["tunstall", "von-werra", "peft-lora", "nlp-transformers"], "agentic-tunstall-nlp-transformers"),
        ("jurafsky", ["jurafsky", "slp", "speech-and-language-processing"], "agentic-jurafsky-slp-language-models"),
        ("goldberg", ["goldberg", "neural-nlp"], "agentic-goldberg-nn-nlp"),
        ("ravichandiran", ["ravichandiran", "llm-engineering"], "agentic-ravichandiran-llm-engineering"),
        ("briggs", ["briggs", "chain-of-thought", "prompt-engineering"], "agentic-briggs-prompt-engineering"),
        ("kamphuis", ["kamphuis", "in-context-reasoning"], "agentic-kamphuis-in-context-reasoning"),
        ("chollet", ["chollet", "deep-learning-intuition"], "agentic-chollet-deep-learning-intuition"),
        # Cluster 3
        ("russell-norvig", ["russell-norvig", "aima", "rational-agents", "peas-framework"], "agentic-russell-norvig-aima"),
        ("ghallab", ["ghallab", "automated-planning", "strips", "pddl", "htn"], "agentic-ghallab-automated-planning"),
        ("geffner", ["geffner", "heuristic-planning", "delete-relaxation"], "agentic-geffner-heuristic-search-planning"),
        ("sutton-barto", ["sutton-barto", "sutton", "reinforcement-learning", "bellman-equation"], "agentic-sutton-barto-reinforcement-learning"),
        ("pearl-search", ["pearl-search", "a-star", "heuristic-search"], "agentic-pearl-heuristic-search"),
        ("kaelbling", ["kaelbling", "pomdp", "belief-state"], "agentic-kaelbling-pomdp-planning"),
        ("thrun", ["thrun", "probabilistic-robotics", "slam", "kalman-filter"], "agentic-thrun-probabilistic-robotics"),
        ("silver", ["silver", "mcts", "alphazero", "monte-carlo-tree-search"], "agentic-silver-mcts-decision-trees"),
        ("yao", ["yao", "react", "thought-action-observation"], "agentic-yao-react-interleaved-reasoning"),
        ("shinn", ["shinn", "reflexion", "verbal-reinforcement"], "agentic-shinn-reflexion-self-correction"),
        # Cluster 4
        ("karpathy", ["karpathy", "vibe-coding", "conversational-programming"], "agentic-karpathy-vibe-coding-paradigm"),
        ("csikszentmihalyi", ["csikszentmihalyi", "flow-state", "developer-flow"], "agentic-csikszentmihalyi-flow-state"),
        ("hunt", ["pragmatic-programmer", "tracer-bullets", "dry-principle", "broken-windows"], "agentic-hunt-pragmatic-programmer"),
        ("raymond", ["cathedral-bazaar", "linus-law", "release-early-often"], "agentic-raymond-cathedral-bazaar"),
        ("graham", ["paul-graham", "hackers-painters"], "agentic-graham-hackers-painters"),
        ("beck-xp", ["extreme-programming", "pair-programming"], "agentic-beck-extreme-programming"),
        ("ries", ["eric-ries", "lean-startup", "mvp", "build-measure-learn"], "agentic-ries-lean-mvp-feedback"),
        ("knapp", ["design-sprint", "knapp", "facade-prototyping"], "agentic-knapp-design-sprint-prototyping"),
        ("norman-ux", ["don-norman", "human-centered-design"], "agentic-norman-human-centered-interfaces"),
        ("krug", ["krug", "dont-make-me-think"], "agentic-krug-intuitive-interaction"),
        # Cluster 5
        ("lewis-rag", ["patrick-lewis", "rag", "retrieval-augmented-generation"], "agentic-lewis-rag-foundations"),
        ("manning", ["manning", "inverted-index", "bm25"], "agentic-manning-information-retrieval"),
        ("baeza-yates", ["baeza-yates", "modern-retrieval", "ndcg"], "agentic-baeza-yates-vector-retrieval"),
        ("malkov", ["malkov", "hnsw", "ann-search"], "agentic-malkov-hnsw-vector-indexing"),
        ("anderson", ["anderson", "act-r", "declarative-procedural"], "agentic-anderson-actr-cognitive-memory"),
        ("baddeley", ["baddeley", "working-memory", "central-executive"], "agentic-baddeley-working-memory-buffers"),
        ("tulving", ["tulving", "episodic-memory", "autonoetic-consciousness"], "agentic-tulving-episodic-memory-retrieval"),
        ("sowa", ["sowa", "conceptual-graphs"], "agentic-sowa-knowledge-representation"),
        ("baader", ["baader", "description-logics", "tbox-abox"], "agentic-baader-description-logics-ontologies"),
        ("graph-rag", ["graph-rag", "neo4j-patterns", "property-graphs"], "agentic-robinson-graph-rag-knowledge"),
        # Cluster 6
        ("gulwani", ["gulwani", "program-synthesis", "programming-by-example"], "agentic-gulwani-program-synthesis"),
        ("dragon-book", ["dragon-book", "compiler-parsing"], "agentic-aho-dragon-compiler-parsing"),
        ("cooper-torczon", ["cooper-torczon", "compiler-optimization", "ssa-form"], "agentic-cooper-compiler-ir-optimization"),
        ("muchnick", ["muchnick", "dataflow-analysis", "live-variables"], "agentic-muchnick-cfg-dataflow-analysis"),
        ("nystrom", ["nystrom", "crafting-interpreters", "bytecode-vm", "pratt-parsing"], "agentic-nystrom-crafting-interpreters"),
        ("fowler-dsl", ["fowler-dsl", "domain-specific-languages", "fluent-interface"], "agentic-fowler-domain-specific-languages"),
        ("antlr4", ["antlr4", "parr", "grammar-dsl"], "agentic-parr-antlr4-grammar-dsl"),
        ("tapl", ["tapl", "pierce", "type-systems", "type-soundness"], "agentic-pierce-type-systems-soundness"),
        ("harper", ["harper", "pfpl", "operational-semantics"], "agentic-harper-practical-foundations-pl"),
        ("sicp", ["sicp", "abelson-sussman", "metacircular-evaluator"], "agentic-sicp-evaluator-metacircular"),
        # Cluster 7
        ("amodei", ["amodei", "ai-safety", "reward-hacking"], "agentic-amodei-concrete-ai-safety"),
        ("hendrycks", ["hendrycks", "mmlu", "llm-benchmarking"], "agentic-hendrycks-benchmarking-evals"),
        ("perez", ["perez", "red-teaming", "adversarial-fuzzing"], "agentic-perez-red-teaming-adversarial"),
        ("constitutional-ai", ["constitutional-ai", "rlaif", "anthropic-constitutional"], "agentic-anthropic-constitutional-ai"),
        ("ozkaya", ["ozkaya", "swe-bench", "software-evals", "pass-at-k"], "agentic-ozkaya-llm-software-evals"),
        ("beck-tdd", ["tdd-contracts", "triangulation"], "agentic-beck-tdd-verifiable-contracts"),
        ("claessen", ["claessen", "hughes", "quickcheck", "property-based-testing"], "agentic-claessen-property-based-testing"),
        ("maciver", ["maciver", "hypothesis-testing", "test-shrinking"], "agentic-maciver-invariant-shrinking"),
        ("clarke", ["clarke", "model-checking", "temporal-logic"], "agentic-clarke-model-checking-invariants"),
        ("baier-katoen", ["baier-katoen", "ltl", "ctl", "buchi-automata"], "agentic-baier-temporal-logic-ltl-ctl"),
        # Cluster 8
        ("kleppmann", ["kleppmann", "ddia", "distributed-consistency", "linearizability"], "agentic-kleppmann-distributed-consistency"),
        ("tanenbaum", ["tanenbaum", "distributed-systems", "rpc-middleware"], "agentic-tanenbaum-distributed-systems"),
        ("lamport", ["lamport", "logical-clocks", "happens-before", "vector-clocks"], "agentic-lamport-logical-clocks"),
        ("ongaro", ["ongaro", "raft", "raft-consensus"], "agentic-ongaro-raft-distributed-consensus"),
        ("hohpe", ["hohpe", "enterprise-integration", "pipes-and-filters"], "agentic-hohpe-enterprise-integration-patterns"),
        ("sam-newman", ["sam-newman", "microservices-isolation", "circuit-breakers"], "agentic-newman-microservices-tool-isolation"),
        ("software-architecture", ["software-architecture", "fitness-functions"], "agentic-richards-software-architecture-tradeoffs"),
        ("fielding", ["fielding", "rest", "hateoas"], "agentic-fielding-rest-agent-apis"),
        ("henning", ["henning", "protocol-buffers", "rpc-contracts"], "agentic-henning-rpc-schema-contracts"),
        ("mcp-protocol", ["mcp-protocol", "model-context-protocol", "mcp-server"], "agentic-mcp-protocol-specification"),
        # Cluster 9
        ("soar", ["soar", "soar-architecture", "laird"], "agentic-laird-soar-cognitive-architecture"),
        ("minsky", ["minsky", "society-of-mind", "censors-suppressors"], "agentic-minsky-society-of-mind"),
        ("kahneman", ["kahneman", "system-1-system-2", "thinking-fast-and-slow"], "agentic-kahneman-dual-process-thinking"),
        ("hofstadter", ["hofstadter", "geb", "strange-loops"], "agentic-hofstadter-strange-loops-recursion"),
        ("herbert-simon", ["herbert-simon", "bounded-rationality", "satisficing"], "agentic-simon-bounded-rationality-heuristics"),
        ("newell", ["newell", "unified-cognition", "problem-space"], "agentic-newell-unified-cognition"),
        ("clarion", ["clarion", "ron-sun", "implicit-explicit"], "agentic-sun-clarion-implicit-explicit"),
        ("brenden-lake", ["brenden-lake", "concept-learning", "meta-learning"], "agentic-lake-cognitive-concept-learning"),
        ("schmidhuber", ["schmidhuber", "godel-machine", "artificial-curiosity"], "agentic-schmidhuber-intrinsic-curiosity"),
        ("pei-wang", ["pei-wang", "nars", "non-axiomatic-logic"], "agentic-wang-nars-non-axiomatic-reasoning"),
        # Cluster 10
        ("shneiderman", ["shneiderman", "human-centered-ai", "hcai"], "agentic-shneiderman-human-centered-ai"),
        ("brooks-law", ["brooks-law", "mythical-man-month", "conceptual-integrity"], "agentic-brooks-mythical-man-month"),
        ("ousterhout", ["ousterhout", "philosophy-software-design", "deep-modules"], "agentic-ousterhout-philosophy-software-design"),
        ("clean-architecture", ["clean-architecture", "uncle-bob", "dependency-inversion"], "agentic-martin-clean-architecture-boundaries"),
        ("feathers", ["feathers", "legacy-code", "characterization-tests"], "agentic-feathers-legacy-code-refactoring"),
        ("accelerate", ["accelerate", "dora-metrics", "continuous-delivery"], "agentic-forsgren-accelerate-dora-metrics"),
        ("phoenix-project", ["phoenix-project", "three-ways", "theory-of-constraints"], "agentic-kim-phoenix-project-flow-theory"),
        ("domain-driven-design", ["domain-driven-design", "eric-evans", "bounded-context", "ubiquitous-language"], "agentic-evans-domain-driven-design"),
        ("donella-meadows", ["donella-meadows", "systems-thinking", "stocks-and-flows"], "agentic-meadows-systems-thinking-feedback"),
        ("kevin-kelly", ["kevin-kelly", "the-inevitable", "cognifying"], "agentic-kelly-autonomous-cognition-flows"),
    ]

    alias_lines = ["    // Top 100 Books for Agentic Engineering & Vibe Coding Aliases"]
    for _, triggers, target_skill in aliases:
        conds = " || ".join(f'lower == "{t}"' for t in triggers)
        alias_lines.append(f'    if {conds} {{\n        return find_built_in_skill("{target_skill}");\n    }}')
    
    alias_block = "\n".join(alias_lines) + "\n"
    
    target_find = '    let lower = name.to_lowercase().replace(\'_\', "-");\n'
    if target_find not in content:
        raise ValueError("Could not find insertion point in find_built_in_skill()")
    
    content = content.replace(target_find, target_find + alias_block, 1)
    print("  [+] Inserted alias lookups into find_built_in_skill()")

    # 3. Generate domain prefixes for infer_domain()
    domain_prefixes = """        ("agentic-", "agentic"),
        ("agentic", "agentic"),
        ("vibe", "vibe"),
"""
    target_prefixes = 'fn infer_domain(name: &str) -> String {\n    let lower = name.to_lowercase();\n    let prefixes = [\n'
    if target_prefixes not in content:
        raise ValueError("Could not find insertion point in infer_domain()")
    
    content = content.replace(target_prefixes, target_prefixes + domain_prefixes, 1)
    print("  [+] Inserted domain prefixes into infer_domain()")

    # 4. Generate all 100 pub fn agentic_*() definitions
    fn_defs = []
    for i, s in enumerate(SKILLS, 1):
        skill_id = s["id"]
        fn_name = skill_id.replace("-", "_")
        skill_file = SKILLS_DIR / skill_id / "SKILL.md"
        skill_content = skill_file.read_text(encoding="utf-8")
        
        escaped_desc = s["desc"].replace('\\', '\\\\').replace('"', '\\"')
        
        # Raw string delimiters
        r_open = 'r##"' if '#"' in skill_content else 'r#"'
        r_close = '"##' if '#"' in skill_content else '"#'
        
        fn_def = f"""/// {i}. {skill_id} Skill
pub fn {fn_name}() -> EccSkill {{
    EccSkill::new(
        "{skill_id}",
        "{escaped_desc}",
        {r_open}{skill_content}
{r_close},
    )
}}
"""
        fn_defs.append(fn_def)

    all_fn_defs = "\n// =========================================================================\n// Top 100 Books for Agentic Engineering & Vibe Code Developers\n// =========================================================================\n\n" + "\n".join(fn_defs)
    
    # Append to the end of content
    content = content.rstrip() + "\n" + all_fn_defs + "\n"
    print("  [+] Appended 100 constructor functions to src/ecc/skills.rs")

    print(f"Writing updated {SKILLS_RS}...")
    SKILLS_RS.write_text(content, encoding="utf-8")
    print("[+] Successfully patched src/ecc/skills.rs with all 100 Agentic skills!")

if __name__ == "__main__":
    main()
