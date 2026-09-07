use tauri::{Emitter, Manager};

use crate::config::AppState;
use crate::platform::simulate_ctrl_v;
use crate::window_overlay::{apply_overlay_geometry, ensure_overlay_exists, raise_overlay_native};

pub fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Главное окно Dicta не найдено".to_string())?;

    window
        .unminimize()
        .map_err(|error| format!("Не удалось развернуть главное окно: {error}"))?;
    window
        .show()
        .map_err(|error| format!("Не удалось показать главное окно: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("Не удалось сфокусировать главное окно: {error}"))
}

#[cfg(windows)]
fn get_active_monitor_by_cursor(app: &tauri::AppHandle) -> Option<tauri::Monitor> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut pt).is_ok() {
            return app
                .monitor_from_point(pt.x as f64, pt.y as f64)
                .ok()
                .flatten();
        }
    }
    None
}

#[tauri::command]
pub async fn show_overlay(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let overlay = ensure_overlay_exists(&app)?;

    #[cfg(windows)]
    let active_monitor = get_active_monitor_by_cursor(&app);
    #[cfg(not(windows))]
    let active_monitor = None;

    let monitor_opt = active_monitor
        .or_else(|| {
            app.get_webview_window("main")
                .and_then(|w| w.current_monitor().ok().flatten())
        })
        .or_else(|| overlay.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    apply_overlay_geometry(&overlay, &state.settings, monitor_opt.as_ref());

    if let Err(e) = overlay.set_skip_taskbar(true) {
        eprintln!("[Dicta] show_overlay: set_skip_taskbar failed: {}", e);
    }
    overlay
        .show()
        .map_err(|e| format!("Не удалось показать overlay: {e}"))?;
    let _ = app.emit("request-widget-sync", ());
    raise_overlay_native(&overlay)?;

    Ok(())
}

#[tauri::command]
pub async fn hide_overlay(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
    state.settings_persistence.flush()
}

#[tauri::command]
pub async fn reveal_main_window(app: tauri::AppHandle) -> Result<(), String> {
    show_main_window(&app)
}

#[tauri::command]
pub async fn copy_and_paste(
    text: String,
    paste: Option<bool>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let text_clone = text.clone();
    let write_res = if let Some(window) = app.get_webview_window("main") {
        window.clipboard().write_text(text_clone)
    } else {
        app.clipboard().write_text(text_clone)
    };

    write_res.map_err(|error| {
        eprintln!("[Dicta] Clipboard write failed: {error}");
        format!("Не удалось записать текст в буфер обмена: {error}")
    })?;

    if paste.unwrap_or(true) {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        simulate_ctrl_v().map_err(|error| {
            eprintln!("[Dicta] Paste simulation failed: {error}");
            format!("Не удалось вставить текст: {error}")
        })?;
        if let Some(overlay) = app.get_webview_window("overlay") {
            let _ = raise_overlay_native(&overlay);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn pick_screen_color() -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        tokio::task::spawn_blocking(|| {
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::sync::Mutex;
            use windows::core::w;
            use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, WPARAM};
            use windows::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC};
            use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
            use windows::Win32::UI::WindowsAndMessaging::*;

            // 1. Wait until mouse button is released if it was already pressed when clicking the eyedropper
            let start = std::time::Instant::now();
            while (unsafe { GetAsyncKeyState(0x01) } as u16 & 0x8000) != 0 {
                std::thread::sleep(std::time::Duration::from_millis(15));
                if start.elapsed().as_secs() > 2 {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(40));

            static RESULT_COLOR: Mutex<Option<String>> = Mutex::new(None);
            static FINISHED: AtomicBool = AtomicBool::new(false);

            *RESULT_COLOR.lock().unwrap() = None;
            FINISHED.store(false, Ordering::SeqCst);

            unsafe extern "system" fn picker_wndproc(
                hwnd: HWND,
                msg: u32,
                wparam: WPARAM,
                lparam: LPARAM,
            ) -> LRESULT {
                match msg {
                    WM_SETCURSOR => {
                        if let Ok(cursor) = LoadCursorW(None, IDC_CROSS) {
                            SetCursor(cursor);
                            return LRESULT(1);
                        }
                        DefWindowProcW(hwnd, msg, wparam, lparam)
                    }
                    WM_LBUTTONDOWN => {
                        let mut pt = POINT { x: 0, y: 0 };
                        let _ = GetCursorPos(&mut pt);

                        // Hide overlay immediately so GetPixel reads the pure screen underneath
                        let _ = ShowWindow(hwnd, SW_HIDE);

                        let hdc = GetDC(HWND::default());
                        let color = GetPixel(hdc, pt.x, pt.y);
                        let _ = ReleaseDC(HWND::default(), hdc);

                        if color.0 != 0xFFFFFFFF {
                            let r = (color.0 & 0xFF) as u8;
                            let g = ((color.0 >> 8) & 0xFF) as u8;
                            let b = ((color.0 >> 16) & 0xFF) as u8;
                            let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                            *RESULT_COLOR.lock().unwrap() = Some(hex);
                        }
                        FINISHED.store(true, Ordering::SeqCst);
                        let _ = DestroyWindow(hwnd);
                        LRESULT(0)
                    }
                    WM_RBUTTONDOWN => {
                        FINISHED.store(true, Ordering::SeqCst);
                        let _ = DestroyWindow(hwnd);
                        LRESULT(0)
                    }
                    WM_KEYDOWN => {
                        if wparam.0 == 0x1B {
                            FINISHED.store(true, Ordering::SeqCst);
                            let _ = DestroyWindow(hwnd);
                            return LRESULT(0);
                        }
                        DefWindowProcW(hwnd, msg, wparam, lparam)
                    }
                    WM_DESTROY => {
                        FINISHED.store(true, Ordering::SeqCst);
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }

            unsafe {
                let class_name = w!("DictaScreenPickerClass");
                let cursor = LoadCursorW(None, IDC_CROSS).unwrap_or_default();
                let wnd_class = WNDCLASSW {
                    lpfnWndProc: Some(picker_wndproc),
                    lpszClassName: class_name,
                    hCursor: cursor,
                    ..Default::default()
                };
                let _ = RegisterClassW(&wnd_class);

                let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
                let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
                let w = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                let h = GetSystemMetrics(SM_CYVIRTUALSCREEN);

                let hwnd = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                    class_name,
                    w!("DictaScreenPicker"),
                    WS_POPUP | WS_VISIBLE,
                    x,
                    y,
                    w,
                    h,
                    HWND::default(),
                    None,
                    None,
                    None,
                );
                if hwnd.0 == 0 {
                    return Err("Не удалось создать окно пипетки".to_string());
                }

                // Set alpha to 1 (virtually invisible to human eye, but intercepts all mouse clicks)
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 1, LWA_ALPHA);
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y, w, h, SWP_SHOWWINDOW);

                let mut msg = MSG::default();
                let timeout = std::time::Duration::from_secs(45);
                let loop_start = std::time::Instant::now();

                while !FINISHED.load(Ordering::SeqCst) {
                    if loop_start.elapsed() > timeout {
                        let _ = DestroyWindow(hwnd);
                        break;
                    }

                    // Also check for Escape asynchronously in case the window didn't capture focus
                    if (GetAsyncKeyState(0x1B) as u16 & 0x8000) != 0 {
                        let _ = DestroyWindow(hwnd);
                        break;
                    }

                    if PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                        if msg.message == WM_QUIT {
                            break;
                        }
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                }

                let res = RESULT_COLOR.lock().unwrap().clone();
                Ok(res)
            }
        })
        .await
        .map_err(|e| format!("Ошибка потока выбора цвета: {e}"))?
    }

    #[cfg(not(windows))]
    {
        Ok(None)
    }
}
