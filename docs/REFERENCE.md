<a id="reference-top"></a>

# Sigma — complete reference

Everything about the project in one document: getting started, commands, chat and app hooks, webxdc, state, output, backups, distribution, development, and verification.

Generated from the project documentation with `python3 scripts/build-reference.py`. Update the source guides, then regenerate this reference. Command-specific `--help` is the authority for supported arguments.

## Contents

- [🗿 sigma](#section-1)
- [Command reference](#section-2)
- [State and operations](#section-3)
- [Give sigma to an AI: create, send, and operate a webxdc app](#section-4)
- [Chat command hooks](#section-5)
- [App event hooks](#section-6)
- [Bot automation](#section-7)
- [Agent chat tools](#section-8)
- [Agent owner and sender permissions](#section-9)
- [Agent model selection](#section-10)
- [Experimental Delta Chat calls](#section-11)
- [Agent process isolation and database protection](#section-12)
- [Webxdc apps and shared updates](#section-13)
- [Webxdc realtime](#section-14)
- [Voice messages](#section-15)
- [JSON output](#section-16)
- [Backup and restoration](#section-17)
- [Build, install and release packaging](#section-18)
- [Architecture and development](#section-19)
- [Testing](#section-20)

<a id="section-1"></a>

# 🗿 sigma

[Website](https://omidz4t.github.io/sigma/) · [Download a release](https://github.com/omidz4t/sigma/releases) · [Source](https://github.com/omidz4t/sigma)

**All documentation in one place:** [complete reference](#reference-top), or open the [standalone browser reference](#reference-top).

A stateful Rust CLI for bots, calling [chatmail core](https://github.com/chatmail/core) directly. The original account lives in `~/.config/sigma`. Named accounts live in `~/.config/sigma-accounts/accounts/NAME`; select one with `--account NAME` or save a default with `account-default NAME`. `--data-path DIRECTORY` still selects an explicit account directory. Keys, credentials, messages and attachments stay private in each account directory.

```sh
./sigma init --name "My bot"
./sigma give-qr --output ./invite.svg
./sigma create-contact "https://i.delta.chat/#FULL_INVITATION"
./sigma send-message CONTACT_ID "Hello!"
./sigma send-file CONTACT_ID ./report.pdf --caption "Report"
./sigma send-file CONTACT_ID ./message.txt --as-text
./sigma send-photo CONTACT_ID ./photo.jpg
./sigma send-webxdc CONTACT_ID ./app.xdc
./sigma send-voice CONTACT_ID ./recording.ogg --duration 3500
./sigma receive
./sigma chats --json
./sigma read-messages CHAT_ID --mark-read
./sigma reply MESSAGE_ID "Thanks!"
./sigma react MESSAGE_ID '👍'
```

Replace the placeholders with returned IDs or full invitation links. Account provisioning and messaging require network access. `listen` runs continuous receiving and retries queued mail; stop it before another command uses the same account.

## Agent skill

Install the Sigma skill for your AI agent:

```sh
npx skills add omidz4t/sigma --skill sigma
```

The [Sigma skill](../skills/sigma/SKILL.md) teaches account setup, connecting with a Delta Chat invitation, sending messages/files, and operating hooks or shared webxdc apps. It checks the installed CLI's help for version compatibility and keeps private account data out of source control. [Browse on skills.sh](https://skills.sh/omidz4t/sigma/sigma).

## Webxdc apps operated by an AI

Create an app, send it once under a saved name, and control it without stopping its listener:

```sh
./sigma send-webxdc CONTACT_ID ./app.xdc --app demo --json
./sigma app-hook-add demo live ./examples/realtime-log-hook.py --type realtime
./sigma app-start demo --json
./sigma app-info demo --json
./sigma app-send demo '{"type":"ping"}' --json
./sigma app-update demo '{"payload":{"text":"Hello"}}' --json
./sigma app-events demo --after 0 --json
./sigma app-stop demo --json
```

`app-start` keeps a background worker online. Wait for `realtime_joined` before sending live packets. App identity, durable state and captured events are saved under `~/.config/sigma`; restart with the same name to continue. The recipient must open the same app instance. Hand another AI the [AI workflow guide](#section-4).

## Groups and channels

```sh
./sigma create-group "Team"
./sigma add-to-group GROUP_ID CONTACT_ID
./sigma send-message GROUP_ID "Hello team" --chat
./sigma send-photo GROUP_ID ./photo.jpg --chat
./sigma group-members GROUP_ID
./sigma group-link GROUP_ID
./sigma create-channel "Updates"
./sigma invite-to-channel CHANNEL_ID CONTACT_ID
./sigma send-channel-message CHANNEL_ID "News"
```

The file, photo, voice, text and webxdc send commands support `--chat` for numeric chat/group/channel IDs. Group management includes invitations, joining, member listing/removal, renaming and leaving. Channel management includes subscriber listing/removal, editing, joining and leaving. Channel subscriptions require recipient acceptance; run the owner with `listen` during joining.

## Message tools and bot automation

```sh
./sigma search-messages "meeting" --chat-id CHAT_ID --json
./sigma edit-message MESSAGE_ID "Corrected text"
./sigma forward-message MESSAGE_ID CHAT_ID
./sigma export-file MESSAGE_ID ./attachment.pdf
./sigma download-file MESSAGE_ID ./attachment.pdf
./sigma delete-message MESSAGE_ID
./sigma listen --auto-reply "Message received"
./sigma listen --on-message ./my-hook --hook-timeout 30 --json
```

Hooks receive a message JSON object on stdin and return optional reply text on stdout. Hooks run only when explicitly enabled. JSON Lines output is available with `--json` on every command.

## Accounts and local agents

```sh
./sigma account-create my-bot --name "My bot" --profile-photo ./avatar.png
./sigma account-list
./sigma account-default my-bot
./sigma status                         # uses the saved default
./sigma status --account legacy        # the original account
./sigma account-delete old-bot          # permanently remove a stopped, nondefault account
```

Create a dedicated agent account, connect it to your contact invitation, and keep it online in the background:

```sh
./sigma agent-connect codex --new-account codex-bot --name "Codex" \
  --profile-photo ./avatar.png --to me --invite "FULL_CONTACT_INVITATION" \
  --system-prompt "Give helpful, concise replies."
./sigma agent-list --account codex-bot
./sigma agent-stop codex --account codex-bot
# Restart the same account and saved alias:
./sigma agent-connect codex --account codex-bot --to me
./sigma create-contact me "FULL_CONTACT_INVITATION" --account my-bot
./sigma agent-connect grok --alias researcher --account my-bot --to me --model MODEL_ID
```

Each agent has a persistent alias (`--alias NAME`, default: provider name) and an automatic numeric ID, both scoped to its account. The first `--to` records the initial recipient and initializes the sole bot master. Use `/master` or `agent-master` for an explicit transfer; see [bot master management](#section-9). Restarts can omit `--to` and select the agent with `--alias NAME` or `--agent-id ID`; another recipient is refused. `agent-list` includes stopped identities, and `agent-stop` accepts either their alias or ID. To delegate access, approve the person; to transfer authority, change the bot master.

Native Grok and Codex reuse one persistent provider session per agent, so messages share conversation history across worker restarts. Existing chat history is imported once when the session is first created; later turns use the provider’s own history. `agent-list` reports the saved `session_id`. Native Grok and Codex support `--model`, `--system-prompt TEXT` and `--system-prompt-file PATH`. `--name` and `--profile-photo` set the chat account's profile. New accounts need their own contact handshake: `--invite LINK` saves the `--to me` alias in that account. Existing aliases remain account-local. Use `--exec ./adapter` for another local CLI, or `--foreground` for a foreground connection. See [account operations](#section-3) and [agent setup and limits](#section-7).

Agents also receive [private chat tools](#section-8) in their starting instructions. They can send voice notes, files, videos, captioned images and multiple standalone messages, react/edit within their bound chat, and change their own account name/photo. Per-worker signed authorization fixes the recipient and account; media is staged in a private agent outbox.

## Backup and restore

```sh
./sigma backup ./backups --passphrase-file ./backup-password
./sigma restore ./backups/BACKUP.tar --data-path ./restored --passphrase-file ./backup-password
```

Restoration requires a new or empty destination and refuses existing accounts. Core backup passphrases protect the database; attachment blobs in the TAR remain unencrypted. Without a passphrase file the database is also unencrypted.

## Build and install

Requires Rust 1.89 or newer, a C/C++ compiler, CMake, Perl, make and pkg-config. On Debian/Ubuntu:

```sh
sudo apt-get install build-essential cmake perl pkg-config
cargo build --locked --release
./target/release/sigma --help
scripts/install.sh --prefix "$HOME/.local"
```

`./sigma` is a Cargo launcher; the standalone binary is `target/release/sigma`. First builds compile core's substantial networking/encryption dependencies, including vendored OpenSSL and SQLCipher. Direct dependencies are chatmail core, Tokio, UUID, HMAC and SHA-256 (for persistent sessions and signed agent actions); argument parsing, JSON output and hooks require no CLI framework or additional crates. Core's revision and Cargo.lock are pinned.

Every command has a handler and its own help definition under `src/handlers/`:

```sh
./sigma --help
./sigma send-photo --help
./sigma help backup
```

## Documentation

- [AI webxdc workflow](#section-4)
- [Command reference](#section-2)
- [State and account operations](#section-3)
- [JSON output](#section-16)
- [Bot hooks and automatic replies](#section-7)
- [Backup and restoration](#section-17)
- [Build, installation and releases](#section-18)
- [Testing and live checks](#section-20)
- [Architecture and development](#section-19)

GitHub CI checks formatting, Clippy, tests and release compilation. The manual release workflow builds Linux/macOS archives with checksums and publishes only when explicitly dispatched for an existing tag. Sigma is licensed under [MIT](../LICENSE). Chatmail core remains MPL-2.0; see [third-party notices](../THIRD_PARTY_NOTICES.md).

### Webxdc state and button events

Send apps with `send-webxdc`, queue shared state with `send-webxdc-update`, read it with `webxdc-updates`, and receive button events with `listen-webxdc-updates`. See the [webxdc guide](#section-13) and included `examples/button-app.xdc`.

Use `send-webxdc-realtime` for ephemeral text/binary packets and `listen-webxdc-realtime` to receive live button events. The listener supports `--send` for sending and receiving in one session. See the [realtime guide](#section-14) and `examples/realtime-app.xdc`.

Send local recordings as voice notes with `send-voice`. See the [voice guide](#section-15).

Change photos with `set-profile-photo PATH`, `set-group-photo GROUP_ID PATH` and `set-channel-photo CHANNEL_ID PATH`. Pass a quoted empty string to clear a photo. Use `send-photo` for a photo message.

Run a defined executable on app button events with `app-start NAME --on-event ./hook.py`. Hook settings and outcomes persist with the app. See the [app hook guide](#section-6) and included `examples/app-hook.py` / `examples/hook-app.xdc`.

Register multiple filtered app hooks with `app-hook-add NAME HOOK_ID EXECUTABLE --type realtime --match '{"type":"button_clicked"}'`. Use `app-hook-list NAME` and `app-hook-remove NAME HOOK_ID` while running or stopped. See the [hook guide](#section-6).

App hooks require an explicit `--type realtime|update|message|command`; command hooks can select `--command /ping`. Workers join realtime only while a realtime hook (or legacy broad hook) exists. See [typed hooks](#section-6).

For slash commands directly on a chat without an app, use `chat-hook-add CHAT_ID ID EXECUTABLE --command /ping` and run `listen`. See [chat command hooks](#section-5).

Example app archives and test media are generated from text source; run `python3 scripts/prepare-assets.py` before using example `.xdc` files. `cargo test` generates its own fixtures automatically. No compiled binaries or account credentials are committed.

Agent bridges enforce [owner-controlled sender permissions](#section-9), with owner-only `/approve`, `/assign`, `/revoke` replies and online/offline `agent-permissions` CLI management. Groups retain separate sessions and scoped media/Webxdc tools.

Owners can [list and switch agent models](#section-10) using `/models` and `/model MODEL_ID` in chat, or `agent-models` and `agent-model` from the CLI, while preserving conversation history.

Native Codex bridges support [experimental audio calls](#section-11) restricted to the bot master. Agent processes use [Linux OS isolation](#section-12) to protect Sigma databases and control state; no new Cargo dependencies were added. Grok CLI live calls are not currently enabled.

Native Codex bridges can opt in to outbound audio calls with `--allow-outbound-calls`. The bot master can share a contact and ask the agent to call it. This requires installed Chromium; see [outbound call tools and security](#section-11).


---

<a id="section-2"></a>

# Command reference

`sigma COMMAND [ARGS] [OPTIONS]`. `sigma --help` lists everything; `sigma COMMAND --help` / `-h` and `sigma help COMMAND` show the handler's usage, description, supported options and example. Help/version need no account or network access.

Account-based commands support `--account NAME` (saved default, initially `legacy`) or `--data-path DIRECTORY` (explicit directory), plus `--json`. The two account selectors are mutually exclusive. Account management commands operate on the named-account registry under absolute `HOME` and support `--json`, without either selector. Explicit relative data paths are relative to the working directory. Value options accept `--key value` or `--key=value`; flags take no value. Options can appear before or after positional arguments. Unknown, repeated, missing or command-inappropriate options fail. Use `--` before literal arguments starting with a dash. Quote shell text, paths containing spaces and invitation links.

| Command | Arguments | Additional options | Purpose |
| --- | --- | --- | --- |
| `account-create` | `NAME` | `--name TEXT`, `--profile-photo PATH`, `--account-qr QR` | Provision a new named account without changing the default |
| `account-list` | none | none | List named accounts, the original legacy account and the selected default |
| `account-default` | `NAME` | none | Save the account used when no selector is supplied |
| `account-delete` | `NAME` | none | Permanently delete a stopped, nondefault local account |
| `agent-connect` | `PROVIDER` | `--alias NAME`, `--agent-id ID`, `--to TARGET` (required for new identities), `--new-account NAME`, `--invite LINK`, `--name TEXT`, `--profile-photo PATH`, `--account-qr QR`, `--exec PATH`, `--model MODEL_ID`, `--system-prompt TEXT`, `--system-prompt-file PATH`, `--allow-outbound-calls` (native Codex), `--foreground`, `--hook-timeout SEC` | Start a background agent bridge; model and prompt options require native Grok/Codex |
| `agent-act` | `ACTION ARGS...` | `--caption TEXT`, `--duration MS`, `--request-id UUID` | Use the current agent’s signed, bound-chat media/message/profile tools; no account/recipient selectors |
| `agent-calls` | `ID_OR_ALIAS [COUNT]` | Account options, `--json` | List account call history and agent request failures, with duration |
| `agent-call-cancel` | `ID_OR_ALIAS CALL_ID_OR_REQUEST_UUID_OR_all` | Account options, `--json` | Cancel pending requests or end calls without deleting history |
| `agent-list` | none | none | List saved agent IDs, aliases, recipient bindings and running/stopped state |
| `agent-models` | `ID_OR_ALIAS` | none | List the native provider’s cached model IDs and names |
| `agent-model` | `ID_OR_ALIAS [MODEL_ID/default]` | none | Show or persist the bot-wide model selection for its next turn |
| `agent-master` | `ID_OR_ALIAS [CONTACT_ID]` | none | Show or transfer the sole bot master; clear delegated approvals on transfer |
| `agent-permissions` | `ID_OR_ALIAS list` or `ID_OR_ALIAS approve/assign/revoke CHAT_ID CONTACT_ID` | none | Manage sender permissions online or stopped; ownership remains permanent |
| `agent-stop` | `ID_OR_ALIAS` | none | Stop the selected agent bridge and cancel its active invocation |
| `group-members` | `GROUP_ID` | none | List locally stored group members with contact IDs and names |
| `remove-from-group` | `GROUP_ID CONTACT_ID` | none | Remove a regular contact from the group |
| `rename-group` | `GROUP_ID NAME` | none | Rename a group containing/owned by this account |
| `channel-members` | `CHANNEL_ID` | none | List locally stored channel members with contact IDs and names |
| `remove-from-channel` | `CHANNEL_ID CONTACT_ID` | none | Remove a regular contact from the channel |
| `rename-channel` | `CHANNEL_ID NAME` | none | Rename a channel containing/owned by this account |
| `set-channel-description` | `CHANNEL_ID TEXT` | none | Set the owned channel description |
| `set-channel-photo` | `CHANNEL_ID PATH` | none | Set the owned channel photo |
| `leave-group` | `GROUP_ID` | none | Leave this group through core |
| `leave-channel` | `CHANNEL_ID` | none | Leave this channel through core |
| `group-link` | `GROUP_ID` | none | Print a secure-join invitation for a group you belong to |
| `invite-to-group` | `GROUP_ID CONTACT_ID` | none | Send a group invitation link to an existing contact |
| `join-group` | `INVITATION` | `--timeout` | Join a full group invitation and print the resulting local chat ID after the handshake |
| `join-channel` | `INVITATION` | `--timeout` | Join a full channel invitation and print the resulting local chat ID after the handshake |
| `search-messages` | `QUERY` | `--chat-id` | Search stored messages with core |
| `delete-message` | `MESSAGE_ID` | `--for-all` | Delete a stored message through core, including synchronized deletion on your devices/server |
| `edit-message` | `MESSAGE_ID TEXT` | none | Edit the text of your own stored message |
| `forward-message` | `MESSAGE_ID CHAT_ID` | none | Forward a stored message to an explicit destination chat ID, including groups |
| `export-file` | `MESSAGE_ID PATH` | none | Export an already downloaded attachment to a new local path |
| `download-file` | `MESSAGE_ID PATH` | `--timeout` | Download a partial message if necessary, then export its attachment to a new path |
| `backup` | `DIRECTORY` | `--passphrase-file` | Export a core backup containing keys, credentials, chats, messages and blobs |
| `restore` | `BACKUP_FILE` | `--passphrase-file` | Restore a core backup into a NEW/empty account directory |
| `init` | `` | `--name`, `--profile-photo-address`, `--account-qr` | Create a bot identity and provision a chatmail account |
| `status` | `` | none | Show account status |
| `add-contact` | `ADDRESS` | `--name` | Add a contact; print its ID |
| `create-contact` | `SECUREJOIN_ID` | `--timeout` | Join a contact invitation; print its contact ID |
| `securejoin-id` | `` | none | Print this bot's secure-join invitation |
| `join-link` | `` | none | Print this bot's join link (alias: securejoin-id) |
| `give-qr` | `` | `--output` | Save a QR SVG and print its join link |
| `contacts` | `` | none | List contacts (ID, name, address) |
| `send-message` | `TARGET TEXT` | `--chat`, `--timeout` | Send to a contact ID or secure-join invitation |
| `reply` | `MESSAGE_ID TEXT` | none | Send a quoted reply to a message in its chat |
| `send-file` | `TARGET FILE` | `--chat`, `--caption`, `--as-text`, `--timeout` | Send a local file to a contact ID or invitation |
| `send-voice` | `TARGET AUDIO_FILE` | `--chat`, `--caption`, `--duration`, `--timeout` | Send a recording as a voice message |
| `send-photo` | `TARGET PHOTO` | `--chat`, `--caption`, `--timeout` | Send an image as a photo message |
| `send-webxdc` | `TARGET APP.xdc` | `--chat`, `--caption`, `--timeout` | Send a webxdc app to a contact or chat |
| `send-webxdc-update` | `MESSAGE_ID UPDATE_JSON` | | Queue app state |
| `send-webxdc-realtime` | `MESSAGE_ID DATA` | `--file`, `--timeout` | Send live text or binary bytes |
| `listen-webxdc-realtime` | `MESSAGE_ID` | `--send`, `--send-file`, `--timeout` | Join and receive live app packets |
| `webxdc-updates` | `MESSAGE_ID` | `--after` | Read stored app state |
| `listen-webxdc-updates` | `MESSAGE_ID` | `--after` | Receive app updates and button events |
| `create-channel` | `NAME` | none | Create a broadcast channel; print its chat ID |
| `create-group` | `NAME` | none | Create an encrypted group; print its chat ID |
| `add-to-group` | `GROUP_ID CONTACT_ID` | none | Add an existing contact to a group |
| `channel-link` | `CHANNEL_ID` | none | Print a channel subscription invitation |
| `invite-to-channel` | `CHANNEL_ID CONTACT_ID` | none | Send a subscription invitation |
| `send-channel-message` | `CHANNEL_ID TEXT` | none | Post to your broadcast channel |
| `chats` | `` | none | List chats (ID, name) |
| `messages` | `CHAT_ID` | none | Read locally stored messages |
| `receive` | `` | `--timeout` | Fetch mail once and print unread messages |
| `read-messages` | `CHAT_ID` | `--mark-read` | Read stored history; optionally --mark-read |
| `react` | `MESSAGE_ID [EMOJI]` | `--timeout` | Set/replace a reaction; omit emoji to remove it |
| `reactions` | `MESSAGE_ID` | none | Show stored reactions for a message |
| `listen` | `` | `--auto-reply`, `--on-message`, `--hook-timeout` | Receive and print messages until Ctrl-C |
| `help` | `[COMMAND]` | none | Show general or command-specific help |

## Identities and invitations

```sh
sigma init --name 'Helper' --profile-photo-address ./avatar.png
sigma status
sigma join-link
sigma give-qr --output ./invite.svg
sigma create-contact 'https://i.delta.chat/#FULL_INVITATION'
```

`init` enables bot mode, stores supplied profile fields and provisions a relay using core. Repeating it retains the identity and an already configured transport. Omitted name/photo options preserve existing values. `--profile-photo-address` accepts a local file, not a URL. `--account-qr` passes a provider provisioning QR to core. Provisioning may leave partial state on failure; retry with the same account path.

`SECUREJOIN_ID` means a full contact QR payload/invitation, not just its fingerprint or invite token. Modern `https://i.delta.chat/#...` / `https://i.delta.chat#...` and legacy `OPENPGP4FPR:...` forms are recognized, then validated by core. `create-contact` accepts contact invitations only; `join-group` and `join-channel` accept their corresponding invitation types. Joining runs networking until core reports handshake completion, times out (default 120 seconds) or is interrupted. Stop the inviter's other CLI commands and run its `listen` while joining. On timeout, partial state remains; `listen` can continue pending work.

`securejoin-id` and `join-link` are aliases. `give-qr` uses core to render the same printed contact invitation into an SVG; default `DATA_PATH/join.svg`. Parent directories must exist, and existing files are refused. Open the SVG in a browser/image viewer to scan it. Links and QR images contain invitation credentials; share them intentionally.

## Sending, replying and reactions

```sh
sigma send-message CONTACT_ID 'Hello'
sigma send-file CONTACT_ID ./report.pdf --caption 'Report'
sigma send-file CONTACT_ID ./message.txt --as-text
sigma send-photo CONTACT_ID ./photo.jpg --caption 'Hello!'
sigma send-webxdc CONTACT_ID ./app.xdc
sigma send-message CHAT_ID 'Hello group' --chat
sigma send-file CHAT_ID ./report.pdf --chat
sigma send-photo CHAT_ID ./photo.jpg --chat
sigma send-webxdc CHAT_ID ./app.xdc --chat
sigma reply MESSAGE_ID 'Thanks!'
sigma react MESSAGE_ID '👍'
sigma reactions MESSAGE_ID
sigma react MESSAGE_ID
```

Without `--chat`, the four `send-*` commands take a numeric contact ID or a full contact invitation. With `--chat`, their target must be a numeric existing chat ID, including groups and owned channels. Contact IDs and chat IDs are distinct. Core rejects read-only or otherwise non-sendable chats. `--timeout` limits secure join, not SMTP submission.

`send-file` copies a regular local file into core's blob storage. `--as-text` reads UTF-8 and sends one ordinary text message instead of an attachment. Empty/whitespace-only and invalid UTF-8 files fail. Text is preserved when passed to core; transport/recipient processing can normalize terminal line breaks. `--caption` cannot be combined with `--as-text`. `send-photo` uses core's image message type, leaving supported formats, resizing and GIF detection to core. `send-webxdc` requires a local `.xdc` ZIP with a readable `index.html`, checks it through core before joining/sending, and uses the webxdc message type. It sends app instances, not app status updates or realtime events. No JavaScript is executed by this CLI.

Use source files outside core's blob directory: deduplication may rename files already inside it. Attachment preparation failures can leave copied blobs. All sending uses core's transport/cryptography; no parallel protocol implementation exists in sigma.

`reply` loads a stored message, sets its quote through core, and submits to the same chat. Group replies go to the group. Core supplies media summaries for messages without text and preserves quote encryption requirements. Quoting a file does not resend its bytes. Replies do not mark the original read.

`react` replaces your existing reaction; omitting the emoji or passing a quoted empty string retracts it. Core accepts one token shorter than 30 UTF-8 bytes, and the CLI rejects inputs core would truncate. Channel reaction restrictions apply. It waits for outgoing submission with a default 120-second timeout. `reactions` displays local counts; run receive/listen to synchronize.

Successful synchronous sends confirm SMTP submission, not remote delivery. Core may leave a failed/timed-out send queued. Use `listen` before repeating the send to avoid duplicates. Commands explicitly reporting “queued” or “updated locally” require `listen` to transmit their pending updates.

## Groups and channels

```sh
sigma create-group 'Team'
sigma add-to-group GROUP_ID CONTACT_ID
sigma group-members GROUP_ID
sigma rename-group GROUP_ID 'New team'
sigma group-link GROUP_ID
sigma invite-to-group GROUP_ID CONTACT_ID
sigma join-group 'https://i.delta.chat/#GROUP_INVITATION'
sigma remove-from-group GROUP_ID CONTACT_ID
sigma leave-group GROUP_ID

sigma create-channel 'News'
sigma channel-link CHANNEL_ID
sigma invite-to-channel CHANNEL_ID CONTACT_ID
sigma join-channel 'https://i.delta.chat/#CHANNEL_INVITATION'
sigma channel-members CHANNEL_ID
sigma rename-channel CHANNEL_ID 'New news'
sigma set-channel-description CHANNEL_ID 'Project updates'
sigma set-channel-photo CHANNEL_ID ./avatar.png
sigma send-channel-message CHANNEL_ID 'New release'
sigma remove-from-channel CHANNEL_ID CONTACT_ID
sigma leave-channel INCOMING_CHANNEL_ID
```

New groups/channels initially have no external members. Encrypted groups require key contacts; use `create-contact` before `add-to-group`, rather than only adding an address. Adding an existing group member is idempotent. Group management requires current self-membership; channel editing/subscriber management requires an owned outgoing channel. Lists are local snapshots. Group members can converse; channel subscribers receive read-only posts and do not see other subscribers.

Channel subscriptions must happen through core's invitation handshake; no public API is used to silently add a subscriber. `invite-to-group`/`invite-to-channel` deliver invitation links as direct messages, while the recipient explicitly joins. Keep the inviting account running with `listen`. `group-link`/`channel-link` let you share links yourself. Removing a member/subscriber and renaming/editing can queue system updates. A new group is unpromoted until core sends its first group message. Historical delivery to new subscribers is not guaranteed.

Pass a quoted empty string to `set-channel-description` or `set-channel-photo` to clear it. `leave-group` removes self from a group; `leave-channel` unsubscribes from an incoming channel. Neither deletes local history. Owned channels cannot be left as if subscribed. Group/chat type checks prevent accidentally applying these commands to a contact ID.

## Receiving, history and message tools

```sh
sigma receive --timeout 120
sigma chats --json
sigma read-messages CHAT_ID
sigma read-messages CHAT_ID --mark-read
sigma search-messages 'meeting' --chat-id CHAT_ID
sigma edit-message MESSAGE_ID 'Corrected text'
sigma forward-message MESSAGE_ID DESTINATION_CHAT_ID
sigma export-file MESSAGE_ID ./attachment.pdf
sigma download-file MESSAGE_ID ./attachment.pdf --timeout 120
sigma delete-message MESSAGE_ID
sigma delete-message OWN_ENCRYPTED_MESSAGE_ID --for-all
```

`receive` performs one core `background_fetch` round then prints all fresh/unread messages, including old unread messages. It can return after one transport fetched mail, so it does not guarantee synchronization of every transport/history item. Core excludes muted/blocked chats, blocked contacts and contact requests from this unread list. It does not wait for outgoing delivery. `listen` keeps networking running, prints incoming message contents and core events, and retries outgoing work until Ctrl-C.

`read-messages` displays stored history without fetching; `--mark-read` explicitly calls core's read API after output. Pending read receipts require networking. `messages` retains the earlier compact format in human mode; detailed JSON includes the shared message object. History includes sender, text, timestamp, type, state, local attachment path and reaction summary. `search-messages` uses core's local search and optionally limits to one chat. Message, contact and chat IDs are local to the selected account and not interchangeable.

`edit-message` changes your own message text through core and queues an edit request; core enforces text/info/encryption restrictions. `forward-message` takes an explicit destination CHAT_ID, verifies sendability and queues a forward. `delete-message` invokes core's deletion across your devices/server; `--for-all` additionally requests recipient deletion where core permits it (own encrypted messages). Do not assume remote propagation before running networking.

`export-file` copies an already available attachment. `download-file` schedules a full message download when partial, runs networking with timeout/Ctrl-C handling, and then exports its attachment. Existing destination files are never overwritten. Core/server availability and decryption determine whether downloads are possible. A timed-out download can continue later through `listen`.

## Automation, JSON, backups and releases

- [Automation](#section-7): explicit fixed replies and executable message hooks with bounded time/output.
- [JSON output](#section-16): record shapes and scripting contract.
- [Backup/restore](#section-17): encrypted archives and refusal to overwrite an existing account.
- [Build/install/releases](#section-18): local install, archive/checksum creation, manually dispatched hosted releases.
- [Testing](#section-20): offline checks and explicit isolated live integration testing.

Exit status is 0 on success/help/version and 1 for invalid arguments, state, filesystem, core or network errors. Per-account locking prevents multiple commands from concurrently using the same state. Stop `listen` before another command operates on its account. No command pushes repository changes.

## Webxdc shared state

`send-webxdc-update MESSAGE_ID UPDATE_JSON` queues state for an app instance. `webxdc-updates MESSAGE_ID [--after SERIAL]` reads stored updates. `listen-webxdc-updates MESSAGE_ID [--after SERIAL]` replays and listens for updates, including button events emitted by the app. See [webxdc.md](#section-13) for a complete workflow and runnable example.

`send-webxdc-realtime` sends ephemeral text or binary bytes; `listen-webxdc-realtime` joins an app channel and receives live packets, optionally sending initial text or a file. Both peers must be connected. See [webxdc-realtime.md](#section-14).

See [voice.md](#section-15) for sending voice notes with optional duration metadata.

## Named app workers for AI tools

Use `send-webxdc TARGET FILE --app NAME` to save an app identity. `app-bind NAME MESSAGE_ID` registers a previously sent/received app. `app-list` and `app-info NAME` inspect saved apps. `app-start NAME` starts a background worker; `app-run NAME` runs it in the foreground. While it is running, `app-send NAME DATA [--file]` sends live data and `app-update NAME UPDATE_JSON` sends durable state. `app-events NAME [--after SEQ]` reads the persisted event journal even while stopped. `app-stop NAME` leaves the channel and releases the core account lock. All support `--json` and `--data-path`; see [AI.md](#section-4) for the complete workflow and limitations.

## Profile, group and channel photos

`set-profile-photo PATH` changes the bot avatar without provisioning an account. `set-group-photo GROUP_ID PATH` and `set-channel-photo CHANNEL_ID PATH` change chat icons through core. Pass a quoted empty string to clear a photo. Run networking (`listen` or an app worker) to transmit queued chat-photo changes; the bot avatar is shared with contacts by outgoing messages. `status --json` exposes the bot `profile_photo`; `chats --json` exposes each chat’s `profile_photo`.

`app-start` and `app-run` accept `--on-event PATH`, `--hook-timeout SEC` and `--no-hook` to save or clear an executable app-event hook. See [app-hooks.md](#section-6).

`app-hook-add NAME HOOK_ID EXECUTABLE --type realtime|update|message|command [--command /NAME] [--match JSON] [--hook-timeout SEC]` adds a named event-specific hook. `app-hook-list NAME` lists all definitions, and `app-hook-remove NAME HOOK_ID` removes one without stopping other hooks. They support live workers and stopped apps. See [app-hooks.md](#section-6).

Named workers join realtime only when a realtime hook (or legacy `all` hook) exists. Adding the first joins; removing the last leaves. Update, message and command hooks continue with mail networking only. Message/command hooks are scoped to the app chat.

## Chat command hooks

`chat-hook-add CHAT_ID HOOK_ID EXECUTABLE --command /NAME [--hook-timeout SEC]` saves a chat-specific slash-command hook. `chat-hook-list CHAT_ID` lists definitions; `chat-hook-remove CHAT_ID HOOK_ID` removes one. Run `listen` to execute them, without any webxdc app. Stop workers/listeners before editing. See [chat-hooks.md](#section-5).

### Contact aliases

```sh
./sigma create-contact me "https://i.delta.chat/#FULL_INVITATION"
./sigma send-message me "Hello!"
./sigma send-file me ./report.pdf
./sigma send-photo me ./photo.jpg
./sigma send-voice me ./recording.ogg
./sigma send-webxdc me ./app.xdc --app demo
```

The alias is saved only after secure join completes. The one-argument `create-contact INVITE_ID` form still works. Aliases are case-sensitive, start with an ASCII letter, and contain up to 64 letters, digits, underscores or hyphens. An existing alias cannot be overwritten. Unknown aliases fail without sending. `--chat` continues to require a numeric chat ID. `send-file --as-text` also accepts an alias.

Aliases persist in `contact-aliases.tsv` inside the selected account directory (`~/.config/sigma` by default), protected by the account lock. They retain both contact ID and address to detect stale mappings. Use the same `--data-path` when sending. This sidecar is private and is not included in core's encrypted database backup; preserve it separately when moving your account. Aliases do not change contact display names.

### Default bot profile

`./sigma init` creates new accounts with the display name **Sigma** and the right-facing Sigma Moai logo as the profile photo. The image is embedded in the executable, so it needs no download or source checkout. Override these defaults with `--name` and `--profile-photo-address`. Repeating `init` preserves an existing profile unless an explicit override is supplied.


---

<a id="section-3"></a>

# State and operations

The original account is named `legacy` and remains in `~/.config/sigma`. Named accounts live in `~/.config/sigma-accounts/accounts/NAME`. Account selection is shared across working directories and resolved from absolute `HOME`.

```sh
sigma account-create work --name "Work bot" --profile-photo ./avatar.png
sigma account-list --json
sigma status --account work
sigma account-default work
sigma status                        # now uses work
sigma account-default legacy        # return to the original account
sigma account-delete work           # permanently delete its local data
```

`--account NAME` overrides the saved default for every account-based command, including agent/app management, backups and restore. `--data-path PATH` selects an explicit directory and bypasses the named-account selection; it cannot be combined with `--account`. Without either option, Sigma reads `~/.config/sigma-accounts/default-account`, initially falling back to `legacy`. An invalid saved selection fails rather than silently opening another identity. Changing the default affects future commands; running workers keep their account.

`account-create NAME` reserves a new private account directory, provisions it through core and optionally sets `--name`, `--profile-photo PATH` and `--account-qr QR`. It refuses existing names and never changes the saved default. On failure the partially created identity is preserved; retry with `init --account NAME`. `init --account NAME` can also initialize a named account directly. Account names follow the same ASCII naming rules as contact aliases. `legacy` is reserved for the original account.

`account-list` lists the legacy identity when its database exists and all managed named directories, marking the default and reporting whether a database exists. It works while workers are running. Arbitrary `--data-path` accounts are not automatically registered or enumerated. Account management commands take the name as a positional argument and do not accept `--account` or `--data-path`.

Each data path contains `account.db`, core-managed SQLite sidecars and blob files, and `sigma.lock`. Keep the entire directory together. Core owns database migrations and storage; sigma does not implement a second database or account format. There is no application-level database encryption configured. Treat the directory as private: it contains private keys, transport credentials, and message history. On Unix, init sets directory permissions to 0700.

Use a dedicated directory per bot. Messaging commands require an existing database; account/worker listing and account management do not open that database. Creation uses `init` or `account-create`. Failed init can leave a partially initialized account; fix the error and repeat init using the same path. Status reports whether transport configuration completed.

An operating-system file lock allows one sigma process per account. A lock error means another process is using that path. The lock releases automatically on exit, including crashes; the lock file itself remains. Do not remove it while a process is running. Other applications that open core databases do not necessarily honor this CLI lock.

`listen` keeps networking active and prints core events. It receives messages and retries outgoing messages. Automatic replies require explicit `--auto-reply` or `--on-message` options; ordinary listen does not reply. Stop it before using another command with that account, then restart it. `messages` and `read-messages` read stored history without fetching. `receive` runs one bounded core fetch and prints fresh/unread messages without marking them read. `read-messages --mark-read` explicitly acknowledges the displayed history. Ctrl-C stops core networking before exit.

Back up the entire account directory with sigma stopped. Restore into a private directory and pass that directory with `--data-path`. Never commit account directories. The default `~/.config/sigma` is outside the project repository. Legacy `.delta` and `.sigma` directories remain ignored by Git; custom directories must be outside the repository or added to your local ignore rules.

`account-delete NAME` permanently removes that account's complete local directory, including keys, credentials, message history, attachments, aliases, app definitions and worker logs. It refuses the currently selected default, accounts held by another Sigma process and symlinked account directories. Stop workers and select another default before deleting. Deleting the original `legacy` directory does not affect the separate managed-account directory. Partially created named accounts can also be removed. Deletion removes local state, not remote mailboxes or copies held by contacts. No real accounts are deleted by the test suite.

## Migrating from the previous project name

Sigma uses `~/.config/sigma` instead of the former `~/.config/delta`. Stop every old listener or app worker before switching. Move the **entire** old account directory only if the new destination does not exist; never merge databases or blob directories. Keys, messages, contacts, named apps, hooks and captured events stay together. The CLI does not automatically move directories.

```sh
# After stopping the old CLI and checking that the destination is absent:
test ! -e "$HOME/.config/sigma" && mv "$HOME/.config/delta" "$HOME/.config/sigma"
./sigma status
```

Alternatively, use `./sigma --data-path "$HOME/.config/delta" status` to keep an existing account at its old location. The same applies to a former project-local `.delta` account. Do not run the old and new executables simultaneously against one account: their lock filenames differ. Keep registered hook executables at their saved paths, or remove and re-add their definitions after moving the checkout.

Core backup/export and import are exposed as `backup` and `restore`; see [backups.md](#section-17). `restore` refuses existing/nonempty accounts. Message edits, deletions, forwarding, membership changes and hook replies may queue work that is persisted by core and later transmitted by `listen`. JSON output changes presentation only and never changes persistence or enables automation.

Named app metadata and event journals are saved in `apps/NAME/` under the same account directory. A background worker owns the core lock while app control/read commands use private local IPC or read the journal. See [AI.md](#section-4) for exact state, cursors, restart and backup behavior.

Contact aliases created by `create-contact ALIAS INVITE_ID` are stored in the private account-local `contact-aliases.tsv` sidecar. They survive CLI restarts and resolve to a validated contact ID/address pair. Moving only the core database or using core backup/restore does not move this sidecar; preserve it separately if aliases are needed on the destination.


## Dedicated agent accounts

`agent-connect PROVIDER --new-account NAME --name TEXT --profile-photo PATH --to ALIAS --invite LINK` provisions a fresh named account before starting its worker. It does not change the default. The invitation handshake creates a new contact and saves ALIAS in that account; aliases and numeric contact IDs are never copied between identities. Alternatively pass the full invitation directly as `--to LINK`. To use an existing account, replace `--new-account NAME` with `--account NAME` and use an alias already saved there (or join a new invitation). Name/photo options can also update an existing agent account when connecting.


Agent identities are saved under `agents/ALIAS/agent.json` within the account, with stable numeric IDs reserved by `agents/next-id`. Their first recipient binding survives stopping, account moves and worker restarts; keep these sidecars with the account when backing it up. Agent aliases, contact aliases and account names are separate namespaces. IDs are account-local. Reassigning a contact alias cannot reassign an agent; select a saved agent by ID or alias to restart it with its stored recipient. Account deletion also removes these identities and bindings.


Native agent `session.json` files point to a single persistent provider conversation. Keep them alongside agent bindings during backup/moves. Actual native conversation history is managed by the provider CLI in its own storage; copying only Sigma account files to another machine does not copy that history. Stops and restarts retain the pointer. Missing provider history or interrupted first-time session creation fails explicitly rather than selecting another conversation.


Agent `outbox/` directories hold generated/staged assets and private action transport files. `action-results/` holds signed-action request/result history for safe retries; keep it with the account when migrating. The capability itself is generated per provider turn and held in memory/environment, so finishing a turn invalidates its capability. Account deletion removes these local files with the rest of that identity.

Owner-managed `agents/ALIAS/approvals.json` stores chat/contact fingerprint approvals. Additional chats keep provider sessions, outboxes and action-result caches under `agents/ALIAS/chat-CHAT_ID/`. Preserve these sidecars when migrating. See [agent permissions](#section-9).

Bot-wide model selection is persisted in `agents/ALIAS/model.json`; existing startup flags initialize it only when absent. Preserve this sidecar with the account. Native sessions keep their IDs when switching. A `custom-adapter` marker identifies bridges whose external adapter manages its own models. See [agent models](#section-10).

`master.json` records the current sole bot master; `agent.json` retains the first recipient as provenance. Additional master-private sessions are keyed by chat ID and do not automatically import previous master-private history. Each conversation’s `provider-home/` contains isolated native provider runtime/auth/session copies; preserve it with `session.json` for resumptions. The database format is unchanged: [OS process isolation](#section-12) protects it from agents. See [calls](#section-11) and [master transfers](#section-9).


---

<a id="section-4"></a>

# Give sigma to an AI: create, send, and operate a webxdc app

A webxdc is an offline HTML/CSS/JavaScript mini app packaged as an `.xdc` ZIP and opened inside Delta Chat. It can share state through chat without a hosted backend. Read the [short webxdc explanation](#section-13) before packaging an app.

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

Both reach the app's status-update listener. Client notification settings still apply. `info` and `notify` are independent, as explained in [webxdc.md](#section-13).

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

You can send `examples/realtime-app.xdc` directly to try this workflow. It returns a pong for a JSON ping and has a button for live clicks. `examples/button-app.xdc` demonstrates durable status updates. See the [official webxdc API](https://webxdc.org/docs/spec/api.html) for the app-side API and [realtime guide](#section-14) for binary packets and transport limits.

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

Start the app with `app-start NAME --on-event ./hook.py --hook-timeout 10`. The saved hook receives realtime and status-update events as JSON, can return realtime/status-update replies, and retains its configuration across worker restarts. See [app-hooks.md](#section-6) for the exact protocol, filtering and at-least-once behavior.

For event-specific handlers, use `app-hook-add NAME HOOK_ID EXECUTABLE --type realtime --match '{"type":"button_clicked"}'`. Register more than one ID for independent handlers, inspect them with `app-hook-list NAME --json`, and remove one with `app-hook-remove NAME HOOK_ID`. No worker stop is required for these commands. The old `--on-event` remains the broad `default` hook; remove that ID if you only want filtered hooks.

Workers with only `--type update`, `message`, or `command` hooks do not join realtime. Message/command hooks are scoped to the app chat; use `--type command --command /ping` and return `{"reply":"pong"}` for a quoted chat reply. See the hook guide for exact input fields.

For slash commands directly on a chat without an app, use `chat-hook-add CHAT_ID ID EXECUTABLE --command /ping` and run `listen`. See [chat command hooks](#section-5).


---

<a id="section-5"></a>

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


---

<a id="section-6"></a>

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

For slash commands directly on a chat without an app, use `chat-hook-add CHAT_ID ID EXECUTABLE --command /ping` and run `listen`. See [chat command hooks](#section-5).


---

<a id="section-7"></a>

# Bot automation

Networking is enabled by `listen`. Automatic replies are opt-in:

```sh
sigma listen --auto-reply 'Thanks, your message was received.'
sigma listen --on-message ./handler.py --hook-timeout 10 --json
```

Choose either a fixed `--auto-reply` or an executable `--on-message` hook. They cannot be combined. `--hook-timeout` requires a hook and defaults to 30 seconds. The same account lock covers the listener, so the hook must not invoke another sigma process against this account directory. Instead, write the reply to stdout.

The executable is invoked directly, without a shell or extra arguments. Use an executable file with an appropriate shebang; paths with spaces work when quoted. One message JSON object and a newline are written to stdin. The object is described in [output.md](#section-16). UTF-8 stdout becomes a quoted text reply to that message in the same chat. Empty/whitespace-only stdout sends no reply. Stderr is inherited for diagnostics.

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

Native agents now receive the [agent chat tool guide](#section-8) in their starting instructions, with a per-worker capability available only through their process environment. `agent-act` performs media, standalone messages, reactions, edits and own-account profile changes through the running worker, so it does not contend for the account lock. Codex's writable workspace is the agent outbox (passed through `--cd`), and the signed file-mailbox fallback works when its sandbox denies local socket connections. Grok is given permission rules for the literal action command and outbox writes. Neither path permits a recipient/account override. Use `[SIGMA_NO_REPLY]` at the end of a tool-only final answer to suppress an additional quoted reply.

Agent bridges enforce sender authorization before starting adapters or providers. The incoming prompt includes sender/chat/message metadata and paths of attachments staged into the current outbox. Owner-only quoted `/approve` or `/assign` commands approve a person in that chat and answer the selected message; `/revoke` removes permission. The same policy is available through `agent-permissions` on the command line, even while a worker holds the account lock. See [agent permissions](#section-9).


---

<a id="section-8"></a>

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

A private Unix socket handles normal calls. If the socket is unavailable or a provider sandbox refuses it, the same CLI exchanges signed requests through a private `.sigma-inbox` mailbox under the outbox. The mailbox contains signatures, not the raw capability, and uses the same request/result deduplication and authorization as the socket. Other agents' tokens fail on both transports. The API scopes messages; the bridge additionally applies [OS process isolation](#section-12).

Codex uses a workspace-write sandbox rooted at the agent outbox so it can prepare generated files and use the mailbox, without making the source checkout its writable workspace. Grok receives allow rules for the literal `sigma agent-act ...` command and file writing/editing under that outbox; other shell commands retain its configured permission handling. Both providers keep their existing exact session IDs. Custom adapters keep the plain-text stdin/stdout contract and receive the same environment/instruction variables; they can opt into these tools themselves.

No HTTP listener or public network endpoint is opened. Agent outboxes, socket/mailbox state and request results stay in the private account directory. Finishing a provider turn or stopping the worker closes its action service; the next turn has a different capability.

Use `agent-act messages` to find IDs for edits and reactions, including automatic replies. Results include sender ownership, attachment paths and text summaries capped at 1,000 characters. Use `next_before` as the second argument to page older messages. Request history separately from action batches.

To choose a quoted reply explicitly, use `"$SIGMA_BIN" agent-act reply "$SIGMA_INCOMING_MESSAGE_ID" "Your answer"`, then end the final output with `[SIGMA_NO_REPLY]` to suppress the automatic response. Use `messages` to find an older message ID. `send-message` remains unquoted.

## Webxdc in brief

Create an offline HTML/JS app ZIP named `.xdc`, with `index.html` at its root, in the outbox. `send-webxdc` sends it and returns the local app message ID. Use `webxdc-update ID '{"payload":{"value":1}}'` for durable shared state; optional `info`/`notify` fields make updates visible. Updates change state, not app code. Use `webxdc-updates ID AFTER_SERIAL` separately from batches, parse `updates_json` and save `last_serial`. Responses are bounded; request newer serials if old history exceeds the limit.

`webxdc-realtime ID TEXT` requires both peers online in the same app, waits up to 20 seconds for a peer, sends best-effort ephemeral data and leaves. It has no replay or delivery acknowledgement. The bridge continues networking for durable updates. These tools do not install app hooks or continuously feed realtime events into the model. Do not start another account/app worker while the bridge holds the account lock. All app IDs must belong to the current authorized chat. See [sender permissions](#section-9) for group authorization.

Incoming voice/audio is handled by the [agent transcription workflow](#section-15): the agent transcribes using its own capabilities and uses the result as input, or returns `[SIGMA_VOICE_UNSUPPORTED]` so Sigma reacts with ❌. Sigma does not transcribe on the agent’s behalf.

Bridge processes now use an outer Linux sandbox, with the account/database and authority files hidden. Only the conversation outbox and isolated provider runtime are writable host-backed state. Bot master transfer invalidates old action scopes; delegates never obtain master-transfer or call-answer authority. See [bot master permissions](#section-9) and [database protection](#section-12).


---

<a id="section-9"></a>

# Agent owner and sender permissions

The first `agent-connect --to` recipient initializes the sole **bot master** (also called the owner in older commands/output). Mastery can be transferred explicitly; changing the `--to` alias does not transfer it. Sigma checks the sender before invoking any provider or adapter. By default the bot answers only that owner: in the original direct chat and in groups that contain both the owner and the bot. Other senders are ignored. Adding the bot to a group does not approve that group's members.

## Approve from chat

In the group, **reply to a person's actual message** with one of these exact commands:

- `/approve` or `/assign`: allow that person in this chat and have the bot answer the quoted message now.
- `/revoke`: remove that person's permission in this chat.

Only the original owner can issue these commands. Extra arguments, missing quotes, cross-chat quotes, system messages and self/owner approval targets are refused. Approvals are stored by chat, contact and address/fingerprint: changing a person's identity requires approval again. The current master cannot be removed with `/revoke`; use an explicit master transfer. An approved person cannot approve others or change the bot account's name/photo. Ordinary messages, quoted instructions and app payloads cannot grant permission.

## List and remove access from chat

Send `/access` to see the owner and saved approvals, with names, contact IDs, chat IDs and active/inactive status. Only the owner can list or remove access. Use `/access 2` for the next page (20 approvals per page).

- In your private owner chat with this bot, `/access` lists approvals across all its chats. `/access remove CONTACT_ID` removes all of that person's approvals for this bot.
- In a group, `/access` lists only that group's approvals. `/access remove CONTACT_ID` removes only that group's approval, keeping approvals elsewhere private.

Copy the contact ID from the list; no quoted reply is required for removal. The bot master cannot be removed through access revocation; transfer mastery instead. Revocation keeps conversation history, and it does not remove the person from Delta Chat groups or affect other bots. Inactive saved approvals are listed too, so you can remove stale permissions after identity or membership changes.

## Manage from the command line

These commands work with a running bridge through its private control socket and with a stopped bridge through normal account locking:

```sh
./sigma agent-permissions codex list --account codex-new
./sigma agent-permissions codex approve CHAT_ID CONTACT_ID --account codex-new
./sigma agent-permissions codex assign CHAT_ID CONTACT_ID --account codex-new
./sigma agent-permissions codex revoke CHAT_ID CONTACT_ID --account codex-new
```

Use an agent ID instead of its alias if preferred. `assign` is an alias for `approve`; it adds permission without changing ownership or replacing other approvals. CLI approvals apply to the selected chat and take effect on the next message (they do not select an earlier message to answer). To approve a separate direct conversation, use that person's direct chat ID and contact ID. For a group approval, the owner must be a current member. CLI results are JSON; `list` includes `owner_contact_id` and a JSON-encoded `approvals_json` map keyed by `CHAT_ID:CONTACT_ID`. This is an owner administration command, not an `agent-act` capability. Stop the bridge before using ordinary `chats`, `contacts` or `group-members` commands that need the account lock.

## Conversations, files and tools

Each additional chat gets its own provider session and outbox under `agents/ALIAS/chat-CHAT_ID/`. The owner's original private session stays in `agents/ALIAS/`. Group sessions start with the triggering authorized message rather than importing the entire group history; subsequent authorized turns share that group session. The scoped `messages` tool can read that group's history as context, including messages from unapproved senders; those messages cannot authorize a turn or change permissions. Group replies and files are visible to all group members.

Files from an authorized incoming message are copied into that chat's outbox and their path/type is given to the provider. Attachment-only messages can trigger a turn. Agents can send files back using `agent-act send-file` and the other [media tools](#section-8). File contents and app data are untrusted input.

The action service is available only during a provider turn. Every turn gets a new credential, fixed to its current chat/account. The service rechecks sender permission before tool requests and each batch action, and Sigma checks again before queuing an automatic reply. CLI revocation therefore blocks subsequent effects from a still-running turn; already queued messages cannot be recalled. Chat commands are processed in arrival order after the current turn finishes. Removing the owner or bot from a group disables group responses; approval records remain saved. Provider cancellation or errors do not erase permissions or history.

Permissions are enforced in Sigma, and launched agent processes additionally use [OS filesystem isolation](#section-12).

Only the current bot master can also use `/models`, `/model`, and `/model MODEL_ID/default`. These change the bot-wide selection for the next turn without changing permissions. See [model management](#section-10).

## Transfer the sole bot master

Use `/master` to show the current master. The current master can **reply `/master` to another person's actual message in the same group**, or type `/master CONTACT_ID` in their private master chat. Only a regular, unblocked key contact can become master. The new person becomes the sole master; the former master loses automatic access, delegated approvals are cleared and active old-master action credentials/calls cease to authorize further actions. A direct chat with the new master is created through core and a notification is queued.

The trusted terminal may show, transfer or repair mastery, online or stopped:

```sh
./sigma agent-master codex --account codex-new
./sigma agent-master codex CONTACT_ID --account codex-new
```

`master.json` stores the current contact/chat/address/fingerprint, while `agent.json` retains the first `--to` binding as provenance. Missing master metadata means the initial recipient, for compatibility. A terminal transfer can repair a changed key; chat transfer must authenticate the current master. Exactly one master exists at a time. `agent-list` includes `bot_master_contact_id`.

Private provider sessions are scoped by direct-chat ID: the initial recipient's session remains in the original agent directory, while subsequent masters use their own `chat-CHAT_ID/`. Existing private history is not automatically imported into a new master's session. Transfer back to a former master resumes that person's earlier private session. Group sessions stay shared within the group; transferring mastery does not erase already-shared group messages/history. New master notifications and typed confirmations are sent by Sigma, not by the language model.

Approval removal occurs before the atomic master-file replacement. A failure during transfer can remove approvals without transferring mastery; this fails closed rather than carrying old delegations into a new master. Retrying from the trusted terminal can repair the state. `/master` and `/call` are reserved typed administration commands and are never delegated to the agent.


---

<a id="section-10"></a>

# Agent model selection

Native Codex and Grok bridges support owner-managed model selection from chat and the CLI. The selection applies to this bot across all its authorized chats. It takes effect on the next provider turn; a turn already running continues with its original model. Private and group session IDs/history remain intact.

## Chat commands

Only the current bot master can use these commands, privately or in a group containing the owner and bot:

- `/models`: list model IDs and names from this provider's locally cached catalog, with its fetch timestamp.
- `/model`: show the selected model.
- `/model MODEL_ID`: save a new selection after validating the ID against that catalog.
- `/model default`: use the provider CLI's configured default. Sigma does not claim an exact model ID when none is pinned.

These commands are handled by Sigma and never sent to the model, including when an approved guest attempts them. Switching models does not change sender permissions, bot ownership, account profile or conversation bindings.

## Command line

Works while the bridge runs, through its private control socket, and while it is stopped:

```sh
./sigma agent-models codex --account codex-new
./sigma agent-model codex --account codex-new
./sigma agent-model codex gpt-6.1-sol --account codex-new
./sigma agent-model codex default --account codex-new

./sigma agent-models grok --account grok-new
./sigma agent-model grok MODEL_ID --account grok-new
```

Use an agent's numeric ID instead of its alias if preferred. CLI output is JSON; model lists contain `provider`, `source`, `fetched_at`, and `models` (each with `id` and `name`). A null `model` means the provider CLI default. `agent-list` reflects the persisted selection too. These owner administration commands are unavailable inside agent turns; `agent-act` cannot change the model.

## Catalogs and persistence

Catalogs come from `CODEX_HOME/models_cache.json` or `~/.codex/models_cache.json`, and `GROK_HOME/models_cache.json` or `~/.grok/models_cache.json`. Sigma reads only model IDs, display names and the catalog timestamp for output; it never outputs cached credentials, endpoint details or raw provider configuration. Hidden Codex entries are excluded. No model IDs are hardcoded by Sigma.

The catalog is a cached provider menu, not a fresh entitlement check. Initialize or refresh it by using/logging into the provider CLI (Grok provides `grok models`). Missing, empty or malformed catalogs produce an error; unknown IDs leave the selection unchanged. Provider availability may change after a catalog fetch, so the provider may still refuse a listed model during a turn. Select another listed model or `default` if that occurs. The current model can be queried and reset to `default` even without a catalog.

`agents/ALIAS/model.json` stores the bot-wide selection. The startup `agent-connect --model` value initializes this file when it does not yet exist. Once saved, the selection survives worker restarts and takes precedence over old startup flags; use `agent-model` or `/model` to change it. Switching never rewrites `session.json` or chat-specific session files. The starting instructions and provider CLI model argument both receive the selected ID on each turn.

Custom `--exec` adapters manage their own model selection; these model-management commands refuse them. A private `custom-adapter` marker records that mode for stopped-agent checks. Provider sessions still determine what model-specific features can be used after switching.


---

<a id="section-11"></a>

# Experimental Delta Chat calls

Sigma can answer **audio calls to native Codex bridges**, restricted to the bot master. Native Grok Build and custom text adapters are not advertised as call-capable. Incoming calls require no browser or additional WebRTC/codec library. Opt-in outgoing calls use installed Chromium as described below; no Cargo dependency or browser package is installed by Sigma.

## Use it

Call the Codex bot using Delta Chat's audio-call button in your private bot-master conversation. The bot forwards your SDP offer to the installed Codex app-server's experimental `thread/realtime/start` WebRTC transport and returns `thread/realtime/sdp` through chatmail core's `accept_incoming_call`. Delta Chat includes a video transceiver in audio-only offers. Sigma offers only the bundled audio section to the provider, preserves its mid and bundled ICE candidates, and restores rejected video and application sections in the answer to keep Delta Chat’s media ordering. Delta Chat’s negotiated SCTP channels carry mute/ICE messages that are incompatible with provider realtime events and can close the provider transport; they must not be forwarded. Codex uses its server-side connection for realtime controls instead. Initial SDP candidates must establish the audio connection; Delta Chat’s data-channel ICE trickling and remote mute indicators are unavailable. Actual video-call attempts remain declined. Audio is exchanged directly between the calling client and OpenAI's realtime service; Sigma does not decode, record, transcribe or re-encode the call audio.

- After acceptance, Sigma requests a brief spoken greeting.
- `/call` shows this bridge's call capability and limitations.
- `/call stop` ends the current call. Delta Chat's hang-up button works too.
- Only the current bot master may call, even if another person has an approved text/voice-note permission. Other callers are declined without starting a provider session.
- Only one call runs per bridge. Video calls are declined with a request to use audio. Typed administration commands remain available during a call; ordinary text turns wait for a later message rather than running concurrently against the same session.
- A master transfer invalidates the call's scope. The call checks that scope every second. The bridge also ends calls on remote hang-up, provider failure, shutdown, or a 20-minute limit. Negotiation is bounded to 60 seconds.

The backing Codex thread uses the selected text model and its saved thread ID. Resume requests use `excludeTurns: true` so large conversation histories are kept inside Codex rather than copied into the signaling response. This preserves the thread’s history and avoids the 1 MiB signaling-event limit. The realtime voice service selects its own configured realtime model; Sigma does not pass a text-model ID as an audio-model ID. Master-private sessions stay separate after a transfer. The scoped chat tools are available during a call; client-side requests to approve permissions or execute external tools are refused. Spoken administration commands do not change mastery, access or model settings; type those commands instead.

## Research and verification

Research performed on 2026-11-13 with installed `codex-cli 0.160.0`, `grok 1.0.77` and the pinned chatmail core. The installed Codex schema generator reports the experimental WebRTC transport. A live negotiation probe using a Delta-style offer (including its unused video transceiver) obtained a valid SDP answer using the existing login, both directly and through Sigma's isolated app-server path. A subsequent headless Chromium media test reproduced a disconnect when Delta Chat’s mute/ICE messages were sent. With those channels rejected, WebRTC remained connected for 36 seconds and received non-silent speech (verified through inbound RTP packets and audio energy). The test used the existing installed Playwright/Chromium tooling; no project dependency was added. **Phone-to-agent audio, connectivity across NATs and interruption behavior still require a successful real Delta Chat call test.**

Primary sources:

- [Codex app-server documentation](https://learn.chatgpt.com/docs/app-server) and the [upstream realtime protocol](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/realtime.rs): thread-scoped realtime, WebRTC SDP offers/answers, sideband events. The installed CLI's generated schema is the implementation reference.
- [Delta Chat calls webapp](https://github.com/deltachat/calls-webapp): SDP signaling through core, browser-managed WebRTC, ICE gathering and negotiated channels. Core's existing `calls` module supplies incoming/ended events and accept/end operations.
- [Grok CLI reference](https://docs.x.ai/build/cli/reference) and installed CLI help: no verified live-call transport for Grok Build. xAI separately documents a [speech-to-speech API](https://docs.x.ai/developers/model-capabilities/audio/speech-to-speech) and a [WebRTC cookbook](https://docs.x.ai/cookbook/examples/voice-agent-webrtc) requiring API credentials and a separate media bridge. That is not interchangeable with the installed CLI subscription/session, so Sigma does not claim or fake Grok call support.

Provider account access, live-audio quotas and experimental protocol versions can affect availability. A provider error declines/ends the call with a short message to the still-authorized master. Offers/answers are not printed in Sigma logs. `tests/fixtures/call-offer.sdp` is a negotiation-only offer with a dummy fingerprint, not a real media connection.

## Opt-in outbound calls

Start a **native Codex** bridge with `--allow-outbound-calls` to let it place Delta Chat audio calls at its bot master's request:

```sh
./sigma agent-connect codex --to me --alias assistant --model gpt-6.1-sol --allow-outbound-calls
# Existing identity (stop its worker first):
./sigma agent-connect codex --agent-id 1 --account codex-new --allow-outbound-calls
```

This permission is off by default and applies only to this worker's lifetime. Include the flag on each restart; restarting without it disables outgoing calls. `agent-list --json` reports `outbound_calls`. Grok and custom adapters reject the flag because their outbound audio transport is not implemented.

In the master's private chat, share one contact card and say “Call this person and ask when tomorrow's meeting starts.” Alternatively, supply a contact ID, saved contact alias, address, or Delta Chat contact invitation link. Calling another agent uses that agent's Delta Chat contact. Its receiving policy still applies: a bot that only accepts calls from its own master may decline.

The agent receives this authenticated tool in its instructions. Resumed Codex turns also receive the current capability catalog in the turn input, because `exec resume` can retain the original developer instructions. This keeps existing conversations aware of newly enabled or disabled tools without resetting their history:

```sh
"$SIGMA_BIN" agent-act call TARGET "What to say or accomplish"
"$SIGMA_BIN" agent-act call message:123 "Ask when tomorrow's meeting starts"
```

`message:123` must refer to a single contact card sent by the master in this private chat. The trusted worker imports the original attachment; an agent-edited copy is not used. The tool runs inside the agent's capability scope, not as an unrestricted terminal dialer. Call actions cannot be batched. A successful result means **queued**, not answered; the worker reports dialing and termination in the master's chat. A call starts after the current text turn ends. There is one call at a time, a 90-second ringing limit and a 20-minute overall limit. `/call stop` cancels setup, ringing or an active call. Master transfer or lost authorization ends it. Calls never grant the recipient text access or mastery.

Each outgoing call uses a fresh isolated Codex conversation with only the supplied call brief. It does not inherit the master's private chat history or scoped messaging/administration credentials. Do not put private information in the brief unless it is intended for the recipient. The caller identifies itself as an AI assistant. The call does not generate a transcript report in the master's chat.

### Optional browser runtime

Codex's installed WebRTC interface answers offers; it does not originate them. Outbound calls therefore use **an installed Chromium** as a local audio bridge. Sigma controls it over private inherited pipes, with a fresh browser profile and no debugging TCP port. Chromium handles WebRTC, codecs and audio forwarding between the recipient and Codex; Delta Chat's mute/ICE channels terminate locally. Outbound audio passes through this machine. Incoming calls retain the existing direct provider path.

No Cargo dependency or browser package is installed by Sigma. It checks `SIGMA_CALL_BROWSER` (absolute executable path), standard Chromium/Chrome executables on `PATH`, then an existing Playwright browser cache. To choose a browser explicitly:

```sh
SIGMA_CALL_BROWSER=/absolute/path/to/chromium ./sigma agent-connect codex --to me --allow-outbound-calls
```

Keep the installed browser current. Chromium runs with its sandbox enabled. TURN/STUN configuration comes from chatmail core. If Chromium is absent or cannot run, outgoing calls fail with an explanation. The flag is not available for browsers on remote machines.

## Call history and cancellation from the terminal

```sh
./sigma agent-calls codex --account codex-new
./sigma agent-calls codex 100 --account codex-new --json
./sigma agent-call-cancel codex 123 --account codex-new
./sigma agent-call-cancel codex REQUEST_UUID --account codex-new
./sigma agent-call-cancel codex all --account codex-new
```

`agent-calls ID_OR_ALIAS [COUNT]` lists the newest 50 entries by default (maximum 100). It combines the selected account's stored incoming/outgoing calls, including archived chats, with this agent's outbound request journal. Actual calls include numeric `call_id`, chat, direction, core `call_status`, timestamp and connected `duration_seconds`. Outbound requests also have a UUID `request_id`, request `status`, and a failure reason when applicable. A request can fail before any call message exists; it will still appear in the journal. Queued and setup time are not counted as connected duration. Core call history predates this feature; request-only failure history starts with this version.

Cancel by numeric call ID after dialing starts, or by request UUID during queue/setup. `all` cancels this agent's queued requests and all currently open calls in the selected account. Commands work while the worker is running through its control socket, or while stopped through core. They require the trusted terminal and are unavailable inside agent turns. In chat, the master can continue using `/call stop`. Canceling does not delete history. Interrupted queued requests are not redialed after a restart.

The journal is stored privately under `agents/ALIAS/calls/`. It contains status, identifiers, timestamps and bounded error text; call briefs, invitation secrets and audio are not saved there. Sigma continues to use core APIs for all call database reads and updates.

Shared vCards may include large embedded profile photos. Outbound calling accepts cards up to 8 MiB, removes the photo (including folded lines) before importing identity/key fields, and keeps a 64 KiB bound on the remaining identity data. The original shared attachment is unchanged.


---

<a id="section-12"></a>

# Agent process isolation and database protection

Sigma owns the chatmail context, database connection and control policy. Provider CLIs and custom agent adapters communicate through scoped `agent-act` requests; they do not get a database handle or general account-management interface.

On Linux, bridge subprocesses now run through the system `/usr/bin/bwrap` executable ([bubblewrap](https://github.com/containers/bubblewrap), the sandboxing tool used by Flatpak). It was already installed on the development host. No new Cargo dependencies or downloaded runtime packages are added. Linux namespaces must be usable. If the sandbox cannot start or is unavailable, Sigma refuses agent execution; it does not fall back to an unrestricted process. Other operating systems currently need a separately implemented isolation backend before secure bridge execution is available.

The outer sandbox applies to the provider process itself, its tools and descendants, including custom adapters and the Codex call app-server:

- Host filesystem mounts are read-only. The selected account root, the legacy Sigma account and the named-account registry are hidden behind private empty mounts. Host database/WAL/SHM files, master/approval/model settings, action caches and control sockets are not writable or available through those paths.
- The current conversation's outbox and a private provider-runtime home are the writable host-backed directories. The action socket/mailbox remains accessible in the outbox. Attempts to create fake account/control files in private empty mounts do not change the host account or policy.
- Each conversation has its own `provider-home/`, seeded once from the provider's authentication/configuration/catalog files. A trusted `provider-home-initialized` marker outside the writable home prevents subsequent trusted copies through provider-controlled paths. Host login changes are not automatically recopied; refresh authentication from the trusted terminal while stopped if required, preserving the private session files. Existing native session files matching the exact saved thread ID are copied there for migration; unrelated provider sessions are not copied. Private state preserves subsequent resumptions. Do not delete these directories when migrating an active bridge.
- Original host provider configuration is not made writable. Runtime-home changes remain inside that isolated bridge's private storage and do not change the host provider configuration. Authentication copies remain sensitive private account state.
- PID, IPC and UTS namespaces isolate processes. A fresh `/proc`, `/dev`, `/tmp` and `/run` hide host process/control paths. Network access remains available for the provider. `kill_on_drop`, bubblewrap parent-death handling and the PID namespace bound process lifetime to the worker/turn.

The normal provider sandbox remains enabled as another layer. File generation in the outbox still works; account changes happen only through explicitly allowed Sigma actions. The role/capability check is repeated by the trusted worker and is not delegated to a model prompt. Master transfer invalidates old scopes, including ongoing calls. Generic user-configured `listen --on-message` hooks are outside this agent-bridge isolation change and retain their existing trusted-program behavior.

## Why the database was not encrypted

The pinned chatmail core explicitly marks database encryption deprecated in `Context::open` ([source](https://github.com/chatmail/core/blob/12e3a2a70a1c0dbca09fc341721ad13d8bbaffe4/src/context.rs)). More importantly, encryption at rest does not stop a process with write access from corrupting, replacing or deleting the encrypted file. Giving an agent the encryption key would not create a useful access boundary either.

Sigma therefore leaves the database format unchanged and blocks agent filesystem access at the OS boundary. It continues to make legitimate database updates through chatmail core for messages and account operations. There is no raw SQL rewrite or passphrase conversion of existing accounts. This is protection from launched agent processes, not an at-rest encryption feature.

The trusted terminal/worker and host OS administrator remain trusted. Host compromise, kernel vulnerabilities and unrelated privileged/local network services are not solved by this policy; separate users/containers or a dedicated machine can provide additional separation. Backups must protect the database, keys and private provider homes together.

## Outbound call permission

`--allow-outbound-calls` creates an in-memory capability in the trusted worker. The agent cannot enable it by editing files or adding fields to an action request. Signed `call` actions require the current bot master’s private-chat scope and are replay-protected. Guests and groups cannot dial, even if they have ordinary text access. The queue is not recovered after a restart. The trusted terminal must pass the flag again.

Outbound calls use an optional installed Chromium process with a fresh profile, fixed Sigma-owned JavaScript, sandbox enabled, and private CDP pipes. It handles audio locally and has no provider credentials injected into its page. The provider runs in its existing bubblewrap isolation with a fresh session, only the supplied brief, and no Sigma action token. Recipients gain no administrative authority. Incoming-call rules are unchanged.


---

<a id="section-13"></a>

# Webxdc apps and shared updates

## What is webxdc? (short version)

A **webxdc** is a mini web app shared as an attachment and opened inside Delta Chat:

- **Package:** an `.xdc` file is a ZIP (Deflate or Store), with `index.html` at the root plus local HTML/CSS/JavaScript and assets. `manifest.toml` can name the app; `icon.png` or `icon.jpg` can provide its icon.
- **Runtime:** the page runs offline in a sandbox without ordinary internet access. Bundle dependencies locally; no web server or API key is needed. For an existing website, build static output and use hash-based or single-page navigation instead of server routes.
- **Sharing:** a static page needs no special API. Interactive shared state uses the client-provided `webxdc.js` with `sendUpdate` and `setUpdateListener`. Do not bundle `webxdc.js`. Updates travel through chat when connected; arbitrary clicks and local storage are not automatically synchronized.
- **Sigma:** `./sigma send-webxdc me ./app.xdc` sends the app. Each sent app message is a separate shared instance. State updates do not replace app code; send a new package for new code.

[Official format](https://webxdc.org/docs/spec/format.html) · [JavaScript API](https://webxdc.org/docs/spec/api.html)

For an AI operating an app over time, start with the [named app workflow](#section-4): `send-webxdc --app NAME`, `app-start`, `app-send`, `app-update`, and `app-events`. The ID-based commands below remain available for direct use.

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

For ephemeral live packets, see the [realtime guide](#section-14) and `examples/realtime-app.xdc`.

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


---

<a id="section-14"></a>

# Webxdc realtime

For sending while listening, and saving app identity/events across AI tool calls, use the [named app worker workflow](#section-4). The commands below are the direct, foreground alternatives.

Realtime commands use chatmail core's peer channels. The app and CLI must be online in the same shared app instance. Packets are ephemeral: they have no saved history, replay cursor, or delivery receipt. For state that must survive disconnects, use [webxdc status updates](#section-13).

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

The messenger provides `webxdc.js`; do not include it in the archive. The example source is in `examples/realtime-app/`, and `examples/realtime-app.xdc` is ready to send. Rebuild it with the same Python ZIP recipe in the [status-update guide](#section-13), substituting `realtime-app` for `button-app`. No extra Rust dependencies were added.

Named app workers (`app-start` / `app-run`) join only while a realtime hook is registered. Add one with `app-hook-add NAME live ./examples/realtime-log-hook.py --type realtime`; removing the last leaves the channel. Update/message/command hooks do not enable realtime. The dedicated low-level commands above explicitly join on invocation.


---

<a id="section-15"></a>

# Voice messages

Send an existing local recording as a voice message using chatmail core's `Voice` message type:

```sh
./sigma send-voice CONTACT_ID ./recording.ogg --duration 3500
./sigma send-voice CHAT_ID ./recording.ogg --chat --duration 3500
./sigma send-voice 'https://i.delta.chat/#FULL_INVITATION' ./recording.ogg
```

Contact IDs, secure-join invitations, and existing chat/group/channel IDs follow the same rules as `send-file` and `send-photo`. `--chat` selects a chat ID. Sending to incoming read-only channels is rejected through core. Account storage defaults to `~/.config/sigma`; override it with `--data-path`.

`--duration` is optional metadata in milliseconds, from 0 to 2147483647; 0 or omission means unknown. The CLI does not measure duration, record your microphone, transcribe, validate codecs, or convert audio. Supply an actual recording in a format supported by your recipients. Empty files are rejected. `--caption` optionally supplies message text, whose presentation depends on the receiving client. Use `send-file` for a regular file attachment.

```sh
./sigma send-voice 10 ./recording.ogg --duration 3500 --caption 'Voice note' --json
./sigma send-voice --help
```

The JSON send result contains `message_id` and `chat_id`. `read-messages --json` and `listen --json` show `viewtype: "Voice"`, attachment details and `duration_ms`. A successful send means SMTP submission; playback needs to be checked in the receiving app. Failed submissions may remain queued; run `listen` before retrying.

`tests/fixtures/voice-tone.wav` is a one-second synthetic test tone, not a speech recording. It is used to verify voice-message type, unchanged audio bytes, and duration metadata through core without adding audio dependencies.

## Incoming voice notes for agents

Authorized voice notes and audio attachments reach the same persistent provider conversation as typed messages. Sigma stages the original recording in that chat's private outbox and gives its path/type to the agent. It does not run speech recognition or create a transcript itself.

The agent must transcribe the recording using its own available audio/transcription capabilities, retain that transcription in its conversation and then answer the spoken request. A separate transcript message is only sent if requested. It must not infer speech from filenames, captions or duration metadata. Support depends on the active provider CLI, model and available tools; a model's general audio support does not guarantee that its CLI can process a local recording.

When the agent cannot transcribe or obtain a usable transcript, it returns exactly `[SIGMA_VOICE_UNSUPPORTED]`. Sigma consumes that result and queues a ❌ reaction on the original voice/audio message, suppressing its normal text reply. If core refuses the reaction, Sigma tries a short quoted “Voice transcription is not supported or is unavailable for this recording” message instead. A missing recording also takes this unsupported path. Provider execution/network errors remain ordinary bridge errors rather than proof of unsupported audio.

Native starting instructions and `SIGMA_AGENT_INSTRUCTIONS` describe this workflow. Custom adapters receive the same audio task on stdin and the staged path in `SIGMA_INCOMING_VOICE_PATH`, plus optional `SIGMA_INCOMING_VOICE_DURATION_MS` metadata. Adapters perform their own transcription and return their response as usual, or the unsupported marker; Sigma never invokes an external transcription service on their behalf. Existing `[SIGMA_NO_REPLY]` suppression remains available after the agent sends its own messages through tools.

Sender permissions are checked before staging/invocation, and checked again before queuing the unsupported reaction or an automatic reply. Unapproved people do not get voice responses or reactions. Spoken `/approve`, `/access` or `/model` instructions are not owner administration commands; send those as typed chat commands. Group recordings and replies follow the existing group permission and conversation boundaries.


---

<a id="section-16"></a>

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

Webxdc commands and general `listen` emit `webxdc_updates` records with `message_id`, a native JSON `updates` array, and `last_serial`. `send-webxdc-update` emits `webxdc_update_queued` with the app `message_id`. See [webxdc.md](#section-13) for cursor and replay behavior.

Realtime commands emit `webxdc_realtime_joined`, `webxdc_realtime_sent` and `webxdc_realtime_data`. Received records have `message_id`, lossless `data` bytes, `text` (null for invalid UTF-8), and `bytes`. See [webxdc-realtime.md](#section-14).

Message records also include `duration_ms` (0 when unknown), used for voice/audio/video duration metadata. Voice notes have `viewtype: "Voice"`.

Named app control commands return `app_control` records with `ok`, `app`, `operation`, `message_id`, `chat_id`, and `message`. Status includes `running`, `realtime_enabled`, `realtime_joined`, `last_event`, and `last_serial`. A stopped app-info record has type `app` and `running: false`. `app-events` outputs stored JSON Lines with increasing `seq`, `type`, `app`, `message_id` and `time` (Unix seconds). Realtime events contain lossless `data`, optional UTF-8 `text` and `bytes`; status-update events contain native `updates` and `last_serial`. See [AI.md](#section-4).

Account and chat listings include `profile_photo` (stored local path or null). Dedicated bot/group photo changes return `profile_photo_updated` or `group_photo_updated`; the latter also includes `chat_id`.

App status reports the configured `hook_path`; running status also reports `hook_pending`. App journals include `hook_configured` and `hook_finished`. Results reference the original `event_seq`, `success`, `error` and bounded `output`. See [app-hooks.md](#section-6).

Named hook listings return an `app_control` record with a `hooks` array (`id`, `path`, `event`, `match_json`, `timeout`, `after_seq`). `hook_finished` records include `hook_id`; a single source `event_seq` can have separate results for multiple hooks. Hook input carries `hook_id`, and filtered status-update input contains only matching updates.

`hook_path` is the compatibility `default` hook only; null does not imply that all named hooks are disabled. Use `app-hook-list` for the authoritative saved list, including when the worker is stopped.

Hook listings include an explicit `type` (`realtime`, `update`, `message`, `command`, or legacy `all`) and canonical `event`. Message/command journal records expose `incoming_message_id`, `text`, `sender_id`, and the detailed `message`; commands also have `command` and `args`. Hook stdin adds `hook_id` and `state_path`.


---

<a id="section-17"></a>

# Backup and restoration

```sh
sigma backup ./backups --passphrase-file ./backup-password
sigma restore ./backups/sigma-chat-backup-EXAMPLE.tar --data-path ./restored --passphrase-file ./backup-password
```

`backup DIRECTORY` invokes core's backup exporter and creates a private output directory on Unix (0700). The backup contains account keys, transport credentials, chats, messages and attachments. Core excludes device-dependent settings and resets device-info messages on import. Core chooses the archive filename. JSON output includes file paths from core's export events. Do not place backups in publicly served or version-controlled folders.

Core encrypts the exported database when `--passphrase-file` is supplied; attachment blobs remain unencrypted TAR entries. The passphrase does not encrypt the entire archive. Without this option, the database is also unencrypted. The file option reads a UTF-8 passphrase, removing one trailing newline (and an optional preceding carriage return); other spaces are preserved. Empty passphrase files are rejected. Use the same passphrase file to restore an encrypted backup. Passphrases are not exposed in command arguments or output. The source password file's permissions remain the user's responsibility.

`restore BACKUP_FILE` requires a new or empty account directory. Existing `account.db` files and nonempty directories are refused before opening a context. Use `--data-path` to choose the destination. Core performs archive and passphrase validation. The original backup is not modified. If import fails, the destination can contain partial state; inspect it and use another empty directory for a retry. The command never erases an existing account to retry automatically.

Stop `listen` before backup or restore. Restored accounts retain the original identity; avoid running the old and restored copies concurrently as separate bots. Verify a restoration with `status`, `chats` and local `read-messages` before starting networking. Keep whole-directory copies with sigma stopped if you also need a raw filesystem snapshot.

Core export does not include sigma’s `apps/` metadata and event journals. To retain saved app names, cursors and captured realtime history, stop the app worker and copy the whole account directory, including `apps/`. After a core-only restore, use `app-bind NAME MESSAGE_ID` to register a restored app message.

Contact aliases are CLI sidecar state (`contact-aliases.tsv`), so core's encrypted backup does not include them. Preserve that private file separately when moving accounts, alongside any other CLI sidecar state you need. Sending checks both the saved contact ID and address before using an alias.


---

<a id="section-18"></a>

# Build, install and release packaging

Build with stable Rust (minimum 1.89), C/C++ tools, CMake, Perl, make and pkg-config:

```sh
cargo build --locked --release
scripts/install.sh --prefix "$HOME/.local"
```

The installer builds a locked release by default and installs `PREFIX/bin/sigma` (default `~/.local/bin/sigma`). `--binary PATH` installs an already built/downloaded executable without invoking Cargo. Add the installation directory to PATH. Installation replaces a binary at that destination but never changes the account directory.

```sh
scripts/install.sh --prefix ./test-install --binary ./target/release/sigma
scripts/package.sh --version 0.1.0
```

`package.sh` packages the existing release binary, README, license and docs into `dist/sigma-VERSION-OS-ARCH.tar.gz`, with a `.sha256` checksum file. `--binary PATH` and `--output DIRECTORY` override source/destination. It does not compile, push tags or publish anything. Versions allow letters, digits, dots, underscores and hyphens. Verify downloaded archives with `sha256sum -c` (or `shasum -a 256 -c` on macOS) before extracting/installing.

The standard CI workflow runs format checks, Clippy, offline tests and a Linux release build. Pushing a `v*` tag starts `.github/workflows/release.yml`; it can also be manually dispatched with an existing version tag. It checks out that tag, validates and builds on Linux and macOS, packages archives/checksums, and publishes only after both builds pass. Build jobs have read-only repository permissions; only the publish job has contents write permission.

Releases live at <https://github.com/omidz4t/sigma/releases>. Each release includes raw executables named `sigma-linux-x86_64` or `sigma-darwin-ARCH`, matching `.sha256` files, and documentation bundles. Executables are release assets, never Git source files. The AI prompt uses the version from Cargo.toml to link its matching release.

For a manual stable release, review/commit the source, create a version tag matching Cargo.toml (for example `v0.2.0`), and push the tag. Automatic testing releases use the Python workflow below. An explicit manual dispatch is an alternative for an existing tag; do not dispatch twice for an already published version. Windows packaging is not supplied. The install/package scripts target Unix systems.

Sample `.xdc` archives and media are generated from text source. `cargo test` generates its own fixtures through `build.rs`; use `python3 scripts/prepare-assets.py` for runnable example apps or live-test fixtures. Packaging runs that generator automatically. After documentation changes, regenerate `docs/REFERENCE.md` and `landing/static/reference.html` with `python3 scripts/build-reference.py`; after a release build, regenerate the prompt with `python3 scripts/prepare-landing.py`.

## Automatic versions and testing releases

Following [madmail's release workflow](https://github.com/themadorg/madmail/blob/main/docs/TDD/release-automation.md), Sigma uses a dependency-free Python 3.11+ script and Conventional Commits. After main passes CI, `feat:` bumps minor; `fix:`, `perf:` and `revert:` bump patch. A `!` or `BREAKING CHANGE:` bumps minor by default; only explicit `--allow-major` permits a major bump. Documentation, maintenance and CI commits alone do not release.

```sh
python3 scripts/release.py                 # preview; no writes
python3 scripts/release.py --check         # verify numeric versions agree
python3 scripts/release.py --write         # commit and tag locally (clean main)
python3 scripts/release.py --write --push  # atomically push main and tag
```

The script reads the highest merged stable/unstable version tag, updates `.version`, Cargo manifests, `CHANGELOG.md` and the AI prompt release links, then creates `chore(release): X.Y.Z [skip ci]` and annotated `vX.Y.Z-unstable`. Numeric package versions remain X.Y.Z. An atomic, non-forced push refuses concurrent remote changes. Retrying a prepared release does not bump it again. Use one release operator per checkout.

CI calls the release workflow directly after pushing: GitHub's default Actions token does not trigger a second tag workflow. Linux and macOS builds must pass before GitHub publishes the testing prerelease. Stable `vX.Y.Z` tags remain manually published via the existing tag workflow; choose a stable version newer than the last stable release. No release signing or production credentials are added. The script is an independent Sigma implementation of the workflow, not copied madmail source.


---

<a id="section-19"></a>

# Architecture and development

`src/main.rs` handles startup and routes help/version without opening an account. `src/cli.rs` parses and validates arguments, `src/account.rs` opens and locks account storage, and `src/handlers/mod.rs` dispatches each command to its own handler file. Handlers call core's profile configuration, `init_transports`, contact/chat APIs, message APIs, secure-join QR generation/validation/handshakes, and I/O lifecycle directly. Secure-join commands subscribe before starting the handshake so immediate progress events are retained, wait for progress 1000 for the target contact, and stop I/O on success, failure, timeout, or Ctrl-C. Tokio supplies async execution, signals, and event selection. There is no external RPC server, CLI framework, HTTP client, or second persistence layer.

QR exports generate one invitation with core, render that same value through core's `create_qr_svg`, and refuse existing output paths. No image or QR dependencies are added. Tests cover SVG export and preserving existing files.

Core is pinned to commit `12e3a2a70a1c0dbca09fc341721ad13d8bbaffe4`. To update, review upstream API/storage changes, update the revision in Cargo.toml, regenerate Cargo.lock, and run the checks below. Upstream source and API reference: <https://github.com/chatmail/core>.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

Unit tests cover CLI argument boundaries and validation, account locking, attachment copying/captions against a real offline core context, and UTF-8 text file loading (including Unicode, line breaks, empty files, and invalid encoding). File sends use `Message` with `Viewtype::File`, core blob deduplication, and synchronous SMTP submission. Ordinary tests and CI must not provision real accounts or send real messages. The separately invoked `scripts/live-test.py --run` explicitly provisions two disposable accounts and validates traffic only between them. Live integration testing uses two isolated test accounts and never opens the default account. See [testing.md](#section-20). Successful compilation does not verify a relay's availability or delivery to a recipient.

The launcher requires Cargo and runs from the caller's working directory, preserving relative data paths. Packaging should distribute the release binary as `sigma`; it needs no Cargo installation at runtime. GitHub CI builds a Linux binary and uploads it as a workflow artifact. Publishing releases and pushing changes are separate operator actions.

Contributions should document new commands and options here and in the built-in help. Preserve core as the authority for identity, messaging, encryption, and transport behavior.

Channel creation delegates to `chat::create_broadcast`. Channel-only operations check for `Chattype::OutBroadcast` before generating subscription invites or sending posts. Inviting a contact delivers core's channel secure-join link through a direct message, leaving subscription to core's secure-join protocol. The public `add_contact_to_chat` API rejects broadcast additions outside a handshake; the CLI does not modify membership tables directly. Offline tests cover channel persistence, rejection of direct chats, and channel argument validation. The opt-in live test covers subscription and delivery.

Receiving uses `Context::background_fetch` with timeout and Ctrl-C handling, followed by `get_fresh_msgs`. Detailed history and incoming events share message rendering through public core getters. Read acknowledgements delegate to `message::markseen_msgs` only for explicit `read-messages --mark-read`. The existing human `messages` output stays compatible; --json uses detailed message objects. The opt-in live test covers receiving and attachment export/download when complete; partial-message recovery and remote read receipts remain scenario-dependent.

Reactions delegate to core's `reaction::send_reaction` and `get_msg_reactions`. The CLI validates the token against core's `Reaction` normalization, preventing silent truncation from accidentally removing or changing a reaction. Sending starts I/O and polls the outgoing message state until submission/failure, with timeout and Ctrl-C cleanup. Local reaction counts are included in detailed message records. Unit tests cover setting/retracting syntax, compound emoji sequences, invalid tokens/IDs, and timeout validation. The opt-in live test covers reaction submission and synchronization.

`send-file` and `send-photo` share attachment preparation and target resolution. Photo sends create `Viewtype::Image` messages and leave image processing to core. Offline tests use a generated one-pixel PNG fixture to verify the image message type, caption, blob contents, and photo argument validation. No image-processing dependencies are added.

Each command defines its help text in a `HELP: CommandHelp` constant in its own handler file. `COMMAND_HELP` in `src/help.rs` references those constants and supplies shared rendering only. General help summaries use the same handler definitions; general help header/options live in `src/handlers/help.rs`. Update handler help together with dispatch/validation when introducing a command. Both `COMMAND --help`/`-h` and `help COMMAND` bypass account and network initialization. Tests exercise help targets for every registered command, command-specific option visibility, and literal `--help` message text after `--`.

## Source layout and command handlers

| Location | Responsibility |
| --- | --- |
| `src/main.rs` | Process entry point, errors, early help/version handling, account lifecycle |
| `src/cli.rs` | Standard-library argument parser and command/option validation |
| `src/help.rs` | Help schema, registry of handler definitions, and shared rendering |
| `src/account.rs` | Persistent account opening, directory permissions, process lock |
| `src/handlers/mod.rs` | Command-to-handler dispatch |
| `src/handlers/COMMAND_NAME.rs` | One command's `HELP` definition and `handle` function, using underscores for hyphens |
| `src/shared.rs` | Secure join, common sending, attachments, message rendering, and validation helpers |
| `src/tests.rs` | Existing CLI, storage, attachment, help, and validation tests |

Every command has a handler, including `help`, version flags, and the `join-link` alias. The alias delegates to the `securejoin-id` handler. Handlers receive the opened core context and parsed arguments; handlers that need the account directory also receive its path. Account locking remains held for the entire dispatch. Shared send preparation and target resolution avoid duplicating transport logic across text, file, and photo handlers.

To add a command, create its module under `src/handlers/`, expose a `handle` function, register the module and dispatch arm in `src/handlers/mod.rs`, add argument/option validation in `src/cli.rs`, and define its `HELP` metadata in that handler and register the reference in `src/help.rs`. Update user documentation and relevant tests. Keep help/version independent of account opening and networking.

`src/handlers/reply.rs` owns reply help and execution. It loads a target by `MsgId`, builds a text message with core's `set_quote`, checks the original chat's sending permission, and calls `send_msg_sync` in that chat. Offline tests create a core device message to verify quoted text, parent identity, original chat selection, Unicode/multiline reply text, and missing-message errors without sending anything.

Default storage resolves to `$HOME/.config/sigma` in `src/account.rs`; explicit `--data-path` bypasses HOME resolution. Missing, empty, or relative HOME values fail with a clear error rather than creating state in the working directory. Tests exercise default resolution and overrides without mutating process environment variables.

`src/handlers/create_group.rs` owns the `create-group` help and handler, using core's `chat::create_group` for encrypted groups. Offline tests verify group type, encryption, self-membership, and persistence after reopening the context. Group creation is separate from broadcast channel creation and does not alter membership or invite contacts.

`src/handlers/add_to_group.rs` owns membership command help and execution. It verifies group type, self-membership, and encrypted-group contact requirements, delegates to `chat::add_contact_to_chat`, then verifies local membership before reporting success. Offline tests generate a local identity/vCard through core, import a key contact, verify addition/persistence/idempotence, and reject address-only contacts, direct chats, channels, and reserved contacts. No network account provisioning or mail submission occurs in these tests.

`src/handlers/send_webxdc.rs` owns webxdc help, package preparation, and sending. It creates `Viewtype::Webxdc` attachments, checks the `.xdc` extension, and reads `index.html` through core's `get_webxdc_blob` before joining/sending. Default targets use the existing contact/secure-join resolver; `--chat` selects a numeric chat ID with a sending-permission check. Offline tests use a minimal locally authored webxdc fixture and reject corrupt archives, missing entry points, invalid targets/options, and absent files. No archive, webxdc, or CLI dependencies are added.

## Extended command and output layers

`src/output.rs` supplies dependency-free JSON serialization, typed result records and the legacy human-text envelope. All command stdout uses this layer. `--json` is global; diagnostic errors are emitted on stderr. `src/automation.rs` executes explicit hooks through Tokio processes without shell expansion, writes message JSON to stdin, bounds UTF-8 stdout to 64 KiB, applies a timeout, and queues core-quoted replies. Hook errors do not stop the listener.

`shared::send_to_target` resolves contact invitations, numeric contacts or explicit `--chat` targets for every send type. Management handlers use shared type/membership checks. Message editing, deleting, forwarding and download/export call core APIs directly. Download and secure-join helpers stop I/O on completion/failure/cancellation. Restore only opens a new/empty account path. Core backup/import runs with networking stopped under the per-account lock.

The release workflow is manual and requires an existing tag; local scripts do not push or publish. The separate opt-in live test runner and the regular offline suite are described in [testing.md](#section-20). Direct Rust dependencies are core, Tokio, UUID, HMAC and SHA-256; JSON, process hooks and CLI parsing add no external crates.

`src/apps.rs` implements the named app registry, durable event journal and private Unix-socket worker control. Individual app command handlers own their help. Client commands bypass direct core account opening; only the app worker accesses core while it is running. Tokio net support is used; no additional crates are introduced.

On macOS, the root enables the `all` feature of the already-transitive `socket2` 0.5 dependency. Core’s netwatch dependency requires raw socket and UnixStream conversions behind that feature. This does not add a new networking implementation or CLI framework.


`src/agents.rs` implements account-scoped background agent workers and their private Unix-socket controls. `agent-connect` dispatches a child with `--foreground`; that child alone opens/locks core. `agent-list` and `agent-stop` bypass account opening, query live sockets, and never signal saved PIDs. The runtime removes its socket on graceful exit; stale sockets are ignored by listing and replaced only by a worker holding the account lock. Native Grok invocations pass model and system-prompt options as separate arguments. Custom adapters retain the stdin/stdout contract. Offline tests cover control access under an account lock, stale socket replacement, graceful stopping, prompt/model argument forwarding and prompt-file validation.


`src/accounts.rs` resolves named accounts and a persisted default while preserving the original `legacy` directory. Registry mutations hold a private registry lock; deletion additionally requires the target account lock, refuses the default and symlinks, and never reaches into remote mailbox storage. Account, agent and app entry points share account resolution. Dedicated-agent provisioning goes through the existing init handler before worker dispatch. Offline tests exercise selection precedence, corrupt defaults, name validation, duplicate creation, protected deletion, isolation of legacy storage, and native Grok/Codex argument forwarding. Actual provider and recipient experiments are opt-in operations outside the normal test suite.


`src/agent_registry.rs` persists provider/alias/ID and the first recipient's core contact ID, address, fingerprint and direct chat. Allocation uses an account-local registry lock and atomic private writes; the counter advances before a new record is committed, allowing gaps without reuse on interrupted creation. The worker verifies the saved identity before changing profiles or queuing a notification. Recipient resolution on restarts bypasses mutable contact aliases unless explicitly supplied, in which case the resolved person must match. Management resolves numeric IDs to aliases and reports stopped saved records alongside live socket status. Offline tests cover persistence, equivalent contact selectors, contact-alias reassignment, refused recipient/provider changes, multiple aliases for one provider and stopping by numeric ID.


`src/agent_sessions.rs` owns exact provider-session creation/resumption, immediate private session-pointer writes, native structured-output parsing and one-time migration from the bound core chat history. Codex JSON events persist the thread ID before a turn completes. Grok uses UUID v4 for creation and explicit UUID resumption with response-ID verification. Neither provider uses ephemeral sessions or a shared “latest” session. Offline mocks verify exact resume arguments, ID stability, failure retention, pending-creation protection and refusal to switch sessions. The ignored `native_provider_sessions_live_memory` test exercises two real turns through each installed CLI only when explicitly invoked.


`src/agent_actions.rs` implements typed, strictly validated agent actions, their CLI, HMAC-SHA256 authorization and private socket/mailbox transports. The service shares the worker's existing core context and runs independently while a model turn waits for tool results. Per-worker keys remain in process environments; signed requests bind UUIDs to canonical action sets. Durable request results prevent duplicate queueing across retries and restarts, and pending/partial records fail explicitly. Preflight validates the complete batch and every message/file scope before mutation; core remains authoritative for media handling, edit restrictions and reactions. Tests cover both transports, wrong/cross-agent keys, signature tampering, recipient overrides, unrelated message IDs, symlink escape, unquoted media, profile updates, replay and startup key rotation. The ignored live action test invokes the real provider shell tools against an offline core chat.

`agent_master` owns authenticated sole-master transfer metadata; `agent_isolation` launches bridge children in system bubblewrap with hidden account state and a private native-provider home. `agent_calls` relays existing core call SDP to Codex app-server over Tokio JSON-lines stdio, with no media/codec/SDK dependencies. See [calls](#section-11) and [security](#section-12) for verified behavior and limits.


---

<a id="section-20"></a>

# Testing

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

Offline tests use real core contexts, generated local identities and imported vCards without connecting to relays. They cover argument boundaries, handler help, account locks/default paths, attachments/photos/webxdc validation, quoted replies, group/channel management, member checks, message edits/forwarding/search/deletion, attachment export, hook execution limits, JSON escaping, and an database-encrypted backup/restore round trip that refuses existing accounts.

## Explicit live integration checks

```sh
cargo build --locked
python3 scripts/live-test.py --run --binary target/debug/sigma --report /tmp/sigma-live-report.json
```

These opt-in tests create TWO disposable relay accounts in a private temporary directory, join them through secure join, and send test traffic only between those accounts. They never use `~/.config/sigma` and do not contact your saved contacts. An automatically allocated successful run removes local test account state; `--work-dir DIRECTORY` reuses and retains an explicitly chosen test directory. Failures retain the private state/log directory named in the report for inspection. Relay accounts themselves are not remotely deleted by this script. It requires Python 3 and available relay/network services. It is not run by ordinary `cargo test` or CI.

Checks include text delivery, receiving, quoted replies/reactions, files/text files/photos/webxdc, attachment export/download when already complete, group joins and all sending types with `--chat`, channel subscription/posting/editing/removal, message edits/search/forward/deletion, fixed replies/executable hooks, backup restore, and membership removal/leave. Network availability affects results. A live run does not prove every server condition, partial-message download recovery, remote deletion propagation, or every recipient client implementation works.

Install and packaging can be tested without touching system directories by providing a temporary prefix/output and an already built binary. Test release archives and their checksums before publishing. Local Linux verification cannot validate the hosted macOS build or GitHub publication permissions.

The recorded successful isolated run is saved in [live-validation.json](../docs/live-validation.json). It records the checked scenarios and their limits without account credentials or invitation links.

## Webxdc status updates

Offline tests cover update JSON validation through core, app message type checks, cursor filtering, persistence across reopening the account, and the core status-update event. The included `examples/button-app.xdc` provides a manual client test: send the app, queue a text payload with `send-webxdc-update`, start `listen-webxdc-updates`, open the app on a second device, and press its button. Confirm the CLI text appears in the app and a `button_clicked` payload appears in the listener. See [webxdc.md](#section-13).

A live two-account check also sent a state update to a second account, then returned a simulated button-event payload through core to the dedicated listener; general `listen --json` emitted the structured update as well. That transport check does not replace opening and clicking the actual app in a messenger client.

## Webxdc realtime

Offline tests verify argument validation, UTF-8 and binary encoding, empty/maximum/oversized payloads, bounded file reads, and binary JSON output. A live two-account test verified bidirectional UTF-8 packets and a lossless binary packet sent by the send-only command. Actual button rendering/clicking in a messenger client requires the included `examples/realtime-app.xdc`; see [webxdc-realtime.md](#section-14).

The realtime demo was also opened in the user’s test client: its automatic pong and five manually triggered button-click packets reached the CLI. Connection timeout without a peer and Ctrl-C during connection both cleaned up successfully and released the account lock.

## Voice messages

The voice regression test checks command options and invalid durations, voice type and duration persistence through core, unchanged audio bytes, and rejection of empty recordings. The fixture is a one-second synthetic WAV test tone. Playback and presentation must be checked in a receiving messenger.

Visible/silent update modes were also tested against a controlled peer: both state updates arrived, and only the update containing `notify` generated an `IncomingWebxdcNotify` event. The visible update included an `info` chat entry; the silent update omitted both fields.

## Named app workers

Offline tests cover safe names and event cursors, alias collision protection, binding only webxdc messages, saved identities, private file permissions, torn journal-tail recovery, monotonic event sequences and recovering the committed status-update cursor.

Live two-account checks exercised separate CLI processes sending and receiving through background workers, both notification modes, malformed update handling without killing the worker, history reads while stopped, restart and event-cursor resumption. A second check covered `send-webxdc --app` registration, duplicate-name rejection before resend, repeated `app-start`, responsive status while waiting for a peer, rejection of premature realtime sends, durable updates while no realtime peer is connected, and clean shutdown while joining. The worker shutdown regression is checked by requiring a final persisted `worker_stopped` event.

## App event hooks

Offline tests verify saved executable/timeout configuration, executable permissions, pending-event recovery versus completed results, action JSON validation, durable response submission through core, and disabling a saved hook. Existing executable-hook tests cover timeouts, process exit, UTF-8 and output limits.

Live checks used separate app workers: realtime and status-update button events invoked the executable and returned durable responses to the peer. Hook configuration survived a worker restart. A deliberately slow hook timed out while the worker stayed responsive, and disabling the hook persisted. The user's hook demo also delivered both button-event types, ran the executable successfully, and displayed replies in the actual client.

Multiple named hooks are tested independently: exact event types, recursive payload subset matching, excluding unrelated updates from batch input, per-hook completion/recovery, and legacy single-hook migration. Live tests verified two hooks on one event, no invocation for unmatched packets, a separate status-update hook, dynamic add/list/remove, removal surviving restart, cancellation of a running hook, and strictly increasing journal sequences after live configuration changes.

Typed app hooks were verified with controlled live accounts: update, message and `/ping` command hooks ran with realtime disabled; the quoted command reply arrived at the peer. Adding the first realtime hook joined, a packet triggered it, removing the last left, and restarting with non-realtime hooks stayed disabled. `app-send` refused to autojoin. The unit suite contains 36 tests.

Chat-level hooks were tested independently of webxdc: saved registration/listing, `/ping` received through `listen`, quoted response delivered to the controlled peer, and removal. The suite now contains 37 tests.


Native agent session memory can be checked with the explicitly opted-in test below. It uses installed Codex/Grok authentication, makes real provider requests and creates disposable provider conversations; regular tests never run it.

```sh
cargo test --locked native_provider_sessions_live_memory -- --ignored --nocapture
```

The test sends a code word in one turn, recalls it in the next, and checks that both turns used the same persisted provider session ID.


A separate opted-in action test lets the installed Codex and Grok CLIs call the signed messaging tool. It queues messages only in an offline self-chat and does not start mail networking; the provider requests themselves use the network and local authentication:

```sh
cargo build --locked
cargo test --locked native_providers_can_use_scoped_action_tools -- --ignored --nocapture
```

Agent guard tests exercise owner-only quoted approval commands, chat scope, persisted identity matching, revocation of active action credentials, profile restrictions, owner group membership, and CLI permission changes online under the account lock and offline. Scoped Webxdc tests send a validated app, exchange durable state and reject app IDs in unrelated chats before any realtime side effects. Live peer-to-peer realtime delivery is not covered by those offline tests.

Chat access tests cover owner-only `/access` listing and removal, private bot-wide versus group-local scope, nonowner denial, permanent-owner protection and invalid arguments.

Model management tests validate CLI syntax, sanitized Codex/Grok catalog parsing, hidden-model filtering, rejection of unknown IDs and custom adapters, persistent selection over old startup flags, default reset, unchanged session files, owner-only chat commands, and online/offline CLI operation. Catalog fixtures include dummy credentials to check they are not exposed as model metadata.

Incoming-audio tests use a simulated transcription-capable adapter and the existing synthetic tone fixture to check original-byte staging, audio path/duration environment, transcription instructions, normal response threading and unsupported-result cross reactions. They verify the handoff protocol, not real speech recognition quality or a provider’s current audio capabilities.

Bot-master tests cover authenticated chat transfer, terminal transfer, unique authority, cleared delegations and invalidation of old scopes. Linux sandbox tests attempt to read/write database and master files and to overwrite an unrelated host file, while allowing outbox assets. Live provider action tests also migrate an existing exact-ID conversation from the original provider home into isolation.

```sh
cargo test --locked native_codex_call_negotiates_sdp_in_isolation -- --ignored --nocapture
```

The opt-in call test uses installed Codex authentication/network and a dummy-fingerprint SDP fixture. It proves the isolated app-server obtains an SDP answer and reaches core's incoming-call acceptance check; the core fixture is deliberately outgoing, so it cannot create a real phone/audio connection. A real Delta Chat phone call remains a separate manual interoperability test. No user database is modified by these offline/test accounts.

Outbound call coverage includes flag validation, master-only capability checks, disabled-by-default behavior and signed-request replay protection. With installed Chromium, run `cargo test --locked outbound_browser_bridges_audio_both_directions -- --ignored` for a real local two-leg WebRTC test that verifies non-silent audio in both directions. It uses synthetic tones and makes no call to a person. A live Codex probe also verified speech forwarding through the browser bridge; a real recipient's phone/network remains an interoperability check.


---

