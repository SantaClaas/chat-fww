# Things not covered

- WebSocket recovery when connection is lost
- User authentication. Currently you just need to know the user name to sign in
- Virtualization of chat message lists. Too many messages will currently have a performance impact
  - Possible solution: [TanStack Virtual](https://tanstack.com/virtual/latest/docs/introduction)
- Message time is controlled by client and not server and client can write in it whatever they want

# My Wishlist

- Add data view for time send, time received in milliseconds to get information how long the message was possibly in travel
- Show time on tap if chat partner is in different time zone. Needs to save date in non-utc as ISO 8601
- Notifications
