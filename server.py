import socket
import threading

def handle_client(conn, addr):
    print(f"新しい接続: {addr}")
    while True:
        try:
            message = conn.recv(1024).decode()
            if not message:
                break
            print(f"クライアントからのメッセージ: {message}")
            response = input("返信: ")
            conn.send(response.encode())
        except:
            break
    print(f"接続が終了しました: {addr}")
    conn.close()

def start_server():
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind(('0.0.0.0', 5000))
    server.listen(1)
    print("サーバーがポート5000でリッスン中...")

    while True:
        conn, addr = server.accept()
        client_thread = threading.Thread(target=handle_client, args=(conn, addr))
        client_thread.start()

if __name__ == "__main__":
    start_server()