use crate::audio::{
    cancel_recording_internal, get_microphone_devices as audio_get_microphone_devices,
    start_loopback_internal, start_recording_internal, stop_loopback_internal,
    stop_recording_internal, AudioDevice,
};
use crate::config::AppState;
use crate::transcription::{try_transcribe_and_polish, TranscriptionOptions, TranscriptionResult};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionError {
    message: String,
    retry_available: bool,
}

impl TranscriptionError {
    fn without_audio(message: String) -> Self {
        Self {
            message,
            retry_available: false,
        }
    }

    fn with_audio(message: String) -> Self {
        Self {
            message,
            retry_available: true,
        }
    }
}

#[tauri::command]
pub fn get_microphone_devices() -> Result<Vec<AudioDevice>, String> {
    audio_get_microphone_devices()
}

#[tauri::command]
pub fn start_recording(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    device_label: Option<String>,
) -> Result<(), String> {
    {
        let mut last = state.last_audio.lock().unwrap_or_else(|e| e.into_inner());
        *last = None;
    }
    match start_recording_internal(app.clone(), device_label, &state.recording) {
        Ok(()) => {
            eprintln!("[Dicta] start_recording: OK");
            let _ = crate::shortcut::register_escape_cancel(&app);
            Ok(())
        }
        Err(e) => {
            eprintln!("[Dicta] start_recording: FAILED: {}", e);
            let _ = crate::shortcut::unregister_escape_cancel(&app);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn cancel_recording(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _ = crate::shortcut::unregister_escape_cancel(&app);
    cancel_recording_internal(&state.recording)
}

#[tauri::command]
pub async fn stop_recording_and_transcribe(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    options: TranscriptionOptions,
) -> Result<TranscriptionResult, TranscriptionError> {
    let _ = crate::shortcut::unregister_escape_cancel(&app);
    let base64_audio =
        stop_recording_internal(&state.recording).map_err(TranscriptionError::without_audio)?;
    let result = try_transcribe_and_polish(&base64_audio, &options).await;

    // Retain audio only when retry can help. Successful recordings should not
    // remain in memory for the lifetime of the application.
    if result.is_err() {
        let mut last = state.last_audio.lock().unwrap_or_else(|e| e.into_inner());
        *last = Some(base64_audio);
    }

    result.map_err(TranscriptionError::with_audio)
}

#[tauri::command]
pub async fn retry_transcription(
    state: tauri::State<'_, AppState>,
    options: TranscriptionOptions,
) -> Result<TranscriptionResult, TranscriptionError> {
    let base64_audio = {
        let mut last = state.last_audio.lock().unwrap_or_else(|e| e.into_inner());
        match last.take() {
            Some(audio) => audio,
            None => {
                return Err(TranscriptionError::without_audio(
                    "Нет записи для повторной отправки".to_string(),
                ))
            }
        }
    };

    let result = try_transcribe_and_polish(&base64_audio, &options).await;

    if result.is_err() {
        let mut last = state.last_audio.lock().unwrap_or_else(|e| e.into_inner());
        *last = Some(base64_audio);
    }

    result.map_err(TranscriptionError::with_audio)
}

#[tauri::command]
pub fn start_loopback(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    device_label: Option<String>,
) -> Result<(), String> {
    match start_loopback_internal(app, device_label, &state.loopback) {
        Ok(()) => {
            eprintln!("[Dicta] start_loopback: OK");
            Ok(())
        }
        Err(e) => {
            eprintln!("[Dicta] start_loopback: FAILED: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn stop_loopback(state: tauri::State<'_, AppState>) -> Result<(), String> {
    stop_loopback_internal(&state.loopback)
}
