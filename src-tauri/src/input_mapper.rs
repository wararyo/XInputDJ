use crate::midi_handler::{send_cc_change, send_note_on, send_note_off};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::f32::consts::PI;
use crate::xinput_handler::{ControllerState, ButtonState};

#[derive(Debug, Clone, Copy, PartialEq)]
enum DeckType {
    Left,
    Right,
    Common,
}

impl DeckType {
    fn midi_channel(&self) -> u8 {
        match self {
            DeckType::Left => 0,
            DeckType::Right => 1,
            DeckType::Common => 15,
        }
    }
}

#[derive(Debug, Clone, Copy)]
// スティックからCC値の変換方法
enum Behavior {
    CCAbsolute,    // 通常の角度→CC値の変換
    CCRelative,    // 角度の差分→CC値の変換
    Note,          // ノートオン/オフの送信
}

// CCマッピング用の構造体
struct CCMapping {
    button_getter: fn(&ButtonState) -> bool,
    cc_number: Option<u8>,
    note_number: Option<u8>,
    description: &'static str,
    deck: DeckType,
    behavior: Behavior,
}

lazy_static::lazy_static! {
    static ref RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    // 右デッキと左デッキそれぞれにアサインされているCC値
    static ref CURRENT_CC: Arc<Mutex<[(DeckType, u8); 2]>> = Arc::new(Mutex::new([
        (DeckType::Left, 28),
        (DeckType::Right, 28),
    ]));
    // スティックの最後の位置
    static ref LAST_STICK_POS: Arc<Mutex<[(f32, f32); 2]>> = Arc::new(Mutex::new([
        (0.0, 0.0), // Left stick (x, y)
        (0.0, 0.0), // Right stick (x, y)
    ]));
    // 最後にスティックが倒されていたかどうか
    static ref STICK_NOTE_STATE: Arc<Mutex<[bool; 2]>> = Arc::new(Mutex::new([
        false, // Left stick note state (true = note on)
        false, // Right stick note state (true = note on)
    ]));
    // ボタンの最後の状態
    static ref LAST_BUTTON_STATE: Arc<Mutex<Option<ButtonState>>> = Arc::new(Mutex::new(None));

    // レイヤーA（通常時）のCCマッピング
    static ref CC_MAPPINGS_A: Vec<CCMapping> = vec![
        // 左デッキのマッピング
        CCMapping { button_getter: |b| b.down, cc_number: Some(25), note_number: None, description: "Down", deck: DeckType::Left, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.left, cc_number: Some(26), note_number: None, description: "Left", deck: DeckType::Left, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.up, cc_number: Some(24), note_number: None, description: "Up", deck: DeckType::Left, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.right, cc_number: Some(23), note_number: None, description: "Right", deck: DeckType::Left, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.l, cc_number: Some(28), note_number: None, description: "L", deck: DeckType::Left, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.lt, cc_number: Some(9), note_number: None, description: "LT", deck: DeckType::Left, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.l_stick, cc_number: Some(6), note_number: Some(6), description: "L stick", deck: DeckType::Left, behavior: Behavior::CCRelative },
        
        // 右デッキのマッピング
        CCMapping { button_getter: |b| b.south, cc_number: Some(25), note_number: None, description: "South", deck: DeckType::Right, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.east, cc_number: Some(26), note_number: None, description: "East", deck: DeckType::Right, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.north, cc_number: Some(24), note_number: None, description: "North", deck: DeckType::Right, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.west, cc_number: Some(23), note_number: None, description: "West", deck: DeckType::Right, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.r, cc_number: Some(28), note_number: None, description: "R", deck: DeckType::Right, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.rt, cc_number: Some(9), note_number: None, description: "RT", deck: DeckType::Right, behavior: Behavior::CCAbsolute },
        CCMapping { button_getter: |b| b.r_stick, cc_number: Some(6), note_number: Some(6), description: "R stick", deck: DeckType::Right, behavior: Behavior::CCRelative },
    ];

    // レイヤーB（スタート/セレクトボタン押下時）のCCマッピング
    static ref CC_MAPPINGS_B: Vec<CCMapping> = vec![
        // 左デッキのマッピング
        CCMapping { button_getter: |b| b.down, cc_number: None, note_number: Some(0), description: "Down (Note 0)", deck: DeckType::Left, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.left, cc_number: None, note_number: Some(1), description: "Left (Note 1)", deck: DeckType::Left, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.up, cc_number: None, note_number: Some(27), description: "Up (Note 27)", deck: DeckType::Left, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.right, cc_number: None, note_number: Some(2), description: "Right (Note 2)", deck: DeckType::Left, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.l, cc_number: None, note_number: Some(20), description: "L (Note 20)", deck: DeckType::Left, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.lt, cc_number: None, note_number: Some(21), description: "LT (Note 21)", deck: DeckType::Left, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.l_stick, cc_number: None, note_number: Some(7), description: "L stick (Note 7)", deck: DeckType::Common, behavior: Behavior::Note },
        
        // 右デッキのマッピング
        CCMapping { button_getter: |b| b.south, cc_number: None, note_number: Some(0), description: "A (Note 0)", deck: DeckType::Right, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.east, cc_number: None, note_number: Some(2), description: "B (Note 2)", deck: DeckType::Right, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.north, cc_number: None, note_number: Some(27), description: "Y (Note 27)", deck: DeckType::Right, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.west, cc_number: None, note_number: Some(1), description: "X (Note 1)", deck: DeckType::Right, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.r, cc_number: None, note_number: Some(20), description: "R (Note 20)", deck: DeckType::Right, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.rt, cc_number: None, note_number: Some(21), description: "RT (Note 21)", deck: DeckType::Right, behavior: Behavior::Note },
        CCMapping { button_getter: |b| b.r_stick, cc_number: None, note_number: Some(7), description: "R stick (Note 7)", deck: DeckType::Common, behavior: Behavior::Note },
    ];
}

