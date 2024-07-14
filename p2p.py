import socket
import threading
import tkinter as tk
from tkinter import scrolledtext

class P2PChat:
    def __init__(self, host, port):
        self.host = host
        self.port = port
        self.peer_address = None
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind((self.host, self.port))
        self.socket.listen(1)

        self.gui = tk.Tk()
        self.gui.title("P2P Chat")
        self.chat_area = scrolledtext.ScrolledText(self.gui, state='disabled')
        self.chat_area.pack(padx=20, pady=5)
        self.msg_entry = tk.Entry(self.gui)
        self.msg_entry.pack(padx=20, pady=5, fill=tk.X)
        self.send_button = tk.Button(self.gui, text="Send", command=self.send_message)
        self.send_button.pack(pady=5)

        self.connect_frame = tk.Frame(self.gui)
        self.connect_frame.pack(pady=10)
        self.peer_ip_entry = tk.Entry(self.connect_frame)
        self.peer_ip_entry.pack(side=tk.LEFT, padx=5)
        self.peer_port_entry = tk.Entry(self.connect_frame, width=6)
        self.peer_port_entry.pack(side=tk.LEFT, padx=5)
        self.connect_button = tk.Button(self.connect_frame, text="Connect", command=self.connect_to_peer)
        self.connect_button.pack(side=tk.LEFT, padx=5)

        threading.Thread(target=self.accept_connections, daemon=True).start()

    def accept_connections(self):
        while True:
            conn, addr = self.socket.accept()
            self.peer_address = addr
            threading.Thread(target=self.handle_peer, args=(conn,), daemon=True).start()

    def handle_peer(self, conn):
        while True:
            try:
                data = conn.recv(1024).decode()
                if not data:
                    break
                self.display_message(f"Peer: {data}")
            except:
                break
        conn.close()

    def connect_to_peer(self):
        peer_ip = self.peer_ip_entry.get()
        peer_port = int(self.peer_port_entry.get())
        self.peer_address = (peer_ip, peer_port)
        self.display_message(f"Connected to peer at {peer_ip}:{peer_port}")

    def send_message(self):
        message = self.msg_entry.get()
        if self.peer_address:
            try:
                with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                    s.connect(self.peer_address)
                    s.sendall(message.encode())
                self.display_message(f"You: {message}")
                self.msg_entry.delete(0, tk.END)
            except:
                self.display_message("Failed to send message")
        else:
            self.display_message("Not connected to a peer")

    def display_message(self, message):
        self.chat_area.config(state='normal')
        self.chat_area.insert(tk.END, message + '\n')
        self.chat_area.config(state='disabled')
        self.chat_area.see(tk.END)

    def run(self):
        self.gui.mainloop()

if __name__ == "__main__":
    # ローカルIPアドレスとポート番号を指定
    local_ip = "0.0.0.0"  # すべてのインターフェースでリッスン
    local_port = 5000  # 任意のポート番号を指定
    
    chat = P2PChat(local_ip, local_port)
    chat.run()