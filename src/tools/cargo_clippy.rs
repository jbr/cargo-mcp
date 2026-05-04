use crate::state::CargoTools;
use crate::tools::cargo_utils::{create_cargo_command, execute_cargo_command};
use anyhow::Result;
use mcplease::{
    traits::{Tool, WithExamples},
    types::Example,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Run cargo clippy for linting suggestions
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema, clap::Args)]
#[serde(rename = "cargo_clippy")]
pub struct CargoClippy {
    /// Optional package name to lint (for workspaces)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub package: Option<String>,

    /// Lint all packages in the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub workspace: Option<bool>,

    /// Lint only the library
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub lib: Option<bool>,

    /// Lint only the specified binary (may be repeated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bin: Option<Vec<String>>,

    /// Lint all binaries
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bins: Option<bool>,

    /// Lint only the specified example (may be repeated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub example: Option<Vec<String>>,

    /// Lint all examples
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub examples: Option<bool>,

    /// Lint only the specified test target (may be repeated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub test: Option<Vec<String>>,

    /// Lint all tests
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub tests: Option<bool>,

    /// Lint only the specified bench target (may be repeated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bench: Option<Vec<String>>,

    /// Lint all benches
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub benches: Option<bool>,

    /// Lint all targets
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub all_targets: Option<bool>,

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

    /// Apply suggested fixes automatically
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub fix: Option<bool>,

    /// Allow fix on dirty working directory (implies --fix)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub allow_dirty: Option<bool>,

    /// Allow fix with staged changes (implies --fix)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub allow_staged: Option<bool>,

    /// Run without accessing the network
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub offline: Option<bool>,

    /// Run without checking lockfile is up-to-date
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub frozen: Option<bool>,

    /// Run without modifying Cargo.lock
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub locked: Option<bool>,

    /// Number of parallel jobs to run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub jobs: Option<u32>,

    /// Do not deny warnings (omit the default `-D warnings` clippy arg)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub no_deny_warnings: Option<bool>,

    /// Additional clippy lint args appended after `--` (e.g., ["-W", "clippy::pedantic"])
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub clippy_args: Option<Vec<String>>,

    /// Optional Rust toolchain to use (e.g., 'stable', 'nightly', '1.70.0')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub toolchain: Option<String>,

    /// Optional environment variables to set for the cargo command
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(skip)]
    pub cargo_env: Option<HashMap<String, String>>,
}

impl WithExamples for CargoClippy {
    fn examples() -> Vec<Example<Self>> {
        vec![
            Example {
                description: "Basic cargo clippy in current project",
                item: Self::default(),
            },
            Example {
                description: "Run clippy on a specific package",
                item: Self {
                    package: Some("my-lib".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Run clippy with automatic fixes",
                item: Self {
                    fix: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Lint all targets across the workspace with all features",
                item: Self {
                    workspace: Some(true),
                    all_targets: Some(true),
                    all_features: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Lint only the library in release mode",
                item: Self {
                    lib: Some(true),
                    release: Some(true),
                    ..Self::default()
                },
            },
            Example {
                description: "Lint a specific binary with selected features and a target triple",
                item: Self {
                    bin: Some(vec!["my-bin".into()]),
                    features: Some(vec!["feature-a".into(), "feature-b".into()]),
                    target: Some("x86_64-unknown-linux-gnu".into()),
                    ..Self::default()
                },
            },
            Example {
                description: "Apply fixes even with a dirty working tree, with extra pedantic lints",
                item: Self {
                    fix: Some(true),
                    allow_dirty: Some(true),
                    clippy_args: Some(vec!["-W".into(), "clippy::pedantic".into()]),
                    ..Self::default()
                },
            },
            Example {
                description: "Run clippy with nightly toolchain",
                item: Self {
                    toolchain: Some("nightly".into()),
                    ..Self::default()
                },
            },
        ]
    }
}

impl Tool<CargoTools> for CargoClippy {
    fn execute(self, state: &mut CargoTools) -> Result<String> {
        let project_path = state.ensure_rust_project(None)?;

        // Use toolchain from args, session default, or none
        let toolchain = self
            .toolchain
            .or_else(|| state.get_default_toolchain(None).unwrap_or(None));

        let mut args = vec!["clippy"];

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

        if let Some(ref examples) = self.example {
            for example in examples {
                args.extend_from_slice(&["--example", example]);
            }
        }

        if self.examples.unwrap_or(false) {
            args.push("--examples");
        }

        if let Some(ref tests) = self.test {
            for test in tests {
                args.extend_from_slice(&["--test", test]);
            }
        }

        if self.tests.unwrap_or(false) {
            args.push("--tests");
        }

        if let Some(ref benches) = self.bench {
            for bench in benches {
                args.extend_from_slice(&["--bench", bench]);
            }
        }

        if self.benches.unwrap_or(false) {
            args.push("--benches");
        }

        if self.all_targets.unwrap_or(false) {
            args.push("--all-targets");
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

        if self.fix.unwrap_or(false)
            || self.allow_dirty.unwrap_or(false)
            || self.allow_staged.unwrap_or(false)
        {
            args.push("--fix");
        }

        if self.allow_dirty.unwrap_or(false) {
            args.push("--allow-dirty");
        }

        if self.allow_staged.unwrap_or(false) {
            args.push("--allow-staged");
        }

        if self.offline.unwrap_or(false) {
            args.push("--offline");
        }

        if self.frozen.unwrap_or(false) {
            args.push("--frozen");
        }

        if self.locked.unwrap_or(false) {
            args.push("--locked");
        }

        let jobs_str;
        if let Some(jobs) = self.jobs {
            jobs_str = jobs.to_string();
            args.extend_from_slice(&["--jobs", &jobs_str]);
        }

        // Clippy lint args after `--`
        let has_extra_clippy_args = self
            .clippy_args
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let deny_warnings = !self.no_deny_warnings.unwrap_or(false);

        if deny_warnings || has_extra_clippy_args {
            args.push("--");
            if deny_warnings {
                args.extend_from_slice(&["-D", "warnings"]);
            }
            if let Some(ref extra) = self.clippy_args {
                for a in extra {
                    args.push(a);
                }
            }
        }

        let cmd = create_cargo_command(&args, toolchain.as_deref(), self.cargo_env.as_ref());
        execute_cargo_command(cmd, &project_path, "cargo clippy")
    }
}
