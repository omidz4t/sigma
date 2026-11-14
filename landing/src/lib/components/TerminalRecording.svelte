<script>
  import { onMount } from 'svelte';
  import { asset } from '$app/paths';
  import { t } from '#lib/i18n.svelte.js';
  import 'asciinema-player/dist/bundle/asciinema-player.css';
  let { source } = $props();
  let terminal;
  let failed = $state(false);
  onMount(() => {
    let player;
    let cancelled = false;
    import('asciinema-player').then(module => {
      if (cancelled) return;
      player = module.create(asset(source), terminal, {
        autoPlay: false, preload: true, fit: 'width', terminalFontSize: 14,
        idleTimeLimit: 2, speed: 1.25, theme: 'asciinema',
        terminalFontFamily: 'ui-monospace, SFMono-Regular, Consolas, monospace'
      });
    }).catch(() => { failed = true; });
    return () => { cancelled = true; player?.dispose(); };
  });
</script>
<div class="ai-terminal" bind:this={terminal}></div>
{#if failed}<p class="docs-note">{t('The recording could not load. You can download it below.')}</p>{/if}
<a class="ai-recording-download" href={asset(source)} download>{t('Download terminal recording')}</a>
