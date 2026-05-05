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
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_fmt_check")]
pub struct CargoFmtCheck {
    /// Optional package(s) to check (may be repeated; for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<Vec<String>>,

    /// Check all packages in the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all: Option<bool>,

    /// Rust edition to use for rustfmt (e.g., '2021')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub edition: Option<String>,

    /// rustfmt config overrides (key=value, may be repeated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub config: Option<Vec<String>>,

    /// Specific files to check (passed to rustfmt after `--`)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub files: Option<Vec<String>>,

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
                item: Self::default(),
            },
            Example {
                description: "Check formatting across all workspace packages",
                item: Self {
                    all: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Check formatting for specific packages",
                item: Self {
                    package: Some(vec!["my-lib".into(), "my-bin".into()]),
                    ..Self::default()
                },
            },
            Example {
                description: "Check specific files with rustfmt config overrides",
                item: Self {
                    files: Some(vec!["src/lib.rs".into(), "src/main.rs".into()]),
                    config: Some(vec![
                        "max_width=120".into(),
                        "group_imports=StdExternalCrate".into(),
                    ]),
                    edition: Some("2021".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Check formatting with nightly toolchain",
                item: Self {
                    toolchain: Some("nightly".into()),
                    ..Self::default()
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoFmtCheck {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["fmt"];

        if let Some(ref packages) = self.package {
            for p in packages {
                args.extend_from_slice(&["--package", p]);
            }
        }

        if self.all.unwrap_or(false) {
            args.push("--all");
        }

        // Separator for rustfmt-side args
        args.push("--");
        args.push("--check");

        if let Some(ref edition) = self.edition {
            args.extend_from_slice(&["--edition", edition]);
        }

        if let Some(ref configs) = self.config {
            for c in configs {
                args.extend_from_slice(&["--config", c]);
            }
        }

        if let Some(ref files) = self.files {
            for f in files {
                args.push(f);
            }
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo fmt --check")
    }
}
