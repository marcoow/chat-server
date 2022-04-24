# Chat Server

This is the Websockets server to establish the Web RTC connection.

Use [ngrok](http://ngrok.com) to make your local server available via SSL for everyone:

```bash
ngrok http 4000
```

(the client will need to establish the socket connection to "wss://<domain received from ngrok>/room", e.g. "wss://c8a4-109-125-100-80.eu.ngrok.io/room")

Then run the server via:

```bash
cargo run
```
