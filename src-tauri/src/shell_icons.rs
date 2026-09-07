use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use tauri::{Manager, Theme};
use windows::core::{w, Interface, HSTRING, PROPVARIANT};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RPC_E_CHANGED_MODE, WPARAM};
use windows::Win32::Storage::EnhancedStorage::{
    PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand,
    PKEY_AppUserModel_RelaunchDisplayNameResource, PKEY_AppUserModel_RelaunchIconResource,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READWRITE,
};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};
use windows::Win32::UI::Shell::{
    DefSubclassProc, FOLDERID_Desktop, FOLDERID_Programs, FOLDERID_RoamingAppData, IShellLinkW,
    RemoveWindowSubclass, SHChangeNotify, SHGetKnownFolderPath, SetWindowSubclass, ShellLink,
    KF_FLAG_DEFAULT, SHCNE_UPDATEITEM, SHCNF_PATHW, SLGP_RAWPATH,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, LoadImageW, SendMessageW, HICON, ICON_BIG, IMAGE_ICON, LR_DEFAULTSIZE,
    LR_LOADFROMFILE, WM_NCDESTROY, WM_SETICON, WM_SETTINGCHANGE,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn system_theme() -> Theme {
    let mut light = 1u32;
    let mut size = std::mem::size_of_val(&light) as u32;
    unsafe {
        let _ = RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("SystemUsesLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut light as *mut u32).cast()),
            Some(&mut size),
        );
    }
    if light == 0 {
        Theme::Dark
    } else {
        Theme::Light
    }
}

fn icon_path(exe: &Path, theme: Theme) -> PathBuf {
    exe.with_file_name("icons").join(if theme == Theme::Dark {
        "icon-dark.ico"
    } else {
        "icon-light.ico"
    })
}

struct Apartment(bool);

impl Apartment {
    fn enter() -> windows::core::Result<Self> {
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if result.is_err() && result != RPC_E_CHANGED_MODE {
            result.ok()?;
        }
        Ok(Self(result.is_ok()))
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() }
        }
    }
}

fn known_folder(id: &windows::core::GUID) -> windows::core::Result<PathBuf> {
    unsafe {
        let raw = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None)?;
        let path = raw.to_string();
        CoTaskMemFree(Some(raw.0.cast()));
        Ok(PathBuf::from(path?))
    }
}

fn update_shortcut(path: &Path, exe: &Path, icon: &Path) -> Result<()> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        let file: IPersistFile = link.cast()?;
        let path_string = HSTRING::from(path);
        file.Load(&path_string, STGM_READWRITE)?;
        let mut target = [0u16; 32768];
        link.GetPath(&mut target, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)?;
        let target = PathBuf::from(String::from_utf16_lossy(
            &target[..target.iter().position(|c| *c == 0).unwrap_or(target.len())],
        ));
        // Never change another application's shortcut, even if it is named Dicta.
        if std::fs::canonicalize(target).ok().as_deref() != Some(exe) {
            return Ok(());
        }
        link.SetIconLocation(&HSTRING::from(icon), 0)?;
        let store: IPropertyStore = link.cast()?;
        store.SetValue(
            &PKEY_AppUserModel_RelaunchIconResource,
            &PROPVARIANT::from(format!("{},0", icon.display()).as_str()),
        )?;
        store.Commit()?;
        file.Save(&path_string, true)?;
        SHChangeNotify(
            SHCNE_UPDATEITEM,
            SHCNF_PATHW,
            Some(path_string.as_ptr().cast()),
            None,
        );
    }
    Ok(())
}

fn sync_shortcuts_for_theme(exe: &Path, theme: Theme) -> Result<()> {
    let _apartment = Apartment::enter()?;
    let icon = icon_path(exe, theme);
    if !icon.is_file() {
        return Err(format!("Missing icon: {}", icon.display()).into());
    }
    let canonical_exe = std::fs::canonicalize(exe)?;
    let programs = known_folder(&FOLDERID_Programs)?;
    let folders = [
        known_folder(&FOLDERID_Desktop)?,
        programs.join("Dicta"),
        programs,
        known_folder(&FOLDERID_RoamingAppData)?
            .join("Microsoft/Internet Explorer/Quick Launch/User Pinned/TaskBar"),
    ];
    let mut first_error = None;
    for folder in folders {
        let entries = match std::fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                first_error.get_or_insert_with(|| error.into());
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
            {
                if let Err(error) = update_shortcut(&path, &canonical_exe, &icon) {
                    first_error.get_or_insert(error);
                }
            }
        }
    }
    match first_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

pub fn sync_shortcuts() -> Result<()> {
    sync_shortcuts_for_theme(&std::env::current_exe()?, system_theme())
}

struct TaskbarIcon(HICON);

impl Drop for TaskbarIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}

fn set_taskbar_icon(hwnd: HWND, exe: &Path, icon: &Path) -> windows::core::Result<TaskbarIcon> {
    unsafe {
        let handle = LoadImageW(
            None,
            &HSTRING::from(icon),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE | LR_DEFAULTSIZE,
        )?;
        let owned = TaskbarIcon(HICON(handle.0));
        let store: IPropertyStore = SHGetPropertyStoreForWindow(hwnd)?;
        for (key, value) in [
            (
                PKEY_AppUserModel_RelaunchCommand,
                format!("\"{}\"", exe.display()),
            ),
            (
                PKEY_AppUserModel_RelaunchDisplayNameResource,
                "Dicta".into(),
            ),
            (
                PKEY_AppUserModel_RelaunchIconResource,
                format!("{},0", icon.display()),
            ),
            (PKEY_AppUserModel_ID, "com.gajsin.dicta".into()),
        ] {
            store.SetValue(&key, &PROPVARIANT::from(value.as_str()))?;
        }
        // Tauri's set_icon updates ICON_SMALL only; the taskbar uses ICON_BIG.
        SendMessageW(
            hwnd,
            WM_SETICON,
            WPARAM(ICON_BIG as usize),
            LPARAM(owned.0 .0),
        );
        Ok(owned)
    }
}

