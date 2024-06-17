# Things not covered

- WebSocket recovery when connection is lost
- User authentication. Currently you just need to know the user name to sign in
- Virtualization of chat message lists. Too many messages will currently have a performance impact
  - Possible solution: [TanStack Virtual](https://tanstack.com/virtual/latest/docs/introduction)
