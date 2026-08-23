//! RngKit desktop application backend.

pub mod collection;
pub mod commands;
pub mod coordinator;
pub mod diagnostics;
pub mod discovery;
pub mod dto;
pub mod errors;
pub mod lifecycle;
pub mod preferences;

use std::sync::Mutex;

use collection::CollectionHandle;
use commands::dialogs::DialogHandle;
use coordinator::AppCoordinator;
use discovery::DiscoveryHandle;
use preferences::{MonitorRect, PreferencesHandle, clamp_geometry};
use tauri::{Manager, PhysicalPosition, PhysicalSize};

/// Reachable `rngkit-core` git revision pinned by this application.
pub const RNGKIT_CORE_REVISION: &str = "183f3c7811f5593b3b42c2558ac726552b86687d";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppCoordinator::new()))
        .manage(DiscoveryHandle::live())
        .manage(CollectionHandle::live())
        .manage(lifecycle::LifecycleHandle::new())
        .setup(|app| {
            let prefs_path = app
                .path()
                .app_config_dir()
                .map(|dir| dir.join(preferences::PREFERENCES_FILE_NAME))
                .expect("app config directory");
            let prefs = PreferencesHandle::load(prefs_path);
            {
                let coordinator = app.state::<Mutex<AppCoordinator>>();
                let mut coordinator = coordinator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                coordinator.apply_persisted_draft(&prefs.current());
                if let Some(warning) = prefs.warning() {
                    coordinator.set_preferences_warning(Some(warning));
                    coordinator.record_diagnostic(
                        dto::ErrorCode::UnsupportedInput,
                        "preferences were reset or incomplete",
                    );
                }
            }
            if let Some(window) = app.get_webview_window("main") {
                restore_window_geometry(&window, &prefs);
            }
            app.manage(DialogHandle::live(app.handle().clone()));
            app.manage(prefs);
            Ok(())
        })
        .on_window_event(|window, event| {
            let Some(prefs) = window.try_state::<PreferencesHandle>() else {
                return;
            };
            match event {
                tauri::WindowEvent::Moved(position) => {
                    prefs.update_position(position.x, position.y);
                }
                tauri::WindowEvent::Resized(size) => {
                    prefs.update_size(size.width, size.height);
                }
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let _ = prefs.persist();
                    let policy = window
                        .try_state::<Mutex<AppCoordinator>>()
                        .map(|coordinator| {
                            let coordinator = coordinator
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            let policy =
                                lifecycle::close_policy(coordinator.snapshot().collection.state);
                            if policy != lifecycle::ClosePolicy::Allow {
                                api.prevent_close();
                            }
                            policy
                        })
                        .unwrap_or(lifecycle::ClosePolicy::Allow);
                    if policy != lifecycle::ClosePolicy::Allow {
                        lifecycle::on_close_requested(window, policy);
                    }
                }
                _ => {}
            }
        });

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::state::get_app_state,
        commands::discovery::refresh_sources,
        commands::discovery::select_source,
        commands::preferences::set_sample_bits,
        commands::preferences::set_interval_seconds,
        commands::preferences::set_fold,
        commands::preferences::set_theme,
        commands::dialogs::choose_output_folder,
        commands::collection::start_collection,
        commands::collection::stop_collection,
        commands::collection::start_another_session,
        commands::collection::open_session_folder,
        commands::lifecycle::copy_diagnostics,
        commands::lifecycle::stop_and_exit,
        commands::dev::apply_dev_scenario,
    ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::state::get_app_state,
        commands::discovery::refresh_sources,
        commands::discovery::select_source,
        commands::preferences::set_sample_bits,
        commands::preferences::set_interval_seconds,
        commands::preferences::set_fold,
        commands::preferences::set_theme,
        commands::dialogs::choose_output_folder,
        commands::collection::start_collection,
        commands::collection::stop_collection,
        commands::collection::start_another_session,
        commands::collection::open_session_folder,
        commands::lifecycle::copy_diagnostics,
        commands::lifecycle::stop_and_exit,
    ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running RngKit");
}

fn restore_window_geometry<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    prefs: &PreferencesHandle,
) {
    let Some(saved) = prefs.current().window else {
        return;
    };
    let monitors: Vec<MonitorRect> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            MonitorRect {
                x: position.x,
                y: position.y,
                width: size.width,
                height: size.height,
            }
        })
        .collect();
    let clamped = clamp_geometry(saved, &monitors);
    let _ = window.set_position(PhysicalPosition::new(clamped.x, clamped.y));
    let _ = window.set_size(PhysicalSize::new(clamped.width, clamped.height));
    prefs.set_clamped_window(clamped);
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
