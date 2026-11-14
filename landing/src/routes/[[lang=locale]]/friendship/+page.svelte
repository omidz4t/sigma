<script>
  import SiteFooter from '#lib/components/SiteFooter.svelte';
  import { onMount, setContext } from 'svelte';
  import { asset } from '$app/paths';
  import { resolve } from '#lib/i18n.svelte.js';
  import { t } from '#lib/i18n.svelte.js';
  import { roles, reasons, security, intro } from '#lib/friendship-data.js';
  import LanguageSwitcher from '#lib/components/LanguageSwitcher.svelte';
  import DeltaChatIcon from '#lib/components/DeltaChatIcon.svelte';
  import NarratedText from '#lib/components/NarratedText.svelte';
  import StoryNarration from '#lib/components/StoryNarration.svelte';
  const narration = $state({ block: '', word: -1 });
  setContext('friendship-narration', narration);
  let story;
  const apps = [
    { name: 'Android', icon: 'googleplay', href: 'https://delta.chat/en/download#android' },
    { name: 'iPhone / iPad', icon: 'appstore', href: 'https://delta.chat/en/download#ios' },
    { name: 'macOS', icon: 'apple', href: 'https://delta.chat/en/download#macos' },
    { name: 'Windows', icon: 'microsoftstore', href: 'https://delta.chat/en/download#windows' },
    { name: 'Linux', icon: 'linux', href: 'https://delta.chat/en/download#linux' },
    { name: 'Ubuntu Touch', icon: 'ubuntu', href: 'https://delta.chat/en/download#ubuntutouch' }
  ];
  onMount(() => {
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    const elements = story.querySelectorAll('[data-reveal]');
    story.classList.add('story-motion');
    const observer = new IntersectionObserver(entries => {
      for (const entry of entries) if (entry.isIntersecting) { entry.target.classList.add('story-visible'); observer.unobserve(entry.target); }
    }, { threshold: 0.08, rootMargin: '0px 0px -35px 0px' });
    for (const element of elements) observer.observe(element);
    return () => observer.disconnect();
  });
</script>

<svelte:head>
  <title>{t('Why this friendship lasts.')} — sigma</title>
  <meta name="description" content={t('Three different roles. One shared conversation.')} />
</svelte:head>
<div class="friendship-page friendship-story" bind:this={story}>
  <div class="friendship-story-header">
    <header class="friendship-nav">
      <a class="friendship-header-back" href={resolve('/')} aria-label={t('Back to Sigma')} title={t('Back to Sigma')}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" /></svg></a>
      <a class="docs-brand" href={resolve('/')}><img src={asset('/assets/logo/moai-right-far.png')} width="46" height="46" alt="" />sigma</a>
      <LanguageSwitcher />
    </header>
    <div class="friendship-hero" data-reveal>
      <img class="friendship-portrait" src={asset('/assets/community/friendship-clean.png')} width="1536" height="1024" alt={t('Sigma, MADMAIL, and Delta Chat posing together as cartoon friends.')} fetchpriority="high" />
      <NarratedText as="h1" block="title" text={t('Why this friendship lasts.')} />
      <NarratedText class="friendship-lede" block="lede" text={t('Your agent needs a way back to you.')} />
      <NarratedText class="friendship-intro" block="intro" text={t(intro)} />
      <div class="story-audio-anchor"><StoryNarration controller={narration} /></div>
    </div>
  </div>
  <main class="friendship-content friendship-story-content">
    <div class="friendship-roles">
      {#each roles as role, index}
        <article data-reveal>
          <a class="friendship-project" href={role.href}>
            {#if role.image}<img src={asset(role.image)} width="56" height="56" alt="" />{:else}<DeltaChatIcon />{/if}
            <NarratedText as="h2" block={`role-${index}-name`} text={role.name} />
          </a>
          <NarratedText as="h3" block={`role-${index}-title`} text={t(role.role)} />
          <NarratedText block={`role-${index}-body`} text={t(role.description)} />
          {#if role.name === 'Delta Chat'}
            <nav class="friendship-app-links" aria-label={t('Get Delta Chat on your devices')}>
              {#each apps as app}<a href={app.href}><img class="store-icon" src={asset(`/assets/stores/${app.icon}.svg`)} width="19" height="19" alt="" />{app.name}</a>{/each}
              <a href="https://chatmail.at/clients">{t('More compatible apps')} ↗</a>
            </nav>
          {/if}
        </article>
      {/each}
    </div>
    <aside class="friendship-security" data-reveal>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m9 12 2 2 4-4M12 3 3 6v6c0 5 9 9 9 9s9-4 9-9V6l-9-3Z" /></svg>
      <div><NarratedText block="delta-security" text={t(security)} /><a href="https://delta.chat/en/help#security">{t('Read about encryption and security')} ↗</a></div>
    </aside>
    <section class="friendship-reasons">
      {#each reasons as reason, index}
        <article class="friendship-chapter" data-reveal>
          <div class="friendship-chapter-art"><img src={asset(reason.image)} width="480" height="480" alt="" loading="lazy" /><span class="story-chapter-number" aria-hidden="true">{String(index + 1).padStart(2, '0')}</span></div>
          <div class="friendship-chapter-copy">
            <NarratedText as="h2" block={`${reason.id}-title`} text={t(reason.title)} />
            {#each reason.paragraphs as paragraph, p}<NarratedText block={`${reason.id}-p${p}`} text={t(paragraph)} />{/each}
          </div>
        </article>
      {/each}
    </section>
    <nav class="friendship-sources" aria-label={t('Further reading')} data-reveal>
      <a href="https://webxdc.org/">{t('Webxdc: private mini apps')}</a>
      <a href="https://delta.chat/en/privacy">{t('Delta Chat privacy and relay retention')}</a>
      <a href="https://github.com/themadorg/madmail#quick-setup">{t('MADMAIL setup')}</a>
      <a href={resolve('/docs/app-hook-add')}>{t('Sigma app hooks')}</a>
    </nav>
    <p class="friendship-note">{t('This portrait celebrates compatible tools and shared ideas. Sigma, MADMAIL, and Delta Chat are independent projects.')}</p>
    <a class="button" href={resolve('/docs')}>{t('Read the reference')}</a>
  </main>
</div>

<SiteFooter />
