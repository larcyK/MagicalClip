import socket
import threading

def receive_messages(client_socket):
    while True:
        try:
            message = client_socket.recv(1024).decode()
            print(f"\nサーバーからのメッセージ: {message}")
        except:
            print("サーバーとの接続が切断されました。")
            break

def start_client():
    client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    client.connect(('SERVER_IP', 5000))
    print("サーバーに接続しました。")

    receive_thread = threading.Thread(target=receive_messages, args=(client,))
    receive_thread.start()

    while True:
        message = input("メッセージ: ")
        client.send(message.encode())

if __name__ == "__main__":
    start_client()