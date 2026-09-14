//// Tagisan Gleam Sovereign Multi-Agent & Dialectical Debate Engine.
////
//// Production-grade BEAM/OTP actor models, formally verified debate state machines,
//// type-safe MCP tool decoders, and binary ETF packet serialization for Gleam.

import tagisan/agent
import tagisan/debate
import tagisan/mcp
import tagisan/protocol

pub const version = "0.2.0"

/// Returns current Tagisan engine version
pub fn get_version() -> String {
  version
}

/// Convenience helper to scaffold an autonomous sovereign agent spec
pub fn new_agent(name: String, role: String) -> agent.AgentState {
  agent.AgentState(
    name: name,
    role: role,
    iterations: 0,
    active: True,
  )
}

/// Convenience helper to initialize a dialectical debate state machine
pub fn new_debate(proposer: String, claim: String, reasoning: String) -> debate.DebateState {
  debate.Thesis(proposer: proposer, claim: claim, reasoning: reasoning)
}
