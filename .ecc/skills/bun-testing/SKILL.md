---
name: "bun-testing"
description: "Run and write tests with Bun's built-in test runner. Use when user needs fast testing, Jest-compatible syntax, mocking, snapshots, or coverage. Keywords: bun test, testing, jest, vitest, mock, spy, coverage, snapshot, TDD."
---
# Bun Testing

Bun includes a blazing-fast Jest-compatible test runner. Tests run up to 100x faster than Jest.

## Quick Start

```bash
# Run all tests
bun test

# Run specific file
bun test src/utils.test.ts

# Run with pattern
bun test --test-name-pattern "should calculate"

# Watch mode
bun test --watch

# Coverage
bun test --coverage
```

## Test File Conventions

Bun automatically finds test files matching:
- `*.test.ts`, `*.test.js`
- `*.spec.ts`, `*.spec.js`
- Files in `__tests__/` directories

## Writing Tests

### Basic Test Syntax

```typescript
import { describe, it, expect, test } from "bun:test";

describe("Math operations", () => {
  it("should add numbers correctly", () => {
    expect(1 + 1).toBe(2);
  });

  it("should handle negative numbers", () => {
    expect(-1 + 1).toBe(0);
  });

  // Alternative: use test() instead of it()
  test("multiplication works", () => {
    expect(2 * 3).toBe(6);
  });
});
```

### Lifecycle Hooks

```typescript
import { describe, it, beforeAll, afterAll, beforeEach, afterEach } from "bun:test";

describe("Database tests", () => {
  let db;

  beforeAll(async () => {
    db = await connectToDatabase();
  });

  afterAll(async () => {
    await db.close();
  });

  beforeEach(async () => {
    await db.clear();
  });

  afterEach(() => {
    console.log("Test completed");
  });

  it("should insert data", async () => {
    await db.insert({ name: "test" });
    expect(await db.count()).toBe(1);
  });
});
```

### Async Tests

```typescript
it("handles async operations", async () => {
  const result = await fetchData();
  expect(result).toBeDefined();
});

it("handles promises", () => {
  return fetchData().then(result => {
    expect(result).toBeDefined();
  });
});
```

## Matchers

### Common Matchers

```typescript
// Equality
expect(value).toBe(expected);           // Strict equality (===)
expect(value).toEqual(expected);        // Deep equality
expect(value).toStrictEqual(expected);  // Deep + type equality

// Truthiness
expect(value).toBeTruthy();
expect(value).toBeFalsy();
expect(value).toBeNull();
expect(value).toBeUndefined();
expect(value).toBeDefined();

// Numbers
expect(value).toBeGreaterThan(3);
expect(value).toBeGreaterThanOrEqual(3);
expect(value).toBeLessThan(5);
expect(value).toBeCloseTo(0.3, 5);  // Floating point

// Strings
expect(string).toMatch(/pattern/);
expect(string).toContain("substring");
expect(string).toHaveLength(10);

// Arrays/Iterables
expect(array).toContain(item);
expect(array).toContainEqual({ name: "test" });
expect(array).toHaveLength(3);

// Objects
expect(obj).toHaveProperty("key");
expect(obj).toHaveProperty("nested.key", "value");
expect(obj).toMatchObject({ key: "value" });

// Exceptions
expect(() => fn()).toThrow();
expect(() => fn()).toThrow(Error);
expect(() => fn()).toThrow("error message");

// Negation
expect(value).not.toBe(unexpected);
```

### Async Matchers

```typescript
await expect(promise).resolves.toBe(expected);
await expect(promise).rejects.toThrow();
```

## Mocking

### Mock Functions

```typescript
import { mock, expect } from "bun:test";

const fn = mock(() => "default");

fn();
fn("arg1", "arg2");

expect(fn).toHaveBeenCalled();
expect(fn).toHaveBeenCalledTimes(2);
expect(fn).toHaveBeenCalledWith("arg1", "arg2");

// Set return value
fn.mockReturnValue("mocked");
expect(fn()).toBe("mocked");

// Set implementation
fn.mockImplementation((x) => x * 2);
expect(fn(5)).toBe(10);
```

