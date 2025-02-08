mod xinput_handler;
mod midi_handler;

use crate::xinput_handler::{start_xinput_thread, stop_xinput_thread};
use crate::midi_handler::{open_midi_port, close_midi_port, get_midi_ports, send_cc_change};
use tauri::AppHandle;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_gamepad_thread(app_handle: AppHandle) {
    start_xinput_thread(app_handle);
}

#[tauri::command]
fn stop_gamepad_thread() {
    stop_xinput_thread();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_gamepad_thread,
            stop_gamepad_thread,
            open_midi_port,
            close_midi_port,
            get_midi_ports,
            send_cc_change
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
