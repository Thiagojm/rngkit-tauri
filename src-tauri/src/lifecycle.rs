//! Window-close policy. Active sessions are never silently abandoned.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::collection::CollectionHandle;
use crate::coordinator::AppCoordinator;
use crate::dto::CollectionState;
use crate::preferences::PreferencesHandle;

pub const CLOSE_REQUESTED_EVENT: &str = "rngkit-close-requested";

/// What a close request should do for the current collection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosePolicy {
    Allow,
    Confirm,
    WaitForFinalize,
}

/// Frontend prompt after a close is intercepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClosePromptMode {
    Confirm,
    Finalizing,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClosePromptPayload {
    pub mode: ClosePromptMode,
}

/// Process-wide "close after this session finishes" flag.
#[derive(Debug, Default)]
pub struct LifecycleHandle {
    pending_exit: AtomicBool,
    closer_started: AtomicBool,
}

impl LifecycleHandle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request_exit(&self) {
        self.pending_exit.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.pending_exit.load(Ordering::SeqCst)
    }

    /// Returns true only for the first closer task.
    #[must_use]
    pub fn start_closer(&self) -> bool {
        self.closer_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    fn reset_closer(&self) {
        self.closer_started.store(false, Ordering::SeqCst);
    }
}

#[must_use]
pub fn close_policy(state: CollectionState) -> ClosePolicy {
    match state {
        CollectionState::Collecting => ClosePolicy::Confirm,
        CollectionState::Stopping => ClosePolicy::WaitForFinalize,
        CollectionState::Idle
        | CollectionState::Discovering
        | CollectionState::Ready
        | CollectionState::Completed
        | CollectionState::Failed => ClosePolicy::Allow,
    }
}

#[must_use]
pub fn is_terminal_or_idle(state: CollectionState) -> bool {
    !matches!(
        state,
        CollectionState::Collecting | CollectionState::Stopping
    )
}

/// Handle the policy captured while the native close was prevented. Keeping
/// that single decision avoids losing the close if the worker finishes next.
pub fn on_close_requested<R: Runtime>(window: &tauri::Window<R>, policy: ClosePolicy) {
    match policy {
        ClosePolicy::Allow => {}
        ClosePolicy::Confirm => {
            let _ = window.emit(
                CLOSE_REQUESTED_EVENT,
                ClosePromptPayload {
                    mode: ClosePromptMode::Confirm,
                },
            );
        }
        ClosePolicy::WaitForFinalize => {
            let _ = window.emit(
                CLOSE_REQUESTED_EVENT,
                ClosePromptPayload {
                    mode: ClosePromptMode::Finalizing,
                },
            );
            begin_exit_after_stop(window.app_handle());
        }
    }
}

#[must_use]
pub fn should_prevent_close(state: CollectionState) -> bool {
    close_policy(state) != ClosePolicy::Allow
}

/// Cooperative stop, then destroy the window only after the worker finishes.
pub fn begin_exit_after_stop<R: Runtime>(app: &AppHandle<R>) {
    let Some(lifecycle) = app.try_state::<LifecycleHandle>() else {
        return;
    };
    lifecycle.request_exit();

    let Some(coordinator) = app.try_state::<Mutex<AppCoordinator>>() else {
        persist_and_destroy(app);
        return;
    };
    let state = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if coordinator.snapshot().collection.state == CollectionState::Collecting {
            let _ = coordinator.request_stop();
        }
        coordinator.snapshot().collection.state
    };
    if matches!(
        state,
        CollectionState::Collecting | CollectionState::Stopping
    ) {
        if let Some(collection) = app.try_state::<CollectionHandle>() {
            collection.request_cancel();
        }
    }
    if !lifecycle.start_closer() {
        return;
    }
    if is_terminal_or_idle(state) {
        persist_and_destroy(app);
        return;
    }
    let Some(collection) = app.try_state::<CollectionHandle>() else {
        persist_and_destroy(app);
        return;
    };
    let collection = collection.inner().clone();
    let app_for_thread = app.clone();
    if std::thread::Builder::new()
        .name("rngkit-exit".into())
        .spawn(move || {
            collection.join_previous();
            persist_and_destroy(&app_for_thread);
        })
        .is_err()
    {
        // Keep a worker-completion retry possible; never force-destroy here.
        lifecycle.reset_closer();
    }
}

/// Worker completion may close the window when Stop and exit is already pending.
pub fn on_worker_finished<R: Runtime>(app: &AppHandle<R>) {
    let Some(lifecycle) = app.try_state::<LifecycleHandle>() else {
        return;
    };
    if lifecycle.is_pending() {
        begin_exit_after_stop(app);
    }
}

fn persist_and_destroy<R: Runtime>(app: &AppHandle<R>) {
    if let Some(prefs) = app.try_state::<PreferencesHandle>() {
        let _ = prefs.persist();
    }
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.destroy();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{ClosePolicy, LifecycleHandle, close_policy, is_terminal_or_idle};
    use crate::dto::CollectionState;

    #[test]
    fn collecting_requires_confirmation() {
        assert_eq!(
            close_policy(CollectionState::Collecting),
            ClosePolicy::Confirm
        );
        assert!(!is_terminal_or_idle(CollectionState::Collecting));
    }

    #[test]
    fn stopping_waits_without_a_new_stop() {
        assert_eq!(
            close_policy(CollectionState::Stopping),
            ClosePolicy::WaitForFinalize
        );
    }

    #[test]
    fn idle_ready_and_terminal_states_exit() {
        for state in [
            CollectionState::Idle,
            CollectionState::Discovering,
            CollectionState::Ready,
            CollectionState::Completed,
            CollectionState::Failed,
        ] {
            assert_eq!(close_policy(state), ClosePolicy::Allow);
            assert!(is_terminal_or_idle(state));
        }
    }

    #[test]
    fn closer_starts_once() {
        let handle = LifecycleHandle::new();
        handle.request_exit();
        assert!(handle.is_pending());
        assert!(handle.start_closer());
        assert!(!handle.start_closer());
    }
}
