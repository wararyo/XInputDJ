use crate::midi_handler::send_cc_change;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::f32::consts::PI;
use crate::xinput_handler::{ControllerState, ButtonState};

#[derive(Debug, Clone, Copy)]
enum StickType {
    Left,
    Right,
}

impl StickType {
    fn midi_channel(&self) -> u8 {
        match self {
            StickType::Left => 0,
            StickType::Right => 1,
        }
    }
}

// CCマッピング用の構造体
struct CCMapping {
    button_getter: fn(&ButtonState) -> bool,
    cc_number: u8,
    description: &'static str,
    stick: StickType,
}

lazy_static::lazy_static! {
    static ref RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CURRENT_CC: Arc<Mutex<[(StickType, u8); 2]>> = Arc::new(Mutex::new([
        (StickType::Left, 11),
        (StickType::Right, 1),
    ]));

    // すべてのCCマッピング
    static ref CC_MAPPINGS: Vec<CCMapping> = vec![
        // 左スティックのマッピング
        CCMapping { button_getter: |b| b.down, cc_number: 1, description: "Down", stick: StickType::Left },
        CCMapping { button_getter: |b| b.left, cc_number: 2, description: "Left", stick: StickType::Left },
        CCMapping { button_getter: |b| b.up, cc_number: 3, description: "Up", stick: StickType::Left },
        CCMapping { button_getter: |b| b.right, cc_number: 4, description: "Right", stick: StickType::Left },
        CCMapping { button_getter: |b| b.l, cc_number: 5, description: "L", stick: StickType::Left },
        CCMapping { button_getter: |b| b.lt, cc_number: 6, description: "LT", stick: StickType::Left },
        CCMapping { button_getter: |b| b.l_stick, cc_number: 7, description: "L stick", stick: StickType::Left },
        
        // 右スティックのマッピング
        CCMapping { button_getter: |b| b.south, cc_number: 1, description: "South", stick: StickType::Right },
        CCMapping { button_getter: |b| b.east, cc_number: 2, description: "East", stick: StickType::Right },
        CCMapping { button_getter: |b| b.north, cc_number: 3, description: "North", stick: StickType::Right },
        CCMapping { button_getter: |b| b.west, cc_number: 4, description: "West", stick: StickType::Right },
        CCMapping { button_getter: |b| b.r, cc_number: 5, description: "R", stick: StickType::Right },
        CCMapping { button_getter: |b| b.rt, cc_number: 6, description: "RT", stick: StickType::Right },
        CCMapping { button_getter: |b| b.r_stick, cc_number: 7, description: "R stick", stick: StickType::Right },
    ];
}

fn get_current_cc(stick: StickType) -> u8 {
    let current_cc = CURRENT_CC.lock().unwrap();
    match stick {
        StickType::Left => current_cc[0].1,
        StickType::Right => current_cc[1].1,
    }
}

fn set_current_cc(stick: StickType, cc: u8) {
    let mut current_cc = CURRENT_CC.lock().unwrap();
    match stick {
        StickType::Left => current_cc[0].1 = cc,
        StickType::Right => current_cc[1].1 = cc,
    }
}

pub fn start_mapping() -> Sender<ControllerState> {
    let (tx, rx) = channel::<ControllerState>();
    let running = Arc::clone(&RUNNING);
    {
        let mut guard = running.lock().unwrap();
        *guard = true;
    }

    println!("\nInitial CC mappings:");
    println!("Left stick: CC#{}", get_current_cc(StickType::Left));
    println!("Right stick: CC#{}", get_current_cc(StickType::Right));
    println!("\nAvailable CC mappings:");
    println!("Left stick:");
    for mapping in CC_MAPPINGS.iter().filter(|m| matches!(m.stick, StickType::Left)) {
        println!("  {} button: CC#{}", mapping.description, mapping.cc_number);
    }
    println!("Right stick:");
    for mapping in CC_MAPPINGS.iter().filter(|m| matches!(m.stick, StickType::Right)) {
        println!("  {} button: CC#{}", mapping.description, mapping.cc_number);
    }
    println!("");

    thread::spawn(move || {
        handle_controller_events(rx);
    });

    tx
}

pub fn stop_mapping() {
    let mut running = RUNNING.lock().unwrap();
    *running = false;
}

fn calculate_midi_cc_value(x: f32, y: f32, deadzone: f32) -> Option<u8> {
    let distance = (x * x + y * y).sqrt();
    
    if distance < deadzone {
        return None;
    }

    // 角度を計算（アークタンジェント）
    // 12時方向が0、反時計回りに回すにつれて-πへと減少、時計回りに回すにつれてπへと増加、6時方向が境界
    let angle = f32::atan2(x, y);
    
    // 角度を0.0から1.0の範囲に正規化（8時付近で0.0、12時方向が0.5、4時付近で1.0）
    let mut value = ((angle / PI) * 1.2 / 2.0) + 0.5;

    // 値を0.0から1.0の範囲にクリップ
    value = value.max(0.0).min(1.0);

    // MIDI値に変換（0-127）
    Some((value * 127.0) as u8)
}

fn update_cc_if_changed(stick: StickType, new_cc: u8, description: &str, last_cc: &mut u8) -> bool {
    if new_cc != *last_cc {
        *last_cc = new_cc;
        set_current_cc(stick, new_cc);
        println!("{} stick CC changed to: {} ({})", 
            match stick {
                StickType::Left => "Left",
                StickType::Right => "Right",
            },
            new_cc,
            description
        );
        true
    } else {
        false
    }
}

fn process_stick(x: f32, y: f32, cc: u8, stick: StickType, deadzone: f32) {
    if let Some(midi_value) = calculate_midi_cc_value(x, y, deadzone) {
        if let Err(e) = send_cc_change(stick.midi_channel(), cc, midi_value) {
            eprintln!("Failed to send MIDI CC ({} Stick): {:?}", 
                match stick {
                    StickType::Left => "Left",
                    StickType::Right => "Right",
                }, 
                e
            );
        }
    }
}

fn process_cc_mapping(state: &ControllerState, last_left_cc: &mut u8, last_right_cc: &mut u8) {
    for mapping in CC_MAPPINGS.iter() {
        if (mapping.button_getter)(&state.buttons) {
            match mapping.stick {
                StickType::Left => {
                    update_cc_if_changed(StickType::Left, mapping.cc_number, mapping.description, last_left_cc);
                },
                StickType::Right => {
                    update_cc_if_changed(StickType::Right, mapping.cc_number, mapping.description, last_right_cc);
                }
            }
            break;
        }
    }
}

fn handle_controller_events(rx: Receiver<ControllerState>) {
    const DEADZONE: f32 = 0.75;
    let mut last_left_cc = get_current_cc(StickType::Left);
    let mut last_right_cc = get_current_cc(StickType::Right);

    while *RUNNING.lock().unwrap() {
        match rx.recv() {
            Ok(state) => {
                let [left_x, left_y] = state.sticks.left;
                let [right_x, right_y] = state.sticks.right;
                
                // CCナンバー変更チェック
                process_cc_mapping(&state, &mut last_left_cc, &mut last_right_cc);
                
                // スティックの処理
                process_stick(left_x, left_y, last_left_cc, StickType::Left, DEADZONE);
                process_stick(right_x, right_y, last_right_cc, StickType::Right, DEADZONE);
            }
            Err(_) => break,
        }
    }
}
