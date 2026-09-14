defmodule Tagisan.Port do
  @moduledoc """
  GenServer that spawns and supervises the native `tgs` executable
  communicating over stdin/stdout using Erlang's 4-byte network-endian
  packet framing: `Port.open({:spawn_executable, path}, [:binary, {:packet, 4}])`.
  """
  use GenServer
  require Logger

  @name __MODULE__

  defstruct [:port, :executable_path, pending_calls: %{}]

  # ---------------------------------------------------------------------------
  # Client API
  # ---------------------------------------------------------------------------

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: @name)
  end

  def ping(timeout \\ 5000) do
    GenServer.call(@name, :ping, timeout)
  end

  def status(timeout \\ 5000) do
    GenServer.call(@name, :status, timeout)
  end

  def ask(prompt, timeout \\ 30_000) do
    GenServer.call(@name, {:ask, prompt}, timeout)
  end

  def send_command(term, timeout \\ 5000) do
    GenServer.call(@name, {:command, term}, timeout)
  end

  # ---------------------------------------------------------------------------
  # GenServer Callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(opts) do
    executable =
      Keyword.get(opts, :executable) ||
      System.get_env("TAGISAN_BIN") ||
      System.find_executable("tgs") ||
      System.find_executable("tagisan")

    state = %__MODULE__{executable_path: executable}

    case open_port(state) do
      {:ok, port} ->
        Logger.info("[Tagisan.Port] Connected to native binary at #{executable}")
        {:ok, %{state | port: port}}

      {:error, reason} ->
        Logger.warning("[Tagisan.Port] Native binary not found: #{inspect(reason)}. Running in pure Elixir emulation mode.")
        {:ok, state}
    end
  end

  @impl true
  def handle_call(command, _from, %{port: port} = state) when not is_nil(port) do
    payload = :erlang.term_to_binary(command)
    send(port, {self(), {:command, payload}})

    receive do
      {^port, {:data, binary_data}} ->
        response = :erlang.binary_to_term(binary_data)
        {:reply, response, state}
    after
      5000 ->
        {:reply, {:error, :timeout}, state}
    end
  end

  # Pure Elixir fallback emulation when native binary is absent
  @impl true
  def handle_call(:ping, _from, %{port: nil} = state) do
    {:reply, :pong, state}
  end

  @impl true
  def handle_call(:status, _from, %{port: nil} = state) do
    status_map = %{
      engine: :tagisan_pure_beam,
      status: :ready,
      version: "0.2.0",
      actors: Registry.count(Tagisan.AgentRegistry)
    }
    {:reply, {:ok, status_map}, state}
  end

  @impl true
  def handle_call({:ask, prompt}, _from, %{port: nil} = state) do
    reply = "Tagisan Elixir Sovereign Actor processed prompt: #{prompt}"
    {:reply, {:ok, reply}, state}
  end

  @impl true
  def handle_call({:command, term}, _from, %{port: nil} = state) do
    {:reply, {:ok, term}, state}
  end

  @impl true
  def handle_info({port, {:exit_status, status}}, %{port: port} = state) do
    Logger.error("[Tagisan.Port] Native port exited with status #{status}. Attempting restart...")
    case open_port(state) do
      {:ok, new_port} -> {:noreply, %{state | port: new_port}}
      {:error, _} -> {:noreply, %{state | port: nil}}
    end
  end

  @impl true
  def handle_info(_msg, state) do
    {:noreply, state}
  end

  # ---------------------------------------------------------------------------
  # Private Helpers
  # ---------------------------------------------------------------------------

  defp open_port(%{executable_path: nil}), do: {:error, :not_found}

  defp open_port(%{executable_path: path}) do
    try do
      port =
        Port.open({:spawn_executable, path}, [
          :binary,
          {:packet, 4},
          :use_stdio,
          args: ["otp", "port"]
        ])

      {:ok, port}
    rescue
      e -> {:error, e}
    end
  end
end
