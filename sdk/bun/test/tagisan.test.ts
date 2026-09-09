import { describe, test, expect } from "bun:test";
import {
  TagisanClient,
  TagisanDebate,
  TagisanDag,
  TagisanAgent,
  TagisanSqliteMemory,
  TagisanMcpServer,
  TagisanFFI,
  type ToolDefinition,
} from "../index";

describe("TagisanClient", () => {
  test("initializes with default options", () => {
    const client = new TagisanClient();
    expect(client.options.maxBudgetUsd).toBe(5.0);
    expect(client.options.defaultProvider).toBe("auto");
    expect(client.options.defaultModel).toBe("auto");
    expect(client.options.timeoutMs).toBe(60000);
  });

  test("initializes with custom options", () => {
    const client = new TagisanClient({
      maxBudgetUsd: 10.0,
      defaultProvider: "anthropic",
      defaultModel: "claude-3-5-sonnet",
      timeoutMs: 15000,
    });
    expect(client.options.maxBudgetUsd).toBe(10.0);
    expect(client.options.defaultProvider).toBe("anthropic");
    expect(client.options.defaultModel).toBe("claude-3-5-sonnet");
    expect(client.options.timeoutMs).toBe(15000);
  });

  test("evaluates TypeScript via evalBun with math and await", async () => {
    const client = new TagisanClient();
    const code = `
      interface Vector2D { x: number; y: number; }
      const v: Vector2D = { x: 3, y: 4 };
      const mag = await Promise.resolve(Math.sqrt(v.x * v.x + v.y * v.y));
      console.log('Magnitude:', mag);
    `;
    const res = await client.evalBun(code);
    expect(res.exitCode).toBe(0);
    expect(res.stdout).toContain("Magnitude: 5");
  });

  test("returns version string", async () => {
    const client = new TagisanClient();
    const ver = await client.version();
    expect(ver.length).toBeGreaterThan(0);
  });
});

describe("TagisanDebate", () => {
  test("configures debate rounds and models", () => {
    const client = new TagisanClient();
    const debate = client
      .createDebate("Rust vs Go for Microservices", { rounds: 3 })
      .setThesisModel("claude-3-5-sonnet")
      .setAntithesisModel("gpt-4o")
      .setLakandiwaModel("gemini-2.0-flash");

    expect(debate.getTopic()).toBe("Rust vs Go for Microservices");
    expect(debate.getRounds()).toBe(3);
  });

  test("executes multi-round dialectical debate", async () => {
    const client = new TagisanClient();
    const debate = client.createDebate("Microservices vs Monoliths", { rounds: 2 });
    const result = await debate.execute();

    expect(result.topic).toBe("Microservices vs Monoliths");
    expect(result.rounds).toBe(2);
    expect(result.turns.length).toBeGreaterThanOrEqual(5); // 2 thesis + 2 antithesis + 1 lakandiwa
    expect(result.lakandiwaSynthesis).toContain("Lakandiwa Synthesis");
    expect(result.totalDurationMs).toBeGreaterThanOrEqual(0);
  });
});

