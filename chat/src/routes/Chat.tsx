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
    setter((previous) => [message, ...previous]);
  }

  createEffect(() => console.debug(messages()));

  return (
    <>
      <header class="px-1 py-2 grid grid-cols-[3rem,1fr,3rem] gap-1 bg-slate-100">
        <a href="/" class="p-3 text-slate-900">
          <span class="sr-only">go back</span>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="24px"
            viewBox="0 -960 960 960"
            width="24px"
            fill="currentColor"
          >
            <path d="m313-440 224 224-57 56-320-320 320-320 57 56-224 224h487v80H313Z" />
          </svg>
        </a>
        <h1 class="text-xl font-normal content-center text-center">
          {contactName}
        </h1>
      </header>

      <main class="bg-slate-100 grid grid-rows-[1fr_auto] px-4 pb-2">
        <Show
          when={messages() && messages()!.length > 0}
          fallback={
            <article class="text-center bg-slate-50 rounded-3xl p-4">
              Start chatting by writing a message
            </article>
          }
        >
          <ol class="bg-slate-50 rounded-3xl p-4 flex flex-col-reverse gap-4">
            <For each={messages()}>
              {(message) => (
                <li
                  class=" px-4 py-1 max-w-fit text-slate-50"
                  classList={{
                    "bg-gradient-to-br from-slate-800 to-slate-600 rounded-bl-xl rounded-t-xl self-end":
                      message.sender === name(),
                    "bg-gradient-to-tl from-orange-600 to-orange-400 rounded-tr-xl rounded-b-xl":
                      message.sender !== name(),
                  }}
                >
                  {message.text}
                </li>
              )}
            </For>
          </ol>
        </Show>

        {/* <div class="fixed inset-x-0 p-4 bottom-0"> */}
        <form onSubmit={handleSubmit} class="relative flex items-center mt-2">
          <label for="message" class="sr-only">
            Message
          </label>
          <input
            type="text"
            name="message"
            id="message"
            placeholder="Type a message..."
            class="block w-full rounded-2xl px-4 border-0 py-1.5 pr-14 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-orange-600 sm:text-sm sm:leading-6"
          />
          <div class="absolute inset-y-0 right-0 flex py-1.5 pr-1.5">
            <kbd class="inline-flex items-center px-1 font-sans text-xs text-gray-400 border border-gray-200 rounded-2xl">
              ↵
            </kbd>
          </div>
        </form>
        {/* </div> */}
      </main>
    </>
  );
}