struct ThemeIcons {
    app: tauri::AppHandle,
    theme: Cell<Option<Theme>>,
    taskbar_icon: RefCell<Option<TaskbarIcon>>,
}

impl ThemeIcons {
    fn refresh(&self, hwnd: HWND) -> Result<()> {
        let theme = system_theme();
        if self.theme.get() == Some(theme) {
            return Ok(());
        }
        let exe = std::env::current_exe()?;
        let icon = set_taskbar_icon(hwnd, &exe, &icon_path(&exe, theme))?;
        self.taskbar_icon.replace(Some(icon));
        self.theme.set(Some(theme));
        crate::update_theme_icons(&self.app, theme);
        if let Err(error) = sync_shortcuts_for_theme(&exe, theme) {
            eprintln!("[Dicta] Failed to update shortcut icons: {error}");
        }
        Ok(())
    }
}

unsafe extern "system" fn theme_changed(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    if message == WM_NCDESTROY {
        let _ = RemoveWindowSubclass(hwnd, Some(theme_changed), id);
        let result = DefSubclassProc(hwnd, message, wparam, lparam);
        drop(Box::from_raw(data as *mut ThemeIcons));
        return result;
    }
    if message == WM_SETTINGCHANGE {
        let icons = &*(data as *const ThemeIcons);
        if let Err(error) = icons.refresh(hwnd) {
            eprintln!("[Dicta] Failed to update shell icons: {error}");
        }
    }
    DefSubclassProc(hwnd, message, wparam, lparam)
}

pub fn attach(app: &tauri::AppHandle) -> Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or("Main window is missing")?;
    let hwnd = HWND(window.hwnd()?.0 as isize);
    let icons = Box::new(ThemeIcons {
        app: app.clone(),
        theme: Cell::new(None),
        taskbar_icon: RefCell::new(None),
    });
    icons.refresh(hwnd)?;
    let data = Box::into_raw(icons);
    unsafe {
        if !SetWindowSubclass(hwnd, Some(theme_changed), 0x44494354, data as usize).as_bool() {
            drop(Box::from_raw(data));
            return Err(windows::core::Error::from_win32().into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WINDOW_STYLE, WM_GETICON,
    };

    #[test]
    fn updates_taskbar_and_own_shortcut_for_both_themes() -> Result<()> {
        let _apartment = Apartment::enter()?;
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("shell-icons-test-{}", std::process::id()));
        std::fs::create_dir_all(directory.join("icons"))?;
        let exe = directory.join("dicta.exe");
        std::fs::write(&exe, [])?;
        let canonical_exe = std::fs::canonicalize(&exe)?;
        for (source, target) in [
            ("icon.ico", "icon-dark.ico"),
            ("icon-light.ico", "icon-light.ico"),
        ] {
            std::fs::copy(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("icons")
                    .join(source),
                directory.join("icons").join(target),
            )?;
        }
        let shortcut = directory.join("Renamed Dicta.lnk");
        let other_shortcut = directory.join("Other.lnk");
        unsafe {
            for (path, target) in [
                (&shortcut, &exe),
                (&other_shortcut, &std::env::current_exe()?),
            ] {
                let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
                link.SetPath(&HSTRING::from(target.as_path()))?;
                link.cast::<IPersistFile>()?
                    .Save(&HSTRING::from(path.as_path()), true)?;
            }
            let other_before = std::fs::read(&other_shortcut)?;
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("Dicta icon test"),
                WINDOW_STYLE::default(),
                0,
                0,
                1,
                1,
                None,
                None,
                None,
                None,
            );
            assert_ne!(hwnd.0, 0);
            let mut previous_icon = None;
            for theme in [Theme::Light, Theme::Dark, Theme::Light] {
                let path = icon_path(&exe, theme);
                let icon = set_taskbar_icon(hwnd, &exe, &path)?;
                assert_eq!(
                    SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_BIG as usize), LPARAM(0)).0,
                    icon.0 .0
                );
                previous_icon.replace(icon);
                update_shortcut(&shortcut, &canonical_exe, &path)?;
                update_shortcut(&other_shortcut, &canonical_exe, &path)?;
                let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
                link.cast::<IPersistFile>()?
                    .Load(&HSTRING::from(shortcut.as_path()), STGM_READWRITE)?;
                let mut actual = [0u16; 32768];
                let mut index = -1;
                link.GetIconLocation(&mut actual, &mut index)?;
                let end = actual.iter().position(|c| *c == 0).unwrap();
                assert_eq!(PathBuf::from(String::from_utf16(&actual[..end])?), path);
                assert_eq!(index, 0);
                let store: IPropertyStore = SHGetPropertyStoreForWindow(hwnd)?;
                assert_eq!(
                    windows::core::BSTR::try_from(
                        &store.GetValue(&PKEY_AppUserModel_RelaunchIconResource)?
                    )?
                    .to_string(),
                    format!("{},0", path.display())
                );
            }
            assert_eq!(std::fs::read(&other_shortcut)?, other_before);
            DestroyWindow(hwnd)?;
        }
        std::fs::remove_dir_all(directory)?;
        Ok(())
    }
}
