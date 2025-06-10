# Cargo MCP Server

> [!CAUTION]
> This repository is written by AI as an experiment. Although I
> ([jbr](https://github.com/jbr)) review every line, the code quality is not necessarily identical
> to what I would have written. Caveat usor.

A Model Context Protocol (MCP) server that provides safe access to Cargo operations for Rust projects.

## Features

This MCP server exposes the following Cargo tools:

- **cargo_check** - Verify code compiles without producing executables
- **cargo_clippy** - Run the Clippy linter for code suggestions  
- **cargo_test** - Execute project tests
- **cargo_fmt_check** - Check code formatting without modifying files
- **cargo_build** - Build the project (debug or release mode)
- **cargo_bench** - Run benchmarks
- **cargo_add** - Add dependencies to Cargo.toml
- **cargo_remove** - Remove dependencies from Cargo.toml
- **cargo_update** - Update dependencies

All tools support setting custom environment variables via the `env` parameter.

## Installation

```bash
cargo build --release
```

## Usage with Claude Desktop

Add this to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "cargo-mcp": {
      "command": "/path/to/cargo-mcp/target/release/cargo-mcp",
      "args": ["serve"]
    }
  }
}
```

## Tool Usage

All tools accept an optional `env` parameter to set environment variables for the cargo command:

```json
{
  "env": {
    "CARGO_LOG": "debug",
    "RUSTFLAGS": "-C target-cpu=native",
    "CARGO_TARGET_DIR": "/tmp/my-target"
  }
}
```

### cargo_check
Verify that your Rust code compiles:
```json
{
  "name": "cargo_check",
  "arguments": {
    "path": "/path/to/rust/project",
    "package": "optional-package-name",
    "env": {
      "CARGO_LOG": "debug"
    }
  }
}
```

### cargo_clippy  
Get linting suggestions:
```json
{
  "name": "cargo_clippy", 
  "arguments": {
    "path": "/path/to/rust/project",
    "package": "optional-package-name",
    "fix": false,
    "env": {
      "RUSTFLAGS": "-D warnings"
    }
  }
}
```

### cargo_test
Run tests:
```json
{
  "name": "cargo_test",
  "arguments": {
    "path": "/path/to/rust/project", 
    "package": "optional-package-name",
    "test_name": "optional-specific-test",
    "env": {
      "RUST_TEST_THREADS": "1"
    }
  }
}
```

### cargo_fmt_check
Check formatting:
```json
{
  "name": "cargo_fmt_check",
  "arguments": {
    "path": "/path/to/rust/project",
    "env": {
      "CARGO_LOG": "info"
    }
  }
}
```

### cargo_build
Build the project:
```json
{
  "name": "cargo_build",
  "arguments": {
    "path": "/path/to/rust/project",
    "package": "optional-package-name", 
    "release": false,
    "env": {
      "RUSTFLAGS": "-C target-cpu=native"
    }
  }
}
```

### cargo_bench
Run benchmarks:
```json
{
  "name": "cargo_bench",
  "arguments": {
    "path": "/path/to/rust/project",
    "package": "optional-package-name",
    "bench_name": "optional-specific-benchmark",
    "baseline": "optional-baseline-name",
    "env": {
      "CARGO_LOG": "debug"
    }
  }
}
```

### cargo_add
Add dependencies:
```json
{
  "name": "cargo_add",
  "arguments": {
    "path": "/path/to/rust/project",
    "dependencies": ["serde", "tokio@1.0"],
    "dev": false,
    "optional": false,
    "features": ["derive"],
    "env": {
      "CARGO_LOG": "info"
    }
  }
}
```

### cargo_remove
Remove dependencies:
```json
{
  "name": "cargo_remove",
  "arguments": {
    "path": "/path/to/rust/project",
    "dependencies": ["unused-dep"],
    "dev": false,
    "env": {
      "CARGO_LOG": "info"
    }
  }
}
```

### cargo_update
Update dependencies:
```json
{
  "name": "cargo_update",
  "arguments": {
    "path": "/path/to/rust/project",
    "package": "optional-package-name",
    "dependencies": ["specific-dep-to-update"],
    "dry_run": false,
    "env": {
      "CARGO_LOG": "debug"
    }
  }
}
```

## Environment Variables

The `env` parameter allows you to set environment variables that will be passed to the cargo command. Common useful environment variables include:

- **CARGO_LOG** - Set logging level (`trace`, `debug`, `info`, `warn`, `error`)
- **RUSTFLAGS** - Pass flags to the Rust compiler (e.g., `-C target-cpu=native`)
- **CARGO_TARGET_DIR** - Override the target directory for build artifacts
- **RUST_TEST_THREADS** - Control test parallelism
- **CARGO_INCREMENTAL** - Enable/disable incremental compilation

## Safety Features

- Only whitelisted Cargo commands are available
- Path validation ensures the target is a valid Rust project (has Cargo.toml)
- No arbitrary command execution
- All commands run in the specified project directory
- Environment variables are safely passed through to cargo processes

## Development

To test the server manually:

```bash
cargo run -- serve
```

Then send MCP requests via stdin. Example initialization:

```json
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}
```

## License

MIT
