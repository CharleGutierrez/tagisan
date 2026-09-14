defmodule Tagisan.Agent do
  @moduledoc """
  Autonomous Tagisan agent GenServer.
  Maintains private state, prompt history, and provides synchronous `ask`
  and asynchronous `cast` communication.
  """
  use GenServer
  require Logger

  defstruct [
    :name,
    :role,
    :model,
    :system_prompt,
    memory: [],
    step_count: 0
  ]

  # ---------------------------------------------------------------------------
  # Client API
  # ---------------------------------------------------------------------------

  def start_link(opts) do
    name = Keyword.fetch!(opts, :name)
    GenServer.start_link(__MODULE__, opts, name: via_tuple(name))
  end

  def ask(agent_ref, prompt, timeout \\ 30_000) do
    GenServer.call(resolve_ref(agent_ref), {:ask, prompt}, timeout)
  end

  def cast_message(agent_ref, message) do
    GenServer.cast(resolve_ref(agent_ref), {:message, message})
  end

  def get_state(agent_ref) do
    GenServer.call(resolve_ref(agent_ref), :get_state)
  end

  def reset(agent_ref) do
    GenServer.call(resolve_ref(agent_ref), :reset)
  end

  # ---------------------------------------------------------------------------
  # GenServer Callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(opts) do
    name = Keyword.fetch!(opts, :name)
    role = Keyword.get(opts, :role, "generalist")
    model = Keyword.get(opts, :model, "auto")
    system_prompt = Keyword.get(opts, :system_prompt, "You are a sovereign Tagisan agent.")

    state = %__MODULE__{
      name: name,
      role: role,
      model: model,
      system_prompt: system_prompt,
      memory: [],
      step_count: 0
    }

    {:ok, state}
  end

  @impl true
  def handle_call({:ask, prompt}, _from, state) do
    new_step = state.step_count + 1
    response = "Agent [#{state.name} (#{state.role})] response to: #{prompt}"
    new_memory = [{:user, prompt}, {:agent, response} | state.memory]

    new_state = %{state | memory: new_memory, step_count: new_step}
    {:reply, {:ok, response}, new_state}
  end

  @impl true
  def handle_call(:get_state, _from, state) do
    {:reply, state, state}
  end

  @impl true
  def handle_call(:reset, _from, state) do
    {:reply, :ok, %{state | memory: [], step_count: 0}}
  end

  @impl true
  def handle_cast({:message, message}, state) do
    new_memory = [{:event, message} | state.memory]
    {:noreply, %{state | memory: new_memory}}
  end

  @impl true
  def terminate(reason, state) do
    Logger.debug("[Tagisan.Agent:#{state.name}] Terminating: #{inspect(reason)}")
    :ok
  end

  # ---------------------------------------------------------------------------
  # Registry Helpers
  # ---------------------------------------------------------------------------

  def via_tuple(name) do
    {:via, Registry, {Tagisan.AgentRegistry, name}}
  end

  defp resolve_ref(name) when is_binary(name) or is_atom(name), do: via_tuple(name)
  defp resolve_ref(pid) when is_pid(pid), do: pid
end
