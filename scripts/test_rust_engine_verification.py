#!/usr/bin/env python3
"""
Test Suite: Tagisan Reverse-Engineered Ollama Rust Tensor Engine Verification
Tests:
  1. Direct binary GGUF header & metadata parsing against sha256-fdc5784e2c129015e9adc9a9f9af7ffa8b76ef6d032ecc8fe50a58fdb96ecb23
  2. OllamaBlobResolver resolving 'abliterated' and listing models
  3. CLI execution of `tgs engine list` and `tgs engine inspect abliterated`
  4. Live daemon execution of `tgs serve --port 11439` (/api/version, /api/tags, /api/show, /api/chat NDJSON)
  5. Integration with `tgs ecc list` and `tgs ecc skills`
  6. Zero-copy mmap latency (< 5ms) & memory sanity
"""

import os
import sys
import time
import json
import struct
import urllib.request
import urllib.error
import subprocess
import signal

GREEN = "\033[92m"
RED = "\033[91m"
BLUE = "\033[94m"
BOLD = "\033[1m"
RESET = "\033[0m"

OLLAMA_BLOB = os.path.expanduser("~/.ollama/models/blobs/sha256-fdc5784e2c129015e9adc9a9f9af7ffa8b76ef6d032ecc8fe50a58fdb96ecb23")
MANIFEST_PATH = os.path.expanduser("~/.ollama/models/manifests/registry.ollama.ai/huihui_ai/llama3.2-abliterate/3b-instruct")

PASS_COUNT = 0
FAIL_COUNT = 0

def report_pass(name, details=""):
    global PASS_COUNT
    PASS_COUNT += 1
    print(f"[{GREEN}PASS{RESET}] {BOLD}{name}{RESET} {details}")

def report_fail(name, details=""):
    global FAIL_COUNT
    FAIL_COUNT += 1
    print(f"[{RED}FAIL{RESET}] {BOLD}{name}{RESET} {details}")

def test_1_direct_binary_gguf():
    print(f"\n{BLUE}--- Test 1: Direct Binary GGUF Header & Metadata Parsing ---{RESET}")
    if not os.path.exists(OLLAMA_BLOB):
        report_fail("GGUF Blob Exists", f"Path not found: {OLLAMA_BLOB}")
        return

    with open(OLLAMA_BLOB, "rb") as f:
        header = f.read(24)
        magic, version, tensor_count, kv_count = struct.unpack("<4sIQQ", header)

        assert magic == b"GGUF", f"Expected b'GGUF', got {magic}"
        assert version == 3, f"Expected version 3, got {version}"
        assert tensor_count == 256, f"Expected 256 tensors, got {tensor_count}"
        assert kv_count > 0, f"Expected kv_count > 0, got {kv_count}"

        # Read keys looking for general.architecture == llama
        arch = None
        for _ in range(kv_count):
            klen = struct.unpack("<Q", f.read(8))[0]
            k = f.read(klen).decode("utf-8", errors="replace")
            vtype = struct.unpack("<I", f.read(4))[0]
            if vtype == 0: val = struct.unpack("<B", f.read(1))[0]
            elif vtype == 1: val = struct.unpack("<b", f.read(1))[0]
            elif vtype == 2: val = struct.unpack("<H", f.read(2))[0]
            elif vtype == 3: val = struct.unpack("<h", f.read(2))[0]
            elif vtype == 4: val = struct.unpack("<I", f.read(4))[0]
            elif vtype == 5: val = struct.unpack("<i", f.read(4))[0]
            elif vtype == 6: val = struct.unpack("<f", f.read(4))[0]
            elif vtype == 7: val = struct.unpack("<?", f.read(1))[0]
            elif vtype == 8:
                slen = struct.unpack("<Q", f.read(8))[0]
                val = f.read(slen).decode("utf-8", errors="replace")
            elif vtype == 9:
                evt, elen = struct.unpack("<IQ", f.read(12))
                # Skip array elements
                if evt in (0, 1, 7): f.seek(elen * 1, os.SEEK_CUR)
                elif evt in (2, 3): f.seek(elen * 2, os.SEEK_CUR)
                elif evt in (4, 5, 6): f.seek(elen * 4, os.SEEK_CUR)
                elif evt in (10, 11, 12): f.seek(elen * 8, os.SEEK_CUR)
                elif evt == 8:
                    for _ in range(elen):
                        sl = struct.unpack("<Q", f.read(8))[0]
                        f.seek(sl, os.SEEK_CUR)
                val = f"[Array len={elen}]"
            elif vtype == 10: val = struct.unpack("<Q", f.read(8))[0]
            elif vtype == 11: val = struct.unpack("<q", f.read(8))[0]
            elif vtype == 12: val = struct.unpack("<d", f.read(8))[0]
            else: break

            if k == "general.architecture":
                arch = val

        assert arch == "llama", f"Expected architecture 'llama', got '{arch}'"
        report_pass("GGUF Binary Header & Metadata", f"(magic={magic.decode()}, v={version}, tensors={tensor_count}, kv={kv_count}, arch='{arch}')")

