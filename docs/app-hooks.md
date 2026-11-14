# App event hooks

A named app worker supports multiple named executable hooks, each with an event type and optional payload filter. Hooks have explicit types: `realtime`, `update`, `message`, or `command`. Message and command hooks apply to incoming messages in the named app’s chat. The worker keeps receiving and accepting control commands while hooks run.

## Register, list and remove hooks

```sh
chmod +x ./examples/app-hook.py
./sigma send-webxdc CONTACT_ID ./examples/hook-app.xdc --app hooked --json

# Register separate hooks for the two button transports:
./sigma app-hook-add hooked live-buttons ./examples/app-hook.py \
  --type realtime --match '{"type":"button_clicked"}' --hook-timeout 10
./sigma app-hook-add hooked saved-buttons ./examples/app-hook.py \
  --type update --match '{"type":"button_clicked"}' --hook-timeout 10

./sigma app-start hooked
./sigma app-hook-list hooked --json
./sigma app-hook-remove hooked live-buttons
```

All three management commands work while the app is running or stopped. Registration is saved immediately and takes effect on future events. IDs are unique within an app; duplicates are refused. Multiple hooks can match the same event, and each runs independently. Removing one cancels its queued/running work and leaves the others active. It cannot undo external effects the executable already performed.

The recipient opens **Sigma hook demo** and clicks either button. The example executable writes a local marker and returns a reply, which appears in the app and chat. Keep the executable at its registered path; its code is not copied into the account or included in core backups.

### Specific events and payload filters

`--type` is required and selects `realtime`, `update`, `message`, or `command`. The older `--event realtime_received|webxdc_updates|message_received|command_received|all` remains supported. `--match` is an optional recursive JSON object subset:

```sh
./sigma app-hook-add hooked submit ./my-hook \
  --type update --match '{"action":"submit","form":{"id":"signup"}}'
```

Additional payload fields are allowed. Nested objects match recursively; arrays and scalar values require equality. For realtime, sigma decodes the packet's UTF-8 text as JSON and matches its payload directly. Binary/non-JSON packets do not match a JSON filter. Without a filter, binary packets can trigger a realtime hook.

For status updates, matching is against each update's `payload`. Only matching updates are included in that hook's input array; unrelated updates are excluded. The original batch cursor and event sequence remain available. One invocation can receive multiple matching updates in a batch.

### Messages and slash commands

```sh
./sigma app-hook-add hooked texts ./examples/message-hook.py \
  --type message --match '{"text":"hello bot"}'
./sigma app-hook-add hooked ping ./examples/command-hook.py \
  --type command --command /ping
```

A message hook sees incoming non-info messages from other people in this app’s chat, including slash commands. A command hook sees slash commands; `--command /ping` matches that exact first token. Its input has `type: "command_received"`, `command`, `args`, `text`, `sender_id`, `incoming_message_id`, and a `message` object. Message input has `type: "message_received"` and the same message fields without command/args. Filters for these two types match the event object. Return `{"reply":"pong"}` to quote and reply to the triggering message. The app’s own message ID remains `message_id`.

### Realtime connections

Without a realtime hook, a worker receives durable updates and messages but does **not** join realtime. Adding the first realtime hook joins immediately; removing the last leaves immediately. This persists across restart. `app-info` exposes `realtime_enabled` and `realtime_joined`. `app-send` refuses to join automatically: register a realtime hook first. For observation without replies, use `examples/realtime-log-hook.py`.

### Compatibility with --on-event

`app-start NAME --on-event PATH --hook-timeout 10` still defines a broad hook named `default`. Existing single-hook configurations migrate to that ID. It receives both webxdc event types without a payload filter and therefore enables realtime. It does not receive messages or commands. Remove it with `app-hook-remove NAME default` if only your filtered named hooks should run.

