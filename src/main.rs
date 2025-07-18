mod state;
mod tools;

#[cfg(test)]
mod tests;

use anyhow::Result;
use mcplease::server_info;
use state::CargoTools;

const INSTRUCTIONS: &str = "Cargo operations for Rust projects.

Use set_working_directory to set the project directory first, then run cargo commands.";

fn main() -> Result<()> {
    let mut state = CargoTools::new()?;

    mcplease::run::<tools::Tools, _>(&mut state, server_info!(), Some(INSTRUCTIONS))
}
