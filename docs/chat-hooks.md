# Chat command hooks

Register slash commands directly on a chat ID. No webxdc app is needed; the same commands work in direct chats and groups where the bot can send messages.

```sh
./sigma chats --json
./sigma chat-hook-add CHAT_ID ping ./examples/command-hook.py --command /ping
./sigma chat-hook-list CHAT_ID --json
./sigma listen --json
# Someone sends /ping hello in this chat; the hook replies with a quoted "pong: hello".
# Stop listen with Ctrl-C before editing hooks:
./sigma chat-hook-remove CHAT_ID ping
```

The chat ID is local to the bot, not a contact ID. Each hook has a unique ID within that chat. Multiple hook IDs can select the same command and all run. Exact, case-sensitive first-token matching means `/ping` matches `/ping hello`, but not `/pingother`. The remainder is `args`. Hooks ignore outgoing/self messages and informational messages.

Definitions persist in `~/.config/sigma/chat-hooks/CHAT_ID.json`; `--data-path` changes the account directory. Stop the existing app worker or listener before managing definitions or starting `listen`, because only one process owns the account. Registration survives CLI restarts. The listener loads definitions for each incoming command and never joins webxdc realtime.

## Executable protocol

The executable receives one JSON object plus a newline on stdin:

```json
{"type":"command_received","hook_id":"ping","chat_id":13,"incoming_message_id":163,"text":"/ping hello","command":"/ping","args":"hello","message":{"message_id":163,"chat_id":13,"sender_id":11}}
```

`message` is the full standard message record. Run the executable directly, without shell evaluation or arguments; use an executable wrapper when needed. Treat remote text as data. Executables must stay at their saved absolute paths.

Empty stdout means no reply. Otherwise return `{"reply":"pong"}`. The bot sends a quoted reply in the triggering chat. Chat hooks accept only the reply action; app `send` and `update` actions belong to app hooks. Execution is bounded by `--hook-timeout` (default 30 seconds) and 64 KiB UTF-8 stdout. Failures emit `chat_hook_finished` with `success: false`; other matching hooks still run. Stderr is diagnostic output.

The listener runs hooks serially and reports results on stdout. It does not maintain a durable hook retry queue or replay old stored messages on startup. Commands received through normal core synchronization can trigger hooks; make external effects idempotent using `incoming_message_id` and `hook_id`. Do not invoke CLI commands needing the locked account from inside a hook; return a reply instead.
