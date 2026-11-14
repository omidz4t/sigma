# Webxdc realtime

For sending while listening, and saving app identity/events across AI tool calls, use the [named app worker workflow](AI.md). The commands below are the direct, foreground alternatives.

Realtime commands use chatmail core's peer channels. The app and CLI must be online in the same shared app instance. Packets are ephemeral: they have no saved history, replay cursor, or delivery receipt. For state that must survive disconnects, use [webxdc status updates](webxdc.md).

## Send an app and connect

```sh
./sigma send-webxdc CONTACT_ID ./examples/realtime-app.xdc --json
# Use the returned message_id below; 95 is an example.
./sigma listen-webxdc-realtime 95 --send '{"type":"ping","text":"Hello live!"}' --json
```

The recipient opens **Sigma realtime demo** and keeps it open. The CLI advertises its peer channel through core and waits for a connection (120 seconds by default; adjust with `--timeout`). After joining, it sends the initial packet and keeps receiving. The included app displays the ping and returns a `pong` packet. Press **Send realtime button event** in the app to send a `button_clicked` packet to the CLI.

Opening an older app message creates a different realtime session. Use the same newly shared instance on both sides. Local message IDs differ between accounts. A joined event reports core's channel handshake; it does not guarantee future connectivity or delivery of any packet.

## Send and receive

```sh
# Join and listen without sending initial data:
./sigma listen-webxdc-realtime 95 --json

# Send one UTF-8 packet, allow a brief queue grace period, then leave:
./sigma send-webxdc-realtime 95 'Hello live!' --timeout 60

# Send arbitrary binary bytes:
./sigma send-webxdc-realtime 95 ./packet.bin --file

# Send a binary packet after joining, then keep receiving:
./sigma listen-webxdc-realtime 95 --send-file ./packet.bin --json
```

`--send` and `--send-file` are mutually exclusive. Data is limited to 128000 bytes, including UTF-8 encoding overhead. Empty packets are allowed. File reads are bounded to reject oversized inputs without loading an arbitrarily large file. A send result means submitted to core, not acknowledged by a recipient.

One account supports one running command at a time. Use the listener's `--send` option to send an initial packet and receive replies in the same process. Stop it with Ctrl-C before another command uses that account. Stopping or a connection timeout leaves the realtime channel and stops core networking. The send-only command allows two seconds for its outgoing queue before leaving; this does not guarantee remote delivery.

There is no realtime history command or `--after` option. Restarting creates a new connection and cannot recover packets sent while disconnected. General `listen` does not join realtime channels automatically; use the dedicated command.

## JSON output

Connection and send records:

```json
{"type":"webxdc_realtime_joined","command":"listen-webxdc-realtime","message_id":95}
{"type":"webxdc_realtime_sent","command":"listen-webxdc-realtime","message_id":95,"bytes":5}
```

A received packet contains lossless bytes and optional decoded text:

```json
{"type":"webxdc_realtime_data","command":"listen-webxdc-realtime","message_id":95,"data":[104,101,108,108,111],"text":"hello","bytes":5}
```

`text` is null for bytes that are not valid UTF-8; `data` always contains the exact byte array. JSON sent by an app remains a string in `text`; parse it in your consumer if needed. Packet contents are untrusted app data and are never executed. Normal message hooks do not run on realtime packets.

## App JavaScript

The [experimental realtime API](https://webxdc.org/docs/spec/joinRealtimeChannel.html) supplies a channel accepting `Uint8Array` packets:

```html
<script src="webxdc.js"></script>
<script>
if (typeof webxdc.joinRealtimeChannel === 'function') {
  const channel = webxdc.joinRealtimeChannel();
  channel.setListener(bytes => {
    console.log(new TextDecoder().decode(bytes));
  });
  // Call this in your button handler:
  function sendClick() {
    channel.send(new TextEncoder().encode(JSON.stringify({type: 'button_clicked'})));
  }
  window.addEventListener('pagehide', () => channel.leave());
}
</script>
```

The messenger provides `webxdc.js`; do not include it in the archive. The example source is in `examples/realtime-app/`, and `examples/realtime-app.xdc` is ready to send. Rebuild it with the same Python ZIP recipe in the [status-update guide](webxdc.md), substituting `realtime-app` for `button-app`. No extra Rust dependencies were added.

Named app workers (`app-start` / `app-run`) join only while a realtime hook is registered. Add one with `app-hook-add NAME live ./examples/realtime-log-hook.py --type realtime`; removing the last leaves the channel. Update/message/command hooks do not enable realtime. The dedicated low-level commands above explicitly join on invocation.
