---
name: "bun-shell"
description: "Cross-platform shell scripting with Bun's $ template literal. Use for build scripts, file manipulation, git automation, deployment tasks, and any shell commands. Keywords: bun shell, cross-platform, scripting, automation, $ template, build, deploy, git, file operations."
---
# Bun Shell Scripting

Cross-platform bash-like shell scripting in JavaScript/TypeScript using Bun's `$` template literal API.

**Documentation**: [bun.sh/docs/runtime/shell](https://bun.sh/docs/runtime/shell)

## Quick Start

```typescript
import { $ } from "bun";

// Basic command execution
await $`echo "Hello World!"`;

// Capture output as text
const output = await $`ls -la`.text();

// Parse JSON output
const pkg = await $`cat package.json`.json();

// Iterate over lines
for await (const line of $`git log --oneline -5`.lines()) {
  console.log(line);
}
```

## Core Syntax

### Template Literal Execution

```typescript
import { $ } from "bun";

// Variables are auto-escaped (prevents shell injection)
const filename = "foo.js; rm -rf /";
await $`ls ${filename}`; // Safe: runs "ls 'foo.js; rm -rf /'"

// Command chaining
await $`mkdir -p dist && npm run build`;

// Piping
const count = await $`cat file.txt | wc -l`.text();

// Redirection to file
await $`echo "content" > output.txt`;

// Input from JavaScript objects
const response = new Response("hello world");
await $`cat < ${response}`;
```

### Output Methods

| Method | Returns | Auto-Quiet |
|--------|---------|------------|
| `.text()` | `Promise<string>` | Yes |
| `.json()` | `Promise<T>` | Yes |
| `.lines()` | `AsyncIterable<string>` | Yes |
| `.blob()` | `Promise<Blob>` | Yes |
| `.bytes()` | `Promise<Uint8Array>` | Yes |
| `.arrayBuffer()` | `Promise<ArrayBuffer>` | Yes |

### Configuration Methods

```typescript
// Suppress output (buffer only)
const { stdout, stderr } = await $`echo hi`.quiet();

// Change working directory
await $`ls`.cwd("/tmp");

// Set environment variables
await $`echo $FOO`.env({ FOO: "bar" });

// Disable throwing on non-zero exit
const result = await $`exit 1`.nothrow();
console.log(result.exitCode); // 1
```

## Error Handling

```typescript
import { $ } from "bun";

// Default: throws on non-zero exit
try {
  await $`exit 1`;
} catch (err) {
  if (err instanceof $.ShellError) {
    console.log(err.exitCode);   // 1
    console.log(err.stdout);     // Buffer
    console.log(err.stderr);     // Buffer
  }
}

// Manual exit code handling
const { exitCode, stdout, stderr } = await $`command`.nothrow().quiet();
if (exitCode !== 0) {
  console.error("Command failed:", stderr.toString());
}
```

## Built-in Commands (Cross-Platform)

These work on Windows, macOS, and Linux without system dependencies:

| Command | Description |
|---------|-------------|
| `cd` | Change directory |
| `ls` | List files |
| `rm` | Remove files/dirs |
| `cat` | Display file contents |
| `echo` | Print text |
| `pwd` | Print working directory |
| `mkdir` | Create directory |
| `mv` | Move/rename (partial) |
| `touch` | Create empty file |
| `which` | Locate command |
| `exit` | Exit with code |
| `true` / `false` | Return 0/1 |
| `basename` / `dirname` | Path manipulation |
| `seq` | Generate sequences |
| `yes` | Repeat string |

## Environment Variables

```typescript
import { $ } from "bun";

// Inline environment variable
await $`FOO=bar bun -e 'console.log(process.env.FOO)'`;

// With interpolation
const value = "hello";
await $`MY_VAR=${value} some-command`;

// Global default
$.env({ NODE_ENV: "production" });

// Per-command override
await $`npm run build`.env({ DEBUG: "true" });

// Access current process env
await $`echo $HOME`; // Uses process.env by default
```

## Glob Patterns

```typescript
import { $ } from "bun";

// Wildcards
await $`ls *.js`;
await $`rm -rf **/*.log`;

// Brace expansion
await $`echo {a,b,c}.txt`;  // a.txt b.txt c.txt

// Combined patterns
await $`ls src/**/*.{ts,tsx}`;
```

## Raw Strings (Disable Escaping)

```typescript
import { $ } from "bun";

// Bypass auto-escaping (use with caution!)
await $`echo ${{ raw: '$(date)' }}`;

// For command substitution
await $`echo ${{ raw: '$(git rev-parse HEAD)' }}`;
```

## Redirection

```typescript
import { $ } from "bun";
import { file } from "bun";

// Output to file
await $`ls > files.txt`;
await $`ls > ${file("output.txt")}`;

// Append to file
await $`echo "line" >> log.txt`;

// Output to buffer
const buffer = Buffer.alloc(1024);
await $`ls > ${buffer}`;

// Redirect stderr to stdout
await $`command 2>&1`;

// Input from various sources
await $`cat < ${new Response("data")}`;
await $`cat < ${file("input.txt")}`;
await $`cat < ${new Uint8Array([72, 105])}`;
```

## Global Configuration

```typescript
import { $ } from "bun";

// Set global defaults
$.cwd("/project");
$.env({ NODE_ENV: "development" });
$.throws(false);  // or $.nothrow()

// Per-command overrides still work
await $`npm test`.cwd("/other").env({ CI: "true" }).throws(true);
```

## Comparison with Alternatives

| Feature | Bun Shell | zx | execa |
|---------|-----------|----|----- |
| **Runtime** | Bun only | Node.js, Deno | Node.js |
| **Approach** | New shell impl | Wrapper around system shell | Pure JS, no shell |
| **Performance** | 20x faster than zx | Baseline | Similar to zx |
| **Cross-platform** | Yes (built-ins) | Requires extra packages | Yes |
| **Dependencies** | 0 (built-in) | Several | 12+ |
| **Security** | Auto-escapes | Auto-escapes | Manual escaping |
| **Streaming** | Limited | Full | Full |

## Limitations

1. **No real-time streaming**: `.lines()` waits for command completion
2. **Context isolation**: Each `await $` runs in fresh context (cd doesn't persist)
3. **mv partial**: Missing cross-device support
4. **Windows**: 98% test parity (edge cases may differ)

## Best Practices

1. **Always await**: `$` returns a promise, not immediate execution
2. **Use .nothrow() for optional commands**: Prevents script crashes
3. **Prefer built-ins**: `rm`, `ls`, etc. are faster than PATH commands
4. **Use .cwd() instead of cd**: Maintains context for the command
5. **Escape is default**: Trust the auto-escaping, use raw only when needed
