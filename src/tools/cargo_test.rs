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
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
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

    /// Don't capture stdout/stderr of tests, allow printing to console
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_capture: Option<bool>,

    /// Use this when you only need to check whether tests pass or fail.
    /// Displays one character per test instead of one line, producing compact output.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub quiet: Option<bool>,

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
                item: Self::default(),
            },
            Example {
                description: "Run tests for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run a specific test",
                item: Self {
                    test_name: Some("test_addition".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests with no capture (show println! output)",
                item: Self {
                    no_capture: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests with custom environment",
                item: Self {
                    cargo_env: Some(
                        [
                            ("RUST_LOG".into(), "debug".into()),
                            ("TEST_ENV".into(), "true".into()),
                        ]
                        .into(),
                    ),
                    ..Self::default()
                },
            },
            Example {
                description: "Run all tests with compact output (one char per test)",
                item: Self {
                    quiet: Some(true),
                    ..Self::default()
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoTest {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["test"];

        if self.quiet.unwrap_or(false) {
            args.push("--quiet");
        }

        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if let Some(ref test_name) = self.test_name {
            args.push(test_name);
        }

        // Add --nocapture if requested
        if self.no_capture.unwrap_or(false) {
            args.extend_from_slice(&["--", "--nocapture"]);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo test")
    }
}
