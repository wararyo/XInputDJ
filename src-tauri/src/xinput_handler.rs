use std::sync::{Arc, Mutex};
use std::thread;
use rusty_xinput::XInputHandle;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct StickState {
    pub left: [f32; 2],
    pub right: [f32; 2],
}

#[derive(Debug)]
pub struct ButtonState {
    pub south: bool,
    pub east: bool,
    pub west: bool,
    pub north: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub l: bool,
    pub lt: bool,
    pub r: bool,
    pub rt: bool,
    pub l_stick: bool,
    pub r_stick: bool,
}

#[derive(Debug)]
pub struct ControllerState {
    pub sticks: StickState,
    pub buttons: ButtonState,
}

lazy_static::lazy_static! {
    static ref RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

// XInputからの入力の受け取りを開始する
pub fn start_xinput_thread(state_sender: Sender<ControllerState>) -> Result<(), String> {
    let running = Arc::clone(&RUNNING);
    {
        // すでにスレッドが動いている場合は何もしない
        let mut guard = running.lock().unwrap();
        if *guard {
            return Ok(());
        }
        *guard = true;
    }

    let handle = XInputHandle::load_default()
        .map_err(|e| {
            *running.lock().unwrap() = false;
            format!("Failed to initialize XInput: {:?}", e)
        })?;

    // 初回にコントローラーの状態が取得できなければエラーとする
    // 現状のコードではスレッド開始後にコントローラーが切断された場合にエラーを通知できないが、一旦許容する
    handle.get_state(0)
        .map_err(|e| {
            *running.lock().unwrap() = false;
            format!("Failed to get initial controller state: {:?}", e)
        })?;

    thread::spawn(move || {
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
                        eprintln!("Controller disconnected: {:?}", e);
                        break;
                    }
                }
                Ok(state) => {
                    consecutive_errors = 0;
                    
                    let left_stick = state.left_stick_normalized();
                    let right_stick = state.right_stick_normalized();
                    
                    let controller_state = ControllerState {
                        sticks: StickState {
                            left: [left_stick.0, left_stick.1],
                            right: [right_stick.0, right_stick.1],
                        },
                        buttons: ButtonState {
                            south: state.south_button(),      // A button
                            east: state.east_button(),        // B button
                            west: state.west_button(),        // X button
                            north: state.north_button(),      // Y button
                            up: state.arrow_up(),
                            down: state.arrow_down(),
                            left: state.arrow_left(),
                            right: state.arrow_right(),
                            l: state.left_shoulder(),
                            lt: state.left_trigger_bool(),
                            r: state.right_shoulder(),
                            rt: state.right_trigger_bool(),
                            l_stick: state.left_thumb_button(),
                            r_stick: state.right_thumb_button(),
                        },
                    };
                    
                    if let Err(e) = state_sender.send(controller_state) {
                        eprintln!("Failed to send controller state: {:?}", e);
                        break;
                    }
                }
            }
        }

        *running.lock().unwrap() = false;
    });

    Ok(())
}

// XInputからの入力の受け取りを停止する
pub fn stop_xinput_thread() {
    let mut running = RUNNING.lock().unwrap();
    *running = false;
}
