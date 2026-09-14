defmodule Tagisan.Supervisor do
  @moduledoc """
  DynamicSupervisor managing the lifecycle of parallel autonomous Tagisan agents.
  Capable of spawning and managing thousands of concurrent actor processes.
  """
  use DynamicSupervisor

  @name __MODULE__

  def start_link(init_arg \\ []) do
    DynamicSupervisor.start_link(__MODULE__, init_arg, name: @name)
  end

  @impl true
  def init(_init_arg) do
    DynamicSupervisor.init(strategy: :one_for_one)
  end

  @doc """
  Spawns a new agent process under dynamic supervision.
  """
  def start_agent(opts) do
    spec = {Tagisan.Agent, opts}
    DynamicSupervisor.start_child(@name, spec)
  end

  @doc """
  Terminates a running agent process.
  """
  def stop_agent(agent_ref) do
    pid = resolve_pid(agent_ref)

    if pid && Process.alive?(pid) do
      DynamicSupervisor.terminate_child(@name, pid)
    else
      {:error, :not_found}
    end
  end

  @doc """
  Count currently active supervised agents.
  """
  def count_agents do
    DynamicSupervisor.count_children(@name)
  end

  defp resolve_pid(pid) when is_pid(pid), do: pid

  defp resolve_pid(name) do
    case Registry.lookup(Tagisan.AgentRegistry, name) do
      [{pid, _}] -> pid
      [] -> nil
    end
  end
end
