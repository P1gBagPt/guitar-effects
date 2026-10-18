pub mod audio;
pub mod decoder;
pub mod dsp;
pub mod tabs;

use audio::device::AudioHostInfo;
use audio::{AudioEngine, AudioTelemetry};
use dsp::PedalMeta;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub audio_engine: Arc<Mutex<AudioEngine>>,
}

#[tauri::command]
fn get_audio_hosts(state: State<'_, AppState>) -> AudioHostInfo {
    state.audio_engine.lock().get_host_info()
}

#[tauri::command]
fn start_audio_engine(
    state: State<'_, AppState>,
    input_device: Option<String>,
    output_device: Option<String>,
    sample_rate: u32,
    buffer_size: u32,
) -> Result<(), String> {
    state.audio_engine.lock().start_engine(
        input_device.as_deref(),
        output_device.as_deref(),
        sample_rate,
        buffer_size,
    )
}

#[tauri::command]
fn stop_audio_engine(state: State<'_, AppState>) {
    state.audio_engine.lock().stop_engine();
}

#[tauri::command]
fn get_pedalboard(state: State<'_, AppState>) -> Vec<PedalMeta> {
    state.audio_engine.lock().get_pedals()
}

#[tauri::command]
fn set_pedal_state(state: State<'_, AppState>, pedal_id: String, enabled: bool) -> bool {
    state.audio_engine.lock().set_pedal_enabled(&pedal_id, enabled)
}

#[tauri::command]
fn set_pedal_parameter(state: State<'_, AppState>, pedal_id: String, param_id: u32, value: f32) -> bool {
    state.audio_engine.lock().set_pedal_param(&pedal_id, param_id, value)
}

#[tauri::command]
fn reorder_pedalboard(state: State<'_, AppState>, new_order: Vec<String>) {
    state.audio_engine.lock().reorder_pedals(&new_order);
}

#[tauri::command]
fn set_channel_volumes(state: State<'_, AppState>, guitar: f32, backing: f32, master: f32) {
    let eng = state.audio_engine.lock();
    eng.set_guitar_volume(guitar);
    eng.set_backing_volume(backing);
    eng.set_master_volume(master);
}

#[tauri::command]
fn get_audio_telemetry(state: State<'_, AppState>) -> AudioTelemetry {
    state.audio_engine.lock().get_telemetry()
}

#[tauri::command]
fn transport_play(state: State<'_, AppState>) {
    state.audio_engine.lock().play_transport();
}

#[tauri::command]
fn transport_pause(state: State<'_, AppState>) {
    state.audio_engine.lock().pause_transport();
}

#[tauri::command]
fn transport_seek(state: State<'_, AppState>, seconds: f64) {
    state.audio_engine.lock().seek_transport(seconds);
}

#[tauri::command]
fn load_backing_track_file(state: State<'_, AppState>, file_path: String) -> Result<f64, String> {
    let path = Path::new(&file_path);
    let decoded = decoder::decode_audio_file(path, 48000)?;
    let duration = decoded.duration_seconds;
    state.audio_engine.lock().load_backing_track(decoded);
    Ok(duration)
}

#[tauri::command]
fn start_recording_session(state: State<'_, AppState>, file_path: String) -> Result<(), String> {
    state.audio_engine.lock().start_recording(PathBuf::from(file_path))
}

#[tauri::command]
fn stop_recording_session(state: State<'_, AppState>) -> Result<(), String> {
    state.audio_engine.lock().stop_recording()
}

#[tauri::command]
fn import_tab_url(url: String) -> Result<(String, String), String> {
    let importer = tabs::scraper::TabImporter::new();
    importer.fetch_tab_base64(&url)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let audio_engine = Arc::new(Mutex::new(AudioEngine::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(AppState {
            audio_engine,
        })
        .invoke_handler(tauri::generate_handler![
            get_audio_hosts,
            start_audio_engine,
            stop_audio_engine,
            get_pedalboard,
            set_pedal_state,
            set_pedal_parameter,
            reorder_pedalboard,
            set_channel_volumes,
            get_audio_telemetry,
            transport_play,
            transport_pause,
            transport_seek,
            load_backing_track_file,
            start_recording_session,
            stop_recording_session,
            import_tab_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running guitar workstation application");
}
