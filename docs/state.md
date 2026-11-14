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

Core backup/export and import are exposed as `backup` and `restore`; see [backups.md](backups.md). `restore` refuses existing/nonempty accounts. Message edits, deletions, forwarding, membership changes and hook replies may queue work that is persisted by core and later transmitted by `listen`. JSON output changes presentation only and never changes persistence or enables automation.

Named app metadata and event journals are saved in `apps/NAME/` under the same account directory. A background worker owns the core lock while app control/read commands use private local IPC or read the journal. See [AI.md](AI.md) for exact state, cursors, restart and backup behavior.

Contact aliases created by `create-contact ALIAS INVITE_ID` are stored in the private account-local `contact-aliases.tsv` sidecar. They survive CLI restarts and resolve to a validated contact ID/address pair. Moving only the core database or using core backup/restore does not move this sidecar; preserve it separately if aliases are needed on the destination.


## Dedicated agent accounts

`agent-connect PROVIDER --new-account NAME --name TEXT --profile-photo PATH --to ALIAS --invite LINK` provisions a fresh named account before starting its worker. It does not change the default. The invitation handshake creates a new contact and saves ALIAS in that account; aliases and numeric contact IDs are never copied between identities. Alternatively pass the full invitation directly as `--to LINK`. To use an existing account, replace `--new-account NAME` with `--account NAME` and use an alias already saved there (or join a new invitation). Name/photo options can also update an existing agent account when connecting.


Agent identities are saved under `agents/ALIAS/agent.json` within the account, with stable numeric IDs reserved by `agents/next-id`. Their first recipient binding survives stopping, account moves and worker restarts; keep these sidecars with the account when backing it up. Agent aliases, contact aliases and account names are separate namespaces. IDs are account-local. Reassigning a contact alias cannot reassign an agent; select a saved agent by ID or alias to restart it with its stored recipient. Account deletion also removes these identities and bindings.


Native agent `session.json` files point to a single persistent provider conversation. Keep them alongside agent bindings during backup/moves. Actual native conversation history is managed by the provider CLI in its own storage; copying only Sigma account files to another machine does not copy that history. Stops and restarts retain the pointer. Missing provider history or interrupted first-time session creation fails explicitly rather than selecting another conversation.


Agent `outbox/` directories hold generated/staged assets and private action transport files. `action-results/` holds signed-action request/result history for safe retries; keep it with the account when migrating. The capability itself is generated per provider turn and held in memory/environment, so finishing a turn invalidates its capability. Account deletion removes these local files with the rest of that identity.

Owner-managed `agents/ALIAS/approvals.json` stores chat/contact fingerprint approvals. Additional chats keep provider sessions, outboxes and action-result caches under `agents/ALIAS/chat-CHAT_ID/`. Preserve these sidecars when migrating. See [agent permissions](agent-permissions.md).

Bot-wide model selection is persisted in `agents/ALIAS/model.json`; existing startup flags initialize it only when absent. Preserve this sidecar with the account. Native sessions keep their IDs when switching. A `custom-adapter` marker identifies bridges whose external adapter manages its own models. See [agent models](agent-models.md).

`master.json` records the current sole bot master; `agent.json` retains the first recipient as provenance. Additional master-private sessions are keyed by chat ID and do not automatically import previous master-private history. Each conversation’s `provider-home/` contains isolated native provider runtime/auth/session copies; preserve it with `session.json` for resumptions. The database format is unchanged: [OS process isolation](agent-security.md) protects it from agents. See [calls](agent-calls.md) and [master transfers](agent-permissions.md#transfer-the-sole-bot-master).
