use crate::state::CargoTools;
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Set the working directory for cargo operations
/// 
/// This sets the shared working directory that will be used by all AI tools,
/// not just cargo-mcp. Other MCP servers like fs-mcp will also use this directory.
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "set_working_directory")]
pub struct SetWorkingDirectory {
    /// Path to set as the working directory
    /// Can be absolute or relative to current directory
    pub path: String,
}

impl WithExamples for SetWorkingDirectory {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Set working directory to current directory",
                item: Self {
                    path: ".".into(),
                },
            },
            Example {
                description: "Set working directory to a Rust project",
                item: Self {
                    path: "~/my-rust-project".into(),
                },
            },
            Example {
                description: "Set working directory using absolute path",
                item: Self {
                    path: "/Users/username/projects/my-app".into(),
                },
            },
        ]
    }
}

impl Tool<CargoTools> for SetWorkingDirectory {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let expanded_path = PathBuf::from(&*shellexpand::tilde(&self.path));
        let canonical_path = std::fs::canonicalize(&expanded_path)
            .map_err(|e| anyhow::anyhow!("Could not resolve path '{}': {}", self.path, e))?;

        state.set_working_directory(canonical_path.clone(), None)?;

        // Check if it's a Rust project and provide helpful feedback
        let cargo_toml = canonical_path.join("Cargo.toml");
        if cargo_toml.exists() {
            Ok(format!(
                "✅ Working directory set to: {}\n🦀 Rust project detected (Cargo.toml found)",
                canonical_path.display()
            ))
        } else {
            Ok(format!(
                "✅ Working directory set to: {}\n⚠️  No Cargo.toml found - this doesn't appear to be a Rust project",
                canonical_path.display()
            ))
        }
    }
}