describe("TagisanDag", () => {
  test("registers tasks and validates graph", () => {
    const client = new TagisanClient();
    const dag = client.createDag();

    dag.addTask({
      id: "fetch_data",
      description: "Fetch market data",
      dependencies: [],
    });
    dag.addTask({
      id: "analyze_risk",
      description: "Analyze market risk",
      dependencies: ["fetch_data"],
    });
    dag.addTask({
      id: "generate_report",
      description: "Generate final executive report",
      dependencies: ["analyze_risk"],
    });

    expect(dag.taskCount()).toBe(3);
    const validation = dag.validateGraph();
    expect(validation.valid).toBe(true);
    expect(validation.errors.length).toBe(0);
  });

  test("detects missing dependencies", () => {
    const client = new TagisanClient();
    const dag = client.createDag();

    dag.addTask({
      id: "task_b",
      description: "Task B",
      dependencies: ["non_existent_a"],
    });

    const validation = dag.validateGraph();
    expect(validation.valid).toBe(false);
    expect(validation.errors[0]).toContain("non_existent_a");
  });

  test("detects circular dependencies", () => {
    const client = new TagisanClient();
    const dag = client.createDag();

    dag.addTask({
      id: "task_1",
      description: "Task 1",
      dependencies: ["task_2"],
    });
    dag.addTask({
      id: "task_2",
      description: "Task 2",
      dependencies: ["task_1"],
    });

    const validation = dag.validateGraph();
    expect(validation.valid).toBe(false);
    expect(validation.cycle).toBeDefined();
    expect(validation.errors.some((e) => e.includes("Circular dependency"))).toBe(true);
  });

  test("computes topological sort and execution waves", () => {
    const client = new TagisanClient();
    const dag = client.createDag();

    // Diamond dependency:
    //      start
    //     /     \
    //   left   right
    //     \     /
    //       end
    dag.addTask({ id: "start", description: "Root", dependencies: [] });
    dag.addTask({ id: "left", description: "Left branch", dependencies: ["start"] });
    dag.addTask({ id: "right", description: "Right branch", dependencies: ["start"] });
    dag.addTask({ id: "end", description: "Merge", dependencies: ["left", "right"] });

    const order = dag.topologicalSort();
    expect(order.indexOf("start")).toBeLessThan(order.indexOf("left"));
    expect(order.indexOf("start")).toBeLessThan(order.indexOf("right"));
    expect(order.indexOf("left")).toBeLessThan(order.indexOf("end"));
    expect(order.indexOf("right")).toBeLessThan(order.indexOf("end"));

    const waves = dag.getExecutionWaves();
    expect(waves.length).toBe(3);
    expect(waves[0]).toEqual(["start"]);
    expect(waves[1].sort()).toEqual(["left", "right"].sort());
    expect(waves[2]).toEqual(["end"]);
  });

  test("executes DAG with task handlers and output propagation", async () => {
    const client = new TagisanClient();
    const dag = client.createDag();

    dag.addTask({
      id: "step1",
      description: "Compute base number",
      dependencies: [],
      handler: () => 10,
    });

    dag.addTask({
      id: "step2_double",
      description: "Double the input",
      dependencies: ["step1"],
      handler: (inputs) => (inputs["step1"] as number) * 2,
    });

    dag.addTask({
      id: "step3_add",
      description: "Add 5 to doubled value",
      dependencies: ["step2_double"],
      handler: (inputs) => (inputs["step2_double"] as number) + 5,
    });

    const res = await dag.execute();
    expect(res.success).toBe(true);
    expect(res.errors.length).toBe(0);
    expect(res.tasks.get("step1")?.output).toBe(10);
    expect(res.tasks.get("step2_double")?.output).toBe(20);
    expect(res.tasks.get("step3_add")?.output).toBe(25);
  });
});

describe("TagisanAgent", () => {
  test("registers tools and updates persona", () => {
    const client = new TagisanClient();
    const agent = client.createAgent({
      name: "SecurityAuditor",
      persona: "security-auditor",
      systemPrompt: "You are an elite cybersecurity specialist.",
    });

    const calcTool: ToolDefinition = {
      name: "calculator",
      description: "Evaluates mathematical expressions",
      parameters: { type: "object" },
    };

    agent.registerTool(calcTool, async (args) => {
      return `Computed result: ${JSON.stringify(args)}`;
    });

    expect(agent.toolCount()).toBe(1);
    expect(agent.getTool("calculator")).toBeDefined();
    expect(agent.options.persona).toBe("security-auditor");

    agent.setPersona("architect");
    expect(agent.options.persona).toBe("architect");
  });

  test("executes agent turn with tool invocation", async () => {
    const client = new TagisanClient();
    const agent = client.createAgent({
      name: "DevOpsAgent",
      persona: "sre-engineer",
    });

    const statusTool: ToolDefinition = {
      name: "cluster_health",
      description: "Checks k8s cluster health",
      parameters: {},
    };

    agent.registerTool(statusTool, async () => {
      return "Cluster status: 100% healthy, 0 alerts.";
    });

    const result = await agent.run("Please check cluster_health immediately");
    expect(result.completed).toBe(true);
    expect(result.finalAnswer).toContain("Cluster status: 100% healthy");
    expect(result.steps.length).toBeGreaterThan(0);
    expect(agent.getHistory().length).toBeGreaterThanOrEqual(2);
  });
});

