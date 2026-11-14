<script>
  import { language, t } from '#lib/i18n.svelte.js';
  import { tick } from 'svelte';
  import { page } from '$app/state';
  import { pageMarkdown } from '#lib/doc-markdown.js';
  let status = $state('');
  let fallback = $state('');
  let fallbackDialog;
  let fallbackText;
  const markdown = $derived.by(() => { language.current; return pageMarkdown(page.params.command); });
  async function copy(forAI = false) {
    const text = forAI
      ? `Use this Sigma CLI documentation to help with my task. Verify commands against the installed binary and ask before sending messages or changing account state.\nSource: ${page.url.href}\n\n${markdown}`
      : markdown;
    try {
      await navigator.clipboard.writeText(text);
      status = forAI ? 'Copied for AI.' : 'Markdown copied.';
    } catch {
      fallback = text;
      fallbackDialog.showModal();
      // Svelte updates the textarea value before selecting it.
      await tick();
      fallbackText.focus();
      fallbackText.select();
      status = "Copy the selected text with Ctrl+C or ⌘C.";
    }
  }
  function download() {
    const url = URL.createObjectURL(new Blob([markdown], { type: 'text/markdown;charset=utf-8' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = `sigma-${page.params.command || 'commands'}.md`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    status = "Markdown downloaded.";
  }
</script>

<div class="docs-page-actions" aria-label={t("Use this documentation")}>
  <button type="button" onclick={() => copy(true)}><svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 0 1-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 0 1 1.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 0 0-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 0 1-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 0 0-3.375-3.375h-1.5a1.125 1.125 0 0 1-1.125-1.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H9.75" /></svg>{t("Copy for AI")}</button>
  <button type="button" onclick={() => copy()}>{t("Copy Markdown")}</button>
  <button type="button" onclick={download}>{t("Download .md")}</button>
</div>
<p class="docs-actions-status" role="status" aria-live="polite">{t(status)}</p>
<dialog class="docs-copy-dialog" bind:this={fallbackDialog} aria-labelledby="docs-copy-title">
  <div class="dialog-header"><h2 id="docs-copy-title">{t("Copy documentation")}</h2><button type="button" onclick={() => fallbackDialog.close()}>{t("Close")}</button></div>
  <label for="docs-copy-text">{t("Select and copy with Ctrl+C or ⌘C")}</label>
  <textarea id="docs-copy-text" bind:this={fallbackText} value={fallback} readonly rows="15"></textarea>
</dialog>
