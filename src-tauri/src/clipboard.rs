use arboard::Clipboard;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{push_data_to_send_queue, APP_STATE};

#[derive(Clone, Serialize, Deserialize, Debug, specta::Type)]
pub enum ClipboardType {
    Text,
    Image,
    File,
}

#[derive(Clone, Serialize, Deserialize, Debug, specta::Type)]
pub struct ClipboardData {
    pub uuid: String,
    pub data_type: ClipboardType,
    pub data: String,
    pub datetime: String
}

pub async fn update_clipboard(data: Vec<u8>) {
    let mut clipboard = Clipboard::new().unwrap();
    let text = std::str::from_utf8(&data).unwrap();
    clipboard.set_text(text).unwrap();
    {
        let mut state = APP_STATE.lock().await;
        state.last_clipboard = text.to_string();
    }
}

pub async fn add_clipboard_data(data: Vec<u8>, timestamp: Option<DateTime<Utc>>) {
    let timestamp = match timestamp {
        Some(ts) => ts,
        None => Utc::now()
    };
    let text = std::str::from_utf8(&data).unwrap();
    {
        let mut state = APP_STATE.lock().await;
        state.clipboard_history.push(ClipboardData {
            data_type: ClipboardType::Text,
            data: text.to_string(),
            datetime: timestamp.to_rfc3339(),
            uuid: Uuid::new_v4().to_string()
        });
    }
}

pub async fn monitor_clipboard() {
    let mut clipboard = Clipboard::new().unwrap();
    let mut last_clipboard = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => String::new(),
    };
    loop {
        let current_clipboard = match clipboard.get_text() {
            Ok(text) => text,
            Err(_) => String::new(),
        };
        {
            last_clipboard = APP_STATE.lock().await.last_clipboard.clone();
        }
        if current_clipboard != last_clipboard {
            println!("Clipboard changed: {}", current_clipboard);
            {
                let data = current_clipboard.as_bytes().to_vec();
                push_data_to_send_queue(data.clone()).await;
                add_clipboard_data(data.clone(), None).await;
                update_clipboard(data.clone()).await;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

#[tauri::command]
#[specta::specta]
pub async fn copy_clipboard_from(uuid: String) {
    let data = {
        let state = APP_STATE.lock().await;
        state.clipboard_history.iter().find(|data| data.uuid == uuid).cloned()
    };
    if let Some(data) = data {
        let data = data.data.as_bytes().to_vec();

        update_clipboard(data.clone()).await;
        push_data_to_send_queue(data.clone()).await;
        let timestamp = Utc::now();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        add_clipboard_data(data.clone(), Some(timestamp)).await;
    } else {
        println!("Data with UUID {} not found", uuid);
    }
}