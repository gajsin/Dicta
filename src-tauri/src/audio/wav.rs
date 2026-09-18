pub fn samples_to_wav_base64(samples: &[f32], sample_rate: u32, channels: u16) -> String {
    use base64::Engine;
    let num_samples = samples.len() as u32;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = num_samples * (bits_per_sample as u32 / 8);
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * 32767.0) as i16;
        wav.extend_from_slice(&i16_val.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(&wav)
}

#[cfg(test)]
mod tests {
    use super::samples_to_wav_base64;
    use base64::Engine;

    #[test]
    fn creates_valid_pcm_wav_header_and_payload() {
        let encoded = samples_to_wav_base64(&[-1.0, 0.0, 1.0], 16_000, 1);
        let wav = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .expect("valid base64");

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 6);
        assert_eq!(wav.len(), 50);
    }
}
