// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use specta::collect_types;
use tauri::Manager;
use tauri_specta::ts;
use tokio::{
    io::{self, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream}, sync::Mutex,
};
use arboard::Clipboard;
use chrono::{DateTime, Utc};
use std::sync::{mpsc, Arc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, specta::Type)]
enum ClipboardType {
    Text,
    Image,
    File,
}

#[derive(Clone, Serialize, Deserialize, Debug, specta::Type)]
struct ClipboardData {
    data_type: ClipboardType,
    data: String,
    datetime: String
}

struct AppState {
    send_data_queue: Vec<Vec<u8>>,
    clipboard_history: Vec<ClipboardData>
}

lazy_static! {
    static ref APP_STATE: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState {
        send_data_queue: Vec::new(),
        clipboard_history: Vec::new()
    }));
}

async fn monitor_clipboard() {
    let mut clipboard = Clipboard::new().unwrap();
    let mut last_clipboard = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => String::new(),
    };
    loop {
        let current_clipboard = clipboard.get_text().unwrap();
        if current_clipboard != last_clipboard {
            println!("Clipboard changed: {}", current_clipboard);
            APP_STATE.lock().await.send_data_queue.push(current_clipboard.as_bytes().to_vec());
            APP_STATE.lock().await.clipboard_history.push(ClipboardData {
                data_type: ClipboardType::Text,
                data: current_clipboard.clone(),
                datetime: Utc::now().to_rfc3339()
            });
            last_clipboard = current_clipboard;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

fn update_clipboard(data: Vec<u8>) {
    let mut clipboard = Clipboard::new().unwrap();
    let text = std::str::from_utf8(&data).unwrap();
    clipboard.set_text(text).unwrap();
}

async fn process_tcp_stream(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    loop {
        let buf = &mut [0; 1 << 16];
        match stream.try_read(buf) {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed by server");
                    break;
                }
                let data = buf[..n].to_vec();
                println!("Received {} bytes", n);
                println!("Data: {:?}", std::str::from_utf8(&data));
                update_clipboard(data);
            }
            Err(e) => {
                println!("Failed to read from socket; err = {:?}", e);
            }
        }

        let mut state = APP_STATE.lock().await;
        while !state.send_data_queue.is_empty() {
            let data = state.send_data_queue.remove(0);
            match stream.write_all(&data).await {
                Ok(_) => {
                    println!("Sent {} bytes", data.len());
                }
                Err(e) => {
                    println!("Failed to write to socket; err = {:?}", e);
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[tauri::command]
#[specta::specta]
async fn connect(address: String, port: u16) -> Result<(), String> {
    println!("Connecting to server at {}:{}", address, port);
    let addr = format!("{}:{}", address, port);
    let stream = match TcpStream::connect(&addr).await {
        Ok(stream) => stream,
        Err(err) => return Err(err.to_string()),
    };
    tokio::spawn(async move {
        process_tcp_stream(stream).await;
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn start_listening() -> Result<(), String> {
    println!("Starting server...");
    let listener = match TcpListener::bind("0.0.0.0:8080").await {
        Ok(listener) => listener,
        Err(err) => return Err(err.to_string()),
    };
    
    loop {
        match listener.accept().await {

            Ok((stream, addr)) => {
                println!("Connected to server at {}", addr);
                tokio::spawn(async move {
                    process_tcp_stream(stream).await;
                });
            }
            Err(e) => {
                println!("Failed to connect to server; err = {:?}", e);
            }
        };

        // Wait for a second before attempting to reconnect
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("Attempting to reconnect...");
    }
}

#[tauri::command]
#[specta::specta]
async fn get_clipboard_history() -> Vec<ClipboardData> {
    let state = APP_STATE.lock().await;
    state.clipboard_history.clone()
}

#[test]
fn export_bindings() {
    ts::export(collect_types![
        start_listening,
        connect,
        get_clipboard_history
    ], 
    "../src/bindings.ts")
    .unwrap();
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            std::thread::spawn(move || loop {
                app_handle
                    .emit_all("back-to-front", "ping frontend".to_string())
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1))
            });
            std::thread::spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(monitor_clipboard());
            });
            std::thread::spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        match start_listening().await {
                            Ok(_) => println!("Listening for connections..."),
                            Err(err) => println!("Failed to start server: {}", err),
                        }
                    });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_listening,
            connect,
            get_clipboard_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
