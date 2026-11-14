<script>
  import { getContext } from 'svelte';
  let { text, block, as = 'p', class: className = '' } = $props();
  const narration = getContext('friendship-narration');
  const tokens = $derived(text.match(/\S+\s*/gu) || []);
</script>
<svelte:element this={as} class={className} data-narration-block={block}>
  {#each tokens as token, index}<span class:spoken-word={narration.block === block && narration.word === index} style={`--word-index:${Math.min(index, 14)}`} data-read-word={`${block}:${index}`}>{token}</span>{/each}
</svelte:element>
