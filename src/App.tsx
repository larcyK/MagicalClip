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

function areClipboardDataArraysEqual(arr1: commands.ClipboardData[], arr2: commands.ClipboardData[]): boolean {
  if (arr1.length !== arr2.length) return false;

  const map1 = new Map<string, commands.ClipboardData>();
  const map2 = new Map<string, commands.ClipboardData>();

  for (const item of arr1) {
    map1.set(item.uuid, item);
  }

  for (const item of arr2) {
    map2.set(item.uuid, item);
  }

  if (map1.size !== map2.size) return false;

  for (const [uuid, data1] of map1.entries()) {
    const data2 = map2.get(uuid);
    if (!data2) return false;

    if (data1.data_type !== data2.data_type || data1.data !== data2.data || data1.datetime !== data2.datetime) {
      return false;
    }
  }

  return true;
}

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

  async function saveClipboardHistory() {
    await commands.saveAppData();
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
      const currentHistory = clipboardHistory();
      if (!areClipboardDataArraysEqual(history, currentHistory)) {
        setClipboardHistory(history);
      }
    }, 100);
  });

  return (
    <div class="container">
      <h1>Magical Clip</h1>

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

            <button type="submit" class="common-button">Connect</button>
          </HStack>
        </form>

        <button onClick={saveClipboardHistory} class="common-button">Save</button>

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
