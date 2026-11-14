<script>
  let { text, active = false } = $props();
  let length = $state(0);
  $effect(() => {
    const command = text;
    if (!active) { length = 0; return; }
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) { length = command.length; return; }
    length = 0;
    const start = performance.now();
    const timer = setInterval(() => {
      length = Math.min(command.length, Math.floor((performance.now() - start) / 24));
      if (length === command.length) clearInterval(timer);
    }, 24);
    return () => clearInterval(timer);
  });
</script>
<code class="typed-command"><span class="typed-command-space">{text}</span><span class="typed-command-ink" aria-hidden="true">{text.slice(0, length)}{#if active && length < text.length}<span class="typing-cursor">▏</span>{/if}</span></code>
<style>
  .typed-command { display: grid; direction: ltr; text-align: left; }
  .typed-command-space, .typed-command-ink { grid-area: 1 / 1; white-space: pre-wrap; overflow-wrap: anywhere; }
  .typed-command-space { color: transparent; }
  .typed-command-ink { pointer-events: none; }
  .typing-cursor { color: #bdd394; }
</style>
