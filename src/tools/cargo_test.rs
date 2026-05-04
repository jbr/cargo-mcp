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

    /// Test all packages in the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub workspace: Option<bool>,

    /// Optional specific test name to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub test_name: Option<String>,

    /// Don't capture stdout/stderr of tests, allow printing to console
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_capture: Option<bool>,

    /// Test only the library
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub lib: Option<bool>,

    /// Test only the specified binary (may be repeated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bin: Option<Vec<String>>,

    /// Test all binaries
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bins: Option<bool>,

    /// Test all targets (lib, bins, tests, benches, examples)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all_targets: Option<bool>,

    /// Test only doc tests
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub doc: Option<bool>,

    /// Space or comma separated list of features to activate
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub features: Option<Vec<String>>,

    /// Activate all available features
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all_features: Option<bool>,

    /// Do not activate the `default` feature
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_default_features: Option<bool>,

    /// Build artifacts in release mode, with optimizations
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub release: Option<bool>,

    /// Build artifacts with the specified profile
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub profile: Option<String>,

    /// Build for the target triple
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub target: Option<String>,

    /// Number of parallel jobs to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub jobs: Option<u32>,

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
                description: "Run only library tests in release mode",
                item: Self {
                    lib: Some(true),
                    release: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests for a specific binary with selected features",
                item: Self {
                    bin: Some(vec!["my-bin".into()]),
                    features: Some(vec!["feature-a".into(), "feature-b".into()]),
                    ..Self::default()
                },
            },
            Example {
                description: "Run tests with all features enabled and a custom profile",
                item: Self {
                    all_features: Some(true),
                    profile: Some("test-fast".into()),
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

        if let Some(ref package) = self.package {
            args.extend_from_slice(&["--package", package]);
        }

        if self.workspace.unwrap_or(false) {
            args.push("--workspace");
        }

        if self.lib.unwrap_or(false) {
            args.push("--lib");
        }

        if let Some(ref bins) = self.bin {
            for bin in bins {
                args.extend_from_slice(&["--bin", bin]);
            }
        }

        if self.bins.unwrap_or(false) {
            args.push("--bins");
        }

        if self.all_targets.unwrap_or(false) {
            args.push("--all-targets");
        }

        if self.doc.unwrap_or(false) {
            args.push("--doc");
        }

        let features_joined;
        if let Some(ref features) = self.features {
            features_joined = features.join(",");
            args.extend_from_slice(&["--features", &features_joined]);
        }

        if self.all_features.unwrap_or(false) {
            args.push("--all-features");
        }

        if self.no_default_features.unwrap_or(false) {
            args.push("--no-default-features");
        }

        if self.release.unwrap_or(false) {
            args.push("--release");
        }

        if let Some(ref profile) = self.profile {
            args.extend_from_slice(&["--profile", profile]);
        }

        if let Some(ref target) = self.target {
            args.extend_from_slice(&["--target", target]);
        }

        let jobs_str;
        if let Some(jobs) = self.jobs {
            jobs_str = jobs.to_string();
            args.extend_from_slice(&["--jobs", &jobs_str]);
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
