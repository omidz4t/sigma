<script>
  import { t } from '#lib/i18n.svelte.js';
  import { resolve } from '#lib/i18n.svelte.js';
  let { data } = $props();
  const command = $derived(data.command);
</script>

<svelte:head><title>{command.name} {t("— sigma")}</title><meta name="description" content={t(command.description)} /></svelte:head>
<a class="docs-back" href={resolve('/docs')}>{t("All commands")}</a>
<h1>{command.name}</h1>
<p>{t(command.description)}</p>
<h2>{t("Usage")}</h2>
<pre><code>{command.usage}</code></pre>
<h2>{t("Options")}</h2>
<pre><code>{command.options}</code></pre>
<h2>{t("Example")}</h2>
<pre><code>{command.example}</code></pre>
<p class="docs-note">{t("Options may appear before or after arguments. Quote text, paths with spaces, and invitation links. Use")} <code>--</code> {t("before literal arguments beginning with")} <code>-</code>.</p>
<p class="docs-note">{t("Run")} <code>./sigma {command.name} --help</code> {t("to check the help for your installed binary. Help never opens account storage or starts networking.")}</p>
