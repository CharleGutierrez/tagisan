# Bun Shell Scripting

Bun's `$` shell provides cross-platform shell scripting that works on Windows, macOS, and Linux.

## Basic Syntax

```typescript
import { $ } from "bun";

// Simple command
await $`echo "Hello World"`;

// Store output
const result = await $`ls -la`.text();
console.log(result);

// JSON output
const data = await $`cat package.json`.json();
```

## Variable Interpolation

```typescript
const filename = "test.txt";
const dir = "./output";

// Variables are automatically escaped
await $`cat ${filename}`;
await $`mkdir -p ${dir}`;

// Arrays expand to multiple arguments
const files = ["a.txt", "b.txt", "c.txt"];
await $`rm ${files}`;  // rm a.txt b.txt c.txt
```

## Piping and Chaining

```typescript
// Pipe output
const result = await $`cat file.txt | grep pattern | wc -l`.text();

// Command chaining
await $`cd project && npm install && npm test`;

// Conditional execution
await $`test -f config.json && cat config.json`;
```

## Output Modes

```typescript
// Get as string (trimmed)
const text = await $`echo "hello"`.text();

// Get as JSON
const json = await $`cat data.json`.json();

// Get as bytes
const bytes = await $`cat binary.bin`.bytes();

// Get as lines array
const lines = await $`cat file.txt`.lines();

// Get as Blob
const blob = await $`cat image.png`.blob();
```

## Quiet Mode (Suppress Output)

```typescript
// Normal: prints to stdout
await $`echo "visible"`;

// Quiet: suppresses output, still captures
const { stdout, stderr, exitCode } = await $`echo "hidden"`.quiet();
```

## Error Handling

```typescript
import { $ } from "bun";

// By default, throws on non-zero exit
try {
  await $`exit 1`;
} catch (err) {
  console.error("Command failed:", err.exitCode);
}

// Get exit code without throwing
const proc = await $`exit 1`.quiet().nothrow();
console.log(proc.exitCode);  // 1

// Check exit code
if (proc.exitCode !== 0) {
  console.error(proc.stderr.toString());
}
```

## Environment Variables

```typescript
// Set for single command
await $`MY_VAR=value printenv MY_VAR`;

// Set via env option
await $`printenv MY_VAR`.env({ MY_VAR: "value" });

// Access current env
const path = await $`echo $PATH`.text();
```

## Working Directory

```typescript
// Change directory for command
await $`ls`.cwd("/tmp");

// Chain with cd
await $`cd /tmp && ls`;
```

## Built-in Commands

Bun Shell includes cross-platform implementations of common commands:

| Command | Description |
|---------|-------------|
| `cd` | Change directory |
| `ls` | List directory |
| `cat` | Read files |
| `echo` | Print output |
| `rm` | Remove files |
| `mkdir` | Create directories |
| `mv` | Move/rename files |
| `cp` | Copy files |
| `pwd` | Print working directory |
| `which` | Find command path |
| `exit` | Exit with code |

## Advanced Patterns

### Parallel Execution
```typescript
// Run commands in parallel
await Promise.all([
  $`npm run build:frontend`,
  $`npm run build:backend`,
  $`npm run build:styles`,
]);
```

### Streaming Output
```typescript
// Stream stdout
for await (const chunk of $`long-running-command`.stdout) {
  process.stdout.write(chunk);
}
```

### Script Files
```typescript
#!/usr/bin/env bun
// build.ts
import { $ } from "bun";

console.log("Building project...");
await $`tsc`;
await $`bun build src/index.ts --outdir dist`;
console.log("Done!");
```

Make executable:
```bash
chmod +x build.ts
./build.ts
```

## Comparison with Alternatives

| Feature | Bun Shell | zx | execa |
|---------|-----------|-----|-------|
| Cross-platform | Yes | Partial | Partial |
| Built-in commands | Yes | No | No |
| Template literals | Yes | Yes | No |
| TypeScript | Native | Via ts-node | Via ts-node |
| Performance | Fastest | Slower | Slower |

## Real-World Examples

### Git Automation
```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const branch = await $`git branch --show-current`.text();
const status = await $`git status --porcelain`.text();

if (status.trim()) {
  await $`git add .`;
  await $`git commit -m "Auto-commit from script"`;
  await $`git push origin ${branch}`;
}
```

### Build Script
```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Clean
await $`rm -rf dist`;

// Build
await $`bun build src/index.ts --outdir dist --minify`;

// Copy assets
await $`cp -r public/* dist/`;

// Report
const size = await $`du -sh dist`.text();
console.log(`Build complete: ${size}`);
```

### Deployment Script
```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const env = process.env.NODE_ENV || "development";

await $`docker build -t myapp:latest .`;
await $`docker push myapp:latest`;

if (env === "production") {
  await $`kubectl rollout restart deployment/myapp`;
}
```
