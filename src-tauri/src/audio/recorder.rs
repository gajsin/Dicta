use cpal::traits::{DeviceTrait, StreamTrait};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use tauri::Emitter;

use super::resample::resample_to_16k;
use super::wav::samples_to_wav_base64;

pub struct SendStream(pub cpal::Stream);
unsafe impl Send for SendStream {}

pub struct RecordingState {
    pub stream: SendStream,
    pub sample_rate: u32,
    pub tx_stop: Sender<()>,
    pub rx_result: Receiver<Vec<f32>>,
    pub worker: JoinHandle<()>,
}

pub fn start_recording_internal(
    app: tauri::AppHandle,
    device_label: Option<String>,
    recording_mutex: &std::sync::Mutex<Option<RecordingState>>,
) -> Result<(), String> {
    cancel_recording_internal(recording_mutex)?;

    let host = cpal::default_host();
    let device = super::devices::select_input_device(&host, device_label.as_deref())?;

    let supported = device
        .default_input_config()
        .map_err(|e| format!("Ошибка конфигурации микрофона: {}", e))?;

    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels();
    let sample_fmt = supported.sample_format();
    eprintln!(
        "[Dicta] Mic: {}Hz, {}ch, {:?}",
        sample_rate, channels, sample_fmt
    );

    let config = cpal::StreamConfig {
        channels,
        sample_rate: supported.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };

    let (tx_samples, rx_samples) = channel::<Vec<f32>>();
    let (tx_stop, rx_stop) = channel::<()>();
    let (tx_result, rx_result) = channel::<Vec<f32>>();

    let app_clone = app.clone();
    let sample_rate_val = sample_rate;
    let channels_val = channels;

    let worker = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut last_emit = std::time::Instant::now();
        let max_samples = 10 * 60 * sample_rate_val; // 10 minutes limit

        loop {
            if rx_stop.try_recv().is_ok() {
                break;
            }

            match rx_samples.recv_timeout(std::time::Duration::from_millis(10)) {
                Ok(data) => {
                    for chunk in data.chunks_exact(channels_val as usize) {
                        let sum: f32 = chunk.iter().sum();
                        buffer.push(sum / channels_val as f32);
                    }

                    if buffer.len() >= max_samples as usize {
                        eprintln!("[Dicta] Maximum recording length reached (10 min)");
                        let _ = app_clone.emit(
                            "recording-timeout",
                            "Максимальная длительность записи (10 минут) достигнута.",
                        );
                        break;
                    }

                    if !data.is_empty() {
                        let sum_sq: f32 = data.iter().map(|&s| s * s).sum();
                        let rms = (sum_sq / data.len() as f32).sqrt();

                        if last_emit.elapsed() >= std::time::Duration::from_millis(80) {
                            let normalized = ((rms - 0.002).max(0.0) * 22.0).min(1.0);
                            let _ = app_clone.emit("audio-level", normalized);
                            last_emit = std::time::Instant::now();
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        let _ = tx_result.send(buffer);
    });

    let app_err = app.clone();
    let err_cb = move |err: cpal::StreamError| {
        let err_str = err.to_string();
        eprintln!("[Dicta] Mic stream error: {}", err_str);
        if !err_str.contains("device no longer available") && !err_str.contains("closed") {
            let _ = app_err.emit(
                "recording-device-error",
                format!("Микрофон недоступен: {}", err_str),
            );
        }
    };
    let tx_samples_cb = tx_samples.clone();

    let stream = match sample_fmt {
        cpal::SampleFormat::F32 => {
            let tx = tx_samples_cb.clone();
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let _ = tx.send(data.to_vec());
                },
                err_cb,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let tx = tx_samples_cb.clone();
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    let f32_data: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    let _ = tx.send(f32_data);
                },
                err_cb,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let tx = tx_samples_cb.clone();
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    let f32_data: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    let _ = tx.send(f32_data);
                },
                err_cb,
                None,
            )
        }
        other => return Err(format!("Неподдерживаемый формат сэмплов: {:?}", other)),
    }
    .map_err(|e| format!("Не удалось создать аудиопоток: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Не удалось начать запись: {}", e))?;

    let mut state = recording_mutex.lock().map_err(|_| "Lock poisoned")?;
    *state = Some(RecordingState {
        stream: SendStream(stream),
        sample_rate,
        tx_stop,
        rx_result,
        worker,
    });

    eprintln!("[Dicta] ● Recording started");
    Ok(())
}

pub fn stop_recording_internal(
    recording_mutex: &std::sync::Mutex<Option<RecordingState>>,
) -> Result<String, String> {
    let mut state = recording_mutex.lock().map_err(|_| "Lock poisoned")?;
    let rec = state.take().ok_or("Запись не была начата")?;

    let RecordingState {
        stream,
        sample_rate,
        tx_stop,
        rx_result,
        worker,
    } = rec;

    let _ = stream.0.pause();
    drop(stream);

    let _ = tx_stop.send(());
    worker
        .join()
        .map_err(|_| "Поток обработки аудио аварийно завершился".to_string())?;
    let samples = rx_result
        .recv()
        .map_err(|_| "Не удалось получить аудиоданные из потока".to_string())?;

    eprintln!(
        "[Dicta] Recorded {} mono samples ({:.1}s) at native {}Hz",
        samples.len(),
        samples.len() as f64 / sample_rate as f64,
        sample_rate
    );

    if samples.is_empty() {
        return Err("Аудиобуфер пуст — запись не захватила данные".to_string());
    }

    let samples_16k = resample_to_16k(&samples, sample_rate);
    let b64 = samples_to_wav_base64(&samples_16k, 16000, 1);
    eprintln!("[Dicta] WAV mono 16kHz base64: {} bytes", b64.len());
    Ok(b64)
}

pub fn cancel_recording_internal(
    recording_mutex: &std::sync::Mutex<Option<RecordingState>>,
) -> Result<(), String> {
    let recording = {
        let mut state = recording_mutex.lock().map_err(|_| "Lock poisoned")?;
        state.take()
    };

    let Some(recording) = recording else {
        return Ok(());
    };

    let RecordingState {
        stream,
        tx_stop,
        rx_result,
        worker,
        ..
    } = recording;

    let pause_result = stream
        .0
        .pause()
        .map_err(|error| format!("Не удалось остановить аудиопоток: {error}"));
    drop(stream);
    let _ = tx_stop.send(());

    worker
        .join()
        .map_err(|_| "Поток обработки аудио аварийно завершился".to_string())?;
    drop(rx_result);
    pause_result?;

    eprintln!("[Dicta] ○ Recording cancelled");
    Ok(())
}
