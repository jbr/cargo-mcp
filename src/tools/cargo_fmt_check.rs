use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Check if code is properly formatted without modifying files
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_fmt_check")]
pub struct CargoFmtCheck {
    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoFmtCheck {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Check formatting in current project",
                item: Self {
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Check formatting with nightly toolchain",
                item: Self {
                    toolchain: Some("nightly".into()),
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoFmtCheck {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;
        
        // Use toolchain from args, session default, or none
        let toolchain = self.toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));


        let args = vec!["fmt", "--check"];
        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo fmt --check")
    }
}
