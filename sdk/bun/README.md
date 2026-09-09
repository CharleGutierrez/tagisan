# @tagisan/sdk

Official Bun & TypeScript SDK for **Tagisan (🇵🇭 Tagisan ng Talino)** — The high-performance Multi-LLM Collaboration, Adversarial Debate & DAG Workflow Engine in Rust.

## Features

- **🥟 Bun Native**: Sub-10ms startup, zero-config TypeScript execution, top-level await, native bundler and test runner integration.
- **⚔️ TagisanDebate**: Dialectical thesis vs. antithesis multi-round debate with impartial Lakandiwa synthesis.
- **🕸️ TagisanDag**: Dynamic multi-agent DAG task graph execution with cycle detection, topological sorting, and parallel wave scheduling.
- **🤖 TagisanAgent**: Autonomous multi-turn agent with tool registry, system personas, and conversational history.
- **🛡️ AgentShield Integration**: End-to-end security screening preventing fork bombs, credential exfiltration, and destructive filesystem calls.

---

## Installation

```bash
bun add @tagisan/sdk
```

---

## Quick Start

### 1. Initialize Client

```typescript
import { TagisanClient } from "@tagisan/sdk";

const client = new TagisanClient({
  maxBudgetUsd: 5.00,
  defaultProvider: "auto",
});

// Evaluate TypeScript code directly via Tagisan Bun Runtime
const evalResult = await client.evalBun(`
  interface User { id: number; name: string }
  const u: User = { id: 1, name: "Alice" };
  console.log("User:", JSON.stringify(u));
`);
console.log(evalResult.stdout);
```

---

### 2. Dialectical Adversarial Debate (`TagisanDebate`)

```typescript
const debate = client
  .createDebate("Rust vs Go for High-Throughput Microservices", { rounds: 2 })
  .setThesisModel("claude-3-5-sonnet")
  .setAntithesisModel("gpt-4o")
  .setLakandiwaModel("gemini-2.0-flash");

const result = await debate.execute();
console.log("Lakandiwa Synthesis:\n", result.lakandiwaSynthesis);
```

---

### 3. Multi-Agent DAG Workflow (`TagisanDag`)

```typescript
const dag = client.createDag();

dag.addTask({
  id: "fetch_data",
  description: "Fetch market statistics",
  dependencies: [],
  handler: () => ({ revenue: 1000000, margin: 0.25 }),
});

dag.addTask({
  id: "compute_profit",
  description: "Calculate net profit",
  dependencies: ["fetch_data"],
  handler: (inputs) => {
    const data = inputs["fetch_data"] as { revenue: number; margin: number };
    return data.revenue * data.margin;
  },
});

dag.addTask({
  id: "generate_report",
  description: "Produce financial summary",
  dependencies: ["compute_profit"],
  handler: (inputs) => {
    const profit = inputs["compute_profit"];
    return `Q3 Net Profit: $${profit.toLocaleString()}`;
  },
});

const execution = await dag.execute();
console.log("DAG Execution Result:", execution.tasks.get("generate_report")?.output);
```

---

### 4. Autonomous Agent with Custom Tools (`TagisanAgent`)

```typescript
const agent = client.createAgent({
  name: "InfraEngineer",
  persona: "sre-engineer",
  systemPrompt: "You are a Principal Reliability Engineer.",
});

agent.registerTool(
  {
    name: "inspect_pod_health",
    description: "Check status of k8s pods",
    parameters: { type: "object" },
  },
  async () => "All 48 pods healthy in production-us-east-1."
);

const response = await agent.run("Please inspect_pod_health and report.");
console.log(response.finalAnswer);
```

---

## Testing

Run unit and integration tests using Bun's built-in fast test runner:

```bash
bun test
```
