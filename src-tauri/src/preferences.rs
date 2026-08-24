//! Schema-versioned atomic preferences. Candidate tokens are never stored.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rngkit_core::{Fold, IntervalSeconds, SampleBits};
use serde::{Deserialize, Serialize};

use crate::dto::ThemePreference;
use crate::errors::SafeError;

pub const SCHEMA_VERSION: u32 = 1;
pub const PREFERENCES_FILE_NAME: &str = "preferences.json";
pub const MIN_WINDOW_WIDTH: u32 = 800;
pub const MIN_WINDOW_HEIGHT: u32 = 600;
pub const DEFAULT_WINDOW_WIDTH: u32 = 1280;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 800;
pub const DEFAULT_OUTPUT_DIRECTORY_NAME: &str = "RngKit";

const CORRUPT_WARNING: &str =
    "Saved settings could not be restored. Default session settings are in use.";
const MISSING_FOLDER_WARNING: &str =
    "The selected output folder is unavailable. The default RngKit folder is in use.";
const DEFAULT_FOLDER_WARNING: &str =
    "The default RngKit folder is unavailable. Choose an output folder.";
const SAVE_FAILED: &str = "Settings could not be saved.";

/// Visible display rectangle used to clamp restored window geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Physical window position and size persisted with preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl WindowGeometry {
    #[must_use]
    pub fn default_window() -> Self {
        Self {
            x: 0,
            y: 0,
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
        }
    }
}

/// Persistable session draft. Candidate tokens are not included.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionDraft {
    pub sample_bits: u32,
    pub interval_seconds: u32,
    pub fold: Option<u32>,
    pub output_root: Option<PathBuf>,
    pub theme: ThemePreference,
}

impl std::fmt::Debug for SessionDraft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionDraft")
            .field("sample_bits", &self.sample_bits)
            .field("interval_seconds", &self.interval_seconds)
            .field("fold", &self.fold)
            .field(
                "output_root_label",
                &self.output_root.as_deref().map(output_root_label),
            )
            .field("theme", &self.theme)
            .finish_non_exhaustive()
    }
}

/// Validated in-memory preferences. The output path stays backend-only.
#[derive(Clone, PartialEq, Eq)]
pub struct Preferences {
    pub sample_bits: u32,
    pub interval_seconds: u32,
    pub fold: Option<u32>,
    pub output_root: Option<PathBuf>,
    pub theme: ThemePreference,
    pub window: Option<WindowGeometry>,
}

impl std::fmt::Debug for Preferences {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Preferences")
            .field("sample_bits", &self.sample_bits)
            .field("interval_seconds", &self.interval_seconds)
            .field("fold", &self.fold)
            .field(
                "output_root_label",
                &self.output_root.as_deref().map(output_root_label),
            )
            .field("has_output_root", &self.output_root.is_some())
            .field("theme", &self.theme)
            .field("window", &self.window)
            .finish()
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            sample_bits: 2048,
            interval_seconds: 1,
            fold: None,
            output_root: None,
            theme: ThemePreference::System,
            window: None,
        }
    }
}

/// Result of reading a preferences file. Missing files are not a warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadOutcome {
    pub preferences: Preferences,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreferencesFile {
    schema_version: u32,
    output_root: Option<PathBuf>,
    sample_bits: u32,
    interval_seconds: u32,
    fold: Option<u32>,
    theme: ThemePreference,
    #[serde(default)]
    window: Option<WindowGeometry>,
}

/// Process-owned preferences file with atomic replace.
pub struct PreferencesHandle {
    path: PathBuf,
    inner: Mutex<LoadOutcome>,
}

impl PreferencesHandle {
    #[must_use]
    pub fn load(path: PathBuf) -> Self {
        let outcome = load_from_path(&path);
        Self {
            path,
            inner: Mutex::new(outcome),
        }
    }

