import { describe, test, expect } from "bun:test";
import { TagisanVellaClient } from "../index";

describe("TagisanVellaClient", () => {
  test("initializes and reports nominal governor status", async () => {
    const client = new TagisanVellaClient();
    const status = await client.getStatus();

    expect(status.eStopActive).toBe(false);
    expect(status.maxOrderValueUsd).toBe(1_000_000);
    expect(status.maxOrderSize).toBe(100_000);
    expect(status.maxLeverage).toBe(50.0);
    expect(status.maxRobotVelocityMs).toBe(30.0);
    expect(status.allowedScadaCoils).toEqual([0, 10_000]);
  });

  test("latches and clears physical E-Stop correctly", async () => {
    const client = new TagisanVellaClient();

    expect(client.isEStopActive()).toBe(false);

    const tripRes = await client.tripEStop("Thermal breach detected in reactor core");
    expect(tripRes.success).toBe(true);
    expect(tripRes.status).toBe("EMERGENCY_STOP_LATCHED");
    expect(client.isEStopActive()).toBe(true);

    const clearRes = await client.clearEStop("Cooling restored and verified");
    expect(clearRes.success).toBe(true);
    expect(clearRes.status).toBe("EMERGENCY_STOP_CLEARED");
    expect(client.isEStopActive()).toBe(false);

    const logs = client.getAuditLog();
    expect(logs.length).toBeGreaterThanOrEqual(2);
    expect(logs[0]).toContain("E-STOP TRIPPED");
    expect(logs[1]).toContain("E-STOP CLEARED");
  });

  test("executes trading orders within policy bounds and rejects violations", async () => {
    const client = new TagisanVellaClient();

    // Valid order
    const orderRes = await client.submitOrder({
      symbol: "AAPL",
      orderType: "bid",
      price: 150.0,
      size: 500,
      leverage: 10.0,
    });
    expect(orderRes.cleared).toBe(true);
    expect(orderRes.orderValue).toBe(75_000);

    // Violation: order value exceeds $1,000,000
    expect(
      client.submitOrder({
        symbol: "BTC",
        orderType: "bid",
        price: 70_000.0,
        size: 20, // $1,400,000
      })
    ).rejects.toThrow("exceeds max limit");

    // Violation: leverage exceeds 50x
    expect(
      client.submitOrder({
        symbol: "ETH",
        orderType: "bid",
        price: 2_500.0,
        size: 10,
        leverage: 100.0,
      })
    ).rejects.toThrow("Leverage 100x exceeds max limit");

    // Violation: E-stop active blocks trading
    await client.tripEStop("Emergency drill");
    expect(
      client.submitOrder({
        symbol: "AAPL",
        orderType: "bid",
        price: 150.0,
        size: 10,
      })
    ).rejects.toThrow("blocked while E-Stop is active");
    await client.clearEStop("Drill complete");
  });

  test("safely actuates SCADA coils within allowed register bounds", async () => {
    const client = new TagisanVellaClient();

    const res = await client.actuateCoil({
      coilAddress: 42,
      state: true,
    });
    expect(res.status).toBe("actuated");
    expect(res.coilAddress).toBe(42);
    expect(res.state).toBe(true);

    // Out-of-bounds coil address
    expect(
      client.actuateCoil({
        coilAddress: 99_999,
        state: true,
      })
    ).rejects.toThrow("outside allowed bounds");
  });

  test("validates drone kinematics and blocks excessive velocity", async () => {
    const client = new TagisanVellaClient();

    const normal = await client.validateMotion({ velocityMs: 15.0 });
    expect(normal.allowed).toBe(true);

    expect(client.validateMotion({ velocityMs: 45.0 })).rejects.toThrow("exceeds max limit");
  });

  test("simulates pharmaceutical molecular docking affinity", async () => {
    const client = new TagisanVellaClient();
    const sim = await client.simulateMolecularDocking({
      compoundSmiles: "CC(=O)OC1=CC=CC=C1C(=O)O",
      targetProtein: "SARS-CoV-2 Mpro",
    });

    expect(sim.status).toBe("simulated");
    expect(sim.affinity).toContain("94.2%");
    expect(sim.target).toBe("SARS-CoV-2 Mpro");
  });

  test("evaluates high-stakes domain action through adversarial debate and Borda consensus", async () => {
    const client = new TagisanVellaClient();

    // Safe proposal
    const approvedVerdict = await client.submitProposalForDebate({
      domain: "scada",
      actionType: "coolant_valve_override",
      target: "VALVE_COOLANT_MAIN",
      parameters: { safe: true, coilAddress: 10 },
    });

    expect(approvedVerdict.approved).toBe(true);
    expect(approvedVerdict.winningOption).toBe("EXECUTE_WITH_SAFETY_BOUNDS");
    expect(approvedVerdict.bordaPoints["EXECUTE_WITH_SAFETY_BOUNDS"]).toBeGreaterThan(
      approvedVerdict.bordaPoints["ABORT_ACTION"]
    );

    // Dangerous proposal
    const rejectedVerdict = await client.submitProposalForDebate({
      domain: "trading",
      actionType: "unhedged_flash_loan",
      target: "UNHEDGED_50M_SHORT",
      parameters: { safe: false },
    });

    expect(rejectedVerdict.approved).toBe(false);
    expect(rejectedVerdict.winningOption).toBe("ABORT_ACTION");
  });

  // =========================================================================
  // Phase 2 Deep-Systems Superpowers Test Suite
  // =========================================================================

  test("Phase 2 Superpower: VectorSync synchronizes memory and performs hybrid search", async () => {
    const client = new TagisanVellaClient();

    const pushRes = await client.syncVectors({ action: "push" });
    expect(pushRes.status).toBe("success");
    expect(pushRes.stats?.pushed).toBe(12);

    const searchRes = await client.syncVectors({
      action: "search",
      queryVector: [0.2, 0.4, 0.8],
      topK: 2,
    });
    expect(searchRes.status).toBe("success");
    expect(searchRes.hits).toHaveLength(2);
    expect(searchRes.hits![0].source).toBe("fused_vella");
    expect(searchRes.hits![0].score).toBeGreaterThan(0.9);
  });

  test("Phase 2 Superpower: DigitalTwin physical simulation blocks dangerous commands", async () => {
    const client = new TagisanVellaClient();

    // Safe simulation
    const safeScada = await client.simulateDigitalTwin({
      domain: "scada",
      scada: { ticks: 10, heatLoadSpike: 5 },
    });
    expect(safeScada.safeToExecute).toBe(true);
    expect(safeScada.safetyMarginPercent).toBeGreaterThan(50);

    // Destructive heat spike blocked before physical execution
    const dangerousScada = await client.simulateDigitalTwin({
      domain: "scada",
      scada: { heatLoadSpike: 100 },
    });
    expect(dangerousScada.safeToExecute).toBe(false);
    expect(dangerousScada.violation).toContain("Catastrophic boiler overpressure");

    // Kinematic violation blocked
    const dangerousRobot = await client.simulateDigitalTwin({
      domain: "robotics",
      robotics: { linearVelocityMps: 45.0 },
    });
    expect(dangerousRobot.safeToExecute).toBe(false);
    expect(dangerousRobot.violation).toContain("exceeds sovereign policy threshold");
  });

  test("Phase 2 Superpower: Web3 MPC Treasury Guardian enforces multi-agent consensus", async () => {
    const client = new TagisanVellaClient();

    const proposal = await client.proposeTreasuryTx({
      recipient: "0x71C836643F37e133a1e2B4913A17BAb38A3fA68c",
      amountEth: 5.25,
      purpose: "Critical Substation Maintenance Grant",
      threshold: 2,
    });

    expect(proposal.proposalId).toBeDefined();
    expect(proposal.canonicalHash).toContain("VELLA_MPC_TREASURY_TX");

    const sig1 = await client.coSignTreasuryTx(proposal.proposalId, "proposer");
    const sig2 = await client.coSignTreasuryTx(proposal.proposalId, "auditor");

    expect(sig1.signed).toBe(true);
    expect(sig2.signed).toBe(true);
    expect(sig1.role).toBe("proposer");
    expect(sig2.role).toBe("auditor");

    // Unmet threshold rejects
    expect(client.executeTreasuryTx(proposal.proposalId, 1, 2)).rejects.toThrow("Threshold unmet");

    // Satisfied threshold executes
    const execution = await client.executeTreasuryTx(proposal.proposalId, 2, 2);
    expect(execution.executed).toBe(true);
    expect(execution.txHash).toContain(proposal.proposalId);
  });

  test("Phase 2 Superpower: FHE Privacy Shield performs zero-knowledge clinical evaluation", async () => {
    const client = new TagisanVellaClient();

    const rawBiomarker = 12; // Private patient marker
    const fheResult = await client.evaluateFheRisk(rawBiomarker);

    expect(fheResult.zeroKnowledge).toBe(true);
    expect(fheResult.inputEncrypted).toBe(true);
    // (12 * 3) + 5 = 41
    expect(fheResult.decryptedScore).toBe(41);
    expect(fheResult.model).toContain("ciphertext space");
  });

  test("Phase 2 Superpower: SpaceCopilot propagates orbits and plans conjunction maneuvers", async () => {
    const client = new TagisanVellaClient();

    const orbit = await client.propagateOrbit(
      "1 25544U 98067A   20343.51863588  .00001556  00000-0  36025-4 0  9993",
      "2 25544  51.6447  30.4192 0001476  85.5927 274.5714 15.49185209258812",
      30.0
    );
    expect(orbit.eciCoordinates.altitudeKm).toBeGreaterThan(300);
    expect(orbit.orbitalVelocityKmS).toBeGreaterThan(7.0);

    const conjunction = await client.assessConjunction(
      ["tle1", "tle2"],
      ["debris1", "debris2"]
    );
    expect(conjunction.collisionAlert).toBe(true);
    expect(conjunction.missDistanceKm).toBeLessThan(5.0);

    const plan = await client.planAvoidanceManeuver(conjunction.missDistanceKm, 15.0);
    expect(plan.deltaVTotalMps).toBeGreaterThan(0.5);
    expect(plan.propellantExpenditureKg).toBeGreaterThan(0.2);
    expect(plan.status).toBe("MANEUVER_OPTIMIZED_AND_LOCKED");
  });

  test("Phase 2 Superpower: API Scaffolder generates Bun servers and self-heals schema drift", async () => {
    const client = new TagisanVellaClient();

    const scaffold = await client.scaffoldApiServer(["agent_executions", "scada_telemetry"], 4000);
    expect(scaffold.generatedCodeLength).toBeGreaterThan(1000);
    expect(scaffold.modelsScaffolded).toContain("agent_executions");
    expect(scaffold.bunEntryPoint).toContain("export default");

    const healed = await client.selfHealApiServer(
      "export interface agent_executions { id: string; }",
      ["agent_id", "status"]
    );
    expect(healed.detectedDrift).toBe(true);
    expect(healed.missingFields).toContain("agent_id");
    expect(healed.healed).toBe(true);
  });

  test("Phase 2 Superpower: CyberDefense drill neutralizes 100% of attack vectors", async () => {
    const client = new TagisanVellaClient();

    const drill = await client.runCyberDefenseDrill();
    expect(drill.vectorsTested).toBe(6);
    expect(drill.vectorsNeutralized).toBe(6);
    expect(drill.neutralizationRatePercent).toBe(100.0);
    expect(drill.postureGrade).toContain("A+ SOVEREIGN SHIELD");
  });
});
