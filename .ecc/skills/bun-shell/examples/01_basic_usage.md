# Basic Bun Shell Usage

## Example 1: Simple Commands

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Print to stdout
await $`echo "Hello from Bun Shell!"`;

// Capture as text
const version = await $`bun --version`.text();
console.log(`Bun version: ${version.trim()}`);

// List files
await $`ls -la`;
```

## Example 2: Variable Interpolation

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const name = "World";
const count = 5;

// Safe string interpolation (auto-escaped)
await $`echo "Hello, ${name}!"`;
await $`echo "Count: ${count}"`;

// Works with expressions
await $`echo "Sum: ${2 + 2}"`;
await $`echo "Date: ${new Date().toISOString()}"`;
```

## Example 3: Piping Commands

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Pipe output through multiple commands
const wordCount = await $`echo "one two three four five" | wc -w`.text();
console.log(`Words: ${wordCount.trim()}`);

// Multi-stage pipe
const sortedUnique = await $`echo "c\na\nb\na\nc" | sort | uniq`.text();
console.log(`Sorted unique:\n${sortedUnique}`);

// Filter files
const tsFiles = await $`ls src/ | grep ".ts$"`.nothrow().text();
console.log(`TypeScript files:\n${tsFiles}`);
```

## Example 4: Environment Variables

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Inline env var
await $`NODE_ENV=production bun -e "console.log(process.env.NODE_ENV)"`;

// Dynamic env var
const env = "development";
await $`MY_ENV=${env} bun -e "console.log('Env:', process.env.MY_ENV)"`;

// Using .env() method
const output = await $`echo $CUSTOM_VAR`
  .env({ CUSTOM_VAR: "my-value" })
  .text();
console.log(`Custom var: ${output.trim()}`);
```

## Example 5: Working Directory

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Change directory for single command
const files = await $`ls`.cwd("/tmp").text();
console.log("Files in /tmp:", files.trim().split('\n').length);

// Note: cd doesn't persist between commands
await $`cd /tmp`;
const pwd1 = await $`pwd`.text();
console.log(`PWD after cd: ${pwd1.trim()}`); // Still original directory!

// Use .cwd() instead
const pwd2 = await $`pwd`.cwd("/tmp").text();
console.log(`PWD with .cwd(): ${pwd2.trim()}`); // /tmp
```

## Example 6: Quiet Mode

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Default: prints to stdout
await $`echo "This prints to console"`;

// Quiet: only captures, no output
const { stdout } = await $`echo "This is captured"`.quiet();
console.log("Captured:", stdout.toString());

// .text() automatically quiets
const text = await $`echo "Also captured"`.text();
console.log("Text:", text);
```

## Example 7: JSON Output

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Parse JSON output
const pkg = await $`cat package.json`.json() as { name: string; version: string };
console.log(`Package: ${pkg.name}@${pkg.version}`);

// With jq (if available)
const deps = await $`cat package.json | jq '.dependencies | keys'`.nothrow().json();
if (deps) {
  console.log("Dependencies:", deps);
}
```

## Example 8: Line-by-Line Processing

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Iterate over lines
console.log("Files in current directory:");
for await (const line of $`ls -1`.lines()) {
  console.log(`  - ${line}`);
}

// Process each line
let count = 0;
for await (const file of $`find . -name "*.ts" -type f`.lines()) {
  count++;
  if (count <= 5) {
    console.log(`Found: ${file}`);
  }
}
console.log(`Total .ts files: ${count}`);
```

## Example 9: Command Chaining

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// AND operator - second runs only if first succeeds
await $`mkdir -p temp && echo "Created temp"`;

// Command sequence in single template
await $`
  cd temp &&
  echo "hello" > file.txt &&
  cat file.txt
`.cwd(".");

// Clean up
await $`rm -rf temp`;
```

## Example 10: Glob Patterns

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Single wildcard
await $`ls *.json`;

// Recursive glob
const allTs = await $`ls **/*.ts`.nothrow().text();
console.log("All TypeScript files:", allTs);

// Brace expansion
await $`echo {a,b,c}.txt`; // a.txt b.txt c.txt

// Combined
const sources = await $`ls src/**/*.{ts,tsx}`.nothrow().text();
console.log("Source files:", sources);
```

## Running These Examples

Save any example as a `.ts` file and run with:

```bash
bun run example.ts
```

Or make it executable:

```bash
chmod +x example.ts
./example.ts
```
