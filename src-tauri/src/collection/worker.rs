//! One application-owned worker thread around the synchronous engine.

use std::sync::Mutex;

use rngkit_engine::{
    CancelToken, Clock, EngineConfig, FakeClock, run_session, run_session_with_clock,
};

use super::sink::CoordinatorSink;
use super::{EventSender, SourceOpener, map_engine_error};
use crate::coordinator::{AppCoordinator, CollectionStart};
use crate::errors::SafeError;

pub fn run_collection(
    coordinator: &Mutex<AppCoordinator>,
    opener: &dyn SourceOpener,
    use_fake_clock: bool,
    sender: &dyn EventSender,
    cancel: &CancelToken,
    plan: CollectionStart,
) {
    let session_id = plan.session_id.clone();
    let output_root = plan.output_root.clone();
    let opened = match opener.open(&plan.source_config, cancel) {
        Ok(source) => source,
        Err(error) => {
            fail_before_engine(coordinator, sender, &session_id, error);
            return;
        }
    };
    let mut source = BoxedSource(opened);

    let config = EngineConfig {
        output_root: plan.output_root,
        sample_bits: plan.sample_bits,
        interval: plan.interval,
    };
    let mut sink = CoordinatorSink::new(coordinator, session_id.clone(), output_root, sender);
    let result = if use_fake_clock {
        let clock = FakeClock::new();
        run_with_clock(&mut source, config, cancel, &mut sink, &clock)
    } else {
        run_session(&mut source, config, cancel, &mut sink)
    };
    if let Err(error) = result {
        let mapped = map_engine_error(&error);
        finish_worker_failure(coordinator, &session_id, mapped);
    }
}

fn run_with_clock<S, K, C>(
    source: &mut S,
    config: EngineConfig,
    cancel: &CancelToken,
    sink: &mut K,
    clock: &C,
) -> Result<rngkit_engine::SessionOutcome, rngkit_engine::EngineError>
where
    S: rngkit_core::EntropySource,
    K: rngkit_engine::EventSink,
    C: Clock,
{
    run_session_with_clock(source, config, cancel, sink, clock)
}

struct BoxedSource(Box<dyn rngkit_core::EntropySource>);

impl rngkit_core::EntropySource for BoxedSource {
    fn descriptor(&self) -> &rngkit_core::SourceDescriptor {
        self.0.descriptor()
    }

    fn read_bits(
        &mut self,
        bits: rngkit_core::SampleBits,
    ) -> Result<Vec<u8>, rngkit_core::SourceError> {
        self.0.read_bits(bits)
    }
}

fn fail_before_engine(
    coordinator: &Mutex<AppCoordinator>,
    sender: &dyn EventSender,
    session_id: &str,
    error: SafeError,
) {
    let diagnostic = error.message().to_owned();
    let event = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.ingest_collection_update(
            session_id,
            crate::coordinator::CollectionUpdate::TerminalFailure { error, diagnostic },
        )
    };
    if let Ok(event) = event {
        let _ = sender.send_event(event);
    }
}

fn finish_worker_failure(coordinator: &Mutex<AppCoordinator>, session_id: &str, error: SafeError) {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_worker_failure(session_id, error);
}
