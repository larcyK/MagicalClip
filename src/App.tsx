import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { open } from '@tauri-apps/api/dialog'
import { writeText, readText } from '@tauri-apps/api/clipboard';
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");
  const [clipboard, setClipboard] = createSignal("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  async function setClipboardText() {
    await writeText(clipboard());
  }

  function selectFile() {
    console.log('select file');
    open({ directory: false, multiple: false }).then((result) => {
      console.log(result);
    }).catch((error) => {
      console.error(error);
    });
  }

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

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
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

      <p>{greetMsg()}</p>
    </div>
  );
}

export default App;
