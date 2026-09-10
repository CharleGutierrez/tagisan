---
name: bun-native-apis
description: High-performance Bun native APIs, bun:sqlite vector storage, Bun.serve, and zero-transpile TypeScript
---

# Bun Native Performance & APIs

## Best Practices
1. Fast I/O: Prefer `Bun.file(path)` and `Bun.write(path, content)` over legacy `node:fs` streams.
2. Built-in SQLite: Leverage `import { Database } from "bun:sqlite"` with prepared statements for zero-overhead persistence.
3. High-Throughput HTTP: Use `Bun.serve({ fetch(req) { ... } })` with native WebSocket pub/sub for sub-millisecond latency.
4. TypeScript Runtime: Rely on Bun's built-in transpiler without separate `tsc` build steps during development.
