# Give sigma to an AI: create, send, and operate a webxdc app

A webxdc is an offline HTML/CSS/JavaScript mini app packaged as an `.xdc` ZIP and opened inside Delta Chat. It can share state through chat without a hosted backend. Read the [short webxdc explanation](webxdc.md#what-is-webxdc-short-version) before packaging an app.

Use the named app commands for an AI that needs to send an app, interact with it, and return later. The background worker owns the live connection; subsequent CLI commands control that worker. The AI does not need to hold a blocking terminal listener open or remember numeric message IDs.

## Complete workflow

```sh
# Once per bot account (skip if already initialized):
./sigma init --name 'App bot'

# After creating app.xdc, send it once and save its identity as demo:
./sigma send-webxdc CONTACT_ID ./app.xdc --app demo --json
# For a group/chat ID, also pass --chat.

# Explicitly enable realtime observation (omit for update-only operation):
./sigma app-hook-add demo live ./examples/realtime-log-hook.py --type realtime

# Keep networking active in the background:
./sigma app-start demo --json
./sigma app-info demo --json

# Once realtime_joined is true, send live text or JSON text:
./sigma app-send demo '{"type":"ping"}' --json

# Send durable app state while the same worker keeps listening:
./sigma app-update demo '{"payload":{"text":"Hello from the AI"}}' --json

# Read captured realtime packets and status updates without blocking:
./sigma app-events demo --after 0 --json
```

The recipient must open the **same shared app instance** and join its realtime channel. `app-start` returns promptly even if no peer has connected. Poll `app-info` until `realtime_joined` is true before `app-send`. This flag means a peer joined the current worker session; it is not a continuous liveness or delivery receipt. `app-update` works while the worker is waiting for a realtime peer because it uses core's durable status-update transport.

Each event has an increasing `seq`. Save the last successfully processed sequence and pass it to `app-events --after SEQ` in later calls. This event cursor is distinct from core's status-update serials. For example:

```sh
./sigma app-events demo --after 12 --json
./sigma app-send demo ./packet.bin --file --json
./sigma app-list --json
./sigma app-stop demo --json

# Later, even from a different working directory:
./sigma app-info demo --json
./sigma app-events demo --after 12 --json
./sigma app-start demo --json
```

Names contain 1–48 ASCII letters, digits, hyphens or underscores and cannot start with a hyphen. A name is unique within an account. `send-webxdc --app demo` refuses an existing name before resending anything. If the app was already sent without a name, register its existing local message ID:

```sh
./sigma app-bind demo MESSAGE_ID --json
```

Do not resend the `.xdc` just to reconnect: a new app message is a different instance. To deploy new app code, send a new archive with a new name; updating an app's state does not replace its code.

## Notifications

Use either style through the running worker:

```sh
# Visible chat entry and user notification request:
./sigma app-update demo '{"payload":{"text":"Visible"},"info":"App changed","notify":{"*":"App changed"}}'

# Silent state update:
./sigma app-update demo '{"payload":{"text":"Silent"}}'
```

Both reach the app's status-update listener. Client notification settings still apply. `info` and `notify` are independent, as explained in [webxdc.md](webxdc.md).

## App-side code

The messenger provides `webxdc.js`. Include a script reference but do not put that file in the archive. Use both transports if the app needs live interactions and durable shared state:

```html
<script src="webxdc.js"></script>
<button id="click">Send live click</button>
<pre id="state"></pre>
<script>
webxdc.setUpdateListener(update => {
  document.getElementById('state').textContent = JSON.stringify(update.payload);
}, 0);
if (typeof webxdc.joinRealtimeChannel === 'function') {
  const channel = webxdc.joinRealtimeChannel();
  channel.setListener(bytes => {
    const text = new TextDecoder().decode(bytes);
    document.getElementById('state').textContent = text;
    // Interpret the data according to your application's own protocol.
  });
  document.getElementById('click').onclick = () => {
    channel.send(new TextEncoder().encode(JSON.stringify({type: 'button_clicked'})));
  };
  window.addEventListener('pagehide', () => channel.leave());
}
</script>
```

You can send `examples/realtime-app.xdc` directly to try this workflow. It returns a pong for a JSON ping and has a button for live clicks. `examples/button-app.xdc` demonstrates durable status updates. See the [official webxdc API](https://webxdc.org/docs/spec/api.html) for the app-side API and [realtime guide](webxdc-realtime.md) for binary packets and transport limits.

## What survives a restart

The account directory defaults to `~/.config/sigma`; always pass the same `--data-path` to every command if overriding it. These commands use the same core account, keys and messages as the rest of sigma.

```text
~/.config/sigma/
  account.db, account.db-blobs/, ...  # Core account, app messages, durable updates and queues
  apps/demo/
    app.json                        # Saved app name, message/chat IDs, status-update cursor
    events.jsonl                    # Captured events with increasing seq
    worker.log, worker.stderr       # Worker process diagnostics
    control.sock                    # Private local control socket while running
```

The event journal is written and synced before a successful command response. It stores received realtime bytes/text, outgoing submissions, status-update batches, worker lifecycle events and relevant notifications. On startup, the worker repairs an incomplete final journal line and recovers its committed status-update cursor. Event records can be read while running or stopped.

A live network connection cannot be stored on disk. Restarting the worker rejoins realtime only if a realtime hook is saved. Previously captured realtime packets remain in the journal, but realtime packets sent **while offline** are not recoverable from peers. Use durable status updates for state that must survive disconnection. Handle app data as untrusted input, not shell commands. Sending succeeds on submission, not on remote acknowledgement; a failed or interrupted request can have an uncertain outcome, so inspect state/events before retrying a state-changing command.

## Worker lifecycle and limits

`app-start` starts a separate background process and returns its status. Repeating it for a running app returns that status. `app-run demo` is the foreground equivalent; Ctrl-C stops it. `app-stop` leaves realtime, stops networking and releases the account lock. The worker stays alive when the launching CLI exits, but it is not an OS service and does not automatically restart after a reboot.

Workers use private Unix sockets and currently support Linux/macOS. Only one app worker can own a given account at a time. `app-list`, `app-info`, `app-events`, `app-send`, `app-update` and `app-stop` work while that worker is running. Other commands, including sending a new app, still need direct account access: stop the worker first, then restart it afterward. Use separate `--data-path` accounts for simultaneous independent workers.

Logs and app state remain local and private (0700 directories, 0600 state/journal/socket files). They are not automatically sent to other peers. Core's `backup` archive does not include the CLI's `apps/` directory; use a whole-account-directory copy with the worker stopped to preserve this metadata and journal. After a core-only restore, you can register a restored app message with `app-bind`.

## React to button events with an executable

Start the app with `app-start NAME --on-event ./hook.py --hook-timeout 10`. The saved hook receives realtime and status-update events as JSON, can return realtime/status-update replies, and retains its configuration across worker restarts. See [app-hooks.md](app-hooks.md) for the exact protocol, filtering and at-least-once behavior.

For event-specific handlers, use `app-hook-add NAME HOOK_ID EXECUTABLE --type realtime --match '{"type":"button_clicked"}'`. Register more than one ID for independent handlers, inspect them with `app-hook-list NAME --json`, and remove one with `app-hook-remove NAME HOOK_ID`. No worker stop is required for these commands. The old `--on-event` remains the broad `default` hook; remove that ID if you only want filtered hooks.

Workers with only `--type update`, `message`, or `command` hooks do not join realtime. Message/command hooks are scoped to the app chat; use `--type command --command /ping` and return `{"reply":"pong"}` for a quoted chat reply. See the hook guide for exact input fields.

For slash commands directly on a chat without an app, use `chat-hook-add CHAT_ID ID EXECUTABLE --command /ping` and run `listen`. See [chat command hooks](chat-hooks.md).
