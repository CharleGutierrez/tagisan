---
name: "bun-toolkit"
description: "Execute JavaScript/TypeScript with Bun runtime. Use when user needs fast JS execution, package management, or cross-platform shell scripting. Keywords: bun, javascript, typescript, runtime, fast, npm alternative, package manager, shell script."
---
# Bun Toolkit

Bun is an all-in-one JavaScript runtime and toolkit designed for speed. This skill enables fast script execution, package management, and cross-platform shell scripting.

## Variables

- **BUN_INSTALL_SCRIPT_WINDOWS**: `powershell -c "irm bun.sh/install.ps1|iex"`
- **BUN_INSTALL_SCRIPT_UNIX**: `curl -fsSL https://bun.sh/install | bash`
- **BUN_PATH_WINDOWS**: `$env:USERPROFILE\.bun\bin`
- **BUN_PATH_UNIX**: `~/.bun/bin`

## Prerequisites

### Check Installation

```bash
# Check if Bun is installed
bun --version
```

### Install Bun

**Windows (PowerShell):**
```powershell
powershell -c "irm bun.sh/install.ps1|iex"
# Add to PATH if needed
$env:Path += ";$env:USERPROFILE\.bun\bin"
```

**macOS/Linux:**
```bash
curl -fsSL https://bun.sh/install | bash
source ~/.bashrc  # or ~/.zshrc
```

**Via npm (cross-platform):**
```bash
npm install -g bun
```

## Core Capabilities

### 1. Run Scripts (Fastest JS Runtime)
```bash
# Run TypeScript directly (no config needed)
bun run script.ts

# Run with hot reloading
bun --hot run server.ts

# Run with watch mode
bun --watch run script.ts
```

### 2. Package Management (25x faster than npm)
```bash
# Install dependencies
bun install

# Add package
bun add package-name
bun add -d dev-package  # dev dependency
bun add -g global-pkg   # global

# Remove package
bun remove package-name

# Update packages
bun update
```

### 3. Shell Scripting (Cross-platform)
```typescript
// Cross-platform shell with Bun.$
import { $ } from "bun";

// Template literal syntax
await $`echo "Hello from Bun shell!"`;

// Piping
const result = await $`cat file.txt | grep pattern`.text();

// Variable interpolation
const dir = "./src";
await $`ls -la ${dir}`;

// Error handling
const { exitCode, stdout, stderr } = await $`command`.quiet();
```

### 4. HTTP Server
```typescript
Bun.serve({
  port: 3000,
  fetch(req) {
    return new Response("Hello from Bun!");
  },
});
```

### 5. File I/O
```typescript
// Read file
const file = Bun.file("./data.json");
const content = await file.text();
const json = await file.json();

// Write file
await Bun.write("output.txt", "Hello World");
await Bun.write("data.json", JSON.stringify({ key: "value" }));
```

### 6. SQLite (Built-in)
```typescript
import { Database } from "bun:sqlite";

const db = new Database("mydb.sqlite");
db.run("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)");
db.run("INSERT INTO users (name) VALUES (?)", ["Alice"]);
const users = db.query("SELECT * FROM users").all();
```

## Quick Commands Reference

| Task | Command |
|------|---------|
| Run script | `bun run script.ts` |
| Install deps | `bun install` |
| Add package | `bun add <pkg>` |
| Dev server | `bun --hot run server.ts` |
| Run tests | `bun test` |
| Bundle | `bun build ./src/index.ts --outdir ./dist` |
| Create executable | `bun build --compile ./app.ts --outfile app` |
| Init project | `bun init` |
| Create from template | `bun create react-app my-app` |

## Cookbook

Extended documentation for specific features.

| Feature | When to Read | Documentation |
|---------|--------------|---------------|
| Installation | Setting up Bun on any platform | [cookbook/installation.md](cookbook/installation.md) |
| Runtime APIs | Using Bun-specific APIs | [cookbook/runtime-apis.md](cookbook/runtime-apis.md) |
| Package Management | Advanced package operations | [cookbook/package-management.md](cookbook/package-management.md) |
| Shell Scripting | Cross-platform automation | [cookbook/shell-scripting.md](cookbook/shell-scripting.md) |

## Examples

### Example 1: Run TypeScript Directly
```bash
# No tsconfig or compilation needed
echo 'console.log("Hello, TypeScript!")' > hello.ts
bun run hello.ts
```

### Example 2: Quick HTTP Server
```bash
echo 'Bun.serve({ fetch: () => new Response("Hello!"), port: 3000 })' > server.ts
bun run server.ts
# Server running at http://localhost:3000
```

### Example 3: Process JSON Files
```typescript
// process.ts
const data = await Bun.file("./input.json").json();
data.processed = true;
data.timestamp = new Date().toISOString();
await Bun.write("./output.json", JSON.stringify(data, null, 2));
console.log("Processed!");
```

## Workflow

### Starting a New Project

1. **Initialize project:**
   ```bash
   mkdir my-project && cd my-project
   bun init
   ```

2. **Add dependencies:**
   ```bash
   bun add express zod
   bun add -d typescript @types/node
   ```

3. **Create entry point:**
   ```typescript
   // index.ts
   console.log("Hello, Bun!");
   ```

4. **Run:**
   ```bash
   bun run index.ts
   ```

### Converting npm Project to Bun

1. **Delete node_modules and lockfile:**
   ```bash
   rm -rf node_modules package-lock.json yarn.lock pnpm-lock.yaml
   ```

2. **Install with Bun:**
   ```bash
   bun install
   ```

3. **Run scripts:**
   ```bash
   bun run dev
   bun run build
   ```

## Node.js Compatibility

Bun is designed for Node.js compatibility:

- **Fully Supported:** fs, path, crypto, http, https, net, url, stream, buffer, events
- **Mostly Supported:** child_process, worker_threads, cluster
- **Not Supported:** vm (use Workers), domain (deprecated)

## Troubleshooting

**"bun: command not found"**
- Windows: Add `$env:USERPROFILE\.bun\bin` to PATH
- Unix: Run `source ~/.bashrc` or restart terminal

**Package installation fails:**
```bash
# Clear cache and reinstall
bun pm cache rm
rm -rf node_modules bun.lockb
bun install
```

**TypeScript errors:**
- Bun uses its own TS transpiler - no `tsc` needed
- For type-checking: `bun x tsc --noEmit`

**Hot reload not working:**
- Use `bun --hot run` not `bun run --hot`
- Ensure no syntax errors in code
