---
name: "bun-builder"
description: "Bundle and build JavaScript/TypeScript with Bun's native bundler. Use when user needs to bundle apps, create executables, or optimize builds. Keywords: bun build, bundler, bundle, compile, executable, minify, tree-shake, esbuild alternative."
---
# Bun Builder

Bun's native bundler is faster than esbuild and webpack, with built-in TypeScript/JSX support and the ability to create standalone executables.

## Quick Start

```bash
# Bundle for browser
bun build ./src/index.ts --outdir ./dist

# Bundle for Node.js
bun build ./src/index.ts --outdir ./dist --target node

# Create executable
bun build ./src/cli.ts --compile --outfile myapp

# Minified production build
bun build ./src/index.ts --outdir ./dist --minify
```

## CLI Options

### Basic Options

```bash
bun build <entrypoints...> [options]

# Entry points
bun build ./src/index.ts
bun build ./src/a.ts ./src/b.ts  # Multiple entries

# Output
--outdir <path>     # Output directory
--outfile <path>    # Single output file (for --compile)

# Target environment
--target browser    # Default, for web browsers
--target bun        # For Bun runtime
--target node       # For Node.js

# Format
--format esm        # ES modules (default)
--format cjs        # CommonJS
--format iife       # Immediately-invoked function

# Optimization
--minify            # Minify output
--minify-whitespace # Minify whitespace only
--minify-syntax     # Minify syntax only
--minify-identifiers # Minify identifiers only

# Source maps
--sourcemap         # Inline source map
--sourcemap=external # Separate .map file
--sourcemap=linked  # Link to external map

# Other
--splitting         # Enable code splitting
--watch             # Watch mode
--define KEY=VALUE  # Define compile-time constants
--external <pkg>    # Exclude package from bundle
--packages external # All node_modules external
```

## Bun.build() API

```typescript
const result = await Bun.build({
  entrypoints: ["./src/index.ts"],
  outdir: "./dist",
  target: "browser", // "browser" | "bun" | "node"
  format: "esm",     // "esm" | "cjs" | "iife"
  splitting: true,
  sourcemap: "external",
  minify: true,

  // Externals
  external: ["react", "react-dom"],
  packages: "external", // All node_modules

  // Naming
  naming: {
    entry: "[name].[hash].js",
    chunk: "[name]-[hash].js",
    asset: "[name]-[hash][ext]",
  },

  // Define constants
  define: {
    "process.env.NODE_ENV": JSON.stringify("production"),
  },

  // Plugins
  plugins: [myPlugin()],

  // Loaders
  loader: {
    ".svg": "file",
    ".txt": "text",
  },
});

// Check for errors
if (!result.success) {
  console.error("Build failed:");
  for (const log of result.logs) {
    console.error(log);
  }
  process.exit(1);
}

// Output info
for (const output of result.outputs) {
  console.log(`${output.path} (${output.size} bytes)`);
}
```

## Creating Executables

### Single-File Executable

```bash
# Basic executable
bun build --compile ./src/app.ts --outfile myapp

# With embedded assets
bun build --compile ./src/app.ts --outfile myapp \
  --define "import.meta.dir" "'./assets'"

# Cross-compile
bun build --compile --target=bun-linux-x64 ./app.ts
bun build --compile --target=bun-darwin-arm64 ./app.ts
bun build --compile --target=bun-windows-x64 ./app.ts
```

### Available Targets

| Target | Platform | Architecture |
|--------|----------|--------------|
| `bun-linux-x64` | Linux | x64 |
| `bun-linux-arm64` | Linux | ARM64 |
| `bun-darwin-x64` | macOS | x64 |
| `bun-darwin-arm64` | macOS | ARM64 (Apple Silicon) |
| `bun-windows-x64` | Windows | x64 |

### Embedding Files

```typescript
// Files in same directory as script are embedded
const data = await Bun.file(import.meta.dir + "/data.json").text();

// Reference files relative to entry point
import config from "./config.json";
```

```bash
# Build with embedded assets
bun build --compile ./src/app.ts --asset-dir ./assets
```

## Plugins

### Plugin Structure

```typescript
import { BunPlugin } from "bun";

const myPlugin: BunPlugin = {
  name: "my-plugin",
  setup(build) {
    // onResolve: Custom module resolution
    build.onResolve({ filter: /^virtual:.*/ }, (args) => {
      return {
        path: args.path,
        namespace: "virtual",
      };
    });

    // onLoad: Custom loading
    build.onLoad({ filter: /.*/, namespace: "virtual" }, (args) => {
      return {
        contents: `export default "Virtual module: ${args.path}"`,
        loader: "js",
      };
    });
  },
};

// Use in build
await Bun.build({
  entrypoints: ["./src/index.ts"],
  plugins: [myPlugin],
});
```

