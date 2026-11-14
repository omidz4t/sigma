<script>
  import { t } from '#lib/i18n.svelte.js';
  import { onMount } from 'svelte';
  import { asset } from '$app/paths';
  import { SIGMA_BINARY_URL } from '#lib/prompt-data.js';
  const releaseTag = new URL(SIGMA_BINARY_URL).pathname.split('/')[5];
  const releaseUrl = `https://github.com/omidz4t/sigma/releases/tag/${releaseTag}`;
  import { setupLogoMotion } from '#lib/logo/motion.js';
  import { setupLogoDock } from '#lib/logo/dock.js';
  let slot;
  onMount(() => {
    const stopMotion = setupLogoMotion(slot, asset);
    const stopDock = setupLogoDock(slot);
    return () => { stopMotion(); stopDock(); };
  });
</script>
    <div id="brand-slot" bind:this={slot} class="brand-slot">
    <div class="brand"><a class="brand-home" href="#top" aria-label={t("Sigma — back to top")}><span class="brand-logo" aria-hidden="true" data-angle="center"><img id="brand-logo-image" src={asset('/assets/logo/moai-center.png')} width="224" height="224" alt="" draggable="false" fetchpriority="high"></span><span class="brand-name"><span class="brand-owner">{t("omidz4t/")}</span>sigma</span></a><a class="brand-release" href={releaseUrl} aria-label={`Sigma ${releaseTag} release`}>{releaseTag}</a></div>
    </div>
