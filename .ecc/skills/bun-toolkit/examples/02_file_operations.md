# Example 2: File Operations with Bun

Bun provides fast, ergonomic APIs for file I/O.

## Reading Files

```typescript
// read.ts

// Read as text
const textFile = Bun.file("./data.txt");
const text = await textFile.text();
console.log(text);

// Read as JSON
const jsonFile = Bun.file("./config.json");
const config = await jsonFile.json();
console.log(config);

// Read as ArrayBuffer (binary)
const binaryFile = Bun.file("./image.png");
const buffer = await binaryFile.arrayBuffer();
console.log(`Size: ${buffer.byteLength} bytes`);

// File metadata
const file = Bun.file("./package.json");
console.log(`Size: ${file.size} bytes`);
console.log(`Type: ${file.type}`);
console.log(`Exists: ${await file.exists()}`);
```

## Writing Files

```typescript
// write.ts

// Write string
await Bun.write("output.txt", "Hello, World!");

// Write JSON
const data = { name: "Bun", version: "1.0" };
await Bun.write("output.json", JSON.stringify(data, null, 2));

// Copy file
await Bun.write("copy.txt", Bun.file("original.txt"));

// Write from fetch response
const response = await fetch("https://example.com/data.json");
await Bun.write("downloaded.json", response);
```

## Streaming Large Files

```typescript
// stream.ts

// Stream reading
const file = Bun.file("large-file.txt");
const stream = file.stream();

for await (const chunk of stream) {
  // Process chunk (Uint8Array)
  console.log(`Received ${chunk.length} bytes`);
}

// Pipe to writer
const writer = Bun.file("output.txt").writer();
for await (const chunk of Bun.file("input.txt").stream()) {
  writer.write(chunk);
}
await writer.end();
```

## Glob Pattern Matching

```typescript
// glob.ts

const glob = new Bun.Glob("**/*.ts");

// Find all TypeScript files
for await (const file of glob.scan(".")) {
  console.log(file);
}

// Check if file matches pattern
console.log(glob.match("src/index.ts")); // true
console.log(glob.match("README.md"));    // false
```

## Practical Example: Process All JSON Files

```typescript
// process-json.ts

const glob = new Bun.Glob("data/**/*.json");

for await (const path of glob.scan(".")) {
  const file = Bun.file(path);
  const data = await file.json();

  // Add timestamp
  data.processedAt = new Date().toISOString();

  // Write back
  await Bun.write(path, JSON.stringify(data, null, 2));

  console.log(`Processed: ${path}`);
}
```

## Practical Example: Merge CSV Files

```typescript
// merge-csv.ts

const glob = new Bun.Glob("input/*.csv");
const lines: string[] = [];
let headerWritten = false;

for await (const path of glob.scan(".")) {
  const content = await Bun.file(path).text();
  const fileLines = content.trim().split("\n");

  if (!headerWritten) {
    lines.push(fileLines[0]); // Header from first file
    headerWritten = true;
  }

  // Add data rows (skip header)
  lines.push(...fileLines.slice(1));
}

await Bun.write("merged.csv", lines.join("\n"));
console.log(`Merged ${lines.length - 1} rows`);
```

## Running the Examples

```bash
bun run read.ts
bun run write.ts
bun run stream.ts
bun run glob.ts
bun run process-json.ts
bun run merge-csv.ts
```
