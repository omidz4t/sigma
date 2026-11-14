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

- [Automation](automation.md): explicit fixed replies and executable message hooks with bounded time/output.
- [JSON output](output.md): record shapes and scripting contract.
- [Backup/restore](backups.md): encrypted archives and refusal to overwrite an existing account.
- [Build/install/releases](releases.md): local install, archive/checksum creation, manually dispatched hosted releases.
- [Testing](testing.md): offline checks and explicit isolated live integration testing.

Exit status is 0 on success/help/version and 1 for invalid arguments, state, filesystem, core or network errors. Per-account locking prevents multiple commands from concurrently using the same state. Stop `listen` before another command operates on its account. No command pushes repository changes.

## Webxdc shared state

`send-webxdc-update MESSAGE_ID UPDATE_JSON` queues state for an app instance. `webxdc-updates MESSAGE_ID [--after SERIAL]` reads stored updates. `listen-webxdc-updates MESSAGE_ID [--after SERIAL]` replays and listens for updates, including button events emitted by the app. See [webxdc.md](webxdc.md) for a complete workflow and runnable example.

`send-webxdc-realtime` sends ephemeral text or binary bytes; `listen-webxdc-realtime` joins an app channel and receives live packets, optionally sending initial text or a file. Both peers must be connected. See [webxdc-realtime.md](webxdc-realtime.md).

See [voice.md](voice.md) for sending voice notes with optional duration metadata.

## Named app workers for AI tools

Use `send-webxdc TARGET FILE --app NAME` to save an app identity. `app-bind NAME MESSAGE_ID` registers a previously sent/received app. `app-list` and `app-info NAME` inspect saved apps. `app-start NAME` starts a background worker; `app-run NAME` runs it in the foreground. While it is running, `app-send NAME DATA [--file]` sends live data and `app-update NAME UPDATE_JSON` sends durable state. `app-events NAME [--after SEQ]` reads the persisted event journal even while stopped. `app-stop NAME` leaves the channel and releases the core account lock. All support `--json` and `--data-path`; see [AI.md](AI.md) for the complete workflow and limitations.

## Profile, group and channel photos

`set-profile-photo PATH` changes the bot avatar without provisioning an account. `set-group-photo GROUP_ID PATH` and `set-channel-photo CHANNEL_ID PATH` change chat icons through core. Pass a quoted empty string to clear a photo. Run networking (`listen` or an app worker) to transmit queued chat-photo changes; the bot avatar is shared with contacts by outgoing messages. `status --json` exposes the bot `profile_photo`; `chats --json` exposes each chat’s `profile_photo`.

`app-start` and `app-run` accept `--on-event PATH`, `--hook-timeout SEC` and `--no-hook` to save or clear an executable app-event hook. See [app-hooks.md](app-hooks.md).

`app-hook-add NAME HOOK_ID EXECUTABLE --type realtime|update|message|command [--command /NAME] [--match JSON] [--hook-timeout SEC]` adds a named event-specific hook. `app-hook-list NAME` lists all definitions, and `app-hook-remove NAME HOOK_ID` removes one without stopping other hooks. They support live workers and stopped apps. See [app-hooks.md](app-hooks.md).

Named workers join realtime only when a realtime hook (or legacy `all` hook) exists. Adding the first joins; removing the last leaves. Update, message and command hooks continue with mail networking only. Message/command hooks are scoped to the app chat.

## Chat command hooks

`chat-hook-add CHAT_ID HOOK_ID EXECUTABLE --command /NAME [--hook-timeout SEC]` saves a chat-specific slash-command hook. `chat-hook-list CHAT_ID` lists definitions; `chat-hook-remove CHAT_ID HOOK_ID` removes one. Run `listen` to execute them, without any webxdc app. Stop workers/listeners before editing. See [chat-hooks.md](chat-hooks.md).

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
