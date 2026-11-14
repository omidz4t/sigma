<script>
  import LanguageSwitcher from '#lib/components/LanguageSwitcher.svelte';
  import { t } from '#lib/i18n.svelte.js';
  import DocsActions from '#lib/components/DocsActions.svelte';
  import repoStats from '#lib/repo-stats.json';
  import { onMount } from 'svelte';
  import { asset } from '$app/paths';
  import { resolve } from '#lib/i18n.svelte.js';
  import { page } from '$app/state';
  import { commandGroups } from '#lib/commands.js';
  let { children } = $props();
  let menu;
  function closeMenu() { menu?.close(); }
  function closeBackdrop(event) {
    if (event.target !== menu) return;
    const bounds = menu.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) closeMenu();
  }
  onMount(() => {
    const desktop = window.matchMedia('(min-width: 761px)');
    const changed = () => { if (desktop.matches) closeMenu(); };
    desktop.addEventListener('change', changed);
    return () => desktop.removeEventListener('change', changed);
  });
  let search = $state('');
  let selection = $state(null);
  const activeGroup = $derived(commandGroups.find(group => group.commands.some(command => command.name === page.params.command))?.title || 'Init');
  const expanded = $derived(selection && selection.command === page.params.command ? selection.title : activeGroup);
  function toggleGroup(title) {
    selection = { command: page.params.command, title: expanded === title ? null : title };
  }
  const filtered = $derived(commandGroups.map(group => ({
    ...group,
    commands: group.commands.filter(command => `${t(group.title)} ${command.name} ${t(command.description)}`.toLowerCase().includes(search.toLowerCase()))
  })).filter(group => group.commands.length));
</script>

{#snippet commandNavigation(prefix)}
      <a class="docs-overview" href={resolve('/docs')} onclick={closeMenu}>{t("Command reference")}</a>
      <label for={`${prefix}-search`}>{t("Find a command")}</label>
      <input id={`${prefix}-search`} type="search" bind:value={search} placeholder={t("Search commands…")} />
      <nav aria-label={t("Commands")}>
        {#each filtered as group}
          <div class="docs-command-group">
            <button class="docs-group-toggle" type="button" aria-expanded={Boolean(search) || expanded === group.title} aria-controls={`${prefix}-commands-${group.title.replaceAll(' ', '-')}`} onclick={() => toggleGroup(group.title)}>
              {t(group.title)}
              <svg class="icon" class:group-open={Boolean(search) || expanded === group.title} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m9 5 7 7-7 7" /></svg>
            </button>
            <div class="docs-group-links" id={`${prefix}-commands-${group.title.replaceAll(' ', '-')}`} hidden={!search && expanded !== group.title}>
            {#each group.commands as command}
              <a href={resolve('/docs/[command]', { command: command.name })} onclick={closeMenu} aria-current={page.params.command === command.name ? 'page' : undefined}>{command.name}</a>
            {/each}
            </div>
          </div>
        {:else}
          <p class="docs-empty">{t("No matching commands.")}</p>
        {/each}
      </nav>
{/snippet}

<div class="documentation">
  <header class="docs-header">
    <a href={resolve('/')} class="docs-brand"><img src={asset('/assets/logo/moai-center.png')} width="46" height="46" alt="" />sigma</a>
    <div class="docs-header-links">
      <LanguageSwitcher />
      <a class="docs-github-link" aria-label={t("Sigma on GitHub")} href="https://github.com/omidz4t/sigma"><svg class="icon github-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" focusable="false"><path d="M12 .75a11.25 11.25 0 0 0-3.558 21.923c.563.103.768-.244.768-.542 0-.267-.01-.975-.015-1.913-3.13.68-3.79-1.51-3.79-1.51-.512-1.3-1.25-1.646-1.25-1.646-1.022-.7.077-.686.077-.686 1.13.08 1.725 1.16 1.725 1.16 1.005 1.722 2.636 1.225 3.278.937.102-.728.393-1.225.715-1.507-2.499-.285-5.126-1.25-5.126-5.565 0-1.23.44-2.234 1.16-3.022-.116-.285-.503-1.43.11-2.98 0 0 .945-.302 3.094 1.154A10.78 10.78 0 0 1 12 6.175c.956.005 1.918.13 2.817.378 2.148-1.456 3.092-1.154 3.092-1.154.615 1.55.228 2.695.112 2.98.722.788 1.158 1.792 1.158 3.022 0 4.327-2.631 5.277-5.138 5.555.404.35.765 1.04.765 2.098 0 1.515-.014 2.737-.014 3.11 0 .3.203.65.774.54A11.252 11.252 0 0 0 12 .75Z" /></svg><span class="docs-github-label">GitHub</span></a>
      <div class="github-star-widget">
        <a class="github-star-action" href="https://github.com/omidz4t/sigma" aria-label={t("Star sigma on GitHub")}><svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true" focusable="false"><path stroke-linecap="round" stroke-linejoin="round" d="M11.48 3.499a.562.562 0 0 1 1.04 0l2.125 5.111a.563.563 0 0 0 .475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 0 0-.182.557l1.285 5.385a.562.562 0 0 1-.84.61l-4.725-2.885a.562.562 0 0 0-.586 0L6.982 20.54a.562.562 0 0 1-.84-.61l1.285-5.386a.562.562 0 0 0-.182-.557l-4.204-3.602a.562.562 0 0 1 .321-.988l5.518-.442a.563.563 0 0 0 .475-.345l2.125-5.11Z" /></svg>Star</a>
        <a class="github-star-count" href="https://github.com/omidz4t/sigma" aria-label={`${repoStats.stars} GitHub stars`}>{repoStats.stars.toLocaleString('en-US')}</a>
      </div>
    </div>
  </header>
  <div class="docs-mobile-bar">
    <button type="button" onclick={() => menu.showModal()} aria-haspopup="dialog" aria-controls="docs-menu"><svg class="icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" d="M4 6h16M4 12h16M4 18h16" /></svg>{t("Commands")}</button>
    <span>{page.params.command || 'Overview'}</span>
  </div>
  <dialog id="docs-menu" class="docs-menu" bind:this={menu} aria-labelledby="docs-menu-title" onpointerdown={closeBackdrop}>
    <div class="docs-menu-header"><h2 id="docs-menu-title">{t("Commands")}</h2><button type="button" onclick={closeMenu} aria-label={t("Close navigation")}><svg class="icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" d="M6 6l12 12M6 18 18 6" /></svg></button></div>
    <div class="docs-menu-content">{@render commandNavigation('mobile')}</div>
  </dialog>
  <div class="docs-columns">
    <aside class="docs-sidebar">
      {@render commandNavigation('desktop')}
    </aside>
    <main class="docs-content">
      {#key page.params.command}<DocsActions />{/key}
      {@render children()}
    </main>
  </div>
</div>
