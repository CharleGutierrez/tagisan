# Git Automation with Bun Shell

## Check Repository Status

```typescript
import { $ } from "bun";

async function checkGitStatus() {
  // Check for uncommitted changes
  const { exitCode: hasChanges } = await $`git diff --quiet`.nothrow();

  // Check for staged changes
  const { exitCode: hasStaged } = await $`git diff --cached --quiet`.nothrow();

  // Check for untracked files
  const untracked = await $`git ls-files --others --exclude-standard`.text();

  return {
    hasUncommittedChanges: hasChanges !== 0,
    hasStagedChanges: hasStaged !== 0,
    hasUntrackedFiles: untracked.trim().length > 0,
    untrackedFiles: untracked.trim().split('\n').filter(Boolean)
  };
}
```

## Get Current Branch and Commit

```typescript
import { $ } from "bun";

async function getGitInfo() {
  const branch = await $`git rev-parse --abbrev-ref HEAD`.text();
  const commit = await $`git rev-parse --short HEAD`.text();
  const message = await $`git log -1 --pretty=%B`.text();

  return {
    branch: branch.trim(),
    commit: commit.trim(),
    message: message.trim()
  };
}
```

## Automated Commit with Conventional Format

```typescript
import { $ } from "bun";

type CommitType = 'feat' | 'fix' | 'docs' | 'style' | 'refactor' | 'test' | 'chore';

async function conventionalCommit(type: CommitType, scope: string, message: string) {
  // Stage all changes
  await $`git add -A`;

  // Check if there are changes to commit
  const { exitCode } = await $`git diff --cached --quiet`.nothrow();
  if (exitCode === 0) {
    console.log("No changes to commit");
    return false;
  }

  // Create commit message
  const fullMessage = scope ? `${type}(${scope}): ${message}` : `${type}: ${message}`;

  // Commit (message is auto-escaped)
  await $`git commit -m ${fullMessage}`;
  return true;
}

// Usage
await conventionalCommit('feat', 'auth', 'add OAuth2 login support');
```

## Git Log Parsing

```typescript
import { $ } from "bun";

interface Commit {
  hash: string;
  author: string;
  date: string;
  message: string;
}

async function getRecentCommits(count = 10): Promise<Commit[]> {
  const commits: Commit[] = [];

  // Use custom format for easy parsing
  const format = '%H|%an|%ad|%s';
  for await (const line of $`git log -${count} --pretty=format:${format}`.lines()) {
    const [hash, author, date, message] = line.split('|');
    commits.push({ hash, author, date, message });
  }

  return commits;
}
```

## Branch Management

```typescript
import { $ } from "bun";

async function createFeatureBranch(name: string) {
  // Ensure we're on main/master
  const currentBranch = (await $`git rev-parse --abbrev-ref HEAD`.text()).trim();

  if (currentBranch !== 'main' && currentBranch !== 'master') {
    console.log(`Warning: Creating branch from ${currentBranch}`);
  }

  // Pull latest
  await $`git pull origin ${currentBranch}`.nothrow();

  // Create and checkout new branch
  const branchName = `feature/${name}`;
  await $`git checkout -b ${branchName}`;

  return branchName;
}

async function cleanupMergedBranches() {
  // Get merged branches (excluding main/master)
  const merged = await $`git branch --merged`.text();
  const branches = merged
    .split('\n')
    .map(b => b.trim())
    .filter(b => b && !b.startsWith('*') && !['main', 'master'].includes(b));

  for (const branch of branches) {
    console.log(`Deleting merged branch: ${branch}`);
    await $`git branch -d ${branch}`.nothrow();
  }
}
```

## Tag Release

```typescript
import { $ } from "bun";

async function tagRelease(version: string, message?: string) {
  const tagMessage = message || `Release ${version}`;

  // Create annotated tag
  await $`git tag -a v${version} -m ${tagMessage}`;

  // Push tag
  await $`git push origin v${version}`;

  console.log(`Tagged and pushed v${version}`);
}
```

## Git Diff for CI

```typescript
import { $ } from "bun";

async function getChangedFiles(base = 'main') {
  const diff = await $`git diff --name-only ${base}...HEAD`.text();
  return diff.trim().split('\n').filter(Boolean);
}

async function hasChangesIn(patterns: string[], base = 'main') {
  const files = await getChangedFiles(base);

  return patterns.some(pattern => {
    const regex = new RegExp(pattern.replace('*', '.*'));
    return files.some(f => regex.test(f));
  });
}

// Usage in CI
const needsBackendBuild = await hasChangesIn(['src/api/*', 'package.json']);
const needsFrontendBuild = await hasChangesIn(['src/components/*', 'src/pages/*']);
```

## Stash Management

```typescript
import { $ } from "bun";

async function stashAndPull() {
  // Stash current changes
  const stashResult = await $`git stash push -m "Auto-stash before pull"`.text();
  const didStash = !stashResult.includes('No local changes');

  // Pull latest
  await $`git pull --rebase`;

  // Pop stash if we stashed
  if (didStash) {
    const { exitCode } = await $`git stash pop`.nothrow();
    if (exitCode !== 0) {
      console.error("Conflict when applying stash!");
      return false;
    }
  }

  return true;
}
```
