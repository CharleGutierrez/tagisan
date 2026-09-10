# Bun Runtime APIs

Comprehensive guide to Bun's built-in APIs for high-performance JavaScript.

## Bun.serve() - HTTP Server

```typescript
const server = Bun.serve({
  port: 3000,
  hostname: "0.0.0.0",

  // Main request handler
  fetch(req, server) {
    const url = new URL(req.url);

    if (url.pathname === "/") {
      return new Response("Hello!");
    }

    if (url.pathname === "/json") {
      return Response.json({ message: "Hello JSON!" });
    }

    return new Response("Not Found", { status: 404 });
  },

  // Error handler
  error(error) {
    return new Response(`Error: ${error.message}`, { status: 500 });
  },
});

console.log(`Server running at http://localhost:${server.port}`);
```

### WebSocket Support
```typescript
Bun.serve({
  port: 3000,
  fetch(req, server) {
    // Upgrade to WebSocket
    if (server.upgrade(req)) {
      return;  // Handled by WebSocket
    }
    return new Response("HTTP");
  },
  websocket: {
    open(ws) {
      console.log("Client connected");
    },
    message(ws, message) {
      ws.send(`Echo: ${message}`);
    },
    close(ws) {
      console.log("Client disconnected");
    },
  },
});
```

## Bun.file() - File Reading

```typescript
const file = Bun.file("./data.txt");

// File metadata
console.log(file.size);  // bytes
console.log(file.type);  // MIME type

// Read as different types
const text = await file.text();
const json = await file.json();
const buffer = await file.arrayBuffer();
const stream = file.stream();

// Check existence
const exists = await file.exists();
```

## Bun.write() - File Writing

```typescript
// Write string
await Bun.write("output.txt", "Hello World");

// Write JSON
await Bun.write("data.json", JSON.stringify({ key: "value" }));

// Write from Response
const res = await fetch("https://example.com/image.png");
await Bun.write("image.png", res);

// Write from BunFile
await Bun.write("copy.txt", Bun.file("original.txt"));

// Write with options
await Bun.write("output.txt", "content", {
  mode: 0o644,  // File permissions
});
```

## Bun.spawn() - Subprocess

```typescript
// Simple spawn
const proc = Bun.spawn(["ls", "-la"]);
const output = await new Response(proc.stdout).text();

// With options
const proc2 = Bun.spawn(["node", "script.js"], {
  cwd: "./project",
  env: { NODE_ENV: "production" },
  stdout: "pipe",
  stderr: "pipe",
});

// Wait for completion
const exitCode = await proc2.exited;

// Kill process
proc.kill();
```

### Synchronous Spawn
```typescript
const { stdout, stderr, exitCode } = Bun.spawnSync(["echo", "hello"]);
console.log(stdout.toString());
```

## bun:sqlite - SQLite Database

```typescript
import { Database } from "bun:sqlite";

// Open/create database
const db = new Database("mydb.sqlite");
// Or in-memory: new Database(":memory:")

// Create table
db.run(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE
  )
`);

// Insert
const insert = db.prepare("INSERT INTO users (name, email) VALUES (?, ?)");
insert.run("Alice", "alice@example.com");

// Query
const query = db.prepare("SELECT * FROM users WHERE name = ?");
const user = query.get("Alice");
const allUsers = db.query("SELECT * FROM users").all();

// Transaction
const insertMany = db.transaction((users) => {
  for (const user of users) {
    insert.run(user.name, user.email);
  }
});
insertMany([
  { name: "Bob", email: "bob@example.com" },
  { name: "Charlie", email: "charlie@example.com" },
]);

// Close
db.close();
```

## Bun.password - Password Hashing

```typescript
// Hash password (uses Argon2id by default)
const hash = await Bun.password.hash("mypassword");

// Verify password
const isValid = await Bun.password.verify("mypassword", hash);

// With bcrypt
const bcryptHash = await Bun.password.hash("password", {
  algorithm: "bcrypt",
  cost: 12,
});
```

## Bun.sleep() - Async Sleep

```typescript
// Sleep for 1 second
await Bun.sleep(1000);

// Sleep with Date
await Bun.sleepSync(500);  // Synchronous version
```

## Bun.env - Environment Variables

```typescript
// Read environment variable
const apiKey = Bun.env.API_KEY;

// With default
const port = Bun.env.PORT ?? "3000";

// All env vars
console.log(Bun.env);
```

## Bun.Glob - File Matching

```typescript
const glob = new Bun.Glob("**/*.ts");

// Scan directory
for await (const file of glob.scan(".")) {
  console.log(file);
}

// Match string
glob.match("src/index.ts");  // true
```

## HTMLRewriter - HTML Transformation

```typescript
const html = "<html><body><h1>Hello</h1></body></html>";

const rewriter = new HTMLRewriter()
  .on("h1", {
    element(el) {
      el.setInnerContent("Modified!");
    },
  });

const result = rewriter.transform(new Response(html));
const output = await result.text();
```

## bun:ffi - Foreign Function Interface

```typescript
import { dlopen, FFIType, ptr } from "bun:ffi";

// Load C library
const lib = dlopen("./mylib.so", {
  add: {
    args: [FFIType.i32, FFIType.i32],
    returns: FFIType.i32,
  },
});

// Call function
const result = lib.symbols.add(2, 3);  // 5
```

## Performance APIs

```typescript
// High-resolution timer
const start = Bun.nanoseconds();
// ... work ...
const elapsed = Bun.nanoseconds() - start;
console.log(`Took ${elapsed / 1e6}ms`);

// Memory info
console.log(Bun.main);  // Entry point path
console.log(Bun.version);  // Bun version
console.log(Bun.revision);  // Git commit
```

## Streaming

```typescript
// Create readable stream
const stream = new ReadableStream({
  start(controller) {
    controller.enqueue("Hello ");
    controller.enqueue("World!");
    controller.close();
  },
});

// Pipe streams
const response = await fetch("https://example.com");
await Bun.write("file.html", response);  // Streams automatically
```
