//! Collection worker, fake sources, and narrowly scoped session-folder opener.

mod sink;
mod worker;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use rngkit_core::{EntropySource, SampleBits, SourceDescriptor, SourceError, SourceErrorKind};
use rngkit_engine::{CancelToken, EngineError};
use rngkit_sources::{OpenedSource, SourceConfig};
use tauri::Manager;

use crate::coordinator::AppCoordinator;
use crate::dto::{CollectionEventDto, CollectionState};
use crate::errors::SafeError;

pub use sink::{FailingSender, VecSender};
pub use worker::run_collection;

/// Sends sequenced collection events. Channel failure is terminal.
pub trait EventSender: Send + Sync {
    fn send_event(&self, event: CollectionEventDto) -> Result<(), EngineError>;
}

/// Opens one entropy source for a reconstructed [`SourceConfig`].
pub trait SourceOpener: Send + Sync {
    fn open(
        &self,
        config: &SourceConfig,
        cancel: &CancelToken,
    ) -> Result<Box<dyn EntropySource>, SafeError>;
}

/// Opens a backend-known session directory. Never accepts a frontend path.
pub trait FolderOpener: Send + Sync {
    fn open_folder(&self, path: &Path) -> Result<(), SafeError>;
}

struct WorkerSlot {
    cancel: Option<CancelToken>,
    handle: Option<JoinHandle<()>>,
}

/// Shared collection runtime stored in Tauri state.
#[derive(Clone)]
pub struct CollectionHandle {
    opener: Arc<dyn SourceOpener>,
    folder: Arc<dyn FolderOpener>,
    use_fake_clock: bool,
    slot: Arc<Mutex<WorkerSlot>>,
    opened_folders: Arc<Mutex<Vec<PathBuf>>>,
}

impl CollectionHandle {
    #[must_use]
    pub fn live() -> Self {
        Self::new(
            Arc::new(LiveSourceOpener),
            Arc::new(LiveFolderOpener),
            false,
        )
    }

    /// Deterministic tests: fake source, fake clock, no hardware.
    #[must_use]
    pub fn fake() -> Self {
        Self::new(
            Arc::new(FakeSourceOpener::default()),
            Arc::new(RecordingFolderOpener),
            true,
        )
    }

    /// Real PseudoRNG adapter with a fake clock. Cancels after `max_samples`.
    /// Does not enumerate or open hardware.
    #[must_use]
    pub fn pseudo_for_tests(max_samples: u32) -> Self {
        Self::new(
            Arc::new(PseudoTestOpener { max_samples }),
            Arc::new(RecordingFolderOpener),
            true,
        )
    }

    #[must_use]
    pub fn failing_open(kind: SourceErrorKind) -> Self {
        Self::new(
            Arc::new(FailingSourceOpener { kind }),
            Arc::new(RecordingFolderOpener),
            true,
        )
    }

