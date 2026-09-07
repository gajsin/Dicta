use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut};

static LAST_HOTKEY_MS: AtomicU64 = AtomicU64::new(0);
pub const DEFAULT_HOTKEY: &str = "F9";

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn normalize_primary_key(token: &str) -> String {
    let upper = token.trim().to_uppercase();

    if upper.len() == 1 && upper.as_bytes()[0].is_ascii_alphabetic() {
        return format!("Key{upper}");
    }
    if upper.len() == 1 && upper.as_bytes()[0].is_ascii_digit() {
        return format!("Digit{upper}");
    }

    let physical_code = match upper.as_str() {
        "Й" => "KeyQ",
        "Ц" => "KeyW",
        "У" => "KeyE",
        "К" => "KeyR",
        "Е" => "KeyT",
        "Н" => "KeyY",
        "Г" => "KeyU",
        "Ш" => "KeyI",
        "Щ" => "KeyO",
        "З" => "KeyP",
        "Х" => "BracketLeft",
        "Ъ" => "BracketRight",
        "Ф" => "KeyA",
        "Ы" => "KeyS",
        "В" => "KeyD",
        "А" => "KeyF",
        "П" => "KeyG",
        "Р" => "KeyH",
        "О" => "KeyJ",
        "Л" => "KeyK",
        "Д" => "KeyL",
        "Ж" => "Semicolon",
        "Э" => "Quote",
        "Я" => "KeyZ",
        "Ч" => "KeyX",
        "С" => "KeyC",
        "М" => "KeyV",
        "И" => "KeyB",
        "Т" => "KeyN",
        "Ь" => "KeyM",
        "Б" => "Comma",
        "Ю" => "Period",
        "Ё" => "Backquote",
        "!" => "Digit1",
        "@" | "\"" => "Digit2",
        "#" => "Digit3",
        "$" => "Digit4",
        "%" => "Digit5",
        "^" => "Digit6",
        "&" => "Digit7",
        "*" => "Digit8",
        "(" => "Digit9",
        ")" => "Digit0",
        "_" => "Minus",
        "{" => "BracketLeft",
        "}" => "BracketRight",
        "|" => "Backslash",
        ":" => "Semicolon",
        "<" => "Comma",
        ">" => "Period",
        "?" => "Slash",
        "~" => "Backquote",
        "DEL" => "Delete",
        "INS" => "Insert",
        "PAGE_UP" => "PageUp",
        "PAGE_DOWN" => "PageDown",
        "UP" => "ArrowUp",
        "DOWN" => "ArrowDown",
        "LEFT" => "ArrowLeft",
        "RIGHT" => "ArrowRight",
        other => return other.to_string(),
    };

    physical_code.to_string()
}

fn parse_hotkey_parts(hotkey_str: &str) -> Result<Shortcut, String> {
    let value = hotkey_str.trim();
    let value = if value.is_empty() {
        DEFAULT_HOTKEY
    } else {
        value
    };

    let mut modifiers = Modifiers::empty();
    let mut primary_key: Option<String> = None;

    for raw_token in value.split('+') {
        let token = raw_token.trim();
        if token.is_empty() {
            return Err("Горячая клавиша содержит пустую часть".to_string());
        }

        match token.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers.insert(Modifiers::CONTROL),
            "ALT" | "OPTION" => modifiers.insert(Modifiers::ALT),
            "SHIFT" => modifiers.insert(Modifiers::SHIFT),
            "WIN" | "META" | "SUPER" | "CMD" | "COMMAND" => modifiers.insert(Modifiers::SUPER),
            _ if primary_key.is_none() => primary_key = Some(normalize_primary_key(token)),
            _ => {
                return Err(
                    "Горячая клавиша должна содержать ровно одну основную клавишу".to_string(),
                )
            }
        }
    }

    let primary_key = primary_key.ok_or_else(|| {
        "Горячая клавиша должна содержать основную клавишу, например F9 или Ctrl+Space".to_string()
    })?;

    let mut parser_value = Vec::with_capacity(5);
    if modifiers.contains(Modifiers::CONTROL) {
        parser_value.push("Control".to_string());
    }
    if modifiers.contains(Modifiers::ALT) {
        parser_value.push("Alt".to_string());
    }
    if modifiers.contains(Modifiers::SHIFT) {
        parser_value.push("Shift".to_string());
    }
    if modifiers.contains(Modifiers::SUPER) {
        parser_value.push("Super".to_string());
    }
    parser_value.push(primary_key);

    Shortcut::from_str(&parser_value.join("+"))
        .map_err(|error| format!("Неподдерживаемая горячая клавиша: {error}"))
}

pub fn parse_hotkey(hotkey_str: &str) -> Result<Shortcut, String> {
    parse_hotkey_parts(hotkey_str)
}

