# Agent chat tools

Running agents receive starting instructions explaining how to send media, send several independent messages, react, edit their messages and update their account profile. Ask your bound agent naturally, for example “send this as a voice message”, “send the image with this caption”, or “change your name to Helper”. The instructions include literal paths suitable for headless tool permissions, the incoming message ID and a private outbox location. Existing agents receive these instructions on their next provider turn after upgrading/restarting the worker; their native session stays the same.

The tool command is `sigma agent-act ACTION ...`. It uses the authorized agent's environment and works during its authorized turn while the worker holds the account lock. It never opens a second core account or asks for a new recipient. Commands outside an authorized agent environment fail rather than falling back to the default account.

## Available actions

| Action | Arguments | Options | Result |
| --- | --- | --- | --- |
| `send-message` | `TEXT` | none | One independent text message without a quote |
| `send-voice` | `PATH` | `--caption TEXT`, `--duration MS` | Existing audio sent as a voice note |
| `send-file` | `PATH` | `--caption TEXT` | File attachment |
| `send-video` | `PATH` | `--caption TEXT`, `--duration MS` | Existing video sent with the video message type |
| `send-photo` | `PATH` | `--caption TEXT` | Image with optional caption |
| `send-webxdc` | `APP.xdc` | `--caption TEXT` | Send a validated Webxdc ZIP with index.html |
| `webxdc-update` | `MESSAGE_ID JSON` | none | Queue durable shared app state containing payload |
| `webxdc-updates` | `MESSAGE_ID [AFTER_SERIAL]` | none | Read updates_json and last_serial for an app in this chat |
| `webxdc-realtime` | `MESSAGE_ID TEXT` | none | Send ephemeral data to an online peer in this app instance |
| `messages` | `[COUNT [BEFORE_MESSAGE_ID]]` | none | Read recent messages and IDs in the bound chat (default 20, maximum 50) |
| `reply` | `MESSAGE_ID TEXT` | none | Send an explicit quoted reply within the bound chat |
| `react` | `MESSAGE_ID [EMOJI]` | none | Set a reaction; omit/empty the emoji to remove it |
| `edit-message` | `MESSAGE_ID TEXT` | none | Edit this account's own message in the bound chat |
| `set-name` | `TEXT` | none | Change this account's display name |
| `set-photo` | `PATH` | none | Change this account's profile photo; empty path clears it |
| `batch` | `JSON_FILE` | none | Run up to 32 ordered actions |

Durations are nonnegative milliseconds up to 2147483647. Text and captions are limited to 64 KiB; a complete signed request is limited to 1 MiB. Media must be an existing regular file inside `SIGMA_AGENT_OUTBOX`; relative media paths resolve there. Paths escaping through `..`, absolute paths outside the outbox, symlinks leading outside it, and private mailbox files are refused. Generate or copy files into the outbox first. Sigma does not synthesize speech, generate images, record audio or transcode media; the provider uses its own available tools to create those assets. Playback depends on the recipient's supported formats.

The shell examples below use the environment variable notation; actual native starting instructions render literal executable and media paths for Grok's headless permission rules:

```sh
"$SIGMA_BIN" agent-act send-message "First message"
"$SIGMA_BIN" agent-act send-message "Second message"
"$SIGMA_BIN" agent-act send-voice "$SIGMA_AGENT_OUTBOX/voice.ogg" --duration 3500
"$SIGMA_BIN" agent-act send-photo "$SIGMA_AGENT_OUTBOX/image.png" --caption "A caption"
"$SIGMA_BIN" agent-act send-video "$SIGMA_AGENT_OUTBOX/video.mp4" --caption "A clip"
"$SIGMA_BIN" agent-act set-name "Helper"
"$SIGMA_BIN" agent-act set-photo "$SIGMA_AGENT_OUTBOX/avatar.png"
```

Actions send unquoted messages; they do not reply to the triggering message. To avoid an additional automatic quoted text response after sending with tools, the agent ends its final answer with `[SIGMA_NO_REPLY]`, ideally that marker alone. Sigma suppresses that turn's entire automatic text reply, including commentary combined into Grok's final output. Ordinary final text still becomes a quoted reply.

## Multiple actions, results and retries

A batch file contains an array, for example:

```json
[
  {"action":"send-message","text":"Here are the results"},
  {"action":"send-photo","path":"image.png","caption":"Preview"},
  {"action":"send-voice","path":"voice.ogg","duration":3500},
  {"action":"set-name","text":"Helper"}
]
```

Other supported fields are `message_id`, `text`, `path`, `caption`, `duration` and `after`, only where applicable. Unknown fields are refused. Every action's scope and file path is checked before a batch starts; a later runtime error can still leave earlier actions completed. Results always use JSON with `ok`, `request_id`, `results` and `error`; sending actions report their queued message IDs. Queued submission is not a recipient delivery guarantee. Core edit/reaction restrictions still apply, and account profile changes propagate through subsequent outgoing messages.

