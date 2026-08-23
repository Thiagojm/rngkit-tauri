//! Tauri command handlers. Command functions translate IPC to coordinator calls.

pub mod state;

#[cfg(debug_assertions)]
pub mod dev;