pub fn canonicalize_hotkey(hotkey_str: &str) -> Result<String, String> {
    let shortcut = parse_hotkey_parts(hotkey_str)?;
    let mut parts = Vec::with_capacity(5);

    if shortcut.mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if shortcut.mods.contains(Modifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if shortcut.mods.contains(Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    if shortcut.mods.contains(Modifiers::SUPER) {
        parts.push("Win".to_string());
    }
    parts.push(shortcut.key.to_string());

    Ok(parts.join("+"))
}

pub fn register_global_hotkey(
    app: tauri::AppHandle,
    hotkey_str: String,
    state: &crate::config::AppState,
) -> Result<(), String> {
    let canonical_hotkey = canonicalize_hotkey(&hotkey_str)?;
    let previous_hotkey = {
        let active = state
            .active_hotkey
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(ref current) = *active {
            if current == &canonical_hotkey {
                eprintln!(
                    "[Dicta] Hotkey {} is already registered, skipping re-registration",
                    canonical_hotkey
                );
                return Ok(());
            }
        }
        active.clone()
    };

    let shortcut = parse_hotkey(&canonical_hotkey)?;
    let global_shortcut = app.global_shortcut();

    global_shortcut
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let now = now_millis();
                let last = LAST_HOTKEY_MS.load(Ordering::Relaxed);
                if now.saturating_sub(last) < 200 {
                    eprintln!("[Dicta] Ignored duplicate hotkey press (debounced <200ms)");
                    return;
                }
                LAST_HOTKEY_MS.store(now, Ordering::Relaxed);

                eprintln!("[Dicta] ▶ Global hotkey pressed → emitting toggle-recording");
                if let Some(win) = app.get_webview_window("main") {
                    if let Err(e) = win.emit("toggle-recording", ()) {
                        eprintln!("[Dicta] Failed to emit toggle-recording: {}", e);
                    }
                } else {
                    eprintln!("[Dicta] Main window not found — cannot emit toggle-recording");
                }
            }
        })
        .map_err(|e| format!("Ошибка регистрации хоткея: {}", e))?;

    if let Some(previous_hotkey) = previous_hotkey {
        let previous_shortcut = parse_hotkey(&previous_hotkey)?;
        if let Err(error) = global_shortcut.unregister(previous_shortcut) {
            let _ = global_shortcut.unregister(shortcut);
            return Err(format!(
                "Не удалось заменить прежнюю горячую клавишу: {error}"
            ));
        }
    }

    {
        let mut active = state
            .active_hotkey
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *active = Some(canonical_hotkey.clone());
    }

    eprintln!("[Dicta] Dynamic hotkey registered: {}", canonical_hotkey);
    Ok(())
}

pub fn register_escape_cancel(app: &tauri::AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(None, tauri_plugin_global_shortcut::Code::Escape);
    let global_shortcut = app.global_shortcut();
    if global_shortcut.is_registered(shortcut) {
        return Ok(());
    }

    global_shortcut
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                eprintln!(
                    "[Dicta] ▶ Global Escape pressed during recording → emitting cancel-recording"
                );
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.emit("cancel-recording", ());
                }
            }
        })
        .map_err(|e| format!("Ошибка регистрации Escape: {}", e))?;

    eprintln!("[Dicta] Dynamic Escape shortcut registered for recording state");
    Ok(())
}

pub fn unregister_escape_cancel(app: &tauri::AppHandle) -> Result<(), String> {
    let shortcut = Shortcut::new(None, tauri_plugin_global_shortcut::Code::Escape);
    let global_shortcut = app.global_shortcut();
    if global_shortcut.is_registered(shortcut) {
        let _ = global_shortcut.unregister(shortcut);
        eprintln!("[Dicta] Dynamic Escape shortcut unregistered");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_hotkey;
    use tauri_plugin_global_shortcut::{Code, Modifiers};

    #[test]
    fn accepts_lock_navigation_and_media_keys() {
        let caps_lock = parse_hotkey("Shift+Win+CapsLock").expect("CapsLock is supported");
        assert_eq!(caps_lock.key, Code::CapsLock);
        assert_eq!(caps_lock.mods, Modifiers::SHIFT | Modifiers::SUPER);

        assert_eq!(
            parse_hotkey("NumLock").expect("NumLock is supported").key,
            Code::NumLock
        );
        assert_eq!(
            parse_hotkey("Escape").expect("Escape is supported").key,
            Code::Escape
        );
        assert_eq!(
            parse_hotkey("AudioVolumeMute")
                .expect("media keys are supported")
                .key,
            Code::AudioVolumeMute
        );
    }

    #[test]
    fn accepts_browser_physical_key_codes() {
        let shortcut = parse_hotkey("Control+Shift+KeyA").expect("browser key codes are supported");
        assert_eq!(shortcut.key, Code::KeyA);
        assert_eq!(shortcut.mods, Modifiers::CONTROL | Modifiers::SHIFT);

        assert_eq!(
            parse_hotkey("Digit7")
                .expect("browser digit codes are supported")
                .key,
            Code::Digit7
        );
    }

    #[test]
    fn rejects_more_than_one_primary_key() {
        assert!(parse_hotkey("Ctrl+A+B").is_err());
    }

    #[test]
    fn accepts_every_primary_key_class_emitted_by_the_browser_recorder() {
        let representative_codes = [
            "KeyZ",
            "Digit0",
            "F24",
            "Numpad0",
            "NumpadAdd",
            "NumpadDecimal",
            "NumpadDivide",
            "NumpadEnter",
            "NumpadEqual",
            "NumpadMultiply",
            "NumpadSubtract",
            "Backquote",
            "Backslash",
            "Backspace",
            "BracketLeft",
            "BracketRight",
            "CapsLock",
            "Comma",
            "Delete",
            "End",
            "Enter",
            "Equal",
            "Escape",
            "Home",
            "Insert",
            "Minus",
            "NumLock",
            "PageDown",
            "PageUp",
            "Pause",
            "Period",
            "PrintScreen",
            "Quote",
            "ScrollLock",
            "Semicolon",
            "Slash",
            "Space",
            "Tab",
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "ArrowUp",
            "AudioVolumeDown",
            "AudioVolumeMute",
            "AudioVolumeUp",
            "MediaPause",
            "MediaPlay",
            "MediaPlayPause",
            "MediaStop",
            "MediaTrackNext",
            "MediaTrackPrevious",
        ];

        for code in representative_codes {
            assert!(
                parse_hotkey(&format!("Ctrl+{code}")).is_ok(),
                "frontend emitted unsupported code: {code}"
            );
        }
    }
}
