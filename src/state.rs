use anyhow::{Result, anyhow};
use fieldwork::Fieldwork;
use mcplease::session::SessionStore;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Formatter},
    path::PathBuf,
};

/// Shared context data that can be used across multiple MCP servers
#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq)]
pub struct SharedContextData {
    /// Current working context path
    context_path: Option<PathBuf>,
}

/// Session data specific to cargo operations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CargoSessionData {
    /// Default toolchain to use for cargo commands (e.g., "stable", "nightly", "1.70.0")
    default_toolchain: Option<String>,
}

/// Cargo tools with session support
#[derive(Fieldwork)]
#[fieldwork(get, get_mut)]
pub struct CargoTools {
    /// Private session store for cargo-specific state
    session_store: SessionStore<CargoSessionData>,
    /// Shared context store for cross-server communication (working directory)
    shared_context_store: SessionStore<SharedContextData>,
    #[field(set, with)]
    default_session_id: &'static str,
}

impl Debug for CargoTools {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CargoTools")
            .field("session_store", &self.session_store)
            .field("shared_context_store", &self.shared_context_store)
            .field("default_session_id", &self.default_session_id)
            .finish()
    }
}

impl CargoTools {
    /// Create a new CargoTools instance
    pub fn new() -> Result<Self> {
        // Private session store for cargo-specific state
        let mut private_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        private_path.push(".ai-tools");
        private_path.push("sessions");
        private_path.push("cargo-mcp.json");
        let session_store = SessionStore::new(Some(private_path))?;

        // Shared context store for cross-server communication (working directory)
        let mut shared_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        shared_path.push(".ai-tools");
        shared_path.push("sessions");
        shared_path.push("shared-context.json");
        let shared_context_store = SessionStore::new(Some(shared_path))?;

        let mut tools = Self {
            session_store,
            shared_context_store,
            default_session_id: "default",
        };

        // Check for default toolchain from environment variable
        if let Ok(toolchain) = std::env::var("CARGO_MCP_DEFAULT_TOOLCHAIN") {
            if !toolchain.is_empty() {
                log::info!(
                    "Setting default toolchain from CARGO_MCP_DEFAULT_TOOLCHAIN: {toolchain}"
                );
                tools.set_default_toolchain(Some(toolchain), None)?;
            }
        }

        Ok(tools)
    }

    /// Get context (working directory) for a session
    pub fn get_context(&mut self, session_id: Option<&str>) -> Result<Option<PathBuf>> {
        let session_id = session_id.unwrap_or_else(|| self.default_session_id());
        let shared_data = self.shared_context_store.get_or_create(session_id)?;
        Ok(shared_data.context_path.clone())
    }

    /// Set working directory for a session (shared across MCP servers)
    pub fn set_working_directory(&mut self, path: PathBuf, session_id: Option<&str>) -> Result<()> {
        let session_id = session_id.unwrap_or_else(|| self.default_session_id());
        self.shared_context_store.update(session_id, |data| {
            data.context_path = Some(path);
        })
    }

    /// Get cargo-specific session data
    pub fn get_cargo_session(&mut self, session_id: Option<&str>) -> Result<&CargoSessionData> {
        let session_id = session_id.unwrap_or_else(|| self.default_session_id());
        self.session_store.get_or_create(session_id)
    }

    /// Update cargo-specific session data
    pub fn update_cargo_session<F>(&mut self, session_id: Option<&str>, fun: F) -> Result<()>
    where
        F: FnOnce(&mut CargoSessionData),
    {
        let session_id = session_id.unwrap_or_else(|| self.default_session_id());
        self.session_store.update(session_id, fun)
    }

    /// Get the default toolchain for this session
    pub fn get_default_toolchain(&mut self, session_id: Option<&str>) -> Result<Option<String>> {
        let session_data = self.get_cargo_session(session_id)?;
        Ok(session_data.default_toolchain.clone())
    }

    /// Set the default toolchain for this session
    pub fn set_default_toolchain(
        &mut self,
        toolchain: Option<String>,
        session_id: Option<&str>,
    ) -> Result<()> {
        self.update_cargo_session(session_id, |data| {
            data.default_toolchain = toolchain;
        })
    }

    /// Check if the current working directory is a Rust project
    pub fn ensure_rust_project(&mut self, session_id: Option<&str>) -> Result<PathBuf> {
        let context = self
            .get_context(session_id)?
            .ok_or_else(|| anyhow!("No working directory set. Use set_working_directory first."))?;

        let cargo_toml = context.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(anyhow!(
                "Not a Rust project: Cargo.toml not found in {}",
                context.display()
            ));
        }

        Ok(context)
    }
}
