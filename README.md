# 🗿 sigma

**Bring your terminal and AI agents into Delta Chat.**

Sigma is a stateful Rust command-line tool built on [chatmail core](https://github.com/chatmail/core). Send messages and files, run chat commands, share interactive webxdc apps, and connect local AI agents to conversations. Your messaging account and its state stay on your machine.

[Website](https://omidz4t.github.io/sigma/) · [Documentation](https://omidz4t.github.io/sigma/docs/) · [Releases](https://github.com/omidz4t/sigma/releases) · [Report an issue](https://github.com/omidz4t/sigma/issues)

## Features

- **Messaging:** send text, photos, files, voice notes, reactions, and replies in private chats, groups, and channels.
- **AI agents:** connect Codex or Grok, retain provider sessions across restarts, and manage who can interact with each bot.
- **Interactive apps:** share webxdc apps, exchange durable updates and realtime events, and operate apps through background workers.
- **Automation:** run message hooks and slash commands, with JSON Lines output for scripts.
- **Multiple accounts:** give each bot its own identity, profile, contacts, and local state.
- **Experimental audio calls:** connect native Codex agents to Delta Chat calls, with opt-in outbound calling and recording.

Sigma is an independent project and is not affiliated with Delta Chat. Agent features require an installed, authenticated provider CLI. Secure agent execution currently requires Linux with bubblewrap; calls have additional provider and platform requirements. See [agent isolation](docs/agent-security.md) and [call support and limitations](docs/agent-calls.md).

## Installation

### Download a binary

Download the executable for your platform from [GitHub Releases](https://github.com/omidz4t/sigma/releases), along with its matching `.sha256` file. Linux and macOS release artifacts are supported; Windows packages are not currently provided.

For the Linux x86_64 download, verify and install it with:

```sh
sha256sum -c sigma-linux-x86_64.sha256
install -Dm755 sigma-linux-x86_64 "$HOME/.local/bin/sigma"
sigma --help
```

Make sure `~/.local/bin` is on your `PATH`. On macOS, verify checksums with `shasum -a 256 -c` and use the matching macOS executable. See the [installation guide](docs/releases.md) for details.

### Build from source

Requires **Rust 1.89+**, a C/C++ compiler, CMake, Perl, make, and pkg-config. On Debian or Ubuntu:

```sh
sudo apt-get install build-essential cmake perl pkg-config
git clone https://github.com/omidz4t/sigma.git
cd sigma
scripts/install.sh --prefix "$HOME/.local"
sigma --help
```

The installer builds a locked release and installs it to `~/.local/bin/sigma`. To build without installing, run `cargo build --locked --release`; the executable is `target/release/sigma`. The repository's `./sigma` script is a Cargo launcher.

## Quick start

Get your contact invitation link from Delta Chat, then create a bot and send yourself a message:

```sh
sigma init --name "My bot"
sigma create-contact me "FULL_DELTA_CHAT_INVITATION_LINK"
sigma send-message me "Hello from Sigma!"
```

Replace `FULL_DELTA_CHAT_INVITATION_LINK` with your full invitation link. The alias `me` saves that contact for later commands. Provisioning and messaging require network access.

To invite someone to your bot instead, generate its QR code:

```sh
sigma give-qr --output ./invite.svg
```

Keep the account online to receive messages and retry queued deliveries:

```sh
sigma listen --json
```

Stop `listen` before running another command against the same account. For a one-time receive, use `sigma receive`.

## Connect an AI agent

On Linux, install bubblewrap (`sudo apt-get install bubblewrap` on Debian/Ubuntu), and install and sign in to the provider CLI you want to use. Then connect it to the contact created above:

```sh
sigma agent-connect codex --to me --alias assistant
```

Or connect Grok:

```sh
sigma agent-connect grok --to me --alias researcher
```

The bridge runs in the background. The first `--to` contact becomes the **bot master**, who controls access. Each agent keeps a persistent identity and provider session.

```sh
sigma agent-list
sigma agent-stop assistant
# Restart the saved identity:
sigma agent-connect codex --alias assistant
```

Use a separate account for a separate bot identity:

```sh
sigma agent-connect codex --new-account research-bot --name "Researcher" \
  --alias researcher --to me --invite "FULL_DELTA_CHAT_INVITATION_LINK"
```

See [agent setup](docs/automation.md#local-agent-bridge), [permissions](docs/agent-permissions.md), [models](docs/agent-models.md), and [agent chat tools](docs/agent-actions.md). Native Codex also supports [experimental audio calls](docs/agent-calls.md); Grok call support is not enabled.

### Install the Sigma skill

Teach your coding agent how to use Sigma:

```sh
npx skills add omidz4t/sigma --skill sigma
```

The [skill](skills/sigma/SKILL.md) covers account setup, messaging, hooks, and shared apps. It checks the installed CLI's help for compatibility. You can also [find it on skills.sh](https://skills.sh/omidz4t/sigma/sigma).

## Usage examples

### Send files and read messages

```sh
sigma send-file me ./report.pdf --caption "Today's report"
sigma send-photo me ./photo.jpg
sigma send-voice me ./recording.ogg --duration 3500
sigma chats --json
sigma read-messages CHAT_ID --mark-read
sigma reply MESSAGE_ID "Thanks!"
```

Replace `CHAT_ID` and `MESSAGE_ID` with IDs returned by Sigma.

### Create a group

```sh
sigma create-group "Team"
sigma add-to-group GROUP_ID CONTACT_ID
sigma send-message GROUP_ID "Hello, team!" --chat
```

Use the returned group and contact IDs. The `--chat` flag targets a chat ID instead of a contact.

### Share an interactive app

```sh
sigma send-webxdc me ./app.xdc --app demo --json
sigma app-start demo --json
sigma app-update demo '{"payload":{"text":"Hello from Sigma"}}' --json
sigma app-events demo --after 0 --json
sigma app-stop demo --json
```

The recipient opens the shared app in Delta Chat. Named app workers retain state and events across restarts. See the [webxdc guide](docs/webxdc.md), [realtime guide](docs/webxdc-realtime.md), and [AI app workflow](docs/AI.md).

### Run a message hook

```sh
sigma listen --on-message ./my-hook --hook-timeout 30 --json
```

The executable receives a message JSON object on stdin and may return reply text on stdout. Hooks run only when explicitly configured. See [message hooks](docs/automation.md) and [chat command hooks](docs/chat-hooks.md).

## Accounts and storage

The original account lives in `~/.config/sigma`. Named accounts live in `~/.config/sigma-accounts/accounts/NAME`.

```sh
sigma account-create my-bot --name "My bot"
sigma account-list
sigma account-default my-bot
sigma status --account my-bot
```

Use `--account NAME` to select an account, or `--data-path DIRECTORY` for an explicit location. Account directories contain private keys, credentials, messages, and attachments; keep them out of source control. Local storage does not mean messages stay local: chat relays deliver them, and connected AI providers process messages sent to their agents.

Back up an account with:

```sh
sigma backup ./backups --passphrase-file ./backup-password
sigma restore ./backups/BACKUP.tar --data-path ./restored \
  --passphrase-file ./backup-password
```

Restore requires a new or empty destination. A backup passphrase protects the database, **not attachment blobs in the TAR archive**. See [account operations](docs/state.md) and [backups](docs/backups.md).

## Documentation

Every command includes built-in help:

```sh
sigma --help
sigma send-file --help
sigma help agent-connect
```

| Guide | Contents |
| --- | --- |
| [Complete reference](docs/REFERENCE.md) | Documentation in one file |
| [Command reference](docs/cli.md) | Commands, options, and examples |
| [JSON output](docs/output.md) | Output format for scripts and integrations |
| [Agent permissions](docs/agent-permissions.md) | Bot master, approvals, and group access |
| [App hooks](docs/app-hooks.md) | Responding to app events and commands |
| [Build and releases](docs/releases.md) | Installation, packaging, and versioning |
| [Architecture](docs/development.md) | Project structure and development |
| [Testing](docs/testing.md) | Automated tests and optional live checks |

## Development and contributions

Bug reports and pull requests are welcome. For a bug report, include your Sigma version, operating system, reproduction steps, and relevant logs with credentials and private messages removed.

Run the local checks before submitting a change:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Tests generate their own fixtures. To create runnable example apps and media, run `python3 scripts/prepare-assets.py`. Live network/provider checks are documented separately in the [testing guide](docs/testing.md). For website development, see [landing/README.md](landing/README.md).

## License

Sigma is licensed under the [MIT License](LICENSE). Chatmail core is licensed under MPL-2.0; see [third-party notices](THIRD_PARTY_NOTICES.md).
