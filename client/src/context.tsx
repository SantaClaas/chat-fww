import {
  JSX,
  Signal,
  createContext,
  createEffect,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { ChatMessage } from "./routes/Chat";

// Equivalent to the server-side Rust enum type
// #[derive(Serialize)]
// #[serde(tag = "type")]
// enum ClientMessage {
//     ChatMessage{ message: Arc<ChatMessage>},
//     AddUser { name: Arc<str>},
//     RemoveUser{name: Arc<str>},
//     SynchronizeMessage { message: Arc<ChatMessage> },
// }
type Message =
  | { type: "ChatMessage"; message: ChatMessage }
  | { type: "AddUser"; name: string }
  | { type: "RemoveUser"; name: string }
  | { type: "SynchronizeMessage"; message: ChatMessage };

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

/**
 *
 * @param message
 * @param contact Contact needs to be set separately and is not provided by the message
 * because the message could be from the current user but a different client for syncing
 */
function addChatMessage(message: ChatMessage, contact: string) {
  let signal = messagesByUser.get(contact);

  if (signal === undefined) {
    signal = createSignal<ChatMessage[]>([]);
    messagesByUser.set(message.sender, signal);
  }

  const [, setMessages] = signal;

  setMessages((messages) => [message, ...messages]);
}

// Some things to ensure security in production
const isSecureRequired =
  window.location.protocol === "https:" ||
  import.meta.env.MODE !== "development";

export const backendUrl = new URL(
  (isSecureRequired ? "https://" : "http://") +
    (import.meta.env.VITE_DEV_BACKEND_HOST ?? window.location.host)
);
console.debug("Backend at", backendUrl);
const socketUrl = new URL(backendUrl.href);
socketUrl.protocol = isSecureRequired ? "wss:" : "ws:";

async function fetchUsers() {
  const response = await fetch(backendUrl + "users");
  const data = await response.json();
  return data;
}

const [users, { mutate }] = createResource<string[]>(fetchUsers);

function addUser(name: string) {
  mutate((previous) => {
    if (previous === undefined) return [name];
    const index = previous.indexOf(name);
    if (index === -1) return [...previous, name];
    return previous;
  });
}

function removeUser(name: string) {
  mutate((previous) => previous?.filter((user) => user !== name));
}

async function handleMessage(event: MessageEvent) {
  if (typeof event.data !== "string")
    throw new Error("Message is not a string");

  console.debug("Received message", event.data);
  // For now we just pray it's the right type 🙂
  const message = JSON.parse(event.data) as Message;

  switch (message.type) {
    case "ChatMessage":
      // If we get a message from a different user, we need to use the sender to find the chat
      addChatMessage(message.message, message.message.sender);
      break;
    case "AddUser":
      addUser(message.name);
      break;
    case "RemoveUser":
      removeUser(message.name);
      break;
    case "SynchronizeMessage":
      // If we get a message from us but a different client, we need to use the find the intended recipient
      // of the message to fin the chat partner
      addChatMessage(message.message, message.message.recipient);
      break;
  }
}

const state = { socket, name, setName, messagesByUser, users };
const Context = createContext(state);

// Close socket if name goes to null. Meaning the user signs out
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
  const newSocket = new WebSocket(`${socketUrl.href}messages/${id}`);
  newSocket.addEventListener("message", handleMessage);
  setSocket(newSocket);
  return newSocket;
}, socket());

// Custom hooks and component to simplify usage
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
