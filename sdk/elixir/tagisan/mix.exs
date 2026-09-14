defmodule Tagisan.MixProject do
  use Mix.Project

  def project do
    [
      app: :tagisan,
      version: "0.2.0",
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      description: "Elixir SDK for Tagisan - Sovereign Multi-Agent Architecture & BEAM/OTP Engine",
      package: package(),
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Tagisan.Application, []}
    ]
  end

  defp deps do
    [
      {:ex_doc, ">= 0.0.0", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      maintainers: ["Charle Gutierrez"],
      licenses: ["MIT"],
      links: %{"GitHub" => "https://github.com/tagisan/tagisan"}
    ]
  end
end
