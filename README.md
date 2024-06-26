# Melt

A small chat app for the project in Frameworks and Workflows class at Macromedia University of Applied Sciences.

# How to use

The easiest way to check out what the app does and how the code runs is to visit [the demo](https://melt.azurewebsites.net) hosted on Azure using their free tier and the Docker container built with the GitHub action. Expect longer loading times if the app hasn't been started in a while.

The second easiest way to demo the app is using the docker image hosted on the github container registry [ghcr.io/santaclaas/chat-fww:main](https://ghcr.io/santaclaas/chat-fww:main) or building it yourself from the [`Dockerfile`](/Dockerfile). This however requires a Docker installation and knowledge how to wield that power.

The third way is only recommended if you're already familiar with the tools used to built this and you want to run it directly on your machine. You need to have the tools installed and know how to run the [Vite](https://vitejs.dev/) app in [`./client/`](./client/) and the Rust server in [`./server/`](./server/). [How to install Rust](https://www.rust-lang.org/tools/install) for the server app. [How to install Node.js ](https://nodejs.org/en/learn/getting-started/how-to-install-nodejs) for the Vite app.

# Cool things in the app

- SolidJS Vite single page application
- Docker container combining frontend and backend into one app using the backend server to host the SPA
- GitHub action to automatically build the container
- Actor model for easy concurrency in Rust
- Star network topology using the delivery service actor to send messages between user actors to avoid full mesh topology which would cause a lot of memory overhead as every user would need to know of every other user
- Handmade logo

There's probably more but those are some of my personal highlights.

# Known limitations

- WebSocket do not recover their connection if they lose it. Currently a page refresh is required to reconnect in that case. There is also no warning or information helping the user in that case.
  - A possible solution could be a retry with exponential backoff and upper limit that requires manual user intervention after a fixed amount of retries.
- There is no user authentication. Currently you just need to know the user name to sign in. So don't share sensitive information. You are warned.
- Virtualization of lists. Too many messages or users will probably have a performance impact on the UI.
  - Possible solution: [TanStack Virtual](https://tanstack.com/virtual/latest/docs/introduction)
- Edge cases like messages that are too long have not been tested. There should probably be an upper limit on messages.
- No rate limiting. Bad actors could easily DoS.
- Message time is controlled by client and not server and client can write in it whatever they want
- Sometimes when refreshing too hard users stay alive on the server even though all their sockets have been closed and no client is connected
- Unknown limitations. I know there are limitations I don't know yet.

# Nice to haves

- Add data view for time send, time received in milliseconds to get information how long the message was possibly in travel
- Show time on tap if chat partner is in different time zone. Needs to save date in non-utc as ISO 8601
- Notifications

# Future plans

- Integrate End to End Encryption using [Messaging Layer Security](https://messaginglayersecurity.rocks/) an IETF RFC
- Make this into a full mobile app using Expo and react native with a shared Rust core
- Make sharing moments easier with view once photos
- Use GRPC or HTTP3/QUIC for the network layer instead of websockets
