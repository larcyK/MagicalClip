import { createSignal, For, onMount } from "solid-js";
import { open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event';
import "./App.css";
import * as commands from "./bindings";
import {
  HStack,
  VStack,
} from "@hope-ui/solid";
import ClipboardHistoryCard from "./ClipboardHistoryCard";

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

  async function copyClipboardFrom(uuid: string) {
    await commands.copyClipboardFrom(uuid);
  }

  async function deleteClipboardHistory(uuid: string) {
    await commands.deleteClipboardHistory(uuid);
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
      <h1>Magical Clip</h1>

      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      <VStack gap="20px">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            connect();
          }}
        >
          <HStack gap="10px">
            <input
              class="common-input"
              id="address-input"
              onChange={(e) => setAddress(e.currentTarget.value)}
              placeholder="Enter a server address..."
            />

            <input
              class="common-input"
              id="port-input"
              onChange={(e) => setPort(Number(e.currentTarget.value))}
              placeholder="Enter a server port..."
            />

            <button type="submit">Connect</button>
          </HStack>
        </form>

        <button onClick={selectFile} class="common-button">Click to open dialog</button>
        <button onClick={startListening} class="common-button">Click to start listening</button>

        <VStack gap="10px" width="95%">
          <For each={clipboardHistory()}>
            {item => (
              <ClipboardHistoryCard
                clipboardData={item}
                onCopy={() => {
                  console.log("copy");
                  copyClipboardFrom(item.uuid);
                }}
                onDelete={() => {
                  console.log("delete");
                  deleteClipboardHistory(item.uuid);
                }}
              />
            )}
          </For>
        </VStack>
      </VStack>
    </div>
  );
}

export default App;
