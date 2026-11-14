<script>
  import { t } from '#lib/i18n.svelte.js';
  import { onMount } from 'svelte';
  import { asset } from '$app/paths';
  let { active = false } = $props();
  let field;
  let bodies = $state([
    { name: 'Codex', icon: 'codex', angle: .4, speed: .19 },
    { name: 'Grok', icon: 'grok', angle: 3.54, speed: .19 }
  ]);
  let geometry = $state({ cx: 0, cy: 0, rx: 0, ry: 0 });
  let drag = null;
  let reduced = false;
  function position(angle) {
    return { x: geometry.cx + geometry.rx * Math.cos(angle), y: geometry.cy + geometry.ry * Math.sin(angle) };
  }
  function move(event) {
    if (!drag || drag.pointer !== event.pointerId) return;
    const rect = field.getBoundingClientRect();
    bodies[drag.index].angle = Math.atan2((event.clientY - rect.top - geometry.cy) / Math.max(1, geometry.ry), (event.clientX - rect.left - geometry.cx) / Math.max(1, geometry.rx));
  }
  function grab(event, index) {
    if (event.button !== 0) return;
    event.stopPropagation();
    drag = { index, pointer: event.pointerId };
    event.currentTarget.setPointerCapture(event.pointerId);
  }
  function release() { drag = null; }
  onMount(() => {
    const media = matchMedia('(prefers-reduced-motion: reduce)');
    const preference = () => { reduced = media.matches; };
    preference(); media.addEventListener('change', preference);
    const title = field.parentElement.querySelector('h3');
    function measure() {
      const rect = field.getBoundingClientRect();
      const text = title.getBoundingClientRect();
      geometry = {
        cx: rect.width / 2,
        cy: text.top - rect.top + text.height / 2,
        rx: Math.max(0, Math.min(rect.width / 2 - 32, text.width / 2 + 65)),
        ry: Math.max(0, Math.min(text.height / 2 + 60, text.top - rect.top + text.height / 2 - 30, rect.bottom - (text.top + text.height / 2) - 30))
      };
    }
    const observer = new ResizeObserver(measure);
    observer.observe(field); observer.observe(title);
    measure();
    let frame, previous;
    function animate(now) {
      const dt = previous ? Math.min((now - previous) / 1000, .05) : 0;
      previous = now;
      if (active && !reduced) for (const [i, body] of bodies.entries()) {
        const button = field.querySelectorAll('button')[i];
        if (drag?.index === i || button === document.activeElement || button.matches(':hover')) continue;
        body.angle += body.speed * dt;
      }
      frame = requestAnimationFrame(animate);
    }
    frame = requestAnimationFrame(animate);
    return () => { cancelAnimationFrame(frame); observer.disconnect(); media.removeEventListener('change', preference); };
  });
</script>
<div class="provider-field" bind:this={field}>
  {#each bodies as body, index}
    {@const point = position(body.angle)}
    <button class="provider-orb" type="button" style:left={`${point.x}px`} style:top={`${point.y}px`} aria-label={`${body.name}: ${t('Drag around the orbit, or press Enter to advance')}`} title={`${body.name} · ${t('Drag along the orbit')}`} onpointerdown={(event) => grab(event, index)} onpointermove={move} onpointerup={release} onpointercancel={release} onlostpointercapture={release} onclick={(event) => { if (event.detail === 0) body.angle += Math.PI / 4; }}>
      <img src={asset(`/assets/brands/${body.icon}.svg`)} alt="" draggable="false" />
      <span>{body.name}</span>
    </button>
  {/each}
</div>
<style>
  .provider-field{position:absolute;inset:0;pointer-events:none;z-index:0;overflow:hidden}
  .provider-orb{position:absolute;transform:translate(-50%,-50%);width:60px;height:64px;border:0;border-radius:0;background:transparent;color:var(--ink);display:flex;flex-direction:column;gap:6px;align-items:center;justify-content:center;box-shadow:none;pointer-events:auto;touch-action:none;cursor:grab;padding:6px;user-select:none;opacity:.75}
  .provider-orb:hover,.provider-orb:focus-visible{opacity:1;background:transparent;box-shadow:none}
  .provider-orb:focus-visible{outline:2px solid var(--ink);outline-offset:3px;border-radius:6px}
  .provider-orb:active{cursor:grabbing}
  img{width:34px;height:34px;pointer-events:none}
  @media(prefers-color-scheme:dark){img{filter:invert(1)}}
  span{font-size:11px;line-height:1;pointer-events:none}
</style>
