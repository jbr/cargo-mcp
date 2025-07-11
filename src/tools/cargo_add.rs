use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::{Result, anyhow};
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Add dependencies to Cargo.toml using cargo add
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_add")]
pub struct CargoAdd {
    /// List of dependencies to add (e.g., ['serde', 'tokio@1.0'])
    pub dependencies: Vec<String>,

    /// Optional package name (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Add as development dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub dev: Option<bool>,

    /// Add as optional dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub optional: Option<bool>,

    /// Optional features to enable
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub features: Option<Vec<String>>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoAdd {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Add a simple dependency",
                item: Self {
                    dependencies: vec!["serde".into()],
                    package: None,
                    dev: None,
                    optional: None,
                    features: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Add multiple dependencies with versions",
                item: Self {
                    dependencies: vec!["serde@1.0".into(), "tokio@1.0".into()],
                    package: None,
                    dev: None,
                    optional: None,
                    features: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Add a dev dependency",
                item: Self {
                    dependencies: vec!["criterion".into()],
                    package: None,
                    dev: Some(true),
                    optional: None,
                    features: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Add dependency with features",
                item: Self {
                    dependencies: vec!["tokio".into()],
                    package: None,
                    dev: None,
                    optional: None,
                    features: Some(vec!["full".into()]),
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoAdd {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        if self.dependencies.is_empty() {
            return Err(anyhow!("No dependencies specified"));
        }

        let project_path = state.ensure_rust_project(None)?;

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        // Combine session env vars with command-specific env vars
        let mut env_vars = state.get_cargo_env(None)?.clone();
        if let Some(cmd_env) = &self.cargo_env {
            env_vars.extend(cmd_env.clone());
        }

        let mut args = vec!["add"];

        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.dev.unwrap_or(false) {
            args.push("--dev");
        }

        if self.optional.unwrap_or(false) {
            args.push("--optional");
        }

        let features_str;

        if let Some(ref features) = self.features {
            if !features.is_empty() {
                features_str = features.join(",");
                args.extend_from_slice(&["--features", &features_str]);
            }
        }

        // Add the dependencies
        for dep in &self.dependencies {
            args.push(dep);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), Some(&env_vars));
        execute_cargo_command(cmd, &project_path, "cargo add")
    }
}
