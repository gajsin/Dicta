use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::settings_persistence::SettingsPersistence;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderApiKeys {
    #[serde(default)]
    pub polza: String,
    #[serde(default)]
    pub openai: String,
    #[serde(default)]
    pub openrouter: String,
    #[serde(default)]
    pub groq: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_stt_provider")]
    pub stt_provider: String,
    #[serde(default = "default_llm_provider")]
    pub llm_provider: String,
    #[serde(default)]
    pub provider_api_keys: ProviderApiKeys,
    #[serde(default)]
    pub verified_providers: Vec<String>,
    #[serde(default = "default_stt_model")]
    pub selected_model: String,
    #[serde(default = "default_language")]
    pub selected_language: String,
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub postprocess_enabled: bool,
    #[serde(default = "default_postprocess_mode")]
    pub postprocess_mode: String,
    #[serde(default = "default_postprocess_model")]
    pub postprocess_model: String,
    #[serde(default)]
    pub selected_device_id: String,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default)]
    pub window_width: Option<f64>,
    #[serde(default)]
    pub window_height: Option<f64>,
    #[serde(default)]
    pub overlay_x: Option<i32>,
    #[serde(default)]
    pub overlay_y: Option<i32>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_accent_opacity")]
    pub accent_opacity: i16,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_true")]
    pub save_history: bool,
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_accent_color() -> String {
    "#FF9500".to_string()
}

fn default_accent_opacity() -> i16 {
    100
}

fn default_postprocess_mode() -> String {
    "raw".to_string()
}

fn default_stt_model() -> String {
    "whisper-large-v3-turbo".to_string()
}

fn default_language() -> String {
    "auto".to_string()
}

fn default_ui_language() -> String {
    "system".to_string()
}

fn default_hotkey() -> String {
    "F9".to_string()
}

fn default_postprocess_model() -> String {
    "openai/gpt-oss-120b".to_string()
}

fn default_stt_provider() -> String {
    "groq".to_string()
}

fn default_llm_provider() -> String {
    "groq".to_string()
}

impl ProviderApiKeys {
    pub fn get(&self, provider: &str) -> Option<&String> {
        match provider {
            "polza" => Some(&self.polza),
            "openai" => Some(&self.openai),
            "openrouter" => Some(&self.openrouter),
            "groq" => Some(&self.groq),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, provider: &str) -> Option<&mut String> {
        match provider {
            "polza" => Some(&mut self.polza),
            "openai" => Some(&mut self.openai),
            "openrouter" => Some(&mut self.openrouter),
            "groq" => Some(&mut self.groq),
            _ => None,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            stt_provider: default_stt_provider(),
            llm_provider: default_llm_provider(),
            provider_api_keys: ProviderApiKeys::default(),
            verified_providers: Vec::new(),
            selected_model: default_stt_model(),
            selected_language: default_language(),
            ui_language: default_ui_language(),
            hotkey: default_hotkey(),
            postprocess_enabled: false,
            postprocess_mode: default_postprocess_mode(),
            postprocess_model: default_postprocess_model(),
            selected_device_id: String::new(),
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            overlay_x: None,
            overlay_y: None,
            theme: default_theme(),
            accent_color: default_accent_color(),
            accent_opacity: default_accent_opacity(),
            autostart: false,
            save_history: true,
        }
    }
}

impl Settings {
    pub fn clear_provider_key(&mut self, provider: &str) -> Result<(), String> {
        self.provider_api_keys
            .get_mut(provider)
            .ok_or_else(|| "Неизвестный провайдер".to_string())?
            .clear();
        if self.stt_provider == provider {
            self.api_key.clear();
        }
        self.verified_providers
            .retain(|verified| verified != provider);
        Ok(())
    }

