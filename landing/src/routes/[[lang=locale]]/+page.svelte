<script>
  import DeltaChatFoundation from '#lib/components/DeltaChatFoundation.svelte';
  import SiteFooter from '#lib/components/SiteFooter.svelte';
  import LanguageSwitcher from '#lib/components/LanguageSwitcher.svelte';
  import { t } from '#lib/i18n.svelte.js';
  import DeltaChatIcon from '#lib/components/DeltaChatIcon.svelte';
  import { asset } from '$app/paths';
  import { resolve } from '#lib/i18n.svelte.js';
  import repoStats from '#lib/repo-stats.json';
  import StarBubble from '#lib/components/StarBubble.svelte';
  import Logo from '#lib/components/Logo.svelte';
  import UseCases from '#lib/components/UseCases.svelte';
  import { SIGMA_AI_PROMPT, SIGMA_BINARY_URL, SIGMA_REFERENCE_URL } from '#lib/prompt-data.js';
  const releaseTag = new URL(SIGMA_BINARY_URL).pathname.split('/')[5];
  const releaseUrl = `https://github.com/omidz4t/sigma/releases/tag/${releaseTag}`;
  let audience = $state('human');
  function switchAudience(event) {
    const tabs = [...event.currentTarget.querySelectorAll('[role=tab]')];
    const index = tabs.indexOf(document.activeElement);
    const rtl = getComputedStyle(event.currentTarget).direction === 'rtl';
    let next;
    if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = tabs.length - 1;
    else if (event.key === 'ArrowRight') next = (index + (rtl ? -1 : 1) + tabs.length) % tabs.length;
    else if (event.key === 'ArrowLeft') next = (index + (rtl ? 1 : -1) + tabs.length) % tabs.length;
    else return;
    event.preventDefault();
    audience = tabs[next].dataset.audience;
    tabs[next].focus();
  }
  let dialog;
  let textarea;
  let prompt = $state(SIGMA_AI_PROMPT.replaceAll('{{BINARY_URL}}', SIGMA_BINARY_URL).replaceAll('{{REFERENCE_URL}}', SIGMA_REFERENCE_URL));
  let status = $state('Give your AI the context to build with sigma.');
  let modalStatus = $state('');
  let copied = $state(false);
  function openPrompt() { if (!dialog.open) dialog.showModal(); }
  function closeOnBackdrop(event) {
    if (event.target !== dialog) return;
    const box = dialog.getBoundingClientRect();
    if (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom) dialog.close();
  }
  async function copyPrompt() {
    try {
      if (!navigator.clipboard?.writeText) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(prompt);
      status = modalStatus = "Copied. Paste the prompt into your AI chat.";
      copied = true;
    } catch {
      openPrompt();
      textarea.focus();
      textarea.select();
      status = modalStatus = "Select and copy the prompt with Ctrl+C or ⌘C.";
    }
  }
</script>

<svelte:head>
  <title>{t("sigma — messaging from your terminal")}</title>
  <meta name="description" content={t("A stateful Rust CLI for messaging bots and webxdc apps, powered by chatmail core.")} />
  <link rel="icon" type="image/svg+xml" href={asset('/logo.svg')} />
</svelte:head>

