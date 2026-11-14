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
