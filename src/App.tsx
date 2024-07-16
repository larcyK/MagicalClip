import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog'
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { emit, listen } from '@tauri-apps/api/event';
import "./App.css";

function App() {
  const [address, setAddress] = createSignal("");
  const [port, setPort] = createSignal("");
  const [clipboard, setClipboard] = createSignal("");

  async function connect() {
    await invoke('connect', { address: address(), port: port() });
  }

  async function startListening() {
    await invoke('start_listening')
  }

  async function setClipboardText() {
    await writeText(clipboard());
  }

  function emitMessage() {
    emit('front-to-back', "hello from front")
  }

  function selectFile() {
    console.log('select file');
    open({ directory: false, multiple: false }).then((result) => {
      console.log(result);
    }).catch((error) => {
      console.error(error);
    });
  }

  let unlisten: any;
  async function f() {
    unlisten = await listen('back-to-front', event => {
      console.log(`back-to-front ${event.payload} ${new Date()}`)
    });
  }
  f();

  return (
    <div class="container">
      <h1>Welcome to Tauri!</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={logo} class="logo solid" alt="Solid logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      <div class="column">
      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          connect();
        }}
      >
        <input
          id="address-input"
          onChange={(e) => setAddress(e.currentTarget.value)}
          placeholder="Enter a server address..."
        />

        <input
          id="port-input"
          onChange={(e) => setPort(e.currentTarget.value)}
          placeholder="Enter a server port..."
        />

        <button type="submit">Connect</button>
      </form>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          setClipboardText();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setClipboard(e.currentTarget.value)}
          placeholder="Enter text to set clipboard..."
        />
        <button type="submit">Set Clipboard</button>
      </form>

      <button onClick={selectFile}>Click to open dialog</button>
      <button onClick={emitMessage}>Click to emit message</button>
      <button onClick={startListening}>Click to start listening</button>
      </div>
    </div>
  );
}

export default App;
