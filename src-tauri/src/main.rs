// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tokio::{
    io::{self, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

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

                loop {
                    stream.writable().await.map_err(|e| e.to_string())?;
                    if let Err(e) = stream.write_all(b"hello world\n").await {
                        println!("Failed to write to socket; err = {:?}", e);
                        break;
                    }

                    let buf = &mut [0; 4096];
                    match stream.try_read(buf) {
                        Ok(n) => {
                            if n == 0 {
                                println!("Connection closed by server");
                                break;
                            }
                            println!("Received {} bytes", n);
                            println!("{:?}", std::str::from_utf8(&buf[..n]));
                        }
                        Err(e) => {
                            println!("Failed to read from socket; err = {:?}", e);
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
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
            let id = app.listen_global("front-to-back", front_to_back);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_listening,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
