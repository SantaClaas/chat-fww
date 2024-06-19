import { useNavigate } from "@solidjs/router";
import { useAppContext } from "../context";
import { createEffect } from "solid-js";

function Alert() {
  return (
    <div class="p-4 rounded-md bg-yellow-50">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg
            class="text-yellow-400 size-5"
            xmlns="http://www.w3.org/2000/svg"
            height="20px"
            viewBox="0 -960 960 960"
            width="20px"
            fill="currentColor"
            aria-hidden="true"
          >
            <path d="m40-120 440-760 440 760H40Zm138-80h604L480-720 178-200Zm302-40q17 0 28.5-11.5T520-280q0-17-11.5-28.5T480-320q-17 0-28.5 11.5T440-280q0 17 11.5 28.5T480-240Zm-40-120h80v-200h-80v200Zm40-100Z" />
          </svg>
        </div>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-yellow-800">Attention needed</h3>
          <div class="mt-2 text-sm text-yellow-700">
            <p>
              Lorem ipsum dolor sit amet consectetur adipisicing elit. Aliquid
              pariatur, ipsum similique veniam quo totam eius aperiam dolorum.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function SetUp() {
  const { name, setName } = useAppContext();
  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    // @ts-ignore Access form element called "name" and get the value. TS doesn't know about the DOM structure.
    const name = event.target.name.value;
    setName(name);
    localStorage.setItem("name", name);
  }

  const navigate = useNavigate();
  createEffect(() => {
    if (name() === null) return;
    navigate("/");
  });

  return (
    <div class="flex row-start-2 min-h-full flex-1 flex-col justify-center px-6 py-12 lg:px-8">
      <div class="sm:mx-auto sm:w-full sm:max-w-sm">
        {/* TODO use melt logo  */}
        <img
          class="mx-auto h-10 w-auto"
          src="https://tailwindui.com/img/logos/mark.svg?color=orange&shade=600"
          alt="Melt logo"
        />
      </div>

      <div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
        <form class="space-y-6" onSubmit={handleSubmit}>
          <div>
            <label
              for="name"
              class="block text-sm font-medium leading-6 text-gray-900"
            >
              Name
            </label>
            <div class="mt-2">
              <input
                id="name"
                name="name"
                autocomplete="username"
                required
                class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-orange-600 sm:text-sm sm:leading-6"
              />
            </div>
          </div>

          <div>
            <button
              type="submit"
              class="flex w-full justify-center rounded-md bg-orange-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-orange-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-orange-600"
            >
              Start
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
