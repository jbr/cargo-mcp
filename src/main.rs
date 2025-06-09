use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::BufReader;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;

#[derive(Parser)]
#[command(name = "cargo-mcp")]
#[command(about = "A Model Context Protocol server for Cargo operations")]
struct Cli {
    /// Optional "mcp" argument when invoked as `cargo mcp`
    #[arg(value_name = "SUBCOMMAND")]
    cargo_subcommand: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP server
    Serve,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum McpMessage {
    Request(McpRequest),
    Notification(McpNotification),
}

#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Value, // Requests always have an id
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpNotification {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    // No id field for notifications
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Value, // Always present for responses
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolCallParams {
    name: String,
    arguments: Option<Value>,
}

struct CargoMcpServer {
    tools: Vec<Tool>,
}

impl CargoMcpServer {
    fn new() -> Self {
        let tools = vec![
            Tool {
                name: "cargo_check".to_string(),
                description: "Run cargo check to verify the code compiles".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name to check (for workspaces)"
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "cargo_clippy".to_string(),
                description: "Run cargo clippy for linting suggestions".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name to lint (for workspaces)"
                        },
                        "fix": {
                            "type": "boolean",
                            "description": "Apply suggested fixes automatically",
                            "default": false
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "cargo_test".to_string(),
                description: "Run cargo test to execute tests".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name to test (for workspaces)"
                        },
                        "test_name": {
                            "type": "string",
                            "description": "Optional specific test name to run"
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "cargo_fmt_check".to_string(),
                description: "Check if code is properly formatted without modifying files"
                    .to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "cargo_build".to_string(),
                description: "Build the project with cargo build".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name to build (for workspaces)"
                        },
                        "release": {
                            "type": "boolean",
                            "description": "Build in release mode",
                            "default": false
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "cargo_bench".to_string(),
                description: "Run cargo bench to execute benchmarks".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name to benchmark (for workspaces)"
                        },
                        "bench_name": {
                            "type": "string",
                            "description": "Optional specific benchmark name to run"
                        },
                        "baseline": {
                            "type": "string",
                            "description": "Optional baseline name for comparison"
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "cargo_add".to_string(),
                description: "Add dependencies to Cargo.toml using cargo add".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name (for workspaces)"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "List of dependencies to add (e.g., ['serde', 'tokio@1.0'])"
                        },
                        "dev": {
                            "type": "boolean",
                            "description": "Add as development dependencies",
                            "default": false
                        },
                        "optional": {
                            "type": "boolean",
                            "description": "Add as optional dependencies",
                            "default": false
                        },
                        "features": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Optional features to enable"
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path", "dependencies"]
                }),
            },
            Tool {
                name: "cargo_remove".to_string(),
                description: "Remove dependencies from Cargo.toml using cargo remove".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name (for workspaces)"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "List of dependencies to remove"
                        },
                        "dev": {
                            "type": "boolean",
                            "description": "Remove from development dependencies",
                            "default": false
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path", "dependencies"]
                }),
            },
            Tool {
                name: "cargo_update".to_string(),
                description: "Update dependencies using cargo update".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the Rust project directory"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional package name (for workspaces)"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Optional specific dependencies to update"
                        },
                        "dry_run": {
                            "type": "boolean",
                            "description": "Perform a dry run to see what would be updated",
                            "default": false
                        },
                        "env": {
                            "type": "object",
                            "description": "Optional environment variables to set",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    },
                    "required": ["path"]
                }),
            },
        ];

        Self { tools }
    }

    async fn handle_message(&self, message: McpMessage) -> Option<McpResponse> {
        match message {
            McpMessage::Request(request) => Some(self.handle_request(request).await),
            McpMessage::Notification(notification) => {
                self.handle_notification(notification).await;
                None // Notifications don't get responses
            }
        }
    }

    async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tool_call(request.id, request.params).await,
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }

    async fn handle_notification(&self, _notification: McpNotification) {
        // Handle notifications like "notifications/initialized"
        // For now, we just ignore them since they don't require responses
    }

    fn handle_initialize(&self, id: Value) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "cargo-mcp",
                    "version": "0.1.0"
                }
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Value) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": self.tools
            })),
            error: None,
        }
    }

    async fn handle_tool_call(&self, id: Value, params: Option<Value>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let tool_call: ToolCallParams = match serde_json::from_value(params) {
            Ok(tc) => tc,
            Err(e) => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: format!("Invalid tool call params: {e}"),
                        data: None,
                    }),
                };
            }
        };

        let result = match self.execute_tool(&tool_call).await {
            Ok(output) => output,
            Err(e) => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32603,
                        message: format!("Tool execution failed: {e}"),
                        data: None,
                    }),
                };
            }
        };

        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "content": [{
                    "type": "text",
                    "text": result
                }]
            })),
            error: None,
        }
    }

    async fn execute_tool(&self, tool_call: &ToolCallParams) -> Result<String> {
        let empty_args = json!({});
        let args = tool_call.arguments.as_ref().unwrap_or(&empty_args);
        let path = args
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow!("Path argument is required"))?;

        let project_path = PathBuf::from(path);
        if !project_path.exists() {
            return Err(anyhow!("Project path does not exist: {}", path));
        }

        // Verify it's a Rust project
        if !project_path.join("Cargo.toml").exists() {
            return Err(anyhow!(
                "Not a Rust project: Cargo.toml not found in {}",
                path
            ));
        }

        match tool_call.name.as_str() {
            "cargo_check" => self.run_cargo_check(project_path, args).await,
            "cargo_clippy" => self.run_cargo_clippy(project_path, args).await,
            "cargo_test" => self.run_cargo_test(project_path, args).await,
            "cargo_fmt_check" => self.run_cargo_fmt_check(project_path, args).await,
            "cargo_build" => self.run_cargo_build(project_path, args).await,
            "cargo_bench" => self.run_cargo_bench(project_path, args).await,
            "cargo_add" => self.run_cargo_add(project_path, args).await,
            "cargo_remove" => self.run_cargo_remove(project_path, args).await,
            "cargo_update" => self.run_cargo_update(project_path, args).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_call.name)),
        }
    }

    async fn run_cargo_check(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("check").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo check", env_vars).await
    }

    async fn run_cargo_clippy(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("clippy").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if args.get("fix").and_then(|f| f.as_bool()).unwrap_or(false) {
            cmd.arg("--fix");
        }

        cmd.arg("--");
        cmd.arg("-D");
        cmd.arg("warnings");

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo clippy", env_vars).await
    }

    async fn run_cargo_test(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if let Some(test_name) = args.get("test_name").and_then(|t| t.as_str()) {
            cmd.arg(test_name);
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo test", env_vars).await
    }

    async fn run_cargo_fmt_check(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.args(["fmt", "--check"]).current_dir(&project_path);

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo fmt --check", env_vars).await
    }

    async fn run_cargo_build(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if args
            .get("release")
            .and_then(|r| r.as_bool())
            .unwrap_or(false)
        {
            cmd.arg("--release");
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo build", env_vars).await
    }

    async fn run_cargo_bench(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("bench").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if let Some(bench_name) = args.get("bench_name").and_then(|b| b.as_str()) {
            cmd.arg(bench_name);
        }

        if let Some(baseline) = args.get("baseline").and_then(|b| b.as_str()) {
            cmd.args(["--", "--save-baseline", baseline]);
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo bench", env_vars).await
    }

    async fn run_cargo_add(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("add").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if args.get("dev").and_then(|d| d.as_bool()).unwrap_or(false) {
            cmd.arg("--dev");
        }

        if args
            .get("optional")
            .and_then(|o| o.as_bool())
            .unwrap_or(false)
        {
            cmd.arg("--optional");
        }

        if let Some(features) = args.get("features").and_then(|f| f.as_array())
            && !features.is_empty() {
                let features_str = features
                    .iter()
                    .filter_map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                cmd.args(["--features", &features_str]);
            }

        // Add the dependencies
        if let Some(deps) = args.get("dependencies").and_then(|d| d.as_array()) {
            for dep in deps {
                if let Some(dep_str) = dep.as_str() {
                    cmd.arg(dep_str);
                }
            }
        } else {
            return Err(anyhow!("No dependencies specified"));
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo add", env_vars).await
    }

    async fn run_cargo_remove(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("remove").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if args.get("dev").and_then(|d| d.as_bool()).unwrap_or(false) {
            cmd.arg("--dev");
        }

        // Add the dependencies to remove
        if let Some(deps) = args.get("dependencies").and_then(|d| d.as_array()) {
            for dep in deps {
                if let Some(dep_str) = dep.as_str() {
                    cmd.arg(dep_str);
                }
            }
        } else {
            return Err(anyhow!("No dependencies specified"));
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo remove", env_vars).await
    }

    async fn run_cargo_update(&self, project_path: PathBuf, args: &Value) -> Result<String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("update").current_dir(&project_path);

        if let Some(package) = args.get("package").and_then(|p| p.as_str()) {
            cmd.args(["--package", package]);
        }

        if args
            .get("dry_run")
            .and_then(|d| d.as_bool())
            .unwrap_or(false)
        {
            cmd.arg("--dry-run");
        }

        // Add specific dependencies to update if provided
        if let Some(deps) = args.get("dependencies").and_then(|d| d.as_array()) {
            for dep in deps {
                if let Some(dep_str) = dep.as_str() {
                    cmd.args(["--package", dep_str]);
                }
            }
        }

        let env_vars = args.get("env").and_then(|e| e.as_object());
        self.execute_command(cmd, "cargo update", env_vars).await
    }

    async fn execute_command(&self, mut cmd: Command, command_name: &str, env_vars: Option<&serde_json::Map<String, Value>>) -> Result<String> {
        // Apply environment variables if provided
        if let Some(env_map) = env_vars {
            for (key, value) in env_map {
                if let Some(value_str) = value.as_str() {
                    cmd.env(key, value_str);
                }
            }
        }

        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = format!("=== {command_name} ===\n");

        if output.status.success() {
            result.push_str("✅ Command completed successfully\n\n");
        } else {
            result.push_str(&format!(
                "❌ Command failed with exit code: {}\n\n",
                output.status.code().unwrap_or(-1)
            ));
        }

        if !stdout.is_empty() {
            result.push_str("STDOUT:\n");
            result.push_str(&stdout);
            result.push('\n');
        }

        if !stderr.is_empty() {
            result.push_str("STDERR:\n");
            result.push_str(&stderr);
            result.push('\n');
        }

        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Skip the "mcp" argument if present (when invoked as `cargo mcp`)
    match cli.cargo_subcommand.as_deref() {
        Some("mcp") | None => {
            // Continue with normal processing
            match cli.command {
                Some(Commands::Serve) | None => {
                    let server = CargoMcpServer::new();
                    run_server(server).await?;
                }
            }
        }
        Some(other) => {
            eprintln!("Unknown subcommand: {other}");
            eprintln!("This tool is designed to be used as 'cargo-mcp' or 'cargo mcp serve'");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_server(server: CargoMcpServer) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let message: McpMessage = match serde_json::from_str(trimmed) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Failed to parse message: {e}");
                        continue;
                    }
                };

                if let Some(response) = server.handle_message(message).await {
                    let response_json = serde_json::to_string(&response)?;

                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {e}");
                break;
            }
        }
    }

    Ok(())
}
