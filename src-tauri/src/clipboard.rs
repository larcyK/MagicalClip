use std::borrow::Cow;

use arboard::{Clipboard, ImageData};
use chrono::{DateTime, Utc};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use tauri::api::file;
use uuid::Uuid;

use crate::{config::{save_app_data, save_image}, tcp::{push_image_to_send_queue, push_text_to_send_queue}, APP_STATE};

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
        state.last_clipboard = RawClipboardData::Text(text);
    }
    save_app_data().await;
}

pub async fn update_image_clipboard(img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let dynamic_image = DynamicImage::ImageRgba8(img_buffer.clone());
    let (width, height) = dynamic_image.dimensions();
    let image_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: Cow::Owned(dynamic_image.to_bytes()),
    };
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_image(image_data).unwrap();
    {
        let mut state = APP_STATE.lock().await;
        state.last_clipboard = RawClipboardData::Image(img_buffer);
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

pub async fn add_image_clipboard_data(file_name: String, timestamp: Option<DateTime<Utc>>) {
    let timestamp = match timestamp {
        Some(ts) => ts,
        None => Utc::now()
    };
    {
        let mut state = APP_STATE.lock().await;
        state.clipboard_history.push(ClipboardData {
            data_type: ClipboardType::Image,
            data: file_name.to_string(),
            datetime: timestamp.to_rfc3339(),
            uuid: Uuid::new_v4().to_string()
        });
    }
}

#[derive(Clone, Debug)]
pub enum RawClipboardData {
    Image(ImageBuffer<Rgba<u8>, Vec<u8>>),
    Text(String),
}

impl PartialEq<RawClipboardData> for RawClipboardData {
    fn eq(&self, other: &RawClipboardData) -> bool {
        match (self, other) {
            (RawClipboardData::Image(uuid1), RawClipboardData::Image(uuid2)) => uuid1 == uuid2,
            (RawClipboardData::Text(text1), RawClipboardData::Text(text2)) => text1 == text2,
            _ => false,
        }
    }
}

fn image_data_to_image_buffer(data: ImageData) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let width = data.width as u32;
    let height = data.height as u32;
    let bytes = data.bytes;
    let img_buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, bytes.to_vec()).unwrap();

    img_buffer
}

pub async fn monitor_clipboard() {
    let mut clipboard = Clipboard::new().unwrap();
    let mut last_clipboard = match clipboard.get_text() {
        Ok(text) => RawClipboardData::Text(text),
        Err(_) => RawClipboardData::Text(String::new()),
    };
    loop {
        let mut image_uuid = String::new();
        let current_clipboard = match clipboard.get_text() {
            Ok(text) => RawClipboardData::Text(text),
            Err(_) => match clipboard.get_image() {
                Ok(image) => {
                    let image_buffer = image_data_to_image_buffer(image);
                    println!("Image saved");
                    image_uuid = Uuid::new_v4().to_string();
                    RawClipboardData::Image(image_buffer)
                }
                Err(_) => RawClipboardData::Text(String::new()),
            },
        };
        {
            last_clipboard = APP_STATE.lock().await.last_clipboard.clone();
        }
        if current_clipboard != last_clipboard {
            match current_clipboard {
                RawClipboardData::Text(text) => {
                    push_text_to_send_queue(text.clone()).await;
                    add_text_clipboard_data(text.clone(), None).await;
                    update_text_clipboard(text.clone()).await;
                },
                RawClipboardData::Image(image) => {
                    let file_name = format!("{}.png", image_uuid);
                    let _ = save_image(&image, file_name.clone(), image::ImageFormat::Png).await;
                    push_image_to_send_queue(&image).await;
                    add_image_clipboard_data(file_name.clone(), None).await;
                    update_image_clipboard(image.clone()).await;
                }
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