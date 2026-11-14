<script>
  import { t } from '#lib/i18n.svelte.js';
  import { resolve } from '#lib/i18n.svelte.js';
  import { commands, commandGroups } from '#lib/commands.js';
</script>

<svelte:head><title>{t("Command reference — sigma")}</title><meta name="description" content={t("Every Sigma CLI command, with usage, options and examples.")} /></svelte:head>
<h1>{t("Command reference")}</h1>
<p>{t("Usage, options, and examples for all")} {commands.length} {t("sigma commands. Search the list or choose a command below.")}</p>
<p>{t("Account state defaults to")} <code>~/.config/sigma</code>{t(". Use the same")} <code>--data-path</code> {t("for commands that share an account.")}</p>
<div class="command-directory">
  {#each commandGroups as group}
    <section class="command-index-group" aria-labelledby={`index-${group.title.replaceAll(' ', '-')}`}>
      <h2 id={`index-${group.title.replaceAll(' ', '-')}`}>{t(group.title)}</h2>
      {#each group.commands as command}
    <a href={resolve('/docs/[command]', { command: command.name })}>
      <h3>{command.name}</h3>
      <p>{t(command.description)}</p>
    </a>
      {/each}
    </section>
  {/each}
</div>
