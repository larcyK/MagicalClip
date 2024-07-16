import { createSignal, For, onMount } from "solid-js";
import logo from "./assets/logo.svg";
import { open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event';
import "./App.css";
import * as commands from "./bindings";

function App() {
  const [address, setAddress] = createSignal("");
  const [port, setPort] = createSignal(5000);
  const [clipboardHistory, setClipboardHistory] = createSignal<commands.ClipboardData[]>([]);

  async function connect() {
    console.log('connect', address(), port());
    await commands.connect(address(), port());
  }

  async function startListening() {
    await commands.startListening();
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

  onMount(() => {
    setInterval(async () => {
      const history = await commands.getClipboardHistory();
      setClipboardHistory(history);
    }, 100);
  });

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
          onChange={(e) => setPort(Number(e.currentTarget.value))}
          placeholder="Enter a server port..."
        />

        <button type="submit">Connect</button>
      </form>

      <button onClick={selectFile}>Click to open dialog</button>
      <button onClick={startListening}>Click to start listening</button>

      <ul>
        <For each={clipboardHistory()}>
          {item => (
            <li>{item.data}</li>
          )}
        </For>
      </ul>
      </div>
    </div>
  );
}

export default App;