describe("TagisanSqliteMemory", () => {
  test("inserts vectors and computes cosine similarity in bun:sqlite", () => {
    const memory = new TagisanSqliteMemory(":memory:");

    const v1 = new Float32Array([1.0, 0.0, 0.0]);
    const v2 = new Float32Array([0.0, 1.0, 0.0]);
    const v3 = new Float32Array([0.7071, 0.7071, 0.0]);

    memory.insert("doc-x", "Vector pointing east", v1, { axis: "x" });
    memory.insert("doc-y", "Vector pointing north", v2, { axis: "y" });
    memory.insert("doc-xy", "Vector pointing northeast", v3, { axis: "xy" });

    expect(memory.count()).toBe(3);

    // Query vector pointing east
    const query = new Float32Array([1.0, 0.0, 0.0]);
    const matches = memory.searchCosine(query, 2, 0.5);

    expect(matches.length).toBe(2);
    expect(matches[0].id).toBe("doc-x");
    expect(matches[0].score).toBeCloseTo(1.0, 4);
    expect(matches[1].id).toBe("doc-xy");
    expect(matches[1].score).toBeCloseTo(0.7071, 3);

    memory.close();
  });
});

describe("TagisanMcpServer", () => {
  test("implements JSON-RPC 2.0 MCP protocol methods", async () => {
    const server = new TagisanMcpServer("test-mcp-server", "1.0.0");

    server.registerTool({
      name: "echo",
      description: "Echoes input text",
      inputSchema: { type: "object" },
      handler: (args) => `Echo: ${args.message}`,
    });

    // 1. Initialize
    const initReq = {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {},
    };
    const initRes: any = await server.handleMessage(initReq);
    expect(initRes.id).toBe(1);
    expect(initRes.result.serverInfo.name).toBe("test-mcp-server");
    expect(initRes.result.capabilities.tools).toBeDefined();

    // 2. Tools list
    const listReq = {
      jsonrpc: "2.0",
      id: 2,
      method: "tools/list",
      params: {},
    };
    const listRes: any = await server.handleMessage(listReq);
    expect(listRes.id).toBe(2);
    expect(listRes.result.tools.length).toBe(1);
    expect(listRes.result.tools[0].name).toBe("echo");

    // 3. Tool call
    const callReq = {
      jsonrpc: "2.0",
      id: 3,
      method: "tools/call",
      params: {
        name: "echo",
        arguments: { message: "Hello MCP from Bun!" },
      },
    };
    const callRes: any = await server.handleMessage(callReq);
    expect(callRes.id).toBe(3);
    expect(callRes.result.content[0].text).toBe("Echo: Hello MCP from Bun!");

    // 4. Ping
    const pingReq = { jsonrpc: "2.0", id: 4, method: "ping" };
    const pingRes: any = await server.handleMessage(pingReq);
    expect(pingRes.id).toBe(4);
    expect(pingRes.result).toEqual({});

    // 5. Unknown method error
    const errReq = { jsonrpc: "2.0", id: 5, method: "non_existent_method" };
    const errRes: any = await server.handleMessage(errReq);
    expect(errRes.id).toBe(5);
    expect(errRes.error.code).toBe(-32601);
  });
});

describe("TagisanFFI", () => {
  test("instantiates and validates FFI bindings when library is present", () => {
    const fs = require("fs");
    const path = require("path");
    const soPath = path.resolve(__dirname, "../../../target/debug/libtagisan.so");

    if (fs.existsSync(soPath)) {
      const ffi = new TagisanFFI(soPath);
      expect(ffi.version()).toContain("tagisan-0.1.0-bun-native");

      const v1 = new Float32Array([1.0, 0.0, 0.0]);
      const v2 = new Float32Array([1.0, 0.0, 0.0]);
      const sim = ffi.cosineSimilarity(v1, v2);
      expect(sim).toBeCloseTo(1.0, 4);

      const digest = ffi.blake3Digest("tagisan-bun-native");
      expect(digest.length).toBe(64);

      expect(ffi.shieldScan("console.log('clean code');")).toBe(true);
      expect(ffi.shieldScan("const x = ':(){ :|:& };:';")).toBe(false);
    } else {
      console.log("Note: libtagisan.so not compiled yet, skipping binary FFI test until cargo build.");
    }
  });
});

