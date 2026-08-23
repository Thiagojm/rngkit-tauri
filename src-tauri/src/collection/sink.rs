//! Maps engine events onto sequenced DTOs and updates the coordinator first.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rngkit_engine::{EngineError, EngineEvent, EventSink};

use super::EventSender;
use crate::coordinator::{AppCoordinator, CollectionUpdate};
use crate::dto::CollectionEventDto;
use crate::errors::SafeError;

pub struct CoordinatorSink<'a> {
    coordinator: &'a Mutex<AppCoordinator>,
    session_id: String,
    output_root: std::path::PathBuf,
    sender: &'a dyn EventSender,
}

impl<'a> CoordinatorSink<'a> {
    pub fn new(
        coordinator: &'a Mutex<AppCoordinator>,
        session_id: String,
        output_root: std::path::PathBuf,
        sender: &'a dyn EventSender,
    ) -> Self {
        Self {
            coordinator,
            session_id,
            output_root,
            sender,
        }
    }
}

impl EventSink for CoordinatorSink<'_> {
    fn emit(&mut self, event: EngineEvent) -> Result<(), EngineError> {
        let update = map_event(&event, &self.output_root);
        let dto = {
            let mut coordinator = lock(self.coordinator);
            coordinator
                .ingest_collection_update(&self.session_id, update)
                .map_err(|error| EngineError::Sink(error.message().to_owned()))?
        };
        self.sender.send_event(dto)
    }
}

fn map_event(event: &EngineEvent, output_root: &Path) -> CollectionUpdate {
    match event {
        EngineEvent::SessionStarted { stem } => CollectionUpdate::SessionStarted {
            stem: stem.to_string(),
            directory: output_root.join(stem.to_string()),
        },
        EngineEvent::SampleCommitted { record, snapshot } => CollectionUpdate::SampleCommitted {
            sample_index: snapshot.index.get(),
            sample_count: snapshot.sample_count,
            elapsed_label: elapsed_label(record.elapsed),
            ones_proportion_label: format!("{:.4}", snapshot.proportion),
            cumulative_z_label: format!("{:+.2}", snapshot.z),
        },
        EngineEvent::TimingOverrun { .. } => CollectionUpdate::TimingOverrun,
        EngineEvent::SessionStopped {
            committed,
            overruns,
        } => CollectionUpdate::CleanStop {
            sample_count: *committed,
            overrun_count: *overruns,
        },
        EngineEvent::SessionFailed {
            kind, diagnostic, ..
        } => CollectionUpdate::TerminalFailure {
            error: super::map_engine_kind(kind),
            diagnostic: diagnostic.clone(),
        },
        _ => CollectionUpdate::TerminalFailure {
            error: SafeError::unexpected_failure(),
            diagnostic: "unrecognized engine event".into(),
        },
    }
}

fn elapsed_label(elapsed: Option<Duration>) -> String {
    let secs = elapsed.map(|duration| duration.as_secs()).unwrap_or(0);
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn lock(coordinator: &Mutex<AppCoordinator>) -> MutexGuard<'_, AppCoordinator> {
    coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Collects events for deterministic tests.
#[derive(Clone, Default)]
pub struct VecSender {
    events: std::sync::Arc<Mutex<Vec<CollectionEventDto>>>,
}

impl VecSender {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn events(&self) -> Vec<CollectionEventDto> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl EventSender for VecSender {
    fn send_event(&self, event: CollectionEventDto) -> Result<(), EngineError> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
        Ok(())
    }
}

/// Fails the first matching event kind so channel loss is terminal.
pub struct FailingSender {
    inner: VecSender,
    fail_kind: &'static str,
    failed: Mutex<bool>,
}

impl FailingSender {
    #[must_use]
    pub fn on_sample_committed(inner: VecSender) -> Self {
        Self::on_kind(inner, "sampleCommitted")
    }

    #[must_use]
    pub fn on_clean_stop(inner: VecSender) -> Self {
        Self::on_kind(inner, "cleanStop")
    }

    fn on_kind(inner: VecSender, fail_kind: &'static str) -> Self {
        Self {
            inner,
            fail_kind,
            failed: Mutex::new(false),
        }
    }

    #[must_use]
    pub fn events(&self) -> Vec<CollectionEventDto> {
        self.inner.events()
    }
}

impl EventSender for FailingSender {
    fn send_event(&self, event: CollectionEventDto) -> Result<(), EngineError> {
        let kind = match &event {
            CollectionEventDto::SessionStarted { .. } => "sessionStarted",
            CollectionEventDto::SampleCommitted { .. } => "sampleCommitted",
            CollectionEventDto::TimingOverrun { .. } => "timingOverrun",
            CollectionEventDto::CleanStop { .. } => "cleanStop",
            CollectionEventDto::TerminalFailure { .. } => "terminalFailure",
        };
        if kind == self.fail_kind {
            let mut failed = self
                .failed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !*failed {
                *failed = true;
                return Err(EngineError::Sink("channel closed".into()));
            }
        }
        self.inner.send_event(event)
    }
}
