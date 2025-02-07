mod xinput_handler;

use crate::xinput_handler::{start_xinput_thread, stop_xinput_thread};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_gamepad_thread() {
    start_xinput_thread();
}

#[tauri::command]
fn stop_gamepad_thread() {
    stop_xinput_thread();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_gamepad_thread, stop_gamepad_thread])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
