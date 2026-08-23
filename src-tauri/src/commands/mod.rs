//! Tauri command handlers. Command functions translate IPC to coordinator calls.

pub mod collection;
pub mod dialogs;
pub mod discovery;
pub mod preferences;
pub mod state;

#[cfg(debug_assertions)]
pub mod dev;

use crate::coordinator::AppCoordinator;
use crate::dto::AppStateDto;
use crate::errors::SafeError;
use crate::preferences::PreferencesHandle;

pub(crate) fn persist_session_draft(
    prefs: &PreferencesHandle,
    coordinator: &AppCoordinator,
) -> Result<(), SafeError> {
    prefs.save_draft(coordinator.session_draft())
}

pub(crate) fn update_and_persist_session_draft(
    prefs: &PreferencesHandle,
    coordinator: &mut AppCoordinator,
    update: impl FnOnce(&mut AppCoordinator) -> Result<(), SafeError>,
) -> Result<AppStateDto, SafeError> {
    let previous = coordinator.session_draft();
    update(coordinator)?;
    if let Err(error) = persist_session_draft(prefs, coordinator) {
        coordinator.restore_session_draft(&previous);
        return Err(error);
    }
    Ok(coordinator.snapshot())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn failed_persistence_rolls_back_the_authoritative_coordinator() {
        let root = std::env::temp_dir().join(format!(
            "rngkit-command-prefs-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&root).expect("temp");
        let blocking_file = root.join("not-a-directory");
        fs::write(&blocking_file, b"block").expect("blocker");
        let prefs = PreferencesHandle::load(blocking_file.join("preferences.json"));
        let mut coordinator = AppCoordinator::new();
        let before = coordinator.session_draft();

        update_and_persist_session_draft(&prefs, &mut coordinator, |coordinator| {
            coordinator.set_sample_bits(16)
        })
        .expect_err("save must fail");

        assert_eq!(coordinator.session_draft(), before);
    }
}
