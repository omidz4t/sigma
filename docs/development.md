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

Unit tests cover CLI argument boundaries and validation, account locking, attachment copying/captions against a real offline core context, and UTF-8 text file loading (including Unicode, line breaks, empty files, and invalid encoding). File sends use `Message` with `Viewtype::File`, core blob deduplication, and synchronous SMTP submission. Ordinary tests and CI must not provision real accounts or send real messages. The separately invoked `scripts/live-test.py --run` explicitly provisions two disposable accounts and validates traffic only between them. Live integration testing uses two isolated test accounts and never opens the default account. See [testing.md](testing.md). Successful compilation does not verify a relay's availability or delivery to a recipient.

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

The release workflow is manual and requires an existing tag; local scripts do not push or publish. The separate opt-in live test runner and the regular offline suite are described in [testing.md](testing.md). Direct Rust dependencies are core, Tokio, UUID, HMAC and SHA-256; JSON, process hooks and CLI parsing add no external crates.

`src/apps.rs` implements the named app registry, durable event journal and private Unix-socket worker control. Individual app command handlers own their help. Client commands bypass direct core account opening; only the app worker accesses core while it is running. Tokio net support is used; no additional crates are introduced.

On macOS, the root enables the `all` feature of the already-transitive `socket2` 0.5 dependency. Core’s netwatch dependency requires raw socket and UnixStream conversions behind that feature. This does not add a new networking implementation or CLI framework.


`src/agents.rs` implements account-scoped background agent workers and their private Unix-socket controls. `agent-connect` dispatches a child with `--foreground`; that child alone opens/locks core. `agent-list` and `agent-stop` bypass account opening, query live sockets, and never signal saved PIDs. The runtime removes its socket on graceful exit; stale sockets are ignored by listing and replaced only by a worker holding the account lock. Native Grok invocations pass model and system-prompt options as separate arguments. Custom adapters retain the stdin/stdout contract. Offline tests cover control access under an account lock, stale socket replacement, graceful stopping, prompt/model argument forwarding and prompt-file validation.


`src/accounts.rs` resolves named accounts and a persisted default while preserving the original `legacy` directory. Registry mutations hold a private registry lock; deletion additionally requires the target account lock, refuses the default and symlinks, and never reaches into remote mailbox storage. Account, agent and app entry points share account resolution. Dedicated-agent provisioning goes through the existing init handler before worker dispatch. Offline tests exercise selection precedence, corrupt defaults, name validation, duplicate creation, protected deletion, isolation of legacy storage, and native Grok/Codex argument forwarding. Actual provider and recipient experiments are opt-in operations outside the normal test suite.


`src/agent_registry.rs` persists provider/alias/ID and the first recipient's core contact ID, address, fingerprint and direct chat. Allocation uses an account-local registry lock and atomic private writes; the counter advances before a new record is committed, allowing gaps without reuse on interrupted creation. The worker verifies the saved identity before changing profiles or queuing a notification. Recipient resolution on restarts bypasses mutable contact aliases unless explicitly supplied, in which case the resolved person must match. Management resolves numeric IDs to aliases and reports stopped saved records alongside live socket status. Offline tests cover persistence, equivalent contact selectors, contact-alias reassignment, refused recipient/provider changes, multiple aliases for one provider and stopping by numeric ID.


`src/agent_sessions.rs` owns exact provider-session creation/resumption, immediate private session-pointer writes, native structured-output parsing and one-time migration from the bound core chat history. Codex JSON events persist the thread ID before a turn completes. Grok uses UUID v4 for creation and explicit UUID resumption with response-ID verification. Neither provider uses ephemeral sessions or a shared “latest” session. Offline mocks verify exact resume arguments, ID stability, failure retention, pending-creation protection and refusal to switch sessions. The ignored `native_provider_sessions_live_memory` test exercises two real turns through each installed CLI only when explicitly invoked.


`src/agent_actions.rs` implements typed, strictly validated agent actions, their CLI, HMAC-SHA256 authorization and private socket/mailbox transports. The service shares the worker's existing core context and runs independently while a model turn waits for tool results. Per-worker keys remain in process environments; signed requests bind UUIDs to canonical action sets. Durable request results prevent duplicate queueing across retries and restarts, and pending/partial records fail explicitly. Preflight validates the complete batch and every message/file scope before mutation; core remains authoritative for media handling, edit restrictions and reactions. Tests cover both transports, wrong/cross-agent keys, signature tampering, recipient overrides, unrelated message IDs, symlink escape, unquoted media, profile updates, replay and startup key rotation. The ignored live action test invokes the real provider shell tools against an offline core chat.

`agent_master` owns authenticated sole-master transfer metadata; `agent_isolation` launches bridge children in system bubblewrap with hidden account state and a private native-provider home. `agent_calls` relays existing core call SDP to Codex app-server over Tokio JSON-lines stdio, with no media/codec/SDK dependencies. See [calls](agent-calls.md) and [security](agent-security.md) for verified behavior and limits.
