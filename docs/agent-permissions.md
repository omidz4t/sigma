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

Files from an authorized incoming message are copied into that chat's outbox and their path/type is given to the provider. Attachment-only messages can trigger a turn. Agents can send files back using `agent-act send-file` and the other [media tools](agent-actions.md). File contents and app data are untrusted input.

The action service is available only during a provider turn. Every turn gets a new credential, fixed to its current chat/account. The service rechecks sender permission before tool requests and each batch action, and Sigma checks again before queuing an automatic reply. CLI revocation therefore blocks subsequent effects from a still-running turn; already queued messages cannot be recalled. Chat commands are processed in arrival order after the current turn finishes. Removing the owner or bot from a group disables group responses; approval records remain saved. Provider cancellation or errors do not erase permissions or history.

Permissions are enforced in Sigma, and launched agent processes additionally use [OS filesystem isolation](agent-security.md).

Only the current bot master can also use `/models`, `/model`, and `/model MODEL_ID/default`. These change the bot-wide selection for the next turn without changing permissions. See [model management](agent-models.md).

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
