use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo bench to execute benchmarks
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_bench")]
pub struct CargoBench {
    /// Optional package name to benchmark (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Optional specific benchmark name to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bench_name: Option<String>,

    /// Optional baseline name for comparison
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub baseline: Option<String>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoBench {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Run all benchmarks",
                item: Self {
                    package: None,
                    bench_name: None,
                    baseline: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run a specific benchmark",
                item: Self {
                    package: None,
                    bench_name: Some("my_benchmark".into()),
                    baseline: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run benchmarks for a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    bench_name: None,
                    baseline: None,
                    toolchain: None,
                    cargo_env: None,
                },
            },
            Example {
                description: "Run benchmarks with a baseline for comparison",
                item: Self {
                    package: None,
                    bench_name: None,
                    baseline: Some("main".into()),
                    toolchain: None,
                    cargo_env: None,
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoBench {
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

        let mut args = vec!["bench"];
        
        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if let Some(ref bench_name) = self.bench_name {
            args.push(bench_name);
        }

        if let Some(ref baseline) = self.baseline {
            args.extend_from_slice(&["--", "--save-baseline", baseline]);
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), Some(&env_vars));
        execute_cargo_command(cmd, &project_path, "cargo bench")
    }
}
