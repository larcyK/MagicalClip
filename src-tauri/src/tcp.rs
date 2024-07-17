use tokio::{io::{AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}};

use crate::{clipboard::{add_clipboard_data, update_clipboard}, config::save_app_data, APP_STATE};

pub async fn push_data_to_send_queue(data: Vec<u8>) {
    let mut state = APP_STATE.lock().await;
    state.send_data_queue.push(data);
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
                let data = buf[..n].to_vec();
                println!("Received {} bytes", n);
                println!("Data: {:?}", std::str::from_utf8(&data));
                {
                    add_clipboard_data(data.clone(), None).await;
                    update_clipboard(String::from_utf8(data.clone()).unwrap().as_bytes().to_vec()).await;
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
            match stream.write_all(&data).await {
                Ok(_) => println!("Sent {} bytes", data.len()),
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