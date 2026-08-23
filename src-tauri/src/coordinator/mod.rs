//! Application coordinator. Session files are a later checkpoint.

mod fixtures;
mod state;

pub use fixtures::{DevScenario, bitb_candidate, pseudo_candidate};
pub use state::AppCoordinator;
