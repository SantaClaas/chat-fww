import { For, Show, createEffect } from "solid-js";
import { useAppContext } from "../context";
import { Navigate, useNavigate } from "@solidjs/router";
import TopAppBar from "../components/TopAppBar";

export default function Index() {
  const { socket, name, setName, users } = useAppContext();
  const navigate = useNavigate();

  if (socket() === undefined || name() === null)
    return <Navigate href="/setup" />;

  createEffect(() => {
    if (name() === null) navigate("/setup");
  });

  // Available chats
  const usersWithoutSelf = () => users()?.filter((user) => user !== name());

  return (
    <>
      <TopAppBar
        header="Melt"
        trailingAction={
          <button onClick={() => setName(null)} class="p-3 text-slate-900">
            <span class="sr-only">sign out</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              height="24px"
              viewBox="0 -960 960 960"
              width="24px"
              aria-hidden="true"
              fill="currentColor"
            >
              <path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h280v80H200Zm440-160-55-58 102-102H360v-80h327L585-622l55-58 200 200-200 200Z" />
            </svg>
          </button>
        }
      />
      <main class="bg-slate-100 px-4 pb-2 h-full">
        <Show when={!users.loading} fallback={<p>Loading...</p>}>
          <Show
            when={usersWithoutSelf() && usersWithoutSelf()!.length > 0}
            fallback={<p>No one available to chat</p>}
          >
            <ul class="bg-slate-50 rounded-3xl flex flex-col h-full">
              <For each={usersWithoutSelf()}>
                {(user) => (
                  <li class="text-base pl-4 pr-6 py-2 min-h-14 content-center">
                    <a href={`/chats/${user}`}>{user}</a>
                  </li>
                )}
              </For>
            </ul>
          </Show>
        </Show>
      </main>
    </>
  );
}
