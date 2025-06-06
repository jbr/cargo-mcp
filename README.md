# Cargo MCP Server

A Model Context Protocol (MCP) server that provides safe access to Cargo operations for Rust projects.

## Features

This MCP server exposes the following Cargo tools:

- **cargo_check** - Verify code compiles without producing executables
- **cargo_clippy** - Run the Clippy linter for code suggestions  
- **cargo_test** - Execute project tests
- **cargo_fmt_check** - Check code formatting without modifying files
- **cargo_build** - Build the project (debug or release mode)

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

### cargo_check
Verify that your Rust code compiles:
```json
{
  "name": "cargo_check",
  "arguments": {
    "path": "/path/to/rust/project",
    "package": "optional-package-name"
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
    "fix": false
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
    "test_name": "optional-specific-test"
  }
}
```

### cargo_fmt_check
Check formatting:
```json
{
  "name": "cargo_fmt_check",
  "arguments": {
    "path": "/path/to/rust/project"
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
    "release": false
  }
}
```

## Safety Features

- Only whitelisted Cargo commands are available
- Path validation ensures the target is a valid Rust project (has Cargo.toml)
- No arbitrary command execution
- All commands run in the specified project directory

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
