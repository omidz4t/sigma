<script>
  import { asset } from '$app/paths';
  import { resolve } from '#lib/i18n.svelte.js';
  import { t } from '#lib/i18n.svelte.js';
  import DeltaChatIcon from './DeltaChatIcon.svelte';
  let { dockedLogo = false } = $props();
</script>

<footer class="site-footer" class:standalone={!dockedLogo}>
  <div class="footer-inner">
    <div class="footer-top">
      <a class="footer-name" href={dockedLogo ? "#top" : resolve("/")}>{#if !dockedLogo}<img src={asset("/assets/logo/moai-center.png")} width="88" height="88" alt="" />{/if}<span>sigma</span></a>
      <nav aria-label={t("Footer")}>
        <a href={resolve('/docs')}><svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true" focusable="false"><path stroke-linecap="round" stroke-linejoin="round" d="M12 6.042A8.967 8.967 0 0 0 6 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 0 1 6 18c2.305 0 4.408.867 6 2.292m0-14.25a8.967 8.967 0 0 1 6-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0 0 18 18a8.967 8.967 0 0 0-6 2.292m0-14.25v14.25" /></svg>{t("Documentation")}</a>
        <a href="https://github.com/omidz4t/sigma"><svg class="icon github-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" focusable="false"><path d="M12 .75a11.25 11.25 0 0 0-3.558 21.923c.563.103.768-.244.768-.542 0-.267-.01-.975-.015-1.913-3.13.68-3.79-1.51-3.79-1.51-.512-1.3-1.25-1.646-1.25-1.646-1.022-.7.077-.686.077-.686 1.13.08 1.725 1.16 1.725 1.16 1.005 1.722 2.636 1.225 3.278.937.102-.728.393-1.225.715-1.507-2.499-.285-5.126-1.25-5.126-5.565 0-1.23.44-2.234 1.16-3.022-.116-.285-.503-1.43.11-2.98 0 0 .945-.302 3.094 1.154A10.78 10.78 0 0 1 12 6.175c.956.005 1.918.13 2.817.378 2.148-1.456 3.092-1.154 3.092-1.154.615 1.55.228 2.695.112 2.98.722.788 1.158 1.792 1.158 3.022 0 4.327-2.631 5.277-5.138 5.555.404.35.765 1.04.765 2.098 0 1.515-.014 2.737-.014 3.11 0 .3.203.65.774.54A11.252 11.252 0 0 0 12 .75Z" /></svg>GitHub</a>
        <a href="https://github.com/omidz4t/sigma/releases"><svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true" focusable="false"><path stroke-linecap="round" stroke-linejoin="round" d="M9 6.75V15m0 0-3-3m3 3 3-3m1.5-9H5.625c-.621 0-1.125.504-1.125 1.125v16.5c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V8.25m-6-6L19.5 8.25m-6-6V7.125c0 .621.504 1.125 1.125 1.125H19.5" /></svg>{t("Releases")}</a>
      </nav>
    </div>
    <p class="footer-affiliation">{t("Sigma is an unofficial project and is not affiliated with {brand}.").split('{brand}')[0]}<DeltaChatIcon /> Delta Chat{t("Sigma is an unofficial project and is not affiliated with {brand}.").split('{brand}')[1]}</p>
    <div class="footer-bottom">
      <span>{t("sigma ·")} <a href="https://github.com/omidz4t/sigma/blob/main/LICENSE">{t("MIT licensed")}</a></span>
      <a href="https://github.com/chatmail/core">{t("Powered by")} <DeltaChatIcon /> {t("chatmail core")} <svg class="icon" focusable="false" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m4.5 19.5 15-15m0 0H8.25m11.25 0v11.25" /></svg></a>
    </div>
  </div>
</footer>

<style>
  .standalone .footer-name{visibility:visible;width:auto;height:auto;display:flex;flex-direction:column;align-items:center;gap:4px;line-height:1.2}
  .footer-name img{display:block;object-fit:contain}
</style>
