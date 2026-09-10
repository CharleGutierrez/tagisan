# File Operations with Bun Shell

## File System Basics

```typescript
import { $ } from "bun";

// List files with details
await $`ls -la`;

// List specific patterns
await $`ls *.json`;
await $`ls **/*.ts`;

// Create directories (recursive)
await $`mkdir -p src/components/ui`;

// Remove files/directories
await $`rm -rf node_modules dist`;

// Copy files
await $`cp config.example.json config.json`;

// Move/rename
await $`mv old-name.ts new-name.ts`;

// Create empty file
await $`touch .gitkeep`;
```

## File Content Operations

```typescript
import { $ } from "bun";

// Read file to string
const content = await $`cat package.json`.text();

// Read as JSON
const pkg = await $`cat package.json`.json() as { name: string; version: string };

// Read specific lines (first 10)
const first10 = await $`head -10 README.md`.text();

// Read last lines
const last20 = await $`tail -20 server.log`.text();

// Count lines
const lineCount = parseInt(await $`wc -l < file.txt`.text());

// Word count
const wordCount = await $`wc -w < file.txt`.text();
```

## Write and Append

```typescript
import { $ } from "bun";

// Write to file (overwrite)
await $`echo "Hello World" > output.txt`;

// Append to file
await $`echo "New line" >> output.txt`;

// Write multi-line content
const content = `
{
  "name": "my-app",
  "version": "1.0.0"
}
`;
await $`echo ${content} > config.json`;

// Write from JavaScript object to buffer then file
import { file } from "bun";
const data = { key: "value" };
await Bun.write(file("data.json"), JSON.stringify(data, null, 2));
```

## Search and Find

```typescript
import { $ } from "bun";

// Find files by name
const tsFiles = await $`find src -name "*.ts"`.text();

// Find files modified in last 24 hours
const recent = await $`find . -mtime -1 -type f`.text();

// Find large files (>1MB)
const large = await $`find . -size +1M -type f`.text();

// Search file contents
const matches = await $`grep -r "TODO" src/`.text();

// Search with line numbers
const withLines = await $`grep -rn "console.log" src/`.text();

// Find and process (safe with bun shell)
for await (const file of $`find src -name "*.test.ts"`.lines()) {
  console.log(`Test file: ${file}`);
}
```

## File Permissions

```typescript
import { $ } from "bun";

// Make executable
await $`chmod +x scripts/deploy.sh`;

// Set specific permissions
await $`chmod 644 config.json`;

// Check permissions
const perms = await $`ls -la script.sh`.text();
```

## Archive Operations

```typescript
import { $ } from "bun";

// Create tar.gz archive
await $`tar -czvf backup.tar.gz src/ package.json`;

// Extract archive
await $`tar -xzvf backup.tar.gz`;

// Create zip (if available)
await $`zip -r archive.zip dist/`;

// List archive contents
const contents = await $`tar -tzvf backup.tar.gz`.text();
```

## Batch File Operations

```typescript
import { $ } from "bun";

// Rename multiple files
async function batchRename(pattern: string, from: string, to: string) {
  for await (const file of $`find . -name "${pattern}"`.lines()) {
    const newName = file.replace(from, to);
    await $`mv ${file} ${newName}`;
    console.log(`Renamed: ${file} -> ${newName}`);
  }
}

// Usage: rename all .jsx to .tsx
await batchRename("*.jsx", ".jsx", ".tsx");
```

## Safe File Operations with Checks

```typescript
import { $ } from "bun";

async function safeDelete(path: string) {
  // Check if exists
  const { exitCode } = await $`test -e ${path}`.nothrow();

  if (exitCode !== 0) {
    console.log(`${path} does not exist`);
    return false;
  }

  // Check if directory
  const { exitCode: isDir } = await $`test -d ${path}`.nothrow();

  if (isDir === 0) {
    // It's a directory - use rm -rf
    await $`rm -rf ${path}`;
  } else {
    // It's a file
    await $`rm ${path}`;
  }

  return true;
}

async function safeCopy(src: string, dest: string, overwrite = false) {
  // Check source exists
  const { exitCode: srcExists } = await $`test -e ${src}`.nothrow();
  if (srcExists !== 0) {
    throw new Error(`Source not found: ${src}`);
  }

  // Check destination
  const { exitCode: destExists } = await $`test -e ${dest}`.nothrow();
  if (destExists === 0 && !overwrite) {
    throw new Error(`Destination exists: ${dest}`);
  }

  await $`cp -r ${src} ${dest}`;
}
```

## Directory Traversal

```typescript
import { $ } from "bun";

async function getDirectoryTree(dir: string, depth = 3) {
  const tree = await $`find ${dir} -maxdepth ${depth} -type d`.text();
  return tree.trim().split('\n');
}

async function getDirectorySize(dir: string) {
  const size = await $`du -sh ${dir}`.text();
  return size.split('\t')[0];
}

async function countFilesByExtension(dir: string) {
  const counts: Record<string, number> = {};

  for await (const file of $`find ${dir} -type f`.lines()) {
    const ext = file.split('.').pop() || 'no-extension';
    counts[ext] = (counts[ext] || 0) + 1;
  }

  return counts;
}
```

## Temporary Files

```typescript
import { $ } from "bun";

async function withTempFile<T>(fn: (path: string) => Promise<T>): Promise<T> {
  const tempFile = `/tmp/bun-temp-${Date.now()}.txt`;

  try {
    return await fn(tempFile);
  } finally {
    await $`rm -f ${tempFile}`.nothrow();
  }
}

// Usage
await withTempFile(async (temp) => {
  await $`echo "temporary data" > ${temp}`;
  const content = await $`cat ${temp}`.text();
  console.log(content);
});
```

## Sync/Backup Operations

```typescript
import { $ } from "bun";

async function syncDirectories(src: string, dest: string) {
  // Create dest if not exists
  await $`mkdir -p ${dest}`;

  // Use rsync if available, fallback to cp
  const { exitCode } = await $`which rsync`.nothrow();

  if (exitCode === 0) {
    await $`rsync -av --delete ${src}/ ${dest}/`;
  } else {
    await $`rm -rf ${dest}/*`;
    await $`cp -r ${src}/* ${dest}/`;
  }
}

async function createBackup(dir: string, backupDir = './backups') {
  const timestamp = new Date().toISOString().slice(0, 10);
  const backupName = `backup-${timestamp}.tar.gz`;

  await $`mkdir -p ${backupDir}`;
  await $`tar -czvf ${backupDir}/${backupName} ${dir}`;

  // Clean old backups (keep last 5)
  await $`ls -t ${backupDir}/*.tar.gz | tail -n +6 | xargs rm -f`.nothrow();

  return `${backupDir}/${backupName}`;
}
```