    pub fn validate_and_normalize(&mut self) -> Result<(), String> {
        for (kind, provider) in [
            ("распознавания", self.stt_provider.as_str()),
            ("постобработки", self.llm_provider.as_str()),
        ] {
            if !matches!(provider, "polza" | "openai" | "openrouter" | "groq") {
                return Err(format!("Неизвестный провайдер {kind}: {provider}"));
            }
        }

        crate::transcription::postprocessing::validate_postprocess_mode(&self.postprocess_mode)?;
        if !matches!(self.theme.as_str(), "light" | "dark" | "system") {
            return Err(format!("Неизвестная тема оформления: {}", self.theme));
        }
        let valid_accent = self.accent_color.len() == 7
            && self.accent_color.starts_with('#')
            && self.accent_color[1..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit());
        if valid_accent {
            self.accent_color.make_ascii_uppercase();
        } else {
            self.accent_color = default_accent_color();
        }
        self.accent_opacity = self.accent_opacity.clamp(0, 100);
        if self.selected_model.trim().is_empty() {
            return Err("Модель распознавания не указана".to_string());
        }
        if self.selected_language.trim().is_empty() {
            self.selected_language = default_language();
        }

        let mut verified_providers = Vec::new();
        for provider in std::mem::take(&mut self.verified_providers) {
            let has_key = self
                .provider_api_keys
                .get(&provider)
                .is_some_and(|key| !key.trim().is_empty());
            if has_key && !verified_providers.contains(&provider) {
                verified_providers.push(provider);
            }
        }
        self.verified_providers = verified_providers;

        self.hotkey = crate::shortcut::canonicalize_hotkey(&self.hotkey)?;
        self.api_key = self
            .provider_api_keys
            .get(&self.stt_provider)
            .cloned()
            .unwrap_or_default();
        Ok(())
    }
}

pub struct AppState {
    pub settings_path: Arc<Mutex<Option<PathBuf>>>,
    pub settings: Arc<Mutex<Option<Settings>>>,
    pub settings_persistence: SettingsPersistence,
    pub active_hotkey: Mutex<Option<String>>,
    pub recording: Mutex<Option<crate::audio::RecordingState>>,
    pub loopback: Mutex<Option<crate::audio::LoopbackState>>,
    /// Последняя записанная аудиодорожка (base64 WAV) — хранится для retry,
    /// чтобы при ошибке сети не приходилось диктовать заново.
    pub last_audio: Mutex<Option<String>>,
}

fn merge_legacy_key(settings: &mut Settings) {
    if !settings.api_key.trim().is_empty() {
        if let Some(provider_key) = settings.provider_api_keys.get_mut(&settings.stt_provider) {
            if provider_key.trim().is_empty() {
                *provider_key = settings.api_key.trim().to_string();
            }
        }
    }
    settings.api_key = settings
        .provider_api_keys
        .get(&settings.stt_provider)
        .cloned()
        .unwrap_or_default();
}

pub fn save_settings_to_file(
    settings: &Settings,
    settings_path: &Option<PathBuf>,
) -> Result<(), String> {
    let path = settings_path.as_ref().ok_or_else(|| {
        "Путь к settings.json недоступен: переменная APPDATA не найдена".to_string()
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Не удалось создать папку настроек: {error}"))?;
    }

    let mut canonical = settings.clone();
    merge_legacy_key(&mut canonical);
    canonical.validate_and_normalize()?;

    let json = serde_json::to_string_pretty(&canonical)
        .map_err(|error| format!("Не удалось сериализовать настройки: {error}"))?;
    let temporary_path = path.with_extension("json.tmp");
    std::fs::write(&temporary_path, json)
        .map_err(|error| format!("Не удалось записать временный файл настроек: {error}"))?;
    if let Err(error) = std::fs::rename(&temporary_path, path) {
        let _ = std::fs::remove_file(&temporary_path);
        return Err(format!("Не удалось заменить settings.json: {error}"));
    }
    eprintln!("[Dicta] Settings saved to {:?}", path);

    Ok(())
}

pub fn load_settings_from_file(
    settings_path: &Option<PathBuf>,
) -> Result<Option<Settings>, String> {
    let Some(path) = settings_path.as_ref() else {
        return Ok(None);
    };
    let json = match std::fs::read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Не удалось прочитать settings.json ({}): {error}",
                path.display()
            ))
        }
    };
    let mut settings: Settings = serde_json::from_str(&json).map_err(|error| {
        format!(
            "Не удалось разобрать settings.json ({}): {error}",
            path.display()
        )
    })?;
    merge_legacy_key(&mut settings);
    settings.validate_and_normalize()?;
    eprintln!("[Dicta] Settings loaded from {:?}", path);
    Ok(Some(settings))
}

#[cfg(test)]
mod tests {
    use super::{load_settings_from_file, save_settings_to_file, ProviderApiKeys, Settings};
    use std::path::PathBuf;

