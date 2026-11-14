<script>
  import { asset } from '$app/paths';
  import { t } from '#lib/i18n.svelte.js';
  let { frame } = $props();
</script>

<img src={asset(`/assets/showcase/${frame.image}`)} width="720" height="1656" alt={t(frame.caption)} loading="lazy" />
{#if frame.pointer}
  <span class="showcase-pointer" style={`left:${frame.pointer[0]}%;top:${frame.pointer[1]}%`} aria-hidden="true">
    <svg viewBox="0 0 90 90" fill="none"><path d="M12 8C10 43 33 62 74 73M57 53l17 20-27 3" stroke="currentColor" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" /></svg>
    <span class="showcase-tap-ring"></span>
  </span>
{/if}
