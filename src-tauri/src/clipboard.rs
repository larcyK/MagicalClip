use arboard::Clipboard;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::save_app_data, tcp::push_text_to_send_queue, APP_STATE};

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

pub async fn update_text_clipboard(text: String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(text.clone()).unwrap();
    {
        let mut state = APP_STATE.lock().await;
        state.last_clipboard = text.clone();
    }
    save_app_data().await;
}

pub async fn add_text_clipboard_data(text: String, timestamp: Option<DateTime<Utc>>) {
    let timestamp = match timestamp {
        Some(ts) => ts,
        None => Utc::now()
    };
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
                let text = current_clipboard;
                push_text_to_send_queue(text.clone()).await;
                add_text_clipboard_data(text.clone(), None).await;
                update_text_clipboard(text.clone()).await;
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
        let text = data.data;

        update_text_clipboard(text.clone()).await;
        push_text_to_send_queue(text.clone()).await;
        let timestamp = Utc::now();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        add_text_clipboard_data(text.clone(), Some(timestamp)).await;
    } else {
        println!("Data with UUID {} not found", uuid);
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_clipboard_history() -> Vec<ClipboardData> {
    let state = APP_STATE.lock().await;
    state.clipboard_history.clone()
}

#[tauri::command]
#[specta::specta]
pub async fn delete_clipboard_history(uuid: String) {
    let mut state = APP_STATE.lock().await;
    state.clipboard_history.retain(|data| data.uuid != uuid);
}