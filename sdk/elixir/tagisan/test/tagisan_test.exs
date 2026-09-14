defmodule TagisanTest do
  use ExUnit.Case
  doctest Tagisan

  setup do
    Application.ensure_all_started(:tagisan)
    :ok
  end

  test "ping returns pong" do
    assert Tagisan.ping() == :pong
  end

  test "status returns ready and engine info" do
    {:ok, status} = Tagisan.status()
    assert status.status == :ready
  end

  test "spawns dynamic agent and executes call and cast" do
    agent_name = "test_agent_#{System.unique_integer([:positive])}"
    {:ok, pid} = Tagisan.start_agent(name: agent_name, role: "architect")

    assert Process.alive?(pid)

    {:ok, response} = Tagisan.agent_ask(agent_name, "What is your architecture?")
    assert String.contains?(response, agent_name)
    assert String.contains?(response, "architect")

    assert Tagisan.agent_cast(agent_name, {:log, "Event recorded"}) == :ok

    state = Tagisan.Agent.get_state(agent_name)
    assert state.step_count == 1
    assert length(state.memory) == 3

    assert Tagisan.stop_agent(agent_name) == :ok
    refute Process.alive?(pid)
  end

  test "orchestrates dialectical debate across multiple agents" do
    a1_name = "proponent_#{System.unique_integer([:positive])}"
    a2_name = "opponent_#{System.unique_integer([:positive])}"

    {:ok, _} = Tagisan.start_agent(name: a1_name, role: "proponent")
    {:ok, _} = Tagisan.start_agent(name: a2_name, role: "opponent")

    participants = [
      [name: a1_name, role: "proponent"],
      [name: a2_name, role: "opponent"]
    ]

    debate_result = Tagisan.debate("Should AI systems be sovereign?", participants, 2)
    assert length(debate_result.transcript) == 5

    Tagisan.stop_agent(a1_name)
    Tagisan.stop_agent(a2_name)
  end

  test "ETF term serialization roundtrip in pure Elixir" do
    term = {:ok, %{name: "Tagisan", version: "0.2.0", tags: [:beam, :otp, :rust]}}
    binary = :erlang.term_to_binary(term)
    decoded = :erlang.binary_to_term(binary)
    assert decoded == term
  end
end
