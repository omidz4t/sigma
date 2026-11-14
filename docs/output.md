# JSON output

Add `--json` to any command. Stdout becomes JSON Lines: one complete JSON object per line, with escaped Unicode/control characters and no human headings. Diagnostics go to stderr. Command errors go to stderr as `{"type":"error","message":"..."}` and return exit status 1. Success returns 0. An empty listing may produce no records. Native libraries may also write diagnostics to stderr (for example, SQLCipher when a backup password is wrong), so stderr is not guaranteed to contain only JSON. Parse structured error lines individually.

Every stdout record has `type` and `command`. Records for account state, contacts, chats, membership, sent messages, search/history/incoming messages, backups, joins and new management operations include named fields. Help and a few informational results use a `result` record with a `text` field. Version and reaction counts have typed records. Use the fields documented by record type rather than parsing human text where typed fields exist.

Examples:

```sh
sigma status --json
sigma chats --json
sigma read-messages 12 --json
sigma search-messages meeting --chat-id 12 --json
sigma send-message 12 'Hello' --chat --json
```

| Type | Fields (besides type/command) |
| --- | --- |
| `account` | `configured`, `name`, `addresses` |
| `contact` | `contact_id`, `name`, `address`, `key_contact` |
| `chat` | `chat_id`, `name`, `chat_type` |
| `message` | `message` object with IDs, sender, text, state, type and attachments |
| `member` | `chat_id`, `contact_id`, `name`, `address` |
| `account_ready` | `data_path`, `addresses` |
| `reactions` | `message_id`, `counts` (emoji/count/from_self objects) |
| `version` | `version` |
| `created` | `contact_id` or `chat_id` |
| `sent` | `message_id`, `chat_id` |
| `invitation` | `link`, and `chat_id` when applicable |
| `joined` | `chat_id` |
| `backup` | `files` (paths emitted by core) |
| `restored` | `database` |
| `auto_reply_queued` | `message_id`, `parent_id`, `chat_id` |
| `event` | `event` (core Debug event text; not a stable structured protocol) |
| `result` | `text` |

Message objects include `message_id`, `chat_id`, `sender_id`, `sender_name`, `sender_address`, `timestamp` (Unix seconds), `state`, `viewtype`, `text`, `filename`, `file`, `reactions`, `parent_id`, `download_state`, `forwarded` and `edited`. Optional filenames, local paths and parents use JSON null. Reaction summaries retain core's display format. Fields may be added as the CLI grows. Hooks receive this same message object directly, without the outer record envelope.

Keep invitation links, addresses, message text and file paths private where appropriate. `--json` changes presentation only; it does not enable networking, acknowledgements or hooks by itself. Help remains available without HOME/account access, including `sigma send-file --help --json`.

Webxdc commands and general `listen` emit `webxdc_updates` records with `message_id`, a native JSON `updates` array, and `last_serial`. `send-webxdc-update` emits `webxdc_update_queued` with the app `message_id`. See [webxdc.md](webxdc.md) for cursor and replay behavior.

Realtime commands emit `webxdc_realtime_joined`, `webxdc_realtime_sent` and `webxdc_realtime_data`. Received records have `message_id`, lossless `data` bytes, `text` (null for invalid UTF-8), and `bytes`. See [webxdc-realtime.md](webxdc-realtime.md).

Message records also include `duration_ms` (0 when unknown), used for voice/audio/video duration metadata. Voice notes have `viewtype: "Voice"`.

Named app control commands return `app_control` records with `ok`, `app`, `operation`, `message_id`, `chat_id`, and `message`. Status includes `running`, `realtime_enabled`, `realtime_joined`, `last_event`, and `last_serial`. A stopped app-info record has type `app` and `running: false`. `app-events` outputs stored JSON Lines with increasing `seq`, `type`, `app`, `message_id` and `time` (Unix seconds). Realtime events contain lossless `data`, optional UTF-8 `text` and `bytes`; status-update events contain native `updates` and `last_serial`. See [AI.md](AI.md).

Account and chat listings include `profile_photo` (stored local path or null). Dedicated bot/group photo changes return `profile_photo_updated` or `group_photo_updated`; the latter also includes `chat_id`.

App status reports the configured `hook_path`; running status also reports `hook_pending`. App journals include `hook_configured` and `hook_finished`. Results reference the original `event_seq`, `success`, `error` and bounded `output`. See [app-hooks.md](app-hooks.md).

Named hook listings return an `app_control` record with a `hooks` array (`id`, `path`, `event`, `match_json`, `timeout`, `after_seq`). `hook_finished` records include `hook_id`; a single source `event_seq` can have separate results for multiple hooks. Hook input carries `hook_id`, and filtered status-update input contains only matching updates.

`hook_path` is the compatibility `default` hook only; null does not imply that all named hooks are disabled. Use `app-hook-list` for the authoritative saved list, including when the worker is stopped.

Hook listings include an explicit `type` (`realtime`, `update`, `message`, `command`, or legacy `all`) and canonical `event`. Message/command journal records expose `incoming_message_id`, `text`, `sender_id`, and the detailed `message`; commands also have `command` and `args`. Hook stdin adds `hook_id` and `state_path`.
