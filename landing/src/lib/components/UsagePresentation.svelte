<script>
  import { onMount, tick } from 'svelte';
  import { fade } from 'svelte/transition';
  import { asset } from '$app/paths';
  import { resolve } from '#lib/i18n.svelte.js';
  import { t } from '#lib/i18n.svelte.js';
  import { agentCaptures } from '#lib/agent-captures.js';
  import { agentSlides } from '#lib/agent-showcase-data.js';
  import FloatingProviders from './FloatingProviders.svelte';
  import AgentShowcaseVisual from './AgentShowcaseVisual.svelte';
  import { terminalSteps } from '#lib/terminal-showcase-data.js';
  import ShowcaseScreen from './ShowcaseScreen.svelte';
  import TypedCommand from './TypedCommand.svelte';
  import DeltaChatIcon from './DeltaChatIcon.svelte';
  let { kind = 'terminal' } = $props();
  const frames = terminalSteps.flatMap((step, stepIndex) => step.frames.map((frame, subIndex) => ({ ...frame, title: step.title, stepIndex, subIndex })));
  let mobileAgents = $state(false);
  const displayedAgentSlides = $derived(agentSlides.flatMap((slide, sourceIndex) =>
    mobileAgents && slide.visual
      ? [{ ...slide, visual: null, sourceIndex }, { visual: slide.visual, imageOnly: true, sourceIndex }]
      : [{ ...slide, sourceIndex }]));
  function hasImage(index) { return kind === 'terminal' ? index > 0 : index > 0 && index < 4; }
  const standardSlides = $derived(Array.from({ length: kind === 'terminal' ? frames.length + 2 : 5 }, (_, sourceIndex) => sourceIndex).flatMap(sourceIndex =>
    mobileAgents && hasImage(sourceIndex) ? [{ sourceIndex }, { sourceIndex, imageOnly: true }] : [{ sourceIndex }]));
  const total = $derived(kind === 'agents' ? displayedAgentSlides.length : standardSlides.length);
  const sourceActive = $derived(standardSlides[active]?.sourceIndex ?? 0);
  function textPosition(index) { return standardSlides.findIndex(slide => slide.sourceIndex === index); }
  let active = $state(0);
  const phoneFrame = $derived(kind === 'agents' ? null : kind === 'terminal'
    ? sourceActive > 0 ? frames[Math.max(0, sourceActive - 2)] : null
    : sourceActive > 0 && sourceActive < 4 ? {
      image: ['01-app.png', 'feedback-no.png', '02-revised-app.png'][sourceActive - 1],
      caption: ['Watch an agent build a webxdc, send it to your phone, and respond to your feedback.', 'Ask for a change. Get the updated webxdc in Delta Chat.', 'The revised app after the phone requested a light theme and a live launch countdown.'][sourceActive - 1]
    } : null);
  const mobileImageOnly = $derived(kind === 'agents' ? displayedAgentSlides[active]?.imageOnly : standardSlides[active]?.imageOnly);
  const mobileAgentVisual = $derived(mobileAgents && kind === 'agents' ? agentSlides[displayedAgentSlides[active]?.sourceIndex]?.visual : null);
  function previewBackground(sourceIndex) {
    if (!mobileAgents) return undefined;
    const image = kind === 'agents'
      ? agentCaptures[agentSlides[sourceIndex]?.visual]?.[0]
      : kind === 'terminal'
        ? sourceIndex > 0 ? frames[Math.max(0, sourceIndex - 2)]?.image : null
        : ['01-app.png', 'feedback-no.png', '02-revised-app.png'][sourceIndex - 1];
    if (!image) return undefined;
    const directory = kind === 'agents' ? 'agent-demo' : kind === 'terminal' ? 'showcase' : 'ai-demo';
    return `url("${asset(`/assets/${directory}/${image}`)}")`;
  }
  let reducedMotion = $state(false);
  let track;
  let presentation;
  let escaped = false;
  let lastAdvance = -Infinity;
  let height = $state(0);
  function measure() {
    if (!track || !presentation) return;
    const navHeight = presentation.querySelector('.presentation-nav').getBoundingClientRect().height;
    presentation.style.setProperty('--deck-nav-height', `${navHeight}px`);
    height = Math.max(1, innerHeight - navHeight);
    presentation.style.setProperty('--deck-height', `${height}px`);
  }
  let transitionId = 0;
  let slideAnimations = [];
  function cancelSlideTransition() {
    transitionId += 1;
    for (const animation of slideAnimations) animation.cancel();
    slideAnimations = [];
  }
  async function go(index) {
    if (index < 0 || index >= total) return;
    cancelSlideTransition();
    const requestId = transitionId;
    if (index !== active) lastAdvance = performance.now();
    const slides = kind === 'agents' ? displayedAgentSlides : standardSlides;
    const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const slideBetweenPairs = mobileAgents && !reduce && slides[index]?.sourceIndex !== slides[active]?.sourceIndex;
    const direction = index > active ? 1 : -1;
    const layers = () => [track, ...presentation.querySelectorAll('.presentation-shared-phone, .mobile-agent-backdrop')];
    async function slide(from, to) {
      const animations = layers().map(layer => {
        // Individual translate preserves the screenshot's centering transform.
        return layer.animate([
          { translate: `${from}% 0` },
          { translate: `${to}% 0` }
        ], { duration: 180, easing: 'ease-in-out', fill: 'none' });
      });
      slideAnimations = animations;
      await Promise.all(animations.map(animation => animation.finished.catch(() => {})));
    }
    if (slideBetweenPairs) {
      await slide(0, -direction * 100);
      if (requestId !== transitionId) return;
    }
    const delta = track.children[index].getBoundingClientRect().left - track.getBoundingClientRect().left;
    track.scrollBy({ left: delta, behavior: reduce || slideBetweenPairs ? 'instant' : 'smooth' });
    if (slideBetweenPairs) {
      active = index;
      await tick();
      if (requestId !== transitionId) return;
      await slide(direction * 100, 0);
      if (requestId === transitionId) slideAnimations = [];
    }
  }
  function nextSection() {
    cancelSlideTransition();
    escaped = true;
    const chapter = presentation.closest('.usage-chapter');
    const next = chapter.nextElementSibling || document.querySelector('.delta-foundation');
    next?.scrollIntoView({ behavior: 'instant', block: 'start' });
  }
  function update() {
    const center = track.getBoundingClientRect().left + track.clientWidth / 2;
    const distances = [...track.children].map(slide => Math.abs(slide.getBoundingClientRect().left + slide.clientWidth / 2 - center));
    active = distances.indexOf(Math.min(...distances)); measure();
  }
  function keys(event) {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const rtl = getComputedStyle(track).direction === 'rtl';
    go(event.key === 'Home' ? 0 : event.key === 'End' ? total - 1 : active + ((event.key === 'ArrowRight') !== rtl ? 1 : -1));
  }
  onMount(() => {
    const mobileQuery = matchMedia('(max-width: 760px)');
    async function adaptAgentSlides() {
      const sourceIndex = kind === 'agents' ? displayedAgentSlides[active]?.sourceIndex ?? 0 : sourceActive;
      mobileAgents = mobileQuery.matches;
      await tick();
      active = kind === 'agents' ? displayedAgentSlides.findIndex(slide => slide.sourceIndex === sourceIndex) : textPosition(sourceIndex);
      const delta = track.children[active].getBoundingClientRect().left - track.getBoundingClientRect().left;
      track.scrollBy({ left: delta, behavior: 'instant' });
      measure();
    }
    adaptAgentSlides();
    mobileQuery.addEventListener('change', adaptAgentSlides);
    reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const observer = new ResizeObserver(measure);
    for (const slide of track.children) observer.observe(slide);
    let accumulated = 0;
    let lastWheel = -Infinity;
    let gestureUsed = false;
    let gestureDirection = 0;
    let touchStart;
    let touchHandled = false;
    let previousTop = presentation.getBoundingClientRect().top;
    function enforceEntry() {
      const top = presentation.getBoundingClientRect().top;
      // Catch native touch inertia, keyboard scrolling and large wheel deltas too.
      if (!escaped && previousTop > 2 && top < -2 && active < total - 1) {
        window.scrollTo({ top: scrollY + top, behavior: 'instant' });
        previousTop = 0;
      } else previousTop = top;
    }
    function canCapture() {
      const rect = presentation.getBoundingClientRect();
      if (rect.bottom <= 0 || rect.top >= innerHeight) escaped = false;
      // Never pull a reader back into a chapter they have already left.
      return !escaped && Math.abs(rect.top) <= 2;
    }
    function atBoundary(direction) { return direction > 0 ? active === total - 1 : active === 0; }
    function advance(direction) {
      go(active + direction);
      const top = presentation.getBoundingClientRect().top + window.scrollY;
      window.scrollTo({ top, behavior: 'instant' });
    }
    function consumeContent(amount) {
      const current = track.children[active];
      const copy = current.querySelector('.presentation-copy');
      const slide = copy && copy.scrollHeight > copy.clientHeight + 1 ? copy : current;
      const maximum = slide.scrollHeight - slide.clientHeight;
      if ((amount > 0 && slide.scrollTop < maximum - 1) || (amount < 0 && slide.scrollTop > 1)) {
        slide.scrollTop = Math.max(0, Math.min(maximum, slide.scrollTop + amount));
        return true;
      }
      return false;
    }
    function wheel(event) {
      if (event.defaultPrevented || event.ctrlKey || Math.abs(event.deltaX) > Math.abs(event.deltaY) || !event.deltaY) return;
      if (event.target.closest?.('textarea, input, select, [contenteditable=true], dialog, [role=dialog]')) return;
      const now = performance.now();
      const direction = Math.sign(event.deltaY);
      const amount = event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? innerHeight : 1);
      // Trackpad momentum belongs to the same gesture, even after an animation ends.
      if (now - lastWheel > 180 || direction !== gestureDirection) {
        accumulated = 0;
        gestureUsed = false;
      }
      lastWheel = now;
      gestureDirection = direction;
      const rect = presentation.getBoundingClientRect();
      const entering = direction > 0 ? rect.top > 2 && rect.top <= amount : rect.top < -2 && rect.top >= amount;
      if (!escaped && entering && Math.abs(rect.top) < innerHeight) {
        event.preventDefault();
        window.scrollTo({ top: rect.top + scrollY, behavior: 'instant' });
        accumulated = 0;
        return;
      }
      if (!canCapture()) { accumulated = 0; return; }
      if (gestureUsed || now - lastAdvance < 450) { event.preventDefault(); return; }
      if (consumeContent(amount)) { event.preventDefault(); return; }
      if (atBoundary(direction)) { accumulated = 0; return; }
      event.preventDefault();
      accumulated += amount;
      if (Math.abs(accumulated) >= 60) {
        accumulated = 0;
        gestureUsed = true;
        advance(direction);
      }
    }
    function touchBegin(event) {
      const owned = presentation.contains(event.target) && !event.target.closest?.('dialog, [role=dialog], .provider-orb');
      touchStart = owned && event.touches.length === 1
        ? { x: event.touches[0].clientX, y: event.touches[0].clientY, axis: null } : null;
      touchHandled = false;
      if (touchStart) {
        cancelSlideTransition();
        lastAdvance = -Infinity;
      }
    }
    function touchMove(event) {
      if (event.defaultPrevented || !touchStart || event.touches.length !== 1) return;
      const dx = event.touches[0].clientX - touchStart.x;
      const dy = event.touches[0].clientY - touchStart.y;
      if (!touchStart.axis) {
        if (Math.max(Math.abs(dx), Math.abs(dy)) < 10) return;
        touchStart.axis = Math.abs(dx) > Math.abs(dy) ? 'x' : 'y';
      }
      if (touchHandled) { event.preventDefault(); return; }
      const vertical = touchStart.axis === 'y';
      const distance = vertical ? dy : dx;
      const rtl = getComputedStyle(track).direction === 'rtl';
      // Finger down advances; finger up returns. Horizontal follows reading direction.
      const direction = Math.sign(distance) * (vertical || rtl ? 1 : -1);
      if (vertical && consumeContent(-dy)) {
        event.preventDefault();
        touchStart.y = event.touches[0].clientY;
        return;
      }
      if (atBoundary(direction)) return;
      event.preventDefault();
      if (Math.abs(distance) >= 36) {
        touchHandled = true;
        go(active + direction);
      }
    }
    function touchEnd() { touchStart = null; touchHandled = false; }
    function pageKeys(event) {
      if (event.key === 'Escape') { escaped = true; return; }
      if (event.target.closest?.('button, a, input, textarea, select, [contenteditable=true]')) return;
      const direction = event.key === 'ArrowDown' || event.key === 'PageDown' || (event.key === ' ' && !event.shiftKey) ? 1 : event.key === 'ArrowUp' || event.key === 'PageUp' || (event.key === ' ' && event.shiftKey) ? -1 : 0;
      if (!direction || !canCapture()) return;
      if (consumeContent(direction * height * .8)) { event.preventDefault(); return; }
      if (atBoundary(direction)) return;
      event.preventDefault();
      if (performance.now() - lastAdvance >= 700) advance(direction);
    }
    measure();
    window.addEventListener('scroll', enforceEntry, { passive: true });
    window.addEventListener('resize', measure);
    window.addEventListener('wheel', wheel, { passive: false });
    window.addEventListener('touchstart', touchBegin, { passive: true });
    window.addEventListener('touchmove', touchMove, { passive: false });
    window.addEventListener('touchend', touchEnd);
    window.addEventListener('touchcancel', touchEnd);
    window.addEventListener('keydown', pageKeys);
    return () => {
      cancelSlideTransition();
      mobileQuery.removeEventListener('change', adaptAgentSlides);
      observer.disconnect();
      window.removeEventListener('scroll', enforceEntry);
      window.removeEventListener('resize', measure);
      window.removeEventListener('wheel', wheel);
      window.removeEventListener('touchstart', touchBegin);
      window.removeEventListener('touchmove', touchMove);
      window.removeEventListener('touchend', touchEnd);
      window.removeEventListener('touchcancel', touchEnd);
      window.removeEventListener('keydown', pageKeys);
    };
  });
