use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use super::recorder::SendStream;

pub struct MonitorBuffer {
    samples: VecDeque<f32>,
    capacity: usize,
    prefill: usize,
    started: bool,
}

impl MonitorBuffer {
    pub fn with_prefill(capacity: usize, prefill: usize) -> Self {
        let prefill = prefill.min(capacity);
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity,
            prefill,
            started: prefill == 0,
        }
    }

    pub fn push(&mut self, sample: f32) {
        if self.capacity == 0 {
            return;
        }
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn pop(&mut self) -> f32 {
        if !self.started {
            if self.samples.len() < self.prefill {
                return 0.0;
            }
            self.started = true;
        }

        let sample = self.samples.pop_front().unwrap_or(0.0);
        if self.samples.is_empty() && self.prefill > 0 {
            self.started = false;
        }
        sample
    }
}

pub struct LoopbackState {
    pub input_stream: SendStream,
    pub output_stream: SendStream,
}

pub fn start_loopback_internal(
    app: tauri::AppHandle,
    device_label: Option<String>,
    loopback_mutex: &Mutex<Option<LoopbackState>>,
) -> Result<(), String> {
    {
        let mut state = loopback_mutex.lock().map_err(|_| "Lock poisoned")?;
        if let Some(old) = state.take() {
            let _ = old.input_stream.0.pause();
            let _ = old.output_stream.0.pause();
        }
    }

    let host = cpal::default_host();
    let input_device = super::devices::select_input_device(&host, device_label.as_deref())?;
    let output_device = host
        .default_output_device()
        .ok_or("Устройство вывода звука не найдено.")?;
    let output_config = output_device
        .default_output_config()
        .map_err(|e| format!("Ошибка конфигурации устройства вывода: {e}"))?;
    let input_config = compatible_input_config(&input_device, output_config.sample_rate())?;

    let in_rate = input_config.sample_rate().0;
    let in_channels = input_config.channels() as usize;
    let in_fmt = input_config.sample_format();
    let out_channels = output_config.channels() as usize;
    let out_fmt = output_config.sample_format();

    eprintln!(
        "[Dicta] Microphone monitor: {}Hz, input {}ch {:?}, output {}ch {:?}",
        in_rate, in_channels, in_fmt, out_channels, out_fmt
    );

    let in_stream_config = cpal::StreamConfig {
        channels: input_config.channels(),
        sample_rate: input_config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };
    let out_stream_config = cpal::StreamConfig {
        channels: output_config.channels(),
        sample_rate: output_config.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let err_in = |err: cpal::StreamError| eprintln!("[Dicta] Microphone test error: {err}");
    let err_out =
        |err: cpal::StreamError| eprintln!("[Dicta] Microphone monitor output error: {err}");
    let monitor_buffer = Arc::new(Mutex::new(MonitorBuffer::with_prefill(
        (in_rate / 4) as usize,
        (in_rate / 25) as usize,
    )));

    let output_stream = match out_fmt {
        cpal::SampleFormat::F32 => device_output_monitor_build::<f32>(
            &output_device,
            &out_stream_config,
            monitor_buffer.clone(),
            err_out,
        )?,
        cpal::SampleFormat::I16 => device_output_monitor_build::<i16>(
            &output_device,
            &out_stream_config,
            monitor_buffer.clone(),
            err_out,
        )?,
        cpal::SampleFormat::U16 => device_output_monitor_build::<u16>(
            &output_device,
            &out_stream_config,
            monitor_buffer.clone(),
            err_out,
        )?,
        other => return Err(format!("Неподдержимый формат устройства вывода: {other:?}")),
    };

    let input_stream = match in_fmt {
        cpal::SampleFormat::F32 => device_input_monitor_build::<f32>(
            &input_device,
            &in_stream_config,
            app,
            monitor_buffer,
            err_in,
        )?,
        cpal::SampleFormat::I16 => device_input_monitor_build::<i16>(
            &input_device,
            &in_stream_config,
            app,
            monitor_buffer,
            err_in,
        )?,
        cpal::SampleFormat::U16 => device_input_monitor_build::<u16>(
            &input_device,
            &in_stream_config,
            app,
            monitor_buffer,
            err_in,
        )?,
        other => return Err(format!("Неподдержимый формат микрофона: {other:?}")),
    };

    output_stream
        .play()
        .map_err(|e| format!("Не удалось запустить воспроизведение микрофона: {e}"))?;
    input_stream.play().map_err(|e| {
        let _ = output_stream.pause();
        format!("Не удалось запустить микрофон: {e}")
    })?;

    let mut state = loopback_mutex.lock().map_err(|_| "Lock poisoned")?;
    *state = Some(LoopbackState {
        input_stream: SendStream(input_stream),
        output_stream: SendStream(output_stream),
    });
    eprintln!("[Dicta] ● Microphone monitor started");
    Ok(())
}

pub fn stop_loopback_internal(loopback_mutex: &Mutex<Option<LoopbackState>>) -> Result<(), String> {
    let mut state = loopback_mutex.lock().map_err(|_| "Lock poisoned")?;
    if let Some(loopback) = state.take() {
        let _ = loopback.input_stream.0.pause();
        let _ = loopback.output_stream.0.pause();
        eprintln!("[Dicta] ○ Microphone monitor stopped");
    }
    Ok(())
}

fn compatible_input_config(
    device: &cpal::Device,
    sample_rate: cpal::SampleRate,
) -> Result<cpal::SupportedStreamConfig, String> {
    let default = device
        .default_input_config()
        .map_err(|e| format!("Ошибка конфигурации микрофона: {e}"))?;
    if default.sample_rate() == sample_rate {
        return Ok(default);
    }

    let preferred_channels = default.channels();
    device
        .supported_input_configs()
        .map_err(|e| format!("Не удалось получить режимы микрофона: {e}"))?
        .filter(|config| {
            config.min_sample_rate() <= sample_rate && sample_rate <= config.max_sample_rate()
        })
        .min_by_key(|config| u8::from(config.channels() != preferred_channels))
        .map(|config| config.with_sample_rate(sample_rate))
        .ok_or_else(|| {
            format!(
                "Микрофон не поддерживает частоту устройства вывода {} Гц",
                sample_rate.0
            )
        })
}

fn device_input_monitor_build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    app: tauri::AppHandle,
    monitor_buffer: Arc<Mutex<MonitorBuffer>>,
    err_cb: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, String>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let mut last_emit = std::time::Instant::now();
    let channels = config.channels as usize;
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if data.is_empty() {
                    return;
                }

                if let Ok(mut buffer) = monitor_buffer.try_lock() {
                    for frame in data.chunks(channels) {
                        let mono = frame
                            .iter()
                            .map(|sample| cpal::Sample::to_sample::<f32>(*sample))
                            .sum::<f32>()
                            / frame.len() as f32;
                        buffer.push(mono);
                    }
                }

                if last_emit.elapsed() >= std::time::Duration::from_millis(80) {
                    let sum_sq = data
                        .iter()
                        .map(|sample| {
                            let sample = cpal::Sample::to_sample::<f32>(*sample);
                            sample * sample
                        })
                        .sum::<f32>();
                    let normalized = ((sum_sq / data.len() as f32).sqrt() * 16.0).min(1.0);
                    let _ = app.emit("loopback-audio-level", normalized);
                    last_emit = std::time::Instant::now();
                }
            },
            err_cb,
            None,
        )
        .map_err(|e| format!("Не удалось создать входной поток: {e}"))
}