### YAML Plugin Example

```typescript
import { BunPlugin } from "bun";
import { parse } from "yaml";

const yamlPlugin: BunPlugin = {
  name: "yaml",
  setup(build) {
    build.onLoad({ filter: /\.ya?ml$/ }, async (args) => {
      const content = await Bun.file(args.path).text();
      const data = parse(content);
      return {
        contents: `export default ${JSON.stringify(data)}`,
        loader: "js",
      };
    });
  },
};
```

### SVG Component Plugin

```typescript
const svgPlugin: BunPlugin = {
  name: "svg-component",
  setup(build) {
    build.onLoad({ filter: /\.svg$/ }, async (args) => {
      const svg = await Bun.file(args.path).text();
      return {
        contents: `
          export default function SVG(props) {
            return (${svg.replace(/<svg/, "<svg {...props}")});
          }
        `,
        loader: "jsx",
      };
    });
  },
};
```

## Loaders

Built-in loaders for different file types:

| Loader | Extensions | Output |
|--------|------------|--------|
| `js` | .js, .mjs | JavaScript |
| `jsx` | .jsx | JSX → JavaScript |
| `ts` | .ts, .mts | TypeScript → JavaScript |
| `tsx` | .tsx | TSX → JavaScript |
| `json` | .json | JSON → JS object |
| `toml` | .toml | TOML → JS object |
| `text` | Any | Raw text string |
| `file` | Any | URL to asset |
| `napi` | .node | Native addon |
| `wasm` | .wasm | WebAssembly module |

```typescript
await Bun.build({
  entrypoints: ["./app.ts"],
  loader: {
    ".svg": "file",      // Copy and return URL
    ".txt": "text",      // Import as string
    ".html": "text",     // Import as string
  },
});
```

## Configuration (bunfig.toml)

```toml
# bunfig.toml

[build]
outdir = "./dist"
target = "browser"
format = "esm"
sourcemap = "external"
minify = true
splitting = true

[build.external]
packages = ["react", "react-dom"]

[build.define]
"process.env.NODE_ENV" = "'production'"

[build.loader]
".svg" = "file"
".txt" = "text"
```

## Common Build Patterns

### React SPA

```bash
bun build ./src/index.tsx --outdir ./dist \
  --minify \
  --splitting \
  --sourcemap=external \
  --define "process.env.NODE_ENV='production'"
```

### Node.js Library

```bash
bun build ./src/index.ts --outdir ./dist \
  --target node \
  --format esm \
  --packages external \
  --sourcemap
```

### CLI Tool

```bash
bun build --compile ./src/cli.ts --outfile mycli \
  --minify \
  --target bun-linux-x64
```

### Multi-Entry Build

```typescript
await Bun.build({
  entrypoints: [
    "./src/client.ts",
    "./src/worker.ts",
    "./src/admin.ts",
  ],
  outdir: "./dist",
  splitting: true,
  minify: true,
});
```

## Comparison with Alternatives

| Feature | Bun | esbuild | Vite | webpack |
|---------|-----|---------|------|---------|
| Speed | Fastest | Fast | Fast (dev) | Slow |
| TypeScript | Native | Native | Via plugin | Via loader |
| Executables | Yes | No | No | No |
| Dev server | Yes | No | Yes | Yes |
| Plugins | Simple | Simple | Rollup | Complex |
| Tree shaking | Yes | Yes | Yes | Yes |
| Code splitting | Yes | Yes | Yes | Yes |

## Troubleshooting

**"Cannot resolve module"**
- Check the import path
- Use `--external` for packages not to bundle
- Add to `external` array

**Large bundle size:**
- Enable tree shaking (default)
- Use `--minify`
- Check for side-effect imports
- Use `--packages external` for libraries

**Source maps not working:**
- Use `--sourcemap=external` for production
- Ensure source maps enabled in browser devtools

**Plugin not loading files:**
- Check filter regex pattern
- Ensure namespace matches between onResolve and onLoad
- Debug with console.log in plugin

**Executable too large:**
- Bun runtime is embedded (~90MB base)
- Use `--minify`
- Consider shipping as regular script for smaller size
