//! Start, stop, start-another, and open-session-folder IPC.

use std::sync::{Arc, Mutex};

use rngkit_engine::{CancelToken, EngineError};
use tauri::ipc::Channel;
use tauri::{AppHandle, Runtime, State};

use crate::collection::{CollectionHandle, EventSender};
use crate::coordinator::AppCoordinator;
use crate::dto::{AppStateDto, CollectionEventDto};
use crate::errors::SafeError;

struct ChannelSender(Channel<CollectionEventDto>);

impl EventSender for ChannelSender {
    fn send_event(&self, event: CollectionEventDto) -> Result<(), EngineError> {
        self.0
            .send(event)
            .map_err(|error| EngineError::Sink(error.to_string()))
    }
}

#[tauri::command]
pub fn start_collection<R: Runtime>(
    app: AppHandle<R>,
    on_event: Channel<CollectionEventDto>,
    coordinator: State<'_, Mutex<AppCoordinator>>,
    collection: State<'_, CollectionHandle>,
) -> Result<AppStateDto, SafeError> {
    collection.join_previous();
    let plan = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_collection()?
    };
    let session_id = plan.session_id.clone();
    let cancel = CancelToken::new();
    let sender: Arc<dyn EventSender> = Arc::new(ChannelSender(on_event));
    if let Err(error) = collection.spawn_worker(app, sender, plan, cancel) {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = coordinator.finish_worker_failure(&session_id, error.clone());
        return Err(error);
    }
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(coordinator.snapshot())
}

#[tauri::command]
pub fn stop_collection(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    collection: State<'_, CollectionHandle>,
) -> Result<AppStateDto, SafeError> {
    {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.request_stop()?;
    }
    collection.request_cancel();
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(coordinator.snapshot())
}

#[tauri::command]
pub fn start_another_session(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    collection: State<'_, CollectionHandle>,
) -> Result<AppStateDto, SafeError> {
    collection.join_previous();
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    coordinator.start_another()?;
    Ok(coordinator.snapshot())
}

#[tauri::command]
pub fn open_session_folder(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    collection: State<'_, CollectionHandle>,
) -> Result<AppStateDto, SafeError> {
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    collection.open_known_session_folder(&coordinator)?;
    Ok(coordinator.snapshot())
}

#[tauri::command]
pub fn open_collection_working_folder(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    collection: State<'_, CollectionHandle>,
) -> Result<AppStateDto, SafeError> {
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    collection.open_known_output_root(&coordinator)?;
    Ok(coordinator.snapshot())
}
