<script>
  import { asset } from '$app/paths';
  import { t } from '#lib/i18n.svelte.js';
  let { kind } = $props();
  import { agentCaptures as captures } from '#lib/agent-captures.js';
  const capture = $derived(captures[kind]);
</script>
<aside class="agent-visual" aria-label={t('Agent conversation')}>
  {#if capture}
    <figure class="agent-phone-capture">
      <img src={asset(`/assets/agent-demo/${capture[0]}`)} width="720" height="1656" alt={t(capture[1])} />
      <figcaption>{t(kind === 'profiles' ? 'Real Delta Chat capture. “Me” is the local Bot Master account.' : 'Captured in Delta Chat. Separate demo accounts; real agent responses.')}</figcaption>
    </figure>
  {/if}
</aside>
