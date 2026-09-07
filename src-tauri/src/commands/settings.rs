use crate::config::{load_settings_from_file, AppState, Settings};
use crate::shortcut::register_global_hotkey;
use crate::transcription::http_client;
use crate::window_overlay::set_autostart_internal;
use tauri::Emitter;

fn rollback_hotkey(
    app: &tauri::AppHandle,
    state: &AppState,
    previous_hotkey: Option<&String>,
) -> Option<String> {
    let previous_hotkey = previous_hotkey?;
    register_global_hotkey(app.clone(), previous_hotkey.clone(), state)
        .err()
        .map(|error| format!("не удалось восстановить hotkey {previous_hotkey}: {error}"))
}

fn settings_error_with_rollback(primary_error: String, rollback_errors: Vec<String>) -> String {
    if rollback_errors.is_empty() {
        primary_error
    } else {
        format!(
            "{primary_error}. Ошибки отката: {}",
            rollback_errors.join("; ")
        )
    }
}

#[tauri::command]
pub fn sync_settings(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    mut settings: Settings,
) -> Result<(), String> {
    settings.validate_and_normalize()?;

    let previous_settings = {
        let current = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref s) = *current {
            settings.window_x = s.window_x;
            settings.window_y = s.window_y;
            settings.window_width = s.window_width;
            settings.window_height = s.window_height;
            settings.overlay_x = s.overlay_x;
            settings.overlay_y = s.overlay_y;
        }
        current.clone()
    };
    let previous_autostart = previous_settings
        .as_ref()
        .map(|current_settings| current_settings.autostart);
    let previous_hotkey = state
        .active_hotkey
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    register_global_hotkey(app.clone(), settings.hotkey.clone(), &state)?;
    let autostart_changed = previous_autostart != Some(settings.autostart);
    if autostart_changed {
        if let Err(error) = set_autostart_internal(settings.autostart) {
            let rollback_errors = rollback_hotkey(&app, state.inner(), previous_hotkey.as_ref())
                .into_iter()
                .collect();
            return Err(settings_error_with_rollback(error, rollback_errors));
        }
    }

    {
        let mut current = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        *current = Some(settings);
    }
    if let Err(error) = state.settings_persistence.flush() {
        {
            let mut current = state.settings.lock().unwrap_or_else(|e| e.into_inner());
            *current = previous_settings.clone();
        }
        let mut rollback_errors = Vec::new();
        if autostart_changed {
            if let Some(previous_autostart) = previous_autostart {
                if let Err(rollback_error) = set_autostart_internal(previous_autostart) {
                    rollback_errors.push(format!(
                        "не удалось восстановить автозапуск: {rollback_error}"
                    ));
                }
            }
        }
        if let Some(rollback_error) = rollback_hotkey(&app, state.inner(), previous_hotkey.as_ref())
        {
            rollback_errors.push(rollback_error);
        }
        return Err(settings_error_with_rollback(error, rollback_errors));
    }

    eprintln!("[Dicta] Settings synced");
    if let Err(error) = app.emit("settings-updated", ()) {
        eprintln!("[Dicta] Failed to notify other windows about settings: {error}");
    }
    Ok(())
}

#[tauri::command]
pub fn delete_provider_api_key(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    provider: String,
) -> Result<(), String> {
    let previous_settings = {
        let s_guard = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        s_guard
            .clone()
            .ok_or_else(|| "Настройки ещё не загружены".to_string())?
    };
    let mut updated_settings = previous_settings.clone();
    updated_settings.clear_provider_key(&provider)?;

    {
        let mut s_guard = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        *s_guard = Some(updated_settings);
    }
    if let Err(error) = state.settings_persistence.flush() {
        let mut s_guard = state.settings.lock().unwrap_or_else(|e| e.into_inner());
        *s_guard = Some(previous_settings);
        return Err(error);
    }

    if let Err(error) = app.emit("settings-updated", ()) {
        eprintln!("[Dicta] Failed to notify other windows about key deletion: {error}");
    }
    Ok(())
}

#[tauri::command]
pub fn load_settings(state: tauri::State<'_, AppState>) -> Result<Option<Settings>, String> {
    let path_guard = state
        .settings_path
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    load_settings_from_file(&path_guard)
}

#[tauri::command]
pub async fn check_provider_api_key(provider: String, api_key: String) -> Result<bool, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Ok(false);
    }

    let client = http_client();
    match provider.as_str() {
        "groq" => {
            let res = client
                .get("https://api.groq.com/openai/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Ошибка подключения к Groq: {}", e))?;
            provider_key_status(res.status())
        }
        "openai" => {
            let res = client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Ошибка подключения к OpenAI: {}", e))?;
            provider_key_status(res.status())
        }
        "openrouter" => {
            let res = client
                .get("https://openrouter.ai/api/v1/auth/key")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Ошибка подключения к OpenRouter: {}", e))?;
            provider_key_status(res.status())
        }
        "polza" => {
            let res = client
                .get("https://polza.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Ошибка подключения к Polza: {}", e))?;
            provider_key_status(res.status())
        }
        _ => Err("Неизвестный провайдер".to_string()),
    }
}

// Authentication rejection invalidates verification. Rate limits, server and
// transport failures leave the key's previous verification intact.
fn provider_key_status(status: reqwest::StatusCode) -> Result<bool, String> {
    match status {
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => Ok(false),
        status if status.is_success() => Ok(true),
        status => Err(format!("API вернул статус {status}")),
    }
}

#[cfg(test)]
mod tests {
    use super::provider_key_status;
    use reqwest::StatusCode;

    #[test]
    fn authentication_is_distinct_from_temporary_failure() {
        assert_eq!(provider_key_status(StatusCode::OK), Ok(true));
        assert_eq!(provider_key_status(StatusCode::UNAUTHORIZED), Ok(false));
        assert_eq!(provider_key_status(StatusCode::FORBIDDEN), Ok(false));
        assert!(provider_key_status(StatusCode::TOO_MANY_REQUESTS).is_err());
        assert!(provider_key_status(StatusCode::SERVICE_UNAVAILABLE).is_err());
    }
}
