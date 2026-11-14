<script>
  import { page } from '$app/state';
  import { language, chooseLanguage } from '#lib/i18n.svelte.js';
  import '../app.css';
  let { children } = $props();
  chooseLanguage(page.params.lang || 'en', false);
  $effect.pre(() => { chooseLanguage(page.params.lang || 'en', false); });
  $effect(() => {
    document.documentElement.lang = language.current;
    document.documentElement.dir = language.current === 'fa' ? 'rtl' : 'ltr';
  });
</script>

{@render children()}
