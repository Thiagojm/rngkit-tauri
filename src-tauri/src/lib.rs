//! RngKit desktop application backend.

pub mod commands;
pub mod coordinator;
pub mod discovery;
pub mod dto;
pub mod errors;

use std::sync::Mutex;

use coordinator::AppCoordinator;
use discovery::DiscoveryHandle;

/// Reachable `rngkit-core` git revision pinned by this application.
pub const RNGKIT_CORE_REVISION: &str = "183f3c7811f5593b3b42c2558ac726552b86687d";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppCoordinator::new()))
        .manage(DiscoveryHandle::live());

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::state::get_app_state,
        commands::discovery::refresh_sources,
        commands::discovery::select_source,
        commands::dev::apply_dev_scenario,
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::state::get_app_state,
        commands::discovery::refresh_sources,
        commands::discovery::select_source,
    ]);

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
            "183f3c7811f5593b3b42c2558ac726552b86687d"
        );
    }

    #[test]
    fn linked_library_crates_compile_without_hardware() {
        let bits = rngkit_core::SampleBits::new(8).expect("sample bits");
        assert_eq!(bits.get(), 8);
        let _ = std::any::type_name::<rngkit_analysis::Accumulator>();
        let _ = std::any::type_name::<rngkit_sources::SourceConfig>();
        let _ = std::any::type_name::<rngkit_sources::DiscoveryReport>();
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
