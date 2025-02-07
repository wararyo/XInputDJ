use std::sync::{Arc, Mutex};
use std::thread;
use rusty_xinput::XInputHandle;
use tauri::{AppHandle, Emitter};

lazy_static::lazy_static! {
    static ref RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
}

pub fn start_xinput_thread(app_handle: AppHandle) {
    let running = Arc::clone(&RUNNING);
    thread::spawn(move || {
        let handle = XInputHandle::load_default().unwrap();

        // Quick rumble test. Note that the controller might not _have_ rumble.
        println!("rumble on: {:?}", handle.set_state(0, 1000, 1000));
        thread::sleep(std::time::Duration::from_millis(160));
        println!("rumble off: {:?}", handle.set_state(0, 0, 0));

        // Show stick values, loop until the button is pressed to stop.
        loop {
            if !*running.lock().unwrap() {
                println!("end");
                break;
            }
            thread::sleep(std::time::Duration::from_millis(16));
            match handle.get_state(0) {
                Err(e) => {
                    println!("xinput_get_state error: {:?}", e);
                    break;
                }
                Ok(state) => {
                    if state.east_button() {
                        break;
                    } else {
                        let left_stick = state.left_stick_normalized();
                        let right_stick = state.right_stick_normalized();
                        app_handle.emit_to("main", "gamepad_input", (left_stick, right_stick)).unwrap();
                    }
                }
            }
        }
    });
}

pub fn stop_xinput_thread() {
    let mut running = RUNNING.lock().unwrap();
    *running = false;
}
