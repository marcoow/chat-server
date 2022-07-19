# Chat Server

This is the Websockets server to establish the Web RTC connection.

Run it via:

```bash
cargo run
```

or

```bash
cargo run -- --dev
```

to run the server in development mode which will for example change each match's duration to only 15s.

Use [`cloudflared`](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/run-tunnel/trycloudflare/) to make your local server available via SSL for everyone. First, get the Tunnel credentials from 1Password and save them into `./cloudflared/credentials.json`. Then run:

```bash
cloudflared tunnel --credentials-file .cloudflared/credentials.json run private-chat-roulette
```

(the client will need to establish the socket connection to "wss://<domain received from cloudflared>/room", e.g. "wss://plugins-flag-pay-limits.trycloudflare.com/room")
