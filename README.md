# 🗿 sigma

[Website](https://omidz4t.github.io/sigma/) · [Download a release](https://github.com/omidz4t/sigma/releases) · [Source](https://github.com/omidz4t/sigma)

**All documentation in one place:** [complete reference](docs/REFERENCE.md), or open the [standalone browser reference](landing/static/reference.html).

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

The [Sigma skill](skills/sigma/SKILL.md) teaches account setup, connecting with a Delta Chat invitation, sending messages/files, and operating hooks or shared webxdc apps. It checks the installed CLI's help for version compatibility and keeps private account data out of source control. [Browse on skills.sh](https://skills.sh/omidz4t/sigma/sigma).

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

`app-start` keeps a background worker online. Wait for `realtime_joined` before sending live packets. App identity, durable state and captured events are saved under `~/.config/sigma`; restart with the same name to continue. The recipient must open the same app instance. Hand another AI the [AI workflow guide](docs/AI.md).

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

Each agent has a persistent alias (`--alias NAME`, default: provider name) and an automatic numeric ID, both scoped to its account. The first `--to` records the initial recipient and initializes the sole bot master. Use `/master` or `agent-master` for an explicit transfer; see [bot master management](docs/agent-permissions.md#transfer-the-sole-bot-master). Restarts can omit `--to` and select the agent with `--alias NAME` or `--agent-id ID`; another recipient is refused. `agent-list` includes stopped identities, and `agent-stop` accepts either their alias or ID. To delegate access, approve the person; to transfer authority, change the bot master.

Native Grok and Codex reuse one persistent provider session per agent, so messages share conversation history across worker restarts. Existing chat history is imported once when the session is first created; later turns use the provider’s own history. `agent-list` reports the saved `session_id`. Native Grok and Codex support `--model`, `--system-prompt TEXT` and `--system-prompt-file PATH`. `--name` and `--profile-photo` set the chat account's profile. New accounts need their own contact handshake: `--invite LINK` saves the `--to me` alias in that account. Existing aliases remain account-local. Use `--exec ./adapter` for another local CLI, or `--foreground` for a foreground connection. See [account operations](docs/state.md) and [agent setup and limits](docs/automation.md#local-agent-bridge).

Agents also receive [private chat tools](docs/agent-actions.md) in their starting instructions. They can send voice notes, files, videos, captioned images and multiple standalone messages, react/edit within their bound chat, and change their own account name/photo. Per-worker signed authorization fixes the recipient and account; media is staged in a private agent outbox.

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

- [AI webxdc workflow](docs/AI.md)
- [Command reference](docs/cli.md)
- [State and account operations](docs/state.md)
- [JSON output](docs/output.md)
- [Bot hooks and automatic replies](docs/automation.md)
- [Backup and restoration](docs/backups.md)
- [Build, installation and releases](docs/releases.md)
- [Testing and live checks](docs/testing.md)
- [Architecture and development](docs/development.md)

GitHub CI checks formatting, Clippy, tests and release compilation. The manual release workflow builds Linux/macOS archives with checksums and publishes only when explicitly dispatched for an existing tag. Sigma is licensed under [MIT](LICENSE). Chatmail core remains MPL-2.0; see [third-party notices](THIRD_PARTY_NOTICES.md).

### Webxdc state and button events

Send apps with `send-webxdc`, queue shared state with `send-webxdc-update`, read it with `webxdc-updates`, and receive button events with `listen-webxdc-updates`. See the [webxdc guide](docs/webxdc.md) and included `examples/button-app.xdc`.

Use `send-webxdc-realtime` for ephemeral text/binary packets and `listen-webxdc-realtime` to receive live button events. The listener supports `--send` for sending and receiving in one session. See the [realtime guide](docs/webxdc-realtime.md) and `examples/realtime-app.xdc`.

Send local recordings as voice notes with `send-voice`. See the [voice guide](docs/voice.md).

Change photos with `set-profile-photo PATH`, `set-group-photo GROUP_ID PATH` and `set-channel-photo CHANNEL_ID PATH`. Pass a quoted empty string to clear a photo. Use `send-photo` for a photo message.

Run a defined executable on app button events with `app-start NAME --on-event ./hook.py`. Hook settings and outcomes persist with the app. See the [app hook guide](docs/app-hooks.md) and included `examples/app-hook.py` / `examples/hook-app.xdc`.

Register multiple filtered app hooks with `app-hook-add NAME HOOK_ID EXECUTABLE --type realtime --match '{"type":"button_clicked"}'`. Use `app-hook-list NAME` and `app-hook-remove NAME HOOK_ID` while running or stopped. See the [hook guide](docs/app-hooks.md).

App hooks require an explicit `--type realtime|update|message|command`; command hooks can select `--command /ping`. Workers join realtime only while a realtime hook (or legacy broad hook) exists. See [typed hooks](docs/app-hooks.md).

For slash commands directly on a chat without an app, use `chat-hook-add CHAT_ID ID EXECUTABLE --command /ping` and run `listen`. See [chat command hooks](docs/chat-hooks.md).

Example app archives and test media are generated from text source; run `python3 scripts/prepare-assets.py` before using example `.xdc` files. `cargo test` generates its own fixtures automatically. No compiled binaries or account credentials are committed.

Agent bridges enforce [owner-controlled sender permissions](docs/agent-permissions.md), with owner-only `/approve`, `/assign`, `/revoke` replies and online/offline `agent-permissions` CLI management. Groups retain separate sessions and scoped media/Webxdc tools.

Owners can [list and switch agent models](docs/agent-models.md) using `/models` and `/model MODEL_ID` in chat, or `agent-models` and `agent-model` from the CLI, while preserving conversation history.

Native Codex bridges support [experimental audio calls](docs/agent-calls.md) restricted to the bot master. Agent processes use [Linux OS isolation](docs/agent-security.md) to protect Sigma databases and control state; no new Cargo dependencies were added. Grok CLI live calls are not currently enabled.

Native Codex bridges can opt in to outbound audio calls with `--allow-outbound-calls`. The bot master can share a contact and ask the agent to call it. Add `--record-calls` to save both sides of incoming/outgoing calls locally and send recordings to the bot master. This requires installed Chromium; see [outbound call tools and security](docs/agent-calls.md#opt-in-outbound-calls).
