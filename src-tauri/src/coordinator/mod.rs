//! Application coordinator. Discovery and session files are later checkpoints.

mod fixtures;
mod state;

pub use fixtures::{DevScenario, bitb_candidate, pseudo_candidate};
pub use state::AppCoordinator;
