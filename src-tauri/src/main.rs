// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tokio::{
    io::{self, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream}, sync::Mutex,
};
use arboard::Clipboard;
use std::sync::{mpsc, Arc};
use lazy_static::lazy_static;

struct AppState {
    send_data_queue: Vec<Vec<u8>>,
}

lazy_static! {
    static ref APP_STATE: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState {
        send_data_queue: Vec::new(),
    }));
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn front_to_back(event: tauri::Event) {
    println!(
        "got front-to-back with payload {:?}",
        event.payload().unwrap()
    );
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
            last_clipboard = current_clipboard;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
async fn start_listening() -> Result<(), String> {
    println!("Starting server...");
    let listener = match TcpListener::bind("0.0.0.0:8080").await {
        Ok(listener) => listener,
        Err(err) => return Err(err.to_string()),
    };
    
    loop {
        match listener.accept().await {

            Ok((mut stream, addr)) => {
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
            let id = app.listen_global("front-to-back", front_to_back);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_listening,
            connect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
