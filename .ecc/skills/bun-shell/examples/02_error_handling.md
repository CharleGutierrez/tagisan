# Error Handling in Bun Shell

## Example 1: Default Throwing Behavior

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// By default, non-zero exit codes throw
try {
  await $`exit 1`;
} catch (err) {
  if (err instanceof $.ShellError) {
    console.log("Command failed!");
    console.log("Exit code:", err.exitCode);
    console.log("Stdout:", err.stdout.toString());
    console.log("Stderr:", err.stderr.toString());
  }
}
```

## Example 2: Using .nothrow()

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Prevent throwing, check exit code manually
const result = await $`exit 42`.nothrow();

console.log("Exit code:", result.exitCode);  // 42
console.log("Did it fail?", result.exitCode !== 0);
```

## Example 3: Combining .nothrow() and .quiet()

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Common pattern: capture output and handle errors manually
const { exitCode, stdout, stderr } = await $`cat nonexistent-file.txt`
  .nothrow()
  .quiet();

if (exitCode !== 0) {
  console.error("Error:", stderr.toString());
} else {
  console.log("Content:", stdout.toString());
}
```

## Example 4: Optional Commands

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Check if a command exists
async function commandExists(cmd: string): Promise<boolean> {
  const { exitCode } = await $`which ${cmd}`.nothrow().quiet();
  return exitCode === 0;
}

// Use it
if (await commandExists('docker')) {
  console.log("Docker is available");
  await $`docker --version`;
} else {
  console.log("Docker not found");
}

// Check multiple
const tools = ['git', 'node', 'rust', 'go'];
for (const tool of tools) {
  const exists = await commandExists(tool);
  console.log(`${tool}: ${exists ? 'installed' : 'not found'}`);
}
```

## Example 5: Retry Logic

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

async function retryCommand(
  command: string,
  maxRetries = 3,
  delayMs = 1000
): Promise<string> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const result = await $`${command}`.text();
      return result;
    } catch (err) {
      console.log(`Attempt ${attempt}/${maxRetries} failed`);

      if (attempt === maxRetries) {
        throw new Error(`Command failed after ${maxRetries} attempts`);
      }

      await new Promise(r => setTimeout(r, delayMs));
    }
  }
  throw new Error("Unreachable");
}

// Usage
try {
  const result = await retryCommand('curl -sf http://example.com');
  console.log("Success:", result.slice(0, 100));
} catch (err) {
  console.error("All retries failed:", err);
}
```

## Example 6: Global Error Configuration

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

// Disable throwing globally
$.nothrow();
// Or: $.throws(false);

// Now all commands don't throw
const result1 = await $`exit 1`;
console.log("Exit code 1:", result1.exitCode);

const result2 = await $`exit 2`;
console.log("Exit code 2:", result2.exitCode);

// Re-enable for specific command
const result3 = await $`exit 0`.throws(true);
console.log("Exit code 0:", result3.exitCode);

// Re-enable globally
$.throws(true);

// This would throw now:
// await $`exit 1`;
```

## Example 7: ShellError Details

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

try {
  // Run a command that writes to both stdout and stderr, then fails
  await $`
    echo "stdout output" &&
    echo "stderr output" >&2 &&
    exit 1
  `;
} catch (err) {
  if (err instanceof $.ShellError) {
    console.log("--- ShellError Details ---");
    console.log("Exit Code:", err.exitCode);
    console.log("Message:", err.message);
    console.log("Stdout:", err.stdout.toString().trim());
    console.log("Stderr:", err.stderr.toString().trim());
    console.log("--------------------------");
  } else {
    throw err;
  }
}
```

## Example 8: Graceful Degradation

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

async function getSystemInfo() {
  // Try preferred command, fall back to alternatives
  let memInfo: string;

  const { exitCode: hasFreem } = await $`which free`.nothrow().quiet();
  if (hasFreem === 0) {
    memInfo = await $`free -h`.text();
  } else {
    // macOS alternative
    const { exitCode: hasVmStat } = await $`which vm_stat`.nothrow().quiet();
    if (hasVmStat === 0) {
      memInfo = await $`vm_stat`.text();
    } else {
      memInfo = "Memory info not available";
    }
  }

  return memInfo;
}

console.log(await getSystemInfo());
```

## Example 9: Timeout Handling

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

async function runWithTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number
): Promise<T> {
  const timeout = new Promise<never>((_, reject) => {
    setTimeout(() => reject(new Error('Timeout')), timeoutMs);
  });

  return Promise.race([promise, timeout]);
}

// Usage
try {
  const result = await runWithTimeout(
    $`sleep 10 && echo "done"`.text(),
    2000  // 2 second timeout
  );
  console.log(result);
} catch (err) {
  if (err instanceof Error && err.message === 'Timeout') {
    console.error("Command timed out!");
  } else {
    throw err;
  }
}
```

## Example 10: Error Aggregation

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

interface CommandResult {
  command: string;
  success: boolean;
  output?: string;
  error?: string;
}

async function runCommands(commands: string[]): Promise<CommandResult[]> {
  const results: CommandResult[] = [];

  for (const cmd of commands) {
    const { exitCode, stdout, stderr } = await $`${cmd}`.nothrow().quiet();

    results.push({
      command: cmd,
      success: exitCode === 0,
      output: exitCode === 0 ? stdout.toString() : undefined,
      error: exitCode !== 0 ? stderr.toString() : undefined
    });
  }

  return results;
}

// Run multiple commands
const results = await runCommands([
  'echo "hello"',
  'ls nonexistent',
  'pwd',
  'exit 42'
]);

console.log("Results:");
for (const r of results) {
  console.log(`  ${r.command}: ${r.success ? 'OK' : 'FAILED'}`);
}

const failures = results.filter(r => !r.success);
if (failures.length > 0) {
  console.log(`\n${failures.length} command(s) failed`);
}
```
