use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo clippy for linting suggestions
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_clippy")]
pub struct CargoClippy {
    /// Optional package name to lint (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Apply suggested fixes automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub fix: Option<bool>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoClippy {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Basic cargo clippy in current project",
                item: Self {
                    package: None,
                    toolchain: None,
                    fix: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run clippy on a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    toolchain: None,
                    fix: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run clippy with automatic fixes",
                item: Self {
                    package: None,
                    toolchain: None,
                    fix: Some(true),
                    cargo_env: None,
                },
            },
            Example {
                description: "Run clippy with nightly toolchain",
                item: Self {
                    package: None,
                    toolchain: Some("nightly".into()),
                    fix: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoClippy {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        
        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        // Combine session env vars with command-specific env vars
        let mut env_vars = state.get_cargo_env(None)?.clone();
        if let Some(cmd_env) = &self.cargo_env {
            env_vars.extend(cmd_env.clone());
        }

        let mut args = vec!["clippy"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.fix.unwrap_or(false) {
            args.push("--fix");
        }

        // Add clippy arguments
        args.extend_from_slice(&["--", "-D", "warnings"]);

        let cmd = create_cargo_command(&args, toolchain.as_deref(), Some(&env_vars));
        execute_cargo_command(cmd, &project_path, "cargo clippy")
    }
}
