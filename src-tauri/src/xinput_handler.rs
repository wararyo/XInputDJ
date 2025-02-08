use std::sync::{Arc, Mutex};
use std::thread;
use rusty_xinput::XInputHandle;
use tauri::{AppHandle, Emitter};

lazy_static::lazy_static! {
    static ref RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

// XInputからの入力の受け取りを開始する
pub fn start_xinput_thread(app_handle: AppHandle) {
    let running = Arc::clone(&RUNNING);
    {
        // すでにスレッドが動いている場合は何もしない
        let mut guard = running.lock().unwrap();
        if *guard {
            return;
        }
        *guard = true;
    }

    thread::spawn(move || {
        let handle = match XInputHandle::load_default() {
            Ok(h) => h,
            Err(e) => {
                app_handle.emit_to("main", "gamepad_error", format!("Failed to initialize XInput: {:?}", e)).unwrap();
                return;
            }
        };

        let mut consecutive_errors = 0;
        const MAX_ERRORS: u32 = 5; // この回数だけエラーが続いたらコントローラーが切断されたとみなす

        loop {
            if !*running.lock().unwrap() {
                break;
            }

            thread::sleep(std::time::Duration::from_millis(16));
            match handle.get_state(0) {
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors >= MAX_ERRORS {
                        app_handle.emit_to("main", "gamepad_error", format!("Controller disconnected: {:?}", e)).unwrap();
                        break;
                    }
                }
                Ok(state) => {
                    consecutive_errors = 0;
                    if state.east_button() {
                        break;
                    } else {
                        let left_stick = state.left_stick_normalized();
                        let right_stick = state.right_stick_normalized();
                        if let Err(e) = app_handle.emit_to("main", "gamepad_input", (left_stick, right_stick)) {
                            eprintln!("Failed to emit gamepad input: {:?}", e);
                        }
                    }
                }
            }
        }

        *running.lock().unwrap() = false;
    });
}

// XInputからの入力の受け取りを停止する
pub fn stop_xinput_thread() {
    let mut running = RUNNING.lock().unwrap();
    *running = false;
}
