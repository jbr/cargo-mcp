use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo test to execute tests
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_test")]
pub struct CargoTest {
    /// Optional package name to test (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional specific test name to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub test_name: Option<String>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoTest {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Run all tests in current project",
                item: Self {
                    package: None,
                    test_name: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run tests for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    test_name: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run a specific test",
                item: Self {
                    package: None,
                    test_name: Some("test_addition".into()),
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run tests with custom environment",
                item: Self {
                    package: None,
                    test_name: None,
                    toolchain: None,
                    cargo_env: Some([
                        ("RUST_LOG".into(), "debug".into()),
                        ("TEST_ENV".into(), "true".into()),
                    ].into()),
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoTest {
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

        let mut args = vec!["test"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if let Some(ref test_name) = self.test_name {
            args.push(test_name);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), Some(&env_vars));
        execute_cargo_command(cmd, &project_path, "cargo test")
    }
}
