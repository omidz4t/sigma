# Bot automation

Networking is enabled by `listen`. Automatic replies are opt-in:

```sh
sigma listen --auto-reply 'Thanks, your message was received.'
sigma listen --on-message ./handler.py --hook-timeout 10 --json
```

Choose either a fixed `--auto-reply` or an executable `--on-message` hook. They cannot be combined. `--hook-timeout` requires a hook and defaults to 30 seconds. The same account lock covers the listener, so the hook must not invoke another sigma process against this account directory. Instead, write the reply to stdout.

The executable is invoked directly, without a shell or extra arguments. Use an executable file with an appropriate shebang; paths with spaces work when quoted. One message JSON object and a newline are written to stdin. The object is described in [output.md](output.md). UTF-8 stdout becomes a quoted text reply to that message in the same chat. Empty/whitespace-only stdout sends no reply. Stderr is inherited for diagnostics.

Example executable Python hook (Python is required only for this example):

```python
#!/usr/bin/env python3
import json
import sys
message = json.load(sys.stdin)
if message['text'].strip().lower() == 'ping':
    print('pong')
```

```sh
chmod +x ./handler.py
sigma listen --on-message ./handler.py
```

Hook stdout is capped at 64 KiB and must be UTF-8. A nonzero exit, output overflow or timeout reports an automation error and keeps listening without sending a reply. The child is killed when the operation is dropped; Ctrl-C interrupts a running hook and then stops core I/O. Hook stderr is not transformed into JSON.

Automation handles core `IncomingMsg` events. Contact requests do not generate these events until the chat is accepted (for example, by sending a message to that contact). Stored request messages are not replayed when the chat is accepted. Own messages and core information messages do not trigger replies. Incoming app/file/image messages can trigger hooks, with their type and attachment path in the JSON. Replies to read-only chats are rejected by core. During a running listener, recently handled message IDs are remembered (a 4096-ID window); this is not a persistent exactly-once queue across restarts. Hooks run serially, so slow hooks delay processing of later events while core continues receiving in the background.

A queued automatic reply is reported as `auto_reply_queued`; networking sends it afterward. There is no guarantee that the reply reaches the recipient before the listener is stopped. Replies to messages sent by another bot can create loops if both bots automatically reply to each other; design hook filters for the conversations you intend to automate.


## Local agent bridge

```sh
sigma agent-connect codex --new-account codex-bot --name "Codex" \
  --profile-photo ./avatar.png --to me --invite "FULL_CONTACT_INVITATION" \
  --system-prompt "Give concise, helpful replies."
sigma agent-list --account codex-bot
sigma agent-stop codex --account codex-bot
sigma agent-connect codex --account codex-bot --to me
sigma agent-connect grok --to me
sigma agent-connect grok --alias researcher --to me --model MODEL_ID --system-prompt "Be concise."
sigma agent-connect grok --to me --system-prompt-file ./instructions.txt
sigma agent-list --json
sigma agent-stop researcher
# Restart the saved identity using its first recipient:
sigma agent-connect grok --alias researcher
# IDs printed by agent-list also work:
sigma agent-connect grok --agent-id ID
sigma agent-stop ID
sigma agent-connect grok --to me --exec ./grok-adapter --hook-timeout 120
```

Each agent has a saved **alias** and automatic positive numeric **ID** within its account. Use `--alias NAME` when creating an identity; omitting it uses the provider name (`grok`, `codex`, etc.). Aliases follow the contact-alias naming rules and are separate from contact aliases and account names. IDs are allocated monotonically and survive stopping/restarting. `--agent-id ID` selects an existing identity instead of `--alias`; it cannot be combined with `--alias` or `--new-account`. The positional argument remains the provider, and a saved identity cannot switch providers.

The first `--to` binds the agent to the resolved contact ID, address, key fingerprint (when available) and direct chat. A new identity requires `--to`; a saved identity may omit it and use the stored person. An explicitly supplied alias or numeric contact ID must resolve to that same person. Changing or reassigning a contact alias does not move the agent. A new invitation is refused for an existing identity before a handshake or connection message is sent. The original full invitation may be repeated as `--to` only when it was the identity's original target; it uses the saved person without joining again. To talk to someone else, create a new agent alias. Bindings are saved before the initial notification is queued, so even a failed send cannot accidentally release an identity for reassignment.