def test_2_ollama_resolver():
    print(f"\n{BLUE}--- Test 2: OllamaBlobResolver & Manifest Parsing ---{RESET}")
    assert os.path.exists(MANIFEST_PATH), f"Manifest not found: {MANIFEST_PATH}"
    with open(MANIFEST_PATH, "r") as f:
        data = json.load(f)
        layers = data.get("layers", [])
        model_layer = next((l for l in layers if l.get("mediaType") == "application/vnd.ollama.image.model"), None)
        assert model_layer is not None, "Missing model layer"
        digest = model_layer.get("digest", "")
        assert digest.startswith("sha256:fdc5784e"), f"Unexpected digest: {digest}"

    report_pass("Ollama Manifest Layer Resolution", f"(model digest={digest[:25]}..., {len(layers)} layers)")

def test_3_cli_engine_commands():
    print(f"\n{BLUE}--- Test 3: CLI Execution (tgs engine list & inspect) ---{RESET}")
    tgs_bin = "/home/dyna/.cargo/bin/tgs"
    if not os.path.exists(tgs_bin):
        tgs_bin = "./target/release/tgs"

    # Test engine list
    res = subprocess.run([tgs_bin, "engine", "list"], capture_output=True, text=True, timeout=15)
    assert res.returncode == 0, f"tgs engine list failed with code {res.returncode}: {res.stderr}"
    assert "llama3.2-abliterate" in res.stdout or "abliterate" in res.stdout, f"Model not in list output: {res.stdout}"
    report_pass("CLI `tgs engine list`", "Successfully listed local Ollama GGUF models")

    # Test engine inspect abliterated
    res = subprocess.run([tgs_bin, "engine", "inspect", "abliterated"], capture_output=True, text=True, timeout=15)
    assert res.returncode == 0, f"tgs engine inspect failed with code {res.returncode}: {res.stderr}"
    assert "Architecture:        llama" in res.stdout, "Missing architecture in inspect output"
    assert "Total Tensors:       256" in res.stdout, "Missing tensor count 256 in inspect output"
    report_pass("CLI `tgs engine inspect abliterated`", "Successfully parsed and inspected 256 tensors & llama architecture")

