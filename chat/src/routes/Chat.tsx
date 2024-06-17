import { useParams } from "@solidjs/router";
import { useWebSocket } from "../context";

export default function Chat() {
  const paramters = useParams();
  const socket = useWebSocket();

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    // @ts-ignore
    const message: string = event.target.message.value;
    console.debug(message);

    socket()?.send(message);

    // @ts-ignore
    event.target.reset();
  }
  return (
    <main>
      <h1>Chat with {decodeURI(paramters.name)}</h1>
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
