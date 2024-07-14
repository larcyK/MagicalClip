import socket
import threading
import pyperclip
import time

def handle_client(conn):
    while True:
        try:
            data = conn.recv(1024).decode()
            if not data:
                break
            pyperclip.copy(data)
            print(f"Received and copied to clipboard: {data}")
        except:
            break
    conn.close()

def start_server():
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind(('0.0.0.0', 5000))
    server.listen(1)
    print("Server listening on port 5000")

    while True:
        conn, addr = server.accept()
        print(f"Connected by {addr}")
        client_thread = threading.Thread(target=handle_client, args=(conn,))
        client_thread.start()

def monitor_clipboard():
    last_value = pyperclip.paste()
    while True:
        current_value = pyperclip.paste()
        if current_value != last_value:
            last_value = current_value
            send_to_client(current_value)

def send_to_client(data):
    client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        client.connect(('CLIENT_IP', 5000))
        client.sendall(data.encode())
    except:
        print("Failed to send data to client")
    finally:
        client.close()

if __name__ == "__main__":
    server_thread = threading.Thread(target=start_server)
    server_thread.daemon = True
    server_thread.start()

    monitor_thread = threading.Thread(target=monitor_clipboard)
    monitor_thread.daemon = True
    monitor_thread.start()

    while True:
        time.sleep(1)