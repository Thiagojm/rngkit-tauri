//! Isolated device-setup folder resolution. Tests never open real folders.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use rngkit_lib::commands::device_setup::{DeviceSetupHandle, DeviceSetupOpener, RESOURCE_NAME};
use rngkit_lib::dto::ErrorCode;
use rngkit_lib::errors::SafeError;

static TEMP_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

struct RecordingOpener;

impl DeviceSetupOpener for RecordingOpener {
    fn open_folder(&self, _path: &Path) -> Result<(), SafeError> {
        Ok(())
    }
}

struct FailingOpener;

impl DeviceSetupOpener for FailingOpener {
    fn open_folder(&self, _path: &Path) -> Result<(), SafeError> {
        Err(SafeError::unexpected_failure())
    }
}

fn temp_root() -> PathBuf {
    let counter = TEMP_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "rngkit-device-setup-{}-{}-{counter}",
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
fn opens_a_regular_device_setup_directory() {
    let root = temp_root();
    let setup = root.join(RESOURCE_NAME);
    fs::create_dir_all(setup.join("windows")).expect("setup");
    let handle = DeviceSetupHandle::fake(root.clone(), Arc::new(RecordingOpener));
    handle.open_folder().expect("open");
    let opened = handle.opened();
    assert_eq!(opened.len(), 1);
    assert_eq!(
        fs::canonicalize(&opened[0]).expect("opened"),
        fs::canonicalize(&setup).expect("setup")
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn opens_when_the_resource_root_contains_spaces() {
    let parent = temp_root();
    let root = parent.join("support kit");
    fs::create_dir_all(root.join(RESOURCE_NAME)).expect("setup");
    let handle = DeviceSetupHandle::fake(root, Arc::new(RecordingOpener));
    handle.open_folder().expect("open");
    assert_eq!(handle.opened().len(), 1);
    let _ = fs::remove_dir_all(&parent);
}

#[test]
fn missing_directory_is_unavailable() {
    let root = temp_root();
    let handle = DeviceSetupHandle::fake(root.clone(), Arc::new(RecordingOpener));
    let error = handle.open_folder().expect_err("missing");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
    assert!(handle.opened().is_empty());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn file_at_setup_path_is_unavailable() {
    let root = temp_root();
    fs::write(root.join(RESOURCE_NAME), b"not a directory").expect("file");
    let handle = DeviceSetupHandle::fake(root.clone(), Arc::new(RecordingOpener));
    let error = handle.open_folder().expect_err("file");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
    assert!(handle.opened().is_empty());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unsafe_link_is_unavailable() {
    let parent = temp_root();
    let root = parent.join("root");
    let outside = parent.join("outside");
    fs::create_dir_all(&root).expect("root");
    fs::create_dir_all(&outside).expect("outside");
    let link = root.join(RESOURCE_NAME);
    if !try_dir_link(&outside, &link) {
        eprintln!("UNVERIFIED directory-link coverage");
        let _ = fs::remove_dir_all(&parent);
        return;
    }
    let handle = DeviceSetupHandle::fake(root, Arc::new(RecordingOpener));
    let error = handle.open_folder().expect_err("link");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
    assert!(handle.opened().is_empty());
    let _ = fs::remove_dir_all(&parent);
}

#[test]
fn opener_failure_is_unexpected() {
    let root = temp_root();
    fs::create_dir_all(root.join(RESOURCE_NAME)).expect("setup");
    let handle = DeviceSetupHandle::fake(root.clone(), Arc::new(FailingOpener));
    let error = handle.open_folder().expect_err("opener");
    assert_eq!(error.code, ErrorCode::UnexpectedFailure);
    assert!(handle.opened().is_empty());
    let _ = fs::remove_dir_all(&root);
}

fn try_dir_link(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).expect("unix dir symlink");
        true
    }
    #[cfg(windows)]
    {
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &target.to_string_lossy(),
            ])
            .status();
        matches!(status, Ok(code) if code.success())
    }
}