<!-- Heroicons (https://heroicons.com), used under the following license:
MIT License

Copyright (c) Tailwind Labs, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
-->
  <div class="landing-language"><LanguageSwitcher /></div>
  <main id="top">
    <Logo />
    <p class="intro">{t("A stateful Rust CLI built on {core}. Send messages, run chat commands, and share webxdc apps. Your account stays on your machine. One binary, batteries included.").split('{core}')[0]}<span class="core-credit"><a href="https://delta.chat/" class="delta-chat-credit" aria-label="Delta Chat"><DeltaChatIcon /></a> <bdi>{t('chatmail core')}</bdi></span>{t("A stateful Rust CLI built on {core}. Send messages, run chat commands, and share webxdc apps. Your account stays on your machine. One binary, batteries included.").split('{core}')[1]}</p>
    <p class="latest-release"><a href={releaseUrl}>sigma {releaseTag} <svg class="icon" focusable="false" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m4.5 19.5 15-15m0 0H8.25m11.25 0v11.25" /></svg></a></p>
    <div class="quickstart-tabs" role="tablist" tabindex="-1" aria-label={t('Get started')} onkeydown={switchAudience}>
      <button type="button" role="tab" id="human-tab" data-audience="human" aria-selected={audience === 'human'} aria-controls="human-quickstart" tabindex={audience === 'human' ? 0 : -1} onclick={() => audience = 'human'}>{t('For humans')}</button>
      <button type="button" role="tab" id="ai-tab" data-audience="ai" aria-selected={audience === 'ai'} aria-controls="ai-quickstart" tabindex={audience === 'ai' ? 0 : -1} onclick={() => audience = 'ai'}>{t('For AI')}</button>
      <button type="button" role="tab" id="agents-tab" data-audience="agents" aria-selected={audience === 'agents'} aria-controls="agents-quickstart" tabindex={audience === 'agents' ? 0 : -1} onclick={() => audience = 'agents'}>{t('For Agents')}</button>
    </div>
    <div class="quickstart-panel" id="human-quickstart" role="tabpanel" aria-labelledby="human-tab" hidden={audience !== 'human'} tabindex="0">
    <pre aria-label={t("Example commands")}><code><span># Download for Linux x86_64. Create a bot. Start a conversation.</span>
<span>curl -fL https://github.com/omidz4t/sigma/releases/latest/download/sigma-linux-x86_64 -o sigma
chmod +x sigma</span>
./sigma init
./sigma create-contact me "Delta Chat invite link"
./sigma send-message me "Hello!"</code></pre>
    </div>
    <div class="quickstart-panel ai-quickstart" id="ai-quickstart" role="tabpanel" aria-labelledby="ai-tab" hidden={audience !== 'ai'} tabindex="0">
      <p>{t('Install the skill, then paste the AI prompt into your agent.')}</p>
      <pre><code>npx skills add omidz4t/sigma --skill sigma</code></pre>
    </div>
    <div class="quickstart-panel ai-quickstart" id="agents-quickstart" role="tabpanel" aria-labelledby="agents-tab" hidden={audience !== 'agents'} tabindex="0">
      <p>{t('Connect an installed, signed-in Codex or Grok CLI to your Delta Chat. Run either agent command below.')}</p>
      <pre dir="ltr"><code>./sigma init
./sigma create-contact me "Delta Chat invite link"
./sigma agent-connect codex --to me --alias assistant
./sigma agent-connect grok --to me --alias researcher</code></pre>
      <p>{t('The first recipient becomes the bot master. Your agent keeps the conversation across messages. Add --model to choose a model.')}</p>
      <a class="use-case-link" href={resolve('/docs/agent-connect')}>{t('Read the agent setup guide')} ↗</a>
    </div>
    <div class="actions">
      <a class="button" href={resolve('/docs')}>{t("Read the reference")} <svg class="icon" focusable="false"  xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
  <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 19.5 15-15m0 0H8.25m11.25 0v11.25"/>
</svg></a>
      <button id="copy-prompt" type="button" onclick={copyPrompt}><span id="copy-label">{t(copied ? 'Copied' : 'Copy AI prompt')}</span> <svg class="icon" focusable="false" id="copy-icon" hidden={copied ? true : undefined} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
  <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 0 1-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 0 1 1.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 0 0-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 0 1-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 0 0-3.375-3.375h-1.5a1.125 1.125 0 0 1-1.125-1.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H9.75"/>
</svg><svg class="icon" focusable="false" id="copied-icon" hidden={!copied ? true : undefined} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
  <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5"/>
</svg></button>
      <button id="view-prompt" type="button" onclick={openPrompt} aria-haspopup="dialog" aria-controls="prompt-dialog">{t("View prompt")}</button>
    </div>
    <p id="copy-status" class="status" role="status" aria-live="polite">{t(status)}</p>
    <UseCases />
    <dialog id="prompt-dialog" bind:this={dialog} onpointerdown={closeOnBackdrop} class="prompt-dialog" aria-labelledby="prompt-dialog-title">
      <div class="dialog-header">
        <h2 id="prompt-dialog-title">{t("AI prompt")}</h2>
        <button id="close-prompt" onclick={() => dialog.close()} class="close-dialog" type="button" aria-label={t("Close prompt")}><svg class="icon" focusable="false" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12"/>
</svg></button>
      </div>
      <label for="ai-prompt">{t("Copy or customize these instructions")}</label>
      <textarea id="ai-prompt" bind:this={textarea} bind:value={prompt} rows="12" spellcheck="false"></textarea>
      <div class="dialog-footer">
        <p id="modal-copy-status" class="status" role="status" aria-live="polite">{t(modalStatus)}</p>
        <span id="prompt-token-count" class="token-count" title={t("Rough estimate using 4 characters per token; actual counts vary by model.")}>≈ {Math.ceil(prompt.length / 4).toLocaleString()} {t("tokens")}</span>
        <button id="copy-prompt-modal" type="button" onclick={copyPrompt}><span id="modal-copy-label">{t(copied ? 'Copied' : 'Copy AI prompt')}</span><svg class="icon" focusable="false" id="modal-copy-icon" hidden={copied ? true : undefined} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
  <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 0 1-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 0 1 1.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 0 0-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 0 1-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 0 0-3.375-3.375h-1.5a1.125 1.125 0 0 1-1.125-1.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H9.75"/>
</svg><svg class="icon" focusable="false" id="modal-copied-icon" hidden={!copied ? true : undefined} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" data-slot="icon">
  <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 12.75 6 6 9-13.5"/>
</svg></button>
      </div>
    </dialog>
  </main>



<DeltaChatFoundation />

<section class="star-section" aria-labelledby="star-title">
  <div class="star-inner">
    {#if repoStats.stars < 200}
      <div class="star-sad-note">
        <StarBubble />
        <span class="sad-face-frames" role="img" aria-label={t("Sad face")}>
          <img class="sad-eyes-open" src={asset('/assets/stickers/sad-smiley.png')} width="100" height="100" alt="" />
          <img class="sad-eyes-closed" src={asset('/assets/stickers/sad-smiley-closed.png')} width="100" height="100" alt="" />
        </span>
      </div>
    {/if}
    <svg class="star-emblem" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true" focusable="false"><path stroke-linecap="round" stroke-linejoin="round" d="M11.48 3.499a.562.562 0 0 1 1.04 0l2.125 5.111a.563.563 0 0 0 .475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 0 0-.182.557l1.285 5.385a.562.562 0 0 1-.84.61l-4.725-2.885a.562.562 0 0 0-.586 0L6.982 20.54a.562.562 0 0 1-.84-.61l1.285-5.386a.562.562 0 0 0-.182-.557l-4.204-3.602a.562.562 0 0 1 .321-.988l5.518-.442a.563.563 0 0 0 .475-.345l2.125-5.11Z" /></svg>
    <h2 id="star-title">{t("A little star.")}<br />{t("A big help.")}</h2>
    <p>{t("If sigma is useful to you, give us a star on GitHub. It really helps people discover the project and encourages us to keep building.")}</p>
    <div class="github-star-wrap">
    <div class="github-star-widget" title={`Star count updated ${repoStats.updatedAt.slice(0, 10)} during the site build`}>
      <a class="github-star-action" href="https://github.com/omidz4t/sigma" aria-label={t("Star sigma on GitHub")}>
        <svg class="icon github-icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" focusable="false"><path d="M12 .75a11.25 11.25 0 0 0-3.558 21.923c.563.103.768-.244.768-.542 0-.267-.01-.975-.015-1.913-3.13.68-3.79-1.51-3.79-1.51-.512-1.3-1.25-1.646-1.25-1.646-1.022-.7.077-.686.077-.686 1.13.08 1.725 1.16 1.725 1.16 1.005 1.722 2.636 1.225 3.278.937.102-.728.393-1.225.715-1.507-2.499-.285-5.126-1.25-5.126-5.565 0-1.23.44-2.234 1.16-3.022-.116-.285-.503-1.43.11-2.98 0 0 .945-.302 3.094 1.154A10.78 10.78 0 0 1 12 6.175c.956.005 1.918.13 2.817.378 2.148-1.456 3.092-1.154 3.092-1.154.615 1.55.228 2.695.112 2.98.722.788 1.158 1.792 1.158 3.022 0 4.327-2.631 5.277-5.138 5.555.404.35.765 1.04.765 2.098 0 1.515-.014 2.737-.014 3.11 0 .3.203.65.774.54A11.252 11.252 0 0 0 12 .75Z" /></svg>
        Star
      </a>
      <a class="github-star-count" href="https://github.com/omidz4t/sigma" aria-label={`${repoStats.stars} GitHub stars`}>{repoStats.stars.toLocaleString('en-US')}</a>
    </div>
    </div>
  </div>
</section>

<section class="sigma-house" aria-labelledby="sigma-house-title">
  <div class="sigma-house-inner">
    <div class="sigma-house-art">
      <img class="sigma-house-people" src={asset('/assets/community/sigma-house-five.png')} alt="" width="1920" height="819" loading="lazy" />
      <a class="sigma-house-madmail" href="https://github.com/themadorg/madmail" aria-label="MADMAIL"><img src={asset('/assets/brands/madmail.png')} alt="MADMAIL" width="320" height="320" loading="lazy" /></a>
    </div>
    <h2 id="sigma-house-title">{t('Need a Sigma house?')}</h2>
    <p>{t('Give your Sigma a home with MADMAIL.')}</p>
    <a class="button sigma-house-link" href="https://github.com/themadorg/madmail">{t('Explore MADMAIL')} <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m4.5 19.5 15-15m0 0H8.25m11.25 0v11.25" /></svg></a>
  </div>
</section>

<section class="friendship-teaser" aria-labelledby="friendship-title">
  <div class="friendship-teaser-inner">
    <img src={asset('/assets/community/friendship-clean.png')} alt={t('Sigma, MADMAIL, and Delta Chat posing together as cartoon friends.')} width="1536" height="1024" loading="lazy" />
    <div>
      <h2 id="friendship-title">{t('Why this friendship lasts?!')}</h2>
      <p>{t('Find out how Sigma, MADMAIL, and Delta Chat work together—and why this friendship lasts.')}</p>
      <a class="button" href={resolve('/friendship')}>{t('Read the explainer')} <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="m4.5 19.5 15-15m0 0H8.25m11.25 0v11.25" /></svg></a>
    </div>
  </div>
</section>

<SiteFooter dockedLogo />
