#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Compile updated TGS MS Power Platform MasterClass HTML into PDF via Playwright
"""

import os
import sys
import time
from pathlib import Path
from playwright.sync_api import sync_playwright

def main():
    repo_root = Path(__file__).resolve().parent.parent
    docs_dir = repo_root / "docs"
    html_path = docs_dir / "TGS_MS_POWER_PLATFORM_MASTERCLASS_COURSE.html"
    pdf_path = docs_dir / "TGS_MS_POWER_PLATFORM_MASTERCLASS_COURSE.pdf"

    if not html_path.exists():
        print(f"Error: {html_path} not found")
        sys.exit(1)

    print(f"Reading HTML: {html_path} ({html_path.stat().st_size:,} bytes)...")
    start = time.time()
    
    file_url = html_path.as_uri()

    with sync_playwright() as p:
        print("Launching Chromium...")
        browser = p.chromium.launch()
        page = browser.new_page()
        
        print(f"Navigating to: {file_url}...")
        page.goto(file_url, wait_until="load", timeout=120000)
        
        print("Generating PDF...")
        page.pdf(
            path=str(pdf_path),
            format="A4",
            print_background=True,
            prefer_css_page_size=True,
            margin={"top": "18mm", "right": "14mm", "bottom": "18mm", "left": "14mm"}
        )
        browser.close()

    elapsed = time.time() - start
    print(f"SUCCESS: PDF compiled in {elapsed:.1f}s -> {pdf_path} ({pdf_path.stat().st_size:,} bytes)")

if __name__ == "__main__":
    main()
