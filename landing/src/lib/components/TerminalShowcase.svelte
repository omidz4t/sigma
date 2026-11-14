<script>
  import { t } from '#lib/i18n.svelte.js';
  import { onMount } from 'svelte';
  import { asset } from '$app/paths';
  import ShowcaseScreen from './ShowcaseScreen.svelte';

  const initial = { image: '01-initial.png', caption: 'Delta Chat before connecting the new Sigma bot. The QR button is at the top right.' };
  const steps = [
    {
      title: 'Create the bot account.',
      frames: [{ ...initial, description: 'This creates a separate bot account on your machine. It does not connect the bot to you yet. Keep Delta Chat open on your phone.', command: './sigma init' }]
    },
    {
      title: 'Connect with your phone.',
      frames: [
        { ...initial, pointer: [82.2, 7.2], description: 'In Delta Chat, tap the QR button at the top right. Open QR Invite Code to find your personal invitation.' },
        { image: '02-qr.png', pointer: [34, 73.5], caption: 'The actual QR Invite Code screen. Tap LINK below the QR code.', description: 'The actual QR Invite Code screen. Tap LINK below the QR code.' },
        { image: '03-copy-link.png', pointer: [67, 79.8], caption: 'Tap COPY TO CLIPBOARD, then use the copied Delta Chat link in your terminal.', description: 'Tap LINK, then COPY TO CLIPBOARD. Paste that invitation into the terminal command in the next step. init alone never links the accounts.', command: './sigma create-contact me "Delta Chat invite link"' },
        { image: '04-contact.png', caption: 'Secure join completed by create-contact. Sigma now appears as a bot in Delta Chat.', description: 'Replace the quoted placeholder with the link you copied. create-contact completes secure join and saves the alias me. This is the step that connects you to the bot.', command: './sigma create-contact me "Delta Chat invite link"' }
      ]
    },
    {
      title: 'Send a message and a file.',
      frames: [
        { image: '05-message.png', caption: 'The real message, sent by Sigma and received in the Android app.', description: 'Send to the saved alias. Your message arrives in the encrypted conversation with Sigma.', command: './sigma send-message me "Hello from the terminal! This message was sent with Sigma."' },
        { image: '06-file.png', caption: 'The report.txt attachment delivered by send-file.', description: 'Send a local report, backup, or build artifact to the same conversation.', command: './sigma send-file me ./report.txt' }
      ]
    }
  ];
  const frames = steps.flatMap((step, stepIndex) => step.frames.map((frame, subIndex) => ({ ...frame, stepIndex, subIndex })));
  let container;
  let active = $state(0);
  onMount(() => {
    for (const frame of frames) {
      const image = new Image();
      image.src = asset(`/assets/showcase/${frame.image}`);
    }
    const panels = [...container.querySelectorAll('[data-frame]')];
    let scheduled = 0;
    function updateActive() {
      scheduled = 0;
      const title = container.querySelector('.showcase-sticky-title');
      const topEdge = title.getBoundingClientRect().bottom;
      const mobile = window.matchMedia('(max-width: 760px)').matches;
      let next = 0;
      // Keep the result until its whole command has passed the sticky heading.
      // On mobile the inline phone capture is part of the same panel.
      for (let index = 0; index < panels.length - 1; index += 1) {
        const panel = panels[index];
        const box = mobile
          ? panel.querySelector('.showcase-mobile-screen')
          : panel.querySelector('pre') || panel.querySelector('p');
        if (box.getBoundingClientRect().bottom > topEdge) break;
        next = index + 1;
      }
      active = next;
    }
    function scheduleUpdate() {
      if (!scheduled) scheduled = requestAnimationFrame(updateActive);
    }
    window.addEventListener('scroll', scheduleUpdate, { passive: true });
    window.addEventListener('resize', scheduleUpdate);
    scheduleUpdate();
    return () => {
      window.removeEventListener('scroll', scheduleUpdate);
      window.removeEventListener('resize', scheduleUpdate);
      if (scheduled) cancelAnimationFrame(scheduled);
    };
  });
  function jump(stepIndex, subIndex = 0) {
    active = frames.findIndex(frame => frame.stepIndex === stepIndex && frame.subIndex === subIndex);
    container.querySelector(`[data-frame="${active}"]`).scrollIntoView({
      block: 'center',
      behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth'
    });
  }
</script>

<div class="terminal-showcase" bind:this={container}>
  <div class="showcase-heading">
    <h4>{t('From a command to your conversation.')}</h4>
    <p>{t('Real screenshots from Delta Chat on Android. Scroll to follow along.')}</p>
  </div>
  <div class="showcase-grid">
    <div class="showcase-steps">
      <header class="showcase-sticky-title" aria-live="polite" aria-atomic="true">
        {#key frames[active].stepIndex}
          <div class="showcase-title-change">
            <span class="showcase-step-number">{String(frames[active].stepIndex + 1).padStart(2, '0')} / 03</span>
            <h5>{t(steps[frames[active].stepIndex].title)}</h5>
          </div>
        {/key}
      </header>
      {#each steps as step, stepIndex}
        <section class="showcase-stage" data-step={stepIndex} aria-label={t(step.title)}>
          {#each step.frames as frame, subIndex}
            {@const index = frames.findIndex(item => item.stepIndex === stepIndex && item.subIndex === subIndex)}
            <div class="showcase-step" class:step-active={active === index} data-frame={index}>
              {#if step.frames.length > 1}
                <nav class="showcase-substeps" aria-label={t(step.title)}>
                  {#each step.frames as _, i}
                    <button type="button" aria-label={`${stepIndex + 1}.${i + 1}`} aria-current={i === subIndex ? 'step' : undefined} onclick={() => jump(stepIndex, i)}>{stepIndex + 1}.{i + 1}</button>
                  {/each}
                </nav>
              {/if}
              <p>{t(frame.description)}</p>
              {#if frame.command}<pre class:command-highlight={active === index}><code>{frame.command}</code></pre>{/if}
              <div class="showcase-mobile-screen"><ShowcaseScreen {frame} /></div>
            </div>
          {/each}
        </section>
      {/each}
    </div>
    <figure class="showcase-phone">
      <div class="showcase-screen">
        {#key active}<ShowcaseScreen frame={frames[active]} />{/key}
      </div>
      <figcaption>{t(frames[active].caption)}</figcaption>
    </figure>
  </div>
  <p class="showcase-footnote">{t('Real captures using separate demo accounts. The illustrated demo invitation was revoked after recording. Use your own link; keep it private.')}</p>
</div>
