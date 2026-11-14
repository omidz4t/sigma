<script>
  import { t } from '#lib/i18n.svelte.js';
  import UsagePresentation from '#lib/components/UsagePresentation.svelte';
</script>
<section class="use-cases" aria-labelledby="use-cases-title">
  <header class="use-cases-banner"><p class="use-cases-question">{t('You’re asking, what’s it for?!')}</p><h2 id="use-cases-title">{t('A few things you can do with sigma.')}</h2></header>
  <section class="usage-chapter terminal-chapter" aria-label={t('Terminal')}><UsagePresentation kind="terminal" /></section>
  <section class="usage-chapter ai-chapter" aria-label={t('AI agent')}><UsagePresentation kind="ai" /></section>
  <section class="usage-chapter agents-chapter" aria-label={t('Agents & calls')}><UsagePresentation kind="agents" /></section>
</section>
