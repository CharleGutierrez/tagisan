# Bun Package Management

Bun's package manager is 25x faster than npm with near-instant installs.

## Basic Commands

### Install
```bash
# Install all dependencies
bun install

# Install with frozen lockfile (CI)
bun install --frozen-lockfile

# Install without dev dependencies
bun install --production
```

### Add Packages
```bash
# Add dependency
bun add package-name

# Add multiple
bun add react react-dom

# Add dev dependency
bun add -d typescript @types/node

# Add optional dependency
bun add -o sharp

# Add peer dependency
bun add --peer react

# Add global package
bun add -g eslint

# Add specific version
bun add lodash@4.17.21

# Add from git
bun add github:user/repo
bun add git+https://github.com/user/repo.git

# Add from tarball
bun add https://example.com/package.tgz
```

### Remove Packages
```bash
bun remove package-name
bun remove -g global-package
```

### Update Packages
```bash
# Update all
bun update

# Update specific package
bun update lodash

# Interactive update
bun update --interactive
```

## Lockfile

Bun uses a binary lockfile (`bun.lockb`) for faster parsing:

```bash
# View lockfile as text
bun bun.lockb

# Upgrade from npm/yarn/pnpm
# Just delete old lockfile and run:
rm package-lock.json yarn.lock pnpm-lock.yaml
bun install
```

## Workspaces (Monorepos)

```json
// package.json
{
  "workspaces": [
    "packages/*",
    "apps/*"
  ]
}
```

```bash
# Install all workspace dependencies
bun install

# Run script in specific workspace
bun run --filter=@myorg/package-a build

# Add dependency to workspace
bun add lodash --filter=@myorg/package-a
```

## bunfig.toml Configuration

```toml
# bunfig.toml

[install]
# Use yarn-style node_modules
nodeModulesDir = true

# Registry configuration
registry = "https://registry.npmjs.org"

# Scoped registries
[install.scopes]
"@myorg" = { registry = "https://npm.myorg.com", token = "..." }

# Cache location
[install.cache]
dir = "~/.bun/install/cache"

# Trusted dependencies (scripts allowed to run)
[install.trustedDependencies]
"esbuild" = true
"sharp" = true
```

## Private Registries

```toml
# bunfig.toml
[install]
registry = "https://npm.mycompany.com"

[install.scopes]
"@private" = {
  registry = "https://npm.mycompany.com",
  token = "$NPM_TOKEN"  # From env var
}
```

Or via npm config:
```bash
bun config set registry https://npm.mycompany.com
bun config set //npm.mycompany.com/:_authToken $NPM_TOKEN
```

## Overrides

```json
// package.json
{
  "overrides": {
    "lodash": "4.17.21",
    "react": "^18.0.0"
  }
}
```

## Patching Dependencies

```bash
# Create patch
bun patch lodash

# Edit node_modules/lodash/...
# Then commit the patch
bun patch --commit lodash
```

Creates `.patches/lodash.patch` file.

## Scripts

```json
// package.json
{
  "scripts": {
    "dev": "bun --hot run src/index.ts",
    "build": "bun build src/index.ts --outdir dist",
    "test": "bun test",
    "lint": "bun x eslint ."
  }
}
```

```bash
# Run script
bun run dev
bun dev  # Shorthand for common scripts

# Run with arguments
bun run build -- --minify
```

## Package Execution (bunx)

```bash
# Run package without installing
bunx create-react-app my-app

# Run specific version
bunx typescript@5.0.0 --version

# Run from npm
bunx npm:cowsay "Hello"
```

## Cache Management

```bash
# Clear cache
bun pm cache rm

# View cache
bun pm cache ls

# Prune unused packages
bun pm cache prune
```

## Migration from npm/yarn/pnpm

### From npm
```bash
rm -rf node_modules package-lock.json
bun install
```

### From yarn
```bash
rm -rf node_modules yarn.lock
bun install
```

### From pnpm
```bash
rm -rf node_modules pnpm-lock.yaml
bun install
```

## Comparison

| Feature | bun | npm | yarn | pnpm |
|---------|-----|-----|------|------|
| Install speed | ~25x faster | 1x | ~2x | ~3x |
| Lockfile | Binary | JSON | YAML | YAML |
| Disk usage | Low | High | Medium | Low |
| Workspaces | Yes | Yes | Yes | Yes |
| Patching | Yes | No | Yes | Yes |
| Plug'n'Play | No | No | Yes | No |

## Troubleshooting

### Corrupted lockfile
```bash
rm bun.lockb
bun install
```

### Package not found
```bash
# Clear cache
bun pm cache rm
bun install
```

### Postinstall script fails
```bash
# Skip scripts
bun install --ignore-scripts

# Then run manually if needed
cd node_modules/problematic-package && npm run postinstall
```

### Version conflicts
```bash
# Check why package installed
bun pm why package-name

# Force specific version
# In package.json overrides
```
