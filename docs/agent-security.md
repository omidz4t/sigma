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
