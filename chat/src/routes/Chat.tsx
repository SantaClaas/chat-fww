import { useNavigate, useParams } from "@solidjs/router";
import { useName, useWebSocket } from "../context";
import { createEffect } from "solid-js";

/**
 * This type has to be kept in sync with the server-side types
 */
export type Message = {
  to: string;
  from: string;
  text: string;
  /**
   * UTC unix timestamp in seconds
   */
  time: number;
};

export default function Chat() {
  const parameters = useParams();
  const socket = useWebSocket();
  const [name] = useName();
  const navigate = useNavigate();

  const isValidContactName = () =>
    typeof parameters.name === "string" && parameters.name.length > 0;
  const contactName = () =>
    isValidContactName() ? decodeURI(parameters.name) : null;

  createEffect(() => {
    if (!isValidContactName()) {
      // Go back to chat list
      //TODO inform user what happened
      navigate("/");
      return;
    }
  });

  function handleSubmit(event: SubmitEvent) {
    if (!(event.target instanceof HTMLFormElement))
      throw new Error("Invalid event target for form submission");

    event.preventDefault();
    const messageText: string = event.target.message.value;
    console.debug(messageText);

    const from = name();
    const to = contactName();
    if (from === null || to === null) return;

    const message = {
      to,
      from,
      text: messageText,
      time: Date.now(),
    } satisfies Message;

    socket()?.send(JSON.stringify(message));

    event.target.reset();
  }
  return (
    <main>
      <h1>Chat with {contactName()}</h1>
      <ol></ol>
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
