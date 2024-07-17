use std::str::EncodeUtf16;

use arboard::ImageData;
use base64::{prelude::BASE64_STANDARD, Engine};
use image::ImageBuffer;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}};

use crate::{clipboard::{add_text_clipboard_data, update_text_clipboard}, config::save_app_data, APP_STATE};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum TcpDataType {
    Text,
    Blob,
    Image,
    Command
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TcpData {
    data_type: TcpDataType,
    data: String
}

async fn push_data_to_send_queue(data: TcpData) {
    let mut state = APP_STATE.lock().await;
    state.send_data_queue.push(data);
}

pub async fn push_blob_to_send_queue(data: Vec<u8>) {
    let data = TcpData {
        data_type: TcpDataType::Blob,
        data: BASE64_STANDARD.encode(data.as_slice())
    };
    push_data_to_send_queue(data).await;
}

pub async fn push_text_to_send_queue(text: String) {
    let data = TcpData {
        data_type: TcpDataType::Text,
        data: text
    };
    push_data_to_send_queue(data).await;
}

pub async fn push_image_to_send_queue(data: &ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let data = TcpData {
        data_type: TcpDataType::Image,
        data: BASE64_STANDARD.encode(data.as_raw())
    };
    push_data_to_send_queue(data).await;
}

fn find_json_end(data: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let mut depth = 0;
    let mut in_string = false;
    for (i, c) in data.chars().enumerate() {
        match c {
            '"' if !in_string => in_string = true,
            '"' if in_string => in_string = false,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
    }
    Err("No matching end found for JSON object".into())
}

fn split_json(data: &[u8]) -> Result<Vec<TcpData>, Box<dyn std::error::Error>> {
    let mut jsons = Vec::new();
    let mut start = 0;
    let data_str = std::str::from_utf8(data)?;

    while start < data_str.len() {
        match serde_json::from_str::<TcpData>(&data_str[start..]) {
            Ok(value) => {
                jsons.push(value);
                let end = find_json_end(&data_str[start..])?;
                start += end;
            }
            Err(_) => break,
        }
    }
    Ok(jsons)
}

pub async fn process_tcp_stream(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);
    loop {
        let buf = &mut [0; 1 << 16];
        match stream.try_read(buf) {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed by server");
                    break;
                }
                let json_data = buf[..n].to_vec();
                match serde_json::from_slice::<TcpData>(&json_data) {
                    Ok(data) => {
                        println!("Received data from server: {:?}", data.data_type);
                        match data.data_type {
                            TcpDataType::Text => {
                                add_text_clipboard_data(data.data.clone(), None).await;
                                update_text_clipboard(data.data.clone()).await;
                            }
                            TcpDataType::Blob => {
                            }
                            TcpDataType::Image => {
                            }
                            TcpDataType::Command => {
                            }
                        }
                    }
                    Err(_) => {
                        println!("Failed to parse JSON data: {:?}", std::str::from_utf8(&json_data));
                    }
                }
            }
            Err(e) => {
                println!("Failed to read from socket; err = {:?}", e);
            }
        }

        let mut data_to_send = Vec::new();
        {
            let mut state = APP_STATE.lock().await;
            while let Some(data) = state.send_data_queue.pop() {
                data_to_send.push(data);
            }
        }

        for data in data_to_send {
            let serialized_data = serde_json::to_string(&data).unwrap();
            match stream.write_all(serialized_data.as_bytes()).await {
                Ok(_) => println!("Send data to server: {}", serialized_data),
                Err(e) => {
                    println!("Failed to write to socket; err = {:?}", e);
                    break;
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[tauri::command]
#[specta::specta]
pub async fn tcp_connect(address: String, port: u16) -> Result<(), String> {
    println!("Connecting to server at {}:{}", address, port);
    let addr = format!("{}:{}", address, port);
    let stream = match TcpStream::connect(&addr).await {
        Ok(stream) => {
            {
                let mut state = APP_STATE.lock().await;
                state.server_address = Some(address);
                state.server_port = port;
            }
            save_app_data().await;
            stream
        }
        Err(err) => return Err(err.to_string()),
    };
    tokio::spawn(async move {
        process_tcp_stream(stream).await;
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn start_listening() -> Result<(), String> {
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