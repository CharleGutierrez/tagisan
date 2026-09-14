defmodule Tagisan do
  @moduledoc """
  High-level Elixir client for the Tagisan Sovereign Agent Engine.
  Connects Elixir OTP actors with native Rust `tgs` processes via 4-byte framed Port protocol.
  """

  alias Tagisan.{Agent, Debate, Port, Supervisor}

  @doc """
  Ping the Tagisan native port process.
  """
  def ping do
    Port.ping()
  end

  @doc """
  Query native engine status and registered actors.
  """
  def status do
    Port.status()
  end

  @doc """
  Send a direct completion query or task to the Tagisan engine.
  """
  def ask(prompt, timeout \\ 30_000) do
    Port.ask(prompt, timeout)
  end

  @doc """
  Spawn an autonomous agent actor under the dynamic supervision tree.
  """
  def start_agent(opts) do
    Supervisor.start_agent(opts)
  end

  @doc """
  Stop an agent by PID or registered name.
  """
  def stop_agent(agent_ref) do
    Supervisor.stop_agent(agent_ref)
  end

  @doc """
  Send a question or command to a specific agent actor.
  """
  def agent_ask(agent_ref, prompt, timeout \\ 30_000) do
    Agent.ask(agent_ref, prompt, timeout)
  end

  @doc """
  Cast an asynchronous message to an agent actor.
  """
  def agent_cast(agent_ref, message) do
    Agent.cast_message(agent_ref, message)
  end

  @doc """
  Run an adversarial dialectical debate across multiple agents.
  """
  def debate(topic, participant_configs, rounds \\ 3) do
    Debate.run(topic, participant_configs, rounds)
  end
end
