<script>
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { languages, language, localizedPath, t } from '#lib/i18n.svelte.js';
  const id = $props.id();
  let dialog;
  function openLanguages() {
    if (!dialog.open) dialog.showModal();
    dialog.querySelector('[aria-pressed="true"]')?.focus();
  }
  function selectLanguage(code) {
    dialog.close();
    goto(localizedPath(page.url.pathname, code) + page.url.search + page.url.hash);
  }
  function closeOnBackdrop(event) {
    if (event.target !== dialog) return;
    const rect = dialog.getBoundingClientRect();
    if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) dialog.close();
  }
</script>

<button type="button" class="language-picker" aria-label={t('Language')} aria-haspopup="dialog" aria-controls={`language-dialog-${id}`} onclick={openLanguages}>
  <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m10.5 21 5.25-11.25L21 21m-9-3h7.5M3 5.25h12m-6-3v3m1.048 9.5A18.022 18.022 0 0 1 3.87 9.75m8.38-4.5a18.03 18.03 0 0 1-8.5 12" /></svg>
  <span lang={language.current}>{languages.find(item => item.code === language.current)?.label}</span>
</button>
<dialog class="language-dialog" id={`language-dialog-${id}`} bind:this={dialog} aria-labelledby={`language-title-${id}`} onpointerdown={closeOnBackdrop}>
  <div class="dialog-header">
    <h2 id={`language-title-${id}`}>{t('Language')}</h2>
    <button class="close-dialog" type="button" aria-label={t('Close')} onclick={() => dialog.close()}>
      <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" d="m6 6 12 12M6 18 18 6" /></svg>
    </button>
  </div>
  <div class="language-options">
    {#each languages as item}
      <button type="button" class="language-option" aria-pressed={language.current === item.code} onclick={() => selectLanguage(item.code)}>
        <span lang={item.code} dir={item.code === 'fa' ? 'rtl' : 'ltr'}>{item.label}</span>
        {#if language.current === item.code}
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m5 12 4 4L19 6" /></svg>
        {/if}
      </button>
    {/each}
  </div>
</dialog>