`--new-account NAME` creates and provisions a dedicated named identity without changing the default. The name must be new; on partial provisioning failure retry `init --account NAME`, then connect with `--account NAME`. `--name TEXT` and `--profile-photo PATH` set the chat account’s profile, including on existing accounts. `--profile-photo-address` remains an alias. A new account must perform its own contact handshake: use `--to me --invite FULL_CONTACT_INVITATION` to join and save an account-local `me` alias, or pass the invitation directly as `--to LINK`. Do not reuse numeric IDs from another account. `--invite` refuses an alias that already exists; omit it on restarts.

The target uses the same contact IDs and case-sensitive account-local aliases as `send-message`. Without `--invite`, an alias such as `me` must already exist (create it with `create-contact me INVITATION`). Self/special contacts are rejected. The agent name must follow the alias naming rules. Without `--exec`, Sigma searches PATH for an executable matching that name. With `--exec`, Sigma runs that executable file directly, without arguments or shell evaluation. Missing executables fail before a notification is queued.

For `grok` without `--exec`, Sigma invokes `grok --prompt-file PRIVATE_FILE --output-format json --permission-mode dontAsk`, avoiding the interactive UI and interactive permission prompts. Add `--model MODEL_ID` to pass that model ID to every Grok invocation; omit it to keep Grok’s configured default. Model availability is validated by Grok. Only native Grok and Codex accept `--model`; it is rejected with `--exec`, so configure custom adapters directly. Set starting instructions with `--system-prompt TEXT` or `--system-prompt-file PATH` (mutually exclusive, native Grok/Codex only). The UTF-8 file is read once when the worker starts and must contain nonblank text. Sigma passes the prompt using Grok’s `--system-prompt-override` on every invocation. The listing reports whether a system prompt is set, without including its contents. Conversation history is retained in the native provider session described below.

For native `codex`, Sigma invokes `codex --no-daemon --ask-for-approval never exec --sandbox workspace-write --json --skip-git-repo-check -`, reading the user message from stdin. Sigma saves the reported `thread.started` ID immediately and returns only the final assistant message from the event stream. Later messages use `exec --sandbox workspace-write resume SESSION_ID --json --skip-git-repo-check -` with that exact ID. Ephemeral mode and “resume latest” are never used. `--model` selects the Codex model. Starting instructions are passed as the separate `--config developer_instructions=...` argument, adding developer instructions rather than replacing Codex's built-in system instructions. Grok's starting instructions use `--system-prompt-override`. Both native providers use the installed CLI's authentication and model configuration. Sigma appends `Model: MODEL_ID.` to their starting instructions when `--model` is supplied, including when no custom system prompt is given. Without `--model`, the instructions explicitly say the provider CLI’s configured default is in use and that Sigma has not supplied an exact model ID; Sigma does not guess it. Custom adapter stdin is unchanged. No API keys are copied into Sigma account metadata. Each message runs a CLI turn in the same saved conversation, rather than creating a new conversation.

Native sessions are tracked privately in `DATA_PATH/agents/ALIAS/session.json`. Grok reads each turn from a temporary private prompt file (removed after the turn), so initial history can exceed operating-system argument-length limits. Grok receives a new UUID through `--session-id` only on its first turn and uses `--resume EXACT_UUID` thereafter. Sigma checks the `sessionId` returned by Grok; Codex’s returned thread ID must also match the saved one. Session pointers survive worker stops and restarts and are visible as `session_id` in `agent-list`. Aliases/accounts never share session pointers.

When a native agent has no saved session yet, Sigma imports the text history of its bound direct chat once, with explicit user/assistant roles, before the current message. This includes conversations from the old stateless bridge. Startup notifications, informational messages, unrelated chats and other senders are excluded. The imported history is data; earlier requests are not rerun as new instructions. After that first bootstrap, only new incoming text is submitted, and the provider manages conversation history and compaction. A bootstrap beyond 512 KiB fails instead of silently losing earlier messages. Attachment bytes are not imported.

A failed or timed-out turn retains the saved provider session ID. Sigma never silently replaces it or falls back to a fresh/latest thread. If initial creation is interrupted before Codex reports an ID, the pending record prevents automatic creation of a second conversation; restore the original provider session ID before retrying. A missing/deleted provider session also requires explicit repair. Provider history remains in the installed provider CLI’s own storage, so back up that storage as well as Sigma’s pointer when migrating machines. Deleting a Sigma account does not delete the provider’s external session history. Native assistant replies remain limited to 64 KiB, while structured provider output is limited to 4 MiB per turn.

