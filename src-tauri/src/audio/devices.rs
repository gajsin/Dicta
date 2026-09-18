use cpal::traits::{DeviceTrait, HostTrait};

#[derive(serde::Serialize)]
pub struct AudioDevice {
    pub name: String,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
}

pub fn get_microphone_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let default_device_name = host.default_input_device().and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| format!("Не удалось получить список микрофонов: {}", e))?;

    let mut result = Vec::new();
    for d in devices {
        if let Ok(name) = d.name() {
            let is_default = Some(&name) == default_device_name.as_ref();
            eprintln!(
                "[Dicta] Found input device: {} {}",
                name,
                if is_default { "(default)" } else { "" }
            );
            result.push(AudioDevice { name, is_default });
        }
    }
    eprintln!("[Dicta] Total input devices found: {}", result.len());
    Ok(result)
}

pub fn select_input_device(
    host: &cpal::Host,
    device_label: Option<&str>,
) -> Result<cpal::Device, String> {
    if let Some(label) = device_label {
        let device = host
            .input_devices()
            .map_err(|e| format!("Ошибка поиска микрофонов: {e}"))?
            .find(|device| device.name().is_ok_and(|name| name == label));
        return device
            .or_else(|| host.default_input_device())
            .ok_or_else(|| {
                "Выбранный микрофон не найден, микрофон по умолчанию также недоступен.".to_string()
            });
    }

    host.default_input_device()
        .ok_or_else(|| "Микрофон не найден. Подключите микрофон и перезапустите.".to_string())
}
