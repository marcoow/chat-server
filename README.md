# Chat Server

This is the Websockets server to establish the Web RTC connection.

Run it via:

```bash
cargo run
```

Use [`cloudflared`](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/run-tunnel/trycloudflare/) to make your local server available via SSL for everyone:

```bash
cloudflared tunnel --url http://localhost:4000
```

(the client will need to establish the socket connection to "wss://<domain received from cloudflared>/room", e.g. "wss://plugins-flag-pay-limits.trycloudflare.com/room")
