---
name: sigma
description: Use Sigma to connect a bot to a user's Delta Chat account, send messages and files from the terminal, and operate chat hooks or shared webxdc apps. Use for Sigma messaging workflows and Delta Chat notifications from an AI agent.
---

# Sigma

Sigma is a stateful Rust CLI built on chatmail core. It provisions its own bot account and keeps keys, credentials, contacts, messages and attachments locally. No messaging API key is required. It is an unofficial project, unaffiliated with Delta Chat.

## Find or install the CLI

Check `command -v sigma` and an existing project-local `./sigma`, then run `--version` and `--help`. Use the actual executable path in subsequent commands; examples below assume `./sigma`. Help is local and does not open account storage or start networking.

If absent, get the binary matching the host OS and CPU from the [official releases](https://github.com/omidz4t/sigma/releases). Prefer the latest stable release; use a prerelease only when the user's requested capability needs it and they accept that choice. Inspect release assets instead of guessing platform support. Download the matching `.sha256` file and verify the binary before running it, then `chmod +x sigma` on Unix. Do not use an unrelated package named sigma. Existing installations need not be upgraded just to use this skill.

Check `./sigma COMMAND --help` for the installed version before relying on options. In particular, inspect `create-contact --help`: some older binaries accept only an invitation and return a numeric contact ID, without saved aliases. Use that ID for subsequent sends if aliases are unavailable.

## Create, connect, send

Use the user's chosen account directory. The default is `~/.config/sigma`; pass the same `--data-path DIRECTORY` on every command for another account. Inspect an existing account with `status` rather than replacing or deleting it. Run `init` for a new bot:

```sh
./sigma init
```

The default name is Sigma and the photo is the right-facing Sigma logo on versions supporting these defaults. Respect explicit name/photo preferences. **`init` creates the bot account; it does not connect the bot to the user.**

To connect, obtain the user's own full Delta Chat contact invitation. Explain the phone steps: open Delta Chat → QR button → QR Invite Code → LINK → COPY TO CLIPBOARD. Ask for the invitation only if they have not provided one. The inviter should stay online while joining. Do not fabricate an invitation or use a provisioning QR, group invitation or channel invitation as a contact link.

For a binary supporting aliases:

```sh
./sigma create-contact me "https://i.delta.chat/#FULL_INVITATION"
./sigma send-message me "Hello from Sigma!"
./sigma send-file me ./report.pdf --caption "Finished report"
```

Replace the quoted placeholder with the actual invitation before execution. Wait for `create-contact` to succeed, then use the saved alias or returned ID. Aliases are case-sensitive, scoped to one account and cannot overwrite an existing alias; inspect `contacts` or choose a new alias if needed. Do not treat a failed join as a successful connection.

Execute sends only for the recipient and content authorized by the user's request. A request to set up Sigma alone does not authorize sending a test message or private files. Once sending is authorized, proceed without repeatedly asking permission. Quote links, message text and paths. Keep invitation links, account directories, credentials and private messages out of public logs, screenshots and source control.

## What webxdc means

A **webxdc** is a small HTML/CSS/JavaScript app sent as an `.xdc` attachment and opened inside Delta Chat. It is a ZIP archive with `index.html` at its root; bundle images, fonts and scripts locally because the app runs offline in a sandbox without ordinary internet access. No hosted website or API key is needed. Static pages need no webxdc API. For shared state, reference the client-provided `webxdc.js` (do not bundle it) and use `sendUpdate` / `setUpdateListener`; updates travel through chat when networking is available. UI clicks are not automatically shared. Send with `./sigma send-webxdc me ./app.xdc`. Each sent app message is a separate instance; updating its state does not replace its code. See `references/workflows.md` for operation details.

## Read results and recover

Use `--json` when parsing output; it emits JSON Lines, not necessarily one JSON document. Preserve returned IDs. Report command submission separately from confirmed recipient receipt: SMTP success alone does not prove delivery.

A failed send can remain queued. Inspect the error before resubmitting; `listen` keeps networking active, receives messages and retries queued sends. Do not blindly resend and create duplicates. Stop a foreground `listen` with Ctrl-C before another direct CLI command opens the same account. If the user requests continuous operation, use the environment's supported background-process lifecycle and report how to stop it.

Read `references/workflows.md` when the task needs message history, groups/channels, slash-command hooks, or shared webxdc apps. Named app workers have their own control commands and do allow `app-send`/`app-update` while running; do not apply the foreground-listener restriction to those controls.

For additional commands, consult installed help first, then the [command documentation](https://omidz4t.github.io/sigma/docs/) or [complete reference](https://github.com/omidz4t/sigma/blob/main/docs/REFERENCE.md). Installed help takes precedence when documentation describes a different version.
