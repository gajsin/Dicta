use crate::config::{save_settings_to_file, Settings};
use std::path::PathBuf;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

const DEFAULT_SAVE_DELAY: Duration = Duration::from_millis(250);

enum SaveRequest {
    Deferred,
    Flush(mpsc::Sender<Result<(), String>>),
}

pub struct SettingsPersistence {
    requests: mpsc::Sender<SaveRequest>,
    #[cfg(test)]
    completed_saves: Arc<AtomicU64>,
}

impl SettingsPersistence {
    pub fn new(
        settings: Arc<Mutex<Option<Settings>>>,
        settings_path: Arc<Mutex<Option<PathBuf>>>,
    ) -> Self {
        Self::with_delay(settings, settings_path, DEFAULT_SAVE_DELAY)
    }

    fn with_delay(
        settings: Arc<Mutex<Option<Settings>>>,
        settings_path: Arc<Mutex<Option<PathBuf>>>,
        delay: Duration,
    ) -> Self {
        let (requests, receiver) = mpsc::channel();
        #[cfg(test)]
        let completed_saves = Arc::new(AtomicU64::new(0));
        #[cfg(test)]
        let worker_completed_saves = Arc::clone(&completed_saves);

        std::thread::Builder::new()
            .name("dicta-settings-writer".to_string())
            .spawn(move || {
                run_save_worker(
                    receiver,
                    settings,
                    settings_path,
                    #[cfg(test)]
                    worker_completed_saves,
                    delay,
                );
            })
            .expect("failed to start Dicta settings writer");

        Self {
            requests,
            #[cfg(test)]
            completed_saves,
        }
    }

    pub fn request(&self) {
        if self.requests.send(SaveRequest::Deferred).is_err() {
            eprintln!("[Dicta] Settings writer is unavailable");
        }
    }

    pub fn flush(&self) -> Result<(), String> {
        let (result_sender, result_receiver) = mpsc::channel();
        self.requests
            .send(SaveRequest::Flush(result_sender))
            .map_err(|_| "Фоновая запись settings.json недоступна".to_string())?;
        result_receiver
            .recv()
            .map_err(|_| "Фоновая запись settings.json прервана".to_string())?
    }

    #[cfg(test)]
    pub fn completed_save_count(&self) -> u64 {
        self.completed_saves.load(Ordering::Relaxed)
    }
}

fn save_current_settings(
    settings: &Arc<Mutex<Option<Settings>>>,
    settings_path: &Arc<Mutex<Option<PathBuf>>>,
) -> Result<(), String> {
    let snapshot = settings
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
        .ok_or_else(|| "Настройки ещё не загружены".to_string())?;
    let path = settings_path
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    save_settings_to_file(&snapshot, &path)
}

fn perform_save(
    settings: &Arc<Mutex<Option<Settings>>>,
    settings_path: &Arc<Mutex<Option<PathBuf>>>,
    #[cfg(test)] completed_saves: &AtomicU64,
) -> Result<(), String> {
    let result = save_current_settings(settings, settings_path);
    #[cfg(test)]
    if result.is_ok() {
        completed_saves.fetch_add(1, Ordering::Relaxed);
    }
    result
}

fn run_save_worker(
    receiver: mpsc::Receiver<SaveRequest>,
    settings: Arc<Mutex<Option<Settings>>>,
    settings_path: Arc<Mutex<Option<PathBuf>>>,
    #[cfg(test)] completed_saves: Arc<AtomicU64>,
    delay: Duration,
) {
    while let Ok(request) = receiver.recv() {
        match request {
            SaveRequest::Flush(reply) => {
                let _ = reply.send(perform_save(
                    &settings,
                    &settings_path,
                    #[cfg(test)]
                    &completed_saves,
                ));
            }
            SaveRequest::Deferred => loop {
                match receiver.recv_timeout(delay) {
                    Ok(SaveRequest::Deferred) => continue,
                    Ok(SaveRequest::Flush(reply)) => {
                        let _ = reply.send(perform_save(
                            &settings,
                            &settings_path,
                            #[cfg(test)]
                            &completed_saves,
                        ));
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Err(error) = perform_save(
                            &settings,
                            &settings_path,
                            #[cfg(test)]
                            &completed_saves,
                        ) {
                            eprintln!("[Dicta] Failed to save deferred settings: {error}");
                        }
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        if let Err(error) = perform_save(
                            &settings,
                            &settings_path,
                            #[cfg(test)]
                            &completed_saves,
                        ) {
                            eprintln!("[Dicta] Failed to save settings on shutdown: {error}");
                        }
                        return;
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsPersistence;
    use crate::config::Settings;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn temporary_settings_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "dicta-debounce-{test_name}-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ))
    }

    #[test]
    fn rapid_geometry_updates_are_saved_once_after_the_last_event() {
        let path = temporary_settings_path("latest");
        let settings = Arc::new(Mutex::new(Some(Settings::default())));
        let settings_path = Arc::new(Mutex::new(Some(path.clone())));
        let persistence = SettingsPersistence::with_delay(
            Arc::clone(&settings),
            Arc::clone(&settings_path),
            Duration::from_millis(100),
        );

        {
            let mut current = settings.lock().expect("settings lock");
            current.as_mut().expect("settings").window_x = Some(10);
        }
        persistence.request();
        std::thread::sleep(Duration::from_millis(60));
        {
            let mut current = settings.lock().expect("settings lock");
            current.as_mut().expect("settings").window_x = Some(20);
        }
        persistence.request();

        std::thread::sleep(Duration::from_millis(60));
        assert!(
            !path.exists(),
            "the first request must be coalesced instead of writing mid-drag"
        );

        std::thread::sleep(Duration::from_millis(80));
        let saved = std::fs::read_to_string(&path).expect("latest geometry is persisted");
        assert!(saved.contains("\"windowX\": 20"));
        assert_eq!(persistence.completed_save_count(), 1);

        let _ = std::fs::remove_file(path);
    }
}
