use std::fs;
use std::path::PathBuf;
use directories::BaseDirs;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::load().unwrap_or_default());
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    default_midi_port: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            default_midi_port: None,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            return Ok(Settings::default());
        }

        let contents = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings: {}", e))
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path()?;
        
        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        fs::write(&config_path, contents)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf, String> {
        Ok(BaseDirs::new()
            .ok_or("Could not determine config directory")?
            .config_dir()
            .join("xinputdj")
            .join("config.json"))
    }

    pub fn get_settings() -> Settings {
        SETTINGS.lock().unwrap().clone()
    }

    pub fn set_default_midi_port(port: Option<String>) -> Result<(), String> {
        let mut settings = SETTINGS.lock().unwrap();
        settings.default_midi_port = port;
        settings.save()
    }
}
