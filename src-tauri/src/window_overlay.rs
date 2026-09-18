use std::sync::Mutex;
use tauri::Manager;

use crate::config::Settings;
use crate::settings_persistence::SettingsPersistence;

pub fn update_window_state(
    window: &tauri::WebviewWindow,
    settings_mutex: &Mutex<Option<Settings>>,
    settings_persistence: &SettingsPersistence,
    pos: Option<tauri::PhysicalPosition<i32>>,
    size: Option<tauri::PhysicalSize<u32>>,
) {
    if let Ok(minimized) = window.is_minimized() {
        if minimized {
            return;
        }
    }
    if let Ok(visible) = window.is_visible() {
        if !visible {
            return;
        }
    }

    let changed = {
        let mut guard = settings_mutex.lock().unwrap_or_else(|e| e.into_inner());
        let Some(s) = guard.as_mut() else {
            return;
        };
        let mut changed = false;
        if let Some(p) = pos {
            if p.x > -10000 && p.y > -10000 {
                s.window_x = Some(p.x);
                s.window_y = Some(p.y);
                changed = true;
            }
        }
        if let Some(sz) = size {
            if let Ok(factor) = window.scale_factor() {
                let logical: tauri::LogicalSize<f64> = sz.to_logical(factor);
                if logical.width >= 200.0 && logical.height >= 300.0 {
                    s.window_width = Some(logical.width);
                    s.window_height = Some(logical.height);
                    changed = true;
                }
            }
        }
        changed
    };
    if changed {
        settings_persistence.request();
    }
}

pub fn update_overlay_position(
    settings_mutex: &Mutex<Option<Settings>>,
    settings_persistence: &SettingsPersistence,
    pos: tauri::PhysicalPosition<i32>,
) {
    {
        let mut guard = settings_mutex.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut s) = *guard {
            s.overlay_x = Some(pos.x);
            s.overlay_y = Some(pos.y);
        } else {
            return;
        }
    }
    settings_persistence.request();
}

pub fn set_autostart_internal(enabled: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Не удалось получить путь к приложению: {}", e))?;
    let exe_str = exe_path
        .to_str()
        .ok_or_else(|| "Недопустимый путь к приложению".to_string())?;

    const RUN_KEY: &str = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    if enabled {
        // Run values need an explicitly quoted executable path when the
        // installation directory contains spaces.
        let quoted_exe = format!("\"{exe_str}\"");
        let output = std::process::Command::new("reg")
            .args(["add", RUN_KEY, "/v", "Dicta", "/t", "REG_SZ", "/d"])
            .arg(quoted_exe)
            .arg("/f")
            .output()
            .map_err(|error| format!("Не удалось запустить настройку автозапуска: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "Windows не включила автозапуск: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    } else {
        let output = std::process::Command::new("reg")
            .args(["delete", RUN_KEY, "/v", "Dicta", "/f"])
            .output()
            .map_err(|error| format!("Не удалось запустить настройку автозапуска: {error}"))?;

        if !output.status.success() {
            // Deleting an already absent value is idempotent and should not
            // surface as a settings error.
            let query_output = std::process::Command::new("reg")
                .args(["query", RUN_KEY, "/v", "Dicta"])
                .output()
                .map_err(|error| format!("Не удалось проверить состояние автозапуска: {error}"))?;
            if query_output.status.success() {
                return Err(format!(
                    "Windows не отключила автозапуск: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
        }
    }
    Ok(())
}

pub fn attach_overlay_moved_handler(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let app_handle_for_events = app.clone();
        let w = overlay.clone();
        overlay.on_window_event(move |event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = w.hide();
            }
            tauri::WindowEvent::Moved(pos) => {
                let state = app_handle_for_events.state::<crate::config::AppState>();
                update_overlay_position(&state.settings, &state.settings_persistence, *pos);
            }
            _ => {}
        });
    }
}

pub fn build_overlay_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    tauri::webview::WebviewWindowBuilder::new(
        app,
        "overlay",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Dicta Widget")
    .inner_size(440.0, 80.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .focusable(false)
    .visible(false)
    .skip_taskbar(true)
    .build()
    .map_err(|e| format!("Не удалось создать overlay-окно: {}", e))
}

pub fn ensure_overlay_exists(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    if let Some(overlay) = app.get_webview_window("overlay") {
        return Ok(overlay);
    }

    eprintln!("[Dicta] Overlay window missing — recreating it");
    let overlay = build_overlay_window(app)?;
    attach_overlay_moved_handler(app);
    eprintln!("[Dicta] Overlay window recreated successfully");
    Ok(overlay)
}

#[cfg(windows)]
pub fn raise_overlay_native(overlay: &tauri::WebviewWindow) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_EX_NOACTIVATE,
    };

    let tauri_hwnd = overlay
        .hwnd()
        .map_err(|e| format!("Не удалось получить HWND overlay: {e}"))?;
    let hwnd = HWND(tauri_hwnd.0 as isize);

    unsafe {
        let current_ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if (current_ex as u32 & WS_EX_NOACTIVATE.0) == 0 {
            SetWindowLongW(hwnd, GWL_EXSTYLE, current_ex | WS_EX_NOACTIVATE.0 as i32);
        }

        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("Windows не подняла overlay в z-order: {e}"))?;

        Ok(())
    }
}

#[cfg(not(windows))]
pub fn raise_overlay_native(overlay: &tauri::WebviewWindow) -> Result<(), String> {
    overlay
        .set_always_on_top(true)
        .map_err(|e| format!("Не удалось поднять overlay: {e}"))?;
    Ok(())
}

fn calculate_overlay_position(monitor: &tauri::Monitor) -> (i32, i32) {
    let scale_factor = monitor.scale_factor();
    let width = (440.0 * scale_factor) as i32;
    let height = (80.0 * scale_factor) as i32;
    let work_area = monitor.work_area();
    let x =
        work_area.position.x + (work_area.size.width as i32) - width - (24.0 * scale_factor) as i32;
    let y = work_area.position.y + (work_area.size.height as i32)
        - height
        - (24.0 * scale_factor) as i32;
    (x, y)
}

pub fn apply_overlay_geometry(
    overlay: &tauri::WebviewWindow,
    settings_mutex: &Mutex<Option<Settings>>,
    monitor_opt: Option<&tauri::Monitor>,
) {
    if let Some(monitor) = monitor_opt {
        let scale_factor = monitor.scale_factor();
        let width = (440.0 * scale_factor) as u32;
        let height = (80.0 * scale_factor) as u32;
        if let Err(e) = overlay.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
            width, height,
        ))) {
            eprintln!("[Dicta] apply_overlay_geometry: set_size failed: {}", e);
        }
    }

    let (x, y) = {
        let guard = settings_mutex.lock().unwrap_or_else(|e| e.into_inner());
        match &*guard {
            Some(s) if s.overlay_x.is_some() && s.overlay_y.is_some() => {
                (s.overlay_x.unwrap(), s.overlay_y.unwrap())
            }
            _ => {
                let resolved_monitor = monitor_opt
                    .cloned()
                    .or_else(|| overlay.current_monitor().ok().flatten());
                if let Some(ref monitor) = resolved_monitor {
                    calculate_overlay_position(monitor)
                } else {
                    return;
                }
            }
        }
    };

    if let Err(e) = overlay.set_position(tauri::PhysicalPosition::new(x, y)) {
        eprintln!("[Dicta] apply_overlay_geometry: set_position failed: {}", e);
    }
}
