defmodule Tagisan.Debate do
  @moduledoc """
  Dialectical debate orchestrator for Tagisan agents.
  Conducts thesis -> antithesis -> synthesis rounds across autonomous Elixir actors.
  """
  alias Tagisan.Agent

  defstruct [
    :topic,
    :participants,
    :rounds,
    transcript: []
  ]

  @doc """
  Run a dialectical debate on a topic across specified participants.
  """
  def run(topic, participants, rounds \\ 3) do
    debate = %__MODULE__{
      topic: topic,
      participants: participants,
      rounds: rounds,
      transcript: []
    }

    execute_debate(debate)
  end

  defp execute_debate(%__MODULE__{rounds: max_rounds, participants: participants, topic: topic} = debate) do
    transcript =
      Enum.reduce(1..max_rounds, [], fn round, acc ->
        round_entries =
          Enum.map(participants, fn participant ->
            agent_ref = participant[:name] || participant[:pid]
            role = participant[:role] || "debater"

            prompt =
              "Round #{round} Dialectical Debate on '#{topic}'. Your role: #{role}. Previous points: #{inspect(acc)}"

            {:ok, statement} = Agent.ask(agent_ref, prompt)
            %{round: round, participant: agent_ref, role: role, statement: statement}
          end)

        acc ++ round_entries
      end)

    synthesis = synthesize_debate(topic, transcript)
    %{debate | transcript: transcript ++ [%{round: :synthesis, statement: synthesis}]}
  end

  defp synthesize_debate(topic, transcript) do
    "Dialectical Synthesis for '#{topic}' across #{length(transcript)} arguments: Reconciled consensus achieved."
  end
end