    fn new(
        opener: Arc<dyn SourceOpener>,
        folder: Arc<dyn FolderOpener>,
        use_fake_clock: bool,
    ) -> Self {
        Self {
            opener,
            folder,
            use_fake_clock,
            slot: Arc::new(Mutex::new(WorkerSlot {
                cancel: None,
                handle: None,
            })),
            opened_folders: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn join_previous(&self) {
        let handle = {
            let mut slot = self
                .slot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            slot.cancel = None;
            slot.handle.take()
        };
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }

    pub fn request_cancel(&self) {
        let slot = self
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(cancel) = &slot.cancel {
            cancel.cancel();
        }
    }

    pub fn spawn_worker<R: tauri::Runtime>(
        &self,
        app: tauri::AppHandle<R>,
        sender: Arc<dyn EventSender>,
        plan: crate::coordinator::CollectionStart,
        cancel: CancelToken,
    ) -> Result<(), SafeError> {
        self.join_previous();
        let opener = Arc::clone(&self.opener);
        let use_fake_clock = self.use_fake_clock;
        let worker_cancel = cancel.clone();
        let handle = std::thread::Builder::new()
            .name("rngkit-collect".into())
            .spawn(move || {
                let coordinator = app.state::<Mutex<AppCoordinator>>();
                run_collection(
                    coordinator.inner(),
                    opener.as_ref(),
                    use_fake_clock,
                    sender.as_ref(),
                    &worker_cancel,
                    plan,
                );
                crate::lifecycle::on_worker_finished(&app);
            })
            .map_err(|_| SafeError::unexpected_failure())?;
        let mut slot = self
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slot.cancel = Some(cancel);
        slot.handle = Some(handle);
        Ok(())
    }

    pub fn run_blocking(
        &self,
        coordinator: &Mutex<AppCoordinator>,
        sender: &dyn EventSender,
        plan: crate::coordinator::CollectionStart,
    ) {
        let cancel = CancelToken::new();
        {
            let mut slot = self
                .slot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            slot.cancel = Some(cancel.clone());
        }
        run_collection(
            coordinator,
            self.opener.as_ref(),
            self.use_fake_clock,
            sender,
            &cancel,
            plan,
        );
        let mut slot = self
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slot.cancel = None;
    }

    pub fn open_known_session_folder(&self, coordinator: &AppCoordinator) -> Result<(), SafeError> {
        match coordinator.snapshot().collection.state {
            CollectionState::Completed | CollectionState::Failed => {}
            _ => {
                return Err(SafeError::invalid_transition(
                    "Open the session folder after collection finishes.",
                ));
            }
        }
        let path = coordinator
            .session_directory()
            .ok_or_else(|| SafeError::invalid_configuration("No session folder is available."))?;
        self.folder.open_folder(path)?;
        self.opened_folders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(path.to_path_buf());
        Ok(())
    }

    #[must_use]
    pub fn opened_folders(&self) -> Vec<PathBuf> {
        self.opened_folders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

struct LiveSourceOpener;

impl SourceOpener for LiveSourceOpener {
    fn open(
        &self,
        config: &SourceConfig,
        _cancel: &CancelToken,
    ) -> Result<Box<dyn EntropySource>, SafeError> {
        match config {
            SourceConfig::Bitb { serial: None, .. } | SourceConfig::Trng { path: None } => {
                return Err(SafeError::source_unavailable());
            }
            _ => {}
        }
        rngkit_sources::open(config.clone())
            .map(|source| Box::new(source) as Box<dyn EntropySource>)
            .map_err(|error| map_source_kind(error.kind()))
    }
}

struct LiveFolderOpener;

impl FolderOpener for LiveFolderOpener {
    fn open_folder(&self, path: &Path) -> Result<(), SafeError> {
        let mut command = if cfg!(windows) {
            std::process::Command::new("explorer")
        } else {
            std::process::Command::new("xdg-open")
        };
        command
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|_| SafeError::unexpected_failure())
    }
}

struct RecordingFolderOpener;

impl FolderOpener for RecordingFolderOpener {
    fn open_folder(&self, _path: &Path) -> Result<(), SafeError> {
        Ok(())
    }
}

struct FakeSourceOpener {
    samples: Vec<Vec<u8>>,
    fail: Option<SourceErrorKind>,
}

impl Default for FakeSourceOpener {
    fn default() -> Self {
        Self {
            samples: vec![vec![0xFF], vec![0x00], vec![0xAA]],
            fail: None,
        }
    }
}

impl SourceOpener for FakeSourceOpener {
    fn open(
        &self,
        config: &SourceConfig,
        cancel: &CancelToken,
    ) -> Result<Box<dyn EntropySource>, SafeError> {
        let descriptor = descriptor_for(config)?;
        Ok(Box::new(FakeSource {
            descriptor,
            samples: self.samples.clone(),
            index: 0,
            cancel: cancel.clone(),
            fail: self.fail,
        }))
    }
}

struct FakeSource {
    descriptor: SourceDescriptor,
    samples: Vec<Vec<u8>>,
    index: usize,
    cancel: CancelToken,
    fail: Option<SourceErrorKind>,
}

impl EntropySource for FakeSource {
    fn descriptor(&self) -> &SourceDescriptor {
        &self.descriptor
    }

    fn read_bits(&mut self, bits: SampleBits) -> Result<Vec<u8>, SourceError> {
        if let Some(kind) = self.fail {
            return Err(SourceError::new(kind, "fake source failed"));
        }
        let size = bits.bytes().map_err(|error| {
            SourceError::new(SourceErrorKind::InvalidRequest, error.to_string())
        })?;
        if self.index >= self.samples.len() {
            self.cancel.cancel();
            return Ok(vec![0; size]);
        }
        let mut sample = self.samples[self.index].clone();
        self.index += 1;
        if sample.len() != size {
            sample.resize(size, 0);
        }
        if self.index >= self.samples.len() {
            self.cancel.cancel();
        }
        Ok(sample)
    }
}

struct PseudoTestOpener {
    max_samples: u32,
}

impl SourceOpener for PseudoTestOpener {
    fn open(
        &self,
        _config: &SourceConfig,
        cancel: &CancelToken,
    ) -> Result<Box<dyn EntropySource>, SafeError> {
        let opened = rngkit_sources::open(SourceConfig::Pseudo {
            max_range_samples: None,
        })
        .map_err(|error| map_source_kind(error.kind()))?;
        Ok(Box::new(CancelAfter {
            inner: opened,
            remaining: self.max_samples,
            cancel: cancel.clone(),
        }))
    }
}

struct CancelAfter {
    inner: OpenedSource,
    remaining: u32,
    cancel: CancelToken,
}

impl EntropySource for CancelAfter {
    fn descriptor(&self) -> &SourceDescriptor {
        self.inner.descriptor()
    }

    fn read_bits(&mut self, bits: SampleBits) -> Result<Vec<u8>, SourceError> {
        let bytes = self.inner.read_bits(bits)?;
        if self.remaining > 0 {
            self.remaining -= 1;
            if self.remaining == 0 {
                self.cancel.cancel();
            }
        }
        Ok(bytes)
    }
}

struct FailingSourceOpener {
    kind: SourceErrorKind,
}

impl SourceOpener for FailingSourceOpener {
    fn open(
        &self,
        _config: &SourceConfig,
        _cancel: &CancelToken,
    ) -> Result<Box<dyn EntropySource>, SafeError> {
        Err(map_source_kind(self.kind))
    }
}

fn descriptor_for(config: &SourceConfig) -> Result<SourceDescriptor, SafeError> {
    let (id, label, fold) = match config {
        SourceConfig::Bitb { fold, .. } => {
            (rngkit_core::SourceId::bitb(), "BitBabbler", Some(*fold))
        }
        SourceConfig::Trng { .. } => (rngkit_core::SourceId::trng(), "TrueRNG v1/v2/v3", None),
        SourceConfig::Rdseed { .. } => (rngkit_core::SourceId::rdseed(), "Intel RDSEED", None),
        SourceConfig::Pseudo { .. } => (rngkit_core::SourceId::pseudo(), "PseudoRNG", None),
        _ => {
            return Err(SafeError::expired_selection());
        }
    };
    SourceDescriptor::new(id, label, None, fold).map_err(|_| SafeError::unexpected_failure())
}

pub(crate) fn map_source_kind(kind: SourceErrorKind) -> SafeError {
    match kind {
        SourceErrorKind::PermissionDenied => {
            SafeError::permission_denied("Permission to use the selected source was denied.")
        }
        SourceErrorKind::DeviceBusy => SafeError::source_busy(),
        SourceErrorKind::Disconnected => SafeError::source_disconnected(),
        SourceErrorKind::Timeout => SafeError::source_timed_out(),
        SourceErrorKind::NotAvailable
        | SourceErrorKind::DeviceNotFound
        | SourceErrorKind::EntropyUnavailable => SafeError::source_unavailable(),
        _ => SafeError::unexpected_failure(),
    }
}

pub(crate) fn map_engine_error(error: &EngineError) -> SafeError {
    match error {
        EngineError::Source(source) => map_source_kind(source.kind()),
        EngineError::Recording(rngkit_recording::RecordingError::AlreadyExists { .. }) => {
            SafeError::output_exists()
        }
        EngineError::Config(_) => {
            SafeError::invalid_configuration("The session configuration is not valid.")
        }
        EngineError::Sink(_) => SafeError::channel_lost(),
        _ => SafeError::unexpected_failure(),
    }
}

pub(crate) fn map_engine_kind(kind: &str) -> SafeError {
    match kind {
        "source" => SafeError::source_unavailable(),
        "config" => SafeError::invalid_configuration("The session configuration is not valid."),
        "sink" => SafeError::channel_lost(),
        _ => SafeError::unexpected_failure(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::ErrorCode;

    #[test]
    fn maps_source_kinds_to_stable_safe_errors() {
        assert_eq!(
            map_source_kind(SourceErrorKind::DeviceBusy).code,
            ErrorCode::SourceBusy
        );
        assert_eq!(
            map_source_kind(SourceErrorKind::Disconnected).code,
            ErrorCode::SourceDisconnected
        );
        assert_eq!(
            map_source_kind(SourceErrorKind::Timeout).code,
            ErrorCode::SourceTimedOut
        );
        assert_eq!(
            map_source_kind(SourceErrorKind::NotAvailable).code,
            ErrorCode::SourceUnavailable
        );
    }
}