fn get_active_mappings(state: &ControllerState) -> &'static Vec<CCMapping> {
    if state.buttons.start || state.buttons.select {
        &CC_MAPPINGS_B
    } else {
        &CC_MAPPINGS_A
    }
}

fn get_current_cc(deck: DeckType) -> u8 {
    let current_cc = CURRENT_CC.lock().unwrap();
    match deck {
        DeckType::Left => current_cc[0].1,
        DeckType::Right => current_cc[1].1,
        DeckType::Common => unreachable!("DeckType common doesn't have a CC number"),
    }
}

fn set_current_cc(deck: DeckType, cc: u8) {
    let mut current_cc = CURRENT_CC.lock().unwrap();
    match deck {
        DeckType::Left => current_cc[0].1 = cc,
        DeckType::Right => current_cc[1].1 = cc,
        DeckType::Common => unreachable!("DeckType common doesn't have a CC number"),
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
    println!("Left deck: CC#{}", get_current_cc(DeckType::Left));
    println!("Right deck: CC#{}", get_current_cc(DeckType::Right));
    println!("\nAvailable CC mappings (Layer A):");
    println!("Left deck:");
    for mapping in CC_MAPPINGS_A.iter().filter(|m| matches!(m.deck, DeckType::Left)) {
        if let Some(cc) = mapping.cc_number {
            println!("  {} button: CC#{}", mapping.description, cc);
        }
    }
    println!("Right deck:");
    for mapping in CC_MAPPINGS_A.iter().filter(|m| matches!(m.deck, DeckType::Right)) {
        if let Some(cc) = mapping.cc_number {
            println!("  {} button: CC#{}", mapping.description, cc);
        }
    }
    println!("\nAvailable CC mappings (Layer B - Start/Select button):");
    println!("Left deck:");
    for mapping in CC_MAPPINGS_B.iter().filter(|m| matches!(m.deck, DeckType::Left)) {
        if let Some(note) = mapping.note_number {
            println!("  {} button: Note#{}", mapping.description, note);
        }
    }
    println!("Right deck:");
    for mapping in CC_MAPPINGS_B.iter().filter(|m| matches!(m.deck, DeckType::Right)) {
        if let Some(note) = mapping.note_number {
            println!("  {} button: Note#{}", mapping.description, note);
        }
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

fn calculate_midi_cc_value_absolute(x: f32, y: f32, deadzone: f32) -> Option<u8> {
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

fn calculate_midi_cc_value_relative(x: f32, y: f32, deck: DeckType, deadzone: f32, steps: f32) -> Option<u8> {
    let distance = (x * x + y * y).sqrt();
    let angle = f32::atan2(x, y);
    let mut last_stick_pos = LAST_STICK_POS.lock().unwrap();
    let stick_idx = match deck {
        DeckType::Left => 0,
        DeckType::Right => 1,
        DeckType::Common => unreachable!("DeckType common doesn't have a stick"),
    };

    // デッドゾーン内の場合は現在の位置を保存して終了
    if distance < deadzone {
        last_stick_pos[stick_idx] = (x, y);
        return None;
    }
    
    let last_angle = f32::atan2(last_stick_pos[stick_idx].0, last_stick_pos[stick_idx].1);
    
    // 角度の差分を計算（-π から π の範囲）
    let mut diff = angle - last_angle;
    
    // 境界をまたぐ場合の補正
    if diff > PI {
        diff -= 2.0 * PI;
    } else if diff <= -PI {
        diff += 2.0 * PI;
    }
    
    // 一周をstepsステップとして正規化
    let normalized = diff / (PI * 2.0) * steps;
    let mut value = normalized.trunc().min(127.0).max(-127.0) as i32;
    if value < 0 {
        value += 128;
    }
    if value == 0 {
        return None;
    } else {
        // 現在の角度を保存
        last_stick_pos[stick_idx] = (x, y);
        return Some(value as u8);
    }
}

fn update_cc_if_changed(deck: DeckType, new_control_number: u8, description: &str, last_control_number: &mut u8) -> bool {
    if new_control_number != *last_control_number {
        *last_control_number = new_control_number;
        set_current_cc(deck, new_control_number);
        println!("{} deck control number changed to: {} ({})", 
            match deck {
                DeckType::Left => "Left",
                DeckType::Right => "Right",
                DeckType::Common => unreachable!("DeckType common doesn't have a CC number"),
            },
            new_control_number,
            description
        );
        true
    } else {
        false
    }
}

fn process_stick(x: f32, y: f32, control_number: u8, deck: DeckType, deadzone_cc: f32, deadzone_on: f32, deadzone_off: f32) {
    let distance = (x * x + y * y).sqrt();
    let stick_idx = match deck {
        DeckType::Left => 0,
        DeckType::Right => 1,
        DeckType::Common => unreachable!("DeckType common doesn't have a stick"),
    };

    // 現在のCCに対応するマッピングを取得
    let mapping = CC_MAPPINGS_A.iter()
        .find(|m| m.cc_number.map_or(false, |cc| cc == control_number) && m.deck == deck);

    if let Some(mapping) = mapping {
        // ノート処理
        if let Some(note_number) = mapping.note_number {
            let mut note_state = STICK_NOTE_STATE.lock().unwrap();
            let is_pressed = if note_state[stick_idx] {
                distance >= deadzone_off
            } else {
                distance > deadzone_on
            };

            match (is_pressed, note_state[stick_idx]) {
                (true, false) => {
                    // スティックが倒された
                    note_state[stick_idx] = true;
                    if let Err(e) = send_note_on(deck.midi_channel(), note_number, 127) {
                        eprintln!("Failed to send MIDI Note On ({} Deck): {:?}",
                            match deck {
                                DeckType::Left => "Left",
                                DeckType::Right => "Right",
                                DeckType::Common => "Common",
                            },
                            e
                        );
                    }
                },
                (false, true) => {
                    // スティックが元に戻った
                    note_state[stick_idx] = false;
                    if let Err(e) = send_note_off(deck.midi_channel(), note_number) {
                        eprintln!("Failed to send MIDI Note Off ({} Deck): {:?}",
                            match deck {
                                DeckType::Left => "Left",
                                DeckType::Right => "Right",
                                DeckType::Common => "Common",
                            },
                            e
                        );
                    }
                },
                _ => (),
            }
        }

        // CC処理
        let midi_value = match mapping.behavior {
            Behavior::CCAbsolute => calculate_midi_cc_value_absolute(x, y, deadzone_cc),
            Behavior::CCRelative => calculate_midi_cc_value_relative(x, y, deck, deadzone_cc, 360.0),
            _ => None,
        };

        if let Some(value) = midi_value {
            if let Err(e) = send_cc_change(deck.midi_channel(), control_number, value) {
                eprintln!("Failed to send MIDI CC ({} Deck): {:?}",
                    match deck {
                        DeckType::Left => "Left",
                        DeckType::Right => "Right",
                        DeckType::Common => "Common",
                    },
                    e
                );
            }
        }
    }
}

fn process_button(state: &ControllerState, last_left_cc: &mut u8, last_right_cc: &mut u8) {
    let active_mappings = get_active_mappings(state);
    let mut last_button_state = LAST_BUTTON_STATE.lock().unwrap();
    
    for mapping in active_mappings.iter() {
        let current_pressed = (mapping.button_getter)(&state.buttons);
        let was_pressed = last_button_state.as_ref().map_or(false, |last_state| (mapping.button_getter)(last_state));
        
        match mapping.behavior {
            Behavior::Note => {
                if let Some(note_number) = mapping.note_number {
                    if current_pressed && !was_pressed {
                        // ボタンが押された瞬間
                        if let Err(e) = send_note_on(mapping.deck.midi_channel(), note_number, 127) {
                            eprintln!("Failed to send MIDI Note On ({} Deck): {:?}",
                                match mapping.deck {
                                    DeckType::Left => "Left",
                                    DeckType::Right => "Right",
                                    DeckType::Common => "Common",
                                },
                                e
                            );
                        }
                    } else if !current_pressed && was_pressed {
                        // ボタンが離された瞬間
                        if let Err(e) = send_note_off(mapping.deck.midi_channel(), note_number) {
                            eprintln!("Failed to send MIDI Note Off ({} Deck): {:?}",
                                match mapping.deck {
                                    DeckType::Left => "Left",
                                    DeckType::Right => "Right",
                                    DeckType::Common => "Common",
                                },
                                e
                            );
                        }
                    }
                }
            },
            _ => if current_pressed {
                // CCの場合は従来通りの処理
                if let Some(cc_number) = mapping.cc_number {
                    match mapping.deck {
                        DeckType::Left => {
                            update_cc_if_changed(DeckType::Left, cc_number, mapping.description, last_left_cc);
                        },
                        DeckType::Right => {
                            update_cc_if_changed(DeckType::Right, cc_number, mapping.description, last_right_cc);
                        },
                        DeckType::Common => (), // Commonの場合は何もしない
                    }
                }
            }
        }
    }

    // 現在の状態を保存
    *last_button_state = Some(state.buttons.clone());
}

fn handle_controller_events(rx: Receiver<ControllerState>) {
    const DEADZONE_CC: f32 = 0.75;   // CCおよびノートオン用のデッドゾーン
    const DEADZONE_OFF: f32 = 0.7;   // ノートオフ用のデッドゾーン
    let mut last_left_cc = get_current_cc(DeckType::Left);
    let mut last_right_cc = get_current_cc(DeckType::Right);

    while *RUNNING.lock().unwrap() {
        match rx.recv() {
            Ok(state) => {
                let [left_x, left_y] = state.sticks.left;
                let [right_x, right_y] = state.sticks.right;
                
                // ボタンの処理
                process_button(&state, &mut last_left_cc, &mut last_right_cc);
                
                // スティックの処理
                if state.buttons.start || state.buttons.select {
                    // レイヤーBではスティックの挙動はライブラリの曲選択で固定
                    let midi_value = (
                        calculate_midi_cc_value_relative(left_x, left_y, DeckType::Left, DEADZONE_CC, 12.0),
                        calculate_midi_cc_value_relative(right_x, right_y, DeckType::Right, DEADZONE_CC, 12.0)
                    );

                    if let Some(value) = midi_value.0 {
                        if let Err(e) = send_cc_change(DeckType::Common.midi_channel(), 0, value) {
                            eprintln!("Failed to send MIDI CC (Common Deck): {:?}", e);
                        }
                    }
                    if let Some(value) = midi_value.1 {
                        if let Err(e) = send_cc_change(DeckType::Common.midi_channel(), 0, value) {
                            eprintln!("Failed to send MIDI CC (Common Deck): {:?}", e);
                        }
                    }
                } else {
                    // レイヤーAでは現在設定されているCCに応じた挙動を行う
                    process_stick(left_x, left_y, last_left_cc, DeckType::Left, DEADZONE_CC, DEADZONE_CC, DEADZONE_OFF);
                    process_stick(right_x, right_y, last_right_cc, DeckType::Right, DEADZONE_CC, DEADZONE_CC, DEADZONE_OFF);
                }

            }
            Err(_) => break,
        }
    }
}
