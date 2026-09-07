pub mod postprocessing;
pub mod stt;

use crate::config::ProviderApiKeys;
pub use postprocessing::*;
use serde::{Deserialize, Serialize};
pub use stt::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
    pub raw_text: String,
    pub processed_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionOptions {
    pub provider_api_keys: ProviderApiKeys,
    pub stt_provider: String,
    pub llm_provider: String,
    pub model: String,
    pub language: String,
    pub postprocess_enabled: bool,
    pub postprocess_mode: String,
    pub postprocess_model: String,
}

pub async fn try_transcribe_and_polish(
    base64_audio: &str,
    options: &TranscriptionOptions,
) -> Result<TranscriptionResult, String> {
    validate_postprocess_mode(&options.postprocess_mode)?;

    let raw_text = transcribe_audio_internal(
        &options.stt_provider,
        &options.provider_api_keys,
        &options.model,
        &options.language,
        base64_audio,
    )
    .await?;

    let processed_text =
        if options.postprocess_enabled && !raw_text.is_empty() && options.postprocess_mode != "raw"
        {
            match polish_transcript_internal(
                &options.llm_provider,
                &options.provider_api_keys,
                &raw_text,
                &options.postprocess_mode,
                &options.postprocess_model,
            )
            .await
            {
                Ok(polished) => polished,
                Err(e) => {
                    eprintln!("[Dicta] LLM polishing failed: {}, using raw STT text", e);
                    raw_text.clone()
                }
            }
        } else {
            raw_text.clone()
        };

    Ok(TranscriptionResult {
        raw_text,
        processed_text,
    })
}