    fn temporary_settings_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "dicta-{test_name}-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ))
    }

    #[test]
    fn settings_document_persists_provider_keys_locally() {
        let path = temporary_settings_path("persistence");
        let settings = Settings {
            stt_provider: "groq".to_string(),
            llm_provider: "openai".to_string(),
            hotkey: "Shift+Win+CapsLock".to_string(),
            accent_opacity: 35,
            api_key: "gsk_secret".to_string(),
            verified_providers: vec!["groq".to_string(), "openai".to_string()],
            provider_api_keys: ProviderApiKeys {
                groq: "gsk_secret".to_string(),
                openai: "sk-openai-secret".to_string(),
                ..ProviderApiKeys::default()
            },
            ..Settings::default()
        };

        save_settings_to_file(&settings, &Some(path.clone())).expect("settings are saved");
        let json = std::fs::read_to_string(&path).expect("settings document exists");
        assert!(json.contains("\"sttProvider\": \"groq\""));
        assert!(json.contains("\"llmProvider\": \"openai\""));
        assert!(json.contains("\"hotkey\": \"Shift+Win+CapsLock\""));
        assert!(json.contains("\"accentOpacity\": 35"));
        assert!(json.contains("\"groq\": \"gsk_secret\""));
        assert!(json.contains("\"openai\": \"sk-openai-secret\""));
        assert!(json.contains("\"verifiedProviders\""));

        let loaded = load_settings_from_file(&Some(path.clone()))
            .expect("saved settings are valid")
            .expect("saved settings exist");
        assert_eq!(loaded.provider_api_keys.groq, "gsk_secret");
        assert_eq!(loaded.provider_api_keys.openai, "sk-openai-secret");
        assert_eq!(loaded.accent_opacity, 35);
        assert_eq!(loaded.verified_providers, ["groq", "openai"]);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn settings_document_can_be_replaced_after_the_first_save() {
        let path = temporary_settings_path("overwrite");
        let mut settings = Settings::default();
        save_settings_to_file(&settings, &Some(path.clone())).expect("first save succeeds");

        settings.theme = "dark".to_string();
        save_settings_to_file(&settings, &Some(path.clone())).expect("second save succeeds");

        let saved = load_settings_from_file(&Some(path.clone()))
            .expect("saved document is valid")
            .expect("saved document exists");
        assert_eq!(saved.theme, "dark");

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn saving_without_a_settings_path_is_not_reported_as_success() {
        let error = save_settings_to_file(&Settings::default(), &None)
            .expect_err("missing APPDATA path must be visible to the caller");
        assert!(error.contains("settings.json"));
    }

    #[test]
    fn malformed_settings_document_returns_an_actionable_error() {
        let path = temporary_settings_path("malformed");
        std::fs::write(&path, "{broken").expect("invalid fixture is written");

        let error = load_settings_from_file(&Some(path.clone()))
            .expect_err("malformed JSON must not silently become defaults");
        assert!(error.contains("settings.json"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn validation_rejects_unknown_providers_and_normalizes_hotkey() {
        let mut settings = Settings {
            stt_provider: "unknown".to_string(),
            ..Settings::default()
        };
        assert!(settings.validate_and_normalize().is_err());

        settings.stt_provider = "groq".to_string();
        settings.hotkey = "Shift+Win+CapsLock".to_string();
        settings
            .validate_and_normalize()
            .expect("supported settings validate");
        assert_eq!(settings.hotkey, "Shift+Win+CapsLock");
    }

    #[test]
    fn clearing_a_provider_key_keeps_legacy_alias_consistent() {
        let mut settings = Settings {
            stt_provider: "groq".to_string(),
            api_key: "gsk_secret".to_string(),
            verified_providers: vec!["groq".to_string()],
            provider_api_keys: ProviderApiKeys {
                groq: "gsk_secret".to_string(),
                openai: "sk_other".to_string(),
                ..ProviderApiKeys::default()
            },
            ..Settings::default()
        };

        settings
            .clear_provider_key("groq")
            .expect("known provider can be cleared");
        assert!(settings.provider_api_keys.groq.is_empty());
        assert!(settings.api_key.is_empty());
        assert!(settings.verified_providers.is_empty());
        assert_eq!(settings.provider_api_keys.openai, "sk_other");
        assert!(settings.clear_provider_key("unknown").is_err());
    }
}
