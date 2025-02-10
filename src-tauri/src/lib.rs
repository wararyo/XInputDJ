mod xinput_handler;
mod midi_handler;
mod input_mapper;

use crate::xinput_handler::{start_xinput_thread, stop_xinput_thread};
use crate::midi_handler::{open_midi_port, close_midi_port, get_midi_ports};
use crate::input_mapper::{start_mapping, stop_mapping};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_system(midi_port: String) -> Result<String, String> {
    open_midi_port(midi_port)?;
    let controller_sender = start_mapping();
    start_xinput_thread(controller_sender);
    Ok("System started".to_string())
}

#[tauri::command]
fn stop_system() {
    stop_mapping();
    stop_xinput_thread();
    close_midi_port();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_system,
            stop_system,
            get_midi_ports,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
