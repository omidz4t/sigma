# Webxdc apps and shared updates

## What is webxdc? (short version)

A **webxdc** is a mini web app shared as an attachment and opened inside Delta Chat:

- **Package:** an `.xdc` file is a ZIP (Deflate or Store), with `index.html` at the root plus local HTML/CSS/JavaScript and assets. `manifest.toml` can name the app; `icon.png` or `icon.jpg` can provide its icon.
- **Runtime:** the page runs offline in a sandbox without ordinary internet access. Bundle dependencies locally; no web server or API key is needed. For an existing website, build static output and use hash-based or single-page navigation instead of server routes.
- **Sharing:** a static page needs no special API. Interactive shared state uses the client-provided `webxdc.js` with `sendUpdate` and `setUpdateListener`. Do not bundle `webxdc.js`. Updates travel through chat when connected; arbitrary clicks and local storage are not automatically synchronized.
- **Sigma:** `./sigma send-webxdc me ./app.xdc` sends the app. Each sent app message is a separate shared instance. State updates do not replace app code; send a new package for new code.

[Official format](https://webxdc.org/docs/spec/format.html) · [JavaScript API](https://webxdc.org/docs/spec/api.html)

For an AI operating an app over time, start with the [named app workflow](AI.md): `send-webxdc --app NAME`, `app-start`, `app-send`, `app-update`, and `app-events`. The ID-based commands below remain available for direct use.

`send-webxdc` sends an `.xdc` app. `send-webxdc-update` sends shared state to an existing instance, and `listen-webxdc-updates` receives state sent by the app, including button clicks. Each app instance is identified by the local message ID returned when you sent or received the original app message. Peers have different local IDs for the same shared instance.

## Try the button demo

The repository includes `examples/button-app.xdc` and its HTML/manifest source in `examples/button-app/`. Send it to a contact:

```sh
./sigma send-webxdc CONTACT_ID ./examples/button-app.xdc --json
# Example output: {"type":"sent","command":"send-webxdc","message_id":80,"chat_id":10}
```

Use the actual returned `message_id` in these commands (80 below is an example):

```sh
./sigma send-webxdc-update 80 '{"payload":{"text":"Hello from CLI"},"summary":"CLI update"}'
./sigma listen-webxdc-updates 80 --json
```

Keep the listener running. The recipient opens the app and presses **Send button event**. The CLI prints a `webxdc_updates` record containing a payload with `action: "button_clicked"`. The CLI's earlier text also appears inside the app. Networking transmits queued updates while the listener runs; stopping immediately does not guarantee delivery. Updates use chatmail transport and can take time to arrive.

The account lock allows one command at a time per account. To send another CLI update, stop the listener with Ctrl-C, run `send-webxdc-update`, and restart the listener. Its default replay includes all stored updates so you can recover missed events.

## Read and resume

```sh
./sigma webxdc-updates 80 --json
./sigma webxdc-updates 80 --after 5 --json
./sigma listen-webxdc-updates 80 --after 5 --json
```

`webxdc-updates` reads local history without networking. The dedicated listener first prints the stored updates after the cursor, then prints new batches. Empty initial history produces an empty array. Each JSON record has this shape:

```json
{"type":"webxdc_updates","command":"listen-webxdc-updates","message_id":80,"updates":[{"payload":{"action":"button_clicked"},"serial":6,"max_serial":6}],"last_serial":6}
```

Persist `last_serial` after processing a batch and pass it as `--after` when restarting. Serials are local, may have gaps, and belong to this account; never reuse a peer's cursor. Updates contain both local and remote state. Repeated events within one listener run do not replay already emitted serials. To avoid replay across restarts, supply your saved cursor. Payloads are arbitrary app-defined JSON, not trusted commands; this listener prints them and never executes them.

General `listen --json` also emits structured `webxdc_updates` records for apps whose state changes, alongside its normal messages/events. On the first update event for an app in that listener run, it includes all locally stored updates for that instance. Use the dedicated listener for filtering to one app and resuming from a known cursor. Message `--on-message` hooks do not run on app updates.

## Make your app emit button events

Use the [official webxdc API](https://webxdc.org/docs/spec/api.html):

```html
<script src="webxdc.js"></script>
<button id="send">Click</button>
<pre id="state"></pre>
<script>
webxdc.setUpdateListener(update => {
  document.getElementById('state').textContent = JSON.stringify(update.payload);
}, 0);
document.getElementById('send').onclick = () => {
  webxdc.sendUpdate({payload: {action: 'button_clicked'}});
};
</script>
```

Reference `webxdc.js` in your HTML; the messenger supplies it, so do not package that file. A button only reaches the CLI if your app explicitly calls `sendUpdate()`. The CLI cannot observe arbitrary DOM clicks in another client.

To rebuild the included example with Python's standard library (Python is needed only for packaging this example):

```sh
python3 - <<'PY'
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED
with ZipFile('examples/button-app.xdc', 'w', compression=ZIP_DEFLATED) as app:
    for path in sorted(Path('examples/button-app').iterdir()):
        app.write(path, path.name)
PY
```

A status update changes app data, not its HTML/JavaScript code. To distribute a new version of the app code, send a new `.xdc` file; it creates a separate instance. Core validates update JSON and chat permissions. `send-webxdc-update` reports **queued**, not delivered; start either listener to transmit it.

All four commands support `--help`, `--json` and `--data-path`. Account data defaults to `~/.config/sigma`. No extra Rust dependencies are needed for these commands.

For ephemeral live packets, see the [realtime guide](webxdc-realtime.md) and `examples/realtime-app.xdc`.

## Visible and silent updates

The same command supports both styles. A visible update can add an entry in the chat with `info` and request a user notification with `notify`:

```sh
./sigma send-webxdc-update 83 '{"payload":{"text":"Visible update"},"info":"The app changed","notify":{"*":"The app changed"}}'
```

A silent update omits both fields:

```sh
./sigma send-webxdc-update 83 '{"payload":{"text":"Silent update"}}'
./sigma listen-webxdc-updates 83 --json
```

Both update the app's shared state and reach `setUpdateListener`. Silent here means no requested notification and no added info entry; the app may still change its own interface. `summary` only changes the app summary, so it is not a substitute for `notify`.

`info` and `notify` are independent: you can add an info entry without requesting a notification, or request a notification without adding a chat entry. The `"*"` notify key targets receiving peers; targeted keys use the app-specific `webxdc.selfAddr`, not ordinary email addresses. Core suppresses wildcard notifications for muted chats; OS/client settings and whether the app is open can affect visible alerts. Notification requests do not guarantee a popup. See the [official field definitions](https://webxdc.org/docs/spec/sendUpdate.html).
