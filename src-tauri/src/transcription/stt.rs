use crate::config::ProviderApiKeys;
use base64::{engine::general_purpose, Engine as _};

pub fn provider_label(provider: &str) -> &'static str {
    match provider {
        "polza" => "Polza.ai",
        "openai" => "OpenAI",
        "openrouter" => "OpenRouter",
        "groq" => "Groq",
        _ => "провайдера",
    }
}

pub fn api_key_for_provider(
    provider: &str,
    provider_api_keys: &ProviderApiKeys,
) -> Result<String, String> {
    let key = match provider {
        "polza" => &provider_api_keys.polza,
        "openai" => &provider_api_keys.openai,
        "openrouter" => &provider_api_keys.openrouter,
        "groq" => &provider_api_keys.groq,
        _ => return Err(format!("Неизвестный провайдер: {}", provider)),
    };

    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(format!(
            "Добавьте API-ключ для {}",
            provider_label(provider)
        ));
    }

    Ok(key)
}

fn audio_transcription_endpoint(provider: &str) -> Result<&'static str, String> {
    match provider {
        "openai" => Ok("https://api.openai.com/v1/audio/transcriptions"),
        "groq" => Ok("https://api.groq.com/openai/v1/audio/transcriptions"),
        _ => Err(format!(
            "{} не поддерживает прямой endpoint распознавания речи",
            provider_label(provider)
        )),
    }
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

fn push_multipart_field(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}

fn build_audio_multipart_body(
    model: &str,
    language: &str,
    audio_base64: &str,
) -> Result<(String, Vec<u8>), String> {
    let audio_bytes = if audio_base64.bytes().any(|b| b.is_ascii_whitespace()) {
        let compact: Vec<u8> = audio_base64
            .bytes()
            .filter(|b| !b.is_ascii_whitespace())
            .collect();
        general_purpose::STANDARD.decode(compact)
    } else {
        general_purpose::STANDARD.decode(audio_base64.as_bytes())
    }
    .map_err(|err| format!("Не удалось подготовить аудио: {}", err))?;

    let boundary = format!("dicta-{}", now_millis());
    let mut body = Vec::new();

    push_multipart_field(&mut body, &boundary, "model", model);
    push_multipart_field(&mut body, &boundary, "response_format", "json");
    push_multipart_field(&mut body, &boundary, "temperature", "0");
    if language != "auto" && !language.trim().is_empty() {
        push_multipart_field(&mut body, &boundary, "language", language);
    }
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"dicta.wav\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
    body.extend_from_slice(&audio_bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    Ok((boundary, body))
}

fn transcribe_polza_payload(model: &str, language: &str, data_uri: &str) -> serde_json::Value {
    let model = match model {
        ""
        | "whisper-large-v3"
        | "whisper-large-v3-turbo"
        | "openai/whisper-large-v3"
        | "openai/whisper-large-v3-turbo" => "whisper-1",
        value => value,
    };
    let mut payload = serde_json::json!({
        "model": model,
        "file": data_uri,
        "response_format": "json",
        "temperature": 0
    });
    if language != "auto" && !language.trim().is_empty() {
        payload["language"] = serde_json::Value::String(language.to_string());
    }
    payload
}

fn extract_text_from_json(body: &str, provider: &str) -> Result<String, String> {
    let payload: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| format!("Невалидный JSON ответа {}", provider_label(provider)))?;
    let text = payload
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        return Err(format!("{} вернул пустой текст", provider_label(provider)));
    }
    Ok(text)
}

async fn transcribe_polza_internal(
    provider_api_keys: &ProviderApiKeys,
    model: &str,
    language: &str,
    audio_base64: &str,
) -> Result<String, String> {
    let api_key = api_key_for_provider("polza", provider_api_keys)?;
    let mime_type = "audio/wav";
    let data_uri = format!("data:{};base64,{}", mime_type, audio_base64);
    let client = http_client();
    let payload = transcribe_polza_payload(model, language, &data_uri);
    let response = client
        .post("https://polza.ai/api/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("Сеть (Polza.ai): {}", err))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| format!("Чтение ответа (Polza.ai): {}", err))?;

    if !status.is_success() {
        return Err(format!("Polza.ai API {}: {}", status, body));
    }

    extract_text_from_json(&body, "polza")
}

async fn transcribe_openai_compatible_internal(
    provider: &str,
    provider_api_keys: &ProviderApiKeys,
    model: &str,
    language: &str,
    audio_base64: &str,
) -> Result<String, String> {
    let api_key = api_key_for_provider(provider, provider_api_keys)?;
    let endpoint = audio_transcription_endpoint(provider)?;
    let (boundary, body) = build_audio_multipart_body(model, language, audio_base64)?;

    let client = http_client();
    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(body)
        .send()
        .await
        .map_err(|err| format!("Сеть ({}): {}", provider_label(provider), err))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| format!("Чтение ответа ({}): {}", provider_label(provider), err))?;

    if !status.is_success() {
        return Err(format!(
            "{} API {}: {}",
            provider_label(provider),
            status,
            body
        ));
    }

    extract_text_from_json(&body, provider)
}

