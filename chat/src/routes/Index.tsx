import { For, Match, Show, createResource, createSignal } from "solid-js";
import { useName, useWebSocket } from "../context";
import { Navigate } from "@solidjs/router";

async function fetchUsers() {
  const response = await fetch("http://localhost:3000/users");
  const data = await response.json();
  return data;
}
export default function Index() {
  const socket = useWebSocket();
  const [name] = useName();

  if (socket() === undefined || name() === undefined)
    return <Navigate href="/setup" />;

  // Available chats

  const [users] = createResource<string[]>(fetchUsers);

  return (
    <Show when={!users.loading} fallback={<p>Loading...</p>}>
      <Show
        when={users() && users()!.length > 0}
        fallback={<p>No one available to chat</p>}
      >
        <ul>
          <For each={users()}>
            {(user) => (
              <li>
                <a href={`/chats/${user}`}>{user}</a>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </Show>
  );
}
