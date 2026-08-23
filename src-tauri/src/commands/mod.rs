//! Tauri command handlers. Command functions translate IPC to coordinator calls.

pub mod discovery;
pub mod state;

#[cfg(debug_assertions)]
pub mod dev;
