use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

pub fn resample_to_16k(samples: &[f32], from_sample_rate: u32) -> Vec<f32> {
    if from_sample_rate == 16000 {
        return samples.to_vec();
    }

    let chunk_size = 1024;
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 128,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = match SincFixedIn::<f32>::new(
        16000.0 / from_sample_rate as f64,
        2.0,
        params,
        chunk_size,
        1,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "[Dicta] Failed to create SincFixedIn resampler: {}, falling back to linear",
                e
            );
            return resample_linear(samples, from_sample_rate);
        }
    };

    let mut output = Vec::new();
    let mut input_buffer = vec![vec![0.0f32; chunk_size]; 1];

    let mut i = 0;
    while i < samples.len() {
        let end = (i + chunk_size).min(samples.len());
        let len = end - i;

        if len == chunk_size {
            input_buffer[0].copy_from_slice(&samples[i..end]);
            if let Ok(resampled) = resampler.process(&input_buffer, None) {
                output.extend_from_slice(&resampled[0]);
            }
        } else {
            input_buffer[0][..len].copy_from_slice(&samples[i..end]);
            for val in &mut input_buffer[0][len..] {
                *val = 0.0;
            }
            if let Ok(resampled) = resampler.process(&input_buffer, None) {
                let ratio = 16000.0 / from_sample_rate as f64;
                let expected_out_len = (len as f64 * ratio).round() as usize;
                let resampled_chunk = &resampled[0];
                let out_len = expected_out_len.min(resampled_chunk.len());
                output.extend_from_slice(&resampled_chunk[..out_len]);
            }
        }
        i += chunk_size;
    }

    output
}

fn resample_linear(samples: &[f32], from_sample_rate: u32) -> Vec<f32> {
    let mut resampled = Vec::new();
    let ratio = from_sample_rate as f64 / 16000.0;
    let new_len = (samples.len() as f64 / ratio).floor() as usize;
    resampled.reserve(new_len);
    for i in 0..new_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f64;
        if idx + 1 < samples.len() {
            let val = samples[idx] * (1.0 - frac) as f32 + samples[idx + 1] * frac as f32;
            resampled.push(val);
        } else if idx < samples.len() {
            resampled.push(samples[idx]);
        }
    }
    resampled
}

#[cfg(test)]
mod tests {
    use super::resample_to_16k;

    #[test]
    fn leaves_native_16khz_samples_unchanged() {
        let samples = vec![-0.5, 0.0, 0.5];
        assert_eq!(resample_to_16k(&samples, 16_000), samples);
    }

    #[test]
    fn resamples_48khz_to_expected_duration() {
        let samples = vec![0.25; 48_000];
        let result = resample_to_16k(&samples, 48_000);
        assert!(
            (15_900..=16_100).contains(&result.len()),
            "unexpected output length: {}",
            result.len()
        );
    }
}
