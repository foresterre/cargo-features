# cargo-features

Power tools for working with Cargo features

```
Usage: cargo features <COMMAND>

Commands:
  discover  Discover which crates enabled specific conditional compilation features
  powerset  Run a cargo command for every combination in the feature powerset
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## cargo features discover

Discover which crates enabled specific conditional compilation features

```
Usage: cargo features discover [OPTIONS]

Options:
  -a, --all                            Show all available features even if not currently enabled
  -m, --manifest-path <MANIFEST_PATH>  Path to Cargo.toml (defaults to current directory)
  -h, --help                           Print help
  -V, --version                        Print version
```

## cargo features powerset

Run a cargo command for every combination in the feature powerset

```
Usage: cargo features powerset [OPTIONS] <CARGO_COMMAND>...

Arguments:
  <CARGO_COMMAND>...  Cargo command to run (e.g., "build", "test", "check")

Options:
      --skip-empty                     Skip the empty feature set (no features enabled)
  -f, --features <FEATURES>            Specific features to include in powerset (default: all)
      --manifest-path <MANIFEST_PATH>  Path to Cargo.toml
  -h, --help                           Print help
  -V, --version                        Print version
```