The compatibility flag replaces only `default` and requires a stopped worker. `--no-hook` on app-start/app-run clears all saved hooks and also requires a stopped worker. Normal `app-start NAME` reuses every saved hook. Use the management commands to add/remove specific hooks live.

Hooks are executed directly without a shell or arguments; use an executable wrapper if needed. Python is required only for the supplied example.

## Input

One JSON event plus a newline is written to stdin. It contains the same fields as the corresponding `app-events` record, plus the matching `hook_id` and an absolute `state_path` pointing to this named app's private directory:

```json
{"seq":12,"type":"realtime_received","hook_id":"live-buttons","app":"hooked","message_id":95,"time":1794400000,"text":"{}","data":[123,125],"bytes":2,"state_path":"/home/user/.config/sigma/apps/hooked"}
```

For realtime packets, `type` is `realtime_received`, `data` is a byte array, and `text` is UTF-8 or null. For status updates, `type` is `webxdc_updates`, with an `updates` array and `last_serial`. A batch can contain more than one button action. Hooks receive both local and remote status updates, including any newly captured startup catch-up; filter for your own button/event protocol. In particular, ignore your hook's response updates to prevent loops. Hooks do not receive worker lifecycle, notification, or hook-result records.

The app must explicitly send an event with its webxdc API. A DOM button click alone is not observable by the bot. The example supports both APIs.

## Output and replies

Empty/whitespace-only stdout means no reply. Otherwise return one JSON object with any of these fields:

```json
{"send":"UTF-8 realtime reply","update":{"payload":{"text":"Durable reply"},"info":"Button handled"}}
```

- `send` submits one realtime packet through the existing worker connection. A peer must have joined; normal 128000-byte limits apply.
- `reply` sends a quoted chat reply for message/command events only.
- `update` submits a webxdc status update through core, with the usual payload/info/notify fields.

All fields are checked before action execution; core also validates the update and permissions. Valid actions are processed in update, quoted reply, then realtime send order. Multiple actions are not atomic: a later failure does not undo an earlier submission. Neither action acknowledges remote delivery.

Stdout is capped at 64 KiB and must be UTF-8. Invalid output, nonzero exit, timeout, or an action error produces `hook_finished` with `success: false` and an error; the worker keeps running. Successful hooks produce `success: true`. Results include the `hook_id`, source `event_seq`, and bounded `output`. Completion and retry tracking are independent for each matching hook. Stderr goes to the worker's diagnostic file. Use stderr for diagnostics, not stdout.

Hooks run serially, separately from the worker's control/event loop. Up to 256 events can wait; if full, the event is retained in the journal with an explicit failed hook result. Slow hooks therefore cannot grow an unlimited live queue. A stopped worker kills a running hook through Tokio's child cleanup.

## Persistence and retries

The named hook configurations are stored in `apps/NAME/app.json`. Received events and `hook_finished` results are synced into `events.jsonl`. The example additionally writes `apps/NAME/hook-calls.jsonl`, proving the executable ran.

Adding a hook establishes its own new journal boundary: already journaled older events are not replayed through that hook. Newly captured status-update catch-up can still invoke it. On normal restart, each saved hook retries its matching events lacking that hook’s finished result. Finished failures are recorded and are not automatically retried. Disabling/replacing the hook abandons pending events from its old configuration.

This is at-least-once execution across interruption, not exactly-once external side effects. A crash after an external effect but before the result was committed can run the hook again. Make your hook idempotent using the stable app identity, `hook_id` and `seq`. For status-update processing, also consider update serials because new journal batches can be formed during crash recovery.

Hooks may invoke `app-send` or `app-update` because the control loop remains responsive, but returning actions on stdout is the simplest way to reply. Do not run commands requiring direct access to the locked core account. Remote payloads are untrusted data: your executable defines the permitted behavior.

For slash commands directly on a chat without an app, use `chat-hook-add CHAT_ID ID EXECUTABLE --command /ping` and run `listen`. See [chat command hooks](chat-hooks.md).
