//! RngKit desktop application backend.

pub mod commands;
pub mod coordinator;
pub mod dto;
pub mod errors;

use std::sync::Mutex;

use coordinator::AppCoordinator;

/// Reachable `rngkit-core` git revision pinned by this application.
pub const RNGKIT_CORE_REVISION: &str = "3f327e9e88679c26683323f116cd6d7b3ea64fff";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppCoordinator::new()));

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::state::get_app_state,
        commands::dev::apply_dev_scenario,
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![commands::state::get_app_state]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running RngKit");
}

#[cfg(test)]
mod tests {
    use super::RNGKIT_CORE_REVISION;

    #[test]
    fn pins_gate_a_library_revision() {
        assert_eq!(
            RNGKIT_CORE_REVISION,
            "3f327e9e88679c26683323f116cd6d7b3ea64fff"
        );
    }

    #[test]
    fn linked_library_crates_compile_without_hardware() {
        let bits = rngkit_core::SampleBits::new(8).expect("sample bits");
        assert_eq!(bits.get(), 8);
        let _ = std::any::type_name::<rngkit_analysis::Accumulator>();
        let _ = std::any::type_name::<rngkit_sources::SourceConfig>();
        let _ = std::any::type_name::<rngkit_engine::EngineConfig>();
        let _ = rngkit_recording::CONCATENATION_KIND;
        let _ = rngkit_xlsx::REF_PLUS;
    }

    #[test]
    fn production_start_is_idle() {
        let snapshot = crate::coordinator::AppCoordinator::new().snapshot();
        assert_eq!(snapshot.collection.state, crate::dto::CollectionState::Idle);
        assert!(snapshot.collection.candidates.is_empty());
    }
}
