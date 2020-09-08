defmodule Ockam.MixProject do
  use Mix.Project

  @version "0.10.0-dev"

  @elixir_requirement "~> 1.10"

  @ockam_github_repo "https://github.com/ockam-network/ockam"
  @ockam_github_repo_path "implementations/elixir/applications/ockam"

  def project do
    [
      app: :ockam,
      version: @version,
      elixir: @elixir_requirement,
      consolidate_protocols: Mix.env() != :test,
      elixirc_options: [warnings_as_errors: true],
      deps: deps(),
      aliases: aliases(),

      # lint
      dialyzer: [flags: ["-Wunmatched_returns", :error_handling, :underspecs]],

      # test
      test_coverage: [output: "_build/cover"],

      # hex
      description: "A collection of tools for building connected systems that you can trust.",
      package: package(),

      # docs
      name: "Ockam",
      docs: docs()
    ]
  end

  # mix help compile.app for more
  def application do
    [
      mod: {Ockam, []},
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:telemetry, "~> 0.4.2"},
      {:ex_doc, "~> 0.22.2", only: :dev, runtime: false},
      {:credo, "~> 1.4", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.0", only: [:dev], runtime: false}
    ]
  end

  # used by hex
  defp package do
    [
      links: %{"GitHub" => @ockam_github_repo},
      licenses: ["Apache-2.0"]
    ]
  end

  # used by ex_doc
  defp docs do
    [
      main: "Ockam",
      source_url_pattern:
        "#{@ockam_github_repo}/blob/v#{@version}/#{@ockam_github_repo_path}/%{path}#L%{line}"
    ]
  end

  defp aliases do
    [
      compile: [&compile_native/1, "compile"],
      clean: ["clean", &clean_native/1],
      docs: "docs --output _build/docs --formatter html",
      test: "test --no-start --cover",
      lint: ["format --check-formatted", "credo --strict"],
      dialyzer: ["dialyzer --format dialyxir"]
    ]
  end

  defp native_build_path(), do: Path.join([Mix.Project.build_path(), "native"])

  defp native_priv_path() do
    Path.join([Mix.Project.app_path(), "priv", "native"])
  end

  defp compile_native(_args) do
    :ok = cmake_generate()
    :ok = cmake_build()
    :ok = copy_to_priv()
    :ok
  end

  defp clean_native(_) do
    File.rm_rf!(native_build_path())
    File.rm_rf!(native_priv_path())
  end

  defp cmake_generate() do
    {_, 0} =
      System.cmd(
        "cmake",
        ["-S", "native", "-B", native_build_path(), "-DBUILD_SHARED_LIBS=ON"],
        into: IO.stream(:stdio, :line),
        env: [{"ERL_INCLUDE_DIR", erl_include_dir()}]
      )
    :ok
  end

  defp cmake_build() do
    {_, 0} =
      System.cmd(
        "cmake",
        ["--build", native_build_path()],
        into: IO.stream(:stdio, :line),
        env: [{"ERL_INCLUDE_DIR", erl_include_dir()}]
      )
    :ok
  end

  defp erl_include_dir() do
    [:code.root_dir(), Enum.concat('erts-', :erlang.system_info(:version)), 'include']
    |> Path.join()
    |> to_string
  end

  defp copy_to_priv() do
    priv_path = native_priv_path()
    File.mkdir_p!(priv_path)

    # this likely only works on macos,
    # TODO(mrinal): make this work on all operating systems
    Path.join([native_build_path(), "**", "*.dylib"])
    |> Path.wildcard()
    |> Enum.each(fn(lib) ->
      filename = Path.basename(lib, ".dylib")
      destination = Path.join(priv_path, "#{filename}.so")
      File.cp!(lib, destination)
    end)

    :ok
  end
end
