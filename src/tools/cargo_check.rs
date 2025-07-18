use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo check to verify the code compiles
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_check")]
pub struct CargoCheck {
    /// Optional package name to check (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoCheck {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Basic cargo check in current project",
                item: Self {
                    package: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Check a specific package in a workspace",
                item: Self {
                    package: Some("my-lib".into()),
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Check using nightly toolchain",
                item: Self {
                    package: None,
                    toolchain: Some("nightly".into()),
                    cargo_env: None,
                },
            },
            Example {
                description: "Check with custom environment variables",
                item: Self {
                    package: None,
                    toolchain: None,
                    cargo_env: Some(
                        [
                            ("RUSTFLAGS".into(), "-D warnings".into()),
                            ("CARGO_TARGET_DIR".into(), "./target-check".into()),
                        ]
                        .into(),
                    ),
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoCheck {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["check"];

        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo check")
    }
}