async fn transcribe_openrouter_internal(
    provider_api_keys: &ProviderApiKeys,
    model: &str,
    language: &str,
    audio_base64: &str,
) -> Result<String, String> {
    let api_key = api_key_for_provider("openrouter", provider_api_keys)?;
    let client = http_client();

    let mut payload = serde_json::json!({
        "model": model,
        "input_audio": {
            "data": audio_base64,
            "format": "wav"
        },
        "temperature": 0
    });
    if language != "auto" && !language.trim().is_empty() {
        payload["language"] = serde_json::Value::String(language.to_string());
    }

    let response = client
        .post("https://openrouter.ai/api/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://github.com/gajsin/dicta")
        .header("X-OpenRouter-Title", "Dicta")
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("Сеть (OpenRouter STT): {}", err))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| format!("Чтение ответа (OpenRouter STT): {}", err))?;

    if !status.is_success() {
        return Err(format!("OpenRouter STT API {}: {}", status, body));
    }

    extract_text_from_json(&body, "openrouter")
}

pub async fn transcribe_audio_internal(
    provider: &str,
    provider_api_keys: &ProviderApiKeys,
    model: &str,
    language: &str,
    audio_base64: &str,
) -> Result<String, String> {
    match provider {
        "polza" => {
            transcribe_polza_internal(provider_api_keys, model, language, audio_base64).await
        }
        "openai" | "groq" => {
            transcribe_openai_compatible_internal(
                provider,
                provider_api_keys,
                model,
                language,
                audio_base64,
            )
            .await
        }
        "openrouter" => {
            transcribe_openrouter_internal(provider_api_keys, model, language, audio_base64).await
        }
        _ => Err(format!(
            "Провайдер распознавания '{}' не поддерживается",
            provider
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_audio_multipart_body, transcribe_polza_payload};
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn multipart_body_contains_required_transcription_fields() {
        let audio = general_purpose::STANDARD.encode(b"RIFF-test");
        let (boundary, body) =
            build_audio_multipart_body("whisper-1", "ru", &audio).expect("valid multipart body");
        let body = String::from_utf8_lossy(&body);

        assert!(body.contains(&format!("--{boundary}")));
        assert!(body.contains("name=\"model\"\r\n\r\nwhisper-1"));
        assert!(body.contains("name=\"language\"\r\n\r\nru"));
        assert!(body.contains("filename=\"dicta.wav\""));
        assert!(body.contains("RIFF-test"));
    }

    #[test]
    fn automatic_language_does_not_send_a_language_specific_prompt() {
        let audio = general_purpose::STANDARD.encode(b"RIFF-test");
        let (_, body) = build_audio_multipart_body("whisper-large-v3-turbo", "auto", &audio)
            .expect("multipart body");
        let body = String::from_utf8_lossy(&body);

        assert!(!body.contains("name=\"language\""));
        assert!(!body.contains("name=\"prompt\""));
    }

    #[test]
    fn invalid_audio_base64_is_rejected() {
        assert!(build_audio_multipart_body("whisper-1", "auto", "%%%").is_err());
    }

    #[test]
    fn polza_auto_language_is_omitted_and_legacy_model_is_normalized() {
        let payload = transcribe_polza_payload("whisper-large-v3", "auto", "audio");
        assert_eq!(payload["model"], "whisper-1");
        assert!(payload.get("language").is_none());
        assert!(payload.get("prompt").is_none());
    }

    #[test]
    fn polza_explicit_language_is_preserved() {
        let payload = transcribe_polza_payload("whisper-1", "ru", "audio");
        assert_eq!(payload["model"], "whisper-1");
        assert_eq!(payload["language"], "ru");
    }
}
