use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Update dependencies using cargo update
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_update")]
pub struct CargoUpdate {
    /// Optional package name (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional specific dependencies to update
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub dependencies: Option<Vec<String>>,

    /// Perform a dry run to see what would be updated
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub dry_run: Option<bool>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoUpdate {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Update all dependencies",
                item: Self {
                    package: None,
                    dependencies: None,
                    dry_run: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Dry run to see what would be updated",
                item: Self {
                    package: None,
                    dependencies: None,
                    dry_run: Some(true),
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Update specific dependencies",
                item: Self {
                    package: None,
                    dependencies: Some(vec!["serde".into(), "tokio".into()]),
                    dry_run: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Update dependencies for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    dependencies: None,
                    dry_run: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoUpdate {
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

        let mut args = vec!["update"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.dry_run.unwrap_or(false) {
            args.push("--dry-run");
        }

        // Add specific dependencies to update if provided
        if let Some(ref deps) = self.dependencies {
            for dep in deps {
                args.extend_from_slice(&["--package", dep]);
            }
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), Some(&env_vars));
        execute_cargo_command(cmd, &project_path, "cargo update")
    }
}
