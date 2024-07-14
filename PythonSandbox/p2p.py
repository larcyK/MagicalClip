import socket
import threading
import tkinter as tk
from tkinter import scrolledtext
import pyperclip
import time

class P2PChat:
    def __init__(self, host, port):
        self.host = host
        self.port = port
        self.peer_address = None
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.socket.bind((self.host, self.port))
        self.socket.listen(1)
        self.peer_socket = None
        self.last_clipboard = pyperclip.paste()

        self.gui = tk.Tk()
        self.gui.title(f"P2P Chat - {self.host}:{self.port}")
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

        self.clipboard_var = tk.IntVar()
        self.clipboard_check = tk.Checkbutton(self.gui, text="Enable Clipboard Sharing", variable=self.clipboard_var)
        self.clipboard_check.pack(pady=5)

        threading.Thread(target=self.accept_connections, daemon=True).start()
        threading.Thread(target=self.monitor_clipboard, daemon=True).start()

    def accept_connections(self):
        while True:
            conn, addr = self.socket.accept()
            if self.peer_socket:
                conn.close()
                continue
            self.peer_socket = conn
            self.peer_address = addr
            self.display_message(f"Connected with {addr[0]}:{addr[1]}")
            threading.Thread(target=self.handle_peer, args=(conn,), daemon=True).start()

    def handle_peer(self, conn):
        while True:
            try:
                data = conn.recv(1024).decode()
                if not data:
                    break
                if data.startswith("CLIPBOARD:"):
                    if self.clipboard_var.get():
                        pyperclip.copy(data[10:])
                        self.display_message(f"Received clipboard content: {data[10:]}")
                else:
                    self.display_message(f"Peer: {data}")
            except:
                break
        conn.close()
        self.peer_socket = None
        self.peer_address = None
        self.display_message("Disconnected from peer")

    def connect_to_peer(self):
        if self.peer_socket:
            self.display_message("Already connected to a peer")
            return
        peer_ip = self.peer_ip_entry.get()
        peer_port = int(self.peer_port_entry.get())
        try:
            self.peer_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.peer_socket.connect((peer_ip, peer_port))
            self.peer_address = (peer_ip, peer_port)
            self.display_message(f"Connected to peer at {peer_ip}:{peer_port}")
            threading.Thread(target=self.handle_peer, args=(self.peer_socket,), daemon=True).start()
        except Exception as e:
            self.display_message(f"Failed to connect: {str(e)}")
            self.peer_socket = None

    def send_message(self):
        message = self.msg_entry.get()
        if self.peer_socket:
            try:
                self.peer_socket.sendall(message.encode())
                self.display_message(f"You: {message}")
                self.msg_entry.delete(0, tk.END)
            except:
                self.display_message("Failed to send message")
                self.peer_socket = None
                self.peer_address = None
        else:
            self.display_message("Not connected to a peer")

    def send_clipboard(self, content):
        if self.peer_socket and self.clipboard_var.get():
            try:
                self.peer_socket.sendall(f"CLIPBOARD:{content}".encode())
                self.display_message(f"Clipboard content sent: {content}")
            except:
                self.display_message("Failed to send clipboard content")
                self.peer_socket = None
                self.peer_address = None

    def monitor_clipboard(self):
        while True:
            current_clipboard = pyperclip.paste()
            if current_clipboard != self.last_clipboard:
                self.last_clipboard = current_clipboard
                self.send_clipboard(current_clipboard)
            time.sleep(1)

    def display_message(self, message):
        self.chat_area.config(state='normal')
        self.chat_area.insert(tk.END, message + '\n')
        self.chat_area.config(state='disabled')
        self.chat_area.see(tk.END)

    def run(self):
        self.gui.mainloop()

if __name__ == "__main__":
    import socket
    local_ip = socket.gethostbyname(socket.gethostname())
    local_port = 5000
    
    chat = P2PChat(local_ip, local_port)
    chat.run()