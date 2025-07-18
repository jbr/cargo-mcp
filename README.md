# Cargo MCP Server

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
- **cargo_clean** - Remove artifacts that cargo has generated in the past
- **cargo_run** - Run a binary or example


All tools support setting custom environment variables via the `cargo_env` parameter and rust
toolchain with the `toolchain` parameter.

## Installation

```bash
cargo install cargo-mcp
```

## Usage with Claude Desktop

Add this to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "cargo-mcp": {
      "command": "/path/to/cargo-mcp/cargo-mcp",
      "args": ["serve"]
    }
  }
}
```

Optionally, include `"env": {"CARGO_MCP_DEFAULT_TOOLCHAIN": {{toolchain}} }` in the arguments where
`{{toolchain}}` is something like "nightly" or "stable"


## Safety Features

- Only whitelisted Cargo commands are available
- Path validation ensures the target is a valid Rust project (has Cargo.toml)
- No arbitrary command execution
- All commands run in the specified project directory

## License

MIT or APACHE-2.0
