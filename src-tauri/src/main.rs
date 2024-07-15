#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Manager;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use arboard::Clipboard;

#[derive(Clone)]
struct SharedState {
  app_state: Arc<AppState>,
}

#[derive(Clone)]
struct AppState {
  peer_socket: Arc<Mutex<Option<TcpStream>>>,
  clipboard: Arc<Mutex<Clipboard>>,
}

#[tauri::command]
async fn connect_to_peer(state: tauri::State<'_, AppState>, ip: String, port: u16) -> Result<(), String> {
  let socket = TcpStream::connect(format!("{}:{}", ip, port)).await.map_err(|e| e.to_string())?;
  *state.peer_socket.lock().await = Some(socket);
  Ok(())
}

#[tauri::command]
async fn send_message(state: tauri::State<'_, AppState>, message: String) -> Result<(), String> {
  if let Some(socket) = &mut *state.peer_socket.lock().await {
    println!("Sending message: {}", message);
    socket.write_all(message.as_bytes()).await.map_err(|e| e.to_string())?;
  } else {
    return Err("Not connected to a peer".into());
  }
  Ok(())
}

#[tauri::command]
async fn send_clipboard(state: tauri::State<'_, AppState>) -> Result<(), String> {
  let content = state.clipboard.lock().await.get_text().map_err(|e| e.to_string())?;
  if let Some(socket) = &mut *state.peer_socket.lock().await {
    socket.write_all(format!("CLIPBOARD:{}", content).as_bytes()).await.map_err(|e| e.to_string())?;
  } else {
    return Err("Not connected to a peer".into());
  }
  Ok(())
}

#[tauri::command]
async fn start_server(app_handle: tauri::AppHandle, port: u16) -> Result<(), String> {

  let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.map_err(|e| e.to_string())?;
  let app_handle = Arc::new(app_handle);
  let state: Arc<AppState> = Arc::new(app_handle.state::<AppState>().inner().clone());

  let shared_state = SharedState {
      app_state: state.clone(),
  };

  tokio::spawn(async move {
    loop {
      if let Ok((socket, _)) = listener.accept().await {
        *state.peer_socket.lock().await = Some(socket);
        let app_handle = app_handle.clone();
        let peer_socket = Arc::clone(&state.peer_socket);
        let clipboard = Arc::clone(&state.clipboard);

        tokio::spawn(async move {
          let mut buffer = [0; 1024];
          loop {
            if let Some(socket) = &mut *peer_socket.lock().await {
              match socket.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                  let message = String::from_utf8_lossy(&buffer[..n]).to_string();
                  if message.starts_with("CLIPBOARD:") {
                    let content = message[10..].to_string();
                    clipboard.lock().await.set_text(content).unwrap();
                    app_handle.emit_all("clipboard_received", ()).unwrap();
                  } else {
                    app_handle.emit_all("message_received", message).unwrap();
                  }
                },
                Err(_) => break,
              }
            } else {
              break;
            }
          }
          *peer_socket.lock().await = None;
        });
      }
    }
  });

  Ok(())
}

fn main() {
  tauri::Builder::default()
    .manage(AppState {
      peer_socket: Arc::new(Mutex::new(None)),
      clipboard: Arc::new(Mutex::new(Clipboard::new().unwrap())),
    })
    .invoke_handler(tauri::generate_handler![connect_to_peer, send_message, send_clipboard, start_server])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}