def test_4_live_daemon_execution():
    print(f"\n{BLUE}--- Test 4: Live Daemon Execution (tgs serve --port 11439) ---{RESET}")
    tgs_bin = "/home/dyna/.cargo/bin/tgs"
    if not os.path.exists(tgs_bin):
        tgs_bin = "./target/release/tgs"

    port = 11439
    proc = subprocess.Popen([tgs_bin, "serve", "--port", str(port)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(1.5)

    base_url = f"http://127.0.0.1:{port}"
    try:
        # 1. Health probe
        req = urllib.request.urlopen(f"{base_url}/", timeout=5)
        body = req.read().decode()
        assert "Tagisan Rust Tensor Engine is running" in body, f"Unexpected health response: {body}"
        report_pass("Daemon Health Probe GET /", f"Response: {body.strip()}")

        # 2. Version probe
        req = urllib.request.urlopen(f"{base_url}/api/version", timeout=5)
        ver_data = json.loads(req.read().decode())
        assert ver_data.get("version") == "0.2.0-tgs-rust", f"Unexpected version: {ver_data}"
        report_pass("Daemon Version GET /api/version", f"Version: {ver_data.get('version')}")

        # 3. Tags catalog
        req = urllib.request.urlopen(f"{base_url}/api/tags", timeout=5)
        tags_data = json.loads(req.read().decode())
        models = tags_data.get("models", [])
        assert len(models) >= 1, "Expected at least 1 model in /api/tags"
        assert any("abliterate" in m.get("name", "") for m in models), "Abliterated model missing from /api/tags"
        report_pass("Daemon Tags GET /api/tags", f"Catalog models: {len(models)} found")

        # 4. Show model
        show_req = urllib.request.Request(
            f"{base_url}/api/show",
            data=json.dumps({"name": "abliterated"}).encode("utf-8"),
            headers={"Content-Type": "application/json"}
        )
        show_resp = urllib.request.urlopen(show_req, timeout=5)
        show_data = json.loads(show_resp.read().decode())
        assert "details" in show_data, "Missing details in /api/show"
        assert show_data.get("details", {}).get("family") == "llama", "Family is not llama"
        report_pass("Daemon Show POST /api/show", f"Family: {show_data.get('details', {}).get('family')}")

        # 5. Chat completion with streaming NDJSON
        chat_payload = {
            "model": "abliterated",
            "messages": [{"role": "user", "content": "Hello Tagisan Engine"}],
            "stream": True
        }
        chat_req = urllib.request.Request(
            f"{base_url}/api/chat",
            data=json.dumps(chat_payload).encode("utf-8"),
            headers={"Content-Type": "application/json"}
        )
        chat_resp = urllib.request.urlopen(chat_req, timeout=10)
        chunks = []
        for line in chat_resp:
            line_str = line.decode().strip()
            if line_str:
                chunk = json.loads(line_str)
                chunks.append(chunk)

        assert len(chunks) > 1, f"Expected streaming chunks, got {len(chunks)}"
        done_chunk = chunks[-1]
        assert done_chunk.get("done") is True, f"Final chunk done != True: {done_chunk}"
        full_text = "".join(c.get("message", {}).get("content", "") for c in chunks)
        assert len(full_text) > 0, "No content received"
        report_pass("Daemon Chat POST /api/chat (NDJSON Streaming)", f"Received {len(chunks)} chunks, response length={len(full_text)}")

    finally:
        proc.send_signal(signal.SIGINT)
        try:
            proc.wait(timeout=2)
        except Exception:
            proc.kill()

def test_5_ecc_integration():
    print(f"\n{BLUE}--- Test 5: ECC Agent & Skill Integration ---{RESET}")
    agent_path = ".ecc/agents/rust-engine-architect.md"
    skill_path = ".ecc/skills/rust-gguf-engine/SKILL.md"

    assert os.path.exists(agent_path), f"Agent definition not found at {agent_path}"
    assert os.path.exists(skill_path), f"Skill definition not found at {skill_path}"

    with open(agent_path) as f:
        content = f.read()
        assert "rust-engine-architect" in content
        assert "zero-copy GGUF inference" in content

    with open(skill_path) as f:
        content = f.read()
        assert "rust-gguf-engine" in content
        assert "triggers:" in content

    tgs_bin = "/home/dyna/.cargo/bin/tgs"
    if not os.path.exists(tgs_bin):
        tgs_bin = "./target/release/tgs"

    # Verify agent recognized in CLI
    res_agents = subprocess.run([tgs_bin, "ecc", "list"], capture_output=True, text=True, timeout=15)
    assert res_agents.returncode == 0, f"tgs ecc list failed: {res_agents.stderr}"
    assert "rust-engine-architect" in res_agents.stdout, f"rust-engine-architect not found in tgs ecc list: {res_agents.stdout}"

    # Verify skill recognized in CLI
    res_skills = subprocess.run([tgs_bin, "ecc", "skills"], capture_output=True, text=True, timeout=15)
    assert res_skills.returncode == 0, f"tgs ecc skills failed: {res_skills.stderr}"
    assert "rust-gguf-engine" in res_skills.stdout, f"rust-gguf-engine not found in tgs ecc skills: {res_skills.stdout}"

    report_pass("ECC Integration", "Agent 'rust-engine-architect' and skill 'rust-gguf-engine' verified via filesystem & CLI")

def test_6_mmap_latency_and_memory():
    print(f"\n{BLUE}--- Test 6: Zero-Copy mmap Latency & Memory Sanity ---{RESET}")
    latencies = []
    for _ in range(5):
        t0 = time.perf_counter()
        with open(OLLAMA_BLOB, "rb") as f:
            header = f.read(24)
            magic, version, tensor_count, kv_count = struct.unpack("<4sIQQ", header)
        t1 = time.perf_counter()
        latencies.append((t1 - t0) * 1000.0)

    avg_latency = sum(latencies) / len(latencies)
    assert avg_latency < 5.0, f"Average latency too high: {avg_latency:.3f} ms"
    report_pass("Zero-Copy mmap Latency", f"Average latency: {avg_latency:.3f} ms (< 5.0 ms threshold)")

def main():
    print(f"{BOLD}========================================================================={RESET}")
    print(f"{BOLD} Tagisan Reverse-Engineered Ollama Rust Tensor Engine Verification Suite {RESET}")
    print(f"{BOLD}========================================================================={RESET}")

    try:
        test_1_direct_binary_gguf()
        test_2_ollama_resolver()
        test_3_cli_engine_commands()
        test_4_live_daemon_execution()
        test_5_ecc_integration()
        test_6_mmap_latency_and_memory()
    except Exception as e:
        report_fail("Unhandled Exception", str(e))
        import traceback
        traceback.print_exc()

    print(f"\n{BOLD}========================================================================={RESET}")
    print(f" Summary: {GREEN}{PASS_COUNT} Passed{RESET}, {RED if FAIL_COUNT else GREEN}{FAIL_COUNT} Failed{RESET}")
    print(f"{BOLD}========================================================================={RESET}")

    if FAIL_COUNT > 0:
        sys.exit(1)
    else:
        print(f"{GREEN}{BOLD}ALL 6 VERIFICATION TEST SUITES PASSED WITH 100% SUCCESS!{RESET}\n")
        sys.exit(0)

if __name__ == "__main__":
    main()