The custom adapter executable contract is **plain UTF-8 message text plus a newline on stdin, plain UTF-8 reply on stdout**. Use an executable wrapper for a CLI that expects a prompt argument, flags, or different output formatting. Sigma starts one process for each incoming message; the adapter is responsible for any conversation/session memory. Keep diagnostics on stderr. Do not point this at an interactive terminal UI.

For example, if your installed agent accepts `--prompt TEXT` (replace this invocation with the actual options supported by your CLI):

```python
#!/usr/bin/env python3
import subprocess
import sys
prompt = sys.stdin.read()
subprocess.run(["/absolute/path/to/your-agent", "--prompt", prompt], check=True)
```

Save that as `grok-adapter`, make it executable with `chmod +x ./grok-adapter`, and pass `--exec ./grok-adapter`. Keep credentials in the local agent's environment/configuration, outside this repository.

Sigma queues a connection message to the selected direct chat and starts receiving. Only incoming messages from that contact in that exact chat, with nonempty text, reach the executable; unrelated contacts, groups, own messages and informational messages are excluded. Attachment bytes are not passed to the agent; captions can be passed as text. Old stored messages are not replayed. Successful stdout is queued as a quoted reply in the original chat. Whitespace-only output sends nothing. Nonzero exits, invalid UTF-8, output beyond 64 KiB, or the per-message timeout (default 120 seconds) report an error locally and keep the bridge listening. Recent event duplicates are skipped for the running connection; there is no durable exactly-once queue across restarts.

`agent-connect` starts a background worker by default and returns after it becomes ready. `agent-list` shows saved identities with their ID, alias, provider, bound target and running/stopped state. Running workers also show their PID, model and system-prompt flag. Stale sockets do not count as running; stopped identities remain listed. `agent-stop ID_OR_ALIAS` asks the selected worker to stop, cancels its active invocation and waits for the account lock to be released. These management commands work while the worker is online. All three commands use the same `--account NAME` or `--data-path PATH`; omitted selectors use the saved default (initially `legacy` in `~/.config/sigma`). Use the same selector for connecting, listing and stopping a dedicated account. A second worker cannot share an account already used by another agent, app worker or listener. To change the model or prompt, stop the worker and connect again using its alias or ID. The recipient binding cannot change.

Workers use private control sockets and logs under `DATA_PATH/agents/ALIAS/worker.log`. Their durable identity and recipient binding live in `agent.json` beside that log, and `agents/next-id` reserves future IDs. These files stay private with the account. Existing bridges from versions without durable IDs acquire their ID/binding when restarted; preserve their original recipient during that migration. Stop commands contact the live worker rather than signaling a saved PID. On startup failure, the command points to the worker log. Worker management currently requires Unix (Linux/macOS). The command has been renamed from `a2a-connect` to `agent-connect`.

With `--foreground`, the worker runs in the current terminal until Ctrl-C or `agent-stop`. Stop it before running other messaging commands on the same account. Ctrl-C in the foreground stops networking and cancels an active invocation. Queued messages require the connection to remain online for delivery. The connection message indicates a configured local bridge, not a completed remote-agent handshake. This is a local process bridge, not an implementation of the HTTP Agent2Agent protocol. No direct provider API integration is assumed.


## Acting on the bound chat

Native agents now receive the [agent chat tool guide](agent-actions.md) in their starting instructions, with a per-worker capability available only through their process environment. `agent-act` performs media, standalone messages, reactions, edits and own-account profile changes through the running worker, so it does not contend for the account lock. Codex's writable workspace is the agent outbox (passed through `--cd`), and the signed file-mailbox fallback works when its sandbox denies local socket connections. Grok is given permission rules for the literal action command and outbox writes. Neither path permits a recipient/account override. Use `[SIGMA_NO_REPLY]` at the end of a tool-only final answer to suppress an additional quoted reply.

Agent bridges enforce sender authorization before starting adapters or providers. The incoming prompt includes sender/chat/message metadata and paths of attachments staged into the current outbox. Owner-only quoted `/approve` or `/assign` commands approve a person in that chat and answer the selected message; `/revoke` removes permission. The same policy is available through `agent-permissions` on the command line, even while a worker holds the account lock. See [agent permissions](agent-permissions.md).
