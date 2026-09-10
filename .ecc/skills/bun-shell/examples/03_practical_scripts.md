# Practical Bun Shell Scripts

## Script 1: Project Initializer

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const projectName = process.argv[2];
if (!projectName) {
  console.error("Usage: bun init-project.ts <project-name>");
  process.exit(1);
}

console.log(`Creating project: ${projectName}`);

// Create directory structure
await $`mkdir -p ${projectName}/src`;
await $`mkdir -p ${projectName}/tests`;

// Initialize with bun
await $`cd ${projectName} && bun init -y`.cwd(".");

// Create basic files
await $`echo 'console.log("Hello!");' > ${projectName}/src/index.ts`;
await $`echo '# ${projectName}' > ${projectName}/README.md`;

// Initialize git
await $`cd ${projectName} && git init`.cwd(".");

console.log(`Project ${projectName} created!`);
await $`ls -la ${projectName}`;
```

## Script 2: Dependency Audit

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

console.log("=== Dependency Audit ===\n");

// Get package.json
const pkg = await $`cat package.json`.json() as {
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
};

const deps = {
  ...pkg.dependencies,
  ...pkg.devDependencies
};

console.log(`Total packages: ${Object.keys(deps).length}`);

// Check for outdated packages
console.log("\nOutdated packages:");
const outdated = await $`bun outdated`.nothrow().text();
console.log(outdated || "All packages up to date!");

// Check node_modules size
const size = await $`du -sh node_modules`.nothrow().text();
console.log(`\nnode_modules size: ${size.split('\t')[0]}`);

// Find largest packages
console.log("\nLargest packages:");
await $`du -sh node_modules/* | sort -rh | head -10`;
```

## Script 3: Log Analyzer

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const logFile = process.argv[2] || 'server.log';

console.log(`Analyzing: ${logFile}\n`);

// Check if file exists
const { exitCode } = await $`test -f ${logFile}`.nothrow();
if (exitCode !== 0) {
  console.error(`File not found: ${logFile}`);
  process.exit(1);
}

// Total lines
const totalLines = parseInt(await $`wc -l < ${logFile}`.text());
console.log(`Total lines: ${totalLines}`);

// Error count
const errors = await $`grep -c "ERROR" ${logFile}`.nothrow().text();
console.log(`Errors: ${errors.trim()}`);

// Warning count
const warnings = await $`grep -c "WARN" ${logFile}`.nothrow().text();
console.log(`Warnings: ${warnings.trim()}`);

// Last 5 errors
console.log("\nLast 5 errors:");
await $`grep "ERROR" ${logFile} | tail -5`;

// Unique error types
console.log("\nUnique error patterns:");
await $`grep "ERROR" ${logFile} | cut -d']' -f2 | sort | uniq -c | sort -rn | head -5`;
```

## Script 4: Port Manager

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const action = process.argv[2];
const port = process.argv[3];

async function listPorts() {
  console.log("Active ports:");
  await $`lsof -i -P -n | grep LISTEN`.nothrow();
}

async function findPort(port: string) {
  console.log(`Processes on port ${port}:`);
  const result = await $`lsof -i :${port}`.nothrow();
  if (result.exitCode !== 0) {
    console.log("No process found on this port");
  }
}

async function killPort(port: string) {
  const pid = await $`lsof -t -i :${port}`.nothrow().text();
  if (pid.trim()) {
    await $`kill -9 ${pid.trim()}`;
    console.log(`Killed process ${pid.trim()} on port ${port}`);
  } else {
    console.log("No process found on this port");
  }
}

switch (action) {
  case 'list':
    await listPorts();
    break;
  case 'find':
    if (!port) {
      console.error("Usage: bun ports.ts find <port>");
      process.exit(1);
    }
    await findPort(port);
    break;
  case 'kill':
    if (!port) {
      console.error("Usage: bun ports.ts kill <port>");
      process.exit(1);
    }
    await killPort(port);
    break;
  default:
    console.log("Usage: bun ports.ts <list|find|kill> [port]");
}
```

## Script 5: Git Branch Cleaner

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

console.log("=== Git Branch Cleaner ===\n");

// Get current branch
const current = (await $`git rev-parse --abbrev-ref HEAD`.text()).trim();
console.log(`Current branch: ${current}`);

// Get merged branches (excluding main/master/current)
const merged = await $`git branch --merged`.text();
const branchesToDelete = merged
  .split('\n')
  .map(b => b.trim())
  .filter(b => b && !b.startsWith('*'))
  .filter(b => !['main', 'master', 'develop'].includes(b));

if (branchesToDelete.length === 0) {
  console.log("\nNo merged branches to clean up!");
  process.exit(0);
}

console.log(`\nBranches to delete (${branchesToDelete.length}):`);
for (const branch of branchesToDelete) {
  console.log(`  - ${branch}`);
}

// Confirm before deleting
console.log("\nDeleting...");
for (const branch of branchesToDelete) {
  await $`git branch -d ${branch}`;
  console.log(`  Deleted: ${branch}`);
}

console.log("\nCleanup complete!");
```

## Script 6: Docker Cleanup

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

console.log("=== Docker Cleanup ===\n");

// Check if docker is running
const { exitCode } = await $`docker info`.nothrow().quiet();
if (exitCode !== 0) {
  console.error("Docker is not running!");
  process.exit(1);
}

// Show current usage
console.log("Current usage:");
await $`docker system df`;

// Stop all running containers
console.log("\nStopping containers...");
const running = await $`docker ps -q`.nothrow().text();
if (running.trim()) {
  await $`docker stop $(docker ps -q)`.nothrow();
}

// Remove stopped containers
console.log("Removing stopped containers...");
await $`docker container prune -f`;

// Remove dangling images
console.log("Removing dangling images...");
await $`docker image prune -f`;

// Remove unused volumes
console.log("Removing unused volumes...");
await $`docker volume prune -f`;

// Show new usage
console.log("\nNew usage:");
await $`docker system df`;
```

## Script 7: Backup Creator

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const sourceDir = process.argv[2] || '.';
const backupDir = process.argv[3] || './backups';

const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
const backupName = `backup-${timestamp}.tar.gz`;

console.log(`Creating backup of ${sourceDir}...`);

// Create backup directory
await $`mkdir -p ${backupDir}`;

// Create exclude file for common patterns
const excludes = [
  'node_modules',
  '.git',
  'dist',
  'build',
  '.cache',
  '*.log'
];

// Create tarball
await $`tar -czvf ${backupDir}/${backupName} \
  ${excludes.map(e => `--exclude='${e}'`).join(' ')} \
  ${sourceDir}`.nothrow();

// Get backup size
const size = await $`du -h ${backupDir}/${backupName}`.text();
console.log(`\nBackup created: ${backupName}`);
console.log(`Size: ${size.split('\t')[0]}`);

// Keep only last 5 backups
console.log("\nCleaning old backups...");
await $`ls -t ${backupDir}/*.tar.gz | tail -n +6 | xargs rm -f`.nothrow();

// List backups
console.log("\nExisting backups:");
await $`ls -lh ${backupDir}/*.tar.gz`;
```

## Script 8: Health Checker

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

interface HealthCheck {
  name: string;
  command: string;
  expected?: string;
}

const checks: HealthCheck[] = [
  { name: 'Disk Space', command: 'df -h / | tail -1' },
  { name: 'Memory', command: 'free -h | grep Mem' },
  { name: 'Load Average', command: 'uptime' },
  { name: 'Docker', command: 'docker info > /dev/null && echo "Running"' },
  { name: 'Node', command: 'node --version' },
  { name: 'Bun', command: 'bun --version' },
];

console.log("=== System Health Check ===\n");

for (const check of checks) {
  process.stdout.write(`${check.name}: `);

  const { exitCode, stdout, stderr } = await $`${check.command}`.nothrow().quiet();

  if (exitCode === 0) {
    console.log(stdout.toString().trim().split('\n')[0]);
  } else {
    console.log(`FAILED - ${stderr.toString().trim() || 'Unknown error'}`);
  }
}

// Check web services
console.log("\n=== Web Services ===\n");

const services = [
  { name: 'Google', url: 'https://www.google.com' },
  { name: 'GitHub', url: 'https://api.github.com' },
  { name: 'Localhost:3000', url: 'http://localhost:3000' },
];

for (const service of services) {
  process.stdout.write(`${service.name}: `);

  const { exitCode } = await $`curl -sf -o /dev/null -w "%{http_code}" ${service.url}`
    .nothrow()
    .quiet();

  console.log(exitCode === 0 ? 'OK' : 'UNREACHABLE');
}
```

## Script 9: Release Script

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

const versionBump = process.argv[2] || 'patch';

console.log(`=== Release (${versionBump}) ===\n`);

// Ensure clean working directory
const status = await $`git status --porcelain`.text();
if (status.trim()) {
  console.error("Working directory not clean!");
  console.log(status);
  process.exit(1);
}

// Ensure on main branch
const branch = (await $`git rev-parse --abbrev-ref HEAD`.text()).trim();
if (branch !== 'main' && branch !== 'master') {
  console.error(`Not on main branch (on: ${branch})`);
  process.exit(1);
}

// Pull latest
console.log("Pulling latest...");
await $`git pull`;

// Run tests
console.log("Running tests...");
await $`bun test`;

// Bump version
console.log(`Bumping ${versionBump} version...`);
await $`npm version ${versionBump} --no-git-tag-version`;

// Get new version
const pkg = await $`cat package.json`.json() as { version: string };
const newVersion = pkg.version;
console.log(`New version: ${newVersion}`);

// Build
console.log("Building...");
await $`bun run build`;

// Commit and tag
await $`git add -A`;
await $`git commit -m "Release v${newVersion}"`;
await $`git tag -a v${newVersion} -m "Release v${newVersion}"`;

// Push
console.log("Pushing...");
await $`git push && git push --tags`;

console.log(`\nReleased v${newVersion}!`);
```

## Script 10: Dev Environment Setup

```typescript
#!/usr/bin/env bun
import { $ } from "bun";

console.log("=== Dev Environment Setup ===\n");

// Check required tools
const tools = ['git', 'node', 'bun', 'docker'];
const missing: string[] = [];

for (const tool of tools) {
  const { exitCode } = await $`which ${tool}`.nothrow().quiet();
  if (exitCode !== 0) {
    missing.push(tool);
    console.log(`${tool}: NOT FOUND`);
  } else {
    const version = await $`${tool} --version`.nothrow().text();
    console.log(`${tool}: ${version.trim().split('\n')[0]}`);
  }
}

if (missing.length > 0) {
  console.error(`\nMissing tools: ${missing.join(', ')}`);
  process.exit(1);
}

// Install dependencies
console.log("\nInstalling dependencies...");
await $`bun install`;

// Set up git hooks
console.log("Setting up git hooks...");
const { exitCode: hasHusky } = await $`test -d .husky`.nothrow();
if (hasHusky === 0) {
  await $`bun run prepare`.nothrow();
}

// Create .env from example
const { exitCode: hasEnvExample } = await $`test -f .env.example`.nothrow();
const { exitCode: hasEnv } = await $`test -f .env`.nothrow();
if (hasEnvExample === 0 && hasEnv !== 0) {
  console.log("Creating .env from .env.example...");
  await $`cp .env.example .env`;
}

// Start Docker services
const { exitCode: hasCompose } = await $`test -f docker-compose.yml`.nothrow();
if (hasCompose === 0) {
  console.log("\nStarting Docker services...");
  await $`docker compose up -d`;
}

console.log("\nDev environment ready!");
console.log("Run 'bun run dev' to start development server.");
```
