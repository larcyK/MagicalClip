// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod tcp;
mod config;

use clipboard::{ClipboardData};
use config::{get_app_data_path, load_app_data};
use specta::collect_types;
use tauri::Manager;
use tauri_specta::ts;
use tcp::{start_listening, tcp_connect};
use tokio::{
    sync::Mutex
};
use std::sync::Arc;
use lazy_static::lazy_static;

struct AppState {
    server_address: Option<String>,
    server_port: u16,
    app_data_path: String,
    last_clipboard: String,
    send_data_queue: Vec<Vec<u8>>,
    clipboard_history: Vec<ClipboardData>
}

lazy_static! {
    static ref APP_STATE: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState {
        server_address: None,
        server_port: 0,
        app_data_path: String::new(),
        last_clipboard: String::new(),
        send_data_queue: Vec::new(),
        clipboard_history: Vec::new()
    }));
}

#[test]
fn export_bindings() {
    ts::export(collect_types![
        tcp::start_listening,
        tcp::tcp_connect,
        clipboard::get_clipboard_history,
        clipboard::delete_clipboard_history,
        clipboard::copy_clipboard_from,
        config::save_app_data,
        config::delete_app_data
    ], 
    "../src/bindings.ts")
    .unwrap();
}

fn main() {
    tauri::Builder::default()
        .setup(move |app| {
            let app_handle = app.app_handle();
            {
                let app_handle = app_handle.clone();
                std::thread::spawn(move || loop {
                    app_handle
                        .clone()
                        .emit_all("back-to-front", "ping frontend".to_string())
                        .unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(1))
                });
            }
            tauri::async_runtime::spawn(async move {
                let app_handle = app_handle.clone();
                load_app_data(&app_handle.config()).await;

                let mut address: Option<String> = None;
                let mut port: u16 = 0;
                {
                    let state = APP_STATE.lock().await;
                    address = state.server_address.clone();
                    port = state.server_port;
                }
                if let Some(addr) = address {
                    tcp_connect(addr, port).await.unwrap();
                }

                std::thread::spawn(|| {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(clipboard::monitor_clipboard());
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
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tcp::start_listening,
            tcp::tcp_connect,
            clipboard::get_clipboard_history,
            clipboard::delete_clipboard_history,
            clipboard::copy_clipboard_from,
            config::save_app_data,
            config::delete_app_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
