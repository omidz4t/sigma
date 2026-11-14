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

The recorded successful isolated run is saved in [live-validation.json](live-validation.json). It records the checked scenarios and their limits without account credentials or invitation links.

## Webxdc status updates

Offline tests cover update JSON validation through core, app message type checks, cursor filtering, persistence across reopening the account, and the core status-update event. The included `examples/button-app.xdc` provides a manual client test: send the app, queue a text payload with `send-webxdc-update`, start `listen-webxdc-updates`, open the app on a second device, and press its button. Confirm the CLI text appears in the app and a `button_clicked` payload appears in the listener. See [webxdc.md](webxdc.md).

A live two-account check also sent a state update to a second account, then returned a simulated button-event payload through core to the dedicated listener; general `listen --json` emitted the structured update as well. That transport check does not replace opening and clicking the actual app in a messenger client.

## Webxdc realtime

Offline tests verify argument validation, UTF-8 and binary encoding, empty/maximum/oversized payloads, bounded file reads, and binary JSON output. A live two-account test verified bidirectional UTF-8 packets and a lossless binary packet sent by the send-only command. Actual button rendering/clicking in a messenger client requires the included `examples/realtime-app.xdc`; see [webxdc-realtime.md](webxdc-realtime.md).

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
