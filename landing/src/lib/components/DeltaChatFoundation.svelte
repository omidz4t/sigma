<script>
  import { asset } from '$app/paths';
  import { t } from '#lib/i18n.svelte.js';
  import { onMount } from 'svelte';
  let section;
  let visible = $state(false);
  let pageVisible = $state(true);
  onMount(() => {
    const observer = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; });
    observer.observe(section);
    const updateVisibility = () => { pageVisible = !document.hidden; };
    updateVisibility();
    document.addEventListener('visibilitychange', updateVisibility);
    return () => {
      observer.disconnect();
      document.removeEventListener('visibilitychange', updateVisibility);
    };
  });
  const topics = [
    { title: 'A trusted, modern messenger.', text: 'Your bot’s chat uses the same end-to-end encryption as your conversations with family and friends. Create separate chatmail profiles for yourself and your bots without providing a phone number or an existing email address. Independent relays keep you connected, even when you choose different servers.', href: 'https://delta.chat/en/help#technical-details', source: 'Delta Chat protocols' },
    { title: 'Iroh for live collaboration.', text: 'Realtime webxdc apps use Iroh to connect participants over encrypted peer-to-peer channels. Live events are ephemeral; apps can also send durable updates through chat so people who were offline can catch up.', href: 'https://delta.chat/en/2024-11-20-webxdc-realtime', source: 'How realtime works' },
    { title: 'A private preview, inside the chat.', text: 'Webxdc packages HTML, CSS, JavaScript, and assets into a mini app. Its sandbox blocks ordinary internet access, so a static preview can run locally without a public URL or tracking requests. Shared updates go through the host messenger’s APIs; server-side features need adaptation.', href: 'https://webxdc.org/', source: 'Explore webxdc' },
    { title: 'Small relays. Useful infrastructure.', text: 'Store-and-forward delivery lets people receive messages after reconnecting. Chatmail relays focus on encrypted delivery, while previews run on your device. MADMAIL brings message delivery and encryption enforcement into a single server binary you can host. Resource needs depend on traffic, attachment sizes, and retention.', href: 'https://github.com/themadorg/madmail', source: 'Explore MADMAIL' }
  ];
</script>
<section bind:this={section} class="delta-foundation" class:roaming={visible && pageVisible} aria-labelledby="delta-foundation-title">
  <div class="foundation-background" aria-hidden="true">
    <img class="roaming-logo delta-logo" src={asset('/assets/brands/delta-chat.svg')} width="88" height="88" alt="" loading="lazy" />
    <img class="roaming-logo madmail-logo" src={asset('/assets/brands/madmail.png')} width="88" height="88" alt="" loading="lazy" />
  </div>
  <div class="foundation-intro">
  <div class="foundation-heading">
    <h2 id="delta-foundation-title">{#each t('Built on Delta Chat. Open all the way down.').split(/(?<=\.)\s+/) as sentence}<span class="heading-line">{sentence}</span>{/each}</h2>
    <p>{t('Built on a resilient network of simple store-and-forward relays. Proven protocols carry your messages; end-to-end encryption keeps their contents private.')}</p>
  </div>
    <img class="foundation-art" src={asset("/assets/friendship/together.webp")} width="480" height="480" loading="lazy" alt={t("Sigma connects your terminal, your phone, and a server you can host yourself.")} />
  </div>
  <div class="foundation-topics">
    {#each topics as topic, index}
      <article>
        <span class="foundation-number" aria-hidden="true">0{index + 1}</span>
        <h3>{t(topic.title)}</h3>
        <p>{t(topic.text)}</p>
        <a href={topic.href}>{t(topic.source)} ↗</a>
      </article>
    {/each}
  </div>
</section>
<style>
  .delta-foundation{position:relative;isolation:isolate;overflow:hidden;max-width:1100px;margin:0 auto;padding:80px 32px}
  .foundation-intro{display:grid;grid-template-columns:1.4fr 1fr;align-items:center;gap:32px;margin-bottom:44px}
  .foundation-heading{max-width:720px}
  .foundation-art{display:block;width:100%;height:auto;object-fit:contain;border-radius:24px}
  .heading-line{display:block}
  h2{font-size:clamp(30px,4vw,46px);line-height:1.15;letter-spacing:-1.5px;font-weight:500;margin:16px 0}
  p{color:var(--muted);line-height:1.85}
  .foundation-topics{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}
  article{display:flex;flex-direction:column;min-width:0;border:1px solid var(--line);border-radius:20px;padding:28px;background:color-mix(in srgb,var(--paper) 94%,var(--ink) 6%)}
  article p{margin:0 0 24px}
  article a{align-self:flex-start;margin-top:auto}
  .foundation-number{font:12px ui-monospace,monospace;color:var(--muted)}
  h3{font-size:23px;line-height:1.3;font-weight:500;margin:16px 0}
  a{font-size:13px;text-decoration:underline;text-underline-offset:4px}
  @media(max-width:650px){.delta-foundation{padding:56px 24px}.foundation-topics{grid-template-columns:1fr;gap:18px}article{padding:24px}.foundation-intro{grid-template-columns:1fr;gap:20px}.foundation-art{max-width:360px;margin-inline:auto}}
  .foundation-background{position:absolute;inset:0;z-index:-1;pointer-events:none;user-select:none}
  .roaming-logo{position:absolute;object-fit:contain;opacity:.12;animation:foundation-drift 36s ease-in-out infinite alternate;animation-play-state:paused}
  .delta-logo{left:5%;top:12%}
  .madmail-logo{right:8%;top:62%;animation-direction:alternate-reverse;animation-duration:43s}
  .roaming .roaming-logo{animation-play-state:running}
  @keyframes foundation-drift{
    0%{transform:translate(0,0) rotate(-12deg)}
    35%{transform:translate(55px,75px) rotate(8deg)}
    70%{transform:translate(-10px,150px) rotate(-5deg)}
    100%{transform:translate(85px,210px) rotate(14deg)}
  }
  @media(prefers-reduced-motion:reduce){.roaming-logo{animation:none}}
</style>