### Spying

```typescript
import { spyOn, expect } from "bun:test";

const obj = {
  method: () => "original",
};

const spy = spyOn(obj, "method");

obj.method();
expect(spy).toHaveBeenCalled();

// Mock implementation
spy.mockReturnValue("mocked");
expect(obj.method()).toBe("mocked");

// Restore original
spy.mockRestore();
```

### Module Mocking

```typescript
import { mock, expect } from "bun:test";

// Mock entire module
mock.module("./database", () => ({
  query: mock(() => [{ id: 1 }]),
  connect: mock(() => Promise.resolve()),
}));

// Now imports use the mock
import { query } from "./database";
const result = query();
expect(result).toEqual([{ id: 1 }]);
```

## Snapshots

```typescript
import { expect } from "bun:test";

it("matches snapshot", () => {
  const user = { name: "John", age: 30 };
  expect(user).toMatchSnapshot();
});

it("matches inline snapshot", () => {
  const result = calculateSum([1, 2, 3]);
  expect(result).toMatchInlineSnapshot(`6`);
});

// Update snapshots
// bun test --update-snapshots
```

## Coverage

```bash
# Run with coverage
bun test --coverage

# Set thresholds in bunfig.toml
```

```toml
# bunfig.toml
[test]
coverage = true
coverageThreshold = { line = 80, function = 80, branch = 70 }
```

## Configuration

### bunfig.toml

```toml
[test]
# Test runner settings
root = "./tests"
preload = ["./setup.ts"]
timeout = 5000

# Coverage
coverage = true
coverageDir = "./coverage"
coverageReporters = ["text", "lcov"]

# Patterns
include = ["**/*.test.ts"]
exclude = ["node_modules", "dist"]
```

### CLI Options

```bash
bun test [options] [patterns]

Options:
  --watch               Watch mode
  --coverage            Enable coverage
  --update-snapshots    Update snapshots
  --timeout <ms>        Test timeout
  --bail                Stop on first failure
  --only                Run tests marked .only
  --todo                Show todo tests
  --test-name-pattern   Filter by test name
  --preload <file>      Preload script
  -t, --test-name-pattern <pattern>  Filter tests by name
```

## Workflow

### Test-Driven Development

1. **Write failing test:**
   ```typescript
   it("should add user", async () => {
     const user = await addUser({ name: "John" });
     expect(user.id).toBeDefined();
   });
   ```

2. **Run test (fails):**
   ```bash
   bun test
   ```

3. **Implement feature**

4. **Run test (passes):**
   ```bash
   bun test
   ```

5. **Refactor with confidence**

### CI Integration

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v1
      - run: bun install
      - run: bun test --coverage
```

## Migration from Jest

Most Jest tests work unchanged:

```typescript
// Jest
import { describe, it, expect, jest } from '@jest/globals';

// Bun
import { describe, it, expect, mock } from 'bun:test';

// jest.fn() → mock()
// jest.spyOn() → spyOn()
// jest.mock() → mock.module()
```

## Performance Tips

1. **Use `bun:test` imports** - Native, not polyfilled
2. **Avoid global setup** - Use `beforeAll` in describe blocks
3. **Mock I/O** - Database, network, file system
4. **Run in parallel** - Default behavior, no config needed
5. **Use `--bail`** - Stop on first failure during development

## Troubleshooting

**Tests not found:**
- Check file naming (*.test.ts, *.spec.ts)
- Check bunfig.toml include/exclude patterns

**Timeout errors:**
- Increase timeout: `bun test --timeout 10000`
- Check for unresolved promises

**Mock not working:**
- Ensure `mock.module()` called before import
- Use `mock.module()` for ESM, not `jest.mock()`
