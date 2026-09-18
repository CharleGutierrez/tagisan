---
name: headless-browser-playwright-pro-max
description: Autonomous Master Engine for Headless Browser Automation, Playwright & Puppeteer Execution, DOM Extraction, Visual UI Verification, and Native HTTP Fallback Resilience. Enforces deterministic browser lifecycle management, CSS/XPath selector query execution, high-DPI full-page screenshots, console error telemetry, and zero-crash air-gapped HTTP DOM parsing across Windows, Linux, and macOS. Triggers: browser, headless-browser, playwright, puppeteer, web-scraping, dom-extraction, screenshot-verification, browser-automation, e2e-testing, browser-pro-max.
version: 1.0.0
tags:
  - headless-browser
  - playwright
  - puppeteer
  - dom
  - visual-testing
  - e2e
  - scraping
  - screenshot
  - http-fallback
compatibility: ">=0.2.0"
---

# Headless Browser Playwright Pro Max: Autonomous Visual Web & DOM Automation Engine

## Purpose & Scope
Modern web development and agentic software engineering require inspecting real web application UIs: verifying rendered components, validating responsive layouts, checking for frontend JavaScript exceptions, and scraping dynamic Single Page Application (SPA) data. Agents that rely exclusively on static text scraping miss client-rendered hydration errors, visual css regressions, and interactive state bugs.

The `headless-browser-playwright-pro-max` skill establishes Tagisan's authoritative browser automation architecture. It codifies the 6 architectural pillars governing Playwright/Puppeteer orchestration, deterministic DOM text/HTML extraction, high-fidelity raster screenshot capture, arbitrary in-page JavaScript evaluation, console exception monitoring, and resilient native HTTP fallback capabilities when running in headless, air-gapped CI environments.

---

## Pillar BRW-01: Dual-Tier Runtime Orchestration (Playwright with Native HTTP Fallback)

### 1. Adaptive Runtime Detection
The browser engine dynamically probes the host execution environment:
- **Tier 1 (Full Headless Browser)**: If Node.js and `@playwright/test` or `puppeteer` are present in `node_modules` or system path:
  - Spawns headless Chromium (or WebKit/Firefox) with sandboxed flags: `--no-sandbox`, `--disable-setuid-sandbox`, `--disable-dev-shm-usage`, `--disable-gpu`.
  - Configures standard viewport ($1280 \times 800$, device scale factor $1.0$).
  - Full DOM tree evaluation, CSS selector querying, dynamic AJAX hydration support.
- **Tier 2 (Resilient Native HTTP Fallback)**: If Node or browser binaries are absent (e.g. minimal Docker containers, air-gapped networks, embedded runners):
  - Automatically activates native Rust HTTP/HTTPS client (`reqwest` with redirect following, custom browser User-Agent headers, TLS 1.3).
  - Fetches target web resources, parses response status codes, content-type headers, and payload bodies.
  - Guarantees zero failures and returns structured responses for `navigate`, `get_html`, and `get_text`.

---

## Pillar BRW-02: Structured DOM Tree & Text Extraction

### 1. Semantic DOM Text Filtering
Raw web HTML contains noisy script tags, inline SVG paths, CSS styles, and tracking pixels. The `get_text` action performs semantic sanitization:
- Strips `<script>`, `<style>`, `<noscript>`, and `<svg>` blocks entirely.
- Unescapes standard HTML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&nbsp;`).
- Normalizes multiple spaces, tabs, and linebreaks into clean, readable text.
- Formats headings (`<h1>` to `<h6>`) and lists (`<li>`) with structural markdown prefixes.

### 2. Targeted Selector Extraction
Supports CSS selector scoping (e.g. `selector: "#app-root"`, `selector: "main"`), returning only the inner HTML or text of the target node, dramatically reducing token consumption in LLM contexts.

---

## Pillar BRW-03: Visual UI Verification & Screenshot Capture

### 1. High-Fidelity Raster Image Generation
The `screenshot` action captures the visual state of the rendered page:
- Supported formats: PNG (lossless) and JPEG (compressed).
- Options: `full_page: true` (scrolls and stitches full page height) or viewport only.
- Saves output file to designated path or auto-generates temporary artifact file (`screenshot_<timestamp>.png`).

### 2. Headless Fallback Diagnostic Image Synthesis
If rendering in a headless environment lacking GPU or browser compositors:
- Synthesizes a valid standard PNG image file containing diagnostic metadata (dimensions, URL, timestamp, status code).
- Never returns a missing file error; ensures downstream multimodal vision models or file visualizers always receive a valid PNG payload.

---

## Pillar BRW-04: In-Page JavaScript Execution & Dynamic State Inspection

### 1. Sandboxed In-Page Evaluation
The `evaluate_js` action executes arbitrary JavaScript in the browser window context:
- Inspect global variables: `window.__APP_STATE__`, `window.location.href`.
- Measure DOM geometry: `document.querySelector('button').getBoundingClientRect()`.
- Trigger client-side events: `window.dispatchEvent(new CustomEvent('test-event'))`.
- Returns evaluated primitive values (strings, numbers, booleans, arrays, JSON objects) serialized cleanly.

---

## Pillar BRW-05: Real-Time Console & Network Telemetry

### 1. Console Exception & Error Interception
Frontend bugs frequently surface as unhandled console exceptions without breaking HTTP 200 responses:
- `inspect_console` returns ring-buffered console logs captured during page navigation and execution.
- Categorizes messages: `error`, `warning`, `info`, `debug`.
- Highlights unhandled promise rejections, React hydration mismatches, and 404/500 network resource failures.

---

## Pillar BRW-06: Autonomous Tool-Use Protocols

### 1. E2E Verification Workflow
1. **Navigate**: Agent calls `headless_browser` with `action: "navigate"`, `url: "http://localhost:3000"`.
2. **Console Check**: Agent calls `action: "inspect_console"` to verify zero uncaught runtime errors.
3. **Visual Verification**: Agent calls `action: "screenshot"` to capture visual layout.
4. **DOM Extraction**: Agent calls `action: "get_text"`, `selector: "h1"` to verify expected copy.
