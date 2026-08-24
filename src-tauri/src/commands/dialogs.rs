//! Native directory dialog. The selected path stays in Rust.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Runtime, State};
use tauri_plugin_dialog::DialogExt;

use crate::coordinator::AppCoordinator;
use crate::dto::AppStateDto;
use crate::errors::SafeError;
use crate::preferences::PreferencesHandle;

use super::update_and_persist_session_draft;

pub trait FolderPicker: Send + Sync {
    fn pick_folder(&self, title: &str, current: Option<&Path>) -> Option<PathBuf>;
    fn pick_file(&self, title: &str, current: Option<&Path>) -> Option<PathBuf>;
    fn pick_files(&self, title: &str, current: Option<&Path>) -> Option<Vec<PathBuf>>;
}

pub struct LiveFolderPicker<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> LiveFolderPicker<R> {
    #[must_use]
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> FolderPicker for LiveFolderPicker<R> {
    fn pick_folder(&self, title: &str, current: Option<&Path>) -> Option<PathBuf> {
        let mut builder = self.app.dialog().file().set_title(title);
        if let Some(directory) = current.filter(|path| path.is_dir()) {
            builder = builder.set_directory(directory);
        }
        builder.blocking_pick_folder()?.into_path().ok()
    }

    fn pick_file(&self, title: &str, current: Option<&Path>) -> Option<PathBuf> {
        let mut builder = self
            .app
            .dialog()
            .file()
            .set_title(title)
            .add_filter("RngKitPSG v3", &["bin", "csv"]);
        if let Some(directory) = current.filter(|path| path.is_dir()) {
            builder = builder.set_directory(directory);
        }
        builder.blocking_pick_file()?.into_path().ok()
    }

    fn pick_files(&self, title: &str, current: Option<&Path>) -> Option<Vec<PathBuf>> {
        let mut builder = self
            .app
            .dialog()
            .file()
            .set_title(title)
            .add_filter("RngKitPSG v3 CSV", &["csv"]);
        if let Some(directory) = current.filter(|path| path.is_dir()) {
            builder = builder.set_directory(directory);
        }
        let files = builder.blocking_pick_files()?;
        let mut paths = Vec::with_capacity(files.len());
        for file in files {
            paths.push(file.into_path().ok()?);
        }
        Some(paths)
    }
}

#[derive(Clone)]
pub struct DialogHandle {
    inner: Arc<dyn FolderPicker>,
}

impl DialogHandle {
    #[must_use]
    pub fn live<R: Runtime>(app: AppHandle<R>) -> Self {
        Self {
            inner: Arc::new(LiveFolderPicker::new(app)),
        }
    }

    #[must_use]
    pub fn fake(picker: FakeFolderPicker) -> Self {
        Self {
            inner: Arc::new(picker),
        }
    }

    #[must_use]
    pub fn pick_folder(&self, title: &str, current: Option<&Path>) -> Option<PathBuf> {
        self.inner.pick_folder(title, current)
    }

    #[must_use]
    pub fn pick_file(&self, title: &str, current: Option<&Path>) -> Option<PathBuf> {
        self.inner.pick_file(title, current)
    }

    #[must_use]
    pub fn pick_files(&self, title: &str, current: Option<&Path>) -> Option<Vec<PathBuf>> {
        self.inner.pick_files(title, current)
    }
}

#[derive(Default)]
pub struct FakeFolderPicker {
    next: Mutex<Option<Vec<PathBuf>>>,
}

impl FakeFolderPicker {
    #[must_use]
    pub fn with_folder(path: PathBuf) -> Self {
        Self {
            next: Mutex::new(Some(vec![path])),
        }
    }

    #[must_use]
    pub fn with_files(paths: Vec<PathBuf>) -> Self {
        Self {
            next: Mutex::new(Some(paths)),
        }
    }

    #[must_use]
    pub fn cancelled() -> Self {
        Self {
            next: Mutex::new(None),
        }
    }
}

impl FolderPicker for FakeFolderPicker {
    fn pick_folder(&self, _title: &str, _current: Option<&Path>) -> Option<PathBuf> {
        self.next
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .and_then(|mut paths| paths.pop())
    }

    fn pick_file(&self, title: &str, current: Option<&Path>) -> Option<PathBuf> {
        self.pick_folder(title, current)
    }

    fn pick_files(&self, _title: &str, _current: Option<&Path>) -> Option<Vec<PathBuf>> {
        self.next
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

pub fn apply_picked_folder(
    coordinator: &mut AppCoordinator,
    picker: &dyn FolderPicker,
) -> Result<AppStateDto, SafeError> {
    coordinator.ensure_configurable()?;
    let current = coordinator.output_root().map(Path::to_path_buf);
    match picker.pick_folder("Choose output folder", current.as_deref()) {
        Some(path) => {
            coordinator.set_output_root(&path)?;
            Ok(coordinator.snapshot())
        }
        None => Ok(coordinator.snapshot()),
    }
}

#[tauri::command]
pub async fn choose_output_folder(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    dialogs: State<'_, DialogHandle>,
    prefs: State<'_, PreferencesHandle>,
) -> Result<AppStateDto, SafeError> {
    let current = {
        let coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.ensure_configurable()?;
        coordinator.output_root().map(Path::to_path_buf)
    };

    let handle = (*dialogs).clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        handle.pick_folder("Choose output folder", current.as_deref())
    })
    .await
    .map_err(|_| SafeError::unexpected_failure())?;

    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(path) = picked {
        return update_and_persist_session_draft(&prefs, &mut coordinator, |coordinator| {
            coordinator.set_output_root(&path)
        });
    }
    Ok(coordinator.snapshot())
}
