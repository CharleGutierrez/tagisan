# Example 1: Quick Start with Bun

Get started with Bun in under 2 minutes.

## Check Installation

```bash
bun --version
```

If not installed, see [cookbook/installation.md](../cookbook/installation.md).

## Create a Simple Script

```bash
# Create file
echo 'console.log("Hello from Bun!")' > hello.ts

# Run it
bun run hello.ts
# Output: Hello from Bun!
```

## TypeScript Just Works

```typescript
// greet.ts
interface User {
  name: string;
  age: number;
}

function greet(user: User): string {
  return `Hello, ${user.name}! You are ${user.age} years old.`;
}

const user: User = { name: "Alice", age: 30 };
console.log(greet(user));
```

```bash
bun run greet.ts
# Output: Hello, Alice! You are 30 years old.
```

## Quick HTTP Server

```typescript
// server.ts
Bun.serve({
  port: 3000,
  fetch(req) {
    const url = new URL(req.url);

    if (url.pathname === "/") {
      return new Response("Welcome to Bun!");
    }

    if (url.pathname === "/api/hello") {
      return Response.json({ message: "Hello, API!" });
    }

    return new Response("Not Found", { status: 404 });
  },
});

console.log("Server running at http://localhost:3000");
```

```bash
bun run server.ts
# Server running at http://localhost:3000
```

## One-Liner Evaluation

```bash
# Quick calculations
bun -e "console.log(Math.PI * 10 ** 2)"

# JSON processing
bun -e "console.log(JSON.stringify({a:1,b:2}, null, 2))"

# Fetch data
bun -e "fetch('https://api.github.com').then(r=>r.json()).then(console.log)"
```

## Initialize a Project

```bash
mkdir my-project && cd my-project
bun init

# Creates:
# - package.json
# - tsconfig.json
# - index.ts
# - .gitignore
# - README.md
```

## Next Steps

- Add packages: `bun add express zod`
- Run tests: `bun test`
- Build: `bun build ./src/index.ts --outdir ./dist`
