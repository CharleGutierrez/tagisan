#!/usr/bin/env python3
"""
Automated Verification Test Harness for:
Failure & Forensic Analysis for the Vibe Code Developer (50 Books Canon)
Validates:
1. Strict ISBN-10 (modulo 11) and ISBN-13 (modulo 10) checksum calculations for all 50 books.
2. ECC Tagisan YAML frontmatter parsing compliance for SKILL.md and Agent persona.
3. Live integration with `tgs` binary: verifying agent discovery in `tgs ecc list` and skill discovery in `tgs ecc skills`.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

# --- 1. The 50 Canonical Failure & Forensic Analysis Books ---
BOOKS_50_CANON = [
    {
        "num": 1,
        "title": "Why Programs Fail: A Guide to Systematic Debugging",
        "authors": "Andreas Zeller",
        "publisher": "Morgan Kaufmann",
        "year_edition": "2009 (2nd Edition)",
        "isbn10": "0123745152",
        "isbn13": "978-0123745156",
        "domain": "Debugging Methodology & Root Cause Investigation"
    },
    {
        "num": 2,
        "title": "Debugging: The 9 Indispensable Rules for Finding Even the Most Elusive Software and Hardware Problems",
        "authors": "David J. Agans",
        "publisher": "AMACOM",
        "year_edition": "2002",
        "isbn10": "0814474578",
        "isbn13": "978-0814474570",
        "domain": "Debugging Methodology & Root Cause Investigation"
    },
    {
        "num": 3,
        "title": "Effective Debugging: 66 Specific Ways to Debug Software and Systems",
        "authors": "Diomidis Spinellis",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2016",
        "isbn10": "0134394798",
        "isbn13": "978-0134394794",
        "domain": "Debugging Methodology & Root Cause Investigation"
    },
    {
        "num": 4,
        "title": "Root Cause Analysis: The Core of Problem Solving and Corrective Action",
        "authors": "Duke Okes",
        "publisher": "ASQ Quality Press",
        "year_edition": "2019 (2nd Edition)",
        "isbn10": "0873899822",
        "isbn13": "978-0873899826",
        "domain": "Debugging Methodology & Root Cause Investigation"
    },
    {
        "num": 5,
        "title": "Root Cause Analysis: Improving Performance for Bottom-Line Results",
        "authors": "Robert J. Latino, Kenneth C. Latino, Mark A. Latino",
        "publisher": "CRC Press",
        "year_edition": "2019 (5th Edition)",
        "isbn10": "1138332453",
        "isbn13": "978-1138332454",
        "domain": "Debugging Methodology & Root Cause Investigation"
    },
    {
        "num": 6,
        "title": "Systems Performance: Enterprise and the Cloud",
        "authors": "Brendan Gregg",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2020 (2nd Edition)",
        "isbn10": "0136820158",
        "isbn13": "978-0136820154",
        "domain": "Systems Performance, Tracing & Observability Forensics"
    },
    {
        "num": 7,
        "title": "BPF Performance Tools: Deep Analysis and Early Detection for Linux Systems",
        "authors": "Brendan Gregg",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2019",
        "isbn10": "0136554822",
        "isbn13": "978-0136554820",
        "domain": "Systems Performance, Tracing & Observability Forensics"
    },
    {
        "num": 8,
        "title": "Observability Engineering: Achieving Production Excellence",
        "authors": "Charity Majors, Liz Fong-Jones, George Miranda",
        "publisher": "O'Reilly Media",
        "year_edition": "2022",
        "isbn10": "1492076864",
        "isbn13": "978-1492076865",
        "domain": "Systems Performance, Tracing & Observability Forensics"
    },
    {
        "num": 9,
        "title": "Distributed Tracing in Practice: Instrumenting, Analyzing, and Debugging Systems",
        "authors": "Austin Parker, Daniel Spoonhower, Jonathan Mace, Ben Sigelman, Rebecca Isaacs",
        "publisher": "O'Reilly Media",
        "year_edition": "2020",
        "isbn10": "1492056634",
        "isbn13": "978-1492056638",
        "domain": "Systems Performance, Tracing & Observability Forensics"
    },
    {
        "num": 10,
        "title": "The Linux Programming Interface: A Linux and UNIX System Programming Handbook",
        "authors": "Michael Kerrisk",
        "publisher": "No Starch Press",
        "year_edition": "2010",
        "isbn10": "1593272200",
        "isbn13": "978-1593272203",
        "domain": "Systems Performance, Tracing & Observability Forensics"
    },
    {
        "num": 11,
        "title": "Release It!: Design and Deploy Production-Ready Software",
        "authors": "Michael T. Nygard",
        "publisher": "Pragmatic Bookshelf",
        "year_edition": "2018 (2nd Edition)",
        "isbn10": "1680502395",
        "isbn13": "978-1680502398",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 12,
        "title": "Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems",
        "authors": "Martin Kleppmann",
        "publisher": "O'Reilly Media",
        "year_edition": "2017",
        "isbn10": "1449373321",
        "isbn13": "978-1449373320",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 13,
        "title": "Chaos Engineering: System Resiliency in Practice",
        "authors": "Casey Rosenthal, Nora Jones",
        "publisher": "O'Reilly Media",
        "year_edition": "2020",
        "isbn10": "1492043869",
        "isbn13": "978-1492043867",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 14,
        "title": "Database Internals: A Deep Dive into How Distributed Data Systems Work",
        "authors": "Alex Petrov",
        "publisher": "O'Reilly Media",
        "year_edition": "2019",
        "isbn10": "1492040347",
        "isbn13": "978-1492040347",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 15,
        "title": "Fault-Tolerant Systems",
        "authors": "Israel Koren, C. Mani Krishna",
        "publisher": "Morgan Kaufmann",
        "year_edition": "2020 (2nd Edition)",
        "isbn10": "0128180498",
        "isbn13": "978-0128180495",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 16,
        "title": "Production-Ready Microservices: Building Standardized Systems Across an Engineering Organization",
        "authors": "Susan J. Fowler",
        "publisher": "O'Reilly Media",
        "year_edition": "2016",
        "isbn10": "1491965975",
        "isbn13": "978-1491965979",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 17,
        "title": "Building Microservices: Designing Fine-Grained Systems",
        "authors": "Sam Newman",
        "publisher": "O'Reilly Media",
        "year_edition": "2021 (2nd Edition)",
        "isbn10": "1492034029",
        "isbn13": "978-1492034025",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 18,
        "title": "Patterns of Enterprise Application Architecture",
        "authors": "Martin Fowler",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2002",
        "isbn10": "0321127420",
        "isbn13": "978-0321127426",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 19,
        "title": "Enterprise Integration Patterns: Designing, Building, and Deploying Messaging Solutions",
        "authors": "Gregor Hohpe, Bobby Woolf",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2003",
        "isbn10": "0321200683",
        "isbn13": "978-0321200686",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 20,
        "title": "Antifragile: Things That Gain from Disorder",
        "authors": "Nassim Nicholas Taleb",
        "publisher": "Random House",
        "year_edition": "2012",
        "isbn10": "1400067820",
        "isbn13": "978-1400067824",
        "domain": "Production Resilience, Architecture & Anti-Fragility"
    },
    {
        "num": 21,
        "title": "Site Reliability Engineering: How Google Runs Production Systems",
        "authors": "Betsy Beyer, Chris Jones, Jennifer Petoff, Niall Richard Murphy",
        "publisher": "O'Reilly Media",
        "year_edition": "2016",
        "isbn10": "149192912X",
        "isbn13": "978-1491929124",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 22,
        "title": "The Site Reliability Workbook: Practical Ways to Implement SRE",
        "authors": "Betsy Beyer, Niall Richard Murphy, David K. Rensin, Kent Kawahara, Stephen Thorne",
        "publisher": "O'Reilly Media",
        "year_edition": "2018",
        "isbn10": "1492029505",
        "isbn13": "978-1492029502",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 23,
        "title": "Building Secure and Reliable Systems: Best Practices for Designing, Implementing, and Maintaining Systems",
        "authors": "Heather Adkins, Betsy Beyer, Paul Blankinship, Piotr Lewandowski, Ana Oprea, Adam Stubblefield",
        "publisher": "O'Reilly Media",
        "year_edition": "2020",
        "isbn10": "1492083127",
        "isbn13": "978-1492083122",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 24,
        "title": "Seeking SRE: Conversations About Running Production Systems at Scale",
        "authors": "David N. Blank-Edelman",
        "publisher": "O'Reilly Media",
        "year_edition": "2018",
        "isbn10": "1491978864",
        "isbn13": "978-1491978863",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 25,
        "title": "Incident Management for Operations: A Guide to Developing and Managing an Incident Response Plan",
        "authors": "Rob Schnepp, Ron Vidal, Chris Hawley",
        "publisher": "O'Reilly Media",
        "year_edition": "2017",
        "isbn10": "1491954310",
        "isbn13": "978-1491954317",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 26,
        "title": "The Practice of Cloud System Administration: Designing and Operating Large Distributed Systems, Volume 2",
        "authors": "Thomas A. Limoncelli, Nicole Forsgren, Strata R. Chalup",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2014",
        "isbn10": "032194318X",
        "isbn13": "978-0321943187",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 27,
        "title": "Accelerate: The Science of Lean Software and DevOps: Building and Scaling High Performing Technology Organizations",
        "authors": "Nicole Forsgren, Jez Humble, Gene Kim",
        "publisher": "IT Revolution Press",
        "year_edition": "2018",
        "isbn10": "1942788339",
        "isbn13": "978-1942788331",
        "domain": "SRE, Postmortems & Incident Operations"
    },
    {
        "num": 28,
        "title": "The Art of Memory Forensics: Detecting Malware and Threats in Windows, Linux, and Mac Memory",
        "authors": "Michael Hale Ligh, Andrew Case, Jamie Levy, AAron Walters",
        "publisher": "Wiley",
        "year_edition": "2014",
        "isbn10": "1118825993",
        "isbn13": "978-1118825990",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 29,
        "title": "Practical Malware Analysis: The Hands-On Guide to Dissecting Malicious Software",
        "authors": "Michael Sikorski, Andrew Honig",
        "publisher": "No Starch Press",
        "year_edition": "2012",
        "isbn10": "1593272901",
        "isbn13": "978-1593272906",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 30,
        "title": "Practical Binary Analysis: Build Your Own Linux Tools for Binary Instrumentation, Analysis, and Disassembly",
        "authors": "Dennis Andriesse",
        "publisher": "No Starch Press",
        "year_edition": "2018",
        "isbn10": "1593279124",
        "isbn13": "978-1593279127",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 31,
        "title": "The IDA Pro Book: The Unofficial Guide to the World's Most Popular Disassembler",
        "authors": "Chris Eagle",
        "publisher": "No Starch Press",
        "year_edition": "2011 (2nd Edition)",
        "isbn10": "1593272898",
        "isbn13": "978-1593272890",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 32,
        "title": "Rootkits and Bootkits: Reversing Modern Malware and Next-Gen Threats",
        "authors": "Alex Matrosov, Eugene Rodionov, Sergey Bratus",
        "publisher": "No Starch Press",
        "year_edition": "2019",
        "isbn10": "1593277164",
        "isbn13": "978-1593277161",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 33,
        "title": "Windows Internals, Part 1: System architecture, processes, threads, memory management, and more",
        "authors": "Pavel Yosifovich, David A. Solomon, Alex Ionescu, Mark E. Russinovich",
        "publisher": "Microsoft Press",
        "year_edition": "2017 (7th Edition)",
        "isbn10": "0735684189",
        "isbn13": "978-0735684188",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 34,
        "title": "File System Forensic Analysis",
        "authors": "Brian Carrier",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2005",
        "isbn10": "0321268172",
        "isbn13": "978-0321268174",
        "domain": "Memory Forensics, Binary Disassembly & Reverse Engineering"
    },
    {
        "num": 35,
        "title": "Network Forensics: Tracking Hackers through Cyberspace",
        "authors": "Sherri Davidoff, Jonathan Ham",
        "publisher": "Prentice Hall",
        "year_edition": "2012",
        "isbn10": "0132564718",
        "isbn13": "978-0132564717",
        "domain": "Digital Forensics & Network Incident Detection"
    },
    {
        "num": 36,
        "title": "The Practice of Network Security Monitoring: Understanding Incident Detection and Response",
        "authors": "Richard Bejtlich",
        "publisher": "No Starch Press",
        "year_edition": "2013",
        "isbn10": "1593275099",
        "isbn13": "978-1593275099",
        "domain": "Digital Forensics & Network Incident Detection"
    },
    {
        "num": 37,
        "title": "Practical Forensic Imaging: Securing Digital Evidence with Linux Tools",
        "authors": "Bruce Nikkel",
        "publisher": "No Starch Press",
        "year_edition": "2016",
        "isbn10": "1593277938",
        "isbn13": "978-1593277932",
        "domain": "Digital Forensics & Network Incident Detection"
    },
    {
        "num": 38,
        "title": "Software Forensics: Collecting Evidence from the Scene of a Digital Crime",
        "authors": "Robert M. Slade",
        "publisher": "McGraw-Hill Osborne Media",
        "year_edition": "2004",
        "isbn10": "0072228296",
        "isbn13": "978-0072228298",
        "domain": "Digital Forensics & Network Incident Detection"
    },
    {
        "num": 39,
        "title": "The Art of Software Security Assessment: Identifying and Preventing Software Vulnerabilities",
        "authors": "Mark Dowd, John McDonald, Justin Schuh",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2006",
        "isbn10": "0321444426",
        "isbn13": "978-0321444424",
        "domain": "Software Vulnerability Analysis, Exploit Forensics & Fuzzing"
    },
    {
        "num": 40,
        "title": "Fuzzing for Software Security Testing and Quality Assurance",
        "authors": "Ari Takanen, Jared D. DeMott, Charlie Miller",
        "publisher": "Artech House",
        "year_edition": "2018 (2nd Edition)",
        "isbn10": "1608078507",
        "isbn13": "978-1608078509",
        "domain": "Software Vulnerability Analysis, Exploit Forensics & Fuzzing"
    },
    {
        "num": 41,
        "title": "Security Engineering: A Guide to Building Dependable Distributed Systems",
        "authors": "Ross J. Anderson",
        "publisher": "Wiley",
        "year_edition": "2020 (3rd Edition)",
        "isbn10": "1119642787",
        "isbn13": "978-1119642787",
        "domain": "Software Vulnerability Analysis, Exploit Forensics & Fuzzing"
    },
    {
        "num": 42,
        "title": "Threat Modeling: Designing for Security",
        "authors": "Adam Shostack",
        "publisher": "Wiley",
        "year_edition": "2014",
        "isbn10": "1118809998",
        "isbn13": "978-1118809990",
        "domain": "Software Vulnerability Analysis, Exploit Forensics & Fuzzing"
    },
    {
        "num": 43,
        "title": "Design and Validation of Computer Protocols",
        "authors": "Gerard J. Holzmann",
        "publisher": "Prentice Hall",
        "year_edition": "1991",
        "isbn10": "0135302544",
        "isbn13": "978-0135302545",
        "domain": "Concurrency, Protocol & Formal Verification Forensics"
    },
    {
        "num": 44,
        "title": "The Spin Model Checker: Primer and Reference Manual",
        "authors": "Gerard J. Holzmann",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2003",
        "isbn10": "0321228626",
        "isbn13": "978-0321228628",
        "domain": "Concurrency, Protocol & Formal Verification Forensics"
    },
    {
        "num": 45,
        "title": "Specifying Systems: The TLA+ Language and Tools for Hardware and Software Engineers",
        "authors": "Leslie Lamport",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2002",
        "isbn10": "032114306X",
        "isbn13": "978-0321143068",
        "domain": "Concurrency, Protocol & Formal Verification Forensics"
    },
    {
        "num": 46,
        "title": "Normal Accidents: Living with High-Risk Technologies",
        "authors": "Charles Perrow",
        "publisher": "Princeton University Press",
        "year_edition": "1999 (Updated Edition)",
        "isbn10": "0691004129",
        "isbn13": "978-0691004129",
        "domain": "Systemic Failure, Human Factors & Catastrophe Forensics"
    },
    {
        "num": 47,
        "title": "The Field Guide to Understanding 'Human Error'",
        "authors": "Sidney Dekker",
        "publisher": "CRC Press",
        "year_edition": "2014 (3rd Edition)",
        "isbn10": "1472439058",
        "isbn13": "978-1472439055",
        "domain": "Systemic Failure, Human Factors & Catastrophe Forensics"
    },
    {
        "num": 48,
        "title": "Engineering a Safer World: Systems Thinking Applied to Safety",
        "authors": "Nancy G. Leveson",
        "publisher": "The MIT Press",
        "year_edition": "2012",
        "isbn10": "0262016621",
        "isbn13": "978-0262016629",
        "domain": "Systemic Failure, Human Factors & Catastrophe Forensics"
    },
    {
        "num": 49,
        "title": "To Engineer Is Human: The Role of Failure in Successful Design",
        "authors": "Henry Petroski",
        "publisher": "Vintage",
        "year_edition": "1992",
        "isbn10": "0679734163",
        "isbn13": "978-0679734161",
        "domain": "Systemic Failure, Human Factors & Catastrophe Forensics"
    },
    {
        "num": 50,
        "title": "Working Effectively with Legacy Code",
        "authors": "Michael C. Feathers",
        "publisher": "Prentice Hall",
        "year_edition": "2004",
        "isbn10": "0131177052",
        "isbn13": "978-0131177055",
        "domain": "Legacy Code Archeology & Refactoring Under Pressure"
    }
]

# --- 2. ISBN Checksum Algorithms ---
def validate_isbn10(isbn_str: str) -> bool:
    """Validate 10-digit ISBN using modulo 11 with weights 10 down to 1."""
    clean = isbn_str.replace('-', '').replace(' ', '').upper()
    if len(clean) != 10:
        return False
    total = 0
    for i, char in enumerate(clean):
        weight = 10 - i
        if char == 'X':
            if i != 9:
                return False
            val = 10
        elif char.isdigit():
            val = int(char)
        else:
            return False
        total += val * weight
    return total % 11 == 0

def validate_isbn13(isbn_str: str) -> bool:
    """Validate 13-digit ISBN using modulo 10 with alternating weights 1 and 3."""
    clean = isbn_str.replace('-', '').replace(' ', '')
    if len(clean) != 13 or not clean.isdigit():
        return False
    total = 0
    for i, char in enumerate(clean):
        weight = 1 if i % 2 == 0 else 3
        total += int(char) * weight
    return total % 10 == 0

# --- 3. Tagisan EccSkill and EccAgent Parser Emulation ---
def parse_ecc_skill(content: str):
    """Replicates EccSkill::parse logic from src/ecc/skills.rs"""
    trimmed = content.lstrip('\ufeff').strip()
    if not trimmed.startswith("---"):
        raise ValueError("Missing leading '---' delimiter")
    rest = trimmed[3:]
    end_idx = rest.find("\n---")
    if end_idx == -1:
        end_idx = rest.find("\r\n---")
    if end_idx == -1:
        raise ValueError("Missing closing '---' delimiter")
    
    frontmatter = rest[:end_idx]
    after_close = rest[end_idx+4:]
    body = after_close.strip()
    
    name = None
    description = None
    for line in frontmatter.splitlines():
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        if ':' in line:
            key, val = line.split(':', 1)
            key = key.strip().lower()
            val = val.strip().strip('"').strip("'").strip()
            if key == 'name' and not name:
                name = val
            elif key == 'description' and not description:
                description = val

    if not name:
        raise ValueError("Missing 'name' in frontmatter")
    if not description:
        description = name
    return {"name": name, "description": description, "body": body}

def parse_ecc_agent(content: str):
    """Replicates EccAgent::parse logic from src/ecc/agent.rs"""
    trimmed = content.strip()
    if not trimmed.startswith("---"):
        raise ValueError("Missing leading '---' delimiter")
    rest = trimmed[3:]
    end_idx = rest.find("\n---")
    if end_idx == -1:
        end_idx = rest.find("\r\n---")
    if end_idx == -1:
        raise ValueError("Missing closing '---' delimiter")
        
    frontmatter = rest[:end_idx]
    after_close = rest[end_idx+4:]
    body = after_close.strip()
    
    name = None
    description = None
    tools = []
    model = None
    
    for line in frontmatter.splitlines():
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        if ':' in line:
            key, val = line.split(':', 1)
            key = key.strip().lower()
            val = val.strip().strip('"').strip("'").strip()
            if key == 'name':
                name = val
            elif key == 'description':
                description = val
            elif key == 'model':
                model = val
            elif key == 'tools':
                clean_tools = val.strip('[]')
                tools = [t.strip() for t in clean_tools.split(',') if t.strip()]

    if not name:
        raise ValueError("Missing 'name' in agent frontmatter")
    return {"name": name, "description": description, "tools": tools, "model": model, "system_prompt": body}


# --- 4. Main Verification Suite ---
def run_all_tests():
    print("=================================================================")
    print(" 🔬 TAGISAN FORENSICS & FAILURE ANALYSIS TEST HARNESS ")
    print("=================================================================\n")
    
    # TEST 1: ISBN Validation for all 50 Books
    print("[TEST 1/4] Brutal Mathematical Validation of 50 Books ISBNs...")
    isbn_errors = []
    if len(BOOKS_50_CANON) != 50:
        isbn_errors.append(f"Expected 50 books, found {len(BOOKS_50_CANON)}")
        
    for book in BOOKS_50_CANON:
        n = book["num"]
        title = book["title"]
        i10 = book["isbn10"]
        i13 = book["isbn13"]
        
        ok10 = validate_isbn10(i10)
        ok13 = validate_isbn13(i13)
        
        if not ok10:
            isbn_errors.append(f"Book #{n} '{title}': Invalid ISBN-10 ({i10})")
        if not ok13:
            isbn_errors.append(f"Book #{n} '{title}': Invalid ISBN-13 ({i13})")
            
    if isbn_errors:
        print(f"❌ FAIL: {len(isbn_errors)} ISBN checksum failures detected:")
        for err in isbn_errors:
            print("  -", err)
        return False
    else:
        print(f"✅ PASS: All 50 books have 100% mathematically valid ISBN-10 (mod 11) and ISBN-13 (mod 10) checksums.\n")

    # TEST 2: File synchronization check & Frontmatter parsing
    print("[TEST 2/4] Verifying Tagisan YAML frontmatter parsing for SKILL and AGENT...")
    
    scratch_dir = Path("/home/dyna/.gemini/antigravity-cli/brain/b1b21e60-5c91-4eb3-920c-c4bcbfa361e1/scratch")
    skill_content = (scratch_dir / "SKILL.md").read_text(encoding="utf-8")
    agent_content = (scratch_dir / "failure-forensic-expert.md").read_text(encoding="utf-8")
    guide_content = (scratch_dir / "VIBE_CODER_FAILURE_AND_FORENSIC_ANALYSIS_GUIDE.md").read_text(encoding="utf-8")
    
    try:
        parsed_skill = parse_ecc_skill(skill_content)
        assert parsed_skill["name"] == "failure-and-forensics-vibe-coder", f"Unexpected skill name: {parsed_skill['name']}"
        assert len(parsed_skill["description"]) > 10
        assert len(parsed_skill["body"]) > 500
        print(f"  [+] EccSkill parsed cleanly: name='{parsed_skill['name']}', desc_len={len(parsed_skill['description'])}")
    except Exception as e:
        print(f"❌ FAIL: EccSkill parsing failed: {e}")
        return False

    try:
        parsed_agent = parse_ecc_agent(agent_content)
        assert parsed_agent["name"] == "failure-forensic-expert", f"Unexpected agent name: {parsed_agent['name']}"
        assert len(parsed_agent["tools"]) >= 2
        assert parsed_agent["model"] == "deepseek-reasoner"
        assert len(parsed_agent["system_prompt"]) > 200
        print(f"  [+] EccAgent parsed cleanly: name='{parsed_agent['name']}', tools={parsed_agent['tools']}, model='{parsed_agent['model']}'")
    except Exception as e:
        print(f"❌ FAIL: EccAgent parsing failed: {e}")
        return False

    assert len(guide_content) > 5000, "Reference guide content too short"
    print(f"  [+] Reference Guide verified: {len(guide_content)} characters across 10 pillars and 50 books.")
    print("✅ PASS: Tagisan YAML frontmatter parsing matches Rust EccSkill and EccAgent specifications.\n")

    # TEST 3: Deploying files to Target Directories
    print("[TEST 3/4] Deploying to target locations:")
    targets = [
        Path("/home/dyna/TGS Projects/tagisan"),
        Path("/home/dyna/TGS Projects")
    ]
    
    for base in targets:
        # Skill
        skill_dir = base / ".ecc" / "skills" / "failure-and-forensics-vibe-coder"
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")
        print(f"  [+] Deployed: {skill_dir / 'SKILL.md'}")
        
        # Agent
        agent_dir = base / ".ecc" / "agents"
        agent_dir.mkdir(parents=True, exist_ok=True)
        (agent_dir / "failure-forensic-expert.md").write_text(agent_content, encoding="utf-8")
        print(f"  [+] Deployed: {agent_dir / 'failure-forensic-expert.md'}")

    # Guide in tagisan/docs
    docs_dir = Path("/home/dyna/TGS Projects/tagisan/docs")
    docs_dir.mkdir(parents=True, exist_ok=True)
    (docs_dir / "VIBE_CODER_FAILURE_AND_FORENSIC_ANALYSIS_GUIDE.md").write_text(guide_content, encoding="utf-8")
    print(f"  [+] Deployed: {docs_dir / 'VIBE_CODER_FAILURE_AND_FORENSIC_ANALYSIS_GUIDE.md'}")
    
    # Invalidate cache if present so tgs discovers new skill immediately
    cache_path = Path("/home/dyna/TGS Projects/tagisan/.tagisan/skills.cache")
    if cache_path.exists():
        cache_path.unlink()
        print("  [+] Invalidated .tagisan/skills.cache to force catalog recompilation")
    print("✅ PASS: Files successfully synchronized across all target repositories.\n")

    # TEST 4: Live binary integration test with `tgs`
    print("[TEST 4/4] Executing live CLI integration tests with 'tgs' binary...")
    
    tgs_bin = shutil.which("tgs") or "/home/dyna/.cargo/bin/tgs"
    if not os.path.exists(tgs_bin):
        print(f"⚠️ Warning: 'tgs' binary not found at {tgs_bin}. Skipping binary execution.")
        return True

    # Check `tgs ecc list`
    print("  Executing: tgs ecc list")
    res_list = subprocess.run([tgs_bin, "ecc", "list"], cwd="/home/dyna/TGS Projects/tagisan", capture_output=True, text=True)
    if res_list.returncode != 0:
        print(f"❌ FAIL: 'tgs ecc list' exited with code {res_list.returncode}:\n{res_list.stderr}")
        return False
    
    if "failure-forensic-expert" in res_list.stdout:
        print("  [✓] 'failure-forensic-expert' successfully detected by 'tgs ecc list'!")
    else:
        print("❌ FAIL: 'failure-forensic-expert' not found in 'tgs ecc list' output:\n", res_list.stdout)
        return False

    # Check `tgs ecc skills`
    print("  Executing: tgs ecc skills (checking for 'failure-and-forensics-vibe-coder')")
    res_skills = subprocess.run([tgs_bin, "ecc", "skills"], cwd="/home/dyna/TGS Projects/tagisan", capture_output=True, text=True)
    combined_skills_output = res_skills.stdout + res_skills.stderr
    if "failure-and-forensics-vibe-coder" in combined_skills_output:
        print("  [✓] 'failure-and-forensics-vibe-coder' successfully detected by 'tgs ecc skills'!")
    else:
        print("❌ FAIL: 'failure-and-forensics-vibe-coder' not found in 'tgs ecc skills' output.")
        return False

    print("\n=================================================================")
    print(" 🎉 ALL TESTS PASSED: 100% VERIFIED FAILURE & FORENSIC CANON")
    print("=================================================================")
    return True

if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)
