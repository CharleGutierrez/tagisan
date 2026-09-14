//// Autonomous Sovereign Agent GenServer using Gleam OTP Actor Model.

import gleam/erlang/process.{type Subject}
import gleam/otp/actor

pub type AgentState {
  AgentState(
    name: String,
    role: String,
    iterations: Int,
    active: Bool,
  )
}

pub type TaskResult {
  TaskSuccess(task_id: String, output: String, latency_ms: Int)
  TaskFailure(task_id: String, error: String)
}

pub type AgentStatus {
  AgentStatus(
    name: String,
    role: String,
    total_tasks: Int,
    operational: Bool,
  )
}

pub type Msg {
  Ping(reply_to: Subject(String))
  ExecuteTask(task_id: String, objective: String, reply_to: Subject(TaskResult))
  QueryStatus(reply_to: Subject(AgentStatus))
  Reset
  Crash(reason: String)
}

/// Initializer for Agent actor
pub fn init(name: String, role: String) -> AgentState {
  AgentState(
    name: name,
    role: role,
    iterations: 0,
    active: True,
  )
}

/// Core GenServer message handler with exhaustive pattern matching
pub fn handle_msg(msg: Msg, state: AgentState) -> actor.Next(Msg, AgentState) {
  case msg {
    Ping(reply_to) -> {
      process.send(reply_to, "pong")
      actor.continue(state)
    }

    ExecuteTask(task_id, objective, reply_to) -> {
      let result = TaskSuccess(
        task_id: task_id,
        output: "Executed objective: " <> objective <> " by agent " <> state.name,
        latency_ms: 42,
      )
      process.send(reply_to, result)
      let new_state = AgentState(
        name: state.name,
        role: state.role,
        iterations: state.iterations + 1,
        active: state.active,
      )
      actor.continue(new_state)
    }

    QueryStatus(reply_to) -> {
      let status = AgentStatus(
        name: state.name,
        role: state.role,
        total_tasks: state.iterations,
        operational: state.active,
      )
      process.send(reply_to, status)
      actor.continue(state)
    }

    Reset -> {
      let reset_state = AgentState(
        name: state.name,
        role: state.role,
        iterations: 0,
        active: True,
      )
      actor.continue(reset_state)
    }

    Crash(reason) -> {
      actor.Stop(process.Abnormal(reason))
    }
  }
}