Every call gets a UUID request ID. Pass `--request-id UUID` explicitly when a call may need retries; retry the same ID with the same actions. Completed and partial results are saved privately under `agents/ALIAS/action-results/`. Reusing the ID returns that result without sending again; changing its actions is refused. If the worker is interrupted between queueing and saving a result, the pending record refuses an automatic repeat and asks you to inspect queued messages. Do not blindly retry with a new ID. Do not modify staged media while retrying the same request.

## Authorization and execution

Each provider turn generates its own private random capability, scoped to its current authorized chat. Only its subprocesses receive `SIGMA_AGENT_CAPABILITY`, `SIGMA_AGENT_SOCKET`, `SIGMA_AGENT_MAILBOX`, `SIGMA_AGENT_OUTBOX`, `SIGMA_AGENT_INSTRUCTIONS` and `SIGMA_BIN`. The current message ID is also available as `SIGMA_INCOMING_MESSAGE_ID`. The raw capability is not included in model-visible starting instructions, agent listings, action results or stored request files.

The client signs the request UUID and complete action set using HMAC-SHA256. The worker verifies the signature before acting. A credential for another agent does not authorize this worker, and changing a signed action invalidates it. The service fixes the account and current authorized chat from the owner permission policy; there are no `--to`, `--chat`, `--account` or `--data-path` overrides. Reaction/edit IDs must belong to that exact chat, and editing requires an own message. Name/photo changes use only the worker's own core account and are refused for approved guests.

A private Unix socket handles normal calls. If the socket is unavailable or a provider sandbox refuses it, the same CLI exchanges signed requests through a private `.sigma-inbox` mailbox under the outbox. The mailbox contains signatures, not the raw capability, and uses the same request/result deduplication and authorization as the socket. Other agents' tokens fail on both transports. The API scopes messages; the bridge additionally applies [OS process isolation](agent-security.md).

Codex uses a workspace-write sandbox rooted at the agent outbox so it can prepare generated files and use the mailbox, without making the source checkout its writable workspace. Grok receives allow rules for the literal `sigma agent-act ...` command and file writing/editing under that outbox; other shell commands retain its configured permission handling. Both providers keep their existing exact session IDs. Custom adapters keep the plain-text stdin/stdout contract and receive the same environment/instruction variables; they can opt into these tools themselves.

No HTTP listener or public network endpoint is opened. Agent outboxes, socket/mailbox state and request results stay in the private account directory. Finishing a provider turn or stopping the worker closes its action service; the next turn has a different capability.

Use `agent-act messages` to find IDs for edits and reactions, including automatic replies. Results include sender ownership, attachment paths and text summaries capped at 1,000 characters. Use `next_before` as the second argument to page older messages. Request history separately from action batches.

To choose a quoted reply explicitly, use `"$SIGMA_BIN" agent-act reply "$SIGMA_INCOMING_MESSAGE_ID" "Your answer"`, then end the final output with `[SIGMA_NO_REPLY]` to suppress the automatic response. Use `messages` to find an older message ID. `send-message` remains unquoted.

## Webxdc in brief

Create an offline HTML/JS app ZIP named `.xdc`, with `index.html` at its root, in the outbox. `send-webxdc` sends it and returns the local app message ID. Use `webxdc-update ID '{"payload":{"value":1}}'` for durable shared state; optional `info`/`notify` fields make updates visible. Updates change state, not app code. Use `webxdc-updates ID AFTER_SERIAL` separately from batches, parse `updates_json` and save `last_serial`. Responses are bounded; request newer serials if old history exceeds the limit.

`webxdc-realtime ID TEXT` requires both peers online in the same app, waits up to 20 seconds for a peer, sends best-effort ephemeral data and leaves. It has no replay or delivery acknowledgement. The bridge continues networking for durable updates. These tools do not install app hooks or continuously feed realtime events into the model. Do not start another account/app worker while the bridge holds the account lock. All app IDs must belong to the current authorized chat. See [sender permissions](agent-permissions.md) for group authorization.

Incoming voice/audio is handled by the [agent transcription workflow](voice.md#incoming-voice-notes-for-agents): the agent transcribes using its own capabilities and uses the result as input, or returns `[SIGMA_VOICE_UNSUPPORTED]` so Sigma reacts with ❌. Sigma does not transcribe on the agent’s behalf.

Bridge processes now use an outer Linux sandbox, with the account/database and authority files hidden. Only the conversation outbox and isolated provider runtime are writable host-backed state. Bot master transfer invalidates old action scopes; delegates never obtain master-transfer or call-answer authority. See [bot master permissions](agent-permissions.md) and [database protection](agent-security.md).
