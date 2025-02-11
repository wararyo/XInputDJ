mod xinput_handler;
mod midi_handler;
mod input_mapper;
mod settings;

use crate::xinput_handler::{start_xinput_thread, stop_xinput_thread};
use crate::midi_handler::{open_midi_port, close_midi_port, get_midi_ports};
use crate::input_mapper::{start_mapping, stop_mapping};
use crate::settings::Settings;

#[tauri::command]
fn start_system(midi_port: String) -> Result<String, String> {
    open_midi_port(midi_port)?;
    let controller_sender = start_mapping();
    if let Err(e) = start_xinput_thread(controller_sender) {
        stop_mapping();
        close_midi_port();
        return Err(e);
    }
    // 接続したMIDIポートを保存
    Settings::set_default_midi_port(Some(midi_port.clone()))?;
    Ok("System started".to_string())
}

#[tauri::command]
fn stop_system() {
    stop_mapping();
    stop_xinput_thread();
    close_midi_port();
}

#[tauri::command]
fn get_settings() -> Settings {
    Settings::get_settings()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_system,
            stop_system,
            get_midi_ports,
            get_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
