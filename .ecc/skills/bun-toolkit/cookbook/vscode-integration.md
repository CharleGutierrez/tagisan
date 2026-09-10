# Bun VS Code Integration

Set up VS Code for optimal Bun development experience.

## Install Extension

1. Open VS Code Extensions (Ctrl+Shift+X)
2. Search for "Bun for Visual Studio Code"
3. Install the official extension by Oven

Or via command line:
```bash
code --install-extension oven.bun-vscode
```

## Features

### Debugger
- Set breakpoints in TypeScript/JavaScript
- Step through code
- Inspect variables
- Debug tests

### Test Runner Integration
- Run tests from VS Code Test Explorer
- Click to run individual tests
- View test results inline

### Lock File Viewer
- View `bun.lockb` as human-readable text
- Inspect package versions

## Debug Configuration

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "bun",
      "request": "launch",
      "name": "Debug Bun Script",
      "program": "${file}",
      "cwd": "${workspaceFolder}",
      "stopOnEntry": false
    },
    {
      "type": "bun",
      "request": "launch",
      "name": "Debug Bun Tests",
      "program": "${workspaceFolder}/node_modules/bun/bin/bun",
      "args": ["test"],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "bun",
      "request": "attach",
      "name": "Attach to Bun",
      "url": "ws://localhost:6499/"
    }
  ]
}
```

## Debug via CLI

```bash
# Start with debugger waiting for connection
bun --inspect-wait run script.ts

# Start with debugger, break on first line
bun --inspect-brk run script.ts

# Start with debugger on specific port
bun --inspect=0.0.0.0:6499 run script.ts
```

## Tasks Configuration

Create `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Bun: Install",
      "type": "shell",
      "command": "bun install",
      "group": "build"
    },
    {
      "label": "Bun: Dev",
      "type": "shell",
      "command": "bun --hot run src/index.ts",
      "isBackground": true,
      "problemMatcher": []
    },
    {
      "label": "Bun: Build",
      "type": "shell",
      "command": "bun build src/index.ts --outdir dist --minify",
      "group": {
        "kind": "build",
        "isDefault": true
      }
    },
    {
      "label": "Bun: Test",
      "type": "shell",
      "command": "bun test",
      "group": {
        "kind": "test",
        "isDefault": true
      }
    },
    {
      "label": "Bun: Test Watch",
      "type": "shell",
      "command": "bun test --watch",
      "isBackground": true,
      "problemMatcher": []
    }
  ]
}
```

## Settings

Add to `.vscode/settings.json`:

```json
{
  // Use Bun as default runtime
  "bun.runtime": "bun",

  // TypeScript settings for Bun
  "typescript.tsdk": "node_modules/typescript/lib",

  // Format on save
  "editor.formatOnSave": true,

  // Use Bun for running scripts
  "npm.packageManager": "bun",

  // Terminal default
  "terminal.integrated.defaultProfile.windows": "PowerShell",

  // Exclude from search
  "search.exclude": {
    "**/node_modules": true,
    "**/.git": true,
    "**/dist": true,
    "bun.lockb": true
  }
}
```

## TypeScript Configuration

Create/update `tsconfig.json` for Bun:

```json
{
  "compilerOptions": {
    "lib": ["ESNext"],
    "module": "ESNext",
    "target": "ESNext",
    "moduleResolution": "bundler",
    "moduleDetection": "force",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "composite": true,
    "strict": true,
    "downlevelIteration": true,
    "skipLibCheck": true,
    "jsx": "react-jsx",
    "allowSyntheticDefaultImports": true,
    "forceConsistentCasingInFileNames": true,
    "allowJs": true,
    "types": ["bun-types"]
  }
}
```

Install Bun types:
```bash
bun add -d bun-types
```

## Keyboard Shortcuts

Add to `keybindings.json`:

```json
[
  {
    "key": "ctrl+shift+b",
    "command": "workbench.action.tasks.runTask",
    "args": "Bun: Build"
  },
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "Bun: Test"
  },
  {
    "key": "f5",
    "command": "workbench.action.debug.start",
    "when": "!inDebugMode"
  }
]
```

## Snippets

Create `.vscode/bun.code-snippets`:

```json
{
  "Bun Server": {
    "prefix": "bunserve",
    "body": [
      "Bun.serve({",
      "  port: ${1:3000},",
      "  fetch(req) {",
      "    return new Response(${2:\"Hello!\"});",
      "  },",
      "});",
      "",
      "console.log(\"Server running at http://localhost:${1:3000}\");"
    ]
  },
  "Bun Test": {
    "prefix": "buntest",
    "body": [
      "import { describe, it, expect } from \"bun:test\";",
      "",
      "describe(\"${1:suite}\", () => {",
      "  it(\"${2:should work}\", () => {",
      "    expect(${3:true}).toBe(true);",
      "  });",
      "});"
    ]
  },
  "Bun File Read": {
    "prefix": "bunfile",
    "body": [
      "const file = Bun.file(\"${1:./path}\");",
      "const content = await file.${2|text,json,arrayBuffer|}();"
    ]
  }
}
```

## Recommended Extensions

- **Bun for Visual Studio Code** - Official Bun extension
- **Error Lens** - Inline error display
- **Pretty TypeScript Errors** - Better error messages
- **Thunder Client** - API testing (for testing Bun servers)

## Troubleshooting

**Debugger not connecting:**
- Ensure Bun extension is installed
- Check debug port is not in use
- Try `bun --inspect-wait` to wait for debugger

**TypeScript errors:**
- Install `bun-types`: `bun add -d bun-types`
- Add to tsconfig.json types array

**Test Explorer not finding tests:**
- Ensure files match `*.test.ts` pattern
- Reload VS Code window
