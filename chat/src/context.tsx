import {
  JSX,
  Signal,
  createContext,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
import { ChatMessage } from "./routes/Chat";

const [name, setName] = createSignal<string | null>(
  localStorage.getItem("name")
);

createEffect(() => {
  const value = name();
  if (value === null) {
    localStorage.removeItem("name");
    return;
  }

  localStorage.setItem("name", value);
});

const [socket, setSocket] = createSignal<WebSocket | undefined>();
// Not sure if using a map is better
const messagesByUser = new Map<string, Signal<ChatMessage[]>>();

async function handleMessage(event: MessageEvent) {
  if (typeof event.data !== "string")
    throw new Error("Message is not a string");

  console.debug("Received message", event.data);
  // For now we just pray it's right 🙂
  const message = JSON.parse(event.data) as ChatMessage;

  let signal = messagesByUser.get(message.sender);

  if (signal === undefined) {
    signal = createSignal<ChatMessage[]>([]);
    messagesByUser.set(message.sender, signal);
  }

  const [, setMessages] = signal;

  setMessages((messages) => [message, ...messages]);
}

const state = { socket, name, setName, messagesByUser };
const Context = createContext(state);

// Close socket if name goes to null. Meaningif the user signs out
createEffect<ReturnType<typeof name>>((previous) => {
  const value = name();
  if (!(value === null && previous !== null)) return value;

  const currentSocket = socket();
  if (currentSocket === undefined) return value;

  console.debug("Closing socket");
  currentSocket.close();
  setSocket(undefined);

  return value;
}, name());

// Use new socket if name changes
createEffect<WebSocket | undefined>((previous) => {
  const id = name();
  if (id === null) return previous;

  previous?.removeEventListener("message", handleMessage);
  previous?.close();

  console.debug("Opening socket");
  const newSocket = new WebSocket(`ws://localhost:3000/messages/${id}`);
  newSocket.addEventListener("message", handleMessage);
  setSocket(newSocket);
  return newSocket;
}, socket());

export function ContextProvider(properties: { children: JSX.Element }) {
  return (
    <Context.Provider value={state}>{properties.children}</Context.Provider>
  );
}

export function useAppContext(onmessage?: (event: MessageEvent) => any) {
  const { socket, ...rest } = useContext(Context);

  createEffect<WebSocket | undefined>((previous) => {
    const current = socket();
    if (onmessage === undefined || current === undefined) return current;

    previous?.removeEventListener("message", onmessage);
    current.addEventListener("message", onmessage);
    return current;
  }, socket());

  return { socket, ...rest };
}
