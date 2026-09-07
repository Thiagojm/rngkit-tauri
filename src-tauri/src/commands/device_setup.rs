//! Open the packaged device-setup folder. Never accepts a frontend path.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime, State};

use crate::coordinator::AppCoordinator;
use crate::dto::AppStateDto;
use crate::errors::SafeError;
use crate::reports::spawn_open;

pub const RESOURCE_NAME: &str = "device-setup";

pub trait DeviceSetupResolver: Send + Sync {
    fn resource_root(&self) -> Result<PathBuf, SafeError>;
    fn setup_dir(&self) -> Result<PathBuf, SafeError>;
}

pub trait DeviceSetupOpener: Send + Sync {
    fn open_folder(&self, path: &Path) -> Result<(), SafeError>;
}

#[derive(Clone)]
pub struct DeviceSetupHandle {
    resolver: Arc<dyn DeviceSetupResolver>,
    opener: Arc<dyn DeviceSetupOpener>,
    opened: Arc<Mutex<Vec<PathBuf>>>,
}

impl DeviceSetupHandle {
    #[must_use]
    pub fn live<R: Runtime>(app: AppHandle<R>) -> Self {
        Self {
            resolver: Arc::new(TauriResourceResolver { app }),
            opener: Arc::new(LiveDeviceSetupOpener),
            opened: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[must_use]
    pub fn fake(root: PathBuf, opener: Arc<dyn DeviceSetupOpener>) -> Self {
        Self {
            resolver: Arc::new(FixedResolver { root }),
            opener,
            opened: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn open_folder(&self) -> Result<(), SafeError> {
        let root = self.resolver.resource_root()?;
        let setup = self.resolver.setup_dir()?;
        let validated = validate_setup_dir(&root, &setup)?;
        self.opener.open_folder(&validated)?;
        self.opened
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(validated);
        Ok(())
    }

    #[must_use]
    pub fn opened(&self) -> Vec<PathBuf> {
        self.opened
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[tauri::command]
pub fn open_device_setup_folder(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    device_setup: State<'_, DeviceSetupHandle>,
) -> Result<AppStateDto, SafeError> {
    device_setup.open_folder()?;
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(coordinator.snapshot())
}

fn unavailable() -> SafeError {
    SafeError::invalid_configuration("The device setup folder is unavailable.")
        .with_recovery("Reinstall RngKit if the support files are missing.")
}

fn validate_setup_dir(resource_root: &Path, setup_dir: &Path) -> Result<PathBuf, SafeError> {
    if !setup_dir.is_absolute() {
        return Err(unavailable());
    }
    if setup_dir.file_name() != Some(OsStr::new(RESOURCE_NAME)) {
        return Err(unavailable());
    }
    let metadata = fs::symlink_metadata(setup_dir).map_err(|_| unavailable())?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(unavailable());
    }
    let canonical_root = fs::canonicalize(resource_root).map_err(|_| unavailable())?;
    let canonical_setup = fs::canonicalize(setup_dir).map_err(|_| unavailable())?;
    if canonical_setup.file_name() != Some(OsStr::new(RESOURCE_NAME)) {
        return Err(unavailable());
    }
    let Some(parent) = canonical_setup.parent() else {
        return Err(unavailable());
    };
    if parent != canonical_root.as_path() {
        return Err(unavailable());
    }
    Ok(canonical_setup)
}

struct TauriResourceResolver<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> DeviceSetupResolver for TauriResourceResolver<R> {
    fn resource_root(&self) -> Result<PathBuf, SafeError> {
        self.app.path().resource_dir().map_err(|_| unavailable())
    }

    fn setup_dir(&self) -> Result<PathBuf, SafeError> {
        self.app
            .path()
            .resolve(RESOURCE_NAME, BaseDirectory::Resource)
            .map_err(|_| unavailable())
    }
}

struct FixedResolver {
    root: PathBuf,
}

impl DeviceSetupResolver for FixedResolver {
    fn resource_root(&self) -> Result<PathBuf, SafeError> {
        Ok(self.root.clone())
    }

    fn setup_dir(&self) -> Result<PathBuf, SafeError> {
        Ok(self.root.join(RESOURCE_NAME))
    }
}

struct LiveDeviceSetupOpener;

impl DeviceSetupOpener for LiveDeviceSetupOpener {
    fn open_folder(&self, path: &Path) -> Result<(), SafeError> {
        spawn_open(path)
    }
}
