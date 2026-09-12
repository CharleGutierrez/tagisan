#!/usr/bin/env python3
"""
Patch src/ecc/skills.rs to register all 50 Data Analytics skills.
"""

import os
import re

SKILLS_DIR = "/home/dyna/TGS Projects/tagisan/.ecc/skills"
TARGET_FILE = "/home/dyna/TGS Projects/tagisan/src/ecc/skills.rs"

SKILL_ORDER = [
    # Cluster 1
    "analytics-kimball-dimensional-modeling",
    "analytics-etl-pipeline-patterns",
    "analytics-data-vault-architecture",
    "analytics-dbt-modeling-dag",
    "analytics-advanced-sql-windowing",
    "analytics-celko-relational-logic",
    # Cluster 2
    "analytics-tukey-eda-heuristics",
    "analytics-practical-statistics",
    "analytics-intuitive-statistics",
    "analytics-statistical-learning-islp",
    "analytics-bayesian-rethinking",
    "analytics-mathematical-inference",
    # Cluster 3
    "analytics-kohavi-ab-experimentation",
    "analytics-causal-mixtape",
    "analytics-econometric-causality",
    "analytics-counterfactual-causality",
    # Cluster 4
    "analytics-kleppmann-data-intensive",
    "analytics-akidau-stream-processing",
    "analytics-kafka-event-streaming",
    "analytics-data-engineering-lifecycle",
    "analytics-realtime-olap-pipelines",
    # Cluster 5
    "analytics-python-pandas-wrangling",
    "analytics-duckdb-embedded-olap",
    "analytics-polars-lazy-processing",
    "analytics-high-performance-compute",
    "analytics-cli-data-science",
    # Cluster 6
    "analytics-tufte-visual-display",
    "analytics-storytelling-with-data",
    "analytics-wilke-data-visualization",
    "analytics-few-dashboard-design",
    "analytics-cairo-visual-integrity",
    "analytics-d3-interactive-graphics",
    # Cluster 7
    "analytics-parmenter-kpi-framework",
    "analytics-lean-startup-metrics",
    "analytics-hubbard-measurement-value",
    "analytics-semantic-metadata-layer",
    "analytics-semantic-metrics-governance",
    "analytics-okr-goal-tracking",
    # Cluster 8
    "analytics-hyndman-time-series",
    "analytics-box-jenkins-arima",
    "analytics-anomaly-outlier-detection",
    "analytics-python-signal-processing",
    # Cluster 9
    "analytics-feature-engineering-pipeline",
    "analytics-geron-ml-pipelines",
    "analytics-kuhn-predictive-modeling",
    "analytics-elements-statistical-learning",
    # Cluster 10
    "analytics-redman-data-quality",
    "analytics-data-observability-monitors",
    "analytics-dama-data-governance",
    "analytics-dataops-automated-testing",
]