</script>

<div class="usage-presentation" class:has-shared-phone={!!phoneFrame && !mobileAgents} class:mobile-layered={mobileAgents} class:preview-overlay={mobileAgents && !mobileImageOnly} bind:this={presentation} role="region" aria-label={t(kind === 'terminal' ? 'Terminal' : kind === 'agents' ? 'Agents & calls' : 'AI agent')}>
  <nav class="presentation-nav" aria-label={t('Usage slides')}>
    <div class="presentation-navigation">
      <span class="presentation-count" aria-live="polite">{String(active + 1).padStart(2, '0')} / {total}</span>
      <button type="button" disabled={active === 0} onclick={() => go(active - 1)} onkeydown={keys} aria-label={t('Previous slide')}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="m15 18-6-6 6-6" /></svg></button>
      <button type="button" disabled={active === total - 1} onclick={() => go(active + 1)} onkeydown={keys} aria-label={t('Next slide')}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="m9 18 6-6-6-6" /></svg></button>
      <button type="button" class="presentation-next-section" onclick={nextSection} aria-label={t('Next section')} title={t('Next section')}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg></button>
    </div>
  </nav>
  <div class="presentation-track" bind:this={track} onscroll={update} style:height={height ? `${height}px` : undefined}>
    {#if kind === 'terminal'}
    <article class="presentation-slide presentation-title-slide" style:--slide-preview={previewBackground(0)} inert={active !== textPosition(0)} aria-hidden={active !== textPosition(0)}>
      <div class="presentation-copy">
        <p class="presentation-usecase-number">{t('Use case number 1')}</p>
        <h3>{t('From terminal to Delta Chat app')}</h3>
        <p>{t('Connect your terminal to Delta Chat. Send a message, share a file, or post an update to a chat, group, or channel.')}</p>
      </div>
    </article>
    {#if mobileAgents && hasImage(0)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(0) + 1} aria-hidden={active !== textPosition(0) + 1}></article>{/if}
    <article class="presentation-slide" style:--slide-preview={previewBackground(1)} inert={active !== textPosition(1)} aria-hidden={active !== textPosition(1)}>
      <div class="presentation-copy">
        <h3>{t('From your terminal to your phone.')}</h3>
        <p>{t('Connect your terminal to Delta Chat. Send a message, share a file, or post an update to a chat, group, or channel.')}</p>
        <p>{t('Send a report, a backup, or a finished build straight to your')} <a class="text-highlight" href="https://delta.chat/"><DeltaChatIcon /> {t('Delta Chat')}</a> {t('account. One command, and the file is in your chat.')}</p>
        <p class="presentation-invitation"><strong>{t('No complicated account setup.')}</strong><strong>{t('No API key needed.')}</strong></p>
      </div>
    </article>
    {#if mobileAgents && hasImage(1)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(1) + 1} aria-hidden={active !== textPosition(1) + 1}></article>{/if}
    {#each frames as frame, index}
      <article class="presentation-slide" style:--slide-preview={previewBackground(index + 2)} inert={active !== textPosition(index + 2)} aria-hidden={active !== textPosition(index + 2)}>
        <div class="presentation-copy">
          <h3>{t(frame.title)}</h3>
          <p>{t(frame.description)}</p>
          {#if frame.command}<pre class:command-highlight={active === textPosition(index + 2)}>{#if frame.command.includes('create-contact')}<TypedCommand text={frame.command} active={active === textPosition(index + 2)} />{:else}<code>{frame.command}</code>{/if}</pre>{/if}
        </div>
      </article>
    {#if mobileAgents && hasImage(index + 2)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(index + 2) + 1} aria-hidden={active !== textPosition(index + 2) + 1}></article>{/if}
    {/each}
    {:else if kind === 'agents'}
      {#each displayedAgentSlides as slide, index}
        <article class="presentation-slide agent-slide" style:--slide-preview={slide.imageOnly ? undefined : previewBackground(slide.sourceIndex)} class:agent-image-slide={slide.imageOnly} class:presentation-title-slide={index === 0} class:presentation-hooks={!slide.visual && index !== 0} inert={active !== index} aria-hidden={active !== index}>
          {#if index === 0}<FloatingProviders active={active === 0} />{/if}
          {#if !slide.imageOnly}<div class="presentation-copy">
            {#if index === 0}<p class="presentation-usecase-number">{t('Use case number 3')}</p>{/if}
            {#if index !== 0}<div class="agent-provider-logos">
              <a href="https://openai.com/codex/" aria-label="Codex"><img src={asset('/assets/brands/codex.svg')} alt="" /></a>
              <a href="https://grok.com/" aria-label="Grok"><img src={asset('/assets/brands/grok.svg')} alt="" /></a>
              <span class="agent-providers-soon">+ … ({t('soon')})</span>
            </div>{/if}
            <h3>{#if index === 0}<span>{t('Your agent.')}</span><span>{t('In your chat.')}</span><span>{t('On a call.')}</span>{:else}{t(slide.title)}{/if}</h3>
            <p>{t(slide.description)}</p>
            {#if index === 0}
              <button class="agent-call-preview" onclick={() => go(displayedAgentSlides.findIndex(item => item.sourceIndex === agentSlides.findIndex(source => source.visual === 'contact')))}>{t('See a real call')} <span aria-hidden="true">→</span></button>
            {/if}
            {#if slide.command}<pre dir="ltr"><code>{slide.command}</code></pre>{/if}
            {#if slide.note}<p class="presentation-note">{t(slide.note)}</p>{/if}
            {#if index === displayedAgentSlides.length - 1}<a class="use-case-link" href="https://github.com/omidz4t/sigma/blob/main/docs/agent-calls.md">{t('Read the agent call guide')} ↗</a>{/if}
          </div>
          {/if}
          {#if slide.visual && !mobileAgents}<AgentShowcaseVisual kind={slide.visual} />{/if}
        </article>
      {/each}
    {:else}
    <article class="presentation-slide presentation-title-slide" style:--slide-preview={previewBackground(0)} inert={active !== textPosition(0)} aria-hidden={active !== textPosition(0)}>
      <div class="presentation-copy">
        <p class="presentation-usecase-number">{t('Use case number 2')}</p>
        <h3>{t('Your AI agent, connected to Delta Chat')}</h3>
        <p>{t('Give your agent the Sigma skill. Ask for a change, then receive the result as a webxdc in your chat.')}</p>
      </div>
    </article>
    {#if mobileAgents && hasImage(0)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(0) + 1} aria-hidden={active !== textPosition(0) + 1}></article>{/if}
    <article class="presentation-slide presentation-resource" style:--slide-preview={previewBackground(1)} inert={active !== textPosition(1)} aria-hidden={active !== textPosition(1)}>
      <div class="presentation-copy">
        <h3>{t('Teach your AI the Sigma skill.')}</h3>
        <p>{t('Install the skill once so your AI agent knows how to use Sigma. It can connect contacts, send messages and files, and share webxdc apps.')}</p>
        <pre><code>npx skills add omidz4t/sigma --skill sigma</code></pre>
        <p>{t('To add someone, give your agent their {brand} invitation link and ask it to connect them.').split('{brand}')[0]}<DeltaChatIcon /> Delta Chat{t('To add someone, give your agent their {brand} invitation link and ask it to connect them.').split('{brand}')[1]}</p>
        <p>{t('For example: “Add Alex as a contact using this invitation link, then send Alex my report.”')}</p>
        <a class="use-case-link" href="https://skills.sh/omidz4t/sigma/sigma">{t('Get the Sigma skill on skills.sh')} ↗</a>
      </div>
    </article>
    {#if mobileAgents && hasImage(1)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(1) + 1} aria-hidden={active !== textPosition(1) + 1}></article>{/if}
    <article class="presentation-slide" style:--slide-preview={previewBackground(2)} inert={active !== textPosition(2)} aria-hidden={active !== textPosition(2)}>
      <div class="presentation-copy">
        <h3>{t('Ask for a change.')}</h3>
        <p>{t('Ask your agent to update your website and send you a webxdc preview using Sigma.')}</p>
        <pre><code>{t('Make the page use a light theme and add a live countdown. Send the updated preview to me in Delta Chat as a webxdc.')}</code></pre>
        <p>{t('You stay in Delta Chat. Send a request, open the result, and reply with a slash command when you want a revision or approval. Sigma connects that conversation to your agent.')}</p>
      </div>
    </article>
    {#if mobileAgents && hasImage(2)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(2) + 1} aria-hidden={active !== textPosition(2) + 1}></article>{/if}
    <article class="presentation-slide" style:--slide-preview={previewBackground(3)} inert={active !== textPosition(3)} aria-hidden={active !== textPosition(3)}>
      <div class="presentation-copy">
        <h3>{t('Open the result in your chat.')}</h3>
        <p>{t('Your agent packages the updated page as a webxdc and sends it with Sigma. Open it in Delta Chat, try it, and ask for the next change.')}</p>
      </div>
    </article>
    {#if mobileAgents && hasImage(3)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(3) + 1} aria-hidden={active !== textPosition(3) + 1}></article>{/if}
    <article class="presentation-slide presentation-hooks" style:--slide-preview={previewBackground(4)} inert={active !== textPosition(4)} aria-hidden={active !== textPosition(4)}>
      <div class="presentation-copy">
        <h3>{t('Hooks that turn clicks into actions.')}</h3>
        <p>{t('Share a webxdc app and connect its events to an executable hook. Build a shared to-do list, approval buttons, or a counter that remembers where you left off.')}</p>
        <pre><code><span># Try the included button-and-hook demo from the repository.</span>
python3 scripts/prepare-assets.py
./sigma send-webxdc INVITE_ID ./examples/hook-app.xdc --app demo
./sigma app-hook-add demo buttons ./examples/app-hook.py \
  --type update --match '&#123;"type":"button_clicked"&#125;'
./sigma app-start demo</code></pre>
        <p>{t('Click')} <strong>{t('Run hook via status update')}</strong> {t('in the shared app. The example hook records button events in the app’s local state directory and sends a durable update back. Custom hooks receive JSON on stdin and return actions on stdout. App identities, hook definitions, and captured events persist across restarts.')}</p>
        <a class="use-case-link" href={asset('/reference.html') + '#section-6'}>{t('Explore app hooks')} ↗</a>
        <p class="presentation-note">{t('These examples use an initialized sigma account. Replace')} <code>INVITE_ID</code> {t('with your full')} <DeltaChatIcon /> Delta Chat {t('invitation link. You can also use a known contact ID.')}</p>
      </div>
    </article>
    {#if mobileAgents && hasImage(4)}<article class="presentation-slide screenshot-slide" inert={active !== textPosition(4) + 1} aria-hidden={active !== textPosition(4) + 1}></article>{/if}
    {/if}
  </div>
  {#if mobileAgentVisual}
    <div class="mobile-agent-backdrop" aria-hidden={!mobileImageOnly}><AgentShowcaseVisual kind={mobileAgentVisual} /></div>
  {/if}
  {#if phoneFrame}
    <figure class="presentation-phone presentation-shared-phone" aria-hidden={mobileAgents && !mobileImageOnly}>
      <div class="presentation-screen">
        {#key phoneFrame.image + JSON.stringify(phoneFrame.pointer)}
          <div class="presentation-screen-layer" transition:fade={{ duration: reducedMotion ? 0 : 300 }}>
            {#if kind === 'terminal'}
              <ShowcaseScreen frame={phoneFrame} />
            {:else}
              <img src={asset(`/assets/ai-demo/${phoneFrame.image}`)} width="720" height="1656" alt={t(phoneFrame.caption)} />
            {/if}
          </div>
        {/key}
      </div>
      <figcaption>{t(phoneFrame.caption)}</figcaption>
    </figure>
  {/if}

</div>
