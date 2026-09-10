# Build Scripts with Bun Shell

## Clean and Build

```typescript
import { $ } from "bun";

async function cleanBuild() {
  console.log("Cleaning build artifacts...");
  await $`rm -rf dist build .cache`;

  console.log("Installing dependencies...");
  await $`bun install`;

  console.log("Building...");
  await $`bun run build`;

  console.log("Build complete!");
}
```

## Multi-Stage Build with Error Handling

```typescript
import { $ } from "bun";

interface BuildResult {
  success: boolean;
  stage: string;
  error?: string;
}

async function multistageBuild(): Promise<BuildResult> {
  const stages = [
    { name: 'lint', cmd: 'bun run lint' },
    { name: 'typecheck', cmd: 'bun run typecheck' },
    { name: 'test', cmd: 'bun test' },
    { name: 'build', cmd: 'bun run build' },
  ];

  for (const stage of stages) {
    console.log(`\n[${stage.name}] Running...`);

    try {
      await $`${stage.cmd}`.quiet();
      console.log(`[${stage.name}] Passed`);
    } catch (err) {
      if (err instanceof $.ShellError) {
        return {
          success: false,
          stage: stage.name,
          error: err.stderr.toString()
        };
      }
      throw err;
    }
  }

  return { success: true, stage: 'complete' };
}
```

## Watch Mode with Rebuild

```typescript
import { $ } from "bun";
import { watch } from "fs";

async function watchAndBuild(srcDir = './src') {
  console.log(`Watching ${srcDir} for changes...`);

  let building = false;

  const rebuild = async () => {
    if (building) return;
    building = true;

    console.log('\nRebuilding...');
    const start = Date.now();

    try {
      await $`bun run build`.quiet();
      console.log(`Built in ${Date.now() - start}ms`);
    } catch (err) {
      console.error('Build failed');
    }

    building = false;
  };

  watch(srcDir, { recursive: true }, (event, filename) => {
    if (filename?.endsWith('.ts') || filename?.endsWith('.tsx')) {
      console.log(`Changed: ${filename}`);
      rebuild();
    }
  });

  // Initial build
  await rebuild();
}
```

## Parallel Build Tasks

```typescript
import { $ } from "bun";

async function parallelBuild() {
  console.log("Running parallel build tasks...");

  const results = await Promise.allSettled([
    $`bun run build:client`.quiet(),
    $`bun run build:server`.quiet(),
    $`bun run build:worker`.quiet(),
  ]);

  const failed = results.filter(r => r.status === 'rejected');
  if (failed.length > 0) {
    console.error(`${failed.length} build tasks failed`);
    process.exit(1);
  }

  console.log("All builds completed successfully");
}
```

## Build with Version Injection

```typescript
import { $ } from "bun";

async function buildWithVersion() {
  // Get version from package.json
  const pkg = await $`cat package.json`.json() as { version: string };
  const version = pkg.version;

  // Get git commit hash
  const commit = (await $`git rev-parse --short HEAD`.text()).trim();

  // Build timestamp
  const buildTime = new Date().toISOString();

  // Run build with injected env vars
  await $`bun run build`.env({
    VITE_APP_VERSION: version,
    VITE_GIT_COMMIT: commit,
    VITE_BUILD_TIME: buildTime,
  });

  console.log(`Built v${version} (${commit}) at ${buildTime}`);
}
```

## Bundle Size Check

```typescript
import { $ } from "bun";

async function checkBundleSize(maxKB = 500) {
  // Build first
  await $`bun run build`.quiet();

  // Get bundle size
  const sizeOutput = await $`du -sk dist`.text();
  const sizeKB = parseInt(sizeOutput.split('\t')[0]);

  console.log(`Bundle size: ${sizeKB}KB (max: ${maxKB}KB)`);

  if (sizeKB > maxKB) {
    console.error('Bundle size exceeds limit!');

    // Show largest files
    console.log('\nLargest files:');
    await $`find dist -type f -exec du -k {} + | sort -rn | head -10`;

    process.exit(1);
  }
}
```

## Docker Build Integration

```typescript
import { $ } from "bun";

async function dockerBuild(tag: string) {
  const imageName = `myapp:${tag}`;

  console.log(`Building Docker image: ${imageName}`);

  // Build the image
  await $`docker build -t ${imageName} .`;

  // Get image size
  const info = await $`docker images ${imageName} --format "{{.Size}}"`.text();
  console.log(`Image size: ${info.trim()}`);

  // Optional: push to registry
  // await $`docker push ${imageName}`;

  return imageName;
}
```

## Build Artifact Archiving

```typescript
import { $ } from "bun";

async function archiveBuild() {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const archiveName = `build-${timestamp}.tar.gz`;

  // Build first
  await $`bun run build`;

  // Create archive
  await $`tar -czvf ${archiveName} dist/`;

  // Get archive info
  const size = await $`du -h ${archiveName}`.text();
  console.log(`Created ${archiveName} (${size.split('\t')[0]})`);

  return archiveName;
}
```

## CI/CD Pipeline Script

```typescript
import { $ } from "bun";

async function ciPipeline() {
  const startTime = Date.now();

  console.log("=== CI Pipeline Started ===\n");

  // Install dependencies
  console.log("1. Installing dependencies...");
  await $`bun install --frozen-lockfile`;

  // Lint
  console.log("\n2. Running linter...");
  await $`bun run lint`;

  // Type check
  console.log("\n3. Type checking...");
  await $`bun run typecheck`;

  // Test with coverage
  console.log("\n4. Running tests...");
  await $`bun test --coverage`;

  // Build
  console.log("\n5. Building...");
  await $`bun run build`;

  // E2E tests (if applicable)
  const { exitCode } = await $`test -f playwright.config.ts`.nothrow();
  if (exitCode === 0) {
    console.log("\n6. Running E2E tests...");
    await $`bun run test:e2e`;
  }

  const duration = ((Date.now() - startTime) / 1000).toFixed(1);
  console.log(`\n=== Pipeline Complete (${duration}s) ===`);
}
```

## Monorepo Build Script

```typescript
import { $ } from "bun";

async function buildMonorepo() {
  // Find all packages
  const packagesOutput = await $`ls packages`.text();
  const packages = packagesOutput.trim().split('\n');

  console.log(`Building ${packages.length} packages...`);

  // Build in dependency order (assumes topological order)
  for (const pkg of packages) {
    console.log(`\nBuilding ${pkg}...`);
    await $`bun run build`.cwd(`packages/${pkg}`);
  }

  console.log("\nAll packages built!");
}
```
