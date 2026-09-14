defmodule Tagisan.Application do
  @moduledoc """
  Root OTP Application and Supervisor for Tagisan.
  Manages:
  1. Registry for named agents (`Tagisan.AgentRegistry`)
  2. Native port communication GenServer (`Tagisan.Port`)
  3. DynamicSupervisor for autonomous agents (`Tagisan.Supervisor`)
  """
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Registry, keys: :unique, name: Tagisan.AgentRegistry},
      Tagisan.Port,
      Tagisan.Supervisor
    ]

    opts = [strategy: :one_for_one, name: Tagisan.RootSupervisor]
    Supervisor.start_link(children, opts)
  end
end
