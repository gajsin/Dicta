use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tauri::State;

use crate::config::AppState;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    id: String,
    timestamp: String,
    duration: f64,
    processed_text: String,
    raw_text: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HistoryChange {
    Add { item: HistoryItem },
    Delete { id: String },
    Clear,
    Restore { items: Vec<HistoryItem> },
}

fn history_path(state: &AppState) -> Result<PathBuf, String> {
    let settings_path = state
        .settings_path
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    Ok(settings_path
        .as_ref()
        .ok_or("Путь к истории недоступен: APPDATA не найдена")?
        .with_file_name("history.json"))
}

fn read(path: &Path) -> Result<Vec<HistoryItem>, String> {
    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json)
            .map_err(|error| format!("Не удалось разобрать history.json: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("Не удалось прочитать history.json: {error}")),
    }
}

fn write(path: &Path, items: &[HistoryItem]) -> Result<(), String> {
    let parent = path.parent().ok_or("Путь к истории недоступен")?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("Не удалось создать папку истории: {error}"))?;
    let json = serde_json::to_vec(items)
        .map_err(|error| format!("Не удалось сериализовать историю: {error}"))?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, json)
        .map_err(|error| format!("Не удалось записать временный файл истории: {error}"))?;
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("Не удалось заменить history.json: {error}"));
    }
    Ok(())
}

fn merge(items: &mut Vec<HistoryItem>, additions: Vec<HistoryItem>) -> bool {
    let mut ids: HashSet<String> = items.iter().map(|item| item.id.clone()).collect();
    let mut changed = false;
    for item in additions {
        if ids.insert(item.id.clone()) {
            items.push(item);
            changed = true;
        }
    }
    if changed {
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }
    changed
}

#[tauri::command]
pub fn load_history(
    state: State<'_, AppState>,
    legacy_items: Vec<HistoryItem>,
) -> Result<Vec<HistoryItem>, String> {
    let _lock = state.history_lock.lock().unwrap_or_else(|e| e.into_inner());
    let path = history_path(&state)?;
    let mut items = read(&path)?;
    if merge(&mut items, legacy_items) {
        write(&path, &items)?;
    }
    Ok(items)
}

#[tauri::command]
pub fn change_history(state: State<'_, AppState>, change: HistoryChange) -> Result<(), String> {
    let _lock = state.history_lock.lock().unwrap_or_else(|e| e.into_inner());
    let path = history_path(&state)?;
    let mut items = read(&path)?;
    match change {
        HistoryChange::Add { item } => {
            merge(&mut items, vec![item]);
        }
        HistoryChange::Delete { id } => items.retain(|item| item.id != id),
        HistoryChange::Clear => items.clear(),
        HistoryChange::Restore { items: restored } => {
            merge(&mut items, restored);
        }
    }
    write(&path, &items)
}

#[cfg(test)]
mod tests {
    use super::{merge, read, write, HistoryItem};

    #[test]
    fn merge_keeps_both_origins_without_duplicates_and_persists_them() {
        let item = |id: &str, timestamp: &str| HistoryItem {
            id: id.into(),
            timestamp: timestamp.into(),
            duration: 1.0,
            processed_text: id.into(),
            raw_text: Some(id.into()),
        };
        let mut items = vec![item("dev", "2026-09-16T12:00:00Z")];
        assert!(merge(
            &mut items,
            vec![
                item("prod", "2026-09-17T12:00:00Z"),
                item("dev", "2026-09-16T12:00:00Z")
            ]
        ));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "prod");
        let path =
            std::env::temp_dir().join(format!("dicta-history-test-{}.json", std::process::id()));
        write(&path, &items).expect("history can be written");
        assert_eq!(read(&path).expect("history can be read").len(), 2);
        let _ = std::fs::remove_file(path);
    }
}
