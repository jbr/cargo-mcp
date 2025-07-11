use anyhow::Result;
use std::{collections::HashMap, path::PathBuf, process::Command};

/// Helper to create a cargo command with optional toolchain and environment variables
pub fn create_cargo_command(
    cargo_args: &[&str],
    toolchain: Option<&str>,
    env_vars: Option<&HashMap<String, String>>,
) -> Command {
    let mut cmd = if let Some(toolchain) = toolchain {
        let mut cmd = Command::new("rustup");
        cmd.args(["run", toolchain, "cargo"]);
        cmd.args(cargo_args);
        cmd
    } else {
        let mut cmd = Command::new("cargo");
        cmd.args(cargo_args);
        cmd
    };

    // Apply environment variables if provided
    if let Some(env_map) = env_vars {
        for (key, value) in env_map {
            cmd.env(key, value);
        }
    }

    cmd
}

/// Execute a cargo command and format the output for MCP response
pub fn execute_cargo_command(
    mut cmd: Command,
    project_path: &PathBuf,
    command_name: &str,
) -> Result<String> {
    cmd.current_dir(project_path);

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = format!("=== {} ===\n", command_name);
    result.push_str(&format!(
        "📁 Working directory: {}\n",
        project_path.display()
    ));
    result.push_str(&format!("🔧 Command: {}\n\n", format_command(&cmd)));

    if output.status.success() {
        result.push_str("✅ Command completed successfully\n\n");
    } else {
        result.push_str(&format!(
            "❌ Command failed with exit code: {}\n\n",
            output.status.code().unwrap_or(-1)
        ));
    }

    if !stdout.is_empty() {
        result.push_str("📤 STDOUT:\n");
        result.push_str(&stdout);
        if !stdout.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
    }

    if !stderr.is_empty() {
        result.push_str("📤 STDERR:\n");
        result.push_str(&stderr);
        if !stderr.ends_with('\n') {
            result.push('\n');
        }
        result.push('\n');
    }

    if stdout.is_empty() && stderr.is_empty() {
        result.push_str("ℹ️  No output produced\n");
    }

    Ok(result)
}

/// Format a command for display
fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|arg| shell_escape(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ");

    if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args)
    }
}

/// Simple shell escaping for display purposes
fn shell_escape(arg: &str) -> String {
    if arg.contains(' ') || arg.contains('"') || arg.contains('\'') || arg.contains('\\') {
        format!("{:?}", arg) // Uses Rust's debug escaping
    } else {
        arg.to_string()
    }
}
