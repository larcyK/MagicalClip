use std::{fs::{self, create_dir_all, remove_dir}, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::api::path::{ app_config_dir };

use crate::{clipboard, APP_STATE};

#[derive(Clone, Serialize, Deserialize)]
pub enum ClipboardType {
    Text,
    Image,
    File,
}

impl ClipboardType {
    pub fn into_clipboard_type(&self) -> clipboard::ClipboardType {
        match self {
            ClipboardType::Text => clipboard::ClipboardType::Text,
            ClipboardType::Image => clipboard::ClipboardType::Image,
            ClipboardType::File => clipboard::ClipboardType::File,
        }
    }

    fn from_clipboard_type(clipboard_type: &clipboard::ClipboardType) -> Self {
        match clipboard_type {
            clipboard::ClipboardType::Text => ClipboardType::Text,
            clipboard::ClipboardType::Image => ClipboardType::Image,
            clipboard::ClipboardType::File => ClipboardType::File,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub uuid: String,
    pub data_type: ClipboardType,
    // pub data: Vec<u8>,
    pub data: String,
    pub datetime: String
}

#[derive(Deserialize, Serialize)]
pub struct AppData {
    pub history: Vec<ClipboardData>,
    pub server_address: Option<String>,
    pub server_port: u16
}

#[derive(Deserialize, Serialize)]
struct RawAppData {
    history: Vec<RawClipboardData>,
    server_address: Option<String>,
    server_port: u16
}

#[derive(Deserialize, Serialize)]
struct RawClipboardData {
    uuid: String,
    data_type: ClipboardType,
    data: String,
    datetime: String
}

impl RawAppData {
    pub fn convert_to_app_data(self) -> AppData {
        let history = self.history.into_iter().map(|data| {
            ClipboardData {
                uuid: data.uuid,
                data_type: data.data_type,
                datetime: data.datetime,
                // data: BASE64_STANDARD.decode(data.data).unwrap()
                data: data.data
            }
        }).collect();
        AppData { 
            history,
            server_address: self.server_address,
            server_port: self.server_port
        }
    }
}

fn get_app_file_path(tauri_config: &tauri::Config, file_name: &'static str) -> Option<PathBuf> {
    let path = app_config_dir(tauri_config)?.join(file_name);

    if path.exists() && !path.is_file() {
        remove_dir(&path).ok()?;
    }

    if !path.exists() {
        let dir = path.parent()?;

        create_dir_all(&dir).ok()?;
    }

    Some(path)
}

pub fn get_app_data_path(tauri_config: &tauri::Config) -> Option<PathBuf> {
    let path = get_app_file_path(tauri_config, "app_data.json");
    println!("App data path: {:?}", path);
    path
}

pub fn get_app_data(tauri_config: &tauri::Config) -> Option<AppData> {
    let path = get_app_data_path(tauri_config)?;

    let json_data = std::fs::read_to_string(path).ok()?;

    let raw_app_data: RawAppData = serde_json::from_str(&json_data).ok()?;
    Some(raw_app_data.convert_to_app_data())

    // let app_data: AppData = serde_json::from_str(&json_data).ok()?;
    // Some(app_data)
}

pub async fn load_app_data(config: &tauri::Config) where tauri::Config: Send {
    if let Some(app_data) = get_app_data(&config) {
        let mut state = APP_STATE.lock().await;
        state.clipboard_history = app_data.history.into_iter().map(|data| {
            clipboard::ClipboardData {
                uuid: data.uuid,
                data_type: data.data_type.into_clipboard_type(),
                datetime: data.datetime,
                data: data.data
            }
        }).collect();
        state.server_address = app_data.server_address;
        state.server_port = app_data.server_port;
    }
    {
        let mut state = APP_STATE.lock().await;
        let path = get_app_data_path(&config).unwrap().to_string_lossy().to_string();
        state.app_data_path = path;
    }
}

#[tauri::command]
#[specta::specta]
pub async fn save_app_data() {
    let state = APP_STATE.lock().await;
    let app_data = AppData {
        server_address: state.server_address.clone(),
        server_port: state.server_port,
        history: state.clipboard_history.iter().map(|data| {
            ClipboardData {
                uuid: data.uuid.clone(),
                data_type: ClipboardType::from_clipboard_type(&data.data_type),
                datetime: data.datetime.clone(),
                data: data.data.clone()
            }
        }).collect()
    };
    
    let path = PathBuf::from(&state.app_data_path);

    if !path.exists() {
        let dir = path.parent().unwrap();
        create_dir_all(&dir).ok();
    }

    println!("Save app data (port: {}): {:?}", state.server_port, path);

    let json_data = serde_json::to_string(&app_data).unwrap();
    std::fs::write(path, json_data).unwrap();
}

#[tauri::command]
#[specta::specta]
pub async fn delete_app_data() {
    let mut state = APP_STATE.lock().await;
    state.clipboard_history.clear();
    save_app_data().await;
}