#!/usr/bin/env python3
"""
scripts/patch_access365_skills_rs.py

Patches src/ecc/skills.rs to:
1. Append all 350 `pub fn acc_*() -> EccSkill` constructors.
2. Register all 350 in `all_built_in_skills()`.
3. Add high-leverage alias lookups to `find_built_in_skill()`.
4. Add domain prefixes ("acc-", "access-365-", "access-365", "access-", "msaccess-") to `infer_domain()`.
"""

import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_RS = BASE_DIR / "src" / "ecc" / "skills.rs"
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

sys.path.insert(0, str(BASE_DIR))
from scripts.generate_access365_skills import get_all_skills

def main():
    skills = get_all_skills()
    print(f"Loaded {len(skills)} Access 365 skills from generator.")
    assert len(skills) == 350, f"Expected 350 skills, got {len(skills)}"

    print(f"Reading {SKILLS_RS}...")
    content = SKILLS_RS.read_text(encoding="utf-8")

    # 1. Generate calls for all_built_in_skills()
    calls = []
    for s in skills:
        fn_name = s["id"].replace("-", "_")
        calls.append(f"        {fn_name}(),")
    
    calls_block = "\n        // Top 350 Skills for Microsoft Access 365 & Vibe Code Developers\n" + "\n".join(calls) + "\n"
    
    target_all_skills = "        o365_saas_autonomous_multi_agent_saas_swarms(),\n    ]"
    if target_all_skills not in content:
        raise ValueError("Could not find insertion point 'o365_saas_autonomous_multi_agent_saas_swarms(),\\n    ]' in all_built_in_skills()")
    
    replacement_all_skills = f"        o365_saas_autonomous_multi_agent_saas_swarms(),{calls_block}    ]"
    content = content.replace(target_all_skills, replacement_all_skills, 1)
    print("  [+] Inserted 350 skill calls into all_built_in_skills()")

    # 2. Generate high-leverage alias lookups for find_built_in_skill()
    alias_lines = ["    // Top 350 Skills for Microsoft Access 365 & Vibe Code Developers Aliases"]
    for s in skills:
        target_skill = s["id"]
        triggers = s["triggers"]
        if triggers:
            conds = " || ".join(f'lower == "{t.lower()}"' for t in triggers)
            alias_lines.append(f'    if {conds} {{\n        return find_built_in_skill("{target_skill}");\n    }}')
    
    alias_block = "\n".join(alias_lines) + "\n"
    
    target_find = '    let lower = name.to_lowercase().replace(\'_\', "-");\n'
    if target_find not in content:
        raise ValueError("Could not find insertion point in find_built_in_skill()")
    
    content = content.replace(target_find, target_find + alias_block, 1)
    print("  [+] Inserted alias lookups into find_built_in_skill()")

    # 3. Generate domain prefixes for infer_domain()
    domain_prefixes = """        ("acc-", "access-365"),
        ("access-365-", "access-365"),
        ("access-365", "access-365"),
        ("access-", "access-365"),
        ("msaccess-", "access-365"),
        ("ms-access-", "access-365"),
"""
    target_prefixes = 'fn infer_domain(name: &str) -> String {\n    let lower = name.to_lowercase();\n    let prefixes = [\n'
    if target_prefixes not in content:
        raise ValueError("Could not find insertion point in infer_domain()")
    
    content = content.replace(target_prefixes, target_prefixes + domain_prefixes, 1)
    print("  [+] Inserted domain prefixes into infer_domain()")

    # 4. Generate all 350 pub fn acc_*() definitions
    fn_defs = []
    for i, s in enumerate(skills, 1):
        skill_id = s["id"]
        fn_name = skill_id.replace("-", "_")
        skill_file = SKILLS_DIR / skill_id / "SKILL.md"
        skill_content = skill_file.read_text(encoding="utf-8")
        
        escaped_desc = s["desc"].replace('\\', '\\\\').replace('"', '\\"')
        
        # Raw string delimiters
        r_open = 'r##"' if '#"' in skill_content else 'r#"'
        r_close = '"##' if '#"' in skill_content else '"#'
        
        fn_def = f"""/// {i}. {skill_id} Skill (Cluster {s["cluster"]})
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

    all_fn_defs = "\n// =========================================================================\n// Top 350 Skills for Microsoft Access 365 & Vibe Code Developers\n// =========================================================================\n\n" + "\n".join(fn_defs)
    
    # Append to the end of content
    content = content.rstrip() + "\n" + all_fn_defs + "\n"
    print("  [+] Appended 350 constructor functions to src/ecc/skills.rs")

    print(f"Writing updated {SKILLS_RS}...")
    SKILLS_RS.write_text(content, encoding="utf-8")
    print("[+] Successfully patched src/ecc/skills.rs with all 350 Access 365 skills!")

if __name__ == "__main__":
    main()
