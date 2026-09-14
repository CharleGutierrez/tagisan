import gleeunit
import gleeunit/should
import tagisan
import tagisan/agent
import tagisan/debate
import tagisan/mcp
import tagisan/protocol

pub fn main() {
  gleeunit.main()
}

pub fn version_test() {
  tagisan.get_version()
  |> should.equal("0.2.0")
}

pub fn debate_state_machine_full_lifecycle_test() {
  let initial = debate.Thesis(
    proposer: "alpha_architect",
    claim: "Zero-copy Actor Channels optimize memory",
    reasoning: "Eliminates cache thrashing and lock contention",
  )

  // Step 1: Audit -> Antithesis
  let audit_action = debate.Audit(
    auditor: "security_auditor",
    critique: "Potential race condition under buffer wrap-around",
    risk_score: 0.35,
  )
  let assert Ok(step1) = debate.transition(initial, audit_action)

  // Step 2: Synthesize -> Synthesis
  let synth_action = debate.Synthesize(
    judge: "chief_judge",
    resolution: "Apply monotonic ring indices to eliminate wrap-around races",
    borda_points: 98,
  )
  let assert Ok(step2) = debate.transition(step1, synth_action)

  // Step 3: Adjudicate -> Verdict
  let adjudicate_action = debate.Adjudicate(
    approved: True,
    reason: "Formally proven and safe for production deployment",
    timestamp: 1726300000,
  )
  let assert Ok(step3) = debate.transition(step2, adjudicate_action)

  debate.is_approved(step3)
  |> should.equal(True)

  // Step 4: Further transition must fail on terminal verdict
  debate.transition(step3, audit_action)
  |> should.be_error()
}

pub fn debate_illegal_transition_test() {
  let initial = debate.Thesis(
    proposer: "alpha",
    claim: "Direct jump to adjudication",
    reasoning: "Skip audit",
  )

  let illegal_action = debate.Adjudicate(
    approved: True,
    reason: "Premature approval",
    timestamp: 100,
  )

  debate.transition(initial, illegal_action)
  |> should.be_error()
}

pub fn mcp_tool_decode_test() {
  let result = mcp.decode_tool_call("run_command", "{\"command\": \"cargo test\"}")
  let assert Ok(call) = result

  call.name
  |> should.equal("run_command")

  let empty_err = mcp.decode_tool_call("", "{}")
  should.be_error(empty_err)
}

pub fn protocol_etf_envelope_test() {
  let payload = protocol.StringPacket("tagisan_ping")
  let call = protocol.call_envelope(1, payload)

  case call {
    protocol.TuplePacket([protocol.AtomPacket("$gen_call"), _, _]) -> True
    _ -> False
  }
  |> should.equal(True)
}
