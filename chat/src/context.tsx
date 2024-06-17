import {
  JSX,
  createContext,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";

const [name, setName] = createSignal<string | null>(
  localStorage.getItem("name")
);
const [socket, setSocket] = createSignal<WebSocket | undefined>();
const state = { socket, name, setName };
const Context = createContext(state);
createEffect<WebSocket | undefined>((previous) => {
  const id = name();
  if (id === null) return previous;

  previous?.close();

  const newSocket = new WebSocket(`ws://localhost:3000/messages/${id}`);
  setSocket(newSocket);
  return newSocket;
}, socket());

export function ContextProvider(properties: { children: JSX.Element }) {
  return (
    <Context.Provider value={state}>{properties.children}</Context.Provider>
  );
}

export function useName() {
  const { name, setName } = useContext(Context);

  return [name, setName] as const;
}

export function useWebSocket(onmessage?: (event: MessageEvent) => any) {
  const { socket } = useContext(Context);

  createEffect<WebSocket | undefined>((previous) => {
    const current = socket();
    if (onmessage === undefined || current === undefined) return current;

    previous?.removeEventListener("message", onmessage);
    current.addEventListener("message", onmessage);
    return current;
  }, socket());

  return socket;
}
