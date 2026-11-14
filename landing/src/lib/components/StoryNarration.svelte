<script>
  import { onMount } from 'svelte';
  import { asset } from '$app/paths';
  import { language, t } from '#lib/i18n.svelte.js';
  let { controller } = $props();
  let audio;
  let cues = $state([]);
  let duration = $state(0);
  let time = $state(0);
  let playing = $state(false);
  let started = $state(false);
  let follow = $state(true);
  let rate = $state('1');
  let failed = $state(false);
  let ready = $state(false);
  let lastBlock = '';
  let lastScroll = 0;
  let reduced = false;
  const locale = $derived(language.current);
  const source = $derived(asset(`/assets/friendship/audio/${locale}.mp3`));
  function format(seconds) { return `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, '0')}`; }
  function stop() {
    audio?.pause(); if (audio) audio.currentTime = 0;
    window.scrollTo({ top: window.scrollY, behavior: 'instant' });
    playing = false; started = false; time = 0; controller.block = ''; controller.word = -1; lastBlock = '';
  }
  async function toggle() {
    if (!audio) return;
    if (!audio.paused) { audio.pause(); return; }
    try { await audio.play(); started = true; failed = false; }
    catch { failed = true; }
  }
  function update() {
    time = audio.currentTime;
    if (!started || audio.ended) { controller.block = ''; controller.word = -1; return; }
    let low = 0, high = cues.length;
    while (low < high) { const mid = (low + high) >> 1; if (cues[mid].start <= time) low = mid + 1; else high = mid; }
    const cue = cues[low - 1];
    if (!cue || time > cue.end + 0.3) { controller.word = -1; return; }
    controller.block = cue.block; controller.word = cue.word;
    if (playing && follow) {
      const word = document.querySelector(`[data-read-word="${cue.block}:${cue.word}"]`);
      if (!word) return;
      const rect = word.getBoundingClientRect();
      const changed = cue.block !== lastBlock;
      if ((changed || rect.top < 80 || rect.bottom > innerHeight * 0.65) && performance.now() - lastScroll > 700) {
        word.scrollIntoView({ behavior: reduced ? 'instant' : 'smooth', block: 'center' });
        lastScroll = performance.now(); lastBlock = cue.block;
      }
    }
  }
  onMount(() => { reduced = matchMedia('(prefers-reduced-motion: reduce)').matches; follow = !reduced; return stop; });
  $effect(() => {
    const selected = locale;
    stop(); cues = []; ready = false; failed = false;
    const abort = new AbortController();
    fetch(asset(`/assets/friendship/audio/${selected}.json`), { signal: abort.signal })
      .then(response => { if (!response.ok) throw new Error('Narration unavailable'); return response.json(); })
      .then(data => { cues = data.words; ready = true; })
      .catch(error => { if (error.name !== 'AbortError') failed = true; });
    return () => abort.abort();
  });
</script>
<div class="story-audio" class:story-audio-active={started} aria-label={t('Listen to this story')}>
  <div class="story-audio-controls">
    <button type="button" class="story-play" onclick={toggle} disabled={!ready || failed} aria-label={t(playing ? 'Pause narration' : 'Play narration')}>
      {#if playing}<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M6 4h4v16H6zm8 0h4v16h-4z" /></svg>{:else}<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="m8 4 12 8-12 8z" /></svg>{/if}
      <span>{t(playing ? 'Pause' : 'Listen')}</span>
    </button>
    <label class="story-follow"><input type="checkbox" bind:checked={follow} />{t('Auto-scroll')}</label>
    <label class="story-speed"><span class="sr-only">{t('Playback speed')}</span><select bind:value={rate} onchange={() => audio.playbackRate = Number(rate)}><option value="0.85">0.85×</option><option value="1">1×</option><option value="1.25">1.25×</option><option value="1.5">1.5×</option></select></label>
    {#if playing}
    <button type="button" class="story-stop" onclick={stop} aria-label={`${t('Stop')} · ${t('Close')}`} title={t('Stop')}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" /></svg></button>
    {/if}
  </div>
  <div class="story-audio-progress"><span>{format(time)}</span><input type="range" min="0" max={duration || 1} step="0.1" value={time} oninput={event => { audio.currentTime = Number(event.currentTarget.value); update(); }} aria-label={t('Narration position')} disabled={!ready} /><span>{format(duration)}</span></div>
  {#if failed}<p role="status">{t('Audio could not load. You can keep reading and try again later.')}</p>{/if}
  <audio bind:this={audio} src={source} preload="metadata" onloadedmetadata={() => { duration = audio.duration; audio.playbackRate = Number(rate); }} ontimeupdate={update} onplay={() => playing = true} onpause={() => playing = false} onended={() => { playing = false; controller.block = ''; controller.word = -1; }} onerror={() => failed = true}></audio>
</div>
