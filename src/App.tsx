import { createSignal, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

function App() {
  const [messages, setMessages] = createSignal<string[]>([]);
  const [inputMessage, setInputMessage] = createSignal('');
  const [peerIp, setPeerIp] = createSignal('');
  const [peerPort, setPeerPort] = createSignal('');
  const [isClipboardSharing, setIsClipboardSharing] = createSignal(false);

  onMount(async () => {
    await invoke('start_server', { port: 5000 });
    
    await listen('message_received', (event) => {
      setMessages((prev) => [...prev, `Peer: ${event.payload}`]);
    });

    await listen('clipboard_received', () => {
      setMessages((prev) => [...prev, 'Received clipboard content']);
    });
  });

  const sendMessage = async () => {
    if (inputMessage()) {
      try {
        await invoke('send_message', { message: inputMessage() });
        setMessages((prev) => [...prev, `You: ${inputMessage()}`]);
        setInputMessage('');
      } catch (error) {
        setMessages((prev) => [...prev, `Error: ${error}`]);
      }
    }
  };

  const connectToPeer = async () => {
    if (peerIp() && peerPort()) {
      try {
        await invoke('connect_to_peer', { ip: peerIp(), port: parseInt(peerPort()) });
        setMessages((prev) => [...prev, `Connected to peer at ${peerIp()}:${peerPort()}`]);
      } catch (error) {
        setMessages((prev) => [...prev, `Connection error: ${error}`]);
      }
    }
  };

  const toggleClipboardSharing = async (event: Event & { target: HTMLInputElement }) => {
    setIsClipboardSharing(event.target.checked);
    if (event.target.checked) {
      startClipboardMonitoring();
    }
  };

  const startClipboardMonitoring = async () => {
    while (isClipboardSharing()) {
      try {
        await invoke('send_clipboard');
      } catch (error) {
        console.error('Failed to send clipboard:', error);
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  };

  return (
    <div>
      <div style={{ height: '300px', 'overflow-y': 'scroll', border: '1px solid #ccc', padding: '10px', 'margin-bottom': '10px' }}>
        {messages().map((message, index) => (
          <div>{message}</div>
        ))}
      </div>
      <input
        type="text"
        value={inputMessage()}
        onInput={(e) => setInputMessage(e.target.value)}
        placeholder="Enter message"
      />
      <button onClick={sendMessage}>Send</button>
      <br />
      <input
        type="text"
        value={peerIp()}
        onInput={(e) => setPeerIp(e.target.value)}
        placeholder="Peer IP"
      />
      <input
        type="number"
        value={peerPort()}
        onInput={(e) => setPeerPort(e.target.value)}
        placeholder="Peer Port"
      />
      <button onClick={connectToPeer}>Connect</button>
      <br />
      <label>
        <input
          type="checkbox"
          checked={isClipboardSharing()}
          onChange={toggleClipboardSharing}
        /> Enable Clipboard Sharing
      </label>
    </div>
  );
}

export default App;