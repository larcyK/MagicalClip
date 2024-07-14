import socket
import threading
import pyperclip

SERVER_IP = 'SERVER_IP'
def receive_from_server():
    client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    client.connect((SERVER_IP, 5000))
    
    while True:
        try:
            data = client.recv(1024).decode()
            if not data:
                break
            pyperclip.copy(data)
            print(f"Received and copied to clipboard: {data}")
        except:
            break
    client.close()

def monitor_clipboard():
    last_value = pyperclip.paste()
    while True:
        current_value = pyperclip.paste()
        if current_value != last_value:
            last_value = current_value
            send_to_server(current_value)

def send_to_server(data):
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        server.connect((SERVER_IP, 5000))
        server.sendall(data.encode())
    except:
        print("Failed to send data to server")
    finally:
        server.close()

if __name__ == "__main__":
    receive_thread = threading.Thread(target=receive_from_server)
    receive_thread.start()

    monitor_thread = threading.Thread(target=monitor_clipboard)
    monitor_thread.start()