fn device_output_monitor_build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    monitor_buffer: Arc<Mutex<MonitorBuffer>>,
    err_cb: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, String>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    const MONITOR_GAIN: f32 = 0.8;
    let channels = config.channels as usize;
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                if let Ok(mut buffer) = monitor_buffer.try_lock() {
                    for frame in output.chunks_mut(channels) {
                        let sample = T::from_sample(buffer.pop() * MONITOR_GAIN);
                        frame.fill(sample);
                    }
                } else {
                    output.fill(T::from_sample(0.0));
                }
            },
            err_cb,
            None,
        )
        .map_err(|e| format!("Не удалось создать выходной поток: {e}"))
}

#[cfg(test)]
mod tests {
    use super::MonitorBuffer;

    #[test]
    fn live_monitor_preserves_sample_order() {
        let mut buffer = MonitorBuffer::with_prefill(4, 0);
        buffer.push(0.25);
        buffer.push(-0.5);

        assert_eq!(buffer.pop(), 0.25);
        assert_eq!(buffer.pop(), -0.5);
        assert_eq!(buffer.pop(), 0.0);
    }

    #[test]
    fn live_monitor_discards_oldest_samples_when_latency_limit_is_reached() {
        let mut buffer = MonitorBuffer::with_prefill(2, 0);
        buffer.push(0.1);
        buffer.push(0.2);
        buffer.push(0.3);

        assert_eq!(buffer.pop(), 0.2);
        assert_eq!(buffer.pop(), 0.3);
    }

    #[test]
    fn live_monitor_prefills_before_playback_and_recovers_after_underflow() {
        let mut buffer = MonitorBuffer::with_prefill(8, 2);
        buffer.push(0.4);
        assert_eq!(buffer.pop(), 0.0);

        buffer.push(0.5);
        assert_eq!(buffer.pop(), 0.4);
        assert_eq!(buffer.pop(), 0.5);
        assert_eq!(buffer.pop(), 0.0);

        buffer.push(0.6);
        assert_eq!(buffer.pop(), 0.0);
        buffer.push(0.7);
        assert_eq!(buffer.pop(), 0.6);
    }
}
