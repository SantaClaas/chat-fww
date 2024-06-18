import { useNavigate, useParams } from "@solidjs/router";
import { useAppContext } from "../context";
import { For, Show, createEffect, createSignal } from "solid-js";

/**
 * This type has to be kept in sync with the server-side types
 */
export type ChatMessage = {
  recipient: string;
  sender: string;
  text: string;
  /**
   * UTC unix timestamp in seconds
   */
  time_utc: number;
};

export default function Chat() {
  const parameters = useParams();
  const { socket, name, messagesByUser } = useAppContext();
  const navigate = useNavigate();

  if (parameters.name === undefined || parameters.name.length === 0) {
    navigate("/");
    return;
  }

  const contactName = decodeURI(parameters.name);

  const messagesSignal = () => {
    let signal = messagesByUser.get(contactName);
    if (signal === undefined) {
      signal = createSignal<ChatMessage[]>([]);
      messagesByUser.set(contactName, signal);
    }

    return signal;
  };

  const messages = () => {
    const messages = messagesSignal();
    if (messages === undefined) return;
    return messages[0]();
  };

  const setMessages = () => {
    const messages = messagesSignal();
    if (messages === undefined) return;
    return messages[1];
  };

  function handleSubmit(event: SubmitEvent) {
    if (!(event.target instanceof HTMLFormElement))
      throw new Error("Invalid event target for form submission");

    event.preventDefault();
    const text: string = event.target.message.value;
    console.debug(text);

    const sender = name();
    const recipient = contactName;
    if (sender === null || recipient === null) return;

    const message = {
      recipient,
      sender,
      text: text,
      time_utc: Date.now(),
    } satisfies ChatMessage;

    socket()?.send(JSON.stringify(message));
    event.target.reset();

    // Add to local messages
    const setter = setMessages();
    if (setter === undefined) return;
    setter((previous) => [...previous, message]);
  }

  createEffect(() => console.debug(messages()));

  return (
    <main>
      <h1>Chat with {contactName}</h1>
      <Show when={messages() && messages()!.length > 0}>
        <ol>
          <For each={messages()}>{(message) => <li>{message.text}</li>}</For>
        </ol>
      </Show>
      <form onSubmit={handleSubmit}>
        <label
          for="message"
          class="block text-sm font-medium leading-6 text-gray-900"
        >
          Message
        </label>
        <div class="relative flex items-center mt-2">
          <input
            type="text"
            name="message"
            id="message"
            class="block w-full rounded-md border-0 py-1.5 pr-14 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-orange-600 sm:text-sm sm:leading-6"
          />
          <div class="absolute inset-y-0 right-0 flex py-1.5 pr-1.5">
            <kbd class="inline-flex items-center px-1 font-sans text-xs text-gray-400 border border-gray-200 rounded">
              ↵
            </kbd>
          </div>
        </div>
      </form>
    </main>
  );
}