    /// Load preferences and prepare the safe default output root when no valid
    /// saved root is available. The resolver is injected so startup behavior
    /// remains deterministic and filesystem failures are testable.
    #[must_use]
    pub fn load_with_documents<F>(path: PathBuf, resolve_documents: F) -> Self
    where
        F: FnOnce() -> Result<PathBuf, SafeError>,
    {
        let mut outcome = load_from_path(&path);
        if outcome.preferences.output_root.is_none() {
            match create_default_output_root(resolve_documents) {
                Ok(root) => outcome.preferences.output_root = Some(root),
                Err(_) if outcome.warning.is_none() => {
                    outcome.warning = Some(DEFAULT_FOLDER_WARNING.into());
                }
                Err(_) => {}
            }
        }
        Self {
            path,
            inner: Mutex::new(outcome),
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn current(&self) -> Preferences {
        self.lock().preferences.clone()
    }

    #[must_use]
    pub fn warning(&self) -> Option<String> {
        self.lock().warning.clone()
    }

    pub fn save_draft(&self, draft: SessionDraft) -> Result<(), SafeError> {
        let mut outcome = self.lock();
        let mut proposed = outcome.preferences.clone();
        proposed.sample_bits = draft.sample_bits;
        proposed.interval_seconds = draft.interval_seconds;
        proposed.fold = draft.fold;
        proposed.output_root = draft.output_root.clone();
        proposed.theme = draft.theme;
        persist(&self.path, &proposed)?;
        outcome.preferences = proposed;
        if draft.output_root.is_some() {
            outcome.warning = None;
        }
        Ok(())
    }

    pub fn update_position(&self, x: i32, y: i32) {
        let mut outcome = self.lock();
        let mut window = outcome
            .preferences
            .window
            .unwrap_or_else(WindowGeometry::default_window);
        window.x = x;
        window.y = y;
        outcome.preferences.window = Some(window);
    }

    pub fn update_size(&self, width: u32, height: u32) {
        let mut outcome = self.lock();
        let mut window = outcome
            .preferences
            .window
            .unwrap_or_else(WindowGeometry::default_window);
        window.width = width.max(MIN_WINDOW_WIDTH);
        window.height = height.max(MIN_WINDOW_HEIGHT);
        outcome.preferences.window = Some(window);
    }

    pub fn set_clamped_window(&self, geometry: WindowGeometry) {
        self.lock().preferences.window = Some(geometry);
    }

    pub fn persist(&self) -> Result<(), SafeError> {
        let outcome = self.lock();
        persist(&self.path, &outcome.preferences)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, LoadOutcome> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[must_use]
pub fn load_from_path(path: &Path) -> LoadOutcome {
    if !path.exists() {
        return LoadOutcome {
            preferences: Preferences::default(),
            warning: None,
        };
    }
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return LoadOutcome {
                preferences: Preferences::default(),
                warning: Some(CORRUPT_WARNING.into()),
            };
        }
    };
    match serde_json::from_slice::<PreferencesFile>(&bytes) {
        Ok(file) if file.schema_version == SCHEMA_VERSION => from_file(file),
        Ok(_) => LoadOutcome {
            preferences: Preferences::default(),
            warning: Some(CORRUPT_WARNING.into()),
        },
        Err(_) => LoadOutcome {
            preferences: Preferences::default(),
            warning: Some(CORRUPT_WARNING.into()),
        },
    }
}

fn from_file(file: PreferencesFile) -> LoadOutcome {
    let Ok(sample_bits) = SampleBits::new(file.sample_bits) else {
        return corrupt_outcome();
    };
    let Ok(interval_seconds) = IntervalSeconds::new(file.interval_seconds) else {
        return corrupt_outcome();
    };
    let fold = match file.fold {
        None => None,
        Some(value) => {
            let Some(fold) = u8::try_from(value)
                .ok()
                .and_then(|fold| Fold::new(fold).ok())
            else {
                return corrupt_outcome();
            };
            Some(u32::from(fold.get()))
        }
    };
    let mut preferences = Preferences {
        sample_bits: sample_bits.get(),
        interval_seconds: interval_seconds.get(),
        fold,
        theme: file.theme,
        output_root: None,
        window: file.window.map(sanitize_window),
    };
    let mut warning = None;

    if let Some(root) = file.output_root {
        match validate_output_root(&root) {
            Ok(path) => preferences.output_root = Some(path),
            Err(_) => {
                if warning.is_none() {
                    warning = Some(MISSING_FOLDER_WARNING.into());
                }
            }
        }
    }

    LoadOutcome {
        preferences,
        warning,
    }
}

fn corrupt_outcome() -> LoadOutcome {
    LoadOutcome {
        preferences: Preferences::default(),
        warning: Some(CORRUPT_WARNING.into()),
    }
}

fn sanitize_window(geometry: WindowGeometry) -> WindowGeometry {
    WindowGeometry {
        x: geometry.x,
        y: geometry.y,
        width: geometry.width.max(MIN_WINDOW_WIDTH),
        height: geometry.height.max(MIN_WINDOW_HEIGHT),
    }
}

/// Keep the restored window on a visible monitor. Empty monitor lists keep size minima.
#[must_use]
pub fn clamp_geometry(saved: WindowGeometry, monitors: &[MonitorRect]) -> WindowGeometry {
    let mut width = saved.width.max(MIN_WINDOW_WIDTH);
    let mut height = saved.height.max(MIN_WINDOW_HEIGHT);
    if monitors.is_empty() {
        return WindowGeometry {
            x: saved.x,
            y: saved.y,
            width,
            height,
        };
    }

    let containing = monitors
        .iter()
        .find(|monitor| contains_point(monitor, saved.x, saved.y));
    let target = containing.copied().unwrap_or(monitors[0]);
    if target.width > 0 {
        width = width.min(target.width.max(MIN_WINDOW_WIDTH));
    }
    if target.height > 0 {
        height = height.min(target.height.max(MIN_WINDOW_HEIGHT));
    }

    let max_x = target
        .x
        .saturating_add_unsigned(target.width.saturating_sub(width));
    let max_y = target
        .y
        .saturating_add_unsigned(target.height.saturating_sub(height));
    let x = if containing.is_some() {
        saved.x.clamp(target.x, max_x.max(target.x))
    } else {
        target.x
    };
    let y = if containing.is_some() {
        saved.y.clamp(target.y, max_y.max(target.y))
    } else {
        target.y
    };

    WindowGeometry {
        x,
        y,
        width,
        height,
    }
}

fn contains_point(monitor: &MonitorRect, x: i32, y: i32) -> bool {
    let right = monitor.x.saturating_add_unsigned(monitor.width);
    let bottom = monitor.y.saturating_add_unsigned(monitor.height);
    x >= monitor.x && y >= monitor.y && x < right && y < bottom
}

/// Confirm the path exists and is a directory. The path never enters a DTO.
pub fn validate_output_root(path: &Path) -> Result<PathBuf, SafeError> {
    let metadata = fs::metadata(path).map_err(|_| {
        SafeError::invalid_configuration("The selected output folder is not available.")
    })?;
    if !metadata.is_dir() {
        return Err(SafeError::invalid_configuration(
            "The selected output folder is not available.",
        ));
    }
    Ok(path.to_path_buf())
}

fn create_default_output_root<F>(resolve_documents: F) -> Result<PathBuf, SafeError>
where
    F: FnOnce() -> Result<PathBuf, SafeError>,
{
    let documents =
        resolve_documents().map_err(|_| SafeError::permission_denied(DEFAULT_FOLDER_WARNING))?;
    let root = documents.join(DEFAULT_OUTPUT_DIRECTORY_NAME);
    fs::create_dir_all(&root).map_err(|_| SafeError::permission_denied(DEFAULT_FOLDER_WARNING))?;
    validate_output_root(&root)
}

#[must_use]
pub fn output_root_label(path: &Path) -> String {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(name) if !name.is_empty() && !looks_like_path(name) => name.to_owned(),
        _ => "Selected folder".into(),
    }
}

fn looks_like_path(label: &str) -> bool {
    label.contains(":\\") || label.contains('/') || label.contains('\\') || label.starts_with("COM")
}

fn persist(path: &Path, preferences: &Preferences) -> Result<(), SafeError> {
    let file = PreferencesFile {
        schema_version: SCHEMA_VERSION,
        output_root: preferences.output_root.clone(),
        sample_bits: preferences.sample_bits,
        interval_seconds: preferences.interval_seconds,
        fold: preferences.fold,
        theme: preferences.theme,
        window: preferences.window,
    };
    let bytes = serde_json::to_vec_pretty(&file).map_err(|_| SafeError::unexpected_failure())?;
    atomic_write(path, &bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), SafeError> {
    let parent = path
        .parent()
        .ok_or_else(|| SafeError::permission_denied(SAVE_FAILED))?;
    fs::create_dir_all(parent).map_err(|_| SafeError::permission_denied(SAVE_FAILED))?;
    let tmp = parent.join("preferences.json.tmp");
    write_tmp(&tmp, bytes)?;
    replace_file(&tmp, path)
}

fn write_tmp(tmp: &Path, bytes: &[u8]) -> Result<(), SafeError> {
    let mut file = File::create(tmp).map_err(|_| SafeError::permission_denied(SAVE_FAILED))?;
    file.write_all(bytes)
        .map_err(|_| SafeError::permission_denied(SAVE_FAILED))?;
    file.sync_all()
        .map_err(|_| SafeError::permission_denied(SAVE_FAILED))?;
    Ok(())
}

#[cfg(windows)]
fn replace_file(tmp: &Path, dest: &Path) -> Result<(), SafeError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{REPLACEFILE_WRITE_THROUGH, ReplaceFileW};

    if !dest.exists() {
        return fs::rename(tmp, dest).map_err(|_| SafeError::permission_denied(SAVE_FAILED));
    }

    let replaced: Vec<u16> = dest.as_os_str().encode_wide().chain(Some(0)).collect();
    let replacement: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: Both paths are NUL-terminated for the duration of the call. The optional
    // backup, exclusion, and reserved pointers are null as required by ReplaceFileW.
    let result = unsafe {
        ReplaceFileW(
            replaced.as_ptr(),
            replacement.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    if result == 0 {
        let _ = fs::remove_file(tmp);
        Err(SafeError::permission_denied(SAVE_FAILED))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(tmp: &Path, dest: &Path) -> Result<(), SafeError> {
    fs::rename(tmp, dest).map_err(|_| SafeError::permission_denied(SAVE_FAILED))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rngkit-pref-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("temp");
        dir
    }

    #[test]
    fn missing_file_uses_defaults_without_warning() {
        let path = temp_dir().join(PREFERENCES_FILE_NAME);
        let outcome = load_from_path(&path);
        assert_eq!(outcome.preferences, Preferences::default());
        assert!(outcome.warning.is_none());
    }

    #[test]
    fn corrupt_json_resets_with_warning() {
        let dir = temp_dir();
        let path = dir.join(PREFERENCES_FILE_NAME);
        fs::write(&path, b"{not json").expect("write");
        let outcome = load_from_path(&path);
        assert_eq!(outcome.preferences, Preferences::default());
        assert_eq!(outcome.warning.as_deref(), Some(CORRUPT_WARNING));
    }

    #[test]
    fn unknown_schema_resets_with_warning() {
        let dir = temp_dir();
        let path = dir.join(PREFERENCES_FILE_NAME);
        let payload = json!({
            "schemaVersion": 99,
            "outputRoot": null,
            "sampleBits": 8,
            "intervalSeconds": 1,
            "fold": null,
            "theme": "dark"
        });
        fs::write(&path, serde_json::to_vec(&payload).expect("json")).expect("write");
        let outcome = load_from_path(&path);
        assert_eq!(outcome.preferences.theme, ThemePreference::System);
        assert_eq!(outcome.warning.as_deref(), Some(CORRUPT_WARNING));
    }

    #[test]
    fn extra_token_fields_are_rejected() {
        let dir = temp_dir();
        let path = dir.join(PREFERENCES_FILE_NAME);
        let payload = json!({
            "schemaVersion": 1,
            "outputRoot": null,
            "sampleBits": 8,
            "intervalSeconds": 1,
            "fold": null,
            "theme": "system",
            "selectedToken": "mock-bitb-1",
            "sourceId": "bitb"
        });
        fs::write(&path, serde_json::to_vec(&payload).expect("json")).expect("write");
        let outcome = load_from_path(&path);
        assert_eq!(outcome.preferences, Preferences::default());
        assert_eq!(outcome.warning.as_deref(), Some(CORRUPT_WARNING));
    }

    #[test]
    fn invalid_semantic_field_resets_the_entire_file() {
        let dir = temp_dir();
        let folder = dir.join("sessions");
        fs::create_dir_all(&folder).expect("folder");
        let path = dir.join(PREFERENCES_FILE_NAME);
        let payload = json!({
            "schemaVersion": 1,
            "outputRoot": folder,
            "sampleBits": 7,
            "intervalSeconds": 3,
            "fold": 2,
            "theme": "dark"
        });
        fs::write(&path, serde_json::to_vec(&payload).expect("json")).expect("write");

        let outcome = load_from_path(&path);

        assert_eq!(outcome.preferences, Preferences::default());
        assert_eq!(outcome.warning.as_deref(), Some(CORRUPT_WARNING));
    }

    #[test]
    fn atomic_write_round_trip_omits_tokens() {
        let dir = temp_dir();
        let folder = dir.join("sessions");
        fs::create_dir_all(&folder).expect("folder");
        let path = dir.join(PREFERENCES_FILE_NAME);
        let preferences = Preferences {
            sample_bits: 16,
            interval_seconds: 2,
            fold: Some(1),
            theme: ThemePreference::Dark,
            output_root: Some(folder.clone()),
            window: None,
        };
        persist(&path, &preferences).expect("save");

        let outcome = load_from_path(&path);
        assert!(outcome.warning.is_none());
        assert_eq!(outcome.preferences.sample_bits, 16);
        assert_eq!(outcome.preferences.interval_seconds, 2);
        assert_eq!(outcome.preferences.fold, Some(1));
        assert_eq!(outcome.preferences.theme, ThemePreference::Dark);
        assert_eq!(
            outcome.preferences.output_root.as_deref(),
            Some(folder.as_path())
        );

        let raw = fs::read_to_string(&path).expect("read");
        assert!(!raw.contains("selectedToken"));
        assert!(!raw.contains("token"));
        assert!(!raw.contains("sourceId"));
        assert!(!raw.contains("family"));
        assert!(!path.with_extension("json.tmp").exists());
        assert!(!path.with_extension("json.bak").exists());
    }

    #[test]
    fn failed_replace_preserves_the_existing_preferences() {
        let dir = temp_dir();
        let path = dir.join(PREFERENCES_FILE_NAME);
        fs::write(&path, b"original").expect("write original");
        let missing_tmp = dir.join("missing.tmp");

        replace_file(&missing_tmp, &path).expect_err("replace must fail");

        assert_eq!(fs::read(&path).expect("read original"), b"original");
    }

    #[test]
    fn failed_save_does_not_mutate_in_memory_preferences() {
        let dir = temp_dir();
        let blocking_file = dir.join("not-a-directory");
        fs::write(&blocking_file, b"block").expect("blocker");
        let handle = PreferencesHandle::load(blocking_file.join(PREFERENCES_FILE_NAME));
        let before = handle.current();
        let draft = SessionDraft {
            sample_bits: 16,
            interval_seconds: 2,
            fold: Some(1),
            output_root: None,
            theme: ThemePreference::Dark,
        };

        handle.save_draft(draft).expect_err("save must fail");

        assert_eq!(handle.current(), before);
    }

    #[test]
    fn missing_output_directory_is_dropped() {
        let dir = temp_dir();
        let path = dir.join(PREFERENCES_FILE_NAME);
        let payload = json!({
            "schemaVersion": 1,
            "outputRoot": dir.join("gone"),
            "sampleBits": 24,
            "intervalSeconds": 3,
            "fold": 2,
            "theme": "light"
        });
        fs::write(&path, serde_json::to_vec(&payload).expect("json")).expect("write");
        let outcome = load_from_path(&path);
        assert!(outcome.preferences.output_root.is_none());
        assert_eq!(outcome.preferences.sample_bits, 24);
        assert_eq!(outcome.preferences.theme, ThemePreference::Light);
        assert_eq!(outcome.warning.as_deref(), Some(MISSING_FOLDER_WARNING));
    }

    #[test]
    fn clamp_moves_offscreen_window_onto_a_monitor() {
        let monitors = [MonitorRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }];
        let saved = WindowGeometry {
            x: 4000,
            y: -200,
            width: 900,
            height: 700,
        };
        let clamped = clamp_geometry(saved, &monitors);
        assert_eq!(clamped.x, 0);
        assert_eq!(clamped.y, 0);
        assert_eq!(clamped.width, 900);
        assert_eq!(clamped.height, 700);
    }

    #[test]
    fn clamp_keeps_physical_coordinates_on_a_secondary_monitor() {
        let monitors = [MonitorRect {
            x: 1920,
            y: 0,
            width: 1920,
            height: 1080,
        }];
        let saved = WindowGeometry {
            x: 2000,
            y: 40,
            width: 1000,
            height: 800,
        };
        let clamped = clamp_geometry(saved, &monitors);
        assert_eq!(clamped.x, 2000);
        assert_eq!(clamped.y, 40);
    }

    #[test]
    fn output_root_label_never_includes_a_path() {
        let path = PathBuf::from("Users").join("dev").join("Sessions");
        assert_eq!(output_root_label(&path), "Sessions");
        let windows_shaped = output_root_label(Path::new(r"C:\Users\dev\Sessions"));
        assert!(!windows_shaped.contains(":\\"));
        assert!(!windows_shaped.contains('\\'));
        assert_eq!(output_root_label(Path::new("/")), "Selected folder");
        #[cfg(windows)]
        {
            assert_eq!(windows_shaped, "Sessions");
            assert_eq!(output_root_label(Path::new("D:\\")), "Selected folder");
        }
    }
}