def parse_skill_md(skill_name):
    skill_path = os.path.join(SKILLS_DIR, skill_name, "SKILL.md")
    with open(skill_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Split frontmatter
    parts = content.split("---", 2)
    if len(parts) < 3:
        raise ValueError(f"Invalid frontmatter in {skill_path}")
    
    frontmatter = parts[1]
    body = parts[2].strip()

    name = ""
    description = ""
    for line in frontmatter.splitlines():
        line = line.strip()
        if line.startswith("name:"):
            name = line[len("name:"):].strip().strip('"').strip("'")
        elif line.startswith("description:"):
            description = line[len("description:"):].strip().strip('"').strip("'")

    return name, description, body

def main():
    print(f"Reading {TARGET_FILE}...")
    with open(TARGET_FILE, "r", encoding="utf-8") as f:
        skills_rs = f.read()

    # 1. Generate function calls for all_built_in_skills()
    calls = []
    for skill_name in SKILL_ORDER:
        fn_name = skill_name.replace("-", "_")
        calls.append(f"        {fn_name}(),")
    calls_block = "\n        // Data Analytics & Analytics Engineering Architect Skills (Top 50 Books)\n" + "\n".join(calls)

    target_all_skills = "        erp_tax_engine_jurisdiction_rules(),\n    ]"
    replacement_all_skills = f"        erp_tax_engine_jurisdiction_rules(),{calls_block}\n    ]"

    if target_all_skills not in skills_rs:
        raise ValueError("Could not find insertion point in all_built_in_skills()")
    skills_rs = skills_rs.replace(target_all_skills, replacement_all_skills, 1)

    # 2. Generate high-leverage alias lookups for find_built_in_skill()
    alias_code = """    // Data Analytics & Analytics Engineering Skills Aliases
    if lower == "kimball" || lower == "star-schema" || lower == "dimensional-modeling" || lower == "snowflake-schema" {
        return find_built_in_skill("analytics-kimball-dimensional-modeling");
    }
    if lower == "etl-patterns" || lower == "kimball-etl" || lower == "data-warehouse-etl" {
        return find_built_in_skill("analytics-etl-pipeline-patterns");
    }
    if lower == "data-vault" || lower == "raw-vault" || lower == "business-vault" {
        return find_built_in_skill("analytics-data-vault-architecture");
    }
    if lower == "dbt" || lower == "dbt-dag" || lower == "analytics-engineering" {
        return find_built_in_skill("analytics-dbt-modeling-dag");
    }
    if lower == "window-functions" || lower == "advanced-sql" || lower == "sql-windowing" {
        return find_built_in_skill("analytics-advanced-sql-windowing");
    }
    if lower == "celko" || lower == "nested-sets" || lower == "relational-division" {
        return find_built_in_skill("analytics-celko-relational-logic");
    }
    if lower == "tukey-eda" || lower == "tukey" || lower == "eda" || lower == "tukey-fences" {
        return find_built_in_skill("analytics-tukey-eda-heuristics");
    }
    if lower == "practical-statistics" || lower == "bootstrap-resampling" || lower == "permutation-test" {
        return find_built_in_skill("analytics-practical-statistics");
    }
    if lower == "intuitive-statistics" || lower == "naked-statistics" || lower == "simpsons-paradox" {
        return find_built_in_skill("analytics-intuitive-statistics");
    }
    if lower == "statistical-learning" || lower == "islp" || lower == "bias-variance" {
        return find_built_in_skill("analytics-statistical-learning-islp");
    }
    if lower == "bayesian-rethinking" || lower == "mcelreath" || lower == "collider-bias" {
        return find_built_in_skill("analytics-bayesian-rethinking");
    }
    if lower == "mathematical-inference" || lower == "all-of-statistics" || lower == "wasserman" {
        return find_built_in_skill("analytics-mathematical-inference");
    }
    if lower == "ab-testing" || lower == "srm" || lower == "sample-ratio-mismatch" || lower == "kohavi" {
        return find_built_in_skill("analytics-kohavi-ab-experimentation");
    }
    if lower == "causal-mixtape" || lower == "rubin-causal-model" || lower == "potential-outcomes" || lower == "analytics-causal-inference" {
        return find_built_in_skill("analytics-causal-mixtape");
    }
    if lower == "difference-in-differences" || lower == "did" || lower == "econometric-causality" || lower == "mostly-harmless-econometrics" {
        return find_built_in_skill("analytics-econometric-causality");
    }
    if lower == "counterfactual" || lower == "counterfactual-causality" || lower == "ipw" {
        return find_built_in_skill("analytics-counterfactual-causality");
    }
    if lower == "duckdb" || lower == "embedded-olap" {
        return find_built_in_skill("analytics-duckdb-embedded-olap");
    }
    if lower == "polars" || lower == "lazyframe" {
        return find_built_in_skill("analytics-polars-lazy-processing");
    }
    if lower == "tufte" || lower == "data-ink" || lower == "chartjunk" {
        return find_built_in_skill("analytics-tufte-visual-display");
    }
    if lower == "storytelling-with-data" || lower == "knaflic" {
        return find_built_in_skill("analytics-storytelling-with-data");
    }
    if lower == "bullet-graph" || lower == "stephen-few" {
        return find_built_in_skill("analytics-few-dashboard-design");
    }
    if lower == "kpi" || lower == "parmenter" || lower == "kri" {
        return find_built_in_skill("analytics-parmenter-kpi-framework");
    }
    if lower == "cohort-retention" || lower == "lean-analytics" || lower == "omtm" || lower == "aarrr" {
        return find_built_in_skill("analytics-lean-startup-metrics");
    }
    if lower == "time-series-forecasting" || lower == "stl-decomposition" || lower == "hyndman" {
        return find_built_in_skill("analytics-hyndman-time-series");
    }
    if lower == "arima" || lower == "box-jenkins" || lower == "sarima" {
        return find_built_in_skill("analytics-box-jenkins-arima");
    }
    if lower == "isolation-forest" || lower == "outlier-detection" || lower == "anomaly-detection" {
        return find_built_in_skill("analytics-anomaly-outlier-detection");
    }
    if lower == "data-observability" || lower == "5-pillars-observability" || lower == "freshness-monitoring" {
        return find_built_in_skill("analytics-data-observability-monitors");
    }
    if lower == "data-governance" || lower == "dama" || lower == "dmbok" || lower == "master-data-management" {
        return find_built_in_skill("analytics-dama-data-governance");
    }
    if lower == "dataops" || lower == "dataops-testing" || lower == "data-pipeline-testing" {
        return find_built_in_skill("analytics-dataops-automated-testing");
    }
"""
    target_find_skill = "    if lower == \"tax-engine\" || lower == \"sales-tax\" || lower == \"vat\" || lower == \"tax-nexus\" {\n        return find_built_in_skill(\"erp-tax-engine-jurisdiction-rules\");\n    }\n    all_built_in_skills().into_iter().find(|s| s.name == lower)"
    replacement_find_skill = "    if lower == \"tax-engine\" || lower == \"sales-tax\" || lower == \"vat\" || lower == \"tax-nexus\" {\n        return find_built_in_skill(\"erp-tax-engine-jurisdiction-rules\");\n    }\n" + alias_code + "    all_built_in_skills().into_iter().find(|s| s.name == lower)"

    if target_find_skill not in skills_rs:
        raise ValueError("Could not find insertion point in find_built_in_skill()")
    skills_rs = skills_rs.replace(target_find_skill, replacement_find_skill, 1)

    # 3. Add domain prefixes to infer_domain()
    prefixes_code = """        ("analytics", "analytics"),
        ("warehouse", "warehouse"),
        ("olap", "olap"),
        ("dbt", "dbt"),
        ("statistics", "statistics"),
        ("experimentation", "experimentation"),
        ("streaming", "streaming"),
        ("duckdb", "duckdb"),
        ("polars", "polars"),
        ("visualization", "visualization"),
        ("metrics", "metrics"),
        ("forecasting", "forecasting"),
        ("observability", "observability"),
        ("dataops", "dataops"),
"""
    target_prefixes = '        ("kanban", "erp"),\n    ];'
    replacement_prefixes = f'        ("kanban", "erp"),\n{prefixes_code}    ];'

    if target_prefixes not in skills_rs:
        raise ValueError("Could not find insertion point in infer_domain()")
    skills_rs = skills_rs.replace(target_prefixes, replacement_prefixes, 1)

    # 4. Generate all 50 pub fn analytics_*() definitions
    fn_defs = []
    for i, skill_name in enumerate(SKILL_ORDER, start=121):
        name, description, body = parse_skill_md(skill_name)
        fn_name = skill_name.replace("-", "_")

        escaped_desc = description.replace('\\', '\\\\').replace('"', '\\"')

        # Raw string literal r#"..."#
        # If body contains "#, we can use r##"..."##
        r_delim = 'r##"' if '#"' in body else 'r#"'
        r_close = '"##' if '#"' in body else '"#'

        fn_def = f"""/// {i}. {skill_name} Skill
pub fn {fn_name}() -> EccSkill {{
    EccSkill::new(
        "{name}",
        "{escaped_desc}",
        {r_delim}{body}
{r_close},
    )
}}
"""
        fn_defs.append(fn_def)

    all_fn_defs = "\n" + "\n".join(fn_defs)

    target_load_skills = "/// Discover and load all ECC skills from a directory (scanning both `*.md` and `<dir>/SKILL.md`)\npub fn load_skills_from_dir"
    replacement_load_skills = all_fn_defs + "\n" + target_load_skills

    if target_load_skills not in skills_rs:
        raise ValueError("Could not find insertion point for function definitions before load_skills_from_dir")
    skills_rs = skills_rs.replace(target_load_skills, replacement_load_skills, 1)

    print(f"Writing updated {TARGET_FILE}...")
    with open(TARGET_FILE, "w", encoding="utf-8") as f:
        f.write(skills_rs)

    print("Successfully patched src/ecc/skills.rs with all 50 Analytics skills!")

if __name__ == "__main__":
    main()
