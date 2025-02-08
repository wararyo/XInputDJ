use crate::midi_handler::send_cc_change;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::f32::consts::PI;
use crate::xinput_handler::StickState;

lazy_static::lazy_static! {
    static ref RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub fn start_mapping() -> Sender<StickState> {
    let (tx, rx) = channel::<StickState>();
    let running = Arc::clone(&RUNNING);
    {
        let mut guard = running.lock().unwrap();
        *guard = true;
    }

    thread::spawn(move || {
        handle_stick_events(rx);
    });

    tx
}

pub fn stop_mapping() {
    let mut running = RUNNING.lock().unwrap();
    *running = false;
}

fn handle_stick_events(rx: Receiver<StickState>) {
    const DEADZONE: f32 = 0.75; // スティックの入力を無視する範囲（中心からの距離）

    while *RUNNING.lock().unwrap() {
        match rx.recv() {
            Ok(state) => {
                // スティックの座標から距離と角度を計算
                let [x, y] = state.left;
                let distance = (x * x + y * y).sqrt();
                
                // デッドゾーン以下の入力は無視
                if distance < DEADZONE {
                    continue;
                }

                // 角度を計算（アークタンジェント）
                // 12時方向が0、反時計回りに回すにつれて-πへと減少、時計回りに回すにつれてπへと増加、6時方向が境界
                let angle = f32::atan2(x, y);
                
                // 角度を0.0から1.0の範囲に正規化（8時付近で0.0、12時方向が0.5、4時付近で1.0）
                let mut value = ((angle / PI) * 1.2 / 2.0) + 0.5;

                // 値を0.0から1.0の範囲にクリップ
                value = value.max(0.0).min(1.0);

                // MIDI値に変換（0-127）
                let midi_value = (value * 127.0) as u8;
                
                // CC#11 (Expression)にY座標を送信
                if let Err(e) = send_cc_change(0, 11, midi_value) {
                    eprintln!("Failed to send MIDI CC: {:?}", e);
                }
            }
            Err(_) => break,
        }
    }
}
