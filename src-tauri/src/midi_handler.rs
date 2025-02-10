use midir::{MidiOutput, MidiOutputConnection};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MIDI_CONNECTION: Mutex<Option<MidiOutputConnection>> = Mutex::new(None);
}

#[derive(serde::Serialize)]
pub struct MidiDevice {
    id: String,
    name: String,
}

#[tauri::command]
pub fn get_midi_ports() -> Result<Vec<MidiDevice>, String> {
    let midi_out = MidiOutput::new("XInputDJ").map_err(|e| e.to_string())?;
    let ports = midi_out.ports();
    let mut devices = Vec::new();

    for (_, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_out.port_name(port) {
            devices.push(MidiDevice {
                id: port.id(),
                name,
            });
        }
    }

    Ok(devices)
}

pub fn open_midi_port(port_id: String) -> Result<String, String> {
    let midi_out = MidiOutput::new("XInputDJ").map_err(|e| e.to_string())?;
    let port_id_clone = port_id.clone();
    
    if let Some(port) = midi_out.find_port_by_id(port_id) {
        let port_name = midi_out.port_name(&port).map_err(|e| e.to_string())?;
        let conn = midi_out.connect(&port, "XInputDJ-Output")
            .map_err(|e| e.to_string())?;
        
        let mut midi_conn = MIDI_CONNECTION.lock().unwrap();
        *midi_conn = Some(conn);

        Ok(format!("Connected to MIDI device: {}", port_name))
    } else {
        Err(format!("MIDI port {} not found", port_id_clone))
    }
}

pub fn close_midi_port() {
    let mut midi_conn = MIDI_CONNECTION.lock().unwrap();
    *midi_conn = None;
}

pub fn send_cc_change(channel: u8, controller: u8, value: u8) -> Result<(), String> {
    let mut midi_conn = MIDI_CONNECTION.lock().unwrap();
    if let Some(conn) = midi_conn.as_mut() {
        // MIDI CC message: Status byte (0xB0 | channel), controller number, value
        let message = [0xB0 | (channel & 0x0F), controller & 0x7F, value & 0x7F];
        conn.send(&message).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("No MIDI connection available".to_string())
    }
}
