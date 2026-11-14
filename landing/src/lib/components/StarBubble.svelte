<script>
  import { t } from '#lib/i18n.svelte.js';
  import { onMount } from 'svelte';

  const messages = [
    'Too few stars…',
    'A star would help.',
    'Got a spare star?',
    'Small click, big smile.',
    'Rooting for 200!',
    'Help sigma shine.',
    'One more? Please?',
    'Made something cool?',
    'Stars keep us going.',
    'Thanks for being here.'
  ];
  let bubble;
  let phase = $state('hidden');
  let text = $state('');

  onMount(() => {
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
    let timer;
    let inView = false;
    let index = 0;
    let length = 0;

    function tick() {
      if (reduced.matches) return;
      if (!inView || document.hidden) {
        timer = setTimeout(tick, 500);
        return;
      }
      let delay;
      if (phase === 'hidden') {
        phase = 'thinking';
        text = '';
        delay = 1100;
      } else if (phase === 'thinking') {
        phase = 'typing';
        length = 0;
        delay = 75;
      } else if (phase === 'typing') {
        text = t(messages[index]).slice(0, ++length);
        if (length === t(messages[index]).length) {
          phase = 'shown';
          delay = 3000;
        } else {
          delay = 65;
        }
      } else {
        phase = 'hidden';
        index = (index + 1) % messages.length;
        delay = 2400;
      }
      timer = setTimeout(tick, delay);
    }

    function reset() {
      clearTimeout(timer);
      phase = reduced.matches ? 'shown' : 'hidden';
      text = reduced.matches ? t(messages[0]) : '';
      timer = setTimeout(tick, 1400);
    }
    const observer = new IntersectionObserver(([entry]) => { inView = entry.isIntersecting; });
    observer.observe(bubble.closest('.star-section'));
    reduced.addEventListener('change', reset);
    reset();
    return () => {
      clearTimeout(timer);
      observer.disconnect();
      reduced.removeEventListener('change', reset);
    };
  });
</script>

<span bind:this={bubble} class="star-speech" class:bubble-hidden={phase === 'hidden'} aria-hidden="true">
  {#if phase === 'thinking'}
    <span class="bubble-dots"><i></i><i></i><i></i></span>
  {:else}
    {text}<span class="bubble-caret" class:active={phase === 'typing'}></span>
  {/if}
</span>
